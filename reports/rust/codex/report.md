# Rust semantic coverage candidate checklist

## Scope and baseline assumptions

This report identifies Rust features and project conditions whose omission could hide materially distinct entities, identities, relationships, name or type resolution, visibility, dispatch, source selection, build behavior, or diagnostics. It is intentionally independent of any graph schema or analysis method.

Baseline assumptions:

- The reference implementation is `rustc` with Cargo and rustup-distributed standard-library components.
- The primary baseline is stable Rust 1.85.0 and Cargo 1.85.0, released with the Rust 2024 edition. A project should also expose edition-sensitive behavior rather than treating the edition as a compiler-version alias.
- Host and target may differ. Target triples, target capabilities, linker choice, enabled features, environment, and compiler channel are semantic inputs.
- Cargo package, crate, module, item, generic instantiation, macro expansion, and linked native object are distinct concepts. A single package may produce several crates, and a crate may be compiled several times under different roles or feature sets.
- Stable behavior is preferred for the baseline. Selected nightly-only and recently stabilized cases belong where they expose a distinct boundary, but must be clearly gated.
- Source authority is ordered roughly as: Rust Reference and edition material for language rules; Cargo documentation for package/build behavior; `rustc` documentation for implementation behavior; standard-library documentation for language-coupled library types and macros; accepted Rust RFCs for design context; ecosystem documentation only for ecosystem conventions.

Each checklist entry includes an observable demonstration rather than fixture code or layout.

## Crates, packages, targets, and source selection

### RUST-PROJ-001 — Package versus crate identity

- **Distinct behavior:** A Cargo package is distribution metadata; each library, binary, example, integration test, benchmark, and build script is a separately compiled crate with its own crate root and namespace.
- **Observable content:** One package declaring a library and multiple named targets, with a shared source file referenced by more than one target.
- **Variants and failures:** Implicit versus explicit target discovery; same textual file in separate crate identities; duplicate target names are rejected; `crate` refers to the current compilation, not the package.
- **Constraints / authority / confidence / source:** Cargo-defined with language-level crate semantics; **high**. Verify in Cargo documentation on packages and targets and the Rust Reference on crates.

### RUST-PROJ-002 — Crate name, package name, and dependency rename

- **Distinct behavior:** Package names may contain hyphens while Rust crate identifiers use underscores, and a dependency can be bound under a third local name.
- **Observable content:** A dependency whose package name, library target name, and local dependency key are observably different, followed by paths using the local crate name.
- **Variants and failures:** `package =` rename; explicit `[lib] name`; name collision with another dependency or local item; macro-generated absolute paths.
- **Constraints / authority / confidence / source:** Cargo and Rust name-resolution behavior; **high**. Verify in Cargo dependency and target documentation and the Rust Reference on paths.

### RUST-PROJ-003 — Workspace membership and dependency inheritance

- **Distinct behavior:** A workspace groups packages for resolution, locking, commands, and inherited metadata without creating a Rust namespace relationship.
- **Observable content:** Root and member manifests using explicit members, exclusions, default members, and `workspace = true` dependency or package fields.
- **Variants and failures:** Virtual versus rooted workspace; nested path packages not automatically members in all cases; resolver selection; missing inherited key diagnostics.
- **Constraints / authority / confidence / source:** Cargo-defined; **high**. Verify in Cargo workspace documentation.

### RUST-PROJ-004 — Dependency kinds and host/target compilation

- **Distinct behavior:** Normal, development, and build dependencies participate in different target builds; build dependencies execute for the host even during cross-compilation.
- **Observable content:** The same package used under more than one dependency kind, plus a build script that reports host and target.
- **Variants and failures:** Dev dependencies excluded from ordinary library builds; proc-macro dependencies are host artifacts; target-specific build dependency; version unification can still couple kinds depending on resolver.
- **Constraints / authority / confidence / source:** Cargo-defined; **high**. Verify in Cargo dependency and build-script documentation.

### RUST-PROJ-005 — Cargo feature unification and resolver versions

- **Distinct behavior:** Features are additive build inputs and may unify across dependency edges; resolver versions alter which target, development, and build dependency features unify.
- **Observable content:** Optional and non-optional dependencies reached by multiple packages with disjoint feature requests, exercised under resolver 1, 2, or 3 where supported.
- **Variants and failures:** `dep:` suppression of implicit feature; `pkg?/feature`; default features; command-selected features; mutually exclusive features require explicit diagnostics because Cargo does not make them exclusive.
- **Constraints / authority / confidence / source:** Cargo-defined; **high**. Verify in Cargo feature resolver documentation.

### RUST-PROJ-006 — Optional dependencies and weak feature forwarding

- **Distinct behavior:** An optional dependency is absent unless activated, while weak feature forwarding enables a dependency feature only if that dependency is otherwise selected.
- **Observable content:** Optional dependency with `dep:name`, `name/feature`, and `name?/feature` activation paths.
- **Variants and failures:** Public feature-name compatibility; accidental implicit feature; feature references to undeclared dependencies are manifest errors.
- **Constraints / authority / confidence / source:** Cargo-defined; **high**. Verify in Cargo features documentation.

### RUST-PROJ-007 — Target-specific dependency tables

- **Distinct behavior:** Cargo selects dependency edges by target triple or supported `cfg` predicates, so the dependency graph can differ by compilation target.
- **Observable content:** Unix, Windows, architecture, and exact-triple dependency sections that expose different APIs with a common conditional caller.
- **Variants and failures:** Host versus target confusion for build dependencies; unsupported keys such as feature-based target table conditions; overlapping tables merge.
- **Constraints / authority / confidence / source:** Cargo-defined; **high**. Verify in Cargo platform-specific dependency documentation.

### RUST-PROJ-008 — Source replacement, patches, and multiple versions

- **Distinct behavior:** Dependency identity includes package name, version, and source; patches can replace resolution, and multiple semver-incompatible versions may coexist as distinct crates.
- **Observable content:** Two dependency versions exposed through aliases and a workspace patch changing one source.
- **Variants and failures:** `[patch]` versus path dependency; same type name from different crate versions is type-incompatible; unused patch warning; lockfile pins actual source.
- **Constraints / authority / confidence / source:** Cargo-defined; **high**. Verify in Cargo source replacement, patch, and resolver documentation.

### RUST-PROJ-009 — Lockfile and reproducible resolution

- **Distinct behavior:** `Cargo.lock` records selected package identities, checksums, and dependency edges; library and binary publishing/use conventions differ, but Cargo behavior is determined by lockfile presence and command flags.
- **Observable content:** A committed lockfile with two versions of a transitive package and a manifest range that permits more than the locked version.
- **Variants and failures:** `--locked`, `--frozen`, and offline mode; lockfile-format support varies by Cargo version; path packages lack registry checksums.
- **Constraints / authority / confidence / source:** Cargo-defined; **high**. Verify in Cargo lockfile and command documentation.

### RUST-PROJ-010 — Build script generated configuration and files

- **Distinct behavior:** `build.rs` runs before crate compilation and can emit link inputs, environment variables, custom `cfg` names/values, rerun triggers, metadata, and generated Rust source.
- **Observable content:** Build script emitting `rustc-cfg`, `rustc-check-cfg`, `rustc-env`, link directives, and a file in `OUT_DIR` included by the crate.
- **Variants and failures:** Directive ordering can affect linker arguments; undeclared custom cfg produces checking warnings; non-UTF-8 output/path issues; build failure prevents crate analysis; generated content changes without source-tree edits.
- **Constraints / authority / confidence / source:** Cargo-defined protocol plus `rustc` flags; **high**. Verify in Cargo build-script documentation and `rustc` cfg checking documentation.

### RUST-PROJ-011 — Generated source inclusion

- **Distinct behavior:** `include!` parses another file in the caller's syntactic and macro-hygiene context, whereas `include_str!` and `include_bytes!` produce values and do not add Rust entities.
- **Observable content:** Source generated into `OUT_DIR` and included as items, plus text and bytes included as constants.
- **Variants and failures:** Relative path base; syntax must fit expression or item context; diagnostics may refer to included paths; generated files might not exist before the build script runs.
- **Constraints / authority / confidence / source:** Language and standard built-in macro behavior; **high**. Verify in standard-library built-in macro documentation and the Rust Reference on macros.

### RUST-PROJ-012 — Procedural macro crate role

- **Distinct behavior:** A `proc-macro` crate is compiled for the host, exports only procedural macro entry points under special rules, and its generated tokens become source for a target crate.
- **Observable content:** Function-like, derive, and attribute procedural macros used from another crate, with helper attributes and generated referenced items.
- **Variants and failures:** Panics and emitted `compile_error!`; token spans affect diagnostics and hygiene; proc-macro dependency cycles are rejected; proc-macro API is implementation-distributed standard library.
- **Constraints / authority / confidence / source:** Language/Cargo special crate type and standard `proc_macro` API; **high**. Verify in Rust Reference procedural macro and Cargo target documentation.

### RUST-PROJ-013 — Crate types and linker-facing artifacts

- **Distinct behavior:** `rlib`, `dylib`, `cdylib`, `staticlib`, `proc-macro`, and executable crate types produce different linkage boundaries and exported metadata.
- **Observable content:** A library target built under several compatible crate types, including a foreign-consumable artifact with explicit exported symbols.
- **Variants and failures:** Platform linker support; mixing crate types; Rust ABI not stable across compiler versions; `cdylib` omits ordinary Rust dependency metadata.
- **Constraints / authority / confidence / source:** `rustc` and Cargo-defined; **high**. Verify in Rust Reference linkage material and `rustc` crate-type documentation.

### RUST-PROJ-014 — Profiles, panic strategy, and code-generation configuration

- **Distinct behavior:** Profile settings change overflow checks, debug assertions, panic strategy, Link Time Optimization (LTO), codegen units, symbols, and optimization without changing most source.
- **Observable content:** Custom development/release profiles and package overrides, with `cfg(debug_assertions)` and overflow- or panic-sensitive behavior.
- **Variants and failures:** Test/bench profile restrictions; dependencies can have different optimization; `panic = "abort"` affects unwinding and some target/test combinations; profile inheritance.
- **Constraints / authority / confidence / source:** Cargo and `rustc` implementation behavior; **high**. Verify in Cargo profiles documentation and `rustc` code-generation options.

### RUST-PROJ-015 — Edition per crate

- **Distinct behavior:** Edition changes parsing, name resolution, prelude contents, lint migrations, and some inferred behavior; interacting crates may use different editions without changing Application Binary Interface (ABI) identity by edition alone.
- **Observable content:** Workspace crates on 2018, 2021, and 2024 editions that call each other and use edition-sensitive syntax or prelude names.
- **Variants and failures:** Keywords accepted as raw identifiers; array `IntoIterator`; macro fragment behavior; unsafe-operation diagnostics; missing edition defaults in old manifests.
- **Constraints / authority / confidence / source:** Language/Cargo-defined; **high**. Verify in Rust Edition Guide, Rust Reference editions appendix, and Cargo manifest documentation.

### RUST-PROJ-016 — `no_std`, `no_core`, and runtime entry contracts

- **Distinct behavior:** `#![no_std]` replaces the standard prelude/runtime dependency with `core` and optionally `alloc`; `#![no_core]` and custom language items are unstable; custom targets may require panic handlers and entry symbols.
- **Observable content:** A `no_std` library, an allocator-using target, and a freestanding binary with conditional panic handler and entry point.
- **Variants and failures:** Test harness normally needs `std`; allocator and unwinding availability; duplicate panic handlers; `no_main`; nightly gates for deepest runtime customization.
- **Constraints / authority / confidence / source:** Language and target/runtime implementation; **high** for `no_std`, **medium** for unstable/custom-target edges. Verify in Rust Reference crate attributes and `rustc` platform documentation.

## Modules, paths, namespaces, and visibility

### RUST-MOD-001 — Module declaration and source-file mapping

- **Distinct behavior:** Inline and out-of-line modules create identical namespace kinds but derive source paths differently; `#[path]` overrides discovery.
- **Observable content:** Inline module, modern `name.rs` plus `name/sub.rs`, legacy `name/mod.rs`, and `#[path]` module.
- **Variants and failures:** Duplicate candidate files are errors in relevant layouts; path base differs for inline/non-module files; a file is not a module unless declared.
- **Constraints / authority / confidence / source:** Language plus `rustc` file-loading rules; **high**. Verify in Rust Reference module source filenames material.

### RUST-MOD-002 — Separate namespaces

- **Distinct behavior:** Rust resolves names in type, value, macro, lifetime, and label namespaces, permitting some same-spelled entities while rejecting collisions within one namespace.
- **Observable content:** Unit-like struct or tuple struct constructors, constants, modules, traits, macros, lifetimes, and labels sharing selected spellings.
- **Variants and failures:** Struct constructors occupy value namespace; named-field structs do not create a callable constructor; imports can introduce namespace-specific ambiguity.
- **Constraints / authority / confidence / source:** Language-defined; **high**. Verify in Rust Reference namespaces appendix.

### RUST-MOD-003 — Absolute, crate-relative, self, and super paths

