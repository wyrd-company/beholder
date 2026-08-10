---
relationships:
  references:
    - ../../ground-rules
    - reports/csharp/synthesis/checklist
---

# Project brief: types-and-values

## Purpose

An idiomatic library that declares the fundamental C# type kinds and exercises
value-type construction, copying, and nullability: classes, structs, interfaces,
enums, delegates, records, tuples, arrays, anonymous types, and nullable value and
reference forms.

## Exclusive directory

`fixtures/csharp/projects/types-and-values/`

## Difficulty

complex

## Assigned canonical identifiers

- CS-CAN-019 — fundamental declared type kinds.
- CS-CAN-020 — records and synthesized members.
- CS-CAN-021 — primary constructors.
- CS-CAN-022 — arrays and covariance.
- CS-CAN-023 — nullable value types and lifted operations.
- CS-CAN-024 — nullable reference annotations and flow state.
- CS-CAN-025 — tuples and element names.
- CS-CAN-026 — anonymous types.
- CS-CAN-031 — struct construction, defaults, copies, and mutation.
- CS-CAN-032 — enums.
- CS-CAN-058 — boxing, unboxing, and constrained value-type dispatch.

Valid variants named by these items are in scope, including record classes and
record structs, positional and nominal forms, class and struct primary
constructors, rectangular and jagged arrays, covariant reference-array
conversion, the enabled, disabled, annotations-only, warnings-only, and oblivious
nullable scopes, and the several tuple and anonymous-type shapes. A cross-assembly
consumer for nullable annotations and old-metadata consumption is internal to this
project.

## Declared build contexts

- Default `net10.0`.
- Earlier `LangVersion` selections where an assigned item is version-gated (for
  example record structs, non-record primary constructors, and parameterless and
  auto-default struct construction).

## Dependency needs

None. Standard library and project-local assemblies only.

## Generated-source needs

None.

## Planned tests

none

## Required interfaces

`Taskfile.yml` and `coverage.md` as defined in `fixtures/csharp/ground-rules.md`.
