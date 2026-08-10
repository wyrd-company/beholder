# Project brief: api-facades-and-lazy-globals

## Purpose

Demonstrate idiomatic API-shape and global-state conventions: prelude globs,
blanket extension traits, sealed traits, newtype and `Deref` wrappers, and
re-export facades that control resolution, implementability, and identity; lazy
global initialization through standard and legacy macro forms; and common API
naming and standard-trait conventions.

## Exclusive directory

`fixtures/rust/projects/api-facades-and-lazy-globals/`

## Difficulty

routine

## Assigned coverage

- **RS-CAN-ECO-005** — a prelude glob, a blanket extension trait, a sealed trait,
  a newtype or `Deref` wrapper, and a re-export facade consumed through facade
  and core paths.
- **RS-CAN-ECO-006** — stable standard lazy state and one pinned legacy macro case,
  referencing the generated hidden behavior through ordinary syntax.
- **RS-CAN-ECO-010** — the no-`get_` getter, `as_`/`to_`/`into_` conversion, and
  standard-trait naming families distinguished from compiler-enforced semantics.

Named counterexamples (a downstream sealed-impl failure, an extension-method
ambiguity, a reentrancy or panic path, a valid non-conforming API) are preserved
as documentation beside their valid demonstrations.

## Declared build contexts

- Default: `x86_64-unknown-linux-gnu`, edition 2024, stable 1.97.1.
- A cross-crate boundary (a local downstream crate) for sealed-trait and facade
  identity effects.

## Dependency needs

One third-party dependency, `lazy_static`, for the legacy macro-generated hidden
wrapper case in RS-CAN-ECO-006. It is pinned, vendored with source location,
license, and notices, and available offline. The facade, sealed-trait, extension,
naming, and standard-library lazy coverage uses no third-party crate. No vendored
crate is shared with another project.

## Generated-source needs

None as tracked source. The legacy lazy macro expands during compilation.

## Planned tests

None.

## Required interfaces

Provide `Taskfile.yml` with `build`, `lint`, and `test` as defined in the ground
rules, and `coverage.md` mapping each assigned identifier to its stable source
location.