- **Distinct behavior:** `crate`, `self`, `super`, leading `::`, and extern-prelude roots select different path anchors, with edition-sensitive leading path rules.
- **Observable content:** Equivalent and deliberately shadowed paths from nested modules, including an external crate whose name matches a root module.
- **Variants and failures:** Too many `super` segments; `::name` extern-prelude behavior in 2018+; `self::` versus lexical item resolution.
- **Constraints / authority / confidence / source:** Language/edition-defined; **high**. Verify in Rust Reference path expressions and use declarations.

### RUST-MOD-004 — `use` imports, groups, aliases, and underscore imports

- **Distinct behavior:** Imports create bindings rather than module ownership, may import multiple namespaces, and support aliases, nested groups, `self`, and `_` trait-only imports.
- **Observable content:** Grouped imports, rename, re-import of module `self`, glob, and `Trait as _` enabling method lookup without a nameable binding.
- **Variants and failures:** Duplicate bindings; imports are order-independent; `use` paths have special resolution; underscore import allowed for traits and macros in relevant contexts.
- **Constraints / authority / confidence / source:** Language-defined; **high**. Verify in Rust Reference use declarations.

### RUST-MOD-005 — Glob imports and ambiguity

- **Distinct behavior:** Glob imports lazily introduce visible names, may be shadowed by explicit/local bindings, and can remain ambiguous until a name is used.
- **Observable content:** Two globs exporting same-spelled items, one unused and one referenced, plus an explicit import resolving another collision.
- **Variants and failures:** Public glob re-export changes downstream API; enum variant globs; macro namespace; future upstream additions can break consumers.
- **Constraints / authority / confidence / source:** Language-defined; **high**. Verify in Rust Reference name resolution and use declarations.

### RUST-MOD-006 — Re-exports and canonical public paths

- **Distinct behavior:** `pub use` exposes an existing entity through a new public path without changing its identity, subject to the original item's effective visibility.
- **Observable content:** Private implementation module containing public items re-exported at crate root, including renamed and glob re-exports.
- **Variants and failures:** Re-exporting insufficiently visible items; downstream path stability despite internal location; duplicate public paths; re-exported macros.
- **Constraints / authority / confidence / source:** Language-defined; **high**. Verify in Rust Reference visibility and use declarations.

### RUST-MOD-007 — Restricted visibility

- **Distinct behavior:** `pub`, private, `pub(crate)`, `pub(super)`, `pub(self)`, and `pub(in path)` form visibility boundaries tied to ancestor modules.
- **Observable content:** Nested modules with each restriction and accesses from child, sibling, parent, same crate, and downstream crate.
- **Variants and failures:** `pub(in)` path must resolve to an ancestor and uses edition-sensitive path syntax; public item containing private types triggers lints/errors by position; fields have independent visibility.
- **Constraints / authority / confidence / source:** Language-defined; **high**. Verify in Rust Reference visibility and privacy.

### RUST-MOD-008 — Effective visibility and private-in-public reachability

- **Distinct behavior:** Nominal `pub` is capped by containing module reachability; public interfaces may refer to hidden types in some opaque or associated positions but not leak unusable private names.
- **Observable content:** Public items under private modules, re-exported items, public signatures involving aliases or associated types, and deliberate privacy diagnostics.
- **Variants and failures:** Lints versus hard errors have evolved; reachable but unnameable types; documentation visibility differs from semantic reachability.
- **Constraints / authority / confidence / source:** Language with compiler diagnostics; **medium-high**. Verify in Rust Reference visibility and `rustc` privacy lint documentation.

### RUST-MOD-009 — Extern prelude, `extern crate`, and aliases

- **Distinct behavior:** Cargo-provided direct dependencies populate the extern prelude in modern editions; `extern crate` remains meaningful for aliases, macro import legacy, sysroot crates, and special self aliases.
- **Observable content:** Modern implicit dependency path, `extern crate package as alias`, `extern crate self as public_name`, and edition-2015 macro import.
- **Variants and failures:** Transitive dependencies are not automatically direct extern-prelude entries; `--extern` controls actual availability; alias conflicts.
- **Constraints / authority / confidence / source:** Language/edition plus Cargo invocation; **high**. Verify in Rust Reference external crates/preludes and Edition Guide.

### RUST-MOD-010 — Standard, external, and language preludes

- **Distinct behavior:** Every module receives implicit prelude names and attributes; standard prelude contents depend on edition and `no_std`; explicit shadowing is allowed.
- **Observable content:** Unqualified prelude traits/types, an edition-specific prelude addition, a shadowing local item, and `#![no_implicit_prelude]` in a controlled module/crate.
- **Variants and failures:** `std` versus `core` prelude; extern prelude and macro-use prelude are separate; derive and tool attributes have distinct resolution.
- **Constraints / authority / confidence / source:** Language and standard-library-defined; **high**. Verify in Rust Reference preludes and standard library prelude modules.

### RUST-MOD-011 — Item shadowing, scopes, and forward references

- **Distinct behavior:** Item declarations generally scope over an entire module/block, while local bindings begin after declaration and can shadow previous bindings; generic parameters and labels have their own scopes.
- **Observable content:** Function called before textual declaration, nested item, repeated `let` shadowing with changed type, and illegal capture by an inner function item.
- **Variants and failures:** Block items cannot capture dynamic environment; closure can; pattern-binding scopes in guards/arms; shadowing differs from mutation.
- **Constraints / authority / confidence / source:** Language-defined; **high**. Verify in Rust Reference scopes and name resolution.

### RUST-MOD-012 — Associated-item and method path resolution

- **Distinct behavior:** `Type::item`, `<Type as Trait>::item`, method-call syntax, and value-namespace constructors use different candidate searches and can require fully qualified syntax.
- **Observable content:** Inherent and trait items with same name, ambiguous associated constants, and Universal Function Call Syntax (UFCS) disambiguation.
- **Variants and failures:** Trait must be in scope for method calls in many cases; type-relative paths may traverse associated types; receiver autoderef changes candidates.
- **Constraints / authority / confidence / source:** Language-defined with `rustc` resolution details; **high**. Verify in Rust Reference paths and method-call expressions.

## Item kinds, data types, and representation

### RUST-TYPE-001 — Struct forms and constructor identity

- **Distinct behavior:** Named-field, tuple, and unit structs create types; tuple/unit forms also create value-namespace constructors, while field visibility and update syntax affect construction rights.
- **Observable content:** All three forms, private/public fields, destructuring, functional update syntax, and constructor used as a function value.
- **Variants and failures:** Update moves or copies remaining fields; private fields block external construction and update; zero-field braced struct differs syntactically from unit struct.
- **Constraints / authority / confidence / source:** Language-defined; **high**. Verify in Rust Reference struct items and expressions.

### RUST-TYPE-002 — Enums, variants, and discriminants

- **Distinct behavior:** Variants are constructors in the enum namespace/path, can have unit/tuple/struct payloads, and may have explicit or computed discriminants under representation rules.
- **Observable content:** Mixed variant forms, explicit discriminants, exhaustive matching, variant imports, and discriminant observation.
- **Variants and failures:** Duplicate discriminants can be allowed; overflow rejected; data-bearing explicit discriminants require permitted representation conditions; layout is otherwise not a C contract.
- **Constraints / authority / confidence / source:** Language-defined with layout qualifications; **high**. Verify in Rust Reference enum items and type layout.

### RUST-TYPE-003 — Unions

- **Distinct behavior:** Union fields overlap storage; field reads are unsafe, writes are generally safe, destruction and allowed field types follow special rules, and active-field state is programmer-maintained.
- **Observable content:** Union with `Copy` fields and `ManuallyDrop` non-`Copy` field, safe write, unsafe read, and pattern restrictions.
- **Variants and failures:** Invalid bit patterns create Undefined Behavior (UB); union fields must not require ordinary drop; representation attributes alter foreign layout.
- **Constraints / authority / confidence / source:** Language-defined, safety details partly implementation/unsafe-guideline territory; **high**. Verify in Rust Reference union items and Rustonomicon.

### RUST-TYPE-004 — Type aliases versus new nominal types

- **Distinct behavior:** A type alias introduces another name for the same type, while tuple structs and enums create distinct nominal identities; aliases cannot provide new inherent implementations.
- **Observable content:** Alias and single-field tuple struct wrapping the same underlying type, with assignment/coercion and trait-implementation contrasts.
- **Variants and failures:** Generic and associated type aliases; recursive alias rejection; alias constructors are limited and path/visibility-sensitive; unstable inherent associated types.
- **Constraints / authority / confidence / source:** Language-defined; **high**. Verify in Rust Reference type aliases and struct items.

### RUST-TYPE-005 — Primitive, tuple, array, slice, and never types

- **Distinct behavior:** Built-in types have special syntax and operations; arrays carry length in the type, slices are dynamically sized, tuples are structural, and `!` coerces from diverging expressions.
- **Observable content:** Array lengths from const expressions, array-to-slice coercion, one-element tuple syntax, empty tuple, and a diverging branch joining another type.
- **Variants and failures:** Unsupported array lengths for certain trait implementations on older compilers; never type remains special in some generic positions; indexing can panic.
- **Constraints / authority / confidence / source:** Language-defined plus standard trait implementations; **high**. Verify in Rust Reference types and expressions.

### RUST-TYPE-006 — References, raw pointers, and mutability

- **Distinct behavior:** Shared and mutable references carry validity, aliasing, lifetime, and alignment requirements; raw pointers relax borrowing guarantees but require unsafe dereference.
- **Observable content:** Reborrows, raw pointer formation from both reference kinds, address-of raw syntax, unsafe dereference, and null/misaligned pointer APIs.
- **Variants and failures:** Creating a reference to invalid or unaligned data is UB even if not read; raw-pointer provenance and aliasing details are not fully specified; `&raw` avoids intermediate invalid references.
- **Constraints / authority / confidence / source:** Core language with implementation/unsafe-model gaps; **high** for syntax, **medium** for provenance. Verify in Rust Reference pointer/reference types, Rustonomicon, and Unsafe Code Guidelines project material.

### RUST-TYPE-007 — Function items, function pointers, and closures

- **Distinct behavior:** Each function definition and monomorphization has a unique zero-sized function-item type; it can coerce to an `fn` pointer, while capturing and noncapturing closures have anonymous distinct types.
- **Observable content:** Two function items of same signature, explicit `fn` coercion, noncapturing closure coercion, capturing closure, and calls through each.
- **Variants and failures:** `unsafe fn`, `extern ABI fn`, higher-ranked function pointers; incompatible item types can coerce at a join; closure does not coerce if it captures.
- **Constraints / authority / confidence / source:** Language-defined; **high**. Verify in Rust Reference function item, function pointer, and closure types.

### RUST-TYPE-008 — Dynamically sized types and `?Sized`

- **Distinct behavior:** Slices, `str`, and trait objects are Dynamically Sized Types (DSTs); most generic type parameters implicitly require `Sized`, and a DST normally appears only behind a wide pointer or as the last struct field.
- **Observable content:** `T: ?Sized` generic, custom trailing-slice struct, sized and unsized coercions, and invalid by-value/local DST positions.
- **Variants and failures:** `Self: ?Sized` for traits; metadata differs for slices and trait objects; `extern type` is unstable; arbitrary self types remain constrained.
- **Constraints / authority / confidence / source:** Language-defined, custom DST construction may need unsafe code; **high**. Verify in Rust Reference dynamically sized types and type parameters.

### RUST-TYPE-009 — Trait objects and object safety/dyn compatibility

- **Distinct behavior:** `dyn Trait` erases concrete type behind data and virtual-table metadata and only accepts traits satisfying dyn-compatibility rules.
- **Observable content:** Object-safe trait with supertraits, dynamic call, associated type binding, lifetime bound, and a deliberately non-dyn-compatible trait.
- **Variants and failures:** Generic methods, `Self` in disallowed positions, associated constants, and `Self: Sized` exemptions; auto-trait and lifetime bounds form the object type identity.
- **Constraints / authority / confidence / source:** Language-defined, vtable layout implementation-defined; **high**. Verify in Rust Reference trait objects and dyn compatibility.

### RUST-TYPE-010 — `impl Trait` in argument and return position

- **Distinct behavior:** Argument-position `impl Trait` behaves like an anonymous generic parameter for callers, while return-position `impl Trait` denotes one hidden concrete type chosen by the defining function.
- **Observable content:** Both positions, multiple return branches with same hidden type, nested use, and caller turbofish contrast with named generics.
- **Variants and failures:** Captured generic/lifetime parameters depend on edition and precise-capture syntax; distinct opaque definitions have distinct identity; incompatible return branch types fail.
- **Constraints / authority / confidence / source:** Language-defined with recent edition changes; **high**. Verify in Rust Reference `impl Trait` and Edition Guide 2024 capture material.

### RUST-TYPE-011 — Return-position `impl Trait` in traits

- **Distinct behavior:** Trait methods may return opaque types whose concrete type is selected by each implementation, conceptually involving anonymous associated types and limiting dyn use.
- **Observable content:** Trait method returning `impl Iterator`, two implementations with different hidden types, generic use, and attempted trait-object use.
- **Variants and failures:** `async fn` in traits uses related opaque return semantics; auto-trait bounds become public commitments; dyn compatibility commonly fails for such methods unless excluded with `Self: Sized`.
- **Constraints / authority / confidence / source:** Stable language feature; **high**. Verify in Rust Reference traits/`impl Trait` and accepted RFC material.

