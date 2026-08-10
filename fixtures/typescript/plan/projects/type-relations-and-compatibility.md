---
relationships:
  references:
    - ground-rules
    - reports/typescript/synthesis/checklist
---

# Project brief: type-relations-and-compatibility

## Purpose

A library that demonstrates the core type relations: the primitive, literal, top,
and bottom types; structural compatibility, freshness, and weak types; unions,
intersections, and discriminated unions; nullability and optional distinctions;
readonly views; and callable compatibility and parameter variance.

## Exclusive directory

`fixtures/typescript/projects/type-relations-and-compatibility/`

## Difficulty

`routine`

## Assigned coverage

Only valid-source obligations are corpus content. Excess-property typos,
weak-type errors, default-branch masking, and JSON-omission failures are excluded
per ground rules.

- TS-CAN-058 — primitives and literals contrasted with `object`, `{}`, `unknown`,
  `any`, `never`, and `void` through assignments, calls, property operations,
  exhaustiveness, and callback returns; `any` propagation and evolving `any`;
  `unknown` refinement; `never` from impossible branches; a `void` callback
  accepting a value return; boxed primitives; nullability.
- TS-CAN-059 — independently declared equal shapes assigned; a fresh object
  literal versus an intermediate variable; an extra-property object passed through
  assertion, spread, `satisfies`, and generic inference; a weak all-optional
  target; required, optional, readonly, call, construct, and index members; a
  branded primitive intersection; recursive shapes; exact optional types.
- TS-CAN-060 — tagged and untagged unions with member narrowing; intersection of
  compatible call and object types; contradictory intersections reducing to
  `never`; an optional or mutated discriminant; union absorption; an
  object-literal union; conditional-type distribution interaction.
- TS-CAN-063 — `null`, `undefined`, a missing optional property, a required
  property containing `undefined`, an optional or default parameter, an optional
  tuple element, and presence tests under strict and legacy null modes;
  `exactOptionalPropertyTypes`; mapped optional modifiers; object spread; optional
  chaining; an explicit `undefined` write; non-strict collapse.
- TS-CAN-065 — one mutable object viewed through mutable and readonly aliases;
  readonly property, index, array, and tuple forms; mapped modifier add and
  remove; a shallow const assertion; nested mutation; runtime mutation through
  another alias; a getter-only property; readonly-array variance; `Object.freeze`
  typing; write rejection through a readonly view.
- TS-CAN-067 — callbacks with broader and narrower parameters, different counts,
  optional and rest parameters, discarded return values, generic callbacks, and
  method versus function-property declarations under strict and permissive
  checking; a contravariant function property; an intentionally bivariant method;
  a `void` target return; a constructor signature; an overload set;
  `strictFunctionTypes` off.

## Declared compilation contexts

- Strict checking and a `strictFunctionTypes`-off context.
- `strictNullChecks` on and off.
- An `exactOptionalPropertyTypes` context.

## Dependency needs

None.

## Generated-source needs

None.

## Planned tests

`none`. Type relations are created by source and configuration; no assigned
coverage is created by test source.

## Required interfaces

- `Taskfile.yml` defines `build`, `lint`, and `test` as thin wrappers per ground
  rules; `build` compiles under each declared strictness context.
- `coverage.md` records the ground-rules coverage table for every assigned
  identifier, using the compilation-context column where non-default and
  additional locators for distinct valid variants.
