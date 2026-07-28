//! Measures beholder's heuristic reference graph against an external resolved
//! index.
//!
//! Beholder neither produces nor requires a SCIP index. This tool reads one that
//! already exists so the heuristic resolver can carry a measured error bar
//! instead of an assurance. Methodology is in `docs/resolver-accuracy.md`.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::PathBuf;

use anyhow::{Context, Result};
use beholder::graph;
use beholder::phase1::FileAnalysis;
use beholder::resolve::Heuristic;
use beholder::Config;
use clap::Parser;

/// Definition role bit in a SCIP occurrence's `symbol_roles`.
const DEFINITION: i32 = 0x1;

#[derive(Parser)]
#[command(name = "beholder-oracle", about, long_about = None)]
struct Cli {
    /// Repository to analyze.
    repository: PathBuf,
    /// SCIP index describing that repository.
    #[arg(long, default_value = "index.scip")]
    index: PathBuf,
    /// Revision to analyze. Defaults to the working tree.
    #[arg(long)]
    revision: Option<String>,
    /// Emit the full comparison as JSON instead of a summary.
    #[arg(long)]
    json: bool,
    /// How many example disagreements to show per class.
    #[arg(long, default_value_t = 3)]
    examples: usize,
}

/// One directed reference between two beholder symbols.
type Pair = (String, String);

fn main() -> Result<()> {
    let cli = Cli::parse();

    let config = Config::load(&cli.repository)?;
    let files = match &cli.revision {
        Some(revision) => {
            let repo = git2::Repository::discover(&cli.repository)?;
            beholder::walk::read_revision(&repo, revision, &config)?
        }
        None => beholder::walk::walk_worktree(&cli.repository, &config)?,
    };

    let analyzed: Vec<FileAnalysis> = files
        .iter()
        .map(|file| beholder::phase1::analyze(&file.path, &file.contents))
        .collect();

    let built = graph::build(&analyzed, &Heuristic);
    let ours: BTreeSet<Pair> = built
        .edges
        .iter()
        .map(|e| (e.from.clone(), e.to.clone()))
        .collect();

    let symbols = SymbolTable::build(&analyzed);
    let oracle = read_oracle(&cli.index, &symbols)?;

    let report = Report::compare(&ours, &oracle.edges, &symbols, &oracle);

    if cli.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        report.print(&cli.repository, cli.examples);
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Beholder's view
// ---------------------------------------------------------------------------

/// Beholder symbols, indexed by where they live so a position can be resolved
/// back to the symbol that contains it.
struct SymbolTable {
    /// Repo-relative path to that file's symbols, innermost last.
    by_file: HashMap<String, Vec<Located>>,
    /// Symbol id to its qualified path, for classifying disagreements.
    qualified: HashMap<String, String>,
    /// How many symbols in the project share each simple name.
    name_counts: HashMap<String, usize>,
    /// Simple name of each symbol id.
    name_of: HashMap<String, String>,
}

struct Located {
    id: String,
    start_line: usize,
    end_line: usize,
}

impl SymbolTable {
    fn build(files: &[FileAnalysis]) -> Self {
        let mut by_file: HashMap<String, Vec<Located>> = HashMap::new();
        let mut qualified = HashMap::new();
        let mut name_counts: HashMap<String, usize> = HashMap::new();
        let mut name_of = HashMap::new();

        for file in files {
            for symbol in &file.symbols {
                by_file.entry(file.path.clone()).or_default().push(Located {
                    id: symbol.id.clone(),
                    start_line: symbol.start_line,
                    end_line: symbol.end_line,
                });

                let name = symbol
                    .qualified_path
                    .rsplit("::")
                    .next()
                    .unwrap_or(&symbol.qualified_path)
                    .to_owned();

                qualified.insert(symbol.id.clone(), symbol.qualified_path.clone());
                *name_counts.entry(name.clone()).or_default() += 1;
                name_of.insert(symbol.id.clone(), name);
            }
        }

        Self {
            by_file,
            qualified,
            name_counts,
            name_of,
        }
    }

    /// The innermost symbol containing a 1-based line in a file.
    fn at(&self, path: &str, line: usize) -> Option<&str> {
        self.by_file
            .get(path)?
            .iter()
            .filter(|s| s.start_line <= line && line <= s.end_line)
            .max_by_key(|s| s.start_line)
            .map(|s| s.id.as_str())
    }

    /// How to describe a symbol when reporting a disagreement.
    fn class(&self, id: &str) -> &'static str {
        let Some(qualified) = self.qualified.get(id) else {
            return "unknown";
        };

        if qualified.contains(" as ") {
            "trait_impl_method"
        } else if qualified.contains("::") {
            "associated_item"
        } else {
            "free_item"
        }
    }

    fn name_is_ambiguous(&self, id: &str) -> bool {
        self.name_of
            .get(id)
            .and_then(|name| self.name_counts.get(name))
            .is_some_and(|count| *count > 1)
    }
}

// ---------------------------------------------------------------------------
// The oracle's view
// ---------------------------------------------------------------------------

#[derive(Default)]
struct Oracle {
    edges: BTreeSet<Pair>,
    /// Occurrences dropped because an endpoint is not a beholder symbol.
    outside_model: usize,
    documents: usize,
    occurrences: usize,
}