### RUST-TYPE-012 — Recursive types and indirection

- **Distinct behavior:** Directly recursive value types have infinite size and are rejected, while recursion through pointer-like indirection is sized; recursive aliases and opaque cycles have separate checks.
- **Observable content:** Valid boxed recursive enum and invalid direct recursive struct or alias cycle.
- **Variants and failures:** Mutual recursion; DST recursion; drop-check cycles; layout cycle diagnostics.
- **Constraints / authority / confidence / source:** Language/compiler-defined; **high**. Verify in Rust Reference type layout and `rustc` diagnostics.

### RUST-TYPE-013 — Representation attributes and layout

- **Distinct behavior:** Default Rust representation permits field reordering and other implementation choices; `repr(C)`, integer enum reprs, `transparent`, `packed`, and `align` impose selected layout guarantees.
- **Observable content:** Equivalent structures under each representation, field offsets/size assertions, and foreign declarations consuming C-compatible forms.
- **Variants and failures:** Invalid attribute combinations; `packed` unaligned-reference hazards; `transparent` eligibility; niche optimizations remain outside most guarantees.
- **Constraints / authority / confidence / source:** Language-defined guarantees plus target ABI; **high**. Verify in Rust Reference type layout and representation attributes.

### RUST-TYPE-014 — Type inference, coercion, and coercion sites

- **Distinct behavior:** Inference variables are constrained across expressions; coercions occur only at designated sites and may propagate through arrays, tuples, blocks, and branches.
- **Observable content:** Function item to pointer, noncapturing closure to pointer, mut-to-shared reference, deref coercion, unsizing, and least-upper-bound branch coercion.
- **Variants and failures:** Explicit type annotation changes inference; numeric fallback; method receiver adjustments are related but distinct; no implicit user-defined conversion.
- **Constraints / authority / confidence / source:** Language-defined with compiler inference behavior; **high**. Verify in Rust Reference type coercions and inference.

### RUST-TYPE-015 — Cast expressions

- **Distinct behavior:** `as` supports a closed set of numeric, pointer, function, enum-discriminant, and trait-object related casts rather than general conversion.
- **Observable content:** Widening/truncating numeric casts, enum-to-integer, pointer conversions, function-pointer cast, and rejected invalid cast.
- **Variants and failures:** Float-to-int saturating behavior in current Rust; pointer-to-integer provenance concerns; fat-to-thin pointer cast drops metadata; unsafe transmutation is distinct.
- **Constraints / authority / confidence / source:** Language-defined with pointer-model caveats; **high**. Verify in Rust Reference cast expressions.

## Generics, lifetimes, and associated entities

### RUST-GEN-001 — Generic parameters and defaults

- **Distinct behavior:** Items can bind lifetime, type, and const parameters; defaults are permitted only in selected item contexts and parameter ordering/default rules constrain declarations.
- **Observable content:** Struct, enum, function, trait, implementation, and alias using all parameter kinds, with type/const defaults where legal.
- **Variants and failures:** Late- versus early-bound lifetimes; unused generic parameters can be rejected for types; defaults in functions are not generally allowed; inference may select defaults.
- **Constraints / authority / confidence / source:** Language-defined; **high**. Verify in Rust Reference generic parameters.

### RUST-GEN-002 — Bounds, `where` clauses, and implied bounds

- **Distinct behavior:** Bounds establish well-formedness and available operations; some lifetime and trait bounds are implied by type well-formedness, while most trait bounds must be stated.
- **Observable content:** Equivalent inline/where bounds, higher-order associated constraints, an implied outlives use, and a missing non-implied trait bound error.
- **Variants and failures:** Trivial bounds checked at definition; generic bounds checked at use/monomorphization stages; `?Sized` only relaxes the implicit bound.
- **Constraints / authority / confidence / source:** Language-defined; **high**. Verify in Rust Reference trait and lifetime bounds.

### RUST-GEN-003 — Const generics and evaluatability

- **Distinct behavior:** Const values can participate in type identity and array lengths; stable const parameter types and allowed generic const expressions are restricted.
- **Observable content:** Type and function parameterized by integer/bool/char const, distinct instantiations, const argument inference, and an expression rejected without unstable generic-const support.
- **Variants and failures:** Braces around nontrivial const arguments; standalone parameter restrictions in types; const equality; nightly `generic_const_exprs` behavior.
- **Constraints / authority / confidence / source:** Language-defined with substantial unstable frontier; **high** for minimal const generics, **medium** for frontier. Verify in Rust Reference const generics and unstable feature tracking.

### RUST-GEN-004 — Associated types and equality constraints

- **Distinct behavior:** Traits and implementations bind associated types selected through projection; equality/trait bounds on projections drive normalization and resolution.
- **Observable content:** Trait with associated type and bound, implementation selecting it, `<T as Trait>::Assoc`, shorthand `T::Assoc`, and constrained generic consumer.
- **Variants and failures:** Ambiguous shorthand; projection cycles/overflow; associated type defaults remain unstable; trait-object bindings.
- **Constraints / authority / confidence / source:** Language-defined; **high**. Verify in Rust Reference associated items and paths.

### RUST-GEN-005 — Generic associated types

- **Distinct behavior:** An associated type can itself bind lifetimes/types/consts, enabling families of projected types tied to a receiver.
- **Observable content:** Lending-style trait with lifetime-parameterized associated type, required `where Self: 'a`, and an implementation returning a borrow.
- **Variants and failures:** Required bounds inferred by current rules; higher-ranked projection bounds; dyn compatibility; borrow-checker limitations and solver evolution.
- **Constraints / authority / confidence / source:** Language-defined, compiler limitations possible; **high**. Verify in Rust Reference associated types and generic associated type stabilization material.

### RUST-GEN-006 — Associated constants

- **Distinct behavior:** Traits and inherent implementations define constants selected through type/trait resolution, with defaults and implementation overrides.
- **Observable content:** Same-spelled inherent and trait associated constants, defaulted trait constant, override, and fully qualified disambiguation.
- **Variants and failures:** Const expressions referring to `Self`; ambiguous `Type::CONST`; dyn incompatibility of traits with associated constants.
- **Constraints / authority / confidence / source:** Language-defined; **high**. Verify in Rust Reference associated items and const items.

### RUST-GEN-007 — Lifetime elision and placeholder lifetimes

- **Distinct behavior:** Elision rules synthesize input/output lifetimes in function and trait-object types; `'_` requests inference without creating a named parameter.
- **Observable content:** Free function, method receiver elision, ambiguous multi-input output error, path with `'_`, and omitted trait-object lifetime.
- **Variants and failures:** Function item versus closure inference; trait-object default lifetime bounds have separate rules; edition idiom lints for hidden lifetimes.
- **Constraints / authority / confidence / source:** Language-defined; **high**. Verify in Rust Reference lifetime elision.

### RUST-GEN-008 — Higher-ranked trait and lifetime bounds

- **Distinct behavior:** `for<'a>` quantifies a bound for every lifetime rather than one selected outside the bound, affecting function pointers, closures, and trait implementations.
- **Observable content:** Callback accepted for any borrowed input, contrasted with a bound tied to one outer lifetime, using both where-clause and bare function syntax.
- **Variants and failures:** Binder placement changes scope; late-bound lifetime instantiation; higher-ranked subtyping and leak-check diagnostics.
- **Constraints / authority / confidence / source:** Language-defined; **high**. Verify in Rust Reference higher-ranked trait bounds and function pointer types.

### RUST-GEN-009 — Variance and subtyping

- **Distinct behavior:** Lifetimes and generic parameters may be covariant, contravariant, or invariant based on their use; this changes legal lifetime shortening and assignment.
- **Observable content:** Shared reference, mutable reference, function argument, and interior-mutability marker examples with accepted/rejected substitutions.
- **Variants and failures:** Raw pointers; `PhantomData` can intentionally set ownership/variance; type parameters otherwise have nominal inferred variance.
- **Constraints / authority / confidence / source:** Language-defined, often explained in Rustonomicon; **high**. Verify in Rust Reference subtyping/variance and Rustonomicon.

### RUST-GEN-010 — Opaque type capture and precise capture

- **Distinct behavior:** Return-position opaque types capture in-scope generic parameters under edition-sensitive defaults; `use<...>` can state an exact capture set where supported.
- **Observable content:** Equivalent functions compiled under 2021 and 2024 capture rules, plus an explicit precise-capture bound and a borrow that demonstrates hidden lifetime capture.
- **Variants and failures:** Capturing lifetimes/types/consts; restrictions for trait methods and nested opaque types; feature support changed recently.
- **Constraints / authority / confidence / source:** Language/edition-defined; **medium-high** on precise-capture edge restrictions. Verify in Rust Reference `impl Trait` and Edition Guide 2024.

## Traits, implementations, and dispatch

### RUST-TRAIT-001 — Inherent versus trait implementations

- **Distinct behavior:** Inherent implementations attach items directly to a nominal type; trait implementations satisfy a separate contract and participate in trait resolution.
- **Observable content:** Same type with multiple inherent impl blocks, one trait impl, same-spelled method, and calls using method and qualified syntax.
- **Variants and failures:** Inherent impl only in defining crate for nominal type; primitive-type inherent impl restrictions; duplicate inherent item names overlap globally.
- **Constraints / authority / confidence / source:** Language-defined; **high**. Verify in Rust Reference implementations.

### RUST-TRAIT-002 — Coherence and orphan rules

- **Distinct behavior:** Trait implementations must not overlap and generally require either the trait or a sufficiently uncovered nominal type to be local, preserving global uniqueness.
- **Observable content:** Legal local-trait/foreign-type and foreign-trait/local-type impls, covered generic wrapper, and rejected foreign-for-foreign or overlapping impl.
- **Variants and failures:** Fundamental types affect coverage; order of type parameters matters; negative reasoning is limited; upstream additions create compatibility hazards.
- **Constraints / authority / confidence / source:** Language-defined with compiler coherence checking; **high**. Verify in Rust Reference trait implementation coherence.

### RUST-TRAIT-003 — Blanket and conditional implementations

- **Distinct behavior:** An impl for all types satisfying bounds creates potentially broad method/trait availability and can overlap with seemingly specific future impls.
- **Observable content:** Blanket extension trait impl, conditional impl on generic wrapper, and a deliberate overlap diagnostic.
- **Variants and failures:** Semver implications; auto-ref receiver can make blanket methods candidates; downstream impl prevention via coherence.
- **Constraints / authority / confidence / source:** Language-defined; **high**. Verify in Rust Reference implementations and coherence.

### RUST-TRAIT-004 — Supertraits and implied trait relationships

- **Distinct behavior:** Implementing a subtrait requires its supertrait obligations, and methods/associated items from supertraits are available through bounded values.
- **Observable content:** Trait hierarchy with inherited method call, missing supertrait impl failure, and a trait object combining permitted supertraits.
- **Variants and failures:** Cycles; associated type constraints on supertraits; supertrait does not automatically create a reverse blanket impl.
- **Constraints / authority / confidence / source:** Language-defined; **high**. Verify in Rust Reference supertraits.

### RUST-TRAIT-005 — Default trait items and overrides

- **Distinct behavior:** Trait methods, associated constants, and eligible associated definitions can supply defaults inherited unless an impl overrides them.
- **Observable content:** Default method calling a required method, one inherited and one overridden implementation, and direct qualified calls.
- **Variants and failures:** Default implementation still checked under trait bounds only; no stable specialization between overlapping impls; object dispatch selects implementation override.
- **Constraints / authority / confidence / source:** Language-defined; **high**. Verify in Rust Reference traits and associated items.

### RUST-TRAIT-006 — Receiver forms and arbitrary self boundaries

- **Distinct behavior:** Methods use specific `self` receiver forms that influence autoderef, borrowing, ownership, and dyn dispatch; only supported receiver shapes are stable.
- **Observable content:** `self`, `&self`, `&mut self`, `Box<Self>`, `Pin<&mut Self>`, and associated function without receiver.
- **Variants and failures:** Receiver aliases and arbitrary self types have version/feature restrictions; consuming object methods may require `Self: Sized`; method-call candidate construction.
- **Constraints / authority / confidence / source:** Language-defined with unstable extensions; **high** for common receivers, **medium** for boundary. Verify in Rust Reference associated functions/methods.

### RUST-TRAIT-007 — Method lookup, autoderef, and autoref

- **Distinct behavior:** Method-call resolution builds candidate receiver types through dereference and unsizing, then searches inherent and in-scope trait methods with automatic borrowing.
- **Observable content:** Custom `Deref`, methods at multiple deref levels, trait/inherent name collision, mutable/shared candidates, and fully qualified disambiguation.
- **Variants and failures:** Candidate ordering can select `&self` trait method before a later inherent `&mut self`; edition-sensitive array `IntoIterator`; ambiguous trait methods.
- **Constraints / authority / confidence / source:** Language-defined algorithm with compiler details; **high**. Verify in Rust Reference method-call expressions.

### RUST-TRAIT-008 — Static versus dynamic dispatch

