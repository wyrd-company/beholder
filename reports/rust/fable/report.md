# Rust — Research Checklist for a Code-Graph Reference Project

Independent research pass. Purpose: enumerate the language, toolchain, build, package, and
ecosystem features of Rust whose omission from a reference project would leave materially
distinct semantic entities, identities, relationships, resolution behavior, types,
visibility, dispatch, source-inclusion decisions, or diagnostics unrepresented.

This report defines no fixture layout, project theme, schema, or scoring system. It is a
candidate checklist with rationale.

---

## 1. Baseline assumptions

- **Language definition.** Rust has no ISO standard. Semantics are defined by The Rust
  Reference plus the rustc stabilization process; `rustc` is the de-facto sole complete
  implementation (gccrs is incomplete). The Ferrocene Language Specification exists as a
  rustc-derived specification effort. Items below are marked "language" when the Reference
  documents them, "official implementation" when rustc/Cargo behavior is the definition,
  and "ecosystem" for conventions.
- **Toolchain baseline.** Stable `rustc`/`cargo` ≥ 1.85 (the release that shipped edition
  2024), assuming a late-2025 stable (~1.9x). All four editions (2015, 2018, 2021, 2024)
  are usable simultaneously, selected per crate.
- **Channels.** Stable behavior only, except where an item explicitly flags nightly-only
  features (which matter as invalid-on-stable cases).
- **Platform.** Any tier-1 target; platform variation is treated via `cfg` and
  cross-compilation items rather than a fixed host.
- **Package manager.** Cargo is assumed as the build driver. Direct `rustc` invocation is
  covered as a separate item because crate name, edition, cfgs, and extern crates are all
  invocation inputs, not source properties.

Checklist item fields: **Behavior** (the distinct semantics), **Observable content** (what
a project would contain to demonstrate it), **Variants & failures**, **Constraints**, and a
combined line for defined-by / confidence / primary verification source. Identifiers
`RS-<CAT>-<NN>` are stable within this report only.

---

## 2. Checklist

### 2.1 Package, crate, and target structure (RS-PKG)

#### RS-PKG-01 · Package vs. crate; name normalization
- **Behavior:** One Cargo package produces many crates (lib, each bin, each test/example/bench, the build script). Package names may contain hyphens; the library crate name is the package name with `-` mapped to `_`, unless overridden by `[lib] name`. The name used in `use` paths is therefore not the registry/package name.
- **Observable content:** A package `demo-core` consumed as `demo_core`; a second package overriding `[lib] name = "altname"` and consumed under that name.
- **Variants & failures:** Binary target names keep hyphens (they are not identifiers); consumers referring to the package name instead of the crate name fail to resolve.
- **Constraints:** Cargo manifest.
- **Defined by:** official implementation (Cargo) · **Confidence:** high · **Verify:** The Cargo Book.

#### RS-PKG-02 · Target auto-discovery and overrides
- **Behavior:** Cargo infers targets from conventional paths: `src/lib.rs`, `src/main.rs`, `src/bin/*.rs` and `src/bin/<name>/main.rs`, `examples/`, `tests/*.rs` and `tests/<dir>/main.rs`, `benches/`. `autobins`/`autotests`/`autoexamples`/`autobenches` disable discovery; explicit `[[bin]]`/`[[test]]` tables with `path` mount arbitrary files as targets.
- **Observable content:** A package exercising every discovery form plus one explicit-path target outside the conventional tree.
- **Variants & failures:** A broken file under `src/bin/` fails the whole build; a `.rs` file reachable by no target is silently not compiled (see RS-MOD-05).
- **Constraints:** Cargo.
- **Defined by:** official implementation (Cargo) · **Confidence:** high · **Verify:** The Cargo Book (Cargo targets chapter).

#### RS-PKG-03 · Library + binaries in one package
- **Behavior:** Bins in the same package link the package's lib as an *external crate* (`use demo_core::...`), producing a cross-crate edge inside a single package. Alternatively (an anti-pattern) a bin can `mod`-include the same source files, duplicating entities into a second crate.
- **Observable content:** `src/main.rs` importing the lib by crate name; a deliberate counterexample where lib and bin both declare `mod util;` over one file.
- **Variants & failures:** Types from the mod-included copy do not unify with the lib's types — "expected `util::T`, found `util::T`" diagnostics.
- **Constraints:** none beyond Cargo layout.
- **Defined by:** official implementation · **Confidence:** high · **Verify:** The Cargo Book.

#### RS-PKG-04 · Crate types
- **Behavior:** `crate-type` selects `rlib` (default), `dylib`, `cdylib`, `staticlib`, `proc-macro`; a lib may declare several at once. `cdylib`/`staticlib` define an exported-symbol surface distinct from the Rust API; `proc-macro` crates export only macros and compile for the host.
- **Observable content:** A lib with `crate-type = ["rlib", "cdylib"]` and `#[no_mangle] pub extern "C"` exports; a separate proc-macro crate.
- **Variants & failures:** Depending on a `proc-macro` crate for ordinary items fails; `dylib` vs `cdylib` symbol/ABI differences.
- **Constraints:** manifest + linker.
- **Defined by:** language (Reference: linkage) + Cargo · **Confidence:** high · **Verify:** The Rust Reference (Linkage), The Cargo Book.

#### RS-PKG-05 · Crate-root inner attributes
- **Behavior:** `#![...]` attributes at the crate root set crate-wide semantics: `#![no_std]`, lint levels, `#![recursion_limit]`, `#![doc(...)]`, `#![feature(...)]` (nightly-only; a hard error on stable).
- **Observable content:** Crate roots carrying distinct inner-attribute sets; one file demonstrating that inner attributes are only legal before items.
- **Variants & failures:** `#![feature]` on stable is E0554-class rejection — a version/channel-dependent validity case.
- **Constraints:** crate root placement.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Attributes).

#### RS-PKG-06 · Entry points and `main`
- **Behavior:** Binaries require `fn main` (or `#[no_main]`) at the crate root; `main` may return any `Termination` type (`()`, `Result<(), E>`). Attribute macros commonly replace `main` (see RS-ECO-03). Test targets get a synthesized harness `main`.
- **Observable content:** A bin with `fn main() -> Result<(), Box<dyn Error>>`; a bin whose `main` is produced by an attribute macro.
- **Variants & failures:** `main` defined only in a submodule → missing entry point error; `#[no_main]` for embedded-style targets.
- **Constraints:** bin targets only.
- **Defined by:** language + std (`Termination`) · **Confidence:** high · **Verify:** The Rust Reference (Crates and source files).

#### RS-PKG-07 · Secondary targets are distinct crates
- **Behavior:** Each test, example, and bench file is compiled as its own crate that links the lib externally. Same-named modules in different targets are distinct entities; dev-dependencies are visible only to these targets.
- **Observable content:** An example and an integration test each defining `mod support;` with same-named items; a dev-dependency used only there.
- **Variants & failures:** Referring to lib internals (non-`pub`) from these targets fails, unlike unit tests.
- **Constraints:** Cargo target model.
- **Defined by:** official implementation · **Confidence:** high · **Verify:** The Cargo Book.

### 2.2 Modules and source inclusion (RS-MOD)

#### RS-MOD-01 · Module-to-file mapping styles
- **Behavior:** `mod m;` loads `m.rs` (2018 style, children in `m/`) or `m/mod.rs` (2015 style). Inline `mod m { ... }` declares the same kind of entity with no file. Both file styles can coexist in one crate.
- **Observable content:** A crate mixing inline modules, `name.rs` + `name/` children, and a `mod.rs` directory module.
- **Variants & failures:** Providing both `m.rs` and `m/mod.rs` is an error; declaring `mod m;` with neither file present is E0583-class.
- **Constraints:** none.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Modules).

#### RS-MOD-02 · `#[path]` remapping
- **Behavior:** `#[path = "..."]` mounts an arbitrary file as a module, decoupling module tree from directory tree; on a nested `mod`, it also changes where children are sought.
- **Observable content:** A module loaded from a non-conventional path; a cfg-gated pair of `#[path]` declarations choosing platform files.
- **Variants & failures:** `#[path]` on an inline module changes the base directory for its file-backed children.
- **Constraints:** path relative to the declaring file's directory context.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Modules / path attribute).

#### RS-MOD-03 · One file mounted at multiple module paths
- **Behavior:** The same source file can be compiled more than once — via two `#[path]` mods, via `include!`, or via lib and bin both declaring it. Each mount is a distinct set of entities; textual identity does not imply semantic identity.
- **Observable content:** One file mounted twice with a type that consequently exists as two incompatible types.
- **Variants & failures:** Cross-mount type mismatch diagnostics; impl coherence counted per mount (duplicate impls of an external trait for the "same" type do not collide because the types differ).
- **Constraints:** none.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Modules).

#### RS-MOD-04 · `include!` family
- **Behavior:** `include!` splices tokens from another file into the invocation site (entities belong to the includer's module); `include_str!`/`include_bytes!` embed data only and create no entities but do create build-input file dependencies.
- **Observable content:** An `include!`d item file (the canonical `OUT_DIR` pattern, see RS-GEN-02) plus an `include_str!` data file.
- **Variants & failures:** Paths inside an included file resolve against the including context (subtle with nested `mod` declarations — worth a deliberate probe); including a file that is also mounted as a module duplicates entities.
- **Constraints:** included content must parse in the invocation position.
- **Defined by:** language (built-in macros) · **Confidence:** high (medium for nested-`mod`-in-include path base) · **Verify:** std documentation for `include!`.

#### RS-MOD-05 · Module tree defines source inclusion
- **Behavior:** Files not reachable through `mod`/`include!`/target discovery are not compiled at all. The compiled source set is a resolution product, not a directory listing.
- **Observable content:** An orphan `.rs` file containing invalid code that does not affect the build.
- **Variants & failures:** The same file becoming compiled (and failing) once a `mod` is added; cfg'd-off `mod` statements excluding whole subtrees per configuration.
- **Constraints:** none.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Modules).

#### RS-MOD-06 · cfg-selected module alternatives
- **Behavior:** `#[cfg(unix)] mod sys; #[cfg(windows)] #[path = "sys_windows.rs"] mod sys;` yields one module identity per configuration with same-named items of different definitions.
- **Observable content:** A platform-split module pair exposing an identical API from different files, consumed by common code.
- **Variants & failures:** Feature-split (not just platform-split) variants; both-disabled configurations breaking downstream resolution.
- **Constraints:** cfg evaluation at build time.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Conditional compilation).

#### RS-MOD-07 · Items inside function bodies
- **Behavior:** Functions, structs, impls, and `use` declarations may be declared inside a body; they are real items scoped to that body and invisible outside. Trait impls declared inside bodies still participate in crate-global coherence and can be found by method resolution.
- **Observable content:** A nested `fn` and a body-local `struct`; a body-local trait impl observed from outside the body via method call (as a deliberate probe).
- **Variants & failures:** Body-local `use` not leaking; shadowing of outer items by body-local items.
- **Constraints:** none.
- **Defined by:** language · **Confidence:** high (medium for the body-local-impl global coherence detail) · **Verify:** The Rust Reference (Items).

### 2.3 Name resolution, imports, and namespaces (RS-RES)

#### RS-RES-01 · `use` declaration forms
- **Behavior:** Simple imports, `as` renames, nested groups `use a::{b, c::{d, e as f}}`, glob `use m::*`, and `use path as _`. Imports are scoped to the containing module and are not inherited by child modules.
- **Observable content:** All forms exercised; a child module failing (or re-importing) a name its parent imported.
- **Variants & failures:** `use` of an item in multiple namespaces imports all of them; self-imports `use m::{self, item}`.
- **Constraints:** edition affects path grammar (see RS-EDI-02).
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Use declarations).

#### RS-RES-02 · Re-exports and multiple public paths
- **Behavior:** `pub use` creates additional public paths to one entity, including renamed and glob re-exports, chains across crates, and `pub extern crate`. One entity may have many valid paths; rustdoc chooses a canonical presentation influenced by `#[doc(inline)]`/`#[doc(no_inline)]`.
- **Observable content:** A deep item re-exported at the crate root; a renamed re-export; a re-export of a dependency's type into this crate's API.
- **Variants & failures:** Non-`pub` `use` is nameable in-module but is not API; glob re-export of a module merging into another namespace.
- **Constraints:** visibility of the re-export governs reachability.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Use declarations, Visibility).

#### RS-RES-03 · Three item namespaces
- **Behavior:** Types, values, and macros occupy separate namespaces. One identifier can denote several entities simultaneously: a unit/tuple struct occupies type and value namespaces (its constructor is a value); `Vec` (type) and `vec!` (macro) coexist; a `const` and a `struct` may share a name.
- **Observable content:** A module where one name resolves to different entities in type, value, and macro positions.
- **Variants & failures:** Importing one name brings all namespaces' bindings; conflicts are per-namespace (a fn and a struct with the same name collide only in the value namespace via the constructor).
- **Constraints:** none.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Names/Namespaces).

#### RS-RES-04 · Glob-import ambiguity and shadowing
- **Behavior:** Two glob imports supplying the same name are legal until the name is *used*, then it is an ambiguity error. An explicit (non-glob) import or local item shadows glob-supplied names.
- **Observable content:** Two globs with an overlapping unused name (valid); the same with a use site (invalid); an explicit import resolving the ambiguity.
- **Variants & failures:** Glob importing an enum's variants `use Enum::*`; glob vs. prelude interaction.
- **Constraints:** none.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Use declarations).

#### RS-RES-05 · Preludes and prelude shadowing
- **Behavior:** Names resolve against layered implicit scopes: the edition-specific std prelude, the extern prelude (direct dependencies nameable without any `use`), and the macro prelude. Local definitions shadow prelude names.
- **Observable content:** Code using `Option`/`String` with no imports; a module defining its own `Result` alias shadowing the prelude; a dependency referenced with no `use` at all via the extern prelude.
- **Variants & failures:** `#[no_implicit_prelude]` module requiring fully explicit paths; edition-dependent prelude contents (RS-EDI-03/04).
- **Constraints:** edition.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Preludes).

#### RS-RES-06 · Path qualifiers
- **Behavior:** `crate::`, `self::`, `super::` (chainable: `super::super::`), leading `::` (forces extern-prelude resolution in 2018+), and `Self` as a type/constructor alias within impls and traits.
- **Observable content:** Sibling-module references via `super`, absolute references via `crate`, a `::dep::item` disambiguation against a local module named like a dependency, `Self { .. }` construction in an impl.
- **Variants & failures:** A local module named identically to a crate forcing `::` or `crate::` disambiguation.
- **Constraints:** edition (2015 leading-`::` means crate root).
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Paths).

#### RS-RES-07 · Raw identifiers
- **Behavior:** `r#type`, `r#async` allow keyword-named entities; needed when consuming crates written before an identifier became a keyword (edition interplay).
- **Observable content:** An item named with a raw identifier and a use site; a 2015-edition crate exposing a then-legal name consumed from 2021+ via `r#`.
- **Variants & failures:** `r#` on non-keywords is allowed; some keywords (`crate`, `self`, `super`, `Self`) cannot be raw identifiers.
- **Constraints:** edition keyword sets differ.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Identifiers).

