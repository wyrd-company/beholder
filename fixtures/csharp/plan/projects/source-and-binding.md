---
relationships:
  references:
    - ../../ground-rules
    - reports/csharp/synthesis/checklist
---

# Project brief: source-and-binding

## Purpose

A coherent, idiomatic multi-file, multi-assembly library and companion
application whose source exercises how C# spells declarations and resolves names:
lexical forms, preprocessing, namespace and using forms, compilation-unit and
entry-point shape, partial and file-local identity, and scoped lookup.

## Exclusive directory

`fixtures/csharp/projects/source-and-binding/`

## Difficulty

complex

## Assigned canonical identifiers

- CS-CAN-001 — lexical identity and literal forms.
- CS-CAN-002 — preprocessor, diagnostic, nullable, and source-mapping directives.
- CS-CAN-003 — compilation units, namespace forms, and open namespaces.
- CS-CAN-004 — using directives, aliases, root qualification, and lookup.
- CS-CAN-005 — global and SDK-generated implicit usings.
- CS-CAN-006 — top-level statements and synthesized entry point.
- CS-CAN-007 — explicit entry points and output kind.
- CS-CAN-008 — partial type identity.
- CS-CAN-009 — partial methods, properties, and indexers.
- CS-CAN-010 — file-local types.
- CS-CAN-011 — nested types, arity, and constructed identity.
- CS-CAN-046 — scoped lookup and shadowing.
- CS-CAN-048 — classic extension-method declaration and lookup.
- CS-CAN-049 — member lookup, hiding, and `new`.

Valid variants named by these items are in scope, including the valid forms of
implicit-usings enable and disable, library-versus-executable output kinds,
handwritten and generated partial parts, and import-driven extension lookup. The
error and ambiguity counterexamples those rows also list are invalid source and
are not fixture content.

## Declared build contexts

- Default `net10.0`.
- Earlier `LangVersion` selections where an assigned item is version-gated (for
  example raw and UTF-8 literals, enhanced `#line`, file-scoped namespaces,
  alias-any-type, file-local types, and C# 13 partial members).

## Dependency needs

None. Standard library and project-local assemblies only.

## Generated-source needs

The SDK-supplied implicit-usings compilation input is retained as build input for
CS-CAN-005. No committed generated source and no `generate` task.

## Planned tests

none

## Required interfaces

`Taskfile.yml` and `coverage.md` as defined in `fixtures/csharp/ground-rules.md`.
