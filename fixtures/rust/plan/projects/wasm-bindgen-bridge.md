# Project brief: wasm-bindgen-bridge

## Purpose

Demonstrate a target-gated Wasm bridge generator: how a bridge macro creates a
foreign-facing surface and glue distinct from Rust `pub` and plain C exports,
producing exported entities, imported foreign entities, and link behavior when
built for a Wasm target.

## Exclusive directory

`fixtures/rust/projects/wasm-bindgen-bridge/`

## Difficulty

complex

## Assigned coverage

- **RS-CAN-ECO-008** — one target-gated Wasm bridge with generated glue, exported
  Rust-to-foreign surface, imported foreign items, and ownership and ABI
  adaptation, built for `wasm32-unknown-unknown`.

Named counterexamples (a non-installed alternative Wasm ABI or interface
generator, a companion foreign artifact produced only by a downstream tool) are
preserved as documentation beside the valid demonstration. Generated
foreign-binding details are documentation-level, not Rust identifiers.

## Declared build contexts

- Default: `x86_64-unknown-linux-gnu`, edition 2024, stable 1.97.1, for host
  compilation, `lint`, and `test`.
- `wasm32-unknown-unknown` (installed in the rustup stable toolchain) as the
  bridge target that `build` exercises.

## Dependency needs

One third-party dependency, `wasm-bindgen`, for the Wasm bridge generator in
RS-CAN-ECO-008. It is pinned, vendored with source location, license, notices,
and offline metadata, including any transitive crates required for an offline
build. No vendored crate is shared with another project. The downstream
JavaScript-binding CLI is not part of the required tasks, so `build`, `lint`, and
`test` remain offline and hermetic.

## Generated-source needs

None as tracked source. The bridge macro expands during compilation.

## Planned tests

None.

## Required interfaces

Provide `Taskfile.yml` with `build`, `lint`, and `test` as defined in the ground
rules, where `build` compiles the crate for `wasm32-unknown-unknown` offline, and
`coverage.md` mapping the assigned identifier to its stable source location.