#### RS-RES-08 · `extern crate` forms
- **Behavior:** Unnecessary since 2018 except for `extern crate alloc` (no_std), `extern crate test`, `extern crate proc_macro`. `extern crate foo as bar;` renames at declaration; `extern crate self as name;` gives a crate a self-alias (used by some macro-authoring crates).
- **Observable content:** A no_std crate with `extern crate alloc`; a legacy 2015-edition member using `extern crate` + `#[macro_use]`.
- **Variants & failures:** Redundant `extern crate` in 2018+ lints; `#[macro_use] extern crate` importing macros wholesale (contrast with path-based macro imports).
- **Constraints:** edition.
- **Defined by:** language · **Confidence:** high (medium for `self as`) · **Verify:** The Rust Reference (Extern crate declarations).

#### RS-RES-09 · Trait-scope imports change call resolution
- **Behavior:** Method-call syntax only considers trait methods whose trait is in scope. `use some::Trait as _;` enables the methods without binding the name. An import is thus a resolution-affecting edge, not merely a naming convenience.
- **Observable content:** A method call that compiles only with the trait imported; the `as _` form; the failure case without the import (E0599-class with "trait is not in scope" help).
- **Variants & failures:** Two imported traits creating ambiguity (see RS-DSP-02).
- **Constraints:** none.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Method-call expressions).

#### RS-RES-10 · Intra-doc links as a resolution surface
- **Behavior:** Rustdoc resolves paths inside doc comments (`[Vec]`, `[crate::x::Y]`, `[Self::method]`) using normal name resolution; broken links are linted.
- **Observable content:** Doc comments with resolved intra-doc links across modules and to re-exports; one deliberately broken link (lint case).
- **Variants & failures:** Namespace disambiguators (`[fn@name]`, `[struct@name]`); links resolving through re-exports.
- **Constraints:** rustdoc runs.
- **Defined by:** official implementation (rustdoc) · **Confidence:** high · **Verify:** The rustdoc Book.

#### RS-RES-11 · Non-ASCII identifiers and normalization
- **Behavior:** Unicode identifiers are legal (since 1.53); rustc normalizes identifiers to NFC, so distinct encodings of the same visual name are one entity. Confusable/mixed-script lints exist.
- **Observable content:** An item with a non-ASCII name and a use site; optionally NFC/NFD source variants demonstrating unification.
- **Variants & failures:** `uncommon_codepoints`/confusables lints; crate names remain ASCII-constrained on crates.io (ecosystem constraint).
- **Constraints:** rustc ≥ 1.53.
- **Defined by:** language · **Confidence:** medium (normalization details) · **Verify:** The Rust Reference (Identifiers).

### 2.4 Visibility and privacy (RS-VIS)

#### RS-VIS-01 · Visibility levels
- **Behavior:** Default privacy (visible to the defining module and its descendants), `pub`, `pub(crate)`, `pub(super)`, `pub(in path)` (path must be an ancestor module).
- **Observable content:** One item at each level with in-bounds and out-of-bounds access attempts (the latter as invalid-program cases, E0603-class).
- **Variants & failures:** `pub(in ...)` with a non-ancestor path is rejected.
- **Constraints:** none.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Visibility and privacy).

#### RS-VIS-02 · Reachability vs. nameability
- **Behavior:** A `pub` item inside a private module is not nameable externally unless re-exported; re-exports widen effective reachability. Items can also be *reachable but unnameable* (leaked through a `pub` signature returning a type from a private module).
- **Observable content:** A pub-in-private item with a crate-root re-export; a pub fn returning a pub-in-private type with no re-export; the `unreachable_pub` lint enabled on a demonstration item.
- **Variants & failures:** Downstream code able to call methods on a value whose type it cannot name.
- **Constraints:** none.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Visibility and privacy).

#### RS-VIS-03 · Field and constructor visibility
- **Behavior:** Struct fields have individual visibility; any private field blocks literal construction and functional-update syntax outside its module. A tuple struct's constructor function's visibility is limited by its fields' visibility.
- **Observable content:** A struct with mixed-visibility fields plus a constructor fn; a tuple struct whose type is `pub` but whose constructor is unusable externally due to a private field.
- **Variants & failures:** `..Default::default()` update syntax also blocked by private fields; `#[non_exhaustive]` producing similar cross-crate effects without privacy (RS-ATT-05).
- **Constraints:** none.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Visibility).

#### RS-VIS-04 · Visibility of trait items, variants, impls
- **Behavior:** Trait associated items inherit the trait's visibility (no per-item `pub`); enum variants and their fields are always as public as the enum; impl blocks carry no visibility, but inherent-impl associated items have their own.
- **Observable content:** A trait with externally callable items; an enum whose variant fields are matched cross-module; an inherent impl mixing `pub` and private methods.
- **Variants & failures:** Attempting `pub` on a trait method is rejected.
- **Constraints:** none.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Visibility).

#### RS-VIS-05 · Private-interface diagnostics
- **Behavior:** Exposing a private type in a more-public signature triggers `private_interfaces`/`private_bounds` lints (historically hard errors, E0446 lineage) — a diagnostic family tied directly to graph visibility relationships.
- **Observable content:** A `pub fn` taking a private type; a `pub` trait bound mentioning a private trait.
- **Variants & failures:** The deliberate, sanctioned version of this is the sealed-trait pattern (RS-ECO-07).
- **Constraints:** lint levels affect validity (RS-ATT-01).
- **Defined by:** language + official implementation (lint machinery) · **Confidence:** medium (current lint names/levels) · **Verify:** rustc lint documentation.

### 2.5 Items and definitions (RS-ITM)

#### RS-ITM-01 · Struct forms
- **Behavior:** Named-field, tuple, and unit structs; generic structs with parameter defaults; zero-sized types; `PhantomData` members carrying type/lifetime relationships without data.
- **Observable content:** All three forms, one generic with a default, one PhantomData-parameterized marker type.
- **Variants & failures:** Unit struct occupying both type and value namespaces (RS-RES-03).
- **Constraints:** none.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Structs).

#### RS-ITM-02 · Enums and variants
- **Behavior:** Unit, tuple, and struct variants; explicit discriminants; `repr(<int>)` enums; variant paths `Enum::V` and `Self::V`; tuple variants as constructor functions (values); `use Enum::*` importing variants.
- **Observable content:** An enum mixing variant forms with an explicit discriminant; a match using globbed variants; a tuple variant passed as a function value (`list.map(Enum::V)`).
- **Variants & failures:** Discriminant collisions are errors; `as` casts of fieldless enums.
- **Constraints:** none.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Enumerations).

#### RS-ITM-03 · Unions
- **Behavior:** `union` items; fields must be `Copy` or wrapped in `ManuallyDrop`; field reads are `unsafe`. A distinct item kind with its own access semantics.
- **Observable content:** A small FFI-style union with an unsafe read and a match-free access pattern.
- **Variants & failures:** Non-Copy field without `ManuallyDrop` is rejected; pattern matching on unions is restricted.
- **Constraints:** none.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Unions).

#### RS-ITM-04 · Constants vs. statics
- **Behavior:** `const` items are inlined per use (no address identity); `static` items are single memory locations; `static mut` exists but access is unsafe and taking references to it is denied in edition 2024. `const _: () = ...;` provides unnamed compile-time assertions. Associated consts exist on traits (with defaults) and impls.
- **Observable content:** A const and a static with address-comparison commentary; a `const _` static assertion; a trait associated const overridden in one impl and defaulted in another.
- **Variants & failures:** `static_mut_refs` (edition-dependent severity); interior-mutability statics (`AtomicU32`, `Mutex`) as the sanctioned pattern.
- **Constraints:** edition 2024 changes `static mut` handling.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Constant items, Static items).

#### RS-ITM-05 · Type aliases vs. newtypes
- **Behavior:** `type` aliases are transparent — no new nominal type, no own impls; a tuple-struct newtype is a distinct nominal type. Aliases can be generic and can name trait-object types. An `impl` written on an alias attaches to the aliased type.
- **Observable content:** An alias and a newtype over the same underlying type, showing interchangeability vs. incompatibility; a generic alias; `type Callback = Box<dyn Fn(u32)>`.
- **Variants & failures:** Attempting to implement a foreign trait "for the alias" hits coherence exactly as for the underlying type.
- **Constraints:** lazy alias checking (some invalid aliases only error at use).
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Type aliases).

#### RS-ITM-06 · Trait declarations
- **Behavior:** Traits with required methods, default method bodies, associated types (bounds allowed; *defaults* are nightly-only), associated consts, and supertraits. Trait items are entities whose "owner" may be the trait (default body) or an impl (override).
- **Observable content:** A trait exercising each associated-item kind; a supertrait chain; a caller using a supertrait method through a subtrait bound.
- **Variants & failures:** Associated-type defaults as an invalid-on-stable case.
- **Constraints:** stable vs. nightly for associated-type defaults.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Traits).

#### RS-ITM-07 · Inherent impls: placement freedom, locality rule
- **Behavior:** Multiple inherent `impl` blocks for one type are allowed and may live in different modules/files of the defining crate. Inherent impls for non-local types are rejected (E0116-class) — method definitions can be far from the type but never outside its crate.
- **Observable content:** A type with impl blocks in two files; an invalid-program case attempting an inherent impl on `Vec<T>`.
- **Variants & failures:** The extension-trait workaround (RS-ECO-07).
- **Constraints:** coherence.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Implementations).

#### RS-ITM-08 · Generic parameters and where clauses
- **Behavior:** Type, lifetime, and const generic parameters (integers, `bool`, `char`); defaults on type and const params; `where` clauses including bounds on associated types (`where T::Item: Clone`); higher-ranked bounds `for<'a>`; named lifetime parameters as declared entities.
- **Observable content:** Functions/types/impls exercising each parameter kind, a const-generic array wrapper, an HRTB bound on a closure parameter.
- **Variants & failures:** Const-generic expressions beyond plain parameters are restricted on stable (`generic_const_exprs` nightly); defaults interact with inference.
- **Constraints:** const generics restricted to certain types on stable.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Generic parameters).

#### RS-ITM-09 · Compile-time evaluation sites
- **Behavior:** `const fn` calls execute at compile time in const contexts: array lengths, enum discriminants, const-generic arguments, `const`/`static` initializers, inline `const { ... }` blocks (1.79+). These are call edges that execute during compilation; failures (panics in const eval) are compile errors, sometimes only post-monomorphization.
- **Observable content:** A `const fn` used in an array length and a const-generic argument; an inline const block; a `const _` assertion that fails when a parameter is wrong (invalid-program case).
- **Variants & failures:** Generic const eval errors surfacing only when instantiated; floats in const contexts (historically restricted).
- **Constraints:** rustc ≥ 1.79 for inline const.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Const evaluation).

#### RS-ITM-10 · Foreign declarations (extern blocks)
- **Behavior:** `extern "C" { fn f(...); static S: T; }` declares entities with no Rust body; identity ties to a linker symbol, not to source. See RS-UNS for the safety/linkage details.
- **Observable content:** An extern block with a fn and a static, plus Rust-side callers.
- **Variants & failures:** Mismatched declaration vs. actual symbol type is not detected by rustc (link-time or UB) — a class of incorrectness invisible to the compiler.
- **Constraints:** edition 2024 requires `unsafe extern`.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (External blocks).

### 2.6 Types, inference, and coercions (RS-TYP)

#### RS-TYP-01 · Inference and literal fallback
- **Behavior:** Types flow bidirectionally within function bodies; unannotated integer literals fall back to `i32`, floats to `f64`. Return-type-directed selection picks impls (`.into()`, `.parse()`, `.collect()`), so the *target type context* decides which impl/edge is used.
- **Observable content:** A `collect::<Vec<_>>()` turbofish, a `let x: HashMap<_,_> = ... .collect()`, an `.into()` whose target type selects among several `From` impls; an ambiguity error case with no annotation (E0282-class).
- **Variants & failures:** Method calls on unconstrained numeric literals (`1.pow(2)` style ambiguity).
- **Constraints:** none.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Type inference is largely rustc-documented; the Book covers fallback).

#### RS-TYP-02 · Coercions
- **Behavior:** Implicit coercions change the type an edge lands on: deref coercion (`&String → &str`, chained), unsizing (`[T; N] → [T]`, `T → dyn Trait`), fn item → fn pointer, non-capturing closure → fn pointer, `&mut T → &T`.
- **Observable content:** Call sites relying on each coercion, including a `Box<MyType> → Box<dyn Trait>` assignment and passing a fn item where a fn pointer is expected.
- **Variants & failures:** Coercion happens at typed "coercion sites" only — an equivalent expression outside such a site fails.
- **Constraints:** none.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Type coercions).

#### RS-TYP-03 · Variance and PhantomData
- **Behavior:** Generic-parameter variance is inferred from field usage; `PhantomData<T>` contributes variance, drop-check, and auto-trait effects with no runtime data — an invisible-but-semantic member.
- **Observable content:** A covariant wrapper vs. an invariant one (via `PhantomData<fn(T) -> T>` or `*mut T`), with a borrow-check consequence in test code.
- **Variants & failures:** `PhantomData<*const T>` removing `Send`/`Sync` (interacts with RS-TRT-05).
- **Constraints:** none.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rustonomicon (variance, PhantomData).

#### RS-TYP-04 · `Sized` defaults and `?Sized`
- **Behavior:** Every generic parameter has an implicit `Sized` bound; `?Sized` removes it, enabling impls and functions over `str`, `[T]`, and `dyn Trait`.
- **Observable content:** A generic impl with `T: ?Sized` used with `str`; the failing counterpart without `?Sized`.
- **Variants & failures:** `Self: Sized` on trait methods as a dyn-compatibility escape hatch (RS-TRT-08).
- **Constraints:** none.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Sized).

#### RS-TYP-05 · Lifetime elision
- **Behavior:** Elided lifetimes in fn signatures follow fixed rules (one input → outputs; `&self` wins); `'_` is an explicit elision marker; trait objects get default lifetimes by context (`Box<dyn Trait>` defaults to `'static`). Entities exist in the semantic signature that the source never names.
- **Observable content:** Functions where elided and fully explicit signatures pair up; a `Box<dyn Trait + '_>` where the default would be wrong.
- **Variants & failures:** Elision failure requiring explicit annotation (multiple inputs, no self); impl-header elision.
- **Constraints:** none.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Lifetime elision).

#### RS-TYP-06 · Never type and diverging expressions
- **Behavior:** `!` (and diverging expressions: `panic!`, `return`, `loop`) coerce to any type, letting match arms and branches unify. Never-type *fallback* details are version-sensitive and partially unstable.
- **Observable content:** A match arm returning `panic!(...)` unified against a concrete type; a fn declared `-> !`.
- **Variants & failures:** Fallback interactions with `?` and trait selection (flag as version-sensitive rather than fixture-critical).
- **Constraints:** full `!` type stabilization is incomplete.
- **Defined by:** language · **Confidence:** medium · **Verify:** The Rust Reference (Never type).

