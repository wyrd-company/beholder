# Project brief: unsafe-ffi-and-abi

## Purpose

Demonstrate Rust's unsafe, foreign-interface, and low-level surface: unsafe
blocks, functions, traits, and impls; validity, initialization, and
`MaybeUninit`; documented pointer-provenance and address APIs; transmutation and
bit compatibility; foreign blocks, functions, statics, and ABI strings; export
names, sections, and link identities; FFI-safe layouts, handles, and callbacks;
unwinding across ABI boundaries; inline and global assembly with target features;
and the `no_std` and `alloc` runtime contracts.

## Exclusive directory

`fixtures/rust/projects/unsafe-ffi-and-abi/`

## Difficulty

complex

## Assigned coverage

- **RS-CAN-UNSAFE-001** — raw dereference, unsafe call, mutable-static access, union
  read, and an unsafe trait impl behind a safe wrapper.
- **RS-CAN-UNSAFE-002** — staged value and array initialization with partial-failure
  cleanup and a justified `assume_init` (documented validity frontier only).
- **RS-CAN-UNSAFE-003** — documented strict-provenance and address APIs,
  in-allocation arithmetic, one-past comparison, and pointer mapping (documented
  API cases only).
- **RS-CAN-UNSAFE-004** — a justified transparent-wrapper transmute with documented
  invalid counterexamples.
- **RS-CAN-FFI-001** — declaring and calling a C-ABI function and static,
  defining a C-export function, and a variadic declaration.
- **RS-CAN-FFI-002** — renamed exported and imported symbols, a sectioned static,
  and native library selection with build-script link directives.
- **RS-CAN-FFI-003** — `repr(C)` struct and union, integer-repr enum, opaque handle,
  nullable pointer and function pointer, and a callback.
- **RS-CAN-FFI-004** — `extern "C"` and `extern "C-unwind"` callbacks with
  `catch_unwind` at a boundary under unwind and abort profiles.
- **RS-CAN-FFI-005** — target-gated `x86_64` inline `asm!` with operands and `sym`,
  `global_asm!`, an intrinsic behind runtime detection, and a `target_feature`
  function.
- **RS-CAN-PROJ-017** — a `no_std` library, an `alloc`-using variant, identical
  core types through `std` and `core` paths, and the target-gated freestanding
  runtime hooks (entry symbol, panic handler, and global allocator) on
  `wasm32-unknown-unknown`.

Named counterexamples (a missing unsafe marker, an invalid transmute size
mismatch, a mismatched foreign declaration, a duplicate symbol, an unsupported ABI
or target feature, a missing panic handler) are preserved as documentation beside
their valid demonstrations, and invalid-bit-pattern, dangling, and aliasing cases
are documented rather than executed.

## Declared build contexts

- Default: `x86_64-unknown-linux-gnu`, edition 2024, stable 1.97.1.
- A `panic = "abort"` profile alongside the default unwind profile for
  RS-CAN-FFI-004.
- `x86_64` inline and global assembly and target-feature and runtime-detection
  paths.
- `#![no_std]` and `alloc` compilation on the host target.
- `wasm32-unknown-unknown` for the freestanding `no_std` runtime-hook variant of
  RS-CAN-PROJ-017 (entry symbol, panic handler, global allocator).
- Rust-defined C-ABI symbols linked across local crate boundaries for foreign
  declaration, ABI, and linking coverage.

## Dependency needs

No third-party dependency. Foreign-interface coverage uses Rust-defined C-ABI
symbols in local crates linked across crate boundaries; native compilation of C
source is owned by `build-scripts-and-codegen`.

## Generated-source needs

None.

## Planned tests

None.

## Required interfaces

Provide `Taskfile.yml` with `build`, `lint`, and `test` as defined in the ground
rules, where `build` exercises the host `no_std`, assembly, and abort-profile
contexts and the `wasm32-unknown-unknown` freestanding build, and `coverage.md`
mapping each assigned identifier to its stable source location.
