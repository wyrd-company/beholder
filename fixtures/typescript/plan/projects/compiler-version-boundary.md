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
  least one unchanged source case run across the documented `typescript` 5.4.5
  to 5.9.3 version boundary that changes an inferred type or emit while remaining
  valid under both releases; the workspace compiler (5.9.3) versus the earlier
  bundled compiler (5.4.5); a dependency minimum compiler version; and changed
  library declarations across the boundary.

The boundary is the TypeScript 5.5 inferred-type-predicate inference change
(checklist version-transition audit, 5.5 row): the same valid source emits a
different inferred declaration under 5.4.5 than under 5.9.3, verified locally by
declaration emit under both vendored compilers. The implementor selects the
specific construct within that documented boundary; the plan names the boundary,
not the source.

## Declared compilation contexts

- Default context using the pinned corpus compiler, `typescript` 5.9.3.
- A second context using the earlier pinned `typescript` 5.4.5 release, vendored
  offline alongside the default compiler. Both releases are locally installable
  (registry-verified) and the boundary is empirically observable between them.

## Dependency needs

Yes. The exact earlier release `typescript` 5.4.5 is vendored with its pinned
version, source location, license and notice files, and package metadata for
offline resolution, alongside the corpus-default `typescript` 5.9.3. The
corpus-default compiler remains the toolchain rather than a project dependency;
`build`, `lint`, and `test` perform no downloads.

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
