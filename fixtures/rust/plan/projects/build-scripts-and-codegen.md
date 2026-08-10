# Project brief: build-scripts-and-codegen

## Purpose

Demonstrate the build-script protocol and the entities it creates: emitted cfg,
environment, and link directives; Rust and data generated into `OUT_DIR` and
brought back into handwritten source through the include family; native library
ownership through `links` and its build-metadata propagation; and a build-time
native or source generator that compiles or emits code consumed by handwritten
Rust.

## Exclusive directory

`fixtures/rust/projects/build-scripts-and-codegen/`

## Difficulty

complex

## Assigned coverage

- **RS-CAN-PROJ-011** — a build script emitting `rustc-cfg`, matching
  `rustc-check-cfg`, `rustc-env`, link search, library, and argument directives,
  metadata, and file- and environment-based rerun triggers, with a host/target
  probe and conditional output.
- **RS-CAN-PROJ-012** — Rust generated into `OUT_DIR` and included as items
  referenced by handwritten code; separate embedded text and bytes constants;
  relative and `OUT_DIR` include paths; one file both included and module-mounted.
- **RS-CAN-PROJ-020** — a `links`-declaring package emitting build metadata and
  a direct dependent consuming it through `DEP_*` variables.
- **RS-CAN-ECO-004** — a build dependency that compiles a hermetic vendored native
  translation unit and exposes the result to handwritten Rust.

Named counterexamples (a build-script panic, an unexpected-cfg warning, a
duplicate `links` claim, metadata not reaching an ordinary crate, a missing
generated output) are preserved as documentation beside their valid
demonstrations.

## Declared build contexts

- Default: `x86_64-unknown-linux-gnu`, edition 2024, stable 1.97.1.
- A host build-script compilation running before target compilation.
- The host C toolchain (`cc`/`gcc` 15.2.0) for the vendored native translation
  unit.

## Dependency needs

One third-party build dependency, `cc`, to compile the vendored native
translation unit for RS-CAN-ECO-004. It is pinned, vendored with source location,
license, and notices, and available offline. The build-script protocol, include
family, and `links` coverage use no third-party crate.

## Generated-source needs

Build scripts generate Rust and data into Cargo's `OUT_DIR` during `build`, and
the native translation unit is compiled during `build`. The generator source
(the build scripts and the vendored native input) is committed; the `OUT_DIR`
output is a build artifact rather than tracked source, so its regeneration by
`build` is expected. No tracked source is generated, and no separate `generate`
task is required.

## Planned tests

None.

## Required interfaces

Provide `Taskfile.yml` with `build`, `lint`, and `test` as defined in the ground
rules, where `build` runs the build scripts and native compilation offline, and
`coverage.md` mapping each assigned identifier to its stable source location.
