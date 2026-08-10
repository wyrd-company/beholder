---
relationships:
  references:
    - ground-rules
    - reports/typescript/synthesis/checklist
---

# Project brief: runtime-constructs-and-emit

## Purpose

A library of runtime-visible constructs whose behavior and emit are the coverage:
enums and const-enum boundaries, async functions and thenables, generators and
iteration protocols, optional chaining, nullish coalescing, and logical
assignment, prototypes and mixins, exceptions, reachability, and exhaustiveness,
evaluation order and helper-mediated downlevel behavior, and explicit resource
management.

## Exclusive directory

`fixtures/typescript/projects/runtime-constructs-and-emit/`

## Difficulty

`complex`

## Assigned coverage

Only valid-source obligations are corpus content. Invalid enum initializers and
isolated-transform rejection, missing Promise or iterable libraries, invalid
assignment targets, duplicate-name mixin failures, unreachable-statement
diagnostics, and incompatible-`tslib` failures are excluded per ground rules.

- TS-CAN-047 — `using` and `await using` with synchronous and asynchronous
  disposable protocols; nested scopes; multiple resources; early return; a thrown
  completion with reverse disposal order and cleanup dispatch observable; a
  downlevel helper path; `SuppressedError`.
- TS-CAN-081 — numeric, string, heterogeneous, constant, computed, union-member,
  merged, and namespace-augmented enums; numeric reverse maps and missing string
  reverse maps; ordinary versus `const enum` emit locally and across a
  declaration boundary; a bit flag; `preserveConstEnums`; an ambient const enum.
- TS-CAN-082 — an async function and method; a rejection; a custom thenable;
  nested promise-like unwrapping; an importer depending on top-level await;
  old-target helper emit; the `Awaited` static result; a cyclic async module.
- TS-CAN-083 — typed generator yield, return, and next channels; delegated yield;
  an async generator; a custom iterable and iterator; `for...of`;
  `for await...of`; early-loop cleanup; `downlevelIteration`; async-from-sync
  iteration; old and new target emit; the iterator-library changes.
- TS-CAN-084 — optional property, element, and call chains; a receiver-preserving
  optional method call; grouping that breaks the chain; `??` versus `||` on zero
  and empty string; `&&=`, `||=`, and `??=` with flow effects; a side-effectful
  getter or index skipped or evaluated; old-target temporaries.
- TS-CAN-085 — prototype dispatch through a base reference; extracted receiver
  loss; prototype mutation; interface-only conformance with no runtime heritage;
  two class-expression mixins composed over a base; `super`; arrow-property
  dispatch; `instanceof`; `Object.assign` shallow copy; intersection instance
  types.
- TS-CAN-086 — throwing Error and non-Error values; guarding an `unknown` catch
  under `useUnknownInCatchVariables`; `finally` overriding completion; calling a
  `never`-returning function; complete and incomplete discriminated unions or
  enums with a `never` proof; async rejection; an assertion function; a disposal
  error; switch fallthrough.
- TS-CAN-087 — order observable across computed keys, getters, spreads,
  arguments, class elements, decorators, and cleanup; native versus lowered
  targets; inline helpers and `importHelpers`/`tslib`; helper deduplication;
  iterable assumptions in spread; native versus transformed fields; a
  helper-created runtime dependency absent from authored imports.

## Declared compilation contexts

- A native modern target and a downlevel target for helper emit,
  `downlevelIteration`, `using`/`await using` helpers, and class-field lowering.
- An `importHelpers`/`tslib` context.

## Dependency needs

None required. `tslib` emission is observable in output. `tslib` may be vendored,
with pinned version, provenance, and license, only if a planned runnable path
requires executing `importHelpers` output; none is planned.

## Generated-source needs

None.

## Planned tests

`none`. Runtime constructs and their emit are created by source and
configuration; no assigned coverage is created by test source.

## Required interfaces

- `Taskfile.yml` defines `build`, `lint`, and `test` as thin wrappers per ground
  rules; `build` compiles under the native and downlevel target contexts.
- `coverage.md` records the ground-rules coverage table for every assigned
  identifier, using the compilation-context column for each target context and
  additional locators for distinct valid variants.
