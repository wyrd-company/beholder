---
relationships:
  references:
    - ground-rules
    - reports/dart/synthesis/checklist
---

# Dart fixture corpus plan overview

## Selected toolchain

- Dart SDK 3.12.2 (stable), sound null safety, on `linux_x64`.
- All corpus source is valid under this pinned SDK. Per-library language
  versions are recorded by each project's manifest and per-library overrides.

The installed toolchain was inspected for this plan. `dart compile` offers `js`,
`jit-snapshot`, `kernel`, `exe`, `aot-snapshot`, and `wasm` subcommands. A C
toolchain (`gcc`, `cc`, `clang`) is present. No web browser and no Flutter SDK
are installed. The SDK experiment set (`allowed_experiments.json`) has an empty
default set; its only defined experiments are `macros` and `enhanced-parts`.

## Build contexts

The corpus declares these locally available build contexts. A project declares
only the subset it needs.

| Context | Meaning | Toolchain command |
|---|---|---|
| `vm-jit` | Default context: Dart VM in JIT mode on `linux_x64`; assertions-enabled and assertions-disabled modes; `dart:mirrors` reflection is available only here | `dart run`, `dart analyze` |
| `native-aot` | Native ahead-of-time executable and snapshot | `dart compile exe`, `dart compile aot-snapshot` |
| `js` | Web target; headless compilation only (no browser runtime) | `dart compile js` |
| `wasm` | WebAssembly/WasmGC target; headless compilation only (no browser runtime) | `dart compile wasm` |

`vm-jit` is the default context for every project. Reflection is a capability of
`vm-jit` rather than a separate target.

## Projects

| Slug | Directory | Difficulty | Purpose |
|---|---|---|---|
| `configuration-and-versions` | `fixtures/dart/projects/configuration-and-versions/` | routine | A small package exercising SDK and per-library language-version selection, compile-time environment values, and assertion configuration |
| `libraries-and-visibility` | `fixtures/dart/projects/libraries-and-visibility/` | complex | A multi-library package exercising directives, prefixes, combinators, exports and façades, parts, underscore privacy, conditional and deferred imports, scope, and wildcards |
| `type-model` | `fixtures/dart/projects/type-model/` | complex | An idiomatic library modeling a generic domain through Dart's nominal and structural type system, generics, variance, and override checking |
| `construction-and-constants` | `fixtures/dart/projects/construction-and-constants/` | complex | A value-oriented library exercising constructors, factories, cascades, implicit operator lowering, the constant subset, canonicalization, and equality |
| `functions-and-dispatch` | `fixtures/dart/projects/functions-and-dispatch/` | complex | A function-oriented library exercising function kinds, function types and tear-offs, dynamic invocation, entry points, and symbols |
| `null-safety-flow-patterns` | `fixtures/dart/projects/null-safety-flow-patterns/` | complex | A control-flow library exercising null safety, flow promotion, statements and switches, the pattern taxonomy and contexts, exceptions, and collections |
| `async-and-isolates` | `fixtures/dart/projects/async-and-isolates/` | complex | A concurrency library exercising futures, generators, streams, event queues and zones, and native isolate messaging |
| `primitive-values` | `fixtures/dart/projects/primitive-values/` | complex | An encoding library exercising numbers with per-target semantics, strings and Unicode, and typed data across compilation targets |
| `packages-and-workspace` | `fixtures/dart/projects/packages-and-workspace/` | complex | A pub workspace of member packages exercising manifests, layout and roots, dependency sources, version solving, overrides, workspaces, tests, and executables |
| `analysis-and-docs` | `fixtures/dart/projects/analysis-and-docs/` | complex | A documented and annotated library exercising analyzer configuration, suppression and generated markers, documentation references, metadata, contract annotations, automated fixes, and synthetic entities |
| `code-generation` | `fixtures/dart/projects/code-generation/` | complex | A code-generation package exercising part-based and standalone-library generation, builder configuration, generated-source formatting, config-driven generation, and the stable no-macros constraint |
| `platforms-and-interop` | `fixtures/dart/projects/platforms-and-interop/` | complex | A cross-platform library exercising the compilation-target matrix, platform-library availability, JIT-versus-AOT reachability, web and WebAssembly compilation, modern JavaScript interop, native FFI, process and filesystem, mirrors, pragmas, external declarations, and non-portable observations |

