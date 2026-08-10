---
relationships:
  references:
    - ../ground-rules
    - ../../../reports/rust/synthesis/checklist
---

# Rust fixture corpus plan overview

## Selected toolchain

- `rustc`/`cargo` 1.97.1 (stable channel, commit 8bab26f4f, 2026-07-14), LLVM 22.
- The selected version is installed at that same commit both as the Homebrew PATH
  build and as the `rustup` toolchain `stable-x86_64-unknown-linux-gnu` (the
  active default under `RUSTUP_HOME=/usr/local/rustup`). The rustup stable
  toolchain additionally carries the cross targets, so it backs any
  cross-compilation context; both builds are the identical compiler.
- No nightly channel is installed or provisioned.
- `rustfmt` 1.9.0 and Clippy 0.1.97 are installed and back the `lint` task.
- A C toolchain (`cc`/`gcc` 15.2.0, target `x86_64-linux-gnu`) is available for
  native-build and foreign-interface coverage.

This release satisfies the checklist comparison floor (Rust and Cargo 1.85 with
edition 2024) and every stable feature the checklist version-gates through 1.97,
including trait upcasting (1.86+), precise capturing (1.82+), async closures
(1.85+), let chains (1.88+ with edition 2024), C string literals (1.77+),
`impl Trait`/`async fn` in traits (1.75+), inline const (1.79+), and the standard
lazy types (1.80+).

## Build contexts

### Default context

- Target `x86_64-unknown-linux-gnu`.
- Edition 2024.
- Stable `rustc`/Cargo 1.97.1.
- Resolver 3 unless a project pins another resolver for comparison.

### Additional available contexts

Projects declare these only where checklist coverage requires them. All are
supported by the selected installation and build offline:

- Editions 2015, 2018, and 2021 as additional per-crate inputs.
- Resolver versions 1 and 2 for feature-unification comparisons.
- Non-default Cargo profiles (release, custom, and package-override profiles),
  including a `panic = "abort"` profile.
- Cargo feature sets, including `--all-features` and `--no-default-features`.
- Procedural-macro host crates (`proc-macro = true`) and build-script host
  crates (`build.rs`) compiled for the host during the same build.
- `cfg`-selected source and dependency edges for non-host operating systems and
  architectures. These branches are valid but inactive on the host; only the
  host edge is compiled, so no non-host standard library is required.
- Cross-compilation to `aarch64-unknown-linux-gnu`, a second full-`std` Linux
  target installed in the rustup stable toolchain, where a project needs a real
  second architecture rather than a host-inactive `cfg` branch.
- Cross-compilation to `wasm32-unknown-unknown`, an OS-less target installed in
  the rustup stable toolchain. It is the freestanding, `no_std`-capable context
  for entry-symbol, panic-handler, and global-allocator runtime hooks, and the
  target for a Wasm bridge generator.
- `#![no_std]` and `alloc` compilation for the host and `wasm32-unknown-unknown`
  targets.
- Rust-defined C-ABI symbols linked across local crate boundaries, `x86_64`
  inline and global assembly, and `x86_64` target-feature and runtime-detection
  paths.
- A vendored C translation unit compiled through the host C toolchain.
- Local path, vendored-registry, and local git sources for dependency-source and
  multiple-version coverage, resolved with Cargo's offline and source-replacement
  configuration.

### Unavailable contexts

These are not installed and are treated as scope exceptions where a conditional
item depends on them:

