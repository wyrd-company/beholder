# Canonical Rust Language-Feature Checklist

## Scope and synthesis rules

This checklist reconciles the Codex (`C`), Claude Fable (`F`), and Google Deep Research (`G`) reports. It describes observable Rust, `rustc`, Cargo, standard-library, and established ecosystem behavior. It does not prescribe a project theme, source layout, analysis method, or downstream consumer.

The stable comparison floor is Rust and Cargo 1.85 with edition 2024 available. Items stabilized later are conditional and version-pinned. Editions remain per-crate inputs and are not compiler-version aliases. Nightly-only behavior is isolated as a conditional negative or research item. Host, target, target triple, enabled features, Cargo command, profile, environment, compiler channel, and linker are semantic build inputs.

Classifications mean:

- **Language-defined:** specified by the Rust Reference or an edition rule.
- **Implementation-defined:** selected by `rustc`, Cargo, rustdoc, libtest, target Application Binary Interface (ABI), linker, or compiler configuration.
- **Unspecified:** intentionally lacks a stable language guarantee.
- **Ecosystem convention:** behavior of a named dependency or established Rust practice, not Rust syntax.

Statuses mean **required** for the stable core, **conditional** when a version, channel, target, command, or dependency selects it, and **unresolved research gap** when primary evidence is missing or the reports conflict.

## Canonical checklist

### Project, crate, package, and build identity

#### RS-CAN-PROJ-001 — Package, target, and crate identity

- **Feature or behavior:** A Cargo package is distribution metadata. Its library, binaries, examples, integration tests, benchmarks, and build script are separate crate compilations with separate roots and namespaces.
- **Fixture obligation:** Demonstrate one package producing a library and more than one secondary target, with one source file deliberately compiled in two crate identities.
- **Variants, interactions, counterexamples, and failures:** Contrast a binary importing its package library as an external crate with a binary mounting the same source through `mod`; show that same-spelled types from the two crates do not unify. Duplicate target names must fail.
- **Constraints:** Cargo target selection; `crate` always denotes the current compilation.
- **Classification · citations · provenance · status:** Implementation-defined with language crate semantics; [Cargo targets][cargo-targets], [crates and source files][ref-crates]; `C: RUST-PROJ-001`, `F: RS-PKG-01/03/07`, `G: RUST-PKG-001` (workspace portion only); **required**.

#### RS-CAN-PROJ-002 — Target discovery, explicit paths, and orphan files

- **Feature or behavior:** Cargo discovers conventional targets, can disable discovery, and can mount arbitrary target paths. A Rust file outside every selected target/module/include path is not compiled.
- **Fixture obligation:** Cover conventional and explicit library/binary/example/test/bench targets, one disabled auto-discovery family, and an invalid orphan `.rs` file that becomes a compile failure when linked.
- **Variants, interactions, counterexamples, and failures:** Include `src/bin/*.rs` and `src/bin/name/main.rs`, directory test/example forms, a broken auto-discovered target, and a cfg-disabled module subtree. Directory presence alone must create no Rust entity.
- **Constraints:** Cargo command and manifest auto-target settings select the source set.
- **Classification · citations · provenance · status:** Implementation-defined; [Cargo targets][cargo-targets], [modules][ref-modules]; `C: RUST-MOD-001/RUST-DIAG-002`, `F: RS-PKG-02/RS-MOD-05`, `G: RUST-BLD-003`; **required**.

#### RS-CAN-PROJ-003 — Package name, crate name, and dependency alias

- **Feature or behavior:** Package, library-target, executable-target, and local dependency names can differ. Hyphens in default library crate names map to underscores; `package =` and `[lib] name` create further distinctions.
- **Fixture obligation:** Give one dependency four observable names: package name, library name, local dependency key, and executable target name; resolve Rust paths through the local crate binding.
- **Variants, interactions, counterexamples, and failures:** Include two package versions under aliases, a local name collision, and macro output using an absolute dependency path. A consumer using the registry package spelling as a Rust identifier must fail where it differs.
- **Constraints:** Cargo manifest and `--extern` invocation.
- **Classification · citations · provenance · status:** Implementation-defined plus language name resolution; [dependency renaming][cargo-deps], [paths][ref-paths]; `C: RUST-PROJ-002`, `F: RS-PKG-01/RS-DEP-04`, `G: none`; **required**.

#### RS-CAN-PROJ-004 — Workspace composition and inheritance

- **Feature or behavior:** A workspace shares resolution, lockfile, output directory, commands, and inheritable metadata without creating a Rust namespace.
- **Fixture obligation:** Demonstrate rooted and virtual workspace forms, members, exclusions, default members, path dependencies, and opted-in inheritance for package fields, dependencies, and lints.
- **Variants, interactions, counterexamples, and failures:** Include a package below the workspace that is excluded, a missing inherited key error, member-added dependency features, and root-only profile/patch behavior. Keep resolver explicit in a virtual workspace.
- **Constraints:** Workspace inheritance requires the supporting Cargo versions; nested workspaces require exclusion.
- **Classification · citations · provenance · status:** Implementation-defined; [workspaces][cargo-workspaces]; `C: RUST-PROJ-003`, `F: RS-DEP-06/07`, `G: RUST-PKG-001`; **required**.

#### RS-CAN-PROJ-005 — Dependency kinds and host/target universes

- **Feature or behavior:** Normal, development, and build dependencies are visible to different crate targets. Build scripts and procedural macros compile for the host during cross-compilation; ordinary dependencies compile for the target.
- **Fixture obligation:** Use one package in normal and build roles, a dev-only dependency, a proc-macro dependency, and host/target reporting from a build script.
- **Variants, interactions, counterexamples, and failures:** Library access to a dev dependency must fail. Under resolver 2+, request a feature only on the host-side build and show separate crate compilations; add a target-specific build dependency.
- **Constraints:** Cargo resolver, selected target, and command.
- **Classification · citations · provenance · status:** Implementation-defined; [dependency kinds][cargo-deps], [build scripts][cargo-build], [resolver][cargo-resolver]; `C: RUST-PROJ-004`, `F: RS-DEP-01/11`, `G: RUST-BLD-002`; **required**.

#### RS-CAN-PROJ-006 — Cargo features, optional dependencies, and forwarding

- **Feature or behavior:** Package features are additive build inputs. Optional dependencies can be activated implicitly or through `dep:`, and dependency features can be forwarded strongly (`dep/feature`) or weakly (`dep?/feature`).
- **Fixture obligation:** Demonstrate default, non-default, optional-dependency, `dep:`, strong-forwarding, weak-forwarding, all-features, and no-default-features builds.
- **Variants, interactions, counterexamples, and failures:** Preserve accidental implicit features, an undeclared dependency-feature reference error, feature-gated public API and impls, and a `compile_error!` for a forbidden feature combination. Cargo does not make requested features mutually exclusive.
- **Constraints:** Package-local feature names; command and dependency requests select them.
- **Classification · citations · provenance · status:** Implementation-defined; [Cargo features][cargo-features]; `C: RUST-PROJ-005/006 and RUST-CFG-002`, `F: RS-CFG-06`, `G: RUST-CFG-002`; **required**.

#### RS-CAN-PROJ-007 — Feature unification and resolver versions

- **Feature or behavior:** Requests for a package version normally union. Resolver versions change unification across build/dev/normal and target-specific edges; resolver 3 also changes Rust-version-aware selection.
- **Fixture obligation:** Build the same workspace member alone and with a sibling under resolver 1, 2, and, where supported, 3; expose the selected feature set and package version.
- **Variants, interactions, counterexamples, and failures:** Show `default-features = false` being overridden by another edge, host/target separation under resolver 2, and mutually exclusive feature designs failing after union.
- **Constraints:** Resolver 3 is supported by recent Cargo and is the edition-2024 default for applicable packages; pin resolver explicitly for comparisons.
- **Classification · citations · provenance · status:** Implementation-defined; [feature unification][cargo-features], [resolver versions][cargo-resolver]; `C: RUST-PROJ-005`, `F: RS-CFG-07/RS-DEP-09`, `G: RUST-CFG-002`; **required**.

#### RS-CAN-PROJ-008 — Target-specific dependencies and cfg-selected graphs

- **Feature or behavior:** Cargo target tables select dependency edges by supported cfg predicates or exact triples, so host and target graphs can differ.
- **Fixture obligation:** Define Unix, Windows, architecture, and exact-triple alternatives with a common cfg-selected caller.
- **Variants, interactions, counterexamples, and failures:** Demonstrate overlapping table merge, host/target confusion for build dependencies, unsupported feature predicates in target tables, and a missing dependency on the wrong target.
- **Constraints:** Target triple and Cargo's supported target-table predicate subset.
- **Classification · citations · provenance · status:** Implementation-defined; [platform-specific dependencies][cargo-platform]; `C: RUST-PROJ-007`, `F: RS-DEP-01`, `G: RUST-CFG-001`; **required**.

#### RS-CAN-PROJ-009 — Dependency sources, patches, and multiple versions

- **Feature or behavior:** Package identity includes name, version, and source. Registry, git, and path sources differ; patches and source replacement alter resolution; semver-incompatible versions can coexist as type-incompatible crates.
- **Fixture obligation:** Resolve registry-equivalent, git-revision, and path dependencies; patch one graph-wide; expose two versions through aliases and demonstrate a same-named-type mismatch.
- **Variants, interactions, counterexamples, and failures:** Include unused-patch warning, a path dependency outside the workspace, transitive patch effect, alternate registry/source replacement configuration, and a consumer accidentally declaring a different transitive version.
- **Constraints:** Cargo configuration, lockfile, network/offline state, and source availability.
- **Classification · citations · provenance · status:** Implementation-defined; [dependency sources and overrides][cargo-deps], [overrides][cargo-overrides]; `C: RUST-PROJ-008`, `F: RS-DEP-02/03/05`, `G: none`; **required**.

#### RS-CAN-PROJ-010 — Lockfile, Rust version, and toolchain selection

- **Feature or behavior:** `Cargo.lock` pins resolved package identities and checksums; `package.rust-version` constrains supported toolchains; rustup toolchain selection and Cargo lockfile-format support independently affect build validity.
- **Fixture obligation:** Preserve a lockfile with duplicate package versions and a wider manifest range; pin a toolchain; declare an honest minimum Rust version.
- **Variants, interactions, counterexamples, and failures:** Exercise `--locked`, `--frozen`, offline mode, `--ignore-rust-version`, a path package without registry checksum, and an older Cargo rejecting a newer lockfile format.
- **Constraints:** Cargo/rustup versions and command flags.
- **Classification · citations · provenance · status:** Implementation-defined; [Cargo lockfile][cargo-lock], [rust-version][cargo-rust-version], [rustup overrides][rustup-overrides]; `C: RUST-PROJ-009`, `F: RS-DEP-03/10`, `G: baseline discussion`; **required**.

#### RS-CAN-PROJ-011 — Build-script protocol and generated configuration

- **Feature or behavior:** `build.rs` is a host crate that runs before target compilation and emits cfg, cfg-checking, environment, link, metadata, warning, and rerun directives; directive order can affect linker arguments.
- **Fixture obligation:** Emit `rustc-cfg`, matching `rustc-check-cfg`, `rustc-env`, link search/library/argument directives, metadata, and both file/environment rerun triggers.
- **Variants, interactions, counterexamples, and failures:** Include a build-script panic, unexpected cfg warning, host/target probe, conditional output, and a changed generated result without a source-tree Rust edit.
- **Constraints:** Cargo directive prefix/version, host environment, output encoding, target linker.
- **Classification · citations · provenance · status:** Implementation-defined; [build scripts][cargo-build], [check-cfg][rustc-check-cfg]; `C: RUST-PROJ-010`, `F: RS-GEN-01/03`, `G: RUST-BLD-001/002`; **required**.

#### RS-CAN-PROJ-012 — Generated Rust and include-family semantics

- **Feature or behavior:** `include!` parses another file in the caller's syntactic and macro-hygiene context and creates entities there. `include_str!` and `include_bytes!` create values and build inputs, not Rust entities.
- **Fixture obligation:** Generate Rust into `OUT_DIR`, include it as items referenced by handwritten code, and embed separate text and bytes constants.
- **Variants, interactions, counterexamples, and failures:** Cover relative and `OUT_DIR` paths, item and expression inclusion, syntax-context mismatch, missing generated output, diagnostics pointing to generated files, and one file both included and module-mounted to produce distinct entities.
- **Constraints:** Build order, invocation position, path base, and generated encoding.
- **Classification · citations · provenance · status:** Language-defined macros plus Cargo generation; [`include!`][std-include], [build-script code generation][cargo-build]; `C: RUST-PROJ-011`, `F: RS-MOD-04/RS-GEN-02`, `G: RUST-BLD-001`; **required**.

#### RS-CAN-PROJ-013 — Procedural-macro crate/build role

- **Feature or behavior:** A `proc-macro` crate is a host artifact that exports only procedural macro entry points. Its emitted tokens become source in another crate; its ordinary dependencies stay in the build-time universe.
- **Fixture obligation:** Build a sibling proc-macro crate with function-like, derive, and attribute entry points and host-only dependencies; use all three from a target crate.
- **Variants, interactions, counterexamples, and failures:** Include helper attributes, panic and `compile_error!`, invalid generated syntax, attempted normal export, dependency-cycle rejection, and host/target difference during cross-compilation.
- **Constraints:** `proc-macro = true`, compiler channel/API, unsandboxed host execution.
- **Classification · citations · provenance · status:** Language-defined mechanism plus implementation execution; [procedural macros][ref-proc-macro], [Cargo targets][cargo-targets]; `C: RUST-PROJ-012`, `F: RS-MAC-06/07/08/09`, `G: RUST-MAC-002`; **required**.

#### RS-CAN-PROJ-014 — Crate types and linker-facing artifacts

- **Feature or behavior:** `rlib`, `dylib`, `cdylib`, `staticlib`, executable, and `proc-macro` outputs have distinct metadata, link, and export boundaries.
- **Fixture obligation:** Build compatible Rust-library and foreign-consumable crate types from one library and expose explicit C-ABI symbols.
- **Variants, interactions, counterexamples, and failures:** Contrast Rust `pub` API with exported symbols, `dylib` versus `cdylib`, mixed crate types, unsupported platform linker combinations, and depending on a proc-macro as an ordinary library.
- **Constraints:** Target/linker support; Rust ABI and metadata are not stable foreign contracts.
- **Classification · citations · provenance · status:** Implementation-defined; [linkage][ref-linkage], [rustc crate types][rustc-crate-type]; `C: RUST-PROJ-013`, `F: RS-PKG-04/RS-UNS-05`, `G: none`; **required**.

#### RS-CAN-PROJ-015 — Profiles and code-generation configuration

- **Feature or behavior:** Profiles change overflow checks, debug assertions, panic strategy, optimization, debug symbols, Link Time Optimization (LTO), and codegen units; package overrides can compile dependencies differently.
- **Fixture obligation:** Define development, release, custom, and package-override profiles and expose `cfg(debug_assertions)`, overflow-sensitive, panic-sensitive, and symbol/codegen-only differences.
- **Variants, interactions, counterexamples, and failures:** Contrast unwind and abort with `catch_unwind`, test/bench restrictions, dependencies optimized differently, profile inheritance, and linker-visible output changes without source changes.
- **Constraints:** Cargo/rustc version, target, test harness, and panic-runtime support.
- **Classification · citations · provenance · status:** Implementation-defined; [profiles][cargo-profiles], [codegen options][rustc-codegen]; `C: RUST-PROJ-014`, `F: RS-DEP-08`, `G: none`; **required**.

#### RS-CAN-PROJ-016 — Editions per crate and mixed-edition graphs

- **Feature or behavior:** Editions change parsing, keywords, paths, preludes, macro fragments, capture rules, temporary scopes, and lints, while crates from different editions interoperate in one graph.
- **Fixture obligation:** Include interacting crates on 2015, 2018, 2021, and 2024, each with at least one edition-sensitive construct and a macro defined in one edition and invoked from another.
- **Variants, interactions, counterexamples, and failures:** Cover raw identifiers, leading `::`, extern-crate/macro imports, array `IntoIterator`, closure capture, macro fragment editions, 2024 unsafe syntax, Return-Position `impl Trait` (RPIT) capture, prelude additions, and migration-lint output.
- **Constraints:** Edition is per crate; edition 2024 requires rustc 1.85+.
- **Classification · citations · provenance · status:** Language-defined; [Edition Guide][edition-guide], [editions appendix][ref-editions]; `C: RUST-PROJ-015 and recent-version inventory`, `F: RS-EDI-01..06`, `G: RUST-EDT-001`; **required**.

#### RS-CAN-PROJ-017 — `no_std`, `alloc`, entry, and runtime contracts

- **Feature or behavior:** `#![no_std]` changes prelude and linked facade; `alloc` is explicitly introduced. Freestanding binaries can require `no_main`, panic handlers, allocators, and entry symbols. `no_core` and custom language items remain unstable.
- **Fixture obligation:** Provide a `no_std` library, an `alloc`-using variant, and target-gated freestanding binary/runtime hooks; show identical core types through `std` and `core` paths where `std` exists.
- **Variants, interactions, counterexamples, and failures:** Cover test harness dependence on `std`, missing/duplicate panic handler or global allocator, unavailable unwinding/allocation, accidental `std` feature unification, and stable rejection of `no_core` customization.
- **Constraints:** Target runtime, allocator, panic strategy, channel, installed standard components.
- **Classification · citations · provenance · status:** Language-defined plus implementation runtime; [`no_std`][ref-preludes], [panic handler][ref-panic-handler], [global allocator][std-global-alloc]; `C: RUST-PROJ-016`, `F: RS-STD-01/02/03`, `G: RUST-NS-002`; **conditional**.

#### RS-CAN-PROJ-018 — Entry points and test-generated mains

- **Feature or behavior:** Binary crate roots require a permitted `main` unless `no_main`; `main` can return a `Termination` type. Test harnesses and common attribute macros synthesize or rewrite entry points.
- **Fixture obligation:** Demonstrate unit-returning and `Result`-returning mains, missing-main failure, `no_main`, a generated test main, and one macro-rewritten main.
- **Variants, interactions, counterexamples, and failures:** A main nested only in a submodule must fail; custom harness/no-main targets must provide their own contract; runtime attribute macros produce a compiled signature distinct from source.
- **Constraints:** Crate type, target runtime, harness, dependency version.
- **Classification · citations · provenance · status:** Language-defined plus implementation/ecosystem generation; [main functions][ref-main], [`Termination`][std-termination], [Cargo tests][cargo-tests]; `C: RUST-PROJ-016/RUST-TEST-002`, `F: RS-PKG-06`, `G: none`; **required**.

#### RS-CAN-PROJ-019 — Source meaning from invocation and external configuration