### 2.7 Traits, impls, and coherence (RS-TRT)

#### RS-TRT-01 · Trait impls are unnamed, crate-global items
- **Behavior:** `impl Trait for Type` may appear in any module of a crate; the relationship holds crate-wide (and downstream) regardless of visibility or imports of the module containing the impl. Impl blocks have no name — identity is (trait, self type, generic arguments).
- **Observable content:** An impl placed in a private submodule far from both trait and type, exercised from elsewhere.
- **Variants & failures:** Two impls of the same (trait, type) pair anywhere in the crate collide (E0119-class).
- **Constraints:** coherence.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Implementations).

#### RS-TRT-02 · Conditional (bounded) impls
- **Behavior:** `impl<T: Clone> Clone for Wrap<T>` makes the relationship hold only for instantiations satisfying the bound — trait membership is conditional on type arguments.
- **Observable content:** `Wrap<String>` usable as `Clone`, `Wrap<NonClone>` not (with the failing case shown).
- **Variants & failures:** Multiple conditional impls with disjoint bounds; interaction with derive-generated bounds (RS-TRT-06).
- **Constraints:** none.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Implementations).

#### RS-TRT-03 · Blanket impls
- **Behavior:** `impl<T: Display> MyTrait for T` implements a trait for an open-ended set of types, including types in other crates and types defined later.
- **Observable content:** A blanket impl plus call sites through it on local and std types; a downstream crate gaining the impl "for free".
- **Variants & failures:** A blanket impl blocks any other impl of that trait for any type (coherence overlap) — a canonical invalid-program companion case.
- **Constraints:** coherence.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Coherence rules in Implementations).

#### RS-TRT-04 · Orphan rule and overlap
- **Behavior:** An impl is legal only if the trait or the self type is local, with refinements for generic covering types and `#[fundamental]` types (`&T`, `&mut T`, `Box<T>` are fundamental — `impl LocalTrait for Box<ForeignType>` counts differently than for `Arc<ForeignType>`). Overlapping impls are rejected.
- **Observable content:** Valid local-trait-for-foreign-type and foreign-trait-for-local-type impls; invalid foreign-trait-for-foreign-type (E0117-class); an overlap error case (E0119-class); a newtype workaround.
- **Variants & failures:** Fundamental-type edge cases; the fact that adding a blanket impl upstream is a breaking change.
- **Constraints:** coherence; exact orphan-rule refinements are subtle.
- **Defined by:** language · **Confidence:** high (medium on fundamental-type fine print) · **Verify:** The Rust Reference; RFC 1023 discussion of coherence (title from memory — verify).

#### RS-TRT-05 · Auto traits
- **Behavior:** `Send`, `Sync`, `Unpin` (and the panic-safety traits) are implemented structurally without impl blocks: a type has them iff its components do. Raw-pointer fields or `PhantomData<*const T>` remove them; `unsafe impl Send for X {}` adds them back explicitly. These are relationships with no source-level impl to point at.
- **Observable content:** A type made non-`Send` by a field; a manual `unsafe impl Send`; a generic type whose `Send`-ness depends on `T` (`Send where T: Send` synthesized).
- **Variants & failures:** Negative impls (`impl !Send for X`) are nightly; a `spawn` call failing because a captured value is non-`Send` (diagnostic case, ties to RS-CLO-05).
- **Constraints:** stable set of auto traits is fixed; user auto traits are nightly.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Auto traits / Special types and traits).

#### RS-TRT-06 · Derived impls and their bounds
- **Behavior:** `#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]` generates trait impls at expansion time. Std derives add `T: Trait` bounds on every generic parameter even when not strictly needed (the "perfect derive" caveat) — the generated relationship is conditional in a way the source doesn't show.
- **Observable content:** Derives on generic and non-generic types; a case where `Wrap<Rc<T>>: Clone` fails to apply because the derive bound requires `T: Clone` though only `Rc<T>: Clone` is needed.
- **Variants & failures:** `Copy` requiring `Clone`; `Eq` requiring `PartialEq`; derive on enums generating per-variant logic.
- **Constraints:** none.
- **Defined by:** official implementation (std derive expansion; exact bounds are impl-defined convention) · **Confidence:** high · **Verify:** std documentation for the derive macros.

#### RS-TRT-07 · Default methods: inherited vs. overridden
- **Behavior:** A trait method with a default body is provided by the trait unless an impl overrides it; calls through the trait dispatch to the override when present. The defining entity of the executed body varies per impl.
- **Observable content:** Two impls of one trait, one overriding the default and one inheriting it, with calls to both.
- **Variants & failures:** Default bodies calling required methods (template-method shape); supertrait default methods reached via subtrait bounds.
- **Constraints:** none.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Traits).

#### RS-TRT-08 · Trait objects and dyn-compatibility
- **Behavior:** `dyn Trait` erases the concrete type behind a vtable. Only dyn-compatible ("object-safe") traits qualify: no generic methods, no by-value `Self` receivers/returns, no associated consts, etc.; `where Self: Sized` exempts an offending method. `dyn Trait + Send + 'static` combines one principal trait with auto traits and a lifetime. Upcasting `dyn Sub → dyn Super` is allowed (stable since 1.86).
- **Observable content:** A dyn-compatible trait used via `Box<dyn T>`; a trait made compatible via `Self: Sized` on one method; an invalid `dyn` of an incompatible trait (E0038-class); an upcast coercion.
- **Variants & failures:** Two non-auto traits in one `dyn` are rejected; `dyn` with associated types requires binding them (`dyn Iterator<Item = u8>`).
- **Constraints:** rustc ≥ 1.86 for upcasting.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Trait objects / dyn compatibility).

#### RS-TRT-09 · Associated-type projection and GATs
- **Behavior:** `<T as Iterator>::Item` names a type through a trait bound; bounds can constrain projections (`T: Iterator<Item = u8>` or `where T::Item: Clone`). Generic associated types (stable 1.65) add generic parameters (usually lifetimes) to associated types.
- **Observable content:** Signatures using projections both in shorthand and fully-qualified form; a small GAT (`type View<'a>`) with an impl and a use site.
- **Variants & failures:** Ambiguous projection when several bounds supply `Item`; associated-type binding syntax in `dyn` and `impl Trait`.
- **Constraints:** GATs 1.65+, with known borrow-checker limitations.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Associated items).

#### RS-TRT-10 · Behavior-bearing std traits
- **Behavior:** Impls of `Add`, `Index`, `Deref`, `Drop`, `From`, `Iterator`, `Display`, etc. change the meaning of *other* code (operators, `for`, `?`, formatting) — writing an impl here creates call edges at distant expression sites (see RS-IMP).
- **Observable content:** One custom impl of each family with the corresponding sugar exercised: `a + b`, `x[i]`, `*p`, implicit drop, `?` conversion, `for` loop, `{}` formatting.
- **Variants & failures:** `Deref` targeting a type with conflicting method names (RS-DSP-05); `Drop` types forbidden `Copy`.
- **Constraints:** operator traits are fixed by the language; no custom operators.
- **Defined by:** language + std · **Confidence:** high · **Verify:** std `ops` module documentation.

#### RS-TRT-11 · Impls on non-nominal self types
- **Behavior:** Impls may target references (`impl Trait for &T`), slices, arrays (generically over length via const generics), tuples, and primitives — subject to coherence. The self type of an impl is a type expression, not necessarily a named item.
- **Observable content:** A local trait implemented for `&LocalType`, `[LocalType]`, `(A, B)`, and `u32`; method calls resolving differently for `T` vs `&T` impls.
- **Variants & failures:** `impl Trait for &T` vs `impl Trait for T` both existing, with autoref choosing between them (RS-DSP-01).
- **Constraints:** coherence.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Implementations).

#### RS-TRT-12 · Nightly-only trait system features (as exclusions)
- **Behavior:** Specialization, negative impls, trait aliases, user-defined auto traits, and associated-type defaults are nightly-only. Their presence makes a program invalid on the stable baseline.
- **Observable content:** At most a quarantined, clearly-marked nightly sample or a commented exclusion note; a stable-build failure case if included.
- **Variants & failures:** Ecosystem code approximating specialization via autoref-based "specialization" hacks (macro trick) — worth one sample as it affects method resolution.
- **Constraints:** channel.
- **Defined by:** official implementation (unstable features) · **Confidence:** high · **Verify:** The Unstable Book.

### 2.8 Method resolution and dispatch (RS-DSP)

#### RS-DSP-01 · Method probe order
- **Behavior:** `x.m()` searches candidate receiver types along the autoderef chain (`T`, `&T`, `&mut T` at each deref step), preferring inherent methods over trait methods at each step. Which impl a call lands on depends on this order; adding an inherent method silently shadows a trait method of the same name.
- **Observable content:** A type with an inherent `get` and a trait `get`, showing the inherent one wins and the trait one needs UFCS; a `Deref` chain where a method resolves on the target type.
- **Variants & failures:** `&T`-impl vs `T`-impl selection by autoref; edition-sensitive cases (RS-DSP-06).
- **Constraints:** none.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Method-call expressions).

#### RS-DSP-02 · Ambiguity and fully qualified syntax
- **Behavior:** Two in-scope traits with the same method on one receiver produce E0034-class ambiguity; resolution requires `Trait::m(&x)` or `<T as Trait>::m(&x)`. Fully qualified syntax also disambiguates associated consts/types and same-named inherent vs. trait items.
- **Observable content:** The ambiguous call (invalid case) plus both disambiguated forms; a `<T as Default>::default()` style associated-function call.
- **Variants & failures:** Ambiguity appearing only when a second `use` is added — an import-induced breakage case.
- **Constraints:** none.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Paths / qualified paths).

#### RS-DSP-03 · Static vs. dynamic dispatch
- **Behavior:** A generic call `fn f<T: Trait>(t: T)` monomorphizes — conceptually one function per instantiation, resolved statically; `fn f(t: &dyn Trait)` dispatches through a vtable at runtime. Identical call syntax, different call-graph semantics (known target set vs. open set).
- **Observable content:** The same trait consumed both ways with several implementing types; a heterogeneous `Vec<Box<dyn Trait>>` loop.
- **Variants & failures:** Enum-dispatch as the closed-set alternative (ecosystem pattern, RS-ECO); `impl Trait` arguments as sugar for generics (RS-CLO-04).
- **Constraints:** none.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Book (trait objects) / Reference.

#### RS-DSP-04 · Indirect callables
- **Behavior:** Fn pointers and closures stored in fields, passed as arguments, or returned make the call target a runtime value; `impl Fn(...)`/`Box<dyn Fn(...)>` bounds are the static contract.
- **Observable content:** A registry mapping names to `fn(...)` pointers; a callback field invoked later; a higher-order function taking `impl Fn` and one taking `&dyn Fn`.
- **Variants & failures:** Fn-item unique types vs. coerced pointers (RS-CLO-02); calling a field named like a method requires `(self.f)()` parens.
- **Constraints:** none.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Call expressions).

#### RS-DSP-05 · Deref-carried methods
- **Behavior:** A wrapper implementing `Deref<Target = T>` exposes `T`'s methods through method syntax on the wrapper; smart pointers (`Box`, `Arc`) pass method calls through the same way. A method appears "on" a type that has no such impl.
- **Observable content:** A newtype with `Deref` whose users call target methods directly; a shadowing case where the wrapper defines its own method with the same name (wrapper wins).
- **Variants & failures:** `Box<dyn Trait>` calling trait methods via deref to the dyn value; associated (non-method) functions do *not* pass through Deref.
- **Constraints:** none.
- **Defined by:** language (+ ecosystem convention on Deref use) · **Confidence:** high · **Verify:** The Rust Reference (Method-call expressions); std `Deref` docs.

#### RS-DSP-06 · Version- and edition-sensitive resolution
- **Behavior:** Method resolution outcomes can change with edition and toolchain version: `array.into_iter()` yields references before 2021 (resolving to `(&array).into_iter()`) and values in 2021+; new std trait/inherent methods can capture calls previously resolving to extension traits (accepted as minor-version breakage).
- **Observable content:** The array `into_iter` case compiled under two editions in one workspace; an extension-trait method with a name later added to std (documented as a hazard, exercised via a locally defined lookalike trait).
- **Variants & failures:** `rustc` lints (`array_into_iter`) that patched the 2015/2018 behavior.
- **Constraints:** editions per crate; toolchain version.
- **Defined by:** language + edition mechanism · **Confidence:** high · **Verify:** The Rust Edition Guide (2021 array IntoIterator).

### 2.9 Implicit calls and desugaring (RS-IMP)

#### RS-IMP-01 · `for` loops and iteration
- **Behavior:** `for x in e` desugars to `IntoIterator::into_iter(e)` plus repeated `Iterator::next` — call edges to trait impls with no visible call syntax.
- **Observable content:** `for` over a `Vec`, over `&Vec` (different `IntoIterator` impls: values vs. references), and over a custom iterator type.
- **Variants & failures:** Implementing `IntoIterator` for a custom collection and for its `&`/`&mut` borrows — three distinct impls reachable from near-identical loops.
- **Constraints:** none.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Loop expressions / for).

#### RS-IMP-02 · The `?` operator
- **Behavior:** `expr?` desugars through the `Try` machinery and, for `Result`, calls `From::from` on the error to convert it to the caller's error type — the choice of `From` impl (thus the edge) is inference-driven by the enclosing signature.
- **Observable content:** Error types with `From` impls chained through `?` across functions; `?` on `Option`; `?` in a `main() -> Result<...>`.
- **Variants & failures:** Missing `From` impl error; `?` mixing `Option`/`Result` rejected; `thiserror #[from]` generating the impls (RS-ECO-02).
- **Constraints:** the `Try` trait itself is unstable to *implement*, stable to use via `Result`/`Option`.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (the `?` operator).

#### RS-IMP-03 · `.await` and async calls
- **Behavior:** Calling an `async fn` constructs a future (no body execution); `.await` desugars to `IntoFuture::into_future` + repeated `Future::poll`. The construction edge and the execution edge are distinct.
- **Observable content:** An async call whose result is stored, then awaited elsewhere; a custom `IntoFuture` type awaited directly.
- **Variants & failures:** An unawaited future (`unused_must_use` diagnostic); `.await` outside async context (invalid).
- **Constraints:** edition ≥ 2018.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Await expressions).

#### RS-IMP-04 · Operators and formatting as trait calls
- **Behavior:** `a + b` → `Add::add`; `==`/`<` → `PartialEq`/`PartialOrd`; `a[i]` → `Index::index`; `*p` → `Deref::deref`; `+=` → `AddAssign`; `{}`/`{:?}` in format macros → `Display`/`Debug` via compiler-built `format_args!`.
- **Observable content:** Custom impls exercised only through the sugar forms, never by named call.
- **Variants & failures:** Operator on references (`&a + &b`) hitting reference impls; formatting a type lacking `Display` (E0277 diagnostic through a macro).
- **Constraints:** none.
- **Defined by:** language + std · **Confidence:** high · **Verify:** The Rust Reference (Operator expressions); std `fmt` docs.

