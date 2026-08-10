---
relationships:
  references:
    - ground-rules
    - overview
---

# Project brief: null-safety-flow-patterns

## Purpose

A control-flow library that exercises null safety, flow promotion, statements
and switches, the pattern taxonomy and its contexts, exceptions, and
collections.

## Exclusive directory

`fixtures/dart/projects/null-safety-flow-patterns/`

## Difficulty

complex

## Assigned coverage

- **DART-CAN-NULL-001** — Nullable and non-nullable types: `T`, `T?`, `Null`,
  nullable type arguments, parameters, returns, fields, and overrides; nullable
  bounds; nullability override compatibility.
- **DART-CAN-NULL-002** — Flow promotion: promotion through tests, null checks,
  assignments, returns, throws, patterns, and branches; eligible private final
  field promotion; a pre-field-promotion language version.
- **DART-CAN-NULL-003** — Definite assignment and `late`: path-complete
  assignment without `late`, lazy `late` initialization, and `late final`;
  top-level, local, static, and instance cases; constructor paths.
- **DART-CAN-NULL-004** — Null-aware selection and fallback: `?.`, null-aware
  indexing, `??`, `??=`, postfix `!`, and whole-chain short-circuiting; side
  effects skipped on null; a successful `!`.
- **DART-CAN-FLOW-001** — Statements, loops, labels, and reachability: loop
  forms, nested labels, `break`, `continue`, `return`, `throw`, and a `Never`
  call; a continue-to-switch label.
- **DART-CAN-FLOW-002** — Switch statements, expressions, and exhaustiveness:
  constant and pattern cases, guards, expression and statement forms, and enum,
  Boolean, nullable, sealed, and record scrutinees; wildcard restoration;
  empty-case grouping; logical-or; a labeled continue.
- **DART-CAN-FLOW-003** — Pattern taxonomy: variable, identifier, wildcard,
  constant, relational, cast, null-check and null-assert, list and rest, map,
  record, object, logical-and and logical-or, and parenthesized patterns with
  guards; nested matches.
- **DART-CAN-FLOW-004** — Pattern contexts and scope: declaration, assignment,
  `if-case`, switch, and `for` destructuring contexts; guard and branch scopes;
  closures capturing bindings; per-iteration bindings.
- **DART-CAN-FLOW-005** — Exceptions and stack flow: `throw`, `rethrow`, `on`,
  typed and untyped `catch`, a stack-trace parameter, and `finally`; catch
  order; a non-`Exception` object; `finally` overriding a return or throw.
- **DART-CAN-COLL-001** — Collection literal kinds and inference: mutable and
  const list, set, and map literals with contextual and explicit type arguments;
  `{}` defaulting to a map; empty and heterogeneous literal inference; an
  unmodifiable const collection; spread inference.
- **DART-CAN-COLL-002** — Collection control flow and spreads: collection `if`
  and `else`, `if-case`, `for` with patterns, `...`, and `...?` in lists, sets,
  and maps; a null source skip; nested forms; const restrictions.
- **DART-CAN-COLL-003** — Null-aware collection elements: the null-eliding
  element and map key and value forms supported by the pinned SDK, contrasted
  with nullable inclusion and interaction with spreads and control flow.

Pattern getter evaluation order and count are exercised only for the guarantees
the specification states; side-effecting accessors are recorded as observations,
per DART-GAP-009.

## Declared build contexts

- `vm-jit` (default).

## Dependency needs

None. SDK libraries only.

## Generated-source needs

None.

## Planned tests

None. All coverage is created by library source.

## Required interfaces

Provide `Taskfile.yml` and `coverage.md` as defined in the ground rules. `build`
resolves offline and analyzes all fixture-owned libraries. `lint` runs `dart
format` and `dart analyze`. `test` runs `dart test` and succeeds with no test
files.
