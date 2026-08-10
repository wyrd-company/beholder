---
relationships:
  references:
    - ../../ground-rules
    - ../../../../reports/python/synthesis/checklist
---

# typed-toolkit

## Purpose

A statically typed utility library exercising the annotation and typing surface:
annotation regimes, generics, protocols, typed dictionaries, overloads, type
parameters, dataclass transforms, `Annotated` metadata, stubs, and typed
distributions.

## Exclusive directory

`fixtures/python/projects/typed-toolkit/`

## Difficulty

`complex`

## Assigned coverage

Primary owner of every annotation, generic, and static-contract item. Represent
all valid variants named by each checklist row as source constructs and
configuration artifacts. Type-checker execution is not fixture content; the
checker-directive and checker-configuration items are satisfied by the presence
of those source constructs and configuration, not by running a checker.

| Identifier | Valid-variant scope |
| --- | --- |
| `PY-CAN-TYPE-001` | Eager and future-stringized regimes; the 3.14 lazy regime is excluded as research gap `PY-GAP-001`. |
| `PY-CAN-TYPE-002` | All named variants. |
| `PY-CAN-TYPE-003` | All named variants. |
| `PY-CAN-TYPE-004` | All named valid variants, including the 3.12 `type` statement. |
| `PY-CAN-TYPE-005` | All named variants, including PEP 696 defaults. |
| `PY-CAN-TYPE-006` | All named variants. |
| `PY-CAN-TYPE-007` | All named variants. |
| `PY-CAN-TYPE-008` | All named variants. |
| `PY-CAN-TYPE-009` | All named variants, including runtime `get_overloads`. |
| `PY-CAN-TYPE-010` | All named variants. |
| `PY-CAN-TYPE-011` | All named variants. |
| `PY-CAN-TYPE-012` | All named variants, including 3.13 `TypeIs`. |
| `PY-CAN-TYPE-013` | All named variants. |
| `PY-CAN-TYPE-014` | All named variants. |
| `PY-CAN-TYPE-015` | Directive and probe constructs as source, including runtime `reveal_type`. |
| `PY-CAN-TYPE-016` | Marker-present, marker-absent, stub, partial, and extension-stub artifacts. |
| `PY-CAN-TYPE-017` | Checker configuration and scoped override as configuration artifacts. |
| `PY-CAN-TYPE-018` | All named variants. |
| `PY-CAN-TYPE-019` | All named variants. |
| `PY-CAN-TYPE-020` | All named variants. |
| `PY-CAN-TYPE-021` | All named variants. |
| `PY-CAN-TYPE-022` | All named valid variants. |

## Declared build contexts

- Default runtime.

## Dependency needs

None. Type checkers, linters, and formatters are named only within configuration
artifacts and are not executed.

## Generated-source needs

None.

## Planned tests

None.

## Excluded variants

- The 3.14 lazy annotation regime of `PY-CAN-TYPE-001`: research gap
  `PY-GAP-001`.

## Required interfaces

Provide `Taskfile.yml` and `coverage.md` as defined in the ground rules.