## Primary assignment

Every in-scope canonical identifier has exactly one primary project below.
Natural overlap into other projects is permitted and is not accounted for here.

### 1. Version, configuration, and source selection

| Canonical ID | Primary project |
|---|---|
| DART-CAN-CFG-001 | `configuration-and-versions` |
| DART-CAN-CFG-002 | `configuration-and-versions` |
| DART-CAN-CFG-003 | `configuration-and-versions` |
| DART-CAN-CFG-004 | `configuration-and-versions` |

### 2. Libraries, directives, privacy, and resolution

| Canonical ID | Primary project |
|---|---|
| DART-CAN-LIB-001 | `libraries-and-visibility` |
| DART-CAN-LIB-002 | `libraries-and-visibility` |
| DART-CAN-LIB-003 | `libraries-and-visibility` |
| DART-CAN-LIB-004 | `libraries-and-visibility` |
| DART-CAN-LIB-005 | `libraries-and-visibility` |
| DART-CAN-LIB-006 | `libraries-and-visibility` |
| DART-CAN-LIB-007 | `libraries-and-visibility` |
| DART-CAN-LIB-008 | `libraries-and-visibility` |
| DART-CAN-LIB-009 | `libraries-and-visibility` |
| DART-CAN-LIB-010 | `libraries-and-visibility` |
| DART-CAN-LIB-011 | `libraries-and-visibility` |

### 3. Type declarations and nominal/structural typing

| Canonical ID | Primary project |
|---|---|
| DART-CAN-TYPE-001 | `type-model` |
| DART-CAN-TYPE-002 | `type-model` |
| DART-CAN-TYPE-003 | `type-model` |
| DART-CAN-TYPE-004 | `type-model` |
| DART-CAN-TYPE-005 | `type-model` |
| DART-CAN-TYPE-006 | `type-model` |
| DART-CAN-TYPE-007 | `type-model` |
| DART-CAN-TYPE-008 | `type-model` |
| DART-CAN-TYPE-009 | `type-model` |
| DART-CAN-TYPE-010 | `type-model` |
| DART-CAN-TYPE-011 | `type-model` |
| DART-CAN-TYPE-012 | `type-model` |

### 4. Generics, inference, functions, and dispatch

| Canonical ID | Primary project |
|---|---|
| DART-CAN-GEN-001 | `type-model` |
| DART-CAN-GEN-002 | `type-model` |
| DART-CAN-GEN-003 | `type-model` |
| DART-CAN-GEN-004 | `type-model` |
| DART-CAN-FN-001 | `functions-and-dispatch` |
| DART-CAN-FN-002 | `functions-and-dispatch` |
| DART-CAN-FN-003 | `functions-and-dispatch` |
| DART-CAN-FN-004 | `functions-and-dispatch` |
| DART-CAN-FN-005 | `platforms-and-interop` |

### 5. Construction, expressions, constants, and identity

| Canonical ID | Primary project |
|---|---|
| DART-CAN-OBJ-001 | `construction-and-constants` |
| DART-CAN-OBJ-002 | `construction-and-constants` |
| DART-CAN-OBJ-003 | `construction-and-constants` |
| DART-CAN-OBJ-004 | `construction-and-constants` |
| DART-CAN-OBJ-005 | `construction-and-constants` |
| DART-CAN-OBJ-006 | `construction-and-constants` |
| DART-CAN-CONST-001 | `construction-and-constants` |
| DART-CAN-CONST-002 | `construction-and-constants` |
| DART-CAN-CONST-003 | `construction-and-constants` |

### 6. Null safety, flow, patterns, and collections

