# TypeScript Code-Graph Coverage Research Checklist

## Scope and assumptions

This report identifies candidate coverage for a broad, idiomatic, self-contained TypeScript project. It concentrates on semantic entities, identity, relationships, name and module resolution, types, visibility, dispatch, source inclusion, emit behavior, and diagnostics. It does not prescribe a graph schema, fixture layout, scoring model, or analysis implementation.

Baseline assumptions:

- The reference implementation is Microsoft's `typescript` package and its `tsc` compiler. Compiler version must be pinned because minor TypeScript releases can change inference, diagnostics, library declarations, and module resolution.
- The confidently covered baseline is TypeScript 5.7-era behavior, with explicit checks for material changes from TypeScript 4.7 through 5.7. Any later compiler should be verified against its release notes.
- ECMAScript semantics come from the ECMAScript Language Specification. TypeScript adds static types, declaration forms, configuration, resolution, and downlevel emit, but it intentionally preserves JavaScript runtime behavior.
- Node.js, browser, bundler, and declaration-only consumers are distinct host conditions. A construct can resolve or run differently in each.
- Strict checking is treated as the semantic baseline, but individual strictness flags and permissive configurations remain coverage targets because they change types and diagnostics.
- Package metadata, dependency declarations, compiler configuration, and generated declaration files are program inputs, not incidental build files.

Authority labels used below:

- **Language**: defined by TypeScript or ECMAScript language rules.
- **Official implementation**: behavior of `tsc`, its resolver, standard declaration libraries, or official language service.
- **Host/ecosystem**: behavior supplied by Node.js, browsers, package managers, bundlers, frameworks, or common package conventions.

## Program construction, configuration, and source identity

### TS-001 — Root-file discovery and configuration boundaries

- **Distinct behavior:** Explicit `files`, `include`, `exclude`, command-line roots, default exclusions, and imported files determine the program's source set. `exclude` filters discovery but does not prevent an excluded file from entering through an import, reference, `types`, or library dependency.
- **Observable content:** Multiple source and declaration files, some discovered by globs, some explicit, some excluded but imported, and one genuinely unreachable file.
- **Variants and failures:** Empty or misspelled globs, duplicate paths, generated output re-entering the input set, `allowJs`, and configuration placed in a parent directory. Expect “no inputs” and overwrite/input collision diagnostics where applicable.
- **Constraints / authority:** Official implementation; configuration and filesystem dependent. Case sensitivity follows host filesystem and compiler canonicalization.
- **Confidence:** High.
- **Likely verification source:** TSConfig Reference and TypeScript compiler source.

### TS-002 — Configuration inheritance and effective options

- **Distinct behavior:** `extends` forms a configuration chain whose relative paths are resolved from the file that declares them. Most fields override rather than deep-merge, while root-file selection has special behavior.
- **Observable content:** A base configuration, a derived configuration that replaces selected arrays or objects, and command-line overrides that produce a measurably different effective program.
- **Variants and failures:** Package-based `extends`, chained bases, missing bases, cycles, and conflicting `include`, `files`, `paths`, `typeRoots`, or output settings.
- **Constraints / authority:** Official implementation; package resolution participates for package-based bases.
- **Confidence:** High.
- **Likely verification source:** TSConfig Reference and TypeScript release notes.

### TS-003 — Project references and composite project identity

- **Distinct behavior:** Referenced projects establish build order and declaration-file boundaries. Consumers normally see the referenced project's public declarations rather than freely merging all implementation sources into one program.
- **Observable content:** At least two composite projects with a reference edge, an exported API crossing the edge, a non-exported implementation symbol, and a change that invalidates the dependent project.
- **Variants and failures:** Missing `composite`, stale or absent declaration outputs, `disableSourceOfProjectReferenceRedirect`, circular references, prepend-era configurations, and solution configurations with no roots.
- **Constraints / authority:** Official implementation and build mode; editor redirection can differ from command-line consumption.
- **Confidence:** High.
- **Likely verification source:** TypeScript Project References documentation and TSConfig Reference.

### TS-004 — Incremental and build-mode state

- **Distinct behavior:** `incremental`, `.tsbuildinfo`, `assumeChangesOnlyAffectDirectDependencies`, and `tsc --build` change invalidation and diagnostic scheduling without changing intended language meaning.
- **Observable content:** A dependency chain where an implementation-only edit, a public declaration edit, and an error removal produce different rebuild scopes.
- **Variants and failures:** Corrupt or stale build information, changed compiler version/options, clean builds, declaration-only builds, and failed upstream projects.
- **Constraints / authority:** Official implementation; `.tsbuildinfo` is implementation data and not a portable semantic contract.
- **Confidence:** High.
- **Likely verification source:** TSConfig Reference and TypeScript build-mode documentation.

### TS-005 — ECMAScript target and downlevel transformation

- **Distinct behavior:** `target` controls syntax transformation, default libraries, class semantics in combination with other flags, and helper generation. Static acceptance does not imply that the runtime provides required built-ins.
- **Observable content:** Modern class features, async functions, iteration, optional chaining, spread, and newer built-ins compiled under both an older and a modern target.
- **Variants and failures:** Native-preserving versus transformed emit, `downlevelIteration`, helper imports, BigInt below ES2020, and missing runtime polyfills despite successful type checking.
- **Constraints / authority:** Official implementation for emit; ECMAScript and host for runtime behavior.
- **Confidence:** High.
- **Likely verification source:** TSConfig Reference, TypeScript Handbook, and ECMAScript Language Specification.

### TS-006 — Standard library selection and ambient host surface

- **Distinct behavior:** `lib`, `noLib`, target defaults, and triple-slash library references select ambient globals and built-in types. DOM, WebWorker, iterable, and ECMAScript library sets can overlap or conflict.
- **Observable content:** References to a browser global, a worker global, and an ECMAScript-version-specific built-in under configurations with different library sets.
- **Variants and failures:** Missing globals, duplicate declarations from incompatible libraries, dependency-provided lib replacements, `skipDefaultLibCheck`, and a source file carrying a library reference.
- **Constraints / authority:** Official implementation and bundled declaration files; actual runtime availability remains host-defined.
- **Confidence:** High.
- **Likely verification source:** TSConfig Reference and TypeScript bundled `lib.*.d.ts` files.

### TS-007 — Strictness family as independent semantic switches

- **Distinct behavior:** `strict` is a moving umbrella over options such as strict null checks, function types, property initialization, bind/call/apply, implicit `any`, implicit `this`, and catch-variable typing. Each changes inferred types or diagnostics.
- **Observable content:** Small declarations whose acceptance or type changes under each strictness member, plus a configuration that overrides one member after enabling `strict`.
- **Variants and failures:** Version-dependent additions to the umbrella, explicit child overrides, declaration-file checking, and code whose emitted JavaScript is unchanged despite errors.
- **Constraints / authority:** Official implementation.
- **Confidence:** High.
- **Likely verification source:** TSConfig Reference and TypeScript release notes.

### TS-008 — Additional soundness and hygiene flags

- **Distinct behavior:** `exactOptionalPropertyTypes`, `noUncheckedIndexedAccess`, `noPropertyAccessFromIndexSignature`, `useUnknownInCatchVariables`, `noImplicitOverride`, unused checks, fallthrough checks, and return checks expose different relations and diagnostics.
- **Observable content:** Optional properties, index signatures, catch variables, overridden members, unused locals/parameters, switch fallthrough, and a function with a missing return.
- **Variants and failures:** Flags depend on strict-null behavior in some cases; unused checks are compiler diagnostics rather than semantic reachability; declaration files may be treated differently.
- **Constraints / authority:** Official implementation; availability is compiler-version dependent.
- **Confidence:** High.
- **Likely verification source:** TSConfig Reference and TypeScript release notes.

### TS-009 — Module emit mode

- **Distinct behavior:** `module` selects preservation or transformation of import/export syntax and can affect acceptance of top-level `await`, `import.meta`, and module-specific constructs. Node-oriented modes determine format per file rather than uniformly.
- **Observable content:** Equivalent modules compiled with preserved ECMAScript modules, CommonJS output, and a Node-oriented mode, including top-level asynchronous code and metadata access.
- **Variants and failures:** `preserve`, `esnext`, CommonJS, System, older AMD/UMD modes, Node16/NodeNext, and module/target combinations that reject syntax.
- **Constraints / authority:** Official implementation for emit; loader behavior is host/ecosystem.
- **Confidence:** High.
- **Likely verification source:** TSConfig Reference and TypeScript release notes.

### TS-010 — Module resolution strategy

- **Distinct behavior:** `moduleResolution` selects distinct algorithms for extensions, directory indexes, package metadata, conditions, and relative-specifier requirements. Resolution can differ even when emitted text is identical.
- **Observable content:** The same import graph checked under Classic, Node10, Node16/NodeNext, and Bundler resolution where meaningful, with a dependency exposing conditional entry points.
- **Variants and failures:** Extensionless relative imports, directory imports, blocked deep imports, package self-references, `resolvePackageJsonExports`, `resolvePackageJsonImports`, and custom conditions.
- **Constraints / authority:** Official implementation models host behavior; exact runtime success is host/ecosystem-defined.
- **Confidence:** High.
- **Likely verification source:** TypeScript Module Resolution documentation, TSConfig Reference, and Node.js package documentation.

### TS-011 — Source extension and implied module format

- **Distinct behavior:** `.ts`, `.tsx`, `.mts`, `.cts`, and their declaration counterparts carry different parsing or module-format implications. JavaScript variants enter when `allowJs` is enabled.
- **Observable content:** Matched TypeScript and declaration files using ordinary, explicit ECMAScript-module, and explicit CommonJS extensions, plus an extension-sensitive import.
- **Variants and failures:** `.d.ts`, `.d.mts`, `.d.cts`, `.js`, `.jsx`, `.mjs`, `.cjs`; wrong declaration format for a package condition; JSX in a non-TSX file; extension rewriting in emit.
- **Constraints / authority:** Language parsing plus official implementation and Node-compatible resolution.
- **Confidence:** High.
- **Likely verification source:** TypeScript Module documentation and TypeScript 4.7 release notes.

### TS-012 — Package metadata and conditional entry points

