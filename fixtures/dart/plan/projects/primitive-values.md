---
relationships:
  references:
    - ground-rules
    - overview
---

# Project brief: primitive-values

## Purpose

An encoding library that exercises numbers with per-target semantics, strings
and Unicode, and typed data across compilation targets.

## Exclusive directory

`fixtures/dart/projects/primitive-values/`

## Difficulty

complex

## Assigned coverage

- **DART-CAN-VAL-001** — Numbers and target semantics: decimal, hex, and
  supported digit-separated literals, arithmetic, `~/`, modulo, shifts,
  overflow-sensitive operations, `NaN`, infinities, signed zero, conversions,
  and numeric literal context typing; an integer literal accepted in a double
  context; per-target range, wrapping, identity, equality, and formatting
  differences across native, Wasm, and JavaScript.
- **DART-CAN-VAL-002** — Strings and Unicode: single, multiline, and raw
  strings, adjacent literals, escapes, interpolation, equality, UTF-16 code
  units, and runes and scalars; raw interpolation suppression; const
  interpolation; code-unit versus scalar length.
- **DART-CAN-VAL-004** — Typed data and views: buffers, typed lists, byte data,
  overlapping views, endianness, and bounds behavior; native versus JavaScript
  backing; supported view kinds.

Per-target numeric and typed-data differences are recorded as target-specific
observations. Backend-dependent representation is not asserted for portable
equality.

## Declared build contexts

- `vm-jit` (default).
- `native-aot`, for native numeric and typed-data representation.
- `js`, for JavaScript numeric range and typed-data backing.
- `wasm`, for WebAssembly numeric and typed-data behavior.

## Dependency needs

None. SDK libraries (`dart:core`, `dart:typed_data`) only.

## Generated-source needs

None.

## Planned tests

None. All coverage is created by library source.

## Required interfaces

Provide `Taskfile.yml` and `coverage.md` as defined in the ground rules. `build`
resolves offline, analyzes all fixture-owned libraries, and compiles the
declared entry points under `native-aot`, `js`, and `wasm`. `lint` runs `dart
format` and `dart analyze`. `test` runs `dart test` and succeeds with no test
files.
