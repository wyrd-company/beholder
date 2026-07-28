//! Phase 1 — per file.
//!
//! A pure function of one file's repo-relative path and its content, with no
//! knowledge of any other file. Identical input always yields identical output,
//! on any machine, in any checkout location. That is what makes the stored
//! result in [`crate::store`] safe to reuse.

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use tree_sitter::{Node, Parser, Query, QueryCursor, StreamingIterator};

use crate::lang::{self, LanguageDef};
use crate::symbol::{content_fingerprint, symbol_id, Symbol};

/// Everything phase 1 knows about one file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileAnalysis {
    /// Repo-relative, forward-slash separated.
    pub path: String,
    /// Language id, or `None` when no grammar claims the file.
    pub language: Option<String>,
    /// Git blob hash of the analyzed content. The cache key's other half.
    pub content_hash: String,
    /// Highest tier reached for this file.
    pub tier: u8,
    /// Tier 0 indentation density. Reported for every file.
    pub density: f32,
    /// Non-blank line count, as tier 0 counts them.
    pub lines: usize,
    /// Empty when the file did not reach tier 1.
    pub symbols: Vec<Symbol>,
}

/// Analyze one file.
pub fn analyze(path: &str, contents: &str) -> FileAnalysis {
    let density = crate::tier0::density(contents);
    let lines = crate::tier0::indentation_widths(contents).len();
    let content_hash = crate::hash::content_hash(contents.as_bytes());

    let Some(language) = lang::for_path(path) else {
        return FileAnalysis {
            path: path.to_owned(),
            language: None,
            content_hash,
            tier: 0,
            density,
            lines,
            symbols: Vec::new(),
        };
    };

    let symbols = tier1_symbols(language, path, contents);

    FileAnalysis {
        path: path.to_owned(),
        language: Some(language.id.to_owned()),
        content_hash,
        tier: 1,
        density,
        lines,
        symbols,
    }
}

// ---------------------------------------------------------------------------
// Compiled queries
// ---------------------------------------------------------------------------

struct Compiled {
    symbols: Query,
    scopes: Query,
    complexity: Query,
}

fn compiled(language: &LanguageDef) -> &'static Compiled {
    static CACHE: OnceLock<Vec<Compiled>> = OnceLock::new();

    let all = CACHE.get_or_init(|| {
        lang::LANGUAGES
            .iter()
            .map(|l| {
                let grammar = (l.grammar)();
                let compile = |name: &str, source: &str| {
                    Query::new(&grammar, source).unwrap_or_else(|e| {
                        panic!("{}/{name}.scm is not a valid query: {e}", l.id)
                    })
                };
                Compiled {
                    symbols: compile("symbols", l.symbols_query),
                    scopes: compile("scopes", l.scopes_query),
                    complexity: compile("complexity", l.complexity_query),
                }
            })
            .collect()
    });

    let index = lang::LANGUAGES
        .iter()
        .position(|l| l.id == language.id)
        .expect("language came from the table");
    &all[index]
}

/// Value of a `#set!` directive on a pattern.
fn property<'q>(query: &'q Query, pattern: usize, key: &str) -> Option<&'q str> {
    query
        .property_settings(pattern)
        .iter()
        .find(|p| &*p.key == key)
        .and_then(|p| p.value.as_deref())
}

/// Substitute `{capture}` placeholders with captured text.
fn render(format: &str, variables: &HashMap<String, String>) -> String {
    let mut out = String::with_capacity(format.len());
    let mut rest = format;

    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let Some(close) = rest[open..].find('}') else {
            break;
        };
        let key = &rest[open + 1..open + close];
        if let Some(value) = variables.get(key) {
            out.push_str(value);
        }
        rest = &rest[open + close + 1..];
    }

    out.push_str(rest);
    out
}

// ---------------------------------------------------------------------------
// Tier 1
// ---------------------------------------------------------------------------