- **Distinct behavior:** `package.json` `type`, `main`, `module`, `types`/`typings`, `exports`, `imports`, and condition maps influence file format and visible entry points. Package metadata can make a physically present file unreachable.
- **Observable content:** A package with public and private subpaths, separate import/require and type branches, and a self-reference.
- **Variants and failures:** Missing `types`, a types branch whose format disagrees with runtime branch, nested conditions, `types@` conditions, wildcard subpaths, null-blocked paths, and legacy fallback when `exports` is absent.
- **Constraints / authority:** Node.js and package ecosystem, modeled by official implementation; bundlers may choose different condition sets.
- **Confidence:** High.
- **Likely verification source:** Node.js packages documentation, TypeScript Module Resolution documentation, and TypeScript release notes.

### TS-013 — Package version-directed declarations

- **Distinct behavior:** `typesVersions` can expose different declaration trees to different TypeScript compiler ranges. Package identity and resolved symbol surfaces therefore depend on compiler version.
- **Observable content:** A dependency with a default declaration entry and at least one compiler-range mapping that changes an exported type.
- **Variants and failures:** Overlapping ranges, mapping fallthrough, interaction with `exports`, unsupported syntax in the fallback declarations, and different lockfile-selected package versions.
- **Constraints / authority:** Official implementation plus package convention and semantic-version range matching.
- **Confidence:** Medium.
- **Likely verification source:** TypeScript declaration publishing documentation and TypeScript compiler source.

### TS-014 — Path remapping, base paths, and virtual directory merging

- **Distinct behavior:** `paths`, `baseUrl`, and `rootDirs` change compile-time lookup and can overlay separate physical directories into one virtual relative structure. They do not inherently rewrite emitted specifiers or configure the runtime.
- **Observable content:** A non-relative alias with fallback targets, and two physical roots whose files import each other through an apparently relative path.
- **Variants and failures:** Wildcard specificity, multiple fallback entries, aliases shadowing packages, a compile-success/runtime-failure mismatch, and deprecated or version-shifted precedence around `baseUrl`.
- **Constraints / authority:** Official implementation; runtime mapping is host/ecosystem-specific.
- **Confidence:** High.
- **Likely verification source:** TSConfig Reference and TypeScript Module Resolution documentation.

### TS-015 — Filesystem canonicalization, casing, and symlinks

- **Distinct behavior:** File identity can differ by host case sensitivity, path spelling, realpath handling, and package-manager symlinks. Duplicate logical modules can create incompatible copies of nominally sensitive types.
- **Observable content:** Imports that differ only in case, a symlinked dependency, and two paths that can resolve to the same or separate physical declaration file.
- **Variants and failures:** `forceConsistentCasingInFileNames`, `preserveSymlinks`, monorepo links, pnpm-style stores, duplicate package versions, and case-only renames.
- **Constraints / authority:** Official implementation plus operating system, filesystem, Node.js, and package manager.
- **Confidence:** High.
- **Likely verification source:** TSConfig Reference, Node.js module documentation, and package-manager documentation.

### TS-016 — JavaScript inputs and JSDoc typing

- **Distinct behavior:** `allowJs` admits JavaScript into the program; `checkJs`, per-file directives, and JSDoc supply or suppress static checking. JavaScript inference and declaration emit differ from native TypeScript annotation syntax.
- **Observable content:** Checked and unchecked JavaScript files with JSDoc typedefs, templates, imports, property declarations, overloads, and TypeScript consumers.
- **Variants and failures:** `// @ts-check`, `// @ts-nocheck`, CommonJS export inference, unsupported JSDoc forms, `.jsx`, declaration emit from JavaScript, and `maxNodeModuleJsDepth`.
- **Constraints / authority:** Official implementation and documented JSDoc support.
- **Confidence:** High.
- **Likely verification source:** TypeScript JavaScript and JSDoc documentation and TSConfig Reference.

### TS-017 — Declaration inputs, emit, and public-surface reachability

- **Distinct behavior:** Declaration files contribute types without ordinary runtime code. `declaration`, `emitDeclarationOnly`, `stripInternal`, and declaration bundling by ecosystem tools expose a computed public type surface.
- **Observable content:** Implementation exports that mention private helpers, ambient declaration inputs, emitted declarations, an `@internal` declaration, and a consumer that only sees output declarations.
- **Variants and failures:** Inferred non-portable names, private names in exported APIs, declaration-map navigation, CommonJS/ECMAScript declaration formats, and `skipLibCheck` hiding dependency declaration errors.
- **Constraints / authority:** Official implementation; bundled declarations are ecosystem tooling.
- **Confidence:** High.
- **Likely verification source:** TypeScript Declaration Files documentation and TSConfig Reference.

### TS-018 — Emit eligibility and diagnostic policy

- **Distinct behavior:** Type errors do not inherently prevent JavaScript emit. `noEmit`, `noEmitOnError`, declaration-only modes, build mode, and recoverable syntax or semantic errors change which artifacts appear.
- **Observable content:** A type-invalid but syntactically emit-capable source compiled under contrasting emit policies, plus a declaration-emit failure.
- **Variants and failures:** `noCheck` in versions that support it, `skipLibCheck`, pretty versus machine diagnostics, project-reference continuation behavior, and output collision diagnostics.
- **Constraints / authority:** Official implementation; option availability and build-mode defaults are version dependent.
- **Confidence:** High.
- **Likely verification source:** TSConfig Reference and TypeScript release notes.

### TS-019 — Single-file transpilation constraints

- **Distinct behavior:** `isolatedModules` and `isolatedDeclarations` reject constructs that cannot be safely transformed or declared without whole-program knowledge. A program may be valid under `tsc` whole-program analysis but invalid for per-file tools.
- **Observable content:** Cross-file type references, type-only exports, const enums, namespaces in script files, and exported declarations whose types need inference.
- **Variants and failures:** `transpileModule`, Babel, SWC, declaration emit, explicit annotations that repair isolated-declaration errors, and version-dependent diagnostic coverage.
- **Constraints / authority:** Official implementation defines flags; third-party transpilers motivate and may extend constraints.
- **Confidence:** High.
- **Likely verification source:** TSConfig Reference and TypeScript 5.5 release notes.

### TS-020 — Generated sources, maps, and source provenance

- **Distinct behavior:** Generated `.ts`, generated `.d.ts`, JavaScript source maps, and declaration maps create distinct authored, generated, emitted, and navigation identities. Graph consumers must not assume one source file per output file.
- **Observable content:** A generated input marked by convention, declaration and JavaScript maps, inline and external sources, and an output whose map points outside the output directory.
- **Variants and failures:** Missing sources, embedded `sourcesContent`, path remapping, generated files excluded but imported, stale generated declarations, and source-map comments disabled.
- **Constraints / authority:** Official implementation for its map format; generator provenance and consumption are ecosystem conventions.
- **Confidence:** Medium.
- **Likely verification source:** TSConfig Reference and source map specification.

## Modules, namespaces, and cross-file binding

### TS-021 — Script files, external modules, and forced module detection

- **Distinct behavior:** A file without module indicators can contribute declarations to the shared global scope. Imports, exports, selected syntax, and `moduleDetection` can instead give it a private module scope.
- **Observable content:** Two script files sharing a global, a file with an otherwise empty export, and the same source under `auto`, `legacy`, and `force` detection.
- **Variants and failures:** JSX files under framework modes, top-level `await`, global name collisions, accidental script status, and CommonJS indicators recognized through JavaScript analysis.
- **Constraints / authority:** Language and official implementation; detection has changed across compiler versions.
- **Confidence:** High.
- **Likely verification source:** TSConfig Reference and TypeScript 4.7 release notes.

### TS-022 — Named, default, aliased, and namespace imports

- **Distinct behavior:** Import bindings are local aliases to exported symbols; default and namespace imports have distinct binding and interop rules. Type and value meanings may coexist behind one spelling.
- **Observable content:** Named imports with renaming, a default import, a namespace import, local shadowing, and references through both the alias and origin.
- **Variants and failures:** Missing exports, duplicate local names, reserved names, imported type used as a value, and namespace objects with statically known properties.
- **Constraints / authority:** ECMAScript plus TypeScript's type/value namespaces and resolver.
- **Confidence:** High.
- **Likely verification source:** TypeScript Handbook Modules documentation and ECMAScript Language Specification.

### TS-023 — Type-only imports and exports

- **Distinct behavior:** `import type`, inline type modifiers, and `export type` establish compile-time-only edges and prevent value use. Their emit and legality depend on module-preservation options.
- **Observable content:** The same declaration imported as a type-only binding and as a value-capable binding, plus type-only re-export and an intentional value-use error.
- **Variants and failures:** Classes and enums that have both type and value meanings, default type-only imports, import attributes restrictions, isolated modules, and `verbatimModuleSyntax`.
- **Constraints / authority:** Language and official implementation; syntax support is version dependent.
- **Confidence:** High.
- **Likely verification source:** TypeScript release notes and Handbook Modules documentation.

### TS-024 — Re-exports, export stars, and ambiguity

- **Distinct behavior:** Direct re-exports, renamed re-exports, namespace re-exports, and star exports create forwarding edges without a local binding. Conflicting star exports can make a name ambiguous or absent.
- **Observable content:** A barrel that uses each re-export form, two star sources exporting the same name, an explicit export that resolves the conflict, and a default export that is not forwarded by a star.
- **Variants and failures:** Type-only re-exports, cyclic barrels, duplicate-but-identical origins, namespace re-exports, and CommonJS sources.
- **Constraints / authority:** ECMAScript plus TypeScript type-space rules.
- **Confidence:** High.
- **Likely verification source:** ECMAScript Language Specification and TypeScript Modules documentation.

### TS-025 — Live bindings and module cycles

- **Distinct behavior:** ECMAScript imports observe live exported bindings, while initialization order in cycles can expose temporal dead zones or partially initialized CommonJS objects. Static dependency edges do not imply safe evaluation order.
- **Observable content:** A cycle containing mutable exports and top-level reads, contrasted with a CommonJS cycle.
- **Variants and failures:** Type-only cycles, function-hoisting-safe cycles, top-level-await cycles, bundler rewrites, and cycles passing only through barrel modules.
- **Constraints / authority:** ECMAScript or Node.js runtime; TypeScript mostly preserves or downlevels the structure.
- **Confidence:** High.
- **Likely verification source:** ECMAScript Language Specification and Node.js modules documentation.

### TS-026 — Import elision and verbatim module syntax

