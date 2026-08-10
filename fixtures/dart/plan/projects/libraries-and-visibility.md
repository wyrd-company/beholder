---
relationships:
  references:
    - ground-rules
    - overview
---

# Project brief: libraries-and-visibility

## Purpose

A multi-library package that exercises Dart's library, directive, privacy, and
resolution model, including façade exports, parts, prefixes, and conditional and
deferred imports.

## Exclusive directory

`fixtures/dart/projects/libraries-and-visibility/`

## Difficulty

complex

## Assigned coverage

- **DART-CAN-LIB-001** — Library identity and directives: omitted, unnamed, and
  legacy named library directives; the library as namespace and privacy unit;
  two libraries sharing a documentary name; library metadata and documentation.
- **DART-CAN-LIB-002** — Import URI authorities and implicit core: `dart:`,
  `package:`, and relative imports; the implicit `dart:core` import; explicit
  `dart:core hide`; a local declaration shadowing a core name.
- **DART-CAN-LIB-003** — Prefixes and merged imports: prefixed and unprefixed
  imports; multiple imports sharing one prefix; repeated imports of one library;
  a distinct prefix for a deferred import; explicit extension application through
  a prefix.
- **DART-CAN-LIB-004** — `show` and `hide` combinators: filtering imports and
  exports; filtering to resolve a collision; chained and repeated combinators; a
  re-export diamond that is not ambiguous.
- **DART-CAN-LIB-005** — Exports and façade libraries: re-exporting selected
  public declarations while importing implementation-only dependencies; a chain,
  a diamond, and a legal export cycle; a public `lib/src` separation as a
  convention.
- **DART-CAN-LIB-006** — Classic parts and ownership: a library owning at least
  one part sharing imports, namespace, and private names; URI and legacy named
  `part of`.
- **DART-CAN-LIB-007** — Underscore privacy: private top-level names, members,
  constructors, types, and extensions; same-text private names in two libraries;
  parts sharing access; two classes in one library accessing each other's
  private members.
- **DART-CAN-LIB-008** — Conditional imports and exports: default, native, and
  web branches selected by `dart.library.*` conditions behind one common API;
  the analyzer default-branch behavior.
- **DART-CAN-LIB-009** — Deferred imports: `deferred as`, awaiting
  `loadLibrary()`, and access through the prefix; the VM trivial load and the
  web code-splitting load unit.
- **DART-CAN-LIB-010** — Lexical scope, shadowing, and legal cycles:
  block/local/member/library lookup; `this` disambiguation; interpolation
  references; mutually importing libraries.
- **DART-CAN-LIB-011** — Wildcard variables: multiple `_` parameters, locals,
  pattern, and catch positions that create no binding in a modern-language file,
  contrasted with an older-version file; member and top-level `_` declarations
  that remain declarations; the unnamed extension idiom.

The generated-part interaction named by DART-CAN-LIB-006 is owned by
`code-generation`; here parts are hand-written classic parts.

## Declared build contexts

- `vm-jit` (default).
- `js`, for the web branch of the conditional-import library and the web
  deferred-load unit.

## Dependency needs

None. SDK libraries and this project's own local libraries only.

## Generated-source needs

None.

## Planned tests

None. All coverage is created by library source.

## Required interfaces

Provide `Taskfile.yml` and `coverage.md` as defined in the ground rules. `build`
resolves offline, analyzes all fixture-owned libraries, and compiles the web
entry point under `js`. `lint` runs `dart format` and `dart analyze`. `test`
runs `dart test` and succeeds with no test files.