- The nightly channel and any nightly-gated feature.
- Any target beyond the three installed in the rustup stable toolchain
  (`x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, and
  `wasm32-unknown-unknown`) — for example Windows, other architectures, other
  Wasm ABIs, or bare-metal embedded targets. Their standard-library or core
  components are not installed and cannot be provisioned.

## Projects

Seventeen projects cover the corpus. Each occupies one exclusive directory under
`fixtures/rust/projects/` and is independently implementable.

| Slug | Directory | Purpose | Difficulty |
| --- | --- | --- | --- |
| `cargo-build-model` | `fixtures/rust/projects/cargo-build-model/` | Cargo package, target, crate, workspace, dependency-graph, feature, resolver, source, lockfile, profile, crate-type, edition-graph, and invocation identity. | complex |
| `build-scripts-and-codegen` | `fixtures/rust/projects/build-scripts-and-codegen/` | Build-script protocol, generated Rust and include-family entities, native `links` ownership, and build-time native/source generation. | complex |
| `macros-and-procedural-macros` | `fixtures/rust/projects/macros-and-procedural-macros/` | Declarative and procedural macros, macro-generated entities, procedural-macro crate role, entry-point synthesis, and a hermetic macro-backed DSL. | complex |
| `modules-visibility-and-resolution` | `fixtures/rust/projects/modules-visibility-and-resolution/` | Modules, file mapping, namespaces, paths, imports, glob resolution, re-exports, visibility, preludes, scopes, identifiers, and intra-doc links. | complex |
| `types-and-layout` | `fixtures/rust/projects/types-and-layout/` | Struct, enum, union, alias, primitive, pointer, function, closure, DST, trait-object, `impl Trait`, recursion, layout, inference, coercion, and cast types. | complex |
| `generics-bounds-and-associated-items` | `fixtures/rust/projects/generics-bounds-and-associated-items/` | Generic parameters, bounds and HRTBs, const generics, associated types and constants, GATs, lifetime elision, variance, and precise capture. | complex |
| `traits-and-dispatch` | `fixtures/rust/projects/traits-and-dispatch/` | Implementations, coherence, blanket and conditional impls, supertraits, receivers, method lookup, dispatch, auto traits, operator/conversion/derive traits, and drop semantics. | complex |
| `ownership-and-borrowing` | `fixtures/rust/projects/ownership-and-borrowing/` | Moves, borrow exclusivity, non-lexical lifetimes, temporary scopes, interior mutability, closure capture, call traits, patterns, and destructuring. | complex |
| `expressions-constants-and-statics` | `fixtures/rust/projects/expressions-constants-and-statics/` | Place/value contexts, blocks, conditionals and let chains, loops, divergence, `?`, ranges, evaluation order, literals, constants, const functions, and statics. | complex |
| `attributes-and-lints` | `fixtures/rust/projects/attributes-and-lints/` | Attribute kinds and positions, conditional compilation, target and custom cfg, diagnostic and API-shaping attributes, and lint levels. | routine |
| `unsafe-ffi-and-abi` | `fixtures/rust/projects/unsafe-ffi-and-abi/` | Unsafe operations, validity and `MaybeUninit`, documented pointer APIs, transmutation, foreign interfaces and ABI, assembly, target features, and `no_std`/`alloc`. | complex |
| `async-and-concurrency` | `fixtures/rust/projects/async-and-concurrency/` | Async functions and blocks, await and polling, pinning, async closures, future sendability, threads, and atomics. | complex |
| `tests-and-documentation` | `fixtures/rust/projects/tests-and-documentation/` | Unit and integration test crate boundaries, harness attributes and generated mains, examples and benches, and documentation tests. | complex |
| `serde-and-error-derives` | `fixtures/rust/projects/serde-and-error-derives/` | Serialization-style derives and error derives with their conversion flow. | routine |
| `async-runtime-and-registration` | `fixtures/rust/projects/async-runtime-and-registration/` | Runtime and observability attribute macros and link-time registration. | complex |
| `api-facades-and-lazy-globals` | `fixtures/rust/projects/api-facades-and-lazy-globals/` | API facades, sealed and extension traits, lazy globals, and common API naming conventions. | routine |
| `wasm-bindgen-bridge` | `fixtures/rust/projects/wasm-bindgen-bridge/` | A target-gated Wasm bridge generator producing foreign-facing glue, exports, and links distinct from Rust `pub` and plain C exports. | complex |

## Primary assignments

Every in-scope canonical identifier has exactly one primary project. Natural
overlap in other projects is allowed and not accounted here.

### `cargo-build-model`

RS-CAN-PROJ-001, RS-CAN-PROJ-002, RS-CAN-PROJ-003, RS-CAN-PROJ-004,
RS-CAN-PROJ-005, RS-CAN-PROJ-006, RS-CAN-PROJ-007, RS-CAN-PROJ-008,
RS-CAN-PROJ-009, RS-CAN-PROJ-010, RS-CAN-PROJ-014, RS-CAN-PROJ-015,
RS-CAN-PROJ-016, RS-CAN-PROJ-019, RS-CAN-PROJ-021, RS-CAN-PROJ-022.

### `build-scripts-and-codegen`

RS-CAN-PROJ-011, RS-CAN-PROJ-012, RS-CAN-PROJ-020, RS-CAN-ECO-004.

### `macros-and-procedural-macros`

RS-CAN-PROJ-013, RS-CAN-PROJ-018, RS-CAN-MAC-001, RS-CAN-MAC-002,
RS-CAN-MAC-003, RS-CAN-MAC-004, RS-CAN-MAC-005, RS-CAN-MAC-006, RS-CAN-MAC-007,
RS-CAN-MAC-008, RS-CAN-ECO-009.

### `modules-visibility-and-resolution`

RS-CAN-MOD-001, RS-CAN-MOD-002, RS-CAN-MOD-003, RS-CAN-MOD-004, RS-CAN-MOD-005,
RS-CAN-MOD-006, RS-CAN-MOD-007, RS-CAN-MOD-008, RS-CAN-MOD-009, RS-CAN-MOD-010,
RS-CAN-MOD-011, RS-CAN-MOD-012, RS-CAN-MOD-013, RS-CAN-MOD-014.

### `types-and-layout`

RS-CAN-TYPE-001, RS-CAN-TYPE-002, RS-CAN-TYPE-003, RS-CAN-TYPE-004,
RS-CAN-TYPE-005, RS-CAN-TYPE-006, RS-CAN-TYPE-007, RS-CAN-TYPE-008,
RS-CAN-TYPE-009, RS-CAN-TYPE-010, RS-CAN-TYPE-011, RS-CAN-TYPE-012,
RS-CAN-TYPE-013, RS-CAN-TYPE-014, RS-CAN-TYPE-015, RS-CAN-TYPE-016.

### `generics-bounds-and-associated-items`

RS-CAN-GEN-001, RS-CAN-GEN-002, RS-CAN-GEN-003, RS-CAN-GEN-004, RS-CAN-GEN-005,
RS-CAN-GEN-006, RS-CAN-GEN-007, RS-CAN-GEN-008.

### `traits-and-dispatch`

RS-CAN-TRAIT-001, RS-CAN-TRAIT-002, RS-CAN-TRAIT-003, RS-CAN-TRAIT-004,
RS-CAN-TRAIT-005, RS-CAN-TRAIT-006, RS-CAN-TRAIT-007, RS-CAN-TRAIT-008,
RS-CAN-TRAIT-009, RS-CAN-TRAIT-010, RS-CAN-TRAIT-011, RS-CAN-TRAIT-012,
RS-CAN-TRAIT-013.

### `ownership-and-borrowing`

RS-CAN-OWN-001, RS-CAN-OWN-002, RS-CAN-OWN-003, RS-CAN-OWN-004, RS-CAN-OWN-005,
RS-CAN-OWN-006, RS-CAN-OWN-007, RS-CAN-OWN-008, RS-CAN-OWN-009, RS-CAN-OWN-010.

### `expressions-constants-and-statics`

RS-CAN-EXPR-001, RS-CAN-EXPR-002, RS-CAN-EXPR-003, RS-CAN-EXPR-004,
RS-CAN-EXPR-005, RS-CAN-EXPR-006, RS-CAN-EXPR-007, RS-CAN-EXPR-008,
RS-CAN-EXPR-009, RS-CAN-CONST-001, RS-CAN-CONST-002, RS-CAN-CONST-003.

### `attributes-and-lints`

RS-CAN-ATTR-001, RS-CAN-ATTR-002, RS-CAN-ATTR-003, RS-CAN-ATTR-004,
RS-CAN-DIAG-001.

### `unsafe-ffi-and-abi`

RS-CAN-UNSAFE-001, RS-CAN-UNSAFE-002, RS-CAN-UNSAFE-003, RS-CAN-UNSAFE-004,
RS-CAN-FFI-001, RS-CAN-FFI-002, RS-CAN-FFI-003, RS-CAN-FFI-004, RS-CAN-FFI-005,
RS-CAN-PROJ-017.

### `async-and-concurrency`

RS-CAN-ASYNC-001, RS-CAN-ASYNC-002, RS-CAN-ASYNC-003, RS-CAN-ASYNC-004,
RS-CAN-ASYNC-005, RS-CAN-CONCUR-001, RS-CAN-CONCUR-002.

### `tests-and-documentation`

RS-CAN-TEST-001, RS-CAN-TEST-002, RS-CAN-TEST-003.

### `serde-and-error-derives`

RS-CAN-ECO-001, RS-CAN-ECO-002.

### `async-runtime-and-registration`

RS-CAN-ECO-003, RS-CAN-ECO-007.

### `api-facades-and-lazy-globals`

RS-CAN-ECO-005, RS-CAN-ECO-006, RS-CAN-ECO-010.

### `wasm-bindgen-bridge`

RS-CAN-ECO-008.

## Exceptions

The following canonical identifiers receive no primary project. Each belongs to
one allowed exception category.

### Invalid-only items

- **RS-CAN-DIAG-002** — Failure phases and invalid programs. Its whole obligation
  is intentionally invalid programs and per-phase compiler failures, which are
  not corpus source. The valid adjacent observations it names (`cfg`-removed code
  is absent, dead but valid code is still type-checked) are covered by
  RS-CAN-ATTR-002.
- **RS-CAN-DIAG-003** — Recursion, type-computation, and solver limits. Its
  demonstrations are compiler rejections at configured resource limits, which are
  invalid or failing source.

### Unavailable conditional contexts

- **RS-CAN-TRAIT-014** — Specialization, trait aliases, and the negative-impl
  frontier. Its distinctive coverage is nightly-only, and no nightly channel is
  installed. Its stable approximations create no coverage beyond RS-CAN-TRAIT-002,
  RS-CAN-TRAIT-003, and RS-CAN-TRAIT-004.

### Research gaps

- **RS-CAN-PROJ-023** — Development-dependency cycle. The checklist classifies it
  as an unresolved research gap whose exact Cargo rules need primary confirmation
  for the selected version, so no known-valid source can be authored.

### Variant-level exceptions within in-scope items

These items keep a primary project for their confirmed valid core. The listed
variant is excluded for the stated category and is not separately assigned.

- **RS-CAN-TEST-003** — the `compile_fail` documentation-test variant is
  invalid-only: it is intentionally non-compiling synthetic Rust, so a passing
  harness does not make it valid corpus source. It is documented as a
  counterexample rather than planned. The runnable, no-run, ignored,
  should-panic (runtime panic in valid source), hidden-line, edition-tagged, and
  README-inclusion variants remain in scope under `tests-and-documentation`.
- **RS-CAN-ATTR-003** — the literal `cfg(true)`/`cfg(false)` predicate variant is
  an unresolved research gap. The target, custom, and checked-cfg core is in
  scope.
- **RS-CAN-GEN-003** — the `generic_const_exprs` frontier variant is nightly
  (unavailable). Minimal const generics are in scope.
- **RS-CAN-TRAIT-005** — the arbitrary-self-types variant is nightly
  (unavailable). The stable receiver forms are in scope.
- **RS-CAN-TRAIT-009** — custom and negative auto-trait impls are nightly
  (unavailable). Standard auto-trait behavior and stable `unsafe impl` are in
  scope.
- **RS-CAN-TYPE-006**, **RS-CAN-UNSAFE-003** — the full pointer-provenance and
  aliasing model beyond documented standard APIs is an unresolved research gap.
  The stable pointer syntax and documented strict-provenance and address APIs are
  in scope.
- **RS-CAN-MAC-007** — exact procedural-macro span and hygiene edge expectations
  are an unresolved research gap. The confirmed behavior (span-carrying
  identifiers and a resolution distinction that does not depend on diagnostic
  coordinates) is in scope under `macros-and-procedural-macros`.

Every other checklist counterexample or failure named inside an in-scope item is
preserved as documentation beside its valid demonstration, per the ground rules,
and needs no separate accounting.

## Independence audit

- Each project occupies one exclusive directory under `fixtures/rust/projects/`;
  the seventeen slugs in the project table are distinct.
- No project depends on another top-level fixture project. Every crate a project
  needs is either its own local crate or a vendored third-party crate named in
  its brief.
- Dependency-source, multiple-version, and transitive-skew coverage is satisfied
  inside each owning project with its own local path, vendored-registry, and
  local git sources; it does not reach across project directories.
- Third-party crates appear only in `build-scripts-and-codegen` (`cc`),
  `serde-and-error-derives` (`serde`, `thiserror`),
  `async-runtime-and-registration` (`tokio`, `tracing`, `inventory`),
  `api-facades-and-lazy-globals` (one lazy-global crate), and
  `wasm-bindgen-bridge` (`wasm-bindgen`). Each brief requires pinning, vendoring,
  provenance, and licensing, and no vendored crate is shared between projects.
- No brief edits shared corpus files, the checklist, or another project.
