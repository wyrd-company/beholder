---
relationships:
  references:
    - ground-rules
    - reports/typescript/synthesis/checklist
---

# Project brief: compiler-options-matrix

## Purpose

One library compiled under contrasting effective-option sets so that the same
source shows how target and library selection, the strictness family,
independent safety and hygiene flags, and check-versus-emit decoupling change
inferred types, diagnostics gating, and emitted output.

## Exclusive directory

`fixtures/typescript/projects/compiler-options-matrix/`

## Difficulty

`complex`

## Assigned coverage

Only valid-source obligations are corpus content. Where a canonical item names
failure or diagnostic variants, those are excluded per ground rules.

- TS-CAN-006 — modern syntax and built-in use under an old and a modern target,
  showing syntax lowering, default `lib` selection, and the target-dependent
  class-field default, plus that successful checking supplies no runtime
  polyfill; explicit `lib`, `noLib`, `/// <reference lib>`, DOM and WebWorker
  overlap, library replacement, `libReplacement`, and `skipDefaultLibCheck`.
- TS-CAN-007 — focused type-changing cases across the strictness family
  (`strictNullChecks`, `noImplicitAny`, `strictFunctionTypes`,
  `strictBindCallApply`, `strictPropertyInitialization`, `noImplicitThis`,
  `useUnknownInCatchVariables`, `alwaysStrict`), including a child option
  overriding `strict: true`; method bivariance versus function-property
  contravariance; catch `unknown` versus `any`; null collapse when strict-null
  checking is off.
- TS-CAN-008 — `exactOptionalPropertyTypes`, `noUncheckedIndexedAccess`,
  `noPropertyAccessFromIndexSignature`, `noImplicitOverride`, unused checks,
  implicit-return checks, switch-fallthrough policy, and unreachable-code policy
  exercised independently as valid or type-changing cases.
- TS-CAN-015 — its valid-source obligations: whole-program-valid source that
  satisfies each `isolatedModules` and `isolatedDeclarations` restriction
  (including the repaired, per-file-transpilable forms), and `noEmit` and
  `emitDeclarationOnly` emit modes over valid programs. The emit-despite-type-
  error, `noEmitOnError`, and `noCheck` decoupling variants require invalid
  source and are excluded.

## Declared compilation contexts

- Multiple effective-option sets under the pinned compiler: an old target and a
  modern target with contrasting default libraries; `strict: true` and a
  child-overridden strictness set; each independent safety and hygiene flag; and
  `noEmit`, `emitDeclarationOnly`, `isolatedModules`, and `isolatedDeclarations`
  configurations.

## Dependency needs

None.

## Generated-source needs

None.

## Planned tests

`none`. Every obligation is expressed in ordinary source under contrasting
configuration; no assigned coverage is created by test source.

## Required interfaces

- `Taskfile.yml` defines `build`, `lint`, and `test` as thin wrappers per ground
  rules; `build` type-checks and emits across every declared option set.
- `coverage.md` records the ground-rules coverage table for every assigned
  identifier, using the compilation-context column to identify the option set
  that creates each coverage.
