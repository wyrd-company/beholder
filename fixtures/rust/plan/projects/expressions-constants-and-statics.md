# Project brief: expressions-constants-and-statics

## Purpose

Demonstrate Rust's expression, control-flow, and compile-time value model: place,
value, and assignee contexts; blocks, statements, tails, and semicolons;
conditional expressions, `if let`, and let chains; loops, labels, iteration, and
value-bearing break; return, divergence, panic, and unreachable code; the `?`
operator and residual conversion; ranges, indexing, and slicing; evaluation order
and short-circuiting; literals and suffixes; constants, inline const, and statics;
const functions; and static promotion with thread-local storage.

## Exclusive directory

`fixtures/rust/projects/expressions-constants-and-statics/`

## Difficulty

complex

## Assigned coverage

- **RS-CAN-EXPR-001** — locals, statics, dereferences, fields, and indexes in
  value, borrow, assignment, and compound-assignment contexts.
- **RS-CAN-EXPR-002** — value and unit blocks, nested item statements, and
  semicolon-sensitive return types.
- **RS-CAN-EXPR-003** — a value-producing `if`, scoped `if let`, and a let chain
  with short-circuit and binding behavior on edition 2024.
- **RS-CAN-EXPR-004** — nested labeled loops and blocks, a value-returning loop,
  continue, and custom `IntoIterator` impls.
- **RS-CAN-EXPR-005** — early returns, an infinite loop, a panic macro, `-> !`,
  and unreachable code in functions and closures.
- **RS-CAN-EXPR-006** — `?` on `Result` with error conversion, `Option`, and
  `main -> Result`, including nested closure and async contexts.
- **RS-CAN-EXPR-007** — every range form, allowed range patterns, built-in and user
  mutable indexing, and range slicing.
- **RS-CAN-EXPR-008** — recorded side effects across tuples, arrays, calls, binary
  operands, assignment, short-circuit conditions, and indexed compound assignment.
- **RS-CAN-EXPR-009** — numeric bases, separators, and suffixes; inferred numerics;
  raw, Unicode, byte, and C-string literals.
- **RS-CAN-CONST-001** — free, associated, and unnamed constants, generic inline
  const, immutable, interior-mutable, and mutable statics, and a compile-time
  assertion.
- **RS-CAN-CONST-002** — a const function called in array length, discriminant,
  static, const-generic, inline-const, and runtime contexts (stable const core).
- **RS-CAN-CONST-003** — a promotable immutable borrow contrasted with
  non-promotable values and stable thread-local storage.

Named counterexamples (an incompatible branch, an invalid valued break, an
out-of-range literal, a static-mut reference denial under edition 2024, a
runtime-dependent value that cannot be promoted, an unconditional-panic index) are
preserved as documentation beside their valid demonstrations.

## Declared build contexts

- Default: `x86_64-unknown-linux-gnu`, edition 2024, stable 1.97.1.
- Edition 2021 where a let-chain or temporary-scope contrast requires the earlier
  rescoping behavior.

## Dependency needs

No third-party dependency.

## Generated-source needs

None.

## Planned tests

None.

## Required interfaces

Provide `Taskfile.yml` with `build`, `lint`, and `test` as defined in the ground
rules, and `coverage.md` mapping each assigned identifier to its stable source
location.
