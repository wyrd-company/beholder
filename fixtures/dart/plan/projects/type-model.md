---
relationships:
  references:
    - ground-rules
    - overview
---

# Project brief: type-model

## Purpose

An idiomatic library that models a generic, non-identifying domain through
Dart's nominal and structural type system, generics, variance, and override
checking.

## Exclusive directory

`fixtures/dart/projects/type-model/`

## Difficulty

complex

## Assigned coverage

- **DART-CAN-TYPE-001** — Classes, abstract declarations, and implicit
  interfaces: `extends`, `implements`, inherited concrete members, abstract
  members, overrides, and `super`; fields as getter/setter contracts.
- **DART-CAN-TYPE-002** — Class modifiers and capability restrictions: `base`,
  `interface`, `final`, `sealed`, abstract combinations, `mixin class`, and
  `base mixin` across a library boundary; sealed exhaustiveness.
- **DART-CAN-TYPE-003** — Mixins and mixin applications: `mixin`, `on`
  constraints, multiple ordered applications, `super` calls, and a named mixin
  application.
- **DART-CAN-TYPE-004** — Simple and enhanced enums: values with fields,
  methods, const constructor arguments, an interface, a mixin, and synthesized
  members; a generic enhanced enum.
- **DART-CAN-TYPE-005** — Extension methods: named, unnamed, private, generic,
  bounded, nullable-target, getter, operator, static, and explicit extension
  forms; prefixed explicit application; extension tear-off.
- **DART-CAN-TYPE-006** — Extension types: representation, constructors,
  members, operators, static members, generics, and `implements`; static
  distinction and runtime erasure; `@redeclare`.
- **DART-CAN-TYPE-007** — Type aliases: legacy function typedef, modern function
  alias, generalized class and type alias, generic bounds, a record alias, and
  constructor access through an alias.
- **DART-CAN-TYPE-008** — Records: positional, named, singleton positional,
  nested, const, generic, return, destructuring, and cross-library record
  shapes; structural subtyping; synthetic getters.
- **DART-CAN-TYPE-009** — Special types: `Object?`, `Object`, `dynamic`, `void`,
  `Never`, `Null`, and `FutureOr<T>` in assignments, calls, returns, promotion,
  and reachability.
- **DART-CAN-TYPE-010** — Type tests, casts, literals, and runtime type: `is`,
  `is!`, `as`, generic type tests, type literals, and `runtimeType`; reified
  generic arguments; type objects as values.
- **DART-CAN-TYPE-011** — Fields and accessors: instance, static, and top-level
  fields; `final`, `late`, `late final`, static `const`, and abstract fields;
  explicit getters and setters; a getter and setter inherited from different
  supertypes.
- **DART-CAN-TYPE-012** — Operators, indexers, and callable objects:
  representative unary and binary operators, `[]`, `[]=`, `==`/`hashCode`, and
  `call`; a callable object assigned to a function type; `>>>`.
- **DART-CAN-GEN-001** — Generic declarations and bounds: generic classes,
  mixins, extensions, extension types, aliases, functions, methods, and
  constructors; upper, dependent, and F-bounds; explicit and inferred arguments;
  the raw form; `extends Object?`.
- **DART-CAN-GEN-002** — Contextual and cross-unit inference: upward and
  downward inference for locals, top-levels, fields, literals, closures,
  records, conditionals, switches, generic calls, and overrides; inferred
  declarations across libraries; least-upper-bound results.
- **DART-CAN-GEN-003** — Generic and function variance: class covariance,
  function parameter contravariance, return covariance, and a mutable generic
  upcast with an inserted runtime write check.
- **DART-CAN-GEN-004** — Override checking and `covariant`: valid return
  covariance, parameter compatibility, optional and named additions, and a
  deliberately narrowed parameter with `covariant`; a call through a supertype
  causing a runtime check; generic substitution.

The JavaScript-interop variant of DART-CAN-TYPE-006 is owned by
`platforms-and-interop`; here extension types are covered by static distinction
and runtime erasure. Runtime type-string observations are recorded, not asserted
for portable equality.

## Declared build contexts

- `vm-jit` (default).
- `native-aot`, for runtime type tests, casts, variance write checks, and
  reified generics under ahead-of-time compilation.

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
analyze`. `test` runs `dart test` and succeeds with no test files.
