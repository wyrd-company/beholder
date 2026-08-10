---
relationships:
  references:
    - ground-rules
    - reports/typescript/synthesis/checklist
---

# Project brief: checked-javascript-inputs

## Purpose

A checked-JavaScript library that shows how TypeScript treats JavaScript source:
the TypeScript JSDoc dialect, inferred CommonJS and constructor-function shapes,
JavaScript imported by TypeScript and the reverse, and declaration emit from
JavaScript.

## Exclusive directory

`fixtures/typescript/projects/checked-javascript-inputs/`

## Difficulty

`routine`

## Assigned coverage

Only valid-source obligations are corpus content. Unsupported or ignored JSDoc
tags and declaration-portability failures are excluded per ground rules.

- TS-CAN-013 — checked and unchecked `.js` and `.jsx` files imported by
  TypeScript and vice versa; `@typedef`, `@callback`, `@template`, `@overload`,
  `@satisfies`, `@import`, casts, prototype members, expando properties, and
  CommonJS assignment exports (`module.exports`, `exports.x`, `require`);
  constructor-function prototypes; `allowJs`, `checkJs`, `@ts-check`,
  `@ts-nocheck`, and `maxNodeModuleJsDepth`; and declaration emit from
  JavaScript.

## Declared compilation contexts

- An `allowJs` and `checkJs` context.
- A declaration-emit-from-JavaScript context (`declaration` or
  `emitDeclarationOnly`).

## Dependency needs

None.

## Generated-source needs

None.

## Planned tests

`none`. JavaScript inference and JSDoc behavior are created by source and
configuration; no assigned coverage is created by test source.

## Required interfaces

- `Taskfile.yml` defines `build`, `lint`, and `test` as thin wrappers per ground
  rules; `build` checks and emits under the declared contexts.
- `coverage.md` records the ground-rules coverage table for TS-CAN-013, using the
  compilation-context column where non-default and additional locators for the
  distinct valid variants.
