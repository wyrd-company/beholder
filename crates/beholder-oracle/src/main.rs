//! Measures beholder's heuristic reference graph against an external resolved
//! index.
//!
//! Beholder neither produces nor requires a SCIP index. This tool reads one that
//! already exists so the heuristic resolver can carry a measured error bar
//! instead of an assurance. Methodology is in `docs/resolver-accuracy.md`.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};

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
    /// SCIP index describing that repository. Generated if absent.
    #[arg(long)]
    index: Option<PathBuf>,
    /// Generate the index with rust-analyzer even if one exists, so the index
    /// and the analysis are guaranteed to describe the same tree.
    #[arg(long)]
    generate: bool,
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

    let provenance = Provenance::read(&cli.repository, cli.revision.as_deref())?;
    provenance.warn();

    let index_path = match (&cli.index, cli.generate) {
        (Some(path), false) => path.clone(),
        _ => generate_index(&cli.repository)?,
    };

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
    let oracle = read_oracle(&index_path, &symbols)?;

    let population: Vec<String> = analyzed
        .iter()
        .flat_map(|f| f.symbols.iter().map(|s| s.id.clone()))
        .collect();

    let oracle_fan_in = fan_in(&oracle.edges);
    let raw_fan_in = fan_in(&ours);
    let high: BTreeSet<Pair> = built
        .edges
        .iter()
        .filter(|e| e.confidence == beholder::resolve::Confidence::High)
        .map(|e| (e.from.clone(), e.to.clone()))
        .collect();
    let high_fan_in = fan_in(&high);

    // A third basis, measured but not shipped: keep every edge, but stop any
    // one low-confidence name match from dominating a symbol's fan-in. This is
    // evidence for how ranking should combine the two, not a ranking.
    const LOW_CONFIDENCE_CAP: usize = 5;
    let damped: HashMap<&str, usize> = population
        .iter()
        .map(|id| {
            let key = id.as_str();
            let all = *raw_fan_in.get(key).unwrap_or(&0);
            let confident = *high_fan_in.get(key).unwrap_or(&0);
            (
                key,
                confident + all.saturating_sub(confident).min(LOW_CONFIDENCE_CAP),
            )
        })
        .collect();

    let mut report = Report::compare(&ours, &oracle.edges, &symbols, &oracle);
    report.provenance = provenance;
    report.high_confidence_edges = high.len();
    report.fan_in = vec![
        FanInReport::build("raw", &population, &raw_fan_in, &oracle_fan_in, &symbols),
        FanInReport::build(
            "high_confidence",
            &population,
            &high_fan_in,
            &oracle_fan_in,
            &symbols,
        ),
        FanInReport::build(
            "high_plus_capped_low",
            &population,
            &damped,
            &oracle_fan_in,
            &symbols,
        ),
    ];

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
    start: (usize, usize),
    end: (usize, usize),
    /// Where this symbol's own name is written, if it has one.
    declaration: Option<(usize, usize, usize)>,
}

