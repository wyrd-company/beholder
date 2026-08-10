---
relationships:
  references:
    - reports/python/synthesis/checklist
---

# Python fixture corpus ground rules

## Purpose

The Python fixture corpus is controlled, valid Python source used to evaluate
code-graph approaches. Source creates the language entities and relationships
that an evaluation rubric later describes. Fixture creation does not define graph
output.

The canonical language-feature checklist is
[`reports/python/synthesis/checklist.md`](../../reports/python/synthesis/checklist.md).
It is the research authority for fixture scope.

## Language scope

Every project builds and runs under one selected release of the installed
CPython interpreter in standards-based `pyproject.toml` form. The plan records
that interpreter version. The default build context is 64-bit x86-64 Linux on a
standard, GIL-enabled CPython build. A project adds another interpreter build,
operating system, architecture, C-interoperability, or ecosystem-tool context
only when valid checklist coverage requires it and that context is installed
locally. The corpus does not provision historical Python interpreters,
free-threaded or JIT interpreter builds, alternative implementations, or other
operating systems that are absent from the local toolchain.

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

All corpus source is valid in its declared context and parses under the selected
interpreter. Compiler and parser rejection, diagnostic text, parser recovery,
and intentionally invalid examples are not fixture content.

## Project model

The corpus contains as many projects as its language coverage naturally
requires. Each project represents an idiomatic Python library, application,
package distribution, or other coherent source tree. Projects need not share a
theme, size, package layout, or source shape.

Themes and example values are generic and non-identifying. They do not use code
analysis, fixture evaluation, or another implementation-related domain.

Each top-level project:

- Occupies one exclusive directory under `fixtures/python/projects/`.
- Is independently implementable and reviewable.
- Contains every local package and distribution it needs.
- Has no dependency on another top-level fixture project.
- Contains `Taskfile.yml` and `coverage.md`; all other files follow the
  project's natural structure.

Checklist features may appear in several projects. The reviewed plan assigns one
project as primary owner for each in-scope item.

## Source and dependencies

Projects use the standard library, project-local packages and distributions, or
vendored third-party distributions. A reviewed project brief names every
third-party dependency, including any PyPA build backend that a packaging
project needs for an isolated build. An implementor may access the network only
to acquire those named dependencies.

Vendored third-party source includes:

- A pinned distribution version or source revision.
- Source location.
- Required license and notice files.
- Wheel or metadata artifacts needed for offline installs and builds.

`build`, `lint`, and `test` perform no downloads. Vendored source is visible to
later code-graph evaluation, while its style and behavior are not fixture review
subjects.

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

- `build` byte-compiles all fixture-owned packages in every declared build
  context and compiles any fixture-owned native or generated artifact the
  project requires.
- `lint` checks that every fixture-owned source file parses and byte-compiles
  under the selected interpreter. It runs no third-party tool and excludes
  vendored source.
- `test` runs the project's standard-library tests so any included test source
  compiles and executes. It succeeds when a project has no test files.

All three tasks pass. They are thin wrappers around ordinary Python and Task
commands. They do not download dependencies, mutate tracked source, or write
validation logs, evidence records, manifests, hashes, or audit output.

The corpus root defines the same three tasks and invokes the corresponding task
in every accepted project.

## Coverage files

Each project's `coverage.md` is the source of truth for its primary assignments.
Its Markdown table records for each assigned checklist item:

- Canonical checklist identifier.
- Stable source path.
- Named declaration, package, distribution, configuration, or test that creates
  coverage.
- Build context when it differs from the default.
- Additional locators when distinct valid variants require them.

Coverage files use stable names rather than line numbers. They describe source
locations, not expected graph nodes, edges, scores, or queries. The root
coverage index links to project coverage files without duplicating their
mappings.

## Planning

Opus at high reasoning effort creates the plan. The planner receives only this
file and the canonical Python checklist. It may inspect the installed Python
toolchain with local shell and read tools. It does not browse, perform external
research, read research inputs, inspect unrelated repositories, or inspect prior
fixture work.

The plan contains one overview and one standalone brief per project. The
overview:

- Records the selected CPython interpreter and available build contexts.
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
`coverage.md`, it does not name fixture source files, fixture-owned packages,
modules, or declarations, domain examples, cases, subcases, call sequences,
code, pseudocode, validation internals, or implementation steps. The implementor
owns those choices.

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
third-party dependency named by the reviewed brief.

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

- Assigned feature or valid variant is absent or misrepresented.
- Project fails `build`, `lint`, or `test` in a declared context.
- A coverage locator does not identify claimed source.
- A dependency is missing, unpinned, unvendored, or missing required provenance,
  license, or notices.
- Project changes another project or a shared corpus file.
- Source is so artificial or unclear that it obscures or misrepresents assigned
  coverage.
- Normal validation downloads dependencies or performs an unsafe external
  action.

Ordinary style preferences are non-blocking. Vendored third-party code quality
is not reviewed.

The reviewer does not create blocking findings about:

- Hostile environments, symbolic links, occupied paths, cleanup traps, or
  sandbox escapes.
- Mutation testing, fault injection, or whether validation checks are
  independently load-bearing.
- Compiler or parser diagnostics, invalid programs, or parser recovery.
- Manifests, hashes, transcripts, audit logs, deterministic evidence, or
  validation hardening.
- Python versions, operating systems, architectures, interpreter builds, or
  build contexts outside the project brief.
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

- Reviewed plan assigns every in-scope checklist item.
- Every planned project has an accepting semantic review.
- Every assigned item appears in its reviewed project coverage file.
- Root `task build`, `task lint`, and `task test` pass.

Acceptance uses reviewed project artifacts and Task results. It does not add a
coverage manifest, coverage parser, final semantic corpus review, code-graph
run, or evaluation rubric.