#### RS-IMP-05 · Drop scheduling
- **Behavior:** `Drop::drop` runs implicitly at scope exit with defined order: locals in reverse declaration order, fields in declaration order, temporaries at statement end. Destructor edges exist with no call syntax; `let _ = x` vs `let _x = x` differ in drop timing.
- **Observable content:** A `Drop`-implementing guard type demonstrating order via side effects; a `ManuallyDrop` suppression; an explicit `drop(x)` call.
- **Variants & failures:** Temporaries in `match` scrutinee lifetimes (a classic source of deadlock bugs with lock guards); edition-2024 tweaks to temporary lifetimes in tail expressions (medium confidence).
- **Constraints:** none.
- **Defined by:** language · **Confidence:** high (medium on 2024 temporary-scope changes) · **Verify:** The Rust Reference (Destructors).

#### RS-IMP-06 · Closure environment capture
- **Behavior:** Closures capture variables by reference, mutable reference, or move as inferred from use; `move` forces by-value. Since edition 2021 closures capture *disjoint fields* rather than whole structs — changing both borrow relationships and drop timing. Captures are data edges from closure to environment.
- **Observable content:** A closure capturing one field of a struct while another field is mutated (compiles in 2021+, not 2018 — a paired-edition demonstration); a `move` closure sent to a thread.
- **Variants & failures:** Capture-induced `Send` loss; drop-order change caused by disjoint capture (2021 migration lint case).
- **Constraints:** edition.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Edition Guide (disjoint capture); Reference (Closure types).

### 2.10 Macros and code generation (RS-MAC)

#### RS-MAC-01 · `macro_rules!` basics
- **Behavior:** Declarative macros match token fragments (`$e:expr`, `$i:ident`, `$t:ty`, `$p:pat`, ...) and expand to items, expressions, statements, or patterns depending on invocation position. Multiple rules; repetition `$(...)*`.
- **Observable content:** A macro with several rules and repetitions, invoked in item and expression positions.
- **Variants & failures:** Fragment-follow restrictions; a non-matching invocation error.
- **Constraints:** fragment semantics vary by edition (see RS-EDI-03/04).
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Macros by example).

#### RS-MAC-02 · Macro scoping and export
- **Behavior:** `macro_rules!` macros are *textually* scoped — visible only after their definition point, flowing into child modules; `#[macro_use]` on a module extends the scope; `#[macro_export]` hoists the macro to the crate root path (usable as `cratename::the_macro`); 2018+ path-based import `use crate::my_macro;` via re-export. `$crate` makes expansion paths crate-stable across consumers.
- **Observable content:** A macro used before its definition (invalid) and after (valid); a `#[macro_export]` macro invoked from another crate; a `pub use` macro re-export namespaced in a module; `$crate::helper` inside an exported macro.
- **Variants & failures:** Ordering sensitivity is unique among item kinds — reordering source changes validity.
- **Constraints:** edition for path-based imports.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Macros by example / scoping).

#### RS-MAC-03 · Hygiene
- **Behavior:** Locals introduced by a macro expansion don't collide with call-site names (and vice versa) — mixed-site hygiene for `macro_rules!`. Items generated by macros are visible at the call site. Entity identity must account for expansion context, not just spelling.
- **Observable content:** A macro declaring an internal `let tmp` used alongside a caller's `tmp`; a macro taking an `$name:ident` to deliberately create a caller-visible item.
- **Variants & failures:** Passing identifiers in explicitly is the standard escape hatch from hygiene; proc macros can choose spans (call-site vs. mixed) with different visibility results.
- **Constraints:** hygiene model only partially documented.
- **Defined by:** language (partially implementation-defined) · **Confidence:** medium · **Verify:** The Rust Reference (Macros by example / hygiene).

#### RS-MAC-04 · Macros that mint named entities
- **Behavior:** Macros can stamp out families of items — structs, impls, modules, tests — whose names come from macro arguments or concatenation (via the `paste` crate; `concat_idents!` is nightly). Entities exist only after expansion; source spans point into the macro.
- **Observable content:** A `macro_rules!` generating several similar fns/impls from a list; a `paste`-style concatenated name consumed elsewhere by ordinary code.
- **Variants & failures:** Generated items referenced by hand-written code (resolution into expansion output); duplicate generation collisions.
- **Constraints:** identifier concatenation needs a proc-macro helper on stable.
- **Defined by:** language + ecosystem (`paste`) · **Confidence:** high · **Verify:** The Rust Reference; `paste` crate docs.

#### RS-MAC-05 · Built-in macros
- **Behavior:** Compiler-implemented macros with special powers: `include!`/`include_str!`/`include_bytes!` (file inclusion), `env!`/`option_env!` (build-environment values baked into the crate), `concat!`, `stringify!`, `file!`/`line!`/`column!`, `cfg!`, `compile_error!`, `format_args!`, `vec!`, `matches!`, `assert!` family, `todo!`/`unimplemented!`.
- **Observable content:** `env!("CARGO_PKG_VERSION")` in a constant; a `compile_error!` behind a cfg guard (configuration-dependent invalidity); `include_str!` of a sibling file.
- **Variants & failures:** `env!` of an unset variable is a compile error while `option_env!` yields `None` — build environment as semantic input.
- **Constraints:** none.
- **Defined by:** official implementation (std/compiler built-ins) · **Confidence:** high · **Verify:** std macro documentation.

#### RS-MAC-06 · Derive proc macros and helper attributes
- **Behavior:** `#[derive(X)]` invokes a proc macro registered via `#[proc_macro_derive(X, attributes(helper))]`, which *adds* items (impls) without altering the annotated item. Declared helper attributes (`#[serde(...)]`, `#[error(...)]`) are inert to the compiler and meaningful only to the macro.
- **Observable content:** A third-party derive with helper attributes altering the generated impl (rename, skip); the same derive without helpers for contrast.
- **Variants & failures:** Helper-attribute name collisions between derives; unknown helper without the derive is an unresolved-attribute error.
- **Constraints:** proc-macro crate required (RS-MAC-09).
- **Defined by:** language (mechanism) + ecosystem (specific derives) · **Confidence:** high · **Verify:** The Rust Reference (Procedural macros).

#### RS-MAC-07 · Attribute proc macros transform items
- **Behavior:** `#[attr]` proc macros *replace* the annotated item's token stream; the compiled entity can differ arbitrarily from the source (rewritten signature, injected statics/impls, wrapped body). Examples: `#[tokio::main]` (rewrites `async fn main` into sync main + runtime), `#[async_trait]` (rewrites methods to return `Box<Pin<dyn Future>>`), `#[tracing::instrument]` (wraps body).
- **Observable content:** At least one attribute macro from a dependency applied to a fn and one applied to an impl/trait; commentary that the source signature ≠ compiled signature.
- **Variants & failures:** Attribute macros on modules (allowed on inline modules); stacking multiple attribute macros (expansion order outside-in).
- **Constraints:** proc-macro crates.
- **Defined by:** language (mechanism) + ecosystem · **Confidence:** high · **Verify:** The Rust Reference (Procedural macros).

#### RS-MAC-08 · Function-like proc macros
- **Behavior:** `name!(...)` proc macros map arbitrary tokens to arbitrary tokens — embedded DSLs (`html!`, SQL macros) — and may perform I/O during expansion (read schema files, contact a database as `sqlx::query!` does), making compilation environment-sensitive.
- **Observable content:** One function-like proc macro invocation producing items or typed expressions from a non-Rust DSL input.
- **Variants & failures:** Environment-dependent expansion success (offline vs. online modes); expansion-time panics surfacing as compile errors at the call site.
- **Constraints:** proc-macro crates; no sandboxing.
- **Defined by:** language (mechanism) + ecosystem · **Confidence:** high · **Verify:** The Rust Reference (Procedural macros).

#### RS-MAC-09 · The proc-macro crate type
- **Behavior:** Proc macros must live in a dedicated `proc-macro = true` crate, compiled for the *host* and executed inside the compiler. Such crates export only macros; their dependencies (`syn`, `quote`, `proc-macro2`) exist in the build-time universe and never in the runtime dependency graph of consumers.
- **Observable content:** A workspace with a `-macros` sibling crate consumed by the main crate; `syn`/`quote` appearing only in the macro crate's manifest.
- **Variants & failures:** Attempting to export a normal fn from a proc-macro crate; cross-compilation where host ≠ target makes the two universes visibly different.
- **Constraints:** Cargo + rustc.
- **Defined by:** official implementation · **Confidence:** high · **Verify:** The Rust Reference (Procedural macros); The Cargo Book.

#### RS-MAC-10 · Macro namespace and name collisions
- **Behavior:** Macros resolve in their own namespace: `vec!` and `Vec` coexist; a derive macro, an attribute macro, and a type may share a name (common: `serde::Serialize` the derive vs. the trait — same path, different namespaces). Declarative macros can be shadowed by later definitions textually.
- **Observable content:** `use serde::Serialize;` serving both derive position and trait-bound position; a locally defined macro shadowing an imported one.
- **Variants & failures:** Ambiguity between a `macro_rules!` and an imported macro of the same name (textual candidate vs. path candidate).
- **Constraints:** none.
- **Defined by:** language · **Confidence:** high (medium on shadowing corner rules) · **Verify:** The Rust Reference (Names/Namespaces, Macros).

#### RS-MAC-11 · Macro-driven failure modes
- **Behavior:** `compile_error!` produces deliberate diagnostics (commonly guarding invalid feature combinations); expansion errors, recursion-limit exhaustion (`#![recursion_limit]` interplay), and `cfg_attr`-injected derives make macro presence configuration-dependent.
- **Observable content:** A `compile_error!` behind `#[cfg(all(feature = "a", feature = "b"))]`; `#[cfg_attr(feature = "serde", derive(Serialize))]` toggling generated impls per feature.
- **Variants & failures:** A recursive macro requiring a raised recursion limit (validity depends on a crate attribute).
- **Constraints:** none.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference.

#### RS-MAC-12 · Hidden-path expansion plumbing
- **Behavior:** Exported macros conventionally expand to `$crate::__private::...` paths — public but `#[doc(hidden)]` items that exist solely for macro output. Real edges run through deliberately hidden API surface (serde, tokio, thiserror all do this).
- **Observable content:** A local exported macro with a `#[doc(hidden)] pub mod __private` support module, consumed from a second crate.
- **Variants & failures:** Semver hazards of hidden-but-public items (ecosystem debate); hidden items still fully resolvable.
- **Constraints:** none.
- **Defined by:** ecosystem convention · **Confidence:** high · **Verify:** serde source layout (convention; no single document).

### 2.11 Conditional compilation and features (RS-CFG)

#### RS-CFG-01 · cfg predicates
- **Behavior:** `#[cfg(...)]` removes items/statements/fields/variants from compilation entirely. Key predicates: `target_os`, `target_family`/`unix`/`windows`, `target_arch`, `target_pointer_width`, `target_endian`, `test`, `debug_assertions`, `feature = "x"`, combined with `any`/`all`/`not`.
- **Observable content:** cfg on an item, a struct field, an enum variant, a match arm(-containing statement), and an impl block; at least one `all(not(...))` combination.
- **Variants & failures:** cfg on expressions is not allowed (statement/item granularity); a cfg'd-off enum variant changing exhaustiveness per configuration.
- **Constraints:** none.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Conditional compilation).

#### RS-CFG-02 · `cfg_attr`
- **Behavior:** `#[cfg_attr(pred, attr)]` conditionally attaches attributes — conditionally deriving traits, conditionally applying `#[path]`, conditionally enabling `#![no_std]`.
- **Observable content:** `#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]`; `#![cfg_attr(not(feature = "std"), no_std)]`.
- **Variants & failures:** Nested/multiple attributes in one `cfg_attr`; cfg_attr chains.
- **Constraints:** none.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Conditional compilation).

#### RS-CFG-03 · `cfg!` macro vs. cfg attribute
- **Behavior:** `cfg!(...)` evaluates to a compile-time boolean but *both branches remain in the compiled crate* and must type-check — unlike the attribute, which removes content. Distinct source-inclusion semantics for near-identical spellings.
- **Observable content:** An `if cfg!(unix) { ... } else { ... }` whose both arms reference items that must exist everywhere, contrasted with an attribute-cfg'd pair.
- **Variants & failures:** A `cfg!` branch calling a platform-only API fails on the other platform (the classic misuse).
- **Constraints:** none.
- **Defined by:** language · **Confidence:** high · **Verify:** std `cfg!` documentation.

#### RS-CFG-04 · Same-name items under exclusive cfgs
- **Behavior:** Multiple definitions of one name with mutually exclusive cfgs are a single logical API with configuration-selected identity — the central "one graph per configuration" driver.
- **Observable content:** Feature-split same-signature functions; platform-split modules (RS-MOD-06); a cfg'd pair where *neither* is enabled in some configuration (downstream unresolved-name failure).
- **Variants & failures:** Overlapping (non-exclusive) cfgs causing duplicate-definition errors only in specific configurations.
- **Constraints:** none.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Conditional compilation).

#### RS-CFG-05 · cfg'd-out code: parsed, not resolved
- **Behavior:** Disabled items must lex and parse but are not name-resolved or type-checked: they may reference nonexistent crates, items, or types. Invalid-if-enabled code legitimately lives in a valid project.
- **Observable content:** A `#[cfg(feature = "never")]` module referencing an undeclared dependency, building cleanly by default and failing when the feature is enabled.
- **Variants & failures:** Syntax errors inside cfg'd-out code still fail the build — the parse/resolve boundary made visible.
- **Constraints:** none.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Conditional compilation).

#### RS-CFG-06 · Cargo features
- **Behavior:** `[features]` defines named flags mapped to `cfg(feature = "...")`; `default` features; optional dependencies (each creates an implicit same-named feature unless referenced via `dep:`); `dep:foo` (dependency-only activation), `foo/bar` (enable dep + its feature), weak `foo?/bar` (feature of dep only if dep already enabled).
- **Observable content:** A manifest exercising defaults, an optional dep with `dep:`, a transitive `foo/bar`, and a weak `foo?/bar`; feature-gated public items and trait impls (per-config coherence surface).
- **Variants & failures:** Feature-gated impls meaning `T: Trait` holds only under a feature; non-additive feature design as the documented anti-pattern.
- **Constraints:** Cargo.
- **Defined by:** official implementation (Cargo) · **Confidence:** high · **Verify:** The Cargo Book (Features).

#### RS-CFG-07 · Feature unification
- **Behavior:** Within one build graph, a crate version's features are the union of everything any participant requested — the code compiled for a dependency depends on *which packages are built together*. Resolver v2 stops unifying across normal/build/dev and target-specific boundaries; `default-features = false` can be silently re-defaulted by another dependent.
- **Observable content:** A workspace where building member A alone vs. the whole workspace changes a shared dependency's enabled features (observable via that dep's cfg-gated API); a `default-features = false` request overridden by a sibling.
- **Variants & failures:** Resolver v1 vs v2 differences; `cargo build -p a` vs `-p a -p b` producing different feature sets.
- **Constraints:** resolver version (manifest `resolver` key).
- **Defined by:** official implementation (Cargo) · **Confidence:** high · **Verify:** The Cargo Book (Feature unification / Resolver).

