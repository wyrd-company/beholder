---
relationships:
  references:
    - ../../ground-rules
    - reports/csharp/synthesis/checklist
---

# Project brief: attributes-and-metadata

## Purpose

An idiomatic library that exercises attributes, compiler-recognized attribute
protocols, module initialization, caller information, and reflection over the
emitted metadata shape, including the non-invariant shapes the language and
runtime are free to vary.

## Exclusive directory

`fixtures/csharp/projects/attributes-and-metadata/`

## Difficulty

complex

## Assigned canonical identifiers

- CS-CAN-081 — attributes, targets, and generic attributes.
- CS-CAN-082 — compiler-recognized attributes and protocols.
- CS-CAN-083 — conditional call and attribute elision.
- CS-CAN-084 — obsolete, experimental, and tooling-only attributes.
- CS-CAN-085 — caller information and argument-expression capture.
- CS-CAN-086 — module initializers.
- CS-CAN-087 — reflection and emitted-versus-source shape.
- CS-CAN-088 — metadata encodings of source-only distinctions.
- CS-CAN-115 — symbols named outside executable expressions.
- CS-CAN-125 — synthesized names and shapes (asserted as non-invariant).
- CS-CAN-126 — ordering and timing latitude (asserted as non-invariant).
- CS-CAN-127 — optimization and scheduling latitude (asserted as non-invariant).
- CS-CAN-128 — platform numeric, memory, path, and ABI latitude on x64.

Valid variants named by these items are in scope, including attributes on every
declaration target and a generic attribute; recognized attributes for conditional
calls, caller info, nullable flow, required members, module initializers,
handlers, and collection and async builders; conditional call and attribute
elision under defined and undefined symbols; warning-level and error-level
obsolete, experimental, and editor-browsable references; caller member, file,
line, and argument-expression capture; several module initializers including one
in a referenced module; reflection over backing fields, accessors, state
machines, closures, anonymous, file-local, and top-level types; a second-assembly
consumer of dynamic, tuple-name, nullable, native-int, and ref encodings; and
`nameof`, `typeof`, and documentation `cref` with emitted XML documentation. The
unspecified-shape items are exposed only as semantic associations, never as exact
private names, orderings, or optimizations. The non-x64 architecture and
endianness observations of CS-CAN-128 are excepted in the overview as an
unavailable context.

## Declared build contexts

- Default `net10.0`.
- Documentation-generating build for CS-CAN-115 (XML documentation output).
- Earlier `LangVersion` selections where an assigned item is version-gated (for
  example generic attributes, caller-argument-expression capture, and module
  initializers).

## Dependency needs

None. Standard library and project-local assemblies only.

## Generated-source needs

None.

## Planned tests

none

## Required interfaces

`Taskfile.yml` and `coverage.md` as defined in `fixtures/csharp/ground-rules.md`.
