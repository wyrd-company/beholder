---
relationships:
  references:
    - ground-rules
    - reports/typescript/synthesis/checklist
---

# Project brief: generics-and-inference

## Purpose

A generic library that covers generic declarations, constraints, defaults, and
inference; const type parameters, instantiation expressions, `NoInfer`, and
variance annotations; index signatures; arrays, tuples, and variadic relations;
hybrid object types; and polymorphic `this`.

## Exclusive directory

`fixtures/typescript/projects/generics-and-inference/`

## Difficulty

`complex`

## Assigned coverage

Only valid-source obligations are corpus content. Failed `keyof` constraints,
empty candidate sets, circular constraints, wrong-variance diagnostics,
out-of-bounds access, and detached-call failures are excluded per ground rules.

- TS-CAN-064 — string, number, symbol, template-pattern, union, and readonly
  index signatures; compatible named members; reads under checked and unchecked
  access settings; a number index constrained by a string index; array and tuple
  indexing; `keyof` widening; an undeclared key gaining `undefined` under
  `noUncheckedIndexedAccess`; an explicit member override.
- TS-CAN-066 — fixed, optional, readonly, labeled, leading, middle, rest, and
  generic variadic tuples through indexing, destructuring, concatenation,
  parameter lists, and spread into calls; a union rest; labels preserved but
  identity-neutral; readonly forms.
- TS-CAN-068 — generic functions, classes, interfaces, aliases, and call and
  construct signatures; constraints; defaults; explicit and inferred arguments;
  nested shadowed parameters; context- and return-driven inference; a default
  referencing an earlier parameter; higher-order inference; a recursive generic.
- TS-CAN-069 — paired ordinary and `const` type-parameter APIs; an instantiation
  expression specializing a generic function value; `NoInfer` blocking one
  inference site; correct `in`, `out`, and `in out` variance annotations; an
  inline versus a pre-widened argument; a readonly candidate against a mutable
  constraint.
- TS-CAN-076 — a stateful callable object with properties; separate call and
  construct signatures returning different types; generic signatures; overloads;
  index members; a runtime implementation assigned to it; an abstract construct
  signature; constructor `this`; number and string index compatibility.
- TS-CAN-077 — a base fluent method returning `this` chained through a derived
  subtype; a `this is T` predicate; a method returning the named base type; an
  explicit `this` parameter; a static-side contrast; an F-bounded generic
  alternative; a mixin or intersection builder.

## Declared compilation contexts

- Default context under the pinned compiler, which supplies instantiation
  expressions and variance annotations, const type parameters, and `NoInfer`.
- A `noUncheckedIndexedAccess` context for TS-CAN-064.

## Dependency needs

None.

## Generated-source needs

None.

## Planned tests

`none`. Generic and inference behavior is created by source and configuration; no
assigned coverage is created by test source.

## Required interfaces

- `Taskfile.yml` defines `build`, `lint`, and `test` as thin wrappers per ground
  rules; `build` type-checks and emits under the declared contexts.
- `coverage.md` records the ground-rules coverage table for every assigned
  identifier, using the compilation-context column where non-default and
  additional locators for distinct valid variants.
