//! Symbol records and symbol identity.
//!
//! Identity is the load-bearing property of the whole index. It must survive a
//! reformat, a comment edit, and a line shift, and it must never encode a line
//! number, an absolute path, or anything about the machine that produced it.

use serde::{Deserialize, Serialize};

/// What a symbol is. Kinds come from the language table, not from crate code.
pub type SymbolKind = String;

/// One function or type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Symbol {
    /// Stable identity. See [`symbol_id`].
    pub id: String,
    /// Repo-relative, forward-slash separated.
    pub path: String,
    pub language: String,
    pub kind: SymbolKind,
    /// Declared name, without scope.
    pub name: String,
    /// Scope segments plus name, joined by the language's separator.
    pub qualified_path: String,
    /// Declared metadata that tells this symbol apart from another with the same
    /// qualified path — a `#[cfg]` guard, for instance. Part of identity
    /// whenever it is present. See [`symbol_id`].
    pub discriminator: Option<String>,
    /// 1-based, inclusive.
    pub start_line: usize,
    /// 0-based byte offset within [`Symbol::start_line`].
    #[serde(default)]
    pub start_column: usize,
    /// 1-based, inclusive.
    pub end_line: usize,
    /// 0-based byte offset within [`Symbol::end_line`], exclusive.
    #[serde(default)]
    pub end_column: usize,
    /// Where the symbol's own name token is written.
    ///
    /// This is what identifies a declaration to an external index: two
    /// declarations can share a line, and a nested one sits inside another's
    /// span, so only the name token distinguishes them.
    #[serde(default)]
    pub declaration: Option<Span>,
    pub cognitive_complexity: u32,
    /// The tier that produced this record.
    pub tier: u8,
    /// Hash of the symbol's token stream. Used to tell a moved or renamed
    /// symbol from a genuinely new one, and to tell a structural edit from a
    /// reformat. Never part of identity. See [`content_fingerprint`].
    pub content_fingerprint: String,
    /// As [`Symbol::content_fingerprint`], but with the symbol's own name left
    /// out. Two revisions of one symbol share this across a rename, which is
    /// what lets a rename be reported as a rename rather than as a delete and
    /// an unrelated add.
    pub body_fingerprint: String,
}

/// A single-line span of source, as an external index spells one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    /// 1-based.
    pub line: usize,
    /// 0-based byte offset within the line.
    pub start_column: usize,
    /// 0-based byte offset within the line, exclusive.
    pub end_column: usize,
}

/// Build a symbol's stable identity.
///
/// Two things can distinguish symbols that share a qualified path, and the order
/// matters.
///
/// `discriminator` is declared, content-derived metadata — a `#[cfg]` guard, a
/// signature. It is applied whenever it is present, never only when a duplicate
/// currently exists. That distinction is the whole point: an identity that
/// appears only in the presence of a twin is an identity that rewrites itself
/// the moment the twin is deleted.
///
/// `ordinal` is the fallback for symbols that remain indistinguishable after
/// that, and it is positional. Two symbols with the same qualified path, the
/// same kind and the same discriminator in one file will renumber if one is
/// removed. Languages avoid that by declaring a discriminator that actually
/// tells their duplicates apart; see `queries/<id>/discriminators.scm`.
pub fn symbol_id(
    language: &str,
    path: &str,
    kind: &str,
    qualified_path: &str,
    discriminator: Option<&str>,
    ordinal: Option<usize>,
) -> String {
    let mut id = format!("{language}:{path}:{kind}:{qualified_path}");

    if let Some(discriminator) = discriminator {
        id.push('@');
        id.push_str(discriminator);
    }

    if let Some(ordinal) = ordinal {
        id.push('#');
        id.push_str(&ordinal.to_string());
    }

    id
}

/// Hash of a symbol's token stream.
///
/// Whitespace never reaches this function and comments are excluded by the
/// caller, so two spellings of the same code produce the same fingerprint. That
/// is what makes a reformat visibly not a change. It also means beholder's
/// notion of "changed" is structural: a comment-only edit is not a change.
///
/// Tokens are supplied as `(kind, text)` pairs in source order.
pub fn content_fingerprint<'a>(tokens: impl IntoIterator<Item = (&'a str, &'a str)>) -> String {
    let mut material = String::new();

    for (kind, text) in tokens {
        material.push_str(kind);
        material.push('\u{0}');
        material.push_str(text);
        material.push('\u{1}');
    }

    crate::hash::hex_sha256(material.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_carries_no_line_number_or_absolute_path() {
        let id = symbol_id(
            "rust",
            "crates/a/src/lib.rs",
            "function",
            "Foo::bar",
            None,
            None,
        );
        assert_eq!(id, "rust:crates/a/src/lib.rs:function:Foo::bar");
        assert!(!id.starts_with('/'));
        assert!(!id.chars().any(|c| c.is_ascii_digit()));
    }

    #[test]
    fn ordinal_only_appears_when_needed() {
        assert!(!symbol_id("rust", "a.rs", "function", "f", None, None).contains('#'));
        assert!(symbol_id("rust", "a.rs", "function", "f", None, Some(2)).ends_with("#2"));
    }

    #[test]
    fn a_discriminator_applies_whether_or_not_a_twin_exists() {
        // The identity of a cfg-guarded symbol must not depend on how many
        // other variants of it happen to be in the file today.
        let guarded = symbol_id("rust", "a.rs", "function", "f", Some("cfg(unix)"), None);
        assert_eq!(guarded, "rust:a.rs:function:f@cfg(unix)");
        assert_ne!(
            guarded,
            symbol_id("rust", "a.rs", "function", "f", None, None)
        );
    }

    #[test]
    fn fingerprint_depends_on_the_token_stream() {
        let a = [("identifier", "g"), ("(", "("), (")", ")")];
        let b = [("identifier", "h"), ("(", "("), (")", ")")];

        assert_eq!(content_fingerprint(a), content_fingerprint(a));
        assert_ne!(content_fingerprint(a), content_fingerprint(b));
    }

    #[test]
    fn fingerprint_separates_kind_from_text() {
        // Concatenation without separators would let ("ab", "c") collide with
        // ("a", "bc").
        assert_ne!(
            content_fingerprint([("ab", "c")]),
            content_fingerprint([("a", "bc")])
        );
    }
}
