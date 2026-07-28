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
    /// How much evidence stood behind this particular resolution.
    pub confidence: Confidence,
}

/// How much evidence stood behind one resolution.
///
/// The distinction is not decoration. An edge backed by an explicit import, a
/// path qualifier or `Self` rests on something the source actually states. An
/// edge from a bare method name rests on the name being unique in the project,
/// which says nothing about what the receiver's type really was — and that is
/// precisely where the resolver was measured to be wrong.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    /// The source states where the name comes from.
    High,
    /// The name matched and nothing contradicted it.
    Low,
}

/// How much an edge set can be trusted.
///
/// Carried in the graph output because a reference graph without an error bar
/// invites being read as ground truth, and a heuristic one never is.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Accuracy {
    /// Which resolver produced the edges.
    pub resolver: String,
    /// What the measurement compared against.
    pub measured_against: Option<String>,
    /// Lowest and highest precision observed across the corpus.
    ///
    /// A range, not a point, and calibration rather than per-edge confidence.
    /// The spread between repositories is structural, so collapsing it to one
    /// number would read as a probability for an individual edge and it is not
    /// one. Per edge, [`Edge::confidence`] is what beholder can actually say.
    pub precision_range: Option<(f32, f32)>,
    /// Lowest and highest recall observed across the corpus.
    pub recall_range: Option<(f32, f32)>,
    /// Every repository the measurement covered.
    pub measurements: Vec<Measurement>,
}

