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
    path_separator: ".",
    optional_trailing_tokens: &[","],
}
```

`optional_trailing_tokens` names punctuation the language's formatter may add or
drop at the end of a list. Those tokens are left out of fingerprints so a
reformat is not read as a structural edit.

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
   kept distinct, flat versus nested complexity, and a reformat changing nothing.
3. Run the identity gate against a real repository in that language:

   ```bash
   task audit:identity REPO=/path/to/repo
   ```

   Phantoms must be zero. A phantom is a symbol change reported in a file that
   did not change, which means identity moved when the code did not.
