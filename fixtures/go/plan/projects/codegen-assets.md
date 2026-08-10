---
relationships:
  references:
    - ground-rules
    - reports/go/synthesis/checklist
    - fixtures/go/plan/overview
---

# Project brief: `codegen-assets`

## Purpose

A documented library that generates part of its own source and embeds static
files. It is the primary owner of package and declaration documentation,
`go:generate`, generated-source markers and bindings, and embedded files.

## Exclusive directory

`fixtures/go/projects/codegen-assets/`

## Difficulty

routine

## Assigned canonical identifiers and distinct valid variants

- **GO-CAN-SRC-010** — package and declaration documentation: `doc.go`,
  declaration comments, doc links, headings and lists, and `Deprecated:` markers,
  including multiple package comments, build-excluded docs, and valid intra- and
  cross-package links, with structured doc links at go1.19 or higher.
- **GO-CAN-GENR-001** — `go:generate` directives with quoting, environment
  expansion, and more than one generator, with generated output and input
  provenance kept, the directive beginning at the start of a line in a Go file.
- **GO-CAN-GENR-002** — generated-source marker and bindings: checked-in generated
  Go plus its non-Go schema or input and the standard generated-code marker, with
  generator-only dependency edges, the generated Go otherwise following normal
  language rules.
- **GO-CAN-GENR-003** — embedded files: `//go:embed` on string, byte slice, and
  `embed.FS` with files and patterns, including hidden-file matching rules, the
  `all:` variant, and build-selected directive files, at go1.16 or higher.

## Declared build contexts

- Default context.
- A build tag that selects a build-conditioned `//go:embed` directive file for
  GO-CAN-GENR-003.

## Dependency needs

Standard library and project-local packages only. No third-party dependency. The
generator is a fixture-owned local package or command within this project.

## Generated-source needs

Required for GO-CAN-GENR-001 and GO-CAN-GENR-002. The project commits both the
generator source and the generated output, along with the non-Go schema or input
and the standard generated-code marker. A separate `generate` task drives
generator invocation because the `go:generate` directive is itself fixture
content; the required validation tasks never regenerate source.

## Planned tests

None. Documentation, generation, marker, and embedding coverage is created by
source, directives, committed generated output, and embedded files rather than by
test source.

## Required `Taskfile.yml` and `coverage.md` interfaces

- `Taskfile.yml` defines `build`, `lint`, and `test` as thin wrappers over
  ordinary Go commands, plus a separate non-validation `generate` task. `build`
  compiles all fixture-owned packages in the declared contexts, including the
  build-selected embed variant. `lint` checks `gofmt` formatting and runs `go vet`
  on fixture-owned packages; committed generated output remains subject to
  `gofmt` and `go vet`. `test` runs Go tests and succeeds with no test files. The
  three validation tasks pass, never regenerate source, and write no logs,
  evidence, manifests, or hashes.
- `coverage.md` is a Markdown table recording, per assigned identifier, the
  canonical identifier, a stable source path, the named declaration, directive,
  package, module, or test that creates coverage, the build context when non-
  default, and additional locators for distinct valid variants. It uses stable
  names, not line numbers.
