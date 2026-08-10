---
relationships:
  references:
    - ground-rules
    - reports/typescript/synthesis/checklist
---

# Project brief: decorators

## Purpose

A library that covers both decorator regimes in separate, mutually exclusive
compilations: standard decorators with their context, replacement, and metadata
behavior, and legacy `experimentalDecorators` with emitted design metadata.

## Exclusive directory

`fixtures/typescript/projects/decorators/`

## Difficulty

`complex`

## Assigned coverage

Only valid-source obligations are corpus content. Incompatible return types,
missing metadata polyfill cases, and decorator-signature-under-the-wrong-regime
failures are excluded per ground rules.

- TS-CAN-054 — standard decorator mode covering class, method, getter and setter,
  field, and auto-accessor targets; factories; replacement values;
  `addInitializer`; typed context; ordering; `Symbol.metadata` where supported;
  static and instance elements; private elements; inheritance; metadata
  inheritance.
- TS-CAN-055 — a separate `experimentalDecorators` compilation covering class,
  member, and parameter decorators and `emitDecoratorMetadata`; `design:type`,
  `design:paramtypes`, and `design:returntype`; a comparison of emit with
  standard decorators; annotation-derived runtime constructor references;
  descriptor mutation.

## Declared compilation contexts

- A standard-decorator compilation (default regime), with a `Symbol.metadata`
  context where supported.
- A separate, mutually exclusive `experimentalDecorators` and
  `emitDecoratorMetadata` compilation.

## Dependency needs

None required. Decorator metadata emission is observable in output. A
metadata-reflection polyfill may be vendored, with pinned version, provenance,
and license, only if a planned runnable path requires executing reflection; none
is planned.

## Generated-source needs

None.

## Planned tests

`none`. Both decorator regimes are created by source and configuration; no
assigned coverage is created by test source.

## Required interfaces

- `Taskfile.yml` defines `build`, `lint`, and `test` as thin wrappers per ground
  rules; `build` compiles both mutually exclusive decorator regimes.
- `coverage.md` records the ground-rules coverage table for TS-CAN-054 and
  TS-CAN-055, using the compilation-context column to distinguish the two
  regimes and additional locators for distinct valid variants.