- **Distinct behavior:** The compiler decides whether syntactic imports survive emit based on use, declaration kind, and options. `verbatimModuleSyntax` makes explicit type-only syntax control erasure and rejects incompatible module syntax rather than rewriting it.
- **Observable content:** Imports used only in annotations, imports of declarations with runtime values, side-effectful modules, and contrasting emit under legacy elision and verbatim syntax.
- **Variants and failures:** Deprecated `importsNotUsedAsValues`/`preserveValueImports`, CommonJS targets, decorator metadata references, isolated modules, and accidental loss or retention of side effects.
- **Constraints / authority:** Official implementation; option set changed materially in TypeScript 5.0.
- **Confidence:** High.
- **Likely verification source:** TypeScript 5.0 release notes and TSConfig Reference.

### TS-027 — Side-effect imports and unresolved-module diagnostics

- **Distinct behavior:** A bare import creates an evaluation edge without bindings. Historically some unresolved bare imports escaped diagnostics; `noUncheckedSideEffectImports` tightens checking and often requires wildcard ambient declarations for assets.
- **Observable content:** A resolvable side-effect module, a misspelled one, and a non-code asset admitted by an ambient wildcard declaration.
- **Variants and failures:** Bundler loaders, CSS or image imports, arbitrary extensions, package side-effects metadata, and tree-shaking that is outside TypeScript's semantic guarantee.
- **Constraints / authority:** Official implementation for checking; asset handling and tree-shaking are ecosystem-defined. Flag support is version dependent.
- **Confidence:** High.
- **Likely verification source:** TypeScript 5.6 release notes and TSConfig Reference.

### TS-028 — Dynamic import, import types, and import metadata

- **Distinct behavior:** Dynamic `import()` is a runtime asynchronous edge; `import()` in type position is compile-time-only; `typeof import()` queries a module value shape. `import.meta` depends on module mode and host.
- **Observable content:** All three import forms against one module, a computed dynamic specifier, and host-supported metadata fields.
- **Variants and failures:** Non-literal dynamic imports, CommonJS transformation, top-level await, custom `ImportMeta` augmentation, and resolution attributes/options on dynamic imports.
- **Constraints / authority:** ECMAScript, TypeScript language, official emit, and host-defined metadata.
- **Confidence:** High.
- **Likely verification source:** ECMAScript Language Specification and TypeScript Modules documentation.

### TS-029 — Import attributes, JSON, and non-code modules

- **Distinct behavior:** Import attributes are preserved for the host and can participate in resolution mode selection. `resolveJsonModule` synthesizes types from JSON values, while other assets generally require ambient declarations or tooling.
- **Observable content:** A JSON import with an attribute where supported, a typed property access, an invalid property access, and a custom-extension module declaration.
- **Variants and failures:** Earlier import-assertion syntax, default-import interop, Node versus bundler requirements, mutable versus readonly expectations, arbitrary extension support, and runtime loader absence.
- **Constraints / authority:** ECMAScript/host for attributes, official implementation for JSON typing, ecosystem for other loaders.
- **Confidence:** Medium.
- **Likely verification source:** TypeScript release notes, TSConfig Reference, and host module documentation.

### TS-030 — CommonJS assignment exports and TypeScript import-equals

- **Distinct behavior:** `export =` models a single CommonJS-style exported entity and pairs with `import = require()`. The forms do not compose freely with ordinary ECMAScript exports.
- **Observable content:** A declaration module using `export =`, a consumer using import-equals, property merging on the exported entity, and an invalid mixed export form.
- **Variants and failures:** `export as namespace`, JavaScript `module.exports`, synthetic default consumers, Node-format `.cts`/`.d.cts`, and restrictions under verbatim syntax.
- **Constraints / authority:** TypeScript language and official CommonJS modeling.
- **Confidence:** High.
- **Likely verification source:** TypeScript Declaration Files and Modules documentation.

### TS-031 — ECMAScript/CommonJS interop flags

- **Distinct behavior:** `esModuleInterop` changes helper emit and checking for namespace/default imports; `allowSyntheticDefaultImports` changes checking only. Neither can make every mismatched runtime module shape correct.
- **Observable content:** CommonJS dependencies shaped as a callable export and as an object export, consumed by namespace, default, and import-equals forms under contrasting flags.
- **Variants and failures:** Transpiler-added `__esModule`, Node native interoperability, Babel behavior, inherited versus own properties, and declaration files that inaccurately model runtime exports.
- **Constraints / authority:** Official implementation models ecosystem conventions; runtime details are host/tool-specific.
- **Confidence:** High.
- **Likely verification source:** TSConfig Reference and TypeScript Module documentation.

### TS-032 — Internal namespaces and qualified binding

- **Distinct behavior:** `namespace` creates a value object and a namespace for qualified names. Multiple declarations can merge, but non-exported members remain local to their original declaration block.
- **Observable content:** A namespace split across files or blocks with exported and non-exported members, nested namespaces, and qualified references from type and value positions.
- **Variants and failures:** Script versus module context, namespace emit ordering, ambient namespaces, aliases through import-equals, and isolated-module restrictions.
- **Constraints / authority:** TypeScript language and official emit.
- **Confidence:** High.
- **Likely verification source:** TypeScript Handbook Namespaces and Declaration Merging documentation.

### TS-033 — Ambient external and wildcard modules

- **Distinct behavior:** Ambient module declarations define types for modules with no TypeScript implementation and can be exact-name or wildcard patterns. Their interpretation differs when nested in an external module versus a script declaration file.
- **Observable content:** Exact and wildcard module declarations, imports matching each, an unmatched import, and a declaration whose top-level module status changes whether it is an augmentation.
- **Variants and failures:** Shorthand untyped modules, relative ambient module names, asset modules, default versus export-assignment shapes, and competing declarations from dependencies.
- **Constraints / authority:** TypeScript declaration language; runtime implementation is host/ecosystem.
- **Confidence:** High.
- **Likely verification source:** TypeScript Declaration Files documentation.

### TS-034 — Module and global augmentation

- **Distinct behavior:** Augmentations merge declarations into an existing resolved module or global scope; they cannot freely add a new default export and require the original declaration to be in the program. Runtime patching is a separate obligation.
- **Observable content:** A module augmentation for an interface-backed instance member, corresponding runtime assignment, a `declare global` block, and a consumer before and after importing the patch module.
- **Variants and failures:** Augmenting the wrong resolved package copy, type-only import that erases required side effects, default-export limitation, global collision, and augmentation inside `.d.ts` versus `.ts`.
- **Constraints / authority:** TypeScript language for type merging; JavaScript runtime for patch installation.
- **Confidence:** High.
- **Likely verification source:** TypeScript Declaration Merging documentation.

### TS-035 — UMD global/module duality

- **Distinct behavior:** `export as namespace` lets a declaration describe a Universal Module Definition package usable as a module and as a global in script contexts, with diagnostics for inappropriate global use inside modules.
- **Observable content:** A UMD-shaped declaration, one module consumer, one script consumer, and a module attempting to use the global spelling.
- **Variants and failures:** `allowUmdGlobalAccess`, `export =` pairing, bundler-provided globals, and name conflicts with other ambient declarations.
- **Constraints / authority:** TypeScript declaration language models an ecosystem packaging convention.
- **Confidence:** High.
- **Likely verification source:** TypeScript Declaration Files documentation and TSConfig Reference.

### TS-036 — Triple-slash directives and legacy dependency edges

- **Distinct behavior:** Path, types, lib, and no-default-lib directives affect preprocessing, ambient inclusion, and output ordering in legacy modes. They are distinct from ECMAScript imports.
- **Observable content:** A declaration file with `types` and `lib` references, a source path reference, and a directive placed incorrectly or targeting a missing file/package.
- **Variants and failures:** `preserve="true"`, `outFile` ordering, `noResolve`, directives after statements, and package declaration entry points that rely on referenced fragments.
- **Constraints / authority:** TypeScript language/official implementation; some uses are legacy but supported.
- **Confidence:** High.
- **Likely verification source:** TypeScript Triple-Slash Directives documentation.

## Declarations, identity, visibility, and object model

### TS-037 — Separate type, value, and namespace declaration spaces

- **Distinct behavior:** One spelling can denote different entities in type, value, and namespace positions. A class or enum contributes both a type and value; an interface only a type; a namespace contributes namespace and usually value meanings.
- **Observable content:** Same-named compatible declarations referenced from all applicable positions, plus an illegal collision and a type-only declaration used as a runtime value.
- **Variants and failures:** Imports alias multiple meanings, `typeof` bridges value to type, declaration merging, and erased output.
- **Constraints / authority:** TypeScript language.
- **Confidence:** High.
- **Likely verification source:** TypeScript Handbook and Declaration Merging documentation.

### TS-038 — Lexical scope, hoisting, and temporal dead zone

- **Distinct behavior:** `var`, `let`, `const`, function, class, parameter, catch, and block declarations have different scopes, hoisting, redeclaration rules, and runtime initialization behavior.
- **Observable content:** Nested blocks and functions with shadowing, legal `var` redeclaration, an illegal lexical redeclaration, closure capture, and a pre-initialization reference.
- **Variants and failures:** Loop-variable capture when downleveled, switch-case shared scope, global `var` versus global lexical bindings, strict mode, and use-before-declaration diagnostics versus runtime failure.
- **Constraints / authority:** ECMAScript runtime and TypeScript diagnostics/emit.
- **Confidence:** High.
- **Likely verification source:** ECMAScript Language Specification and TypeScript Handbook.

### TS-039 — Function overload declarations and implementation identity

- **Distinct behavior:** Multiple overload signatures describe one callable implementation. The implementation signature is not directly visible to callers, and overload order affects resolution.
- **Observable content:** A function with multiple overloads and one broader implementation, successful calls selecting different signatures, a call accepted only by the implementation signature, and incompatible overload/implementation diagnostics.
- **Variants and failures:** Ambient overloads without implementation, class/interface methods, constructor overloads, optional/rest alternatives, and union arguments that match no single overload.
- **Constraints / authority:** TypeScript language and official checker.
- **Confidence:** High.
- **Likely verification source:** TypeScript Handbook Functions documentation.

### TS-040 — Function forms, lexical `this`, and explicit `this` parameters

- **Distinct behavior:** Arrow functions capture lexical `this`; ordinary functions and methods receive it from the call site. A fake first `this` parameter constrains calls but is erased, and contextual `ThisType` can supply object-literal `this`.
- **Observable content:** Method extraction, arrow versus ordinary callbacks, explicit `this` parameter, `this: void`, and object-literal contextual typing.
- **Variants and failures:** `noImplicitThis`, `.call`/`.bind` under strict checking, constructor functions in JavaScript, static versus instance `this`, and callback variance.
- **Constraints / authority:** ECMAScript runtime plus TypeScript type system.
- **Confidence:** High.
- **Likely verification source:** TypeScript Handbook Functions documentation and Utility Types documentation.

