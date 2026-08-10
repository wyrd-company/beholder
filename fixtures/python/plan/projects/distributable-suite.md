---
relationships:
  references:
    - ../../ground-rules
    - ../../../../reports/python/synthesis/checklist
---

# distributable-suite

## Purpose

A multi-distribution packaging suite that builds, describes, discovers, installs,
and resolves several local distributions, covering distribution identity, build
isolation, metadata, layouts, dependencies and markers, extras, versions, wheel
tags, editable installs, package data, runtime metadata, environments, build
hooks, resolution, workspace and inline-script metadata, entry-point plugins and
launchers, and the packaging cross-feature interactions.

## Exclusive directory

`fixtures/python/projects/distributable-suite/`

## Difficulty

`complex`

## Assigned coverage

Primary owner of every distribution and build item, plus the entry-point and
generated-version items whose coverage requires real distributions. Represent all
valid variants named by each checklist row. Several local distributions live
inside this one project directory; none depends on another top-level project.

| Identifier | Valid-variant scope |
| --- | --- |
| `PY-CAN-PKG-001` | All named variants. |
| `PY-CAN-PKG-002` | Isolated build with a vendored constrained backend; offline. |
| `PY-CAN-PKG-003` | All named variants, including one backend-derived dynamic field. |
| `PY-CAN-PKG-004` | Source distribution and wheel with build-only and generated installed files. |
| `PY-CAN-PKG-005` | All named variants; flat and `src` layouts. |
| `PY-CAN-PKG-006` | All named marker and dependency variants. |
| `PY-CAN-PKG-007` | All named variants. |
| `PY-CAN-PKG-008` | All named version-specifier variants. |
| `PY-CAN-PKG-009` | Metadata and grammar alignment; interpreter-boundary variants are modeled through metadata, not by running other interpreters. |
| `PY-CAN-PKG-010` | Pure and native (CPython, `abi3`) tags on the local platform; free-threaded, Windows, and macOS tags are excluded below. |
| `PY-CAN-PKG-011` | All named editable-install variants. |
| `PY-CAN-PKG-012` | All named variants. |
| `PY-CAN-PKG-013` | All named variants. |
| `PY-CAN-PKG-014` | A modeled local direct reference and its recorded origin, without credentials. |
| `PY-CAN-PKG-015` | Two builds from equivalent normalized inputs and one documented countercase. |
| `PY-CAN-PKG-016` | All named variants using local isolated environments. |
| `PY-CAN-PKG-017` | A modeled externally-managed marker, without changing the host installation. |
| `PY-CAN-PKG-018` | All named hook and config-setting variants. |
| `PY-CAN-PKG-019` | An offline local candidate set with one satisfiable branch and one true conflict. |
| `PY-CAN-PKG-020` | Two distributions connected by a path or workspace dependency, resolved offline. |
| `PY-CAN-PKG-021` | An inline-script metadata block run offline through the installed runner. |
| `PY-CAN-PKG-022` | Tool configuration in `pyproject.toml` that changes at least one source-root, target, or inclusion setting. |
| `PY-CAN-DYN-005` | Declared, discovered, and loaded entry points in a private group after a local install. |
| `PY-CAN-DYN-006` | A declared script target and its generated wrapper after a local install. |
| `PY-CAN-INT-004` | A namespace package split across two local distributions with resources and entry points. |
| `PY-CAN-INT-005` | A generated version and source compared across source tree, sdist, wheel, and editable install. |

## Declared build contexts

- Default runtime.
- Packaging: `pip`, `wheel`, and `venv` with a vendored PyPA build backend and
  build isolation disabled, plus the installed `uv` runner for inline-script
  metadata and offline resolution. All installs and builds are offline.
- Native / C extension: for the native wheel tags of `PY-CAN-PKG-010`.
- Child process / interpreter flags: for launcher and environment cases.

## Dependency needs

One PyPA build backend distribution (for example a standards-based backend such
as setuptools or hatchling), acquired once by the implementor, then pinned,
vendored with source location, and shipped with its license and notice files and
the wheel or metadata artifacts needed for offline isolated builds. `pip`,
`wheel`, `venv`, and `uv` are installed system tools and are not vendored.

## Generated-source needs

A committed generated version or metadata artifact for `PY-CAN-INT-005` and
`PY-CAN-PKG-003`, produced by a `generate` task from committed generator input.
Required validation tasks do not regenerate it.

## Planned tests

None.

## Excluded variants

- Free-threaded wheel tags and Windows and macOS platform tags of
  `PY-CAN-PKG-010`, and interpreter-boundary runs of `PY-CAN-PKG-009`: the local
  toolchain is a single standard CPython build on Linux.

## Required interfaces

Provide `Taskfile.yml` and `coverage.md` as defined in the ground rules. `build`
byte-compiles the distributions and builds the native wheel representative;
`generate` produces the committed generated version artifact; installs and
resolution run offline against vendored and local artifacts.
