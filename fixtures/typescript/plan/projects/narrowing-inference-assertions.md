---
relationships:
  references:
    - ground-rules
    - reports/typescript/synthesis/checklist
---

# Project brief: narrowing-inference-assertions

## Purpose

A library that exercises how the compiler refines and infers types from flow and
authored intent: control-flow narrowing and alias analysis, user-defined
predicates and assertion signatures, contextual typing and widening, and
assertions, definite assignment, and `satisfies`.

## Exclusive directory

`fixtures/typescript/projects/narrowing-inference-assertions/`

## Difficulty

`complex`

## Assigned coverage

Only valid-source obligations are corpus content. Mutation invalidating facts,
unsafe-call boundaries, predicate-target-not-assignable, no-common-candidate, and
misspelled-excess-key failures are excluded per ground rules.

- TS-CAN-061 — `typeof`, equality, truthiness, `in`, `instanceof`, discriminants,
  assignment and reachability, aliased conditions, destructured discriminants,
  `switch (true)`, closure narrowing, and constant indexed-access narrowing;
  empty-string and zero truthiness; an optional property on both `in` branches;
  a custom `Symbol.hasInstance`; captured variables; early return; loops.
- TS-CAN-062 — explicit parameter and `this` predicates; a generic predicate;
  `asserts condition`; `asserts x is T`; an inferred predicate used by `filter`;
  and an intentionally unsound but trusted predicate body, which is valid,
  compilable source.
- TS-CAN-078 — the same callback, object, and array with and without context;
  `let` versus `const` versus a readonly property versus a const assertion versus
  an annotation; a heterogeneous array whose best common type changes; contextual
  return; `undefined`-return inference; generic and overload context; union
  contextual signatures; evolving arrays; a JSX callback; const context.
- TS-CAN-079 — annotation versus `satisfies`; `as const satisfies`; widening,
  narrowing, and double assertions; postfix non-null; declaration definite
  assignment; at least one trusted claim that type-checks but fails at runtime,
  which is valid source whose failure is runtime behavior rather than a compiler
  error; JSDoc `@satisfies`.

## Declared compilation contexts

- Default context under the pinned compiler, which supplies the version-sensitive
  inference behavior named above.
- Strict null checking.

## Dependency needs

None.

## Generated-source needs

None.

## Planned tests

`none`. Narrowing, inference, and assertion behavior is created by source and
configuration; no assigned coverage is created by test source.

## Required interfaces

- `Taskfile.yml` defines `build`, `lint`, and `test` as thin wrappers per ground
  rules; `build` type-checks and emits under the declared context.
- `coverage.md` records the ground-rules coverage table for every assigned
  identifier, using additional locators for distinct valid variants.
