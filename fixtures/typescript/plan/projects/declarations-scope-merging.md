---
relationships:
  references:
    - ground-rules
    - reports/typescript/synthesis/checklist
---

# Project brief: declarations-scope-merging

## Purpose

A library centered on declaration identity and scope: the separate type, value,
and namespace spaces of one spelling; lexical scope, hoisting, temporal dead
zones, and binding patterns; interface, cross-kind, and enum declaration merging;
and computed, symbol, and unique-symbol identities.

## Exclusive directory

`fixtures/typescript/projects/declarations-scope-merging/`

## Difficulty

`routine`

## Assigned coverage

Only valid-source obligations are corpus content. Same-space collisions, illegal
lexical redeclaration, pre-initialization reference, conflicting merged property
types, forbidden merges, and downlevel-absence failures are excluded per ground
rules.

- TS-CAN-040 — one spelling with compatible type, value, and namespace
  declarations referenced in annotations, `typeof`, `new`, and qualified access;
  a class or enum as type and value; an interface as type only; namespace value
  and type facets; an import alias carrying multiple meanings.
- TS-CAN-041 — `var`, `let`, `const`, function, class, parameter, catch, block,
  and destructuring bindings with shadowing, closure capture, aliases, defaults,
  nested rest, and parameter patterns; legal `var` redeclaration; switch shared
  scope; loop capture under an old target; script `var` and function on
  `globalThis` versus global lexical bindings.
- TS-CAN-043 — reopening an interface across blocks and files; extending multiple
  bases; merging method overloads; augmenting a global or library interface;
  implementing the result in a class; specialized literal overloads.
- TS-CAN-044 — namespace merging with a function, class, and enum; class and
  interface instance merging; multiple enum declarations; correct runtime
  declaration ordering; exported versus block-private namespace members; value
  before namespace.
- TS-CAN-046 — literal computed keys; well-known symbols; a declared `unique
  symbol` shared between an interface and an object or class; a dynamic computed
  key that cannot name one fixed property; symbol index signatures; `keyof`
  inclusion; declaration emit; a unique-symbol brand.

## Declared compilation contexts

- Default context.
- A downlevel target for loop-capture (TS-CAN-041) and symbol library
  availability (TS-CAN-046).

## Dependency needs

None.

## Generated-source needs

None.

## Planned tests

`none`. Declaration, scope, and merging behavior is created by source and
configuration; no assigned coverage is created by test source.

## Required interfaces

- `Taskfile.yml` defines `build`, `lint`, and `test` as thin wrappers per ground
  rules; `build` compiles the default and downlevel contexts.
- `coverage.md` records the ground-rules coverage table for every assigned
  identifier, using the compilation-context column where non-default and
  additional locators for distinct valid variants.
