# TypeScript — Research Checklist for a Code-Graph Reference Project

This report enumerates TypeScript language, toolchain, build, package, and ecosystem
features whose omission from a reference project would leave materially distinct
program behavior unrepresented. It targets the semantic surface a correct code graph
must capture: entities, identities, relationships, resolution, types, visibility,
dispatch, source inclusion, and diagnostics. It does not assume any graph schema,
product, or analysis approach.

## 1. Baseline assumptions

- **Language/compiler baseline:** TypeScript 5.9, with `tsc` as the reference
  implementation. TypeScript has no maintained formal specification; the archived
  language specification describes roughly TypeScript 1.8 and is obsolete. Behavior
  is defined by the compiler, the handbook, and per-release notes.
- **Runtime/platform baseline:** Node.js 20/22 (LTS lines), npm as package manager.
  Browser/bundler consumption is treated as a first-class variant.
- **Strictness baseline:** `"strict": true` assumed unless an item says otherwise.
  Many items exist precisely because flags fork semantics.
- **Module baseline:** ES modules with `node16`/`nodenext` or `bundler` resolution;
  CommonJS interop remains pervasive and must be represented.
- **Editor tooling:** `tsserver` shares the compiler but adds behavior (inferred
  projects, plugins, auto-imports); divergences are flagged where relevant.
- **Near-future context:** the native-port compiler (TypeScript 7 / `tsgo` previews)
  targets behavioral parity with `tsc`; it is not the baseline here.

"Defined by" values used below: **ECMAScript** (TC39 semantics TypeScript inherits),
**TypeScript language** (documented, stable behavior of the language as implemented
by `tsc`), **tsc implementation** (behavior only pinned down by the implementation),
**Node.js** (resolution/format rules TypeScript models), **Ecosystem convention**.

## 2. Category overview

- A. Modules, imports, and exports (MOD)
- B. Module resolution and file inclusion (RES)
- C. Declarations, names, scoping, and merging (DECL)
- D. Type-system constructs (TYPE)
- E. Classes and object semantics (CLS)
- F. Functions and control flow (FUN)
- G. Enums, namespaces, and emit-bearing syntax (EMIT)
- H. JSX (JSX)
- I. Ambient declarations and global types (AMB)
- J. JavaScript interop and JSDoc (JS)
- K. Configuration-dependent semantics (CFG)
- L. Packages and distribution (PKG)
- M. Project structure and build (PROJ)
- N. Diagnostics, suppression, and invalid programs (DIAG)
- O. Alternative toolchains and runtimes (RUN)

---

## A. Modules, imports, and exports (MOD)

### MOD-01 — Import/export binding matrix
- **Behavior:** Named, default, namespace, and side-effect imports; named/default/star
  re-exports; aliasing via `as`; `export * as ns from`. Each form creates distinct
  binding identities; re-exports create alias edges without local bindings.
- **Show:** Files exercising every form, including alias chains (`export { a as b }`
  re-exported again as `c`) and a side-effect-only import of a module with top-level
  effects.
- **Variants/failures:** Two `export *` sources exporting the same name silently
  exclude it (importing it is an error); an explicit local export shadows a star
  export; `export *` never re-exports `default`.
- **Constraints:** ES module syntax requires the file to be a module (see MOD-06).
- **Defined by:** ECMAScript + TypeScript language. **Confidence:** high.
- **Source:** ECMAScript specification (modules); TypeScript Handbook, Modules chapter.

### MOD-02 — Default exports, including anonymous declarations
- **Behavior:** `export default class {}` and `export default function () {}` create
  nameless entities that still have identity; `export default interface I {}` is a
  legal type-only default; expressions can be default-exported; `export { default as X } from` re-binds a default.
- **Show:** Anonymous default class in one file, default interface in another,
  a re-export of a default under a new name.
- **Variants/failures:** A type alias cannot be default-exported with
  `export default A` (value-position reference); it needs `export { A as default }`
  (medium confidence on the exact diagnostic).
- **Defined by:** ECMAScript + TypeScript language. **Confidence:** high.
- **Source:** TypeScript Handbook, Modules chapter.

### MOD-03 — `export =` and `import x = require(...)`
- **Behavior:** TypeScript-specific syntax modeling a CommonJS single export. The
  classic pattern exports a class or function merged with a namespace. Consumers use
  import-equals, or a default import when `esModuleInterop` is on.
- **Show:** A `.cts` (or CommonJS-mode) module using `export =` with a
  class+namespace merge, consumed both via `import x = require("m")` and via
  `import m from "m"` under interop.
- **Variants/failures:** `export =` cannot coexist with other exports;
  `import ... = require(...)` is rejected when emitting pure ES modules; under
  `verbatimModuleSyntax` with CommonJS emit these forms become mandatory (ESM syntax
  is an error there).
- **Defined by:** TypeScript language. **Confidence:** high.
- **Source:** TypeScript Handbook, Modules Reference.

### MOD-04 — Type-only imports and exports
- **Behavior:** `import type`, `export type`, inline `type` specifiers
  (`import { type A, b }`), and type-only star re-exports (`export type * from`,
  `export type * as ns from`, TS 5.0). These create type-only edges erased at emit;
  names are usable only in type positions.
- **Show:** Each form, plus a mixed import with inline `type` modifiers where the
  value part survives emit and the type part does not.
- **Variants/failures:** `import type Default, { Named }` is illegal (default or
  named bindings, not both); a type-only-imported class cannot appear in `extends`;
  `isolatedModules` requires type-only syntax on re-exported types;
  `verbatimModuleSyntax` makes marking mandatory.
- **Defined by:** TypeScript language. **Confidence:** high.
- **Source:** TypeScript 3.8 / 4.5 / 5.0 release notes; Handbook, Modules Reference.

### MOD-05 — Import elision (type-directed emit)
- **Behavior:** Without `verbatimModuleSyntax`, an unmarked value import used only in
  type positions is dropped from emitted JavaScript. The dependency edge exists in
  the checked program but not at runtime, and the target module's side effects do
  not run.
- **Show:** Import of a side-effectful module referenced only as a type; emitted JS
  lacks the import. The same file under `verbatimModuleSyntax` keeps it.
- **Variants/failures:** `emitDecoratorMetadata` can force retention of imports that
  appear only in type annotations (classic dependency-injection pitfall, including
  spurious circular-import crashes).
- **Defined by:** TypeScript language / tsc implementation. **Confidence:** high.
- **Source:** TSConfig Reference (`verbatimModuleSyntax`); TypeScript 5.0 release notes.

### MOD-06 — Script vs module file semantics
- **Behavior:** A file with no import/export syntax is a global script: its top-level
  declarations join the program-wide global scope. `export {}` opts a file into
  module-ness. `moduleDetection` (`auto`/`force`/`legacy`) changes classification;
  under `auto`, `package.json` `"type": "module"` (with `node16`/`nodenext`) and JSX
  files under `react-jsx` also count as modules.
- **Show:** A script file contributing a global consumed elsewhere without imports;
  the same file with `export {}` added, breaking those references.
- **Variants/failures:** `.d.ts` files follow the same global/module split (see
  AMB-01); `isolatedModules` errors on global script files.
- **Defined by:** TypeScript language (detection rules are tsc-defined).
  **Confidence:** high (medium on the exact `auto` clause list).
- **Source:** TypeScript 4.7 release notes (`moduleDetection`); Handbook, Modules.

### MOD-07 — Dynamic import and import-type queries
- **Behavior:** `await import("m")` creates a lazy value edge. Type-level forms
  `import("m").T` and `typeof import("m")` create module edges with no import
  statement at all.
- **Show:** A lazily loaded module; a type annotation using `import("m").T` in a
  file that never imports `m`; a `typeof import("m")` module-shape type.
- **Variants/failures:** Resolution-mode attributes on import types
  (`import type ... with { "resolution-mode": "import" }`, stabilized TS 5.3) pick
  the ESM vs CJS view of a dual package.
- **Defined by:** ECMAScript (dynamic import) + TypeScript language (import types).
  **Confidence:** high (medium on resolution-mode details).
- **Source:** TypeScript 2.9 and 5.3 release notes.

### MOD-08 — `import.meta` and module-only syntax
- **Behavior:** `import.meta` (typed by the `ImportMeta` interface, augmentable) and
  top-level `await` are legal only under specific `module`/`target` combinations and
  themselves trigger module detection.
- **Show:** `import.meta.url` usage; an `ImportMeta` augmentation (e.g., typed
  `import.meta.env`-style fields); a top-level `await`.
- **Constraints:** Top-level await requires `module` es2022+/esnext/system/node16+
  and `target` ES2017+.
- **Defined by:** ECMAScript + TypeScript language. **Confidence:** high.
- **Source:** TypeScript 3.8/4.5 release notes; TSConfig Reference.

### MOD-09 — JSON modules and import attributes
- **Behavior:** `resolveJsonModule` gives typed imports of `.json` files with an
  inferred structural type. Import attributes (`with { type: "json" }`, TS 5.3;
  legacy `assert` from 4.5) are required by Node for JSON in ESM, and recent
  TypeScript validates this under `nodenext`.
- **Show:** A typed JSON import under CommonJS; the same under `nodenext` ESM with
  a `with { type: "json" }` attribute; code depending on the inferred JSON shape.
- **Variants/failures:** Under ESM, JSON modules expose only a default export;
  named-property imports fail. Exact version where TS began enforcing attributes:
  medium confidence (5.7 era).
- **Defined by:** Node.js + TypeScript language. **Confidence:** medium-high.
- **Source:** TypeScript 5.3/5.7 release notes; Node.js documentation, ECMAScript modules.

### MOD-10 — Arbitrary module-namespace identifier names
- **Behavior:** Exported names need not be identifiers:
  `export { x as "not an identifier" }`, `import { "not an identifier" as y }`
  (TS 5.6). Export identity must be string-keyed, not identifier-keyed.
- **Show:** A module exporting a string-named binding consumed by another file.
- **Defined by:** ECMAScript + TypeScript language. **Confidence:** medium-high.
- **Source:** TypeScript 5.6 release notes.

### MOD-11 — Ambient module declarations and wildcards
- **Behavior:** `declare module "m" { ... }` in a script/declaration context declares
  a module that has no corresponding file. Wildcard patterns (`declare module "*.css"`)
  type whole classes of non-TS imports. Shorthand `declare module "m";` makes all
  imports from it `any`.
- **Show:** A wildcard declaration for an asset extension plus an import of such an
  asset; a shorthand declaration rescuing an untyped dependency.
- **Variants/failures:** Only one `*` per pattern; ecosystem staples are CSS modules,
  images, and bundler client-type packages.
- **Defined by:** TypeScript language. **Confidence:** high.
- **Source:** TypeScript Handbook, Modules / Declaration Files chapters.

### MOD-12 — Module augmentation and global augmentation
- **Behavior:** Inside a module, `declare module "existing-specifier"` merges new or
  extended declarations into another module (e.g., adding fields to a framework's
  request/context interfaces). `declare global` inside a module contributes globals.
  Augmentations apply program-wide once the declaring file is in the program — the
  effective shape of a module depends on the file set.
- **Show:** An augmentation adding a property to an interface exported by another
  package; a `declare global` block augmenting a process-environment interface; a
  test showing behavior changes when the augmenting file is excluded.
- **Variants/failures:** Default exports cannot be augmented; the handbook documents
  augmentation as patching existing declarations rather than introducing arbitrary
  new top-level names (medium confidence on enforcement details).
- **Defined by:** TypeScript language. **Confidence:** high.
- **Source:** TypeScript Handbook, Declaration Merging chapter.

### MOD-13 — Triple-slash directives
- **Behavior:** `/// <reference path="..."/>` adds a file to the program;
  `/// <reference types="..."/>` adds an @types dependency (ecosystem staple for
  bundler client types); `/// <reference lib="..."/>` pulls in a lib file;
  `/// <reference no-default-lib="true"/>` marks a default-lib replacement. Only
  effective at the top of a file; declaration emit can synthesize some directives.
