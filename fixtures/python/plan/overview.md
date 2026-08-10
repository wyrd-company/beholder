---
relationships:
  references:
    - ../ground-rules
    - ../../../reports/python/synthesis/checklist
---

# Python fixture corpus plan overview

## Selected interpreter

The corpus targets the only interpreter installed in the local toolchain:

- **Interpreter:** CPython 3.14.6, 64-bit x86-64 Linux.
- **Build:** standard, GIL-enabled (`Py_GIL_DISABLED = 0`); little-endian;
  64-bit pointers.
- **C toolchain:** GCC 15.2 with CPython development headers present, enabling
  native extension, stable-ABI, and `ctypes` foreign-function coverage.
- **Packaging toolchain:** `pip` and `wheel` with the standard-library `venv`;
  offline installs succeed only against local artifacts. A PyPA build backend is
  not installed and is a vendored dependency where a project needs an isolated
  build.

CPython 3.14.6 is ahead of the checklist's Python 3.13 baseline. All language
features introduced from 3.8 through 3.13 are present on this interpreter and are
demonstrated as valid source under it. Behavior that depends on running an
earlier interpreter, an absent interpreter build, an alternative implementation,
or another operating system is an unavailable conditional context, recorded in
the exceptions. Python 3.14-only behavior that the checklist marks as a research
gap is excluded and is not resolved by this plan.

## Build contexts

The default context carries every language item unless a row below applies. A
project declares only the additional contexts its assigned coverage requires.

| Context | Basis | Availability |
| --- | --- | --- |
| Default runtime | CPython 3.14.6, x86-64 Linux, standard GIL build | Installed |
| Native / C extension | GCC 15.2 + CPython headers; `ctypes`; stable ABI | Installed |
| Packaging | `pip` + `wheel` + `venv`, offline, vendored PyPA backend | Installed |
| Child process / interpreter flags | `sys.executable` with varied flags and env | Installed |
| Multiprocessing start methods | `spawn`, `fork`, `forkserver` on Linux | Installed |
| Isolated diagnostics process | audit, unraisable, thread, and warning hooks | Installed |

The following contexts are unavailable locally and bound the exceptions:
historical interpreters 3.8–3.13, free-threaded and JIT interpreter builds,
alternative implementations, Windows and macOS, and any externally installed
tool that is not present (`pytest`, a type checker, a linter or formatter binary,
Cython, a freezer, or a documentation builder).

## Projects

Fourteen projects cover the in-scope checklist. Each occupies one exclusive
directory under `fixtures/python/projects/`. Sizes and shapes differ by natural
Python boundaries rather than equal packets.

| Slug | Purpose | Difficulty |
| --- | --- | --- |
| `plugin-content-library` | Pluggable content library assembled from regular and namespace packages with dynamic loading and custom import machinery | complex |
| `layered-settings-resolver` | Layered settings resolver exercising binding, scope, and dynamic namespaces | routine |
| `async-task-toolkit` | Cooperative task toolkit of callables, decorators, generators, and coroutines | complex |
| `record-modeling-kit` | Record and entity modeling kit built on classes, descriptors, dataclasses, and enumerations | complex |
| `measurement-types` | Quantity and container value library implementing the data-model protocols | complex |
| `typed-toolkit` | Statically typed utility library exercising the annotation and typing surface | complex |
| `job-runner` | Resilient job runner covering control flow, exceptions, concurrency, and resources | complex |
| `markup-report-toolkit` | Text and markup report toolkit exercising source text, literals, and compilation | complex |
| `dispatch-framework` | Runtime dispatch and registry framework for callables and handlers | complex |
| `distributable-suite` | Multi-distribution packaging suite covering build, metadata, and environment semantics | complex |
| `native-extension-kit` | Native-acceleration kit with C extensions, FFI, and generated modules | complex |
| `portability-layer` | Portability layer over filesystem, locale, time, and process APIs | complex |
| `diagnostics-toolkit` | Diagnostics, warnings, and testing toolkit | complex |
| `feature-integration-suite` | Combined cross-feature integration scenarios | complex |

## Primary assignment

Every in-scope canonical identifier has exactly one primary project. Ranges are
inclusive and contiguous within a checklist section.

| Project | Primary-owned canonical identifiers |
| --- | --- |
| `plugin-content-library` | `PY-CAN-MOD-001` – `PY-CAN-MOD-019` |
| `layered-settings-resolver` | `PY-CAN-BIND-001` – `PY-CAN-BIND-011` |
| `async-task-toolkit` | `PY-CAN-CALL-001` – `PY-CAN-CALL-015` |
| `record-modeling-kit` | `PY-CAN-OBJ-001` – `PY-CAN-OBJ-020` |
| `measurement-types` | `PY-CAN-PROT-001` – `PY-CAN-PROT-013` |
| `typed-toolkit` | `PY-CAN-TYPE-001` – `PY-CAN-TYPE-022` |
| `job-runner` | `PY-CAN-FLOW-001` – `PY-CAN-FLOW-013` |
| `markup-report-toolkit` | `PY-CAN-SRC-001` – `PY-CAN-SRC-014` |
| `dispatch-framework` | `PY-CAN-DYN-001` – `PY-CAN-DYN-004`, `PY-CAN-DYN-007` – `PY-CAN-DYN-013` |
| `distributable-suite` | `PY-CAN-PKG-001` – `PY-CAN-PKG-022`; `PY-CAN-DYN-005`; `PY-CAN-DYN-006`; `PY-CAN-INT-004`; `PY-CAN-INT-005` |
| `native-extension-kit` | `PY-CAN-NATIVE-001` – `PY-CAN-NATIVE-007`; `PY-CAN-NATIVE-010` |
| `portability-layer` | `PY-CAN-PLAT-001` – `PY-CAN-PLAT-010` |
| `diagnostics-toolkit` | `PY-CAN-DIAG-002` – `PY-CAN-DIAG-009`; `PY-CAN-DIAG-013` – `PY-CAN-DIAG-022` |
| `feature-integration-suite` | `PY-CAN-INT-001`; `PY-CAN-INT-002`; `PY-CAN-INT-003`; `PY-CAN-INT-006`; `PY-CAN-INT-007`; `PY-CAN-INT-008` |

