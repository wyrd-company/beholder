# Project brief: traits-and-dispatch

## Purpose

Demonstrate Rust's trait system and dispatch: inherent and trait
implementations; coherence, orphan coverage, and overlap; conditional and blanket
impls; supertraits, defaults, and overrides; receiver forms; method lookup with
autoderef, autoref, and unsizing; ambiguity and fully qualified syntax; static
and dynamic dispatch; auto traits and unsafe overrides; operator, index, deref,
and formatting traits; explicit conversion traits; drop glue with `Copy` and
`Clone`; and derive-generated standard impls.

## Exclusive directory

`fixtures/rust/projects/traits-and-dispatch/`

## Difficulty

complex

## Assigned coverage

- **RS-CAN-TRAIT-001** — inherent impls in separate modules and a trait impl in
  a private module used from elsewhere.
- **RS-CAN-TRAIT-002** — legal local-trait and local-type impls, fundamental-type
  and newtype coverage, and overlap counterexamples.
- **RS-CAN-TRAIT-003** — a conditional wrapper impl and a blanket extension trait
  with satisfying and non-satisfying sites.
- **RS-CAN-TRAIT-004** — a supertrait hierarchy with inherited and overridden
  defaults.
- **RS-CAN-TRAIT-005** — `self`, `&self`, `&mut self`, `Box<Self>`,
  `Pin<&mut Self>`, and no-receiver methods (stable receiver forms).
- **RS-CAN-TRAIT-006** — method lookup through a custom deref chain, autoref, and
  one unsizing step.
- **RS-CAN-TRAIT-007** — an ambiguous call resolved with `Trait::method` and
  `<Type as Trait>::method`, extended to constants, types, and functions.
- **RS-CAN-TRAIT-008** — the same trait consumed through generics, `impl Trait`,
  `&dyn Trait`, boxed objects, and a heterogeneous collection.
- **RS-CAN-TRAIT-009** — composite `Send`/`Sync` inheritance and failure, a
  marker- or pointer-field change, and a justified `unsafe impl`.
- **RS-CAN-TRAIT-010** — arithmetic, assignment, comparison, indexing, deref, and
  formatting traits invoked only through syntax.
- **RS-CAN-TRAIT-011** — infallible and fallible conversion, reciprocal blanket
  `Into`, and `?` error conversion.
- **RS-CAN-TRAIT-012** — nested and field drop order, explicit `drop`, partial
  moves, `ManuallyDrop`, `forget`, and `Copy` versus `Clone`.
- **RS-CAN-TRAIT-013** — derived Debug, Clone, Copy, equality, order, hash, and
  default with a manual impl avoiding an unnecessary bound.

Named counterexamples (a foreign-for-foreign impl, an overlapping impl, a missing
supertrait, a consuming dyn method, a `Copy + Drop` rejection, an invalid implicit
conversion, a direct destructor call) are preserved as documentation beside their
valid demonstrations.

## Declared build contexts

- Default: `x86_64-unknown-linux-gnu`, edition 2024, stable 1.97.1.
- A cross-crate boundary (a local downstream crate) for coherence, orphan, and
  cross-crate conversion effects.

## Dependency needs

No third-party dependency. The cross-crate boundary uses a local crate.

## Generated-source needs

None.

## Planned tests

None.

## Required interfaces

Provide `Taskfile.yml` with `build`, `lint`, and `test` as defined in the ground
rules, and `coverage.md` mapping each assigned identifier to its stable source
location.
