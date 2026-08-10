---
relationships:
  references:
    - ground-rules
    - reports/typescript/synthesis/checklist
---

# Project brief: esm-cjs-interop

## Purpose

A library that models the boundary between ECMAScript modules and CommonJS: the
TypeScript-specific CommonJS modeling forms, interoperable consumption under
contrasting interop settings, how module detection classifies a source, live
bindings and evaluation cycles, and Universal Module Definition duality.

## Exclusive directory

`fixtures/typescript/projects/esm-cjs-interop/`

## Difficulty

`routine`

## Assigned coverage

Only valid-source obligations are corpus content. Illegal export mixing,
inaccurate-declaration and default-shape failures, global collisions,
`isolatedModules` rejection, temporal-dead-zone failure, and the
global-from-module diagnostic are excluded per ground rules.

- TS-CAN-024 — a single CommonJS export with `export =`, consumed through
  `import x = require()`, with properties merged onto the exported class or
  function through a namespace; `.cts` and `.d.cts`; `export as namespace`;
  JavaScript `module.exports`; a synthetic-default consumer.
- TS-CAN-025 — callable and object-shaped CommonJS dependencies consumed with
  namespace, default, and import-equals forms under contrasting `esModuleInterop`
  and `allowSyntheticDefaultImports`; `__esModule`; inherited versus own
  properties; Node native ESM behavior; helper import.
- TS-CAN-026 — two global scripts sharing declarations, then `export {}` making
  the names private; the same candidates under `legacy`, `auto`, and `force`
  module detection; a `.d.ts` global-to-module flip; JSX automatic-runtime
  detection; nearest package `type`; CommonJS indicators in checked JavaScript.
- TS-CAN-030 — one Universal Module Definition package with `export =` plus
  `export as namespace`, consumed as a module and as a script global;
  `allowUmdGlobalAccess`. Conditional (ecosystem convention).
- TS-CAN-032 — an ECMAScript-module cycle with mutable exports and top-level
  reads; a CommonJS cycle; a type-only cycle with no runtime evaluation edge; a
  hoisting-safe function cycle; a barrel-mediated cycle; a top-level-await cycle.

## Declared compilation contexts

- Contrasting `esModuleInterop` and `allowSyntheticDefaultImports` settings.
- `legacy`, `auto`, and `force` module-detection settings.
- ECMAScript-module and CommonJS module modes.
- A Universal Module Definition declaration context.

## Dependency needs

None.

## Generated-source needs

None.

## Planned tests

`none`. Interop, detection, and cycle behavior are created by source and
configuration; no assigned coverage is created by test source.

## Required interfaces

- `Taskfile.yml` defines `build`, `lint`, and `test` as thin wrappers per ground
  rules; `build` compiles under each declared interop and detection context.
- `coverage.md` records the ground-rules coverage table for every assigned
  identifier, using the compilation-context column for each interop or detection
  mode and additional locators for distinct valid variants.
