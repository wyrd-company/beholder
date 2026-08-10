---
relationships:
  references:
    - ground-rules
    - reports/typescript/synthesis/checklist
---

# Project brief: alternate-tools-and-hosts

## Purpose

A library that shows how non-`tsc` tools and non-Node hosts treat TypeScript: an
alternate transpiler and a compiler-API transformer whose transform of one
construct differs while a separate TypeScript check is retained, and
runtime-native TypeScript execution on locally installed hosts.

## Exclusive directory

`fixtures/typescript/projects/alternate-tools-and-hosts/`

## Difficulty

`complex`

## Assigned coverage

Only valid-source obligations are corpus content. Failure paths and rejections
particular to each tool or host are excluded per ground rules.

- TS-CAN-090 — a non-`tsc` transform path with its own configuration and a
  retained separate TypeScript check, and one construct whose transform differs
  (enum, namespace, const enum, decorator, class field, JSX, import elision, or
  isolated declaration); a per-file transform through the locally installed Bun
  transpiler; a before, after, or declaration transformer through the pinned
  compiler API; a synthetic node without a source position; a generated runtime
  edge absent from source.
- TS-CAN-091 — Node.js type-stripping and execution contrasting erasable syntax
  with enum, namespace, parameter-property, and import-equals syntax;
  `erasableSyntaxOnly`; real relative extensions; no `paths`; Bun's distinct
  specifier and resolution model and partial tsconfig support; execution without
  type checking. The Deno resolution variant is excepted (Deno is not
  installed).

## Declared compilation contexts

- A retained `tsc` type-check context.
- A non-`tsc` transform context using the Bun transpiler and the pinned
  compiler-API transformer surface.
- Runtime-native host contexts: Node.js type stripping and Bun.

## Dependency needs

None required. Bun and Node.js are locally installed hosts and tools, and the
compiler API is the pinned `typescript` package.

## Generated-source needs

None.

## Planned tests

`none`. Transform and host behavior is created by source, configuration, and the
locally installed hosts; no assigned coverage is created by test source.

## Required interfaces

- `Taskfile.yml` defines `build`, `lint`, and `test` as thin wrappers per ground
  rules; `build` retains the `tsc` check, and the host and transform contexts are
  exercised through the standard tasks without downloads.
- `coverage.md` records the ground-rules coverage table for TS-CAN-090 and
  TS-CAN-091, using the compilation-context column for each tool or host and
  additional locators for distinct valid variants.
