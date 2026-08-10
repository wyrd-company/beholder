---
relationships:
  references:
    - ../../ground-rules
    - reports/csharp/synthesis/checklist
---

# Project brief: expressions-and-flow

## Purpose

An idiomatic library that exercises C# expression forms and intraprocedural flow:
definite assignment and reachability, the pattern and switch families,
initializers and collection expressions, deconstruction, indexing and ranges,
interpolation, checked arithmetic, expression trees, and query syntax.

## Exclusive directory

`fixtures/csharp/projects/expressions-and-flow/`

## Difficulty

complex

## Assigned canonical identifiers

- CS-CAN-059 — definite assignment, reachability, and variable scope.
- CS-CAN-060 — pattern matching and narrowing.
- CS-CAN-061 — switch selection and exhaustiveness.
- CS-CAN-062 — null operators and conditional access.
- CS-CAN-063 — object, collection, index, and `with` initialization.
- CS-CAN-064 — collection expressions and builders.
- CS-CAN-065 — deconstruction protocol.
- CS-CAN-066 — index, range, and slicing patterns.
- CS-CAN-067 — interpolated strings and handlers.
- CS-CAN-068 — checked and unchecked arithmetic.
- CS-CAN-069 — expression trees.
- CS-CAN-070 — LINQ query translation.

Valid variants named by these items are in scope, including the declaration,
type, constant, relational, logical, property, positional, recursive, list,
slice, `var`, discard, and null patterns; statement and expression switch forms;
object, collection, index, and `with` initializers; collection expressions over
arrays, spans, interfaces, and builder types; instance and extension
`Deconstruct`; copy-versus-view slicing; string, constant-string,
`FormattableString`, and custom-handler interpolation; local and project-wide
checked contexts; and delegate-versus-expression-tree conversion. The
unsupported-construct and non-exhaustiveness counterexamples those rows also list
are invalid source or diagnostic states and are not fixture content.

## Declared build contexts

- Default `net10.0`.
- Earlier `LangVersion` selections where an assigned item is version-gated (for
  example switch expressions, recursive and list patterns, collection
  expressions, implicit index initializers, and interpolated-string handlers).

## Dependency needs

None. Standard library and project-local assemblies only.

## Generated-source needs

None.

## Planned tests

none

## Required interfaces

`Taskfile.yml` and `coverage.md` as defined in `fixtures/csharp/ground-rules.md`.