impl SymbolTable {
    fn build(files: &[FileAnalysis]) -> Self {
        let mut by_file: HashMap<String, Vec<Located>> = HashMap::new();
        let mut qualified = HashMap::new();
        let mut name_counts: HashMap<String, usize> = HashMap::new();
        let mut name_of = HashMap::new();

        for file in files {
            // Seed every analyzed file, so a file that simply declares no
            // symbols is not mistaken for one the analysis never saw.
            by_file.entry(file.path.clone()).or_default();

            for symbol in &file.symbols {
                by_file.entry(file.path.clone()).or_default().push(Located {
                    id: symbol.id.clone(),
                    start: (symbol.start_line, symbol.start_column),
                    end: (symbol.end_line, symbol.end_column),
                    declaration: symbol
                        .declaration
                        .map(|d| (d.line, d.start_column, d.end_column)),
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

    /// The innermost symbol whose span contains a position.
    ///
    /// Positions are (1-based line, 0-based column), compared as pairs so that
    /// two declarations on one line, or a nested one, land where they belong.
    fn containing(&self, path: &str, at: (usize, usize)) -> Option<&str> {
        self.by_file
            .get(path)?
            .iter()
            .filter(|s| s.start <= at && at < s.end)
            .max_by_key(|s| s.start)
            .map(|s| s.id.as_str())
    }

    /// Did the analysis see this file at all?
    fn knows_file(&self, path: &str) -> bool {
        self.by_file.contains_key(path)
    }

    /// The symbol whose own declared name occupies exactly this range.
    ///
    /// Containment is not enough. A field, an enum variant, an associated
    /// constant and a nested declaration all sit inside some symbol's span, and
    /// folding them onto whatever encloses them would turn references to things
    /// beholder does not model into apparently-correct edges — flattering both
    /// precision and recall. A definition is only beholder's symbol when it is
    /// that symbol's own name.
    fn declared_at(&self, path: &str, line: usize, start: usize, end: usize) -> Option<&str> {
        self.by_file
            .get(path)?
            .iter()
            .find(|s| s.declaration == Some((line, start, end)))
            .map(|s| s.id.as_str())
    }

    /// How to describe a symbol when reporting a disagreement.
    fn class(&self, id: &str) -> &'static str {
        let Some(qualified) = self.qualified.get(id) else {
            return "unknown";
        };

        if id.contains(":type:") {
            "type"
        } else if qualified.contains(" as ") {
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

/// Why an oracle occurrence produced no comparable edge.
///
/// Split apart so the exclusion set is auditable rather than one opaque number:
/// each of these is a different claim about what the measurement is not covering.
#[derive(Debug, Default, Clone, serde::Serialize)]
struct Excluded {
    /// The referenced thing is not a kind beholder models — a field, a variant,
    /// an associated constant, a static, a macro.
    unmodeled_referent: usize,
    /// The reference points outside the analyzed file set.
    external_dependency: usize,
    /// The reference sits outside any symbol, so there is nothing to attach an
    /// edge to: an `impl` header, a `use` declaration, a bare module item.
    no_modeled_referrer: usize,
    /// A SCIP function-local binding, which is never a symbol.
    local_binding: usize,
    /// The occurrence range could not be read.
    malformed_range: usize,
    /// A symbol referring to itself. Neither side emits these.
    recursion: usize,
}

#[derive(Default)]
struct Oracle {
    edges: BTreeSet<Pair>,
    excluded: Excluded,
    documents: usize,
    occurrences: usize,
    /// SCIP documents naming a file the analysis never saw.
    unknown_documents: BTreeSet<String>,
    /// How SCIP says its columns are counted.
    encoding: String,
}

/// A SCIP occurrence range as (1-based line, 0-based start column, end column).
///
/// SCIP writes either `[line, col, end_col]` for a single-line range or
/// `[line, col, end_line, end_col]`.
fn range_of(range: &[i32]) -> Option<(usize, usize, usize)> {
    let line = usize::try_from(*range.first()?).ok()?;
    let start = usize::try_from(*range.get(1)?).ok()?;
    let end = match range.len() {
        3 => usize::try_from(*range.get(2)?).ok()?,
        4 => usize::try_from(*range.get(3)?).ok()?,
        _ => return None,
    };
    Some((line + 1, start, end))
}

/// Read a SCIP index and project it onto beholder's symbols.
///
/// A definition occurrence contributes only when its range is exactly some
/// beholder symbol's declared name. A reference occurrence contributes only when
/// it falls inside some beholder symbol's span and names a symbol that cleared
/// the same bar. Everything else is excluded from both sides and counted under
/// the reason it was excluded.
fn read_oracle(path: &Path, symbols: &SymbolTable) -> Result<Oracle> {
    let bytes = std::fs::read(path)
        .with_context(|| format!("reading the SCIP index at {}", path.display()))?;
    let index = <scip::types::Index as protobuf::Message>::parse_from_bytes(&bytes)
        .with_context(|| format!("parsing the SCIP index at {}", path.display()))?;

    let mut definition_of: HashMap<String, String> = HashMap::new();
    // Every symbol declared anywhere in the index, whether or not beholder
    // models it. This is what separates "a field in this repository" from
    // "a function in a dependency".
    let mut declared_in_index: std::collections::HashSet<String> = Default::default();
    let mut oracle = Oracle {
        encoding: format!("{:?}", index.metadata.text_document_encoding),
        ..Oracle::default()
    };

    // Pass one: which SCIP symbols are declarations beholder also models.
    for document in &index.documents {
        for occurrence in &document.occurrences {
            if occurrence.symbol_roles & DEFINITION == 0 || is_local(&occurrence.symbol) {
                continue;
            }
            declared_in_index.insert(occurrence.symbol.clone());
            let Some((line, start, end)) = range_of(&occurrence.range) else {
                continue;
            };
            if let Some(id) = symbols.declared_at(&document.relative_path, line, start, end) {
                definition_of.insert(occurrence.symbol.clone(), id.to_owned());
            }
        }
    }

    // Pass two: every reference between two symbols beholder models.
    for document in &index.documents {
        oracle.documents += 1;

        if !symbols.knows_file(&document.relative_path) {
            oracle
                .unknown_documents
                .insert(document.relative_path.clone());
        }

        for occurrence in &document.occurrences {
            if occurrence.symbol_roles & DEFINITION != 0 {
                continue;
            }
            oracle.occurrences += 1;

            if is_local(&occurrence.symbol) {
                oracle.excluded.local_binding += 1;
                continue;
            }
            let Some((line, start, _)) = range_of(&occurrence.range) else {
                oracle.excluded.malformed_range += 1;
                continue;
            };

            let referrer = symbols.containing(&document.relative_path, (line, start));
            let referent = definition_of.get(&occurrence.symbol);

            match (referrer, referent) {
                (Some(from), Some(to)) if from != to => {
                    oracle.edges.insert((from.to_owned(), to.clone()));
                }
                (Some(_), Some(_)) => oracle.excluded.recursion += 1,
                (None, _) => oracle.excluded.no_modeled_referrer += 1,
                // The name resolves somewhere, but not to anything beholder
                // models. Whether that is a field in this repository or a
                // function in a dependency is the difference between these two.
                (Some(_), None) => {
                    if declared_in_index.contains(&occurrence.symbol) {
                        oracle.excluded.unmodeled_referent += 1;
                    } else {
                        oracle.excluded.external_dependency += 1;
                    }
                }
            }
        }
    }

    Ok(oracle)
}

/// SCIP names function-local bindings `local N`. They are never symbols.
fn is_local(symbol: &str) -> bool {
    symbol.starts_with("local ")
}

// ---------------------------------------------------------------------------
// Comparison
// ---------------------------------------------------------------------------

/// What tree the measurement actually describes.
///
/// A SCIP index is just a file; nothing in it says which revision it was built
/// from. Recording the revision and whether the tree was dirty is the minimum
/// that lets a reader tell whether the two sides saw the same code.
#[derive(Debug, Default, Clone, serde::Serialize)]
struct Provenance {
    revision: Option<String>,
    head: Option<String>,
    dirty: bool,
}

impl Provenance {
    fn read(repository: &Path, revision: Option<&str>) -> Result<Self> {
        let Ok(repo) = git2::Repository::discover(repository) else {
            return Ok(Self::default());
        };

        let head = repo
            .head()
            .ok()
            .and_then(|h| h.target())
            .map(|id| id.to_string());
        let mut options = git2::StatusOptions::new();
        options.include_untracked(false);
        let dirty = repo
            .statuses(Some(&mut options))
            .map(|statuses| !statuses.is_empty())
            .unwrap_or(false);

        Ok(Self {
            revision: revision.map(str::to_owned),
            head,
            dirty,
        })
    }

    fn warn(&self) {
        if self.dirty {
            eprintln!(
                "WARNING: the working tree has uncommitted changes, so the index and the \
                 analysis may describe different code"
            );
        }
    }
}

/// Build a SCIP index for a repository, so index and analysis cannot drift.
fn generate_index(repository: &Path) -> Result<PathBuf> {
    eprintln!("generating a SCIP index with rust-analyzer...");

    let status = std::process::Command::new("rust-analyzer")
        .arg("scip")
        .arg(".")
        .current_dir(repository)
        .status()
        .context("running `rust-analyzer scip`; is rust-analyzer on PATH?")?;

    anyhow::ensure!(status.success(), "rust-analyzer scip failed: {status}");

    Ok(repository.join("index.scip"))
}

#[derive(serde::Serialize)]
struct Report {
    #[serde(default)]
    provenance: Provenance,
    beholder_edges: usize,
    high_confidence_edges: usize,
    fan_in: Vec<FanInReport>,
    oracle_edges: usize,
    agreed: usize,
    precision: f32,
    recall: f32,
    oracle_documents: usize,
    oracle_reference_occurrences: usize,
    oracle_excluded: Excluded,
    oracle_encoding: String,
    oracle_documents_not_analyzed: Vec<String>,
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
            provenance: Provenance::default(),
            high_confidence_edges: 0,
            fan_in: Vec::new(),
            beholder_edges: ours.len(),
            oracle_edges: theirs.len(),
            agreed,
            precision: ratio(agreed, ours.len()),
            recall: ratio(agreed, theirs.len()),
            oracle_documents: oracle.documents,
            oracle_reference_occurrences: oracle.occurrences,
            oracle_excluded: oracle.excluded.clone(),
            oracle_encoding: oracle.encoding.clone(),
            oracle_documents_not_analyzed: oracle.unknown_documents.iter().cloned().collect(),
            false_positives_by_class: classify(&false_positives),
            false_negatives_by_class: classify(&false_negatives),
            false_positive_examples: false_positives.into_iter().cloned().collect(),
            false_negative_examples: false_negatives.into_iter().cloned().collect(),
        }
    }

    fn print(&self, repository: &Path, examples: usize) {
        println!("repository: {}", repository.display());
        if let Some(head) = &self.provenance.head {
            println!(
                "revision: {head}{}",
                if self.provenance.dirty {
                    " (DIRTY)"
                } else {
                    ""
                }
            );
        }
        println!(
            "beholder {} edges ({} high confidence), oracle {} edges, agreed {}",
            self.beholder_edges, self.high_confidence_edges, self.oracle_edges, self.agreed
        );
        println!("precision {:.3}  recall {:.3}", self.precision, self.recall);
        println!(
            "oracle: {} documents, {} reference occurrences, columns {}",
            self.oracle_documents, self.oracle_reference_occurrences, self.oracle_encoding
        );
        println!("excluded from both sides:");
        for (label, count) in [
            (
                "unmodeled referent",
                self.oracle_excluded.unmodeled_referent,
            ),
            (
                "external dependency",
                self.oracle_excluded.external_dependency,
            ),
            (
                "no modeled referrer",
                self.oracle_excluded.no_modeled_referrer,
            ),
            ("local binding", self.oracle_excluded.local_binding),
            ("malformed range", self.oracle_excluded.malformed_range),
            ("recursion", self.oracle_excluded.recursion),
        ] {
            println!("  {count:>7}  {label}");
        }
        if !self.oracle_documents_not_analyzed.is_empty() {
            println!(
                "WARNING: {} SCIP documents name files the analysis never saw",
                self.oracle_documents_not_analyzed.len()
            );
        }

        for report in &self.fan_in {
            println!("\nfan-in, {} basis:", report.basis);
            println!(
                "  spearman {:.3}   top-10 overlap {:.2}   top-25 overlap {:.2}",
                report.spearman, report.top_10_overlap, report.top_25_overlap
            );
            println!(
                "  mean abs error {:.2}   median {:.2}   mean relative {:.2}",
                report.mean_absolute_error,
                report.median_absolute_error,
                report.mean_relative_error
            );
            println!(
                "  worst ten symbols absorb {:.0}% of all overstatement",
                report.overstatement_concentration * 100.0
            );
            for (class, stats) in &report.by_class {
                println!(
                    "    {class:<20} n={:<5} spearman {:.3}  mean abs error {:.2}",
                    stats.symbols, stats.spearman, stats.mean_absolute_error
                );
            }
            for worst in report.worst_overstated.iter().take(3) {
                println!(
                    "    overstated +{:<4} {} (beholder {} vs oracle {})",
                    worst.beholder - worst.oracle,
                    worst.symbol,
                    worst.beholder,
                    worst.oracle
                );
            }
        }

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

// ---------------------------------------------------------------------------
// Fan-in, which is what ranking actually consumes
// ---------------------------------------------------------------------------

/// How well one basis for fan-in tracks the oracle.
///
/// Pair-level precision and recall say whether individual edges are right. They
/// do not say whether the number ranking consumes is right, because errors can
/// cancel or concentrate. This measures the number itself.
#[derive(Debug, serde::Serialize)]
struct FanInReport {
    basis: String,
    symbols: usize,
    /// Symbols the oracle gives any fan-in at all.
    symbols_with_oracle_fan_in: usize,
    mean_absolute_error: f64,
    median_absolute_error: f64,
    /// Mean of |beholder - oracle| / oracle, over symbols the oracle ranks.
    mean_relative_error: f64,
    /// Rank correlation over every symbol. This is what percentile ranking rides on.
    spearman: f64,
    top_10_overlap: f64,
    top_25_overlap: f64,
    /// Share of all overstatement absorbed by the ten worst symbols.
    overstatement_concentration: f64,
    by_class: BTreeMap<String, ClassFanIn>,
    worst_overstated: Vec<Overstatement>,
}

#[derive(Debug, serde::Serialize)]
struct ClassFanIn {
    symbols: usize,
    mean_absolute_error: f64,
    spearman: f64,
}

#[derive(Debug, serde::Serialize)]
struct Overstatement {
    symbol: String,
    beholder: usize,
    oracle: usize,
}

/// Fan-in per symbol, from a set of directed pairs.
fn fan_in(edges: &BTreeSet<Pair>) -> HashMap<&str, usize> {
    let mut counts: HashMap<&str, BTreeSet<&str>> = HashMap::new();
    for (from, to) in edges {
        counts.entry(to.as_str()).or_default().insert(from.as_str());
    }
    counts
        .into_iter()
        .map(|(id, sources)| (id, sources.len()))
        .collect()
}

impl FanInReport {
    fn build(
        basis: &str,
        population: &[String],
        ours: &HashMap<&str, usize>,
        theirs: &HashMap<&str, usize>,
        symbols: &SymbolTable,
    ) -> Self {
        let paired: Vec<(&String, f64, f64)> = population
            .iter()
            .map(|id| {
                (
                    id,
                    *ours.get(id.as_str()).unwrap_or(&0) as f64,
                    *theirs.get(id.as_str()).unwrap_or(&0) as f64,
                )
            })
            .collect();

        let mut absolute: Vec<f64> = paired.iter().map(|(_, a, b)| (a - b).abs()).collect();
        absolute.sort_by(|a, b| a.partial_cmp(b).expect("no NaN"));

        let ranked: Vec<f64> = paired
            .iter()
            .filter(|(_, _, oracle)| *oracle > 0.0)
            .map(|(_, ours, oracle)| (ours - oracle).abs() / oracle)
            .collect();

        let mut overstated: Vec<Overstatement> = paired
            .iter()
            .filter(|(_, ours, oracle)| ours > oracle)
            .map(|(id, ours, oracle)| Overstatement {
                symbol: (*id).clone(),
                beholder: *ours as usize,
                oracle: *oracle as usize,
            })
            .collect();
        overstated.sort_by_key(|o| std::cmp::Reverse(o.beholder - o.oracle));

        let total_overstatement: usize = overstated.iter().map(|o| o.beholder - o.oracle).sum();
        let worst_ten: usize = overstated
            .iter()
            .take(10)
            .map(|o| o.beholder - o.oracle)
            .sum();

        let mut by_class: BTreeMap<String, ClassFanIn> = BTreeMap::new();
        for class in ["free_item", "associated_item", "trait_impl_method", "type"] {
            let subset: Vec<(&String, f64, f64)> = paired
                .iter()
                .filter(|(id, _, _)| symbols.class(id) == class)
                .cloned()
                .collect();
            if subset.is_empty() {
                continue;
            }
            by_class.insert(
                class.to_owned(),
                ClassFanIn {
                    symbols: subset.len(),
                    mean_absolute_error: mean(
                        &subset
                            .iter()
                            .map(|(_, a, b)| (a - b).abs())
                            .collect::<Vec<_>>(),
                    ),
                    spearman: spearman(&subset),
                },
            );
        }

        Self {
            basis: basis.to_owned(),
            symbols: paired.len(),
            symbols_with_oracle_fan_in: paired.iter().filter(|(_, _, o)| *o > 0.0).count(),
            mean_absolute_error: mean(&absolute),
            median_absolute_error: median(&absolute),
            mean_relative_error: mean(&ranked),
            spearman: spearman(&paired),
            top_10_overlap: top_k_overlap(&paired, 10),
            top_25_overlap: top_k_overlap(&paired, 25),
            overstatement_concentration: if total_overstatement == 0 {
                0.0
            } else {
                worst_ten as f64 / total_overstatement as f64
            },
            by_class,
            worst_overstated: overstated.into_iter().take(10).collect(),
        }
    }
}

fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

fn median(sorted: &[f64]) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    sorted[sorted.len() / 2]
}

/// Average ranks, so ties do not invent an ordering that is not there.
fn ranks(values: &[f64]) -> Vec<f64> {
    let mut order: Vec<usize> = (0..values.len()).collect();
    order.sort_by(|a, b| values[*a].partial_cmp(&values[*b]).expect("no NaN"));

    let mut out = vec![0.0; values.len()];
    let mut index = 0;

    while index < order.len() {
        let mut end = index;
        while end + 1 < order.len() && values[order[end + 1]] == values[order[index]] {
            end += 1;
        }
        let rank = (index + end) as f64 / 2.0 + 1.0;
        for slot in &order[index..=end] {
            out[*slot] = rank;
        }
        index = end + 1;
    }

    out
}

/// Spearman correlation: Pearson over ranks.
fn spearman(paired: &[(&String, f64, f64)]) -> f64 {
    if paired.len() < 2 {
        return 0.0;
    }

    let ours = ranks(&paired.iter().map(|(_, a, _)| *a).collect::<Vec<_>>());
    let theirs = ranks(&paired.iter().map(|(_, _, b)| *b).collect::<Vec<_>>());

    let (mean_ours, mean_theirs) = (mean(&ours), mean(&theirs));
    let mut covariance = 0.0;
    let mut variance_ours = 0.0;
    let mut variance_theirs = 0.0;

    for (a, b) in ours.iter().zip(&theirs) {
        covariance += (a - mean_ours) * (b - mean_theirs);
        variance_ours += (a - mean_ours).powi(2);
        variance_theirs += (b - mean_theirs).powi(2);
    }

    if variance_ours == 0.0 || variance_theirs == 0.0 {
        return 0.0;
    }

    covariance / (variance_ours.sqrt() * variance_theirs.sqrt())
}

/// Share of the oracle's top k that beholder's top k also contains.
fn top_k_overlap(paired: &[(&String, f64, f64)], k: usize) -> f64 {
    let top = |pick: fn(&(&String, f64, f64)) -> f64| {
        let mut sorted: Vec<&(&String, f64, f64)> = paired.iter().collect();
        sorted.sort_by(|a, b| pick(b).partial_cmp(&pick(a)).expect("no NaN"));
        sorted
            .into_iter()
            .take(k)
            .map(|(id, _, _)| (*id).clone())
            .collect::<BTreeSet<String>>()
    };

    let ours = top(|entry| entry.1);
    let theirs = top(|entry| entry.2);

    if theirs.is_empty() {
        return 0.0;
    }

    ours.intersection(&theirs).count() as f64 / theirs.len() as f64
}