fn tier1_symbols(language: &'static LanguageDef, path: &str, contents: &str) -> Vec<Symbol> {
    let queries = compiled(language);
    let grammar = (language.grammar)();

    let mut parser = Parser::new();
    if parser.set_language(&grammar).is_err() {
        return Vec::new();
    }
    let Some(tree) = parser.parse(contents, None) else {
        return Vec::new();
    };

    let source = contents.as_bytes();
    let root = tree.root_node();

    let scopes = collect_scopes(&queries.scopes, root, source);
    let mut found = collect_symbols(&queries.symbols, root, source, &scopes, language, path);
    let costs = complexity_by_symbol(&queries.complexity, root, source, &found);

    for (symbol, cost) in found.iter_mut().zip(costs) {
        symbol.symbol.cognitive_complexity = cost;
    }

    finish(found)
}

/// A symbol under construction, still carrying the syntax node it came from.
struct Pending {
    node_id: usize,
    start_byte: usize,
    symbol: Symbol,
}

/// Map every scope node to the qualified-path segment it contributes.
fn collect_scopes(query: &Query, root: Node<'_>, source: &[u8]) -> HashMap<usize, String> {
    let mut scopes = HashMap::new();
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(query, root, source);

    while let Some(m) = matches.next() {
        let mut node = None;
        let mut variables = HashMap::new();

        for capture in m.captures {
            let name = &query.capture_names()[capture.index as usize];
            if *name == "scope" {
                node = Some(capture.node);
            } else {
                variables.insert(
                    (*name).to_owned(),
                    capture.node.utf8_text(source).unwrap_or_default().to_owned(),
                );
            }
        }

        let Some(node) = node else { continue };
        let format = property(query, m.pattern_index, "format").unwrap_or("{name}");
        scopes.insert(node.id(), render(format, &variables));
    }

    scopes
}

fn collect_symbols(
    query: &Query,
    root: Node<'_>,
    source: &[u8],
    scopes: &HashMap<usize, String>,
    language: &LanguageDef,
    path: &str,
) -> Vec<Pending> {
    let mut found = Vec::new();
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(query, root, source);

    while let Some(m) = matches.next() {
        let mut node = None;
        let mut variables = HashMap::new();

        for capture in m.captures {
            let name = &query.capture_names()[capture.index as usize];
            if *name == "symbol" {
                node = Some(capture.node);
            } else {
                variables.insert(
                    (*name).to_owned(),
                    capture.node.utf8_text(source).unwrap_or_default().to_owned(),
                );
            }
        }

        let Some(node) = node else { continue };
        let Some(kind) = property(query, m.pattern_index, "kind") else {
            continue;
        };

        let format = property(query, m.pattern_index, "format").unwrap_or("{name}");
        let name = render(format, &variables);
        let qualified_path = qualify(node, scopes, &name, language.path_separator);

        found.push(Pending {
            node_id: node.id(),
            start_byte: node.start_byte(),
            symbol: Symbol {
                id: String::new(),
                path: path.to_owned(),
                language: language.id.to_owned(),
                kind: kind.to_owned(),
                name,
                qualified_path,
                start_line: node.start_position().row + 1,
                end_line: node.end_position().row + 1,
                cognitive_complexity: 0,
                tier: 1,
                content_fingerprint: content_fingerprint(tokens(node, source)),
            },
        });
    }

    found.sort_by_key(|p| p.start_byte);
    found
}

/// The `(kind, text)` leaf tokens of a subtree, in source order, without
/// comments.
///
/// Leaves are where all the text lives; interior node kinds are implied by the
/// leaf sequence for any given grammar. Comments are dropped so a rewrapped doc
/// comment does not read as a structural change.
fn tokens<'a>(node: Node<'_>, source: &'a [u8]) -> Vec<(&'a str, &'a str)> {
    let mut out = Vec::new();
    collect_tokens(node, source, &mut out);
    out
}

fn collect_tokens<'a>(node: Node<'_>, source: &'a [u8], out: &mut Vec<(&'a str, &'a str)>) {
    let kind = node.kind();

    if kind.contains("comment") {
        return;
    }

    if node.child_count() == 0 {
        if let Ok(text) = node.utf8_text(source) {
            out.push((kind, text));
        }
        return;
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_tokens(child, source, out);
    }
}