- **Show:** A `types` reference pulling ambient globals into scope; a `lib`
  reference enabling APIs the tsconfig `lib` omits.
- **Variants/failures:** Legacy AMD directives (`amd-module`, `amd-dependency`)
  exist but are obsolete.
- **Defined by:** TypeScript language. **Confidence:** high.
- **Source:** TypeScript Handbook, Triple-Slash Directives.

### MOD-14 — Deferred module evaluation (`import defer`)
- **Behavior:** `import defer * as ns from "m"` (TS 5.9; TC39 proposal) binds a
  namespace whose evaluation is deferred until first member access — a module edge
  whose side effects run lazily.
- **Show:** A deferred namespace import of a side-effectful module, with an
  observable ordering difference vs a normal import.
- **Constraints:** Namespace form only; supported only under `module` `esnext`/
  `preserve` (medium confidence on exact set).
- **Defined by:** ECMAScript proposal + TypeScript language. **Confidence:** medium.
- **Source:** TypeScript 5.9 release notes; TC39 proposal-defer-import-eval.

---

## B. Module resolution and file inclusion (RES)

### RES-01 — Resolution strategy matrix
- **Behavior:** `moduleResolution` `classic`/`node10`/`node16`/`nodenext`/`bundler`
  resolve identical specifiers differently: `exports`-map honoring (node16+/bundler)
  vs `main`/`types` (node10); mandatory extensions for relative ESM imports under
  `nodenext`; extensionless and directory-index imports allowed under node10/bundler.
- **Show:** One specifier that resolves under `bundler` but errors under `nodenext`;
  one package resolved through `exports` under node16 but through `main` under node10.
- **Variants/failures:** `classic` is legacy and rarely correct; mixing a Node-flavor
  `module` with a non-matching `moduleResolution` is a config error in recent versions.
- **Defined by:** tsc implementation modeling Node.js. **Confidence:** high.
- **Source:** TSConfig Reference; TypeScript Handbook, Module Resolution.

### RES-02 — Output-extension specifiers mapping to source files
- **Behavior:** Under node16+, relative imports use the *emitted* extension:
  `./util.js` resolves to `util.ts`; `.mjs`↔`.mts`, `.cjs`↔`.cts`. A graph must map
  specifier → source despite the extension mismatch. `allowImportingTsExtensions`
  (requires `noEmit`/`emitDeclarationOnly`) permits `.ts` specifiers;
  `rewriteRelativeImportExtensions` (TS 5.7) rewrites them at emit.
- **Show:** `.js`-suffixed relative imports across `.ts` files; a `.ts`-suffixed
  import under a no-emit bundler config.
- **Defined by:** tsc implementation. **Confidence:** high.
- **Source:** TypeScript 4.7 / 5.0 / 5.7 release notes.

### RES-03 — File-extension family and per-file module format
- **Behavior:** `.ts`/`.tsx`/`.mts`/`.cts` plus declaration variants
  `.d.ts`/`.d.mts`/`.d.cts`. Under node16+, `.mts`/`.cts` force ESM/CJS regardless of
  `package.json` `"type"`; `.ts` inherits from the nearest `package.json`. Format
  changes both emit and the interop rules applied at each import site.
- **Show:** A package mixing `.mts` and `.cts` with imports in both directions; the
  matching `.d.mts`/`.d.cts` outputs.
- **Variants/failures:** A `.ts` and sibling `.d.ts` with the same base name: the
  implementation file wins for imports (medium confidence on all orderings).
- **Defined by:** tsc implementation modeling Node.js. **Confidence:** high.
- **Source:** TypeScript 4.7 release notes; Handbook, Modules Reference.

### RES-04 — `paths`/`baseUrl` aliasing does not rewrite emit
- **Behavior:** `paths` remaps specifiers during type resolution only; emitted JS
  keeps the alias text. Runtime needs a bundler, loader, or post-processor. `paths`
  works without `baseUrl` in modern versions.
- **Show:** An `@app/*` alias resolving to `src/*`, plus the emitted JS showing the
  unrewritten specifier.
- **Variants/failures:** Aliases that shadow real packages; multiple fallback
  targets in one pattern array.
- **Defined by:** tsc implementation; runtime behavior is ecosystem convention.
  **Confidence:** high.
- **Source:** TSConfig Reference (`paths`).

### RES-05 — `rootDirs` virtual directory merge
- **Behavior:** Multiple directories are treated as one virtual directory so relative
  imports resolve across them (generated-code and platform-variant layouts) at check
  time; the runtime layout must actually match.
- **Show:** A source dir and a generated dir merged via `rootDirs`, with a relative
  import crossing them.
- **Defined by:** tsc implementation. **Confidence:** high.
- **Source:** TSConfig Reference (`rootDirs`).

### RES-06 — `moduleSuffixes` platform variants
- **Behavior:** Suffix lists (e.g., `.ios`, `.native`, then empty) make one specifier
  resolve to platform-selected files (TS 4.7; React Native convention) — one edge,
  multiple candidate targets depending on configuration.
- **Show:** `foo.ios.ts` and `foo.ts` both present, resolved differently under two
  configs.
- **Defined by:** tsc implementation; ecosystem convention (React Native).
  **Confidence:** high.
- **Source:** TypeScript 4.7 release notes.

### RES-07 — Declaration lookup for packages
- **Behavior:** For an npm dependency, types come from (in priority order) the
  `exports` map's `types` condition, then `types`/`typings` fields, then `index.d.ts`
  convention, then an `@types/*` fallback. Scoped packages mangle to
  `@types/scope__name`. `typesVersions` selects declaration sets by compiler version.
- **Show:** One dependency of each kind: bundled types via `exports`, bundled via
  `types` field, DefinitelyTyped-only, and an untyped one.
- **Variants/failures:** Untyped import → implicit `any` with a TS7016 diagnostic
  under `noImplicitAny`.
- **Defined by:** tsc implementation + ecosystem convention (DefinitelyTyped).
  **Confidence:** high.
- **Source:** TypeScript Handbook, Module Resolution / Publishing declarations.

### RES-08 — Automatic `@types` inclusion and the `types` option
- **Behavior:** All packages under `typeRoots` (default `node_modules/@types` up the
  directory tree) are auto-included as global program files — globals appear with no
  import anywhere. `"types": [...]`/`[]` restricts this.
- **Show:** A test-framework @types package contributing globals to files that never
  import it; a config with `types: []` removing them.
- **Variants/failures:** Two @types packages declaring the same globals (two test
  frameworks) collide with duplicate-identifier errors.
- **Defined by:** tsc implementation. **Confidence:** high.
- **Source:** TSConfig Reference (`types`, `typeRoots`).

### RES-09 — Program file-set construction
- **Behavior:** `files`/`include`/`exclude` define root files; every resolved import,
  triple-slash reference, and automatic @types inclusion joins the program
  transitively. `exclude` does not prevent inclusion via import. `node_modules` is
  excluded from `include` by default but joined through resolution.
- **Show:** An excluded file still entering the program because a root imports it;
  a file present on disk but absent from the program.
- **Variants/failures:** Same-file-different-casing across imports errors under
  `forceConsistentCasingInFileNames` (default on since 5.0) — case-insensitive
  filesystems mask it otherwise.
- **Defined by:** tsc implementation. **Confidence:** high.
- **Source:** TSConfig Reference (`include`/`exclude`/`files`).

### RES-10 — Symlinks and package-manager layouts
- **Behavior:** Resolution follows real paths by default (pnpm store, workspace
  symlinks dedupe to one file identity); `preserveSymlinks` flips this so one file
  can enter the program under two paths — producing duplicate, incompatible
  declarations of "the same" types.
- **Show:** A workspace-linked package resolved through a symlink; the same setup
  under `preserveSymlinks` producing type-identity mismatches.
- **Defined by:** tsc implementation modeling Node.js. **Confidence:** high.
- **Source:** TSConfig Reference (`preserveSymlinks`).

### RES-11 — Custom resolution conditions
- **Behavior:** `customConditions` (TS 5.0) adds user-defined condition names when
  resolving `exports`/`imports` maps; bundlers define their own (`browser`,
  `development`) — the same specifier can resolve to different typed surfaces per
  condition set.
- **Show:** A dependency whose `exports` map selects different `.d.ts` under a custom
  condition.
- **Constraints:** Only honored under node16/nodenext/bundler resolution.
- **Defined by:** tsc implementation + Node.js conditions model. **Confidence:** medium-high.
- **Source:** TypeScript 5.0 release notes; Node.js documentation, Packages.

---

## C. Declarations, names, scoping, and merging (DECL)

### DECL-01 — Three declaration spaces: value, type, namespace
- **Behavior:** One name can simultaneously denote a value, a type, and a namespace.
  A class is value+type; an enum is value+type; a namespace can merge with either.
  Resolution depends on syntactic position: `X` in a type annotation, `typeof X`,
  `new X()`, and `X.member` can bind different meanings of the same name.
- **Show:** `const X`, `interface X`, and `namespace X` coexisting, with uses in
  value, type, and namespace positions; a case where `typeof X` and `X` (type) differ.
- **Variants/failures:** Collisions within one space error (two classes named `X`);
  cross-space coexistence is legal. Graphs must decide whether this is one entity
  with facets or several linked entities — either way the links must exist.
- **Defined by:** TypeScript language. **Confidence:** high.
- **Source:** TypeScript Handbook, Declaration Merging chapter.

### DECL-02 — Interface declaration merging
- **Behavior:** Same-name interfaces merge across blocks, files, and with lib/global
  interfaces. The merged interface is a single logical entity with multiple
  declaration sites.
- **Show:** An interface declared in two files, plus a merge into a lib-declared
  global interface; members and overloads combined.
- **Variants/failures:** Non-identical type-parameter lists block merging;
  conflicting same-name members with different types error; method overloads from
  later declarations are ordered ahead of earlier ones (medium confidence on the
  exact ordering rule).
- **Defined by:** TypeScript language. **Confidence:** high.
- **Source:** TypeScript Handbook, Declaration Merging.

### DECL-03 — Namespace merging with functions, classes, enums
- **Behavior:** `namespace F` merges with `function F` (a callable with properties),
  with `class C` (static-side additions, nested types), and with `enum E`. The value
  declaration must precede the namespace for value merging.
- **Show:** A function+namespace pair exposing `F()` and `F.helper`; a class with a
  merged namespace providing `C.NestedType`.
- **Variants/failures:** Wrong ordering errors; merging with an arrow-function
  variable does not work (functions with properties on `const` need type-level
  modeling instead).
- **Defined by:** TypeScript language. **Confidence:** high.
- **Source:** TypeScript Handbook, Declaration Merging.

### DECL-04 — Enum declaration merging
- **Behavior:** Multiple `enum E` blocks merge into one enum; only the first can omit
  an initializer for its first member. Enums can also merge with namespaces.
- **Show:** An enum split across two declarations, one adding members with explicit
  initializers.
- **Defined by:** TypeScript language. **Confidence:** high.
- **Source:** TypeScript Handbook, Enums / Declaration Merging.

### DECL-05 — Hoisting, temporal dead zone, shadowing
- **Behavior:** `var` function-scoping vs `let`/`const` block-scoping; function
  declarations hoist; classes have a TDZ; block-level shadowing rebinds names; in a
  global script, `var`/`function` become `globalThis` properties while `let`/`const`
  are global-scope-only bindings.
- **Show:** Shadowed names across nested blocks; a use-before-declaration TDZ error;
  a script-file `var` visible via `globalThis`.
- **Defined by:** ECMAScript. **Confidence:** high.
- **Source:** ECMAScript specification (declarations and scoping).

