---
relationships:
  references:
    - ground-rules
    - reports/typescript/synthesis/checklist
---

# Project brief: program-solution-workspace

## Purpose

A multi-package composite solution that exercises program construction and the
build graph: how roots are discovered, how configuration composes into effective
options, how composite project references order builds and expose declarations
across boundaries, how incremental and build-mode state scopes rebuilds, how
path and virtual-root mapping resolve specifiers, how generated source enters a
program, and how one source participates in more than one program.

## Exclusive directory

`fixtures/typescript/projects/program-solution-workspace/`

## Difficulty

`complex`

## Assigned coverage

Only valid-source obligations are corpus content. Where a canonical item names
failure, missing-input, cycle, stale-state, or diagnostic variants, those are
excluded per ground rules.

- TS-CAN-002 — roots selected by `files`, `include`, command-line arguments, and
  defaults; an `exclude`d file re-entering through an import or reference;
  automatic declaration inclusion; a genuinely unreachable file; `allowJs`
  inclusion.
- TS-CAN-003 — relative, package-based, chained, and array `extends`; option
  replacement rather than deep merge; command-line overrides; `${configDir}`
  substitution; a portable shared package config.
- TS-CAN-004 — at least two `composite` projects; a solution config with
  `files: []`; a reference and build-order edge; public declaration consumption
  across the boundary; hidden implementation symbols; declaration-map and source
  redirection; `disableSourceOfProjectReferenceRedirect`.
- TS-CAN-005 — an implementation-only edit, a public declaration edit, and error
  removal observed through `incremental`/`.tsbuildinfo` and `tsc -b` with
  different rebuild scopes; `assumeChangesOnlyAffectDirectDependencies`.
- TS-CAN-011 — a path alias with ordered fallback targets; a `rootDirs` virtual
  merge across authored and generated roots; a platform suffix selecting
  different files for one specifier; wildcard specificity; a relative cross-root
  import; `.ios`/`.native`/empty suffix order; `paths` without `baseUrl`.
- TS-CAN-016 — generated `.ts` and/or `.d.ts` entering through `include`,
  import, or `rootDirs`; changed generated types after input changes; source and
  map paths distinguishing authored, generated, and emitted identities;
  inline and external maps. Conditional; supported by the generated-source need
  below.
- TS-CAN-017 — one source placed in two configured programs with different
  `types`, `lib`, or strictness, with batch results recorded per program;
  referenced-source redirection. Editor and language-service variants are
  excepted (batch `tsc` corpus, no `tsserver` host).

## Declared compilation contexts

- Default context.
- Solution build mode via `tsc -b` with incremental `.tsbuildinfo` state.
- A `moduleSuffixes` platform-suffix resolution context.
- A `rootDirs` virtual-merge context spanning authored and generated roots.

## Dependency needs

None. Local packages within the project provide every reference target.

## Generated-source needs

Yes. Generated `.ts` and/or `.d.ts` outputs are committed alongside their
generator source and are produced by a dedicated `generate` task. Required
`build`, `lint`, and `test` tasks never regenerate.

## Planned tests

`none`. No assigned coverage is created by test source; every obligation is
expressed in ordinary program source and configuration.

## Required interfaces

- `Taskfile.yml` defines `build`, `lint`, `test`, and a `generate` task, each a
  thin wrapper per ground rules; `build` uses `tsc -b` across declared contexts.
- `coverage.md` records the ground-rules coverage table for every assigned
  identifier: canonical identifier, stable source path, the named declaration or
  configuration that creates coverage, the compilation context when non-default,
  and additional locators for distinct valid variants.
