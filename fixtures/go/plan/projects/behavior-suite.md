---
relationships:
  references:
    - ground-rules
    - reports/go/synthesis/checklist
    - fixtures/go/plan/overview
---

# Project brief: `behavior-suite`

## Purpose

A small, generic library whose fixture value lies in its test, example, and fuzz
source and its `testdata`. It is the primary owner of test-package structure, test
binary synthesis and discovery, subtests and parallelism, executable examples,
fuzz targets and seed corpora, package-relative `testdata`, vet analysis during
test and standalone analysis, and package-graph validity across build and test
configurations.

## Exclusive directory

`fixtures/go/projects/behavior-suite/`

## Difficulty

complex

## Assigned canonical identifiers and distinct valid variants

- **GO-CAN-TST-001** — same-package and external test packages: `p` tests and
  `p_test` tests that import `p`, with unexported access only in same-package
  tests and extra test-only imports.
- **GO-CAN-TST-002** — test binary synthesis and discovery: validly shaped `Test`,
  `Benchmark`, `Fuzz`, `Example`, and `TestMain` declarations whose names and
  signatures control discovery, with a synthesized main and package
  initialization.
- **GO-CAN-TST-003** — subtests, sub-benchmarks, and parallel execution: nested
  `Run`, dynamic names, cleanup, and `Parallel`, with parent and child timing and
  cleanup order.
- **GO-CAN-TST-004** — executable examples bound by naming convention to package,
  type, function, and method, with ordered and unordered output and a no-output
  documentation example, including the external test-package form.
- **GO-CAN-TST-005** — fuzz targets and seed corpus: `F.Fuzz`, `F.Add`, and
  checked-in `testdata/fuzz/<Name>` seeds with supported argument types, in
  ordinary test mode, demonstrating that a pre-1.18 module `go` directive still
  compiles and fuzzes under the installed command.
- **GO-CAN-TST-006** — `testdata` and package-relative data: non-Go and
  intentionally invalid Go files read from `testdata`, proving recursive package
  walks ignore the directory while runtime tests access it.
- **GO-CAN-TST-007** — vet during test and standalone analysis: legal, compiling
  source that triggers representative `printf`, `structtag`, `copylocks`,
  `buildtag`, `lost-cancel`, `tests`, and version-sensitive `loopclosure`
  analyzer checks, with quiet near-neighbors. Distinct valid variants: the curated
  `go test` vet subset versus standalone analyzer output. The source is legal and
  compiles; its analyzer findings are surfaced through standalone analysis, so the
  required `build`, `lint`, and `test` gates stay green.
- **GO-CAN-MOD-011** — import cycles and package-graph validity: valid recursive
  calls and types, and an external test package that avoids importing itself
  through the package-under-test variant, including a would-be cycle present only
  under a test configuration. Quarantined direct and transitive cycles are
  excluded as invalid source.

## Declared build contexts

- Default context.
- The Go test build for the test, example, and fuzz source, and fuzz mode for the
  GO-CAN-TST-005 seed corpus.
- A module `go` directive below go1.18 for the GO-CAN-TST-005 toolchain-versus-
  language-version independence demonstration, built under the installed command.
- A language version below go1.22, selected by a per-file or per-module `go`
  directive, for the version-sensitive `loopclosure` check of GO-CAN-TST-007.
- The installed `go vet` for GO-CAN-TST-007 standalone analyzer output and the
  curated `go test` vet subset.

## Dependency needs

Standard library and project-local packages only. No third-party dependency.

## Generated-source needs

None. Fuzz seed corpora under `testdata/fuzz` are hand-authored fixture data, not
generator output.

## Planned tests

Test source is fixture content here, and each planned test names the coverage its
source creates:

- Same-package and external test files create GO-CAN-TST-001 coverage.
- `Test`, `Benchmark`, `Fuzz`, `Example`, and `TestMain` declarations create
  GO-CAN-TST-002 coverage.
- Nested `Run`, cleanup, and `Parallel` test source creates GO-CAN-TST-003
  coverage.
- Example functions with ordered, unordered, and no-output forms create
  GO-CAN-TST-004 coverage.
- A fuzz target with `F.Add` and a checked-in seed corpus creates GO-CAN-TST-005
  coverage.
- Test source reading `testdata` creates GO-CAN-TST-006 coverage.
- An external test package that breaks a would-be test-configuration cycle creates
  the valid part of GO-CAN-MOD-011 coverage.

GO-CAN-TST-007 coverage is created by legal analyzer-target source and its quiet
near-neighbors, surfaced through standalone `go vet` analysis and the curated
`go test` vet subset rather than by executed tests; it therefore adds no passing
test to the required `test` gate.

These tests exist to create checklist coverage, not to prove the represented
library's domain correctness.

## Required `Taskfile.yml` and `coverage.md` interfaces

- `Taskfile.yml` defines `build`, `lint`, and `test` as thin wrappers over
  ordinary Go commands. `build` compiles all fixture-owned packages, including the
  sub-go1.18 module, in the declared contexts. `lint` checks `gofmt` formatting
  and runs `go vet` on fixture-owned packages. `test` runs the Go tests so the
  included test, example, and fuzz source compiles and executes; the intentionally
  invalid Go files under `testdata` are not compiled by the package build or test
  build. All three pass and write no logs, evidence, manifests, or hashes.
- `coverage.md` is a Markdown table recording, per assigned identifier, the
  canonical identifier, a stable source path, the named declaration, directive,
  package, module, or test that creates coverage, the build context when non-
  default, and additional locators for distinct valid variants. It uses stable
  names, not line numbers.