### DECL-06 — Namespaces, dotted names, and namespace aliases
- **Behavior:** `namespace A.B.C` sugar for nesting; `import Alias = A.B.C` creates a
  namespace alias usable in both value and type positions; `export import` re-exports
  an alias from inside a namespace; legacy `module A {}` keyword form still parses.
- **Show:** A nested namespace tree, an alias shortening a deep path, an
  `export import` chain.
- **Defined by:** TypeScript language. **Confidence:** high.
- **Source:** TypeScript Handbook, Namespaces.

### DECL-07 — Ambient value declarations
- **Behavior:** `declare var/let/const/function/class/enum/namespace` assert existence
  of runtime entities provided externally (scripts, host environments); no emit.
  `declare const enum` additionally affects inlining (see EMIT-02).
- **Show:** A `declare function` for a host-provided global, used and emitted as a
  bare reference.
- **Variants/failures:** Ambient classes assert both type and constructor value;
  mismatches with the real runtime are invisible to the checker.
- **Defined by:** TypeScript language. **Confidence:** high.
- **Source:** TypeScript Handbook, Declaration Files.

### DECL-08 — Destructuring binding identities
- **Behavior:** Object/array destructuring with renaming, defaults, and rest creates
  multiple named bindings from one initializer, each with its own type; nested
  patterns and parameter destructuring included.
- **Show:** `const { a: renamed = fallback, ...rest } = obj` plus a destructured
  parameter with a type annotation on the whole pattern.
- **Defined by:** ECMAScript + TypeScript typing rules. **Confidence:** high.
- **Source:** TypeScript Handbook, Variable Declarations.

### DECL-09 — `unique symbol` and symbol-keyed members
- **Behavior:** `const s: unique symbol` creates a type inhabited only by that
  declaration; members can be keyed by symbols and well-known symbols
  (`[Symbol.iterator]`), so member identity ties to a symbol declaration rather than
  a string name.
- **Show:** A class implementing `[Symbol.iterator]`; an interface keyed by a local
  `unique symbol`; a brand field using it.
- **Variants/failures:** `unique symbol` requires `const`/`readonly static`
  declarations.
- **Defined by:** TypeScript language. **Confidence:** high.
- **Source:** TypeScript 2.7 release notes; Handbook, Symbols.

### DECL-10 — `using` and `await using` declarations
- **Behavior:** Explicit resource management (TS 5.2, TC39 proposal): `using` binds a
  value whose `[Symbol.dispose]` runs at block exit; `await using` for async dispose.
  Adds new declaration kinds with scope-exit semantics and emit helpers.
- **Show:** A `using` declaration on a Disposable, disposal order across nested
  blocks, an `await using` in an async function.
- **Constraints:** Needs `Symbol.dispose` (lib esnext.disposable or polyfill);
  downlevel emit uses try/finally machinery.
- **Defined by:** ECMAScript proposal + TypeScript language. **Confidence:** high.
- **Source:** TypeScript 5.2 release notes; TC39 proposal-explicit-resource-management.

---

## D. Type-system constructs (TYPE)

### TYPE-01 — Interfaces with call, construct, and index signatures
- **Behavior:** Interfaces describe callable, newable, indexable, and hybrid shapes
  (callable-with-properties). Construct signatures type class-like values; string and
  number index signatures interact with named members.
- **Show:** A hybrid interface (call signature + properties), a construct-signature
  type used to pass a class as a value, string+number index signatures together.
- **Variants/failures:** Number index type must be assignable to string index type;
  `interface extends` from multiple bases; an interface extending a class captures
  its shape including private/protected members, making it implementable only within
  that class hierarchy.
- **Defined by:** TypeScript language. **Confidence:** high.
- **Source:** TypeScript Handbook, Objects / Interfaces.

### TYPE-02 — Type aliases vs interfaces
- **Behavior:** Aliases name any type (unions, tuples, conditionals, primitives) and
  never merge; interfaces merge and are always object shapes. Aliases may or may not
  be preserved in hovers and declaration emit — the "same" type can surface under an
  alias name or fully expanded.
- **Show:** A recursive alias; an alias and an interface for the same shape used
  interchangeably; declaration emit showing alias preservation or expansion.
- **Defined by:** TypeScript language (preservation details: tsc implementation).
  **Confidence:** high.
- **Source:** TypeScript Handbook, Everyday Types.

### TYPE-03 — Unions, intersections, discriminated unions
- **Behavior:** Union/intersection algebra with reduction: disjoint literal
  intersections collapse to `never`; unions absorb subtypes (a literal unioned with
  its base disappears). Discriminated unions drive narrowing and exhaustiveness.
- **Show:** A tagged union with a literal discriminant, switch-based narrowing,
  a reduced union demonstrating absorption.
- **Variants/failures:** Reduction means the *displayed/queried* type may differ from
  the written one — graphs recording types must pick written vs computed.
- **Defined by:** TypeScript language. **Confidence:** high.
- **Source:** TypeScript Handbook, Narrowing / Everyday Types.

### TYPE-04 — Literal types, widening, `as const`
- **Behavior:** Literal types with context-dependent widening (`const x = "a"` is
  `"a"`, `let x = "a"` widens to `string`); `as const` freezes literals, makes
  arrays readonly tuples, and suppresses widening.
- **Show:** The same initializer under `let`, `const`, and `as const`, with
  downstream code depending on each shape.
- **Defined by:** TypeScript language. **Confidence:** high.
- **Source:** TypeScript 3.4 release notes; Handbook, Literal Types.

### TYPE-05 — Template literal types and intrinsic string types
- **Behavior:** Template literal types build and match string patterns at the type
  level; `Uppercase`/`Lowercase`/`Capitalize`/`Uncapitalize` are compiler-intrinsic
  aliases; inference from template patterns in conditional types extracts parts.
- **Show:** An event-name scheme `` `on${Capitalize<K>}` `` used in mapped-type key
  remapping; a parser-like conditional extracting a substring type.
- **Defined by:** TypeScript language (intrinsics: tsc implementation).
  **Confidence:** high.
- **Source:** TypeScript 4.1 release notes.

### TYPE-06 — Tuple types: optional, rest, variadic, labeled
- **Behavior:** Tuples support optional elements, rest elements (including leading/
  middle rest since 4.2), variadic spreads of type variables (`[...T]`), element
  labels, and `readonly` forms — all changing arity/assignability semantics.
- **Show:** A variadic `concatArgs<[...A, ...B]>`-style signature; labeled tuple
  parameters surfacing in signatures; readonly tuples rejecting mutation.
- **Defined by:** TypeScript language. **Confidence:** high.
- **Source:** TypeScript 4.0/4.2 release notes.

### TYPE-07 — Generics: constraints, defaults, instantiation, variance
- **Behavior:** Type parameters on functions/classes/interfaces/aliases with
  `extends` constraints and `=` defaults; instantiation expressions create
  specialized function *values* (`const g = f<string>`, TS 4.7); explicit variance
  annotations `in`/`out` (4.7); `const` type parameters (5.0) infer literal-ish
  types; `NoInfer<T>` (5.4) blocks inference sites.
- **Show:** Each mechanism in isolation plus one API combining constraint + default +
  `const` parameter; an instantiation expression passed as a callback.
- **Variants/failures:** Variance annotations are checked against structure (wrong
  annotations error); measured vs annotated variance can diverge on complex types.
- **Defined by:** TypeScript language. **Confidence:** high.
- **Source:** TypeScript 4.7 / 5.0 / 5.4 release notes.

### TYPE-08 — Conditional types and `infer`
- **Behavior:** `T extends U ? X : Y` with distribution over naked type parameters;
  `infer` binds intermediate types (with `extends` constraints on `infer` since 4.7);
  recursion supports deep transformations subject to implementation depth limits.
- **Show:** A distributive conditional over a union vs the `[T] extends [U]`
  non-distributive form; an `infer`-based `Awaited`-like unwrapping type.
- **Variants/failures:** "Type instantiation is excessively deep" — depth limits are
  implementation-defined and version-drifting.
- **Defined by:** TypeScript language (limits: tsc implementation). **Confidence:** high.
- **Source:** TypeScript 2.8 / 4.7 release notes.

### TYPE-09 — Mapped types and key remapping
- **Behavior:** `{ [K in keyof T]: ... }` with homomorphic modifier preservation,
  `+/-` on `readonly`/`?`, and `as`-clause key remapping (filtering keys via `never`,
  renaming via template literals).
- **Show:** A `Mutable<T>` (`-readonly`), a getters-from-props type using remapping,
  a key-filtered projection.
- **Defined by:** TypeScript language. **Confidence:** high.
- **Source:** TypeScript 4.1 release notes; Handbook, Mapped Types.

### TYPE-10 — `keyof`, `typeof`, indexed access
- **Behavior:** `typeof value` creates value→type edges (types derived from runtime
  declarations); `typeof import("m")` types a whole module; `keyof` and `T[K]`
  project structure; combinations (`keyof typeof obj`) are idiomatic.
- **Show:** A config object whose keys type an API via `keyof typeof`; an indexed
  access chain into nested structure.
- **Defined by:** TypeScript language. **Confidence:** high.
- **Source:** TypeScript Handbook, Typeof / Indexed Access Types.

### TYPE-11 — `satisfies` operator
- **Behavior:** Checks an expression against a type without changing its inferred
  type (TS 4.9) — validation edge with no type ascription; composes with `as const`.
- **Show:** A record checked with `satisfies` whose per-key literal types remain
  usable afterward (vs the same code with a type annotation losing them).
- **Defined by:** TypeScript language. **Confidence:** high.
- **Source:** TypeScript 4.9 release notes.

### TYPE-12 — Type predicates and assertion functions
- **Behavior:** `x is T` return types make calls narrow arguments; `asserts x` /
  `asserts x is T` narrow after-call control flow; since 5.5 suitable boolean-return
  functions get *inferred* predicates with no annotation — call-site narrowing
  appears without syntactic marker.
- **Show:** An explicit guard, an assertion function, and a 5.5-style inferred
  predicate (e.g., a `filter(x => x != null)` producing a narrowed array).
- **Variants/failures:** Assertion calls require the callee to have an explicit type
  annotation/named reference; inferred predicates changed observable types of
  existing code when 5.5 landed.
- **Defined by:** TypeScript language. **Confidence:** high.
- **Source:** TypeScript 3.7 / 5.5 release notes.

### TYPE-13 — `this` typing
- **Behavior:** `this` parameters (pseudo-parameters erased at emit — declared arity
  differs from runtime arity), polymorphic `this` return types (fluent builders
  preserved through subclassing), and `ThisType<T>` contextual `this` (options-object
  APIs).
- **Show:** A fluent base class subclassed with chained calls keeping the subtype;
  a function with a `this` parameter bound and passed around; a `ThisType`-driven
  options object with methods seeing a synthetic `this`.
- **Defined by:** TypeScript language. **Confidence:** high.
- **Source:** TypeScript Handbook, `this` types; TypeScript 2.0/2.3 release notes.

### TYPE-14 — Structural assignability and freshness
- **Behavior:** Compatibility is structural; nominal identity mostly does not exist.
  Excess property checks apply only to "fresh" object literals; weak-type checks
  catch all-optional-target mismatches; an empty interface accepts nearly anything.
- **Show:** A literal rejected for an excess property where an intermediate variable
  is accepted; two independently declared identical interfaces used interchangeably.
- **Variants/failures:** This is the central trap for graphs assuming nominal type
  identity; see TYPE-18 for the nominal exceptions.
- **Defined by:** TypeScript language. **Confidence:** high.
- **Source:** TypeScript Handbook, Type Compatibility.

### TYPE-15 — `any`, `unknown`, `never`, `void`
- **Behavior:** `any` disables checking and propagates virally; `unknown` is the safe
  top type requiring narrowing; `never` is the bottom type used for exhaustiveness
  and unreachable code; `void`-returning function types accept value-returning
  implementations (callback convention).
