---
relationships:
  references:
    - ../../ground-rules
    - reports/csharp/synthesis/checklist
---

# Project brief: functions-and-async

## Purpose

An idiomatic library that exercises callable bodies and their generated state
machines: lambdas and captures, local functions, async methods and awaitables,
iterators, `foreach`, async streams, exception handling and filters, disposal,
and `lock` lowering.

## Exclusive directory

`fixtures/csharp/projects/functions-and-async/`

## Difficulty

complex

## Assigned canonical identifiers

- CS-CAN-071 — lambdas, anonymous methods, and capture.
- CS-CAN-072 — local functions.
- CS-CAN-073 — async methods and awaitable and task-like patterns.
- CS-CAN-074 — iterators and enumeration state machines.
- CS-CAN-075 — `foreach` enumeration pattern.
- CS-CAN-076 — async iterators and `await foreach`.
- CS-CAN-077 — exception handling and filters.
- CS-CAN-078 — `using` and disposal patterns.
- CS-CAN-079 — `lock` lowering.

Valid variants named by these items are in scope, including capturing,
noncapturing, and static lambdas and anonymous methods; static, captured,
iterator, async, and attributed local functions; `Task`, `Task<T>`,
`ValueTask<T>`, `async void`, custom awaitable, and custom task-like builder
forms; generic and nongeneric iterators with deferred exceptions and disposal;
enumeration of arrays, strings, struct-enumerator collections, pattern-only
types, extension `GetEnumerator`, and spans; async iteration with cancellation
and async disposal; typed catches and side-effecting filters; statement and
declaration `using` with reverse-order disposal and `await using`; and both the
`Monitor` and `System.Threading.Lock` lock-lowering forms. The
insufficient-lifetime, missing-shape, and invalid-context counterexamples those
rows also list are invalid source and are not fixture content.

## Declared build contexts

- Default `net10.0`.
- Earlier `LangVersion` selections where an assigned item is version-gated (for
  example static lambdas and default lambda parameters, static local functions,
  custom task-like builders, async streams, and the specialized `Lock` form).

## Dependency needs

None. Standard library and project-local assemblies only.

## Generated-source needs

None.

## Planned tests

none

## Required interfaces

`Taskfile.yml` and `coverage.md` as defined in `fixtures/csharp/ground-rules.md`.
