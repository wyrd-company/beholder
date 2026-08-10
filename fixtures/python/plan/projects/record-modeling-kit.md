---
relationships:
  references:
    - ../../ground-rules
    - ../../../../reports/python/synthesis/checklist
---

# record-modeling-kit

## Purpose

A record and entity modeling kit built on the class-creation protocol,
descriptors, properties, slots, abstract and virtual bases, dataclasses, enums,
and named tuples, including dynamic and local class creation.

## Exclusive directory

`fixtures/python/projects/record-modeling-kit/`

## Difficulty

`complex`

## Assigned coverage

Primary owner of every class, descriptor, and identity item. Represent all valid
variants, interactions, and counterexamples named by each checklist row.

| Identifier | Valid-variant scope |
| --- | --- |
| `PY-CAN-OBJ-001` | All named variants. |
| `PY-CAN-OBJ-002` | All named variants. |
| `PY-CAN-OBJ-003` | All named variants. |
| `PY-CAN-OBJ-004` | All named variants. |
| `PY-CAN-OBJ-005` | All named variants. |
| `PY-CAN-OBJ-006` | All named variants. |
| `PY-CAN-OBJ-007` | All named variants. |
| `PY-CAN-OBJ-008` | All named variants. |
| `PY-CAN-OBJ-009` | All named variants. |
| `PY-CAN-OBJ-010` | All named variants. |
| `PY-CAN-OBJ-011` | All named variants. |
| `PY-CAN-OBJ-012` | All named valid variants. |
| `PY-CAN-OBJ-013` | All named variants, including frozen, keyword-only, slots, and matching. |
| `PY-CAN-OBJ-014` | All named variants, including `StrEnum` and flag mixing. |
| `PY-CAN-OBJ-015` | All named variants, including the generic named-tuple syntax. |
| `PY-CAN-OBJ-016` | All named valid variants without asserting interning. |
| `PY-CAN-OBJ-017` | All named valid variants using non-timing-based checks. |
| `PY-CAN-OBJ-018` | All named variants. |
| `PY-CAN-OBJ-019` | All named variants. |
| `PY-CAN-OBJ-020` | All named variants. |

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