- **Feature or behavior:** Crate name/type, edition, cfg set, extern universe, target, codegen flags, and Cargo configuration may be supplied outside source. Hierarchical `.cargo/config.toml` and environment variables can change them.
- **Fixture obligation:** Compile one source under two explicit invocation/config variants and include checked-in Cargo configuration for a target, cfg/rustflag, environment value, source setting, or alias.
- **Variants, interactions, counterexamples, and failures:** Cover `--crate-name`, `--edition`, `--cfg`, `--extern`, `RUSTFLAGS`, config precedence, and an external config that causes environment-dependent output.
- **Constraints:** Direct rustc versus Cargo invocation and configuration discovery location.
- **Classification · citations · provenance · status:** Implementation-defined; [rustc command line][rustc-cli], [Cargo configuration][cargo-config]; `C: baseline and RUST-CFG-001/002`, `F: RS-TCH-01/02`, `G: toolchain-coupling premise`; **required**.

#### RS-CAN-PROJ-020 — Native `links` ownership and build metadata

- **Feature or behavior:** A package `links` value claims one native library name graph-wide and permits build-script metadata to reach direct dependent build scripts through `DEP_*` variables.
- **Fixture obligation:** Build a sys-style package emitting metadata and a direct dependent consuming it.
- **Variants, interactions, counterexamples, and failures:** Include duplicate `links` ownership failure, metadata not reaching ordinary crate environment, build-script override, and cross-compilation configuration.
- **Constraints:** Cargo graph and immediate build-dependency relationships.
- **Classification · citations · provenance · status:** Implementation-defined; [Cargo `links`][cargo-links]; `C: RUST-ECO-005`, `F: RS-GEN-04`, `G: RUST-BLD-002`; **conditional**.

#### RS-CAN-PROJ-021 — Command-dependent compilation sets

- **Feature or behavior:** `cargo build`, `test`, `doc`, `clippy`, example, bench, and target-selection commands compile different crate sets under different cfgs.
- **Fixture obligation:** Make `cfg(test)`, `cfg(doc)`, and `cfg(doctest)` entities observable and map each command to selected crates and generated harnesses.
- **Variants, interactions, counterexamples, and failures:** Include code that fails only under test or docs, dev-dependency availability, an ignored test that still compiles, and rustdoc/clippy-only lint attributes.
- **Constraints:** Command, component availability, requested target/features.
- **Classification · citations · provenance · status:** Implementation-defined; [Cargo commands/targets][cargo-targets], [rustdoc tests][rustdoc-doctest]; `C: RUST-TEST-001..004`, `F: RS-TST-08/RS-TCH-04`, `G: none`; **required**.

#### RS-CAN-PROJ-022 — Fine-grained build identity

- **Feature or behavior:** Name and version do not fully identify a crate compilation. Source, target, role, features, profile, compiler, and metadata disambiguators can create multiple crate instances and physical artifacts.
- **Fixture obligation:** Demonstrate the same package version compiled for host and target or under distinct feature roles, and two package versions compiled together.
- **Variants, interactions, counterexamples, and failures:** Do not infer semantic identity from mangled symbol, path, name, or version alone; show source-identical crate instances with different cfg or target APIs.
- **Constraints:** Cargo resolver and rustc metadata/codegen configuration.
- **Classification · citations · provenance · status:** Implementation-defined; [rustc codegen metadata][rustc-codegen], [Cargo resolver][cargo-resolver]; `C: baseline/RUST-PROJ-004/008`, `F: RS-TCH-05`, `G: toolchain-coupling premise`; **required**.

#### RS-CAN-PROJ-023 — Development-dependency cycle

- **Feature or behavior:** A path dev-dependency can depend normally on the package under test, creating a cycle admitted only in the development graph.
- **Fixture obligation:** If supported by the selected Cargo baseline, make package A dev-depend on helper B while B normally depends on A; consume B from A's tests.
- **Variants, interactions, counterexamples, and failures:** Promote the A-to-B edge to a normal dependency and retain the cycle rejection.
- **Constraints:** Exact Cargo cycle rules need primary confirmation for the selected version.
- **Classification · citations · provenance · status:** Implementation-defined; [development dependencies][cargo-deps]; `C: none`, `F: RS-DEP-12`, `G: none`; **unresolved research gap**.

### Modules, paths, namespaces, and visibility

#### RS-CAN-MOD-001 — Module declaration and file mapping

- **Feature or behavior:** Inline and out-of-line modules create the same namespace kind, while `name.rs`, `name/mod.rs`, child-directory mapping, and `#[path]` choose source files.
- **Fixture obligation:** Mix inline, modern, legacy, and `#[path]` modules with nested children.
- **Variants, interactions, counterexamples, and failures:** Both candidate files, missing file, cfg-selected path alternatives, changed base for nested inline modules, and an undeclared file must be observable.
- **Constraints:** Module declaration context and edition-era layout convention.
- **Classification · citations · provenance · status:** Language-defined with compiler file loading; [modules and source files][ref-modules]; `C: RUST-MOD-001`, `F: RS-MOD-01/02/06`, `G: RUST-BLD-003/RUST-CFG-001`; **required**.

#### RS-CAN-MOD-002 — One file mounted more than once

- **Feature or behavior:** Source text compiled through multiple module, include, or crate roots creates distinct entities; textual identity does not unify nominal types or implementations.
- **Fixture obligation:** Mount one file at two module paths and in two crate roles, then provoke a cross-mount type mismatch.
- **Variants, interactions, counterexamples, and failures:** Compare `#[path]`, `include!`, and lib/bin duplication; note that coherence evaluates the distinct local type identities.
- **Constraints:** Each mount must be reachable in the selected configuration.
- **Classification · citations · provenance · status:** Language-defined; [modules][ref-modules], [`include!`][std-include]; `C: RUST-PROJ-001/RUST-MOD-001`, `F: RS-MOD-03`, `G: none`; **required**.

#### RS-CAN-MOD-003 — Item, value, macro, lifetime, and label namespaces

- **Feature or behavior:** Rust resolves type, value, macro, lifetime, and label names separately. Same spelling can denote multiple entities; constructors occupy value namespace according to item form.
- **Fixture obligation:** Reuse selected spellings across all five namespaces and resolve each in context.
- **Variants, interactions, counterexamples, and failures:** Include tuple/unit struct constructor collision, named-field struct without callable constructor, derive/attribute/bang macro distinctions, same-spelled lifetime and loop label, and a within-namespace duplicate error.
- **Constraints:** Import may introduce bindings into more than one namespace.
- **Classification · citations · provenance · status:** Language-defined; [names and namespaces][ref-namespaces]; `C: RUST-MOD-002`, `F: RS-RES-03/RS-MAC-10`, `G: RUST-NS-001` (its three-namespace summary is incomplete); **required**.

#### RS-CAN-MOD-004 — Path roots and qualifiers

- **Feature or behavior:** `crate`, `self`, `super`, leading `::`, `Self`, extern-prelude roots, and fully qualified paths select different anchors.
- **Fixture obligation:** Resolve equivalent and shadowed paths from nested modules, including a local module named like a dependency and a `Self` constructor/associated item.
- **Variants, interactions, counterexamples, and failures:** Too many `super` segments, edition-2015 versus 2018+ leading `::`, lexical `self::`, `<Type as Trait>::Item`, and associated-type traversal.
- **Constraints:** Edition and trait/type context.
- **Classification · citations · provenance · status:** Language-defined; [paths][ref-paths]; `C: RUST-MOD-003/012`, `F: RS-RES-06/RS-DSP-02`, `G: RUST-TRT-002`; **required**.

#### RS-CAN-MOD-005 — `use` forms and import scope

- **Feature or behavior:** Imports create bindings, not ownership; they support groups, aliases, `self`, glob, and underscore trait/macro imports and are not inherited by child modules.
- **Fixture obligation:** Exercise every form, a child re-import, an order-independent forward item import, and `Trait as _` enabling method lookup.
- **Variants, interactions, counterexamples, and failures:** Duplicate bindings, namespace-multiple imports, special `use` path resolution, and an illegal underscore import target.
- **Constraints:** Edition affects path grammar and legacy macro import.
- **Classification · citations · provenance · status:** Language-defined; [use declarations][ref-use]; `C: RUST-MOD-004`, `F: RS-RES-01/09`, `G: none`; **required**.

#### RS-CAN-MOD-006 — Glob ambiguity and shadowing

- **Feature or behavior:** Glob bindings are lazy, can be shadowed by explicit/local bindings, and may remain ambiguous until a colliding name is used.
- **Fixture obligation:** Import two colliding globs with unused and used names, then resolve another collision explicitly.
- **Variants, interactions, counterexamples, and failures:** Cover enum variants, macro namespace, prelude interaction, and public glob re-export breakage after an upstream addition.
- **Constraints:** Visibility and namespace of imported names.
- **Classification · citations · provenance · status:** Language-defined; [use declarations][ref-use]; `C: RUST-MOD-005`, `F: RS-RES-04`, `G: RUST-VIS-003` (glob re-export variant); **required**.

#### RS-CAN-MOD-007 — Re-exports and multiple public paths

- **Feature or behavior:** `pub use` exposes an existing visible entity through another path without changing identity; a private containing module can be bypassed only when the item itself is sufficiently visible.
- **Fixture obligation:** Re-export a public item from a private implementation module at root, rename it, glob-export selected items, and re-export a dependency type.
- **Variants, interactions, counterexamples, and failures:** Preserve multiple paths, re-export chains, macro re-exports, insufficient item visibility failure, and rustdoc inline/no-inline presentation as a separate documentation effect.
- **Constraints:** Effective visibility caps the re-export.
- **Classification · citations · provenance · status:** Language-defined; [visibility and re-exports][ref-visibility], [use declarations][ref-use]; `C: RUST-MOD-006`, `F: RS-RES-02`, `G: RUST-VIS-003`; **required**.

#### RS-CAN-MOD-008 — Restricted and effective visibility

- **Feature or behavior:** Private, `pub(self)`, `pub(super)`, `pub(crate)`, `pub(in ancestor)`, and `pub` form module-relative boundaries; nominal `pub` is capped by containing reachability.
- **Fixture obligation:** Access each level from child, sibling, parent, same crate, and downstream crate; re-export one otherwise unreachable public item.
- **Variants, interactions, counterexamples, and failures:** Reject non-ancestor or import-alias `pub(in)` paths; cover public signatures involving private types/bounds, reachable-but-unnameable values, and `unreachable_pub`, `private_interfaces`, and `private_bounds` diagnostics.
- **Constraints:** Privacy lint severity has changed by compiler version; assert category, not exact wording.
- **Classification · citations · provenance · status:** Language-defined plus implementation diagnostics; [visibility][ref-visibility], [rustc lints][rustc-lints]; `C: RUST-MOD-007/008`, `F: RS-VIS-01/02/05`, `G: RUST-VIS-001/002`; **required**.

#### RS-CAN-MOD-009 — Field, constructor, variant, trait-item, and impl-item visibility

- **Feature or behavior:** Fields have independent visibility; private fields restrict literal/update construction and tuple constructors. Trait items and enum variants follow their container rules; inherent associated items choose visibility individually.
- **Fixture obligation:** Use mixed-visibility named and tuple structs, cross-crate enum matching, a trait, and an inherent impl with public/private methods.
- **Variants, interactions, counterexamples, and failures:** Reject external struct update or tuple construction through private fields and `pub` on a trait item; compare `#[non_exhaustive]` with privacy.
- **Constraints:** Cross-crate boundary is required for some effects.
- **Classification · citations · provenance · status:** Language-defined; [visibility][ref-visibility], [structs][ref-structs], [traits][ref-traits]; `C: RUST-TYPE-001/RUST-MOD-007`, `F: RS-VIS-03/04`, `G: default privacy family`; **required**.

#### RS-CAN-MOD-010 — Extern prelude, direct nameability, and `extern crate`

- **Feature or behavior:** Cargo passes direct dependencies into the extern prelude. Transitive crates are not directly nameable unless declared or re-exported. `extern crate` still supports legacy macro import, aliases, sysroot crates, `alloc`, `proc_macro`, and self aliases.
- **Fixture obligation:** Use an implicit modern direct dependency, an unnameable transitive type, a re-export, `extern crate alloc`, a renamed extern crate, `extern crate self as ...`, and a 2015 `#[macro_use]` case.
- **Variants, interactions, counterexamples, and failures:** Cover `--extern` omission, alias conflict, transitive version skew, and redundant modern extern-crate lint.
- **Constraints:** Edition, Cargo invocation, `no_std`, and channel for special sysroot crates.
- **Classification · citations · provenance · status:** Language-defined plus Cargo invocation; [external crates][ref-extern-crate], [preludes][ref-preludes]; `C: RUST-MOD-009`, `F: RS-RES-08/RS-DEP-05`, `G: RUST-NS-002`; **required**.

#### RS-CAN-MOD-011 — Standard, core, extern, macro-use, and language preludes

- **Feature or behavior:** Modules receive several implicit prelude layers. Standard/core prelude contents vary by edition and `no_std`; locals can shadow them; `no_implicit_prelude` removes ordinary implicit imports.
- **Fixture obligation:** Resolve unqualified prelude types/traits, one edition addition, a local shadow, explicit core versus std paths, and a controlled no-implicit-prelude module.
- **Variants, interactions, counterexamples, and failures:** Keep extern and macro-use preludes distinct; include built-in/tool attributes and the implicit `'static` lifetime as separate implicit-name categories.
- **Constraints:** Edition, `no_std`, and compiler-provided built-ins.
- **Classification · citations · provenance · status:** Language-defined plus standard-library contents; [preludes][ref-preludes], [edition preludes][std-prelude]; `C: RUST-MOD-010`, `F: RS-RES-05/RS-STD-01/02`, `G: RUST-NS-002`; **required**.

#### RS-CAN-MOD-012 — Item and local scopes, shadowing, and nested items

- **Feature or behavior:** Item declarations generally cover their enclosing module/block regardless of textual order. Local bindings begin after declaration and can shadow with a new type. Nested function items cannot capture dynamic environment; closures can.
- **Fixture obligation:** Call an item before declaration, shadow a local with a changed type, define nested functions/types/imports/impls, and contrast illegal item capture with closure capture.
- **Variants, interactions, counterexamples, and failures:** Include guard/arm binding scope, label/generic scopes, body-local import non-leakage, and duplicate inherent/trait impl coherence beyond textual scope.
- **Constraints:** Exact global effect of a body-local trait impl needs a version-pinned probe.
- **Classification · citations · provenance · status:** Language-defined; [scopes][ref-scopes], [items][ref-items]; `C: RUST-MOD-011`, `F: RS-MOD-07`, `G: none`; **required**, with body-local-impl detail noted in gaps.

#### RS-CAN-MOD-013 — Raw and Unicode identifiers

- **Feature or behavior:** Raw identifiers permit most keyword spellings; Unicode identifiers are accepted and normalized according to Rust identifier rules.
- **Fixture obligation:** Export an older-edition keyword name to a newer-edition consumer using `r#`, and define/use a non-ASCII identifier.
- **Variants, interactions, counterexamples, and failures:** Reject raw forms of reserved path keywords where forbidden; include keyword-set changes, normalization-equivalent spellings, and confusable/uncommon-codepoint lints without treating visual similarity as identity.
- **Constraints:** Edition; Unicode identifier support/version; exact normalization and lint behavior require confirmation.
- **Classification · citations · provenance · status:** Language-defined plus implementation lints; [identifiers][ref-identifiers]; `C: final-audit parser gap`, `F: RS-RES-07/11`, `G: RUST-EDT-001`; **conditional**.

#### RS-CAN-MOD-014 — Intra-documentation link resolution

- **Feature or behavior:** rustdoc resolves paths and namespace disambiguators inside documentation independently of ordinary code use sites and lints broken links.
- **Fixture obligation:** Link to an item, method, `Self` item, re-export, and same-spelled entities using namespace disambiguators; retain a broken-link diagnostic.
- **Variants, interactions, counterexamples, and failures:** Include private/hidden items, cross-module links, re-export paths, and deny-warnings docs failure.
- **Constraints:** rustdoc command/cfg and lint configuration.
- **Classification · citations · provenance · status:** Implementation-defined; [intra-doc links][rustdoc-links]; `C: final-audit documentation gap`, `F: RS-RES-10`, `G: none`; **conditional**.

### Items, types, generics, and layout

#### RS-CAN-TYPE-001 — Struct forms, construction, and update

- **Feature or behavior:** Named-field, tuple, unit, and zero-field braced structs create nominal types; tuple/unit forms also create value constructors. Field visibility and update syntax determine construction rights.
- **Fixture obligation:** Define and construct all forms, destructure them, pass constructors as values, and compare unit with zero-field braced syntax.
- **Variants, interactions, counterexamples, and failures:** Cover update moving/copying remaining fields, private-field external construction/update failure, generic defaults, zero-sized layout, and `PhantomData` fields.
- **Constraints:** Visibility and generic bounds.
- **Classification · citations · provenance · status:** Language-defined; [structs][ref-structs]; `C: RUST-TYPE-001`, `F: RS-ITM-01/RS-VIS-03`, `G: common-traits discussion only`; **required**.

#### RS-CAN-TYPE-002 — Enums, variants, discriminants, and non-exhaustiveness

- **Feature or behavior:** Enum variants have unit, tuple, or struct payloads and may carry explicit/computed discriminants within representation rules. Duplicate discriminant values are rejected.
- **Fixture obligation:** Mix all forms, explicit discriminants, imports, constructor-as-function use, `Self::Variant`, discriminant observation, and exhaustive matching.
- **Variants, interactions, counterexamples, and failures:** Include duplicate and overflow errors, fieldless `as` cast, data-bearing discriminant representation constraints, cfg-removed variants, and cross-crate `#[non_exhaustive]` wildcard requirement.
- **Constraints:** `repr` and cross-crate context; default enum layout is not a C contract.
- **Classification · citations · provenance · status:** Language-defined; [enumerations][ref-enums], [type layout][ref-layout]; `C: RUST-TYPE-002/RUST-EXPR-004`, `F: RS-ITM-02/RS-ATT-05`, `G: representation item`; **required**. This resolves the report conflict against Codex's unverified suggestion that duplicate discriminants may be allowed.

#### RS-CAN-TYPE-003 — Unions

- **Feature or behavior:** Union fields overlap; writes are generally safe, reads are unsafe, ordinary drop fields require `ManuallyDrop`, and active-field validity is maintained by the programmer.
- **Fixture obligation:** Use `Copy` fields and a `ManuallyDrop` field, safe writes, unsafe reads, and FFI representation.
- **Variants, interactions, counterexamples, and failures:** Reject an ordinary non-`Copy` field, show pattern restrictions, and document rather than execute invalid-bit-pattern and inactive-field counterexamples.
- **Constraints:** Unsafe validity rules and optional `repr(C)`.
- **Classification · citations · provenance · status:** Language-defined with unsafe-model edges; [unions][ref-unions], [undefined behavior][ref-ub]; `C: RUST-TYPE-003`, `F: RS-ITM-03`, `G: none`; **required**.