- **Show:** An `any` leak crossing several files; an `unknown` forced through
  narrowing; a `never` exhaustiveness check; `Array.prototype.forEach`-style void
  callback receiving a value-returning arrow.
- **Defined by:** TypeScript language. **Confidence:** high.
- **Source:** TypeScript Handbook, type basics; FAQ on void-returning callbacks.

### TYPE-16 — Values as type sources (inference-chain libraries)
- **Behavior:** Idiomatic libraries derive types from runtime values through
  generics — schema builders whose `Infer<typeof schema>`-style aliases produce
  types with no syntactic declaration anywhere. Type entities exist only as
  instantiation results.
- **Show:** A small schema-builder in-project (no external dependency needed) whose
  inferred output type is consumed across files; declaration emit of such values.
- **Variants/failures:** Declaration emit can fail with "cannot be named" portability
  errors when private types leak; `isolatedDeclarations` (5.5) rejects such exports
  without explicit annotations.
- **Defined by:** TypeScript language + ecosystem convention. **Confidence:** high.
- **Source:** TypeScript Handbook (declaration emit); 5.5 release notes
  (`isolatedDeclarations`).

### TYPE-17 — Lib-provided utility types and global type entities
- **Behavior:** `Partial`, `Pick`, `Record`, `ReturnType`, `Awaited`, etc. are
  ordinary aliases declared in bundled lib files — global type entities resolved
  like any other, and absent/different under modified `lib` settings.
- **Show:** Utility-type usage plus a resolution trail to lib declarations; a shadowing
  local `Partial` demonstrating that they are not keywords.
- **Defined by:** tsc implementation (lib contents ship with the compiler).
  **Confidence:** high.
- **Source:** TypeScript Handbook, Utility Types; bundled `lib.*.d.ts` files.

### TYPE-18 — Nominal-behavior islands
- **Behavior:** Exceptions to structural typing: enums are nominal (two identical
  enums are incompatible); class private/protected members make otherwise-identical
  classes incompatible; `unique symbol` brands create nominal-style tags; branded
  primitive patterns (`string & { __brand: ... }`) are ecosystem convention.
- **Show:** Two structurally identical enums failing assignment; two classes with a
  private field failing; a branded ID type rejecting a raw string.
- **Variants/failures:** Numeric-enum/number assignability rules tightened around
  5.0 (all enums became union enums; out-of-range literals rejected) — medium
  confidence on exact boundaries.
- **Defined by:** TypeScript language. **Confidence:** high (medium on 5.0 enum edges).
- **Source:** TypeScript Handbook, Enums; TypeScript 5.0 release notes.

---

## E. Classes and object semantics (CLS)

### CLS-01 — Heritage clauses: `extends` vs `implements`
- **Behavior:** `extends` is both a type edge and a runtime edge (prototype chain);
  `implements` is type-only and fully erased. `extends` accepts arbitrary
  expressions (mixin factories), not just class names.
- **Show:** A class extending another and implementing two interfaces; emitted JS
  showing `implements` gone; a `class extends factory(Base)` expression heritage.
- **Variants/failures:** Graphs must distinguish type-only vs value edges here or
  they will invent runtime dependencies (or miss them).
- **Defined by:** ECMAScript (`extends`) + TypeScript language (`implements`).
  **Confidence:** high.
- **Source:** TypeScript Handbook, Classes.

### CLS-02 — Modifier privacy vs ECMAScript `#private`
- **Behavior:** `private`/`protected` are compile-time-only (erased; still
  structurally significant, see TYPE-18); `#fields` are runtime-enforced with
  per-class brand identity; `#x in obj` performs a narrowing brand check (TS 4.5).
- **Show:** Both privacy systems in one class; a static `isInstance` using `#x in
  obj`; declaration emit representing `#private` presence.
- **Variants/failures:** Modifier-private members are visible in `.d.ts` and in
  structural comparisons; `#private` members are not accessible even via casts.
- **Defined by:** TypeScript language (`private`) + ECMAScript (`#`).
  **Confidence:** high.
- **Source:** TypeScript Handbook, Classes; TypeScript 4.5 release notes.

### CLS-03 — Parameter properties
- **Behavior:** `constructor(private readonly x: T)` declares and assigns a member in
  one token — a class member with no field declaration site in the class body.
- **Show:** A service class using only parameter properties, with cross-file member
  access.
- **Variants/failures:** Banned under `erasableSyntaxOnly` (5.8) and unsupported by
  Node type stripping; interacts with `useDefineForClassFields` initialization order.
- **Defined by:** TypeScript language. **Confidence:** high.
- **Source:** TypeScript Handbook, Classes.

### CLS-04 — Static side vs instance side
- **Behavior:** Every class yields two types: the instance type `C` and the static
  type `typeof C` (with construct signature). Static members, static blocks
  (TS 4.4), and static inheritance live on the latter; static members cannot
  reference class type parameters.
- **Show:** A generic class with statics; `InstanceType<typeof C>`; a static
  initialization block touching a `#private` field.
- **Defined by:** TypeScript language + ECMAScript (static blocks).
  **Confidence:** high.
- **Source:** TypeScript Handbook, Classes; 4.4 release notes.

### CLS-05 — Abstract classes and abstract construct signatures
- **Behavior:** `abstract` classes cannot be instantiated; abstract methods/accessors
  bind to subclass implementations (dispatch relevance); `abstract new () => T`
  types accept abstract classes as values (mixin-safe factories, TS 4.2).
- **Show:** An abstract base with an abstract method dispatched through a base-typed
  reference; a function accepting `abstract new` constructor types.
- **Defined by:** TypeScript language. **Confidence:** high.
- **Source:** TypeScript Handbook, Classes; 4.2 release notes.

### CLS-06 — Accessors and auto-accessors
- **Behavior:** `get`/`set` pairs form one member with potentially asymmetric types
  (setter wider than getter, TS 4.3; fully unrelated with explicit annotations,
  TS 5.1). `accessor` fields (TS 4.9) desugar to a private backing store plus
  get/set — one declaration, multiple emitted members.
- **Show:** A property accepting `string | number` in its setter but returning
  `number`; an `accessor` field, including its `.d.ts` representation.
- **Defined by:** TypeScript language + ECMAScript (accessor is decorators-proposal
  syntax). **Confidence:** high.
- **Source:** TypeScript 4.3 / 4.9 / 5.1 release notes.

### CLS-07 — `override` and `noImplicitOverride`
- **Behavior:** `override` marks intentional overriding, checked against the base;
  `noImplicitOverride` makes the marker mandatory — configuration changes which
  programs are valid (TS 4.3).
- **Show:** A correct override, a typo'd method name caught by `override`, the same
  file failing only when the flag is on.
- **Defined by:** TypeScript language. **Confidence:** high.
- **Source:** TypeScript 4.3 release notes.

### CLS-08 — Standard (TC39) decorators
- **Behavior:** TS 5.0 implements the stage-3 proposal: decorators on classes,
  methods, getters/setters, fields, and auto-accessors; `addInitializer`; defined
  evaluation vs application order; decorator metadata via `Symbol.metadata`
  (TS 5.2). No parameter decorators. Active only when `experimentalDecorators` is
  OFF.
- **Show:** A class using each decorator target; a metadata-writing decorator read
  back via `Symbol.metadata`; ordering demonstrated with logging decorators.
- **Variants/failures:** `Symbol.metadata` needs lib/polyfill support; decorators
  make classes runtime-transformed — the declared and runtime shapes can diverge.
- **Defined by:** ECMAScript proposal + TypeScript language. **Confidence:** high.
- **Source:** TypeScript 5.0 / 5.2 release notes; TC39 proposal-decorators.

### CLS-09 — Legacy decorators and `emitDecoratorMetadata`
- **Behavior:** `experimentalDecorators` selects the older, incompatible decorator
  semantics, including parameter decorators. `emitDecoratorMetadata` emits
  `design:type`/`design:paramtypes`/`design:returntype` — *type annotations become
  runtime values*, requiring a metadata-reflection polyfill and turning type-only
  references into value dependencies (see MOD-05).
- **Show:** The same decorated class compiled under both decorator regimes with
  different emit; a dependency-injection-style constructor whose parameter types
  appear in emitted metadata.
- **Constraints:** Ecosystem pillars (Angular, NestJS, TypeORM-style frameworks)
  still assume the legacy flags; the two regimes cannot be mixed in one compilation.
- **Defined by:** tsc implementation (legacy design). **Confidence:** high.
- **Source:** TypeScript Handbook, Decorators (legacy); TSConfig Reference.

### CLS-10 — Mixin pattern
- **Behavior:** Functions taking `TBase extends new (...args: any[]) => object` and
  returning `class extends Base` produce synthetic class hierarchies; instance types
  become intersections; hierarchy edges exist only through generic instantiation.
- **Show:** Two mixins composed over a base, with members from all layers used and
  `instanceof` behavior demonstrated.
- **Variants/failures:** Declaration emit of mixin results is a known difficulty
  (anonymous class types); abstract bases need `abstract new` (CLS-05).
- **Defined by:** TypeScript language + ecosystem convention. **Confidence:** high.
- **Source:** TypeScript Handbook, Mixins.

### CLS-11 — `useDefineForClassFields` semantics fork
- **Behavior:** Class fields emit as `[[Define]]` (ECMAScript standard) or `[[Set]]`
  (legacy TS) depending on the flag, whose default flips with `target` ≥ ES2022.
  Changes observable behavior: fields shadowing base-class accessors, initialization
  order, `undefined` stamping of declared-but-uninitialized fields.
- **Show:** A subclass field shadowing a base accessor behaving differently under
  each setting; a `declare` field modifier avoiding emit for augmentation-typed
  fields.
- **Defined by:** ECMAScript + tsc implementation (flag/default). **Confidence:** high.
- **Source:** TypeScript 3.7 release notes; TSConfig Reference.

### CLS-12 — Class expressions and classes as values
- **Behavior:** Anonymous class expressions, classes stored in variables, passed as
  arguments, returned from functions; identity/naming of such entities is
  positional rather than declarative.
- **Show:** A registry mapping names to class expressions; a function returning a
  locally declared class consumed elsewhere.
- **Defined by:** ECMAScript + TypeScript language. **Confidence:** high.
- **Source:** TypeScript Handbook, Classes.

### CLS-13 — Constructor visibility and overloads
- **Behavior:** Overloaded constructors (multiple signatures, one implementation);
  `private constructor` blocks both `new` and `extends` outside the class
  (singleton/factory idioms); `protected constructor` blocks external `new` only.
- **Show:** A singleton with a private constructor and static accessor; an
  overloaded constructor selected differently by two call sites.
- **Defined by:** TypeScript language. **Confidence:** high.
- **Source:** TypeScript Handbook, Classes.

### CLS-14 — Class merging with interfaces and namespaces
- **Behavior:** `class C` + `interface C` merges members into the instance type
  without implementations (mixin-application typing, augmentation patterns);
  `class C` + `namespace C` adds statics/nested types (see DECL-03).
- **Show:** A class whose interface merge adds mixin-applied members that the checker
  accepts on instances.
- **Variants/failures:** The merged members have no implementation site — a graph
  linking "member → defining code" must handle memberless declarations.
- **Defined by:** TypeScript language. **Confidence:** high.
- **Source:** TypeScript Handbook, Declaration Merging.

---

## F. Functions and control flow (FUN)

### FUN-01 — Function and method overloads
- **Behavior:** N overload signatures plus one implementation signature; the
  implementation signature is invisible to callers; resolution picks the first
  matching overload in declaration order. Overloads also appear on methods,
  constructors, and merged interfaces (augmentation can add overloads from another
  file).
- **Show:** An overloaded function whose two call sites resolve to different
  signatures with different return types; an interface merge contributing an
  additional overload.
