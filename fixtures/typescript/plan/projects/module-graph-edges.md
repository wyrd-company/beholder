---
relationships:
  references:
    - ground-rules
    - reports/typescript/synthesis/checklist
---

# Project brief: module-graph-edges

## Purpose

A library whose surface is the module dependency graph itself: the binding forms
that connect modules, type-only edges and elision, re-export forwarding through
barrels, side-effect imports, dynamic import and import types, import attributes
and JSON, and deferred module evaluation.

## Exclusive directory

`fixtures/typescript/projects/module-graph-edges/`

## Difficulty

`complex`

## Assigned coverage

Only valid-source obligations are corpus content. Missing exports, duplicate
local names, illegal combined `import type`, cyclic-ambiguity failures,
misspelled bare imports, missing loaders, invalid module or target combinations,
named-import failures, and rejection under unsupported module modes are excluded
per ground rules.

- TS-CAN-018 — named, renamed, default, namespace, and side-effect imports;
  named, default, star, and namespace re-exports; alias chains; an anonymous
  default class declaration and an anonymous default function declaration as two
  distinct forms; a type-only default interface; a default expression;
  `export { default as X }`; type and value meanings reached through one alias.
- TS-CAN-019 — `import type`, default and inline type modifiers, `export type`,
  type-only star and namespace re-exports, and the same unmarked import under
  legacy elision and under `verbatimModuleSyntax` with retained imports.
- TS-CAN-020 — a barrel with direct, renamed, namespace, type-only, and star
  re-exports; two star sources exposing the same name resolved by an explicit
  local export; explicit local export precedence; a cyclic barrel; an identical
  origin reached twice through two star re-exports; a default not forwarded by a
  star re-export; a CommonJS re-export source; and a re-export without a local
  binding.
- TS-CAN-021 — a resolvable bare import; a non-code side-effect import admitted
  by a wildcard declaration; default checking versus
  `noUncheckedSideEffectImports` with an asset loader present; package
  `sideEffects` metadata; a runtime-patch-installation import.
- TS-CAN-022 — runtime `import()`; a computed dynamic specifier; `import("m").T`;
  `typeof import("m")`; `import.meta` with an augmented `ImportMeta`; top-level
  `await` with a dependent importer; import-type `resolution-mode`.
- TS-CAN-023 — JSON import with a typed valid property and a `with { type:
  "json" }` attribute; a legacy-assertion contrast; a custom extension typed
  through wildcard and per-file declarations; an imported and exported
  string-named binding (arbitrary module-namespace names); `allowArbitraryExtensions`;
  `resolveJsonModule`.
- TS-CAN-033 — `import defer * as ns from "m"` contrasted with an ordinary
  namespace import of a side-effectful module so that the different evaluation
  point is observable; an unused deferred binding; first member access.
  Conditional under `typescript` 5.9.3 with `module: esnext` or `module:
  preserve`, which are the only module modes that accept deferred imports on the
  pinned compiler; every other mode raises TS18060. This was verified locally by
  compiling `import defer` under each mode. The exact runtime and cycle
  constraints tracked by TS-GAP-001 remain unresolved and are not corpus content.

## Declared compilation contexts

- Default context.
- A `verbatimModuleSyntax` context.
- An import-attributes and JSON context.
- A `noUncheckedSideEffectImports` context.
- A `module: esnext` or `module: preserve` context under `typescript` 5.9.3 for
  deferred import evaluation; these are the only modes that accept `import defer`
  on the pinned compiler (verified locally).

## Dependency needs

None. Local modules provide every import target, including the wildcard-declared
asset and the deferred side-effectful module.

## Generated-source needs

None.

## Planned tests

`none`. Every module edge is created by source and configuration; no assigned
coverage is created by test source.

## Required interfaces

- `Taskfile.yml` defines `build`, `lint`, and `test` as thin wrappers per ground
  rules; `build` compiles under each declared context.
- `coverage.md` records the ground-rules coverage table for every assigned
  identifier, using the compilation-context column where non-default and
  additional locators for distinct valid variants.
