//! Tier 2 — resolved references.
//!
//! Turning an identifier occurrence into an edge between two symbols needs the
//! whole file set, which is what makes this phase 2 and not phase 1.
//!
//! Resolution sits behind [`Resolver`] so an externally produced index can take
//! its place. Beholder generates no such index and requires none; the interface
//! exists so that a project which already has one is not forced to accept a
//! heuristic's error bar.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use serde::{Deserialize, Serialize};

use crate::lang;
use crate::phase1::{FileAnalysis, Occurrence};
use crate::Symbol;

/// One resolved reference between two symbols.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Edge {
    /// Symbol id the reference was written inside.
    pub from: String,
    /// Symbol id it resolved to.
    pub to: String,
    /// The syntactic position it was written in, from the references query.
    pub kind: String,
}

/// How much an edge set can be trusted.
///
/// Carried in the graph output because a reference graph without an error bar
/// invites being read as ground truth, and a heuristic one never is.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Accuracy {
    /// Which resolver produced the edges.
    pub resolver: String,
    /// Fraction of emitted edges that are correct, where measured.
    pub precision: Option<f32>,
    /// Fraction of real edges that were emitted, where measured.
    pub recall: Option<f32>,
    /// What the measurement compared against.
    pub measured_against: Option<String>,
    /// Which repositories the measurement covered.
    pub corpus: Option<String>,
}

impl Accuracy {
    /// The recorded accuracy of the heuristic resolver for a language.
    ///
    /// Measurements live in `accuracy.toml` beside this file, produced by the
    /// `beholder-oracle` crate. A language with no recorded measurement reports
    /// `None` rather than a guess.
    pub fn recorded(language: &str) -> Self {
        #[derive(Deserialize)]
        struct Recorded {
            precision: f32,
            recall: f32,
            measured_against: String,
            corpus: String,
        }

        let table: BTreeMap<String, Recorded> =
            toml::from_str(include_str!("../accuracy.toml")).unwrap_or_default();

        match table.get(language) {
            Some(recorded) => Self {
                resolver: HEURISTIC.to_owned(),
                precision: Some(recorded.precision),
                recall: Some(recorded.recall),
                measured_against: Some(recorded.measured_against.clone()),
                corpus: Some(recorded.corpus.clone()),
            },
            None => Self {
                resolver: HEURISTIC.to_owned(),
                precision: None,
                recall: None,
                measured_against: None,
                corpus: None,
            },
        }
    }
}

/// Name of the built-in resolver.
pub const HEURISTIC: &str = "heuristic";

/// What resolution reads.
pub struct ResolveInput<'a> {
    pub files: &'a [FileAnalysis],
}

/// Why an occurrence produced no edge.
///
/// Kept because the shape of the misses is the useful part of a measurement: it
/// says which language feature to teach the resolver next.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Unresolved {
    /// No symbol anywhere carries that name.
    NoCandidate,
    /// Several symbols do, and nothing chose between them.
    Ambiguous,
    /// The occurrence sits outside any symbol, so there is nothing to attach an
    /// edge to.
    NoReferrer,
}

/// Counts of what resolution could not do.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolutionStats {
    pub occurrences: usize,
    pub resolved: usize,
    pub unresolved: BTreeMap<String, usize>,
}

