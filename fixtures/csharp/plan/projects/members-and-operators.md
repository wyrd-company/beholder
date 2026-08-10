---
relationships:
  references:
    - ../../ground-rules
    - reports/csharp/synthesis/checklist
---

# Project brief: members-and-operators

## Purpose

An idiomatic library whose types carry the full range of C# member kinds and
member-level contracts: fields and their modifiers, properties and indexers,
events, constructors and finalizers, required members, accessibility across
assembly boundaries, delegates, and user-defined operators and conversions.

## Exclusive directory

`fixtures/csharp/projects/members-and-operators/`

## Difficulty

routine

## Assigned canonical identifiers

- CS-CAN-033 — delegates and callable identity.
- CS-CAN-034 — fields, constants, readonly, volatile, and static initialization.
- CS-CAN-035 — properties and indexers.
- CS-CAN-036 — events.
- CS-CAN-037 — constructors, finalizers, and initialization order.
- CS-CAN-038 — required-member contracts.
- CS-CAN-039 — accessibility and effective accessibility.
- CS-CAN-040 — user-defined operators and conversions.

Valid variants named by these items are in scope, including static, closed-
instance, method-group, lambda, and boxed-struct delegate targets and multicast
lists; auto, full, expression-bodied, init-only, required, and ref-return
members; field-like and custom events; instance, static, base, and this
constructor chains with a finalizer; and arithmetic, comparison, conversion,
lifted, and checked user operators. The friend-versus-non-friend accessibility
demonstration uses a second assembly internal to this project.

## Declared build contexts

- Default `net10.0`.
- Earlier `LangVersion` selections where an assigned item is version-gated (for
  example init-only and required members, partial properties and indexers, and
  checked user-defined operators).

## Dependency needs

None. Standard library and project-local assemblies only.

## Generated-source needs

None.

## Planned tests

none

## Required interfaces

`Taskfile.yml` and `coverage.md` as defined in `fixtures/csharp/ground-rules.md`.
