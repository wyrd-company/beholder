---
relationships:
  references:
    - ground-rules
    - reports/go/synthesis/checklist
    - fixtures/go/plan/overview
---

# Project brief: `collections-toolkit`

## Purpose

An idiomatic library of generic data structures, numeric utilities, and iteration
helpers. It is the primary owner of the type, method, interface, generic,
expression, and control-flow semantics, and of the recoverable runtime-panic
taxonomy those structures naturally exercise.

## Exclusive directory

`fixtures/go/projects/collections-toolkit/`

## Difficulty

complex

## Assigned canonical identifiers and distinct valid variants

- **GO-CAN-TYP-001** — untyped constants and default types across Boolean, rune,
  integer, floating, complex, and string constants in assignment, calls,
  comparisons, shifts, and inference.
- **GO-CAN-TYP-002** — `iota` and implicit repetition. Variants: explicit and
  omitted expression lists, multiple values, shifts, `_` gaps, reset per
  declaration.
- **GO-CAN-TYP-003** — defined types versus aliases. Variants: definition and
  alias over one target through assignment, conversion, method, reflection, and
  cross-package cases; aliases of instantiated and generic targets, the latter at
  language version go1.23 or higher.
- **GO-CAN-TYP-004** — underlying types, identity, assignability, conversion, and
  named/unnamed types across scalars, composites, pointers, channels, interfaces,
  and type parameters, including implicit versus explicit conversion and
  channel-direction assignment.
- **GO-CAN-TYP-005** — target-sized numeric types and aliases. Variants: `int`,
  `uint`, `uintptr`, fixed-width types, `byte`, and `rune` compared on 32- and
  64-bit targets, including width-boundary constants valid on one target.
- **GO-CAN-TYP-006** — arrays: length in identity, inferred-length literals, value
  copying, comparison, and pointer-to-array slicing.
- **GO-CAN-TYP-007** — slices: nil versus empty, shared backing storage,
  length/capacity, full slicing, append reuse and reallocation, copy, clear, and
  the slice-to-array conversions introduced at go1.17 and go1.20.
- **GO-CAN-TYP-008** — maps: key comparability, nil-map reads, writes,
  delete/clear, two-value lookup, iteration, and reference-like sharing.
- **GO-CAN-TYP-009** — struct fields, tags, identity, layout, comparability, and
  named/anonymous keyed and unkeyed literals with exported and unexported fields.
- **GO-CAN-TYP-010** — embedded fields and selector promotion through value and
  pointer fields at multiple depths, including shallower-depth wins and valid
  explicit paths.
- **GO-CAN-TYP-011** — pointers, addressability, and `new`, including automatic
  method-call addressing and dereferencing and pointer comparison.
- **GO-CAN-TYP-012** — function types and variadics: named and unnamed function
  types, closures, multiple results, variadic calls, and slice spreading.
- **GO-CAN-TYP-013** — channels and directionality: bidirectional to send-only and
  receive-only conversion and assignment in calls and constraints.
- **GO-CAN-TYP-014** — recursive types through pointer, slice, map, function,
  interface, and generic indirection, with mutual recursion split across files and
  recursive generic declarations.
- **GO-CAN-MET-001** — receiver declarations and method ownership on local defined
  non-pointer base types, including generic receiver parameters.
- **GO-CAN-MET-002** — value and pointer method sets, addressable call sugar,
  embedding, and nil pointer receivers.
- **GO-CAN-MET-003** — method values, method expressions (`T.M`, `(*T).M`), and
  a named function type with methods used as an adapter.
- **GO-CAN-IFC-001** — implicit interface implementation and dynamic dispatch,
  including compile-time assertions and static-concrete versus dynamic-interface
  calls.
- **GO-CAN-IFC-002** — interface values and typed nil: a typed nil pointer in a
  non-nil interface inspected by comparison, assertion, reflection, and method
  call.
- **GO-CAN-IFC-003** — interface embedding, method identity, merged identical
  overlaps, and unexported-method sealing.
- **GO-CAN-IFC-004** — assertions and type switches with one- and two-result
  assertions, interface-to-interface assertions, and concrete, interface, and nil
  cases.
- **GO-CAN-GEN-001** — generic declarations and cross-package instantiation with
  explicit, inferred, and partial function instantiation.
- **GO-CAN-GEN-002** — constraint type sets: method elements, embedded
  constraints, unions, `~` terms, `comparable`, and `any`, including the go1.20
  `comparable` satisfaction change.
- **GO-CAN-GEN-003** — type inference through argument, assignment-context,
  result-context, method, and unification-driven inference, with the go1.21 and
  go1.22 inference boundaries.
- **GO-CAN-GEN-004** — operations on type parameters valid for every member of a
  type set: comparison, indexing, slicing, ranging, channel operations,
  conversion, and method calls.
- **GO-CAN-GEN-005** — methods on generic types and a generic alias, at language
  version go1.23 or higher where the released compiler accepts generic aliases
  without an experiment. Distinct valid variants: instantiated-target
  restrictions, cross-package aliasing, and receiver restrictions.