| Canonical ID | Primary project |
|---|---|
| DART-CAN-NULL-001 | `null-safety-flow-patterns` |
| DART-CAN-NULL-002 | `null-safety-flow-patterns` |
| DART-CAN-NULL-003 | `null-safety-flow-patterns` |
| DART-CAN-NULL-004 | `null-safety-flow-patterns` |
| DART-CAN-FLOW-001 | `null-safety-flow-patterns` |
| DART-CAN-FLOW-002 | `null-safety-flow-patterns` |
| DART-CAN-FLOW-003 | `null-safety-flow-patterns` |
| DART-CAN-FLOW-004 | `null-safety-flow-patterns` |
| DART-CAN-FLOW-005 | `null-safety-flow-patterns` |
| DART-CAN-COLL-001 | `null-safety-flow-patterns` |
| DART-CAN-COLL-002 | `null-safety-flow-patterns` |
| DART-CAN-COLL-003 | `null-safety-flow-patterns` |

### 7. Asynchrony, streams, isolates, and values

| Canonical ID | Primary project |
|---|---|
| DART-CAN-ASYNC-001 | `async-and-isolates` |
| DART-CAN-ASYNC-002 | `async-and-isolates` |
| DART-CAN-ASYNC-003 | `async-and-isolates` |
| DART-CAN-ASYNC-004 | `async-and-isolates` |
| DART-CAN-ISO-001 | `async-and-isolates` |
| DART-CAN-ISO-002 | `async-and-isolates` |
| DART-CAN-VAL-001 | `primitive-values` |
| DART-CAN-VAL-002 | `primitive-values` |
| DART-CAN-VAL-003 | `functions-and-dispatch` |
| DART-CAN-VAL-004 | `primitive-values` |

### 8. Packages, workspaces, tests, and project conventions

| Canonical ID | Primary project |
|---|---|
| DART-CAN-PKG-001 | `packages-and-workspace` |
| DART-CAN-PKG-002 | `packages-and-workspace` |
| DART-CAN-PKG-003 | `packages-and-workspace` |
| DART-CAN-PKG-004 | `packages-and-workspace` |
| DART-CAN-PKG-005 | `packages-and-workspace` |
| DART-CAN-PKG-006 | `packages-and-workspace` |
| DART-CAN-PKG-007 | `packages-and-workspace` |
| DART-CAN-PKG-008 | `packages-and-workspace` |
| DART-CAN-PKG-009 | `packages-and-workspace` |

### 9. Static analysis, diagnostics, metadata, and documentation

| Canonical ID | Primary project |
|---|---|
| DART-CAN-DIAG-002 | `analysis-and-docs` |
| DART-CAN-DIAG-003 | `analysis-and-docs` |
| DART-CAN-DIAG-004 | `analysis-and-docs` |
| DART-CAN-DIAG-005 | `analysis-and-docs` |
| DART-CAN-DIAG-006 | `analysis-and-docs` |
| DART-CAN-DIAG-007 | `analysis-and-docs` |
| DART-CAN-DIAG-008 | `analysis-and-docs` |

### 10. Code generation and source transformation

| Canonical ID | Primary project |
|---|---|
| DART-CAN-BUILD-001 | `code-generation` |
| DART-CAN-BUILD-002 | `code-generation` |
| DART-CAN-BUILD-003 | `code-generation` |
| DART-CAN-BUILD-004 | `code-generation` |
| DART-CAN-BUILD-005 | `code-generation` |
| DART-CAN-BUILD-006 | `code-generation` |

### 11. Platforms, compilation, and interoperation

| Canonical ID | Primary project |
|---|---|
| DART-CAN-PLAT-001 | `platforms-and-interop` |
| DART-CAN-PLAT-002 | `platforms-and-interop` |
| DART-CAN-PLAT-003 | `platforms-and-interop` |
| DART-CAN-PLAT-004 | `platforms-and-interop` |
| DART-CAN-PLAT-005 | `platforms-and-interop` |
| DART-CAN-PLAT-006 | `platforms-and-interop` |
| DART-CAN-PLAT-007 | `platforms-and-interop` |
| DART-CAN-PLAT-008 | `platforms-and-interop` |
| DART-CAN-PLAT-009 | `platforms-and-interop` |
| DART-CAN-PLAT-010 | `platforms-and-interop` |
| DART-CAN-PLAT-011 | `platforms-and-interop` |