- **Distinct behavior:** Generic trait bounds are normally monomorphized and statically dispatched; trait objects use runtime dispatch and erase concrete associated implementation identity.
- **Observable content:** Same operation through a generic parameter and `dyn Trait`, with concrete implementations and object conversion.
- **Variants and failures:** Codegen may devirtualize without changing semantics; dyn-compatible methods only; vtable layout not specified; `impl Trait` remains static.
- **Constraints / authority / confidence / source:** Language semantics plus implementation codegen; **high**. Verify in Rust Reference trait objects and monomorphization documentation.

### RUST-TRAIT-009 — Auto traits

- **Distinct behavior:** Auto traits such as `Send`, `Sync`, `Unpin`, unwind-safety traits, and user-defined unstable auto traits are inferred structurally unless an explicit permitted impl changes the result.
- **Observable content:** Composite types that do and do not inherit `Send`/`Sync`, generic assertions, and a marker field changing inference.
- **Variants and failures:** Raw pointers and interior mutability affect results; dyn object auto-trait set is part of its type; user auto traits and negative impls are unstable except compiler/library cases.
- **Constraints / authority / confidence / source:** Language plus standard-library trait definitions; **high** for standard traits, **medium** for unstable custom cases. Verify in Rust Reference special traits and standard-library docs.

### RUST-TRAIT-010 — Negative implementations and negative reasoning

- **Distinct behavior:** Negative impls state that a type will not implement a trait and affect coherence/auto-trait inference; general use remains feature-gated.
- **Observable content:** Standard-library-observable negative property and, in a nightly-only case, explicit `impl !Trait for Type` contrasted with a conflicting positive impl.
- **Variants and failures:** Semver commitment; auto-trait negative impl differs from suppressing via fields; stable compiler accepts only limited built-in/library situations.
- **Constraints / authority / confidence / source:** Unstable language feature; **medium-high**. Verify in unstable Rust Reference material and feature tracking.

### RUST-TRAIT-011 — Operator overloading and place/value semantics

- **Distinct behavior:** Operators map to language-item traits with fixed syntax and evaluation rules; assignment operators, comparison, indexing, dereference, calls, and `?` have specialized contracts.
- **Observable content:** User type implementing arithmetic, assignment, comparison, `Index`/`IndexMut`, `Deref`, and one callable-related counterexample noting stable limits.
- **Variants and failures:** Operators do not all auto-borrow identically; output type may differ; short-circuit boolean operators are not overloadable; `Deref` affects method lookup and coercion.
- **Constraints / authority / confidence / source:** Language plus standard `ops` traits; **high**. Verify in Rust Reference operator expressions and standard-library operator trait docs.

### RUST-TRAIT-012 — `From`/`Into`, `TryFrom`, and explicit conversion conventions

- **Distinct behavior:** Common ecosystem and standard-library conversion traits provide explicit, trait-resolved conversion edges but are not language coercions.
- **Observable content:** User types with `From`, inferred reciprocal `Into`, fallible conversion, and a generic bound contrasted with an invalid implicit assignment.
- **Variants and failures:** Blanket impl interactions and coherence; conversion error associated type; `as` casts remain separate.
- **Constraints / authority / confidence / source:** Standard-library convention with language trait resolution; **high**. Verify in standard-library conversion trait documentation.

### RUST-TRAIT-013 — `Drop`, drop glue, and explicit destructor restrictions

- **Distinct behavior:** Types implementing `Drop` run destructor logic at compiler-selected drop points; compiler-generated drop glue recursively drops fields, and `Drop::drop` cannot be called directly.
- **Observable content:** Nested fields with observable drop order, early `drop` function call, moved-out value, partial initialization, and a direct destructor-call error.
- **Variants and failures:** Panic during unwinding; no drop after `mem::forget` or process abort; temporary scope changes; `ManuallyDrop`; `Copy` and `Drop` are mutually exclusive.
- **Constraints / authority / confidence / source:** Language plus standard library; **high**. Verify in Rust Reference destructors and standard `Drop` documentation.

### RUST-TRAIT-014 — `Copy` and clone semantics

- **Distinct behavior:** `Copy` changes ordinary use from move to bitwise implicit copy and is restricted to types without destructors whose fields are all `Copy`; `Clone` is explicit and may differ semantically.
- **Observable content:** Copy scalar/struct, non-Copy owning type, manual Clone behavior, use-after-assignment contrast, and illegal Copy implementation.
- **Variants and failures:** Derive adds generic bounds that manual impl may avoid; shared references are Copy, mutable references are not; union behavior.
- **Constraints / authority / confidence / source:** Language special trait plus standard library; **high**. Verify in Rust Reference special traits and standard `Copy`/`Clone` docs.

### RUST-TRAIT-015 — `Sized` and unsizing traits

- **Distinct behavior:** `Sized` is implicit on generic types and specially recognized; compiler-controlled `Unsize`, `CoerceUnsized`, and `DispatchFromDyn` govern supported wide-pointer conversions.
- **Observable content:** `?Sized` API accepting slice/trait object and stable built-in coercions, plus a rejected user implementation of compiler-controlled traits.
- **Variants and failures:** Struct-tail unsizing; smart-pointer coercions supplied by standard types; custom coercions are unstable.
- **Constraints / authority / confidence / source:** Language and implementation special traits; **high**. Verify in Rust Reference special types/coercions and standard marker/ops docs.

### RUST-TRAIT-016 — Trait aliases and specialization boundary

- **Distinct behavior:** Trait aliases and specialization would alter bound identity and impl selection but remain unstable; stable code must use ordinary supertraits/blanket impls and non-overlapping impls.
- **Observable content:** Nightly-gated minimal alias and specialized impl paired with stable formulations showing different entity/selection behavior.
- **Variants and failures:** Incomplete specialization soundness/eligibility rules; feature-gate diagnostics; alias is not a new implementable trait in the ordinary sense.
- **Constraints / authority / confidence / source:** Unstable language features; **medium**. Verify in unstable Rust Reference and feature tracking.

## Ownership, borrowing, patterns, and destruction

### RUST-OWN-001 — Move paths and partial moves

- **Distinct behavior:** Non-`Copy` places are moved by value; fields can be moved independently, leaving only unaffected paths usable, subject to destructor restrictions.
- **Observable content:** Struct destructuring that moves one field and borrows/copies another, then accesses remaining fields and attempts whole-value use.
- **Variants and failures:** Types implementing `Drop` restrict field moves; moves through dereference; moves captured by closures; conditional initialization.
- **Constraints / authority / confidence / source:** Language-defined via ownership semantics and compiler analysis; **high**. Verify in Rust Reference value expressions and destructors.

### RUST-OWN-002 — Shared and mutable borrow exclusivity

- **Distinct behavior:** A live mutable borrow excludes other accesses and a live shared borrow excludes mutation, with lifetimes inferred from actual use rather than lexical block alone.
- **Observable content:** Accepted non-overlapping reborrows, conflicting simultaneous borrows, and mutation after last shared use.
- **Variants and failures:** Two-phase borrows in method calls; interior mutability moves checks to runtime; unsafe aliasing can compile but be UB.
- **Constraints / authority / confidence / source:** Language safety contract plus borrow-checker implementation; **high**. Verify in Rust Reference borrow expressions and Rustonomicon aliasing.

### RUST-OWN-003 — Non-lexical lifetimes

- **Distinct behavior:** Borrow lifetime ends according to control-flow use rather than necessarily at enclosing lexical scope, permitting later conflicting access.
- **Observable content:** Borrow used and then superseded by mutation in the same block, plus a branch/loop case where borrow remains live.
- **Variants and failures:** Drop scope remains lexical in many cases; Polonius is an alternative/experimental analysis; diagnostics may vary across compiler versions.
- **Constraints / authority / confidence / source:** `rustc` implementation of language borrow rules; **high**. Verify in edition/non-lexical lifetime material and compiler documentation.

### RUST-OWN-004 — Two-phase borrows

- **Distinct behavior:** Certain compiler-inserted mutable autoref borrows have a reservation phase allowing selected shared evaluation before activation.
- **Observable content:** Method call like mutating a collection using its length in an argument, contrasted with an explicit `&mut` borrow that conflicts.
- **Variants and failures:** Applies only to specific implicit borrows and operators; nested calls and activation points; not a general relaxation of aliasing.
- **Constraints / authority / confidence / source:** `rustc` borrow-checker behavior supporting language expressions; **medium-high**. Verify in compiler borrow-check documentation.

### RUST-OWN-005 — Temporary lifetime and drop scopes

- **Distinct behavior:** Temporaries are dropped at syntactically determined scopes, with lifetime extension in selected `let` and constant/static contexts; edition 2024 changed some temporary scopes.
- **Observable content:** Borrow from a temporary in extending and non-extending patterns, temporaries in `if let`/tail expressions, and destructor order markers.
- **Variants and failures:** Match scrutinee lifetime; block tail temporary scope; edition-2024 `if let` rescoping and tail-expression changes; promotion is distinct.
- **Constraints / authority / confidence / source:** Language/edition-defined; **high**. Verify in Rust Reference destructors/temporary scopes and Edition Guide 2024.

### RUST-OWN-006 — Interior mutability

- **Distinct behavior:** `UnsafeCell` permits mutation behind shared references and underpins cells, reference-counted borrow checks, locks, and atomics; wrappers impose different compile-time/runtime synchronization rules.
- **Observable content:** `Cell`, `RefCell` with successful and panicking borrow, and `UnsafeCell` in a minimal unsafe abstraction.
- **Variants and failures:** `RefCell` is not thread-safe; `Mutex` poisoning is library behavior; niche/layout effects of `UnsafeCell`; invalid aliasing outside the cell remains UB.
- **Constraints / authority / confidence / source:** Language special type plus standard library; **high**. Verify in standard `UnsafeCell`, `Cell`, and `RefCell` documentation.

### RUST-OWN-007 — Closure capture analysis

- **Distinct behavior:** Closures capture used places by shared borrow, unique immutable borrow, mutable borrow, or value; capture precision can select fields rather than whole variables.
- **Observable content:** Closures causing each capture mode, disjoint field captures, `move` closure, and use of uncaptured/partially captured values afterward.
- **Variants and failures:** Packed structs force broader capture; deref through `Box` receives special precision; capture affects closure lifetime, size, and auto traits; edition 2021 changed disjoint capture.
- **Constraints / authority / confidence / source:** Language/edition-defined; **high**. Verify in Rust Reference closure capture and Edition Guide 2021.

### RUST-OWN-008 — `Fn`, `FnMut`, and `FnOnce`

- **Distinct behavior:** Closure body behavior determines which call traits it implements; all closures implement `FnOnce`, while reusable mutation/borrowing permits additional call traits.
- **Observable content:** Read-only, mutating, and consuming closures passed to generic functions with each bound and invoked appropriate numbers of times.
- **Variants and failures:** `move` does not by itself imply only `FnOnce`; async closures have analogous async call traits in recent Rust; call traits are compiler-implemented.
- **Constraints / authority / confidence / source:** Language plus standard special traits; **high**. Verify in Rust Reference closure types and standard `Fn*` docs.

### RUST-OWN-009 — Pattern binding modes and match ergonomics

- **Distinct behavior:** Patterns can move, copy, or borrow subvalues; default binding modes adjust through reference matching, and explicit `ref`, `ref mut`, and `mut` have edition-sensitive restrictions.
- **Observable content:** Same structural pattern applied to owned, shared, and mutable references, with inferred binding types and explicit binding modifiers.
- **Variants and failures:** Rust 2024 tightened modifier/reference-pattern rules under non-move defaults; or-pattern bindings must agree in name/type/mode; partial moves.
- **Constraints / authority / confidence / source:** Language/edition-defined; **high**. Verify in Rust Reference patterns and Edition Guide 2024 match ergonomics changes.

### RUST-OWN-010 — Refutable versus irrefutable patterns

- **Distinct behavior:** `let`, function parameters, and selected constructs require irrefutable patterns, while `if let`, `while let`, `let-else`, and match arms accept refutable patterns.
- **Observable content:** All contexts using enum, tuple, slice, range, and wildcard patterns, including an irrefutability diagnostic.
- **Variants and failures:** Irrefutable `if let` lint; `let-else` else block must diverge; exhaustiveness/usefulness analysis accounts for uninhabited types imperfectly across versions.
- **Constraints / authority / confidence / source:** Language-defined with compiler exhaustiveness analysis; **high**. Verify in Rust Reference patterns and statements.

### RUST-OWN-011 — Or-patterns, guards, and binding scope

- **Distinct behavior:** Or-pattern alternatives share one binding set and type/mode; guards run after a successful pattern but do not make arms exhaustive and have special borrow semantics.
- **Observable content:** Nested or-pattern, guarded arm with bound values, illegal inconsistent alternatives, and apparently exhaustive guarded arms followed by wildcard.
- **Variants and failures:** Guard precedence with or-patterns; moves/borrows in guards; multiple candidate alternatives can reach guard evaluation.
- **Constraints / authority / confidence / source:** Language-defined; **high**. Verify in Rust Reference patterns and match expressions.

### RUST-OWN-012 — Destructuring assignment

