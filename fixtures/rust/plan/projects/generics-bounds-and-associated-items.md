# Project brief: generics-bounds-and-associated-items

## Purpose

Demonstrate Rust's generic system: parameter kinds and defaults; bounds, where
clauses, implied bounds, and higher-ranked trait bounds; const generics and
evaluatability; associated types, constants, and projection; generic associated
types; lifetime elision and trait-object lifetime defaults; variance and
`PhantomData`; and opaque-type precise capture.

## Exclusive directory

`fixtures/rust/projects/generics-bounds-and-associated-items/`

## Difficulty

complex

## Assigned coverage

- **RS-CAN-GEN-001** — lifetime, type, and const parameters with legal defaults
  across structs, enums, functions, traits, impls, aliases, and associated items.
- **RS-CAN-GEN-002** — inline and where bounds, associated projection bounds, an
  implied outlives use, and higher-ranked trait bounds.
- **RS-CAN-GEN-003** — const parameters of integer, boolean, and character types
  with distinct instantiations and compile-time use (minimal const generics).
- **RS-CAN-GEN-004** — associated types and constants with defaults, overrides,
  shorthand, and fully qualified projection.
- **RS-CAN-GEN-005** — a lifetime-parameterized generic associated type with a
  required `where Self: 'a` and a generic consumer.
- **RS-CAN-GEN-006** — function and method lifetime elision, `'_` inference, and
  trait-object default lifetime bounds.
- **RS-CAN-GEN-007** — covariance, contravariance, and invariance, and multiple
  `PhantomData` forms.
- **RS-CAN-GEN-008** — opaque-type capture defaults contrasted across editions
  2021 and 2024 with explicit `use<...>` control.

Named counterexamples (an illegal function default, a missing non-implied bound,
an unsupported const expression, a post-monomorphization failure, a rejected
lifetime substitution) are preserved as documentation beside their valid
demonstrations.

## Declared build contexts

- Default: `x86_64-unknown-linux-gnu`, edition 2024, stable 1.97.1.
- Edition 2021 for the precise-capture default contrast in RS-CAN-GEN-008.

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