### TS-041 — Class inheritance, abstractness, and override relations

- **Distinct behavior:** Classes combine nominal declaration identity with mostly structural instance types. `extends`, `implements`, abstract members, `override`, and `super` create distinct inheritance, conformance, and dispatch relations.
- **Observable content:** Abstract base and concrete derived classes, interface implementation, overridden method, inherited overloads, invalid missing member, and polymorphic base-typed call.
- **Variants and failures:** `noImplicitOverride`, constructor/static-side compatibility, field hiding versus method override, abstract constructor types, and private/protected members affecting assignability.
- **Constraints / authority:** ECMAScript for runtime classes; TypeScript for abstractness and static relations.
- **Confidence:** High.
- **Likely verification source:** TypeScript Handbook Classes documentation and TSConfig Reference.

### TS-042 — Class field initialization and define/assign semantics

- **Distinct behavior:** Field initializers run in a defined order around `super`; `useDefineForClassFields` controls define-style versus assignment-style downlevel emit and can change interaction with inherited setters.
- **Observable content:** Base and derived fields, an inherited setter with a same-named derived field, uninitialized strict property, definite-assignment assertion, and parameter use in initialization.
- **Variants and failures:** Target-dependent defaults, `declare` fields, accessor overriding, initialization before derived constructor body, and reads before initialization.
- **Constraints / authority:** ECMAScript class fields plus official downlevel implementation; defaults changed across target/compiler versions.
- **Confidence:** High.
- **Likely verification source:** TSConfig Reference, TypeScript release notes, and ECMAScript Language Specification.

### TS-043 — Constructor parameter properties

- **Distinct behavior:** Accessibility or `readonly` modifiers on constructor parameters create both a parameter and an instance property from one declaration, with initialization injected into construction.
- **Observable content:** Public, protected/private, and readonly parameter properties; references as parameters in the constructor and as fields outside it; inheritance visibility checks.
- **Variants and failures:** Destructured parameters are not parameter properties, interaction with explicit fields, decorators, derived constructors, declaration emit, and `useDefineForClassFields`.
- **Constraints / authority:** TypeScript language and official emit.
- **Confidence:** High.
- **Likely verification source:** TypeScript Handbook Classes documentation.

### TS-044 — Accessors and auto-accessors

- **Distinct behavior:** Getters and setters form one property-like symbol with read/write types and runtime accessor dispatch. Auto-accessors create hidden storage and accessor behavior, notably for decorator integration.
- **Observable content:** Paired and unpaired accessors, differing getter/setter types where supported, inherited accessor override, and an auto-accessor.
- **Variants and failures:** Readonly inference from getter-only members, target restrictions for older accessors, property/accessor override diagnostics, static accessors, and standard versus legacy decorators.
- **Constraints / authority:** ECMAScript for accessors/auto-accessors; TypeScript checker and emit. Auto-accessor support is version dependent.
- **Confidence:** High.
- **Likely verification source:** TypeScript Handbook Classes documentation and TypeScript 4.9 release notes.

### TS-045 — Public, protected, TypeScript-private, and ECMAScript private names

- **Distinct behavior:** Public members are structurally compared. TypeScript `private` and `protected` members add declaration-origin-sensitive compatibility. ECMAScript `#private` names have hard runtime privacy and unique lexical identity.
- **Observable content:** Classes with each visibility, same-shaped unrelated classes, subclasses accessing protected state, bracket access attempts, and runtime `#private` access failure.
- **Variants and failures:** Parameter properties, private constructors, static private fields, soft privacy through emitted JavaScript, duplicate package copies, and declaration emit.
- **Constraints / authority:** TypeScript for accessibility/compatibility; ECMAScript for `#private` runtime semantics.
- **Confidence:** High.
- **Likely verification source:** TypeScript Handbook Classes documentation and ECMAScript Language Specification.

### TS-046 — Static members, static blocks, and class static side

- **Distinct behavior:** A class declaration creates an instance type and a constructor/static value with a separate type. Static blocks execute in source order and can access private state.
- **Observable content:** Static fields, methods, private state, multiple static blocks, inheritance of static members, and a `typeof` query of the class value.
- **Variants and failures:** Generic class type parameters unavailable to static members, `this` on the static side, static name restrictions, downlevel ordering, and static override compatibility.
- **Constraints / authority:** ECMAScript plus TypeScript static-side checking and emit.
- **Confidence:** High.
- **Likely verification source:** TypeScript Handbook Classes documentation and ECMAScript Language Specification.

### TS-047 — Interface extension and declaration merging

- **Distinct behavior:** Interfaces can extend multiple bases and reopen by name. Compatible members merge; incompatible properties or overload groups produce errors, and later overload groups can receive precedence.
- **Observable content:** Split interface declarations, multiple inheritance, merged method overloads, conflicting property types, and a class implementing the result.
- **Variants and failures:** Generic parameter mismatch, specialized string-literal overload ordering, global augmentation, module-scoped interfaces that do not merge, and duplicate dependency declarations.
- **Constraints / authority:** TypeScript language.
- **Confidence:** High.
- **Likely verification source:** TypeScript Declaration Merging and Interfaces documentation.

### TS-048 — Cross-kind declaration merging

- **Distinct behavior:** Supported combinations such as namespace with class, function, or enum create a shared public name with combined meanings. Many other same-name combinations, especially type aliases, cannot merge.
- **Observable content:** Each supported cross-kind merge with references to both portions, ordering-sensitive runtime namespace initialization, and at least one forbidden merge.
- **Variants and failures:** Exported versus non-exported namespace members, default exports that cannot be augmented by name, module boundaries, ambient declarations, and isolated modules.
- **Constraints / authority:** TypeScript language and official emit.
- **Confidence:** High.
- **Likely verification source:** TypeScript Declaration Merging documentation.

### TS-049 — Enums as type, value, and runtime object

- **Distinct behavior:** Numeric, string, heterogeneous, computed, and constant enum members differ in inference and runtime representation. Numeric enums generally receive reverse mappings; string enums do not.
- **Observable content:** Enum declarations covering constant and computed members, member-type narrowing, runtime property access, reverse lookup, namespace merging, and invalid initializer ordering.
- **Variants and failures:** Ambient enums, union-enum analysis, bit flags, duplicate values, assignability changes across compiler versions, and module re-exports.
- **Constraints / authority:** TypeScript language and official emit.
- **Confidence:** High.
- **Likely verification source:** TypeScript Handbook Enums documentation and release notes.

### TS-050 — Const enums and preservation boundaries

- **Distinct behavior:** `const enum` references may be inlined and the runtime object omitted. `preserveConstEnums`, ambient consumption, isolated transforms, and version skew can change or break that assumption.
- **Observable content:** A const enum used locally and across a declaration boundary, emitted output with and without preservation, and a per-file transform diagnostic.
- **Variants and failures:** Ambient const enums under isolated modules, stale inlined values from a dependency, imports retained only for enum access, and deconstification during declaration publishing.
- **Constraints / authority:** TypeScript language/official implementation; publishing practices are ecosystem convention.
- **Confidence:** High.
- **Likely verification source:** TypeScript Handbook Enums documentation and TSConfig Reference.

### TS-051 — Symbol-keyed and computed declarations

- **Distinct behavior:** Computed property names can resolve to literal string/number keys or unique symbols. `unique symbol` types preserve nominal key identity and require restricted declaration forms.
- **Observable content:** Well-known symbol methods, a declared unique-symbol key used across an interface and object, a computed literal key, and a dynamic computed key that cannot name a fixed property.
- **Variants and failures:** `keyof` inclusion, symbol-index signatures, declaration emit, const versus let symbol identity, and downlevel/runtime Symbol availability.
- **Constraints / authority:** ECMAScript symbols plus TypeScript type rules and selected libraries.
- **Confidence:** High.
- **Likely verification source:** TypeScript Handbook Symbols documentation and ECMAScript Language Specification.

### TS-052 — Destructuring, binding patterns, rest, and spread

- **Distinct behavior:** Object and array binding patterns create multiple scoped symbols with defaults and control-flow-sensitive types. Rest and spread compute new object/tuple shapes and have ordered runtime overwrite behavior.
- **Observable content:** Nested parameter and local destructuring, aliases, defaults, rest bindings, object and tuple spread, and duplicate-property overwrites.
- **Variants and failures:** Optional source properties, discriminants separated from payload, iterable requirements, getter invocation, non-enumerable properties, `exactOptionalPropertyTypes`, and downlevel helper semantics.
- **Constraints / authority:** ECMAScript runtime plus TypeScript inference and official emit.
- **Confidence:** High.
- **Likely verification source:** TypeScript Handbook Variable Declarations and Object Types documentation.

### TS-053 — Explicit resource management declarations

- **Distinct behavior:** `using` and `await using` bind resources whose disposal methods run at scope exit with defined ordering and suppressed-error behavior. Their types rely on disposable protocols and their emit may require helpers.
- **Observable content:** Synchronous and asynchronous resources, nested scopes, early return and throw paths, multiple resources, and invalid resource types.
- **Variants and failures:** Missing disposable libraries, target/runtime support, downlevel helpers, nullish resources, disposal throwing during another exception, and `await using` context restrictions.
- **Constraints / authority:** ECMAScript explicit resource management plus official TypeScript support and bundled libs; version/host dependent.
- **Confidence:** Medium.
- **Likely verification source:** TypeScript 5.2 release notes and ECMAScript proposal/specification material.

### TS-054 — Decorator models and metadata-related edges

- **Distinct behavior:** Current ECMAScript decorators and legacy experimental decorators have different signatures, evaluation/application order, supported targets, and emit. Decorators create runtime references that ordinary type annotations do not.
- **Observable content:** Class, method, field/accessor, and parameter cases appropriate to each model, decorator factories, replacement and initializer behavior, and typed decorator context.
- **Variants and failures:** `experimentalDecorators`, `emitDecoratorMetadata`, metadata's lossy runtime type encoding, parameter decorators limited to legacy mode, auto-accessors, private elements, and incompatible decorator return types.
- **Constraints / authority:** ECMAScript decorator standard plus TypeScript language/legacy implementation; reflect-metadata use is ecosystem convention.
- **Confidence:** High for broad distinctions; medium for edge ordering details.
- **Likely verification source:** TypeScript 5.0 release notes, TSConfig Reference, and ECMAScript decorators specification.