- **Distinct behavior:** Assignment can destructure tuples, structs, arrays, and slices into existing places without creating bindings, with restrictions distinct from `let` patterns.
- **Observable content:** Nested destructuring assignment with `_`, field shorthand, and a pattern form legal in `let` but illegal for assignment.
- **Variants and failures:** Evaluation/drop order; compound-assignment places differ; feature stabilized after original pattern system and may vary on old compilers.
- **Constraints / authority / confidence / source:** Language-defined; **high**. Verify in Rust Reference destructuring assignment.

### RUST-OWN-013 — Drop check and `PhantomData`

- **Distinct behavior:** Generic destructors are checked for access to borrowed data at destruction, and `PhantomData` communicates ownership, variance, drop-check, and auto-trait relationships absent from runtime fields.
- **Observable content:** Generic owner/wrapper types with and without appropriate `PhantomData`, a `Drop` impl, and a borrow rejected due to destruction order.
- **Variants and failures:** Different phantom forms express covariance/contravariance/invariance and ownership; `may_dangle` is unstable/unsafe library implementation machinery.
- **Constraints / authority / confidence / source:** Language/compiler behavior plus standard marker type; **medium-high**. Verify in Rustonomicon and standard `PhantomData` docs.

## Expressions and control flow

### RUST-EXPR-001 — Place, value, and assignee expression contexts

- **Distinct behavior:** The same syntax may denote a place or produce a value; context determines whether a value is moved, copied, borrowed, assigned, or dropped.
- **Observable content:** Local, static, dereference, index, and field expressions used as value, borrow, assignment target, and compound-assignment target.
- **Variants and failures:** Parentheses preserve place context; overloaded index/deref; temporary value promotion; invalid assignment targets.
- **Constraints / authority / confidence / source:** Language-defined; **high**. Verify in Rust Reference expressions and place expressions.

### RUST-EXPR-002 — Blocks, statements, and tail expressions

- **Distinct behavior:** A block is an expression whose type/value comes from its unsemicolon-terminated tail; semicolons can discard values, and item statements do not execute.
- **Observable content:** Blocks returning values, unit via semicolon, nested item declarations, and a tail temporary with observable destruction.
- **Variants and failures:** Semicolon inference after item/flow expressions; edition-2024 tail temporary scope; unsafe/async/const/try blocks have specialized semantics.
- **Constraints / authority / confidence / source:** Language/edition-defined; **high**. Verify in Rust Reference block expressions and statements.

### RUST-EXPR-003 — `if`, `if let`, and let chains

- **Distinct behavior:** Branch types must coerce to a common type; conditional let patterns bind only in successful regions, and let chains combine pattern tests with booleans under version-dependent support.
- **Observable content:** Value-producing `if`, `if let` binding scope, chained lets/booleans if stable on selected compiler, and incompatible branch diagnostic.
- **Variants and failures:** `if` condition is strictly `bool`; edition-2024 temporary rescoping; let-chain stabilization version should be pinned and older compiler failure retained.
- **Constraints / authority / confidence / source:** Language-defined; **medium-high** because let-chain stabilization timing is version-sensitive. Verify in Rust Reference conditional expressions and release notes.

### RUST-EXPR-004 — `match` exhaustiveness and arm typing

- **Distinct behavior:** `match` checks pattern usefulness/exhaustiveness, scopes bindings per arm, evaluates guards, and coerces all arm result types.
- **Observable content:** Enum, integer range, slice, reference, and non-exhaustive external enum matches, including unreachable-arm and non-exhaustive diagnostics.
- **Variants and failures:** `#[non_exhaustive]`; uninhabited variants/types; guards ignored for exhaustiveness; float patterns restrictions; binding modes.
- **Constraints / authority / confidence / source:** Language-defined with compiler algorithm; **high**. Verify in Rust Reference match expressions and patterns.

### RUST-EXPR-005 — Loops, labels, and value-bearing `break`

- **Distinct behavior:** `loop` may produce a value from compatible `break` operands; `while` and `for` produce unit, and labels select nested loop/block control targets.
- **Observable content:** Nested labeled loops, value-returning `loop`, labeled block break, `continue`, and invalid value break from `while`/`for`.
- **Variants and failures:** Diverging infinite loop type; label namespace and shadowing; break from closure/async boundary is illegal.
- **Constraints / authority / confidence / source:** Language-defined; **high**. Verify in Rust Reference loop and break expressions.

### RUST-EXPR-006 — `for` desugaring and iterator selection

- **Distinct behavior:** `for` uses `IntoIterator::into_iter`, pattern matching on successive `Iterator::next` results, and a language-defined desugaring designed around temporary lifetimes.
- **Observable content:** Owned, shared, and mutable iteration; custom `IntoIterator`; refutable-loop-pattern rejection; edition-sensitive array iteration.
- **Variants and failures:** Edition 2021 changed array `.into_iter()` method-call behavior; explicit associated call differs on older editions; iterator adapters are library convention.
- **Constraints / authority / confidence / source:** Language plus standard iterator traits; **high**. Verify in Rust Reference iterator loop expressions and Edition Guide 2021.

### RUST-EXPR-007 — `return`, divergence, and unreachable code

- **Distinct behavior:** `return`, `break`, `continue`, panic paths, and nonreturning calls produce `!` and can coerce to surrounding types; compiler lints unreachable code separately from type validity.
- **Observable content:** Early return in branches, `loop {}`, panic macro, function returning `!`, and code producing an unreachable lint.
- **Variants and failures:** Never-type fallback/version behavior; Foreign Function Interface (FFI) nonreturning conventions; `return` inside closure returns from closure.
- **Constraints / authority / confidence / source:** Language plus lint behavior; **high**. Verify in Rust Reference return/never types and `rustc` lint docs.

### RUST-EXPR-008 — The `?` operator and residual conversion

- **Distinct behavior:** `?` branches on `Try`, returns a converted residual through `FromResidual`, and therefore is not simply an early return of the operand's error type.
- **Observable content:** `Result` with error conversion, `Option`, nested closure/try context, and an incompatible residual diagnostic.
- **Variants and failures:** User implementations of `Try` remain unstable; `try` blocks remain unstable as of the stated baseline; `?` is illegal in incompatible return contexts.
- **Constraints / authority / confidence / source:** Stable language syntax tied to standard traits, extensibility unstable; **high**. Verify in Rust Reference question-mark expressions and standard `ops::Try` docs.

### RUST-EXPR-009 — Range expressions and patterns

- **Distinct behavior:** Range syntax denotes standard range values in expressions but interval tests in patterns, with inclusive/exclusive and open-ended forms subject to different type restrictions.
- **Observable content:** All expression range forms, inclusive integer/char range patterns, slice-rest pattern contrast, and invalid empty/float pattern ranges.
- **Variants and failures:** Inclusive range overflow behavior in iteration; edition parsing ambiguities; range pattern bounds require permitted constants.
- **Constraints / authority / confidence / source:** Language plus standard range types; **high**. Verify in Rust Reference range expressions and patterns.

### RUST-EXPR-010 — Indexing, slicing, and bounds failure

- **Distinct behavior:** Index syntax uses place/value context and `Index`/`IndexMut`; built-in slices and arrays panic on out-of-bounds access, while range indexing selects slice outputs.
- **Observable content:** Built-in and user-overloaded indexing, mutable indexing, range slice, compile-time provable unconditional panic lint, and runtime panic case.
- **Variants and failures:** Evaluation order of receiver/index; UTF-8 strings intentionally lack integer indexing; unchecked methods are unsafe library operations.
- **Constraints / authority / confidence / source:** Language syntax plus standard trait/library behavior; **high**. Verify in Rust Reference array/index expressions and standard ops traits.

### RUST-EXPR-011 — Evaluation order and operand temporaries

- **Distinct behavior:** Rust specifies left-to-right evaluation for many operand lists while logical operators short-circuit and overloaded compound assignment avoids naïve double evaluation.
- **Observable content:** Side-effect markers in tuple/array/call/method/binary operands, short-circuit conditions, and indexed compound assignment.
- **Variants and failures:** Assignment expression operand order is a notable rule to verify; macro expansion can produce different syntax; optimization preserves observable behavior absent UB.
- **Constraints / authority / confidence / source:** Language-defined, with some contexts requiring exact verification; **medium-high**. Verify in Rust Reference expression evaluation order and operator expressions.

### RUST-EXPR-012 — Literals, suffixes, and numeric inference

- **Distinct behavior:** Integer/float literals have bases, separators, suffixes, negation parsing, overflow checks, and unsuffixed inference/defaulting; string/byte/C-string forms produce distinct types.
- **Observable content:** Suffixed and inferred numerics, boundary negative literal, raw strings with hashes, byte strings/chars, and C string literals where supported.
- **Variants and failures:** Out-of-range diagnostics depend on inferred type and overflow lint; C string literals stabilized after older compilers; escapes and Unicode scalar restrictions.
- **Constraints / authority / confidence / source:** Language-defined; **high**. Verify in Rust Reference literals and tokens.

## Constants, statics, macros, and attributes

### RUST-CONST-001 — Const items versus inline const blocks

- **Distinct behavior:** A `const` item is an anonymous value namespace item whose use is inlined conceptually; `const {}` creates a const-evaluation context that may capture surrounding generics unlike nested const items.
- **Observable content:** Associated/free consts, repeated address observation, generic inline const assertion, and a nested const item that cannot capture an outer generic.
- **Variants and failures:** Promotion can still give static storage to references; const item name is not a stable address identity; compile-time evaluation failure.
- **Constraints / authority / confidence / source:** Language-defined; **high**. Verify in Rust Reference constant items and const block expressions.

### RUST-CONST-002 — Statics, mutable statics, and thread-local statics

- **Distinct behavior:** A static denotes one storage location with `'static` lifetime; mutable static access is unsafe, and thread-local facilities provide per-thread identity with attribute/library rules.
- **Observable content:** Immutable static, interior-mutable synchronized static, `static mut` with edition-2024 reference diagnostic, and thread-local declaration.
- **Variants and failures:** Statics containing `Sync` types; initialization must be const; `#[thread_local]` itself is unstable while standard macro is stable; linker sections.
- **Constraints / authority / confidence / source:** Language plus standard library and platform; **high**. Verify in Rust Reference static items, Edition Guide 2024, and standard thread-local docs.

### RUST-CONST-003 — Const functions and const-evaluation restrictions

- **Distinct behavior:** `const fn` may execute at compile time in const contexts and at runtime elsewhere; allowed operations have expanded over compiler releases and invalid evaluated paths are errors only when evaluation requires them.
- **Observable content:** Const function used in array length/static and runtime call, conditional prohibited operation not taken in one const, and failing evaluated path.
- **Variants and failures:** Const trait calls/destructors and allocation remain evolving; overflow and panic become compile errors in required const evaluation; feature gates.
- **Constraints / authority / confidence / source:** Language plus evolving `rustc` const evaluator; **high** generally, **medium** at frontier. Verify in Rust Reference const evaluation and release notes.

### RUST-CONST-004 — Static promotion

- **Distinct behavior:** Selected rvalue expressions borrowed in const-compatible contexts are promoted to hidden static storage, changing reference lifetime without declaring a named static.
- **Observable content:** Promotable immutable borrow accepted as `'static`, contrasted with interior-mutating, destructor-bearing, or runtime-dependent value that is not promoted.
- **Variants and failures:** Promotion is not guaranteed merely because an expression is const-evaluable; temporary lifetime extension differs; address identity assumptions.
- **Constraints / authority / confidence / source:** Language/compiler-defined; **medium-high**. Verify in Rust Reference constant evaluation and promoted expressions.

### RUST-MAC-001 — Declarative macro matching and repetition

- **Distinct behavior:** `macro_rules!` matches token trees by fragment kinds and repetitions, then transcribes syntax; matching has no arbitrary lookahead and ambiguity is diagnosed.
- **Observable content:** Multiple arms, all common semantic fragment kinds, nested repetition with separators, and a local ambiguity/error arm.
- **Variants and failures:** Fragment grammar is edition-sensitive; repetition metavariables must nest consistently; literal tokens after fragments have follow-set limits.
- **Constraints / authority / confidence / source:** Language-defined; **high**. Verify in Rust Reference macros by example.

### RUST-MAC-002 — Macro hygiene and `$crate`

- **Distinct behavior:** Declarative macros use mixed-site hygiene: loop/block labels and local variables resolve at definition site while most items resolve at invocation site; `$crate` names the defining crate.
- **Observable content:** Macro referring to a definition-site helper via `$crate`, invocation-site item, local binding collision, and re-exported macro used downstream.
- **Variants and failures:** `$crate` does not bypass visibility; helper macro paths; procedural macros have span-based hygiene behavior distinct from `macro_rules!`.
- **Constraints / authority / confidence / source:** Language-defined; **high**. Verify in Rust Reference macro hygiene.

### RUST-MAC-003 — Macro scope, export, and shadowing

- **Distinct behavior:** Unqualified macro resolution uses textual and path-based scopes; `#[macro_export]`, legacy `#[macro_use]`, module macros, and re-exports expose names differently from ordinary items.
- **Observable content:** Same-named macros declared at different textual points/modules, qualified invocation, exported macro, and legacy import in an older-edition crate.
- **Variants and failures:** Macro can be used before declaration only through path-based availability; shadowing is textual; exported macro appears at crate root.
- **Constraints / authority / confidence / source:** Language/edition-defined; **high**. Verify in Rust Reference macro scope and importing/exporting.

