---
relationships:
  references:
    - ../../ground-rules
    - ../../../../reports/python/synthesis/checklist
---

# portability-layer

## Purpose

A portability layer over filesystem, locale, time, and process APIs, covering
path representations, case-sensitive import behavior, path identity, text
encoding and newlines, environment and startup configuration, time zones and
clocks, subprocesses, platform-guarded APIs, hash randomization, and
architecture and build flags.

## Exclusive directory

`fixtures/python/projects/portability-layer/`

## Difficulty

`complex`

## Assigned coverage

Primary owner of every platform, filesystem, locale, and environment item.
Represent the valid Linux-lane variants named by each checklist row. Windows and
macOS lanes are excluded below; all guarded files still parse under the selected
interpreter.

| Identifier | Valid-variant scope |
| --- | --- |
| `PY-CAN-PLAT-001` | Text and `pathlib` paths and a guarded POSIX non-UTF-8 bytes name. |
| `PY-CAN-PLAT-002` | Differently-cased candidates asserted on the case-sensitive local filesystem. |
| `PY-CAN-PLAT-003` | Lexical, absolute, resolved, same-file, and guarded symlink and hard-link cases. |
| `PY-CAN-PLAT-004` | Explicit UTF-8, universal-newline input, and subprocess locale and UTF-8-mode variants. |
| `PY-CAN-PLAT-005` | Controlled child interpreters varying hash, UTF-8, warning, path, isolated, and application settings. |
| `PY-CAN-PLAT-006` | Aware and naive values, an ambiguous fold, monotonic versus wall clock, and non-exact timestamp precision. |
| `PY-CAN-PLAT-007` | Argument-vector, text and bytes I/O, environment, exit-code, and guarded shell-command cases. |
| `PY-CAN-PLAT-008` | POSIX and guarded Windows imports behind one shared wrapper with an explicit unsupported path. |
| `PY-CAN-PLAT-009` | Child runs under random, fixed, and disabled hash seed with deterministic sorted output. |
| `PY-CAN-PLAT-010` | Introspected pointer width, byte order, ABI, and the standard build's actual flags. |

## Declared build contexts

- Default runtime.
- Child process / interpreter flags: for startup, hash-seed, and locale child
  runs of `PY-CAN-PLAT-004`, `PY-CAN-PLAT-005`, and `PY-CAN-PLAT-009`.
- POSIX filesystem: symlinks, hard links, and bytes names for
  `PY-CAN-PLAT-001` and `PY-CAN-PLAT-003`.

## Dependency needs

None. The system time-zone database is present and needs no vendored
distribution.

## Generated-source needs

None.

## Planned tests

None.

## Excluded variants

- Windows and macOS lane variants across `PY-CAN-PLAT-001` through
  `PY-CAN-PLAT-010`: the local platform is Linux only. Free-threaded and JIT
  flag values of `PY-CAN-PLAT-010` are recorded as the standard build's actual
  (disabled) flags.

## Required interfaces

Provide `Taskfile.yml` and `coverage.md` as defined in the ground rules.
