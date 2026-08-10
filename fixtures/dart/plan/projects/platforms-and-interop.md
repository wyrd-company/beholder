---
relationships:
  references:
    - ground-rules
    - overview
---

# Project brief: platforms-and-interop

## Purpose

A cross-platform library that exercises the compilation-target matrix,
platform-library availability, JIT-versus-AOT reachability, web and WebAssembly
compilation, modern JavaScript interop, native FFI, process and filesystem,
mirrors, pragmas, external declarations, and non-portable observations.

## Exclusive directory

`fixtures/dart/projects/platforms-and-interop/`

## Difficulty

complex

## Assigned coverage

- **DART-CAN-PLAT-001** — Compilation target matrix: suitable entry points
  compiled under VM JIT, native AOT, JavaScript, and WebAssembly; a
  target-specific entry point.
- **DART-CAN-PLAT-002** — Platform library availability: native-only and
  web-only imports isolated behind conditional libraries (`dart:io`, `dart:ffi`,
  `dart:isolate`, `dart:mirrors`, `dart:js_interop`).
- **DART-CAN-PLAT-003** — JIT versus AOT and reachability: the same valid
  behavior in JIT and AOT with deliberately unreachable code and an entry-point
  preservation pragma; a tree-shaken declaration.
- **DART-CAN-PLAT-004** — JavaScript compilation: a web entry compiled with
  modern web and interop APIs, documenting numeric, deferred, module, and
  minification configurations.
- **DART-CAN-PLAT-005** — WebAssembly compilation: a supported entry point
  compiled to WasmGC with the modern interop requirement.
- **DART-CAN-PLAT-006** — Modern JavaScript interop: `@JS` extension types and
  external members, name mapping, conversions, callbacks, and promise-to-future
  conversion; extension-type erasure.
- **DART-CAN-PLAT-007** — Native FFI: at least one symbol bound through `@Native`
  or lookup, native and Dart signatures, and a struct or union, with the native
  library documented; a leaf call; a finalizer or allocator; a native callback
  and its callback-thread relationship.
- **DART-CAN-PLAT-008** — Process, filesystem, and environment: paths, file I/O,
  process invocation and exit, environment variables, and asynchronous I/O on
  native targets.
- **DART-CAN-PLAT-009** — Mirrors: reflective discovery and invocation in a
  JIT-only source set; private symbols; metadata and generics.
- **DART-CAN-PLAT-010** — Pragmas and compiler annotations: only documented
  backend pragmas, including entry-point preservation where needed.
- **DART-CAN-PLAT-011** — Non-portable observations: runtime type strings, stack
  traces, synthetic names, error messages, source maps, hash values, object
  layout, and tree-shaken boundaries recorded as observations and never asserted
  for portable equality.
- **DART-CAN-FN-005** — External declarations: representative `external`
  functions, methods, getters and setters, fields, and constructors in an actual
  supported binding context (FFI `@Native` and JavaScript interop).

Browser-runtime variants of DART-CAN-PLAT-004, DART-CAN-PLAT-005, and
DART-CAN-PLAT-006 (DOM and CSP behavior, browser embedding, and running compiled
output) are excluded; compilation to the `js` and `wasm` targets is covered.
AOT, JavaScript, and Wasm rejection of mirrors is the invalid side of
DART-CAN-PLAT-009 and is not planned; the valid JIT-only reflective source set is
covered. Native FFI binds a documented system library; no third-party package and
no validation-time native compilation are required.

## Declared build contexts

- `vm-jit` (default), including the JIT-only mirrors source set.
- `native-aot`, for AOT reachability, FFI, and process and filesystem behavior.
- `js`, for JavaScript compilation and interop.
- `wasm`, for WebAssembly compilation and interop.

## Dependency needs

None third-party. SDK libraries only. The FFI coverage binds a documented system
library through `dart:ffi`.

## Generated-source needs

None.

## Planned tests

None. All coverage is created by library and executable source across the
target-specific source sets.

## Required interfaces

Provide `Taskfile.yml` and `coverage.md` as defined in the ground rules. `build`
resolves offline, analyzes all fixture-owned libraries, and compiles the declared
entry points under `native-aot`, `js`, and `wasm`. `lint` runs `dart format` and
`dart analyze`. `test` succeeds with no test files and needs no test-framework
dependency.