- **Variants/failures:** Implementation-signature calls are errors; overload order
  changes results (more-specific-first convention); declaration emit preserves all
  signatures.
- **Defined by:** TypeScript language. **Confidence:** high.
- **Source:** TypeScript Handbook, Functions (overloads).

### FUN-02 — Async functions, generators, async generators
- **Behavior:** `async`/`await` with `Awaited`-based unwrap semantics; `function*`
  and `async function*` typed by `Generator`/`AsyncGenerator` type arguments (yield,
  return, next types all flow); `for await`. Emit differs radically by `target`
  (native vs state-machine downlevel; `downlevelIteration` for ES5 iteration).
- **Show:** A generator whose `next()` argument type is consumed; an async generator
  driven by `for await`; the same code emitted for an old and a new target.
- **Constraints:** Iterator-protocol *types* changed in 5.6 (built-in iterator
  helpers, stricter `IteratorObject` types).
- **Defined by:** ECMAScript + TypeScript language. **Confidence:** high.
- **Source:** TypeScript Handbook, Generators; 2.3 / 5.6 release notes.

### FUN-03 — `this` binding and `noImplicitThis`
- **Behavior:** Arrow functions capture lexical `this`; `function` gets dynamic
  `this` typed by `this` parameters or contextually; `noImplicitThis` errors on
  untyped dynamic `this`. Utility types `ThisParameterType`/`OmitThisParameter`
  manipulate it.
- **Show:** An object method losing `this` when extracted (caught by typing); an
  event-handler API with a declared `this` parameter.
- **Defined by:** ECMAScript (runtime) + TypeScript language (typing).
  **Confidence:** high.
- **Source:** TypeScript Handbook, Functions.

### FUN-04 — Narrowing mechanisms
- **Behavior:** Control-flow narrowing via `typeof`, `instanceof` (including
  `Symbol.hasInstance`-defined, TS 5.3), `in`, truthiness, equality, discriminant
  property checks, aliased condition variables and destructured discriminants
  (4.4), `switch (true)` (5.3), closure narrowing preserved past last assignment
  (5.4), and indexed-access narrowing `obj[key]` with a constant key (5.5).
- **Show:** One function per mechanism, plus one combining an aliased condition with
  a later branch; a version-sensitive case (e.g., `switch (true)`).
- **Variants/failures:** Narrowing results are computed types with no syntax — a
  graph reporting the type of a reference must decide flow-sensitive vs declared.
- **Defined by:** TypeScript language. **Confidence:** high.
- **Source:** TypeScript Handbook, Narrowing; 4.4 / 5.3 / 5.4 / 5.5 release notes.

### FUN-05 — Exhaustiveness and assertion-driven flow
- **Behavior:** `never` checks in `default` branches enforce exhaustiveness over
  discriminated unions; assertion functions (`asserts`) terminate/branch control
  flow at call sites — a callee changes the caller's reachable states.
- **Show:** An exhaustive switch that breaks compilation when a union member is
  added; an `assertIsDefined` helper whose call unlocks property access below it.
- **Defined by:** TypeScript language. **Confidence:** high.
- **Source:** TypeScript Handbook, Narrowing; 3.7 release notes.

### FUN-06 — Non-null and definite-assignment assertions
- **Behavior:** Postfix `!` strips `null | undefined` without runtime effect;
  `let x!: T` / field `name!: T` suppress definite-assignment and
  `strictPropertyInitialization` checks — programmer-asserted facts the checker
  trusts.
- **Show:** A `!` on a lookup result; a class field initialized in a lifecycle
  method with a `!` declaration.
- **Variants/failures:** These are erased at emit; wrong assertions are latent
  runtime failures — semantically "valid program, unsound claim".
- **Defined by:** TypeScript language. **Confidence:** high.
- **Source:** TypeScript 2.0 / 2.7 release notes.

### FUN-07 — Optional chaining, nullish coalescing, logical assignment
- **Behavior:** `?.` (with short-circuiting of entire tails, including optional
  calls `f?.()`), `??`, and `&&=`/`||=`/`??=` — all with narrowing effects and
  target-dependent downlevel emit.
- **Show:** A chained `a?.b?.[c]?.()`; emitted output under an old target showing
  helper expansion; narrowing after `??=`.
- **Defined by:** ECMAScript + TypeScript language. **Confidence:** high.
- **Source:** TypeScript 3.7 / 4.0 release notes.

---

## G. Enums, namespaces, and emit-bearing syntax (EMIT)

### EMIT-01 — Enum runtime semantics
- **Behavior:** Enums are real runtime objects. Numeric enums get reverse mappings
  (`E[E.A] === "A"`); string enums do not; heterogeneous enums mix both. Members are
  constant (computable at compile time) or computed; members have literal types and
  the enum is a union of them.
- **Show:** A numeric enum with reverse-mapping use; a string enum; a computed
  member; narrowing over enum members.
- **Variants/failures:** Enum nominality (TYPE-18); const-expression rules for
  initializers; declaration emit of enums.
- **Defined by:** TypeScript language. **Confidence:** high.
- **Source:** TypeScript Handbook, Enums.

### EMIT-02 — `const enum` inlining
- **Behavior:** `const enum` members are inlined at use sites; the enum object is
  not emitted (unless `preserveConstEnums`). A value dependency in source vanishes
  at runtime. Ambient const enums in published `.d.ts` inline across package
  boundaries.
- **Show:** A const enum used cross-file, with emitted JS containing literals plus
  annotating comments; the same program under `preserveConstEnums`.
- **Variants/failures:** `isolatedModules` bans cross-file ambient const-enum use;
  single-file transpilers cannot inline cross-file — the ecosystem broadly treats
  exported const enums as a hazard.
- **Defined by:** TypeScript language / tsc implementation. **Confidence:** high.
- **Source:** TypeScript Handbook, Enums (const enums); TSConfig Reference
  (`isolatedModules`).

### EMIT-03 — Namespace emit
- **Behavior:** Instantiated namespaces emit as IIFEs assigning onto a shared
  object; types-only namespaces emit nothing; merging with functions/classes
  changes the emitted shape. In script files namespaces can span files (legacy
  `outFile` concatenation).
- **Show:** A namespace with values (emitted) vs one with only types (not emitted);
  the merged function+namespace emit.
- **Defined by:** TypeScript language / tsc implementation. **Confidence:** high.
- **Source:** TypeScript Handbook, Namespaces.

### EMIT-04 — Type-erasure inventory and `erasableSyntaxOnly`
- **Behavior:** Erased: type annotations, interfaces, aliases, type-only imports,
  `implements`, `this` parameters, assertions, `satisfies`, generics. Runtime-
  affecting TypeScript-only syntax: enums, instantiated namespaces, parameter
  properties, `import =`/`export =`, legacy decorators. `erasableSyntaxOnly`
  (TS 5.8) bans the latter set, aligning with Node.js type stripping.
- **Show:** A file compiling to near-identical JS (pure erasure) vs one whose emit
  adds runtime constructs; the flag rejecting the second.
- **Defined by:** TypeScript language. **Confidence:** high.
- **Source:** TypeScript 5.8 release notes (`erasableSyntaxOnly`).

### EMIT-05 — Downlevel emit and helper strategies
- **Behavior:** `target` drives syntax lowering (async, generators, spread, class
  features, optional chaining). Helpers are emitted inline per-file or imported
  from the `tslib` package under `importHelpers` — a build-config-created runtime
  dependency edge.
- **Show:** One file compiled at two targets; the same under `importHelpers` showing
  a `tslib` import appear.
- **Defined by:** tsc implementation. **Confidence:** high.
- **Source:** TSConfig Reference (`target`, `importHelpers`).

### EMIT-06 — Declaration emit
- **Behavior:** `declaration` produces `.d.ts` mirroring the public surface with
  inferred types serialized; `declarationMap` links declarations back to sources.
  Emit can fail on unnameable types ("cannot be named without a reference to…").
  `isolatedDeclarations` (5.5) restricts exports to explicitly annotatable surfaces
  so non-checker tools can emit declarations.
- **Show:** Generated `.d.ts` for a module with inferred exports; a portability
  error case; the same module fixed for `isolatedDeclarations`.
- **Defined by:** tsc implementation. **Confidence:** high.
- **Source:** TypeScript 5.5 release notes; TSConfig Reference (`declaration`).

### EMIT-07 — Source maps and output mapping
- **Behavior:** `sourceMap`/`inlineSourceMap`/`inlineSources`, `outDir`/`rootDir`
  structure mapping — the correspondence between source entities and build
  artifacts that downstream tools (debuggers, coverage, stack traces) rely on.
- **Show:** A build emitting mapped outputs with the rootDir structure preserved.
- **Defined by:** tsc implementation + ecosystem convention (source-map format).
  **Confidence:** high.
- **Source:** TSConfig Reference.

### EMIT-08 — Legacy `outFile` concatenation ordering
- **Behavior:** `outFile` with `module` none/amd/system concatenates script files;
  ordering comes from triple-slash references — file order is program-semantic.
  Legacy but still encountered in older codebases.
- **Show:** Two script files with a reference directive controlling emit order.
- **Defined by:** tsc implementation. **Confidence:** medium-high.
- **Source:** TSConfig Reference (`outFile`).

---

## H. JSX (JSX)

### JSX-01 — `.tsx` parsing differences
- **Behavior:** In `.tsx`, angle-bracket type assertions (`<T>expr`) are invalid
  (`as` required) and generic arrow functions need disambiguation (`<T,>() => ...`
  or a constraint). The same token stream parses differently by extension.
- **Show:** A file pair (`.ts` and `.tsx`) demonstrating the divergence.
- **Defined by:** TypeScript language. **Confidence:** high.
- **Source:** TypeScript Handbook, JSX.

### JSX-02 — JSX modes, factories, and injected imports
- **Behavior:** `jsx` `preserve`/`react`/`react-jsx`/`react-jsxdev`/`react-native`
  changes emit: classic mode calls `jsxFactory`/`jsxFragmentFactory`; automatic mode
  synthesizes an import from `<jsxImportSource>/jsx-runtime` — a module edge that
  appears in no source text. Per-file pragma comments (`@jsx`, `@jsxFrag`,
  `@jsxImportSource`, `@jsxRuntime`) override config per file.
- **Show:** One component compiled under classic and automatic modes; a pragma
  switching one file's import source.
- **Defined by:** tsc implementation + ecosystem convention (React runtime API).
  **Confidence:** high.
- **Source:** TypeScript 4.1 release notes; TSConfig Reference (`jsx`).

### JSX-03 — JSX type resolution
- **Behavior:** Element type checking flows through the `JSX` namespace (resolved
  from the jsx-runtime module under automatic mode, falling back to a global JSX
  namespace), `JSX.IntrinsicElements` for lowercase tags, `JSX.ElementType` (5.1)
  for what may appear as a tag, and component prop/children checking.
- **Show:** An intrinsic element with typed attributes; a custom-element tag added
  via `IntrinsicElements` augmentation; a component rejected as a tag type.
- **Variants/failures:** React's own types moved the JSX namespace into module scope
  (React 18+ types) — resolution differs across @types/react major versions.
- **Defined by:** TypeScript language + ecosystem convention (React types).
  **Confidence:** medium-high.
- **Source:** TypeScript Handbook, JSX; 5.1 release notes.

### JSX-04 — Generic components and explicit type arguments on tags
- **Behavior:** Components can be generic; call-site inference happens at the JSX
  element; explicit type arguments are legal on tags (`<List<Item> ... />`).
- **Show:** A generic component inferred from props at two call sites, plus one
  explicit-argument usage.
- **Defined by:** TypeScript language. **Confidence:** high.
- **Source:** TypeScript 2.9 release notes.