#### RS-CAN-TYPE-004 — Type aliases and nominal newtypes

- **Feature or behavior:** A type alias is another path to the same type; a tuple struct/enum is a new nominal identity. Alias genericity and associated aliases do not create an independent runtime type or coherence identity.
- **Fixture obligation:** Compare assignment, coercion, methods, trait impl coherence, and construction for an alias and newtype over the same representation.
- **Variants, interactions, counterexamples, and failures:** Cover generic/trait-object/associated aliases, recursive alias rejection, constructor path limits, and an impl written through an alias where legal showing it belongs to the underlying local type.
- **Constraints:** Inherent associated types remain unstable; exact lazy-checking and alias-constructor rules are version-sensitive.
- **Classification · citations · provenance · status:** Language-defined; [type aliases][ref-type-aliases]; `C: RUST-TYPE-004`, `F: RS-ITM-05`, `G: none`; **required**, with edge legality retained in research gaps.

#### RS-CAN-TYPE-005 — Primitive, tuple, array, slice, string, and never types

- **Feature or behavior:** Built-in types have special syntax; tuples are structural, arrays include length in type identity, slices/`str` are dynamically sized, and `!` represents divergence and coerces at valid sites.
- **Fixture obligation:** Cover unit and one-element tuples, const array lengths, array-to-slice coercion, string/byte forms, a `-> !` function, and a diverging branch joining a concrete type.
- **Variants, interactions, counterexamples, and failures:** Include index panic, old compiler trait-impl boundaries for arrays, never-type fallback ambiguity, and invalid by-value slice/`str` positions.
- **Constraints:** Some never-type generic/fallback uses remain version-sensitive.
- **Classification · citations · provenance · status:** Language-defined plus standard trait implementations; [types][ref-types], [never type][ref-never]; `C: RUST-TYPE-005/RUST-EXPR-007`, `F: RS-TYP-06`, `G: implicit built-ins`; **required**.

#### RS-CAN-TYPE-006 — References, raw pointers, and mutability

- **Feature or behavior:** References impose validity, alignment, lifetime, and aliasing obligations; raw pointers relax borrow checking but require unsafe dereference. `&raw` avoids creating an intermediate reference.
- **Fixture obligation:** Demonstrate shared/mutable references, reborrows, raw formation from references, address-of-raw syntax, null/misaligned APIs, and unsafe dereference.
- **Variants, interactions, counterexamples, and failures:** Never execute an invalid reference; document unaligned packed-field, dangling, null, and aliasing counterexamples. Separate stable pointer syntax from evolving provenance.
- **Constraints:** `&raw` stabilization/version and unsafe memory model.
- **Classification · citations · provenance · status:** Language-defined core with unspecified/evolving provenance; [pointer types][ref-pointers], [undefined behavior][ref-ub]; `C: RUST-TYPE-006/RUST-UNSAFE-004`, `F: recent-version inventory`, `G: unsafe research gap`; **required** for syntax, provenance portion **conditional**.

#### RS-CAN-TYPE-007 — Function items, function pointers, and closures

- **Feature or behavior:** Each function definition and generic instantiation has a unique zero-sized function-item type; it can coerce to an `fn` pointer. Every closure expression has a distinct anonymous type; only noncapturing closures coerce to function pointers.
- **Fixture obligation:** Compare two same-signature function items, two generic instantiations, explicit pointer coercion, noncapturing closure coercion, capturing closure, and indirect calls.
- **Variants, interactions, counterexamples, and failures:** Cover `unsafe fn`, `extern ABI fn`, higher-ranked pointers, join-point coercion, and failure to coerce a capturing closure.
- **Constraints:** ABI and safety are part of pointer type identity.
- **Classification · citations · provenance · status:** Language-defined; [function item/pointer types][ref-fn-types], [closures][ref-closures]; `C: RUST-TYPE-007`, `F: RS-CLO-02/RS-DSP-04`, `G: none`; **required**.

#### RS-CAN-TYPE-008 — Dynamically sized types, metadata, and `?Sized`

- **Feature or behavior:** Slices, `str`, trait objects, and custom trailing-field forms can be Dynamically Sized Types (DSTs). Type parameters default to `Sized`; `?Sized` relaxes that bound. Wide-pointer metadata differs by pointee kind.
- **Fixture obligation:** Accept both sized and unsized values behind pointers, define a trailing-slice struct, and demonstrate slice and trait-object unsizing.
- **Variants, interactions, counterexamples, and failures:** Reject DST locals/by-value fields and non-final DST fields; include `Self: ?Sized`, pointer metadata distinctions, and unstable `extern type`/unsized-local boundaries.
- **Constraints:** Custom DST construction can require unsafe code; custom coercion traits are unstable.
- **Classification · citations · provenance · status:** Language-defined; [DSTs][ref-dst], [type parameters][ref-generics]; `C: RUST-TYPE-008/RUST-TRAIT-015`, `F: RS-TYP-04`, `G: method unsizing variant`; **required**.

#### RS-CAN-TYPE-009 — Trait objects and dyn compatibility

- **Feature or behavior:** `dyn Trait` erases the concrete type behind runtime data/vtable metadata. Object type identity includes principal trait, auto traits, associated bindings, and lifetime; only dyn-compatible traits qualify.
- **Fixture obligation:** Use a dyn-compatible trait dynamically, bind an associated type/lifetime/auto trait, and contrast an incompatible trait repaired by `Self: Sized` on an offending method.
- **Variants, interactions, counterexamples, and failures:** Cover generic methods, associated consts, prohibited `Self` positions, consuming receivers, two non-auto principal traits, and attempted dyn use of native async/RPIT trait methods.
- **Constraints:** Vtable layout is unspecified; trait upcasting is conditional on rustc 1.86+.
- **Classification · citations · provenance · status:** Language-defined surface with unspecified representation; [trait objects][ref-trait-objects], [dyn compatibility][ref-dyn-compatible]; `C: RUST-TYPE-009`, `F: RS-TRT-08`, `G: method/trait discussion`; **required**, upcasting **conditional**.

#### RS-CAN-TYPE-010 — Argument- and return-position `impl Trait`

- **Feature or behavior:** Argument-position `impl Trait` is an anonymous generic parameter; Return-Position `impl Trait` (RPIT) is one hidden concrete type per definition. Distinct opaque definitions have distinct identity.
- **Fixture obligation:** Pair argument syntax with a named-generic twin; return one opaque iterator from multiple same-concrete-type branches; contrast caller turbofish and incompatible branch errors.
- **Variants, interactions, counterexamples, and failures:** Cover nested opaque use, auto-trait leakage, lifetime/type/const capture, and distinct hidden types from different functions.
- **Constraints:** RPIT capture defaults are edition-sensitive; precise capture is separately version-gated.
- **Classification · citations · provenance · status:** Language-defined; [`impl Trait`][ref-impl-trait]; `C: RUST-TYPE-010`, `F: RS-CLO-03/04`, `G: none`; **required**.

#### RS-CAN-TYPE-011 — RPIT and async functions in traits

- **Feature or behavior:** Since Rust 1.75, trait methods can return opaque types or use `async fn`; each impl chooses a hidden return type. Such methods commonly prevent dyn dispatch unless excluded.
- **Fixture obligation:** Define one iterator-returning and one async trait method, two impls with different hidden types, generic calls, and attempted dyn calls; compare an explicit boxed-future object-safe form.
- **Variants, interactions, counterexamples, and failures:** Preserve returned-future auto-trait commitments and the difficulty of expressing `Send` bounds; separate native behavior from `async_trait` macro rewriting.
- **Constraints:** rustc 1.75+; return-type-notation frontier remains evolving.
- **Classification · citations · provenance · status:** Language-defined plus ecosystem alternative; [`impl Trait` in traits][ref-impl-trait], [Rust 1.75 announcement][rel-175]; `C: RUST-TYPE-011/RUST-ASYNC-004`, `F: RS-CLO-06`, `G: none`; **conditional**.

#### RS-CAN-TYPE-012 — Recursive types and indirection

- **Feature or behavior:** Direct value recursion has infinite size and is rejected; recursion through sized indirection is allowed. Alias, opaque, layout, and drop-check cycles have separate diagnostics.
- **Fixture obligation:** Provide boxed recursive and mutually recursive types plus invalid direct struct, alias, and opaque/layout cycles.
- **Variants, interactions, counterexamples, and failures:** Include DST recursion and a drop-check cycle without relying on exact diagnostic text.
- **Constraints:** Compiler cycle/overflow limits can affect diagnostics.
- **Classification · citations · provenance · status:** Language-defined with implementation diagnostics; [type layout][ref-layout], [recursive types error][rustc-errors]; `C: RUST-TYPE-012`, `F: compilation-limit and type items`, `G: none`; **required**.

#### RS-CAN-TYPE-013 — Representation and layout attributes

- **Feature or behavior:** Default `repr(Rust)` leaves field order, padding, enum encoding, and niches unspecified. `repr(C)`, integer enum reprs, `transparent`, `packed`, and `align` add selected guarantees subject to target ABI.
- **Fixture obligation:** Define comparable forms under every representation, observe size/alignment/offset only where guaranteed, and pass C-compatible forms across FFI.
- **Variants, interactions, counterexamples, and failures:** Invalid combinations, transparent eligibility, packed unaligned-reference failure, generic instances, primitive enum repr, and niche/layout counterexamples must be retained without making default-layout assertions.
- **Constraints:** Target ABI; repr applicability by item kind.
- **Classification · citations · provenance · status:** Language-defined guarantees plus unspecified default/target ABI; [type layout][ref-layout]; `C: RUST-TYPE-013`, `F: RS-UNS-04`, `G: RUST-TYP-003`; **required**.

#### RS-CAN-TYPE-014 — Type inference and target-directed selection

- **Feature or behavior:** Inference flows through expression context; unsuffixed integers/floats fall back, while target type can select implementations for `into`, `parse`, `collect`, and associated items.
- **Fixture obligation:** Use explicit/turbofish and inferred collection targets, multiple `From` choices, numeric fallback, and an unconstrained ambiguity.
- **Variants, interactions, counterexamples, and failures:** Show an annotation changing impl selection, ambiguous numeric methods, associated projection constraints, and no general implicit user conversion.
- **Constraints:** Inference diagnostics are implementation output; semantic outcome follows available bounds.
- **Classification · citations · provenance · status:** Language-defined with compiler inference; [type inference][ref-inference]; `C: RUST-TYPE-014`, `F: RS-TYP-01`, `G: none`; **required**.

#### RS-CAN-TYPE-015 — Coercions and coercion sites

- **Feature or behavior:** A fixed set of implicit coercions occurs only at designated sites and can propagate through arrays, tuples, blocks, and branches.
- **Fixture obligation:** Exercise function-item/pointer, noncapturing-closure/pointer, mutable/shared reference, deref, unsizing, and least-upper-bound branch coercions.
- **Variants, interactions, counterexamples, and failures:** Repeat equivalent expressions outside a coercion site, chain `&Box<String>` to `&str`, unsize smart pointers, and distinguish method receiver adjustments from ordinary function-argument coercion.
- **Constraints:** Supported built-in unsizing and `Deref`/`DerefMut` implementations.
- **Classification · citations · provenance · status:** Language-defined; [type coercions][ref-coercions], [`Deref`][std-deref]; `C: RUST-TYPE-014/RUST-TRAIT-015`, `F: RS-TYP-02`, `G: RUST-TYP-001/002`; **required**.

#### RS-CAN-TYPE-016 — Cast expressions

- **Feature or behavior:** `as` supports a closed set of numeric, enum, pointer, function, and trait-object-related casts; it is not general conversion.
- **Fixture obligation:** Demonstrate widening/truncating numeric, float-to-int, enum-to-integer, raw/fat/thin pointer, and function-pointer casts plus rejected invalid casts.
- **Variants, interactions, counterexamples, and failures:** Preserve saturating float-to-int behavior for current Rust, metadata loss in fat-to-thin cast, address exposure/provenance caveats, and contrast `From`/`TryFrom`/`transmute`.
- **Constraints:** Pointer semantics and exact accepted casts vary at evolving low-level edges.
- **Classification · citations · provenance · status:** Language-defined with provenance caveats; [cast expressions][ref-casts]; `C: RUST-TYPE-015`, `F: RS-TYP/RS-UNS gaps`, `G: unsafe gap`; **required**.

#### RS-CAN-GEN-001 — Generic parameter kinds and defaults

- **Feature or behavior:** Items can bind lifetime, type, and const parameters; ordering, default, and usage rules depend on item context.
- **Fixture obligation:** Define structs, enums, functions, traits, impls, aliases, and associated items using all parameter kinds with legal defaults.
- **Variants, interactions, counterexamples, and failures:** Cover early/late-bound lifetimes, unused type parameters, illegal function defaults, inferred defaults, and parameter shadowing/scope.
- **Constraints:** Stable const parameter types are restricted.
- **Classification · citations · provenance · status:** Language-defined; [generic parameters][ref-generics]; `C: RUST-GEN-001`, `F: RS-ITM-08`, `G: none`; **required**.

#### RS-CAN-GEN-002 — Bounds, where clauses, implied bounds, and HRTBs

- **Feature or behavior:** Bounds establish well-formedness and operations; limited lifetime/trait obligations are implied, while most traits must be explicit. `for<'a>` creates a Higher-Ranked Trait Bound (HRTB).
- **Fixture obligation:** Pair inline and where bounds, associated projection bounds, an implied outlives use, a missing non-implied bound failure, and callbacks valid for every lifetime.
- **Variants, interactions, counterexamples, and failures:** Binder placement/scope, trivial-bound definition checks, monomorphization use checks, `?Sized`, higher-ranked function types, and leak-check failure.
- **Constraints:** Trait solver version may affect edge-case diagnostics.
- **Classification · citations · provenance · status:** Language-defined; [trait and lifetime bounds][ref-bounds]; `C: RUST-GEN-002/008`, `F: RS-ITM-08`, `G: blanket-bound discussion`; **required**.

#### RS-CAN-GEN-003 — Const generics and evaluatability

- **Feature or behavior:** Const arguments participate in type identity. Stable parameter types and generic const expressions are restricted; evaluatability can fail only after a generic is instantiated.
- **Fixture obligation:** Parameterize arrays/types/functions by integer, boolean, and character consts; show distinct instantiations, inferred/explicit args, braces, and compile-time use.
- **Variants, interactions, counterexamples, and failures:** Include const equality, unsupported expressions, post-monomorphization failure, and nightly `generic_const_exprs` as an isolated rejection/conditional case.
- **Constraints:** Channel and compiler version; later inferred-const conveniences need pinning.
- **Classification · citations · provenance · status:** Language-defined with unstable frontier; [const generics][ref-const-generics]; `C: RUST-GEN-003`, `F: RS-ITM-08/09 and version table`, `G: nightly gap`; **required** for minimal const generics, frontier **conditional**.

#### RS-CAN-GEN-004 — Associated types, constants, and projection

- **Feature or behavior:** Traits and impls bind associated types and constants; projections normalize through trait selection, and fully qualified syntax resolves ambiguity.
- **Fixture obligation:** Define bounded associated types/constants with defaults where stable, override them, use shorthand and `<T as Trait>::Assoc`, and constrain equality/bounds.
- **Variants, interactions, counterexamples, and failures:** Cover ambiguous shorthand/constants, projection cycle/overflow, trait-object associated bindings, `Self` in const expressions, and nightly associated-type defaults.
- **Constraints:** Associated-type defaults remain unstable; associated constants make a trait non-dyn-compatible.
- **Classification · citations · provenance · status:** Language-defined; [associated items][ref-associated]; `C: RUST-GEN-004/006`, `F: RS-ITM-06/RS-TRT-09`, `G: trait discussion`; **required**.

#### RS-CAN-GEN-005 — Generic associated types

- **Feature or behavior:** A Generic Associated Type (GAT) binds lifetimes, types, or consts to describe a family of projected types.
- **Fixture obligation:** Implement a lending-style lifetime-parameterized associated type with required `where Self: 'a`, a returned borrow, and a generic consumer.
- **Variants, interactions, counterexamples, and failures:** Include higher-ranked projection bounds, dyn incompatibility, required-bound diagnostics, and known solver/borrow-check limitations.
- **Constraints:** Stable since Rust 1.65; compiler limitations remain version-sensitive.
- **Classification · citations · provenance · status:** Language-defined; [associated types][ref-associated], [Rust 1.65 release][rel-165]; `C: RUST-GEN-005`, `F: RS-TRT-09`, `G: none`; **required**.

#### RS-CAN-GEN-006 — Lifetime elision and trait-object lifetime defaults

- **Feature or behavior:** Function/method elision follows fixed input/output rules; `'_` requests inference. Trait-object default lifetime bounds follow a distinct context-sensitive algorithm.
- **Fixture obligation:** Pair elided and explicit free-function/method signatures, an ambiguous multi-input failure, an inferred `'_` path, and `Box<dyn Trait + '_>` versus default `'static`.
- **Variants, interactions, counterexamples, and failures:** Function item versus closure inference, hidden-lifetime lints, impl headers, and receiver elision.
- **Constraints:** Edition lint defaults can differ; language rules are stable.
- **Classification · citations · provenance · status:** Language-defined; [lifetime elision][ref-lifetime-elision]; `C: RUST-GEN-007`, `F: RS-TYP-05`, `G: implicit lifetime mention`; **required**.

#### RS-CAN-GEN-007 — Variance, subtyping, and `PhantomData`

- **Feature or behavior:** Lifetime/type parameters can be covariant, contravariant, or invariant based on use. `PhantomData` changes variance, ownership/drop-check, and auto-trait relationships without runtime storage.
- **Fixture obligation:** Demonstrate shared/mutable references, function arguments, raw pointers/interior mutability, and multiple phantom forms with accepted/rejected lifetime substitutions.
- **Variants, interactions, counterexamples, and failures:** Include `PhantomData<*const T>` auto-trait effects and ownership-bearing versus non-owning forms.
- **Constraints:** Exact unsafe drop-check frontier includes unstable `may_dangle`.
- **Classification · citations · provenance · status:** Language-defined plus standard marker contract; [subtyping and variance][ref-subtyping], [`PhantomData`][std-phantom]; `C: RUST-GEN-009/RUST-OWN-013`, `F: RS-TYP-03`, `G: none`; **required**.

#### RS-CAN-GEN-008 — Opaque-type precise capture