### RUST-MAC-004 — Derive macros and helper attributes

- **Distinct behavior:** `#[derive]` invokes derive macros that add items, usually trait impls; registered helper attributes are visible to all derives on the item and have inert syntax until interpreted.
- **Observable content:** Built-in and custom derives, generic input, helper attribute, generated impl used through a bound, and duplicate/conflicting impl failure.
- **Variants and failures:** Derive expansion order/interaction should not be assumed beyond rules; automatic generic bounds may be broader than necessary; proc-macro diagnostics/spans.
- **Constraints / authority / confidence / source:** Language procedural macro system plus standard derives; **high**. Verify in Rust Reference derive/procedural macros.

### RUST-MAC-005 — Attribute macros and function-like procedural macros

- **Distinct behavior:** Attribute macros can replace an annotated item token stream, while function-like procedural macros expand at allowed macro invocation sites and may generate any syntactically valid content there.
- **Observable content:** Attribute macro preserving/modifying/removing input and function-like macro generating expressions and items, with downstream references to generated entities.
- **Variants and failures:** Expansion ordering with derives/other attributes; emitted invalid syntax/type errors; spans and absolute paths; macro crate executes arbitrary host code at build time.
- **Constraints / authority / confidence / source:** Language plus host proc-macro API; **high**. Verify in Rust Reference procedural macros.

### RUST-MAC-006 — Built-in macros with compiler semantics

- **Distinct behavior:** Built-in macros such as `cfg!`, `env!`, `option_env!`, `file!`, `line!`, `module_path!`, `concat!`, `format_args!`, `include!`, and `compile_error!` depend on compilation context or produce compiler-recognized forms.
- **Observable content:** Each major contextual category, especially macro-expanded call sites, compile-time environment, formatted arguments, and intentional compile error behind a cfg.
- **Variants and failures:** Build-script `rustc-env`; tracked versus untracked environment changes; source remapping affects file paths; `cfg!` returns bool but does not remove ill-typed branches.
- **Constraints / authority / confidence / source:** Compiler/standard built-in behavior; **high**. Verify in standard macro documentation and `rustc` environment documentation.

### RUST-ATTR-001 — Inner versus outer attributes

- **Distinct behavior:** Outer attributes apply to the following construct, while inner attributes apply to the enclosing crate/module/block-like construct, with position restrictions.
- **Observable content:** Crate-level inner attributes, module inner attributes, outer item/field/variant attributes, and an invalid-position diagnostic.
- **Variants and failures:** Macro expansion can produce attributes; doc comments desugar to `doc`; expression/statement attribute support is restricted.
- **Constraints / authority / confidence / source:** Language-defined; **high**. Verify in Rust Reference attributes.

### RUST-ATTR-002 — Active, inert, derive-helper, and tool attributes

- **Distinct behavior:** Active attributes remove/transform themselves and possibly input; inert attributes remain for later phases; registered tool attributes form a separate namespace-like path.
- **Observable content:** `cfg`, `cfg_attr`, derive/helper, lint, representation, documentation, and a registered tool attribute.
- **Variants and failures:** Unknown attributes are errors except permitted tool namespaces; attribute ordering matters for some active expansion; unsafe attributes in edition 2024.
- **Constraints / authority / confidence / source:** Language/compiler-defined; **high**. Verify in Rust Reference attributes and Edition Guide 2024 unsafe attributes.

### RUST-ATTR-003 — Conditional compilation on items and subparts

- **Distinct behavior:** `#[cfg]` removes annotated syntax before later semantic analysis, and can apply to modules, items, fields, variants, match arms, generic parameters, and some expressions/statements.
- **Observable content:** Conditional definitions and uses at several syntactic levels, with mutually exclusive same-named implementations and a deliberately unguarded missing reference.
- **Variants and failures:** Removed code still must lex/parse enough for attribute processing in surrounding grammar; cfg predicates may be set by target, features, build scripts, or flags; `cfg!` is not removal.
- **Constraints / authority / confidence / source:** Language plus compiler/Cargo inputs; **high**. Verify in Rust Reference conditional compilation and Cargo features docs.

## Unsafe Rust, FFI, ABI, and low-level behavior

### RUST-UNSAFE-001 — Unsafe operations and unsafe blocks

- **Distinct behavior:** Unsafe blocks permit a fixed set of operations but do not weaken other type/borrow checks; declaring an unsafe function shifts a safety precondition to callers.
- **Observable content:** Raw dereference, unsafe function call, mutable static access, union field read, and unsafe trait implementation inside documented scopes.
- **Variants and failures:** Edition 2024 lint requires explicit unsafe blocks inside unsafe functions; unnecessary/unused unsafe diagnostics; compilation success does not prove soundness.
- **Constraints / authority / confidence / source:** Language/edition-defined; **high**. Verify in Rust Reference unsafe keyword/operations and Edition Guide 2024.

### RUST-UNSAFE-002 — Unsafe traits and implementations

- **Distinct behavior:** An unsafe trait carries invariants that implementations must uphold, and each implementation requires `unsafe impl`; using the trait may enable unsafe assumptions elsewhere.
- **Observable content:** Unsafe marker-like trait, safe API relying on its invariant within unsafe code, valid unsafe impl, and missing-unsafe diagnostic.
- **Variants and failures:** Trait methods may themselves be safe or unsafe independently; auto traits and negative impls; incorrect impl compiles but can cause UB.
- **Constraints / authority / confidence / source:** Language-defined safety contract; **high**. Verify in Rust Reference traits and unsafe implementations.

### RUST-UNSAFE-003 — Validity, initialization, and `MaybeUninit`

- **Distinct behavior:** Rust types impose validity invariants beyond bit size; uninitialized or invalid values may cause immediate UB, and `MaybeUninit<T>` suppresses the assumption until initialized.
- **Observable content:** Safe staged initialization with `MaybeUninit`, array initialization with partial-drop cleanup, and nonexecuted/documented invalid-value counterexamples without running UB.
- **Variants and failures:** Zero is invalid for references and many enums; padding is not stable data; `assume_init` obligation; compiler optimizations exploit validity.
- **Constraints / authority / confidence / source:** Language semantics with details in standard docs and unsafe guidance; **high** generally, **medium** at exact validity frontier. Verify in standard `MaybeUninit` docs, Rustonomicon, and Reference behavior-considered-undefined section.

### RUST-UNSAFE-004 — Pointer provenance and exposed addresses

- **Distinct behavior:** Pointer operations carry provenance in the compiler memory model; address exposure/reconstruction APIs distinguish strict provenance operations from legacy casts, though the full model continues to evolve.
- **Observable content:** Pointer arithmetic within allocation, address extraction/mapping, pointer comparison, and documented rejected/UB out-of-bounds or fabricated-reference cases.
- **Variants and failures:** Zero-sized types; one-past pointers; wrapping versus in-bounds offset; integer round trips; Miri model may be stricter or experimental, not language authority.
- **Constraints / authority / confidence / source:** Partly specified/evolving implementation model; **medium**. Verify in standard pointer APIs, Rust Reference UB list, and Unsafe Code Guidelines material.

### RUST-UNSAFE-005 — Transmutation and layout compatibility

- **Distinct behavior:** `transmute` reinterprets bits only when source/destination satisfy size and validity requirements; equal size does not imply valid or stable representation.
- **Observable content:** A valid transparent-wrapper conversion and compile-fail size mismatch, plus documented invalid enum/reference and padding counterexamples.
- **Variants and failures:** Const-eval restrictions for pointer/integer transmute; endian dependence; `transmute_copy`; safer conversion APIs.
- **Constraints / authority / confidence / source:** Unsafe standard intrinsic with language validity/layout rules; **high** for obligations, **medium** for evolving edges. Verify in standard `mem::transmute` docs and Rust Reference layout/UB sections.

### RUST-FFI-001 — Foreign blocks and ABI strings

- **Distinct behavior:** Foreign blocks declare externally defined functions/statics with an ABI, and calls/access have safety determined by declaration and recent syntax rules.
- **Observable content:** `extern "C"` function/static declarations, safe-qualified foreign item where supported, variadic declaration, and link to a small native symbol boundary.
- **Variants and failures:** Edition 2024 requires unsafe extern blocks; supported ABI strings are target-dependent; unwinding ABIs use `-unwind`; calling-convention mismatch is UB.
- **Constraints / authority / confidence / source:** Language plus target ABI/compiler; **high**. Verify in Rust Reference external blocks and Edition Guide 2024.

### RUST-FFI-002 — Exported symbols and unsafe attributes

- **Distinct behavior:** `no_mangle`, `export_name`, and `link_section` affect global linker identity/placement and carry safety obligations; edition 2024 requires unsafe attribute syntax.
- **Observable content:** Public foreign-callable function with explicit ABI/export name and a static in a custom section, inspected conceptually through declared link consumers.
- **Variants and failures:** Duplicate symbol names can cause link errors or UB; platform naming/visibility; dead-code/linker elimination; unsafe attribute syntax differs by edition.
- **Constraints / authority / confidence / source:** Compiler/linker and edition-defined; **high**. Verify in Rust Reference code-generation attributes and Edition Guide 2024.

### RUST-FFI-003 — Link attributes and native libraries

- **Distinct behavior:** `#[link]` and build-script link directives select native library name/kind, modifiers, search paths, and argument order, creating edges not visible as Rust imports.
- **Observable content:** Static and dynamic native library declarations, framework/raw-dylib conditional variants if platform supports them, and build-script emitted search/link directives.
- **Variants and failures:** Target linker and platform formats; native library modifiers; missing or wrong-architecture library produces link failure; cross-compilation host/target separation.
- **Constraints / authority / confidence / source:** `rustc`/Cargo/platform-defined; **high**. Verify in Rust Reference external blocks/linkage and `rustc` native library docs.

### RUST-FFI-004 — C-compatible types and opaque handles

- **Distinct behavior:** Only FFI-safe layouts and ABI-compatible signatures may cross foreign boundaries; Rust references, trait objects, tuples, and ordinary Rust enums often lack a stable C contract.
- **Observable content:** `repr(C)` struct/union, integer-repr enum, nullable pointer/handle, callback function pointer, and improper-ctypes diagnostic cases.
- **Variants and failures:** `bool`/`char` validity differs from arbitrary C integers; nullable function-pointer niche guarantee; ownership and allocator boundaries; long width varies by platform.
- **Constraints / authority / confidence / source:** Language guarantees plus platform ABI and lint implementation; **high**. Verify in Rust Reference type layout/FFI and platform C ABI docs.

### RUST-FFI-005 — Unwinding across ABI boundaries

- **Distinct behavior:** Whether panic/foreign exceptions may unwind across an FFI boundary depends on ABI string and panic strategy; crossing a non-unwind boundary has abort or UB-defined consequences depending on direction.
- **Observable content:** Conditional `extern "C"` and `extern "C-unwind"` callbacks with `catch_unwind` at a boundary and abort-profile configuration, without relying on executing UB.
- **Variants and failures:** Foreign exceptions through Rust frames; destructors during unwinding; not all targets support all unwind ABIs; `UnwindSafe` is advisory library safety.
- **Constraints / authority / confidence / source:** Language/RFC plus compiler/runtime/platform; **medium-high**. Verify in Rust Reference ABI/unwinding material and standard panic docs.

### RUST-FFI-006 — Inline assembly and global assembly

- **Distinct behavior:** `asm!` and `global_asm!` embed target-specific instructions/symbols with explicit registers, operands, options, clobber ABI, labels, and safety contracts.
- **Observable content:** Target-gated minimal assembly using input/output/inout operands and a separate global symbol, plus invalid register/option compile failures.
- **Variants and failures:** Architecture-specific syntax/register classes; LLVM backend constraints; `noreturn`, `nostack`, `preserves_flags`, memory options can make incorrect code UB; some targets unsupported.
- **Constraints / authority / confidence / source:** Stable compiler language extension on supported targets; **medium-high**. Verify in Rust Reference inline assembly.

### RUST-FFI-007 — SIMD and target features

- **Distinct behavior:** Single Instruction, Multiple Data (SIMD) intrinsics/types and `#[target_feature]` make call validity depend on compile-time and runtime CPU capabilities; portable SIMD remains version-sensitive/unstable.
- **Observable content:** Architecture-gated intrinsic behind runtime feature detection and a target-feature function, plus compile-time target-feature cfg.
- **Variants and failures:** Calling target-feature function without capability is unsafe; global `-C target-feature`; cross-target availability; portable SIMD feature gate.
- **Constraints / authority / confidence / source:** `rustc`/standard architecture modules and platform-defined; **medium-high**. Verify in standard architecture intrinsic docs and `rustc` target-feature docs.

## Async, generators, concurrency, and runtime-facing semantics

### RUST-ASYNC-001 — Async function and block opaque future types

- **Distinct behavior:** Each async function/block produces a distinct anonymous future state-machine type; execution does not begin until polled and suspension preserves selected locals.
- **Observable content:** Async function and multiple async blocks with/without `move`, local values spanning `.await`, and explicit generic `Future` consumer.
- **Variants and failures:** Recursive async requires indirection; future size/captures differ; return-position lifetime capture is edition-sensitive; concrete layout is implementation-defined.
- **Constraints / authority / confidence / source:** Language plus standard `Future` contract; **high**. Verify in Rust Reference async items/blocks and standard `Future` docs.

