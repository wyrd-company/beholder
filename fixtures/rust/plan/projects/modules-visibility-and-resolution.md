# Project brief: modules-visibility-and-resolution

## Purpose

Demonstrate how Rust names, mounts, imports, and hides entities: module
declaration and file mapping; a single file mounted more than once; the item,
value, macro, lifetime, and label namespaces; path roots and qualifiers; `use`
forms and import scope; glob ambiguity and shadowing; re-exports and multiple
public paths; restricted and effective visibility; field, variant, and item
visibility; the extern prelude and `extern crate`; the standard and language
preludes; item and local scopes; raw and Unicode identifiers; and intra-doc link
resolution.

## Exclusive directory

`fixtures/rust/projects/modules-visibility-and-resolution/`

## Difficulty

complex

## Assigned coverage

- **RS-CAN-MOD-001** — inline, `name.rs`, `name/mod.rs`, child-directory, and
  `#[path]` modules with nested children.
- **RS-CAN-MOD-002** — one file mounted at two module paths and in two crate roles
  producing distinct entities.
- **RS-CAN-MOD-003** — the type, value, macro, lifetime, and label namespaces
  resolved from shared spellings.
- **RS-CAN-MOD-004** — `crate`, `self`, `super`, leading `::`, `Self`, and
  extern-prelude path roots, including a local module named like a dependency.
- **RS-CAN-MOD-005** — `use` groups, aliases, `self`, glob, and underscore trait
  and macro imports, and child non-inheritance.
- **RS-CAN-MOD-006** — glob ambiguity, shadowing, and public glob re-export.
- **RS-CAN-MOD-007** — `pub use` re-exports, renames, glob exports, and a
  re-exported dependency type.
- **RS-CAN-MOD-008** — private, `pub(self)`, `pub(super)`, `pub(crate)`,
  `pub(in ...)`, and `pub` boundaries across child, sibling, parent, same-crate,
  and downstream-crate access.
- **RS-CAN-MOD-009** — field, constructor, variant, trait-item, and impl-item
  visibility, including cross-crate effects.
- **RS-CAN-MOD-010** — the extern prelude, direct nameability, an unnameable
  transitive crate, `extern crate alloc`, a renamed extern crate, and
  `extern crate self as`.
- **RS-CAN-MOD-011** — standard, core, extern, macro-use, and language preludes,
  an edition addition, a local shadow, and a no-implicit-prelude module.
- **RS-CAN-MOD-012** — item and local scopes, forward reference, shadowing with
  a changed type, and nested items versus closure capture.
- **RS-CAN-MOD-013** — raw identifiers for keyword spellings and a non-ASCII
  identifier.
- **RS-CAN-MOD-014** — intra-doc links to an item, method, `Self` item, re-export,
  and namespace-disambiguated entities.

Named counterexamples (both candidate module files, a missing file, an
undeclared file, a duplicate binding, an insufficient re-export visibility, a
too-many-`super` path, an illegal underscore import, a broken doc link) are
preserved as documentation beside their valid demonstrations.

## Declared build contexts

- Default: `x86_64-unknown-linux-gnu`, edition 2024, stable 1.97.1.
- Editions 2015 and 2018 for legacy path grammar, `macro_use`, and leading-`::`
  differences.
- `cfg`-selected module path alternatives on the host.
- Local crates providing extern-prelude nameability, a re-exported dependency
  type, an unnameable transitive crate, and a transitive version skew.
- Documentation builds for intra-doc link resolution.

## Dependency needs

No third-party dependency. Extern-prelude, re-exported-dependency, transitive,
and version-skew coverage uses locally authored crates, including local crates
vendored at two versions for the transitive-skew variant, resolved offline.

## Generated-source needs

None.

## Planned tests

None. Intra-doc link coverage is created by documentation comments in source and
validated by `lint` and documentation builds, not by test source.

## Required interfaces

Provide `Taskfile.yml` with `build`, `lint`, and `test` as defined in the ground
rules, and `coverage.md` mapping each assigned identifier to its stable source
location.