### TS-055 — JSX syntax, namespace typing, and transform modes

- **Distinct behavior:** `.tsx` parses angle brackets as JSX, derives element/attribute/children types from the `JSX` namespace or configured runtime, and emits or preserves different runtime calls by `jsx` mode.
- **Observable content:** Intrinsic and component elements, required/optional props, children, generic components, a namespaced or customized JSX typing surface, and both classic and automatic runtime modes.
- **Variants and failures:** `jsxFactory`, `jsxFragmentFactory`, `jsxImportSource`, development runtime, `key`-like special attributes supplied by libraries, lowercase versus uppercase tag resolution, and conflicting global JSX declarations.
- **Constraints / authority:** TypeScript JSX checking/emit plus framework and runtime declaration conventions.
- **Confidence:** High.
- **Likely verification source:** TypeScript JSX documentation and TSConfig Reference.

## Type relationships, inference, and flow analysis

### TS-056 — Primitive, literal, top, bottom, and escape-hatch types

- **Distinct behavior:** Primitive types, literals, `object`, `{}`, `unknown`, `any`, `never`, and `void` occupy materially different assignability positions. `any` propagates unsoundness, `unknown` requires refinement, and `never` marks impossible results.
- **Observable content:** Assignments and calls that contrast each type, exhaustive branches yielding `never`, a `void` callback return relation, and operations permitted only through `any`.
- **Variants and failures:** Strict null checks, evolving `any`, implicit `any`, unreachable code diagnostics, `never` inference for declarations versus expressions, and boxed primitive types.
- **Constraints / authority:** TypeScript language/checker.
- **Confidence:** High.
- **Likely verification source:** TypeScript Handbook Everyday Types and Narrowing documentation.

### TS-057 — Structural assignability and member compatibility

- **Distinct behavior:** Most object types are compatible by structure rather than declaration name. Required, optional, readonly, method, call, construct, index, private, and protected members contribute differently.
- **Observable content:** Independently declared compatible interfaces, assignment with extra stored properties, incompatible member types, callable objects, and classes whose private origins prevent otherwise structural assignment.
- **Variants and failures:** Freshness, generic erasure-like comparisons, recursive structures, weak types, optional exactness, variance flags, and enum/unique-symbol nominal pockets.
- **Constraints / authority:** TypeScript language; some algorithms are implementation-defined checker behavior.
- **Confidence:** High.
- **Likely verification source:** TypeScript Handbook Type Compatibility documentation.

### TS-058 — Fresh object literals and excess-property checking

- **Distinct behavior:** Fresh object literals receive excess-property checks that ordinary structural assignments often do not. Context, spreads, assertions, generics, and intermediate variables can remove freshness.
- **Observable content:** The same extra-property object passed directly, stored first, asserted, spread, checked with `satisfies`, and passed through a generic.
- **Variants and failures:** Index signatures, unions, weak target types, optional properties, discriminant typos, exactness misconceptions, and return-context checking.
- **Constraints / authority:** TypeScript checker behavior; not a general exact-object type guarantee.
- **Confidence:** High.
- **Likely verification source:** TypeScript Handbook Object Types documentation.

### TS-059 — Union, intersection, and discriminated-union identity

- **Distinct behavior:** Unions expose only safely common operations until narrowed; intersections combine requirements and can reduce incompatible primitives or properties to `never`. Discriminant properties connect variants to branch-specific members.
- **Observable content:** A tagged union with exhaustive handling, an untagged union, callable/intersected object types, and contradictory intersections.
- **Variants and failures:** Optional or mutated discriminants, unions of object literals, intersections involving private members, normalization differences, and distributed conditional types.
- **Constraints / authority:** TypeScript language/checker; normalization presentation can change by compiler version.
- **Confidence:** High.
- **Likely verification source:** TypeScript Handbook Narrowing and Object Types documentation.

### TS-060 — Control-flow narrowing and alias analysis

- **Distinct behavior:** Types narrow through `typeof`, equality, truthiness, `in`, `instanceof`, discriminant checks, assignments, reachability, and recognized alias relationships, then widen when mutation or calls invalidate facts.
- **Observable content:** Branches using every major guard, aliased conditions, captured variables, reassignment, early returns, loops, optional values, and an unsafe mutation boundary.
- **Variants and failures:** Truthiness losing empty-string/zero cases, `in` with optional properties on both sides, cross-realm `instanceof`, callback closure timing, and version-dependent analysis depth.
- **Constraints / authority:** TypeScript checker; runtime predicates follow ECMAScript.
- **Confidence:** High.
- **Likely verification source:** TypeScript Handbook Narrowing documentation and release notes.

### TS-061 — User-defined type predicates and assertion functions

- **Distinct behavior:** A return predicate narrows a parameter or `this` at call sites; an assertion signature narrows after normal return or marks a condition asserted. The compiler trusts declared predicates beyond what their bodies prove.
- **Observable content:** Predicate function, inferred predicate where supported, `this` predicate, assertion function, generic predicate, and an intentionally unsound body demonstrating trust.
- **Variants and failures:** Predicate target must be assignable to parameter type, callbacks such as array filters, mutation after narrowing, inferred-predicate rules introduced in newer versions, and assertion call-target restrictions.
- **Constraints / authority:** TypeScript language/checker; inference is version dependent.
- **Confidence:** High.
- **Likely verification source:** TypeScript Handbook Narrowing documentation and TypeScript 5.5 release notes.

### TS-062 — Nullability, optional properties, and optional parameters

- **Distinct behavior:** Under strict null checks, `null` and `undefined` are distinct union members. Missing properties, present properties holding `undefined`, optional parameters, and defaulted parameters overlap but are not identical, especially with exact optional types.
- **Observable content:** Nullable values, optional and required-possibly-undefined properties, presence tests, optional/default parameters, and assignments under both exactness modes.
- **Variants and failures:** Legacy non-strict collapse, mapped optional modifiers, object spread, tuple optional elements, JSON omission, and optional chaining that does not validate unrelated values.
- **Constraints / authority:** TypeScript language/checker; runtime object presence follows ECMAScript.
- **Confidence:** High.
- **Likely verification source:** TSConfig Reference and TypeScript Handbook.

### TS-063 — Index signatures and unchecked indexed access

- **Distinct behavior:** String, number, symbol, template-pattern, and union index signatures constrain unknown-key access and named members. `noUncheckedIndexedAccess` adds `undefined` for undeclared keys.
- **Observable content:** Types with each key family, compatible and incompatible named properties, dictionary reads, numeric keys interacting with string keys, and property access restricted by configuration.
- **Variants and failures:** `readonly` indexes, mixed explicit members, `keyof` results, arrays/tuples, `noPropertyAccessFromIndexSignature`, and symbol/template index support by compiler version.
- **Constraints / authority:** TypeScript language/checker.
- **Confidence:** High.
- **Likely verification source:** TypeScript Handbook Object Types documentation and release notes.

### TS-064 — Readonly views versus runtime immutability

- **Distinct behavior:** `readonly` properties, readonly arrays/tuples, mapped modifiers, and const assertions restrict writes through a type view but usually do not freeze runtime objects. Assignability has intentional convenience and unsound edges.
- **Observable content:** Mutable object viewed through readonly type, nested object mutation, readonly tuple/array parameters, mapped removal/addition, and runtime mutation through an alias.
- **Variants and failures:** Getter-only properties, readonly index signatures, variance of readonly arrays, `Object.freeze` library typing, private fields, and shallow const assertions.
- **Constraints / authority:** TypeScript static semantics; runtime behavior is ECMAScript/library-defined.
- **Confidence:** High.
- **Likely verification source:** TypeScript Handbook Object Types and Utility Types documentation.

### TS-065 — Arrays, tuples, labeled elements, and variadic tuples

- **Distinct behavior:** Tuples encode length and positional types, with optional, rest, readonly, and labels. Variadic tuples preserve relationships through generic concatenation and parameter lists.
- **Observable content:** Fixed, optional, readonly, labeled, and variadic tuples; indexed reads; destructuring; spread into calls; and an out-of-bounds access.
- **Variants and failures:** Array assignability, loss of tuple precision through widening, union rest elements, optional-before-required constraints, `noUncheckedIndexedAccess`, and labels having no identity effect.
- **Constraints / authority:** TypeScript language/checker and ECMAScript arrays at runtime.
- **Confidence:** High.
- **Likely verification source:** TypeScript Handbook Object Types documentation and TypeScript 4.0 release notes.

### TS-066 — Function parameter variance and callback compatibility

- **Distinct behavior:** Parameter count, optional/rest parameters, return types, overloads, and variance determine callable compatibility. `strictFunctionTypes` applies contravariant checking more strongly to function properties than to methods, which remain intentionally bivariant in key cases.
- **Observable content:** Assignments between broader/narrower callbacks, method versus function-property forms, discarded return values, optional/rest forms, and a generic callback.
- **Variants and failures:** `void` target returns, event-handler conventions, constructor signatures, overload sets, mutable generic containers, and strictness disabled.
- **Constraints / authority:** TypeScript compatibility rules; intentional unsoundness exists for common JavaScript patterns.
- **Confidence:** High.
- **Likely verification source:** TypeScript Handbook Type Compatibility and TSConfig Reference.

### TS-067 — Overload and call-signature resolution

- **Distinct behavior:** Calls select a single applicable signature using ordered and specificity-sensitive rules; a union of arguments does not automatically distribute across overloads. Generic inference and contextual typing occur during selection.
- **Observable content:** Ordered broad/specific overloads, union argument failure, generic and non-generic candidates, callback contextual typing, and ambiguous or inapplicable calls.
- **Variants and failures:** Implementation signatures hidden from calls, declaration-merge overload ordering, rest/optional candidates, literal overloads, `this` parameters, and changed inference across releases.
- **Constraints / authority:** Official checker algorithm; detailed tie-breaking is implementation-defined and version-sensitive.
- **Confidence:** High for outcomes, medium for complete ordering detail.
- **Likely verification source:** TypeScript Handbook Functions documentation and compiler conformance tests.

### TS-068 — Generic parameters, constraints, defaults, and inference

- **Distinct behavior:** Generic declarations create parameter identities scoped to declarations. Constraints limit admissible arguments; defaults fill omitted arguments; inference gathers candidates from arguments, context, returns, and related type parameters.
- **Observable content:** Generic function, class, interface, and alias with constraints/defaults; explicit and inferred arguments; failed constraint; partial inference limitation; and nested generic scopes shadowing names.
- **Variants and failures:** `keyof` constraints, contextual return inference, empty candidate sets, circular constraints, higher-order inference, instantiation expressions, and defaults referencing earlier parameters.
- **Constraints / authority:** TypeScript language/checker; inference refinements are version dependent.
- **Confidence:** High.
- **Likely verification source:** TypeScript Handbook Generics documentation and release notes.

