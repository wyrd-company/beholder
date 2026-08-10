---
relationships:
  references:
    - ../../ground-rules
    - ../../../../reports/python/synthesis/checklist
---

# job-runner

## Purpose

A resilient job runner covering conditional and loop flow, exception handling and
groups, custom exception hierarchies, assertions, resource lifetime, threading,
multiprocessing, structured async concurrency, signals, and conditional
definitions.

## Exclusive directory

`fixtures/python/projects/job-runner/`

## Difficulty

`complex`

## Assigned coverage

Primary owner of every control-flow, exception, resource, and concurrency item.
Represent all valid variants, interactions, and counterexamples named by each
checklist row.

| Identifier | Valid-variant scope |
| --- | --- |
| `PY-CAN-FLOW-001` | All named variants. |
| `PY-CAN-FLOW-002` | All named variants. |
| `PY-CAN-FLOW-003` | All named variants. |
| `PY-CAN-FLOW-004` | All named variants, including 3.11 notes. |
| `PY-CAN-FLOW-005` | All named variants. |
| `PY-CAN-FLOW-006` | Normal, `-O`, and `-OO` runs via the child-process context. |
| `PY-CAN-FLOW-007` | All named valid variants. |
| `PY-CAN-FLOW-008` | All named valid variants with unspecified-timing caveats respected. |
| `PY-CAN-FLOW-009` | Locked, unlocked, and thread-local cases on the standard GIL build; the free-threaded lane is excluded below. |
| `PY-CAN-FLOW-010` | `spawn`, `fork`, and `forkserver` with an importable top-level worker. |
| `PY-CAN-FLOW-011` | All named variants. |
| `PY-CAN-FLOW-012` | POSIX signal set on the default platform; the Windows set is excluded below. |
| `PY-CAN-FLOW-013` | All named valid variants; all live files parse under the selected interpreter. |

## Declared build contexts

- Default runtime.
- Child process / interpreter flags: for the `-O` and `-OO` runs of
  `PY-CAN-FLOW-006`.
- Multiprocessing start methods: `spawn`, `fork`, and `forkserver` for
  `PY-CAN-FLOW-010`.
- POSIX signals: for `PY-CAN-FLOW-012`.

## Dependency needs

None.

## Generated-source needs

None.

## Planned tests

None.

## Excluded variants

- The free-threaded lane of `PY-CAN-FLOW-009`: the installed build is standard
  and GIL-enabled.
- The Windows signal set of `PY-CAN-FLOW-012`: the local platform is Linux only.

## Required interfaces

Provide `Taskfile.yml` and `coverage.md` as defined in the ground rules.
