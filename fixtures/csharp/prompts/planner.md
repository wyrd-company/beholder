---
relationships:
  references:
    - ground-rules
    - reports/csharp/synthesis/checklist
---

# C# fixture planner prompt

Create project-level implementation plan for controlled C# fixture corpus.

## Inputs

Read these files completely:

- `fixtures/csharp/ground-rules.md`
- `reports/csharp/synthesis/checklist.md`

These are complete context. Do not read other repository files, research reports,
memories, prior work, or conversation history. Do not browse or consult other
agents. You may use shell, read, and write tools within this worktree. You may
inspect the installed .NET SDK, Roslyn, MSBuild, shared frameworks, reference and
runtime packs, and companion compilers to determine feasible build contexts.

## Output

Write:

- `fixtures/csharp/plan/overview.md`
- One file per project under `fixtures/csharp/plan/projects/<project-slug>.md`

Use lower-kebab-case project slugs. Each project directory is
`fixtures/csharp/projects/<project-slug>/`.

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
conditional item under the selected SDK. Contexts include earlier `LangVersion`
selections, additional target frameworks, installed shared frameworks, runtime
identifiers, unsafe compilation, publish modes, and non-C# producers. Do not add
historical SDKs or Roslyn compilers, Windows- or macOS-only frameworks, non-x64
processes, or contexts that require external services.

## Planning boundary

Plan states what each project covers and constraints it operates within. It never
designs source that creates coverage.

Beyond assigned project directory, required `Taskfile.yml`, and required
`coverage.md`, do not include:

- Fixture source file or nested directory names.
- Fixture-owned project, assembly, namespace, type, member, field, property,
  event, parameter, local, constant, label, or test names.
- Declarations, signatures, code snippets, or pseudocode.
- Domain entities, example scenarios, or expected application behavior.
- Implementation cases, subcases, call sequences, or step-by-step instructions.
- Validator internals or commands beyond the required Task interface and declared
  build contexts.

Project briefs name canonical items and valid variants as coverage assignments.
They do not translate those assignments into source designs. Implementor owns
every source choice not fixed by ground rules. A third-party package named under
dependency needs is not a fixture-owned source-design choice.

## Project design

Choose number, purpose, and theme of projects from natural C# boundaries. Do not
divide checklist into equal taxonomic packets. Each top-level project is coherent,
idiomatic, self-contained, and independent. It may contain several local projects
and assemblies, but it cannot depend on another fixture project.

Assign whole projects as `routine` or `complex`. Do not split one project between
implementors.

Tests are planned only when test source creates assigned checklist coverage. For
each planned test, name that coverage. Otherwise record `none`. Do not plan tests
for a represented application's domain correctness.

A real third-party dependency belongs only when it creates distinct assigned
coverage that local fixture projects cannot represent as well. Name dependency and
purpose. Implementor will pin, fetch, vendor, and preserve licensing.

When generated source creates assigned coverage, name generator relationship and
required committed outputs. Required validation tasks never write or rewrite
tracked source. In-process Roslyn generation during compilation is part of the
build, not a regeneration step.

## Overview contents

Overview records:

- Exact installed .NET SDK selected for the corpus, with bundled Roslyn, MSBuild,
  and target CLR.
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
