---
relationships:
  references:
    - ground-rules
    - reports/go/synthesis/checklist
    - fixtures/go/plan/overview
---

# Project brief: `module-federation`

## Purpose

A self-contained multi-module workspace. It is the primary owner of module and
package identity, version directives, minimal version selection, semantic import
versioning, `replace`/`exclude`/`retract`, workspaces, nested-module boundaries,
`internal` visibility, vendoring, hermetic checksums, tracked tool dependencies,
legacy import behavior, and repository-layout semantics.

## Exclusive directory

`fixtures/go/projects/module-federation/`

## Difficulty

complex

## Assigned canonical identifiers and distinct valid variants

- **GO-CAN-MOD-001** — module and package identity: `go.mod` with `module`, `go`,
  and direct `require`, and packages whose import paths derive from the module
  path.
- **GO-CAN-MOD-002** — `go` and `toolchain` directives with `GOTOOLCHAIN=local`
  selection and a file requiring a newer language version within the installed
  release. The automatic-download variant is excluded (see overview exceptions).
- **GO-CAN-MOD-003** — minimal version selection, graph pruning, and lazy loading:
  direct and indirect requirements that select a higher transitive version and a
  test-only dependency, with `// indirect` metadata distinguished from source
  import edges.
- **GO-CAN-MOD-004** — semantic import versioning: coexisting v1 and `/v2` module
  identities with same-named packages aliased in one importer.
- **GO-CAN-MOD-005** — `replace`, `exclude`, and `retract`: a dependency
  redirected to a local module, an excluded version, and retraction metadata
  exposed from a dependency, with main-module-only replacement effect.
- **GO-CAN-MOD-006** — workspaces: `go.work` with two modules importing each other
  and a workspace-level replacement, compared with `GOWORK=off` and workspace
  vendoring.
- **GO-CAN-MOD-007** — nested modules and boundaries: a second `go.mod` under a
  parent module whose subtree is excluded until required, replaced, or added
  to a workspace, compared with an ordinary subdirectory.
- **GO-CAN-MOD-008** — `internal` import visibility: a nested internal package
  imported legally from its parent tree, with nested `internal` elements proving
  path visibility is orthogonal to exportedness.
- **GO-CAN-MOD-009** — vendoring: `vendor/modules.txt` and dependency source
  selected under vendor mode, comparing `-mod=vendor` and `-mod=mod`.
- **GO-CAN-MOD-010** — checksums and canonical source: `go.sum` with hermetic
  local or vendor resolution. External proxy, sumdb, and network variants are
  excluded (see overview exceptions).
- **GO-CAN-MOD-012** — legacy imports: a preserved, inert canonical import-path
  comment that module mode ignores. Relative-import behavior and a GOPATH baseline
  are excluded (see overview exceptions).
- **GO-CAN-MOD-013** — tracked tool dependencies: a go1.24 `tool` directive tied
  to `go tool` with a generator or analyzer dependency, contrasted with a
  pre-1.24 build-tagged blank-import pattern, as dependency edges without runtime
  library imports.
- **GO-CAN-ECO-001** — repository-layout semantics versus convention: multiple
  command packages and tool-significant `internal`, `vendor`, `testdata`, and
  nested modules, treating `pkg` and `src` as ordinary names.
- **GO-CAN-ECO-002** — standard-library versioned package coexistence: two
  independently named or versioned standard packages imported together with
  aliases where names collide, without generalizing to module semantic import
  versioning.

## Declared build contexts

- Default context.
- Workspace on and off (`GOWORK`) for GO-CAN-MOD-006.
- Vendor mode (`-mod=vendor`) and module mode (`-mod=mod`) for GO-CAN-MOD-009.
- `GOTOOLCHAIN=local` for GO-CAN-MOD-002.
- A hermetic local module proxy or prepopulated module cache for the version-
  resolved requirements of GO-CAN-MOD-003, GO-CAN-MOD-004, GO-CAN-MOD-005, and the
  `go.sum` entries of GO-CAN-MOD-010. No network access is required.

## Dependency needs

Standard library and project-local modules only. No third-party dependency; the
required, replaced, excluded, retracted, and versioned modules are all local
modules within this project's directory, resolved hermetically. The `tool`
directive references a local tool module.

## Generated-source needs

None. The `tool` directive names a dependency edge; this project does not commit
generator output.

## Planned tests

- One test creates the GO-CAN-MOD-003 coverage for a test-only dependency edge,
  demonstrating that a requirement is reached only from test source.

Other coverage is created by module metadata, workspace files, vendor directories,
and package structure rather than by test source.

## Required `Taskfile.yml` and `coverage.md` interfaces

- `Taskfile.yml` defines `build`, `lint`, and `test` as thin wrappers over
  ordinary Go commands, exercising the workspace, vendor, and local-resolution
  contexts without downloading anything. `build` compiles all fixture-owned
  packages across the declared modules and contexts. `lint` checks `gofmt`
  formatting and runs `go vet` on fixture-owned packages; vendored source is
  excluded. `test` runs Go tests, including the test-only dependency edge, and
  succeeds with no test files elsewhere. All three pass, perform no downloads, and
  write no logs, evidence, manifests, or hashes.
- `coverage.md` is a Markdown table recording, per assigned identifier, the
  canonical identifier, a stable source path, the named declaration, directive,
  package, module, or test that creates coverage, the build context when non-
  default, and additional locators for distinct valid variants. It uses stable
  names, not line numbers.
