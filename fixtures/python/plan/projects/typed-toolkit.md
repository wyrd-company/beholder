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

Primary owner of the annotation, generic, and static-contract items below.
Represent all valid variants named by each checklist row as source constructs.
Type-checker, linter, and formatter execution is not fixture content. Rows whose
defining obligation is a fixed or selected tool context that is not installed
(`PY-CAN-TYPE-015`, `PY-CAN-TYPE-017`) are not assigned here; they are
unavailable conditional contexts recorded in the overview exceptions. The
remaining items are covered by valid source and stdlib-runtime behavior under the
default context; required rows whose static aspect a checker would judge are
represented as source without executing any tool.

| Identifier | Valid-variant scope |
| --- | --- |
| `PY-CAN-TYPE-001` | Future-stringized regime and raw-versus-resolved hint inspection valid on the selected interpreter; the eager definition-time regime is unavailable (deferred default on 3.14) and the deferred regime's PEP 649 detail stays research gap `PY-GAP-001`. |
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
| `PY-CAN-TYPE-016` | Marker-present, marker-absent, stub, partial, and extension-stub artifacts. |
| `PY-CAN-TYPE-018` | All named variants. |
| `PY-CAN-TYPE-019` | All named variants. |
| `PY-CAN-TYPE-020` | All named variants. |
| `PY-CAN-TYPE-021` | All named variants. |
| `PY-CAN-TYPE-022` | All named valid variants. |

## Declared build contexts

- Default runtime.

## Dependency needs

None.

## Generated-source needs

None.

## Planned tests

None.

## Excluded variants and unavailable items

- The eager definition-time annotation regime of `PY-CAN-TYPE-001`: unavailable
  historical-interpreter variant (the 3.14 default is deferred); the deferred
  regime's PEP 649 detail stays research gap `PY-GAP-001`.
- `PY-CAN-TYPE-015` and `PY-CAN-TYPE-017` are not assigned to this project: no
  type checker, linter, or formatter is installed, so their required tool
  contexts are unavailable conditional contexts (see the overview exceptions).

## Required interfaces

Provide `Taskfile.yml` and `coverage.md` as defined in the ground rules.
