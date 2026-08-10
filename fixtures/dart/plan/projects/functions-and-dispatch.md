---
relationships:
  references:
    - ground-rules
    - overview
---

# Project brief: functions-and-dispatch

## Purpose

A function-oriented library that exercises function kinds, function types and
tear-offs, dynamic invocation, entry points, and symbols.

## Exclusive directory

`fixtures/dart/projects/functions-and-dispatch/`

## Difficulty

complex

## Assigned coverage

- **DART-CAN-FN-001** — Function kinds, parameters, bodies, and capture:
  top-level, static, instance, local, and anonymous functions; block and arrow
  bodies; all parameter forms and defaults; lexical capture and a recursive
  closure; loop-iteration capture; named arguments interleaved where supported.
- **DART-CAN-FN-002** — Function types and tear-offs: structural and generic
  function types, callable-object coercion, and top-level, static, bound, and
  generic method tear-offs; parameter and return variance; `Function`; a
  getter-returning-function versus a method; an extension tear-off.
- **DART-CAN-FN-003** — Dynamic invocation, `Function.apply`, and
  `noSuchMethod`: a valid dynamic call, `Function.apply` with named symbols, and
  a class implementing an interface via `noSuchMethod` forwarders; ordinary
  virtual dispatch versus static, constructor, extension, and extension-type
  resolution.
- **DART-CAN-FN-004** — Entry-point signatures and roots: synchronous and
  asynchronous `main`, arguments, and multiple entry libraries; an isolate entry
  tear-off; an entry-point reachability pragma.
- **DART-CAN-VAL-003** — Symbols and type/name observability: symbol literals
  and `Symbol` construction in `Function.apply`; private-name symbols; symbol
  equality.

The test-entry-root variant of DART-CAN-FN-004 is represented through overlap in
`packages-and-workspace`, and the web-bootstrap variant through overlap in
`platforms-and-interop`; this project owns the executable entry-point variants.
The mirrors variant of DART-CAN-VAL-003 is owned by `platforms-and-interop`.
Minified or obfuscated error text and runtime name strings are recorded, not
asserted for portable equality.

## Declared build contexts

- `vm-jit` (default).
- `native-aot`, for dispatch, tear-off identity, and reachability under
  ahead-of-time compilation.

## Dependency needs

None. SDK libraries only.

## Generated-source needs

None.

## Planned tests

None. All coverage is created by library and executable source.

## Required interfaces

Provide `Taskfile.yml` and `coverage.md` as defined in the ground rules. `build`
resolves offline, analyzes all fixture-owned libraries, and compiles the
declared entry points under `native-aot`. `lint` runs `dart format` and `dart
analyze`. `test` runs `dart test` and succeeds with no test files.
