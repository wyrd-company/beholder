---
relationships:
  references:
    - ground-rules
    - overview
---

# Project brief: async-and-isolates

## Purpose

A concurrency library that exercises futures, generators, streams, event queues
and zones, and native isolate messaging.

## Exclusive directory

`fixtures/dart/projects/async-and-isolates/`

## Difficulty

complex

## Assigned coverage

- **DART-CAN-ASYNC-001** — `Future`, `async`, and `await`: synchronous prefix
  execution, suspension, value and future return flattening, `FutureOr`,
  awaiting in loops, `try`/`finally`, and an asynchronous `main`.
- **DART-CAN-ASYNC-002** — Synchronous and asynchronous generators: `sync*`,
  `async*`, `yield`, and `yield*`, with visible laziness and delegation; error
  and `finally` handling; iterable versus stream behavior.
- **DART-CAN-ASYNC-003** — Stream consumption and subscription modes: `await
  for`, a single-subscription stream, a broadcast stream, a controller,
  transformations, and an error event; two broadcast listeners; cancellation;
  pause and resume.
- **DART-CAN-ASYNC-004** — Event queues, unawaited futures, and zones:
  microtask and event ordering only where documented, `unawaited`, a guarded
  zone, a zone-local value, and an intercepted asynchronous error; callbacks
  bound to zones.
- **DART-CAN-ISO-001** — Isolate spawning and messages: `Isolate.run`,
  `Isolate.spawn`, ports, a top-level or static entry point, and result, error,
  and exit handling on native targets; pause, resume, and kill; no shared
  mutable state.
- **DART-CAN-ISO-002** — Sendability and transferable data: sending primitives,
  collections, ports, and `TransferableTypedData` where supported; immutable
  sharing; an unsendable value.

The web isolate and `spawnUri` web variants of DART-CAN-ISO-001 are excluded as
an unavailable context; isolates are covered on native targets. Documented
microtask, timer, and host-interleaving corners are recorded as observations,
not asserted, per the checklist's implementation-defined notes.

## Declared build contexts

- `vm-jit` (default).
- `native-aot`, for isolate spawning and messaging under ahead-of-time
  compilation.

## Dependency needs

None. SDK libraries (`dart:async`, `dart:isolate`) only.

## Generated-source needs

None.

## Planned tests

None. All coverage is created by library and executable source.

## Required interfaces

Provide `Taskfile.yml` and `coverage.md` as defined in the ground rules. `build`
resolves offline, analyzes all fixture-owned libraries, and compiles the
declared entry points under `native-aot`. `lint` runs `dart format` and `dart
analyze`. `test` runs `dart test` and succeeds with no test files.
