---
relationships:
  references:
    - ground-rules
    - reports/typescript/synthesis/checklist
---

# Project brief: package-publishing-ecosystem

## Purpose

A multi-package ecosystem that models how TypeScript packages are published and
consumed: entry-point and condition maps, package `imports` and self-references,
declaration discovery and version routing, package-manager identity and workspace
consumption, runtime-and-type entry-point agreement, platform variants and
declaration rollups, declaration emit with maps, and dependency declaration
quality.

## Exclusive directory

`fixtures/typescript/projects/package-publishing-ecosystem/`

## Difficulty

`complex`

## Assigned coverage

Only valid-source obligations are corpus content. Misordered or misplaced
conditions, deep-import rejection, inaccessible internal targets, untyped imports,
missing peers, hidden transitive packages, load-time mismatches, stale generators,
and declaration contradictions that surface an error are excluded per ground
rules.

- TS-CAN-014 — consuming `.d.ts` independently from runtime code; emitting
  `.d.ts` and `.d.ts.map`; inferred public types; a private or unnameable leaked
  type; `@internal` stripping under `stripInternal`; navigation back to authored
  sources; `declaration`, `emitDeclarationOnly`, `.d.mts`, and `.d.cts`;
  declaration maps.
- TS-CAN-034 — public and blocked subpaths with `exports`; distinct `import` and
  `require` runtime entries; matching type branches; wildcard patterns; `types`,
  `types@` selectors, `typings`, and `main`; a legacy fallback without `exports`;
  `.d.mts` and `.d.cts` correspondence. The types-ordering fallback tracked by
  TS-GAP-002 is not resolved.
- TS-CAN-035 — a `#`-prefixed internal alias; a package self-name import; a
  dependency whose declaration surface changes under `customConditions`; import
  and require branches; nested maps; wildcard subpaths.
- TS-CAN-036 — packages typed through an `exports` `types` branch, `types` or
  `typings`, a conventional `index.d.ts`, and `@types`; restricting ambient
  inclusion with `types` and `typeRoots`; `typesVersions` changing an exported
  type by compiler range; a scoped `@types/scope__name`.
- TS-CAN-037 — direct, transitive, and peer type dependencies; two installed
  versions of a nominally sensitive declaration; a workspace link; source-alias
  versus built-declaration consumption; npm hoisting. The pnpm isolation and
  store-link variant and the Yarn node-modules and Plug'n'Play variant are
  excepted (pnpm, Yarn, and corepack are not installed).
- TS-CAN-038 — import and require consumers for a dual package; public subpaths;
  matching declaration flavors; a types-only export; a bundler condition.
- TS-CAN-039 — condition- or suffix-selected browser, server, and native
  sources; a generated typed package or module; rolled-up declarations whose
  public entity layout differs from source; development and production, browser
  and server, and React-Native suffixes. Conditional (ecosystem convention),
  supported by the generated-source need below.
- TS-CAN-089 — a dependency declaration with a consumer-visible exported type and
  a declaration-versus-runtime mismatch that remains invisible to checking;
  comparing `skipLibCheck` on and off; `skipDefaultLibCheck`; local versus
  dependency `.d.ts`. The internal-contradiction-surfacing-an-error variant is
  excluded as invalid source.

## Declared compilation contexts

- `node16`/`nodenext` and `bundler` resolution.
- A `customConditions` context.
- `declaration`, `declarationMap`, and `emitDeclarationOnly` emit.
- `skipLibCheck` on and off.
- Suffix- and condition-selected platform sources.

## Dependency needs

None required. Local packages represent every dependency, `@types` package, and
duplicate-version copy. A declaration-bundler may be vendored, with pinned
version, provenance, and license, only if the TS-CAN-039 rollup surface cannot be
produced by local generation.

## Generated-source needs

Yes. TS-CAN-039 generated typed package or module and rolled-up declaration
outputs are committed alongside their generator source and produced by a
dedicated `generate` task. Required `build`, `lint`, and `test` tasks never
regenerate.

## Planned tests

`none`. Package resolution, emit, and declaration behavior are created by source,
package metadata, and configuration; no assigned coverage is created by test
source.

## Required interfaces

- `Taskfile.yml` defines `build`, `lint`, `test`, and a `generate` task as thin
  wrappers per ground rules; `build` compiles under each declared resolution and
  emit context.
- `coverage.md` records the ground-rules coverage table for every assigned
  identifier, using the compilation-context column for each resolution or emit
  context and additional locators for distinct valid variants.
