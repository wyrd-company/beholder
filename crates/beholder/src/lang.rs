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
    /// Query selecting identity discriminators. See `queries/<id>/discriminators.scm`.
    pub discriminators_query: &'static str,
    /// Query selecting identifier occurrences. See `queries/<id>/references.scm`.
    pub references_query: &'static str,
    /// Query selecting import declarations. See `queries/<id>/imports.scm`.
    pub imports_query: &'static str,
    /// How an import tree is spelled.
    pub import_syntax: ImportSyntax,
    /// How a file path becomes a module path.
    pub module_path: ModulePathRules,
    /// Separator between qualified-path segments.
    pub path_separator: &'static str,
    /// Lists whose trailing separator is a formatter's choice.
    pub optional_trailing: &'static [TrailingSeparator],
}

/// How a language spells an import tree.
///
/// Expanding `use a::{b, c as d, e::*}` into individual bindings is one
/// algorithm; only the punctuation differs between languages. Declaring the
/// punctuation keeps the algorithm generic.
pub struct ImportSyntax {
    /// Between path segments.
    pub separator: &'static str,
    pub group_open: char,
    pub group_close: char,
    /// Between items inside a group.
    pub item_separator: char,
    /// Introduces a local alias, as in `use a::b as c`.
    pub alias_keyword: &'static str,
    /// Imports everything under a path.
    pub glob: &'static str,
    /// Refers to the path itself rather than a child, as in `use a::{self, b}`.
    pub self_segment: &'static str,
    /// Leading segments that mean "somewhere in this project" rather than a
    /// dependency, and are dropped before matching.
    pub crate_roots: &'static [&'static str],
}

/// How a repo-relative file path becomes a module path.
pub struct ModulePathRules {
    /// Leading directory segments that name no module. Everything up to and
    /// including the last of these is dropped.
    pub strip_prefixes: &'static [&'static str],
    /// File stems that name their containing directory rather than a module of
    /// their own.
    pub root_stems: &'static [&'static str],
}

impl ModulePathRules {
    /// The module path a file contributes, as segments.
    ///
    /// `crates/core/src/git.rs` is `git`; `crates/core/src/lib.rs` is the crate
    /// root and contributes nothing; `src/a/mod.rs` is `a`.
    pub fn segments(&self, path: &str) -> Vec<String> {
        let without_extension = path.rsplit_once('.').map_or(path, |(stem, _)| stem);
        let mut segments: Vec<&str> = without_extension.split('/').collect();

        if let Some(last) = segments
            .iter()
            .rposition(|segment| self.strip_prefixes.contains(segment))
        {
            segments.drain(..=last);
        }

        if segments
            .last()
            .is_some_and(|stem| self.root_stems.contains(stem))
        {
            segments.pop();
        }

        segments.into_iter().map(str::to_owned).collect()
    }
}

/// A separator a formatter may add or drop at the end of one kind of list
/// without changing meaning.
///
/// Naming the container is the whole point. A trailing comma is optional in a
/// field list and in an argument list, but it is load-bearing in a Rust tuple —
/// `(a,)` is a one-tuple and `(a)` is a parenthesized expression — and inside a
/// macro's token tree, where `f!(x)` and `f!(x,)` are two different inputs to
/// whatever that macro does. A rule that only knew about the token would erase
/// those differences.
pub struct TrailingSeparator {
    /// Node kind of the list that holds the separator.
    pub container: &'static str,
    /// The separator token itself.
    pub token: &'static str,
}

/// Rust lists that tolerate a trailing comma.
///
/// Deliberately absent: `token_tree`, `tuple_expression`, `tuple_type` and
/// `tuple_pattern`, where a trailing comma carries meaning.
const RUST_OPTIONAL_TRAILING: &[TrailingSeparator] = &[
    TrailingSeparator {
        container: "arguments",
        token: ",",
    },
    TrailingSeparator {
        container: "array_expression",
        token: ",",
    },
    TrailingSeparator {
        container: "closure_parameters",
        token: ",",
    },
    TrailingSeparator {
        container: "enum_variant_list",
        token: ",",
    },
    TrailingSeparator {
        container: "field_declaration_list",
        token: ",",
    },
    TrailingSeparator {
        container: "field_initializer_list",
        token: ",",
    },
    TrailingSeparator {
        container: "match_block",
        token: ",",
    },
    TrailingSeparator {
        container: "ordered_field_declaration_list",
        token: ",",
    },
    TrailingSeparator {
        container: "parameters",
        token: ",",
    },
    TrailingSeparator {
        container: "slice_pattern",
        token: ",",
    },
    TrailingSeparator {
        container: "struct_pattern",
        token: ",",
    },
    TrailingSeparator {
        container: "tuple_struct_pattern",
        token: ",",
    },
    TrailingSeparator {
        container: "type_arguments",
        token: ",",
    },
    TrailingSeparator {
        container: "type_parameters",
        token: ",",
    },
    TrailingSeparator {
        container: "use_list",
        token: ",",
    },
    TrailingSeparator {
        container: "where_clause",
        token: ",",
    },
];

/// Every language beholder can reach tier 1 for.
pub static LANGUAGES: &[LanguageDef] = &[LanguageDef {
    id: "rust",
    extensions: &["rs"],
    grammar: || tree_sitter_rust::LANGUAGE.into(),
    symbols_query: include_str!("../queries/rust/symbols.scm"),
    scopes_query: include_str!("../queries/rust/scopes.scm"),
    complexity_query: include_str!("../queries/rust/complexity.scm"),
    discriminators_query: include_str!("../queries/rust/discriminators.scm"),
    references_query: include_str!("../queries/rust/references.scm"),
    imports_query: include_str!("../queries/rust/imports.scm"),
    import_syntax: ImportSyntax {
        separator: "::",
        group_open: '{',
        group_close: '}',
        item_separator: ',',
        alias_keyword: "as",
        glob: "*",
        self_segment: "self",
        crate_roots: &["crate", "self", "super"],
    },
    module_path: ModulePathRules {
        strip_prefixes: &["src"],
        root_stems: &["lib", "main", "mod"],
    },
    path_separator: "::",
    optional_trailing: RUST_OPTIONAL_TRAILING,
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
        for separator in lang.optional_trailing {
            material.push_str(separator.container);
            material.push('\0');
            material.push_str(separator.token);
            material.push('\0');
        }
        material.push_str(&lang.extensions.join(","));
        material.push('\0');
        material.push_str(lang.symbols_query);
        material.push_str(lang.scopes_query);
        material.push_str(lang.complexity_query);
        material.push_str(lang.discriminators_query);
        material.push_str(lang.references_query);
        material.push_str(lang.imports_query);
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
                ("discriminators", lang.discriminators_query),
                ("references", lang.references_query),
                ("imports", lang.imports_query),
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
    fn module_paths_come_from_file_paths() {
        let rules = &by_id("rust").unwrap().module_path;

        assert_eq!(rules.segments("crates/core/src/git.rs"), vec!["git"]);
        assert!(rules.segments("crates/core/src/lib.rs").is_empty());
        assert!(rules.segments("src/main.rs").is_empty());
        assert_eq!(rules.segments("src/a/mod.rs"), vec!["a"]);
        assert_eq!(rules.segments("src/a/b.rs"), vec!["a", "b"]);
        // The last `src` wins, so a crate nested under another path still works.
        assert_eq!(rules.segments("a/src/b/src/c.rs"), vec!["c"]);
    }

    #[test]
    fn fingerprint_is_stable() {
        assert_eq!(table_fingerprint(), table_fingerprint());
    }
}