- **Feature or behavior:** Opaque return types capture generics under edition-sensitive defaults; `use<...>` can state a precise capture set.
- **Fixture obligation:** Compile paired 2021/2024 functions where inferred lifetime capture changes borrow acceptance, then use explicit `use<...>` to control it.
- **Variants, interactions, counterexamples, and failures:** Capture lifetimes/types/consts; empty and nonempty sets; restrictions in trait methods, nested opaque types, and argument-position forms.
- **Constraints:** Precise capture is stable from Rust 1.82; 2024 default capture requires 1.85; exact edge restrictions must be version-pinned.
- **Classification · citations · provenance · status:** Language-defined; [precise capture][edition-2024-rpit], [Rust 1.82 release][rel-182]; `C: RUST-GEN-010`, `F: RS-CLO-08`, `G: edition changes`; **conditional**.

### Traits, implementations, dispatch, ownership, and borrowing

#### RS-CAN-TRAIT-001 — Inherent and trait implementations

- **Feature or behavior:** Inherent impls attach items to a local nominal type; trait impls establish a crate-global relationship independent of the impl module's visibility. Multiple inherent blocks are permitted.
- **Fixture obligation:** Place inherent blocks in separate modules and a trait impl in a private module; use same-spelled inherent/trait methods from elsewhere.
- **Variants, interactions, counterexamples, and failures:** Reject an inherent impl for a foreign type, duplicate inherent items, and duplicate trait impls. Show that impl blocks are unnamed and trait impl availability does not require importing their defining module.
- **Constraints:** Locality and coherence.
- **Classification · citations · provenance · status:** Language-defined; [implementations][ref-impls]; `C: RUST-TRAIT-001`, `F: RS-ITM-07/RS-TRT-01`, `G: trait section`; **required**.

#### RS-CAN-TRAIT-002 — Coherence, orphan coverage, and overlap

- **Feature or behavior:** A trait impl must be globally unique and generally requires the trait or a sufficiently uncovered nominal type to be local. Fundamental types affect coverage.
- **Fixture obligation:** Include legal local-trait/foreign-type and foreign-trait/local-type impls, covered generic wrappers, fundamental `Box`/reference cases, a newtype workaround, and illegal foreign-for-foreign and overlapping impls.
- **Variants, interactions, counterexamples, and failures:** Parameter order, upstream blanket impl hazards, positive/negative conflict, and semver impact of new upstream impls.
- **Constraints:** Exact orphan coverage is language-defined but subtle; specialization/incoherent extensions are unstable.
- **Classification · citations · provenance · status:** Language-defined; [coherence][ref-coherence]; `C: RUST-TRAIT-002`, `F: RS-TRT-04`, `G: RUST-TRT-001`; **required**.

#### RS-CAN-TRAIT-003 — Conditional and blanket implementations

- **Feature or behavior:** Bounds make an impl apply only to satisfying instantiations; blanket impls cover an open set and can prevent later specific impls.
- **Fixture obligation:** Implement a conditional trait for a wrapper and a blanket extension trait across local and standard types, with satisfying and nonsatisfying call sites.
- **Variants, interactions, counterexamples, and failures:** Deliberate overlap, derive-generated bound interaction, downstream automatic membership, auto-ref method candidates, and feature-gated impl presence.
- **Constraints:** Coherence and selected cfg/features.
- **Classification · citations · provenance · status:** Language-defined; [implementations][ref-impls], [coherence][ref-coherence]; `C: RUST-TRAIT-003`, `F: RS-TRT-02/03`, `G: blanket-candidate note`; **required**.

#### RS-CAN-TRAIT-004 — Supertraits, defaults, and overrides

- **Feature or behavior:** A subtrait requires its supertraits. Trait methods/constants can provide defaults; impl overrides select a different executed body.
- **Fixture obligation:** Build a supertrait hierarchy and two impls, one inheriting and one overriding a default that calls a required method.
- **Variants, interactions, counterexamples, and failures:** Missing supertrait impl, cycles, associated constraints, direct qualified calls, object dispatch to overrides, and proof that a supertrait relation does not imply the reverse impl.
- **Constraints:** Default associated-type definitions remain unstable; stable method/constant defaults are required.
- **Classification · citations · provenance · status:** Language-defined; [traits][ref-traits]; `C: RUST-TRAIT-004/005`, `F: RS-ITM-06/RS-TRT-07`, `G: none`; **required**.

#### RS-CAN-TRAIT-005 — Receiver forms and stable self-type boundaries

- **Feature or behavior:** Receiver form controls move/borrow, lookup, and dyn dispatch. Stable receivers include common reference/ownership and supported smart-pointer forms; arbitrary self types extend this only on nightly.
- **Fixture obligation:** Define methods with `self`, `&self`, `&mut self`, `Box<Self>`, `Pin<&mut Self>`, and no receiver.
- **Variants, interactions, counterexamples, and failures:** Consuming dyn methods needing `Self: Sized`, receiver aliases, unsupported arbitrary receiver, and method candidate changes by receiver type.
- **Constraints:** Feature/channel and receiver shape.
- **Classification · citations · provenance · status:** Language-defined stable core; [associated functions and methods][ref-associated]; `C: RUST-TRAIT-006`, `F: RS-TRT-12 nightly boundary`, `G: none`; **required** core, **conditional** frontier.

#### RS-CAN-TRAIT-006 — Method lookup, autoderef, autoref, and unsizing

- **Feature or behavior:** Method resolution builds ordered receiver candidates through dereference, borrowing, and one unsizing step, then searches inherent and in-scope trait methods.
- **Fixture obligation:** Use a custom deref chain, methods at several levels, `T` versus `&T` impls, mutable/shared candidates, and an array-to-slice receiver.
- **Variants, interactions, counterexamples, and failures:** Show an earlier `&self` trait candidate beating a later `&mut self` candidate, wrapper inherent shadowing, trait-not-in-scope failure, and associated functions not flowing through `Deref`.
- **Constraints:** Edition-sensitive array method special case; compiler follows the Reference candidate order.
- **Classification · citations · provenance · status:** Language-defined; [method calls][ref-method-call], [`Deref`][std-deref]; `C: RUST-TRAIT-007`, `F: RS-DSP-01/05/06`, `G: RUST-TYP-001/002`; **required**.

#### RS-CAN-TRAIT-007 — Ambiguity and fully qualified syntax

- **Feature or behavior:** Same-named applicable trait items can make a call/path ambiguous; trait/type qualification selects the intended impl or item.
- **Fixture obligation:** Import two traits with one method, retain the ambiguous call, and resolve it with `Trait::method` and `<Type as Trait>::method`; repeat for constants/types/functions.
- **Variants, interactions, counterexamples, and failures:** Inherent-versus-trait collision, ambiguity introduced only by an import, associated type shorthand ambiguity, and `Trait as _` participation.
- **Constraints:** Candidate traits and bounds in the use-site scope.
- **Classification · citations · provenance · status:** Language-defined; [qualified paths][ref-paths], [method calls][ref-method-call]; `C: RUST-MOD-012/RUST-TRAIT-001`, `F: RS-DSP-02`, `G: RUST-TRT-002`; **required**.

#### RS-CAN-TRAIT-008 — Static and dynamic dispatch

- **Feature or behavior:** Generic bounds and argument-position `impl Trait` normally dispatch statically through monomorphization; trait objects dispatch dynamically through a vtable.
- **Fixture obligation:** Consume the same trait through a generic function, `impl Trait`, `&dyn Trait`, boxed objects, and a heterogeneous object collection.
- **Variants, interactions, counterexamples, and failures:** Dyn-compatible methods only, open versus concrete target sets, potential devirtualization without semantic change, and unspecified vtable layout.
- **Constraints:** Auto-trait/lifetime/associated bindings form object identity.
- **Classification · citations · provenance · status:** Language-defined semantics plus implementation codegen; [trait objects][ref-trait-objects], [monomorphization][rustc-mono]; `C: RUST-TRAIT-008`, `F: RS-DSP-03`, `G: none`; **required**.

#### RS-CAN-TRAIT-009 — Auto traits and manual unsafe overrides

- **Feature or behavior:** `Send`, `Sync`, `Unpin`, and unwind-safety auto traits are inferred structurally; permitted unsafe positive impls can override structural absence, while negative/custom auto traits are generally unstable.
- **Fixture obligation:** Make composite types inherit and fail `Send`/`Sync`, let a generic type depend on `T`, change inference with a marker/raw-pointer field, and provide a justified `unsafe impl`.
- **Variants, interactions, counterexamples, and failures:** Dyn object auto-trait sets, spawn failure, `Unpin` and pinned futures, explicit nightly negative impl, and field-based stable suppression.
- **Constraints:** Standard auto traits stable; custom auto traits and most negative impls require nightly.
- **Classification · citations · provenance · status:** Language-defined plus standard traits; [special traits][ref-special-traits], [`Send` and `Sync`][std-send-sync]; `C: RUST-TRAIT-009/010`, `F: RS-TRT-05/12`, `G: common-traits section`; **required** standard behavior, **conditional** nightly cases.

#### RS-CAN-TRAIT-010 — Operator, index, deref, and formatting traits

- **Feature or behavior:** Operators and formatting map fixed syntax to standard traits with language-specific evaluation/place rules; no user-defined operators exist.
- **Fixture obligation:** Implement arithmetic, assignment, comparison, indexing/mutable indexing, dereference, and display/debug formatting on user types and invoke them only through syntax.
- **Variants, interactions, counterexamples, and failures:** Different output types, reference operands, short-circuit operators not overloadable, stable lack of custom callable impls, missing formatting trait, deref impact on methods/coercions, and indexing bounds failure.
- **Constraints:** Standard `ops`/`fmt` contracts and operator syntax.
- **Classification · citations · provenance · status:** Language-defined plus standard library; [operator expressions][ref-operators], [`std::ops`][std-ops], [`std::fmt`][std-fmt]; `C: RUST-TRAIT-011/RUST-EXPR-010`, `F: RS-TRT-10/RS-IMP-04`, `G: standard-trait convention`; **required**.

#### RS-CAN-TRAIT-011 — Explicit conversion traits

- **Feature or behavior:** `From`/`Into`, `TryFrom`/`TryInto`, and naming families such as `as_`/`to_`/`into_` represent explicit library/ecosystem conversions, not language coercions.
- **Fixture obligation:** Implement infallible and fallible conversion, use reciprocal blanket `Into`, drive a generic bound, and retain an invalid implicit assignment.
- **Variants, interactions, counterexamples, and failures:** Associated error type, coherence/blanket interactions, `?` error conversion, `as` separation, and borrowed/owned method naming conventions.
- **Constraints:** Trait semantics are standard library; method names are convention only.
- **Classification · citations · provenance · status:** Language trait resolution plus ecosystem convention; [conversion traits][std-convert], [Rust API Guidelines naming][api-naming]; `C: RUST-TRAIT-012`, `F: RS-ECO-02 and conversion coverage`, `G: RUST-ECO-002`; **required** trait behavior, naming convention **conditional**.

#### RS-CAN-TRAIT-012 — `Drop`, drop glue, and `Copy`/`Clone`

- **Feature or behavior:** Compiler drop glue recursively schedules destructors; `Drop::drop` cannot be called directly. `Copy` changes moves to implicit bitwise copies and is incompatible with `Drop`; `Clone` is explicit and may differ.
- **Fixture obligation:** Observe nested/local/field drop order, explicit `drop`, partial move/initialization, `ManuallyDrop`, `forget`, Copy and non-Copy use-after-assignment, and manual versus derived Clone.
- **Variants, interactions, counterexamples, and failures:** Direct destructor call, `Copy + Drop` rejection, derive bounds, mutable versus shared references, abort/no-unwind, panic while unwinding, and moved values not dropped.
- **Constraints:** Temporary/drop scopes are separately edition-sensitive.
- **Classification · citations · provenance · status:** Language-defined plus standard library; [destructors][ref-destructors], [`Drop`][std-drop], [`Copy`][std-copy]; `C: RUST-TRAIT-013/014`, `F: RS-IMP-05/RS-ECO-001`, `G: RUST-ECO-001`; **required**.

#### RS-CAN-TRAIT-013 — Derive-generated standard impls and bounds

- **Feature or behavior:** Built-in derives generate trait impls and inferred generic bounds; generated bounds may be stronger than a hand-written impl.
- **Fixture obligation:** Derive Debug, Clone, Copy, equality/order/hash/default across generic structs/enums and compare one manual impl that avoids an unnecessary parameter bound.
- **Variants, interactions, counterexamples, and failures:** `Copy: Clone`, `Eq: PartialEq`, per-variant behavior, duplicate manual/derived impl, and cfg-conditioned derive.
- **Constraints:** Exact expansion/bounds are compiler/standard-derive behavior.
- **Classification · citations · provenance · status:** Implementation-defined language-coupled derive behavior; [derive attribute][ref-derive], [standard derive macros][std-derive]; `C: RUST-MAC-004/RUST-TRAIT-014`, `F: RS-TRT-06`, `G: RUST-ECO-001`; **required**.

#### RS-CAN-TRAIT-014 — Specialization, trait aliases, and negative impl frontier

- **Feature or behavior:** Specialization, trait aliases, general negative impls, custom auto traits, and associated-type defaults change impl selection or identity but remain nightly-only at the reports' cutoffs.
- **Fixture obligation:** Keep minimal channel-pinned examples outside the stable build and stable approximations using supertraits/blanket/nonoverlapping impls.
- **Variants, interactions, counterexamples, and failures:** Stable feature-gate failures, semver commitments, incomplete specialization eligibility, trait alias not being an ordinary new implementable trait, and ecosystem autoref-dispatch tricks.
- **Constraints:** Exact nightly toolchain and feature gates.
- **Classification · citations · provenance · status:** Implementation-defined unstable features; [Unstable Book][unstable-book]; `C: RUST-TRAIT-010/016`, `F: RS-TRT-12`, `G: nightly research gap`; **conditional**.

#### RS-CAN-OWN-001 — Moves, move paths, and partial moves

- **Feature or behavior:** Non-`Copy` values move by value; fields can move independently, leaving only unaffected paths usable, subject to destructor and control-flow checks.
- **Fixture obligation:** Destructure a struct while moving one field and borrowing/copying another, then use remaining fields and retain whole-value failure.
- **Variants, interactions, counterexamples, and failures:** `Drop` type field-move restriction, moves through dereference, closure captures, branch-dependent initialization, and explicit copies.
- **Constraints:** Borrow/move analysis and field `Copy`/`Drop` properties.
- **Classification · citations · provenance · status:** Language-defined; [place/value expressions][ref-place], [destructors][ref-destructors]; `C: RUST-OWN-001`, `F: RS-IMP-06/RS-ECO-001`, `G: Copy variant`; **required**.

#### RS-CAN-OWN-002 — Borrow exclusivity and reborrowing

- **Feature or behavior:** A live mutable borrow excludes other access; a live shared borrow excludes mutation. Reborrows can shorten access and safe borrowing ends by actual use.
- **Fixture obligation:** Include accepted nonoverlapping reborrows, conflicting simultaneous borrows, mutation after last shared use, and mutable-to-shared reborrow.
- **Variants, interactions, counterexamples, and failures:** Interior mutability, raw-pointer aliasing that compiles but may be undefined, deref/index places, and borrow across closure/async boundaries.
- **Constraints:** Core language contract; unsafe alias model has unresolved edges.
- **Classification · citations · provenance · status:** Language-defined with unsafe-model caveats; [borrow expressions][ref-borrow], [undefined behavior][ref-ub]; `C: RUST-OWN-002`, `F: audit omitted borrow checking but related cases`, `G: unsafe gap`; **required**.

#### RS-CAN-OWN-003 — Non-lexical lifetimes and two-phase borrows

- **Feature or behavior:** Non-Lexical Lifetimes (NLL) use control-flow liveness; selected compiler-inserted mutable autoref borrows reserve before activation and permit argument evaluation that explicit `&mut` would reject.
- **Fixture obligation:** Mutate after the last borrow use in one block, retain a branch where it stays live, and compare `collection.push(collection.len())`-style method borrowing with explicit mutable borrowing.
- **Variants, interactions, counterexamples, and failures:** Loops, nested calls, operator desugaring, activation points, and experimental Polonius behavior.
- **Constraints:** Two-phase borrowing applies only to defined implicit borrow categories; Polonius is not baseline.
- **Classification · citations · provenance · status:** Implementation-defined realization of language borrowing; [NLL][edition-2018-nll], [rustc borrow checker][rustc-borrow]; `C: RUST-OWN-003/004`, `F: audit mentions NLL`, `G: none`; **required**.

#### RS-CAN-OWN-004 — Temporary lifetime extension and drop scopes

- **Feature or behavior:** Temporaries drop at syntax-defined scopes; selected `let`, const, and static contexts extend lifetimes. Edition 2024 narrows `if let` and tail-expression temporary scopes.
- **Fixture obligation:** Observe extending and nonextending borrows, match scrutinee, `if let`, block tail, function return, and destructor order in paired 2021/2024 crates.
- **Variants, interactions, counterexamples, and failures:** Different `let _` forms, lock-guard temporary hazards, promotion versus extension, let-chain condition temporaries, and async suspension.
- **Constraints:** Edition and exact syntactic form.
- **Classification · citations · provenance · status:** Language/edition-defined; [destructors and scopes][ref-destructors], [2024 temporary scopes][edition-2024-temp]; `C: RUST-OWN-005`, `F: RS-IMP-05/RS-EDI-04`, `G: edition-change summary`; **required**.

#### RS-CAN-OWN-005 — Interior mutability

- **Feature or behavior:** `UnsafeCell` permits mutation behind shared references and underpins cells, dynamic borrow checking, synchronization, and atomics.
- **Fixture obligation:** Use `Cell`, successful and panicking `RefCell` borrows, and a minimal sound `UnsafeCell` abstraction.
- **Variants, interactions, counterexamples, and failures:** `RefCell` thread unsafety, mutex poisoning as library behavior, niche/layout effects, and aliasing outside the cell remaining undefined.
- **Constraints:** Wrapper-specific runtime/thread guarantees.
- **Classification · citations · provenance · status:** Language special type plus standard library; [`UnsafeCell`][std-unsafecell], [`RefCell`][std-refcell]; `C: RUST-OWN-006`, `F: variance/global-state related items`, `G: none`; **required**.

#### RS-CAN-OWN-006 — Closure capture precision and mode

- **Feature or behavior:** Closures infer shared, unique immutable, mutable, or by-value captures and can capture disjoint places. `move` requests by-value capture but does not alone choose `FnOnce`.
- **Fixture obligation:** Cause every capture mode, disjoint field capture, a move closure, and subsequent use of uncaptured or partially captured values.
- **Variants, interactions, counterexamples, and failures:** Packed structs, special `Box` dereference capture, edition-2018 versus 2021 whole/disjoint capture, drop order, closure size/lifetime, and auto-trait changes.
- **Constraints:** Edition 2021 changed capture precision.
- **Classification · citations · provenance · status:** Language/edition-defined; [closure capture][ref-closures], [2021 disjoint capture][edition-2021-capture]; `C: RUST-OWN-007`, `F: RS-IMP-06`, `G: edition-change summary`; **required**.

