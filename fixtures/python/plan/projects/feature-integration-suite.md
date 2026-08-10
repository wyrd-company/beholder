---
relationships:
  references:
    - ../../ground-rules
    - ../../../../reports/python/synthesis/checklist
---

# feature-integration-suite

## Purpose

A suite of combined cross-feature scenarios whose behavior is not exposed by
isolated constituent coverage: a decorated, overloaded, generic descriptor;
dataclass inheritance with descriptors, slots, and matching; circular imports
with annotations and introspection; async-generator cleanup under cancellation;
multiprocessing with entry modes and pickle identity; and a typed optional native
accelerator fallback.

## Exclusive directory

`fixtures/python/projects/feature-integration-suite/`

## Difficulty

`complex`

## Assigned coverage

Primary owner of the required cross-feature interaction items below. Represent all
valid variants named by each checklist row. The namespace-split and
generated-version interactions are owned by `distributable-suite`.

| Identifier | Valid-variant scope |
| --- | --- |
| `PY-CAN-INT-001` | All named variants combining binding, wrappers, overloads, annotations, and registration on one member. |
| `PY-CAN-INT-002` | All named variants of dataclass inheritance with descriptors, slots, and matching. |
| `PY-CAN-INT-003` | All named variants breaking a cycle statically, then resolving hints with and without namespaces. |
| `PY-CAN-INT-006` | All named variants of async-generator cleanup under cancellation, context, and task group. |
| `PY-CAN-INT-007` | Package, module, and console entry modes with pickle identity, without recursive startup. |
| `PY-CAN-INT-008` | Native and pure implementations selected through narrow handling behind one typed facade. |

## Declared build contexts

- Default runtime, including the asyncio event loop.
- Native / C extension: for the native accelerator of `PY-CAN-INT-008`.
- Multiprocessing start methods and child process: for the entry modes of
  `PY-CAN-INT-007`.

## Dependency needs

None. The C toolchain is an installed system tool, not a vendored distribution.

## Generated-source needs

None.

## Planned tests

- A standard-library test module creating the test-runner main-identity variant
  of `PY-CAN-INT-007`.

## Required interfaces

Provide `Taskfile.yml` and `coverage.md` as defined in the ground rules. `build`
compiles the native accelerator of `PY-CAN-INT-008`; `test` runs the module
named above.
