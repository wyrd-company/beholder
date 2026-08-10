---
relationships:
  references:
    - ../../ground-rules
    - ../../../../reports/python/synthesis/checklist
---

# layered-settings-resolver

## Purpose

A layered settings resolver that merges values across scopes, exercising local,
enclosing, global, and built-in binding, closures, class-body namespaces,
comprehension scope, explicit namespaces, and name mangling.

## Exclusive directory

`fixtures/python/projects/layered-settings-resolver/`

## Difficulty

`routine`

## Assigned coverage

Primary owner of every binding, scope, and dynamic-namespace item. Represent all
valid variants, interactions, and counterexamples named by each checklist row.

| Identifier | Valid-variant scope |
| --- | --- |
| `PY-CAN-BIND-001` | All named variants. |
| `PY-CAN-BIND-002` | All named variants. |
| `PY-CAN-BIND-003` | All named variants. |
| `PY-CAN-BIND-004` | All named variants, including the 3.12 type-parameter scope. |
| `PY-CAN-BIND-005` | All named variants. |
| `PY-CAN-BIND-006` | All named variants. |
| `PY-CAN-BIND-007` | All named valid variants. |
| `PY-CAN-BIND-008` | All named variants, including the 3.13 `locals()` behavior. |
| `PY-CAN-BIND-009` | All named valid introspection variants. |
| `PY-CAN-BIND-010` | All named variants. |
| `PY-CAN-BIND-011` | All named variants. |

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
