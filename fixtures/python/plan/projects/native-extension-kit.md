---
relationships:
  references:
    - ../../ground-rules
    - ../../../../reports/python/synthesis/checklist
---

# native-extension-kit

## Purpose

A native-acceleration kit with a C extension module, limited-API and stable-ABI
artifacts, reference ownership and exception translation, native thread state, a
`ctypes` foreign-function boundary, generated modules and stubs, runtime code
generation, and an executable zip application.

## Exclusive directory

`fixtures/python/projects/native-extension-kit/`

## Difficulty

`complex`

## Assigned coverage

Primary owner of the native and generated-code items below. Represent all valid
variants named by each checklist row. Free-threaded and subinterpreter
declaration variants are excluded below; transpilation, freezing, and
alternative-implementation items are excluded in the overview.

| Identifier | Valid-variant scope |
| --- | --- |
| `PY-CAN-NATIVE-001` | Single- and multi-phase initialization and per-module state; the subinterpreter declaration variant is excluded. |
| `PY-CAN-NATIVE-002` | Limited-API and CPython-version artifacts and their tags. |
| `PY-CAN-NATIVE-003` | All named ownership and error-translation variants. |
| `PY-CAN-NATIVE-004` | GIL release and reacquire around blocking work on the standard build; the free-threaded declaration is excluded. |
| `PY-CAN-NATIVE-005` | All named `ctypes` variants against a locally built native library, including the isolated deliberately-wrong declaration. |
| `PY-CAN-NATIVE-006` | Committed generated `.py` and `.pyi` from retained input, with freshness and provenance. |
| `PY-CAN-NATIVE-007` | All named runtime code-generation variants. |
| `PY-CAN-NATIVE-010` | All named zip-application variants. |

## Declared build contexts

- Default runtime.
- Native / C extension: GCC with CPython headers for the extension module,
  limited-API and stable-ABI artifacts, and the `ctypes` native library.

## Dependency needs

None. The C toolchain is an installed system tool, not a vendored distribution.

## Generated-source needs

Committed generated `.py` and `.pyi` modules for `PY-CAN-NATIVE-006`, produced by
a `generate` task from committed generator input. Required validation tasks do
not regenerate them. Runtime code generation for `PY-CAN-NATIVE-007` occurs at
runtime and commits no generated file.

## Planned tests

None.

## Excluded variants

- The subinterpreter declaration variant of `PY-CAN-NATIVE-001` and the
  free-threaded declaration variant of `PY-CAN-NATIVE-004`: the installed build
  is standard and GIL-enabled and the high-level subinterpreter API is unstable.

## Required interfaces

Provide `Taskfile.yml` and `coverage.md` as defined in the ground rules. `build`
compiles the extension, limited-API, and `ctypes` artifacts; `generate` produces
the committed generated modules.
