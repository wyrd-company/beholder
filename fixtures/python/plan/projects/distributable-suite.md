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
| `PY-CAN-PKG-002` | PEP 517 isolated build whose constrained backend is absent from the caller environment and supplied offline from a vendored wheelhouse; isolation stays enabled. |
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
| `PY-CAN-DYN-005` | Declared, discovered, and loaded entry points in a private group after a local install. |
| `PY-CAN-DYN-006` | A declared script target and its generated wrapper after a local install. |
| `PY-CAN-INT-004` | A namespace package split across two local distributions with resources and entry points. |
| `PY-CAN-INT-005` | A generated version and source compared across source tree, sdist, wheel, and editable install. |

## Declared build contexts

- Default runtime.
- Packaging: `pip`, `wheel`, and `venv` with the vendored, pinned build backend
  supplied through a local wheelhouse so PEP 517 build isolation stays enabled
  and downloads nothing, plus the installed `uv` runner for inline-script
  metadata and offline resolution. All installs and builds are offline.
- Native / C extension: for the native wheel tags of `PY-CAN-PKG-010`.
- Child process / interpreter flags: for launcher and environment cases.

## Dependency needs

The `setuptools` distribution, pinned to `setuptools==84.0.0`, is the only
third-party build dependency. The implementor acquires it once from PyPI (the
network exception) and vendors it with its source location, its MIT license and
notice files, and its single pure-Python wheel, placed in a local wheelhouse so
the isolated build resolves it with `--no-index`. The vendored wheel is pinned by
its published SHA-256
`51a52592b3b99e102b609654876bd65f19f999935166d1352678931132b0c670`, and the
source distribution by SHA-256
`f4695c21257f0d9b537ec2692c941d02ee143b7cc1276941349a546573b2ef73`
(PyPI release <https://pypi.org/project/setuptools/84.0.0/>).

This fixture declares no setup requirements. In upstream `setuptools.build_meta`
v84.0.0, the wheel and sdist build-requirement hooks initialize their requirement
list empty and add only requirements raised from project-declared setup
requirements, and the editable hook delegates to the wheel hook
(`setuptools/build_meta.py` lines 265-310 and 435-438). Because this fixture
declares no setup requirements, every PEP 517 `get_requires_for_build_wheel`,
`get_requires_for_build_sdist`, and `get_requires_for_build_editable` hook
returns an empty list. No unnamed, hook-returned build dependency is acquired or
authorized; `setuptools==84.0.0` remains the sole third-party build dependency.

`pip`, `wheel`, `venv`, and `uv` are installed system tools and are not vendored.
Local feasibility is confirmed as far as this plan permits: `setuptools==84.0.0`
resolves for this interpreter from the index (metadata only), and `pip` supports
offline isolated builds through `--no-index --find-links`.

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
- `PY-CAN-PKG-022` is not assigned to this project: configuring test, typing, and
  lint tools against their per-tool schemas needs those tools, none of which is
  installed, so the context is an unavailable conditional context (see the
  overview exceptions).

## Required interfaces

Provide `Taskfile.yml` and `coverage.md` as defined in the ground rules. `build`
byte-compiles the distributions and builds the native wheel representative;
`generate` produces the committed generated version artifact; installs and
resolution run offline against vendored and local artifacts.
