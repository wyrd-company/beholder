---
relationships:
  references:
    - ground-rules
    - reports/go/synthesis/checklist
    - fixtures/go/plan/overview
---

# Project brief: `pipeline-scheduler`

## Purpose

A concurrent worker and pipeline library. It is the primary owner of goroutine,
channel, `select`, synchronization, atomic, and memory-model semantics, of
context and error-chain propagation, of runtime finalizer and cleanup callbacks,
and of the nondeterminism and optimization boundaries those constructs expose.

## Exclusive directory

`fixtures/go/projects/pipeline-scheduler/`

## Difficulty

complex

## Assigned canonical identifiers and distinct valid variants

- **GO-CAN-CON-001** — goroutine creation and lifetime for named functions,
  closures, and methods with arguments whose evaluation precedes launch, using
  explicit synchronization so required assertions are deterministic.
- **GO-CAN-CON-002** — channel state and communication: buffered, unbuffered, and
  directional channels; send, receive, close, comma-ok, and range; nil-channel
  blocking and closed-receive zero value.
- **GO-CAN-CON-003** — `select` with send, receive, default, nil-channel
  disabling, several simultaneously ready cases, and closed channels, admitting
  nondeterministic ready-case choice.
- **GO-CAN-CON-004** — happens-before synchronization through channel operations
  and close, mutexes, once, wait groups, and atomics, contrasted with an
  unsynchronized access, under the go1.19 memory model.
- **GO-CAN-CON-005** — a true data race and its synchronized near-neighbor,
  built with and without `-race`. Whether the race manifests is not asserted.
- **GO-CAN-CON-006** — atomic operations and alignment: typed and legacy atomic
  operations, compare-and-swap, load and store, and the 32-bit alignment
  requirement for 64-bit legacy atomics.
- **GO-CAN-DYN-005** — runtime callbacks, finalizers, and cleanup with explicit
  reachability control, `KeepAlive`, cancellation, and the versioned cleanup API.
  Finalizer timing is not asserted.
- **GO-CAN-DYN-009** — context and error chains: context cancellation, deadline,
  and cause; sentinel, typed, wrapped, joined, and custom `Is`, `As`, and
  `Unwrap` errors; identity contrasted with message equality.
- **GO-CAN-DIA-009** — nondeterminism and optimization boundaries: map order,
  ready `select`, goroutine scheduling, finalizers, independent evaluation and
  init edges, escape, inlining, devirtualization, generic code generation, and
  dead-code elimination, with portable assertions that admit all allowed
  outcomes.

## Declared build contexts

- Default context.
- `-race` instrumentation for GO-CAN-CON-005 and the race dimension of
  GO-CAN-DIA-009.
- `GOARCH=386` alongside `GOARCH=amd64` for the GO-CAN-CON-006 64-bit atomic
  alignment requirement (compile context).

## Dependency needs

Standard library and project-local packages only. No third-party dependency.

## Generated-source needs

None.

## Planned tests

None. Concurrency, synchronization, and lifecycle coverage is created by library
and command source and its explicit synchronization, not by domain-correctness
tests.

## Required `Taskfile.yml` and `coverage.md` interfaces

- `Taskfile.yml` defines `build`, `lint`, and `test` as thin wrappers over
  ordinary Go commands. `build` compiles all fixture-owned packages in every
  declared build context, including the race-instrumented and `386` variants.
  `lint` checks `gofmt` formatting and runs `go vet` on fixture-owned packages.
  `test` runs Go tests and succeeds with no test files. All three pass and write
  no logs, evidence, manifests, or hashes.
- `coverage.md` is a Markdown table recording, per assigned identifier, the
  canonical identifier, a stable source path, the named declaration, directive,
  package, module, or test that creates coverage, the build context when non-
  default, and additional locators for distinct valid variants. It uses stable
  names, not line numbers.