/// Produces edges between symbols.
pub trait Resolver {
    /// Resolve occurrences into edges, stably sorted.
    fn resolve(&self, input: &ResolveInput<'_>) -> (Vec<Edge>, ResolutionStats);

    /// How much the edges this resolver produces can be trusted.
    fn accuracy(&self, language: &str) -> Accuracy;
}

/// Name-based resolution from imports, module paths and identifier occurrences.
///
/// The governing rule is that an ambiguous name resolves to nothing. Emitting
/// every candidate would let one common method name — `new`, `from`, `fmt` —
/// manufacture fan-in for every type in a repository, which is exactly the
/// signal ranking is built on.
#[derive(Debug, Default, Clone, Copy)]
pub struct Heuristic;

impl Resolver for Heuristic {
    fn resolve(&self, input: &ResolveInput<'_>) -> (Vec<Edge>, ResolutionStats) {
        let index = SymbolIndex::build(input.files);
        let mut edges = BTreeSet::new();
        let mut stats = ResolutionStats::default();

        for file in input.files {
            for occurrence in &file.occurrences {
                stats.occurrences += 1;

                let Some(within) = occurrence.within else {
                    *stats
                        .unresolved
                        .entry(label(Unresolved::NoReferrer))
                        .or_default() += 1;
                    continue;
                };
                let Some(from) = file.symbols.get(within) else {
                    continue;
                };

                match index.resolve(file, occurrence) {
                    Ok(to) => {
                        stats.resolved += 1;
                        // A symbol referring to itself says nothing about what
                        // depends on it, so recursion is not an edge.
                        if to != from.id {
                            edges.insert(Edge {
                                from: from.id.clone(),
                                to,
                                kind: occurrence.kind.clone(),
                            });
                        }
                    }
                    Err(reason) => *stats.unresolved.entry(label(reason)).or_default() += 1,
                }
            }
        }

        (edges.into_iter().collect(), stats)
    }

    fn accuracy(&self, language: &str) -> Accuracy {
        Accuracy::recorded(language)
    }
}

fn label(reason: Unresolved) -> String {
    match reason {
        Unresolved::NoCandidate => "no_candidate",
        Unresolved::Ambiguous => "ambiguous",
        Unresolved::NoReferrer => "no_referrer",
    }
    .to_owned()
}

// ---------------------------------------------------------------------------
// Lookup
// ---------------------------------------------------------------------------

/// Every way a symbol can be named, precomputed.
struct SymbolIndex<'a> {
    /// Module-qualified name to symbol ids: `git::walk_to_tag`.
    by_module_path: HashMap<String, Vec<&'a Symbol>>,
    /// Last segment of the qualified path to symbol ids: `walk_to_tag`.
    by_name: HashMap<&'a str, Vec<&'a Symbol>>,
    /// Symbol ids defined in each file, by last segment.
    by_file: HashMap<&'a str, HashMap<&'a str, Vec<&'a Symbol>>>,
}

impl<'a> SymbolIndex<'a> {
    fn build(files: &'a [FileAnalysis]) -> Self {
        let mut by_module_path: HashMap<String, Vec<&Symbol>> = HashMap::new();
        let mut by_name: HashMap<&str, Vec<&Symbol>> = HashMap::new();
        let mut by_file: HashMap<&str, HashMap<&str, Vec<&Symbol>>> = HashMap::new();

        for file in files {
            let separator = separator_for(file);
            let module = file.module_segments.join(separator);

            for symbol in &file.symbols {
                let name = last_segment(&symbol.qualified_path, separator);

                by_name.entry(name).or_default().push(symbol);
                by_file
                    .entry(file.path.as_str())
                    .or_default()
                    .entry(name)
                    .or_default()
                    .push(symbol);

                let qualified = if module.is_empty() {
                    symbol.qualified_path.clone()
                } else {
                    format!("{module}{separator}{}", symbol.qualified_path)
                };
                by_module_path.entry(qualified).or_default().push(symbol);
            }
        }

        Self {
            by_module_path,
            by_name,
            by_file,
        }
    }

    /// Resolve one occurrence to a single symbol id, or say why not.
    fn resolve(
        &self,
        file: &'a FileAnalysis,
        occurrence: &Occurrence,
    ) -> Result<String, Unresolved> {
        let separator = separator_for(file);
        let mut saw_candidate = false;

        // Strongest signal first. Each rule either names exactly one symbol or
        // steps aside; nothing votes.
        for candidates in [
            self.by_import(file, occurrence, separator),
            self.by_qualifier(occurrence, separator),
            self.in_same_file(file, occurrence),
            self.by_glob_import(file, occurrence, separator),
            self.anywhere(occurrence),
        ] {
            match candidates.len() {
                0 => continue,
                1 => return Ok(candidates[0].id.clone()),
                _ => saw_candidate = true,
            }
        }

        Err(if saw_candidate {
            Unresolved::Ambiguous
        } else {
            Unresolved::NoCandidate
        })
    }