#### RS-CAN-OWN-007 — `Fn`, `FnMut`, and `FnOnce`

- **Feature or behavior:** Closure body use determines implemented call traits: every closure is `FnOnce`, while reusable mutation/read behavior can add `FnMut`/`Fn`.
- **Fixture obligation:** Pass reading, mutating, and consuming closures to matching generic functions and call them the permitted number of times.
- **Variants, interactions, counterexamples, and failures:** Move closures that still implement `Fn`, noncapturing pointer coercion, Copy/Clone closure traits, and passing a consuming closure where `Fn` is required.
- **Constraints:** Call traits are compiler-implemented special traits.
- **Classification · citations · provenance · status:** Language-defined plus standard special traits; [closure types][ref-closures], [`Fn` traits][std-fn]; `C: RUST-OWN-008`, `F: RS-CLO-01`, `G: none`; **required**.

#### RS-CAN-OWN-008 — Pattern binding modes and match ergonomics

- **Feature or behavior:** Patterns move, copy, or borrow subvalues; default binding modes adjust through references. Rust 2024 restricts explicit `ref`, `ref mut`, `mut`, and reference patterns under inherited modes.
- **Fixture obligation:** Match one structure as owned/shared/mutable reference with inferred and explicit binding types in 2021 and 2024 variants.
- **Variants, interactions, counterexamples, and failures:** Partial moves, `@` bindings, nested references, or-pattern agreement, and 2024 reservation diagnostics.
- **Constraints:** Edition and scrutinee type.
- **Classification · citations · provenance · status:** Language/edition-defined; [patterns][ref-patterns], [2024 match ergonomics][edition-2024-match]; `C: RUST-OWN-009`, `F: final audit flags pattern depth/RS-EDI-04`, `G: edition summary`; **required**.

#### RS-CAN-OWN-009 — Refutability, or-patterns, guards, and exhaustiveness

- **Feature or behavior:** Contexts accept either irrefutable or refutable patterns; or-pattern alternatives must bind consistently; guards do not contribute to exhaustiveness.
- **Fixture obligation:** Use patterns in `let`, parameters, `if let`, `while let`, `let-else`, and match arms, with enum/tuple/slice/range/or forms and guarded arms.
- **Variants, interactions, counterexamples, and failures:** Irrefutability error/lint, diverging `let-else`, inconsistent or bindings, guard borrow/move behavior, wildcard after guarded coverage, unreachable and nonexhaustive arms.
- **Constraints:** Compiler exhaustiveness handling of uninhabited types can evolve.
- **Classification · citations · provenance · status:** Language-defined plus compiler analysis; [patterns][ref-patterns], [match expressions][ref-match]; `C: RUST-OWN-010/011/RUST-EXPR-004`, `F: final audit flags pattern depth`, `G: none`; **required**.

#### RS-CAN-OWN-010 — Destructuring assignment and drop check

- **Feature or behavior:** Destructuring assignment writes existing places rather than binding names. Generic destructor checking and `PhantomData` constrain whether borrowed data may outlive/drop safely.
- **Fixture obligation:** Perform nested tuple/struct/array/slice assignment and contrast a pattern legal only in `let`; create generic wrappers with/without suitable phantom ownership and a `Drop` impl.
- **Variants, interactions, counterexamples, and failures:** `_`, field shorthand, evaluation/drop order, illegal binding/refutable forms, borrow rejected by destruction order, and unstable `may_dangle` boundary.
- **Constraints:** Destructuring assignment requires supported stable compiler; drop-check is compiler analysis.
- **Classification · citations · provenance · status:** Language-defined; [destructuring assignment][ref-destructuring], [destructors][ref-destructors]; `C: RUST-OWN-012/013`, `F: PhantomData coverage`, `G: none`; **required**.

### Expressions, control flow, constants, and statics

#### RS-CAN-EXPR-001 — Place, value, and assignee contexts

- **Feature or behavior:** Expression context determines whether syntax denotes a place or value and whether it moves, copies, borrows, assigns, or drops.
- **Fixture obligation:** Use locals, statics, dereferences, fields, and indexes in value, borrow, assignment, and compound-assignment contexts.
- **Variants, interactions, counterexamples, and failures:** Parentheses preserving place context, overloaded deref/index, temporary promotion, invalid assignment targets, and different move/copy outcomes.
- **Constraints:** Type and operator impls.
- **Classification · citations · provenance · status:** Language-defined; [place expressions][ref-place]; `C: RUST-EXPR-001`, `F: RS-IMP-04/05`, `G: none`; **required**.

#### RS-CAN-EXPR-002 — Blocks, statements, tails, and semicolons

- **Feature or behavior:** Blocks are expressions whose tail supplies type/value; semicolons discard values; item statements declare without runtime execution.
- **Fixture obligation:** Demonstrate value and unit blocks, nested item statements, semicolon-sensitive return types, and tail temporary destruction.
- **Variants, interactions, counterexamples, and failures:** Unsafe/async/const blocks, semicolon inference after flow/item expressions, and 2024 tail-scope differences.
- **Constraints:** Edition for temporary scope; specialized block kind.
- **Classification · citations · provenance · status:** Language-defined; [blocks][ref-blocks], [statements][ref-statements]; `C: RUST-EXPR-002`, `F: item/const/async sections`, `G: none`; **required**.

#### RS-CAN-EXPR-003 — Conditional expressions, `if let`, and let chains

- **Feature or behavior:** Conditions require `bool`; branch values coerce to one type. Conditional patterns bind in successful regions; let chains combine pattern tests and booleans.
- **Fixture obligation:** Include value-producing `if`, scoped `if let`, incompatible branch error, and a version-pinned let chain with short-circuit/binding behavior.
- **Variants, interactions, counterexamples, and failures:** 2024 `if let` temporary rescoping, while-let chain where supported, invalid older compiler/edition, and bindings unavailable after a failed chain segment.
- **Constraints:** Let chains require Rust 1.88+ and edition 2024 according to Fable's cited release inventory; stable 1.85 must retain rejection.
- **Classification · citations · provenance · status:** Language-defined; [if expressions][ref-if], [Rust 1.88 release][rel-188]; `C: RUST-EXPR-003 (stabilization unresolved)`, `F: RS-EDI-04/version table`, `G: baseline only`; **conditional**.

#### RS-CAN-EXPR-004 — Loops, labels, iteration, and value-bearing break

- **Feature or behavior:** `loop` can yield compatible `break` values; `while`/`for` yield unit. Labels select nested loops/blocks. `for` uses `IntoIterator` and repeated `Iterator::next` with defined temporary handling.
- **Fixture obligation:** Use nested labeled loops/blocks, value-returning loop, continue, invalid valued break from while/for, and custom owned/shared/mutable `IntoIterator` impls.
- **Variants, interactions, counterexamples, and failures:** Diverging loop, label shadowing, closure/async control-boundary errors, refutable loop-pattern rejection, and edition-sensitive array iteration.
- **Constraints:** Edition for array `.into_iter()` compatibility.
- **Classification · citations · provenance · status:** Language-defined plus standard iterator traits; [loop expressions][ref-loops], [2021 array iteration][edition-2021-intoiter]; `C: RUST-EXPR-005/006`, `F: RS-IMP-01/RS-DSP-06`, `G: none`; **required**.

#### RS-CAN-EXPR-005 — Return, divergence, panic, and unreachable code

- **Feature or behavior:** Return/break/continue/panic/nonreturning calls diverge and can coerce; unreachable-code linting is distinct from type validity.
- **Fixture obligation:** Use early returns, infinite loop, panic macro, `-> !`, and unreachable code in functions and closures.
- **Variants, interactions, counterexamples, and failures:** Closure return target, never fallback, panic abort/unwind, FFI nonreturning convention, and lint-level build failure.
- **Constraints:** Profile, ABI, and version-sensitive never fallback.
- **Classification · citations · provenance · status:** Language-defined plus diagnostics; [return expressions][ref-return], [never type][ref-never], [rustc lints][rustc-lints]; `C: RUST-EXPR-007`, `F: RS-TYP-06/RS-ATT-02`, `G: none`; **required**.

#### RS-CAN-EXPR-006 — `?`, residuals, and conversions

- **Feature or behavior:** `?` branches through `Try` and converts a residual through `FromResidual`; with `Result`, error conversion commonly selects `From` from the enclosing return type.
- **Fixture obligation:** Use `Result` with error conversion, `Option`, `main -> Result`, nested closure/async contexts, and incompatible residual/missing conversion failures.
- **Variants, interactions, counterexamples, and failures:** Mixing `Option`/`Result`, user `Try` impl rejection, unstable try blocks, and generated `From` impl from an ecosystem derive.
- **Constraints:** Stable use on standard types; extensibility remains unstable.
- **Classification · citations · provenance · status:** Language-defined plus standard traits; [question-mark expressions][ref-question], [`Try`][std-try]; `C: RUST-EXPR-008`, `F: RS-IMP-02`, `G: standard-trait convention`; **required**.

#### RS-CAN-EXPR-007 — Ranges, indexing, and slicing

- **Feature or behavior:** Range syntax produces range values in expressions and interval tests in patterns; indexing is a place/value operation routed through built-in or `Index`/`IndexMut` behavior.
- **Fixture obligation:** Exercise every range form, allowed integer/char range patterns, built-in/user mutable indexing, and range slicing.
- **Variants, interactions, counterexamples, and failures:** Empty/invalid/float pattern ranges, inclusive iteration edge, compile-time unconditional-panic lint, runtime bounds panic, UTF-8 string no-integer-indexing, and unsafe unchecked alternative.
- **Constraints:** Range pattern constant/type restrictions; library indexing behavior.
- **Classification · citations · provenance · status:** Language-defined plus standard library; [range expressions][ref-ranges], [index expressions][ref-index]; `C: RUST-EXPR-009/010`, `F: operator coverage`, `G: none`; **required**.

#### RS-CAN-EXPR-008 — Evaluation order, short circuit, and compound assignment

- **Feature or behavior:** Rust specifies operand evaluation order for relevant expression forms; boolean operators short-circuit, and compound assignment evaluates a place according to specialized rules.
- **Fixture obligation:** Record side effects across tuples, arrays, calls, methods, binary operands, assignment, short-circuit conditions, and indexed compound assignment.
- **Variants, interactions, counterexamples, and failures:** Macro-produced syntax, overloaded operators, temporaries, panic during an operand, and optimizer preservation absent undefined behavior.
- **Constraints:** Exact assignment operand order must follow current Reference text, not intuition.
- **Classification · citations · provenance · status:** Language-defined; [expression evaluation order][ref-expressions], [operator expressions][ref-operators]; `C: RUST-EXPR-011`, `F: RS-IMP-04/05`, `G: none`; **required**.

#### RS-CAN-EXPR-009 — Literals, suffixes, escapes, and fallback

- **Feature or behavior:** Numeric, char, string, byte, raw-string, and C-string literals have distinct token, type, escape, suffix, inference, and overflow rules.
- **Fixture obligation:** Include bases/separators/suffixes, inferred numerics, boundary negative literal, raw hashes, Unicode/byte chars and strings, and `c"..."` where supported.
- **Variants, interactions, counterexamples, and failures:** Out-of-range/invalid escape/scalar diagnostics, negation parsing, C string producing `&CStr`, and older compiler rejection.
- **Constraints:** C string literals stable since Rust 1.77; overflow lint/profile affects numeric diagnostics/runtime.
- **Classification · citations · provenance · status:** Language-defined; [literals][ref-literals], [Rust 1.77 release][rel-177]; `C: RUST-EXPR-012`, `F: version inventory`, `G: none`; **required**, C-string variant **conditional** by version.

#### RS-CAN-CONST-001 — Constants, inline const, and statics

- **Feature or behavior:** A const is an inlined value without stable address identity; inline const creates a capturing const-evaluation context. A static is one storage location; mutable static access is unsafe.
- **Fixture obligation:** Define free/associated/unnamed constants, generic inline const, immutable/interior-mutable/mutable statics, repeated address observations, and compile-time assertion.
- **Variants, interactions, counterexamples, and failures:** Nested const cannot capture outer generic, promotion can create static storage, static initialization must be const, Sync requirement, and edition-2024 mutable-static-reference denial.
- **Constraints:** Inline const stable since 1.79; static-mut diagnostics edition-sensitive.
- **Classification · citations · provenance · status:** Language-defined; [constant items][ref-const-items], [static items][ref-static-items], [Rust 1.79 release][rel-179]; `C: RUST-CONST-001/002`, `F: RS-ITM-04/09`, `G: none`; **required**.

#### RS-CAN-CONST-002 — Const functions and evaluation failures

- **Feature or behavior:** A `const fn` can run at compile time in required const contexts and at runtime elsewhere; only evaluated paths must satisfy const rules, and accepted operations expand by compiler version.
- **Fixture obligation:** Call a const fn in array length, discriminant, static, const generic, inline const, and runtime code; preserve a permitted untaken branch and a failing evaluated branch.
- **Variants, interactions, counterexamples, and failures:** Compile-time panic/overflow, generic failure after instantiation, allocation/drop/trait-call frontier, and version-gated mutable references in const.
- **Constraints:** rustc const-stability version/channel.
- **Classification · citations · provenance · status:** Language-defined surface plus evolving implementation evaluator; [const evaluation][ref-const-eval]; `C: RUST-CONST-003`, `F: RS-ITM-09/version table`, `G: none`; **required** stable core, frontier **conditional**.

#### RS-CAN-CONST-003 — Static promotion and thread-local storage

- **Feature or behavior:** Selected borrowed rvalues are promoted to hidden static storage; promotion differs from temporary lifetime extension. Thread-local declarations create per-thread storage identity through stable library macro or unstable attribute.
- **Fixture obligation:** Contrast a promotable immutable borrow with interior-mutating, destructor-bearing, and runtime-dependent values; demonstrate stable thread-local storage.
- **Variants, interactions, counterexamples, and failures:** Const-evaluable but unpromoted values, address assumptions, per-thread identity, unstable `#[thread_local]`, and linker sections.
- **Constraints:** Promotion rules and thread/platform support.
- **Classification · citations · provenance · status:** Language/compiler-defined plus standard library; [const evaluation][ref-const-eval], [`thread_local!`][std-thread-local]; `C: RUST-CONST-002/004`, `F: RS-ITM-04`, `G: none`; **required** promotion, TLS **conditional**.

### Macros, attributes, and conditional compilation

#### RS-CAN-MAC-001 — Declarative matching, fragments, and repetition

- **Feature or behavior:** `macro_rules!` matches token trees using fragment kinds and repetitions, then transcribes syntax; it has limited lookahead and edition-dependent fragment grammars.
- **Fixture obligation:** Use multiple arms, semantic fragment families, nested repetitions with separators, and item/expression/statement/pattern output.
- **Variants, interactions, counterexamples, and failures:** Local ambiguity, no matching arm, repetition nesting mismatch, follow-set violation, recursive expansion limit, 2021 `pat`/`pat_param`, and 2024 `expr`/`expr_2021` differences.
- **Constraints:** Fragment semantics use the macro definition crate's edition.
- **Classification · citations · provenance · status:** Language-defined; [macros by example][ref-macro-rules], [2024 macro fragments][edition-2024-macro]; `C: RUST-MAC-001`, `F: RS-MAC-01/RS-EDI-03/04`, `G: RUST-MAC-001`; **required**.

#### RS-CAN-MAC-002 — Declarative scope, export, import, and shadowing

- **Feature or behavior:** Unqualified macro resolution combines textual and path-based scope. `macro_export`, legacy `macro_use`, ordinary imports/re-exports, and later declarations expose or shadow macros differently.
- **Fixture obligation:** Use a macro before/after textual declaration, from a child module, through a qualified/re-exported path, cross-crate export, and 2015 `macro_use` import.
- **Variants, interactions, counterexamples, and failures:** Export appears at crate root; `local_inner_macros`; same-name imported/textual ambiguity; ordering-sensitive validity; derive/attribute/bang namespace collisions.
- **Constraints:** Edition and macro kind.
- **Classification · citations · provenance · status:** Language-defined; [macro scope][ref-macro-scope]; `C: RUST-MAC-003`, `F: RS-MAC-02/10`, `G: RUST-MAC-001`; **required**.

#### RS-CAN-MAC-003 — Declarative hygiene and `$crate`

- **Feature or behavior:** `macro_rules!` uses mixed-site hygiene; definition-site locals/labels and invocation-site items resolve differently. `$crate` identifies the defining crate but does not bypass visibility.
- **Fixture obligation:** Generate a noncolliding local, accept an explicit identifier that becomes caller-visible, reference an invocation-site item, and call a public helper through `$crate` from a downstream re-export.
- **Variants, interactions, counterexamples, and failures:** Private helper failure, helper macro paths, nested expansions, and expansion-context identity despite same spelling.
- **Constraints:** Some span/diagnostic mapping is implementation-defined.
- **Classification · citations · provenance · status:** Language-defined core; [macro hygiene][ref-macro-hygiene]; `C: RUST-MAC-002`, `F: RS-MAC-03/12`, `G: macro hygiene discussion`; **required**.

#### RS-CAN-MAC-004 — Macro-generated named entities

- **Feature or behavior:** Declarative/procedural macros can generate structs, modules, impls, statics, tests, and companion types absent from handwritten source.
- **Fixture obligation:** Generate a family of named items and impls from a list; reference output from ordinary code; include one stable identifier-concatenation proc macro or derive-generated companion type.
- **Variants, interactions, counterexamples, and failures:** Duplicate output collision, configuration-dependent output, hidden `$crate::__private` support API, generated private/public distinction, and nightly `concat_idents!` exclusion.
- **Constraints:** Exact macro/dependency version; identifier concatenation requires a stable proc-macro helper.
- **Classification · citations · provenance · status:** Language-defined mechanism plus ecosystem convention; [macros][ref-macro-rules], [procedural macros][ref-proc-macro]; `C: RUST-ECO-001/002/003`, `F: RS-MAC-04/12/RS-ECO-06`, `G: RUST-MAC-002`; **required** mechanism, companion dependency **conditional**.

#### RS-CAN-MAC-005 — Derive macros and helper attributes

- **Feature or behavior:** Derive macros add items, usually impls; registered helper attributes are inert to the compiler and interpreted by derives.
- **Fixture obligation:** Apply built-in and custom derives to generic input, use helper attributes to change output, and consume the generated impl through a bound.
- **Variants, interactions, counterexamples, and failures:** Unknown helper without derive, helper-name collision, duplicate/conflicting impl, broad generated bounds, cfg-conditioned derive, and nonassumed derive ordering.
- **Constraints:** Proc-macro crate and dependency version.
- **Classification · citations · provenance · status:** Language-defined mechanism plus implementation/ecosystem output; [derive macros][ref-proc-macro]; `C: RUST-MAC-004`, `F: RS-MAC-06`, `G: RUST-MAC-002`; **required**.

