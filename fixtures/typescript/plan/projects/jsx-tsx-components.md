---
relationships:
  references:
    - ground-rules
    - reports/typescript/synthesis/checklist
---

# Project brief: jsx-tsx-components

## Purpose

A component library that covers TSX parsing and the JSX transform modes, and the
typing of JSX names, components, attributes, and alternative runtime surfaces.

## Exclusive directory

`fixtures/typescript/projects/jsx-tsx-components/`

## Difficulty

`routine`

## Assigned coverage

Only valid-source obligations are corpus content. Angle-bracket assertion in TSX,
invalid component tags, and conflicting global JSX declarations are excluded per
ground rules.

- TS-CAN-056 — paired `.ts` and `.tsx` token streams showing angle-bracket
  assertion versus generic-arrow disambiguation (`<T,>`); JSX compiled under
  `preserve`, classic, automatic, development, and conditional React-Native
  modes; injected runtime imports; and per-file pragmas (`jsxFactory`,
  `jsxFragmentFactory`, `jsxImportSource`, `@jsx`, `@jsxFrag`, `@jsxRuntime`).
- TS-CAN-057 — intrinsic lowercase and component uppercase tags; required and
  optional props; children; generic components with inferred and explicit
  arguments; custom intrinsic augmentation; namespace and namespaced attributes;
  two JSX runtimes with different typing surfaces; `JSX.IntrinsicElements` and
  `JSX.ElementType`; the classic global versus automatic module-scoped JSX
  namespace.

## Declared compilation contexts

- JSX transform modes: `preserve`, classic, automatic, and development.
- A TSX parsing context contrasted with a `.ts` context.
- Two JSX runtime typing-surface contexts.

## Dependency needs

None required. Local JSX runtime declarations represent both runtime typing
surfaces. A framework's JSX types may be vendored, with pinned version,
provenance, and license, only if they create distinct coverage that local
declarations cannot; none is required.

## Generated-source needs

None.

## Planned tests

`none`. JSX parsing and typing are created by source and configuration; no
assigned coverage is created by test source.

## Required interfaces

- `Taskfile.yml` defines `build`, `lint`, and `test` as thin wrappers per ground
  rules; `build` compiles under each declared JSX mode and runtime surface.
- `coverage.md` records the ground-rules coverage table for TS-CAN-056 and
  TS-CAN-057, using the compilation-context column for each JSX mode and
  additional locators for distinct valid variants.