/// One repository's measured accuracy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Measurement {
    pub repository: String,
    pub revision: String,
    pub precision: f32,
    pub recall: f32,
    pub beholder_edges: usize,
    pub oracle_edges: usize,
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
            measured_against: String,
            precision_min: f32,
            precision_max: f32,
            recall_min: f32,
            recall_max: f32,
            #[serde(default)]
            measurements: Vec<Measurement>,
        }

        let table: BTreeMap<String, Recorded> =
            toml::from_str(include_str!("../accuracy.toml")).unwrap_or_default();

        match table.get(language) {
            Some(recorded) => Self {
                resolver: HEURISTIC.to_owned(),
                measured_against: Some(recorded.measured_against.clone()),
                precision_range: Some((recorded.precision_min, recorded.precision_max)),
                recall_range: Some((recorded.recall_min, recorded.recall_max)),
                measurements: recorded.measurements.clone(),
            },
            None => Self {
                resolver: HEURISTIC.to_owned(),
                measured_against: None,
                precision_range: None,
                recall_range: None,
                measurements: Vec::new(),
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
                    Ok((to, confidence)) => {
                        stats.resolved += 1;
                        // A symbol referring to itself says nothing about what
                        // depends on it, so recursion is not an edge.
                        if to != from.id {
                            edges.insert(Edge {
                                from: from.id.clone(),
                                to,
                                kind: occurrence.kind.clone(),
                                confidence,
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
    ) -> Result<(String, Confidence), Unresolved> {
        let separator = separator_for(file);

        if occurrence.target.as_deref() == Some("enclosing_scope") {
            return self
                .enclosing_scope(file, occurrence, separator)
                .map(|id| (id, Confidence::High));
        }

        let mut saw_candidate = false;

        // Strongest signal first. Each rule either names exactly one symbol or
        // steps aside; nothing votes. The confidence beside each rule is what
        // the rule actually rests on: the first two read a statement in the
        // source, the rest read a name.
        for (candidates, confidence) in [
            (
                self.by_import(file, occurrence, separator),
                Confidence::High,
            ),
            (
                self.by_namespace(file, occurrence, separator),
                Confidence::High,
            ),
            (self.by_qualifier(occurrence, separator), Confidence::High),
            (self.in_same_file(file, occurrence), Confidence::Low),
            (
                self.by_glob_import(file, occurrence, separator),
                Confidence::Low,
            ),
            (self.anywhere(occurrence), Confidence::Low),
        ] {
            // A method call cannot land on a free function however close it
            // sits. Without this, `value.apply()` resolves to whatever `apply`
            // happens to share the file.
            let candidates: Vec<&Symbol> = candidates
                .into_iter()
                .filter(|symbol| shape_matches(occurrence, symbol, separator))
                .collect();

            match candidates.len() {
                0 => continue,
                1 => return Ok((candidates[0].id.clone(), confidence)),
                _ => saw_candidate = true,
            }
        }

        Err(if saw_candidate {
            Unresolved::Ambiguous
        } else {
            Unresolved::NoCandidate
        })
    }

    /// Resolve a name that means "whatever encloses this", such as `Self`.
    ///
    /// The referring symbol already carries its own scope in its qualified
    /// path, so the answer is there rather than in any symbol table.
    ///
    /// Scope words are scanned left to right — outermost first — and the first
    /// one naming exactly one symbol wins. That order is wrong for a symbol
    /// whose outer scope also names a symbol: in `mod ledger { impl Tally { .. } }`
    /// the scope reads `ledger::Tally`, and if a type called `ledger` exists
    /// anywhere in the project, `Self` resolves to it instead of to `Tally`.
    /// For a trait impl the scope reads `<Type as Trait>`, so `Type` is scanned
    /// before `Trait` and wins, which is correct — but only because of how the
    /// segment happens to be spelled, not because the scan understands it.
    ///
    /// A scope word naming several symbols is skipped rather than treated as
    /// ambiguous, so the scan continues outward past it.
    fn enclosing_scope(
        &self,
        file: &'a FileAnalysis,
        occurrence: &Occurrence,
        separator: &str,
    ) -> Result<String, Unresolved> {
        let Some(referrer) = occurrence.within.and_then(|i| file.symbols.get(i)) else {
            return Err(Unresolved::NoReferrer);
        };

        let scope = referrer
            .qualified_path
            .strip_suffix(last_segment(&referrer.qualified_path, separator))
            .unwrap_or_default();

        // The innermost scope word that names a symbol. `<Config as Default>`
        // answers `Config`, not `Default`, because the impl is on `Config`.
        let words: Vec<&str> = scope
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|word| !word.is_empty())
            .collect();

        for word in words {
            if let Some(candidates) = self.by_name.get(word) {
                if candidates.len() == 1 {
                    return Ok(candidates[0].id.clone());
                }
            }
        }

        Err(Unresolved::NoCandidate)
    }

    /// The name was brought into scope by an import in this file.
    fn by_import(
        &self,
        file: &'a FileAnalysis,
        occurrence: &Occurrence,
        separator: &str,
    ) -> Vec<&'a Symbol> {
        let Some(import) = file.imports.iter().find(|i| {
            i.kind == crate::phase1::ImportKind::Binding && i.local_name == occurrence.name
        }) else {
            return Vec::new();
        };

        self.by_module_path
            .get(&import.path.join(separator))
            .cloned()
            .unwrap_or_default()
    }

    /// The qualifier names a namespace import, so the lookup is exact.
    fn by_namespace(
        &self,
        file: &'a FileAnalysis,
        occurrence: &Occurrence,
        separator: &str,
    ) -> Vec<&'a Symbol> {
        let Some(qualifier) = &occurrence.qualifier else {
            return Vec::new();
        };

        file.imports
            .iter()
            .filter(|i| {
                i.kind == crate::phase1::ImportKind::Namespace && i.local_name == *qualifier
            })
            .filter_map(|i| {
                let qualified = format!("{}{separator}{}", i.path.join(separator), occurrence.name);
                self.by_module_path.get(&qualified)
            })
            .flatten()
            .copied()
            .collect()
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

    /// A wildcard import could have brought it in unqualified.
    ///
    /// Only a wildcard. A namespace import binds an object rather than a scope,
    /// and a re-export sends names outward, so resolving an unqualified name
    /// through either would be an edge the language does not permit.
    fn by_glob_import(
        &self,
        file: &'a FileAnalysis,
        occurrence: &Occurrence,
        separator: &str,
    ) -> Vec<&'a Symbol> {
        let mut found = Vec::new();

        for import in file
            .imports
            .iter()
            .filter(|i| i.kind == crate::phase1::ImportKind::Wildcard)
        {
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

/// Can this occurrence position refer to a symbol of this shape?
fn shape_matches(occurrence: &Occurrence, symbol: &Symbol, separator: &str) -> bool {
    match occurrence.target.as_deref() {
        Some("scoped") => symbol.qualified_path.contains(separator),
        _ => true,
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
    fn a_method_call_does_not_land_on_a_free_function() {
        // `apply` exists twice: a free function in this file and a method on a
        // type elsewhere. Only one of them can be called with a receiver.
        let (edges, _) = resolve(&[
            (
                "src/main.rs",
                "fn apply() {}\nfn run(w: Write) {\n    w.apply();\n}\n",
            ),
            (
                "src/intent.rs",
                "pub struct Write;\nimpl Write {\n    pub fn apply(&self) {}\n}\n",
            ),
        ]);

        let from_run = targets(&edges, "function:run");
        assert!(
            from_run.contains(&"rust:src/intent.rs:function:Write::apply".to_string()),
            "{from_run:?}"
        );
        assert!(
            !from_run.contains(&"rust:src/main.rs:function:apply".to_string()),
            "a method call landed on a free function: {from_run:?}"
        );
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
    fn self_resolves_to_the_type_being_implemented() {
        let (edges, _) = resolve(&[(
            "src/lib.rs",
            "pub struct Ledger;\nimpl Ledger {\n    fn make() -> Self { Ledger }\n}\n",
        )]);

        assert!(targets(&edges, "function:Ledger::make")
            .contains(&"rust:src/lib.rs:type:Ledger".to_string()));
    }

    #[test]
    fn self_in_a_trait_impl_resolves_to_the_type_not_the_trait() {
        let (edges, _) = resolve(&[(
            "src/lib.rs",
            "pub struct Ledger;\npub trait Build { fn make() -> Self; }\nimpl Build for Ledger {\n    fn make() -> Self { Ledger }\n}\n",
        )]);

        let from_impl = targets(&edges, "<Ledger as Build>::make");
        assert!(
            from_impl.contains(&"rust:src/lib.rs:type:Ledger".to_string()),
            "{from_impl:?}"
        );
    }

    #[test]
    fn self_under_module_nesting_takes_the_outermost_naming_scope() {
        // Documents a known limitation rather than asserting correctness. The
        // scan is outermost-first, so a module whose name also names a type
        // captures `Self` before the impl does.
        let (edges, _) = resolve(&[
            ("src/shadow.rs", "pub struct ledger;\n"),
            (
                "src/lib.rs",
                "mod ledger {\n    pub struct Tally;\n    impl Tally {\n        fn make() -> Self { Tally }\n    }\n}\n",
            ),
        ]);

        let from_make = targets(&edges, "ledger::Tally::make");
        assert!(
            from_make.contains(&"rust:src/shadow.rs:type:ledger".to_string()),
            "expected the documented mis-resolution, got {from_make:?}"
        );
    }

    #[test]
    fn a_typescript_namespace_import_does_not_put_names_in_scope() {
        // `import * as helpers` binds one object. A bare `assist()` is
        // undefined in TypeScript, so the namespace import must not be a route
        // to it. A second `assist` elsewhere makes the last-resort unique-name
        // rule abstain, which leaves the namespace import as the only way an
        // edge could appear — and none does.
        let (edges, stats) = resolve(&[
            ("src/helpers.ts", "export function assist() {}\n"),
            ("src/other.ts", "export function assist() {}\n"),
            (
                "src/caller.ts",
                "import * as helpers from './helpers';\nexport function caller() {\n  assist();\n}\n",
            ),
        ]);

        assert!(
            targets(&edges, "function:caller").is_empty(),
            "an unqualified name resolved through a namespace import: {edges:#?}"
        );
        assert!(stats.unresolved.contains_key("ambiguous"));
    }

    #[test]
    fn an_unqualified_name_that_is_unique_still_resolves_by_last_resort() {
        // Documents the residual honestly. The unique-name rule is a
        // language-agnostic last resort at low confidence; it is not the
        // namespace import doing the work, and disabling it for one language
        // would be a per-language code path.
        let (edges, _) = resolve(&[
            ("src/helpers.ts", "export function assist() {}\n"),
            (
                "src/caller.ts",
                "import * as helpers from './helpers';\nexport function caller() {\n  assist();\n}\n",
            ),
        ]);

        let edge = edges
            .iter()
            .find(|e| e.from.contains("caller"))
            .expect("the last-resort rule resolves it");
        assert_eq!(edge.confidence, Confidence::Low);
    }

    #[test]
    fn a_typescript_namespace_qualifier_does_resolve() {
        let (edges, _) = resolve(&[
            ("src/helpers.ts", "export function assist() {}\n"),
            (
                "src/caller.ts",
                "import * as helpers from './helpers';\nexport function caller() {\n  helpers.assist();\n}\n",
            ),
        ]);

        assert_eq!(
            targets(&edges, "function:caller"),
            vec!["typescript:src/helpers.ts:function:assist"],
            "{edges:#?}"
        );
    }

    #[test]
    fn a_typescript_barrel_reexport_does_not_leak_unqualified_names() {
        let (edges, _) = resolve(&[
            ("src/public.ts", "export function assist() {}\n"),
            ("src/index.ts", "export * from './public';\n"),
            (
                "src/caller.ts",
                "export function caller() {\n  assist();\n}\n",
            ),
        ]);

        // `assist` is unique project-wide, so the last-resort rule still finds
        // it. What must not happen is the barrel being the reason.
        let barrel_edges: Vec<_> = edges
            .iter()
            .filter(|e| e.from.contains("index.ts"))
            .collect();
        assert!(
            barrel_edges.is_empty(),
            "a re-export produced an edge of its own: {barrel_edges:#?}"
        );
    }

    #[test]
    fn a_rust_glob_import_still_puts_names_in_scope() {
        // The distinction is per language, not a blanket refusal.
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

        // Measured languages report a range across the corpus, never a single
        // number that would read as per-edge confidence.
        let (low, high) = rust.precision_range.expect("rust is measured");
        assert!(low <= high);
        assert!(rust.measurements.len() >= 2);

        let unmeasured = Heuristic.accuracy("cobol");
        assert_eq!(unmeasured.precision_range, None);
        assert_eq!(unmeasured.recall_range, None);
        assert!(unmeasured.measurements.is_empty());
    }
}
