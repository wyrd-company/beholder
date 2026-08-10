# Project brief: serde-and-error-derives

## Purpose

Demonstrate the serialization-style and error-derive ecosystem conventions: how
container, variant, and field helpers generate impls whose external names,
bounds, and defaults differ from Rust identifiers, and how error derives generate
`Display`, `Error`, `source`, and `From` impls that become conversion edges
selected by `?`.

## Exclusive directory

`fixtures/rust/projects/serde-and-error-derives/`

## Difficulty

routine

## Assigned coverage

- **RS-CAN-ECO-001** — serialization-style derives showing rename, skip, default,
  flatten or tag adapters, generic bounds, and generated trait use, contrasted
  with a manual impl.
- **RS-CAN-ECO-002** — error derives generating typed conversions and source chains
  consumed through `?`, contrasted with a manual or type-erased boundary.

Named counterexamples (conflicting helpers, a duplicate source or from field, a
missing conversion) are preserved as documentation beside their valid
demonstrations. Generated external names are documentation-level, not Rust
identifiers.

## Declared build contexts

- Default: `x86_64-unknown-linux-gnu`, edition 2024, stable 1.97.1.
- A dependency-feature-gated context for the derive features.

## Dependency needs

Two third-party dependencies:

- `serde` with its derive feature, for RS-CAN-ECO-001.
- `thiserror`, for RS-CAN-ECO-002.

Each is pinned, vendored with source location, license, and notices, and
available offline. No vendored crate is shared with another project.

## Generated-source needs

None as tracked source. The derive macros expand during compilation.

## Planned tests

None.

## Required interfaces

Provide `Taskfile.yml` with `build`, `lint`, and `test` as defined in the ground
rules, where `build` resolves the vendored dependencies offline, and `coverage.md`
mapping each assigned identifier to its stable source location.