#### RS-CFG-08 · Checked cfgs
- **Behavior:** Since 1.80, rustc/cargo validate cfg names/values (`unexpected_cfgs` lint). Custom cfgs must be declared — via build-script `cargo::rustc-check-cfg=cfg(...)` or `[lints.rust.unexpected_cfgs]` configuration — or their use warns.
- **Observable content:** A custom cfg set by a build script with its check-cfg declaration; an undeclared cfg triggering the lint.
- **Variants & failures:** `cfg(true)`/`cfg(false)` literal forms (recent stabilization — verify exact version).
- **Constraints:** rustc ≥ 1.80.
- **Defined by:** official implementation · **Confidence:** high (medium on cfg(true/false) timing) · **Verify:** The Cargo Book (build script check-cfg); rustc blog/release notes.

#### RS-CFG-09 · Documentation-facing cfg conventions
- **Behavior:** `#[cfg(doctest)]` gates items compiled only while collecting doctests; the docs.rs convention `#[cfg_attr(docsrs, doc(cfg(feature = "x")))]` (nightly `doc_cfg`) annotates feature-gated APIs in docs; docs.rs builds commonly use `--all-features` and a `docsrs` cfg.
- **Observable content:** A `cfg(doctest)`-gated doctest holder; the docsrs cfg_attr pattern on a feature-gated module.
- **Variants & failures:** Nightly-only `doc_cfg` making docs builds channel-sensitive.
- **Constraints:** nightly for `doc_cfg`; docs.rs metadata table in Cargo.toml (`[package.metadata.docs.rs]`).
- **Defined by:** ecosystem convention (+ rustdoc nightly features) · **Confidence:** medium · **Verify:** The rustdoc Book; docs.rs documentation.

### 2.12 Cargo dependencies and workspaces (RS-DEP)

#### RS-DEP-01 · Dependency kinds
- **Behavior:** `[dependencies]` (linked into lib/bins), `[dev-dependencies]` (tests/examples/benches only), `[build-dependencies]` (build script only), each optionally target-specific via `[target.'cfg(...)'.dependencies]`. Kind determines which crates each target can resolve.
- **Observable content:** One dependency of each kind, with a deliberate failure: lib code referencing a dev-dependency.
- **Variants & failures:** Target-specific dev-deps; the same package appearing in two kinds (RS-DEP-11).
- **Constraints:** Cargo.
- **Defined by:** official implementation · **Confidence:** high · **Verify:** The Cargo Book (Specifying dependencies).

#### RS-DEP-02 · Dependency sources and overrides
- **Behavior:** Registry (crates.io or alternative registries), `git` (with `branch`/`tag`/`rev`), and `path` sources; `[patch.crates-io]` (and per-registry patch tables) redirect a name graph-wide; vendoring via source replacement in `.cargo/config.toml`.
- **Observable content:** Path dependencies between workspace members; a git dependency pinned to a rev; a `[patch]` redirecting a registry dep to a local path.
- **Variants & failures:** Patch changing resolution for transitive users too; a path dep that isn't a workspace member.
- **Constraints:** Cargo; network vs. offline modes.
- **Defined by:** official implementation · **Confidence:** high · **Verify:** The Cargo Book (Overriding dependencies).

#### RS-DEP-03 · Version resolution and coexisting majors
- **Behavior:** Semver-caret requirements resolved against the registry, pinned by `Cargo.lock`. Semver-incompatible versions of one package coexist in a graph as *distinct crates*: same name, two identities; their same-named types do not unify (the classic "two different versions of crate X" trap).
- **Observable content:** A dependency tree containing two majors of one crate (directly and transitively), plus a deliberate type-mismatch showing non-unification.
- **Variants & failures:** Lockfile presence vs. absence changing resolved versions; `cargo update` semantics.
- **Constraints:** Cargo resolver.
- **Defined by:** official implementation · **Confidence:** high · **Verify:** The Cargo Book (Dependency resolution).

#### RS-DEP-04 · Renamed dependencies
- **Behavior:** `alias = { package = "real-name", ... }` imports a package under a different crate name — the in-source name diverges from the registry identity; also the mechanism for depending on two versions of one package simultaneously.
- **Observable content:** One dependency renamed; two versions of one package imported as `foo_v1`/`foo_v2` and both used.
- **Variants & failures:** Two distinct packages that would otherwise collide on lib name, disambiguated by rename.
- **Constraints:** Cargo.
- **Defined by:** official implementation · **Confidence:** high · **Verify:** The Cargo Book (Renaming dependencies).

#### RS-DEP-05 · Direct vs. transitive nameability
- **Behavior:** Only declared dependencies enter the extern prelude (Cargo passes `--extern` per direct dep); transitive crates' types appear in signatures but cannot be named without a re-export. Libraries therefore conventionally re-export key deps (`pub use`).
- **Observable content:** A dependency whose API returns a transitive crate's type; consumer code failing to name that type directly, then succeeding via the dependency's re-export.
- **Variants & failures:** Version-skew traps when the consumer *also* declares the transitive dep at a different version (two identities again).
- **Constraints:** Cargo/rustc invocation model.
- **Defined by:** official implementation · **Confidence:** high · **Verify:** The Cargo Book; rustc `--extern` documentation.

#### RS-DEP-06 · Workspaces
- **Behavior:** A workspace shares one `Cargo.lock` and target directory across members; the root manifest is either a *virtual* manifest (`[workspace]` only) or also a package. `members`/`exclude` globs, `default-members`; nested workspaces are not allowed (must be excluded).
- **Observable content:** A multi-member workspace with a virtual root, inter-member path deps, and one excluded subdirectory package.
- **Variants & failures:** Virtual workspaces default to resolver v1 unless `resolver` is set explicitly (a documented gotcha, ties to RS-DEP-09).
- **Constraints:** Cargo.
- **Defined by:** official implementation · **Confidence:** high · **Verify:** The Cargo Book (Workspaces).

#### RS-DEP-07 · Workspace inheritance
- **Behavior:** `[workspace.dependencies]` and `[workspace.package]` centralize versions/metadata; members opt in with `dependency.workspace = true` / `field.workspace = true` (Cargo ≥ 1.64). `[workspace.lints]` similarly centralizes lint config (≥ 1.74 with `[lints] workspace = true`).
- **Observable content:** Members inheriting a shared dependency spec, edition, and lint table; one member overriding features on an inherited dep.
- **Variants & failures:** Member adding features to an inherited dependency (allowed) vs. changing its version (not).
- **Constraints:** Cargo ≥ 1.64 / 1.74.
- **Defined by:** official implementation · **Confidence:** high · **Verify:** The Cargo Book (Workspace inheritance).

#### RS-DEP-08 · Profiles and panic strategy
- **Behavior:** `dev`/`release`/`test`/`bench` (+ custom) profiles set `opt-level`, `debug-assertions` (drives `cfg(debug_assertions)` and overflow checks), and `panic = "unwind" | "abort"`. Panic strategy is semantic: `catch_unwind` cannot catch under abort; overflow behavior flips between panic and wrap (RS section 4).
- **Observable content:** Profile tables including a `panic = "abort"` release, a `debug_assertions`-gated code path, and per-dependency profile overrides (`[profile.dev.package."*"]`).
- **Variants & failures:** `panic = "abort"` incompatible with `cargo test` default harness (tests force unwind); profile inheritance for custom profiles.
- **Constraints:** Cargo.
- **Defined by:** official implementation · **Confidence:** high · **Verify:** The Cargo Book (Profiles).

#### RS-DEP-09 · Resolver versions
- **Behavior:** `resolver = "1"` (legacy unification), `"2"` (edition-2021 default; splits feature unification across dep kinds/targets), `"3"` (MSRV-aware version selection, Cargo ≥ 1.84; edition-2024 default). Resolver choice changes both selected versions and feature sets — i.e., the dependency graph itself.
- **Observable content:** Explicit `resolver` in the workspace root; a case where v1 vs v2 changes a build-dependency's features.
- **Variants & failures:** Virtual-workspace default gotcha (RS-DEP-06); MSRV-aware resolution picking an older dep version under v3.
- **Constraints:** Cargo version; manifest.
- **Defined by:** official implementation · **Confidence:** high (medium on v3 fine print) · **Verify:** The Cargo Book (Resolver).

#### RS-DEP-10 · MSRV and toolchain pinning
- **Behavior:** `package.rust-version` declares a minimum supported Rust version; Cargo refuses to build with an older toolchain (bypass: `--ignore-rust-version`). `rust-toolchain.toml` pins the toolchain channel/version per directory tree. Program validity is a function of toolchain version.
- **Observable content:** A member with `rust-version` above some historical rustc; a `rust-toolchain.toml` pinning a specific stable.
- **Variants & failures:** A crate using a recently stabilized feature with an honest `rust-version`; resolver v3 consulting `rust-version` of dependencies.
- **Constraints:** Cargo + rustup.
- **Defined by:** official implementation · **Confidence:** high · **Verify:** The Cargo Book (rust-version); rustup documentation.

#### RS-DEP-11 · One package, two build universes
- **Behavior:** A package used as both a build-dependency and a normal dependency is compiled twice — for host and target — potentially with different features under resolver v2. Same source, two crate instances in one build.
- **Observable content:** A utility crate consumed by both `build.rs` and the lib, with a feature enabled only on the build side.
- **Variants & failures:** Proc-macro deps count as host-side (RS-MAC-09); cross-compilation makes the split visible even for identical features.
- **Constraints:** Cargo.
- **Defined by:** official implementation · **Confidence:** high · **Verify:** The Cargo Book (Features / resolver v2).

#### RS-DEP-12 · Dev-dependency cycles
- **Behavior:** Dev-dependencies may (via path) depend on the package under test itself — a permitted cycle unique to the dev graph (common with `-test-util` companion crates or benchmarking helpers).
- **Observable content:** Package A whose dev-dependency B path-depends on A; tests in A using B.
- **Variants & failures:** The cycle is invalid if promoted to a normal dependency.
- **Constraints:** Cargo.
- **Defined by:** official implementation · **Confidence:** medium · **Verify:** The Cargo Book (dev-dependencies).

### 2.13 Build scripts and generated code (RS-GEN)

#### RS-GEN-01 · build.rs mechanics
- **Behavior:** A `build.rs` is a separate crate compiled for the host with its own `[build-dependencies]`, run before compilation. It communicates via stdout directives — modern `cargo::` prefix (≥ 1.77) or legacy `cargo:` — including `rustc-cfg`, `rustc-env`, `rustc-link-lib`, `rustc-link-search`, `rerun-if-changed`, `rerun-if-env-changed`, `warning`.
- **Observable content:** A build script emitting at least a cfg, an env, and a rerun-if-changed directive; its own dependency used nowhere else.
- **Variants & failures:** Build-script panic fails the build with script output attached; directive typos silently ignored (legacy prefix) vs. checked (`cargo::`, medium confidence on checking).
- **Constraints:** Cargo.
- **Defined by:** official implementation · **Confidence:** high · **Verify:** The Cargo Book (Build scripts).

#### RS-GEN-02 · OUT_DIR code generation
- **Behavior:** The build script writes `.rs` files into `$OUT_DIR`; the crate includes them via `include!(concat!(env!("OUT_DIR"), "/gen.rs"))`. Compiled entities whose source exists only after the build starts and lives outside the source tree, at a per-build path.
- **Observable content:** A generated module with items referenced by handwritten code; generation input tracked with `rerun-if-changed`.
- **Variants & failures:** Generated code failing to compile (diagnostics point into OUT_DIR); generating different content per feature/platform.
- **Constraints:** Cargo.
- **Defined by:** official implementation · **Confidence:** high · **Verify:** The Cargo Book (Build scripts / code generation).

#### RS-GEN-03 · Build-script-injected cfgs and envs
- **Behavior:** `cargo::rustc-cfg=has_foo` enables `#[cfg(has_foo)]` code; `cargo::rustc-env=KEY=value` feeds `env!("KEY")`. Feature detection (probing compiler version or system libraries) makes the compiled item set an artifact of the build environment.
- **Observable content:** A version- or platform-probe in build.rs gating a code path; the paired `rustc-check-cfg` declaration (RS-CFG-08).
- **Variants & failures:** The same source producing different graphs on different machines — the canonical environment-sensitivity case.
- **Constraints:** Cargo.
- **Defined by:** official implementation · **Confidence:** high · **Verify:** The Cargo Book (Build scripts).

#### RS-GEN-04 · `links` and inter-build-script metadata
- **Behavior:** `package.links = "foo"` asserts ownership of native library `foo` (one package per `links` name graph-wide — a uniqueness constraint that can make otherwise-valid graphs unbuildable) and exports `cargo::metadata` values to dependents' build scripts as `DEP_FOO_*` env vars.
- **Observable content:** A `-sys`-style package with `links` and metadata; a dependent build script consuming `DEP_*`.
- **Variants & failures:** Two packages with the same `links` value: resolution failure.
- **Constraints:** Cargo.
- **Defined by:** official implementation · **Confidence:** high · **Verify:** The Cargo Book (links).

#### RS-GEN-05 · Ecosystem code generators
- **Behavior:** Common generators produce large module trees consumed as ordinary Rust: `bindgen` (C headers → items mirroring foreign decls), `prost`/`tonic` (protobuf → nested modules + derives), `cxx` (macro + codegen pair spanning two languages). Generated entities dominate some real crates' graphs.
- **Observable content:** At least one generator wired through build.rs + OUT_DIR + include!, with handwritten code calling generated items.
- **Variants & failures:** Checked-in generated code vs. build-time generation (two source-inclusion policies); generator version drift changing entity sets.
- **Constraints:** external tools/system deps (libclang for bindgen, protoc for prost by default).
- **Defined by:** ecosystem convention · **Confidence:** high · **Verify:** the respective crates' documentation.

### 2.14 Editions and language versioning (RS-EDI)

#### RS-EDI-01 · Editions are per-crate; mixed graphs are normal
- **Behavior:** `edition` is declared per package; a build graph freely mixes editions. Macros carry edition hygiene: tokens behave per the edition of the crate that *wrote* them, so a 2015-edition macro keeps working when expanded inside a 2024 crate.
- **Observable content:** A workspace with members on at least two editions, cross-consuming items and macros.
- **Variants & failures:** A keyword valid as an identifier in the old edition crossing into a newer consumer (raw-identifier interplay, RS-RES-07).
- **Constraints:** rustc supports all editions simultaneously.
- **Defined by:** language + official implementation · **Confidence:** high (medium on macro edition-hygiene corner cases) · **Verify:** The Rust Edition Guide.

