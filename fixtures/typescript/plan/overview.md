---
relationships:
  references:
    - ground-rules
    - reports/typescript/synthesis/checklist
---

# TypeScript fixture corpus plan overview

## Toolchain

The corpus pins one toolchain, made available offline in every project.

- TypeScript compiler: `typescript` `5.9.3` (the latest stable `5.9.x`; the
  checklist baseline). `7.0.x`/`tsgo` and `6.0.x` are excluded per the checklist
  and TS-CAN-092.
- Host runtime: Node.js `v26.5.1` on 64-bit x86 Linux.
- Package manager: npm `11.17.0`.
- Standard library: the `lib.*.d.ts` set bundled with the pinned compiler.
- Additional locally installed host: Bun `1.3.14`.

Effective compiler options are pinned per project through its committed
configuration; the plan does not fix option values beyond naming the
compilation contexts a project requires.

## Compilation contexts

The default compilation context is the pinned compiler and Node.js host on
64-bit x86 Linux under a current module, resolution, and target configuration
with strict checking.

The following additional contexts are locally available and declared by projects
that require them. No context requires an external service.

- Earlier pinned `typescript` release for a version-boundary comparison
  (vendored alongside the default compiler).
- Contrasting effective-option sets under the pinned compiler (target and
  library selection, strictness family, independent safety flags, check/emit
  decoupling, isolated processing).
- Module and resolution modes: `classic`/`node10`, `node16`/`nodenext`,
  `bundler`, CommonJS, preserved ECMAScript modules, and conditional legacy
  `outFile` concatenation.
- Per-file module formats across the TypeScript and JavaScript extension family.
- Host runtimes for runtime-native execution: Node.js type stripping and Bun.
- Non-`tsc` transform paths: the Bun transpiler and the pinned compiler API
  transformer surface.
- Solution build mode via `tsc -b` with incremental state.

## Projects

Nineteen projects own corpus coverage. Each occupies an exclusive directory
under `fixtures/typescript/projects/` and depends on no other fixture project.

1. `program-solution-workspace` — complex. A multi-package composite solution
   exercising program construction, configuration resolution, path and virtual
   root mapping, generated-source inclusion, build orchestration, and one source
   in multiple programs. Assigns TS-CAN-002, TS-CAN-003, TS-CAN-004, TS-CAN-005,
   TS-CAN-011, TS-CAN-016, TS-CAN-017.
2. `compiler-options-matrix` — complex. One library compiled under contrasting
   effective-option sets for target and library, strictness, independent safety
   flags, and check/emit decoupling. Assigns TS-CAN-006, TS-CAN-007, TS-CAN-008,
   TS-CAN-015.
3. `compiler-version-boundary` — complex. Program identity observed across a
   documented compiler-version boundary using two vendored compiler releases.
   Assigns TS-CAN-001.
4. `module-resolution-and-formats` — complex. The module and resolution matrix,
   the extension and per-file-format family, and file identity, casing, real
   paths, and symlinks. Assigns TS-CAN-009, TS-CAN-010, TS-CAN-012.
5. `esm-cjs-interop` — routine. ECMAScript and CommonJS interoperability, module
   detection, live bindings and cycles, `export =`/import-equals, and Universal
   Module Definition duality. Assigns TS-CAN-024, TS-CAN-025, TS-CAN-026,
   TS-CAN-030, TS-CAN-032.
6. `module-graph-edges` — complex. Import and export binding forms, type-only
   edges and elision, re-export forwarding and barrels, side-effect imports,
   dynamic import and import types, import attributes and JSON, and deferred
   module evaluation. Assigns TS-CAN-018, TS-CAN-019, TS-CAN-020, TS-CAN-021,
   TS-CAN-022, TS-CAN-023, TS-CAN-033.
7. `namespaces-ambient-globals` — routine. Internal namespaces and aliases,
   ambient modules, module and global augmentation, triple-slash directives with
   legacy concatenation, and ambient global declarations. Assigns TS-CAN-027,
   TS-CAN-028, TS-CAN-029, TS-CAN-031, TS-CAN-045.
