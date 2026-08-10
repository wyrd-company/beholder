---
relationships:
  references:
    - ../../ground-rules
    - reports/csharp/synthesis/checklist
---

# Project brief: unsafe-and-lowlevel

## Purpose

An idiomatic library that exercises the low-level and unsafe surface of C#:
native-sized integers, ref-like and inline-array structs, unsafe pointers and
function pointers, pinning and stack allocation, explicit layout metadata, and
native interoperation.

## Exclusive directory

`fixtures/csharp/projects/unsafe-and-lowlevel/`

## Difficulty

complex

## Assigned canonical identifiers

- CS-CAN-028 — native-sized integers (64-bit-process demonstration).
- CS-CAN-029 — ref-like, readonly, ref-field, and inline-array structs.
- CS-CAN-030 — unsafe pointer, function-pointer, fixed-buffer, and pinning types.
- CS-CAN-080 — fixed, stack allocation, and ref-escape lifetime.
- CS-CAN-107 — native interoperation and source-generated marshalling.
- CS-CAN-108 — layout and marshalling metadata.

Valid variants named by these items are in scope, including `nint` and `nuint`
constants, casts, arithmetic, `sizeof`, and checked overflow on a 64-bit process;
`ref struct`, `readonly struct`, ref fields, `scoped`, `[UnscopedRef]`, and
`[InlineArray]`; pointer arithmetic and conversions, fixed buffers, `fixed`,
`stackalloc`, and managed and unmanaged `delegate*`; pinning of strings, arrays,
spans, and a custom pinnable pattern; the `DllImport` and `LibraryImport` bindings
of one native operation; and sequential, explicit, packed, fixed-buffer, and union
layouts with inspected size and offsets. The 32-bit-process native-integer range
variant is excepted in the overview as an unavailable context. The pointer-to-
managed, ABI-mismatch, and disabled-unsafe counterexamples those rows also list
are invalid source and are not fixture content.

## Declared build contexts

- Default `net10.0` with unsafe compilation (`AllowUnsafeBlocks`).
- A native companion shared library built from committed C source with the
  locally installed `clang` or `gcc`, for the `DllImport` and `LibraryImport`
  targets of CS-CAN-107.

## Dependency needs

None third-party. The native companion library is built from C source committed
to this project using the locally installed `clang` or `gcc`; no external native
package is acquired and `build` performs no download.

## Generated-source needs

The `LibraryImport` source generator supplied by the SDK emits its partial
counterpart in-process during compilation. No committed generated output and no
`generate` task.

## Planned tests

none

## Required interfaces

`Taskfile.yml` and `coverage.md` as defined in `fixtures/csharp/ground-rules.md`.
