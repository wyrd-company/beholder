---
relationships:
  references:
    - ground-rules
    - planner
    - reports/python/synthesis/checklist
---

# Python fixture plan reviewer prompt

Perform one cold, bounded semantic review of the Python fixture plan.

## Inputs

Read only:

- `fixtures/python/ground-rules.md`
- `reports/python/synthesis/checklist.md`
- `fixtures/python/plan/overview.md`
- Every file under `fixtures/python/plan/projects/`

Do not read research reports, memories, Git history, prior fixture work,
conversation history, or implementation source. Do not browse or consult other
agents. You may use local shell and read tools for checklist accounting and
installed-toolchain inspection. Write only the review report.

## Output

Write `fixtures/python/reviews/plan-review.md`.

The first line is exactly one of:

```text
VERDICT: ACCEPT
```

```text
VERDICT: REJECT
```

For each blocking finding, provide:

- Finding identifier and severity.
- Exact plan and checklist anchors.
- Violated ground rule.
- Concrete missing, contradictory, dependent, or infeasible condition.

Report installed-toolchain observations used during review. Optional notes are
clearly labeled non-blocking.

## Blocking scope

Reject only when:

- An in-scope checklist item lacks one primary project.
- A distinct valid variant lacks an assignment.
- An exclusion is not an invalid-only item or variant, an unsupported
  conditional context, or a research gap.
- A declared build context is unavailable or contradicts another plan statement.
- A project depends on another top-level project.
- Two projects claim the same exclusive directory or a brief permits shared
  edits.
- Assigned features cannot plausibly coexist in a coherent, idiomatic project.
- A brief prescribes fixture source files beyond required interfaces,
  fixture-owned package or declaration names, code, pseudocode, domain examples,
  cases, subcases, call sequences, validator internals, or implementation steps.
- A planned test does not create named assigned checklist coverage.
- A third-party dependency plan conflicts with pinning, vendoring, provenance,
  or licensing rules.
- A generated-source plan makes required validation tasks mutate source.

## Out of scope

Enforce absence of source design under blocking scope. Do not propose
replacement source design or judge implementation choices reserved for the
implementor. Domain behavior, validators, evidence systems, graph output, rubric
assertions, project count, equal work sizes, and preferred themes remain outside
review scope.

Do not create blocking findings about:

- Domain behavior or application correctness.
- Project count, equal project sizes, theme preference, or checklist overlap.
- Compiler or parser diagnostics, invalid-source examples, or parser recovery.
- Validators, manifests, hashes, logs, deterministic evidence, mutation testing,
  fault injection, filesystem hardening, or sandboxing.
- Graph nodes, edges, expected output, scoring, queries, or rubric design.
- Historical Python interpreters, absent interpreter builds, alternative
  implementations, or build contexts outside the declared corpus matrix.
- Production readiness, broad security hardening, performance, packaging,
  deployment, or backward compatibility.

Do not turn optional improvements into blockers.

## Verification pass

When reviewing the one allowed repair pass, verify the original blocking
findings and any direct regression introduced by their fixes. Do not reopen
accepted areas or expand review scope. Continued rejection returns the plan to
human reassessment.
