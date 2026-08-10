---
relationships:
  references:
    - ground-rules
    - overview
---

# Project brief: construction-and-constants

## Purpose

A value-oriented library that exercises object construction, the constant
subset, canonicalization, and identity.

## Exclusive directory

`fixtures/dart/projects/construction-and-constants/`

## Difficulty

complex

## Assigned coverage

- **DART-CAN-OBJ-001** — Generative and named constructors: unnamed and named
  forms, initializing formals, initializer lists, assertions, explicit and
  implicit super calls, and super parameters; initialization order; final and
  non-nullable field coverage.
- **DART-CAN-OBJ-002** — Synthesized default and private construction: the
  implicit default constructor; a class with only private constructors;
  static-only and singleton idioms; implicit `extends Object`.
- **DART-CAN-OBJ-003** — Factories and redirection: a caching or
  subtype-returning factory, a cross-type redirecting factory, a const factory
  redirect, and a same-class redirecting generative constructor; generic
  forwarding; a redirect chain.
- **DART-CAN-OBJ-004** — Constructor tear-offs: `C.new`, named, prefixed, and
  instantiated generic constructor tear-offs as function values; const-context
  identity; alias-mediated constructor access.
- **DART-CAN-OBJ-005** — Cascades and null-shorting cascades: `..`, `?..`,
  calls, assignments, and index sections; the original receiver as the result;
  a first null-aware section that short-circuits later sections; nested cascades.
- **DART-CAN-OBJ-006** — Implicit operator lowering: `a[i] += value`, property
  `??=`, and increment and decrement on user-defined members; the `[]`,
  operator, and `[]=` sequence; a getter then a conditional setter; specified
  single evaluation of a side-effecting receiver and index.
- **DART-CAN-CONST-001** — Constant-expression subset: const variables,
  primitive operations, strings, symbols, type, function, and constructor
  literals, collections, records, annotations, and const constructors with
  potentially-constant initializers; an implicit const context; a generic
  constant; an environment constant.
- **DART-CAN-CONST-002** — Canonicalization and identity: repeated equal const
  objects, collections, records, symbols, and cross-library references compared
  with non-const instances; a const factory; generic instantiation.
- **DART-CAN-CONST-003** — Equality, identity, and hashing: an overridden `==`
  with a matching `hashCode`, `identical`, and map and set keys; records and
  enums as keys; documented numeric and `NaN` cases.

The deferred-load-unit const-identity corner named by DART-CAN-CONST-002 is a
backend observation recorded, not asserted; deferred load units are owned by
`libraries-and-visibility` and `platforms-and-interop`. Hash values are recorded,
not asserted for portable equality.

## Declared build contexts

- `vm-jit` (default).
- `native-aot`, for canonicalization and identity under ahead-of-time
  compilation.

## Dependency needs

None. SDK libraries only.

## Generated-source needs

None.

## Planned tests

None. All coverage is created by library source.

## Required interfaces

Provide `Taskfile.yml` and `coverage.md` as defined in the ground rules. `build`
resolves offline, analyzes all fixture-owned libraries, and compiles the
declared entry point under `native-aot`. `lint` runs `dart format` and `dart
analyze`. `test` succeeds with no test files and needs no test-framework
dependency.
