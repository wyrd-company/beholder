---
relationships:
  references:
    - ground-rules
    - planner
    - reports/typescript/synthesis/checklist
---

# TypeScript fixture plan reviewer prompt

Perform one cold, bounded semantic review of TypeScript fixture plan.

A fresh Codex `gpt-5.6-sol` agent at medium reasoning effort performs this
review.

## Inputs

Read only:

- `fixtures/typescript/ground-rules.md`
- `fixtures/typescript/prompts/plan-reviewer.md`
- `reports/typescript/synthesis/checklist.md`
- `fixtures/typescript/plan/overview.md`
- Every file under `fixtures/typescript/plan/projects/`

The accepted Go ground rules and Go plan-reviewer prompt are available only as
policy templates. Do not read research reports, Go plans, Go fixture source, Go
reviews, another language suite, memories, Git history, prior fixture work,
conversation history, or implementation source. Do not browse or consult other
agents. You may use local shell and read tools for checklist accounting and
installed-toolchain inspection. Write only the review report.

## Output

Write `fixtures/typescript/reviews/plan-review.md`.

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
- A declared compilation context is unavailable or contradicts another plan
  statement.
- A project depends on another top-level project.
- Two projects claim same exclusive directory or a brief permits shared edits.
- Assigned features cannot plausibly coexist in coherent, idiomatic project.
- A brief prescribes fixture source files beyond required interfaces,
  fixture-owned module or declaration names, code, pseudocode, domain examples,
  cases, subcases, call sequences, validator internals, or implementation
  steps.
- A planned test does not create named assigned checklist coverage.
- Third-party dependency plan conflicts with pinning, vendoring, provenance, or
  licensing rules.
- Generated-source plan makes required validation tasks mutate source.

## Out of scope

Enforce absence of source design under blocking scope. Do not propose
replacement source design or judge implementation choices reserved for
implementor.

Do not create blocking findings about:

- Domain behavior or application correctness.
- Project count, equal project sizes, theme preference, or checklist overlap.
- Compiler diagnostics, invalid-source examples, or parser recovery.
- Validators, manifests, hashes, logs, deterministic evidence, mutation
  testing, fault injection, filesystem hardening, or sandboxing.
- Graph nodes, edges, expected output, scoring, queries, or rubric design.
- Historical toolchains, hosts, or compilation contexts outside declared corpus
  matrix.
- Production readiness, broad security hardening, performance, packaging,
  deployment, or backward compatibility.

Do not turn optional improvements into blockers.

## Verification pass

When reviewing one allowed repair pass, verify original blocking findings and
any direct regression introduced by their fixes. Do not reopen accepted areas or
expand review scope. Continued rejection returns plan to human reassessment.
