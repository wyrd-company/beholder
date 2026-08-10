---
relationships:
  references:
    - ground-rules
    - reports/dart/synthesis/checklist
---

# Dart fixture planner prompt

Create the project-level implementation plan for the controlled Dart fixture
corpus.

## Inputs

Read these files completely:

- `fixtures/dart/ground-rules.md`
- `reports/dart/synthesis/checklist.md`

These are complete context. Do not read other repository files, research
reports, memories, prior work, or conversation history. Do not browse or consult
other agents. You may use shell, read, and write tools within this worktree. You
may inspect the installed Dart toolchain to determine feasible build contexts,
including compilation targets, available platform libraries, and the SDK
experiment set.

## Output

Write:

- `fixtures/dart/plan/overview.md`
- One file per project under `fixtures/dart/plan/projects/<project-slug>.md`

Use lower-kebab-case project slugs. Each project directory is
`fixtures/dart/projects/<project-slug>/`.

Do not create fixture source.

## Scope accounting

Treat the canonical checklist as language research rather than implementation
shape. Plan coverage for:

- Every required item with a valid-source demonstration.
- Every distinct valid variant named by an item.
- Every conditional item supported by a declared build context.

Do not plan intentionally invalid source or compiler-error checks. Do not resolve
research gaps. In the overview, list only exclusions:

- Invalid-only items or variants.
- Unsupported conditional contexts.
- Research gaps.

Every other canonical identifier must have exactly one primary project. Natural
overlap is allowed and does not need accounting.

Declare each locally available build context needed to represent a valid
conditional item under the selected SDK. Do not add historical toolchains or
contexts that require external services.

## Planning boundary

The plan states what each project covers and the constraints it operates within.
It never designs source that creates coverage.

Beyond the assigned project directory, the required `Taskfile.yml`, and the
required `coverage.md`, do not include:

- Fixture source file or nested directory names.
- Fixture-owned library, package, type, function, method, field, variable,
  constant, label, or test names.
- Declarations, signatures, code snippets, or pseudocode.
- Domain entities, example scenarios, or expected application behavior.
- Implementation cases, subcases, call sequences, or step-by-step instructions.
- Validator internals or commands beyond the required Task interface and declared
  build contexts.

Project briefs name canonical items and valid variants as coverage assignments.
They do not translate those assignments into source designs. The implementor owns
every source choice not fixed by the ground rules. A third-party package named
under dependency needs is not a fixture-owned source-design choice.

## Project design

Choose the number, purpose, and theme of projects from natural Dart boundaries.
Do not divide the checklist into equal taxonomic packets. Each top-level project
is coherent, idiomatic, self-contained, and independent. It may contain several
local packages or a pub workspace, but it cannot depend on another fixture
project.

Assign whole projects as `routine` or `complex`. Do not split one project between
implementors.

Tests are planned only when test source creates assigned checklist coverage. For
each planned test, name that coverage. Otherwise record `none`. Do not plan tests
for a represented application's domain correctness.

A real third-party package belongs only when it creates distinct assigned
coverage that local packages cannot represent as well. Name the package and
purpose. The implementor will pin, fetch, vendor, and preserve licensing for an
offline build.

When generated source creates assigned coverage, name the generator relationship
and required committed outputs. Required validation tasks never generate files;
generator invocation belongs to a separate `generate` task.

## Overview contents

The overview records:

- The exact installed Dart SDK selected for the corpus.
- The default and additional available build contexts.
- The project list with purpose, directory, and difficulty.
- The primary assignment of every in-scope canonical identifier.
- An exceptions-only list with short reasons.
- An independence audit confirming exclusive directories and no cross-project
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
- Required `Taskfile.yml` and `coverage.md` interfaces from the ground rules.

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
- Run Markdown lint on the files you wrote.

Stop after writing and validating the plan artifacts.