### TS-069 — Const type parameters and literal-preserving inference

- **Distinct behavior:** A `const` type parameter requests literal-like inference for expressions written at the call site without making values immutable or rescuing already widened variables.
- **Observable content:** Paired generic APIs with ordinary and const type parameters, inline object/array arguments, mutable constraints that force fallback, and predeclared widened arguments.
- **Variants and failures:** Readonly constraints, nested literals, explicit type arguments, contextual types, and assignability fallback when the readonly candidate violates a mutable constraint.
- **Constraints / authority:** TypeScript language/checker; introduced in TypeScript 5.0.
- **Confidence:** High.
- **Likely verification source:** TypeScript 5.0 release notes.

### TS-070 — Conditional types, distribution, and `infer`

- **Distinct behavior:** Conditional types choose branches by assignability, defer when generic, distribute over naked type parameters applied to unions, and introduce inferred type variables within branches.
- **Observable content:** Distributive and bracket-suppressed non-distributive forms, nested `infer`, overload extraction behavior, `never` input, and constrained generics.
- **Variants and failures:** `any` and `unknown`, union normalization, recursion depth, multiple inference sites with variance, last-overload inference, and instantiation-depth diagnostics.
- **Constraints / authority:** TypeScript type system/checker; performance limits are implementation-defined.
- **Confidence:** High.
- **Likely verification source:** TypeScript Handbook Conditional Types documentation.

### TS-071 — Mapped types and key remapping

- **Distinct behavior:** Mapped types iterate property keys, preserve or add/remove optional and readonly modifiers, and can rename or filter keys with an `as` clause. Homomorphic forms preserve source modifiers and array-like shapes in special ways.
- **Observable content:** Modifier-preserving and modifier-changing maps, key filtering/remapping, template-generated names, mapping over tuple/array types, and collision of remapped keys.
- **Variants and failures:** Symbol/number keys, optional exactness, union sources, `never` key removal, recursive maps, and index signatures.
- **Constraints / authority:** TypeScript type system/checker.
- **Confidence:** High.
- **Likely verification source:** TypeScript Handbook Mapped Types documentation.

### TS-072 — `keyof`, indexed access, and type queries

- **Distinct behavior:** `keyof` produces property-key unions; indexed access retrieves property types; `typeof` in type position queries value declarations; indexed access over unions and generic keys follows safety constraints.
- **Observable content:** String, number, and symbol keys; generic property lookup; `typeof` on variables/classes/functions; array element extraction; and invalid non-key indexing.
- **Variants and failures:** String index signatures widening `keyof`, optional properties adding `undefined`, `keyof` unions versus intersections, namespace queries, and value/type name collisions.
- **Constraints / authority:** TypeScript language/checker.
- **Confidence:** High.
- **Likely verification source:** TypeScript Handbook Type Operators documentation.

### TS-073 — Template literal types and intrinsic string transforms

- **Distinct behavior:** Template literal types form string literal unions through interpolation and cross-product expansion. Intrinsic uppercase/lowercase/capitalize transformations are compiler-known operations, not ordinary declarations.
- **Observable content:** Event-name derivation from object keys, multiple union substitutions, pattern matching with conditional inference, intrinsic transforms, and a wide `string` input.
- **Variants and failures:** Combinatorial expansion limits, numeric/bigint/boolean interpolation, key remapping integration, Unicode/casing behavior, and recursive parsers hitting instantiation limits.
- **Constraints / authority:** TypeScript type system; intrinsic implementation and complexity limits are compiler-defined.
- **Confidence:** High.
- **Likely verification source:** TypeScript Handbook Template Literal Types documentation.

### TS-074 — Recursive, mutually recursive, and circular types

- **Distinct behavior:** Interfaces, aliases, conditional types, and mapped types can be recursive within permitted forms. Some immediate circular aliases are rejected, while lazy structural recursion is accepted; deep instantiation can terminate with diagnostics.
- **Observable content:** Recursive interface/tree, mutually recursive aliases, recursive tuple/object aliases, a forbidden immediate cycle, and a generic recursion that reaches the compiler depth limit.
- **Variants and failures:** Recursive conditional types, circular mapped-property references, serialization-shaped cycles, performance differences between aliases and interfaces, and compiler-version limit changes.
- **Constraints / authority:** TypeScript checker; depth and complexity limits are implementation-defined.
- **Confidence:** Medium.
- **Likely verification source:** TypeScript release notes and compiler conformance tests.

### TS-075 — Interfaces versus type aliases

- **Distinct behavior:** Both can name object shapes, but interfaces reopen and participate in declaration merging, while aliases can name primitives, unions, tuples, and conditional/mapped forms. Recursive and diagnostic presentation can differ.
- **Observable content:** Equivalent object interface/alias, interface reopening, forbidden alias reopening, alias union, extension/intersection alternatives, and class implementation.
- **Variants and failures:** Extending unions, duplicate properties in intersections, generic recursion, error display expansion, module augmentation, and performance heuristics.
- **Constraints / authority:** TypeScript language/checker.
- **Confidence:** High.
- **Likely verification source:** TypeScript Handbook Object Types documentation.

### TS-076 — Call, construct, hybrid, and overloaded object types

- **Distinct behavior:** Object types can have call signatures, construct signatures, properties, index signatures, and overloads simultaneously. Constructing and calling the same value can yield different instance/result types.
- **Observable content:** A hybrid callable object with state, separate call and construct signatures, generic signatures, multiple overloads, and implementation assignment compatibility.
- **Variants and failures:** Abstract construct signatures, constructor `this`, static class side, interface merging, JavaScript function properties, and overload resolution.
- **Constraints / authority:** TypeScript type system; runtime callable/constructable nature is ECMAScript.
- **Confidence:** High.
- **Likely verification source:** TypeScript Handbook Functions, Generics, and Object Types documentation.

### TS-077 — Polymorphic `this` and fluent APIs

- **Distinct behavior:** The `this` type represents the current subtype in instance members, enabling inherited fluent methods and type guards tied to receiver identity. It differs from naming the base class directly.
- **Observable content:** Base fluent method returning `this`, derived chaining, `this`-based predicate, static-side contrast, and a detached method call.
- **Variants and failures:** F-bounded generics, explicit `this` parameters, mixins, intersection-return builders, private/protected state, and arrow-property methods.
- **Constraints / authority:** TypeScript type system plus ECMAScript call semantics.
- **Confidence:** High.
- **Likely verification source:** TypeScript Handbook Classes and Narrowing documentation.

### TS-078 — Contextual typing, best common type, and widening

- **Distinct behavior:** Expected types flow into expressions, especially function expressions and object/array literals. Without context, the checker computes a best common type and widens fresh literals, potentially losing correlations.
- **Observable content:** The same callback/object/array with and without context, heterogeneous arrays, contextual return types, generic inference context, and a candidate that prevents a common supertype.
- **Variants and failures:** `noImplicitAny`, union contextual signatures, evolving arrays, JSX callbacks, overload candidates, const contexts, and version-dependent inference fixes.
- **Constraints / authority:** TypeScript checker; inference algorithm is implementation-defined.
- **Confidence:** High.
- **Likely verification source:** TypeScript Handbook Type Inference documentation and release notes.

### TS-079 — Literal widening, const assertions, and `satisfies`

- **Distinct behavior:** Literal expressions may widen based on mutability/context. Const assertions retain deep literal/readonly structure for literal syntax. `satisfies` checks compatibility while generally preserving the expression's inferred type instead of replacing it with the target.
- **Observable content:** `let` versus `const`, mutable versus readonly properties, const-asserted object/tuple, annotated object, and `satisfies` with both valid and misspelled keys.
- **Variants and failures:** Excess-property checks, generic contexts, assertion restrictions to eligible expressions, contextual effects of `satisfies`, enum members, and mutation after aliasing.
- **Constraints / authority:** TypeScript language/checker; `satisfies` was introduced in TypeScript 4.9.
- **Confidence:** High.
- **Likely verification source:** TypeScript 3.4 and 4.9 release notes.

### TS-080 — Type assertions, non-null assertions, and definite assignment

- **Distinct behavior:** Type assertions alter static interpretation without runtime checks; double assertions can bridge otherwise unrelated types. Postfix non-null removes nullish constituents, while declaration definite-assignment suppresses initialization analysis.
- **Observable content:** Narrowing and widening assertions, an intentionally unsafe double assertion, non-null access that fails at runtime, and a class field assigned indirectly after construction.
- **Variants and failures:** Angle-bracket assertions unavailable in TSX, const assertions, assertion signatures, erased output, `strictPropertyInitialization`, and optional chaining contrast.
- **Constraints / authority:** TypeScript language; runtime remains ECMAScript.
- **Confidence:** High.
- **Likely verification source:** TypeScript Handbook Everyday Types and Classes documentation.

### TS-081 — Variance annotations and measured variance

- **Distinct behavior:** `in`, `out`, and `in out` annotations state variance for supported generic type declarations and are checked against use. Structural comparison may still expand types where annotations do not control the relation.
- **Observable content:** Covariant producer, contravariant consumer, invariant mutable cell, an incorrect annotation diagnostic, and comparisons between instantiated and structural forms.
- **Variants and failures:** Method bivariance, private members, recursive generics, inference versus checking, annotations unavailable on functions/classes in unsupported positions, and performance-oriented effects.
- **Constraints / authority:** TypeScript type system; introduced in TypeScript 4.7.
- **Confidence:** Medium.
- **Likely verification source:** TypeScript 4.7 release notes.

### TS-082 — Utility types and compiler intrinsics

- **Distinct behavior:** Standard utility aliases compose mapped, conditional, and intrinsic operations; some types such as `Awaited`, `ReturnType`, `Parameters`, `ThisParameterType`, and string manipulators depend on special compiler or library behavior.
- **Observable content:** Representative property-transform, union-filter, function-reflection, promise-unwrapping, and `this` utilities applied to overloads, unions, optional properties, and `any`/`never`.
- **Variants and failures:** Last-overload extraction, exact optional properties, recursive thenables, utility availability by library/compiler version, and user declarations shadowing global names.
- **Constraints / authority:** Official bundled declaration libraries plus compiler intrinsics.
- **Confidence:** High.
- **Likely verification source:** TypeScript Utility Types documentation and bundled library declarations.