### JSX-05 — Non-React JSX consumers
- **Behavior:** `jsxImportSource` retargets the automatic runtime (alternative UI
  frameworks); each source ships its own JSX namespace, changing intrinsic-element
  and children typing wholesale for the same syntax.
- **Show:** Two configs/pragmas pointing the same component file at different JSX
  implementations with observably different prop typing.
- **Defined by:** Ecosystem convention over a tsc mechanism. **Confidence:** medium-high.
- **Source:** TSConfig Reference (`jsxImportSource`).

---

## I. Ambient declarations and global types (AMB)

### AMB-01 — Global vs module declaration files
- **Behavior:** A `.d.ts` without top-level import/export declares globals visible
  program-wide; adding one import flips every declaration into module scope — a
  classic breakage. Global `.d.ts` files are how environment shims and asset-type
  declarations enter projects.
- **Show:** A global `.d.ts` consumed with no imports; the same file made a module
  and the resulting resolution failures; `import()` types used inside a global
  `.d.ts` to reference modules without becoming one.
- **Defined by:** TypeScript language. **Confidence:** high.
- **Source:** TypeScript Handbook, Declaration Files.

### AMB-02 — Global augmentation targets
- **Behavior:** `declare global` from modules and interface merging in global
  `.d.ts` extend host interfaces: window/global objects, `globalThis` variables,
  array/promise prototypes, process-environment interfaces (namespace-in-global
  merging). These change types of expressions everywhere, with no local reference.
- **Show:** An augmentation adding a property to the global object and to a
  process-env-style interface; usage far from the declaration.
- **Defined by:** TypeScript language + ecosystem convention (augmentation targets).
  **Confidence:** high.
- **Source:** TypeScript Handbook, Declaration Merging (global augmentation).

### AMB-03 — Bundled lib files
- **Behavior:** `lib.*.d.ts` files bundled with the compiler define the entire
  ambient world (ES builtins per edition, DOM, WebWorker, Iterable splits).
  Defaults derive from `target`; `lib` overrides; `/// <reference lib>` adds
  per-file. `node_modules/@typescript/lib-*` packages can replace libs (4.5);
  `libReplacement` (5.8) controls the lookup.
- **Show:** The same code valid under `lib: ["dom"]` and invalid without; a lib
  replacement package swapping DOM types.
- **Variants/failures:** Lib contents change per compiler release — global-entity
  sets are version-dependent.
- **Defined by:** tsc implementation. **Confidence:** high.
- **Source:** TSConfig Reference (`lib`); 4.5 / 5.8 release notes.

### AMB-04 — DefinitelyTyped / @types conventions
- **Behavior:** Community-maintained declarations distributed as `@types/*`;
  global-style vs module-style packages; version convention tracks the covered
  library's major.minor; `typeAcquisition` auto-fetches types for JS projects in
  editors. Types can disagree with the actual runtime package (wrong-version or
  wrong-shape hazards).
- **Show:** A dependency typed only via @types, pinned to a deliberately older
  minor to demonstrate type/runtime skew.
- **Defined by:** Ecosystem convention. **Confidence:** high.
- **Source:** DefinitelyTyped repository documentation.

### AMB-05 — UMD global declarations
- **Behavior:** `export as namespace Lib` marks a module's exports as also available
  as a global — but only consumable as a global from script files unless
  `allowUmdGlobalAccess` is set.
- **Show:** A UMD-typed dependency used via import in a module and as a bare global
  in a script file; the module-file global-access error.
- **Defined by:** TypeScript language. **Confidence:** high.
- **Source:** TypeScript Handbook, Declaration Files (UMD).

### AMB-06 — Sibling and shipped declaration files
- **Behavior:** A `foo.js` with sibling `foo.d.ts` is consumed through the
  declarations (implementation unchecked); packages shipping only `.d.ts` +
  compiled JS present a fundamentally different graph surface than source-visible
  code. `skipLibCheck` silences all `.d.ts` diagnostics, including local ones.
- **Show:** A JS+d.ts pair inside the project; a deliberate d.ts/js mismatch that
  the checker cannot see.
- **Defined by:** tsc implementation. **Confidence:** high.
- **Source:** TypeScript Handbook, Declaration Files; TSConfig Reference
  (`skipLibCheck`).

