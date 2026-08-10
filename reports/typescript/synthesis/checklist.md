# Canonical TypeScript Language-Feature Checklist

## Scope and interpretation

This checklist reconciles the Codex, Fable, and Google reports. It defines observable obligations, not a project theme, source layout, graph schema, or implementation strategy.

The reference compiler is Microsoft's `typescript` package. Pin its exact version, standard-library files, runtime, package manager, and effective compiler options. TypeScript has no current normative language specification independent of the compiler. ECMAScript runtime behavior remains governed by ECMA-262. Compiler inference, normalization, diagnostics, resolution modeling, and emit are therefore identified as implementation-defined where appropriate.

The broad current report baseline is TypeScript 5.9. Items introduced in TypeScript 4.7 through 5.9 retain their version boundaries. Older modes and other hosts are conditional variants. Claims about TypeScript 6, TypeScript 7/`tsgo`, or later behavior remain research gaps.

Status meanings:

- **Required:** materially distinct TypeScript behavior that the canonical fixture corpus must demonstrate, possibly in separate mutually exclusive configurations.
- **Conditional:** required only when the named host, package manager, tool, legacy target, or compiler version is in scope.
- **Unresolved research gap:** plausible unique behavior or disputed detail that must be verified before becoming a required fixture obligation.

Provenance uses `C` for the Codex report, `F` for the Fable report, and `G` for the Google report. Citations resolve to primary sources in [Primary-source references](#primary-source-references).

## A. Program construction, configuration, and source identity

### TS-CAN-001 — Compiler, library, and program identity

- **Feature / fixture obligation:** Treat exact TypeScript compiler version, bundled `lib.*.d.ts` set, effective configuration, root program, and host as part of program identity. Run at least one unchanged source case across a documented version boundary that changes an inferred type, diagnostic, resolution result, or emit.
- **Required variants and failures:** Workspace compiler versus editor-bundled compiler; dependency minimum compiler version; changed iterator/library declarations; unknown newer syntax or options under an older compiler.
- **Constraints:** Pin TypeScript 5.9 for current cases and retain explicit 4.7–5.8 transition cases. Do not assume minor releases are behavior-preserving.
- **Classification / status:** implementation-defined; **required**.
- **Evidence / provenance:** [TS release notes], [TSConfig]. C: TS-100. F: DIAG-06, version table, RUN-05. G: completeness discussion.

### TS-CAN-002 — Root-file discovery and transitive inclusion

- **Feature / fixture obligation:** Demonstrate roots selected by `files`, `include`, command-line arguments, and defaults; an `exclude`d file re-entering through an import or reference; automatic declaration inclusion; and a genuinely unreachable file.
- **Required variants and failures:** Empty/misspelled globs, default exclusions, imported `node_modules`, duplicate paths, generated output re-entering input, `allowJs`, no-input and overwrite/input-collision diagnostics.
- **Constraints:** Inclusion and path canonicalization depend on configuration and filesystem. `exclude` filters discovery, not the complete transitive program.
- **Classification / status:** implementation-defined; **required**.
- **Evidence / provenance:** [TSConfig]. C: TS-001. F: RES-09. G: not proposed.

### TS-CAN-003 — Configuration inheritance and effective options

- **Feature / fixture obligation:** Demonstrate relative, package-based, chained, and TypeScript 5.0 array `extends`; replacement rather than assumed deep merge; command-line overrides; and TypeScript 5.5 `${configDir}` substitution.
- **Required variants and failures:** Missing base, cycle, arrays/objects replaced by a child, root-file selection differences, conflicting `paths`, `types`, `typeRoots`, output settings, and a portable shared package config.
- **Constraints:** Relative paths are resolved from the config that declares them. Exact merge behavior is compiler-version dependent.
- **Classification / status:** implementation-defined; **required**.
- **Evidence / provenance:** [TSConfig], [TS 5.0], [TS 5.5]. C: TS-002. F: CFG-10. G: not proposed.

### TS-CAN-004 — Composite references and solution configurations

- **Feature / fixture obligation:** Demonstrate at least two `composite` projects, a solution config with `files: []`, a reference/build-order edge, public declaration consumption across the boundary, hidden implementation symbols, and declaration-map/source redirection.
- **Required variants and failures:** Missing `composite`, incomplete inputs, absent/stale outputs, reference cycles, `disableSourceOfProjectReferenceRedirect`, build invalidation, and a consumer that sees output declarations rather than freely merged sources.
- **Constraints:** Project references require TypeScript 3.0+. Build orchestration is exercised with `tsc -b`; editor redirection is distinct from batch build behavior. The reports' exact cycle diagnostic claims require verification.
- **Classification / status:** implementation-defined; **required**.
- **Evidence / provenance:** [TS Project References], [TS 3.9]. C: TS-003. F: PROJ-01, PROJ-02. G: TS-FEAT-001.

### TS-CAN-005 — Incremental, watch, and build-mode state

- **Feature / fixture obligation:** Demonstrate an implementation-only edit, public declaration edit, and error removal through `incremental`/`.tsbuildinfo` and `tsc -b`, recording different rebuild scopes without treating cached state as language meaning.
- **Required variants and failures:** Clean build, corrupt/stale/deleted build info, compiler/options change, failed upstream project, and `assumeChangesOnlyAffectDirectDependencies`.
- **Constraints:** `.tsbuildinfo`, diagnostic scheduling, watcher invalidation, and performance cutoffs are non-portable compiler state.
- **Classification / status:** implementation-defined; **required**.
- **Evidence / provenance:** [TSConfig], [TS Project References]. C: TS-004. F: PROJ-04. G: TS-FEAT-001 interaction.

### TS-CAN-006 — `target`, standard libraries, and runtime availability

- **Feature / fixture obligation:** Compile modern syntax and built-in use under old and modern targets. Demonstrate three independent effects: syntax lowering, default `lib` selection, and the target-dependent class-field default. Separately show that successful checking does not supply runtime polyfills.
- **Required variants and failures:** Explicit `lib`, `noLib`, `/// <reference lib>`, DOM/WebWorker overlap, library replacement, `libReplacement`, missing globals, BigInt below ES2020, missing Promise/Symbol/runtime built-ins, and `skipDefaultLibCheck`.
- **Constraints:** Emit is compiler-defined; built-in runtime behavior is host-defined; bundled declaration contents change by compiler version.
- **Classification / status:** implementation-defined; **required**.
- **Evidence / provenance:** [TSConfig], [TS 4.5], [TS 5.8], [ECMA-262]. C: TS-005, TS-006. F: CFG-04, AMB-03. G: not proposed.

### TS-CAN-007 — Strictness family

- **Feature / fixture obligation:** Provide focused valid/invalid or type-changing cases for `strictNullChecks`, `noImplicitAny`, `strictFunctionTypes`, `strictBindCallApply`, `strictPropertyInitialization`, `noImplicitThis`, `useUnknownInCatchVariables`, and `alwaysStrict`, including a child option overriding `strict: true`.
- **Required variants and failures:** Function-property contravariance versus method bivariance, catch `unknown` versus `any`, property initialization, implicit `this`/`any`, null collapse when strict-null checking is off, and version changes to the `strict` umbrella.
- **Constraints:** The umbrella is a moving compiler-defined set; emitted JavaScript may remain identical despite changed types or errors.
- **Classification / status:** implementation-defined; **required**.
- **Evidence / provenance:** [TSConfig], [TS 4.4]. C: TS-007. F: CFG-01. G: explicitly identified as missing.

### TS-CAN-008 — Independent safety and diagnostics flags

- **Feature / fixture obligation:** Exercise `exactOptionalPropertyTypes`, `noUncheckedIndexedAccess`, `noPropertyAccessFromIndexSignature`, `noImplicitOverride`, unused checks, implicit-return checks, switch-fallthrough checks, and unreachable-code policy independently.
- **Required variants and failures:** Absent versus present-`undefined` optional properties; dictionary reads gaining `undefined`; dot versus bracket index access; missing `override`; unused locals/parameters; missing returns; fallthrough; reachable versus suppressed unreachable diagnostics.
- **Constraints:** Some flags depend on strict-null checking. Hygiene flags change diagnostics without necessarily changing inferred types or emit.
- **Classification / status:** implementation-defined; **required**.
- **Evidence / provenance:** [TSConfig], [TS 4.1], [TS 4.2], [TS 4.4]. C: TS-008. F: CFG-02, CFG-03, CFG-12. G: strictness gap.

### TS-CAN-009 — Module emit and resolution strategy matrix

- **Feature / fixture obligation:** Resolve and emit the same module dependency structure under meaningful combinations of `module` and `moduleResolution`: legacy `classic`/`node10`, Node16/NodeNext, `bundler`, CommonJS, preserved ECMAScript modules, and conditional legacy targets.
- **Required variants and failures:** Extensionless relative and directory imports, explicit ESM extensions, top-level `await`, `import.meta`, `module: preserve`, incompatible option pairs, `exports`/`imports` honoring, `require()` of ESM accepted/rejected across Node-mode versions, and emitted text that stays equal while resolution changes.
- **Constraints:** Node16/NodeNext require TypeScript 4.7+; `bundler` requires TypeScript 5.0+ and an allowed modern/preserve module mode. Node-oriented modes determine format per file.
- **Classification / status:** implementation-defined; **required**.
- **Evidence / provenance:** [TS Modules], [TSConfig], [TS 4.7], [TS 5.0], [TS 5.4], [Node Packages]. C: TS-009, TS-010. F: RES-01, CFG-05. G: TS-FEAT-002.

### TS-CAN-010 — Extension family and per-file module format

- **Feature / fixture obligation:** Use `.ts`, `.tsx`, `.mts`, `.cts`, `.d.ts`, `.d.mts`, and `.d.cts`, with JavaScript counterparts where enabled. Demonstrate `.js`-suffixed source imports resolving to `.ts`, forced formats, and nearest-package `type` lookup.
- **Required variants and failures:** `.mjs`↔`.mts`, `.cjs`↔`.cts`; JSX rejected outside TSX; mismatched declaration flavor; `allowImportingTsExtensions` in a permitted no-JavaScript-emit mode; TypeScript 5.7 `rewriteRelativeImportExtensions`; and omitted extensions failing under Node ESM rules.
- **Constraints:** Per-file format affects syntax, checking, emit, and interop. Exact sibling `.ts`/`.d.ts` precedence remains a research gap.
- **Classification / status:** implementation-defined; **required**.
- **Evidence / provenance:** [TS Modules], [TS 4.7], [TS 5.0], [TS 5.7], [Node Packages]. C: TS-011, TS-012. F: RES-02, RES-03, PKG-01. G: TS-FEAT-002.

### TS-CAN-011 — `paths`, `baseUrl`, `rootDirs`, and `moduleSuffixes`

- **Feature / fixture obligation:** Demonstrate a path alias with ordered fallback targets, a `rootDirs` virtual merge across authored/generated roots, and a platform suffix selecting different files for one specifier.
- **Required variants and failures:** Wildcard specificity, alias shadowing a package, runtime failure because emitted aliases are not rewritten, relative cross-root import, `.ios`/`.native`/empty suffix order, and `paths` without `baseUrl` in supported versions.
- **Constraints:** These are compile-time lookup mechanisms. Runtime layout or mapping is a separate host contract. The claim that `baseUrl` itself is deprecated is unsupported by the supplied primary evidence.
- **Classification / status:** implementation-defined; **required**.
- **Evidence / provenance:** [TSConfig], [TS 4.1], [TS 4.7]. C: TS-014, TS-099. F: RES-04, RES-05, RES-06. G: TS-FEAT-006.

### TS-CAN-012 — File identity, casing, real paths, and symlinks

- **Feature / fixture obligation:** Demonstrate case-only import differences, a case-only rename, a workspace/package symlink, and duplicate logical access paths that can unify or split declaration identity.
- **Required variants and failures:** Case-sensitive versus insensitive filesystems, `forceConsistentCasingInFileNames`, `preserveSymlinks`, pnpm-style stores, monorepo links, two versions of a nominally sensitive type, and path spellings resolving to one physical file.
- **Constraints:** Results depend on compiler canonicalization, operating system, filesystem, Node, and package-manager layout.
- **Classification / status:** implementation-defined; **required**.
- **Evidence / provenance:** [TSConfig], [Node Modules]. C: TS-015. F: RES-09, RES-10. G: not proposed.

### TS-CAN-013 — JavaScript inputs, JSDoc, and inferred CommonJS shapes

- **Feature / fixture obligation:** Include checked and unchecked `.js`/`.jsx` files imported by TypeScript and vice versa. Use `@typedef`, `@callback`, `@template`, `@overload`, `@satisfies`, `@import`, casts, prototype members, expando properties, and CommonJS assignment exports; emit declarations from JavaScript.
- **Required variants and failures:** `allowJs`, `checkJs`, `@ts-check`, `@ts-nocheck`, unsupported/ignored JSDoc tags, `module.exports`/`exports.x`/`require`, constructor-function prototypes, `maxNodeModuleJsDepth`, and declaration portability failures.
- **Constraints:** TypeScript's JSDoc dialect and JavaScript inference are compiler behavior; JSDoc declaration emit requires supported versions and options.
- **Classification / status:** implementation-defined; **required**.
- **Evidence / provenance:** [TS JavaScript], [TS JSDoc], [TS 3.7], [TS 5.0], [TS 5.5]. C: TS-016. F: JS-01–JS-05. G: TS-FEAT-010.

### TS-CAN-014 — Declaration inputs, public surface, maps, and source correspondence

- **Feature / fixture obligation:** Consume `.d.ts` independently from runtime code; emit `.d.ts` and `.d.ts.map`; show inferred public types, a private/unnameable leaked type, `@internal` stripping, and navigation back to authored sources.
- **Required variants and failures:** `declaration`, `emitDeclarationOnly`, `stripInternal`, declaration maps, sibling JS/declaration mismatch, `.d.mts`/`.d.cts`, stale declarations, source maps with missing/embedded sources, and ecosystem declaration rollups that collapse re-export chains.
- **Constraints:** Declaration files assert runtime shape but do not verify it. Map consumption and declaration bundling are ecosystem behavior.
- **Classification / status:** implementation-defined; **required**.
- **Evidence / provenance:** [TS Declaration Files], [TSConfig], [TS 5.5], [Source Map]. C: TS-017, TS-020. F: EMIT-06, EMIT-07, AMB-06, PKG-09. G: source/declaration-map gap.

### TS-CAN-015 — Check/emit decoupling and isolated processing

- **Feature / fixture obligation:** Compile a type-invalid but emit-capable program under default emit, `noEmit`, `noEmitOnError`, `emitDeclarationOnly`, and TypeScript 5.6 `noCheck`. Separately demonstrate each `isolatedModules` and `isolatedDeclarations` restriction and its repair.
- **Required variants and failures:** Re-exported types without markers, script namespaces, ambient const enums, exported inference needing annotation, declaration-only failure, build-mode continuation, and whole-program-valid code rejected by a per-file tool profile.
- **Constraints:** `isolatedDeclarations` requires TypeScript 5.5+; `noCheck` requires 5.6+; exact build-mode behavior is version-specific.
- **Classification / status:** implementation-defined; **required**.
- **Evidence / provenance:** [TSConfig], [TS 5.5], [TS 5.6]. C: TS-018, TS-019. F: CFG-07, CFG-11, EMIT-04. G: not proposed.

### TS-CAN-016 — Generated sources and build-phase provenance

- **Feature / fixture obligation:** Demonstrate generated `.ts` and/or `.d.ts` entering through `include`, import, or `rootDirs`; a consumer failing before generation; changed generated types after input changes; and source/map paths distinguishing authored, generated, and emitted identities.
- **Required variants and failures:** Generated output excluded but imported, stale generated declarations, generated package content under `node_modules`, missing map sources, inline/external maps, and output pointing outside `outDir`.
- **Constraints:** Generator ordering and provenance are ecosystem contracts; compiler inclusion and maps remain implementation-defined.
- **Classification / status:** ecosystem convention; **conditional**.
- **Evidence / provenance:** [TSConfig], [Source Map]. C: TS-020. F: PKG-08, PROJ-05, EMIT-07. G: not proposed.

### TS-CAN-017 — One source in multiple programs and editor/batch divergence

- **Feature / fixture obligation:** Place one source in two configured programs with different `types`, `lib`, or strictness, plus an orphan/open excluded file in an inferred editor project. Record batch and language-service results separately.
- **Required variants and failures:** Referenced-source redirection, automatic type acquisition, language-service plugin effects, auto-import source choice, stale editor state, and suggestion diagnostics absent from `tsc`.
- **Constraints:** Program identity, not file path alone, determines globals, types, and diagnostics. Editor plugins and host project selection are outside command-line compiler semantics.
- **Classification / status:** implementation-defined; **conditional**.
- **Evidence / provenance:** [TS Server], [TS Project References], [TSConfig]. C: TS-096. F: PROJ-03, PROJ-06. G: listed as a missing category.

## B. Modules, imports, exports, and namespaces

### TS-CAN-018 — Import/export binding matrix and default exports

- **Feature / fixture obligation:** Exercise named, renamed, default, namespace, and side-effect imports; named/default/star/namespace re-exports; alias chains; anonymous default class/function declarations; a type-only default interface; and a default expression.
- **Required variants and failures:** Missing exports, local shadowing, duplicate local names, `export *` omitting default, `export { default as X }`, a type alias needing an export-specifier form rather than value-position `export default Alias`, and type/value meanings reached through one alias.
- **Constraints:** ECMAScript defines runtime bindings; TypeScript adds type-space aliases and type-only default declarations.
- **Classification / status:** language-defined; **required**.
- **Evidence / provenance:** [ECMA-262], [TS Modules]. C: TS-022, TS-024. F: MOD-01, MOD-02. G: not proposed.

### TS-CAN-019 — Type-only edges and import elision

- **Feature / fixture obligation:** Use `import type`, default and inline type modifiers, `export type`, type-only star/namespace re-exports, and the same unmarked import under legacy elision and `verbatimModuleSyntax`.
- **Required variants and failures:** A class/enum used as type and value; type-only class incorrectly used in `extends` or value position; illegal combined default/named `import type`; `isolatedModules`; side effects lost by elision; and retained imports under verbatim syntax.
- **Constraints:** Syntax spans TypeScript 3.8, 4.5, and 5.0. `verbatimModuleSyntax` rejects incompatible ESM syntax rather than rewriting it to CommonJS.
- **Classification / status:** implementation-defined; **required**.
- **Evidence / provenance:** [TS 3.8], [TS 4.5], [TS 5.0], [TS Modules]. C: TS-023, TS-026. F: MOD-04, MOD-05, CFG-08. G: TS-FEAT-005.

### TS-CAN-020 — Re-export forwarding, stars, and ambiguity

- **Feature / fixture obligation:** Build a barrel with direct, renamed, namespace, type-only, and star re-exports. Have two star sources expose the same name; show the ambiguous name unavailable until an explicit export resolves it.
- **Required variants and failures:** Cyclic barrels, identical origin reached twice, default not forwarded by a star, CommonJS source, explicit local export precedence, and re-export without a local binding.
- **Constraints:** Runtime star-export behavior follows ECMAScript; TypeScript performs the parallel type-space operation.
- **Classification / status:** language-defined; **required**.
- **Evidence / provenance:** [ECMA-262], [TS Modules]. C: TS-024. F: MOD-01. G: not proposed.

### TS-CAN-021 — Side-effect imports and unchecked resolution

- **Feature / fixture obligation:** Include a resolvable bare import, a misspelled bare import, and a non-code side-effect import admitted by a wildcard declaration. Compare default checking with `noUncheckedSideEffectImports`.
- **Required variants and failures:** Asset loader present/absent, package `sideEffects` metadata, arbitrary extensions, CSS/image imports, and an import whose only purpose is runtime patch installation.
- **Constraints:** TypeScript 5.6 added the stricter check. Asset execution and tree-shaking are host/tool contracts, not TypeScript guarantees.
- **Classification / status:** implementation-defined; **required**.
- **Evidence / provenance:** [TS 5.6], [TSConfig]. C: TS-027. F: MOD-01, MOD-11. G: not proposed.

### TS-CAN-022 — Dynamic import, import types, `import.meta`, and top-level await

- **Feature / fixture obligation:** Against one module, demonstrate runtime `import()`, a computed dynamic specifier, `import("m").T`, `typeof import("m")`, `import.meta` with an augmented `ImportMeta`, and top-level `await` with a dependent importer.
- **Required variants and failures:** Type-only versus runtime edges, import-type `resolution-mode`, CommonJS transforms/restrictions, host metadata fields, top-level-await cycles, and invalid module/target combinations.
- **Constraints:** Dynamic import, metadata, and top-level await are ECMAScript/host behavior; import types and checking/emit are TypeScript behavior. Top-level await needs a supported target/module pair.
- **Classification / status:** language-defined; **required**.
- **Evidence / provenance:** [ECMA-262], [TS Modules], [TS 3.8], [TS 5.3], [TSConfig]. C: TS-028, TS-083. F: MOD-07, MOD-08. G: TS-FEAT-002 partly.

### TS-CAN-023 — Import attributes, JSON, arbitrary assets, and arbitrary export names

- **Feature / fixture obligation:** Import JSON with a typed valid/invalid property and a current `with { type: "json" }` attribute where required; contrast legacy assertions; type a custom extension through wildcard and per-file declarations; and import/export a string-named binding that is not an identifier.
- **Required variants and failures:** Default-only JSON ESM surface, named import failure, missing runtime loader, `allowArbitraryExtensions`, `{name}.d.{ext}.ts`, `resolveJsonModule`, declaration-only asset types, and host-specific attribute validation.
- **Constraints:** Import attributes are host/ECMAScript-defined; JSON and extension synthesis are compiler-defined. Arbitrary module-namespace names require TypeScript 5.6.
- **Classification / status:** implementation-defined; **required**.
- **Evidence / provenance:** [TS 5.0], [TS 5.3], [TS 5.6], [TS 5.7], [TSConfig], [Node ESM]. C: TS-029, TS-033. F: MOD-09, MOD-10, MOD-11, CFG-13. G: not proposed.

### TS-CAN-024 — `export =` and import-equals

- **Feature / fixture obligation:** Model a single CommonJS export with `export =`, consume it through `import x = require()`, and merge properties onto the exported class/function through a namespace.
- **Required variants and failures:** Illegal mixing with ordinary exports, `.cts`/`.d.cts`, `export as namespace`, JavaScript `module.exports`, synthetic-default consumer, and restrictions under verbatim or pure-ESM emit.
- **Constraints:** These are TypeScript-specific CommonJS modeling forms and do not freely compose with ECMAScript exports.
- **Classification / status:** language-defined; **required**.
- **Evidence / provenance:** [TS Modules], [TS Declaration Files]. C: TS-030. F: MOD-03. G: TS-FEAT-005 interaction.

### TS-CAN-025 — ECMAScript/CommonJS interoperability

- **Feature / fixture obligation:** Consume callable and object-shaped CommonJS dependencies with namespace, default, and import-equals forms under contrasting `esModuleInterop` and `allowSyntheticDefaultImports` settings; inspect both types and helper emit.
- **Required variants and failures:** `__esModule`, inherited versus own properties, Node native ESM behavior, inaccurate declarations, helper import, default-shape mismatch, and transpiler-produced “fake ESM.”
- **Constraints:** `allowSyntheticDefaultImports` changes checking only; `esModuleInterop` also changes emit. Neither guarantees that declarations match runtime exports.
- **Classification / status:** implementation-defined; **required**.
- **Evidence / provenance:** [TSConfig], [TS Modules], [Node ESM]. C: TS-031. F: CFG-06. G: dual-package discussion.

### TS-CAN-026 — Script files and module detection

- **Feature / fixture obligation:** Use two global scripts sharing declarations, then add `export {}` and show the names become private. Run the same candidates under `legacy`, `auto`, and `force` module detection.
- **Required variants and failures:** Global collisions, `.d.ts` global/module flip, JSX automatic-runtime detection, nearest package `type`, top-level await/module indicators, CommonJS indicators in checked JavaScript, and `isolatedModules` rejecting a script case.
- **Constraints:** Detection rules changed in TypeScript 4.7 and exact `auto` triggers are version-sensitive.
- **Classification / status:** implementation-defined; **required**.
- **Evidence / provenance:** [TS 4.7], [TSConfig], [TS Modules]. C: TS-021. F: MOD-06, AMB-01. G: not proposed.

### TS-CAN-027 — Internal namespaces and namespace aliases

- **Feature / fixture obligation:** Split a namespace across declarations/files, include exported and non-exported members, nested/dotted names, `import Alias = A.B`, and `export import` from a namespace; observe emitted value objects versus type-only erasure.
- **Required variants and failures:** Script versus module scope, ambient namespaces, non-exported members invisible to later merge blocks, emit ordering, legacy `module A {}`, and isolated-transform restrictions.
- **Constraints:** Instantiated namespaces have TypeScript-defined emit; type-only namespaces erase. Namespace aliases can span value and type meanings.
- **Classification / status:** language-defined; **required**.
- **Evidence / provenance:** [TS Namespaces], [TS Declaration Merging]. C: TS-032. F: DECL-06, EMIT-03. G: TS-FEAT-004 partly.

### TS-CAN-028 — Ambient exact, wildcard, and shorthand modules

- **Feature / fixture obligation:** Provide exact-name, wildcard, and shorthand ambient modules, with matching and unmatched imports, and demonstrate the different meaning when the declaration file becomes an external module.
- **Required variants and failures:** One-wildcard pattern rule, relative ambient name restrictions, default versus `export =` shape, shorthand `any`, competing declarations, and runtime module absence despite successful checking.
- **Constraints:** Ambient modules describe types only. An identically written `declare module` inside an external module is normally augmentation rather than a new ambient module.
- **Classification / status:** language-defined; **required**.
- **Evidence / provenance:** [TS Declaration Files], [TS Modules]. C: TS-033. F: MOD-11. G: not proposed.

### TS-CAN-029 — Module and global augmentation

- **Feature / fixture obligation:** Augment an existing exported interface, install the corresponding runtime patch through a side-effect import, and add a global through `declare global`; show behavior with the augmentation file included and excluded.
- **Required variants and failures:** Wrong package copy, erased type-only import omitting required patching, default-export limitation, global collision, augmentation in `.ts` versus `.d.ts`, and attempting to introduce unsupported new top-level exports.
- **Constraints:** Type merging is program-wide after inclusion; runtime patch installation is a separate JavaScript obligation.
- **Classification / status:** language-defined; **required**.
- **Evidence / provenance:** [TS Declaration Merging], [TS Modules]. C: TS-034. F: MOD-12, AMB-02. G: TS-FEAT-004.

### TS-CAN-030 — Universal Module Definition global/module duality

- **Feature / fixture obligation:** Describe one Universal Module Definition (UMD) package with `export =` plus `export as namespace`; consume it as a module and as a script global, then use the global from a module to trigger the normal diagnostic.
- **Required variants and failures:** `allowUmdGlobalAccess`, bundler-provided global, name collision, and runtime/declaration mismatch.
- **Constraints:** This is a TypeScript declaration model for an ecosystem packaging convention.
- **Classification / status:** ecosystem convention; **conditional**.
- **Evidence / provenance:** [TS Declaration Files], [TSConfig]. C: TS-035. F: AMB-05. G: not proposed.

### TS-CAN-031 — Triple-slash directives and legacy concatenation

- **Feature / fixture obligation:** Exercise `reference path`, `types`, `lib`, and `no-default-lib` at valid file positions. In a separate legacy configuration, use `outFile` and references to make emit ordering observable.
- **Required variants and failures:** Missing path/type package, directive after a statement, `preserve="true"`, `noResolve`, generated declaration references, AMD directives, and legacy `none`/AMD/System emit.
- **Constraints:** Triple-slash processing is required; `outFile`, AMD, and ordered concatenation are conditional legacy coverage.
- **Classification / status:** implementation-defined; **required**.
- **Evidence / provenance:** [TS Triple-Slash], [TSConfig]. C: TS-036. F: MOD-13, EMIT-08. G: not proposed.

### TS-CAN-032 — Live bindings and module cycles

- **Feature / fixture obligation:** Demonstrate an ECMAScript-module cycle with mutable exports and top-level reads, contrasted with a CommonJS cycle, plus a type-only cycle that has no runtime evaluation edge.
- **Required variants and failures:** Hoisting-safe function cycle, temporal-dead-zone failure, partially initialized CommonJS exports, barrel-mediated cycle, top-level-await cycle, and bundler rewrite as a separate host result.
- **Constraints:** Evaluation and live binding behavior are ECMAScript/Node-defined; TypeScript may preserve or downlevel syntax but does not make cycles safe.
- **Classification / status:** language-defined; **required**.
- **Evidence / provenance:** [ECMA-262], [Node Modules]. C: TS-025. F: only implicit in MOD-05/CLS-09 discussion. G: not proposed.

### TS-CAN-033 — Deferred module evaluation

- **Feature / fixture obligation:** Under TypeScript 5.9, compare `import defer * as ns from "m"` with an ordinary namespace import of a side-effectful module and make the different evaluation point observable.
- **Required variants and failures:** Namespace-only syntax, first member access, unused deferred binding, interaction with a cycle, and rejection under unsupported `module` modes.
- **Constraints:** This tracks the TC39 deferred-import-evaluation proposal. Exact allowed module modes and proposal/runtime support require confirmation against the pinned compiler and host.
- **Classification / status:** language-defined; **conditional**.
- **Evidence / provenance:** [TS 5.9], [TC39 Deferred Import]. C: not proposed because its baseline ended at 5.7. F: MOD-14. G: not proposed.

## C. Packages, declarations, and dependency structure

### TS-CAN-034 — Package entry points, conditional exports, and type conditions

- **Feature / fixture obligation:** Define public and blocked subpaths with `exports`, distinct `import`/`require` runtime entries, matching type branches, wildcard patterns, and a deliberately mismatched or misordered condition object.
- **Required variants and failures:** `types`, `types@` version selectors, `typings`, `main`, legacy fallback without `exports`, null-blocked paths, deep-import rejection, `default` ordering, `.d.mts`/`.d.cts` correspondence, and a runtime branch whose typed surface disagrees.
- **Constraints:** Node condition-object key order is significant. TypeScript models Node for Node modes and uses a different condition set for bundlers. The exact fallback (`any` versus diagnostic versus alternate declaration) after a misplaced `types` branch is unresolved.
- **Classification / status:** ecosystem convention; **required**.
- **Evidence / provenance:** [Node Packages], [TS Modules], [TS 4.7]. C: TS-012, TS-098. F: PKG-02. G: TS-FEAT-003.

### TS-CAN-035 — Package `imports`, self-references, and custom conditions

- **Feature / fixture obligation:** Resolve a `#`-prefixed internal alias, a package self-name import, and one dependency whose declaration surface changes under `customConditions`.
- **Required variants and failures:** Import/require branches, browser/development/custom conditions, nested maps, wildcard subpaths, an inaccessible internal target, and a condition set that selects different exports for the same specifier.
- **Constraints:** `customConditions` requires TypeScript 5.0 and Node16/NodeNext/`bundler` resolution. Actual condition selection remains host-specific.
- **Classification / status:** implementation-defined; **required**.
- **Evidence / provenance:** [Node Packages], [TS Modules], [TS 5.0]. C: TS-010, TS-012, TS-099. F: PKG-03, RES-11. G: TS-FEAT-003 partly.

### TS-CAN-036 — Declaration discovery, automatic ambient packages, and compiler-version routing

- **Feature / fixture obligation:** Include packages typed through an `exports` `types` branch, `types`/`typings`, conventional `index.d.ts`, and `@types`; restrict ambient inclusion with `types`/`typeRoots`; and use `typesVersions` to change an exported type by compiler range.
- **Required variants and failures:** Scoped `@types/scope__name`, `types: []`, ancestor visibility, automatic type acquisition in JavaScript projects, overlapping/falling-through version ranges, untyped import, duplicated global declarations, and dependency package version skew.
- **Constraints:** Lookup precedence interacts with `exports`; `typesVersions` and installed layout are compiler/package-manager dependent.
- **Classification / status:** implementation-defined; **required**.
- **Evidence / provenance:** [TS Modules], [TS Declaration Publishing], [TSConfig], [DefinitelyTyped]. C: TS-013, TS-092. F: RES-07, RES-08, PKG-04, AMB-04. G: TS-FEAT-012 and TS-FEAT-003.

### TS-CAN-037 — Package-manager identity, peers, and workspace consumption

- **Feature / fixture obligation:** Demonstrate direct/transitive/peer type dependencies, two installed versions of a nominally sensitive declaration, a workspace link, and source-alias versus built-declaration consumption modes.
- **Required variants and failures:** npm hoisting, pnpm isolation/store links, Yarn node-modules or Plug'n'Play when in scope, missing peer, platform/optional dependency, a transitive type package hidden by layout, and lockfile-selected type changes.
- **Constraints:** Package layout and peer selection are ecosystem-defined. Structural types may remain compatible while private/protected members, enums, or unique symbols expose duplicate identity.
- **Classification / status:** ecosystem convention; **required**.
- **Evidence / provenance:** [npm Workspaces], [TS Modules], official selected package-manager documentation. C: TS-015, TS-094. F: RES-10, PKG-05, PKG-06, PKG-07. G: not proposed.

### TS-CAN-038 — Runtime/type entry-point agreement

- **Feature / fixture obligation:** Provide import and require consumers for a dual package, public subpaths, matching declaration flavors, and an intentional mismatch in format, exported name, extension, or condition that type-checks but fails to load—or fails resolution before runtime.
- **Required variants and failures:** Default-export interop, dual-package hazard, types-only export, extensionless declaration, externalized dependency, bundler condition, and declaration/runtime version skew.
- **Constraints:** TypeScript verifies declaration relations, not actual JavaScript shape. Loader success is host-defined.
- **Classification / status:** ecosystem convention; **required**.
- **Evidence / provenance:** [TS Modules], [TS Declaration Files], [Node Packages]. C: TS-098. F: PKG-02, AMB-04, AMB-06. G: dual-package discussion.

### TS-CAN-039 — Platform variants, generated packages, and declaration rollups

- **Feature / fixture obligation:** When these practices are in scope, show condition- or suffix-selected browser/server/native sources, a generated typed package/module, and rolled-up declarations whose public entity layout differs from source.
- **Required variants and failures:** Development/production, browser/server, React-Native suffixes, optional native package, generator missing/stale, source versus `node_modules` generation, re-export chains collapsed, private names inlined/renamed, and declaration drift between branches.
- **Constraints:** Generators, suffix conventions, bundler conditions, and declaration bundlers are tool-specific ecosystem contracts.
- **Classification / status:** ecosystem convention; **conditional**.
- **Evidence / provenance:** [TSConfig], official selected generator/bundler documentation. C: TS-020, TS-099. F: RES-06, PKG-08, PKG-09, PROJ-05. G: not proposed.

## D. Declarations, scope, identity, and object model

### TS-CAN-040 — Separate type, value, and namespace spaces

- **Feature / fixture obligation:** Use one spelling with compatible type, value, and namespace declarations; reference it in annotations, `typeof`, `new`, and qualified access; and link the facets without collapsing distinct meanings.
- **Required variants and failures:** Class/enum as type plus value, interface as type only, namespace value/type facets, import alias carrying multiple meanings, illegal same-space collision, and erased declaration used as a value.
- **Constraints:** Name resolution depends on syntactic position. All declaration sites and distinct meanings must remain observable.
- **Classification / status:** language-defined; **required**.
- **Evidence / provenance:** [TS Declaration Merging], [TS Handbook]. C: TS-037. F: DECL-01. G: TS-FEAT-004 partly.

### TS-CAN-041 — Lexical scope, hoisting, temporal dead zones, and binding patterns

- **Feature / fixture obligation:** Demonstrate `var`, `let`, `const`, function, class, parameter, catch, block, and destructuring bindings with shadowing, closure capture, aliases, defaults, nested rest, and parameter patterns.
- **Required variants and failures:** Legal `var` redeclaration, illegal lexical redeclaration, pre-initialization reference, switch shared scope, loop capture under old target, script `var`/function on `globalThis` versus global lexical bindings, and rest/spread overwrite or getter order.
- **Constraints:** Runtime scope and initialization follow ECMAScript; TypeScript adds diagnostics, inferred pattern types, and downleveling.
- **Classification / status:** language-defined; **required**.
- **Evidence / provenance:** [ECMA-262], [TS Variables]. C: TS-038, TS-052. F: DECL-05, DECL-08. G: not proposed.

### TS-CAN-042 — Function, method, and constructor overload declarations

- **Feature / fixture obligation:** Provide ordered overload signatures with one broader implementation for a function, method, and constructor; show calls resolving to different return types and a call accepted only by the hidden implementation signature being rejected.
- **Required variants and failures:** Incompatible overload/implementation, union argument matching no single overload, optional/rest candidates, ambient overloads without implementation, interface-merging overloads, constructor visibility, and ordering-sensitive selection.
- **Constraints:** Outcomes are TypeScript behavior; detailed tie-breaking and contextual inference are compiler/version sensitive.
- **Classification / status:** implementation-defined; **required**.
- **Evidence / provenance:** [TS Functions], [TS Classes]. C: TS-039, TS-067. F: FUN-01, CLS-13. G: not proposed.

### TS-CAN-043 — Interface extension and declaration merging

- **Feature / fixture obligation:** Reopen an interface across blocks/files, extend multiple bases, merge method overloads, augment a global/library interface, and implement the result in a class.
- **Required variants and failures:** Conflicting property types, mismatched generic parameter lists, specialized literal overloads, module-scoped interfaces that do not merge, duplicate dependency declarations, and an interface extending a class with protected/private origin.
- **Constraints:** Multiple declaration sites form one logical symbol. Exact merged overload ordering requires conformance verification.
- **Classification / status:** language-defined; **required**.
- **Evidence / provenance:** [TS Declaration Merging], [TS Object Types]. C: TS-047. F: DECL-02, TYPE-01. G: TS-FEAT-004.

### TS-CAN-044 — Cross-kind and enum declaration merging

- **Feature / fixture obligation:** Demonstrate namespace merging with a function, class, and enum; class+interface instance merging; multiple enum declarations; correct runtime declaration ordering; and a forbidden merge such as alias reopening or class+class.
- **Required variants and failures:** Exported versus block-private namespace members, value before namespace, enum later block needing an initializer, default export not augmentable by name, module boundaries, and ambient versus emitted merges.
- **Constraints:** Supported combinations join type/value/namespace facets but can have declarations with no implementation site.
- **Classification / status:** language-defined; **required**.
- **Evidence / provenance:** [TS Declaration Merging], [TS Enums]. C: TS-048, TS-049. F: DECL-03, DECL-04, CLS-14. G: TS-FEAT-004.

### TS-CAN-045 — Ambient values, global declaration files, and global augmentations

- **Feature / fixture obligation:** Declare external `var`/`let`/`const`/function/class/enum/namespace values with no emit; consume a global `.d.ts`; flip it to module scope with an import; and extend global object/prototype/environment interfaces from both global and module declarations.
- **Required variants and failures:** `import()` types inside a global declaration without making it a module, runtime value absent despite successful checking, duplicate DOM/worker/test globals, `declare global`, and differing file-set inclusion.
- **Constraints:** Ambient declarations assert host behavior. Global meaning is program-set sensitive and can change with one top-level import/export.
- **Classification / status:** language-defined; **required**.
- **Evidence / provenance:** [TS Declaration Files], [TS Declaration Merging]. C: TS-006, TS-033. F: DECL-07, AMB-01, AMB-02, AMB-07. G: TS-FEAT-012 partly.

### TS-CAN-046 — Computed, symbol, and unique-symbol identities

- **Feature / fixture obligation:** Use literal computed keys, well-known symbols, a declared `unique symbol` shared between interface and object/class, and a dynamic computed key that cannot name one fixed property.
- **Required variants and failures:** `const`/`readonly static` restrictions, symbol index signatures, `keyof` inclusion, declaration emit, string/number/symbol key families, downlevel library/runtime absence, and a unique-symbol brand.
- **Constraints:** Runtime symbols follow ECMAScript; fixed key identity and `unique symbol` restrictions are TypeScript-defined.
- **Classification / status:** language-defined; **required**.
- **Evidence / provenance:** [ECMA-262], [TS Symbols], [TS 2.7]. C: TS-051. F: DECL-09. G: not proposed.

### TS-CAN-047 — Explicit resource management

- **Feature / fixture obligation:** Use `using` and `await using` with synchronous/asynchronous disposable protocols, nested scopes, multiple resources, early return, and thrown completion; make reverse disposal order and cleanup dispatch observable.
- **Required variants and failures:** Invalid resource type, nullish resource, missing `esnext.disposable`, older host polyfill, disposal throwing during another error, `SuppressedError`, async-context restriction, and downlevel helper path.
- **Constraints:** TypeScript support begins in 5.2 and tracks ECMAScript explicit resource management. Runtime support/helper behavior depends on target and host.
- **Classification / status:** language-defined; **required**.
- **Evidence / provenance:** [TS 5.2], [TC39 Resource Management]. C: TS-053. F: DECL-10. G: TS-FEAT-007.

### TS-CAN-048 — Function forms, `this`, and call-site receiver semantics

- **Feature / fixture obligation:** Contrast arrow lexical `this`, ordinary function/method dynamic `this`, method extraction, explicit `this` parameters, `this: void`, contextual `ThisType`, and `.call`/`.bind` checking.
- **Required variants and failures:** `noImplicitThis`, callback receiver declaration, constructor functions in JavaScript, static versus instance `this`, detached call, runtime arity excluding fake `this`, and callback variance.
- **Constraints:** Receiver binding is ECMAScript-defined; `this` parameters and contextual typing erase from output.
- **Classification / status:** language-defined; **required**.
- **Evidence / provenance:** [ECMA-262], [TS Functions], [TS Utility Types]. C: TS-040. F: FUN-03, TYPE-13. G: not proposed.

### TS-CAN-049 — Class heritage, abstractness, conformance, and dispatch

- **Feature / fixture obligation:** Use an abstract base, concrete derived class, overridden method, base-typed polymorphic call, multiple implemented interfaces, an abstract construct signature, and `extends` of a class-producing expression.
- **Required variants and failures:** Missing abstract/interface member, `override` and `noImplicitOverride`, static/constructor compatibility, inherited overloads, field hiding versus override, `super`, protected/private assignability, and `implements` erased versus `extends` runtime edge.
- **Constraints:** Runtime inheritance/dispatch follows ECMAScript; abstractness, interface conformance, and override diagnostics are TypeScript-defined.
- **Classification / status:** language-defined; **required**.
- **Evidence / provenance:** [ECMA-262], [TS Classes], [TS 4.2], [TS 4.3]. C: TS-041, TS-086. F: CLS-01, CLS-05, CLS-07. G: not proposed.

### TS-CAN-050 — Class fields, parameter properties, and initialization semantics

- **Feature / fixture obligation:** Use base/derived initialized and uninitialized fields, a field shadowing a base setter, definite assignment, `declare` field, and public/protected/private/readonly constructor parameter properties.
- **Required variants and failures:** `useDefineForClassFields` define versus assignment emit, target-driven default, initialization around `super`, read before initialization, derived constructor injection, explicit-field collision, destructuring not being a parameter property, and `erasableSyntaxOnly` rejection.
- **Constraints:** ECMAScript defines native fields; downlevel form and parameter-property synthesis are compiler-defined. Node native stripping cannot erase parameter properties without transformation.
- **Classification / status:** implementation-defined; **required**.
- **Evidence / provenance:** [TS Classes], [TSConfig], [TS 3.7], [TS 5.8], [ECMA-262]. C: TS-042, TS-043. F: CLS-03, CLS-11. G: not proposed.

### TS-CAN-051 — Accessors and auto-accessors

- **Feature / fixture obligation:** Demonstrate paired and unpaired getters/setters, getter-only readonly inference, asymmetric read/write types, inherited accessor override, static accessor, and an `accessor` field with declaration/emit shape.
- **Required variants and failures:** TypeScript 4.3 wider setter, TypeScript 5.1 explicitly unrelated getter/setter types, property/accessor override diagnostic, target restrictions, hidden backing storage, and decorator integration.
- **Constraints:** JavaScript accessors are ECMAScript-defined. Type relations and downlevel/auto-accessor emit are compiler/version dependent.
- **Classification / status:** implementation-defined; **required**.
- **Evidence / provenance:** [TS Classes], [TS 4.3], [TS 4.9], [TS 5.1]. C: TS-044. F: CLS-06. G: not proposed.

### TS-CAN-052 — Public, protected, soft-private, and `#private`

- **Feature / fixture obligation:** Use all visibility forms in same-shaped related/unrelated classes, subclass protected access, modifier-private access attempt, hard `#private` access/brand check, and declaration emit.
- **Required variants and failures:** Bracket/cast attempts, private/protected declaration-origin incompatibility, private constructor, static private fields, duplicate package copies, `#x in obj` narrowing, and runtime versus type-only privacy.
- **Constraints:** TypeScript `private`/`protected` erase but affect compatibility; ECMAScript private names have unique lexical identity and runtime enforcement.
- **Classification / status:** language-defined; **required**.
- **Evidence / provenance:** [TS Classes], [TS 4.5], [ECMA-262]. C: TS-045. F: CLS-02, TYPE-18. G: not proposed.

### TS-CAN-053 — Static side, class expressions, and constructor visibility

- **Feature / fixture obligation:** Demonstrate instance type `C` versus static `typeof C`, static fields/methods/private state/blocks, static inheritance, anonymous/named class expressions passed or returned as values, overloaded constructors, and private/protected construction.
- **Required variants and failures:** Generic type parameter unavailable to statics, static `this`, static override, multiple block order, `InstanceType`, positional identity of anonymous classes, singleton/factory, external `new`, and subclassing a private constructor.
- **Constraints:** Runtime class values and blocks follow ECMAScript; TypeScript supplies separate static/instance types and visibility/overload checks.
- **Classification / status:** language-defined; **required**.
- **Evidence / provenance:** [TS Classes], [TS 4.4], [ECMA-262]. C: TS-046. F: CLS-04, CLS-12, CLS-13. G: not proposed.

### TS-CAN-054 — Standard decorators

- **Feature / fixture obligation:** Under standard decorator mode, cover class, method, getter/setter, field, and auto-accessor targets; factories; replacement values; `addInitializer`; typed context; ordering; and `Symbol.metadata` where supported.
- **Required variants and failures:** Static/instance elements, private elements, replacement class, inheritance, incompatible return type, missing metadata library/polyfill, metadata inheritance, and no parameter decorator.
- **Constraints:** TypeScript 5.0 introduced this model and 5.2 added metadata support. It cannot share one compilation mode with legacy semantics. Fine-grained ordering remains a targeted research gap.
- **Classification / status:** language-defined; **required**.
- **Evidence / provenance:** [TS 5.0], [TS 5.2], [TC39 Decorators]. C: TS-054. F: CLS-08. G: TS-FEAT-008.

### TS-CAN-055 — Legacy decorators and emitted design metadata

- **Feature / fixture obligation:** In a separate `experimentalDecorators` configuration, cover class/member/parameter decorators and `emitDecoratorMetadata`; compare emit with standard decorators and make annotation-derived runtime constructor references observable.
- **Required variants and failures:** `design:type`, `design:paramtypes`, `design:returntype`, lossy encodings, metadata-reflection polyfill, import retention/cycles caused by metadata, descriptor mutation, and decorator signature used under the wrong regime.
- **Constraints:** Legacy semantics and metadata are compiler-specific. Reflection support is an ecosystem convention. The two decorator regimes are mutually exclusive per compilation.
- **Classification / status:** implementation-defined; **required**.
- **Evidence / provenance:** [TS Decorators], [TSConfig], [TS 5.0]. C: TS-054. F: CLS-09. G: TS-FEAT-008.

### TS-CAN-056 — TSX parsing and JSX transform modes

- **Feature / fixture obligation:** Pair `.ts`/`.tsx` token streams to show angle-bracket assertion and generic-arrow disambiguation; compile JSX under `preserve`, classic, automatic, development, and conditional React-Native modes; demonstrate injected runtime imports and per-file pragmas.
- **Required variants and failures:** `jsxFactory`, `jsxFragmentFactory`, `jsxImportSource`, `@jsx`, `@jsxFrag`, `@jsxRuntime`, implicit import absent from source, angle-bracket assertion rejected in TSX, and `<T,>` generic arrow parsing.
- **Constraints:** Parsing is TypeScript-defined. Runtime/factory APIs are framework conventions. Some modes are mutually exclusive configurations.
- **Classification / status:** implementation-defined; **required**.
- **Evidence / provenance:** [TS JSX], [TSConfig], [TS 4.1]. C: TS-055. F: JSX-01, JSX-02. G: JSX gap with cited compiler-flag behavior.

### TS-CAN-057 — JSX names, components, attributes, and alternative runtimes

- **Feature / fixture obligation:** Type intrinsic lowercase and component uppercase tags, required/optional props, children, generic components with inferred/explicit arguments, custom intrinsic augmentation, namespace/namespaced attributes, and two JSX runtimes with different typing surfaces.
- **Required variants and failures:** `JSX.IntrinsicElements`, `JSX.ElementType`, classic global versus automatic module-scoped JSX namespace, invalid component tag, custom element, conflicting global JSX declarations, and library-specific special attributes.
- **Constraints:** Core checking is TypeScript-defined; the `JSX` namespace and component conventions come from selected framework declarations. TypeScript 5.1 changed valid element-type modeling.
- **Classification / status:** ecosystem convention; **required**.
- **Evidence / provenance:** [TS JSX], [TS 5.1], [TS 2.9]. C: TS-055. F: JSX-03, JSX-04, JSX-05. G: JSX gap.

## E. Type relations, inference, and flow

### TS-CAN-058 — Primitive, literal, top, bottom, and escape-hatch types

- **Feature / fixture obligation:** Contrast primitives and literals with `object`, `{}`, `unknown`, `any`, `never`, and `void` through assignments, calls, property operations, exhaustiveness, and callback returns.
- **Required variants and failures:** `any` propagation and implicit/evolving `any`; `unknown` refinement; `never` from impossible branches and declaration/expression inference; `void` callback accepting a value return; boxed primitives; nullability; and unreachable code.
- **Constraints:** These are TypeScript type relations; emitted runtime values remain JavaScript values.
- **Classification / status:** language-defined; **required**.
- **Evidence / provenance:** [TS Everyday Types], [TS Narrowing], [TS Compatibility]. C: TS-056. F: TYPE-15. G: strictness/generics gaps only.

### TS-CAN-059 — Structural compatibility, freshness, excess properties, and weak types

- **Feature / fixture obligation:** Assign independently declared equal shapes; contrast a fresh object literal with an intermediate variable; pass an extra-property object through assertion, spread, `satisfies`, and generic inference; and include a weak all-optional target.
- **Required variants and failures:** Required/optional/readonly/call/construct/index members, stored extra properties, excess typo, weak-type no-common-property error, index signature escape, generic freshness loss, private-origin exception, a branded primitive intersection rejecting its raw primitive, recursive shapes, and exact optional types.
- **Constraints:** Excess-property checking is not exact-object typing. Compatibility is principally structural with intentional unsoundness.
- **Classification / status:** implementation-defined; **required**.
- **Evidence / provenance:** [TS Compatibility], [TS Object Types]. C: TS-057, TS-058. F: TYPE-14. G: not proposed.

### TS-CAN-060 — Unions, intersections, and discriminated unions

- **Feature / fixture obligation:** Build tagged and untagged unions, narrow their members, intersect compatible call/object types, and show contradictory intersections reducing properties or entire types to `never`.
- **Required variants and failures:** Optional/mutated discriminant, union absorption, object-literal union, private-member intersections, default branch masking a new variant, computed normalization, and conditional-type distribution interaction.
- **Constraints:** Core relations are TypeScript-defined; normalization, member ordering, and displayed aliases are compiler/version dependent.
- **Classification / status:** implementation-defined; **required**.
- **Evidence / provenance:** [TS Narrowing], [TS Object Types]. C: TS-059. F: TYPE-03. G: generics gap.

### TS-CAN-061 — Control-flow narrowing and alias analysis

- **Feature / fixture obligation:** Exercise `typeof`, equality, truthiness, `in`, `instanceof`, discriminants, assignment/reachability, aliased conditions, destructured discriminants, `switch (true)`, closure narrowing, and constant indexed-access narrowing.
- **Required variants and failures:** Empty string/zero lost by truthiness, optional property on both `in` branches, cross-realm/custom `Symbol.hasInstance`, mutation/reassignment invalidating facts, callback timing, captured variables, early return, loop, and unsafe call boundary.
- **Constraints:** Runtime predicates follow ECMAScript; flow analysis depth and refinements changed across TypeScript 4.4, 5.3, 5.4, and 5.5.
- **Classification / status:** implementation-defined; **required**.
- **Evidence / provenance:** [TS Narrowing], [TS 4.4], [TS 5.3], [TS 5.4], [TS 5.5]. C: TS-060. F: FUN-04. G: not proposed.

### TS-CAN-062 — User-defined predicates and assertion signatures

- **Feature / fixture obligation:** Provide explicit parameter and `this` predicates, a generic predicate, `asserts condition`, `asserts x is T`, and a TypeScript 5.5 inferred predicate used by `filter`.
- **Required variants and failures:** Predicate target not assignable to parameter, assertion callee lacking required explicit declaration shape, mutation after narrowing, an intentionally unsound predicate body trusted by the compiler, and older compiler retaining a wider result.
- **Constraints:** The compiler trusts declared predicate contracts rather than proving bodies. Inferred predicates are version-dependent.
- **Classification / status:** implementation-defined; **required**.
- **Evidence / provenance:** [TS Narrowing], [TS 3.7], [TS 5.5]. C: TS-061. F: TYPE-12, FUN-05. G: not proposed.

### TS-CAN-063 — Nullability and optional member/parameter distinctions

- **Feature / fixture obligation:** Contrast `null`, `undefined`, missing optional property, required property containing `undefined`, optional/default parameter, optional tuple element, and presence tests under strict and legacy null modes.
- **Required variants and failures:** `exactOptionalPropertyTypes`, mapped optional modifiers, object spread, JSON omission, optional chaining that does not validate an unrelated value, explicit `undefined` write, and non-strict collapse.
- **Constraints:** Static relations depend on `strictNullChecks` and exactness. Runtime object presence follows ECMAScript.
- **Classification / status:** implementation-defined; **required**.
- **Evidence / provenance:** [TSConfig], [TS 4.4], [TS Everyday Types]. C: TS-062. F: CFG-01, CFG-02. G: strictness gap.

### TS-CAN-064 — Index signatures and indexed access configuration

- **Feature / fixture obligation:** Define string, number, symbol, template-pattern, union, and readonly index signatures; include compatible/incompatible named members and reads under checked/unchecked access settings.
- **Required variants and failures:** Number index type constrained by string index type, array/tuple indexing, `keyof` widening, undeclared key gaining `undefined`, dot access forbidden by configuration, explicit member override, and symbol/template support by version.
- **Constraints:** Key behavior is TypeScript-defined; result types vary with `noUncheckedIndexedAccess` and `noPropertyAccessFromIndexSignature`.
- **Classification / status:** implementation-defined; **required**.
- **Evidence / provenance:** [TS Object Types], [TSConfig], [TS 4.1], [TS 4.4]. C: TS-063. F: TYPE-01, CFG-03. G: strictness gap.

### TS-CAN-065 — Readonly views and runtime mutability

- **Feature / fixture obligation:** View one mutable object through mutable and readonly aliases; exercise readonly property/index/array/tuple forms, mapped modifier add/remove, shallow const assertion, nested mutation, and runtime mutation through another alias.
- **Required variants and failures:** Getter-only property, readonly-array variance, `Object.freeze` typing, private state, write rejection through readonly view, and assignability convenience/unsoundness.
- **Constraints:** `readonly` is normally a static view and does not freeze an object. Actual immutability depends on runtime/library behavior.
- **Classification / status:** language-defined; **required**.
- **Evidence / provenance:** [TS Object Types], [TS Utility Types], [TS 3.4]. C: TS-064. F: TYPE-04, TYPE-06. G: not proposed.

### TS-CAN-066 — Arrays, tuples, labels, and variadic relations

- **Feature / fixture obligation:** Demonstrate fixed, optional, readonly, labeled, leading/middle/rest, and generic variadic tuples through indexing, destructuring, concatenation, parameter lists, and spread into calls.
- **Required variants and failures:** Out-of-bounds access, optional-before-required constraint, widening to array, union rest, `noUncheckedIndexedAccess`, labels preserved but identity-neutral, readonly mutation failure, and target/runtime iterable requirements.
- **Constraints:** Tuple typing is TypeScript-defined; runtime values are ECMAScript arrays.
- **Classification / status:** language-defined; **required**.
- **Evidence / provenance:** [TS Object Types], [TS 4.0], [TS 4.2]. C: TS-065. F: TYPE-06. G: recursive tuple gap.

### TS-CAN-067 — Callable compatibility and parameter variance

- **Feature / fixture obligation:** Assign callbacks with broader/narrower parameters, different counts, optional/rest parameters, discarded return values, generic callbacks, and method versus function-property declarations under strict and permissive checking.
- **Required variants and failures:** Contravariant function property, intentionally bivariant method, `void` target return, constructor signature, overload set, event-handler convention, mutable generic container, and `strictFunctionTypes` off.
- **Constraints:** Compatibility includes deliberate unsoundness for common JavaScript patterns. Method and function-property variance differ.
- **Classification / status:** implementation-defined; **required**.
- **Evidence / provenance:** [TS Compatibility], [TSConfig]. C: TS-066. F: CFG-01, TYPE-15. G: strictness gap.

### TS-CAN-068 — Generic declarations, constraints, defaults, and inference

- **Feature / fixture obligation:** Use generic functions, classes, interfaces, aliases, call/construct signatures, constraints, defaults, explicit and inferred arguments, nested shadowed parameters, and context/return-driven inference.
- **Required variants and failures:** Failed `keyof` constraint, empty candidate set, circular constraint, default referencing earlier parameter, partial inference limitation, higher-order inference, recursive generic, and inference across overload candidates.
- **Constraints:** Generic syntax and relations are TypeScript-defined; candidate collection, priorities, and inference fixes are compiler/version dependent.
- **Classification / status:** implementation-defined; **required**.
- **Evidence / provenance:** [TS Generics], [TS release notes]. C: TS-068. F: TYPE-07. G: only const parameters proposed.

### TS-CAN-069 — Const parameters, instantiation expressions, `NoInfer`, and variance annotations

- **Feature / fixture obligation:** Pair ordinary and `const` type-parameter APIs; specialize a generic function value with an instantiation expression; block one inference site using `NoInfer`; and declare correct/incorrect `in`, `out`, and `in out` generic variance.
- **Required variants and failures:** Inline versus pre-widened argument, readonly candidate falling back against mutable constraint, explicit arguments, discarded incompatible overloads after specialization, wrong variance diagnostic, measured versus annotated recursive variance, and structural expansion outside annotation effect.
- **Constraints:** Instantiation expressions/variance require 4.7; const parameters 5.0; `NoInfer` 5.4.
- **Classification / status:** implementation-defined; **required**.
- **Evidence / provenance:** [TS 4.7], [TS 5.0], [TS 5.4]. C: TS-069, TS-081. F: TYPE-07. G: TS-FEAT-011.

### TS-CAN-070 — Conditional types, distribution, and `infer`

- **Feature / fixture obligation:** Implement distributive and bracket-suppressed conditional types, constrained/nested `infer`, `never` input, overload return extraction, and a recursive unwrapping transformation.
- **Required variants and failures:** `any`, `unknown`, multiple inference sites and variance, last-overload extraction, deferred generic condition, constrained `infer`, recursion, and excessive-instantiation diagnostic.
- **Constraints:** Type operations are TypeScript-defined; recursion depth, performance cutoffs, normalization, and some inference results are compiler-defined.
- **Classification / status:** implementation-defined; **required**.
- **Evidence / provenance:** [TS Conditional Types], [TS 4.7]. C: TS-070. F: TYPE-08. G: explicitly identified as missing.

### TS-CAN-071 — Mapped types and key remapping

- **Feature / fixture obligation:** Exercise homomorphic modifier preservation, `+`/`-` optional and readonly modifiers, key filtering via `never`, renaming via `as`, template-generated names, tuple/array mapping, and remapped-key collisions.
- **Required variants and failures:** Symbol/number keys, exact optional properties, union sources, recursive maps, index signatures, duplicate remapped keys, and non-homomorphic loss of modifiers.
- **Constraints:** These are TypeScript type-level operations; complex recursive behavior and presentation are compiler-sensitive.
- **Classification / status:** language-defined; **required**.
- **Evidence / provenance:** [TS Mapped Types], [TS 4.1]. C: TS-071. F: TYPE-09. G: generics gap.

### TS-CAN-072 — `keyof`, indexed access, and type queries

- **Feature / fixture obligation:** Derive APIs with `keyof typeof value`, generic property lookup `T[K]`, array element extraction, nested indexed access, `typeof` functions/classes/namespaces, and `typeof import()`.
- **Required variants and failures:** String-index widening of `keyof`, number/symbol keys, optional result including `undefined`, unions versus intersections, invalid non-key indexing, and same spelling in value/type spaces.
- **Constraints:** Type queries create static edges to values without executing them.
- **Classification / status:** language-defined; **required**.
- **Evidence / provenance:** [TS Type Operators], [TS Indexed Access]. C: TS-072. F: TYPE-10. G: not proposed.

### TS-CAN-073 — Template literal types and intrinsic string transforms

- **Feature / fixture obligation:** Generate an event-name/key API from object keys, cross-product multiple union substitutions, parse a substring with conditional inference, and apply `Uppercase`, `Lowercase`, `Capitalize`, and `Uncapitalize`.
- **Required variants and failures:** Wide `string`, numeric/bigint/boolean interpolation, key-remapping interaction, Unicode/casing behavior, combinatorial expansion, recursive parser, and instantiation limit.
- **Constraints:** Intrinsic transforms and complexity limits are compiler-defined; the surface syntax is TypeScript-defined.
- **Classification / status:** implementation-defined; **required**.
- **Evidence / provenance:** [TS Template Types], [TS 4.1]. C: TS-073. F: TYPE-05. G: explicitly identified as missing.

### TS-CAN-074 — Recursive and circular type declarations

- **Feature / fixture obligation:** Include recursive interface/tree, recursive tuple/object alias, mutually recursive aliases, a forbidden immediate cycle, and a conditional/mapped recursion that reaches a depth diagnostic.
- **Required variants and failures:** Circular property reference, recursive conditional, serialization-shaped cycle, alias versus interface presentation/performance, and compiler-version limit change.
- **Constraints:** Accepted laziness and rejected immediate cycles are compiler behavior; numerical recursion/instantiation limits are intentionally unspecified.
- **Classification / status:** implementation-defined; **required**.
- **Evidence / provenance:** [TS release notes], TypeScript compiler conformance tests. C: TS-074. F: TYPE-02, TYPE-08. G: recursive tuple gap.

### TS-CAN-075 — Interfaces versus type aliases

- **Feature / fixture obligation:** Define equivalent object shapes as interface and alias; reopen only the interface; use an alias for primitive/union/tuple/conditional forms; compare interface extension with alias intersection and class implementation.
- **Required variants and failures:** Alias reopening error, interface extending a union, incompatible intersection properties, generic recursion, module augmentation, declaration emit/hover alias preservation, and diagnostic expansion.
- **Constraints:** Semantic capabilities are language-defined; presentation and alias preservation are implementation-defined.
- **Classification / status:** language-defined; **required**.
- **Evidence / provenance:** [TS Object Types], [TS Everyday Types]. C: TS-075. F: TYPE-02. G: not proposed.

### TS-CAN-076 — Call, construct, overloaded, and hybrid object types

- **Feature / fixture obligation:** Create a stateful callable object with properties, separate call and construct signatures returning different types, generic signatures, overloads, and index members; assign a runtime implementation to it.
- **Required variants and failures:** Abstract construct signature, constructor `this`, class static side, interface merging, JavaScript function expandos, number/string index compatibility, and overload resolution.
- **Constraints:** Static hybrid shapes are TypeScript-defined; actual callability/constructability follows ECMAScript runtime behavior.
- **Classification / status:** language-defined; **required**.
- **Evidence / provenance:** [TS Functions], [TS Object Types], [TS Generics]. C: TS-076. F: TYPE-01. G: not proposed.

### TS-CAN-077 — Polymorphic `this` and receiver-linked types

- **Feature / fixture obligation:** Give a base fluent method return type `this`, chain it through a derived subtype, add a `this is T` predicate, and contrast with a method returning the named base type.
- **Required variants and failures:** Detached call, explicit `this` parameter, static-side contrast, F-bounded generic alternative, mixin/intersection builder, private/protected state, and arrow-property method.
- **Constraints:** Receiver-dependent type is TypeScript-defined; detached runtime call behavior is ECMAScript-defined.
- **Classification / status:** language-defined; **required**.
- **Evidence / provenance:** [TS Classes], [TS Narrowing]. C: TS-077. F: TYPE-13. G: not proposed.

### TS-CAN-078 — Contextual typing, best common type, and literal widening

- **Feature / fixture obligation:** Present the same callback, object, and array with and without context; contrast `let`, `const`, readonly property, const assertion, annotation, and a heterogeneous array whose best common type changes.
- **Required variants and failures:** Contextual return, TypeScript 5.1 `undefined`-return inference, generic/overload context, union contextual signatures, evolving arrays, JSX callback, no common candidate, const context, already-widened variable, and version-sensitive inference fix.
- **Constraints:** These inference algorithms and displayed results are compiler-defined.
- **Classification / status:** implementation-defined; **required**.
- **Evidence / provenance:** [TS Type Inference], [TS 3.4], [TS release notes]. C: TS-078, TS-079. F: TYPE-04. G: TS-FEAT-009 partly.

### TS-CAN-079 — Assertions, definite assignment, and `satisfies`

- **Feature / fixture obligation:** Compare annotation with `satisfies`; combine `as const satisfies`; use widening/narrowing/double assertions, postfix non-null, and declaration definite-assignment; expose at least one trusted claim that fails at runtime.
- **Required variants and failures:** Misspelled excess key, preserved literal/member type, assertion between unrelated types requiring a double bridge, TSX angle-bracket restriction, non-null runtime failure, lifecycle-initialized field, and JSDoc `@satisfies`.
- **Constraints:** Assertions and `satisfies` erase. `satisfies` requires 4.9; JSDoc form requires 5.0.
- **Classification / status:** language-defined; **required**.
- **Evidence / provenance:** [TS 4.9], [TS 5.0], [TS Everyday Types], [TS Classes]. C: TS-079, TS-080. F: TYPE-11, FUN-06, JS-02. G: TS-FEAT-009.

### TS-CAN-080 — Utility types, compiler intrinsics, and value-derived type chains

- **Feature / fixture obligation:** Apply property-transform, union-filter, function-reflection, `this`, string, and `Awaited` utilities to unions, overloads, optional properties, `any`, and `never`; build a small value/schema API whose exported type exists only through generic instantiation.
- **Required variants and failures:** Last-overload extraction, recursive thenable, exact optional properties, local shadowing of `Partial`, missing lib utility, inferred public type that cannot be named, and `isolatedDeclarations` repair with annotation.
- **Constraints:** Utilities live in bundled libraries, while some names map to compiler intrinsics. Value-derived API patterns are ecosystem convention over language mechanisms.
- **Classification / status:** implementation-defined; **required**.
- **Evidence / provenance:** [TS Utility Types], bundled `lib.*.d.ts`, [TS 5.5]. C: TS-082. F: TYPE-16, TYPE-17. G: not proposed.

## F. Runtime-visible constructs and emit

### TS-CAN-081 — Enums and const-enum boundaries

- **Feature / fixture obligation:** Exercise numeric, string, heterogeneous, constant, computed, union-member, merged, and namespace-augmented enums; observe numeric reverse maps and missing string reverse maps; compare ordinary and `const enum` emit locally and across a declaration boundary.
- **Required variants and failures:** Invalid initializer ordering, duplicate value, bit flag, `preserveConstEnums`, ambient const enum, isolated-transform rejection, stale cross-package inline value, publication-time deconstification, runtime object absent, and assignment between distinct enum types.
- **Constraints:** Enum emit and type rules are TypeScript-defined. Exact number/enum assignability changed around TypeScript 5.0 and requires a pinned conformance case.
- **Classification / status:** implementation-defined; **required**.
- **Evidence / provenance:** [TS Enums], [TS 5.0], [TSConfig]. C: TS-049, TS-050. F: DECL-04, EMIT-01, EMIT-02, TYPE-18. G: declaration-merging item partly.

### TS-CAN-082 — Async functions, promises, and thenable adoption

- **Feature / fixture obligation:** Use async function/method, rejection, custom thenable, nested promise-like unwrapping, and an importer whose evaluation depends on top-level await.
- **Required variants and failures:** Old-target helper emit, missing Promise declaration/runtime, CommonJS top-level-await restriction, cyclic async module, `Awaited` static result, and host unhandled-rejection policy.
- **Constraints:** Promise/await runtime behavior follows ECMAScript; checking, library types, and downleveling are compiler-defined.
- **Classification / status:** language-defined; **required**.
- **Evidence / provenance:** [ECMA-262], [TS Functions], [TSConfig]. C: TS-083. F: FUN-02, MOD-08. G: TS-FEAT-002 partly.

### TS-CAN-083 — Generators, async generators, and iteration protocols

- **Feature / fixture obligation:** Use typed generator yield/return/next channels, delegated yield, async generator, custom iterable/iterator, `for...of`, `for await...of`, and early-loop cleanup.
- **Required variants and failures:** Mismatched next/yield type, arrays versus custom iterable, missing iterable libs, `downlevelIteration`, async-from-sync iteration, old/new target emit, and TypeScript 5.6 iterator-library type changes.
- **Constraints:** Protocol execution is ECMAScript-defined; standard declaration shapes and downlevel helpers are compiler-defined.
- **Classification / status:** implementation-defined; **required**.
- **Evidence / provenance:** [ECMA-262], [TS 2.3], [TS 5.6], [TSConfig]. C: TS-084. F: FUN-02. G: not proposed.

### TS-CAN-084 — Optional chains, nullish coalescing, and logical assignment

- **Feature / fixture obligation:** Exercise optional property, element, and call chains; receiver-preserving optional method call; grouping that breaks the chain; `??` versus `||` on zero/empty string; and `&&=`, `||=`, `??=` with flow effects.
- **Required variants and failures:** Side-effectful getter/index skipped or evaluated, invalid assignment target, mixed logical precedence diagnostic, delete, old-target temporaries, and strict-null result type.
- **Constraints:** Runtime short-circuit/evaluation follows ECMAScript; narrowing and downlevel output are compiler behavior.
- **Classification / status:** language-defined; **required**.
- **Evidence / provenance:** [ECMA-262], [TS 3.7], [TS 4.0]. C: TS-085. F: FUN-07. G: not proposed.

### TS-CAN-085 — Prototypes, interface erasure, and mixins

- **Feature / fixture obligation:** Demonstrate prototype dispatch through a base reference, extracted receiver loss, prototype mutation, interface-only conformance with no runtime heritage, and two class-expression mixins composed over a base.
- **Required variants and failures:** `super`, arrow-property dispatch, `instanceof`, `Object.assign` shallow copy, private fields, statics, abstract constructor constraints, duplicate names, intersection instance types, and difficult anonymous-class declaration emit.
- **Constraints:** Prototype behavior is ECMAScript-defined. Mixin types are a documented TypeScript/ecosystem pattern rather than a distinct runtime feature.
- **Classification / status:** ecosystem convention; **required**.
- **Evidence / provenance:** [ECMA-262], [TS Mixins], [TS Classes]. C: TS-086. F: CLS-10. G: not proposed.

### TS-CAN-086 — Exceptions, reachability, `finally`, and exhaustiveness

- **Feature / fixture obligation:** Throw Error and non-Error values, guard an `unknown` catch, demonstrate `finally` overriding completion, call a `never`-returning function, and handle complete/incomplete discriminated unions or enums with a `never` proof.
- **Required variants and failures:** `useUnknownInCatchVariables`, async rejection, assertion function, unreachable statement, disposal error, switch fallthrough, optional discriminant, default masking additions, and linter-only exhaustiveness rule distinguished from compiler behavior.
- **Constraints:** Exception/completion semantics follow ECMAScript; reachability/narrowing/diagnostics are compiler-defined.
- **Classification / status:** implementation-defined; **required**.
- **Evidence / provenance:** [ECMA-262], [TS Narrowing], [TSConfig], [TS 4.4]. C: TS-087, TS-088. F: FUN-05. G: not proposed.

### TS-CAN-087 — Evaluation order and helper-mediated downlevel behavior

- **Feature / fixture obligation:** Make order observable across computed keys, getters, spreads, arguments, class elements, decorators, and cleanup; compare native and lowered targets; use inline helpers and `importHelpers`/`tslib`.
- **Required variants and failures:** Helper deduplication, incompatible/old `tslib`, iterable assumptions in spread, native versus transformed fields, decorator regime, cleanup during throw, and helper-created runtime dependency absent from authored imports.
- **Constraints:** Intended order follows ECMAScript/proposal semantics; helper selection and downlevel implementation are compiler/toolchain behavior.
- **Classification / status:** implementation-defined; **required**.
- **Evidence / provenance:** [ECMA-262], [TSConfig], official `tslib` package documentation. C: TS-089. F: EMIT-05. G: not proposed.

### TS-CAN-088 — Diagnostic phases, suppression, recovery, and version gates

- **Feature / fixture obligation:** Include isolated parse, binding/redeclaration, type, resolution, declaration-emit, and invalid-option errors; demonstrate partial parser recovery; and use `@ts-ignore`, `@ts-expect-error`, `@ts-check`, and `@ts-nocheck` including an unused expectation.
- **Required variants and failures:** Cascading/related/global diagnostics, stable code versus unstable text/order, missing/untyped import yielding a dangling/`any` edge, errors with emit, editor suggestions, newer syntax under older compiler, deprecated/removed option plus `ignoreDeprecations`, 5.5 regex checking, 5.6 always-truthy/nullish checks, 5.7 never-initialized checks, and 5.8 return-branch refinements.
- **Constraints:** Diagnostic identity, order, recovery, and version gates are compiler-defined.
- **Classification / status:** implementation-defined; **required**.
- **Evidence / provenance:** [TSConfig], [TS release notes], TypeScript compiler API/conformance tests. C: TS-090, TS-091. F: CFG-14, DIAG-01–DIAG-06. G: failure cases across all items.

### TS-CAN-089 — Dependency declaration quality and library-check policy

- **Feature / fixture obligation:** Supply a dependency declaration with an internal contradiction and a consumer-visible exported type; compare `skipLibCheck` on/off; include a declaration/runtime mismatch that remains invisible to checking.
- **Required variants and failures:** Duplicate globals, errors still surfacing at use sites, `skipDefaultLibCheck`, local versus dependency `.d.ts`, patched dependency, unsupported declaration syntax under older compiler, and duplicate versions.
- **Constraints:** `skipLibCheck` suppresses declaration-file body checking but neither removes all referenced types nor establishes runtime correctness.
- **Classification / status:** implementation-defined; **required**.
- **Evidence / provenance:** [TSConfig], [TS Declaration Files]. C: TS-093. F: CFG-09, AMB-06. G: ambient/package discussion.

## G. Alternative tools and hosts

### TS-CAN-090 — Alternate transpilers, loaders, and compiler transformations

- **Feature / fixture obligation:** When a non-`tsc` path is in scope, identify its config, retain a separate TypeScript check, and compare one construct whose transform/rejection differs: enum, namespace, const enum, decorator, class field, JSX, import elision, or isolated declaration.
- **Required variants and failures:** Babel/SWC/esbuild-style per-file transform, transpile-on-load, ESM loader versus CommonJS register hook, ignored `paths`, source-map/helper differences, compiler API before/after/declaration transformer, synthetic node without source position, and generated runtime edge absent from source.
- **Constraints:** Each tool defines its own supported syntax and emit. TypeScript custom transformers do not provide a stable type-checking extension contract.
- **Classification / status:** ecosystem convention; **conditional**.
- **Evidence / provenance:** [Babel TypeScript], [SWC TypeScript], [esbuild TypeScript], [TS Compiler API]. C: TS-095, TS-097. F: RUN-02, RUN-03, final audit custom transformers. G: build-system-plugin gap.

### TS-CAN-091 — Runtime-native TypeScript and non-Node resolution hosts

- **Feature / fixture obligation:** For each selected runtime, demonstrate the supported type-stripping/execution profile and a counterexample. Node coverage must contrast erasable syntax with enum/namespace/parameter-property/import-equals syntax; Deno/Bun coverage must show their distinct specifier/resolution model.
- **Required variants and failures:** Node `erasableSyntaxOnly`, real relative extensions, no `paths`, transform flag/version gate, Deno URL/`npm:`/`jsr:`/import-map resolution, Bun partial tsconfig support, and execution without type checking.
- **Constraints:** Host version boundaries are volatile and must be pinned. Deno and Bun are conditional rather than assumed Node-baseline behavior.
- **Classification / status:** ecosystem convention; **conditional**.
- **Evidence / provenance:** [Node TypeScript], [TS 5.8], [Deno TypeScript], [Bun TypeScript]. C: TS-095, TS-099. F: RUN-01, RUN-04. G: not proposed.

### TS-CAN-092 — Native compiler port and future official implementations

- **Feature / fixture obligation:** Do not treat TypeScript 7/`tsgo` preview parity, TypeScript 6 migration behavior, diagnostic ordering, compiler API, or language-service behavior as interchangeable with `tsc` until the exact implementation/version is selected and compared.
- **Required variants and failures:** Configuration migration, unsupported API/plugin surface, differing performance cutoffs, diagnostic ordering, and editor integration.
- **Constraints:** The Fable report describes parity as a goal, not established equivalence. No supplied report verifies post-5.9 release state.
- **Classification / status:** unspecified; **unresolved research gap**.
- **Evidence / provenance:** [TS Native Port]; release/parity state still requires a fresh primary-source pass. C: TS-100 later-version warning. F: RUN-05. G: not proposed.

## Version-transition audit

These transitions must be represented by the relevant canonical items or rechecked when the pinned compiler changes.

| Version | Material observable changes from the input reports |
| --- | --- |
| 4.7 | Node16/NodeNext resolution and per-file format; `.mts`/`.cts`; `moduleDetection`; `moduleSuffixes`; instantiation expressions; variance annotations; constrained `infer`. |
| 4.8 | Intersection reduction, narrowing, and stricter unconstrained generic behavior. |
| 4.9 | `satisfies`, auto-accessors, and unlisted-property `in` narrowing. |
| 5.0 | Standard decorators, const type parameters, `verbatimModuleSyntax`, `bundler`, custom conditions, arbitrary/imported TypeScript extensions, type-only stars, array config inheritance, enum analysis changes, and option deprecations. |
| 5.1 | `undefined`-return inference, getter/setter refinements, `JSX.ElementType`, and namespaced JSX attributes. |
| 5.2 | `using`/`await using`, decorator metadata context, and tuple-label preservation. |
| 5.3 | Import attributes, import-type resolution mode, `switch (true)`/comparison narrowing, and custom `Symbol.hasInstance` narrowing. |
| 5.4 | `NoInfer`, preserved closure narrowing, and `module: preserve`. |
| 5.5 | Inferred predicates, `isolatedDeclarations`, JSDoc `@import`, `${configDir}`, declaration portability, indexed-access flow, and regular-expression syntax checking. |
| 5.6 | Side-effect import checking, arbitrary module identifiers, `noCheck`, truthy/nullish diagnostics, iterator-library changes, and build options. |
| 5.7 | Relative import-extension rewriting, ES2024 target/library changes, never-initialized checks, JSON attribute validation, and further control-flow checks. |
| 5.8 | `erasableSyntaxOnly`, Node-mode `require()`/ESM changes, `libReplacement`, return-branch checks, and the `node18` module snapshot. |
| 5.9 | Deferred import evaluation, the `node20` module snapshot, and changed configuration-generation defaults; exact constraints require the gaps below. |

## Unresolved disagreements and unsupported claims

These items are not part of the required checklist until verified against the exact compiler/host and a primary source or compiler conformance test.

### TS-GAP-001 — Exact current `import defer` constraints

- Fable uniquely proposes TypeScript 5.9 `import defer` and cites the official release notes/proposal. Verify the exact allowed `module` values, runtime availability, and cycle behavior. Status: **unresolved research gap**.

### TS-GAP-002 — Conditional `types` ordering failure outcome

- Google states that `types` must precede `import`/`require` and otherwise resolution falls to untyped `any`. Node condition order is authoritative, and TypeScript recommends a `types` condition, but exact fallback depends on selected resolution mode, file names, adjacent declarations, and compiler version. Preserve the broken-order case without hard-coding one diagnostic/result. Status: **unresolved research gap**.

### TS-GAP-003 — `baseUrl` deprecation

- Google calls `baseUrl` deprecated; Codex calls precedence deprecated or version-shifted; Fable only states that modern `paths` no longer requires it. The supplied official evidence supports “not required,” not general deprecation. Do not mark `baseUrl` deprecated without new primary evidence. Status: **unresolved research gap**.

### TS-GAP-004 — Project-reference cycles

- Google says a cycle can cause failure or an infinite loop. Official project references define build ordering, but no supplied primary citation establishes “infinite loop.” Require a cycle failure case, then record the pinned compiler's actual diagnostic. Status: **unresolved research gap**.

### TS-GAP-005 — Version-sensitive file and format details

- Verify `.ts` versus sibling `.d.ts` precedence, exact `moduleDetection: auto` triggers, frozen `node18`/`node20` module-mode behavior, JSON attribute enforcement version, Node native-stripping version gates, and current TypeScript 5.9 defaults. Status: **unresolved research gap**.

### TS-GAP-006 — Fine-grained decorators and resource cleanup

- Verify standard-decorator evaluation/application order across static/instance/private/auto-accessor elements, metadata inheritance, replacement classes, and explicit-resource-management `SuppressedError`/helper behavior under the chosen target and `tslib`. Status: **unresolved research gap**.

### TS-GAP-007 — Enum compatibility boundaries

- Fable describes enums as nominal and flags TypeScript 5.0 numeric-enum changes; Codex also treats enum assignability as version-sensitive. Pin exact cross-enum and number/literal assignments in compiler conformance tests before stating a universal rule. Status: **unresolved research gap**.

### TS-GAP-008 — Documentation-comment semantics

- Fable's final audit alone proposes TSDoc/JSDoc `@deprecated` and `@see` as editor/API-tool metadata. The reports do not provide enough primary evidence or observable compiler contract to require it. Status: **unresolved research gap**.

### TS-GAP-009 — Tool- and host-specific breadth

- Yarn Plug'n'Play, language-service protocol details, framework template compilers, type-aware lint programs, watch-event coalescing, CSS module scripts, import maps, and package-manager resolver plugins need separate primary research when those hosts are selected. Status: **unresolved research gap**.

## Completeness audit

### Input-report items omitted or merged

- **Codex:** No numbered semantic item from TS-001 through TS-100 was silently omitted. Each was merged into TS-CAN-001 through TS-CAN-092. Repeated cross-cutting content—target/lib, top-level await, package identity, generated maps, enums, and module resolution—appears once with its interactions preserved. Pure confidence prose and proposed downstream graph concerns were not converted into independent language features.
- **Fable:** All numbered MOD, RES, DECL, TYPE, CLS, FUN, EMIT, JSX, AMB, JS, CFG, PKG, PROJ, DIAG, and RUN items are represented. Closely related items were merged only where their observable obligation is the same. Fable-only future/uncertain claims (`import defer`, native compiler port, exact Node gates) remain conditional or gaps. Its final-audit notes on TSDoc, Plug'n'Play, transformers, language-service plugins, and other runtimes are conditional items or TS-GAP-008/009.
- **Google:** TS-FEAT-001 through TS-FEAT-012 map respectively to TS-CAN-004, 009/010, 034, 043/044/029, 019, 011, 047, 054/055, 079, 013, 069, and 036. Its recommendation that a fixture “must” use one monorepo design was omitted as downstream project design, which the synthesis prompt forbids. Claims about `baseUrl`, infinite-loop cycles, and deterministic `types` fallback were moved to research gaps rather than accepted.

### Categories represented by only one source

- **Fable-only or materially Fable-led:** deferred import evaluation; arbitrary module-namespace identifier names; `moduleSuffixes`; instantiation expressions and `NoInfer` as first-class obligations; `erasableSyntaxOnly`; Node native type stripping; Bun/Deno variants; class-expression identity; solution configs; and documentation-comment semantics.
- **Codex-only or materially Codex-led:** live binding/CommonJS cycle comparison; detailed evaluation-order fixture; compiler API/custom transformer edges; generated-map provenance; explicit resource suppressed-error interaction; and some language-service/build-mode distinctions.
- **Google-only:** No well-supported unique language family survived as a required item. Google's unique categorical assertions either sharpened failure cases or became TS-GAP-002–004.

### Weak or missing primary evidence

- Package-manager layouts, peer/hoist failure, generated packages, declaration bundling, alternate transpilers/loaders, editor project selection, language-service plugins, and framework JSX surfaces require the official documentation of the exact selected tool.
- Detailed overload ordering, alias preservation, parser recovery, recursion/performance limits, file-precedence rules, and diagnostic stability are best grounded in pinned compiler conformance tests because current TypeScript documentation is not normative at that precision.
- Google included many secondary-source redirects. This checklist relies on the official sources named in the reports and does not promote unsupported secondary claims.

### Likely blind spots for another research pass

- TypeScript 6 migration/deprecations and the release/parity state of TypeScript 7/`tsgo` after 5.9.
- Full Node/package `exports`/`imports` matrix: nested conditions, `types@` selectors, pattern trailers, self-reference, `typesVersions`, and dual-package declaration fallbacks.
- Standard decorators, explicit resource management, import attributes, JSON modules, and deferred imports against current ECMAScript and Node versions.
- TypeScript Server (`tsserver`) protocol, inferred/configured project selection, auto-imports, and plugins; framework-specific JSX/template compilers; Yarn Plug'n'Play and non-Node resolver hosts.
- Compiler conformance-suite cases for syntax recovery, overload ordering, union/intersection normalization, declaration naming, recursion limits, and recent strict iterator/control-flow behavior.

## Primary-source references

[Babel TypeScript]: https://babeljs.io/docs/babel-plugin-transform-typescript
[Bun TypeScript]: https://bun.sh/docs/runtime/typescript
[Deno TypeScript]: https://docs.deno.com/runtime/fundamentals/typescript/
[DefinitelyTyped]: https://github.com/DefinitelyTyped/DefinitelyTyped
[ECMA-262]: https://tc39.es/ecma262/
[Node ESM]: https://nodejs.org/api/esm.html
[Node Modules]: https://nodejs.org/api/modules.html
[Node Packages]: https://nodejs.org/api/packages.html
[Node TypeScript]: https://nodejs.org/api/typescript.html
[Source Map]: https://sourcemaps.info/spec.html
[SWC TypeScript]: https://swc.rs/docs/configuration/compilation#jscparser
[TC39 Decorators]: https://github.com/tc39/proposal-decorators
[TC39 Deferred Import]: https://github.com/tc39/proposal-defer-import-eval
[TC39 Resource Management]: https://github.com/tc39/proposal-explicit-resource-management
[TS 2.3]: https://www.typescriptlang.org/docs/handbook/release-notes/typescript-2-3.html
[TS 2.7]: https://www.typescriptlang.org/docs/handbook/release-notes/typescript-2-7.html
[TS 2.9]: https://www.typescriptlang.org/docs/handbook/release-notes/typescript-2-9.html
[TS 3.4]: https://www.typescriptlang.org/docs/handbook/release-notes/typescript-3-4.html
[TS 3.7]: https://www.typescriptlang.org/docs/handbook/release-notes/typescript-3-7.html
[TS 3.8]: https://www.typescriptlang.org/docs/handbook/release-notes/typescript-3-8.html
[TS 3.9]: https://www.typescriptlang.org/docs/handbook/release-notes/typescript-3-9.html
[TS 4.0]: https://www.typescriptlang.org/docs/handbook/release-notes/typescript-4-0.html
[TS 4.1]: https://www.typescriptlang.org/docs/handbook/release-notes/typescript-4-1.html
[TS 4.2]: https://www.typescriptlang.org/docs/handbook/release-notes/typescript-4-2.html
[TS 4.3]: https://www.typescriptlang.org/docs/handbook/release-notes/typescript-4-3.html
[TS 4.4]: https://www.typescriptlang.org/docs/handbook/release-notes/typescript-4-4.html
[TS 4.5]: https://www.typescriptlang.org/docs/handbook/release-notes/typescript-4-5.html
[TS 4.7]: https://www.typescriptlang.org/docs/handbook/release-notes/typescript-4-7.html
[TS 4.9]: https://www.typescriptlang.org/docs/handbook/release-notes/typescript-4-9.html
[TS 5.0]: https://www.typescriptlang.org/docs/handbook/release-notes/typescript-5-0.html
[TS 5.1]: https://www.typescriptlang.org/docs/handbook/release-notes/typescript-5-1.html
[TS 5.2]: https://www.typescriptlang.org/docs/handbook/release-notes/typescript-5-2.html
[TS 5.3]: https://www.typescriptlang.org/docs/handbook/release-notes/typescript-5-3.html
[TS 5.4]: https://www.typescriptlang.org/docs/handbook/release-notes/typescript-5-4.html
[TS 5.5]: https://www.typescriptlang.org/docs/handbook/release-notes/typescript-5-5.html
[TS 5.6]: https://www.typescriptlang.org/docs/handbook/release-notes/typescript-5-6.html
[TS 5.7]: https://www.typescriptlang.org/docs/handbook/release-notes/typescript-5-7.html
[TS 5.8]: https://www.typescriptlang.org/docs/handbook/release-notes/typescript-5-8.html
[TS 5.9]: https://www.typescriptlang.org/docs/handbook/release-notes/typescript-5-9.html
[TS Classes]: https://www.typescriptlang.org/docs/handbook/2/classes.html
[TS Compatibility]: https://www.typescriptlang.org/docs/handbook/type-compatibility.html
[TS Compiler API]: https://github.com/microsoft/TypeScript/wiki/Using-the-Compiler-API
[TS Conditional Types]: https://www.typescriptlang.org/docs/handbook/2/conditional-types.html
[TS Declaration Files]: https://www.typescriptlang.org/docs/handbook/declaration-files/introduction.html
[TS Declaration Merging]: https://www.typescriptlang.org/docs/handbook/declaration-merging.html
[TS Declaration Publishing]: https://www.typescriptlang.org/docs/handbook/declaration-files/publishing.html
[TS Decorators]: https://www.typescriptlang.org/docs/handbook/decorators.html
[TS Enums]: https://www.typescriptlang.org/docs/handbook/enums.html
[TS Everyday Types]: https://www.typescriptlang.org/docs/handbook/2/everyday-types.html
[TS Functions]: https://www.typescriptlang.org/docs/handbook/2/functions.html
[TS Generics]: https://www.typescriptlang.org/docs/handbook/2/generics.html
[TS Handbook]: https://www.typescriptlang.org/docs/handbook/intro.html
[TS Indexed Access]: https://www.typescriptlang.org/docs/handbook/2/indexed-access-types.html
[TS JavaScript]: https://www.typescriptlang.org/docs/handbook/type-checking-javascript-files.html
[TS JSDoc]: https://www.typescriptlang.org/docs/handbook/jsdoc-supported-types.html
[TS JSX]: https://www.typescriptlang.org/docs/handbook/jsx.html
[TS Mapped Types]: https://www.typescriptlang.org/docs/handbook/2/mapped-types.html
[TS Mixins]: https://www.typescriptlang.org/docs/handbook/mixins.html
[TS Modules]: https://www.typescriptlang.org/docs/handbook/modules/reference.html
[TS Native Port]: https://devblogs.microsoft.com/typescript/typescript-native-port/
[TS Namespaces]: https://www.typescriptlang.org/docs/handbook/namespaces.html
[TS Narrowing]: https://www.typescriptlang.org/docs/handbook/2/narrowing.html
[TS Object Types]: https://www.typescriptlang.org/docs/handbook/2/objects.html
[TS Project References]: https://www.typescriptlang.org/docs/handbook/project-references.html
[TS release notes]: https://www.typescriptlang.org/docs/handbook/release-notes/overview.html
[TS Server]: https://github.com/microsoft/TypeScript/wiki/Writing-a-Language-Service-Plugin
[TS Symbols]: https://www.typescriptlang.org/docs/handbook/symbols.html
[TS Template Types]: https://www.typescriptlang.org/docs/handbook/2/template-literal-types.html
[TS Triple-Slash]: https://www.typescriptlang.org/docs/handbook/triple-slash-directives.html
[TS Type Inference]: https://www.typescriptlang.org/docs/handbook/type-inference.html
[TS Type Operators]: https://www.typescriptlang.org/docs/handbook/2/typeof-types.html
[TS Utility Types]: https://www.typescriptlang.org/docs/handbook/utility-types.html
[TS Variables]: https://www.typescriptlang.org/docs/handbook/variable-declarations.html
[TSConfig]: https://www.typescriptlang.org/tsconfig/
[esbuild TypeScript]: https://esbuild.github.io/content-types/#typescript
[npm Workspaces]: https://docs.npmjs.com/cli/using-npm/workspaces/