/// Join enclosing scope segments, outermost first, with the symbol's own name.
fn qualify(
    node: Node<'_>,
    scopes: &HashMap<usize, String>,
    name: &str,
    separator: &str,
) -> String {
    let mut segments = Vec::new();
    let mut current = node.parent();

    while let Some(ancestor) = current {
        if let Some(segment) = scopes.get(&ancestor.id()) {
            segments.push(segment.as_str());
        }
        current = ancestor.parent();
    }

    segments.reverse();
    segments.push(name);
    segments.join(separator)
}

// ---------------------------------------------------------------------------
// Cognitive complexity
// ---------------------------------------------------------------------------

#[derive(Default)]
struct Contributions {
    increment: HashSet<usize>,
    flat: HashSet<usize>,
    nesting: HashSet<usize>,
    no_increment: HashSet<usize>,
    no_nesting: HashSet<usize>,
}

fn complexity_by_symbol(
    query: &Query,
    root: Node<'_>,
    source: &[u8],
    symbols: &[Pending],
) -> Vec<u32> {
    let mut contributions = Contributions::default();
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(query, root, source);

    while let Some(m) = matches.next() {
        for capture in m.captures {
            let id = capture.node.id();
            match &*query.capture_names()[capture.index as usize] {
                "increment" => contributions.increment.insert(id),
                "increment.flat" => contributions.flat.insert(id),
                "nesting" => contributions.nesting.insert(id),
                "no_increment" => contributions.no_increment.insert(id),
                "no_nesting" => contributions.no_nesting.insert(id),
                _ => false,
            };
        }
    }

    let owner: HashMap<usize, usize> = symbols
        .iter()
        .enumerate()
        .map(|(index, pending)| (pending.node_id, index))
        .collect();

    let mut costs = vec![0u32; symbols.len()];
    accumulate(root, 0, None, &contributions, &owner, &mut costs);
    costs
}

fn accumulate(
    node: Node<'_>,
    nesting: u32,
    owner_index: Option<usize>,
    contributions: &Contributions,
    owner: &HashMap<usize, usize>,
    costs: &mut [u32],
) {
    let id = node.id();

    // A symbol boundary starts a fresh nesting frame, so a score always
    // describes the shape of one symbol rather than where it happens to sit.
    let (nesting, owner_index) = match owner.get(&id) {
        Some(index) => (0, Some(*index)),
        None => (nesting, owner_index),
    };

    if let Some(index) = owner_index {
        if !contributions.no_increment.contains(&id) {
            if contributions.increment.contains(&id) {
                costs[index] += 1 + nesting;
            } else if contributions.flat.contains(&id) {
                costs[index] += 1;
            }
        }
    }

    let child_nesting =
        if contributions.nesting.contains(&id) && !contributions.no_nesting.contains(&id) {
            nesting + 1
        } else {
            nesting
        };

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        accumulate(
            child,
            child_nesting,
            owner_index,
            contributions,
            owner,
            costs,
        );
    }
}

// ---------------------------------------------------------------------------
// Identity
// ---------------------------------------------------------------------------

/// Assign identity, disambiguating symbols that are otherwise identical.
fn finish(pending: Vec<Pending>) -> Vec<Symbol> {
    let mut counts: HashMap<(&str, &str), usize> = HashMap::new();
    for p in &pending {
        *counts
            .entry((p.symbol.kind.as_str(), p.symbol.qualified_path.as_str()))
            .or_default() += 1;
    }

    let duplicated: HashSet<(String, String)> = counts
        .into_iter()
        .filter(|(_, n)| *n > 1)
        .map(|((k, q), _)| (k.to_owned(), q.to_owned()))
        .collect();

    let mut seen: HashMap<(String, String), usize> = HashMap::new();
    let mut symbols = Vec::with_capacity(pending.len());

    for p in pending {
        let key = (p.symbol.kind.clone(), p.symbol.qualified_path.clone());
        let ordinal = if duplicated.contains(&key) {
            let counter = seen.entry(key).or_insert(0);
            *counter += 1;
            Some(*counter)
        } else {
            None
        };

        let mut symbol = p.symbol;
        symbol.id = symbol_id(
            &symbol.language,
            &symbol.path,
            &symbol.kind,
            &symbol.qualified_path,
            ordinal,
        );
        symbols.push(symbol);
    }

    symbols
}