### AMB-07 — Global namespace collisions
- **Behavior:** Two included declaration sets defining the same globals (two test
  frameworks' @types, `dom` lib vs `webworker` lib) produce duplicate-identifier or
  incompatible-declaration diagnostics; resolution is by exclusion (`types`, `lib`)
  rather than scoping.
- **Show:** A config permutation that introduces and then resolves such a
  collision.
- **Defined by:** tsc implementation. **Confidence:** high.
- **Source:** TSConfig Reference (`types`, `lib`).

---

## J. JavaScript interop and JSDoc (JS)

### JS-01 — JS files in the program
- **Behavior:** `allowJs` admits `.js`/`.jsx` as program files (typed by inference);
  `checkJs` or per-file `// @ts-check` turns on diagnostics; `// @ts-nocheck` turns
  them off. JS files use looser rules: expando property declaration by assignment,
  CommonJS analysis, parameter defaults from usage.
- **Show:** A mixed TS/JS project with imports in both directions; a checked JS
  file with an error; an expando object accumulating properties.
- **Defined by:** tsc implementation. **Confidence:** high.
- **Source:** TypeScript Handbook, JS Projects / Type Checking JavaScript.

### JS-02 — JSDoc as a type surface
- **Behavior:** JSDoc supplies the full annotation language in JS: `@type`,
  `@param`, `@returns`, `@template`, `@typedef`, `@callback`, `@extends`,
  `@implements`, `@overload` (5.0), `@satisfies` (5.0), `@import` (5.5), and
  cast syntax `/** @type {T} */ (expr)`. Declarations created by `@typedef`/
  `@callback` are exportable type entities with no TS syntax.
- **Show:** A JS module whose types come entirely from JSDoc, imported by TS files;
  a `@typedef` consumed cross-file; an `@import` pulling types from a module.
- **Variants/failures:** JSDoc is parsed only in JS files by default; unsupported
  tags ignored silently — partial modeling risk.
- **Defined by:** tsc implementation (own JSDoc dialect). **Confidence:** high.
- **Source:** TypeScript Handbook, JSDoc Reference; 5.0 / 5.5 release notes.

### JS-03 — CommonJS shape analysis in JS
- **Behavior:** `module.exports = ...`, `exports.name = ...`, and `require()` calls
  in JS files are recognized as module structure — export/import entities with no
  ESM syntax. Interop synthesis lets ESM-style TS import them.
- **Show:** A CJS-style JS module consumed by an ESM-style TS file under interop
  flags.
- **Defined by:** tsc implementation modeling Node.js. **Confidence:** high.
- **Source:** TypeScript Handbook, Modules Reference (CommonJS).

### JS-04 — Class-like and namespace-like JS patterns
- **Behavior:** Constructor functions with prototype assignments, and object-literal
  "namespaces", are synthesized into class/namespace-like entities by the checker
  in JS files.
- **Show:** A prototype-based class in JS with instances typed correctly in TS
  consumers.
- **Defined by:** tsc implementation. **Confidence:** medium-high.
- **Source:** TypeScript Handbook, Type Checking JavaScript Files.

### JS-05 — Declaration emit from JavaScript
- **Behavior:** `allowJs` + `declaration` generates `.d.ts` from JSDoc-annotated JS
  (3.7+) — a build mode where the published typed surface derives from JSDoc.
- **Show:** A JSDoc-typed JS module and its emitted declarations.
- **Defined by:** tsc implementation. **Confidence:** high.
- **Source:** TypeScript 3.7 release notes.

---

## K. Configuration-dependent semantics (CFG)

### CFG-01 — The `strict` family forks the type system
- **Behavior:** Each sub-flag changes types or valid programs, not just diagnostics:
  `strictNullChecks` (whether `null`/`undefined` inhabit every type — the single
  largest semantic fork), `noImplicitAny`, `strictFunctionTypes` (contravariant
  parameter checking for function types — method declarations stay bivariant),
  `strictBindCallApply`, `strictPropertyInitialization`, `noImplicitThis`,
  `useUnknownInCatchVariables`, `alwaysStrict`. `strict` itself is a moving alias:
  new sub-flags join it in later releases.
- **Show:** Small programs whose types (not just error lists) differ per flag —
  e.g., the same variable `string | null` vs `string`; a bivariance case accepted
  only for methods.
- **Defined by:** tsc implementation. **Confidence:** high.
- **Source:** TSConfig Reference (strict flags).

### CFG-02 — `exactOptionalPropertyTypes`
- **Behavior:** Distinguishes "property absent" from "property present with
  `undefined`": `{ x?: T }` stops accepting explicit `undefined` writes.
- **Show:** An assignment legal under default settings and illegal under the flag.
- **Defined by:** tsc implementation. **Confidence:** high.
- **Source:** TypeScript 4.4 release notes.

### CFG-03 — `noUncheckedIndexedAccess` and index-signature access rules
- **Behavior:** Index-signature reads gain `| undefined`; companion flag
  `noPropertyAccessFromIndexSignature` forces bracket access for index-signature
  members — element-access types are config-dependent.
- **Show:** The same dictionary read typed differently under each setting.
- **Defined by:** tsc implementation. **Confidence:** high.
- **Source:** TypeScript 4.1 / 4.2 release notes.

### CFG-04 — `target`: one flag, three effect classes
- **Behavior:** `target` changes (1) syntax downleveling, (2) default `lib` set —
  which globals exist, (3) the `useDefineForClassFields` default — runtime field
  semantics. The same source has different emitted behavior and different global
  entities per target.
- **Show:** One project compiled under two targets with all three deltas observable.
- **Defined by:** tsc implementation. **Confidence:** high.
- **Source:** TSConfig Reference (`target`).

### CFG-05 — `module`: emit format and allowed syntax
- **Behavior:** `module` picks output format and gates syntax: top-level await and
  `import.meta` (esnext/es2022/node16+ only), `import =` (CJS-emitting formats),
  `require` typing. `node16`/`nodenext` derive per-file format from extension and
  `package.json`; `preserve` (5.4) emits imports/exports as written and accepts
  both syntaxes; frozen Node snapshots (`node18` in 5.8, `node20` in 5.9 — medium
  confidence) vs floating `nodenext` (which allowed `require()` of ESM in 5.8,
  matching newer Node).
- **Show:** A dual-format package compiled under node16; a top-level-await file
  rejected under `commonjs`.
- **Defined by:** tsc implementation modeling Node.js. **Confidence:** high (medium
  on frozen-snapshot details).
- **Source:** TSConfig Reference (`module`); 5.4 / 5.8 / 5.9 release notes.

### CFG-06 — Interop flags: `esModuleInterop` / `allowSyntheticDefaultImports`
- **Behavior:** Change which import forms typecheck against CJS modules and what is
  emitted (`__importDefault`/`__importStar` helpers). Without them, default-importing
  CJS is an error and namespace imports of CJS are callable; with them the shapes
  flip. Under real Node ESM, the default export of a CJS module is
  `module.exports` — types must match Node semantics, and "fake ESM" transpiled
  packages create notorious default-shape mismatches.
- **Show:** The same CJS dependency imported under both flag settings with
  different valid forms and emitted helpers.
- **Defined by:** tsc implementation + Node.js. **Confidence:** high.
- **Source:** TSConfig Reference (`esModuleInterop`); Handbook, Modules Reference.

### CFG-07 — `isolatedModules`
- **Behavior:** Restricts programs to what single-file transpilers can compile:
  errors on global script files, re-exporting types without type-only syntax, and
  ambient const-enum use. Changes valid programs without changing types.
- **Show:** A file set valid normally and failing under the flag for each rule.
- **Defined by:** tsc implementation guarding ecosystem convention. **Confidence:** high.
- **Source:** TSConfig Reference (`isolatedModules`).

### CFG-08 — `verbatimModuleSyntax`
- **Behavior:** (5.0) Imports/exports emit exactly as written: anything not marked
  `type` stays; unmarked type-only imports error; ESM syntax in CommonJS-emitting
  files errors (forcing `import =`/`export =`). Supersedes the deprecated
  `importsNotUsedAsValues`/`preserveValueImports`.
- **Show:** A file whose emit differs under the flag (retained import) and one that
  errors until `type` markers are added.
- **Defined by:** tsc implementation. **Confidence:** high.
- **Source:** TypeScript 5.0 release notes.

### CFG-09 — `skipLibCheck`
- **Behavior:** Skips checking of all declaration files — inter-`.d.ts`
  contradictions and broken published types are silently tolerated; near-universal
  in real projects, so the "checked program" excludes much of the type surface.
- **Show:** A broken `.d.ts` in dependencies tolerated with the flag, fatal without.
- **Defined by:** tsc implementation. **Confidence:** high.
- **Source:** TSConfig Reference (`skipLibCheck`).

### CFG-10 — `tsconfig` extends and variables
- **Behavior:** `extends` chains merge configs (relative paths resolve against the
  file that declares them); array `extends` (5.0); npm-package bases (shared-config
  packages); `${configDir}` substitution (5.5) for portable path options.
- **Show:** A base config from a package plus a local override, with a path option
  using `${configDir}`.
- **Defined by:** tsc implementation + ecosystem convention (shared configs).
  **Confidence:** high.
- **Source:** TSConfig Reference (`extends`); 5.0 / 5.5 release notes.

### CFG-11 — Check/emit decoupling
- **Behavior:** `noEmit` (check only), `emitDeclarationOnly`, `noEmitOnError`
  (default off — *errors still emit*), and `noCheck` (5.6; emit without checking).
  Build pipelines commonly split "tsc for types, bundler for JS".
- **Show:** A failing program that still produces outputs; a declaration-only build.
- **Defined by:** tsc implementation. **Confidence:** high.
- **Source:** TSConfig Reference; 5.6 release notes (`noCheck`).

### CFG-12 — Diagnostics-only flags
- **Behavior:** `noUnusedLocals`/`noUnusedParameters`, `noImplicitReturns`,
  `noFallthroughCasesInSwitch`, `allowUnreachableCode` — config-varying diagnostic
  surfaces with no type change; relevant to modeling "what this project considers
  an error".
- **Show:** One violation of each, toggled by config.
- **Defined by:** tsc implementation. **Confidence:** high.
- **Source:** TSConfig Reference.

### CFG-13 — `allowArbitraryExtensions`
- **Behavior:** (5.0) Imports of non-JS extensions resolve to `{name}.d.{ext}.ts`
  declaration files (e.g., `styles.css` → `styles.d.css.ts`) — an alternative to
  wildcard ambient modules with per-file typing.
- **Show:** A CSS import typed by a sibling `d.css.ts` file.
- **Defined by:** tsc implementation. **Confidence:** medium-high.
- **Source:** TypeScript 5.0 release notes.

### CFG-14 — Deprecated/removed options and `ignoreDeprecations`
- **Behavior:** Option sets change by version: 5.0 deprecated (and ~5.5 removed)
  `target: ES3`, `out`, `importsNotUsedAsValues`, `preserveValueImports`, etc.;
  `ignoreDeprecations` gates the transition. Config validity itself is
  version-dependent behavior.
- **Show:** A config valid for 4.9 rejected by 5.5.
- **Defined by:** tsc implementation. **Confidence:** medium-high.
- **Source:** TypeScript 5.0 release notes (deprecations).

---

## L. Packages and distribution (PKG)

### PKG-01 — `package.json` `"type"` and nearest-package lookup
- **Behavior:** Under node16+, each `.ts`/`.js` file's module format comes from the
  nearest ancestor `package.json` `"type"` field (unless the extension forces it).
  One repository can mix CJS and ESM subtrees; the same import syntax means
  different interop per location.
- **Show:** Two subdirectories with different `"type"` values and cross-imports
  between them.
- **Defined by:** Node.js, modeled by tsc. **Confidence:** high.
- **Source:** Node.js documentation, Packages; TypeScript Handbook, Modules Reference.

### PKG-02 — `exports` maps and the `types` condition
- **Behavior:** Subpath exports gate what can be imported; per-condition `types`
  entries let `import` and `require` resolve *different declaration files* for the
  same package — one dependency, two typed surfaces (dual-package hazard at the
  type level). Packages without a `types` condition/compatible layout break under
  node16 resolution while working under node10.
- **Show:** A local dependency (workspace fixture) with subpath exports,
  divergent import/require types, and one deliberately broken consumer config.
- **Variants/failures:** Condition order matters (`default` last); sub-path
  patterns (`./*`); blocked deep imports.
- **Defined by:** Node.js + tsc implementation. **Confidence:** high.
- **Source:** Node.js documentation, Packages; TypeScript Handbook, Modules Reference.

### PKG-03 — `imports` maps and self-references
- **Behavior:** `#`-prefixed internal aliases from `package.json` `imports`
  (condition-sensitive) and importing a package by its own name from inside itself —
  both resolve through package metadata rather than relative paths.
- **Show:** An `#internal/*` alias used across a package; a self-name import in a
  test file.
- **Defined by:** Node.js + tsc implementation. **Confidence:** high.
- **Source:** Node.js documentation, Packages.

### PKG-04 — `typesVersions`
- **Behavior:** Pre-`exports` mechanism selecting declaration directories by
  TypeScript compiler version range — the same package presents different type
  surfaces to different compiler versions.
- **Show:** A fixture dependency whose types differ under a `typesVersions`
  selector.
- **Defined by:** tsc implementation. **Confidence:** medium-high.
- **Source:** TypeScript Handbook, Publishing declaration files.

### PKG-05 — Transitive and peer type dependencies
- **Behavior:** A dependency's `.d.ts` may import types from packages the consumer
  never installed (must resolve through the dependency's own `node_modules` or
  fail); @types packages depend on other @types; peer-typed plugins augment their
  host (see MOD-12).
- **Show:** A dependency whose declarations import from a second-level dependency;
  a broken case where hoisting hides it.
- **Defined by:** tsc implementation + package-manager layout. **Confidence:** high.
- **Source:** TypeScript Handbook, Module Resolution.

### PKG-06 — Duplicate package versions and type identity
- **Behavior:** Two copies of the same declaration package in the tree (nested
  versions, pnpm isolation) produce distinct symbol identities — "Type 'X' is not
  assignable to type 'X'" across the boundary. Structural types usually unify;
  nominal islands (classes with privates, enums, unique symbols) do not.
- **Show:** A fixture with two nested versions of a shared types package and a
  cross-boundary assignment failing.
- **Defined by:** tsc implementation + package-manager behavior. **Confidence:** high.
- **Source:** TypeScript Handbook, Module Resolution (widely observed ecosystem
  behavior).

### PKG-07 — Workspaces and monorepo consumption modes
- **Behavior:** Workspace-linked internal packages can be consumed (a) from source
  via `paths`/bundler aliases, (b) from built `.d.ts` via project references or
  prebuilt dist — same code, two different graph shapes (source entities vs
  declaration entities).
- **Show:** A two-package workspace demonstrating both consumption modes.
- **Defined by:** Ecosystem convention (npm/pnpm/yarn workspaces) + tsc mechanisms.
  **Confidence:** high.
- **Source:** Package-manager documentation (workspaces); TypeScript Handbook,
  Project References.

### PKG-08 — Generated packages and generated sources in `node_modules`
- **Behavior:** Ecosystem code generators write typed clients *into*
  `node_modules` (database-client generators) or into the source tree (API/schema
  codegen) — program entities exist only after a generation step, and regenerate
  with schema changes.
- **Show:** A minimal in-repo generator emitting a typed module consumed by
  hand-written code; documentation of ordering (generate before typecheck).
- **Defined by:** Ecosystem convention. **Confidence:** high.
- **Source:** Ecosystem tool documentation (pattern-level; tool-agnostic).

### PKG-09 — Declaration bundling of published types
- **Behavior:** Publishing pipelines flatten/rollup `.d.ts` (API extractors, dts
  bundlers): the published type surface structurally differs from the source graph
  (re-export chains collapsed, private types renamed/inlined).
- **Show:** A package published with rolled-up declarations vs its multi-file
  source layout.
- **Defined by:** Ecosystem convention. **Confidence:** medium-high.
- **Source:** Ecosystem tool documentation (pattern-level).

---

## M. Project structure and build (PROJ)

### PROJ-01 — Project references
- **Behavior:** `composite` projects with `references` build in dependency order
  (`tsc -b`); imports into a referenced project resolve to its *declaration output*
  (or to sources when `declarationMap` + editor features allow), not its source —
  graph nodes may be artifacts standing in for sources.
  `disableSourceOfProjectReferenceRedirect` and related flags tune this.
- **Show:** A two-project build where editing the dependency without rebuilding
  yields stale types; declaration maps restoring source navigation.
- **Constraints:** Referenced projects must be `composite` (forces `declaration`);
  `include` must cover all inputs.
- **Defined by:** tsc implementation. **Confidence:** high.
- **Source:** TypeScript Handbook, Project References.

### PROJ-02 — Solution-style configurations
- **Behavior:** A root tsconfig with `files: []` and only `references` — a build
  orchestration node that owns no files itself; editors use it to find the right
  project per file.
- **Show:** A root solution config over app/lib/test projects.
- **Defined by:** tsc implementation. **Confidence:** high.
- **Source:** TypeScript 3.9 release notes (solution style).

### PROJ-03 — One file, multiple programs
- **Behavior:** The same source file can belong to several tsconfig programs
  (application config and test config) with different `types`, `lib`, and flags —
  its globals, diagnostics, and even types differ per program. Entity identity "per
  file" is not well-defined without naming the program.
- **Show:** A file valid under the test program (test-framework globals) and
  invalid under the app program.
- **Defined by:** tsc implementation. **Confidence:** high.
- **Source:** TSConfig Reference (program semantics; widely observed setup).

### PROJ-04 — Incremental state
- **Behavior:** `incremental`/`.tsbuildinfo` caches dependency state; stale or
  deleted buildinfo changes rebuild behavior; watch-mode heuristics
  (`assumeChangesOnlyAffectDirectDependencies`) are deliberately unsound.
- **Show:** A build whose outputs differ after touching buildinfo state.
- **Defined by:** tsc implementation. **Confidence:** medium-high.
- **Source:** TSConfig Reference (`incremental`).

### PROJ-05 — In-repo code generation feeding the program
- **Behavior:** Generated `.ts`/`.d.ts` (routes, schemas, API clients, protocol
  stubs) included via `include` globs or `rootDirs`; the program is incomplete or
  erroneous before generation — source inclusion depends on build phase.
- **Show:** A generation script, its output directory in `include`, and a consumer
  import; the failing pre-generation state documented.
- **Defined by:** Ecosystem convention. **Confidence:** high.
- **Source:** Ecosystem practice (pattern-level).

### PROJ-06 — Editor/batch divergence
- **Behavior:** `tsserver` places orphan files in inferred projects with default
  settings (different lib/flags than any tsconfig), honors language-service
  plugins that `tsc` ignores (CSS-module typing plugins), and resolves
  go-to-definition through `declarationMap`. The "graph the editor sees" and the
  "graph the build sees" differ.
- **Show:** A file outside every tsconfig behaving differently in-editor; a plugin
  the CLI build ignores.
- **Defined by:** tsc/tsserver implementation. **Confidence:** medium-high.
- **Source:** TypeScript wiki/documentation on tsserver and language-service plugins.

---

## N. Diagnostics, suppression, and invalid programs (DIAG)

### DIAG-01 — Errors do not stop emit
- **Behavior:** By default `tsc` emits JavaScript for semantically invalid programs
  (`noEmitOnError` off). "Broken but running" is a normal project state; analysis
  must produce a graph for programs with type errors.
- **Show:** A file with a type error whose emitted JS exists and runs.
- **Defined by:** tsc implementation. **Confidence:** high.
- **Source:** TSConfig Reference (`noEmitOnError`).

### DIAG-02 — Suppression directives
- **Behavior:** `// @ts-ignore` (suppress next line), `// @ts-expect-error` (errors
  if nothing is suppressed — inverted diagnostic), `// @ts-nocheck` (whole file,
  including `.ts` since 3.7), `// @ts-check` (opt-in for JS). Suppression alters
  the diagnostic surface, not semantics.
- **Show:** Each directive, including an unused `@ts-expect-error` producing its
  own error.
- **Defined by:** tsc implementation. **Confidence:** high.
- **Source:** TypeScript 3.7 / 3.9 release notes.

### DIAG-03 — Unresolved and untyped imports
- **Behavior:** Unresolvable specifiers error (module-not-found) but still bind an
  `any`-typed name — downstream code checks against `any`; untyped-JS packages
  yield an implicit-`any` module diagnostic under `noImplicitAny`; shorthand
  ambient declarations rescue them. Graphs need dangling-edge representation.
- **Show:** An import of a nonexistent module with downstream usage; an untyped
  package before and after a shorthand declaration.
- **Defined by:** tsc implementation. **Confidence:** high.
- **Source:** TypeScript Handbook, Module Resolution.

### DIAG-04 — Redeclaration: legal merges vs collisions
- **Behavior:** The merge matrix determines duplicate-name outcomes: interface+
  interface merges; class+interface merges; namespace+function merges; class+class
  errors; `let`+`let` errors; `var`+`var` merges; value+type coexistence is legal
  across declaration spaces. Script-file globals collide program-wide.
- **Show:** One instance of each legal merge and each collision, including a
  cross-file script-global collision.
- **Defined by:** TypeScript language. **Confidence:** high.
- **Source:** TypeScript Handbook, Declaration Merging.

### DIAG-05 — Behavior under syntax errors
- **Behavior:** The parser recovers aggressively; files with syntax errors still
  produce partial ASTs, bindings, and diagnostics (editors depend on this). A
  representative project state includes a file mid-edit.
- **Show:** A deliberately malformed file alongside intact ones; the rest of the
  program remains analyzable.
- **Defined by:** tsc implementation. **Confidence:** medium-high.
- **Source:** Compiler behavior (implementation-defined; no normative document).

### DIAG-06 — Version gating of syntax and config
- **Behavior:** Newer syntax under an older compiler yields parse/semantic errors
  (`satisfies` pre-4.9, `using` pre-5.2); unknown tsconfig options error;
  `typesVersions` and published lib floors mitigate. Compiler version is part of
  program identity.
- **Show:** Documentation of the minimum compiler per feature exercised; optionally
  a version-pinned config matrix.
- **Defined by:** tsc implementation. **Confidence:** high.
- **Source:** Per-version TypeScript release notes.

---

## O. Alternative toolchains and runtimes (RUN)

### RUN-01 — Node.js native type stripping
- **Behavior:** Node runs `.ts` directly by erasing types (experimental in 22.6,
  enabled by default in 23.6): erasable syntax only — enums, instantiated
  namespaces, parameter properties, `import =` need a transform flag; no `paths`;
  relative imports need real extensions. TypeScript's `erasableSyntaxOnly` (5.8)
  targets this profile.
- **Show:** A subproject constrained to the strippable profile and documented as
  Node-runnable; a counterexample file that violates it.
- **Defined by:** Node.js (behavior) + tsc (flag). **Confidence:** high (medium on
  exact Node version boundaries).
- **Source:** Node.js documentation (type stripping); TypeScript 5.8 release notes.

### RUN-02 — Single-file transpilers
- **Behavior:** esbuild/swc/Babel compile file-by-file with no cross-file type
  information: `isolatedModules` assumptions required; const-enum inlining,
  type-directed import elision, and legacy decorator metadata differ or are
  unsupported per tool; emit can diverge from `tsc` for the same source.
- **Show:** Files that are `isolatedModules`-clean vs a file demonstrating a
  transpiler-visible divergence (exported const enum).
- **Defined by:** Ecosystem tools (each self-defined). **Confidence:** high at the
  pattern level; medium on per-tool specifics.
- **Source:** Respective tool documentation; TSConfig Reference (`isolatedModules`).

### RUN-03 — Loaders and executors
- **Behavior:** Runtime TS executors (transpile-on-load, register hooks) vary in
  tsconfig fidelity: some honor `paths`, some ignore type checking entirely,
  ESM-loader behavior differs from CJS-register behavior — "runs" and "typechecks"
  are independent properties.
- **Show:** A script executed via a loader while `tsc --noEmit` is the separate
  check step.
- **Defined by:** Ecosystem tools. **Confidence:** medium-high.
- **Source:** Respective tool documentation.

### RUN-04 — Bun and Deno divergences
- **Behavior:** Bun executes TS without type checking (partial tsconfig respect);
  Deno brings its own resolution (URL imports, `npm:`/`jsr:` specifiers, import
  maps) and bundled checker defaults. These are materially different resolution
  and inclusion regimes; a Node-baseline fixture should note them as out-of-scope
  variants rather than silently exclude them.
- **Defined by:** Respective runtimes. **Confidence:** medium.
- **Source:** Bun and Deno official documentation.

### RUN-05 — Native compiler port (TypeScript 7 / `tsgo`)
- **Behavior:** The Go-based native port targets behavioral parity with `tsc`;
  during preview, edge-case divergences (diagnostics ordering, API surface,
  language-service behavior) are possible. TypeScript 6.x serves as the
  configuration bridge.
- **Constraints:** Not baseline; worth tracking because "official implementation"
  is becoming plural.
- **Defined by:** tsc implementation (parity target). **Confidence:** medium.
- **Source:** Microsoft TypeScript blog announcements on the native port.

---

## 3. Features with material recent version changes

| Version | Changes relevant to graph semantics |
| --- | --- |
| 4.7 | `node16`/`nodenext` resolution, per-file module format, `moduleDetection`, instantiation expressions, `in`/`out` variance, `moduleSuffixes`, `extends` constraints on `infer` |
| 4.9 | `satisfies`, auto-accessors (`accessor`), `in`-narrowing refinements |
| 5.0 | Standard TC39 decorators, `const` type parameters, `verbatimModuleSyntax`, `bundler` resolution, `customConditions`, `allowImportingTsExtensions`, `allowArbitraryExtensions`, `export type *`, array `extends`, enums-as-unions completion, flag deprecations |
| 5.1 | Unrelated getter/setter types, `JSX.ElementType`, undefined-return inference |
| 5.2 | `using`/`await using`, decorator metadata (`Symbol.metadata`) |
| 5.3 | Import attributes (`with`), `resolution-mode` on import types, `switch (true)` narrowing, `Symbol.hasInstance` `instanceof` |
| 5.4 | `NoInfer`, closure-narrowing preservation, `module: preserve` |
| 5.5 | Inferred type predicates, `isolatedDeclarations`, JSDoc `@import`, `${configDir}`, regex syntax checking |
| 5.6 | Strict built-in iterator types, arbitrary module identifiers, `--noCheck`, always-truthy-check errors |
| 5.7 | `rewriteRelativeImportExtensions`, never-initialized-variable checks, ES2024 target, JSON-import attribute validation under nodenext |
| 5.8 | `erasableSyntaxOnly`, `require()` of ESM under `nodenext`, `libReplacement`, granular return-branch checks, `module node18` |
| 5.9 | `import defer`, `module node20`, minimal `tsc --init` output |

Inferred type predicates (5.5) and strict iterator types (5.6) are notable for
changing *observable types of unchanged code*. The `strict` alias itself gains
members over time (e.g., `useUnknownInCatchVariables` joined in 4.4).

## 4. Implementation-defined and unspecified behavior

- **No formal specification.** The archived language spec covers ~1.8 and is
  obsolete; the compiler, handbook, and release notes are the norm. Anything not
  documented is pinned only by `tsc` behavior.
- **Inference details.** Union member ordering, alias preservation vs expansion in
  displayed/emitted types, generic inference priorities, and contextual-typing
  corner cases drift between minor versions.
- **Recursion and depth limits.** Conditional-type recursion depth, instantiation
  depth/count limits ("excessively deep" errors) are implementation constants.
- **`strict` as a moving set.** The meaning of `"strict": true` changes across
  releases by design.
- **Lib contents.** `lib.dom.d.ts` and friends are regenerated from web specs each
  release; the global entity set is compiler-version-dependent.
- **Diagnostic identity.** Error codes and messages are not stability-guaranteed.
- **Program-set sensitivity.** Declaration merging, global augmentation, and
  automatic @types inclusion make the meaning of a file depend on which other
  files happen to be in the program.
- **Type identity.** "Same type" has no canonical answer: structural unification,
  alias preservation, and nominal islands coexist; two programs can disagree.
- **tsserver-specific behavior.** Inferred projects, auto-import source selection,
  plugin effects — outside `tsc` entirely.
- **Native port transition.** Behavioral parity of the Go implementation is a goal,
  not a current guarantee.

## 5. Areas where my knowledge may be incomplete

- Exact version boundaries flagged medium above: `module node18`/`node20`
  snapshots, JSON import-attribute enforcement, `import defer` module-option
  constraints, the precise `moduleDetection: auto` clause list, and Node
  type-stripping version gates.
- Full corner-case matrix of `exports`-map resolution (pattern trailers, condition
  interactions) as modeled by each `moduleResolution` mode.
- Standard-decorator fine points: exact evaluation order across static/instance
  and metadata inheritance details.
- Enforcement precision of module-augmentation limits (MOD-12).
- Anything after the January 2026 knowledge cutoff: TypeScript 6.x deprecation
  specifics and TypeScript 7 release state.
- Interface-merge overload ordering and `.ts`/`.d.ts` sibling precedence details
  (marked medium in items).

## 6. Final audit: families possibly still missing

Reviewed and deliberately summarized rather than itemized:

- **Multi-environment lib splits:** the same file checked against `dom` and
  `webworker` (worker code) — covered only via AMB-03/AMB-07; a fixture could make
  it explicit with a worker entry point.
- **Yarn Plug'n'Play:** resolution via `.pnp.cjs` without `node_modules` — a
  distinct resolution host not itemized above (would extend RES-10).
- **Custom transformers / patched compilers:** build-time AST transforms
  (`ts-patch`-style) change emit beyond documented semantics — flagged, not
  itemized.
- **Language-service plugins:** covered in PROJ-06 only at pattern level.
- **Type-aware lint layers:** typescript-eslint's parser services build parallel
  programs; out of scope for language semantics but a real consumer of graph-like
  data.
- **Watch-mode/file-watching semantics:** ordering and coalescing of rebuilds —
  operational rather than semantic; PROJ-04 touches it.
- **WebAssembly/ESM integration and CSS module scripts:** import forms typed today
  only via ambient declarations (MOD-11 covers the mechanism).
- **Internationalization/platform APIs:** purely lib-content questions (AMB-03).
- **Legacy module targets (AMD/UMD/SystemJS):** mechanism itemized (EMIT-08,
  AMB-05); a modern fixture reasonably keeps these minimal.
- **Documentation-comment semantics:** TSDoc/JSDoc `@deprecated` and `@see` affect
  editor behavior and API-report tooling; only JS-02 touches JSDoc as a *type*
  surface. A graph capturing deprecation state would need this family.
- **Performance-only flags** (`skipDefaultLibCheck`, `disableSizeLimit`): no
  semantic impact identified; omitted deliberately.

Nothing else surfaced in a category sweep across syntax, binding, checking,
emit, resolution, project model, and ecosystem layers.

