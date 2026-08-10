# Project brief: async-and-concurrency

## Purpose

Demonstrate Rust's asynchronous and concurrent surface at the language level:
async functions and blocks and their opaque futures; await, polling, and
suspension borrows; pinning and self-referential state; async closures and async
call traits; future sendability across suspension; threads with scoped borrowing;
and atomics with memory orderings.

## Exclusive directory

`fixtures/rust/projects/async-and-concurrency/`

## Difficulty

complex

## Assigned coverage

- **RS-CAN-ASYNC-001** — async functions and move and non-move blocks with locals
  crossing suspension, consumed through a generic `Future` bound.
- **RS-CAN-ASYNC-002** — a custom Pending-to-Ready `Future`/`IntoFuture`, a borrow
  held across suspension, and a dropped suspended future with observable cleanup.
- **RS-CAN-ASYNC-003** — `Unpin` and marker-composed `!Unpin` types, stack and heap
  pinning, permitted projection, and a custom poll receiver.
- **RS-CAN-ASYNC-004** — reading, mutating, and consuming async closures against
  supported call-trait bounds.
- **RS-CAN-ASYNC-005** — near-identical futures with a non-`Send` value before
  versus across a suspension point checked against a `Send` bound.
- **RS-CAN-CONCUR-001** — moving owned data to a thread, a scoped borrow, a `Sync`
  requirement, and panic and join inspection.
- **RS-CAN-CONCUR-002** — load, store, read-modify-write, compare-exchange, and
  fence operations with valid orderings and a cfg-gated atomic width.

Named counterexamples (recursive async without indirection, await outside async,
a borrowed non-scoped thread capture, a non-`Send` spawn, an invalid ordering, a
pointer escape undermining pin) are preserved as documentation beside their valid
demonstrations, and data-race undefined behavior is documented rather than
executed.

## Declared build contexts

- Default: `x86_64-unknown-linux-gnu`, edition 2024, stable 1.97.1.
- `target_has_atomic`-gated atomic width selection on the host.

## Dependency needs

No third-party dependency. Futures are driven by a minimal in-project poll or
block-on facility; ecosystem async runtimes are owned by
`async-runtime-and-registration`.

## Generated-source needs

None.

## Planned tests

None.

## Required interfaces

Provide `Taskfile.yml` with `build`, `lint`, and `test` as defined in the ground
rules, and `coverage.md` mapping each assigned identifier to its stable source
location.