/// Read a SCIP index and project it onto beholder's symbols.
///
/// Every SCIP occurrence is a (file, range, symbol) triple. A definition
/// occurrence names where a symbol lives, which maps it to the beholder symbol
/// whose line range contains it. A reference occurrence maps the same way to the
/// symbol it was written inside. An edge exists when both ends land on beholder
/// symbols; anything else is outside the model and is excluded from both sides
/// rather than counted as a miss.
fn read_oracle(path: &PathBuf, symbols: &SymbolTable) -> Result<Oracle> {
    let bytes = std::fs::read(path)
        .with_context(|| format!("reading the SCIP index at {}", path.display()))?;
    let index = <scip::types::Index as protobuf::Message>::parse_from_bytes(&bytes)
        .with_context(|| format!("parsing the SCIP index at {}", path.display()))?;

    let mut definition_of: HashMap<String, String> = HashMap::new();
    let mut oracle = Oracle::default();

    // Pass one: where each SCIP symbol is defined.
    for document in &index.documents {
        for occurrence in &document.occurrences {
            if occurrence.symbol_roles & DEFINITION == 0 || is_local(&occurrence.symbol) {
                continue;
            }
            if let Some(id) = symbols.at(&document.relative_path, start_line(&occurrence.range)) {
                definition_of.insert(occurrence.symbol.clone(), id.to_owned());
            }
        }
    }

    // Pass two: every reference to a symbol that has a definition here.
    for document in &index.documents {
        oracle.documents += 1;

        for occurrence in &document.occurrences {
            if occurrence.symbol_roles & DEFINITION != 0 || is_local(&occurrence.symbol) {
                continue;
            }
            oracle.occurrences += 1;

            let line = start_line(&occurrence.range);
            let referrer = symbols.at(&document.relative_path, line);
            let referent = definition_of.get(&occurrence.symbol);

            match (referrer, referent) {
                (Some(from), Some(to)) if from != to => {
                    oracle.edges.insert((from.to_owned(), to.clone()));
                }
                // Recursion: beholder does not emit it either.
                (Some(_), Some(_)) => {}
                _ => oracle.outside_model += 1,
            }
        }
    }

    Ok(oracle)
}

/// SCIP names function-local bindings `local N`. They are never symbols.
fn is_local(symbol: &str) -> bool {
    symbol.starts_with("local ")
}

/// SCIP ranges are 0-based and either `[line, col, col]` or
/// `[line, col, line, col]`.
fn start_line(range: &[i32]) -> usize {
    range.first().copied().unwrap_or(0).max(0) as usize + 1
}

// ---------------------------------------------------------------------------
// Comparison
// ---------------------------------------------------------------------------

#[derive(serde::Serialize)]
struct Report {
    beholder_edges: usize,
    oracle_edges: usize,
    agreed: usize,
    precision: f32,
    recall: f32,
    oracle_documents: usize,
    oracle_reference_occurrences: usize,
    oracle_occurrences_outside_the_model: usize,
    false_positives_by_class: BTreeMap<String, usize>,
    false_negatives_by_class: BTreeMap<String, usize>,
    false_positive_examples: Vec<Pair>,
    false_negative_examples: Vec<Pair>,
}

impl Report {
    fn compare(
        ours: &BTreeSet<Pair>,
        theirs: &BTreeSet<Pair>,
        symbols: &SymbolTable,
        oracle: &Oracle,
    ) -> Self {
        let agreed = ours.intersection(theirs).count();
        let false_positives: Vec<&Pair> = ours.difference(theirs).collect();
        let false_negatives: Vec<&Pair> = theirs.difference(ours).collect();

        let classify = |pairs: &[&Pair]| {
            let mut counts: BTreeMap<String, usize> = BTreeMap::new();
            for (_, to) in pairs {
                let ambiguity = if symbols.name_is_ambiguous(to) {
                    "shared_name"
                } else {
                    "unique_name"
                };
                *counts
                    .entry(format!("{}/{ambiguity}", symbols.class(to)))
                    .or_default() += 1;
            }
            counts
        };

        let ratio = |numerator: usize, denominator: usize| {
            if denominator == 0 {
                0.0
            } else {
                numerator as f32 / denominator as f32
            }
        };

        Self {
            beholder_edges: ours.len(),
            oracle_edges: theirs.len(),
            agreed,
            precision: ratio(agreed, ours.len()),
            recall: ratio(agreed, theirs.len()),
            oracle_documents: oracle.documents,
            oracle_reference_occurrences: oracle.occurrences,
            oracle_occurrences_outside_the_model: oracle.outside_model,
            false_positives_by_class: classify(&false_positives),
            false_negatives_by_class: classify(&false_negatives),
            false_positive_examples: false_positives.into_iter().cloned().collect(),
            false_negative_examples: false_negatives.into_iter().cloned().collect(),
        }
    }

    fn print(&self, repository: &PathBuf, examples: usize) {
        println!("repository: {}", repository.display());
        println!(
            "beholder {} edges, oracle {} edges, agreed {}",
            self.beholder_edges, self.oracle_edges, self.agreed
        );
        println!("precision {:.3}  recall {:.3}", self.precision, self.recall);
        println!(
            "oracle: {} documents, {} reference occurrences, {} outside the model",
            self.oracle_documents,
            self.oracle_reference_occurrences,
            self.oracle_occurrences_outside_the_model
        );

        for (title, counts, samples) in [
            (
                "false positives (beholder saw an edge that is not there)",
                &self.false_positives_by_class,
                &self.false_positive_examples,
            ),
            (
                "false negatives (beholder missed a real edge)",
                &self.false_negatives_by_class,
                &self.false_negative_examples,
            ),
        ] {
            println!("\n{title}:");
            let mut ranked: Vec<_> = counts.iter().collect();
            ranked.sort_by_key(|(_, count)| std::cmp::Reverse(**count));
            for (class, count) in ranked {
                println!("  {count:>6}  {class}");
            }
            for (from, to) in samples.iter().take(examples) {
                println!("     e.g. {from}\n       -> {to}");
            }
        }
    }
}
