//! The language metadata table.
//!
//! Language support is data. A language is an entry in [`LANGUAGES`] plus a
//! directory of tree-sitter query files. Nothing in this crate branches on which
//! language it is looking at, and no language has a code path of its own.
//!
//! See `docs/adding-a-language.md` for the exact set of files a new language
//! contributes.

use tree_sitter::Language;

/// One language's metadata.
pub struct LanguageDef {
    /// Stable identifier used in output and in percentile bucketing.
    pub id: &'static str,
    /// File extensions, without the leading dot.
    pub extensions: &'static [&'static str],
    /// The tree-sitter grammar.
    pub grammar: fn() -> Language,
    /// Query selecting symbols. See `queries/<id>/symbols.scm`.
    pub symbols_query: &'static str,
    /// Query selecting qualified-path segments. See `queries/<id>/scopes.scm`.
    pub scopes_query: &'static str,
    /// Query selecting complexity contributions. See `queries/<id>/complexity.scm`.
    pub complexity_query: &'static str,
    /// Separator between qualified-path segments.
    pub path_separator: &'static str,
    /// Punctuation a formatter may add or drop at the end of a list without
    /// changing meaning. Excluded from fingerprints so a reformat that adds a
    /// trailing comma is not mistaken for a structural edit.
    pub optional_trailing_tokens: &'static [&'static str],
}

/// Every language beholder can reach tier 1 for.
pub static LANGUAGES: &[LanguageDef] = &[LanguageDef {
    id: "rust",
    extensions: &["rs"],
    grammar: || tree_sitter_rust::LANGUAGE.into(),
    symbols_query: include_str!("../queries/rust/symbols.scm"),
    scopes_query: include_str!("../queries/rust/scopes.scm"),
    complexity_query: include_str!("../queries/rust/complexity.scm"),
    path_separator: "::",
    optional_trailing_tokens: &[","],
}];

/// The language for a repo-relative path, if any grammar claims its extension.
pub fn for_path(path: &str) -> Option<&'static LanguageDef> {
    let extension = std::path::Path::new(path).extension()?.to_str()?;
    LANGUAGES
        .iter()
        .find(|lang| lang.extensions.contains(&extension))
}

/// The language with this identifier.
pub fn by_id(id: &str) -> Option<&'static LanguageDef> {
    LANGUAGES.iter().find(|lang| lang.id == id)
}

/// Identity of the language table and its queries.
///
/// Changing a query changes analysis output, so it must invalidate stored
/// results the same way a tool version bump does.
pub fn table_fingerprint() -> String {
    let mut material = String::new();
    for lang in LANGUAGES {
        material.push_str(lang.id);
        material.push('\0');
        material.push_str(lang.path_separator);
        material.push('\0');
        material.push_str(&lang.optional_trailing_tokens.join(","));
        material.push('\0');
        material.push_str(&lang.extensions.join(","));
        material.push('\0');
        material.push_str(lang.symbols_query);
        material.push_str(lang.scopes_query);
        material.push_str(lang.complexity_query);
        material.push('\0');
    }
    crate::hash::hex_sha256(material.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_query_compiles_against_its_grammar() {
        for lang in LANGUAGES {
            let grammar = (lang.grammar)();
            for (name, source) in [
                ("symbols", lang.symbols_query),
                ("scopes", lang.scopes_query),
                ("complexity", lang.complexity_query),
            ] {
                tree_sitter::Query::new(&grammar, source)
                    .unwrap_or_else(|e| panic!("{}/{name}.scm failed to compile: {e}", lang.id));
            }
        }
    }

    #[test]
    fn extensions_are_claimed_by_exactly_one_language() {
        let mut seen = Vec::new();
        for lang in LANGUAGES {
            for ext in lang.extensions {
                assert!(!seen.contains(ext), "extension {ext} claimed twice");
                seen.push(ext);
            }
        }
    }

    #[test]
    fn paths_map_to_languages_by_extension() {
        assert_eq!(for_path("src/main.rs").map(|l| l.id), Some("rust"));
        assert_eq!(for_path("README.md").map(|l| l.id), None);
        assert_eq!(for_path("Makefile").map(|l| l.id), None);
    }

    #[test]
    fn fingerprint_is_stable() {
        assert_eq!(table_fingerprint(), table_fingerprint());
    }
}