#### RS-CAN-MAC-006 — Attribute and function-like procedural transformation

- **Feature or behavior:** Attribute macros replace annotated token streams; function-like macros replace invocations at allowed positions and may accept non-Rust domain syntax.
- **Fixture obligation:** Transform/preserve/remove a function or impl/trait, generate both item and expression output, and reference generated entities.
- **Variants, interactions, counterexamples, and failures:** Stacked attribute order, inline-module transformation, rewritten signature/body, arbitrary host I/O/environment dependence, expansion panic/loop, invalid tokens, and offline/online mode.
- **Constraints:** Unsandboxed host execution; exact macro version and inputs.
- **Classification · citations · provenance · status:** Language-defined interface plus implementation/ecosystem execution; [procedural macros][ref-proc-macro]; `C: RUST-MAC-005`, `F: RS-MAC-07/08/RS-TCH-03`, `G: RUST-MAC-002`; **required** mechanism, environment-dependent example **conditional**.

#### RS-CAN-MAC-007 — Procedural spans and hygiene

- **Feature or behavior:** Proc-macro output participates in ordinary resolution and uses attached spans; call-site and mixed-site spans can change resolution and diagnostics. Exact source mapping APIs differ by channel.
- **Fixture obligation:** Generate identifiers/paths with stable span choices and show one caller collision or resolution distinction without depending on exact diagnostic coordinates.
- **Variants, interactions, counterexamples, and failures:** Absolute paths, downstream rename, call/mixed-site behavior, expansion backtraces, and stable versus nightly location APIs.
- **Constraints:** Primary language evidence is incomplete for some span/hygiene edges.
- **Classification · citations · provenance · status:** Implementation-defined around a language API; [`proc_macro::Span`][std-proc-span], [procedural-macro hygiene][ref-proc-macro]; `C: RUST-PROJ-012/RUST-MAC-005 uncertainty`, `F: RS-MAC-03`, `G: RUST-MAC-003`; **unresolved research gap** for exact edge expectations.

#### RS-CAN-MAC-008 — Compiler built-in and contextual macros

- **Feature or behavior:** Built-ins such as `cfg!`, environment/source-location macros, formatting, inclusion, concatenation/stringification, assertions, collections, matching, and `compile_error!` have compiler or standard-library semantics.
- **Fixture obligation:** Cover contextual source location/module path, compile-time environment present/absent, formatting arguments, cfg boolean, intentional gated error, and data/source inclusion.
- **Variants, interactions, counterexamples, and failures:** Macro-expanded call locations, source remapping, unset `env!` versus `option_env!`, tracked/untracked environment, and `cfg!` retaining ill-typed branches.
- **Constraints:** Build-script environment and compiler invocation.
- **Classification · citations · provenance · status:** Implementation-defined built-ins/standard macros; [standard macros][std-macros]; `C: RUST-MAC-006`, `F: RS-MAC-05`, `G: cfg/include sections`; **required**.

#### RS-CAN-ATTR-001 — Inner, outer, active, inert, helper, and tool attributes

- **Feature or behavior:** Attribute position chooses the annotated construct; active attributes transform/remove syntax, while inert/helper/tool attributes persist for later phases or external tools.
- **Fixture obligation:** Use crate/module inner attributes and outer item/field/variant attributes across cfg, cfg_attr, derive/helper, lint, repr, doc, and a registered tool namespace.
- **Variants, interactions, counterexamples, and failures:** Invalid position, unknown attribute/tool namespace, macro-produced attributes, doc-comment desugaring, ordering of active attributes, and restricted statement/expression attributes.
- **Constraints:** Attribute kind, position, edition, and registration.
- **Classification · citations · provenance · status:** Language/implementation-defined; [attributes][ref-attributes]; `C: RUST-ATTR-001/002`, `F: RS-PKG-05/RS-ATT`, `G: none`; **required**.

#### RS-CAN-ATTR-002 — Conditional compilation removal and subparts

- **Feature or behavior:** `cfg` removes annotated syntax before name/type checking; `cfg_attr` conditionally injects attributes; `cfg!` only returns a boolean and removes nothing.
- **Fixture obligation:** Apply cfg to modules, items, fields, variants, arms/statements, impls, and supported generic/subexpression positions; conditionally inject path, derive, repr/link, and no_std attributes.
- **Variants, interactions, counterexamples, and failures:** Disabled code with unresolved names but valid syntax, syntax error still failing, `cfg!` wrong-platform reference failure, nested cfg_attr, and same-name exclusive definitions.
- **Constraints:** Exact legal subitem/expression positions follow the Reference; Fable's blanket claim that cfg on expressions is disallowed is too broad.
- **Classification · citations · provenance · status:** Language-defined; [conditional compilation][ref-cfg]; `C: RUST-ATTR-003/RUST-CFG-003`, `F: RS-CFG-01..05`, `G: RUST-CFG-001`; **required**.

#### RS-CAN-ATTR-003 — Built-in target cfg and checked custom cfg

- **Feature or behavior:** rustc supplies target, panic, atomic, and feature cfgs; Cargo supplies package features; build scripts/flags can add custom cfgs. `check-cfg` diagnoses unexpected names/values.
- **Fixture obligation:** Compile mutually exclusive shipped targets, expose major target cfg families, use a declared custom cfg, and retain an unexpected-cfg warning.
- **Variants, interactions, counterexamples, and failures:** Host versus target cfg, runtime CPU detection versus compile-time target feature, environment variable not being cfg, custom target values, and version-pinned literal `cfg(true/false)` if included.
- **Constraints:** Target triple, compiler version, Cargo 1.80-era automatic checking.
- **Classification · citations · provenance · status:** Implementation-defined; [conditional compilation][ref-cfg], [check-cfg][rustc-check-cfg]; `C: RUST-CFG-001/002`, `F: RS-CFG-08`, `G: RUST-CFG-001`; **required**, literal cfg forms **unresolved research gap**.

#### RS-CAN-ATTR-004 — Diagnostic and API-shaping attributes

- **Feature or behavior:** Lint, deprecation, must-use, non-exhaustive, track-caller, linkage, and optimizer attributes have distinct semantic/diagnostic strength.
- **Fixture obligation:** Use scoped lint levels, deprecated and must-use call sites, non-exhaustive cross-crate types, track-caller, and semantic versus hint codegen attributes.
- **Variants, interactions, counterexamples, and failures:** `let _` suppression, trait/type must-use, variant deprecation, non-exhaustive struct update, `forbid` override failure, and inline/cold not changing semantics.
- **Constraints:** Lint defaults/version and cross-crate placement.
- **Classification · citations · provenance · status:** Language/implementation-defined; [diagnostic attributes][ref-diagnostic-attrs], [type-system attributes][ref-type-attrs]; `C: RUST-DIAG-001/RUST-ATTR-002`, `F: RS-ATT-01/03/04/05/08`, `G: none`; **required**.

### Unsafe Rust, FFI, ABI, and target-specific low-level behavior

#### RS-CAN-UNSAFE-001 — Unsafe blocks, functions, traits, and impls

- **Feature or behavior:** Unsafe syntax permits a fixed operation set or assigns safety obligations; it does not disable other typing/borrowing rules.
- **Fixture obligation:** Perform raw dereference, unsafe call, mutable-static access, union read, and unsafe trait impl inside documented boundaries with a safe wrapper.
- **Variants, interactions, counterexamples, and failures:** Missing unsafe, unnecessary unsafe, wrong unsafe impl marker, safe versus unsafe trait methods, and edition-2024 explicit unsafe block lint inside unsafe fn.
- **Constraints:** Edition 2024 lint/default changes; soundness is not proven by compilation.
- **Classification · citations · provenance · status:** Language-defined; [unsafe keyword][ref-unsafe]; `C: RUST-UNSAFE-001/002`, `F: RS-UNS-01`, `G: unsafe gap`; **required**.

#### RS-CAN-UNSAFE-002 — Validity, initialization, and `MaybeUninit`

- **Feature or behavior:** Rust types impose validity beyond bit size; invalid/uninitialized values can cause immediate Undefined Behavior (UB). `MaybeUninit` delays the validity assumption.
- **Fixture obligation:** Implement staged value and array initialization with partial-failure cleanup and justified `assume_init`.
- **Variants, interactions, counterexamples, and failures:** Document but never execute zeroed references/invalid enums, uninitialized reads, padding comparison, and invalid bool/char; include drop cleanup and const restrictions.
- **Constraints:** Exact validity frontier evolves; rely only on documented standard/Reference guarantees.
- **Classification · citations · provenance · status:** Language-defined obligations with partially unspecified model; [`MaybeUninit`][std-maybeuninit], [undefined behavior][ref-ub]; `C: RUST-UNSAFE-003`, `F: unsafe section`, `G: unsafe research gap`; **required** safe pattern, frontier **conditional**.

#### RS-CAN-UNSAFE-003 — Pointer provenance and exposed addresses

- **Feature or behavior:** Pointer arithmetic and address exposure/reconstruction operate under an evolving provenance model; in-bounds, one-past, alignment, and reference creation obligations remain material.
- **Fixture obligation:** Use documented strict-provenance/address APIs, in-allocation arithmetic, one-past comparison, and pointer mapping; keep invalid cases nonexecuted.
- **Variants, interactions, counterexamples, and failures:** Zero-sized types, wrapping versus in-bounds offset, integer round trip, fabricated reference, out-of-bounds, and Miri observations not treated as final language law.
- **Constraints:** Compiler/library version; full operational model is not settled.
- **Classification · citations · provenance · status:** Unspecified/evolving implementation model; [pointer APIs][std-pointer], [undefined behavior][ref-ub]; `C: RUST-UNSAFE-004`, `F: unsafe gap`, `G: unresolved unsafe modeling`; **unresolved research gap** beyond documented API cases.

#### RS-CAN-UNSAFE-004 — Transmutation and bit/layout compatibility

- **Feature or behavior:** `transmute` reinterprets bits only when size and validity obligations hold; equal size does not imply a stable or valid representation.
- **Fixture obligation:** Include one justified transparent-wrapper conversion and compile-fail size mismatch; document invalid enum/reference/padding examples.
- **Variants, interactions, counterexamples, and failures:** Endianness, pointer/integer const-eval restriction, `transmute_copy`, invalid niches, and safer conversion alternatives.
- **Constraints:** Layout, target, validity, and const context.
- **Classification · citations · provenance · status:** Language/standard unsafe contract with unspecified layout edges; [`transmute`][std-transmute], [layout][ref-layout]; `C: RUST-UNSAFE-005`, `F: unsafe gap`, `G: unsafe research gap`; **conditional**.

#### RS-CAN-FFI-001 — Foreign blocks, functions, statics, and ABI strings

- **Feature or behavior:** Foreign declarations bind Rust items to externally defined symbols under target-supported ABI strings; definitions can also use explicit ABI.
- **Fixture obligation:** Declare and call a C function/static, define a C-export function, include variadic declaration, and use safe-qualified foreign items where supported.
- **Variants, interactions, counterexamples, and failures:** Edition-2024 `unsafe extern`, `C` versus `system`/`C-unwind`, target-unsupported ABI, link failure, and mismatched declaration causing runtime UB rather than a Rust type error.
- **Constraints:** Edition, target ABI/linker, native symbol availability.
- **Classification · citations · provenance · status:** Language-defined surface plus target implementation; [external blocks][ref-extern-blocks], [ABI][ref-abi]; `C: RUST-FFI-001`, `F: RS-ITM-10/RS-UNS-02`, `G: dynamic-link gap`; **conditional**.

#### RS-CAN-FFI-002 — Export names, sections, link names, and native libraries

- **Feature or behavior:** `no_mangle`, `export_name`, `link_name`, `link_section`, `used`, `link`, and build-script link directives create linker identities/edges distinct from Rust paths.
- **Fixture obligation:** Export and import renamed symbols, place a static in a section, select static/dynamic native libraries, and emit build-script search/link directives.
- **Variants, interactions, counterexamples, and failures:** Duplicate symbols, wrong architecture/missing library, ordering, framework/raw-dylib target cases, dead-code elimination, and edition-2024 unsafe attribute syntax.
- **Constraints:** Target object format/linker and edition.
- **Classification · citations · provenance · status:** Implementation/target-defined; [codegen attributes][ref-codegen-attrs], [native linking][rustc-link]; `C: RUST-FFI-002/003`, `F: RS-UNS-03`, `G: dynamic-link gap`; **conditional**.

#### RS-CAN-FFI-003 — FFI-safe layouts, handles, callbacks, and ownership

- **Feature or behavior:** Only ABI-compatible types/signatures can cross a foreign boundary. Ordinary Rust enums/references/tuples/trait objects lack a general C contract.
- **Fixture obligation:** Pass `repr(C)` struct/union, integer-repr enum, opaque pointer handle, nullable pointer/function pointer, and callback; retain improper-ctypes diagnostics.
- **Variants, interactions, counterexamples, and failures:** bool/char invalid foreign values, platform C integer widths, allocator/ownership mismatch, nullable niche guarantee, and ordinary Rust layout counterexamples.
- **Constraints:** Target C ABI and validity rules.
- **Classification · citations · provenance · status:** Language guarantees plus target ABI; [FFI][ref-extern-blocks], [layout][ref-layout]; `C: RUST-FFI-004`, `F: RS-UNS-04/05`, `G: generated C bridge gap`; **conditional**.

#### RS-CAN-FFI-004 — Unwinding across ABI boundaries

- **Feature or behavior:** Permitted unwind propagation depends jointly on ABI string and panic strategy; destructors and foreign exceptions add runtime/platform constraints.
- **Fixture obligation:** Declare `extern "C"` and `extern "C-unwind"` callbacks, use `catch_unwind` at a boundary, and pair with unwind/abort profiles without executing UB.
- **Variants, interactions, counterexamples, and failures:** Foreign exception through Rust frames, panic crossing non-unwind ABI, target unsupported unwind ABI, and advisory `UnwindSafe`.
- **Constraints:** Compiler/runtime/target and profile.
- **Classification · citations · provenance · status:** Language/runtime/target-defined; [unwinding ABI][ref-abi], [`catch_unwind`][std-catch-unwind]; `C: RUST-FFI-005`, `F: RS-UNS-07`, `G: none`; **conditional**.

#### RS-CAN-FFI-005 — Inline/global assembly and target-feature calls

- **Feature or behavior:** Assembly embeds target instructions/symbols; CPU target features make call validity depend on compile-time flags and runtime capabilities.
- **Fixture obligation:** Target-gate minimal `asm!` with operand/register/options and `sym`, `global_asm!` symbol, architecture intrinsic behind runtime detection, and a `target_feature` function.
- **Variants, interactions, counterexamples, and failures:** Invalid register/option, clobber ABI, global `-C target-feature`, unsafe unsupported call, portable SIMD/nightly, and platform-specific symbol syntax.
- **Constraints:** Architecture/backend/support matrix; incorrect options can cause UB.
- **Classification · citations · provenance · status:** Implementation/target-defined extensions; [inline assembly][ref-asm], [target features][rustc-target-features]; `C: RUST-FFI-006/007`, `F: RS-UNS-06/version inventory`, `G: none`; **conditional**.

### Async and concurrency

#### RS-CAN-ASYNC-001 — Async functions/blocks and opaque futures

- **Feature or behavior:** Each async function/block constructs a distinct anonymous future state machine; its body starts only when polled, and retained locals determine layout and auto traits.
- **Fixture obligation:** Construct without polling, then poll/await async functions and multiple move/nonmove blocks with locals crossing suspension; consume through a generic `Future` bound.
- **Variants, interactions, counterexamples, and failures:** Recursive async requires indirection, captures/lifetime capture differ, size/layout unspecified, and unawaited must-use lint.
- **Constraints:** Edition 2018+; compiler-generated layout is implementation-defined.
- **Classification · citations · provenance · status:** Language-defined behavior plus unspecified representation; [async items][ref-async], [`Future`][std-future]; `C: RUST-ASYNC-001`, `F: RS-CLO-05`, `G: none`; **required**.

#### RS-CAN-ASYNC-002 — Await, polling, cancellation, and suspension borrows

- **Feature or behavior:** `.await` converts through `IntoFuture`, polls with task context, and may suspend; dropping the future cancels remaining work and drops retained state.
- **Fixture obligation:** Implement Pending-to-Ready custom Future/IntoFuture, await it, hold a borrow across suspension, and drop a suspended future with observable cleanup.
- **Variants, interactions, counterexamples, and failures:** Await outside async, temporary not crossing later await, lock guard across await, and postfix/autoderef behavior.
- **Constraints:** Executor is ecosystem-specific; Future/poll contract is standard library.
- **Classification · citations · provenance · status:** Language-defined plus standard task API; [await expressions][ref-await], [`Future`][std-future]; `C: RUST-ASYNC-002`, `F: RS-IMP-03`, `G: none`; **required**.

#### RS-CAN-ASYNC-003 — Pinning and self-referential state

- **Feature or behavior:** `Pin<P>` restricts moving pointees that are not `Unpin`; compiler futures can be self-referential and require pinned polling.
- **Fixture obligation:** Define Unpin and explicit `!Unpin`-shaped marker types, pin on stack and heap, project permitted fields, and implement custom Future poll receiver.
- **Variants, interactions, counterexamples, and failures:** Pointer escape undermining pin, safe versus unsafe projection, `pin!` versus `Box::pin`, and auto-trait inference.
- **Constraints:** Standard-library safety contract; negative Unpin impl syntax itself is unstable for user code, so use marker composition.
- **Classification · citations · provenance · status:** Standard language-coupled contract; [`Pin`][std-pin], [`Unpin`][std-unpin]; `C: RUST-ASYNC-003`, `F: async coverage`, `G: none`; **required**.

#### RS-CAN-ASYNC-004 — Async closures and async call traits

- **Feature or behavior:** Async closures can lend borrows from captures to returned futures and implement `AsyncFnOnce`/`AsyncFnMut`/`AsyncFn` according to use.
- **Fixture obligation:** Pass reading, mutating, and consuming async closures to supported generic bounds and contrast them with ordinary closures returning async blocks.
- **Variants, interactions, counterexamples, and failures:** Lending future escaping, repeated calls, move capture, and exact trait-bound spelling by compiler version.
- **Constraints:** Stable from Rust 1.85; recent surface details require version pinning.
- **Classification · citations · provenance · status:** Language-defined plus standard special traits; [Rust 1.85 announcement][rel-185], [`AsyncFn`][std-async-fn]; `C: RUST-ASYNC-005`, `F: RS-CLO-07`, `G: none`; **conditional**.

#### RS-CAN-ASYNC-005 — Future sendability across suspension

