---
relationships:
  references:
    - ground-rules
    - reports/typescript/synthesis/checklist
---

# TypeScript fixture planner prompt

Create project-level implementation plan for controlled TypeScript fixture
corpus.

## Inputs

Read these files completely:

- `fixtures/typescript/ground-rules.md`
- `reports/typescript/synthesis/checklist.md`

The accepted Go planner prompt and Go ground rules are available only as process
templates. Do not copy their scope, project shape, or coverage into TypeScript.

These are complete context. Do not read other repository files, research
reports, Go plans, Go fixture source, Go reviews, another language suite,
memories, prior work, or conversation history. Do not browse or consult other
agents. You may use shell, read, and write tools within this worktree. You may
inspect the installed TypeScript toolchain, including registry metadata needed
to select an installable pinned compiler version, to determine feasible
compilation contexts.

## Output

Write:

- `fixtures/typescript/plan/overview.md`
- One file per project under
  `fixtures/typescript/plan/projects/<project-slug>.md`

Use lower-kebab-case project slugs. Each project directory is
`fixtures/typescript/projects/<project-slug>/`.

Do not create fixture source.

## Scope accounting

Treat canonical checklist as language research rather than implementation shape.
Plan coverage for:

- Every required item with a valid-source demonstration.
- Every distinct valid variant named by an item.
- Every conditional item supported by a declared compilation context.

Do not plan intentionally invalid source or compiler-error checks. Do not
resolve research gaps. In overview, list only exclusions:

- Invalid-only items or variants.
- Unsupported conditional contexts.
- Research gaps.

Every other canonical identifier must have exactly one primary project. Natural
overlap is allowed and does not need accounting.

Declare each locally available compilation context needed to represent a valid
conditional item under the selected compiler and host. Do not add historical
toolchains, hosts, or contexts that are not locally installed or that require
external services.

## Planning boundary

Plan states what each project covers and constraints it operates within. It
never designs source that creates coverage.

Beyond assigned project directory, required `Taskfile.yml`, and required
`coverage.md`, do not include:

- Fixture source file or nested directory names.
- Fixture-owned package, module, namespace, type, interface, class, function,
  method, field, variable, constant, enum, or test names.
- Declarations, signatures, code snippets, or pseudocode.
- Domain entities, example scenarios, or expected application behavior.
- Implementation cases, subcases, call sequences, or step-by-step instructions.
- Validator internals or commands beyond required Task interface and declared
  compilation contexts.

Project briefs name canonical items and valid variants as coverage assignments.
They do not translate those assignments into source designs. Implementor owns
every source choice not fixed by ground rules. A third-party package named under
dependency needs is not a fixture-owned source-design choice. Stating a
structural necessity that a canonical item imposes, such as multiple composite
sub-projects or contrasting effective-option sets, is a project constraint
rather than source design, provided no file, directory, or declaration is named.

## Project design

Choose number, purpose, and theme of projects from natural TypeScript
boundaries. Do not divide checklist into equal taxonomic packets. Each top-level
project is coherent, idiomatic, self-contained, and independent. It may contain
several local packages, but it cannot depend on another fixture project.

Assign whole projects as `routine` or `complex`. Do not split one project
between implementors.

Tests are planned only when test source creates assigned checklist coverage. For
each planned test, name that coverage. Otherwise record `none`. Do not plan
tests for a represented application's domain correctness.

A real third-party dependency belongs only when it creates distinct assigned
coverage that local packages and the pinned toolchain cannot represent as well.
Name dependency and purpose. Implementor will pin, fetch, vendor, and preserve
licensing.

When generated source creates assigned coverage, name generator relationship and
required committed outputs. Required validation tasks never generate files.

## Overview contents

Overview records:

- Exact installed TypeScript compiler, Node.js host, and package manager
  selected for corpus.
- Default and additional available compilation contexts.
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
- Declared compilation contexts.
- Dependency needs.
- Generated-source needs.
- Planned tests and coverage created by their source, or `none`.
- Required `Taskfile.yml` and `coverage.md` interfaces from ground rules.

Do not add logs, manifests, hashes, mutations, expected failures, graph output,
or rubric assertions.

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
