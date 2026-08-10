---
relationships:
  references:
    - ground-rules
    - overview
---

# Project brief: packages-and-workspace

## Purpose

A pub workspace of member packages that exercises manifests, layout and roots,
dependency sources, version solving, overrides, workspaces, tests, and
executables.

## Exclusive directory

`fixtures/dart/projects/packages-and-workspace/`

## Difficulty

complex

## Assigned coverage

- **DART-CAN-PKG-001** — Manifest and package namespace: a valid `pubspec.yaml`
  with name, version, SDK constraint, regular and dev dependencies, and package
  identity independent of directory name; private and unpublished metadata; a
  `package:` URI rooted at `lib/`.
- **DART-CAN-PKG-002** — Package layout and roots: public `lib/`, implementation
  `lib/src/`, multiple `bin/` entry points, `test/`, `example/`, and `tool/` as
  applicable; a public façade re-export; an `executables:` mapping; different
  reachability per root.
- **DART-CAN-PKG-003** — Dependency source kinds: hosted and path dependencies;
  a dev-dependency used from `test`; path identity.
- **DART-CAN-PKG-004** — Version solving, lockfile, and package configuration: a
  preserved lockfile and generated package configuration, with imports resolved
  through the latter; compatible transitive constraints; per-package language
  version; the application-versus-library lockfile convention.
- **DART-CAN-PKG-005** — Overrides and resolution overlays: a root override that
  substitutes a hosted dependency with a local path, and a separate override
  configuration if the SDK supports it; root-only effect.
- **DART-CAN-PKG-006** — Pub workspaces: at least two member packages sharing one
  resolution, lockfile, and package configuration, with cross-member `package:`
  imports; a conditional import in a member.
- **DART-CAN-PKG-007** — Tests as entry libraries: `package:test` discovery,
  groups, setup and teardown, synchronous and asynchronous tests, platform
  selectors, tags, skip, timeout, and `dart_test.yaml`; test import by
  `package:` URI.
- **DART-CAN-PKG-008** — Executables and compiled artifacts: a named or
  dependency executable invoked with `dart run`, arguments, and at least one
  supported snapshot or binary form; an operating-system and architecture
  target.
- **DART-CAN-PKG-009** — Declared platform support: a `platforms:` set where the
  pinned schema supports it, reconciled with platform libraries, conditional
  imports, tests, and build targets.

The Git dependency-source variant of DART-CAN-PKG-003 is excluded as an
unavailable context; hosted and path sources provide the required
package-resolution coverage. Flutter-only variants of DART-CAN-PKG-001,
DART-CAN-PKG-002, and DART-CAN-PKG-009 are excluded; no Flutter SDK is installed.

## Declared build contexts

- `vm-jit` (default).
- `native-aot`, for the compiled executable artifact of DART-CAN-PKG-008.

## Dependency needs

- `package:test` (dev), for the test-framework coverage of DART-CAN-PKG-007.
- One small pinned hosted package, for the hosted dependency source of
  DART-CAN-PKG-003, the version solving of DART-CAN-PKG-004, and the
  hosted-to-path substitution of DART-CAN-PKG-005.

The implementor pins, fetches once, vendors, and preserves the license and
notice files for each, so that `build`, `lint`, and `test` resolve offline.

## Generated-source needs

None. The generated package configuration is produced by offline pub resolution,
not by a fixture generator.

## Planned tests

Planned. Test source creates the coverage for DART-CAN-PKG-007 (test discovery,
grouping, lifecycle, selectors, tags, skip, timeout, and `dart_test.yaml`
configuration) and the test-root reachability of DART-CAN-PKG-002 (a `test/`
entry library, test-only reachability, and private access denied across the
package library boundary).

## Required interfaces

Provide `Taskfile.yml` and `coverage.md` as defined in the ground rules. `build`
resolves offline, analyzes all fixture-owned libraries, and compiles the
executable under `native-aot`. `lint` runs `dart format` and `dart analyze`.
`test` runs `dart test` over the planned test source.