- **Feature or behavior:** A future is `Send` only if state retained across await satisfies `Send`; a non-Send value discarded before suspension need not affect it.
- **Fixture obligation:** Create near-identical futures with a non-Send value before versus across await and check a `Send` bound/spawn-style API.
- **Variants, interactions, counterexamples, and failures:** Explicit scope/drop, async trait return bounds, captured references, executor local task alternative, and body-only change breaking downstream `Send`.
- **Constraints:** Auto-trait analysis and executor API; executor behavior is convention.
- **Classification · citations · provenance · status:** Language-generated type plus standard auto traits; [`Send`][std-send-sync], [`Future`][std-future]; `C: RUST-ASYNC-006`, `F: RS-CLO-05`, `G: none`; **required**.

#### RS-CAN-CONCUR-001 — Threads, scoped borrowing, and join behavior

- **Feature or behavior:** Thread spawning applies `Send`, `Sync`, and lifetime bounds; scoped threads allow non-`'static` borrows by guaranteeing join before scope exit.
- **Fixture obligation:** Move owned data to a thread, reject an ordinary borrowed capture, accept a scoped borrow, require Sync for shared access, and inspect panic/join result.
- **Variants, interactions, counterexamples, and failures:** Dropped join handle detaches, unavailable thread targets, non-Send captures, and mutex/atomic alternatives.
- **Constraints:** Standard-library target support.
- **Classification · citations · provenance · status:** Standard-library contract with language traits/lifetimes; [threads][std-thread]; `C: RUST-CONCUR-001`, `F: tokio/thread capture examples`, `G: none`; **conditional** by target.

#### RS-CAN-CONCUR-002 — Atomics and memory orderings

- **Feature or behavior:** Atomic operations and orderings define synchronization independently of borrowing; available widths are target-dependent.
- **Fixture obligation:** Use load/store/read-modify-write/compare-exchange/fence with valid orderings and cfg-gate an atomic width.
- **Variants, interactions, counterexamples, and failures:** Invalid load/store/failure ordering, compiler versus hardware fence, unsupported width, and non-atomic data race documented as UB.
- **Constraints:** Target atomic support and evolving formal memory-model edges.
- **Classification · citations · provenance · status:** Standard library/target-defined with partially unspecified formal edges; [atomics][std-atomics], [UB data races][ref-ub]; `C: RUST-CONCUR-002`, `F: auto-trait/concurrency examples`, `G: none`; **conditional**.

### Diagnostics, tests, documentation, and compiler limits

#### RS-CAN-DIAG-001 — Lint levels, scopes, expectations, and configuration

- **Feature or behavior:** allow/warn/deny/forbid/expect, command-line force/cap, and Cargo lint tables can turn the same source into success or failure.
- **Fixture obligation:** Apply levels at crate/module/item scopes, fulfilled/unfulfilled expect, nested forbidden override failure, Cargo workspace lint inheritance, and command-line deny/cap.
- **Variants, interactions, counterexamples, and failures:** Unknown lint, changing lint group/default by edition/compiler, dependency capping, CI deny warnings, and Clippy/tool lints.
- **Constraints:** `#[expect]` from 1.81 and Cargo lint-table versions; exact warning text unstable.
- **Classification · citations · provenance · status:** Implementation-defined; [rustc lints][rustc-lints], [Cargo lints][cargo-lints]; `C: RUST-DIAG-001`, `F: RS-ATT-01/02`, `G: naming/Clippy note`; **required**.

#### RS-CAN-DIAG-002 — Failure phases and invalid programs

- **Feature or behavior:** Lexing, parsing, expansion, resolution, privacy, type/trait, borrow, const evaluation, monomorphization, codegen, linking, and runtime expose distinct failure boundaries.
- **Fixture obligation:** Keep isolated, nondefault negative cases for each phase and representative unresolved import, duplicate, privacy, orphan/overlap, ambiguity, unstable feature, missing bound, symbol, and panic failures.
- **Variants, interactions, counterexamples, and failures:** Earlier errors suppress later phases; dead code is type-checked while cfg-removed code is absent; generic failures can wait for instantiation; messages/error codes drift.
- **Constraints:** Version, target, cfg, feature, profile, and whether generic is instantiated.
- **Classification · citations · provenance · status:** Implementation diagnostics over language/toolchain rules; [rustc errors][rustc-errors]; `C: RUST-DIAG-002/003`, `F: RS-ATT-06`, `G: unlinked/macro/build failures`; **required**.

#### RS-CAN-DIAG-003 — Recursion, type computation, and solver limits

- **Feature or behavior:** Crate recursion/type-length limits and compiler trait/monomorphization recursion bounds can reject otherwise meaningful programs at configured resource limits.
- **Fixture obligation:** Exercise recursive macro, deep autoderef/type substitution, recursive monomorphization, and raised versus default limits.
- **Variants, interactions, counterexamples, and failures:** Type-length enforcement version/options, trait-solver overflow, const evaluatability, resource cost of raising limits, and compiler internal errors excluded as normative outcomes.
- **Constraints:** rustc implementation and crate attributes.
- **Classification · citations · provenance · status:** Implementation-defined; [limits][ref-limits]; `C: RUST-DIAG-003/004`, `F: RS-ATT-07/RS-MAC-11`, `G: none`; **conditional**.

#### RS-CAN-TEST-001 — Unit and integration test crate boundaries

- **Feature or behavior:** Unit tests compile with the tested crate and can access private ancestors; each integration test is a separate external crate and sees public API only. `cfg(test)` is set on the crate under test, not dependencies.
- **Fixture obligation:** Access a private item from a unit test, reject it from an integration test, use multiple integration targets and a shared support module, and show lib variants with/without test cfg.
- **Variants, interactions, counterexamples, and failures:** Accidental helper target discovery, same helper compiled per integration crate, dev dependencies, dependency missing cfg(test), and command-selected target sets.
- **Constraints:** Cargo/libtest layout and command.
- **Classification · citations · provenance · status:** Implementation-defined; [Cargo tests][cargo-tests], [test organization][book-tests]; `C: RUST-TEST-001`, `F: RS-TST-01/02`, `G: none`; **required**.

#### RS-CAN-TEST-002 — Harness attributes, generated main, examples, and benches

- **Feature or behavior:** Test attributes register functions with generated libtest main; examples/benches are separate crates. `harness = false` replaces collection with user entry behavior.
- **Fixture obligation:** Include passing, ignored, expected-panic, Result-returning tests; a required-feature example; and harness-less test/bench targets with explicit main.
- **Variants, interactions, counterexamples, and failures:** Signature restrictions, ignored-but-compiled, output/threading harness behavior, panic abort conflict, nightly `#[bench]` rejection, stable ecosystem benchmark framework, and auto/explicit target discovery.
- **Constraints:** Cargo/libtest, profile, channel, required features.
- **Classification · citations · provenance · status:** Implementation-defined plus ecosystem bench convention; [test attributes][ref-test-attrs], [Cargo targets][cargo-targets]; `C: RUST-TEST-002/004`, `F: RS-TST-04/05/06`, `G: none`; **required**, benchmark framework **conditional**.

#### RS-CAN-TEST-003 — Documentation tests and documentation source inclusion

- **Feature or behavior:** rustdoc transforms fenced blocks into synthetic crates and can compile, run, ignore, or expect failure. Documentation included from another file becomes test source.
- **Fixture obligation:** Provide runnable, no-run, compile-fail, ignored, should-panic, hidden-line, and edition-tagged blocks plus crate-level README inclusion.
- **Variants, interactions, counterexamples, and failures:** Synthetic main/crate injection, feature/target environment, private accessibility, `?` hidden tail, exact compile-fail text not asserted, binary/private-item collection uncertainty, and README drift failure.
- **Constraints:** rustdoc version/command/cfg; external included Markdown is a build input, not Rust entity source until transformed.
- **Classification · citations · provenance · status:** Implementation-defined; [rustdoc tests][rustdoc-doctest]; `C: RUST-TEST-003`, `F: RS-TST-03/07`, `G: none`; **required**, disputed collection edges retained as gaps.

### Ecosystem conventions and platform families

#### RS-CAN-ECO-001 — Serialization/schema-style derives

- **Feature or behavior:** Common derive ecosystems interpret container/variant/field helpers to generate impls whose external names, bounds, defaults, and relationships differ from Rust identifiers.
- **Fixture obligation:** With one pinned dependency, demonstrate rename, skip, default, flatten/tag/adapter-like behavior, generic bounds, and generated trait use; compare a manual impl.
- **Variants, interactions, counterexamples, and failures:** Conflicting helpers, cfg-conditioned derive, remote/with adapters, dependency feature gate, and generated details not promoted to Rust semantics.
- **Constraints:** Exact dependency/version and optional features.
- **Classification · citations · provenance · status:** Ecosystem convention; [Serde derive attributes][serde-attrs]; `C: RUST-ECO-002`, `F: RS-ECO-01`, `G: serde example in macro section`; **conditional**.

#### RS-CAN-ECO-002 — Error derives and conversion flow

- **Feature or behavior:** Error derives generate Display/Error/source and `From` impls that become conversion edges selected by `?`; type-erased application errors are a separate convention.
- **Fixture obligation:** Pin one error-derive crate, generate typed conversions/source chains, consume through `?`, and contrast a manual or type-erased application boundary.
- **Variants, interactions, counterexamples, and failures:** Duplicate source/from fields, hidden generated impls, no_std support/features, and missing conversion.
- **Constraints:** Dependency/version; not language behavior.
- **Classification · citations · provenance · status:** Ecosystem convention; [thiserror documentation][thiserror-docs]; `C: RUST-ECO-003`, `F: RS-ECO-02`, `G: none`; **conditional**.

#### RS-CAN-ECO-003 — Runtime/test/observability attribute macros

- **Feature or behavior:** Runtime and instrumentation attributes rewrite async mains/tests/functions into wrappers, runtimes, spans, hidden modules, and changed Send/lifetime contracts.
- **Fixture obligation:** Pin one runtime entry/test attribute and one body-wrapping instrumentation attribute; compare source and compiled semantic shape.
- **Variants, interactions, counterexamples, and failures:** Granular runtime features, spawn `Send + 'static`, local tasks, macro stacking, compile-time max-level elision, and version-specific hidden output.
- **Constraints:** Selected dependencies/features/runtime.
- **Classification · citations · provenance · status:** Ecosystem convention; [Tokio attribute macros][tokio-macros], [tracing instrument][tracing-instrument]; `C: RUST-ECO-003`, `F: RS-ECO-03/04`, `G: tokio example`; **conditional**.

#### RS-CAN-ECO-004 — Build-time native/source generators

- **Feature or behavior:** Build dependencies commonly compile native code or generate Rust bindings/modules in OUT_DIR; generated entities and links depend on host tools and target environment.
- **Fixture obligation:** Use one hermetic vendored native input or deterministic generator and call generated Rust/native items from handwritten code.
- **Variants, interactions, counterexamples, and failures:** Checked-in versus build-time output, missing compiler/header/pkg-config/protoc/libclang, generator drift, vendored/system feature, host/target configuration, and C/C++/protobuf families represented by one exemplar.
- **Constraints:** External toolchain and dependency version; keep default build hermetic.
- **Classification · citations · provenance · status:** Ecosystem/platform convention; [Cargo build scripts][cargo-build]; `C: RUST-ECO-004`, `F: RS-GEN-05`, `G: RUST-BLD-001/002 and FFI gap`; **conditional**.

#### RS-CAN-ECO-005 — API facades, preludes, extension and sealed traits

- **Feature or behavior:** Ecosystem APIs use prelude globs, blanket extension traits, sealed traits, newtype/Deref wrappers, and re-export facades to control resolution, implementability, and identity.
- **Fixture obligation:** Implement each shape locally and consume the same re-exported type through facade/core paths.
- **Variants, interactions, counterexamples, and failures:** Downstream sealed impl failure, hidden-private supertrait versus private-signature sealing, extension method ambiguity, feature forwarding, semver cross-version identity unification, and partial facade splits.
- **Constraints:** Convention; exact API guideline coverage differs.
- **Classification · citations · provenance · status:** Ecosystem convention atop language behavior; [Rust API Guidelines][api-guidelines]; `C: RUST-MOD-006/RUST-TRAIT-003/007`, `F: RS-ECO-07/08`, `G: API Guidelines section`; **conditional**.

#### RS-CAN-ECO-006 — Lazy globals and macro-generated state

- **Feature or behavior:** `LazyLock`/`LazyCell`, `OnceLock`, once_cell, and legacy `lazy_static!` encode runtime global initialization; macros can invent hidden wrapper/Deref entities.
- **Fixture obligation:** Use stable standard lazy state and one pinned legacy macro/library case, then reference generated/hidden behavior through ordinary syntax.
- **Variants, interactions, counterexamples, and failures:** Reentrancy/panic behavior, Sync requirements, const static alternative, hidden struct name, and dependency/version differences.
- **Constraints:** Standard lazy types from Rust 1.80; external cases optional.
- **Classification · citations · provenance · status:** Standard-library behavior plus ecosystem convention; [`LazyLock`][std-lazylock]; `C: interior/static coverage`, `F: RS-ECO-05`, `G: none`; **conditional**.

#### RS-CAN-ECO-007 — Link-time registration

- **Feature or behavior:** Registration crates collect distributed values through linker sections/constructors without direct source references; unlinked producers disappear.
- **Fixture obligation:** If platform coverage is desired, pin one registration crate with producers in separate modules/crates and a consumer iterating the collected set.
- **Variants, interactions, counterexamples, and failures:** Platform section behavior, dead/unlinked crate omission, ctor-before-main ordering, LTO/linker retention, and duplicate registration.
- **Constraints:** Dependency, target, linker, and implementation-defined sections.
- **Classification · citations · provenance · status:** Ecosystem convention on implementation-defined linkage; [inventory documentation][inventory-docs]; `C: linker-facing ecosystem gaps`, `F: RS-ECO-09`, `G: none`; **conditional**.

#### RS-CAN-ECO-008 — Cross-language and Wasm bridge generators

- **Feature or behavior:** Bridge macros/generators create foreign-facing surfaces and glue distinct from Rust `pub` and plain C exports.
- **Fixture obligation:** Represent at most one target-gated bridge (Wasm, C++, Python, or interface generator) or document its exclusion while retaining generated glue/export/link behavior.
- **Variants, interactions, counterexamples, and failures:** Ownership/ABI adaptation, host generator, foreign companion artifacts, installed targets, WASI versus unknown-unknown, and tool version.
- **Constraints:** Heavy target/tool dependencies; not core required.
- **Classification · citations · provenance · status:** Ecosystem/platform convention; [wasm-bindgen guide][wasm-bindgen]; `C: final target/linker gaps`, `F: RS-STD-05/RS-ECO-10`, `G: dynamic FFI gap`; **conditional**.

#### RS-CAN-ECO-009 — Proc-macro-backed external DSLs

- **Feature or behavior:** Function-like macros can parse embedded languages and consult files, schemas, databases, or environment while producing typed expressions/items.
- **Fixture obligation:** Pin a hermetic macro with non-Rust input and deterministic offline data; retain an expansion-time invalid-input failure.
- **Variants, interactions, counterexamples, and failures:** Online/offline modes, schema drift, panic, nondeterminism, unsandboxed I/O, and generated types invisible before expansion.
- **Constraints:** Dependency/version and environment; external services must not be required for the canonical default.
- **Classification · citations · provenance · status:** Ecosystem convention using language proc macros; [procedural macros][ref-proc-macro]; `C: procedural macro environment gap`, `F: RS-MAC-08/RS-TCH-03`, `G: RUST-MAC-002`; **conditional**.

#### RS-CAN-ECO-010 — API naming and common-trait conventions

- **Feature or behavior:** Public Rust APIs commonly use no-`get_` getters, `as_`/`to_`/`into_` conversion names, and standard traits such as Debug/Clone/Default/Eq/Hash.
- **Fixture obligation:** If idiomatic API convention is in scope, show the naming families and standard trait surface while distinguishing them from compiler-enforced semantics.
- **Variants, interactions, counterexamples, and failures:** Clippy diagnostics, owned/borrowed receiver differences, `From` preference, and valid nonconforming APIs.
- **Constraints:** Convention only; no language validity depends on names.
- **Classification · citations · provenance · status:** Ecosystem convention; [Rust API Guidelines][api-guidelines]; `C: RUST-TRAIT-012/014`, `F: standard-trait and API-shape items`, `G: RUST-ECO-001/002`; **conditional**.

## Unresolved disagreements and unsupported claims

These claims are not part of the required checklist until verified against a pinned primary source or compiler release:

1. **Baseline conflict.** `C` assumes stable 1.85, `F` assumes a late-2025 stable and cites features through 1.89+, and `G` suggests 1.88 nightly or an unspecified stable equivalent. The checklist uses 1.85 only as a comparison floor and version-gates every later feature. It does not accept nightly as the canonical baseline.
2. **Enum discriminants.** `C` says duplicate discriminants “can be allowed”; `F` says they are errors. The canonical negative follows the Reference rule that two variants may not share a discriminant. Any exception claim remains unsupported.
3. **Namespace count.** `G` describes a tripartite system. `C` includes lifetime and label namespaces. The canonical item preserves all materially distinct namespaces and treats derive/attribute/bang distinctions as macro-resolution variants.
4. **Public re-export wording.** `G` says a “private item” can be exposed. Canonical behavior distinguishes a public item nested in a private module from an item whose own visibility is private; the latter cannot simply be publicly re-exported.
5. **Let chains.** `C` could not pin stabilization; `F` reports Rust 1.88 and edition 2024. The item is conditional on 1.88+ and retains 1.85 rejection. Confirm exact grammar against the selected toolchain.
6. **`cfg(true)` / `cfg(false)`.** `F` gives an approximate 1.88 stabilization. No report supplies firm primary evidence; preserve as a research gap.
7. **Body-local trait impl reach and coherence.** `F` proposes a body-local impl visible crate-wide with medium confidence. The general impl/coherence rule is required, but this placement probe needs direct Reference/compiler confirmation.
8. **Included-file nested module path base and type-alias edge rules.** `F` marks include path base and some alias behavior as subtle; `C` phrases alias inherent impl ability differently. Preserve probes without asserting unverified outcomes.
9. **Unicode normalization and doc-test collection edges.** `F` marks normalization details and doctests from non-library/private items as uncertain. Keep version-pinned research cases separate from the required core.
10. **Procedural-macro span hygiene and tool behavior.** Language-level token/span effects are plausible, but `G` relies heavily on secondary sources and downstream language-server behavior. Exact collision, fallback, sandboxing, and mapping expectations are research gaps.
11. **Method-candidate failure backtracking.** `G` alleges ambiguous fallback behavior based partly on secondary sources. Required lookup follows the Reference's candidate algorithm; undocumented fallback claims are excluded.
12. **Dev-dependency cycles.** Proposed only by `F` with medium confidence. It remains a standalone research item rather than a required Cargo property.