## Runtime constructs with graph-visible semantic effects

### TS-083 — Async functions, promises, and top-level await

- **Distinct behavior:** Async functions always return promise-like results and `await` recursively adopts thenables according to runtime semantics. Top-level await affects module evaluation and is restricted by target/module context.
- **Observable content:** Async function, async method, custom thenable, rejection path, top-level await, and an importing module whose evaluation depends on it.
- **Variants and failures:** Downlevel helper emit, missing Promise library/runtime, CommonJS restrictions, cyclic async module graphs, `Awaited` type behavior, and unhandled rejection host policy.
- **Constraints / authority:** ECMAScript runtime, TypeScript checking/emit, and host module loader.
- **Confidence:** High.
- **Likely verification source:** ECMAScript Language Specification, TypeScript Handbook, and TSConfig Reference.

### TS-084 — Generators, async generators, and iteration protocols

- **Distinct behavior:** Generator functions relate yield, next-input, and return types; async generators combine asynchronous and iteration protocols. `for...of` and `for await...of` select protocol methods and downlevel behavior.
- **Observable content:** Explicitly typed generator, delegated yield, async generator, custom iterable/iterator, early loop exit invoking cleanup, and a mismatched next/yield type.
- **Variants and failures:** `downlevelIteration`, missing iterable libs, arrays versus custom iterables, iterator helper availability, async-from-sync iteration, and target-dependent helper emit.
- **Constraints / authority:** ECMAScript runtime plus TypeScript libraries/checker and official emit.
- **Confidence:** High.
- **Likely verification source:** ECMAScript Language Specification, TypeScript release notes, and TSConfig Reference.

### TS-085 — Optional chaining and nullish coalescing

- **Distinct behavior:** Optional property, element, and call chains short-circuit only on nullish receivers and preserve method receiver semantics. Nullish coalescing differs from truthiness-based fallback.
- **Observable content:** Every chain form, a getter or index expression with a visible side effect, optional method call, grouping that breaks a chain, and zero/empty-string contrasts with logical OR.
- **Variants and failures:** Assignment restrictions, deleted properties, precedence diagnostics when mixed with logical operators, downlevel temporary variables, and result types under strict null checks.
- **Constraints / authority:** ECMAScript runtime plus TypeScript flow/type analysis and emit.
- **Confidence:** High.
- **Likely verification source:** ECMAScript Language Specification and TypeScript 3.7 release notes.

### TS-086 — Object model dispatch, prototypes, and mixins

- **Distinct behavior:** Method calls dispatch through JavaScript prototypes; interface implementation and structural intersections do not create runtime inheritance. Common mixin patterns synthesize subclass constructors and combined instance types.
- **Observable content:** Base/derived override call, extracted method receiver loss, prototype mutation, interface-only relation, and a generic class-expression mixin preserving constructor constraints.
- **Variants and failures:** Arrow-property dispatch, `super`, private fields in mixins, static members, declaration mixins, `Object.assign` shallow copies, and duplicate method names.
- **Constraints / authority:** ECMAScript runtime plus TypeScript structural/mixin conventions.
- **Confidence:** High.
- **Likely verification source:** TypeScript Handbook Classes and Mixins documentation and ECMAScript Language Specification.

### TS-087 — Exception flow, catch typing, and reachability

- **Distinct behavior:** JavaScript permits throwing any value. Catch variables are `unknown` under strict modern checking unless configured otherwise, and `throw`, `return`, infinite loops, and `never`-returning calls affect reachability and narrowing.
- **Observable content:** Throws of Error and non-Error values, guarded catch handling, finally paths overriding completion, a never-returning function, and unreachable statements.
- **Variants and failures:** `useUnknownInCatchVariables`, async rejection, assertion functions, `allowUnreachableCode`, disposal errors, and control flow through `try`/`finally`.
- **Constraints / authority:** ECMAScript runtime plus TypeScript control-flow diagnostics.
- **Confidence:** High.
- **Likely verification source:** TSConfig Reference, TypeScript 4.4 release notes, and ECMAScript Language Specification.

### TS-088 — Switch exhaustiveness and `never`-based proofs

- **Distinct behavior:** TypeScript narrows discriminated unions across switch cases, but exhaustive checking is commonly expressed through a `never` assignment or return-type/return-path diagnostics rather than one universal exhaustiveness switch.
- **Observable content:** Complete and incomplete switches over a closed discriminated union or enum, deliberate `never` assertion, default case, and fallthrough.
- **Variants and failures:** Open unions through declaration merging, enum members, optional discriminants, default masking newly added variants, `noFallthroughCasesInSwitch`, and ESLint exhaustiveness rules outside compiler semantics.
- **Constraints / authority:** TypeScript checker plus ecosystem lint conventions.
- **Confidence:** High.
- **Likely verification source:** TypeScript Handbook Narrowing documentation and TSConfig Reference.

### TS-089 — Evaluation order and downlevel helper interactions

- **Distinct behavior:** Property access, arguments, decorators, computed keys, spreads, class initialization, and resource cleanup have observable evaluation order. Downlevel emit introduces temporaries and helpers that should preserve specified behavior.
- **Observable content:** Side-effectful computed names, getters, spreads, arguments, class elements, and cleanup arranged so order is observable under modern and older targets.
- **Variants and failures:** Helper deduplication with `importHelpers`, custom/old `tslib`, native versus transformed class fields, decorator model, and iterability assumptions in spread.
- **Constraints / authority:** ECMAScript semantics and official implementation; helper package compatibility is ecosystem/toolchain.
- **Confidence:** High.
- **Likely verification source:** ECMAScript Language Specification, TSConfig Reference, and `tslib` documentation.

## Diagnostics, directives, dependencies, and ecosystem boundaries

### TS-090 — Diagnostic suppression and expectation directives

- **Distinct behavior:** `@ts-ignore`, `@ts-expect-error`, `@ts-check`, and `@ts-nocheck` alter diagnostics at specific lines or files. An unused expectation is itself an error, making negative tests version-sensitive.
- **Observable content:** Suppressed error, correctly expected error, stale expectation with no underlying error, checked JavaScript file, and unchecked TypeScript/JavaScript file where supported.
- **Variants and failures:** Directive placement, multiple diagnostics on one line, compiler-version diagnostic changes, declaration files, build versus editor diagnostics, and linter policies.
- **Constraints / authority:** Official implementation; comments are not runtime semantics.
- **Confidence:** High.
- **Likely verification source:** TypeScript release notes and JavaScript checking documentation.

### TS-091 — Syntax, semantic, declaration, configuration, and resolution errors

- **Distinct behavior:** Different diagnostic phases can coexist and have different source locations and emit consequences. Recovery may still construct partial declarations and relationships after malformed syntax.
- **Observable content:** One isolated example of parse failure, duplicate/semantic binding failure, type failure, declaration-emit failure, invalid option combination, and unresolved module.
- **Variants and failures:** Cascading diagnostics, related-information locations, global diagnostics without a file, locale/message changes, stable numeric diagnostic codes, and editor-only suggestion diagnostics.
- **Constraints / authority:** Official implementation; diagnostic text and ordering are not stable APIs, while codes are more stable but can still evolve.
- **Confidence:** High.
- **Likely verification source:** TypeScript compiler API declarations, source, and conformance tests.

### TS-092 — Dependency type discovery and ambient package inclusion

- **Distinct behavior:** Visible `@types` packages, `types`, `typeRoots`, package-local declarations, and automatic type acquisition determine ambient names and module declarations. Inclusion can change merely because a transitive visible package appears.
- **Observable content:** A runtime package with bundled types, a separate `@types` package, ambient globals gated by `types`, a custom `typeRoots`, and a type-only package conflict.
- **Variants and failures:** Monorepo ancestor visibility, duplicate `@types` versions, scoped package name mapping, `types: []`, automatic type acquisition for JavaScript projects, and package manager hoisting.
- **Constraints / authority:** Official implementation plus package-manager layout and DefinitelyTyped ecosystem convention.
- **Confidence:** High.
- **Likely verification source:** TSConfig Reference and TypeScript Declaration Files publishing/consumption documentation.

### TS-093 — Dependency declaration quality and `skipLibCheck`

- **Distinct behavior:** Third-party declaration files can contain internal errors, incompatible duplicate globals, or inaccurate runtime shapes. `skipLibCheck` suppresses checking declaration-file bodies but does not make all referenced types disappear or guarantee runtime compatibility.
- **Observable content:** A deliberately inconsistent dependency declaration, a consumer-visible exported type, and contrasting diagnostics with library checking enabled/disabled.
- **Variants and failures:** `skipDefaultLibCheck`, duplicate package copies, module augmentation, errors that still surface at use sites, compiler-version incompatibility, and patched dependencies.
- **Constraints / authority:** Official implementation; declaration correctness and patches are ecosystem responsibility.
- **Confidence:** High.
- **Likely verification source:** TSConfig Reference and TypeScript Declaration Files documentation.

### TS-094 — Package manager and lockfile-selected graph

- **Distinct behavior:** Dependency versions, peer resolution, hoisting, workspaces, symlinks, and Plug'n'Play-style resolution determine which declarations and package identities are visible. Same source can acquire different types under a different install graph.
- **Observable content:** Direct and transitive dependencies with two versions of a nominally sensitive type, a peer dependency, a workspace link, and a pinned lockfile.
- **Variants and failures:** npm, pnpm, Yarn node-modules, Yarn Plug'n'Play integrations, optional dependencies, platform-filtered packages, and missing peers.
- **Constraints / authority:** Package-manager/ecosystem behavior; TypeScript supports some layouts directly and others through host integrations.
- **Confidence:** Medium.
- **Likely verification source:** Relevant package-manager documentation and TypeScript Module Resolution documentation.

### TS-095 — Compiler replacement and transpiler divergence

- **Distinct behavior:** Babel, SWC, esbuild, framework compilers, and runtime loaders often erase or transform TypeScript syntax without performing the full `tsc` type check. Their supported syntax and emit semantics can diverge, especially for enums, namespaces, decorators, class fields, and isolated constructs.
- **Observable content:** A manifest/configuration identifying a non-`tsc` transpiler, a separate type-check step, and constructs whose emit or rejection differs between the tools.
- **Variants and failures:** Type stripping in modern hosts, const enums, legacy decorators, JSX, import elision, helper choices, and source maps.
- **Constraints / authority:** Ecosystem tool-specific; TypeScript defines checking and its own emit only.
- **Confidence:** High.
- **Likely verification source:** Official documentation for TypeScript and the selected transpiler.