    /// The name was brought into scope by an import in this file.
    fn by_import(
        &self,
        file: &'a FileAnalysis,
        occurrence: &Occurrence,
        separator: &str,
    ) -> Vec<&'a Symbol> {
        let Some(import) = file
            .imports
            .iter()
            .find(|i| !i.glob && i.local_name == occurrence.name)
        else {
            return Vec::new();
        };

        self.by_module_path
            .get(&import.path.join(separator))
            .cloned()
            .unwrap_or_default()
    }

    /// Something narrowed the lookup: `Type::method`, `module::thing`.
    fn by_qualifier(&self, occurrence: &Occurrence, separator: &str) -> Vec<&'a Symbol> {
        let Some(qualifier) = &occurrence.qualifier else {
            return Vec::new();
        };

        // As a module path first, then as a scope on the symbol itself. An impl
        // block spells its scope in the qualified path, so `Canvas::draw` and
        // `<Canvas as Paint>::draw` both mention `Canvas`.
        let module_qualified = format!("{qualifier}{separator}{}", occurrence.name);
        if let Some(found) = self.by_module_path.get(&module_qualified) {
            return found.clone();
        }

        self.by_name
            .get(occurrence.name.as_str())
            .map(|symbols| {
                symbols
                    .iter()
                    .filter(|symbol| scope_mentions(&symbol.qualified_path, separator, qualifier))
                    .copied()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Defined in the same file, which shadows anything further away.
    fn in_same_file(&self, file: &'a FileAnalysis, occurrence: &Occurrence) -> Vec<&'a Symbol> {
        self.by_file
            .get(file.path.as_str())
            .and_then(|names| names.get(occurrence.name.as_str()))
            .cloned()
            .unwrap_or_default()
    }

    /// A glob import could have brought it in from one of those modules.
    fn by_glob_import(
        &self,
        file: &'a FileAnalysis,
        occurrence: &Occurrence,
        separator: &str,
    ) -> Vec<&'a Symbol> {
        let mut found = Vec::new();

        for import in file.imports.iter().filter(|i| i.glob) {
            let qualified = format!(
                "{}{separator}{}",
                import.path.join(separator),
                occurrence.name
            );
            if let Some(symbols) = self.by_module_path.get(&qualified) {
                found.extend(symbols.iter().copied());
            }
        }

        found
    }

    /// Nothing narrowed it, so the name has to be unique in the project.
    fn anywhere(&self, occurrence: &Occurrence) -> Vec<&'a Symbol> {
        self.by_name
            .get(occurrence.name.as_str())
            .cloned()
            .unwrap_or_default()
    }
}

pub(crate) fn separator_for(file: &FileAnalysis) -> &'static str {
    file.language
        .as_deref()
        .and_then(lang::by_id)
        .map_or("::", |l| l.path_separator)
}

pub(crate) fn last_segment<'s>(qualified_path: &'s str, separator: &str) -> &'s str {
    qualified_path
        .rsplit(separator)
        .next()
        .unwrap_or(qualified_path)
}

