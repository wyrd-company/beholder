---
relationships:
  references:
    - ground-rules
    - reports/typescript/synthesis/checklist
---

# Project brief: compiler-version-boundary

## Purpose

A library whose program identity is observed across a documented compiler-version
boundary, so that one unchanged source case produces a different inferred type,
resolution result, or emit under two vendored compiler releases. This is the one
project whose obligation cannot be met with the single pinned corpus compiler and
so vendors an additional release.

## Exclusive directory

`fixtures/typescript/projects/compiler-version-boundary/`

## Difficulty

`complex`

## Assigned coverage

Only valid-source obligations are corpus content. The unknown-newer-syntax-under-
older-compiler failure variant is excluded per ground rules.

- TS-CAN-001 — exact compiler version, bundled `lib.*.d.ts` set, effective
  configuration, root program, and host treated as program identity, with at
  least one unchanged source case run across a documented version boundary that
  changes an inferred type, resolution result, or emit; workspace compiler versus
  a second bundled compiler; a dependency minimum compiler version; and changed
  iterator or library declarations across the boundary.

## Declared compilation contexts

- Default context using the pinned corpus compiler.
- A second context using one earlier pinned `typescript` release within the
  4.7–5.8 transition range, vendored offline alongside the default compiler.

## Dependency needs

Yes. One additional pinned `typescript` compiler release (earlier than the corpus
default) is vendored with its pinned version, source location, license and notice
files, and package metadata for offline resolution. The corpus-default compiler
remains the toolchain rather than a project dependency.

## Generated-source needs

None.

## Planned tests

`none`. The boundary is observed by compiling the same source under two vendored
compilers; no assigned coverage is created by test source.

## Required interfaces

- `Taskfile.yml` defines `build`, `lint`, and `test` as thin wrappers per ground
  rules; `build` compiles the boundary source under both declared compiler
  contexts.
- `coverage.md` records the ground-rules coverage table for TS-CAN-001, using the
  compilation-context column to distinguish the two compiler releases and
  additional locators for the distinct boundary variants.
