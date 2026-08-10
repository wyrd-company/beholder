---
relationships:
  references:
    - ../../ground-rules
    - reports/csharp/synthesis/checklist
---

# Project brief: generics-and-dispatch

## Purpose

An idiomatic library that exercises the generic type system and the resolution
and dispatch machinery: generic definitions and constraints, variance, interface
members including static-abstract ones, overload resolution, argument passing,
the built-in conversion set, and virtual and interface dispatch.

## Exclusive directory

`fixtures/csharp/projects/generics-and-dispatch/`

## Difficulty

complex

## Assigned canonical identifiers

- CS-CAN-027 — dynamic type and runtime binding.
- CS-CAN-041 — generic definitions, construction, inference, and arity.
- CS-CAN-042 — generic constraints.
- CS-CAN-043 — generic variance.
- CS-CAN-044 — static abstract and virtual interface members.
- CS-CAN-045 — `allows ref struct` anti-constraint.
- CS-CAN-050 — overload resolution.
- CS-CAN-051 — named, optional, and parameter-array arguments.
- CS-CAN-052 — by-reference arguments, returns, locals, and escape.
- CS-CAN-053 — built-in conversion taxonomy.
- CS-CAN-054 — method groups, lambdas, target typing, and natural function types.
- CS-CAN-055 — virtual, abstract, sealed, and override dispatch.
- CS-CAN-056 — interface implementation mapping.
- CS-CAN-057 — default interface implementations.

Valid variants named by these items are in scope, including the several
constraint kinds and the operations each enables, covariant and contravariant
interfaces and delegates, static-abstract implementations invoked through
constrained type parameters, array and span `params` and C# 13 params
collections, `ref`, `out`, `in`, and `ref readonly` passing, and resolved and
ambiguous default-interface diamonds. The ambiguity and inference-failure
counterexamples those rows also list are invalid source and are not fixture
content.

## Declared build contexts

- Default `net10.0`.
- Earlier `LangVersion` selections where an assigned item is version-gated (for
  example static abstract interface members, the `allows ref struct` anti-
  constraint, nullable and `default` constraints, natural function types, and
  params collections).

## Dependency needs

None. Standard library (including the runtime binder for dynamic) and
project-local assemblies only.

## Generated-source needs

None.

## Planned tests

none

## Required interfaces

`Taskfile.yml` and `coverage.md` as defined in `fixtures/csharp/ground-rules.md`.