### TS-096 — Language service, build, and command-line program differences

- **Distinct behavior:** Editors may create inferred or configured projects, redirect referenced-project sources, include open files outside roots, apply plugins, and report suggestions that a clean `tsc` build does not.
- **Observable content:** Configured and inferred-project candidates, an open excluded file, a project reference, and a language-service plugin declaration, with command-line results recorded separately.
- **Variants and failures:** `disableSourceOfProjectReferenceRedirect`, automatic type acquisition, plugin-only diagnostics/navigation, stale editor state, and multiple nearest configurations.
- **Constraints / authority:** Official language service and editor host; plugins are ecosystem-specific.
- **Confidence:** Medium.
- **Likely verification source:** TypeScript compiler/language-service API documentation and TSConfig Reference.

### TS-097 — Custom transformers and compiler API consumers

- **Distinct behavior:** Compiler API programs and custom transformers can synthesize, remove, or rewrite emitted nodes and source maps without changing the checker-visible source graph. Transformers are not a stable semantic extension mechanism for type checking.
- **Observable content:** Configuration or build code that invokes the compiler API, a before/after/declaration transformer, and a generated runtime reference absent from authored TypeScript.
- **Variants and failures:** Unofficial plugin patching, factory API version churn, synthetic nodes lacking source positions, declaration transforms, incremental programs, and third-party wrappers.
- **Constraints / authority:** Official compiler API where public, but many transformer practices depend on implementation details/ecosystem tools.
- **Confidence:** Medium.
- **Likely verification source:** TypeScript Compiler API documentation and compiler API declarations.

### TS-098 — Runtime/type entry-point agreement in published packages

- **Distinct behavior:** A package may type-check through one declaration path but load a different JavaScript path. Correctness requires agreement among conditions, module format, exported names, extensions, and declaration flavor.
- **Observable content:** Import and require consumers, public subpaths, matching `.d.mts`/`.d.cts` declarations, and one intentionally mismatched branch that yields either a resolution or runtime failure.
- **Variants and failures:** Dual-package hazard, default-export interop, extensionless declarations, bundled versus external dependencies, types-only exports, and bundler conditions.
- **Constraints / authority:** Node/package ecosystem modeled by TypeScript resolution; actual loading is host-defined.
- **Confidence:** High.
- **Likely verification source:** TypeScript Declaration Files and Module documentation and Node.js packages documentation.

### TS-099 — Platform- and condition-specific source variants

- **Distinct behavior:** Browser/server, development/production, import/require, and custom-condition branches can select different implementations and declarations. `lib` and global declarations also change the visible platform surface.
- **Observable content:** Conditional package branches with different exports, platform-specific ambient globals, and configurations that resolve the same specifier differently.
- **Variants and failures:** Bundler custom conditions, Node conditions, `browser` field conventions, React Native-style suffixes handled by other tools, optional native packages, and declaration drift between branches.
- **Constraints / authority:** Host/ecosystem with partial official resolver modeling.
- **Confidence:** Medium.
- **Likely verification source:** Node.js packages documentation, TypeScript Module Resolution documentation, and relevant bundler documentation.

### TS-100 — Version-skewed compiler and library behavior

- **Distinct behavior:** TypeScript minor releases can alter inference, narrowing, assignability, standard library declarations, emitted helpers, module resolution, and diagnostics even without source changes.
- **Observable content:** Compiler/package version pins, a second supported compiler run where required, and examples tied to known transition areas rather than assuming cross-version identity.
- **Variants and failures:** Package `typesVersions`, dependency minimum TypeScript versions, editor using a bundled compiler instead of workspace compiler, changed `lib.d.ts`, and stale `tslib`.
- **Constraints / authority:** Official implementation and package ecosystem.
- **Confidence:** High.
- **Likely verification source:** TypeScript release notes, package metadata, and bundled library declarations.

## Material recent-version transitions to verify

The following transitions are especially likely to change graph-visible results. Exact behavior should be verified against the pinned compiler rather than inferred from a broader major-version label.

| Version area | Material change to exercise or re-check | Why graph results can differ |
|---|---|---|
| TypeScript 4.7 | Node16/NodeNext module detection and resolution; `.mts`/`.cts` and declaration counterparts; package `type`/`exports`; instantiation expressions; variance annotations | File format, resolved target, alias identity, emitted extension, and generic surface can change. |
| TypeScript 4.8 | Intersection reduction and narrowing refinements; stricter unconstrained generics | Apparent normalized types and accepted assignments can change. |
| TypeScript 4.9 | `satisfies`; unlisted-property narrowing with `in`; auto-accessors | Expression type retention, flow facts, and class member kinds can change. |
| TypeScript 5.0 | Standard decorators; const type parameters; `verbatimModuleSyntax`; multiple configuration inheritance; broader module-option implications | Runtime decorator edges, inference, import retention, and effective configuration can change. |
| TypeScript 5.1 | Easier implicit returns for `undefined`; accessor type refinements; namespaced JSX attributes | Function return diagnostics, accessor relations, and JSX names can change. |
| TypeScript 5.2 | Explicit resource management; decorator metadata context support; tuple label preservation improvements | New declaration kinds, disposal edges, metadata, and tuple presentation appear. |
| TypeScript 5.3 | Import attributes and improved narrowing around comparisons/`switch (true)` | Module syntax preservation and flow results can change. |
| TypeScript 5.4 | `NoInfer`; closure narrowing refinements; checked import attributes | Generic inference candidate sets, captured-variable flow, and module diagnostics can change. |
| TypeScript 5.5 | Inferred type predicates; isolated declarations; declaration emit portability checks; control-flow refinements | Function return predicates, declaration eligibility, and emitted public types can change. |
| TypeScript 5.6 | Unchecked side-effect import checking; syntactic truthy/nullish diagnostics; iterator typing changes; build behavior options | Resolution errors, diagnostics, iterator types, and downstream builds can change. |
| TypeScript 5.7 | Relative import-extension rewriting and further initialization/control-flow checks | Emitted specifiers, source/output correspondence, and diagnostics can change. |
| Later versions | Any release after the chosen baseline | Review release notes and bundled libraries before treating this checklist as complete. |

## Implementation-defined, host-defined, or intentionally unspecified areas

The project should make these boundaries explicit rather than treating one observed result as universal:

- Type inference, overload tie-breaking details, union/intersection normalization, diagnostic ordering/text, recursion limits, and performance cutoffs are chiefly `tsc` implementation behavior and can change between releases.
- Module specifier resolution is an official compiler model of a host. Runtime success still depends on Node.js, browser import maps, bundlers, loaders, or framework tooling.
- Filesystem case sensitivity, realpath/symlink handling, watcher behavior, newline/path presentation, and package layout depend on operating system and package manager.
- JavaScript property enumeration, evaluation order, prototype dispatch, private fields, modules, promises, and resource cleanup are ECMAScript behavior. TypeScript downlevel emit intends to preserve it within documented target limitations.
- Ambient library declarations describe expected platforms but do not prove runtime availability. Polyfills, host versions, and deployment conditions remain separate.
- Decorator metadata, asset modules, path aliases at runtime, tree-shaking, declaration bundling, framework JSX rules, compiler plugins, and custom transforms are ecosystem contracts.
- Structural typing contains deliberate unsoundness for common JavaScript patterns. Successful assignment is not proof that every mutation or callback use is runtime-safe.
- `.d.ts` files are assertions about runtime code. The compiler generally cannot verify that declarations match the JavaScript they describe.

## Known uncertainty and follow-up verification areas

- Exact post-5.7 features and defaults are outside the confidently recalled baseline and require release-note verification for the compiler actually selected.
- Fine-grained Node.js conditional-type resolution, including every interaction among nested `types` conditions, `typesVersions`, package self-references, and dual packages, merits primary-source and compiler-test confirmation.
- Import attributes, arbitrary extensions, and host-specific module customization continue to evolve across ECMAScript, Node.js, browsers, and bundlers.
- Standard decorator edge cases involving private elements, auto-accessors, inheritance, replacement classes, metadata, and initializer ordering merit specification-level confirmation.
- Explicit resource management proposal integration, disposal error composition, and downlevel helper behavior merit confirmation against the selected target and `tslib`.
- `isolatedDeclarations`, declaration inference portability, and build-mode behavior changed rapidly in the TypeScript 5.5–5.6 period and should be verified against exact compiler diagnostics.
- Language-service project selection and plugin behavior depend heavily on editor host APIs and may not be reproducible through `tsc` alone.
- Compiler performance limits for recursive conditional/mapped/template types are intentionally not stated numerically because they are implementation and version dependent.

## Final coverage audit

Feature families represented:

- Program roots, configuration inheritance, project references, incremental builds, emit policy, generated artifacts, and source identity.
- Targets, standard libraries, strictness, JavaScript/JSDoc inputs, declaration inputs/outputs, and single-file transform constraints.
- ECMAScript and CommonJS modules, package metadata, conditional resolution, aliases, re-exports, cycles, augmentations, namespaces, and triple-slash dependencies.
- Declaration spaces, lexical binding, functions, classes, visibility, fields, accessors, static members, merging, enums, symbols, destructuring, resources, decorators, and JSX.
- Structural compatibility, freshness, unions/intersections, flow narrowing, predicates, nullability, indexed access, readonly views, tuples, variance, overloads, generics, conditional/mapped/template/recursive types, contextual inference, and assertions.
- Async/iteration, optional chaining, runtime dispatch, exception/reachability behavior, evaluation order, diagnostic directives/phases, dependency typing, package managers, alternate transpilers, language service, compiler API, publishing, platform variants, and compiler skew.

Families that may still require expansion after primary-source review:

- New syntax, configuration flags, standard-library declarations, or deprecations introduced after TypeScript 5.7.
- Framework-specific template compilers and JSX conventions beyond TypeScript's common JSX contract.
- Non-Node runtimes and their TypeScript-aware resolution rules, including native type stripping and host-specific declaration libraries.
- Package-manager resolver plugins and Plug'n'Play integrations that replace ordinary filesystem lookup.
- Edge cases discoverable only from compiler conformance suites, especially parser recovery, overload ordering, normalization, and declaration emit naming.

No broad semantic family is knowingly omitted. The highest-risk gaps are evolving module/package resolution, recent declaration-isolation behavior, decorator/resource-management edge cases, and post-baseline compiler changes.