8. `package-publishing-ecosystem` — complex. Package entry points and condition
   maps, package `imports` and self-references, declaration discovery and
   version routing, package-manager identity and workspace consumption,
   runtime/type entry-point agreement, platform variants and declaration
   rollups, declaration emit with maps, and dependency declaration quality.
   Assigns TS-CAN-014, TS-CAN-034, TS-CAN-035, TS-CAN-036, TS-CAN-037,
   TS-CAN-038, TS-CAN-039, TS-CAN-089.
9. `checked-javascript-inputs` — routine. Checked and unchecked JavaScript
   inputs, the TypeScript JSDoc dialect, inferred CommonJS shapes, and
   declaration emit from JavaScript. Assigns TS-CAN-013.
10. `classes-and-members` — complex. Class heritage and dispatch, fields and
    parameter properties, accessors and auto-accessors, member visibility, the
    static side and class expressions, function/method/constructor overloads,
    and receiver semantics. Assigns TS-CAN-042, TS-CAN-048, TS-CAN-049,
    TS-CAN-050, TS-CAN-051, TS-CAN-052, TS-CAN-053.
11. `decorators` — complex. Standard decorators and, in a separate mutually
    exclusive compilation, legacy decorators with emitted design metadata.
    Assigns TS-CAN-054, TS-CAN-055.
12. `jsx-tsx-components` — routine. TSX parsing and JSX transform modes, and JSX
    names, components, attributes, and alternative runtime typing surfaces.
    Assigns TS-CAN-056, TS-CAN-057.
13. `declarations-scope-merging` — routine. Separate type, value, and namespace
    spaces; lexical scope, hoisting, temporal dead zones, and binding patterns;
    interface, cross-kind, and enum declaration merging; and computed, symbol,
    and unique-symbol identities. Assigns TS-CAN-040, TS-CAN-041, TS-CAN-043,
    TS-CAN-044, TS-CAN-046.
14. `type-relations-and-compatibility` — routine. Primitive, literal, top, and
    bottom types; structural compatibility, freshness, and weak types; unions,
    intersections, and discriminated unions; nullability and optional
    distinctions; readonly views; and callable compatibility and variance.
    Assigns TS-CAN-058, TS-CAN-059, TS-CAN-060, TS-CAN-063, TS-CAN-065,
    TS-CAN-067.
15. `narrowing-inference-assertions` — complex. Control-flow narrowing and alias
    analysis, user-defined predicates and assertion signatures, contextual
    typing and widening, and assertions, definite assignment, and `satisfies`.
    Assigns TS-CAN-061, TS-CAN-062, TS-CAN-078, TS-CAN-079.
16. `generics-and-inference` — complex. Generic declarations, constraints,
    defaults, and inference; const type parameters, instantiation expressions,
    `NoInfer`, and variance annotations; index signatures; arrays, tuples, and
    variadic relations; hybrid object types; and polymorphic `this`. Assigns
    TS-CAN-064, TS-CAN-066, TS-CAN-068, TS-CAN-069, TS-CAN-076, TS-CAN-077.
17. `type-level-programming` — complex. Conditional types and `infer`, mapped
    types and key remapping, `keyof`, indexed access, and type queries, template
    literal types and intrinsic string transforms, recursive and circular types,
    interfaces versus type aliases, and utility types and value-derived chains.
    Assigns TS-CAN-070, TS-CAN-071, TS-CAN-072, TS-CAN-073, TS-CAN-074,
    TS-CAN-075, TS-CAN-080.
18. `runtime-constructs-and-emit` — complex. Enums and const-enum boundaries,
    async functions and thenables, generators and iteration protocols, optional
    chains, nullish coalescing, and logical assignment, prototypes and mixins,
    exceptions, reachability, and exhaustiveness, evaluation order and
    helper-mediated downlevel behavior, and explicit resource management.
    Assigns TS-CAN-047, TS-CAN-081, TS-CAN-082, TS-CAN-083, TS-CAN-084,
    TS-CAN-085, TS-CAN-086, TS-CAN-087.
19. `alternate-tools-and-hosts` — complex. Alternate transpilers, loaders, and
    compiler transformations, and runtime-native TypeScript on non-`tsc` hosts.
    Assigns TS-CAN-090, TS-CAN-091.

