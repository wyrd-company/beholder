---
relationships:
  references:
    - ground-rules
    - overview
---

# Project brief: configuration-and-versions

## Purpose

A small package that pins the SDK and language version and demonstrates
configuration- and version-dependent source selection.

## Exclusive directory

`fixtures/dart/projects/configuration-and-versions/`

## Difficulty

routine

## Assigned coverage

- **DART-CAN-CFG-001** — Pinned SDK and language-version selection. Valid
  variants: identical syntax whose validity changes under a per-library older
  version; generated and dependency libraries kept at their own resolved
  versions; a feature used at or above its introduction version.
- **DART-CAN-CFG-002** — Per-library language override. Valid variants: a valid
  `// @dart=` override placed before code; a part that inherits its owning
  library's version; one pre-feature and one post-feature file.
- **DART-CAN-CFG-003** — Compile-time environment. Valid variants: `String`,
  `int`, and `bool` `fromEnvironment` and `hasEnvironment` in constant contexts;
  at least two `-D` values; defaults; the analyzer view versus the compiled
  value; interaction with conditional imports.
- **DART-CAN-CFG-004** — Assertions as configuration-dependent execution. Valid
  variants: an assertion with a message under enabled and disabled assertions; a
  disabled assertion whose side effects do not run; development versus
  production defaults; an explicit enable flag.

Invalid variants named by these items (malformed or too-new overrides, an
override after code, a part's own override, malformed environment values,
non-constant misuse, and a failed const-constructor assertion) are excluded by
the valid-source rule.

## Declared build contexts

- `vm-jit` (default), exercised in both assertions-enabled and
  assertions-disabled modes.
- `native-aot`, for the production assertion default.
- `js`, for the analyzer-view-versus-compiled-value observation of a define.

## Dependency needs

None. SDK libraries only.

## Generated-source needs

None.

## Planned tests

None. All coverage is created by library and executable source.

## Required interfaces

Provide `Taskfile.yml` and `coverage.md` as defined in the ground rules. `build`
resolves offline, analyzes all fixture-owned libraries, and compiles each
declared entry point for `native-aot` and `js`. `lint` runs `dart format` and
`dart analyze`. `test` runs `dart test` and succeeds with no test files.
