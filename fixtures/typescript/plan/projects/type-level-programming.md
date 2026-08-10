---
relationships:
  references:
    - ground-rules
    - reports/typescript/synthesis/checklist
---

# Project brief: type-level-programming

## Purpose

A type-level library that computes types from types: conditional types and
`infer`, mapped types and key remapping, `keyof`, indexed access, and type
queries, template literal types and intrinsic string transforms, recursive and
circular types, interfaces versus type aliases, and utility types and
value-derived chains.

## Exclusive directory

`fixtures/typescript/projects/type-level-programming/`

## Difficulty

`complex`

## Assigned coverage

Only valid-source obligations are corpus content. Excessive-instantiation and
depth diagnostics, remapped-key collisions, forbidden immediate cycles,
invalid non-key indexing, alias-reopening errors, an interface extending a union,
and missing-lib-utility failures are excluded per ground rules.

- TS-CAN-070 — distributive and bracket-suppressed conditional types;
  constrained and nested `infer`; a `never` input; overload return extraction; a
  recursive unwrapping transformation; `any` and `unknown`; a deferred generic
  condition.
- TS-CAN-071 — homomorphic modifier preservation; `+` and `-` optional and
  readonly modifiers; key filtering via `never`; renaming via `as`;
  template-generated names; tuple and array mapping; symbol and number keys;
  union sources; recursive maps; index signatures.
- TS-CAN-072 — `keyof typeof value`; generic property lookup `T[K]`; array element
  extraction; nested indexed access; `typeof` of functions, classes, and
  namespaces; `typeof import()`; string-index widening of `keyof`; number and
  symbol keys; an optional result including `undefined`; unions versus
  intersections.
- TS-CAN-073 — an event-name and key API generated from object keys; a
  cross-product of multiple union substitutions; substring parsing with
  conditional inference; `Uppercase`, `Lowercase`, `Capitalize`, and
  `Uncapitalize`; wide `string`; numeric, bigint, and boolean interpolation;
  key-remapping interaction; a recursive parser.
- TS-CAN-074 — a recursive interface or tree; a recursive tuple or object alias;
  mutually recursive aliases; a conditional or mapped recursion; a circular
  property reference; a serialization-shaped cycle; alias versus interface
  presentation.
- TS-CAN-075 — equivalent object shapes as an interface and an alias; reopening
  only the interface; an alias for primitive, union, tuple, and conditional
  forms; interface extension versus alias intersection versus class
  implementation; generic recursion; module augmentation; declaration-emit alias
  preservation.
- TS-CAN-080 — property-transform, union-filter, function-reflection, `this`,
  string, and `Awaited` utilities applied to unions, overloads, optional
  properties, `any`, and `never`; a small value or schema API whose exported type
  exists only through generic instantiation; last-overload extraction; a
  recursive thenable; an `isolatedDeclarations` repair with annotation.

## Declared compilation contexts

- Default context under the pinned compiler.

## Dependency needs

None.

## Generated-source needs

None.

## Planned tests

`none`. Type-level computation is created by source; no assigned coverage is
created by test source.

## Required interfaces

- `Taskfile.yml` defines `build`, `lint`, and `test` as thin wrappers per ground
  rules; `build` type-checks and emits the library.
- `coverage.md` records the ground-rules coverage table for every assigned
  identifier, using additional locators for distinct valid variants.