#[cfg(test)]
mod tests {
    use super::*;

    fn analyze_rust(source: &str) -> Vec<Symbol> {
        analyze("src/lib.rs", source).symbols
    }

    fn named(symbols: &[Symbol], qualified: &str) -> Symbol {
        symbols
            .iter()
            .find(|s| s.qualified_path == qualified)
            .unwrap_or_else(|| panic!("no symbol {qualified} in {symbols:#?}"))
            .clone()
    }

    #[test]
    fn files_without_a_grammar_still_reach_tier_zero() {
        let result = analyze("config.yml", "root:\n  a: 1\n  b: 2\n  c: 3\n");
        assert_eq!(result.tier, 0);
        assert_eq!(result.language, None);
        assert!(result.density > 0.0);
        assert!(result.symbols.is_empty());
    }

    #[test]
    fn density_is_reported_for_rust_files_too() {
        let result = analyze("src/lib.rs", "fn a() {\n    let x = 1;\n    let y = 2;\n}\n");
        assert_eq!(result.tier, 1);
        assert!(result.density > 0.0);
    }

    #[test]
    fn extracts_functions_and_types() {
        let symbols = analyze_rust("struct Point;\nenum Shade { Light }\nfn draw() {}\n");

        let kinds: Vec<_> = symbols
            .iter()
            .map(|s| (s.kind.as_str(), s.name.as_str()))
            .collect();
        assert_eq!(
            kinds,
            vec![("type", "Point"), ("type", "Shade"), ("function", "draw")]
        );
    }

    #[test]
    fn line_ranges_are_one_based_and_inclusive() {
        let symbols = analyze_rust("\nfn draw() {\n    ();\n}\n");
        assert_eq!(symbols[0].start_line, 2);
        assert_eq!(symbols[0].end_line, 4);
    }

    #[test]
    fn modules_and_impls_qualify_names() {
        let symbols = analyze_rust(
            r#"
mod render {
    struct Canvas;
    impl Canvas {
        fn draw(&self) {}
    }
}
"#,
        );

        assert!(symbols.iter().any(|s| s.qualified_path == "render::Canvas"));
        assert!(symbols
            .iter()
            .any(|s| s.qualified_path == "render::Canvas::draw"));
    }

    #[test]
    fn methods_sharing_a_name_across_impls_stay_distinct() {
        let symbols = analyze_rust(
            r#"
struct Canvas;
trait Paint { fn draw(&self); }
impl Canvas { fn draw(&self) {} }
impl Paint for Canvas { fn draw(&self) {} }
"#,
        );

        let ids: HashSet<_> = symbols.iter().map(|s| s.id.clone()).collect();
        assert_eq!(ids.len(), symbols.len(), "identities collided: {symbols:#?}");
        assert!(symbols
            .iter()
            .any(|s| s.qualified_path == "<Canvas as Paint>::draw"));
        assert!(symbols.iter().any(|s| s.qualified_path == "Canvas::draw"));
    }

    #[test]
    fn identical_symbols_in_one_file_get_ordinals() {
        let symbols = analyze_rust(
            r#"
#[cfg(unix)]
fn platform() {}
#[cfg(windows)]
fn platform() {}
"#,
        );

        let ids: Vec<_> = symbols.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(
            ids,
            vec![
                "rust:src/lib.rs:function:platform#1",
                "rust:src/lib.rs:function:platform#2"
            ]
        );
    }