This assigns 206 of the 216 canonical identifiers. The remaining 10 identifiers
are exceptions below.

## Exceptions

Each excluded identifier belongs to exactly one allowed exception category:
invalid-only item or variant, unavailable conditional context, or research gap.

| Identifier | Category | Reason |
| --- | --- | --- |
| `PY-CAN-SRC-015` | invalid-only item | Its obligation is invalid-versus-nearest-valid syntax contrast; the valid halves already belong to their feature items (functions, generators, coroutines, loops, binding). |
| `PY-CAN-DIAG-001` | invalid-only item | Compilation-failure phases are entirely parser and compiler rejection; no valid source demonstrates them. |
| `PY-CAN-DIAG-010` | unavailable conditional context | `pytest` is not installed; stdlib `unittest` and `doctest` carry required test-infrastructure coverage. |
| `PY-CAN-DIAG-011` | unavailable conditional context | `pytest` is not installed. |
| `PY-CAN-DIAG-012` | unavailable conditional context | `pytest` is not installed. |
| `PY-CAN-DYN-014` | unavailable conditional context | No documentation builder is installed to resolve entity references. |
| `PY-CAN-NATIVE-008` | unavailable conditional context | No Python-to-extension transpiler (Cython) is installed. |
| `PY-CAN-NATIVE-009` | unavailable conditional context | No application freezer is installed. |
| `PY-CAN-NATIVE-011` | unavailable conditional context | No alternative Python implementation is installed. |
| `PY-CAN-NATIVE-012` | unavailable conditional context | The high-level subinterpreter API is unstable on this interpreter and overlaps research gap `PY-GAP-003`. |

### Excluded variants within in-scope items

These items keep their primary project; the listed variants are unavailable
conditional contexts or invalid source and are not planned.

- Historical-interpreter runs and pre-version parse-failure variants across all
  version-gated items (for example `PY-CAN-SRC-005`, `PY-CAN-SRC-009`,
  `PY-CAN-TYPE-004`): only CPython 3.14.6 is provisioned.
- Free-threaded and JIT run variants (for example `PY-CAN-FLOW-009`,
  `PY-CAN-NATIVE-004`, `PY-CAN-PKG-010`, `PY-CAN-PLAT-010`): the installed build
  is standard and GIL-enabled; introspective items record the build's actual
  flags.
- Windows and macOS lane variants across the platform items
  (`PY-CAN-PLAT-001` – `PY-CAN-PLAT-010`) and cross-platform notes elsewhere:
  the local platform is Linux only.
- The `pkg_resources` contrast variant of `PY-CAN-MOD-004`: its stdlib
  `pkgutil` and `__path__` mechanisms carry the item without a setuptools
  dependency in a non-packaging project.
- Coverage-tool arc variants of `PY-CAN-DIAG-015`: no coverage tool is
  installed; stdlib tracing, profiling, and `sys.monitoring` carry the item.

### Research gaps

The checklist research gaps `PY-GAP-001` through `PY-GAP-010` are excluded and
not resolved by this plan. They cover Python 3.14 deferred-annotation semantics,
template strings, the standard subinterpreter API, free-threaded and JIT
guarantees, lock-file and dependency-group standards, the latest typing-spec
extensions, arbitrary dynamic-import target inference, framework-specific runtime
synthesis, alternative-runtime and exotic-platform expectations, and exact
cross-version diagnostic boundaries.

## Independence audit

- Each project occupies one exclusive directory under
  `fixtures/python/projects/` matching its slug. No two projects share a
  directory.
- No project imports, builds, or otherwise depends on another top-level fixture
  project. Every project contains the local packages and distributions its
  coverage requires within its own directory.
- Multi-distribution coverage (namespace split, workspace, and local dependency
  items in `distributable-suite`) is satisfied by several distributions inside
  that one project directory, not by a dependency on another project.
- The native context is used independently by `native-extension-kit`,
  `plugin-content-library` (for its extension-module import representative), and
  `feature-integration-suite` (for its native-accelerator fallback); each
  compiles its own artifact and shares no source with the others.
- Only `distributable-suite` names a third-party dependency (a vendored PyPA
  build backend). All other projects use the standard library, project-local
  code, or the installed C toolchain.
