---
relationships:
  references:
    - ground-rules
    - reports/typescript/synthesis/checklist
---

# Project brief: namespaces-ambient-globals

## Purpose

A library built around TypeScript's non-module declaration surfaces: internal
namespaces and namespace aliases, ambient module declarations, module and global
augmentation, triple-slash directives with legacy ordered concatenation, and
ambient global value declarations.

## Exclusive directory

`fixtures/typescript/projects/namespaces-ambient-globals/`

## Difficulty

`routine`

## Assigned coverage

Only valid-source obligations are corpus content. Competing-declaration and
runtime-absence cases, wrong-package-copy and global-collision failures,
directive-after-statement and missing-path failures, and duplicate-global
failures are excluded per ground rules.

- TS-CAN-027 — a namespace split across declarations and files with exported and
  non-exported members, nested and dotted names, `import Alias = A.B`, and
  `export import` from a namespace; emitted value objects versus type-only
  erasure; ambient namespaces; legacy `module A {}`.
- TS-CAN-028 — exact-name, wildcard, and shorthand ambient modules, with matching
  and declared-but-unmatched imports; the one-wildcard pattern rule; default
  versus `export =` shape; shorthand `any`; and the different meaning when the
  declaration file becomes an external module.
- TS-CAN-029 — augmenting an existing exported interface with the corresponding
  runtime patch installed through a side-effect import; adding a global through
  `declare global`; behavior with the augmentation file included and excluded;
  augmentation in a `.ts` versus a `.d.ts` file.
- TS-CAN-031 — `reference path`, `types`, `lib`, and `no-default-lib` at valid
  file positions; a separate legacy configuration using `outFile` and references
  to make emit ordering observable; `preserve="true"`; generated declaration
  references; AMD and System legacy emit. Triple-slash processing is required;
  `outFile`, AMD, and ordered concatenation are conditional legacy coverage.
- TS-CAN-045 — external `var`/`let`/`const`/function/class/enum/namespace values
  with no emit; consuming a global `.d.ts`; flipping it to module scope with an
  import; extending global object, prototype, and environment interfaces from
  both global and module declarations; `declare global`.

## Declared compilation contexts

- Default context.
- A legacy `outFile` and AMD or System concatenation context for the conditional
  legacy portion of TS-CAN-031.

## Dependency needs

None.

## Generated-source needs

None.

## Planned tests

`none`. Namespace, ambient, and global behavior is created by declarations and
configuration; no assigned coverage is created by test source.

## Required interfaces

- `Taskfile.yml` defines `build`, `lint`, and `test` as thin wrappers per ground
  rules; `build` compiles the default and the legacy concatenation context.
- `coverage.md` records the ground-rules coverage table for every assigned
  identifier, using the compilation-context column for the legacy context and
  additional locators for distinct valid variants.