#### RS-EDI-02 · Edition 2015 → 2018 deltas
- **Behavior:** Uniform paths and `crate::` prefixes; `extern crate` optional; `dyn Trait` introduced (bare trait objects deprecated); `async`/`await`/`try` become keywords; macros importable by path.
- **Observable content:** One legacy 2015 member using `extern crate`, `#[macro_use]`, and 2015 path style, consumed by newer members.
- **Variants & failures:** Leading-`::` meaning differs (crate root in 2015, extern prelude in 2018+).
- **Constraints:** edition.
- **Defined by:** language (edition mechanism) · **Confidence:** high · **Verify:** The Rust Edition Guide (2018).

#### RS-EDI-03 · Edition 2021 deltas
- **Behavior:** Disjoint closure captures (RS-IMP-06); arrays implement `IntoIterator` by value with method-resolution compatibility shims for older editions (RS-DSP-06); prelude adds `TryFrom`/`TryInto`/`FromIterator`; `panic!` message consistency; `$pat` in `macro_rules!` matches top-level or-patterns; reserved prefixed-identifier syntax.
- **Observable content:** Paired same-source demonstrations under 2018 vs. 2021 members where behavior diverges (capture granularity; array iteration; a `TryInto` call needing no import).
- **Variants & failures:** The `pat_param` fragment specifier as the old-behavior opt-back.
- **Constraints:** edition; rustc ≥ 1.56.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Edition Guide (2021).

#### RS-EDI-04 · Edition 2024 deltas
- **Behavior:** Return-position `impl Trait` captures all in-scope lifetimes by default, with `use<...>` precise capturing as the opt-out; `unsafe extern` blocks required; `no_mangle`/`export_name`/`link_section` must be written `#[unsafe(...)]`; references to `static mut` denied; `unsafe_op_in_unsafe_fn` warns by default; `gen` reserved; the `expr` macro fragment matches new expression forms (`expr_2021` preserves old matching); prelude adds `Future`/`IntoFuture`; let-chains in `if`/`while` (stabilized in 1.88) require edition 2024; default resolver becomes v3.
- **Observable content:** A 2024 member exercising `use<>` capture, `unsafe extern`, `#[unsafe(no_mangle)]`, and a let-chain; a 2021 twin showing the same code rejected or differently interpreted.
- **Variants & failures:** RPIT capture change altering borrow-check acceptance between editions — same source, different validity.
- **Constraints:** rustc ≥ 1.85 (≥ 1.88 for let-chains).
- **Defined by:** language · **Confidence:** medium-high (fine print worth verification) · **Verify:** The Rust Edition Guide (2024).

#### RS-EDI-05 · Stability channels as validity boundaries
- **Behavior:** `#![feature(...)]` gates compile only on nightly; the same source is invalid on stable/beta (E0554-class). Channel is thus an input to program validity, orthogonal to edition.
- **Observable content:** A quarantined nightly-only sample (or an intentionally excluded one with a note); a stable-build failure demonstration.
- **Variants & failures:** `RUSTC_BOOTSTRAP=1` bypass (implementation detail; flag but do not rely on).
- **Constraints:** rustup channels.
- **Defined by:** official implementation · **Confidence:** high · **Verify:** The Unstable Book; rustc dev documentation.

#### RS-EDI-06 · Edition migration tooling
- **Behavior:** `cargo fix --edition` mechanically rewrites source for the next edition guided by migration lints — a sanctioned source transformation whose before/after pairs pinpoint exactly the constructs whose meaning changes.
- **Observable content:** A member kept one edition behind with clean migration-lint output, documenting which constructs would be rewritten.
- **Variants & failures:** Idiom lints (`rust-2018-idioms` group) vs. hard migration lints.
- **Constraints:** Cargo/rustc.
- **Defined by:** official implementation · **Confidence:** high · **Verify:** The Rust Edition Guide (transitioning).

### 2.15 Tests, benches, examples, and docs (RS-TST)

#### RS-TST-01 · Unit tests and `cfg(test)`
- **Behavior:** `#[cfg(test)] mod tests` exists only in the unit-test build of the lib; `#[test]` fns are collected by a libtest-generated `main`. The lib crate has two variants (with/without test cfg), and the test entry point is synthetic.
- **Observable content:** In-module tests accessing private items; a `cfg(test)`-only helper; the same lib building without any of it under `cargo build`.
- **Variants & failures:** `#[cfg(test)]` items referenced from non-test code (fails outside test builds).
- **Constraints:** libtest harness default.
- **Defined by:** official implementation (Cargo/libtest) + language (`cfg`) · **Confidence:** high · **Verify:** The Rust Book (testing); The Cargo Book.

#### RS-TST-02 · Integration tests are external crates
- **Behavior:** Each `tests/*.rs` (and `tests/<dir>/main.rs`) compiles as an independent crate linking the lib as an external dependency: private items are inaccessible and the lib is built *without* `cfg(test)`. Shared helpers use the `tests/common/mod.rs` idiom (directories without `main.rs` are not targets).
- **Observable content:** Two integration-test files, one shared `common` module, one failing probe of a private lib item.
- **Variants & failures:** Each test file's separate compilation (same helper compiled per test crate); `#[cfg(test)]` inside integration test files is redundant-but-legal.
- **Constraints:** Cargo target discovery.
- **Defined by:** official implementation · **Confidence:** high · **Verify:** The Cargo Book; The Rust Book (test organization).

#### RS-TST-03 · Doc tests
- **Behavior:** rustdoc extracts fenced code blocks from doc comments and compiles each as a separate mini-crate (implicit `fn main` wrapping, implicit `extern crate`), run by `cargo test` for library targets. Hidden lines (leading `# `) compile but don't render; attributes `ignore`, `no_run`, `should_panic`, `compile_fail`, and edition overrides change compile/run expectations — including *deliberately invalid* code as first-class content.
- **Observable content:** Doctests with hidden setup lines, a `compile_fail` block, a `no_run` block, and one plain runnable example.
- **Variants & failures:** Doctests are not collected from binary targets by default (medium confidence); `?` in doctests needing a hidden `Ok::<(), E>(())` tail.
- **Constraints:** rustdoc.
- **Defined by:** official implementation (rustdoc) · **Confidence:** high · **Verify:** The rustdoc Book (documentation tests).

#### RS-TST-04 · Examples
- **Behavior:** `examples/*.rs` (or `examples/<name>/main.rs`) are runnable crates with dev-dependency access, built by `cargo build --examples`, runnable via `--example`; `required-features` gates them on feature sets.
- **Observable content:** Two examples, one with `required-features`, one library-style example (`crate-type` override) if desired.
- **Variants & failures:** Example failing when its required feature is off (target skipped vs. error — skipped for auto-discovered, medium confidence on exact behavior).
- **Constraints:** Cargo.
- **Defined by:** official implementation · **Confidence:** high · **Verify:** The Cargo Book (Cargo targets).

#### RS-TST-05 · Benches and `harness = false`
- **Behavior:** `#[bench]` requires nightly (`test` crate); the stable convention is a `[[bench]]` target with `harness = false` and a framework like criterion providing its own `main`. `harness = false` also applies to test targets — replacing `#[test]` collection with a user `main` (different entity-discovery model).
- **Observable content:** A criterion-style bench target with `harness = false`; a harness-less integration test with a hand-written `main`.
- **Variants & failures:** `#[bench]` on stable as an invalid-program case.
- **Constraints:** nightly for `#[bench]`; Cargo for harness.
- **Defined by:** official implementation + ecosystem (criterion) · **Confidence:** high · **Verify:** The Cargo Book; the `test` crate docs (unstable).

#### RS-TST-06 · Test attributes and result-returning tests
- **Behavior:** `#[should_panic(expected = "...")]`, `#[ignore]`, and `#[test] fn t() -> Result<(), E>` (Termination-based) modify collection and pass/fail semantics.
- **Observable content:** One of each, including an ignored test still compiled.
- **Variants & failures:** `should_panic` with `panic = "abort"` profile conflicts.
- **Constraints:** libtest.
- **Defined by:** official implementation · **Confidence:** high · **Verify:** The Rust Book (testing); The Rust Reference (test attributes).

#### RS-TST-07 · README and crate-level docs as compiled surface
- **Behavior:** `#![doc = include_str!("../README.md")]` makes README code blocks into doctests — a Markdown file outside `src/` becomes compiled test content; `#[doc(hidden)]` and `#[doc(inline)]`/`#[doc(no_inline)]` shape the documented (and doctest-bearing) surface.
- **Observable content:** A crate using the README-inclusion idiom with at least one fenced Rust block; a hidden pub module.
- **Variants & failures:** README examples breaking the build when the API drifts — docs as a source of build failures.
- **Constraints:** rustdoc.
- **Defined by:** ecosystem convention on rustdoc features · **Confidence:** high · **Verify:** The rustdoc Book.

#### RS-TST-08 · Command-dependent build sets
- **Behavior:** `cargo build`, `cargo test`, and `cargo doc` compile different target sets under different cfgs: unit-test lib (+`cfg(test)`), integration/example/bench crates, doctest mini-crates, docs with `cfg(doc)`. The project's compiled entity set is a function of the command.
- **Observable content:** cfg(test)-, cfg(doc)-, and cfg(doctest)-gated items whose presence varies by command; a note in project docs mapping command → compiled set.
- **Variants & failures:** Code that only fails under `cargo test` (or only under `cargo doc`).
- **Constraints:** Cargo.
- **Defined by:** official implementation · **Confidence:** high · **Verify:** The Cargo Book; The rustdoc Book.

### 2.16 Attributes, lints, diagnostics, invalid programs (RS-ATT)

#### RS-ATT-01 · Lint levels and configuration surfaces
- **Behavior:** `allow`/`warn`/`deny`/`forbid` (+ `expect`, 1.81) apply at crate, module, or item scope; the `[lints]` manifest table (1.74) and `RUSTFLAGS="-D warnings"` configure globally. Whether a program *builds* can depend on lint configuration, not just source.
- **Observable content:** Scoped lint attributes including a `forbid` that a nested `allow` cannot override (error case); a `[lints]` table; an `#[expect]` that is fulfilled and one that is unfulfilled (itself a diagnostic).
- **Variants & failures:** CI-style deny-warnings turning benign code into a failing build — configuration-dependent validity.
- **Constraints:** rustc ≥ 1.74/1.81 for the newer surfaces.
- **Defined by:** official implementation · **Confidence:** high · **Verify:** The rustc Book (lints); The Cargo Book ([lints]).

#### RS-ATT-02 · Reachability lints
- **Behavior:** `dead_code`, `unused_imports`, `unused_variables`, `unreachable_code`, `unreachable_pub` are the compiler's own graph analyses surfaced as diagnostics; their behavior differs between libs and bins (`pub` doesn't escape a binary).
- **Observable content:** An unused private fn (dead_code), an unused import, a `pub` item in a bin flagged dead, `_`-prefix suppression.
- **Variants & failures:** Items kept alive only via `#[allow(dead_code)]`, tests, or macro expansion — reachability nuances.
- **Constraints:** none.
- **Defined by:** official implementation · **Confidence:** high · **Verify:** The rustc Book (lints).

#### RS-ATT-03 · `#[deprecated]`
- **Behavior:** Use-site warnings crossing crate boundaries, with `since`/`note` metadata; deprecation of items, methods, and enum variants.
- **Observable content:** A deprecated item still used somewhere (warning case) plus its replacement.
- **Variants & failures:** Deprecating a trait impl is not supported (attribute placement limits).
- **Constraints:** none.
- **Defined by:** language (attribute) + implementation (diagnostics) · **Confidence:** high · **Verify:** The Rust Reference (diagnostics attributes).

#### RS-ATT-04 · `#[must_use]`
- **Behavior:** On functions or types, makes ignoring a value a warning at *call* sites — callee metadata driving caller diagnostics (`Result` is the canonical carrier).
- **Observable content:** A must_use type and a must_use fn each with an ignoring call site.
- **Variants & failures:** `let _ = ...` suppression; must_use on traits (applies to impl Trait returns).
- **Constraints:** none.
- **Defined by:** language (attribute) · **Confidence:** high · **Verify:** The Rust Reference (diagnostics attributes).

#### RS-ATT-05 · `#[non_exhaustive]`
- **Behavior:** On structs, enums, and variants: downstream crates cannot exhaustively match or literally construct — the *same* code is valid in the defining crate and invalid outside it. Cross-crate semantics differ from same-crate semantics for one entity.
- **Observable content:** A non_exhaustive enum matched with a forced wildcard arm in a consumer crate, and matched exhaustively inside its own crate.
- **Variants & failures:** non_exhaustive struct blocking functional-update construction downstream.
- **Constraints:** effect only across crate boundaries.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (type system attributes).

#### RS-ATT-06 · Representative invalid programs
- **Behavior:** A correct analysis must handle diagnosable-invalid code: unresolved imports (E0432-class), privacy violations (E0603), duplicate definitions (E0428), impl overlap (E0119), orphan impls (E0117), ambiguous methods (E0034), unstable features on stable (E0658), missing trait bound (E0277). Error codes are stable-ish identifiers but should be verified rather than trusted from memory.
- **Observable content:** A quarantined set of minimal not-compiled samples, one per family (kept out of the build via RS-MOD-05 or a dedicated non-member package).
- **Variants & failures:** Errors that appear only in some cfg/feature configurations (RS-CFG-05).
- **Constraints:** exact codes/messages are implementation-defined and version-drifting.
- **Defined by:** official implementation (diagnostics) over language rules · **Confidence:** high (medium on specific codes) · **Verify:** the rustc error-code index.

#### RS-ATT-07 · Compilation limits
- **Behavior:** `#![recursion_limit]` and `#![type_length_limit]` bound macro/trait/type recursion; some programs are valid only with raised limits — validity depending on a crate attribute.
- **Observable content:** A deep recursive macro or type-level construction failing at default limits and passing with a raised limit.
- **Variants & failures:** Limit errors reference the attribute in diagnostics.
- **Constraints:** rustc.
- **Defined by:** official implementation · **Confidence:** high · **Verify:** The Rust Reference (limits attributes).

#### RS-ATT-08 · Semantic vs. hint attributes
- **Behavior:** A spectrum: `#[track_caller]` changes observable behavior (`Location::caller`); `#[no_mangle]`/`#[export_name]` change linkage identity; `#[inline]`/`#[cold]` are optimizer hints with no semantic effect — a graph should distinguish semantic attributes from hints.
- **Observable content:** A `#[track_caller]` assertion helper showing caller locations; hint attributes present but documented as non-semantic.
- **Variants & failures:** `#[inline]` required for cross-crate inlining absent LTO (performance-only, not semantic).
- **Constraints:** edition 2024 wraps linkage attributes in `unsafe(...)`.
- **Defined by:** language + implementation · **Confidence:** high · **Verify:** The Rust Reference (code generation attributes).

### 2.17 Unsafe, FFI, and linkage (RS-UNS)

#### RS-UNS-01 · Unsafe boundaries
- **Behavior:** `unsafe fn` (contract on callers), `unsafe {}` blocks (discharge sites), `unsafe trait` + `unsafe impl` (contract on implementors — `Send`/`Sync` overrides being the common case). Safety obligations are semantic markers a graph can carry.
- **Observable content:** A safe wrapper over an unsafe fn; an unsafe trait with an unsafe impl; `unsafe_op_in_unsafe_fn` behavior difference by edition.
- **Variants & failures:** Unnecessary-unsafe lint; safe fn containing unsafe block vs. unsafe fn.
- **Constraints:** edition 2024 for the in-unsafe-fn lint default.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Unsafety); The Rustonomicon.