## Completeness audit

### Input-report items omitted or merged

No materially unique, plausible report proposal was silently dropped. Duplicates were merged only when they described the same observable behavior:

- `C`'s project/module/type/trait/ownership/expression/macro/unsafe/async/test families map directly to canonical items. Its separate recent-version inventory is represented as constraints on edition, async closure, precise capture, let-chain, C-string, resolver, lockfile, lint, and cfg items.
- `F`'s 165 candidates map to the canonical items or the research list. Closely related pairs were merged: crate/secondary-target identity; trait defaults/supertraits; `Drop`/`Copy`; inference/coercion; macro code generation; cfg forms; test target/harness forms; source generation/build scripts; and API-shape conventions.
- `G`'s canonical language/Cargo claims map to visibility, namespaces, method resolution/coercion, layout, coherence, macros, build generation, cfg/features, workspaces, editions, and ecosystem convention items.

The following report material was deliberately not promoted to a checklist obligation:

- `G`'s code-graph schema, analyzer, language-server, IDE, sandbox/fallback, and downstream-design discussion is outside the synthesis scope. Relevant language facts about unlinked files, proc-macro execution, and spans remain.
- Exact error text, numeric error codes, rust-analyzer behavior, Miri behavior, compiler internal errors, optimization choices, symbol hashes, and rustdoc presentation are not normative identities. They appear only where an implementation-specific conditional is materially observable.
- `F`'s HashMap iteration example is ordinary standard-library runtime behavior and not a distinct language/build feature; it was not proposed as an enumerated checklist item.
- Registry publishing/yanking/authentication, formal verification tools, sanitizer/Profile-Guided Optimization (PGO), incremental caches, and performance-only build details were audit notes rather than candidate language obligations. LTO/codegen/profile effects remain only where they change artifacts or observable configuration.

### Categories represented by only one source

- **Fable only:** raw/Unicode identifiers as full items; intra-doc links; dev-dependency cycles; detailed API facade/sealed/link-registration/bridge/lazy-global conventions; README doctest inclusion; API naming; command/config invocation; fine-grained crate metadata identity.
- **Codex only or materially deeper than the other reports:** partial moves; borrow exclusivity, NLL, two-phase borrows; destructuring assignment; many pattern/control-flow details; static promotion; validity/provenance/transmute; atomics; pinning/cancellation; explicit compiler-phase and computation-limit cases.
- **Google only:** no unique well-supported language feature remained after reconciliation. Its unique language-server and static-code-graph assertions are scope-specific or weakly sourced and remain unsupported claims, not required Rust features.

### Weak or missing primary evidence

- Exact post-1.85 stabilization versions and restrictions: let chains, naked functions, literal cfg predicates, inferred const arguments, expanded const operations, and later trait-solver changes.
- Exact privacy lint severity across historical compiler versions, body-local impl placement effects, type-alias constructor/inherent-impl/lazy-checking edges, and included-file nested module path bases.
- Full pointer provenance/aliasing/validity model, FFI unwinding details on each target, formal atomic model edges, proc-macro span/source mapping, and custom-target JSON behavior.
- rustdoc collection/inlining/cfg rules beyond the core doctest and intra-doc-link behavior, and current status of `doc_cfg`/rustdoc JSON.

### Likely blind spots for another research pass

- Tokenization/parser identity cases: reserved prefixes, raw strings around edition boundaries, Unicode normalization, macro token spacing, nested generic closers, and raw identifiers across macro editions.
- Trait-solver frontiers: advanced associated-type bounds, projection normalization cycles, implied outlives bounds, uninhabited/never-type exhaustiveness, and next-solver acceptance differences.
- Runtime/link platforms: custom/global allocators, allocation-error handlers, panic-runtime linkage, dynamic loading, symbol versioning, linker scripts, Windows raw-dylib, Apple frameworks, WebAssembly imports/exports, and cross-language LTO.
- Cargo frontiers: artifact/public dependencies, custom targets, alternate registries/sparse indexes, build-std, single-file packages, compiler-provided crates, lockfile-version matrices, and precise resolver-3 Minimum Supported Rust Version (MSRV) behavior.
- Procedural macro reproducibility: tracked file/environment APIs, nondeterminism, network access, stable span/source APIs, and dependency-specific generated surfaces.
- Nightly frontier: coroutines/generators/yield, arbitrary self types, unsized locals, specialization, trait aliases, negative bounds/impls, generic const expressions, const traits, portable Single Instruction, Multiple Data (SIMD), try blocks, custom test frameworks, and `no_core`.

## Primary-source reference index

[api-guidelines]: https://rust-lang.github.io/api-guidelines/
[api-naming]: https://rust-lang.github.io/api-guidelines/naming.html
[book-tests]: https://doc.rust-lang.org/book/ch11-03-test-organization.html
[cargo-build]: https://doc.rust-lang.org/cargo/reference/build-scripts.html
[cargo-config]: https://doc.rust-lang.org/cargo/reference/config.html
[cargo-deps]: https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html
[cargo-features]: https://doc.rust-lang.org/cargo/reference/features.html
[cargo-links]: https://doc.rust-lang.org/cargo/reference/build-scripts.html#the-links-manifest-key
[cargo-lints]: https://doc.rust-lang.org/cargo/reference/manifest.html#the-lints-section
[cargo-lock]: https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html
[cargo-overrides]: https://doc.rust-lang.org/cargo/reference/overriding-dependencies.html
[cargo-platform]: https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#platform-specific-dependencies
[cargo-profiles]: https://doc.rust-lang.org/cargo/reference/profiles.html
[cargo-resolver]: https://doc.rust-lang.org/cargo/reference/resolver.html
[cargo-rust-version]: https://doc.rust-lang.org/cargo/reference/rust-version.html
[cargo-targets]: https://doc.rust-lang.org/cargo/reference/cargo-targets.html
[cargo-tests]: https://doc.rust-lang.org/cargo/reference/cargo-targets.html#tests
[cargo-workspaces]: https://doc.rust-lang.org/cargo/reference/workspaces.html
[edition-2018-nll]: https://doc.rust-lang.org/edition-guide/rust-2018/ownership-and-lifetimes/non-lexical-lifetimes.html
[edition-2021-capture]: https://doc.rust-lang.org/edition-guide/rust-2021/disjoint-capture-in-closures.html
[edition-2021-intoiter]: https://doc.rust-lang.org/edition-guide/rust-2021/IntoIterator-for-arrays.html
[edition-2024-macro]: https://doc.rust-lang.org/edition-guide/rust-2024/macro-fragment-specifiers.html
[edition-2024-match]: https://doc.rust-lang.org/edition-guide/rust-2024/match-ergonomics.html
[edition-2024-rpit]: https://doc.rust-lang.org/edition-guide/rust-2024/rpit-lifetime-capture.html
[edition-2024-temp]: https://doc.rust-lang.org/edition-guide/rust-2024/temporary-scope.html
[edition-guide]: https://doc.rust-lang.org/edition-guide/
[inventory-docs]: https://docs.rs/inventory/
[ref-abi]: https://doc.rust-lang.org/reference/items/functions.html#extern-function-qualifier
[ref-asm]: https://doc.rust-lang.org/reference/inline-assembly.html
[ref-associated]: https://doc.rust-lang.org/reference/items/associated-items.html
[ref-async]: https://doc.rust-lang.org/reference/items/functions.html#async-functions
[ref-attributes]: https://doc.rust-lang.org/reference/attributes.html
[ref-await]: https://doc.rust-lang.org/reference/expressions/await-expr.html
[ref-blocks]: https://doc.rust-lang.org/reference/expressions/block-expr.html
[ref-borrow]: https://doc.rust-lang.org/reference/expressions/operator-expr.html#borrow-operators
[ref-bounds]: https://doc.rust-lang.org/reference/trait-bounds.html
[ref-casts]: https://doc.rust-lang.org/reference/expressions/operator-expr.html#type-cast-expressions
[ref-cfg]: https://doc.rust-lang.org/reference/conditional-compilation.html
[ref-codegen-attrs]: https://doc.rust-lang.org/reference/attributes/codegen.html
[ref-closures]: https://doc.rust-lang.org/reference/types/closure.html
[ref-coercions]: https://doc.rust-lang.org/reference/type-coercions.html
[ref-coherence]: https://doc.rust-lang.org/reference/items/implementations.html#trait-implementation-coherence
[ref-const-eval]: https://doc.rust-lang.org/reference/const_eval.html
[ref-const-generics]: https://doc.rust-lang.org/reference/items/generics.html#const-generics
[ref-const-items]: https://doc.rust-lang.org/reference/items/constant-items.html
[ref-crates]: https://doc.rust-lang.org/reference/crates-and-source-files.html
[ref-destructors]: https://doc.rust-lang.org/reference/destructors.html
[ref-destructuring]: https://doc.rust-lang.org/reference/expressions/underscore-expr.html
[ref-diagnostic-attrs]: https://doc.rust-lang.org/reference/attributes/diagnostics.html
[ref-derive]: https://doc.rust-lang.org/reference/attributes/derive.html
[ref-dst]: https://doc.rust-lang.org/reference/dynamically-sized-types.html
[ref-dyn-compatible]: https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility
[ref-editions]: https://doc.rust-lang.org/reference/editions.html
[ref-enums]: https://doc.rust-lang.org/reference/items/enumerations.html
[ref-expressions]: https://doc.rust-lang.org/reference/expressions.html
[ref-extern-blocks]: https://doc.rust-lang.org/reference/items/external-blocks.html
[ref-extern-crate]: https://doc.rust-lang.org/reference/items/extern-crates.html
[ref-fn-types]: https://doc.rust-lang.org/reference/types/function-item.html
[ref-generics]: https://doc.rust-lang.org/reference/items/generics.html
[ref-identifiers]: https://doc.rust-lang.org/reference/identifiers.html
[ref-if]: https://doc.rust-lang.org/reference/expressions/if-expr.html
[ref-impl-trait]: https://doc.rust-lang.org/reference/types/impl-trait.html
[ref-impls]: https://doc.rust-lang.org/reference/items/implementations.html
[ref-index]: https://doc.rust-lang.org/reference/expressions/array-expr.html#array-and-slice-indexing-expressions
[ref-inference]: https://doc.rust-lang.org/reference/type-inference.html
[ref-items]: https://doc.rust-lang.org/reference/items.html
[ref-lifetime-elision]: https://doc.rust-lang.org/reference/lifetime-elision.html
[ref-layout]: https://doc.rust-lang.org/reference/type-layout.html
[ref-limits]: https://doc.rust-lang.org/reference/attributes/limits.html
[ref-linkage]: https://doc.rust-lang.org/reference/linkage.html
[ref-literals]: https://doc.rust-lang.org/reference/expressions/literal-expr.html
[ref-loops]: https://doc.rust-lang.org/reference/expressions/loop-expr.html
[ref-macro-hygiene]: https://doc.rust-lang.org/reference/macros-by-example.html#hygiene
[ref-macro-rules]: https://doc.rust-lang.org/reference/macros-by-example.html
[ref-macro-scope]: https://doc.rust-lang.org/reference/macros-by-example.html#scoping-exporting-and-importing
[ref-main]: https://doc.rust-lang.org/reference/crates-and-source-files.html#main-functions
[ref-match]: https://doc.rust-lang.org/reference/expressions/match-expr.html
[ref-method-call]: https://doc.rust-lang.org/reference/expressions/method-call-expr.html
[ref-modules]: https://doc.rust-lang.org/reference/items/modules.html
[ref-namespaces]: https://doc.rust-lang.org/reference/names/namespaces.html
[ref-never]: https://doc.rust-lang.org/reference/types/never.html
[ref-operators]: https://doc.rust-lang.org/reference/expressions/operator-expr.html
[ref-panic-handler]: https://doc.rust-lang.org/reference/panic.html
[ref-paths]: https://doc.rust-lang.org/reference/paths.html
[ref-patterns]: https://doc.rust-lang.org/reference/patterns.html
[ref-place]: https://doc.rust-lang.org/reference/expressions.html#place-expressions-and-value-expressions
[ref-pointers]: https://doc.rust-lang.org/reference/types/pointer.html
[ref-preludes]: https://doc.rust-lang.org/reference/names/preludes.html
[ref-proc-macro]: https://doc.rust-lang.org/reference/procedural-macros.html
[ref-question]: https://doc.rust-lang.org/reference/expressions/operator-expr.html#the-question-mark-operator
[ref-ranges]: https://doc.rust-lang.org/reference/expressions/range-expr.html
[ref-return]: https://doc.rust-lang.org/reference/expressions/return-expr.html
[ref-scopes]: https://doc.rust-lang.org/reference/names/scopes.html
[ref-special-traits]: https://doc.rust-lang.org/reference/special-types-and-traits.html
[ref-statements]: https://doc.rust-lang.org/reference/statements.html
[ref-static-items]: https://doc.rust-lang.org/reference/items/static-items.html
[ref-structs]: https://doc.rust-lang.org/reference/items/structs.html
[ref-subtyping]: https://doc.rust-lang.org/reference/subtyping.html
[ref-test-attrs]: https://doc.rust-lang.org/reference/attributes/testing.html
[ref-trait-objects]: https://doc.rust-lang.org/reference/types/trait-object.html
[ref-traits]: https://doc.rust-lang.org/reference/items/traits.html
[ref-type-aliases]: https://doc.rust-lang.org/reference/items/type-aliases.html
[ref-type-attrs]: https://doc.rust-lang.org/reference/attributes/type_system.html
[ref-types]: https://doc.rust-lang.org/reference/types.html
[ref-ub]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
[ref-unions]: https://doc.rust-lang.org/reference/items/unions.html
[ref-unsafe]: https://doc.rust-lang.org/reference/unsafe-keyword.html
[ref-use]: https://doc.rust-lang.org/reference/items/use-declarations.html
[ref-visibility]: https://doc.rust-lang.org/reference/visibility-and-privacy.html
[rel-165]: https://blog.rust-lang.org/2022/11/03/Rust-1.65.0/
[rel-175]: https://blog.rust-lang.org/2023/12/28/Rust-1.75.0/
[rel-177]: https://blog.rust-lang.org/2024/03/21/Rust-1.77.0/
[rel-179]: https://blog.rust-lang.org/2024/06/13/Rust-1.79.0/
[rel-182]: https://blog.rust-lang.org/2024/10/17/Rust-1.82.0/
[rel-185]: https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/
[rel-188]: https://blog.rust-lang.org/2025/06/26/Rust-1.88.0/
[rustc-borrow]: https://rustc-dev-guide.rust-lang.org/borrow_check.html
[rustc-check-cfg]: https://doc.rust-lang.org/rustc/check-cfg.html
[rustc-cli]: https://doc.rust-lang.org/rustc/command-line-arguments.html
[rustc-codegen]: https://doc.rust-lang.org/rustc/codegen-options/index.html
[rustc-crate-type]: https://doc.rust-lang.org/rustc/command-line-arguments.html#--crate-type-a-list-of-types-of-crates-for-the-compiler-to-emit
[rustc-errors]: https://doc.rust-lang.org/error_codes/error-index.html
[rustc-link]: https://doc.rust-lang.org/reference/items/external-blocks.html#the-link-attribute
[rustc-lints]: https://doc.rust-lang.org/rustc/lints/index.html
[rustc-mono]: https://rustc-dev-guide.rust-lang.org/backend/monomorph.html
[rustc-target-features]: https://doc.rust-lang.org/reference/attributes/codegen.html#the-target_feature-attribute
[rustdoc-doctest]: https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html
[rustdoc-links]: https://doc.rust-lang.org/rustdoc/write-documentation/linking-to-items-by-name.html
[rustup-overrides]: https://rust-lang.github.io/rustup/overrides.html
[serde-attrs]: https://serde.rs/attributes.html
[std-async-fn]: https://doc.rust-lang.org/std/ops/trait.AsyncFn.html
[std-atomics]: https://doc.rust-lang.org/std/sync/atomic/
[std-catch-unwind]: https://doc.rust-lang.org/std/panic/fn.catch_unwind.html
[std-convert]: https://doc.rust-lang.org/std/convert/
[std-copy]: https://doc.rust-lang.org/std/marker/trait.Copy.html
[std-deref]: https://doc.rust-lang.org/std/ops/trait.Deref.html
[std-derive]: https://doc.rust-lang.org/reference/attributes/derive.html
[std-drop]: https://doc.rust-lang.org/std/ops/trait.Drop.html
[std-fmt]: https://doc.rust-lang.org/std/fmt/
[std-fn]: https://doc.rust-lang.org/std/ops/trait.FnOnce.html
[std-future]: https://doc.rust-lang.org/std/future/trait.Future.html
[std-global-alloc]: https://doc.rust-lang.org/std/alloc/trait.GlobalAlloc.html
[std-include]: https://doc.rust-lang.org/std/macro.include.html
[std-lazylock]: https://doc.rust-lang.org/std/sync/struct.LazyLock.html
[std-macros]: https://doc.rust-lang.org/std/#macros
[std-maybeuninit]: https://doc.rust-lang.org/std/mem/union.MaybeUninit.html
[std-ops]: https://doc.rust-lang.org/std/ops/
[std-phantom]: https://doc.rust-lang.org/std/marker/struct.PhantomData.html
[std-pin]: https://doc.rust-lang.org/std/pin/struct.Pin.html
[std-pointer]: https://doc.rust-lang.org/std/primitive.pointer.html
[std-prelude]: https://doc.rust-lang.org/std/prelude/
[std-proc-span]: https://doc.rust-lang.org/proc_macro/struct.Span.html
[std-refcell]: https://doc.rust-lang.org/std/cell/struct.RefCell.html
[std-send-sync]: https://doc.rust-lang.org/std/marker/trait.Send.html
[std-termination]: https://doc.rust-lang.org/std/process/trait.Termination.html
[std-thread]: https://doc.rust-lang.org/std/thread/
[std-thread-local]: https://doc.rust-lang.org/std/macro.thread_local.html
[std-transmute]: https://doc.rust-lang.org/std/mem/fn.transmute.html
[std-try]: https://doc.rust-lang.org/std/ops/trait.Try.html
[std-unpin]: https://doc.rust-lang.org/std/marker/trait.Unpin.html
[std-unsafecell]: https://doc.rust-lang.org/std/cell/struct.UnsafeCell.html
[thiserror-docs]: https://docs.rs/thiserror/
[tokio-macros]: https://docs.rs/tokio/latest/tokio/attr.main.html
[tracing-instrument]: https://docs.rs/tracing/latest/tracing/attr.instrument.html
[unstable-book]: https://doc.rust-lang.org/unstable-book/
[wasm-bindgen]: https://rustwasm.github.io/docs/wasm-bindgen/