## Primary coverage index

Every in-scope canonical identifier has exactly one primary project. In-scope
count: 90 of 92 canonical identifiers. Exceptions: 2 (TS-CAN-088, TS-CAN-092).

| Canonical identifier | Primary project |
| --- | --- |
| TS-CAN-001 | compiler-version-boundary |
| TS-CAN-002 | program-solution-workspace |
| TS-CAN-003 | program-solution-workspace |
| TS-CAN-004 | program-solution-workspace |
| TS-CAN-005 | program-solution-workspace |
| TS-CAN-006 | compiler-options-matrix |
| TS-CAN-007 | compiler-options-matrix |
| TS-CAN-008 | compiler-options-matrix |
| TS-CAN-009 | module-resolution-and-formats |
| TS-CAN-010 | module-resolution-and-formats |
| TS-CAN-011 | program-solution-workspace |
| TS-CAN-012 | module-resolution-and-formats |
| TS-CAN-013 | checked-javascript-inputs |
| TS-CAN-014 | package-publishing-ecosystem |
| TS-CAN-015 | compiler-options-matrix |
| TS-CAN-016 | program-solution-workspace |
| TS-CAN-017 | program-solution-workspace |
| TS-CAN-018 | module-graph-edges |
| TS-CAN-019 | module-graph-edges |
| TS-CAN-020 | module-graph-edges |
| TS-CAN-021 | module-graph-edges |
| TS-CAN-022 | module-graph-edges |
| TS-CAN-023 | module-graph-edges |
| TS-CAN-024 | esm-cjs-interop |
| TS-CAN-025 | esm-cjs-interop |
| TS-CAN-026 | esm-cjs-interop |
| TS-CAN-027 | namespaces-ambient-globals |
| TS-CAN-028 | namespaces-ambient-globals |
| TS-CAN-029 | namespaces-ambient-globals |
| TS-CAN-030 | esm-cjs-interop |
| TS-CAN-031 | namespaces-ambient-globals |
| TS-CAN-032 | esm-cjs-interop |
| TS-CAN-033 | module-graph-edges |
| TS-CAN-034 | package-publishing-ecosystem |
| TS-CAN-035 | package-publishing-ecosystem |
| TS-CAN-036 | package-publishing-ecosystem |
| TS-CAN-037 | package-publishing-ecosystem |
| TS-CAN-038 | package-publishing-ecosystem |
| TS-CAN-039 | package-publishing-ecosystem |
| TS-CAN-040 | declarations-scope-merging |
| TS-CAN-041 | declarations-scope-merging |
| TS-CAN-042 | classes-and-members |
| TS-CAN-043 | declarations-scope-merging |
| TS-CAN-044 | declarations-scope-merging |
| TS-CAN-045 | namespaces-ambient-globals |
| TS-CAN-046 | declarations-scope-merging |
| TS-CAN-047 | runtime-constructs-and-emit |
| TS-CAN-048 | classes-and-members |
| TS-CAN-049 | classes-and-members |
| TS-CAN-050 | classes-and-members |
| TS-CAN-051 | classes-and-members |
| TS-CAN-052 | classes-and-members |
| TS-CAN-053 | classes-and-members |
| TS-CAN-054 | decorators |
| TS-CAN-055 | decorators |
| TS-CAN-056 | jsx-tsx-components |
| TS-CAN-057 | jsx-tsx-components |
| TS-CAN-058 | type-relations-and-compatibility |
| TS-CAN-059 | type-relations-and-compatibility |
| TS-CAN-060 | type-relations-and-compatibility |
| TS-CAN-061 | narrowing-inference-assertions |
| TS-CAN-062 | narrowing-inference-assertions |
| TS-CAN-063 | type-relations-and-compatibility |
| TS-CAN-064 | generics-and-inference |
| TS-CAN-065 | type-relations-and-compatibility |
| TS-CAN-066 | generics-and-inference |
| TS-CAN-067 | type-relations-and-compatibility |
| TS-CAN-068 | generics-and-inference |
| TS-CAN-069 | generics-and-inference |
| TS-CAN-070 | type-level-programming |
| TS-CAN-071 | type-level-programming |
| TS-CAN-072 | type-level-programming |
| TS-CAN-073 | type-level-programming |
| TS-CAN-074 | type-level-programming |
| TS-CAN-075 | type-level-programming |
| TS-CAN-076 | generics-and-inference |
| TS-CAN-077 | generics-and-inference |
| TS-CAN-078 | narrowing-inference-assertions |
| TS-CAN-079 | narrowing-inference-assertions |
| TS-CAN-080 | type-level-programming |
| TS-CAN-081 | runtime-constructs-and-emit |
| TS-CAN-082 | runtime-constructs-and-emit |
| TS-CAN-083 | runtime-constructs-and-emit |
| TS-CAN-084 | runtime-constructs-and-emit |
| TS-CAN-085 | runtime-constructs-and-emit |
| TS-CAN-086 | runtime-constructs-and-emit |
| TS-CAN-087 | runtime-constructs-and-emit |
| TS-CAN-088 | excepted (invalid-only) |
| TS-CAN-089 | package-publishing-ecosystem |
| TS-CAN-090 | alternate-tools-and-hosts |
| TS-CAN-091 | alternate-tools-and-hosts |
| TS-CAN-092 | excepted (research gap) |

