---
relationships:
  references:
    - ../../ground-rules
    - ../../../../reports/python/synthesis/checklist
---

# async-task-toolkit

## Purpose

A cooperative task toolkit of callables, decorators, generators, coroutines, and
async iteration, with parameter-binding, caching, and context-variable
propagation across its scheduling primitives.

## Exclusive directory

`fixtures/python/projects/async-task-toolkit/`

## Difficulty

`complex`

## Assigned coverage

Primary owner of every function, callable, decorator, and suspension item.
Represent all valid variants, interactions, and counterexamples named by each
checklist row.

| Identifier | Valid-variant scope |
| --- | --- |
| `PY-CAN-CALL-001` | All named variants. |
| `PY-CAN-CALL-002` | All named variants. |
| `PY-CAN-CALL-003` | All named variants. |
| `PY-CAN-CALL-004` | All named variants. |
| `PY-CAN-CALL-005` | All named variants. |
| `PY-CAN-CALL-006` | All named variants. |
| `PY-CAN-CALL-007` | All named variants. |
| `PY-CAN-CALL-008` | All named variants. |
| `PY-CAN-CALL-009` | All named variants. |
| `PY-CAN-CALL-010` | All named variants. |
| `PY-CAN-CALL-011` | All named variants. |
| `PY-CAN-CALL-012` | All named variants. |
| `PY-CAN-CALL-013` | All named variants. |
| `PY-CAN-CALL-014` | All named variants, including the thread-transition case. |
| `PY-CAN-CALL-015` | All named valid variants; the `-OO` doc-removal case uses the child-process context. |

## Declared build contexts

- Default runtime, including the asyncio event loop.
- Child process / interpreter flags: for the `-OO` docstring-removal case of
  `PY-CAN-CALL-015`.

## Dependency needs

None.

## Generated-source needs

None.

## Planned tests

None.

## Required interfaces

Provide `Taskfile.yml` and `coverage.md` as defined in the ground rules.