### RUST-ASYNC-002 — Await expression and suspension

- **Distinct behavior:** `.await` repeatedly polls a future and may suspend the enclosing async context, introducing control-flow and borrow-across-suspension boundaries.
- **Observable content:** Custom future with Pending/Ready transitions, borrow living across await, temporary not living across another await, and `.await` outside async diagnostic.
- **Variants and failures:** Holding lock guards across await is semantic/library risk; drop/cancellation at suspension; await syntax participates in postfix parsing and autoderef/pinning machinery.
- **Constraints / authority / confidence / source:** Language plus standard task API; **high**. Verify in Rust Reference await expressions and `Future` docs.

### RUST-ASYNC-003 — Pinning and self-referential futures

- **Distinct behavior:** `Pin<P>` restricts movement of pointees that are not `Unpin`; compiler-generated futures may be self-referential and therefore require pinned polling.
- **Observable content:** `Unpin` and explicitly non-`Unpin` types, safe pin projection through permitted fields, and a custom future `poll` receiver.
- **Variants and failures:** `Pin` alone does not pin if pointer escape/movement remains possible; projection usually needs unsafe or ecosystem macros; `pin!` and boxed pinning differ.
- **Constraints / authority / confidence / source:** Standard-library contract tightly coupled to language-generated futures; **high**. Verify in standard `Pin`, `Unpin`, and `Future` docs.

### RUST-ASYNC-004 — Async traits and dyn limitations

- **Distinct behavior:** Native async trait methods desugar to implementation-specific opaque future returns and support static generic calls but are generally not dyn-compatible without adaptation.
- **Observable content:** Async trait method with two implementations, generic caller, attempted `dyn` call diagnostic, and explicit boxed-future object-safe alternative.
- **Variants and failures:** `Send` bound on returned future cannot always be expressed ergonomically in public traits; Return Type Notation remains evolving; ecosystem attribute macros generate boxed forms with different semantics.
- **Constraints / authority / confidence / source:** Language feature plus ecosystem alternative; **high** for native behavior, **medium** for evolving bounds. Verify in Rust Reference async functions in traits and relevant accepted RFC material.

### RUST-ASYNC-005 — Async closures and async call traits

- **Distinct behavior:** Async closures can borrow from captured state in their returned future and implement compiler-defined async call traits according to capture/consumption behavior.
- **Observable content:** `async ||` closures that read, mutate, and consume captures; generic callers over supported async call bounds; contrast with closure returning an async block.
- **Variants and failures:** Stabilized with Rust 2024-era compiler support; lending closures constrain repeated calls and escaping futures; exact trait-bound surface changed recently.
- **Constraints / authority / confidence / source:** Recent language/standard special-trait feature; **medium-high**. Verify in Rust 1.85 release material, Edition Guide 2024, and standard async call trait docs.

### RUST-ASYNC-006 — Sendability across threads

- **Distinct behavior:** A future is `Send` only when state retained across suspension is `Send`; values used and dropped before await may not affect the result.
- **Observable content:** Two nearly identical futures where a non-`Send` value does or does not cross an await, checked by a `Send` bound and by a thread-capable executor-style API signature.
- **Variants and failures:** Lexical/drop analysis can change with explicit scopes/drop calls; async trait returned-future bounds; executor choice is ecosystem-specific and not language semantics.
- **Constraints / authority / confidence / source:** Language-generated type plus standard auto traits; **high**. Verify in standard `Future`, `Send`, and async language documentation.

### RUST-CONCUR-001 — Threads, `Send`, `Sync`, and scoped borrowing

- **Distinct behavior:** Thread spawning requires transferred closures and results to meet lifetime/`Send` bounds; scoped threads permit non-`'static` borrows while preserving join-before-scope-exit safety.
- **Observable content:** Owned move into ordinary thread, rejected borrowed capture, accepted scoped borrow, and shared access requiring `Sync`.
- **Variants and failures:** Panic/join result; detached thread on dropped handle; thread availability on target; library, not core language, but auto-trait resolution is semantic.
- **Constraints / authority / confidence / source:** Standard-library API plus language auto traits/lifetimes; **high**. Verify in standard thread docs.

### RUST-CONCUR-002 — Atomics and memory ordering

- **Distinct behavior:** Atomic operations have target-dependent width availability and ordering constraints (`Relaxed`, acquire/release, sequentially consistent) that define cross-thread synchronization rather than ordinary borrowing.
- **Observable content:** Atomic load/store/read-modify-write/compare-exchange with valid orderings, cfg on atomic width, and invalid ordering diagnostic/panic where specified.
- **Variants and failures:** Not all targets support all atomic sizes; compiler fences versus hardware fences; data races through non-atomic unsafe access are UB.
- **Constraints / authority / confidence / source:** Standard library mapped to Rust/C++-style memory model and target implementation; **high** for APIs, **medium** for formal model edges. Verify in standard atomic docs and Rust Reference UB concurrency rules.

## Diagnostics, configuration, tests, and ecosystem-facing project conditions

### RUST-CFG-001 — Built-in target cfg values

- **Distinct behavior:** `rustc` supplies cfg keys/values for architecture, operating system, environment, vendor, pointer width, endianness, families, panic strategy, atomics, and target features.
- **Observable content:** Mutually exclusive modules/items for at least two realistic targets and compile-time assertions or errors for unexpected combinations.
- **Variants and failures:** Custom targets choose values; not every cfg is mutually exclusive; feature detection at runtime differs from compilation; host cfg does not govern target crate.
- **Constraints / authority / confidence / source:** `rustc`/target-defined; **high**. Verify in Rust Reference conditional compilation and `rustc --print cfg` documentation.

### RUST-CFG-002 — Cargo feature cfg and custom cfg checking

- **Distinct behavior:** Cargo maps enabled package features to `cfg(feature = "...")`; arbitrary cfgs can be passed by build scripts/flags, and `check-cfg` declares expected names/values for diagnostics.
- **Observable content:** Default, optional, all-features, no-default-features variants, custom build-script cfg, and an unexpected-cfg warning.
- **Variants and failures:** Features are package-local but unified per compilation; environment variables are not cfgs; command selection can cause a package to compile more than once.
- **Constraints / authority / confidence / source:** Cargo and `rustc`-defined; **high**. Verify in Cargo features and `rustc` check-cfg docs.

### RUST-CFG-003 — `cfg_attr` conditional attributes

- **Distinct behavior:** `cfg_attr` conditionally injects one or several attributes, which can alter representation, derives, lint levels, paths, linkage, and further cfg evaluation.
- **Observable content:** Conditional derive, representation/link attribute, and nested `cfg_attr`, with downstream entities present only in selected configuration.
- **Variants and failures:** Attribute legality checked only when produced; expansion ordering; cannot conditionally inject every crate-level attribute in every position.
- **Constraints / authority / confidence / source:** Language-defined; **high**. Verify in Rust Reference conditional compilation.

### RUST-DIAG-001 — Lint levels and scope

- **Distinct behavior:** Lints may be allow, expect, warn, force-warn by command line, deny, or forbid, with nested attribute scope and cap behavior; warnings can become build failures.
- **Observable content:** Crate/module/item scoped lint levels, `#[expect]` fulfilled/unfulfilled cases, command-line cap/deny configuration, and forbidden-lint override failure.
- **Variants and failures:** Lint names/groups and defaults change by compiler version/edition; Cargo dependency lint capping; unknown lint diagnostics.
- **Constraints / authority / confidence / source:** `rustc` implementation with language attributes; **high**. Verify in `rustc` lint documentation and Rust Reference lint attributes.

### RUST-DIAG-002 — Compile errors across phases

- **Distinct behavior:** Lexing, parsing, macro expansion, name resolution, type checking, borrow checking, const evaluation, monomorphization, code generation, linking, and runtime each expose different failure boundaries.
- **Observable content:** Intentionally isolated negative cases for each phase, configuration-gated so valid project modes remain buildable.
- **Variants and failures:** Earlier errors suppress later analysis; some generic errors appear only when instantiated; dead code can still be type-checked but cfg-removed code is absent; diagnostic wording is not stable.
- **Constraints / authority / confidence / source:** Compiler behavior implementing language/toolchain; **high**. Verify against Rust Reference rules and `rustc` error/lint docs without depending on exact messages.

### RUST-DIAG-003 — Generic instantiation and codegen-time failures

- **Distinct behavior:** Rust type-checks generic bodies under declared bounds, but monomorphization can reveal implementation limits or codegen/link issues for selected instantiations.
- **Observable content:** Generic item used with several concrete types, unused generic item, recursive monomorphization/overflow boundary, and target-specific symbol failure.
- **Variants and failures:** Const-evaluatable obligations; recursion limits; polymorphization may avoid codegen distinctions; compiler internal errors are not normative behavior.
- **Constraints / authority / confidence / source:** Language plus `rustc` implementation; **medium-high**. Verify in `rustc` monomorphization/codegen documentation and Reference generic rules.

### RUST-DIAG-004 — Recursion and type-computation limits

- **Distinct behavior:** `recursion_limit` and `type_length_limit` constrain macro expansion/autoderef/type substitution in implementation-specific ways, producing valid-language programs that a compiler may reject at configured limits.
- **Observable content:** Recursive macro, deeply nested autoderef/type, custom crate limits, and accepted/rejected threshold variants.
- **Variants and failures:** `type_length_limit` enforcement historically depended on compiler options; trait solver overflow has related but distinct limits; raising limits can increase resource use.
- **Constraints / authority / confidence / source:** `rustc` attributes/implementation limits; **medium-high**. Verify in Rust Reference limits attributes and `rustc` docs.

### RUST-TEST-001 — Unit and integration test crate boundaries

- **Distinct behavior:** Unit tests compile with the library crate and can access private items in ancestor modules; each integration test is a separate crate that accesses only public library API.
- **Observable content:** Unit test using private item and integration test using public API, with multiple integration test targets and shared support module behavior.
- **Variants and failures:** `cfg(test)` is set per tested crate, not dependencies; integration support files can accidentally become separate targets depending on location/naming; doctests differ again.
- **Constraints / authority / confidence / source:** Cargo/`rustc` test harness behavior; **high**. Verify in Cargo target/test documentation and Rust testing docs.

### RUST-TEST-002 — Test harness attributes and generated main

- **Distinct behavior:** `#[test]`, `#[ignore]`, `#[should_panic]`, and benchmark/custom-harness settings cause compiler/Cargo harness registration and generated entry behavior.
- **Observable content:** Passing, ignored, expected-panic, result-returning test, and a target with `harness = false` and explicit entry.
- **Variants and failures:** Test functions have signature restrictions; output capture/threading are harness behavior; built-in benchmarks remain unstable; panic abort support varies.
- **Constraints / authority / confidence / source:** `rustc`/Cargo implementation and standard test conventions; **high**. Verify in Rust Reference test attributes and Cargo target docs.

### RUST-TEST-003 — Documentation tests

- **Distinct behavior:** Rust documentation code blocks can be transformed into synthetic crates, compiled, run, or expected to fail independently of ordinary test targets.
- **Observable content:** Runnable, `no_run`, `compile_fail`, ignored, hidden-line, and edition-tagged examples referring to the documented crate.
- **Variants and failures:** Crate injection and synthetic `main`; target/feature environment; private items usually inaccessible; exact compile-fail diagnostics are not stable.
- **Constraints / authority / confidence / source:** `rustdoc` implementation/ecosystem convention; **high**. Verify in rustdoc documentation on documentation tests.

### RUST-TEST-004 — Examples and benchmarks as separate targets

- **Distinct behavior:** Cargo examples and benchmarks are separate crate targets with their own required features, harness choice, crate type, and dependency availability.
- **Observable content:** One example gated by required features and one benchmark/custom harness sharing library API.
- **Variants and failures:** Auto-discovery versus manifest declaration; benchmark framework often ecosystem-provided on stable; dev-dependency availability; build commands select different target sets.
- **Constraints / authority / confidence / source:** Cargo-defined with ecosystem benchmark convention; **high**. Verify in Cargo targets documentation.

### RUST-ECO-001 — `macro_rules!`-based ecosystem code generation

- **Distinct behavior:** Common libraries expose declarative macros that generate modules, impls, statics, or tests; resulting semantic entities depend on feature flags and macro input rather than handwritten items.
- **Observable content:** A small dependency macro with generated public and private items referenced by normal code, without depending on a specific product domain.
- **Variants and failures:** `$crate` paths, downstream hygiene, compile errors emitted by macro, and semver changes in expansion internals; generated private implementation should not be mistaken for stable API.
- **Constraints / authority / confidence / source:** Ecosystem convention atop language macros; **high**. Verify in the selected dependency's own documentation and Rust Reference macros.

### RUST-ECO-002 — Attribute-driven serialization/schema-style derives

- **Distinct behavior:** Widely used derive ecosystems interpret container/variant/field helper attributes to generate trait impls whose effective names, bounds, and field relationships differ from source identifiers.
- **Observable content:** A generic data type with rename, skip/default/flatten-like helper behavior and generated trait use, using a neutral dependency if ecosystem coverage is desired.
- **Variants and failures:** Conflicting helper attributes; derive-generated bounds; remote/adapter patterns; behavior belongs to dependency version, not Rust language.
- **Constraints / authority / confidence / source:** Ecosystem convention; **high** at category level. Verify in the chosen derive crate documentation.

