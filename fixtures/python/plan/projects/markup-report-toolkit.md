---
relationships:
  references:
    - ../../ground-rules
    - ../../../../reports/python/synthesis/checklist
---

# markup-report-toolkit

## Purpose

A text and markup report toolkit exercising source encoding and Unicode
identifiers, line and indentation structure, future statements, literal forms,
formatted strings, numeric edges, displays, assignment, expression evaluation,
compilation modes, docstrings, and soft keywords.

## Exclusive directory

`fixtures/python/projects/markup-report-toolkit/`

## Difficulty

`complex`

## Assigned coverage

Primary owner of every source-text, literal, expression, and compilation item.
Represent all valid variants, interactions, and counterexamples named by each
checklist row. Invalid, malformed, and pre-version parse-failure variants are
excluded per the ground rules and are not planned.

| Identifier | Valid-variant scope |
| --- | --- |
| `PY-CAN-SRC-001` | All named valid variants. |
| `PY-CAN-SRC-002` | All named valid variants. |
| `PY-CAN-SRC-003` | All named valid variants. |
| `PY-CAN-SRC-004` | All named valid variants. |
| `PY-CAN-SRC-005` | All named valid variants under the 3.12+ f-string grammar; malformed and pre-3.12 forms are excluded. |
| `PY-CAN-SRC-006` | All named valid variants. |
| `PY-CAN-SRC-007` | All named valid variants. |
| `PY-CAN-SRC-008` | All named valid variants. |
| `PY-CAN-SRC-009` | Walrus in loop, condition, parenthesized, and comprehension positions; pre-3.8 parse failure excluded. |
| `PY-CAN-SRC-010` | All named variants. |
| `PY-CAN-SRC-011` | `exec`, `eval`, and `single` compile and transform modes on valid trees. |
| `PY-CAN-SRC-012` | Top-level-await compilation with its flag and the ordinary-module contrast. |
| `PY-CAN-SRC-013` | Normal and optimized-build docstring behavior via the child-process context. |
| `PY-CAN-SRC-014` | Soft keywords as identifiers and as syntax on the selected interpreter. |

## Declared build contexts

- Default runtime.
- Child process / interpreter flags: for the `-O` and `-OO` docstring behavior
  of `PY-CAN-SRC-013`.

## Dependency needs

None.

## Generated-source needs

None.

## Planned tests

None.

## Excluded variants

- Malformed and pre-3.12 f-string forms of `PY-CAN-SRC-005`, the pre-3.8 walrus
  parse failure of `PY-CAN-SRC-009`, and parser-target-mismatch variants of
  `PY-CAN-SRC-014`: only CPython 3.14.6 is provisioned and invalid source is not
  fixture content.

## Required interfaces

Provide `Taskfile.yml` and `coverage.md` as defined in the ground rules.
