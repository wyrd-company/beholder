---
relationships:
  references:
    - ground-rules
    - planner
    - reports/go/synthesis/checklist
---

# Go fixture plan reviewer prompt

Perform one cold, bounded semantic review of Go fixture plan.

## Inputs

Read only:

- `fixtures/go/ground-rules.md`
- `reports/go/synthesis/checklist.md`
- `fixtures/go/plan/overview.md`
- Every file under `fixtures/go/plan/projects/`

Do not read research reports, memories, Git history, prior fixture work,
conversation history, or implementation source. Do not browse or consult other
agents. You may use local shell and read tools for checklist accounting and
installed-toolchain inspection. Write only review report.

## Output

Write `fixtures/go/reviews/plan-review.md`.

First line is exactly one of:

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
- An exclusion is not an invalid-only item or variant, unsupported conditional
  context, or research gap.
- A declared build context is unavailable or contradicts another plan statement.
- A project depends on another top-level project.
- Two projects claim same exclusive directory or a brief permits shared edits.
- Assigned features cannot plausibly coexist in coherent, idiomatic project.
- A planned test does not create named assigned checklist coverage.
- Third-party dependency plan conflicts with pinning, vendoring, provenance, or
  licensing rules.
- Generated-source plan makes required validation tasks mutate source.

## Out of scope

Do not create blocking findings about:

- Source file layout beyond required `Taskfile.yml` and `coverage.md`.
- Exact declarations, examples, or implementation choices.
- Domain behavior or application correctness.
- Project count, equal project sizes, theme preference, or checklist overlap.
- Compiler diagnostics, invalid-source examples, or parser recovery.
- Validators, manifests, hashes, logs, deterministic evidence, mutation testing,
  fault injection, filesystem hardening, or sandboxing.
- Graph nodes, edges, expected output, scoring, queries, or rubric design.
- Historical Go toolchains or build contexts outside declared corpus matrix.
- Production readiness, broad security hardening, performance, packaging,
  deployment, or backward compatibility.

Do not turn optional improvements into blockers.

## Verification pass

When reviewing one allowed repair pass, verify original blocking findings and any
direct regression introduced by their fixes. Do not reopen accepted areas or
expand review scope. Continued rejection returns plan to human reassessment.
