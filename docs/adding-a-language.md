# Adding a language

Language support is data. Adding one adds files and one table entry. It adds no
Rust code path, no `match` on a language id, and no branch anywhere in the
crate.

## What a language contributes

Four files and one dependency.

| File | What it does |
| --- | --- |
| `crates/beholder/queries/<id>/symbols.scm` | Selects functions and types, and sets each one's `kind` |
| `crates/beholder/queries/<id>/scopes.scm` | Selects the nodes that contribute qualified-path segments |
| `crates/beholder/queries/<id>/complexity.scm` | Selects the nodes that increment or nest cognitive complexity |
| `crates/beholder/queries/<id>/discriminators.scm` | Selects declared metadata that tells same-named symbols apart |
| `crates/beholder/src/lang.rs` | One `LanguageDef` appended to `LANGUAGES` |
| `crates/beholder/Cargo.toml` | The grammar crate, e.g. `tree-sitter-go` |

If a language needs anything else, that is a design failure. Report it rather
than absorbing it into crate code.

## The table entry

```rust
LanguageDef {
    id: "go",
    extensions: &["go"],
    grammar: || tree_sitter_go::LANGUAGE.into(),
    symbols_query: include_str!("../queries/go/symbols.scm"),
    scopes_query: include_str!("../queries/go/scopes.scm"),
    complexity_query: include_str!("../queries/go/complexity.scm"),
    discriminators_query: include_str!("../queries/go/discriminators.scm"),
    path_separator: ".",
    optional_trailing: GO_OPTIONAL_TRAILING,
}
```

`optional_trailing` names the lists whose trailing separator the language's
formatter may add or drop. Those separators are left out of fingerprints so a
reformat is not read as a structural edit.

Name the container, never just the token. A trailing comma is a formatter's
choice in a field list and in an argument list; inside a macro's token tree it is
part of the macro's input, and in a Rust tuple it is the difference between
`(a,)` and `(a)`. A rule that only knew the token would erase real edits.

## symbols.scm

Every pattern captures the whole symbol as `@symbol` and sets `kind`. Any other
capture is a template variable for the optional `format` property, which
defaults to `{name}`.

```scheme
((function_declaration name: (identifier) @name) @symbol
 (#set! kind "function"))
```

Capture the symbol's own name as `@name` where the language has one. That is
what lets a rename be reported as a rename instead of a delete and an add.

Do not emit anonymous symbols such as closures. They have no name a later
revision can match on, so they generate phantom add/delete pairs. Their
complexity belongs to the symbol that contains them, which happens automatically.

## scopes.scm

A scope node contributes one qualified-path segment to every symbol beneath it.
Use `format` when a segment needs more than one capture.

```scheme
((method_declaration receiver: (_) @receiver) @scope
 (#set! format "({receiver})"))
```

Spell out anything that distinguishes two same-named symbols on the same type.
Rust writes `<Type as Trait>::method` for exactly that reason.

## discriminators.scm

Two symbols can share a qualified path and still be two different symbols. Rust's
case is conditional compilation: `#[cfg(unix)] fn platform` and
`#[cfg(windows)] fn platform` are both `platform`. Other languages have their own
— an overload set is told apart by its signature.

A discriminator is declared, content-derived metadata that tells them apart. It
joins identity whenever it is present, never only when a duplicate happens to
exist. That distinction is the whole point. An identity that appears only in the
presence of a twin rewrites itself the moment the twin is deleted, which reports
one deletion as a removal plus a spurious rename.

```scheme
((attribute_item (attribute (identifier) @guard) @text) @discriminator
 (#set! require.guard "cfg")
 (#set! format "{text}"))
```

`require.<capture>` constrains a capture's text, because a tree-sitter query
cannot do that on its own. Use it to select only the metadata that says *which
symbol this is*. `#[inline]` and `#[derive(Debug)]` describe behaviour, so making
them part of identity would turn adding one into a delete and an add.

Discriminators attach to the symbol that follows them, and they stack.

Symbols that nothing declared can tell apart fall back to a positional ordinal.
That fallback is unstable under deletion of a sibling, which is exactly why a
language should declare a real discriminator instead of relying on it.

## complexity.scm

Five capture classes, resolved per node with suppression winning:

| Capture | Effect |
| --- | --- |
| `@increment` | +1, plus the current nesting level |
| `@increment.flat` | +1, ignoring nesting |
| `@nesting` | descendants are one level deeper |
| `@no_increment` | suppresses any increment on this node |
| `@no_nesting` | suppresses any nesting on this node |

Nesting resets to zero at every symbol boundary, so a score always describes the
shape of one symbol.

The suppression captures are how the awkward cases stay in data. An `else if` is
one decision point, not two, so the `if` inside the `else` is suppressed. A
sequence of the same boolean operator counts once, so an operator directly under
the same operator is suppressed.

## Proving it

Three checks, in order.

1. `cargo test -p beholder lang::` compiles every query against its grammar.
2. Write tier 1 tests beside the Rust ones in `crates/beholder/src/phase1.rs`
   covering: symbol kinds, line ranges, qualified paths, two same-named symbols
   kept distinct, deleting one of them leaving the other's identity alone, flat
   versus nested complexity, a reformat changing nothing, and a trailing
   separator that is *not* formatting still counting as a change.
3. Run the identity gate against a real repository in that language:

   ```bash
   task audit:identity REPO=/path/to/repo
   ```

   Phantoms must be zero. A phantom is a symbol change reported in a file that
   did not change, which means identity moved when the code did not.