#### RS-UNS-02 · Extern functions and ABIs
- **Behavior:** `extern "C"` (and `"system"`, `"C-unwind"`, etc.) on definitions and in extern blocks; variadic parameters in foreign declarations; edition 2024 requires `unsafe extern` with per-item `safe`/`unsafe` qualifiers.
- **Observable content:** A defined `pub extern "C" fn`, an extern block declaring a libc-style function and static, a variadic declaration; the 2024 `unsafe extern` form in the 2024 member.
- **Variants & failures:** Declared-vs-actual signature mismatch is undiagnosed by rustc (UB at link/run time) — a truth-outside-the-graph case.
- **Constraints:** edition for `unsafe extern`.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (External blocks; ABI).

#### RS-UNS-03 · Symbol-level identity
- **Behavior:** `#[no_mangle]`, `#[export_name]`, `#[link_name]`, `#[link(name, kind)]`, `#[link_section]`, `#[used]` attach linker-level names/locations decoupled from Rust paths. Two items can collide at link time while distinct in the source graph (duplicate-symbol errors — invalid at a stage after type checking).
- **Observable content:** An exported no_mangle fn, an imported link_name-renamed foreign fn, and a deliberate duplicate-symbol pair (quarantined failure case).
- **Variants & failures:** Mangled-by-default symbols are implementation-defined (RS section 4).
- **Constraints:** edition 2024 `unsafe(...)` wrappers.
- **Defined by:** language attributes + linker behavior · **Confidence:** high · **Verify:** The Rust Reference (linkage attributes).

#### RS-UNS-04 · repr attributes
- **Behavior:** `repr(C)`, `repr(transparent)`, `repr(u8)`/int reprs on enums, `repr(packed)`, `repr(align(N))` fix layout contracts; without them layout is unspecified. `repr(transparent)` underwrites newtype FFI soundness.
- **Observable content:** A repr(C) struct mirrored in an extern signature; a repr(transparent) newtype passed across FFI; a repr(u8) enum.
- **Variants & failures:** References into `packed` fields (unsafe/denied patterns); enum with fields + explicit repr (allowed forms).
- **Constraints:** none.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Type layout); The Rustonomicon.

#### RS-UNS-05 · cdylib/staticlib export surfaces
- **Behavior:** For `cdylib`/`staticlib` outputs, the meaningful public surface is the exported-symbol set (`#[no_mangle] pub extern "C"`), not the Rust `pub` API — two different notions of "public" for one crate.
- **Observable content:** The RS-PKG-04 cdylib with a documented mapping of Rust items to exported symbols; internal `pub` items that are *not* exported.
- **Variants & failures:** `cbindgen`-style header generation as the ecosystem bridge (RS-ECO-10).
- **Constraints:** target/linker.
- **Defined by:** official implementation · **Confidence:** high · **Verify:** The Cargo Book / Reference (Linkage).

#### RS-UNS-06 · Inline assembly references
- **Behavior:** `asm!`/`global_asm!` with `sym` operands reference Rust items (functions/statics) from assembly text — an edge originating in a non-Rust surface.
- **Observable content:** One minimal `asm!` with a `sym` operand on a supported arch, cfg-gated by `target_arch`.
- **Variants & failures:** Arch-specific validity; `naked` functions (stabilized recently — verify version).
- **Constraints:** per-arch support; unsafe.
- **Defined by:** language (asm! is stabilized per-arch) · **Confidence:** medium · **Verify:** The Rust Reference (Inline assembly).

#### RS-UNS-07 · Unwinding across boundaries
- **Behavior:** `extern "C"` vs `extern "C-unwind"` determines whether panics may cross FFI; `panic = "abort"` profiles interact (unwind through a "C" boundary is UB/abort). Semantics depend on ABI string + profile jointly.
- **Observable content:** Both ABI spellings on declarations, with commentary; profile pairing from RS-DEP-08.
- **Variants & failures:** `catch_unwind` as the Rust-side boundary tool.
- **Constraints:** rustc ≥ 1.71 for C-unwind (medium confidence on version).
- **Defined by:** language · **Confidence:** medium-high · **Verify:** The Rust Reference (ABI / unwinding).

### 2.18 Closures, async, and opaque types (RS-CLO)

#### RS-CLO-01 · Closure types and Fn-trait classification
- **Behavior:** Each closure has a unique unnameable type implementing `FnOnce`, and possibly `FnMut`/`Fn`, as inferred from how it uses captures; `move` changes capture mode, not trait. Anonymous entities with inferred trait memberships.
- **Observable content:** Three closures landing in the three classifications, stored/passed under matching bounds; a failing case passing an `FnOnce` where `Fn` is required.
- **Variants & failures:** Non-capturing closure coercion to `fn` pointer; closures implementing `Copy`/`Clone` when captures allow.
- **Constraints:** none.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Closure types).

#### RS-CLO-02 · Function-item types
- **Behavior:** Every `fn` (and each generic instantiation) has its own zero-sized function-item type, coercible to `fn(...)` pointers; `map(f)` passes the item type, not a pointer, until coercion.
- **Observable content:** A generic fn instantiated twice yielding distinct item types (probe via a type-mismatch case), then unified via explicit `as fn(_) -> _`.
- **Variants & failures:** Diagnostics naming `fn item` vs `fn pointer` mismatches.
- **Constraints:** none.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Function item types).

#### RS-CLO-03 · Return-position `impl Trait`
- **Behavior:** RPIT returns an opaque type: callers know only the bounds *plus leaked auto traits* (Send-ness inferred from the body becomes part of the effective API). One underlying concrete type per function.
- **Observable content:** A fn returning `impl Iterator`, a caller relying on leaked `Send`; a failing version where a body change (adding an `Rc`) breaks a downstream `Send` requirement without any signature change.
- **Variants & failures:** Two branches returning different types behind one RPIT (invalid); lifetime-capture defaults differing by edition (RS-EDI-04, RS-CLO-08).
- **Constraints:** none.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Impl trait).

#### RS-CLO-04 · Argument-position `impl Trait`
- **Behavior:** APIT is sugar for an anonymous generic parameter — not nameable, not turbofish-able; mixing APIT and explicit generics changes what callers can specify.
- **Observable content:** An APIT fn beside its explicit-generic twin; a failing turbofish attempt on the APIT one.
- **Variants & failures:** `impl Trait` in `let` bindings and other positions (recent stabilizations — verify current set).
- **Constraints:** none.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Impl trait).

#### RS-CLO-05 · Async functions and state machines
- **Behavior:** `async fn f() -> T` really returns an opaque `impl Future<Output = T>` compiler-generated state machine; the future's `Send`-ness is inferred from what is held across `.await` points and leaks to spawn-site bounds — a body detail becoming a cross-crate relationship.
- **Observable content:** An async fn whose future is Send, and one holding a non-Send guard across an await, with a spawn-site failure for the latter (canonical diagnostic).
- **Variants & failures:** async blocks vs async fns; recursive async fns needing boxing.
- **Constraints:** edition ≥ 2018.
- **Defined by:** language · **Confidence:** high · **Verify:** The Rust Reference (Async).

#### RS-CLO-06 · Async fns in traits and RPITIT
- **Behavior:** Since 1.75, traits may declare `async fn` and return-position `impl Trait` methods; such traits are currently not dyn-compatible for those methods — motivating the `async_trait` ecosystem rewrite (boxed futures) where `dyn` is needed.
- **Observable content:** A native AFIT trait with two impls used generically; a parallel `#[async_trait]` version used as `dyn`; commentary that the two have different method return entities.
- **Variants & failures:** Send-bound expression problems on AFIT (`trait_variant`-style ecosystem workarounds — medium confidence on current idiom).
- **Constraints:** rustc ≥ 1.75.
- **Defined by:** language + ecosystem · **Confidence:** high · **Verify:** Rust 1.75 release announcement; The Rust Reference.

#### RS-CLO-07 · Async closures
- **Behavior:** `async || {}` with `AsyncFn`/`AsyncFnMut`/`AsyncFnOnce` bounds (stable 1.85) — a distinct callable classification from closures returning futures.
- **Observable content:** An async closure passed under an `AsyncFn` bound, contrasted with a closure returning an async block.
- **Variants & failures:** Borrowing differences between the two encodings (the motivation for the feature).
- **Constraints:** rustc ≥ 1.85.
- **Defined by:** language · **Confidence:** medium-high · **Verify:** Rust 1.85 release announcement.

#### RS-CLO-08 · Precise capturing
- **Behavior:** `impl Trait + use<'a, T>` explicitly lists the generics/lifetimes an opaque type captures (stable 1.82); edition 2024 flips the *default* captures for RPIT. What an opaque return borrows — hence which caller relationships exist — is edition- and annotation-dependent.
- **Observable content:** A fn where `use<>` omission vs. presence changes whether a caller can drop a borrowed input while holding the return value (paired accept/reject cases across editions).
- **Variants & failures:** `use<>` interactions with APIT (restrictions apply).
- **Constraints:** rustc ≥ 1.82; edition for defaults.
- **Defined by:** language · **Confidence:** medium-high · **Verify:** The Rust Edition Guide (2024 RPIT capture); Rust 1.82 release notes.

### 2.19 no_std, the facade, and platforms (RS-STD)

#### RS-STD-01 · no_std crates
- **Behavior:** `#![no_std]` removes the std prelude and links only `core` (plus `alloc` via `extern crate alloc`). The available item universe and prelude change; many crates are dual-mode via a `std` feature.
- **Observable content:** A no_std member using core+alloc; a dual-mode member with `#![cfg_attr(not(feature = "std"), no_std)]`.
- **Variants & failures:** Accidentally pulling std through a dependency (feature unification breaking no_std builds — a classic ecosystem failure).
- **Constraints:** none for library crates; binaries need handlers (RS-STD-03).
- **Defined by:** language + std structure · **Confidence:** high · **Verify:** The Rust Reference; The Embedded Rust Book.

#### RS-STD-02 · Facade identity
- **Behavior:** std re-exports core and alloc items: `std::option::Option` *is* `core::option::Option`; `Vec` lives in alloc. One entity, multiple canonical-looking cross-crate paths — a graph must either unify or deliberately distinguish them.
- **Observable content:** The same type referenced via std and core paths in different members, interoperating without conversion.
- **Variants & failures:** Doc links landing on either crate's page; no_std code naming core paths for items std users know by std paths.
- **Constraints:** none.
- **Defined by:** official implementation (std facade) · **Confidence:** high · **Verify:** std/core documentation cross-references.

#### RS-STD-03 · Graph-unique singletons
- **Behavior:** `#[global_allocator]` (a static implementing `GlobalAlloc`) and `#[panic_handler]` (no_std) must be unique per final artifact; duplicates are link-stage errors. Whole-graph uniqueness constraints attached to items.
- **Observable content:** A custom global allocator in the binary; commentary (or a quarantined case) on the duplicate error.
- **Variants & failures:** `#[alloc_error_handler]` remains nightly (medium confidence).
- **Constraints:** one per linked artifact.
- **Defined by:** language + implementation · **Confidence:** high · **Verify:** The Rust Reference; std `GlobalAlloc` docs.

#### RS-STD-04 · Target-conditional std API
- **Behavior:** Portions of std exist only per target family (`std::os::unix`, `std::os::windows`); resolution of the same path succeeds or fails depending on `--target`. Cross-compilation changes both the cfg set and the resolvable item universe.
- **Observable content:** cfg-gated use of `std::os::unix::fs::PermissionsExt` and a windows counterpart.
- **Variants & failures:** Compiling the wrong branch for a target (resolution failure); tier-3 targets lacking std entirely.
- **Constraints:** installed target components.
- **Defined by:** official implementation (std per-target surface) · **Confidence:** high · **Verify:** std `os` module documentation.

#### RS-STD-05 · Wasm and bridge targets
- **Behavior:** `wasm32-unknown-unknown` typically builds cdylib-style artifacts without `main`; `wasm-bindgen` attribute macros generate extern glue on both sides. Ecosystem-defined export surfaces distinct from both Rust `pub` and plain C symbols.
- **Observable content:** An optional cfg-gated wasm module with `#[wasm_bindgen]` exports (kept target-gated so default builds ignore it).
- **Variants & failures:** wasm-bindgen's generated shims/statics as macro-created entities; WASI targets differing from unknown-unknown.
- **Constraints:** wasm targets installed; ecosystem tooling.
- **Defined by:** ecosystem (+ target definitions) · **Confidence:** medium · **Verify:** the wasm-bindgen guide.

### 2.20 Toolchain and invocation (RS-TCH)

#### RS-TCH-01 · Source meaning depends on invocation
- **Behavior:** Crate name, crate type, edition, cfg set, and the extern-crate universe are all `rustc` flags, conventionally supplied by Cargo. The same file means different things under different invocations; there is no in-source ground truth for these inputs.
- **Observable content:** A note-and-demo pairing one file with two rustc invocations (different `--cfg`/`--edition`) yielding different accepted programs; the Cargo-driven equivalents elsewhere in the checklist.
- **Variants & failures:** `--crate-name` overriding the filename-derived default.
- **Constraints:** none.
- **Defined by:** official implementation · **Confidence:** high · **Verify:** The rustc Book (command-line arguments).

#### RS-TCH-02 · Out-of-manifest build configuration
- **Behavior:** `.cargo/config.toml` (discovered hierarchically) and environment variables (`RUSTFLAGS`, `RUSTDOCFLAGS`, `CARGO_*`) inject rustflags, default targets, aliases, env values, and source replacement — build behavior configured outside the manifest and often outside the repository.
- **Observable content:** A checked-in `.cargo/config.toml` with a rustflags addition and an alias; documentation of the hierarchy/precedence.
- **Variants & failures:** RUSTFLAGS changes invalidating caches and changing cfg-visible behavior (`--cfg` via RUSTFLAGS); config outside the repo making builds environment-dependent.
- **Constraints:** Cargo.
- **Defined by:** official implementation · **Confidence:** high · **Verify:** The Cargo Book (Configuration).

#### RS-TCH-03 · Proc-macro execution environment
- **Behavior:** Proc macros are arbitrary host code executed by the compiler with filesystem/network/env access and no sandbox; expansion output — hence the compiled entity set — can depend on the build environment (sqlx consulting a database; macros reading files).
- **Observable content:** A benign environment-reading proc macro (or a documented dependency exhibiting it) with the environment input pinned for reproducibility.
- **Variants & failures:** Nondeterministic expansion as a graph-stability hazard; offline-mode fallbacks.
- **Constraints:** none (that is the point).
- **Defined by:** official implementation (absence of sandboxing) · **Confidence:** high · **Verify:** The Rust Reference (Procedural macros).

