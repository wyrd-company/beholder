---
relationships:
  references:
    - ground-rules
    - reports/go/synthesis/checklist
---

# Go fixture planner prompt

Create project-level implementation plan for controlled Go fixture corpus.

## Inputs

Read these files completely:

- `fixtures/go/ground-rules.md`
- `reports/go/synthesis/checklist.md`

These are complete context. Do not read other repository files, research reports,
memories, prior work, or conversation history. Do not browse or consult other
agents. You may use shell, read, and write tools within this worktree. You may
inspect installed Go and C tooling to determine feasible build contexts.

## Output

Write:

- `fixtures/go/plan/overview.md`
- One file per project under `fixtures/go/plan/projects/<project-slug>.md`

Use lower-kebab-case project slugs. Each project directory is
`fixtures/go/projects/<project-slug>/`.

Do not create fixture source.

## Scope accounting

Treat canonical checklist as language research rather than implementation shape.
Plan coverage for:

- Every required item with a valid-source demonstration.
- Every distinct valid variant named by an item.
- Every conditional item supported by a declared build context.

Do not plan intentionally invalid source or compiler-error checks. Do not resolve
research gaps. In overview, list only exclusions:

- Invalid-only items or variants.
- Unsupported conditional contexts.
- Research gaps.

Every other canonical identifier must have exactly one primary project. Natural
overlap is allowed and does not need accounting.

Declare each locally available build context needed to represent a valid
conditional item under selected compiler. Do not add historical toolchains or
contexts that require external services.

## Project design

Choose number, purpose, and theme of projects from natural Go boundaries. Do not
divide checklist into equal taxonomic packets. Each top-level project is coherent,
idiomatic, self-contained, and independent. It may contain several local Go
modules, but it cannot depend on another fixture project.

Assign whole projects as `routine` or `complex`. Do not split one project between
implementors.

Tests are planned only when test source creates assigned checklist coverage. For
each planned test, name that coverage. Otherwise record `none`. Do not plan tests
for represented application's domain correctness.

A real third-party dependency belongs only when it creates distinct assigned
coverage that local fixture modules cannot represent as well. Name dependency and
purpose. Implementor will pin, fetch, vendor, and preserve licensing.

When generated source creates assigned coverage, name generator relationship and
required committed outputs. Required validation tasks never generate files.

## Overview contents

Overview records:

- Exact installed Go compiler selected for corpus.
- Default and additional available build contexts.
- Project list with purpose, directory, and difficulty.
- Primary assignment of every in-scope canonical identifier.
- Exceptions-only list with short reasons.
- Independence audit confirming exclusive directories and no cross-project
  dependencies.

## Project brief contents

Each standalone brief contains only implementation context for one project:

- Purpose.
- Exclusive directory.
- Difficulty: `routine` or `complex`.
- Assigned canonical identifiers and distinct valid variants.
- Declared build contexts.
- Dependency needs.
- Generated-source needs.
- Planned tests and coverage created by their source, or `none`.
- Required `Taskfile.yml` and `coverage.md` interfaces from ground rules.

Do not prescribe source file names, declarations, example code, case tables,
validator internals, logs, manifests, hashes, mutations, expected failures, graph
output, or rubric assertions. The implementor owns source design.

## Validation

Before finishing:

- Confirm every in-scope canonical identifier has one primary project.
- Confirm every distinct valid variant is represented by an assignment.
- Confirm every exclusion belongs to one allowed exception category.
- Confirm project directories are exclusive and projects have no dependencies on
  one another.
- Confirm every planned test names checklist coverage created by test source.
- Run Markdown lint on files you wrote.

Stop after writing and validating plan artifacts.
