# Project brief: types-and-layout

## Purpose

Demonstrate Rust's type forms and how the compiler relates, sizes, and converts
them: struct, enum, and union forms; type aliases versus nominal newtypes;
primitive, tuple, array, slice, string, and never types; references and raw
pointers; function-item, function-pointer, and closure types; dynamically sized
types and trait objects; argument- and return-position `impl Trait`, including in
traits; recursive types; representation and layout attributes; type inference;
coercions; and cast expressions.

## Exclusive directory

`fixtures/rust/projects/types-and-layout/`

## Difficulty

complex

## Assigned coverage

- **RS-CAN-TYPE-001** — named-field, tuple, unit, and zero-field braced structs,
  construction, update, and destructuring.
- **RS-CAN-TYPE-002** — enum variant forms, discriminants, fieldless casts, and
  cross-crate non-exhaustiveness.
- **RS-CAN-TYPE-003** — unions with `Copy` and `ManuallyDrop` fields, safe writes,
  and unsafe reads.
- **RS-CAN-TYPE-004** — a type alias versus a nominal newtype over the same
  representation.
- **RS-CAN-TYPE-005** — tuples, const-length arrays, slice coercion, string and
  byte forms, and the never type at valid sites.
- **RS-CAN-TYPE-006** — shared, mutable, and raw references, `&raw` formation, and
  unsafe dereference (documented pointer APIs only).
- **RS-CAN-TYPE-007** — function-item, function-pointer, and closure types and
  their coercions.
- **RS-CAN-TYPE-008** — dynamically sized types, `?Sized`, a trailing-slice
  struct, and pointer metadata.
- **RS-CAN-TYPE-009** — dyn-compatible trait objects with associated, lifetime,
  and auto-trait bindings, and trait upcasting.
- **RS-CAN-TYPE-010** — argument- and return-position `impl Trait` with a named
  generic twin.
- **RS-CAN-TYPE-011** — return-position `impl Trait` and `async fn` in traits with
  differing hidden types.
- **RS-CAN-TYPE-012** — boxed and mutually recursive types through indirection.
- **RS-CAN-TYPE-013** — `repr(Rust)`, `repr(C)`, integer enum reprs, `transparent`,
  `packed`, and `align` where guaranteed.
- **RS-CAN-TYPE-014** — inference flow, numeric fallback, and target-directed
  selection.
- **RS-CAN-TYPE-015** — the fixed coercion set at designated coercion sites.
- **RS-CAN-TYPE-016** — the closed set of `as` casts contrasted with library
  conversions.

Named counterexamples (private-field external construction, duplicate or overflow
discriminants, a non-`Copy` union field, recursive alias rejection, a packed
unaligned reference, an invalid cast, index panic) are preserved as documentation
beside their valid demonstrations, and invalid bit-pattern or reference cases are
documented rather than executed.

## Declared build contexts

- Default: `x86_64-unknown-linux-gnu`, edition 2024, stable 1.97.1.
- A cross-crate boundary (a local downstream crate) for non-exhaustive and
  visibility-sensitive type effects.

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
