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
    discriminators: Query,
}

fn compiled(language: &LanguageDef) -> &'static Compiled {
    static CACHE: OnceLock<Vec<Compiled>> = OnceLock::new();

    let all = CACHE.get_or_init(|| {
        lang::LANGUAGES
            .iter()
            .map(|l| {
                let grammar = (l.grammar)();
                let compile = |name: &str, source: &str| {
                    Query::new(&grammar, source)
                        .unwrap_or_else(|e| panic!("{}/{name}.scm is not a valid query: {e}", l.id))
                };
                Compiled {
                    symbols: compile("symbols", l.symbols_query),
                    scopes: compile("scopes", l.scopes_query),
                    complexity: compile("complexity", l.complexity_query),
                    discriminators: compile("discriminators", l.discriminators_query),
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

/// Does every `require.<capture>` property on this pattern hold?
///
/// A query cannot constrain a capture's text on its own, so a pattern declares
/// the constraint and this checks it. That keeps a rule such as "only `cfg`
/// attributes, not every attribute" in the query file rather than in code.
fn requirements_met(query: &Query, pattern: usize, variables: &HashMap<String, String>) -> bool {
    query
        .property_settings(pattern)
        .iter()
        .filter_map(|p| {
            p.key
                .strip_prefix("require.")
                .map(|capture| (capture, p.value.as_deref().unwrap_or_default()))
        })
        .all(|(capture, expected)| variables.get(capture).map(String::as_str) == Some(expected))
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

    let scopes = collect_labels(&queries.scopes, "scope", root, source);
    let discriminators = collect_labels(&queries.discriminators, "discriminator", root, source);
    let mut found = collect_symbols(
        &queries.symbols,
        root,
        source,
        &scopes,
        &discriminators,
        language,
        path,
    );
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

/// Map every node a query anchors on to the text it contributes.
///
/// Scopes and discriminators are the same shape: a node, an optional `format`
/// template over the pattern's other captures, and optional `require.<capture>`
/// constraints.
fn collect_labels(
    query: &Query,
    anchor: &str,
    root: Node<'_>,
    source: &[u8],
) -> HashMap<usize, String> {
    let mut labels = HashMap::new();
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(query, root, source);

    while let Some(m) = matches.next() {
        let mut node = None;
        let mut variables = HashMap::new();

        for capture in m.captures {
            let name = &query.capture_names()[capture.index as usize];
            if *name == anchor {
                node = Some(capture.node);
            } else {
                variables.insert(
                    (*name).to_owned(),
                    capture
                        .node
                        .utf8_text(source)
                        .unwrap_or_default()
                        .to_owned(),
                );
            }
        }

        let Some(node) = node else { continue };
        if !requirements_met(query, m.pattern_index, &variables) {
            continue;
        }

        let format = property(query, m.pattern_index, "format").unwrap_or("{name}");
        labels.insert(node.id(), normalize_whitespace(&render(format, &variables)));
    }

    labels
}

/// Collapse whitespace runs so a reformatted label is the same label.
fn normalize_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Every discriminator attached to a symbol, outermost first.
///
/// Discriminators sit immediately before the symbol they qualify, and they
/// stack. Walking back over the run of them means `#[cfg(a)] #[cfg(b)] fn f`
/// keeps both.
fn discriminator_for(node: Node<'_>, discriminators: &HashMap<usize, String>) -> Option<String> {
    let mut found = Vec::new();
    let mut current = node.prev_named_sibling();

    while let Some(sibling) = current {
        match discriminators.get(&sibling.id()) {
            Some(text) => found.push(text.as_str()),
            None => break,
        }
        current = sibling.prev_named_sibling();
    }

    if found.is_empty() {
        return None;
    }

    found.reverse();
    Some(found.join(" "))
}

#[allow(clippy::too_many_arguments)]
fn collect_symbols(
    query: &Query,
    root: Node<'_>,
    source: &[u8],
    scopes: &HashMap<usize, String>,
    discriminators: &HashMap<usize, String>,
    language: &LanguageDef,
    path: &str,
) -> Vec<Pending> {
    let mut found = Vec::new();
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(query, root, source);

    while let Some(m) = matches.next() {
        let mut node = None;
        let mut name_node = None;
        let mut variables = HashMap::new();

        for capture in m.captures {
            let name = &query.capture_names()[capture.index as usize];
            if *name == "symbol" {
                node = Some(capture.node);
            } else {
                if *name == "name" {
                    name_node = Some(capture.node);
                }
                variables.insert(
                    (*name).to_owned(),
                    capture
                        .node
                        .utf8_text(source)
                        .unwrap_or_default()
                        .to_owned(),
                );
            }
        }

        let Some(node) = node else { continue };
        let Some(kind) = property(query, m.pattern_index, "kind") else {
            continue;
        };
        if !requirements_met(query, m.pattern_index, &variables) {
            continue;
        }

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
                discriminator: discriminator_for(node, discriminators),
                start_line: node.start_position().row + 1,
                end_line: node.end_position().row + 1,
                cognitive_complexity: 0,
                tier: 1,
                content_fingerprint: content_fingerprint(tokens(node, source, None, language)),
                body_fingerprint: content_fingerprint(tokens(node, source, name_node, language)),
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
/// `skip` excludes one subtree, used to leave the symbol's own name out of the
/// body fingerprint so a rename is recognizable as a rename.
fn tokens<'a>(
    node: Node<'_>,
    source: &'a [u8],
    skip: Option<Node<'_>>,
    language: &LanguageDef,
) -> Vec<(&'a str, &'a str)> {
    let mut out = Vec::new();
    collect_tokens(node, source, skip.map(|n| n.id()), language, &mut out);
    out
}

fn collect_tokens<'a>(
    node: Node<'_>,
    source: &'a [u8],
    skip: Option<usize>,
    language: &LanguageDef,
    out: &mut Vec<(&'a str, &'a str)>,
) {
    let kind = node.kind();

    if kind.contains("comment") || skip == Some(node.id()) {
        return;
    }

    if node.child_count() == 0 {
        if let Ok(text) = node.utf8_text(source) {
            out.push((kind, text));
        }
        return;
    }

    let mut cursor = node.walk();
    let children: Vec<Node<'_>> = node.children(&mut cursor).collect();

    for (index, child) in children.iter().enumerate() {
        if is_optional_trailing(kind, *child, &children[index + 1..], language) {
            continue;
        }
        collect_tokens(*child, source, skip, language, out);
    }
}

/// Is this an optional separator, in a list that tolerates one, with nothing
/// meaningful left after it?
///
/// `struct Ledger { entries: Vec<u32> }` and the same declaration with a
/// trailing comma are the same declaration. A formatter picks one; beholder
/// must not read that choice as a change. The container has to be named for
/// that to be true, because the same comma means something in a tuple and in a
/// macro's token tree.
fn is_optional_trailing(
    container: &str,
    node: Node<'_>,
    rest: &[Node<'_>],
    language: &LanguageDef,
) -> bool {
    !node.is_named()
        && !rest.iter().any(|n| n.is_named())
        && language
            .optional_trailing
            .iter()
            .any(|s| s.container == container && s.token == node.kind())
}

/// Join enclosing scope segments, outermost first, with the symbol's own name.
fn qualify(node: Node<'_>, scopes: &HashMap<usize, String>, name: &str, separator: &str) -> String {
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
            match query.capture_names()[capture.index as usize] {
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
    /// Everything that identity is made of before the positional fallback.
    fn key(symbol: &Symbol) -> (String, String, Option<String>) {
        (
            symbol.kind.clone(),
            symbol.qualified_path.clone(),
            symbol.discriminator.clone(),
        )
    }

    let mut counts: HashMap<(String, String, Option<String>), usize> = HashMap::new();
    for p in &pending {
        *counts.entry(key(&p.symbol)).or_default() += 1;
    }

    let duplicated: HashSet<(String, String, Option<String>)> = counts
        .into_iter()
        .filter(|(_, n)| *n > 1)
        .map(|(k, _)| k)
        .collect();

    let mut seen: HashMap<(String, String, Option<String>), usize> = HashMap::new();
    let mut symbols = Vec::with_capacity(pending.len());

    for p in pending {
        let key = key(&p.symbol);
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
            symbol.discriminator.as_deref(),
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
        let result = analyze(
            "src/lib.rs",
            "fn a() {\n    let x = 1;\n    let y = 2;\n}\n",
        );
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
        assert_eq!(
            ids.len(),
            symbols.len(),
            "identities collided: {symbols:#?}"
        );
        assert!(symbols
            .iter()
            .any(|s| s.qualified_path == "<Canvas as Paint>::draw"));
        assert!(symbols.iter().any(|s| s.qualified_path == "Canvas::draw"));
    }

    #[test]
    fn cfg_variants_are_told_apart_by_their_guard() {
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
                "rust:src/lib.rs:function:platform@cfg(unix)",
                "rust:src/lib.rs:function:platform@cfg(windows)"
            ]
        );
    }

    #[test]
    fn deleting_one_cfg_variant_leaves_the_other_identity_alone() {
        // The failure this replaces: with positional ordinals, deleting
        // platform#1 renamed platform#2 to plain platform, so one deletion
        // reported a removal and a rename.
        let both =
            analyze_rust("#[cfg(unix)]\nfn platform() {}\n#[cfg(windows)]\nfn platform() {}\n");
        let one = analyze_rust("#[cfg(windows)]\nfn platform() {}\n");

        let survivor = both
            .iter()
            .find(|s| s.discriminator.as_deref() == Some("cfg(windows)"))
            .unwrap();

        assert_eq!(one.len(), 1);
        assert_eq!(one[0].id, survivor.id);
    }

    #[test]
    fn stacked_guards_all_reach_identity() {
        let symbols = analyze_rust("#[cfg(unix)]\n#[cfg(feature = \"x\")]\nfn platform() {}\n");
        assert_eq!(
            symbols[0].discriminator.as_deref(),
            Some("cfg(unix) cfg(feature = \"x\")")
        );
    }

    #[test]
    fn attributes_that_say_nothing_about_identity_are_not_discriminators() {
        // #[inline] and #[derive] describe behaviour, not which symbol this is.
        // Treating them as identity would make adding one look like a delete.
        let bare = analyze_rust("fn platform() {}\n");
        let annotated = analyze_rust("#[inline]\nfn platform() {}\n");

        assert_eq!(bare[0].id, annotated[0].id);
        assert_eq!(annotated[0].discriminator, None);
    }

    #[test]
    fn truly_indistinguishable_symbols_still_fall_back_to_ordinals() {
        // The documented residual case: nothing declared tells these apart.
        let symbols = analyze_rust("fn platform() {}\nfn platform() {}\n");
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
    fn a_trailing_comma_in_a_macro_is_not_formatting() {
        // f!(x) and f!(x,) are different inputs to whatever the macro does, so
        // the optional-trailing rule must not reach inside a token tree.
        let one = analyze_rust("fn f() { assert_matches!(x) }\n");
        let two = analyze_rust("fn f() { assert_matches!(x,) }\n");

        assert_ne!(one[0].content_fingerprint, two[0].content_fingerprint);
    }

    #[test]
    fn a_trailing_comma_in_a_tuple_is_not_formatting() {
        // (a,) is a one-tuple; (a) is a parenthesized expression.
        let tuple = analyze_rust("fn f() -> (u32,) { (1,) }\n");
        let paren = analyze_rust("fn f() -> (u32,) { (1) }\n");

        assert_ne!(tuple[0].content_fingerprint, paren[0].content_fingerprint);
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
        let nested =
            analyze_rust("fn f(a: bool, b: bool) {\n    if a {\n        if b {}\n    }\n}\n");

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
        assert_eq!(
            analyze("src/phase1.rs", source),
            analyze("src/phase1.rs", source)
        );
    }

    #[test]
    fn a_trailing_comma_is_not_a_change() {
        // rustfmt adds one when it breaks a list across lines. Reading that as
        // a structural edit would make every reformat noisy.
        let tight = analyze_rust("struct Ledger { entries: u32 }\n");
        let formatted = analyze_rust("struct Ledger {\n    entries: u32,\n}\n");

        assert_eq!(
            tight[0].content_fingerprint,
            formatted[0].content_fingerprint
        );
    }

    #[test]
    fn a_semicolon_is_still_part_of_the_fingerprint() {
        // A trailing expression and a statement are different code, so the
        // optional-trailing rule must not reach that far.
        let tail = analyze_rust("fn f() -> u32 { g() }\n");
        let statement = analyze_rust("fn f() -> u32 { g(); }\n");

        assert_ne!(
            tail[0].content_fingerprint,
            statement[0].content_fingerprint
        );
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
