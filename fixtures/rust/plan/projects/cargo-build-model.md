# Project brief: cargo-build-model

## Purpose

Demonstrate how Cargo assembles crates: package, target, and crate identity;
target discovery; naming and aliasing; workspace composition and inheritance;
the dependency graph and its host/target universes; features and their
unification across resolver versions; dependency sources, patches, and multiple
versions; the lockfile and Rust-version constraints; crate types and profiles;
mixed-edition graphs; command- and configuration-dependent build identity.

## Exclusive directory

`fixtures/rust/projects/cargo-build-model/`

## Difficulty

complex

## Assigned coverage

- **RS-CAN-PROJ-001** — package versus target versus crate identity, including one
  source compiled under two crate identities.
- **RS-CAN-PROJ-002** — conventional and explicit library, binary, example, test,
  and bench targets; a disabled auto-discovery family; a `cfg`-disabled subtree.
- **RS-CAN-PROJ-003** — distinct package, library, executable, and dependency-key
  names; hyphen-to-underscore mapping; two versions under aliases.
- **RS-CAN-PROJ-004** — rooted and virtual workspace forms; members, exclusions,
  default members, path dependencies; opted-in inheritance of package fields,
  dependencies, and lints; root-only profile and patch behavior.
- **RS-CAN-PROJ-005** — normal, development, build, and procedural-macro dependency
  roles and host/target universes, with build-script host/target reporting.
- **RS-CAN-PROJ-006** — default, non-default, optional, `dep:`, strong- and
  weak-forwarded, all-features, and no-default-features builds, plus feature-gated
  API and impls.
- **RS-CAN-PROJ-007** — feature unification under resolver versions 1, 2, and 3.
- **RS-CAN-PROJ-008** — target-specific dependency tables selected by `cfg`
  predicates and exact triples with a common caller.
- **RS-CAN-PROJ-009** — registry-equivalent, git-revision, and path sources;
  graph-wide patch; two coexisting versions surfaced through aliases.
- **RS-CAN-PROJ-010** — a lockfile with duplicate versions under a wider manifest
  range, a pinned toolchain constraint, and an honest minimum Rust version.
- **RS-CAN-PROJ-014** — the `rlib`, `dylib`, `cdylib`, `staticlib`, executable,
  and `proc-macro` crate types and their link and export boundaries.
- **RS-CAN-PROJ-015** — development, release, custom, and package-override profiles
  and their observable codegen and assertion differences.
- **RS-CAN-PROJ-016** — interacting crates on editions 2015, 2018, 2021, and 2024
  with edition-sensitive constructs and a cross-edition macro.
- **RS-CAN-PROJ-019** — one source compiled under two invocation and configuration
  variants, with checked-in Cargo configuration.
- **RS-CAN-PROJ-021** — `cfg(test)`, `cfg(doc)`, and `cfg(doctest)` entities mapped
  to the commands that compile them.
- **RS-CAN-PROJ-022** — fine-grained build identity: one source producing multiple
  crate instances by role, feature, or version.

Named counterexamples (duplicate target names, a missing inherited key, a
library reaching a dev dependency, an unused-patch warning, a registry-spelled
identifier) are preserved as documentation beside their valid demonstrations.

## Declared build contexts

- Default: `x86_64-unknown-linux-gnu`, edition 2024, stable 1.97.1.
- Editions 2015, 2018, and 2021 as additional per-crate inputs.
- Resolver versions 1, 2, and 3.
- Development, release, custom, and package-override profiles.
- Feature sets including all-features and no-default-features.
- `cfg`-selected non-host operating-system and architecture edges; only the host
  edge is compiled.
- Local path, vendored-registry, and local git dependency sources resolved
  offline through checked-in Cargo configuration and source replacement.

## Dependency needs

No third-party dependency. Dependency-role, dependency-source, multiple-version,
and procedural-macro-dependency coverage is created with locally authored crates
presented as path, vendored-registry, and local git sources, and with a local
procedural-macro member. All sources resolve offline with no download.

## Generated-source needs

None. Any build script here reports host and target facts for the dependency-role
demonstration and writes no tracked source. Build-time source generation is owned
by `build-scripts-and-codegen`.

## Planned tests

One planned test. A test-configuration module and unit test supply the
`cfg(test)` portion of RS-CAN-PROJ-021 coverage, which only exists under the test
compilation. No other test source is planned.

## Required interfaces

Provide `Taskfile.yml` with `build`, `lint`, and `test` as defined in the ground
rules, exercising every declared context, and `coverage.md` mapping each assigned
identifier to its stable source location.
