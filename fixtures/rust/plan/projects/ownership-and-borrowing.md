# Project brief: ownership-and-borrowing

## Purpose

Demonstrate Rust's ownership and borrowing model: moves, move paths, and partial
moves; borrow exclusivity and reborrowing; non-lexical lifetimes and two-phase
borrows; temporary lifetime extension and drop scopes; interior mutability;
closure capture precision and mode; the `Fn`, `FnMut`, and `FnOnce` call traits;
pattern binding modes and match ergonomics; refutability, or-patterns, guards,
and exhaustiveness; and destructuring assignment with drop check.

## Exclusive directory

`fixtures/rust/projects/ownership-and-borrowing/`

## Difficulty

complex

## Assigned coverage

- **RS-CAN-OWN-001** — a destructured struct moving one field while borrowing or
  copying another.
- **RS-CAN-OWN-002** — accepted non-overlapping reborrows and mutation after a last
  shared use.
- **RS-CAN-OWN-003** — non-lexical lifetimes, a branch keeping a borrow live, and
  a two-phase autoref borrow.
- **RS-CAN-OWN-004** — extending and non-extending temporaries, match scrutinee,
  `if let`, block tail, and destructor order across editions 2021 and 2024.
- **RS-CAN-OWN-005** — `Cell`, successful and panicking `RefCell` borrows, and a
  minimal sound `UnsafeCell` abstraction.
- **RS-CAN-OWN-006** — every capture mode, disjoint field capture, a move closure,
  and later use of uncaptured values across editions 2018 and 2021.
- **RS-CAN-OWN-007** — reading, mutating, and consuming closures matched to `Fn`,
  `FnMut`, and `FnOnce` bounds.
- **RS-CAN-OWN-008** — one structure matched as owned, shared, and mutable
  references with inferred and explicit binding modes across editions 2021 and
  2024.
- **RS-CAN-OWN-009** — patterns in `let`, parameters, `if let`, `while let`,
  `let-else`, and match arms with enum, tuple, slice, range, or, and guarded
  forms.
- **RS-CAN-OWN-010** — nested destructuring assignment and a generic wrapper with
  and without suitable phantom ownership under a `Drop` impl.

Named counterexamples (a whole-value use after move, conflicting simultaneous
borrows, a `RefCell` runtime panic, an inconsistent or-binding, a refutable
irrefutable-context pattern, a borrow rejected by destruction order) are preserved
as documentation beside their valid demonstrations.

## Declared build contexts

- Default: `x86_64-unknown-linux-gnu`, edition 2024, stable 1.97.1.
- Editions 2018 and 2021 for closure-capture precision and match-ergonomics
  contrasts.

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
