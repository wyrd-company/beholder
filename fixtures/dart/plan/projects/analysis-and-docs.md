---
relationships:
  references:
    - ground-rules
    - overview
---

# Project brief: analysis-and-docs

## Purpose

A documented and annotated library that exercises analyzer configuration,
suppression and generated markers, documentation references, metadata, contract
annotations, automated fixes, and synthetic entities.

## Exclusive directory

`fixtures/dart/projects/analysis-and-docs/`

## Difficulty

complex

## Assigned coverage

- **DART-CAN-DIAG-002** — Analyzer configuration: `analysis_options.yaml`
  include chaining, lints, a severity change, exclusion, and strict casts,
  inference, and raw types; a generated-file exclusion; a compiler-accepted,
  analyzer-only violation.
- **DART-CAN-DIAG-003** — Suppression and generated markers: next-line and
  file-wide ignore comments plus a generated header; a suppressed lint versus an
  unsuppressible language error.
- **DART-CAN-DIAG-004** — Documentation references and doc-only scope: `///` and
  block docs with resolvable bracket references, private, imported, and prefixed
  references, and `@docImport` where supported; `@nodoc`; a doc-only versus a
  code dependency.
- **DART-CAN-DIAG-005** — Metadata targets and constants: built-in and custom
  const annotations on a library, directive, type, member, parameter, type
  parameter, variable, and type use; a const-variable reference; repeated
  annotations; a generic annotation if supported.
- **DART-CAN-DIAG-006** — Analyzer contract annotations: representative
  `package:meta` visibility and behavior contracts with compliant uses across a
  library and package boundary.
- **DART-CAN-DIAG-007** — Automated fixes and migrations: a deprecated API with
  a fixable use, and data-driven fix metadata only where the library supplies
  it; a preview versus an applied fix.
- **DART-CAN-DIAG-008** — Synthetic and anonymous entities: observable implicit
  constructors, interfaces, and supertypes, field accessors, record getters,
  enum members, mixin application classes, `noSuchMethod` forwarders, closures,
  and unnamed extensions; a named versus an anonymous mixin application.

Violating uses named by DART-CAN-DIAG-006 and synthesized display names named by
DART-CAN-DIAG-008 are recorded as observations, not asserted; the corpus
represents the valid, compliant side.

## Declared build contexts

- `vm-jit` (default).

## Dependency needs

- `package:meta`, for the analyzer contract annotations of DART-CAN-DIAG-006.

The implementor pins, fetches once, vendors, and preserves the license and
notice files, so that `build`, `lint`, and `test` resolve offline.

## Generated-source needs

None. The generated-marker demonstration of DART-CAN-DIAG-003 uses a committed
file bearing the standard generated header and ignore comments; no generator is
invoked.

## Planned tests

None. All coverage is created by library source and analyzer configuration.

## Required interfaces

Provide `Taskfile.yml` and `coverage.md` as defined in the ground rules. `build`
resolves offline and analyzes all fixture-owned libraries. `lint` runs `dart
format` and `dart analyze` under the project's analyzer configuration. `test`
runs `dart test` and succeeds with no test files.