## Exceptions

### Item exceptions

- TS-CAN-088 — Diagnostic phases, suppression, recovery, and version gates.
  Invalid-only item. Its obligation is isolated parse, binding, type,
  resolution, declaration-emit, and invalid-option errors, partial parser
  recovery, and suppression of errors, all of which are invalid source or
  diagnostic assertions the ground rules exclude. Its valid-source residue is
  already owned elsewhere: `@ts-check`/`@ts-nocheck` on checked JavaScript by
  TS-CAN-013, and version-gated valid behavior by TS-CAN-001. No distinct valid
  coverage remains for a primary assignment.
- TS-CAN-092 — Native compiler port and future official implementations.
  Research gap. The checklist classifies it as unspecified and an unresolved
  research gap and directs against treating TypeScript 6, TypeScript 7, or
  `tsgo` as interchangeable with `tsc`. The plan does not resolve research gaps.

### Variant exceptions (unsupported conditional contexts)

- TS-CAN-091 Deno resolution variant. Deno is not installed. The item stays in
  scope through the Node.js type-stripping and Bun host contexts, which are
  locally available.
- TS-CAN-037 pnpm isolation and store-link variant, and Yarn node-modules and
  Plug'n'Play variant. pnpm, Yarn, and corepack are not installed. The item
  stays in scope through npm hoisting, workspace links, peer and transitive type
  dependencies, two installed versions of a nominally sensitive declaration, and
  source-alias versus built-declaration consumption, all available under npm.
- TS-CAN-017 editor and language-service variants (auto-import source choice,
  language-service plugin effects, suggestion diagnostics absent from `tsc`,
  stale editor state, inferred editor project). The corpus validates through
  batch `tsc`, not a `tsserver` language-service host. The item stays in scope
  through its batch multi-program identity obligation.

### Research gaps not planned

The checklist's TS-GAP-001 through TS-GAP-009 are research gaps by definition
and are not assigned. Where a canonical item overlaps a gap, only the item's
in-scope conditional obligation is planned and the gap's exact-constraint
question is left unresolved. This applies to TS-CAN-033 relative to TS-GAP-001,
and to related items whose exact edge behavior the gaps track.

## Independence audit

- Each of the nineteen projects occupies one exclusive directory under
  `fixtures/typescript/projects/` matching its slug. No two projects share a
  directory.
- No project brief permits editing another project or a shared corpus file.
- No project depends on another top-level fixture project. Cross-cutting
  features that appear in several projects (module resolution, declaration
  emit, strictness, target and library selection) are each assigned one primary
  owner; incidental reuse of a language feature is natural overlap, not a
  project dependency.
- Dependency needs are confined within each project: the additional vendored
  compiler release is internal to `compiler-version-boundary`, and generated
  outputs are internal to `program-solution-workspace` and
  `package-publishing-ecosystem`.
