---
relationships:
  references:
    - ground-rules
    - overview
---

# Project brief: code-generation

## Purpose

A code-generation package that exercises part-based and standalone-library
generation, builder configuration, generated-source formatting, config-driven
generation, and the stable no-macros constraint.

## Exclusive directory

`fixtures/dart/projects/code-generation/`

## Difficulty

complex

## Assigned coverage

- **DART-CAN-BUILD-001** — Part-based generation: a pinned annotation-triggered
  builder that emits a `.g.dart` part used by handwritten code and sharing
  private names; the annotation-to-output relationship; a committed generated
  file.
- **DART-CAN-BUILD-002** — Standalone-library generation: one generator output
  imported as its own library, distinct from a part output; visibility
  differences between generated public and private declarations.
- **DART-CAN-BUILD-003** — Builder configuration and asset graph: a `build.yaml`
  or equivalent that narrows inputs or changes an option, consuming at least one
  non-Dart input; `generate_for`; a transitive builder.
- **DART-CAN-BUILD-004** — Generated-source and formatting conventions:
  preserved generated headers and ignores and language-version-aware formatting
  without semantic change; a formatter style across language versions.
- **DART-CAN-BUILD-005** — External config-driven generators: one maintained
  generator whose provenance is a configuration or input file rather than an
  annotation, producing a generated library or part.
- **DART-CAN-BUILD-006** — Stable macros negative case: the corpus does not
  require Dart macros; `build_runner` remains the stable generation mechanism.

The optional rejected or obsolete macro sample named by DART-CAN-BUILD-006 is
excluded as invalid or obsolete source. The required negative constraint is
demonstrated by using `build_runner` as the stable mechanism.

## Declared build contexts

- `vm-jit` (default).

## Dependency needs

- A pinned `build`, `build_config`, `build_runner`, and `source_gen` toolchain,
  for the annotation-triggered part and standalone-library generation of
  DART-CAN-BUILD-001 through DART-CAN-BUILD-003.
- One config- or input-file-driven generator package, for DART-CAN-BUILD-005.

The implementor pins, fetches once, vendors, and preserves the license and
notice files for each, so that `build`, `lint`, and `test` resolve offline.

## Generated-source needs

Required. The project commits both generator source and its generated outputs: a
`.g.dart` part output, a standalone generated library, and the config-driven
generator output. A `generate` task invokes the generator and is the only task
that writes generated source. `build`, `lint`, and `test` never regenerate.

## Planned tests

None. All coverage is created by generator source, the annotations and
configuration that drive it, and the committed generated output.

## Required interfaces

Provide `Taskfile.yml` and `coverage.md` as defined in the ground rules, plus a
`generate` task as described above. `build` resolves offline, analyzes all
fixture-owned libraries including the committed generated output, and compiles
the declared entry point. `lint` runs `dart format` and `dart analyze`. `test`
succeeds with no test files and needs no test-framework dependency.
