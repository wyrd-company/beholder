---
relationships:
  references:
    - ground-rules
    - reports/typescript/synthesis/checklist
---

# Project brief: module-resolution-and-formats

## Purpose

A library that resolves and emits one dependency structure across the module and
resolution matrix, uses the full TypeScript and JavaScript extension family with
per-file module formats, and demonstrates how file identity, casing, real paths,
and symlinks unify or split declaration identity.

## Exclusive directory

`fixtures/typescript/projects/module-resolution-and-formats/`

## Difficulty

`complex`

## Assigned coverage

Only valid-source obligations are corpus content. Incompatible option pairs,
`require()`-of-ESM rejection, omitted-extension failures, JSX-outside-TSX, and
case-sensitive rejection are excluded per ground rules; the accepted forms are
kept.

- TS-CAN-009 — the same module dependency structure resolved and emitted under
  meaningful `module` and `moduleResolution` combinations: legacy
  `classic`/`node10`, `node16`/`nodenext`, `bundler`, CommonJS, preserved
  ECMAScript modules, and conditional legacy targets; extensionless relative and
  directory imports, explicit ESM extensions, top-level `await`, `import.meta`,
  `module: preserve`, `exports`/`imports` honoring, and emitted text that stays
  equal while resolution changes.
- TS-CAN-010 — `.ts`, `.tsx`, `.mts`, `.cts`, `.d.ts`, `.d.mts`, and `.d.cts`
  with JavaScript counterparts where enabled; `.js`-suffixed source imports
  resolving to `.ts`; forced formats; nearest-package `type` lookup;
  `.mjs`/`.mts` and `.cjs`/`.cts`; `allowImportingTsExtensions` in a permitted
  no-emit mode; `rewriteRelativeImportExtensions`.
- TS-CAN-012 — a workspace or package symlink; duplicate logical access paths
  that unify through one real path or split into distinct declaration identities;
  `preserveSymlinks`; two versions of a nominally sensitive type; and path
  spellings resolving to one physical file through symlinks. The
  case-insensitive-filesystem variants (case-only import differences that
  resolve, case-only rename unification, and `forceConsistentCasingInFileNames`
  consistency enforcement that depends on case-insensitive resolution) are
  excepted; see below.

## Declared compilation contexts

- Multiple module and resolution modes as listed for TS-CAN-009, including a
  `bundler` context.
- Per-file module-format contexts across the extension family.
- A file-identity context exercising a committed symlink and real-path layout on
  the default case-sensitive 64-bit x86 Linux host.

## Variant exceptions

- The case-insensitive-filesystem variants of TS-CAN-012 (case-only import
  differences that resolve, case-only rename unification, and
  `forceConsistentCasingInFileNames` consistency enforcement that depends on
  case-insensitive resolution) are unsupported conditional contexts. The corpus
  default host is case-sensitive 64-bit x86 Linux, and no case-insensitive
  filesystem or compiler host is locally available without privileged setup that
  `build`, `lint`, and `test` cannot perform. TS-CAN-012 stays in scope through
  its symlink and real-path identity obligations, which the default host
  represents.

## Dependency needs

None. Local packages and a committed symlink layout represent every resolution
target and duplicate-identity case; no package-manager tool is required to
produce the symlink structure.

## Generated-source needs

None.

## Planned tests

`none`. Resolution and format outcomes are created by source and configuration;
no assigned coverage is created by test source.

## Required interfaces

- `Taskfile.yml` defines `build`, `lint`, and `test` as thin wrappers per ground
  rules; `build` compiles under every declared module, resolution, and format
  context.
- `coverage.md` records the ground-rules coverage table for every assigned
  identifier, using the compilation-context column for each resolution or format
  mode and additional locators for distinct valid variants.