- **GO-CAN-EXP-001** — selector resolution across package-qualified, field,
  method, embedded promoted, pointer, and method-expression selectors.
- **GO-CAN-EXP-002** — composite literals for arrays, slices, maps, structs, and
  generic instantiated types with keyed, unkeyed, and nested elision forms.
- **GO-CAN-EXP-003** — indexing and slicing of strings, arrays, pointers-to-array,
  slices, maps, and type parameters, with two- and three-index slices.
- **GO-CAN-EXP-004** — shared call, conversion, and instantiation syntax `T(x)`
  resolving as calls, conversions, generic instantiations, and built-ins.
- **GO-CAN-EXP-005** — multiple assignment and tuple-producing expressions:
  multi-result calls, map lookup, channel receive, assertion, parallel assignment,
  swap, and blank targets.
- **GO-CAN-EXP-006** — evaluation order with specified function-call and
  communication ordering and cases whose relative order admits multiple outcomes.
- **GO-CAN-EXP-007** — named results and naked return with deferred mutation,
  shadowing, and multiple results.
- **GO-CAN-EXP-008** — function literals and closure capture of values and
  addresses, including loop variables on both sides of the go1.22 boundary.
- **GO-CAN-EXP-009** — built-in functions and context-sensitive typing across
  allocation, collection, complex-number, `close`/`panic`/`recover`,
  `append`/`copy`/`delete`/`clear`, `min`/`max`, and print families.
- **GO-CAN-EXP-010** — string, byte, rune, integer, slice, and array/pointer
  conversions, including invalid-UTF-8 replacement and string immutability.
- **GO-CAN-CTL-001** — loop-variable language-version boundary: identical
  three-clause and range closure and address cases under `go1.21` and `go1.22`
  file or module equivalents, plus a variable declared outside the loop.
- **GO-CAN-CTL-002** — `for` and core `range` forms over array, pointer-to-array,
  slice, string, map, and channel with zero, one, and two variables.
- **GO-CAN-CTL-003** — integer range over signed, unsigned, typed, untyped, zero,
  and negative expressions at language version go1.22 or higher.
- **GO-CAN-CTL-004** — range over iterator functions with zero-, one-, and
  two-value yield signatures and early-stop, at language version go1.23 or higher.
- **GO-CAN-CTL-005** — expression and type switches, tagged and tagless, with init
  statements, cases, default, and legal `fallthrough`.
- **GO-CAN-CTL-006** — `if` and switch initialization scopes with legal shadowing
  across init, condition or tag, clauses, and outer blocks.
- **GO-CAN-CTL-007** — branching statements: unlabeled and labeled `break` and
  `continue`, `goto`, and switch-only `fallthrough` in legal placements.
- **GO-CAN-CTL-008** — `defer` timing and order for functions, methods, closures,
  and built-ins with immediately evaluated arguments and last-in-first-out
  execution, plus loop-defer accumulation.
- **GO-CAN-CTL-009** — panic, recover, and runtime versus compile-time failure:
  explicit panic, nested defers, direct and indirect recover, repanic, and the
  go1.21 `panic(nil)` boundary.
- **GO-CAN-DIA-008** — recoverable runtime panics isolated from one another: nil,
  bounds, assertion, divide, closed-channel, map, and explicit panics, each
  recovered in a deferred function on the panicking goroutine. The fatal,
  non-recoverable runtime-state variant is excluded (see overview exceptions).

## Declared build contexts

- Default context.
- `GOARCH=386` alongside `GOARCH=amd64`, for GO-CAN-TYP-005 width comparison
  (compile context).
- Per-file or per-module language versions across the go1.20–go1.24 boundaries for
  GO-CAN-TYP-003, GO-CAN-GEN-002, GO-CAN-GEN-003, GO-CAN-GEN-005, GO-CAN-CTL-001,
  GO-CAN-CTL-003, and GO-CAN-CTL-004.

## Dependency needs

Standard library and project-local packages only. No third-party dependency.

## Generated-source needs

None.

## Planned tests

None. Coverage is created by library source, not by test source; the project does
not test its represented application's domain correctness.

## Required `Taskfile.yml` and `coverage.md` interfaces

- `Taskfile.yml` defines `build`, `lint`, and `test` as thin wrappers over
  ordinary Go commands. `build` compiles all fixture-owned packages in every
  declared build context, including the `386` and language-version variants.
  `lint` checks `gofmt` formatting and runs `go vet` on fixture-owned packages.
  `test` runs Go tests and succeeds with no test files. All three pass and write
  no logs, evidence, manifests, or hashes.
- `coverage.md` is a Markdown table recording, per assigned identifier, the
  canonical identifier, a stable source path, the named declaration, directive,
  package, module, or test that creates coverage, the build context when non-
  default, and additional locators for distinct valid variants. It uses stable
  names, not line numbers.