## Exceptions

### Item-level exclusions

| Canonical ID | Category | Reason |
|---|---|---|
| DART-CAN-CFG-005 | Unsupported conditional context | The pinned SDK's default experiment set is empty. Its only defined experiments, `macros` and `enhanced-parts`, are the cancelled metaprogramming features that DART-CAN-BUILD-006 forbids and DART-GAP-001 leaves unresolved, so no valid-source experiment can be isolated under this SDK. |
| DART-CAN-DIAG-001 | Invalid-only item | The item's whole obligation is invalid programs and diagnostic behavior across syntax, type, resolution, override, capability, const-evaluation, inference-cycle, and style diagnostics. The valid-source corpus represents none of it. |

### Variant-level exclusions

These distinct variants are removed because their context is unavailable or
their only representation is invalid source. The parent item remains in scope
through its valid, available variants.

| Item and variant | Category | Reason |
|---|---|---|
| DART-CAN-PKG-001, DART-CAN-PKG-002, DART-CAN-PKG-009 Flutter variants (Flutter SDK constraint, Flutter configuration, Flutter plugin platform maps) | Unsupported conditional context | No Flutter SDK is installed. Pure-Dart manifest, layout, and platform-metadata variants remain in scope. |
| DART-CAN-PLAT-004, DART-CAN-PLAT-005, DART-CAN-PLAT-006 browser-runtime variants (DOM and CSP behavior, browser embedding, running compiled JavaScript or WebAssembly) | Unsupported conditional context | No browser is installed. Headless compilation to the `js` and `wasm` targets remains in scope. |
| DART-CAN-PKG-003 Git dependency source | Unsupported conditional context | No offline Git package authority is provisioned. Hosted and path sources, which the item requires for package-resolution coverage, remain in scope. |
| DART-CAN-ISO-001 web isolate and `spawnUri` web variant | Unsupported conditional context | Isolates are not available on the web target. Native isolate spawning and messaging remain in scope. |
| DART-CAN-BUILD-006 optional rejected or obsolete macro sample | Invalid-only variant | The optional preserved macro sample is invalid or obsolete under the pinned SDK. The required negative constraint, demonstrated by using `build_runner` as the stable mechanism, remains in scope. |

Context-rejection variants named elsewhere in the checklist (source that is valid
in one target and rejected in another) are represented only by their valid side,
per the valid-source rule in the ground rules. They are not enumerated
individually.

### Research gaps

The checklist's Section 12 gaps DART-GAP-001 through DART-GAP-011 are not part of
the canonical required stable checklist and are not planned. Where a gap concerns
the exact release boundary of an otherwise-available feature, the canonical
behavior is still covered by its owning project; only the disputed boundary is
excluded. `macros` and `augmentations` (DART-GAP-001) are additionally held out
by the DART-CAN-BUILD-006 negative constraint.

## Independence audit

- Every project occupies a single exclusive directory directly under
  `fixtures/dart/projects/`. No two projects share a directory, and the twelve
  directories listed in the Projects table are pairwise distinct.
- No project imports, depends on, or references source in another top-level
  project. Each project contains every local package it needs and resolves only
  the SDK, its own local packages, and its own vendored third-party packages.
- Third-party dependency needs are confined to `packages-and-workspace`
  (`package:test` and one small hosted package), `code-generation` (the pinned
  `build` and generator toolchain), and `analysis-and-docs` (`package:meta`).
  Each vendors its own copies; none is shared across projects.
- `platforms-and-interop` binds a documented system library through `dart:ffi`
  and uses no third-party package. Its native toolchain use is confined to the
  implementor's setup and is not invoked by required validation tasks.
- Every project's required `build`, `lint`, and `test` tasks run offline against
  committed resolution and download nothing.
