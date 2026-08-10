---
relationships:
  references:
    - reports/dart/synthesis/checklist
---

# Dart fixture corpus ground rules

## Purpose

The Dart fixture corpus is controlled, valid Dart source used to evaluate
code-graph approaches. Source creates the language entities and relationships
that an evaluation rubric later describes. Fixture creation does not define graph
output.

The canonical language-feature checklist is
[`reports/dart/synthesis/checklist.md`](../../reports/dart/synthesis/checklist.md).
It is the research authority for fixture scope.

## Language scope

Every project analyzes and runs on one selected stable Dart Software Development
Kit (SDK) with sound null safety. The plan records that SDK version and the
package and per-library language versions used. The default build context is the
Dart Virtual Machine (VM) in Just-in-Time (JIT) mode on 64-bit x86 Linux. A
project adds another compilation target, execution mode, platform-library set,
operating system, or architecture only when valid checklist coverage requires
it. The corpus does not provision historical Dart toolchains.

Corpus coverage includes:

- Every required checklist item with a valid-source demonstration.
- Every valid variant that creates a distinct source structure or semantic
  relationship.
- Every conditional item supported by a declared build context.

Several variants may share one source section. Coverage does not require a
separate project, file, declaration, or validation command for each variant.

The plan records exceptions for:

- Conditions outside declared build contexts.
- Items or variants that require invalid source.
- Checklist items classified as research gaps.

All corpus source is valid in its declared context. Compiler rejection, analyzer
diagnostics, parser recovery, and intentionally invalid examples are not fixture
content. Where the checklist names a counterexample or failure, the corpus
represents only the valid side.

## Project model

The corpus contains as many projects as its language coverage naturally
requires. Each project represents an idiomatic Dart package, executable,
workspace, or other coherent source tree. Projects need not share a theme, size,
package layout, or source shape.

Themes and example values are generic and non-identifying. They do not use code
analysis, fixture evaluation, or another implementation-related domain.

Each top-level project:

- Occupies one exclusive directory under `fixtures/dart/projects/`.
- Is independently implementable and reviewable.
- Contains every local package it needs.
- Has no dependency on another top-level fixture project.
- Contains `Taskfile.yml` and `coverage.md`; all other files follow the
  project's natural structure.

Checklist features may appear in several projects. The reviewed plan assigns one
project as primary owner for each in-scope item.

## Source and dependencies

Projects use the SDK libraries, project-local packages, or vendored third-party
packages. A reviewed project brief names every third-party package. An
implementor may access the network only to acquire those named packages.

Because pub resolution is required before analysis or compilation, each project
commits the artifacts that make resolution offline and reproducible:

- A `pubspec.yaml` with a pinned SDK constraint.
- A committed `pubspec.lock` pinning every resolved package version.
- For every third-party package, a pinned version, its source location, its
  required license and notice files, and the cached package content needed for
  an offline `dart pub get --offline`.

`build`, `lint`, and `test` perform no downloads and run against the committed
resolution. Vendored third-party source is visible to later code-graph
evaluation, while its style and behavior are not fixture review subjects.

When generation provides assigned coverage, the project commits both generator
source and generated output. Required validation tasks never regenerate source.
The plan may require a separate `generate` task when generator invocation is
itself fixture content.

## Tests

Test source exists only when the reviewed project brief requires it to create
assigned checklist coverage. Each planned test identifies the coverage its
source creates. Implementors do not add tests independently.

Tests are fixture input rather than domain-correctness evidence. A project does
not test behavior merely to prove that its represented application works.

## Taskfile interface

Every project defines these Task tasks:

- `build` resolves packages offline, analyzes all fixture-owned libraries in the
  default context, and compiles each declared entry point in every declared
  compilation target.
- `lint` checks `dart format` formatting and runs `dart analyze` on
  fixture-owned source under the project's analyzer configuration. Vendored
  source is excluded.
- `test` runs `dart test` so included test source compiles and executes. It
  succeeds when a project has no test files.

All defined tasks pass. They are thin wrappers around ordinary Dart commands.
They do not download packages, mutate tracked source, write validation logs,
evidence records, manifests, hashes, or audit output.

A project whose coverage requires generator invocation also defines `generate`.
`generate` is the only task permitted to write generated source, and no other
required task invokes it.

The corpus root defines `build`, `lint`, and `test` and invokes the
corresponding task in every accepted project.

## Coverage files

Each project's `coverage.md` is the source of truth for its primary assignments.
Its Markdown table records for each assigned checklist item:

- Canonical checklist identifier.
- Stable source path.
- Named declaration, directive, library, package, or test that creates coverage.
- Build context when it differs from the default.
- Additional locators when distinct valid variants require them.

Coverage files use stable names rather than line numbers. They describe source
locations, not expected graph nodes, edges, scores, or queries. The root
coverage index links to project coverage files without duplicating their
mappings.

## Planning

Opus at high reasoning effort creates the plan. The planner receives only this
file and the canonical Dart checklist. It may inspect the installed Dart
toolchain with local shell and read tools. It does not browse, perform external
research, read research inputs, inspect unrelated repositories, or inspect prior
fixture work.

The plan contains one overview and one standalone brief per project. The
overview:

- Records the selected Dart SDK and available build contexts.
- Assigns every in-scope checklist item to one primary project.
- Lists only scope exceptions and their reasons.
- Confirms that project directories and dependencies are independent.

Each project brief records:

- Project purpose and exclusive directory.
- Assigned checklist items and distinct valid variants.
- Declared build contexts.
- Local or third-party dependency needs.
- Generated-source needs.
- Planned tests, or `none`, with coverage created by each planned test.
- Difficulty as `routine` or `complex` for the entire project.

The plan stops at project boundaries. It identifies coverage ownership and
project constraints without designing source. Beyond required `Taskfile.yml` and
`coverage.md`, it does not name fixture source files, fixture-owned libraries or
declarations, domain examples, cases, subcases, call sequences, code,
pseudocode, validation internals, or implementation steps. The implementor owns
those choices.

The plan receives one cold review from a fresh Codex `gpt-5.6-sol` agent at
medium reasoning effort and one repair pass. Continued rejection stops planning
for human reassessment.

## Implementation

Each implementor receives only:

- This file.
- Its standalone project brief.
- Its assigned canonical checklist rows.
- Its isolated branch and worktree.

Routine projects use Pi `meta/muse-spark-1.2-contributor`. Complex projects use
Codex `gpt-5.6-luna` at maximum reasoning effort. Implementors may use shell,
read, and write tools inside assigned worktrees. External research and unrelated
workspace reads are not inputs. The only network exception is acquisition of a
third-party package named by the reviewed brief.

An implementor chooses natural source structure within its project directory. It
does not edit shared corpus files, another project, the checklist, or the
reviewed brief. It does not drop or reassign coverage. An assignment that does
not fit the project returns to planning rather than moving between projects
during implementation.

## Project review

A fresh Codex `gpt-5.6-sol` reviewer at medium reasoning effort receives only:

- This file.
- Project brief and assigned checklist rows.
- Project diff and coverage file.
- Required Task commands.

The reviewer may block acceptance only when:

- An assigned feature or valid variant is absent or misrepresented.
- The project fails `build`, `lint`, or `test` in a declared context.
- A coverage locator does not identify claimed source.
- A dependency is missing, unpinned, unvendored, or missing required provenance,
  license, or notices.
- The project changes another project or a shared corpus file.
- Source is so artificial or unclear that it obscures or misrepresents assigned
  coverage.
- Normal validation downloads packages or performs an unsafe external action.

Ordinary style preferences are non-blocking. Vendored third-party code quality
is not reviewed.

The reviewer does not create blocking findings about:

- Hostile environments, symbolic links, occupied paths, cleanup traps, or
  sandbox escapes.
- Mutation testing, fault injection, or whether validation checks are
  independently load-bearing.
- Compiler diagnostics, invalid programs, or parser recovery.
- Manifests, hashes, transcripts, audit logs, deterministic evidence, or
  validation hardening.
- Dart SDK versions, operating systems, architectures, or build contexts outside
  the project brief.
- Code-graph output, expected nodes or edges, scoring, or rubric design.
- Production readiness, domain correctness, performance, packaging, deployment,
  broad security hardening, or backward compatibility.
- Features assigned to another project or corpus-wide completeness.
- Style choices that do not obscure or misrepresent assigned coverage.

The review report contains `ACCEPT` or `REJECT`, source-anchored blocking
findings, and results of `task build`, `task lint`, and `task test`. Optional
notes are explicitly non-blocking.

A rejected project receives one repair pass limited to blocking findings. The
same reviewer verifies those findings. Continued rejection returns the project
brief to human reassessment.

## Plan review

The plan reviewer may block acceptance only when:

- An in-scope checklist item or distinct valid variant lacks a primary project.
- A declared build context is unavailable or contradicts another plan statement.
- A project depends on another top-level project or lacks an exclusive
  directory.
- Assigned features cannot plausibly coexist in a coherent, idiomatic project.
- A brief prescribes source design or implementation beyond permitted project
  purpose, assignments, contexts, dependencies, generation needs, and planned
  tests.
- A planned test does not create named checklist coverage.
- Dependency or generated-source handling conflicts with this contract.

The plan reviewer enforces absence of source design without proposing
replacement design or judging choices reserved for the implementor. Domain
behavior, validators, evidence systems, graph output, rubric assertions, project
count, equal work sizes, and preferred themes remain outside review scope.

## Branches and review surfaces

The corpus lives on an orphan Beholder staging branch while it is independent of
Beholder product context. Each plan or project uses a temporary branch and
worktree based on that staging branch. Reviews use local `gitpr` diffs. No
GitHub pull request is created for fixture planning or project work.

Accepted commits are applied to the staging branch without merge commits.
Temporary local and remote branches and worktrees are removed after integration.
When Beholder adoption is authorized, selected artifacts enter `main` through
normal commits rather than a merge of unrelated histories.

## Corpus acceptance

The coordinator accepts the fixture corpus when:

- The reviewed plan assigns every in-scope checklist item.
- Every planned project has an accepting semantic review.
- Every assigned item appears in its reviewed project coverage file.
- Root `task build`, `task lint`, and `task test` pass.

Acceptance uses reviewed project artifacts and Task results. It does not add a
coverage manifest, coverage parser, final semantic corpus review, code-graph
run, or evaluation rubric.
