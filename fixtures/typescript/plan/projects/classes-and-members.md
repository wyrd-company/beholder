---
relationships:
  references:
    - ground-rules
    - reports/typescript/synthesis/checklist
---

# Project brief: classes-and-members

## Purpose

An object-oriented library that exercises the class model end to end: heritage,
abstractness, conformance and dispatch; fields, parameter properties, and
initialization; accessors and auto-accessors; member visibility including hard
private names; the static side and class expressions; overload declarations for
functions, methods, and constructors; and receiver semantics.

## Exclusive directory

`fixtures/typescript/projects/classes-and-members/`

## Difficulty

`complex`

## Assigned coverage

Only valid-source obligations are corpus content. Incompatible overloads,
missing members, read-before-initialization, `erasableSyntaxOnly` rejection,
declaration-origin incompatibility, detached-call, and private-constructor
subclassing failures are excluded per ground rules.

- TS-CAN-042 — ordered overload signatures with one broader implementation for a
  function, a method, and a constructor, with calls resolving to different return
  types; optional and rest candidates; interface-merging overloads; ambient
  overloads without implementation.
- TS-CAN-048 — arrow lexical `this`, ordinary function and method dynamic `this`,
  method extraction, explicit `this` parameters, `this: void`, contextual
  `ThisType`, and `.call`/`.bind` checking; `noImplicitThis`; constructor
  functions in JavaScript; static versus instance `this`.
- TS-CAN-049 — an abstract base, a concrete derived class, an overridden method,
  a base-typed polymorphic call, multiple implemented interfaces, an abstract
  construct signature, and `extends` of a class-producing expression; `override`
  and `noImplicitOverride`; `super`; protected and private assignability; the
  `implements`-erased versus `extends`-runtime edge.
- TS-CAN-050 — base and derived initialized and uninitialized fields; a field
  shadowing a base setter; definite assignment; a `declare` field; public,
  protected, private, and readonly parameter properties; `useDefineForClassFields`
  define-versus-assignment emit; the target-driven default; initialization
  around `super`.
- TS-CAN-051 — paired and unpaired getters and setters; getter-only readonly
  inference; asymmetric read and write types; an inherited accessor override; a
  static accessor; and an `accessor` field with its declaration and emit shape.
- TS-CAN-052 — public, protected, soft-private, and `#private` visibility in
  related and unrelated classes; subclass protected access; hard `#private`
  access and `#x in obj` narrowing; a private constructor; static private fields;
  and declaration emit.
- TS-CAN-053 — instance type `C` versus static `typeof C`; static fields,
  methods, private state, and blocks; static inheritance; anonymous and named
  class expressions passed or returned as values; overloaded constructors;
  protected or private construction; `InstanceType`; multiple static block order.

## Declared compilation contexts

- Default context.
- A downlevel target and a modern target for `useDefineForClassFields` and
  class-field lowering (TS-CAN-050) and auto-accessor emit (TS-CAN-051).
- An `erasableSyntaxOnly` context for the valid erasable forms of TS-CAN-050.

## Dependency needs

None.

## Generated-source needs

None.

## Planned tests

`none`. Class and member behavior is created by source and configuration; no
assigned coverage is created by test source.

## Required interfaces

- `Taskfile.yml` defines `build`, `lint`, and `test` as thin wrappers per ground
  rules; `build` compiles under each declared target and option context.
- `coverage.md` records the ground-rules coverage table for every assigned
  identifier, using the compilation-context column where non-default and
  additional locators for distinct valid variants.