    #[test]
    fn closures_are_not_symbols() {
        // A closure has no name a later revision could match on. Its complexity
        // belongs to the function that contains it.
        let symbols = analyze_rust("fn run() { let f = |x: u32| if x > 0 { x } else { 0 }; }\n");
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "run");
    }

    #[test]
    fn nested_functions_are_scoped_by_their_parent() {
        let symbols = analyze_rust("fn outer() {\n    fn inner() {}\n}\n");
        assert!(symbols.iter().any(|s| s.qualified_path == "outer::inner"));
    }

    #[test]
    fn flat_code_scores_zero() {
        let symbols = analyze_rust("fn f(a: u32) -> u32 {\n    let b = a + 1;\n    b\n}\n");
        assert_eq!(symbols[0].cognitive_complexity, 0);
    }

    #[test]
    fn nesting_is_weighted() {
        let flat = analyze_rust("fn f(a: bool, b: bool) {\n    if a {}\n    if b {}\n}\n");
        let nested = analyze_rust("fn f(a: bool, b: bool) {\n    if a {\n        if b {}\n    }\n}\n");

        assert_eq!(flat[0].cognitive_complexity, 2);
        // outer if = 1, inner if = 1 + 1 nesting
        assert_eq!(nested[0].cognitive_complexity, 3);
    }

    #[test]
    fn else_if_is_a_flat_increment() {
        let symbols = analyze_rust(
            "fn f(a: bool, b: bool) {\n    if a {\n    } else if b {\n    } else {\n    }\n}\n",
        );
        // if = 1, else-if branch = 1, else branch = 1
        assert_eq!(symbols[0].cognitive_complexity, 3);
    }

    #[test]
    fn a_boolean_sequence_counts_once() {
        let one = analyze_rust("fn f(a: bool, b: bool, c: bool) {\n    if a && b && c {}\n}\n");
        let two = analyze_rust("fn f(a: bool, b: bool, c: bool) {\n    if a && b || c {}\n}\n");

        assert_eq!(one[0].cognitive_complexity, 2);
        assert_eq!(two[0].cognitive_complexity, 3);
    }

    #[test]
    fn closure_bodies_nest_without_incrementing() {
        let symbols =
            analyze_rust("fn f(v: Vec<u32>) {\n    v.iter().for_each(|x| if *x > 0 {});\n}\n");
        // closure adds a nesting level but no decision point; the if is 1 + 1
        assert_eq!(symbols[0].cognitive_complexity, 2);
    }

    #[test]
    fn complexity_is_attributed_to_the_innermost_symbol() {
        let symbols = analyze_rust(
            "fn outer(a: bool) {\n    if a {}\n    fn inner(b: bool) {\n        if b {}\n    }\n}\n",
        );

        assert_eq!(named(&symbols, "outer").cognitive_complexity, 1);
        assert_eq!(named(&symbols, "outer::inner").cognitive_complexity, 1);
    }

    #[test]
    fn labelled_jumps_increment() {
        let symbols = analyze_rust(
            "fn f() {\n    'outer: loop {\n        loop {\n            break 'outer;\n        }\n    }\n}\n",
        );
        // outer loop 1, inner loop 1+1, labelled break 1
        assert_eq!(symbols[0].cognitive_complexity, 4);
    }

    #[test]
    fn output_is_identical_across_runs() {
        let source = include_str!("phase1.rs");
        assert_eq!(analyze("src/phase1.rs", source), analyze("src/phase1.rs", source));
    }

    #[test]
    fn reformatting_changes_no_identity_and_no_score() {
        let tight = "fn f(a:bool,b:bool){if a{if b{}}}";
        let loose = "fn f(a: bool, b: bool) {\n    if a {\n        if b {}\n    }\n}\n";

        let a = analyze_rust(tight);
        let b = analyze_rust(loose);

        assert_eq!(a[0].id, b[0].id);
        assert_eq!(a[0].cognitive_complexity, b[0].cognitive_complexity);
        assert_eq!(a[0].content_fingerprint, b[0].content_fingerprint);
    }
}