#### RS-TCH-04 · Alternate front-ends over one semantics
- **Behavior:** rustdoc and clippy re-drive rustc with different goals: rustdoc compiles under `cfg(doc)` and may accept/skip differently (notably around platform-gated items); clippy adds lints without changing semantics. Same source, multiple analysis passes with distinct cfg/diagnostic profiles.
- **Observable content:** A `cfg(doc)`-gated item (documented-on-all-platforms idiom); clippy-specific lint attributes (`#[allow(clippy::...)]`) in source.
- **Variants & failures:** Code that builds but breaks `cargo doc` (doc-only resolution errors, e.g. broken intra-doc links under deny).
- **Constraints:** components installed.
- **Defined by:** official implementation · **Confidence:** high (medium on rustdoc cfg differences) · **Verify:** The rustdoc Book; the Clippy documentation.

#### RS-TCH-05 · Deterministic identity across builds
- **Behavior:** Crate metadata/disambiguators distinguish otherwise-identical crates (same name+version built twice with different features/rustc); symbol hashes and incremental artifacts derive from them. Identity of a *build* of a crate is finer-grained than its name and version.
- **Observable content:** The RS-DEP-11 double-compilation and RS-DEP-03 multi-version cases, with commentary that name+version underdetermines identity.
- **Variants & failures:** `-C metadata` as the underlying flag (implementation detail).
- **Constraints:** none.
- **Defined by:** official implementation · **Confidence:** medium · **Verify:** The rustc Book (codegen options).

### 2.21 Ecosystem conventions (RS-ECO)

#### RS-ECO-01 · serde
- **Behavior:** `#[derive(Serialize, Deserialize)]` plus `#[serde(...)]` helpers (`rename_all`, `skip`, `flatten`, `tag`, `default`) generate impls whose data-model names diverge from source names; integration is conventionally feature-gated (`serde` feature) in libraries.
- **Observable content:** Derived types with renaming/flattening attributes; a feature-gated `cfg_attr` derive; a manual `impl Serialize` for contrast.
- **Variants & failures:** `serde(with = ...)`/remote derive routing through module-path indirection; derive vs. manual impl coexistence rules (one or the other).
- **Constraints:** serde + serde_derive crates (proc-macro).
- **Defined by:** ecosystem convention · **Confidence:** high · **Verify:** serde.rs documentation.

#### RS-ECO-02 · Error-handling crates
- **Behavior:** `thiserror` derives `Error`/`Display` and generates `From` impls from `#[from]` fields — the impls that `?` then routes through (RS-IMP-02); `anyhow::Error` is a type-erased error for application code. Generated conversions form real edges the source never spells.
- **Observable content:** A thiserror enum with `#[error]`/`#[from]` consumed via `?`; an anyhow-using bin layered over the typed lib errors.
- **Variants & failures:** `Box<dyn Error>` as the std-only alternative; `source()` chains.
- **Constraints:** proc-macro crates.
- **Defined by:** ecosystem convention · **Confidence:** high · **Verify:** thiserror/anyhow crate documentation.

#### RS-ECO-03 · Async runtimes (tokio)
- **Behavior:** `#[tokio::main]`/`#[tokio::test]` rewrite fns (attribute-macro entry points); `tokio::spawn` imposes `Send + 'static` bounds (surfacing RS-CLO-05); runtime crates carry heavy feature matrices (`features = ["full"]` vs. granular).
- **Observable content:** A tokio bin + async tests; one spawn-bound failure case; a granular-features manifest.
- **Variants & failures:** `async-std`/`smol` as alternatives; executor-agnostic libraries via `futures` traits only.
- **Constraints:** tokio dependency.
- **Defined by:** ecosystem convention · **Confidence:** high · **Verify:** tokio documentation.

#### RS-ECO-04 · Observability macros
- **Behavior:** `#[tracing::instrument]` wraps function bodies (attribute macro changing the compiled body); `log`/`tracing` macros (`info!`, etc.) are call sites carrying static metadata (targets, fields) resolved at compile time.
- **Observable content:** An instrumented fn; leveled macro calls with explicit `target:`.
- **Variants & failures:** `log` facade vs. `tracing` subscriber wiring; compile-time max-level features eliding call sites (feature-dependent code elimination).
- **Constraints:** dependencies.
- **Defined by:** ecosystem convention · **Confidence:** high · **Verify:** tracing/log crate documentation.

#### RS-ECO-05 · Lazy and global state
- **Behavior:** `std::sync::LazyLock`/`LazyCell` (1.80), `once_cell`, and the `lazy_static!` macro (which generates a hidden struct + `Deref` impl — macro-invented entity names) are the global-initialization idioms.
- **Observable content:** One LazyLock static, one once_cell usage, one lazy_static! (legacy) whose expansion shape is noted.
- **Variants & failures:** `OnceLock::get_or_init` runtime pattern; const-initialized statics where possible.
- **Constraints:** rustc ≥ 1.80 for the std types.
- **Defined by:** std + ecosystem · **Confidence:** high · **Verify:** std documentation; once_cell/lazy_static docs.

#### RS-ECO-06 · Companion-type-generating derives
- **Behavior:** `strum` (`EnumIter` generates a `FooIter` type; `Display`/`EnumString` impls), `derive_builder`/`typed-builder` (generate `FooBuilder` structs), `derive_more` (operator/conversion impls) — derives that mint *new named types and impls* discoverable only post-expansion.
- **Observable content:** One derive that generates a companion type which handwritten code then names and uses.
- **Variants & failures:** Companion-name collisions; documentation attribution of generated types.
- **Constraints:** proc-macro crates.
- **Defined by:** ecosystem convention · **Confidence:** high · **Verify:** the respective crates' documentation.

#### RS-ECO-07 · API-shape conventions
- **Behavior:** Library `prelude` modules (`use lib::prelude::*`), extension traits (`FooExt` with a blanket impl over a foreign type), sealed traits (public trait, private supertrait/module blocking downstream impls), and newtype-plus-Deref wrappers — recurring shapes that stress glob imports, blanket impls, visibility, and deref resolution together.
- **Observable content:** One of each pattern implemented locally: a prelude module, an Ext trait over a std type, a sealed trait with a demonstration that downstream impls fail, a Deref newtype.
- **Variants & failures:** Sealing via method-signature-with-private-type (alternative technique).
- **Constraints:** none.
- **Defined by:** ecosystem convention · **Confidence:** high · **Verify:** Rust API Guidelines (for some of these; conventions otherwise folkloric).

#### RS-ECO-08 · Re-export facades and cross-version identity
- **Behavior:** Crates re-export dependencies wholesale (`pub use dep::*`, or a `foo-core`/`foo` split where the facade re-exports core types); the "semver trick" has an old major re-export types from a new major so both graphs share one type identity. Re-exports are the identity-unification mechanism of the ecosystem.
- **Observable content:** A facade member re-exporting a core member's types; consumers importing the same type via both paths.
- **Variants & failures:** Feature-forwarding facades (`facade/std` enabling `core/std`); partial re-exports creating split APIs.
- **Constraints:** none.
- **Defined by:** ecosystem convention · **Confidence:** medium-high (semver trick specifics) · **Verify:** the semver-trick crate/essay (title from memory — verify).

#### RS-ECO-09 · Link-time registration
- **Behavior:** `inventory`/`linkme` collect distributed registrations (via link sections/ctors) across the crate graph; `ctor` runs functions before `main`. Producers and consumers are connected without any source-level reference between them — edges invisible to pure name resolution.
- **Observable content:** A plugin-style registry: several files register handlers; one consumer iterates the collected set; no direct references among them.
- **Variants & failures:** Platform-dependent linker-section behavior; registrations in unlinked (unused) crates silently missing.
- **Constraints:** platform/linker support.
- **Defined by:** ecosystem convention (on implementation-defined linkage) · **Confidence:** medium · **Verify:** inventory/linkme crate documentation.

#### RS-ECO-10 · Cross-language bridge generators
- **Behavior:** `pyo3` (`#[pyfunction]`, `#[pymodule]`), `wasm-bindgen`, `cxx`, `uniffi`, `cbindgen` export Rust entities to other languages via attribute macros and generators — creating parallel foreign-facing entity surfaces and generated glue items.
- **Observable content:** At most one bridge exercised (target-gated), or explicit exclusion notes; the graph-relevant point is macro-generated glue + a second notion of "exported".
- **Variants & failures:** Bridge macro rewriting signatures (ownership/ABI adaptation); build-time header/binding generation.
- **Constraints:** heavy external toolchain requirements — reasonable to represent by one instance only.
- **Defined by:** ecosystem convention · **Confidence:** medium · **Verify:** the respective project guides.

---

## 3. Features with materially changed support across recent versions

Approximate stabilization points (verify exact minors against official release notes):

| Feature | Change | Version | Confidence |
|---|---|---|---|
| async fn in traits + RPITIT | stabilized | 1.75 | high |
| C-string literals `c"..."` | stabilized | 1.77 | high |
| `offset_of!` | stabilized | 1.77 | medium |
| inline `const { ... }` blocks | stabilized | 1.79 | high |
| associated-type bounds (`T: Tr<Assoc: Bound>`) | stabilized | 1.79 | medium |
| `LazyLock`/`LazyCell`; check-cfg by default | stabilized | 1.80 | high |
| `Error` in `core`; `#[expect]` | stabilized | 1.81 | high |
| precise capturing `use<...>`; `&raw` pointers; `unsafe extern` / unsafe attributes usable | stabilized | 1.82 | medium-high |
| expanded const capabilities (`&mut` in const, etc.) | incremental | 1.83+ | medium |
| MSRV-aware resolver (`resolver = "3"`) | stabilized | 1.84 | high |
| edition 2024; async closures (`AsyncFn*`) | shipped | 1.85 | high |
| trait-object upcasting | stabilized | 1.86 | high |
| let-chains (edition 2024 only); naked functions | stabilized | 1.88 | medium-high |
| `cfg(true)`/`cfg(false)` | stabilized | ~1.88 | medium |
| explicit inferred const args `_` and later const-generic conveniences | incremental | 1.89+ | low-medium |
| Cargo: workspace inheritance | added | 1.64 | high |
| Cargo: `[lints]` table | added | 1.74 | high |
| GATs; `let ... else` | stabilized | 1.65 | high |
| edition 2021 | shipped | 1.56 | high |

Post-1.88 stabilizations are where this report's version knowledge is least reliable.

---

## 4. Implementation-defined and unspecified behavior

- **No independent standard.** The Rust Reference is the primary description but disclaims full normativity; rustc's behavior is the practical definition. The Ferrocene Language Specification is a parallel specification effort.
- **Type layout** without an explicit `repr` is unspecified (field ordering, padding, niche optimizations such as `Option<&T>` being pointer-sized). Programs and analyses must not assume default layout.
- **Symbol mangling** scheme (legacy vs. v0) is implementation-defined and flag-selectable; mangled names are not stable identities.
- **Integer overflow** is configuration-defined, not undefined: panic when overflow checks are on (debug default), two's-complement wrap when off (release default) — semantics selected by profile.
- **Address identity** of consts, ZSTs, and functions is not guaranteed unique or stable (consts are inlined; identical functions may merge or duplicate across codegen units).
- **HashMap iteration order** is randomized per-instance at runtime (RandomState) — a runtime nondeterminism convention worth distinguishing from compile-time determinism.
- **Macro hygiene** details (especially interactions of `macro_rules!` mixed-site hygiene with proc-macro spans) are only partially documented.
- **Diagnostics** (message text, suggestion structure, and to a lesser degree error codes) are versioned implementation output, not language artifacts.
- **Trait-solver internals** (the "next-generation" solver migration) may shift acceptance of edge-case programs between versions without an edition boundary.
- **Const evaluation** is deterministic by design; drop order and evaluation order are *defined* (not implementation-defined) — worth stating because it surprises people in both directions.
- **Proc-macro expansion** is deterministic only if the macro is; the platform imposes no purity (RS-TCH-03).

---

## 5. Areas where this report's knowledge may be incomplete

- Exact stabilization minor-versions after ~1.85, and everything landed after mid-2025 (releases ~1.90+), including any newly stabilized syntax this checklist omits entirely.
- Edition 2024 fine print: default lint levels (`unsafe_op_in_unsafe_fn`), `expr`/`expr_2021` fragment matching details, temporary-lifetime adjustments in tail expressions.
- Current names/levels of the privacy lint family (`private_interfaces`/`private_bounds`) and the exact status of historical hard errors in that space.
- Cargo frontier features: public/private dependencies, artifact dependencies, per-package targets, `-Zscript` single-file packages — believed unstable at cutoff but possibly moved.
- rustdoc JSON output format stability and `doc_cfg` stabilization status.
- `raw-dylib` linking scope, `naked_functions` details, per-arch `asm!` support matrix.
- The precise checking behavior of `cargo::` (vs `cargo:`) build-script directives.
- Whether doctests are collected from private items and non-lib targets under current defaults.

---

## 6. Final audit: families a fixture might still be missing

Reviewed against the checklist; candidates deliberately *not* given full items, with reasons:

- **Pattern-matching depth** (binding modes/match ergonomics, exhaustiveness details, or-pattern semantics): mostly intra-body semantics with limited entity/relationship impact beyond RS-ATT-05; consider one item if graphs model match arms.
- **Borrow checking as such** (NLL specifics): affects validity, not graph shape; representative accept/reject pairs already appear via RS-CLO-08/RS-IMP-06.
- **Custom test frameworks** (`custom_test_frameworks`): nightly; covered in spirit by `harness = false`.
- **Single-file Cargo packages / cargo scripts**: unstable at cutoff; would change source-inclusion assumptions if stabilized — revisit.
- **Sanitizers, PGO, LTO, codegen-units, incremental**: performance/diagnostic toolchain surface without semantic-graph impact; deliberately excluded.
- **Registry operations** (publish, yank, offline mode, authentication): affect resolution inputs over time but not the semantics of a checked-out build with a lockfile; the lockfile item (RS-DEP-03) carries the graph-relevant part.
- **Internationalization of identifiers** got an item (RS-RES-11), but *crate-name* policy (ASCII, punycode-adjacent cases) is registry policy left unexplored.
- **Formal verification/dynamic tools** (miri, kani, fuzzing harnesses): out of scope by the prompt's framing (no analysis-tool recommendations); their *target layouts* (fuzz/ dirs as nested packages) could merit a structural item if fuzzing layouts are common in scope.
- **Lang items and compiler-internal attributes** (`#[lang]`, `#[rustc_*]`): nightly/internal; visible only in std-like code.
- **Metabuild / build-plan features**: unstable, low ecosystem presence.
- **GUI/embedded HAL conventions** (svd2rust-generated peripheral crates): a large generated-code family similar to RS-GEN-05; svd2rust specifically generates enormous module trees and could stress scale — flagged as an optional extension.
- **Binary-size-focused idioms** (`panic_immediate_abort`, std-features rebuilds): nightly `build-std` territory; excluded.

Nothing else surfaced during the audit that changes entity kinds, identity, resolution, dispatch, visibility, source inclusion, or diagnostics beyond what the 165 items above cover.