/// Does the scope part of a qualified path mention this name?
///
/// Splitting on non-identifier characters is what makes `<Canvas as Paint>`
/// answer to both `Canvas` and `Paint` without any language-specific parsing of
/// how an impl block is spelled.
pub(crate) fn scope_mentions(qualified_path: &str, separator: &str, name: &str) -> bool {
    let Some(scope) = qualified_path.strip_suffix(last_segment(qualified_path, separator)) else {
        return false;
    };

    scope
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .any(|word| word == name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phase1;

    fn resolve(files: &[(&str, &str)]) -> (Vec<Edge>, ResolutionStats) {
        let analyzed: Vec<_> = files
            .iter()
            .map(|(path, contents)| phase1::analyze(path, contents))
            .collect();
        Heuristic.resolve(&ResolveInput { files: &analyzed })
    }

    fn targets(edges: &[Edge], from_contains: &str) -> Vec<String> {
        edges
            .iter()
            .filter(|e| e.from.contains(from_contains))
            .map(|e| e.to.clone())
            .collect()
    }

    #[test]
    fn a_call_within_one_file_resolves() {
        let (edges, _) = resolve(&[(
            "src/lib.rs",
            "fn caller() {\n    callee();\n}\nfn callee() {}\n",
        )]);

        assert_eq!(
            targets(&edges, "caller"),
            vec!["rust:src/lib.rs:function:callee"]
        );
    }

    #[test]
    fn an_imported_call_resolves_across_files() {
        let (edges, _) = resolve(&[
            (
                "src/caller.rs",
                "use crate::worker::do_work;\nfn caller() {\n    do_work();\n}\n",
            ),
            ("src/worker.rs", "pub fn do_work() {}\n"),
        ]);

        assert_eq!(
            targets(&edges, "caller"),
            vec!["rust:src/worker.rs:function:do_work"]
        );
    }

    #[test]
    fn a_qualified_call_resolves_through_its_type() {
        let (edges, _) = resolve(&[(
            "src/lib.rs",
            "struct Ledger;\nimpl Ledger { fn tally() {} }\nfn run() {\n    Ledger::tally();\n}\n",
        )]);

        assert!(targets(&edges, "function:run")
            .contains(&"rust:src/lib.rs:function:Ledger::tally".to_string()));
    }

    #[test]
    fn an_ambiguous_name_resolves_to_nothing() {
        // Two unrelated types both have `new`. Guessing would invent fan-in.
        let (edges, stats) = resolve(&[
            (
                "src/a.rs",
                "pub struct A;\nimpl A { pub fn new() -> A { A } }\n",
            ),
            (
                "src/b.rs",
                "pub struct B;\nimpl B { pub fn new() -> B { B } }\n",
            ),
            ("src/c.rs", "fn build(x: A) {\n    x.new();\n}\n"),
        ]);

        // The type mention still resolves; only the method call is ambiguous.
        assert!(
            !targets(&edges, "function:build")
                .iter()
                .any(|to| to.ends_with("new")),
            "{edges:#?}"
        );
        assert!(stats.unresolved.contains_key("ambiguous"));
    }

    #[test]
    fn recursion_is_not_an_edge() {
        let (edges, _) = resolve(&[(
            "src/lib.rs",
            "fn countdown(n: u32) {\n    if n > 0 {\n        countdown(n - 1);\n    }\n}\n",
        )]);

        assert!(edges.is_empty(), "{edges:#?}");
    }

    #[test]
    fn a_type_mention_is_an_edge() {
        let (edges, _) = resolve(&[
            ("src/model.rs", "pub struct Ledger;\n"),
            (
                "src/use.rs",
                "use crate::model::Ledger;\nfn tally(l: Ledger) {}\n",
            ),
        ]);

        assert_eq!(
            targets(&edges, "function:tally"),
            vec!["rust:src/model.rs:type:Ledger"]
        );
    }

    #[test]
    fn a_glob_import_can_resolve_a_name() {
        let (edges, _) = resolve(&[
            ("src/helpers.rs", "pub fn assist() {}\n"),
            (
                "src/caller.rs",
                "use crate::helpers::*;\nfn caller() {\n    assist();\n}\n",
            ),
        ]);

        assert_eq!(
            targets(&edges, "caller"),
            vec!["rust:src/helpers.rs:function:assist"]
        );
    }

    #[test]
    fn edges_are_deterministic_and_stably_sorted() {
        let files = [
            ("src/z.rs", "pub fn zeta() {}\n"),
            ("src/a.rs", "pub fn alpha() {}\n"),
            (
                "src/caller.rs",
                "use crate::z::zeta;\nuse crate::a::alpha;\nfn caller() {\n    zeta();\n    alpha();\n}\n",
            ),
        ];

        let (first, _) = resolve(&files);
        let (second, _) = resolve(&files);

        assert_eq!(first, second);
        let mut sorted = first.clone();
        sorted.sort();
        assert_eq!(first, sorted);
    }

    #[test]
    fn accuracy_is_reported_or_admitted_absent() {
        let rust = Heuristic.accuracy("rust");
        assert_eq!(rust.resolver, HEURISTIC);

        let unmeasured = Heuristic.accuracy("cobol");
        assert_eq!(unmeasured.precision, None);
        assert_eq!(unmeasured.recall, None);
    }
}
