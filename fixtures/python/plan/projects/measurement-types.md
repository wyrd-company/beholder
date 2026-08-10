---
relationships:
  references:
    - ../../ground-rules
    - ../../../../reports/python/synthesis/checklist
---

# measurement-types

## Purpose

A quantity and container value library whose types implement the data-model
protocols: truth testing, comparison, operators, containers, iteration, context
managers, representation, copy and pickle, and buffer views.

## Exclusive directory

`fixtures/python/projects/measurement-types/`

## Difficulty

`complex`

## Assigned coverage

Primary owner of every data-model-protocol and dispatch item. Represent all valid
variants, interactions, and counterexamples named by each checklist row.

| Identifier | Valid-variant scope |
| --- | --- |
| `PY-CAN-PROT-001` | All named variants. |
| `PY-CAN-PROT-002` | All named variants. |
| `PY-CAN-PROT-003` | All named variants. |
| `PY-CAN-PROT-004` | All named variants. |
| `PY-CAN-PROT-005` | All named variants. |
| `PY-CAN-PROT-006` | All named variants. |
| `PY-CAN-PROT-007` | All named variants. |
| `PY-CAN-PROT-008` | All named variants. |
| `PY-CAN-PROT-009` | All named variants. |
| `PY-CAN-PROT-010` | All named valid variants. |
| `PY-CAN-PROT-011` | All named variants. |
| `PY-CAN-PROT-012` | All named valid variants; the untrusted-load prohibition is a documented negative, not an executed load. |
| `PY-CAN-PROT-013` | Python-level buffer export, sliced and cast `memoryview`, and release, per the row. |

## Declared build contexts

- Default runtime.

## Dependency needs

None.

## Generated-source needs

None.

## Planned tests

None.

## Required interfaces

Provide `Taskfile.yml` and `coverage.md` as defined in the ground rules.