### RUST-ECO-003 — Error and command/runtime attribute macro conventions

- **Distinct behavior:** Common derive/attribute libraries synthesize error conversions, display/source methods, async entry points, registrations, or test wrappers, creating control-flow and implementation edges absent from direct syntax.
- **Observable content:** Neutral custom error enum deriving conversions plus one attribute that rewrites an async or test function into a runtime entry.
- **Variants and failures:** Generated bounds and hidden modules; runtime-specific `Send` requirements; feature gates; macro version changes; these are representative ecosystem cases, not language requirements.
- **Constraints / authority / confidence / source:** Ecosystem convention; **medium-high**. Verify in selected crates' documentation.

### RUST-ECO-004 — Build-time native binding or source generation convention

- **Distinct behavior:** Ecosystem build dependencies commonly discover system libraries, compile C/C++ code, or generate bindings/code into `OUT_DIR`, so actual entities and link edges depend on host tools and environment.
- **Observable content:** A hermetic, vendored minimal native input or deterministic generator producing a Rust include and native link directive.
- **Variants and failures:** Missing compiler/header/pkg-config; bindgen output varies with Clang and target; vendored versus system library features; cross-compilation requires separate host/target configuration.
- **Constraints / authority / confidence / source:** Cargo ecosystem convention/platform-defined; **high**. Verify in chosen build crate documentation and Cargo build-script docs.

### RUST-ECO-005 — `build.rs` metadata through `links`

- **Distinct behavior:** A package declaring `links` claims a native library name and can pass build-script metadata to immediate dependents; Cargo rejects multiple packages linking the same native library in one graph.
- **Observable content:** A sys-style package emitting metadata and one dependent build script consuming `DEP_<LINKS>_<KEY>` values.
- **Variants and failures:** `links` value uniqueness; metadata reaches direct dependent build scripts, not ordinary crate environment automatically; overridden build scripts.
- **Constraints / authority / confidence / source:** Cargo-defined, commonly used by `-sys` crates; **high**. Verify in Cargo build-script `links` documentation.

## Recent-version and edition-sensitive support inventory

The following changes deserve explicit version-pinned variants. Exact patch-level behavior should be verified before construction because this report was prepared without live source lookup.

| Change family | Material distinction to preserve | Confidence / likely primary source |
|---|---|---|
| Rust 2024 edition availability | Rust 1.85 introduced the 2024 edition baseline; manifests, Cargo resolver defaulting for edition 2024, and migration lints can change selected behavior. | **High**; Rust 1.85 release material, Edition Guide 2024, Cargo resolver docs. |
| 2024 match ergonomics reservations | `mut`, `ref`, `ref mut`, and explicit reference patterns are restricted when the inherited default binding mode is not move. | **High**; Edition Guide 2024 match ergonomics. |
| 2024 RPIT lifetime capture | Return-position `impl Trait` captures in-scope lifetime parameters by default, aligning with broader parameter capture; precise `use<...>` capture provides control where supported. | **High**; Edition Guide 2024 and Rust Reference `impl Trait`. |
| 2024 temporary scopes | `if let` temporaries and block tail-expression temporaries have narrower/changed destruction scopes. | **High**; Edition Guide 2024 temporary scope chapters. |
| 2024 unsafe syntax/lints | Unsafe extern blocks, explicit unsafe operations within unsafe functions, unsafe forms of selected attributes, and references to mutable statics receive changed requirements/lints. | **High**; Edition Guide 2024 unsafe chapters. |
| 2024 prelude additions | Newly prelude-imported traits/types can change unqualified resolution or cause ambiguity/shadowing in a crate switched editions. | **Medium-high**; Edition Guide 2024 prelude changes and standard prelude docs. |
| Macro fragment editions | `expr` fragment matching in 2024 includes forms excluded by the compatibility fragment, and edition belongs to the macro definition crate. | **High**; Edition Guide 2024 macro fragment specifier changes. |
| Async closures and async call traits | Async closure syntax and lending capture/call behavior became stable in the Rust 1.85 / Rust 2024 time frame. | **Medium-high**; Rust 1.85 release material and standard async call trait docs. |
| Precise opaque capture | `use<...>` capture syntax stabilized around the Rust 1.82 time frame and has restrictions worth pinning. | **Medium** on exact version; release notes and Rust Reference. |
| Native async functions in traits / RPIT in traits | Stable since the Rust 1.75 time frame; older compilers require boxed futures or macro expansion and expose different entities. | **High**; Rust 1.75 release material and accepted RFCs. |
| Generic associated types | Stable since the Rust 1.65 time frame; compiler solver/borrow-check limitations have continued to improve. | **High**; Rust 1.65 release material and Reference. |
| Let chains | Support and exact stabilization occurred after long feature-gated development and may postdate the 1.85 baseline; pin before including as a valid stable case. | **Low** on exact stabilization; Rust release notes and Reference conditional expressions. |
| C string literals | Stable on recent compilers but fail on older baselines; they create `&CStr` rather than byte/string types. | **Medium-high**; release notes and Rust Reference literals. |
| Cargo resolver 3 | Edition-2024 workspaces can default to a newer resolver with changed handling of dependency Rust-version compatibility; pin `resolver` explicitly for comparisons. | **Medium-high**; Cargo resolver docs and Edition Guide 2024. |
| Cargo lockfile versions | New Cargo releases can write a lockfile format older Cargo cannot parse; a fixture testing compiler versions must also pin or regenerate compatible lockfiles. | **High** at category level, **medium** on version mapping; Cargo lockfile docs/release notes. |
| `#[expect]` lint attribute and check-cfg integration | Recent compilers added fulfilled/unfulfilled lint expectations and expanded automatic cfg checking, changing diagnostics without language behavior changes. | **Medium-high**; `rustc` lint and check-cfg docs/release notes. |

## Implementation-defined, unspecified, and deliberately unstable areas

- **Layout under `repr(Rust)`:** Size, alignment beyond stated constraints, field order, enum encoding, niche use, and padding are not a stable cross-version contract. `repr(C)` only supplies the guarantees it explicitly defines and remains target-ABI-dependent.
- **Trait-object representation:** A trait object is a wide pointer semantically; virtual table field order, symbol names, and codegen sharing are implementation details.
- **Closure, async, and opaque types:** Identity is distinct per defining expression/item as specified, but field layout, state-machine representation, generated symbol names, and exact size are implementation-defined.
- **Name mangling and Rust ABI:** Rust ABI and mangled symbol details are not stable interoperability contracts. Compiler version, target, crate metadata, and codegen choices can alter symbols.
- **Monomorphization and optimization:** Observable behavior must be preserved, but whether code is instantiated, shared, inlined, devirtualized, or eliminated is an implementation choice.
- **Pointer provenance and aliasing:** The language defines major UB boundaries, but a complete operational provenance model and some aliasing details remain under development. Compiler and Miri behavior should not silently be treated as final language law.
- **Data-race and atomic model edges:** Data races are UB and standard atomics have documented orderings; some formal details inherit from or track the C++ memory model and compiler implementation.
- **Const evaluation frontier:** Which operations are const-stable is compiler-version-specific. Acceptance at runtime does not imply acceptance in a required const context.
- **Trait solver diagnostics:** Coherence and logical outcomes are language constraints, but overflow, ambiguity diagnostics, and the next-generation trait solver's accepted edge cases may vary by compiler/channel.
- **Borrow-check implementation:** Core safe programs must satisfy ownership rules, but diagnostics and some accepted advanced patterns can change as the borrow checker evolves. Polonius behavior is experimental unless made baseline.
- **Macro/procedural macro spans:** Hygiene has specified components, while source locations, error anchoring, and expansion backtraces have compiler/API limitations and version differences.
- **Evaluation details in UB programs:** Once UB occurs, downstream observations are not meaningful candidates. Invalid cases should be compile-fail or nonexecuted/documented, never executed to establish expected behavior.
- **Target and linker behavior:** Calling conventions, symbol decoration, section naming, TLS, atomic width, unwinding, native archive treatment, and linker errors vary across supported targets and linkers.
- **Resource limits and diagnostics:** Recursion depth, compilation memory/time, incremental cache behavior, diagnostic wording/order, and Internal Compiler Errors (ICEs) are implementation behavior, not stable semantics.
- **Nightly feature gates:** `specialization`, trait aliases, negative impls, custom auto traits, generic const expressions, portable SIMD, generators/coroutines, `try` blocks, custom test frameworks, `no_core`, and several low-level extensions must be isolated and channel-pinned. Their semantics may change.

## Known uncertainty and verification needs

- Exact stabilization releases and restrictions for let chains, precise opaque capture in every item position, async closure call-trait bounds, C string literals, and recent `const` operations should be verified against release notes and current Reference text.
- Exact Cargo resolver-3 behavior, Rust-version-aware selection, lockfile format compatibility, and feature unification across host/target/dev roles should be verified against the Cargo version selected for each test matrix entry.
- Effective visibility and private-in-public rules have moved between hard errors and lints over Rust's history. Negative cases should assert semantic category rather than diagnostic wording unless a compiler version is pinned.
- Exact drop-scope rules are syntax- and edition-sensitive. Complex `match`, `if let`, let-chain, tail-expression, and async cases need primary-source confirmation before expected order is asserted.
- Pointer provenance, reference aliasing, enum validity, uninitialized padding, and FFI unwinding are high-risk areas. Only clearly specified valid cases and clearly labeled nonexecuted counterexamples should be used until verified.
- Custom target JSON schema and support are implementation-specific and partly unstable. Prefer shipped target triples for normative cross-target comparisons, with custom targets isolated as compiler behavior.
- Ecosystem macro behavior belongs to the exact dependency version. Generated implementation details should be recorded as observations, not promoted to Rust semantics.

## Final completeness audit

Feature families covered:

- Package/workspace resolution, target selection, build scripts, generated sources, crate types, profiles, editions, and freestanding builds.
- Module loading, all major path anchors, namespace separation, imports/re-exports, visibility, preludes, scoping, and associated lookup.
- Nominal and structural types, DSTs, pointers/references, trait objects, opaque types, representation, coercion, casts, and inference.
- Lifetime/type/const generics, higher-ranked bounds, projections, Generic Associated Types (GATs), variance, and opaque capture.
- Inherent/trait implementations, coherence, blanket impls, method lookup, static/dynamic dispatch, auto traits, operator and drop special traits, and unstable specialization boundaries.
- Moves, borrowing, Non-Lexical Lifetimes (NLL), closure capture, call traits, patterns, drop check, temporaries, control flow, divergence, and `?`.
- Const evaluation, statics, promotion, declarative/procedural macros, hygiene, attributes, and conditional compilation.
- Unsafe boundaries, validity, pointer provenance, layout/transmute, FFI, linkage, unwinding, assembly, and target features.
- Async futures/closures/traits, pinning, suspension, sendability, threads, and atomics.
- Configuration matrices, lint and compiler-phase diagnostics, tests/doctests/examples, version gates, and representative ecosystem generation.

Potential remaining gaps to audit against primary sources before declaring the checklist closed:

- Rare tokenization/parser interactions such as raw identifiers, reserved prefixes, Unicode identifiers, nested generic `>` parsing, and macro token spacing are only indirectly covered. Include them only where they change identity or edition behavior, not as grammar enumeration.
- Advanced associated-type bounds, implied outlives bounds, projection normalization cycles, and the evolving trait solver may need more negative cases than this candidate list names.
- Exact behavior for uninhabited types, never-type fallback, exhaustive matching across foreign non-exhaustive types, and enum layout across target ABIs merits a dedicated verified matrix.
- Custom allocators, global allocator selection, allocation-error handlers, panic runtime linkage, and sanitizer/instrumentation builds may add distinct runtime/link entities for `no_std` and low-level targets.
- Dynamic loading, symbol versioning, linker scripts, Windows raw-dylib import generation, Apple frameworks, and WebAssembly imports/exports are platform families large enough to require target-specific verification.
- Rustdoc-specific namespace/link resolution, intra-doc links, hidden items, cfg documentation, and re-export inlining can expose a documentation graph distinct from the compiler semantic graph if documentation is in scope.
- Cargo artifact dependencies, public dependency declarations, alternate registries, sparse index behavior, credentialed sources, and package publishing metadata are evolving or unstable areas; include only if the chosen Cargo baseline supports them and build-graph identity is in scope.
- Incremental compilation, metadata-only builds, build standard library, compiler-provided crates, and cross-language Link Time Optimization (LTO) may change physical artifacts without changing source semantics; their relevance should be decided explicitly.
- Procedural macro tracked file/environment APIs, sandboxing absence, nondeterminism, and span/source-file APIs may require a reproducibility-focused supplement.
- Coroutines/generators, coroutine closures, `yield`, arbitrary self types, unsized locals, specialization, trait aliases, negative bounds, generic const expressions, const traits, portable SIMD, and try blocks remain a moving nightly frontier and should never be assumed stable from this report.

No project theme, fixture topology, scoring model, or analysis implementation is prescribed here.
