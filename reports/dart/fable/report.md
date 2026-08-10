# Dart — Research Checklist for a Code-Graph Reference Project

This report enumerates Dart language, toolchain, package, and ecosystem features
whose omission from a reference project would leave materially distinct semantic
behavior unrepresented in a code graph: entity identity, name resolution,
visibility, typing, dispatch, source inclusion, configuration-dependence, and
diagnostics. It is derived from model knowledge only. It does not propose a
fixture design, theme, schema, or scoring system.

## 1. Baseline Version and Implementation Assumptions

- **Language baseline:** Dart 3.8 (stable, May 2025), with notes where earlier
  or later versions differ. All code is assumed sound null-safe; Dart 3 SDKs
  refuse language versions below 2.12.
- **Implementation:** the official Dart SDK is the only maintained
  implementation. It contains two front ends that can disagree at the margins:
  the **analyzer** (IDE/static analysis) and the **common front end (CFE)**
  used by all compilers. Backends: VM JIT, native AOT (`dart compile
  exe`/`aot-snapshot`), dart2js, DDC (dev web), dart2wasm. Kernel (`.dill`) is
  the shared intermediate form. There is no independent second implementation
  to cross-check against.
- **Package management:** pub only. Resolution is materialized in
  `pubspec.lock` and `.dart_tool/package_config.json`; the latter is the ground
  truth for `package:` URI → directory mapping and per-package language
  version.
- **Project shape:** a pub package (or pub workspace) with `lib/`, `bin/`,
  `test/`, `analysis_options.yaml`, and optionally generated sources via
  `build_runner`.
- **Specification status:** the published Dart Language Specification lags the
  shipped language; post-2.10 features are normatively defined by accepted
  feature specifications in the `dart-lang/language` GitHub repository plus SDK
  behavior. Verification pointers below reflect that reality.

## 2. Checklist

Each item: identifier, name, behavior, observable demonstration, variants and
failure cases, constraints, defining authority, confidence, and a verification
source where known.

---

### Category A — Package, Workspace, and Source Layout (PKG)

#### PKG-01 Package manifest and the `package:` namespace
- **Behavior:** `pubspec.yaml` `name:` defines the package identity and the
  `package:<name>/...` URI namespace rooted at `lib/`. Package name need not
  match its directory name.
- **Demonstrate:** a pubspec with `name`, `environment: sdk:`; intra- and
  cross-package `package:` imports.
- **Variants/failures:** invalid names (uppercase, hyphens) are rejected by
  pub; self-imports via `package:` from inside `lib/` are idiomatic.
- **Defined by / confidence:** pub (official tool convention wired into URI
  resolution) · High.
- **Verify:** dart.dev package/pubspec documentation.

#### PKG-02 `lib/` vs `lib/src/` public-API convention
- **Behavior:** only `lib/` (outside `lib/src/`) is conventionally public;
  importing another package's `lib/src/` is legal at the language level but
  flagged by the `implementation_imports` lint.
- **Demonstrate:** a package whose public library re-exports `src/` files;
  another package importing both the public library and (illegitimately) a
  `src/` file.
- **Variants/failures:** graph must distinguish convention-visibility from
  language-visibility; nothing breaks at compile time.
- **Defined by / confidence:** ecosystem convention + linter · High.
- **Verify:** dart.dev package layout conventions; `package:lints` docs.

#### PKG-03 Entry points: `bin/`, `executables:`, and script running
- **Behavior:** each `bin/*.dart` with a `main` is an entry point; pubspec
  `executables:` maps command names to bin scripts (`dart run pkg:cmd`,
  `dart pub global activate`). `test/`, `example/`, `tool/` hold additional
  roots not reachable from `lib/`.
- **Demonstrate:** ≥2 bin entry points, an `executables:` mapping, an example
  program importing the package's public API.
- **Variants/failures:** entry-point-only reachability differs per root; code
  reachable from `test/` but not `bin/` still belongs to the project graph.
- **Defined by / confidence:** pub convention (official) · High.
- **Verify:** dart.dev pubspec documentation.

#### PKG-04 Dependency kinds and overrides
- **Behavior:** `dependencies`, `dev_dependencies`, `dependency_overrides`
  with hosted (semver constraints, `^` caret), `git:` (url/ref/path), `path:`,
  and `sdk: flutter` sources; `pubspec_overrides.yaml` overrides without
  touching the manifest.
- **Demonstrate:** at least hosted + path deps; an override redirecting a
  hosted dep to a path.
- **Variants/failures:** dev deps are invisible to `lib/` code by policy
  (analyzer lint `depend_on_referenced_packages`); overrides change which
  source tree a `package:` URI resolves to — same URI, different files.
- **Defined by / confidence:** pub (official) · High.
- **Verify:** dart.dev dependency documentation.

#### PKG-05 Lockfile and `package_config.json` as resolution ground truth
- **Behavior:** `dart pub get` writes `pubspec.lock` (versions, content
  hashes) and `.dart_tool/package_config.json` (URI roots, per-package
  `languageVersion`). All compilers and the analyzer resolve `package:` URIs
  through package_config, not through pubspec directly.
- **Demonstrate:** committed lockfile for an app; a graph consumer can only be
  correct if it honors package_config rather than guessing.
- **Variants/failures:** stale package_config (edited pubspec without `pub
  get`) makes analysis diverge from manifest intent.
- **Defined by / confidence:** official implementation · High.
- **Verify:** dart.dev pub documentation; `package_config` file format docs in
  the dart-lang ecosystem.

#### PKG-06 Pub workspaces (monorepo resolution)
- **Behavior:** Dart 3.6+ root pubspec declares `workspace:` members; member
  pubspecs declare `resolution: workspace`; one shared lockfile and one
  package_config cover all members, and path-siblings resolve to each other.
- **Demonstrate:** a root workspace with ≥2 member packages that import each
  other by `package:` URI.
- **Variants/failures:** pre-3.6 monorepos use per-package `path:` deps —
  materially different resolution artifacts; mixing the two styles.
- **Defined by / confidence:** pub (official) · High (feature), Medium (edge
  semantics).
- **Verify:** dart.dev workspaces documentation; Dart SDK CHANGELOG.

#### PKG-07 Language versioning: per-package default, per-file override
- **Behavior:** a package's default language version derives from the lower
  bound of its `environment: sdk:` constraint. A file may override downward
  with a `// @dart=2.19`-style comment before any code. Part files cannot
  override; they inherit the library's version.
- **Demonstrate:** one file pinned to an older language version inside a 3.x
  package (e.g., where records/patterns are syntax errors and `_` still
  binds).
- **Variants/failures:** the same token stream is valid in one file and an
  error in a sibling; graph must be language-version-aware per library.
- **Defined by / confidence:** language + official tools · High.
- **Verify:** dart.dev language-versioning documentation.

#### PKG-08 Standalone script mode (no package)
- **Behavior:** `dart run lone.dart` outside any package resolves only
  `dart:` imports (plus, in recent SDKs, an ephemeral resolution when a
  pubspec is absent); language version defaults to the SDK's current version.
  Shebang lines (`#!/usr/bin/env dart`) are tolerated.
- **Demonstrate:** a self-contained script alongside the package, exercising
  the "no package_config" resolution path.
- **Variants/failures:** `package:` imports fail to resolve; behavior differs
  from in-package files with identical content.
- **Defined by / confidence:** official implementation · Medium.
- **Verify:** `dart` CLI documentation.

---

### Category B — Libraries, Directives, and Parts (LIB)

#### LIB-01 Library as the unit of identity
- **Behavior:** the library (one `.dart` file plus its parts) is the unit of
  privacy, scope, and identity. `library;` (unnamed, 2.19+) or `library
  a.b.c;` optionally names it; annotations and library-level doc comments
  attach to the directive. Most files omit the directive entirely.
- **Demonstrate:** libraries with no directive, with `library;` plus metadata,
  and with a legacy dotted name referenced by an old-style `part of`.
- **Variants/failures:** library names are not identities — URIs are; two
  libraries may share a name.
- **Defined by / confidence:** language · High.
- **Verify:** Dart Language Specification; dart.dev libraries documentation.

#### LIB-02 `part` / `part of` (classic parts)
- **Behavior:** parts textually join a library: they share its imports, scope,
  and privacy; they cannot declare their own imports (pre-enhanced-parts).
  `part of 'lib.dart';` (URI form) vs legacy `part of lib.name;`. Generated
  `.g.dart` files are overwhelmingly parts.
- **Demonstrate:** a multi-part library where a part references another part's
  private members; both `part of` forms.
- **Variants/failures:** part-of/part mismatch is a compile error; a part
  included by two libraries is an error; lint `use_string_in_part_of_directives`
  discourages the name form.
- **Defined by / confidence:** language · High.
- **Verify:** Dart Language Specification.

#### LIB-03 Import combinators and duplicate-import merging
- **Behavior:** `show`/`hide` filter an import's namespace; several imports of
  the same library (with different combinators) merge into one namespace; the
  same name imported from two libraries is only an error at the use site.
- **Demonstrate:** `show`, `hide`, repeated imports of one library, and a
  name-collision that is resolved by combinators.
- **Variants/failures:** combinators also filter extensions in/out of implicit
  applicability (`hide SomeExtension`).
- **Defined by / confidence:** language · High.
- **Verify:** Dart Language Specification.

#### LIB-04 Prefixed imports
- **Behavior:** `import '...' as p;` puts the imported namespace behind a
  prefix entity; multiple imports may share one prefix (namespaces merge). A
  prefix is not an expression. Deferred imports must have a unique prefix.
- **Demonstrate:** shared prefixes across two imports; prefixed access
  `p.Foo`; explicit extension application through a prefix (`p.Ext(x).m()`).
- **Variants/failures:** extensions imported behind a prefix still apply
  implicitly (they cannot be invoked as `p.member` but are in scope);
  prefix/type name collisions.
- **Defined by / confidence:** language · High (Medium for the prefixed
  extension nuance).
- **Verify:** Dart Language Specification; dart.dev extension methods docs.

#### LIB-05 Exports and re-export chains
- **Behavior:** `export` splices another library's public namespace into this
  library's exported API; `show`/`hide` filter it; chains and diamonds are
  common ("barrel" files). The same declaration reachable via multiple export
  paths is one entity, not two.
- **Demonstrate:** a public barrel library exporting several `src/` libraries;
  a diamond where two barrels export the same declaration; a filtered export.
- **Variants/failures:** two *different* declarations with one name exported
  into the same namespace is a compile error; export cycles are legal.
- **Defined by / confidence:** language · High.
- **Verify:** Dart Language Specification.

#### LIB-06 Deferred loading
- **Behavior:** `import '...' deferred as d;` defers a library into a separate
  load unit: members only via the prefix, only after `d.loadLibrary()`.
  Deferred types cannot appear in type annotations; constants from deferred
  libraries are restricted. On the VM `loadLibrary` completes trivially; on
  dart2js it drives real code splitting.
- **Demonstrate:** a deferred import with an awaited `loadLibrary()` and a
  post-load call.
- **Variants/failures:** use-before-load throws; the same program has
  materially different load-unit structure per backend.
- **Defined by / confidence:** language + implementation-defined splitting ·
  High.
- **Verify:** dart.dev deferred loading documentation.

#### LIB-07 Conditional imports and exports
- **Behavior:** `import 'io.dart' if (dart.library.js_interop) 'web.dart';`
  selects a URI per compile target using `dart.library.*` environment
  constants; also valid on `export`. The analyzer resolves the *default*
  (first) URI; each compiler resolves per target.
- **Demonstrate:** the stub/io/web three-file idiom where an API is defined by
  a stub and implemented per platform.
- **Variants/failures:** all branches must have the same API to typecheck
  everywhere; a graph that models only the default URI silently misses
  platform variants.
- **Defined by / confidence:** language · High.
- **Verify:** dart.dev library tour / conditional import documentation.

#### LIB-08 `dart:core` implicit import
- **Behavior:** every library implicitly imports `dart:core`. Hiding a core
  name requires an explicit `import 'dart:core' hide print;`. `Future` and
  `Stream` are exported by `dart:core` (since 2.1), but most of `dart:async`
  still requires an explicit import.
- **Demonstrate:** a library that shadows a core name and one that hides it
  via explicit core import.
- **Variants/failures:** local declarations shadow core silently — resolution
  target changes with no diagnostic.
- **Defined by / confidence:** language · High.
- **Verify:** Dart Language Specification; `dart:core` API docs.

#### LIB-09 Import cycles are legal
- **Behavior:** libraries may import each other cyclically; resolution and
  compilation succeed. Only certain inference cycles (see ERR-06) and const
  cycles are errors.
- **Demonstrate:** two libraries with mutual imports and cross-references.
- **Variants/failures:** cycles interact with top-level inference ordering.
- **Defined by / confidence:** language · High.
- **Verify:** Dart Language Specification.

#### LIB-10 URI identity: `package:` vs relative paths into `lib/`
- **Behavior:** libraries are identified by canonical URI. Reaching a file
  under `lib/` via a relative path from outside `lib/` (e.g., from `test/`)
  historically yields a *different* library identity than the `package:` URI —
  duplicated types, incompatible privates. Lints `avoid_relative_lib_imports`
  and `always_use_package_imports` guard this.
- **Demonstrate:** correct `package:` self-imports from `test/`; the report of
  the anti-pattern belongs in diagnostics, not the fixture's happy path.
- **Variants/failures:** whether modern tools fully canonicalize both spellings
  varies by tool and era — a genuine identity edge case for graphs.
- **Defined by / confidence:** official implementation + linter · Medium
  (current tool behavior), High (the hazard exists).
- **Verify:** `package:lints` / linter rule documentation.

#### LIB-11 `dart:` library availability per platform
- **Behavior:** `dart:io`, `dart:ffi`, `dart:isolate`, `dart:mirrors` exist
  only on native; `dart:js_interop`, and legacy `dart:html`/`dart:js`, only on
  web. Importing an unavailable library is a compile error for that target
  while the analyzer accepts it or not based on configuration.
- **Demonstrate:** platform-specific libraries isolated behind conditional
  imports (LIB-07); pubspec `platforms:` metadata.
- **Variants/failures:** `dart:mirrors` is additionally unavailable under AOT
  and dart2js even though it is "native" (PLAT-06).
- **Defined by / confidence:** official implementation · High.
- **Verify:** dart.dev core-library overview.

#### LIB-12 Enhanced parts / augmentations (experimental, uncertain)
- **Behavior:** an accepted feature direction lets part files carry their own
  `import`/`export` directives and lets `augment` declarations split one
  entity's definition across files. Work continued after the macros
  cancellation (see CGEN-05) but stable availability is unclear at baseline.
- **Demonstrate:** only under an experiment flag, if at all; a graph should at
  minimum not crash on `augment` syntax.
- **Variants/failures:** an augmentation makes one semantic entity out of
  several declaration sites — a direct challenge to entity identity.
- **Defined by / confidence:** language (accepted feature spec), shipping
  status uncertain · Low.
- **Verify:** dart-lang/language feature specifications (GitHub); SDK
  CHANGELOG.

---

### Category C — Scoping, Privacy, and Name Resolution (VIS)

#### VIS-01 Underscore privacy is library-scoped, not class-scoped
- **Behavior:** `_name` is private to the *library* (including all parts).
  Two classes in one library see each other's private members; a class and a
  top-level `_helper` share one privacy domain.
- **Demonstrate:** cross-class private access within a library; a part file
  using the main file's privates; the same `_name` text in two libraries as
  two unrelated entities.
- **Variants/failures:** private names are per-library symbols — textual
  identity is not semantic identity.
- **Defined by / confidence:** language · High.
- **Verify:** Dart Language Specification.

#### VIS-02 Private members across library boundaries
- **Behavior:** a subclass in another library inherits private member
  *implementations* but cannot access, override, or implement them. Declaring
  `_m` in the foreign subclass creates an unrelated member; the inherited
  method's internal `_m()` calls still bind to the original library's symbol.
- **Demonstrate:** base class with a private method called from a public
  template method; a subclass in another library declaring a same-text private
  that does *not* override.
- **Variants/failures:** a class in another library implementing an interface
  with private members cannot satisfy them (error unless `noSuchMethod`,
  DSP-03).
- **Defined by / confidence:** language · High.
- **Verify:** Dart Language Specification.

#### VIS-03 Lexical scoping and shadowing
- **Behavior:** blocks nest inside members inside class scope inside library
  scope; inner declarations shadow outer including `dart:core` names and
  instance members (requiring `this.` to disambiguate). String interpolation
  (`$x`, `${e}`) resolves identifiers in the enclosing scope — reference
  edges hide inside string literals.
- **Demonstrate:** a local shadowing a field, a parameter shadowing a
  top-level, interpolation referencing locals and getters.
- **Variants/failures:** static and instance members of one class cannot share
  a base name (error); referring to a local before its declaration is an error
  even if an outer binding exists.
- **Defined by / confidence:** language · High.
- **Verify:** Dart Language Specification.

#### VIS-04 Ambiguity resolved at use site
- **Behavior:** importing the same name from two libraries is fine until the
  name is *used* unprefixed — then a compile error; combinators or prefixes
  resolve it. The same declaration arriving twice via re-export diamonds is
  not ambiguous.
- **Demonstrate:** a controlled collision fixed by `hide` in one import; the
  diamond non-error.
- **Variants/failures:** ambiguity between applicable extensions is a
  different mechanism (EXT-02).
- **Defined by / confidence:** language · High.
- **Verify:** Dart Language Specification.

#### VIS-05 Ecosystem visibility contracts (`package:meta`)
- **Behavior:** `@internal`, `@visibleForTesting`, `@protected`,
  `@visibleForOverriding` narrow *effective* visibility beyond the language;
  the analyzer diagnoses violations as hints/warnings, compilers ignore them.
- **Demonstrate:** an `@visibleForTesting` member used legally from `test/`
  and diagnosably from `lib/` of another package.
- **Variants/failures:** graphs modeling only language visibility miss the
  API surface developers actually honor.
- **Defined by / confidence:** ecosystem (`package:meta`) + analyzer · High.
- **Verify:** `package:meta` API documentation.

#### VIS-06 Wildcard variables (`_` non-binding, Dart 3.7)
- **Behavior:** since 3.7, local declarations named `_` (locals, parameters,
  type parameters, catch clause variables, pattern variables) bind nothing;
  several `_` parameters may coexist; `_` cannot be read. Top-level and member
  declarations named `_` still bind (and an `extension _ on X` idiom exists).
- **Demonstrate:** multiple `_` parameters in one signature; a pre-3.7
  language-versioned file where `_` is an ordinary variable.
- **Variants/failures:** version-sensitive identity: the same code declares an
  entity at 3.6 and none at 3.7.
- **Defined by / confidence:** language · High.
- **Verify:** dart-lang/language wildcard-variables feature specification; SDK
  CHANGELOG.

#### VIS-07 Doc-comment references and doc imports
- **Behavior:** `///` comments resolve square-bracket references (`[Foo]`,
  `[Foo.bar]`) against the library scope; `@docImport` directives in library
  doc comments add doc-only scope without a real dependency edge. dartdoc and
  the analyzer resolve these.
- **Demonstrate:** doc references to members, to prefixed names, and a
  `@docImport`; a dangling reference the analyzer flags
  (`comment_references` lint).
- **Variants/failures:** doc-only edges vs code edges must be distinguishable.
- **Defined by / confidence:** official tooling (dartdoc/analyzer), ecosystem
  convention · Medium (doc imports), High (references).
- **Verify:** dartdoc documentation.

---

### Category D — Type Declarations (CLS)

#### CLS-01 Classes and implicit interfaces
- **Behavior:** every class induces an implicit interface of its instance
  members; `implements C` adopts the interface without inheriting
  implementation, `extends C` inherits both. One entity, two roles.
- **Demonstrate:** a class consumed via `extends` in one place and
  `implements` in another, including implementing a concrete class.
- **Variants/failures:** `implements` requires re-implementing everything
  (including fields, as getter/setter pairs); private members block foreign
  implementers (VIS-02).
- **Defined by / confidence:** language · High.
- **Verify:** Dart Language Specification.

#### CLS-02 Class modifiers (Dart 3.0)
- **Behavior:** `base` (extendable, not implementable outside its library),
  `interface` (implementable, not extendable outside), `final` (neither
  outside), `sealed` (implies abstract; direct subtypes must live in the same
  library; enables exhaustiveness), `abstract` combinations, and `mixin
  class` (usable as class and mixin). Subtypes of `base`/`final`/`sealed`
  must themselves be `base`, `final`, or `sealed` — enforced transitively.
- **Demonstrate:** a sealed hierarchy consumed exhaustively (PAT-01); a
  `final` class rejected for `extends` in a sibling *package*; an `interface
  class` implemented across libraries; a `base mixin`.
- **Variants/failures:** enforcement is at the library boundary — same-library
  code may bypass capability limits; pre-3.0 language-versioned classes remain
  mixable (grandfathering); `@reopen` (META-03) silences the reopening lint.
- **Defined by / confidence:** language · High (core), Medium (grandfathering
  edge rules).
- **Verify:** dart.dev class-modifiers documentation; dart-lang/language
  feature specification.

#### CLS-03 Mixins and mixin application
- **Behavior:** `mixin M on A, B { ... }` declares composable member bundles
  with superinterface constraints; `class C extends S with M1, M2` applies
  them by linearization (later wins for member conflicts, DSP-04); `class C =
  S with M;` is a *named* mixin application — a distinct declaration with no
  body. Mixins also induce implicit interfaces (`implements M` is legal).
- **Demonstrate:** an `on`-constrained mixin using `super.` calls; ordering
  where `M2` overrides `M1`; a named mixin application; a `mixin class` used
  both ways.
- **Variants/failures:** since 3.0 plain `class` cannot be mixed in; mixins
  cannot declare generative constructors (const factory forwarding aside);
  applying a mixin whose `on` type is unsatisfied is an error.
- **Defined by / confidence:** language · High.
- **Verify:** Dart Language Specification; dart.dev mixins documentation.

#### CLS-04 Enhanced enums (Dart 2.17)
- **Behavior:** enums may declare fields, methods, getters, const generative
  constructors, type parameters, `implements`, and `with`. Each value is a
  distinct const instance; `values`, `index`, and `name` are available
  (`name` via the `EnumName` extension in `dart:core`). Enums cannot be
  extended or explicitly instantiated; all enums implement `Enum`.
- **Demonstrate:** an enum with constructor arguments per value, an interface
  implementation, a mixin, and generic type arguments on values.
- **Variants/failures:** switch exhaustiveness over enums (with and without
  `default`); a member named `values` is an error; pre-2.17 files reject the
  syntax.
- **Defined by / confidence:** language · High.
- **Verify:** dart-lang/language enhanced-enums feature specification.

#### CLS-05 Extension types (Dart 3.3)
- **Behavior:** `extension type Meters(int value) { ... }` declares a
  compile-time-only wrapper: static type distinct, runtime type erased to the
  representation. `implements` may expose representation-type members or other
  extension types. Members dispatch statically; `is`/`as` and generics observe
  the erasure (`Meters` is `int` at runtime; `List<Meters>` is a
  `List<int>`).
- **Demonstrate:** a named extension type with constructors, operators, static
  members, an `implements` clause, and a demonstration that `x is Meters`
  succeeds for a plain `int`.
- **Variants/failures:** `@redeclare` when shadowing an interface member;
  const constructors; extension types over nullable representations;
  they are the foundation of modern JS interop (PLAT-08).
- **Defined by / confidence:** language · High (Medium on top-type subtleties).
- **Verify:** dart.dev extension-types documentation; dart-lang/language
  feature specification.

#### CLS-06 Generic declarations, bounds, raw types
- **Behavior:** classes, mixins, extensions, extension types, typedefs,
  functions, and methods take type parameters with `extends` bounds,
  including F-bounded (`T extends Comparable<T>`). Omitted arguments
  instantiate to bounds ("raw types"); `strict-raw-types` analyzer mode turns
  raw usage into diagnostics (config-dependent).
- **Demonstrate:** bounded and F-bounded parameters; raw-type usage with and
  without strict mode; generic methods whose arguments are inferred.
- **Variants/failures:** self-referencing bound cycles are errors; type
  parameters are in scope in `on` clauses and initializers but not in static
  members.
- **Defined by / confidence:** language (+ analyzer config for strictness) ·
  High.
- **Verify:** Dart Language Specification.

#### CLS-07 Typedefs and type aliases
- **Behavior:** legacy function typedefs (`typedef int F(String s);`), modern
  function aliases (`typedef F = int Function(String);`), and generalized
  aliases for *any* type (2.13+), including class aliases usable to invoke
  constructors (`typedef L = List; L.filled(...)`) and aliases of record
  types. Aliases are transparent for subtyping but are distinct declaration
  entities.
- **Demonstrate:** all three forms; a class alias used as a constructor and
  (if accepted by the toolchain) for static access; an alias for a record
  type used in signatures.
- **Variants/failures:** alias cycles are errors; graphs must decide whether
  edges point at the alias, the aliased type, or both.
- **Defined by / confidence:** language · High (Medium for static-member
  access through aliases).
- **Verify:** dart-lang/language generalized-type-aliases feature spec.

#### CLS-08 Field flavors and implicit accessors
- **Behavior:** instance/static fields, `final`, `late`, `late final`,
  `const` (static only), `abstract` fields (declare accessor interface
  without storage), `external` fields (accessors supplied elsewhere, e.g.
  FFI/interop). Every field induces an implicit getter (and setter when
  non-final) — the *accessors*, not the storage, are the interface.
- **Demonstrate:** each flavor; overriding a field with an explicit
  getter/setter pair; implementing a field from an interface with a computed
  getter.
- **Variants/failures:** `late final` allows one runtime assignment; `late`
  without initializer defers definite-assignment errors to runtime (NUL-03).
- **Defined by / confidence:** language · High.
- **Verify:** Dart Language Specification.

#### CLS-09 Getters and setters as first-class members
- **Behavior:** getters and setters are distinct members that happen to share
  a base name; a getter can come from one supertype and the setter from
  another; explicit accessors are indistinguishable at use sites from field
  access.
- **Demonstrate:** getter-only, setter-only, split-inheritance of the pair,
  and an override of only one half.
- **Variants/failures:** modern Dart relaxed the historical requirement that
  getter and setter types correspond — worth one deliberate mismatch case
  (analyzer may still lint).
- **Defined by / confidence:** language · High (Medium on the exact version
  that relaxed getter/setter type correspondence).
- **Verify:** Dart Language Specification; SDK CHANGELOG.

#### CLS-10 Operators and callable objects
- **Behavior:** user-defined operators (`+`, `-` incl. unary, `[]`, `[]=`,
  `==`, `~/`, `>>>` (2.14+), etc.) are instance members with mangled-name
  identity; a `call` method makes instances invocable as `obj(args)` and
  assignable to function types via implicit `call` tear-off.
- **Demonstrate:** an indexable, comparable class with `==`/`hashCode`
  overridden together; a callable class passed where a function type is
  expected.
- **Variants/failures:** operators cannot be torn off; `==` is never invoked
  with a null argument (language handles null first); defining `==` without
  `hashCode` is a classic lint (`hash_and_equals`).
- **Defined by / confidence:** language · High.
- **Verify:** Dart Language Specification.

---

### Category E — Constructors and Instantiation (CTOR)

#### CTOR-01 Generative constructors, initializer lists, initializing formals
- **Behavior:** `C(this.x, {required this.y}) : z = x + 1, assert(x >= 0),
  super.named(...)` — initializing formals bind fields before the body;
  initializer lists run before super constructors; asserts may appear in the
  list.
- **Demonstrate:** initializer-list computation, an assert, explicit
  super-constructor invocation, field-initializer ordering.
- **Variants/failures:** final fields must be initialized by the list/formals;
  `this` is inaccessible in initializer lists (except initializing formals).
- **Defined by / confidence:** language · High.
- **Verify:** Dart Language Specification.

#### CTOR-02 Named constructors and `new` as a name
- **Behavior:** `C.name()` constructors are per-class named entities; the
  unnamed constructor is addressable as `C.new` (for tear-offs and
  redirects). Constructor identity is (type, name) — distinct from methods.
- **Demonstrate:** multiple named constructors; `C.new` used in a tear-off
  and a redirect target.
- **Variants/failures:** a constructor and a static member cannot share a
  name; named constructors are not inherited.
- **Defined by / confidence:** language · High.
- **Verify:** Dart Language Specification.

#### CTOR-03 Factory constructors, incl. redirecting factories
- **Behavior:** `factory C(...) { return ...; }` may return any subtype
  instance (caching, singletons, parsing); `factory C() = D.name;` redirects
  to another type's constructor — a cross-type edge with type-argument
  substitution; `const factory` redirects preserve constness.
- **Demonstrate:** a caching factory, a redirecting factory to a private
  subclass (classic pattern: public interface, hidden implementation), a
  const factory redirect.
- **Variants/failures:** factories cannot use initializer lists or `this`;
  redirect chains must terminate.
- **Defined by / confidence:** language · High.
- **Verify:** Dart Language Specification.

#### CTOR-04 Redirecting generative constructors
- **Behavior:** `C.a() : this.b(1);` forwards to a sibling constructor of the
  same class; no body allowed.
- **Demonstrate:** one redirecting generative constructor in a class with
  several constructors.
- **Variants/failures:** redirect cycles are errors.
- **Defined by / confidence:** language · High.
- **Verify:** Dart Language Specification.

#### CTOR-05 Const constructors and instantiation contexts
- **Behavior:** `const` constructors require all fields final and enable
  compile-time instances; `const C(1)` and an implicit const context (inside
  const literals, metadata, defaults) canonicalize structurally identical
  instances to one object.
- **Demonstrate:** explicit and implicit const instantiation; `identical`
  checks; a const object used in metadata and as a default parameter value.
- **Variants/failures:** invoking a const constructor with non-const args in
  a const context is a compile error; `new`ing a const constructor produces
  fresh objects.
- **Defined by / confidence:** language · High.
- **Verify:** Dart Language Specification.

#### CTOR-06 Super parameters (Dart 2.17)
- **Behavior:** `C.sub(super.x, {super.y})` forwards parameters to the
  implied super constructor without restating them; interacts with positional
  ordering and named-parameter merging.
- **Demonstrate:** positional and named super parameters, mixed with `this.`
  formals.
- **Variants/failures:** pre-2.17 files reject the syntax; cannot combine a
  super parameter with an explicit argument for the same slot.
- **Defined by / confidence:** language · High.
- **Verify:** dart-lang/language super-parameters feature specification.

#### CTOR-07 Default-constructor synthesis, non-inheritance, private constructors
- **Behavior:** a class with no constructors gets an implicit `C()` calling
  the unnamed super constructor; constructors are never inherited; a class
  with only private constructors (`C._()`) cannot be instantiated or
  (usefully) extended outside its library — the singleton / static-only
  idiom (often `abstract final class` post-3.0).
- **Demonstrate:** a class relying on the synthesized default; a singleton
  with `C._()`; a static-only utility class.
- **Variants/failures:** if the superclass lacks an accessible unnamed
  constructor, the subclass must declare one — synthesis fails.
- **Defined by / confidence:** language · High.
- **Verify:** Dart Language Specification.

#### CTOR-08 Constructor tear-offs (Dart 2.15)
- **Behavior:** `C.new`, `C.named`, and `p.C.new` are first-class function
  values; generic classes support instantiated tear-offs
  (`Map<int, String>.new` or inferred). Const-context tear-offs of the same
  constructor with the same type arguments are canonicalized.
- **Demonstrate:** `iterable.map(Wrapper.new)`; an instantiated tear-off
  assigned to a typed function variable; equality between two tear-offs of
  one constructor.
- **Variants/failures:** pre-2.15 files reject the syntax; tear-off equality
  edge cases with non-const type arguments.
- **Defined by / confidence:** language · High.
- **Verify:** dart-lang/language constructor-tearoffs feature specification.

---

### Category F — Functions, Parameters, Function Values (FN)

#### FN-01 Function kinds and closure capture
- **Behavior:** top-level functions, static methods, instance methods, local
  named functions, and anonymous functions/lambdas (block and `=>` bodies).
  Closures capture *variables*, not values; each loop iteration (both
  `for (var i ...)` and `for-in`) introduces a fresh binding, so closures
  capture per-iteration values.
- **Demonstrate:** a list of closures created in a loop observing distinct
  captured values; a local function capturing and mutating an enclosing
  variable.
- **Variants/failures:** capture of mutable variables defeats type promotion
  (TYP-04 interaction).
- **Defined by / confidence:** language · High.
- **Verify:** Dart Language Specification.

#### FN-02 Parameter shapes and defaults
- **Behavior:** required positional, optional positional `[a = 1]`, named
  `{a}`, `{required a}` — named and optional-positional cannot mix in one
  signature; defaults must be constant; named arguments may appear anywhere
  among positional ones at call sites (2.17).
- **Demonstrate:** each shape; a call site interleaving named args before
  positional; required named params with nullable and non-nullable types.
- **Variants/failures:** omitting a required named arg is a compile error; a
  non-nullable optional without default is an error.
- **Defined by / confidence:** language · High.
- **Verify:** Dart Language Specification.

#### FN-03 Function types and subtyping
- **Behavior:** function types are structural: parameter contravariance,
  return covariance, named-parameter name matching, generic function types
  (`T Function<T>(T)`) with their own bound rules. `Function` (bare) is the
  top of function types; `void Function()` accepts any-return functions per
  void rules.
- **Demonstrate:** assignments exercising contravariance/covariance; a
  generic function type as a field type; a typedef'd function type in an API.
- **Variants/failures:** generic function types cannot be type arguments in
  some positions; graphs must treat structurally identical function types as
  one type even across typedef spellings.
- **Defined by / confidence:** language · High.
- **Verify:** Dart Language Specification.

#### FN-04 Tear-off identity and equality
- **Behavior:** referencing a method without calling it produces a closure.
  Top-level/static tear-offs are canonicalized constants (identical);
  instance-method tear-offs from the same object are `==` but not identical;
  generic tear-offs may be explicitly instantiated (`f<int>` as a value,
  2.15) or implicitly instantiated by context.
- **Demonstrate:** equality/identity assertions across tear-off kinds; an
  implicitly instantiated tear-off assigned to `int Function(int)`.
- **Variants/failures:** tear-off of an extension member captures static
  resolution; operators and setters cannot be torn off.
- **Defined by / confidence:** language · High (Medium on canonicalization
  corners across backends).
- **Verify:** dart-lang/language constructor-tearoffs feature specification
  (covers explicit instantiation); Dart Language Specification.

#### FN-05 Dynamic invocation and `call` coercion
- **Behavior:** `Function.apply(f, args, namedArgs)` invokes with runtime
  shapes; calling through `dynamic` defers all signature checks to runtime;
  an object with a `call` method implicitly tears off `call` when assigned to
  a function type.
- **Demonstrate:** a `Function.apply` call with `Symbol` named arguments; a
  dynamic call that fails at runtime with `NoSuchMethodError`; callable-class
  coercion (CLS-10).
- **Variants/failures:** `Function.apply` and minified/AOT builds interact
  with tree shaking (PLAT-05); symbols vs minification (implementation
  detail).
- **Defined by / confidence:** language + core library · High.
- **Verify:** `dart:core` `Function` API documentation.

#### FN-06 External functions and members
- **Behavior:** `external` declarations separate signature from body; bodies
  are supplied by the platform (patch files), FFI (`@Native`), or JS interop.
  A graph sees a declaration with no Dart implementation edge.
- **Demonstrate:** `external` members in FFI and JS-interop contexts
  (PLAT-07/08); note that core libraries use them pervasively.
- **Variants/failures:** an `external` declaration with no binding mechanism
  is a link-time/compile-time error per backend.
- **Defined by / confidence:** language (syntax) + implementation (binding) ·
  High.
- **Verify:** Dart Language Specification.

#### FN-07 `main` entry-point signatures
- **Behavior:** `void main()`, `main(List<String> args)`, and the rarely-used
  two-arg form (args + initial message for isolates); `Future<void> main()
  async` keeps the VM alive until completion of the returned future plus
  pending events.
- **Demonstrate:** an async `main` with args in a bin entry point.
- **Variants/failures:** a library without `main` cannot be an entry point;
  web entry points are wired via script tags/bootstrap rather than the CLI.
- **Defined by / confidence:** language + implementation · High.
- **Verify:** Dart Language Specification.

---

### Category G — Static Types, Inference, Coercions (TYP)

#### TYP-01 Soundness, casts, and the `dynamic` escape hatch
- **Behavior:** the static system is sound: no implicit downcasts *except*
  from `dynamic` (implicit cast inserted, checked at runtime). `as` casts
  and `is`/`is!` tests; failed casts throw `TypeError`.
- **Demonstrate:** a dynamic value flowing into a typed slot (implicit
  check); an explicit `as` on a supertype; an `is` check driving promotion.
- **Variants/failures:** `strict-casts` analyzer mode (TOOL-01) turns the
  dynamic implicit cast into a diagnostic — config changes what is an error.
- **Defined by / confidence:** language · High.
- **Verify:** Dart Language Specification.

#### TYP-02 Inference: locals, members, and cross-unit dependencies
- **Behavior:** `var`/`final` locals infer from initializers; top-level and
  field types infer, potentially depending on other libraries; method return
  types do *not* infer from bodies when an override context exists —
  override inference pulls types from superinterfaces; generic invocations
  infer type arguments from arguments and context.
- **Demonstrate:** inferred top-levels chained across libraries; an override
  whose parameter types are inherited by inference (declared without types);
  generic call inference including downward inference from context type.
- **Variants/failures:** inference dependency *cycles* among top-level
  declarations are compile errors; `strict-inference` mode flags
  under-constrained inference.
- **Defined by / confidence:** language/official implementation (inference is
  spec'd in feature docs) · High.
- **Verify:** dart-lang/language type-inference documentation.

#### TYP-03 Special types: `Object?`, `Object`, `dynamic`, `void`, `Never`, `FutureOr`
- **Behavior:** `Object?` is the top type; `dynamic` is `Object?` with
  dynamic member access; `void` restricts value use; `Never` is the bottom
  type (return type of `throw`-only functions, enables unreachable-aware
  flow); `FutureOr<T>` is a denotable union special-cased by `await` and
  subtyping.
- **Demonstrate:** APIs using each deliberately, including a `Never`-typed
  helper affecting definite-assignment/exhaustiveness and a `FutureOr`
  parameter awaited internally.
- **Variants/failures:** using a `void` expression's value is an error in
  most positions; `dynamic` vs `Object?` produce different call-site edges.
- **Defined by / confidence:** language · High.
- **Verify:** Dart Language Specification.

#### TYP-04 Type promotion and private-final field promotion (Dart 3.2)
- **Behavior:** flow analysis promotes locals on `is`, `!= null`, `== null`
  branches, assignments, and pattern matches. Since 3.2, *private final
  instance fields* promote too — but only when no conflicting declaration in
  the library defeats it (no same-name getter elsewhere, no non-final
  same-name field, no `noSuchMethod`-implemented interface supplying it).
- **Demonstrate:** classic local promotion; field promotion succeeding; a
  library deliberately containing a promotion-defeating conflict to show the
  failure mode.
- **Variants/failures:** promotion is defeated by closure capture and
  reassignment; pre-3.2 language versions never promote fields — same code,
  different errors by version.
- **Defined by / confidence:** language · High.
- **Verify:** dart.dev field-promotion / type-promotion documentation.

#### TYP-05 Definite assignment and `late`
- **Behavior:** flow analysis proves locals assigned before use;
  non-nullable locals may be declared unassigned if all paths assign first.
  `late` locals/fields defer the proof to runtime
  (`LateInitializationError`); `late` initializers run lazily on first read.
- **Demonstrate:** a branch-complete assignment pattern that compiles without
  `late`; a lazy `late` field whose initializer has an observable side
  effect; a `late final` assigned once.
- **Variants/failures:** reading an unassigned `late` throws; second write to
  `late final` throws.
- **Defined by / confidence:** language · High.
- **Verify:** Dart Language Specification (null-safety feature spec).

#### TYP-06 Upper-bound inference in branches and literals
- **Behavior:** conditional expressions, collection literals, and switch
  expressions compute least-upper-bound-style types (UP), which sometimes
  produce surprising supertypes (`Object`, nullable unions) that then shape
  downstream resolution.
- **Demonstrate:** a heterogeneous list literal inferring a common supertype;
  a conditional producing a nullable LUB feeding a member-access error.
- **Variants/failures:** LUB differences are a known analyzer/CFE divergence
  area historically.
- **Defined by / confidence:** language/official implementation · Medium.
- **Verify:** dart-lang/language typing rules documentation.

#### TYP-07 Covariant generics and the `covariant` keyword
- **Behavior:** class type parameters are covariant by use (`List<Cat>` is a
  `List<Animal>`), so writes are runtime-checked — a valid program can throw
  `TypeError` on `animals.add(Dog())`. `covariant` on a parameter opts a
  method into narrowed override parameter types with runtime checks.
- **Demonstrate:** an upcast list write that throws; an override narrowing a
  parameter via `covariant` declared on base or override.
- **Variants/failures:** contravariant positions of type parameters generate
  synthetic runtime checks; declaration-site variance is an unshipped
  language proposal (watch item).
- **Defined by / confidence:** language · High.
- **Verify:** Dart Language Specification.

#### TYP-08 Runtime type objects and type literals
- **Behavior:** `runtimeType` reifies an object's type; type names are
  expressions of type `Type`; generic type literals (`List<int>`) are
  expressions since 2.15; `Type` equality is defined but `identical` on
  `Type` objects and `runtimeType.toString()` formats vary by backend and
  minification.
- **Demonstrate:** type literals as map keys; comparing `runtimeType`
  against a literal (and the lint discouraging it).
- **Variants/failures:** obfuscated/minified builds change `toString`;
  dart2js may unify representations — implementation-defined (see §4).
- **Defined by / confidence:** language + implementation-defined corners ·
  High/Medium.
- **Verify:** `dart:core` `Type` API documentation.

#### TYP-09 Records as structural types (Dart 3.0)
- **Behavior:** `(int, String)` and `({int a, String b})` are structural
  types with no declaration site; positional fields expose synthetic getters
  `$1..$n`, named fields expose named getters; `==`/`hashCode` are
  structural; const records canonicalize; record types participate in
  subtyping field-wise.
- **Demonstrate:** record-returning functions, record type annotations,
  typedefs of record types, singleton positional record syntax `(x,)`,
  nested records, record equality assertions.
- **Variants/failures:** no `is`-based shape narrowing without patterns;
  records cannot implement interfaces; a graph needs a strategy for
  declaration-less types.
- **Defined by / confidence:** language · High.
- **Verify:** dart-lang/language records feature specification.

#### TYP-10 Numeric literal typing (no implicit conversion)
- **Behavior:** Dart has no implicit numeric conversions; instead *integer
  literals* in double contexts are analyzed as doubles (`double d = 1;`
  compiles, `double d = someInt;` does not). `int` and `double` are the only
  `num` subclasses and cannot be extended/implemented by users.
- **Demonstrate:** literal-typing acceptance vs variable-assignment
  rejection.
- **Variants/failures:** platform numeric representation differences are
  PLAT-04.
- **Defined by / confidence:** language · High.
- **Verify:** Dart Language Specification.

---

### Category H — Patterns and Control Flow (PAT)

#### PAT-01 Switch expressions and exhaustiveness
- **Behavior:** `switch (x) { Pattern => expr, ... }` is an expression;
  exhaustiveness is *checked* for always-exhaustive types (enums, `bool`,
  `sealed` hierarchies, records/nullables thereof) — a missing case is a
  compile error, not a warning.
- **Demonstrate:** exhaustive switches over an enum and a sealed hierarchy
  (no default); a deliberately non-exhaustive variant as a diagnostic case;
  `_` restoring exhaustiveness.
- **Variants/failures:** adding a sealed subtype breaks distant switches at
  compile time — a cross-library dependency edge graphs should capture.
- **Defined by / confidence:** language · High.
- **Verify:** dart-lang/language patterns/exhaustiveness feature
  specifications.

#### PAT-02 Pattern taxonomy
- **Behavior:** record patterns, list patterns (with rest `...` and
  `...rest`), map patterns, object patterns (`Point(:var x, y: > 0)` — which
  *call getters*), constant patterns, relational (`> 5`), logical
  (`&&`/`||`), null-check `p?`, null-assert `p!`, cast `p as T`, variable
  and wildcard patterns, and `when` guards.
- **Demonstrate:** each kind at least once across switches and declarations;
  an object pattern whose subpattern getters are the resolution targets.
- **Variants/failures:** constant patterns require const expressions and
  match via the constant's `==`; logical-or branches must bind identical
  variable sets.
- **Defined by / confidence:** language · High.
- **Verify:** dart-lang/language patterns feature specification.

#### PAT-03 Pattern contexts and refutability
- **Behavior:** declaration context (`var (a, b) = pair;` — irrefutable
  patterns only), assignment context (`(a, b) = pair;` assigning existing
  variables), and matching context (switch/if-case — refutable allowed).
  The same syntax means declare-new vs assign-existing depending on context.
- **Demonstrate:** all three contexts; destructuring assignment to
  pre-existing locals; a refutable pattern rejected in a declaration.
- **Variants/failures:** variable-identity questions: pattern variables in
  or-patterns, shared across guard scope.
- **Defined by / confidence:** language · High.
- **Verify:** dart-lang/language patterns feature specification.

#### PAT-04 `if-case` and pattern `for`
- **Behavior:** `if (json case {'name': String name})` matches-and-binds in a
  statement head; `for (var (a, b) in pairs)` destructures per iteration.
- **Demonstrate:** both forms, including an `if-case` with a `when` guard
  and an else branch.
- **Variants/failures:** bindings are scoped to the then-branch (and guard).
- **Defined by / confidence:** language · High.
- **Verify:** dart.dev patterns documentation.

#### PAT-05 Switch statements: 3.0 semantics, labels
- **Behavior:** since 3.0, non-empty cases do not fall through and need no
  `break`; empty cases still fall through to the next; `continue label;`
  jumps between labeled cases; labeled statements target `break`/`continue`.
- **Demonstrate:** grouped empty cases; a labeled case with `continue`; a
  labeled loop break.
- **Variants/failures:** a pre-3.0 language-versioned file retains
  `break`-required semantics — version-sensitive control flow.
- **Defined by / confidence:** language · High.
- **Verify:** dart-lang/language patterns feature specification (switch
  changes); Dart Language Specification.

#### PAT-06 Destructuring as resolution surface
- **Behavior:** object and map patterns generate real member/lookup
  dispatches: object patterns call getters, map patterns call `[]`, list
  patterns call `length` and `[]`. These are graph edges without ordinary
  call syntax.
- **Demonstrate:** a custom class matched by an object pattern whose getters
  have observable side effects (or at least are user-declared, so edges are
  attributable).
- **Variants/failures:** getter-call *order and count* during matching is
  partially unspecified (see §4).
- **Defined by / confidence:** language · High (Medium on invocation-count
  guarantees).
- **Verify:** dart-lang/language patterns feature specification.

---

### Category I — Extension Members (EXT)

#### EXT-01 Static resolution and precedence
- **Behavior:** extension members resolve entirely at compile time against
  the receiver's *static* type; instance members always beat extension
  members; extensions never apply to `dynamic` receivers; the tear-off of an
  extension method closes over the receiver.
- **Demonstrate:** an extension method shadowed by an instance member of the
  same name; the same call succeeding on a typed receiver and becoming a
  runtime `NoSuchMethodError` via `dynamic`.
- **Variants/failures:** extension members on `Object?` apply to nullable
  receivers — deliberate null-receiver dispatch.
- **Defined by / confidence:** language · High.
- **Verify:** dart-lang/language extension-methods feature specification.

#### EXT-02 Applicability conflicts and specificity
- **Behavior:** when several in-scope extensions apply, the most specific
  (by receiver type) wins; a tie is a compile error, resolvable by `hide`
  on an import or explicit application `E(x).m()`.
- **Demonstrate:** two extensions on related types showing specificity; two
  on the same type forcing the ambiguity diagnostic and both resolutions.
- **Variants/failures:** import combinators change which call sites compile
  — resolution depends on directive-level configuration.
- **Defined by / confidence:** language · High.
- **Verify:** dart-lang/language extension-methods feature specification.

#### EXT-03 Unnamed and private extensions; explicit application
- **Behavior:** `extension on String { ... }` is unnamed and library-local
  (cannot be exported or explicitly applied by name); `_`-named extensions
  are private; explicit application `MyExt(x).member` bypasses implicit
  resolution.
- **Demonstrate:** an unnamed extension used within its library; a public
  named extension applied explicitly and via a prefix.
- **Variants/failures:** unnamed extensions still apply implicitly inside
  their library — entities without exportable names.
- **Defined by / confidence:** language · High.
- **Verify:** dart.dev extension-methods documentation.

#### EXT-04 Generic extensions, static members, nullable targets
- **Behavior:** extensions may be generic (`extension ListX<T> on List<T>`),
  declare static members (`ExtName.helper()`), and target nullable or bound
  types (`extension on T?`, `extension NumX<T extends num> on T`).
- **Demonstrate:** a generic extension whose type parameter flows into
  member signatures; a static helper on an extension; an extension on a
  nullable type invoked on `null`.
- **Variants/failures:** static members are accessed via the extension name
  only — an extension is also a namespace entity.
- **Defined by / confidence:** language · High.
- **Verify:** dart-lang/language extension-methods feature specification.

---

### Category J — Inheritance, Overriding, Dispatch (DSP)

#### DSP-01 No overloading; one member per name per kind
- **Behavior:** Dart has no method overloading — a name resolves to exactly
  one member (getter/setter pairs aside). API variety comes from optional
  and named parameters instead. This collapses whole families of resolution
  logic other languages need.
- **Demonstrate:** a class using optional/named parameters where an
  overloaded family would exist elsewhere; a duplicate-name declaration as a
  diagnostic case.
- **Defined by / confidence:** language · High.
- **Verify:** Dart Language Specification.

#### DSP-02 Override rules and `@override`
- **Behavior:** overrides must be signature-compatible: covariant returns,
  contravariant-or-equal parameters (narrowing requires `covariant`), may
  add optional/named parameters, may tighten nullability contravariantly.
  `@override` is only an annotation — its absence changes nothing
  semantically (lint `annotate_overrides` enforces style).
- **Demonstrate:** legal loosening overrides; an added optional parameter;
  a `covariant` narrowing; an incompatible override diagnostic.
- **Variants/failures:** overriding a field with accessors and vice versa;
  overriding `==` obliges `hashCode` by convention.
- **Defined by / confidence:** language (+ lint conventions) · High.
- **Verify:** Dart Language Specification.

#### DSP-03 `noSuchMethod` and synthesized forwarders
- **Behavior:** a concrete `noSuchMethod` override lets a class *implement*
  interfaces without concrete members: the compiler synthesizes forwarder
  stubs for unimplemented interface members, making the class concrete.
  Dynamic calls that miss also route to `noSuchMethod`.
- **Demonstrate:** a class implementing an interface solely via
  `noSuchMethod` (the mechanism beneath older mocking libraries); a dynamic
  miss producing an `Invocation`.
- **Variants/failures:** forwarders are real (synthetic) members a graph
  must decide to represent; interaction with private members of other
  libraries (VIS-02).
- **Defined by / confidence:** language · High.
- **Verify:** Dart Language Specification.

#### DSP-04 `super` resolution and mixin linearization
- **Behavior:** `super.m()` binds statically to the nearest superclass-chain
  implementation — but under mixins the "superclass chain" is the
  *linearized* application order, so a mixin's `super.m()` target depends on
  each application site (`S with M1, M2`: `M2.super` sees `S&M1`).
- **Demonstrate:** two applications of one mixin over different bases
  showing different `super` targets; an `on`-typed super call.
- **Variants/failures:** the intermediate application classes are synthetic
  entities (SYN-02).
- **Defined by / confidence:** language · High.
- **Verify:** Dart Language Specification.

#### DSP-05 Dynamic dispatch vs static resolution inventory
- **Behavior:** instance methods dispatch virtually; static/top-level
  members, constructors, extension members (EXT-01), and extension-type
  members (CLS-05) resolve statically; `dynamic` receivers defer everything
  to runtime. A correct graph distinguishes these four resolution regimes.
- **Demonstrate:** the same conceptual operation expressed through each
  regime.
- **Defined by / confidence:** language · High.
- **Verify:** Dart Language Specification.

#### DSP-06 Method-vs-getter distinction at use sites
- **Behavior:** `obj.m` is a tear-off (value), `obj.m()` an invocation; a
  getter returning a function makes `obj.g()` a getter call *plus* a
  function call — two edges, different members. Graphs must not conflate
  property access with invocation.
- **Demonstrate:** a getter-returning-function alongside a method with the
  same shape; call sites of both.
- **Defined by / confidence:** language · High.
- **Verify:** Dart Language Specification.

#### DSP-07 Operator and assignment lowering
- **Behavior:** compound assignments lower to member calls: `a[i] += 1`
  invokes `[]` then `+` then `[]=`; `x ??= e` reads then conditionally
  writes; `++`/`--` lower to `+ 1`/`- 1`. These implicit invocations are
  resolution edges without visible call syntax.
- **Demonstrate:** a custom indexable class exercised via compound index
  assignment; `??=` on a setter-backed property.
- **Defined by / confidence:** language · High.
- **Verify:** Dart Language Specification.

#### DSP-08 `==`, `identical`, and null handling
- **Behavior:** `a == b` never dispatches when either side is `null`
  (handled by the language); `identical` is a core-library primitive
  bypassing `==`; `hashCode`/`==` contracts drive collection behavior.
- **Demonstrate:** an overridden `==` shown to not receive null; identity
  vs equality assertions on const-canonicalized objects.
- **Variants/failures:** `identical` on numbers differs per platform (§4).
- **Defined by / confidence:** language + core library · High.
- **Verify:** `dart:core` documentation; Dart Language Specification.

---

### Category K — Null Safety and Flow (NUL)

#### NUL-01 Nullable types as distinct types
- **Behavior:** `T?` is a distinct type from `T` throughout signatures,
  generics, and overrides; `Null` is its own type; nullability participates
  in subtyping (`T <: T?`).
- **Demonstrate:** APIs distinguishing `T` vs `T?` in parameters, returns,
  type arguments, and overrides.
- **Defined by / confidence:** language · High.
- **Verify:** dart-lang/language null-safety feature specification.

#### NUL-02 Null-aware operators and short-circuiting
- **Behavior:** `?.`, `?..`, `??`, `??=`, `...?`, and postfix `!`
  (null-assert, throws on null). A null receiver short-circuits the *entire*
  selector chain: in `a?.b.c`, `.c` is skipped when `a` is null.
- **Demonstrate:** chained `?.` with the whole-chain short-circuit; `!`
  both succeeding and throwing; `??=` on nullable locals and properties.
- **Variants/failures:** `!` on a known-non-null operand is a lint; flow
  analysis interacts (promotion after `??` etc.).
- **Defined by / confidence:** language · High.
- **Verify:** dart-lang/language null-safety feature specification.

#### NUL-03 `late` runtime semantics
- **Behavior:** covered structurally in TYP-05/CLS-08; the graph-relevant
  point: `late` moves a static guarantee to a runtime check, changing where
  diagnostics live (compile-time vs `LateInitializationError`).
- **Demonstrate:** one compile-time definite-assignment error variant and
  its `late` runtime-error twin.
- **Defined by / confidence:** language · High.
- **Verify:** dart-lang/language null-safety feature specification.

#### NUL-04 Null-aware collection elements (Dart 3.8)
- **Behavior:** `[a, ?maybeNull, b]` and `{?k: ?v}` include elements only
  when non-null — new element syntax inside collection literals.
- **Demonstrate:** list/set/map literals with `?` elements; a pre-3.8
  language-versioned file where the syntax errors.
- **Defined by / confidence:** language · High (feature), Medium (exact
  scope of map-key/value forms).
- **Verify:** SDK CHANGELOG; dart-lang/language feature specification.

---

### Category L — Constants and Compile-Time Evaluation (CONST)

#### CONST-01 Const expression subset and potentially-const contexts
- **Behavior:** const evaluation covers literals, const constructors,
  arithmetic/logical operators on primitives, `identical`, string
  concatenation/interpolation of constants, const collections and records,
  and (2.15+) constructor/function tear-offs. Initializer lists of const
  constructors are *potentially constant* — evaluated per instantiation.
- **Demonstrate:** const fields, const collections/records, a const
  constructor whose initializer list uses its parameters.
- **Variants/failures:** non-const operations in const contexts are compile
  errors; asserts in const constructors evaluate at compile time for const
  instantiations.
- **Defined by / confidence:** language · High.
- **Verify:** Dart Language Specification.

#### CONST-02 Canonicalization and identity
- **Behavior:** structurally equal const objects are canonicalized to one
  identity (`identical` true) — across libraries too. Caveat: across
  deferred load units canonicalization is not guaranteed (dart2js).
- **Demonstrate:** `identical` across two libraries' const references; const
  list/map/record identity.
- **Variants/failures:** doubles/ints in const contexts on the web (§4).
- **Defined by / confidence:** language + implementation-defined deferred
  corner · High/Medium.
- **Verify:** Dart Language Specification.

#### CONST-03 Compile-time environment (`-D` defines)
- **Behavior:** `const String.fromEnvironment('K')`,
  `int.fromEnvironment`, `bool.fromEnvironment` read compiler `-D` defines —
  the same source produces different constants per build invocation.
  `bool.fromEnvironment('dart.library.io')` mirrors conditional-import
  facts.
- **Demonstrate:** a const configuration flag consumed in code, documented
  with the `dart run -D` / `dart compile -D` invocation that flips it.
- **Variants/failures:** the analyzer does not know your defines — it
  evaluates defaults; compiled behavior diverges from analyzed behavior.
- **Defined by / confidence:** language + tool configuration · High.
- **Verify:** `dart:core` `String.fromEnvironment` API documentation.

#### CONST-04 Const evaluation errors
- **Behavior:** errors during const evaluation (integer division by zero,
  type errors, failed asserts in const constructors, non-const values) are
  compile-time errors — diagnostics produced by evaluation, not parsing.
- **Demonstrate:** one or two representative const-evaluation error cases in
  the invalid-code section of a fixture.
- **Defined by / confidence:** language · High.
- **Verify:** Dart Language Specification.

#### CONST-05 Metadata requires const; generic annotations
- **Behavior:** annotations are const expressions — const constructor
  invocations or const variable references; generic annotation applications
  (`@TypedAnnotation<int>(...)`) are allowed (since ~2.14). Annotation
  arguments are resolution edges into const space.
- **Demonstrate:** custom annotation classes, annotations referencing const
  variables, a generic annotation.
- **Defined by / confidence:** language · High (Medium on the exact version
  for generic annotations).
- **Verify:** Dart Language Specification; SDK CHANGELOG.

---

### Category M — Async, Generators, Isolates (ASY)

#### ASY-01 `async`/`await` semantics
- **Behavior:** an `async` function returns `Future<T>` with `T` inferred
  from `return` statements (`return fut;` awaits implicitly); the body runs
  synchronously until the first suspension point; `await` unwraps
  `Future`/`FutureOr`. Since Dart 3, `await` of a non-future is allowed
  but linted contextually.
- **Demonstrate:** async functions returning values and futures; awaiting
  in loops and try/finally; `Future<void>`.
- **Variants/failures:** `await` outside async contexts is a compile error;
  return-type mismatches (`Future<int>` vs `int`) produce dedicated
  diagnostics.
- **Defined by / confidence:** language · High.
- **Verify:** Dart Language Specification; `dart:async` documentation.

#### ASY-02 Generators: `sync*` and `async*`
- **Behavior:** `sync*` bodies produce lazy `Iterable`s; `async*` produce
  `Stream`s; `yield` emits, `yield*` delegates to a sub-iterable/stream —
  delegation edges between generator bodies.
- **Demonstrate:** both generator kinds, each with a `yield*` delegation;
  laziness observed via side effects.
- **Variants/failures:** `return v;` with a value is illegal in generators;
  cancellation/pausing semantics on `async*` are subtle.
- **Defined by / confidence:** language · High.
- **Verify:** Dart Language Specification.

#### ASY-03 Stream consumption
- **Behavior:** `await for` over streams (in async bodies), single vs
  broadcast streams, `StreamController`, stream transformation chains.
- **Demonstrate:** an `await for` loop; a broadcast controller with two
  listeners; an error event handled by a listener.
- **Variants/failures:** listening twice to a single-subscription stream
  throws at runtime.
- **Defined by / confidence:** core library (`dart:async`) · High.
- **Verify:** `dart:async` API documentation.

#### ASY-04 Unawaited futures and async error flow
- **Behavior:** a future nobody awaits reports errors to the zone (possibly
  crashing); `unawaited()` documents intent; lints `unawaited_futures` /
  `discarded_futures` make this analysis-visible. Errors thrown before the
  first `await` vs after differ in how they surface (sync throw vs failed
  future) for non-async callees.
- **Demonstrate:** an intentionally fire-and-forget call wrapped in
  `unawaited`; a caught async error via `try`/`catch` around `await`.
- **Defined by / confidence:** core library + linter · High.
- **Verify:** `dart:async` documentation; linter rule docs.

#### ASY-05 Isolates
- **Behavior:** `Isolate.spawn`, `Isolate.run` (2.19+), and port-based
  message passing; no shared mutable state; sent objects are copied (some
  are unsendable — open ports, native resources), immutable/const objects
  may be shared within an isolate group. Unavailable on the web (compile-
  or runtime-limited), where workers are a different mechanism.
- **Demonstrate:** an `Isolate.run` computation; a spawn with a top-level
  entry function and `SendPort` round trip; conditional exclusion on web.
- **Variants/failures:** closures capturing unsendable state fail at send
  time; the entry point of `spawn` must be a top-level/static function —
  reachability anchor for tree shaking.
- **Defined by / confidence:** core library + platform constraints · High.
- **Verify:** `dart:isolate` API documentation.

#### ASY-06 Zones
- **Behavior:** `runZoned`/`runZonedGuarded` establish dynamic-extent
  contexts intercepting uncaught errors, `print`, timers, and scheduling;
  zone-local values act as implicit parameters invisible to static
  structure.
- **Demonstrate:** a guarded zone catching an uncaught async error; a
  zone-local value read from deep in a call chain.
- **Variants/failures:** zone behavior is dynamic — a known blind spot for
  static graphs worth representing deliberately.
- **Defined by / confidence:** core library · High.
- **Verify:** `dart:async` `Zone` API documentation.

---

### Category N — Metadata and Annotations (META)

#### META-01 Annotation mechanics and targets
- **Behavior:** `@expr` metadata attaches to libraries, directives, type
  declarations, members, parameters, type parameters, and variables; the
  expression must be const (CONST-05) and is a resolvable reference —
  annotations are first-class graph edges.
- **Demonstrate:** annotations on every target kind, including on an import
  directive and a parameter.
- **Defined by / confidence:** language · High.
- **Verify:** Dart Language Specification.

#### META-02 Behavior-affecting pragmas
- **Behavior:** `@pragma('vm:entry-point')` keeps entities alive under AOT
  tree shaking (enabling reflective/native access); other pragmas
  (`vm:prefer-inline`, `dart2js:noInline`, etc.) steer compilation. These
  change *what exists* in compiled output — metadata with semantics.
- **Demonstrate:** an entry-point pragma on a function reached only
  dynamically.
- **Variants/failures:** pragma strings are unchecked — typos silently do
  nothing; behavior is per-backend.
- **Defined by / confidence:** official implementation (not the language) ·
  High (existence), Medium (full catalog).
- **Verify:** Dart SDK source/documentation on pragma annotations.

#### META-03 `package:meta` contract annotations
- **Behavior:** `@immutable`, `@mustCallSuper`, `@mustBeOverridden`,
  `@useResult`, `@doNotStore`, `@protected`, `@sealed` (pre-3.0 idiom),
  `@reopen`, `@redeclare`, plus visibility ones (VIS-05) — analyzer-only
  contracts that create diagnosable obligations across libraries.
- **Demonstrate:** a handful with both compliant and violating usage sites.
- **Variants/failures:** compilers ignore them entirely; only analysis
  changes.
- **Defined by / confidence:** ecosystem (`package:meta`) + analyzer · High.
- **Verify:** `package:meta` API documentation.

#### META-04 Codegen-trigger annotations
- **Behavior:** annotations like `@JsonSerializable()` (json_serializable),
  `@freezed` (freezed), `@GenerateMocks` (mockito), `@riverpod` drive
  builders to emit sources; the annotation is the causal link between
  handwritten and generated code.
- **Demonstrate:** at least one part-emitting and one library-emitting
  generator in a codegen-enabled fixture (CGEN-01/02).
- **Defined by / confidence:** ecosystem · High.
- **Verify:** respective package documentation on pub.dev.

#### META-05 Interop annotations
- **Behavior:** `@JS(...)` (dart:js_interop) on extension types/members maps
  Dart names to JS names; `@Native<...>(...)` (dart:ffi) binds external
  functions/fields to native symbols; legacy `@staticInterop`/`@anonymous`
  appear in older code.
- **Demonstrate:** within the interop items PLAT-07/08.
- **Defined by / confidence:** official implementation libraries · High.
- **Verify:** `dart:js_interop` and `dart:ffi` API documentation.

---

### Category O — Code Generation and Build Pipelines (CGEN)

#### CGEN-01 Part-based code generation
- **Behavior:** the dominant codegen idiom: a handwritten library declares
  `part 'model.g.dart';` and references not-yet-generated symbols
  (`_$ModelFromJson`); `dart run build_runner build` emits the part, which —
  being a part — can access the library's *private* members. Until codegen
  runs, the project contains unresolved references and a missing-part error.
- **Demonstrate:** a json_serializable-style model with its committed
  `.g.dart`; the pre-generation broken state is itself a scenario worth
  documenting.
- **Variants/failures:** stale generated files diverging from source; a graph
  built without running codegen sees a different (broken) program than one
  built after.
- **Defined by / confidence:** ecosystem (build_runner/source_gen), on
  language `part` semantics · High.
- **Verify:** `build_runner` and `json_serializable` package documentation.

#### CGEN-02 Library-emitting code generation
- **Behavior:** some generators emit standalone libraries instead of parts —
  e.g., mockito's `@GenerateMocks` produces a `.mocks.dart` library that
  *imports* the annotated file; freezed emits `.freezed.dart` parts —
  fixtures should include both topologies.
- **Demonstrate:** one generated importable library and one generated part,
  with the direction of the import/part edges differing.
- **Defined by / confidence:** ecosystem · High.
- **Verify:** `mockito`, `freezed` package documentation.

#### CGEN-03 `build.yaml` configuration
- **Behavior:** `build.yaml` scopes builders (`generate_for` globs), sets
  builder options, and defines custom builders — build-time configuration
  that changes which generated sources exist and with what content.
- **Demonstrate:** a `build.yaml` narrowing generation to a subdirectory or
  toggling a builder option with observable output difference.
- **Defined by / confidence:** ecosystem (`build` package) · High.
- **Verify:** `build_config`/`build_runner` documentation.

#### CGEN-04 Generated-source conventions
- **Behavior:** generated files carry `// GENERATED CODE - DO NOT MODIFY`
  headers, blanket `// ignore_for_file:` directives, and `.g.dart` /
  `.freezed.dart` / `.mocks.dart` naming; ecosystems split on committing
  generated files vs regenerating in CI. Both states must be analyzable.
- **Demonstrate:** headers/ignores in committed generated files; a note on
  regeneration in the build workflow.
- **Defined by / confidence:** ecosystem convention · High.
- **Verify:** `source_gen` documentation.

#### CGEN-05 Macros: cancelled; augmentations uncertain
- **Behavior:** the static-metaprogramming "macros" feature (previewed
  around Dart 3.4, e.g. `JsonCodable`) was **cancelled** — announced by the
  Dart team in early 2025 — with investment redirected to build_runner
  performance and related language work (enhanced parts/augmentations,
  LIB-12). No stable Dart has macros.
- **Demonstrate:** nothing to demonstrate; a checklist guard against
  designing fixtures around macros.
- **Defined by / confidence:** official project decision · High
  (cancellation), Low (successor-feature timeline).
- **Verify:** Dart team announcement (dart.dev blog/Medium); SDK CHANGELOG.

#### CGEN-06 External generators
- **Behavior:** non-build_runner generators are common: `protoc` Dart plugin
  (protobufs), `ffigen` (C headers → `dart:ffi` bindings), `jnigen` (Java),
  `pigeon` (Flutter platform channels), `intl`/ARB message generation. Each
  produces ordinary Dart libraries whose provenance is a config file, not an
  annotation.
- **Demonstrate:** at least one config-driven generated library (e.g., an
  ffigen YAML plus its output) if interop is in scope.
- **Defined by / confidence:** ecosystem/official-adjacent tools · Medium.
- **Verify:** respective package documentation on pub.dev.

---

### Category P — Platforms, Compilation, Configuration (PLAT)

#### PLAT-01 Compile-target matrix
- **Behavior:** one source, many pipelines: VM JIT (`dart run`), native AOT
  (`dart compile exe`/`aot-snapshot`), `dart compile js` (dart2js), DDC (dev
  web), `dart compile wasm` (dart2wasm, 3.4+). Available libraries, numeric
  semantics, deferred-loading reality, reflection, and tree shaking all vary
  by target.
- **Demonstrate:** build documentation/tasks compiling the same package for
  ≥2 targets; target-conditional code paths (LIB-07).
- **Defined by / confidence:** official implementation · High.
- **Verify:** `dart compile` documentation on dart.dev.

#### PLAT-02 Web library migration
- **Behavior:** `dart:html`, `dart:js`, `dart:js_util` are deprecated in
  favor of `package:web` + `dart:js_interop` (3.3+); dart2wasm *only*
  supports the new stack. Codebases exist in both states — a
  version/ecosystem fork in what web code imports.
- **Demonstrate:** web-conditional code using `package:web`-style interop;
  legacy imports only as a diagnostics/migration scenario.
- **Defined by / confidence:** official implementation + ecosystem migration
  · High.
- **Verify:** dart.dev web interop migration documentation.

#### PLAT-03 Numeric semantics per target
- **Behavior:** VM/AOT/wasm: `int` is 64-bit two's complement (wrapping);
  JS targets: `int` is a JS number (53-bit safe integers, no wrap,
  `identical(1, 1.0)` true, `1 is double` true, `double.toString` differs
  e.g. `1` vs `1.0`). Same expressions, different results — the flagship
  implementation-variant behavior.
- **Demonstrate:** documented expected-output differences for a small set of
  numeric expressions per target.
- **Defined by / confidence:** official implementation (documented
  divergence) · High.
- **Verify:** dart.dev numbers documentation (platform differences).

#### PLAT-04 Tree shaking and reachability
- **Behavior:** AOT and dart2js drop unreferenced declarations; only
  statically reachable code exists in output. Dynamic access needs
  `@pragma('vm:entry-point')` (META-02) or explicit references. A code
  graph's reachability story per entry point mirrors this.
- **Demonstrate:** a deliberately unreachable declaration; an entry-point
  pragma; multiple entry points with different reachable sets (PKG-03).
- **Defined by / confidence:** official implementation · High.
- **Verify:** Dart SDK documentation on AOT/dart2js.

#### PLAT-05 `dart:mirrors` availability
- **Behavior:** reflection via `dart:mirrors` works only on the VM JIT; it
  is unavailable under AOT, dart2js, and wasm. Older reflective frameworks
  are therefore platform-locked; modern code uses codegen instead (CGEN).
- **Demonstrate:** at most a JIT-only sample isolated from other targets;
  reasonable to exclude entirely with a note.
- **Defined by / confidence:** official implementation · High.
- **Verify:** `dart:mirrors` API documentation.

#### PLAT-06 FFI (`dart:ffi`)
- **Behavior:** `Pointer`, `Struct`/`Union` subclasses (final classes with
  `@Int32()`-style field annotations and *external* fields),
  `NativeFunction` types, `DynamicLibrary.open`/`lookupFunction`, `@Native`
  external declarations (modern style), `NativeFinalizer`, `Finalizable`.
  Struct classes are declaration-only shells whose layout metadata *is* the
  semantics; not available on web.
- **Demonstrate:** a struct declaration, an `@Native` external function, a
  lookup-based binding, and the C source or a documented stub.
- **Variants/failures:** ABI-dependent types (`Long`, `Size`); leaf calls;
  `ffigen`-generated bindings (CGEN-06); build/link steps outside pub.
- **Defined by / confidence:** official implementation library · High.
- **Verify:** `dart:ffi` API documentation; dart.dev C-interop guide.

#### PLAT-07 JS interop (`dart:js_interop`)
- **Behavior:** modern interop is extension types over `JSObject`
  (`JSString`, `JSArray<T>`, user `@JS` extension types) with `external`
  members, `toJS`/`toDart` conversions, `isA<T>` checks; `@JS('name')`
  renames. Static-only, erased wrappers — the same identity questions as
  CLS-05 plus cross-language name mapping.
- **Demonstrate:** an interop extension type with external members and a
  renaming `@JS` annotation, compiled for a web target.
- **Variants/failures:** `dart:js_interop` is web/wasm-only; legacy
  `package:js` styles still circulate (PLAT-02).
- **Defined by / confidence:** official implementation library · High.
- **Verify:** dart.dev JS interop documentation.

#### PLAT-08 Assertions are configuration-dependent
- **Behavior:** `assert(cond, msg)` executes only when assertions are
  enabled (typical in JIT/dev; disabled in AOT/production unless
  `--enable-asserts`). The same program has different runtime semantics per
  invocation flags — including side effects inside assert expressions.
- **Demonstrate:** an assert with an observable message; documented behavior
  under both configurations.
- **Defined by / confidence:** language (assert semantics) + tool defaults
  (when enabled) · High (Medium on exact per-tool defaults).
- **Verify:** dart.dev assert documentation; `dart` CLI documentation.

#### PLAT-09 Native assets (experimental)
- **Behavior:** an experimental `hook/build.dart` mechanism lets packages
  build/bundle native libraries that `@Native` bindings resolve against,
  replacing manual `DynamicLibrary.open`. Behind a flag around the 3.x
  baseline; stabilization timeline uncertain.
- **Demonstrate:** only as an experimental variant; not a stable-fixture
  candidate at baseline.
- **Defined by / confidence:** official implementation, experimental · Low
  (status), Medium (mechanism).
- **Verify:** dart.dev native-assets documentation; SDK CHANGELOG.

#### PLAT-10 Pubspec platform declarations
- **Behavior:** the pubspec `platforms:` key (and Flutter plugin platform
  maps) declares supported platforms; pub.dev derives/validates them.
  Metadata, not enforcement — but it is the ecosystem's statement of the
  target matrix a graph should reconcile with conditional imports.
- **Demonstrate:** an explicit `platforms:` section consistent with the
  code's `dart:` usage.
- **Defined by / confidence:** pub/ecosystem · Medium.
- **Verify:** dart.dev pubspec documentation.

---

### Category Q — Tooling, Analysis, Formatting, Docs, Tests (TOOL)

#### TOOL-01 `analysis_options.yaml`
- **Behavior:** configures the analyzer: `include:` chains
  (`package:lints/core.yaml` → `recommended.yaml`, `flutter_lints`),
  `linter: rules:`, `analyzer: errors:` severity remapping, `exclude:`
  globs (which *remove files from analysis* — a source-inclusion decision),
  `language:` strict modes (`strict-casts`, `strict-inference`,
  `strict-raw-types`), and experiment enablement.
- **Demonstrate:** an options file exercising include-chaining, one severity
  remap, an exclusion glob covering a real file, and one strict mode.
- **Variants/failures:** excluded files still compile — analysis scope and
  compilation scope diverge.
- **Defined by / confidence:** official tool configuration · High.
- **Verify:** dart.dev analyzer customization documentation.

#### TOOL-02 Suppression comments
- **Behavior:** `// ignore: rule_name` (next line) and
  `// ignore_for_file: rule_name` toggle diagnostics locally — pervasive in
  generated code (CGEN-04) and intentional-violation sites.
- **Demonstrate:** both forms suppressing a named lint and a named
  warning.
- **Defined by / confidence:** official analyzer convention · High.
- **Verify:** dart.dev analyzer documentation.

#### TOOL-03 Diagnostic taxonomy and front-end divergence
- **Behavior:** diagnostics split into compile errors (language-mandated),
  warnings, and lints (opt-in style rules); severity is partially
  configurable (TOOL-01). The analyzer and CFE occasionally disagree
  (accept/reject or message identity) — two official front ends, one
  language.
- **Demonstrate:** representative diagnostics at each severity in an
  invalid-code area; document which tool reports what.
- **Defined by / confidence:** official implementation · High (taxonomy),
  Medium (specific divergences).
- **Verify:** dart.dev diagnostic-messages reference; linter rule index.

#### TOOL-04 Formatter is language-version-aware (Dart 3.7)
- **Behavior:** `dart format` switched to a new "tall" style in 3.7 —
  applied only to code at language version ≥3.7; older-versioned files keep
  the previous style. Formatting output depends on resolved language
  version; trailing commas are now managed by the formatter.
- **Demonstrate:** formatted sources at both language versions in one
  project (pairs with PKG-07).
- **Defined by / confidence:** official tool · High.
- **Verify:** `dart_style` package documentation; SDK CHANGELOG.

#### TOOL-05 `dart fix` and data-driven fixes
- **Behavior:** `dart fix` applies automated remediations for lints and
  deprecations; packages can ship data-driven fix rules (`fix_data.yaml`)
  describing API migrations (rename member, update parameters) that the
  analyzer executes for consumers.
- **Demonstrate:** a deprecated API with an attached data-driven fix in a
  library package.
- **Defined by / confidence:** official tool + package-shipped metadata ·
  Medium.
- **Verify:** dart.dev `dart fix` documentation.

#### TOOL-06 Documentation tooling
- **Behavior:** `dart doc` renders `///` doc comments; `[references]`
  resolve (VIS-07); `@nodoc` conventionally hides members; doc structure
  (categories, templates) via dartdoc directives in comments.
- **Demonstrate:** doc comments across public API with resolving references
  and one `@nodoc`.
- **Defined by / confidence:** official tool + convention · High.
- **Verify:** dartdoc documentation.

#### TOOL-07 Test conventions (`package:test`)
- **Behavior:** tests live in `test/**_test.dart`; `dart test` discovers
  them; `dart_test.yaml` configures tags/platforms/timeouts; per-file
  annotations `@TestOn('vm')`, `@Tags(...)`, `@Skip()`, `@Timeout(...)`
  gate execution per platform — file-level metadata controlling which
  platforms compile/run a file.
- **Demonstrate:** several test files including one platform-restricted via
  `@TestOn` and a `dart_test.yaml`.
- **Variants/failures:** tests import the package under test by `package:`
  URI (LIB-10); `test` compiles files per platform (vm/chrome/node) —
  another configuration axis.
- **Defined by / confidence:** ecosystem-official (`package:test` is
  dart-lang) · High.
- **Verify:** `package:test` documentation.

#### TOOL-08 Experiment flags
- **Behavior:** unshipped features hide behind `--enable-experiment=name`
  (CLI) and analyzer `enable-experiment:` config; source using an
  experiment fails to parse/analyze without the flag — validity is
  configuration-dependent.
- **Demonstrate:** at most a clearly-isolated experimental sample with its
  required flags documented; reasonable to exclude from a stable fixture.
- **Defined by / confidence:** official implementation · High.
- **Verify:** dart.dev experiment-flags documentation.

---

### Category R — Invalid, Ambiguous, and Version-Sensitive Programs (ERR)

#### ERR-01 Broken references and error recovery
- **Behavior:** unresolved import URIs, missing part files, `part of`
  mismatches, and references to undefined names are compile errors — but
  the analyzer still produces a (partial) model. A graph generator's
  behavior on broken projects (crash, skip, partial graph) is itself
  observable behavior.
- **Demonstrate:** a quarantined invalid area with a missing-import file, a
  dangling part, and an undefined-identifier use.
- **Defined by / confidence:** language (errors) + tool behavior (recovery)
  · High.
- **Verify:** dart.dev diagnostic-messages reference.

#### ERR-02 Declaration conflicts and ambiguity diagnostics
- **Behavior:** duplicate top-level names, static/instance name clashes,
  conflicting exports (LIB-05), ambiguous unprefixed uses (VIS-04), and
  ambiguous extensions (EXT-02) each have distinct diagnostics and distinct
  resolution outcomes.
- **Demonstrate:** one minimal case per conflict class in the invalid area.
- **Defined by / confidence:** language · High.
- **Verify:** Dart Language Specification; diagnostic-messages reference.

#### ERR-03 Capability violations (class modifiers)
- **Behavior:** extending a `final` class, implementing a `base` class, or
  subtyping a `sealed` class outside its library are compile errors whose
  *trigger is cross-library structure* — the same declaration pair is legal
  in one file and illegal split across packages.
- **Demonstrate:** violation cases across library and package boundaries
  (pairs with CLS-02).
- **Defined by / confidence:** language · High.
- **Verify:** dart.dev class-modifiers documentation.

#### ERR-04 Static-error vs runtime-error boundary
- **Behavior:** a correct graph should not conflate programs that fail to
  compile with valid programs that throw: covariant write failures
  (TYP-07), failed `as` casts, `late` violations (NUL-03), dynamic-call
  misses (FN-05), single-subscription stream reuse (ASY-03) are *valid*
  programs.
- **Demonstrate:** a runtime-failures suite that compiles cleanly.
- **Defined by / confidence:** language · High.
- **Verify:** Dart Language Specification.

#### ERR-05 Language-version-sensitive validity
- **Behavior:** identical source is valid or invalid depending on resolved
  language version: records/patterns pre-3.0, `>>>` pre-2.14, super
  parameters pre-2.17, `_` binding vs non-binding at 3.7 (VIS-06),
  null-aware elements pre-3.8. Version comes from pubspec or `// @dart=`
  (PKG-07).
- **Demonstrate:** version-pinned files exhibiting flipped validity for two
  or three features.
- **Defined by / confidence:** language + versioning mechanism · High.
- **Verify:** dart.dev language-versioning documentation; SDK CHANGELOG.

#### ERR-06 Cross-unit inference and const cycles
- **Behavior:** top-level inference cycles (TYP-02), const dependency
  cycles, typedef cycles, and inheritance cycles are errors that only
  manifest from *relationships among declarations*, sometimes across
  libraries — no single declaration is wrong.
- **Demonstrate:** one minimal instance of an inference or const cycle in
  the invalid area.
- **Defined by / confidence:** language/official implementation · Medium
  (exact cycle rules) / High (existence).
- **Verify:** dart-lang/language inference documentation.

---

### Category S — Synthetic and Implicit Entities (SYN)

#### SYN-01 Implicit constructors and supertypes
- **Behavior:** entities exist that no source declares: default
  constructors (CTOR-07), the implicit `extends Object`, `Enum` as enum
  supertype, implicit interfaces (CLS-01). Graphs must decide whether these
  appear as nodes/edges.
- **Demonstrate:** classes relying on each synthesis, referenced from
  elsewhere (e.g., a default constructor invoked cross-library).
- **Defined by / confidence:** language · High.
- **Verify:** Dart Language Specification.

#### SYN-02 Mixin-application intermediate classes
- **Behavior:** `class C extends S with M1, M2` introduces anonymous
  intermediate superclasses (conceptually `S&M1`, `S&M1&M2`) that appear in
  stack traces, `runtimeType` chains, and super-resolution (DSP-04); named
  applications (`class C = S with M;`) make one of them a declared entity.
- **Demonstrate:** a mixin-heavy hierarchy plus one named application;
  observation via `super` behavior rather than string forms.
- **Variants/failures:** the *names* of anonymous applications are
  implementation detail (§4).
- **Defined by / confidence:** language (existence) / implementation
  (naming) · High.
- **Verify:** Dart Language Specification.

#### SYN-03 Enum-synthesized members
- **Behavior:** `values`, `index`, `name` (via `dart:core`'s `EnumName`
  extension), default `toString`, and the const value instances themselves
  are synthesized from an enum declaration — several members per
  declaration with no member syntax.
- **Demonstrate:** uses of each synthesized member as resolution targets.
- **Defined by / confidence:** language + core library · High.
- **Verify:** Dart Language Specification; `dart:core` `Enum` docs.

#### SYN-04 Accessor and field synthesis
- **Behavior:** fields synthesize getters/setters (CLS-08); record shapes
  synthesize `$1..$n`/named getters (TYP-09); extension types synthesize
  the representation getter (CLS-05); `noSuchMethod` synthesizes forwarders
  (DSP-03). Use sites reference the synthetic accessor, not the storage.
- **Demonstrate:** overriding a field with accessors and calling through
  the interface; record getter uses.
- **Defined by / confidence:** language · High.
- **Verify:** Dart Language Specification.

#### SYN-05 Anonymous entities: closures and their identities
- **Behavior:** anonymous functions, unnamed extensions (EXT-03), and
  anonymous mixin applications have no source name; tools invent display
  names (`<anonymous closure>`, `main.<fn>`). Any naming a graph uses for
  them is convention, not language.
- **Demonstrate:** nested closures passed as arguments; the fixture merely
  needs them present.
- **Defined by / confidence:** language (existence) / implementation
  (naming, unspecified) · High.
- **Verify:** Dart Language Specification.

---

## 3. Features with Material Recent Change

| Version | Change |
| --- | --- |
| 2.12 (2021) | Sound null safety; FFI stable. |
| 2.13 | Generalized type aliases (non-function typedefs). |
| 2.14 | `>>>` operator; generic annotation applications (approx.). |
| 2.15 | Constructor tear-offs; explicit generic instantiation; generic type literals. |
| 2.17 | Enhanced enums; super parameters; named args anywhere. |
| 2.19 | Unnamed `library;`; `Isolate.run`. |
| 3.0 (May 2023) | Records; patterns; switch expressions and exhaustiveness; class modifiers; sealed types; switch statements no longer need `break`; language versions <2.12 rejected; plain classes no longer mixable. |
| 3.2 | Private final field promotion. |
| 3.3 | Extension types; `dart:js_interop` stable; `dart:html` deprecation direction set. |
| 3.4 | `dart compile wasm` (dart2wasm) usable; macros preview (later cancelled). |
| 3.6 | Digit separators (`1_000_000`); pub workspaces. |
| 3.7 | Wildcard variables (`_` non-binding); formatter "tall" style (language-version-gated). |
| 3.8 | Null-aware collection elements. |
| Early 2025 | Macros feature cancelled (announcement, not a release). |
| 3.9+ (uncertain) | "Dot shorthands" (`.enumValue` context-inferred static access) in preview/possible stabilization; native assets progressing. Low confidence on exact versions. |

## 4. Implementation-Defined or Unspecified Behavior

- **Numeric representation per target** (PLAT-03): int width, wrapping,
  `identical` on numbers, `double.toString` formats.
- **`Type` object identity and `runtimeType.toString()`** across backends,
  minification, and obfuscation (TYP-08).
- **Const canonicalization across deferred load units** (CONST-02, LIB-06).
- **Names of synthetic entities**: anonymous mixin applications, closures,
  forwarders (SYN-02/05); stack-trace formats generally.
- **`hashCode` values** (stable only within a run; `Object.hash` unspecified
  beyond contract).
- **Pattern-matching invocation counts/order** of getters during matching in
  some positions (PAT-06).
- **Tree-shaking outcomes** — which entities exist in output is a compiler
  decision within documented constraints (PLAT-04).
- **Analyzer vs CFE divergences** in recovery, LUB corners, and a small set
  of accept/reject differences (TOOL-03, TYP-06).
- **Event-loop interleaving details** and zone-mediated scheduling beyond
  the documented microtask/event-queue contract (ASY-06).
- **Environment-define visibility to the analyzer** — analysis evaluates
  `fromEnvironment` with defaults, compiled code with `-D` values
  (CONST-03).

## 5. Areas Where My Knowledge May Be Incomplete

- Exact shipping status and semantics of **enhanced parts / augmentations**
  (LIB-12) at and after the 3.8 baseline.
- **Dot shorthands**: which release previews/stabilizes it, and its final
  resolution rules (it adds a new context-dependent static-member
  resolution form — highly graph-relevant when it lands).
- **Native assets** stabilization timeline and final hook API (PLAT-09).
- Precise version numbers for a few mid-2.x changes (generic annotations;
  relaxation of getter/setter type correspondence).
- Fine-grained **pub workspace** semantics (lockfile edge cases, override
  interactions) beyond the core model (PKG-06).
- Whether current CFE/analyzer versions fully canonicalize relative-vs-
  `package:` dual spellings of `lib/` URIs (LIB-10).
- The complete catalog of recognized `@pragma` strings per backend
  (META-02).
- dart2wasm-specific restrictions beyond JS-interop requirements (PLAT-01).
- Details of `dart_test.yaml`'s full schema and platform matrix (TOOL-07).

## 6. Final Audit — Families Possibly Missing or Deliberately Out of Scope

Reviewed against the checklist; candidates a downstream design may still
want to weigh:

- **Flutter-specific surface**: widget classes, `BuildContext` conventions,
  asset bundling (`flutter:` pubspec section), platform channels, plugin
  federation, `flutter_gen`. Excluded here because a Flutter fixture is not
  self-contained (requires the Flutter SDK), but it is the ecosystem's
  largest Dart population — a conscious scoping decision.
- **Internationalization codegen** (`intl`, ARB files → generated message
  libraries): covered only generically under CGEN-06.
- **Serialization ecosystems beyond json_serializable** (built_value,
  protobuf, drift ORM): same codegen topologies as CGEN-01/02; distinct
  only in scale.
- **Dependency-injection frameworks** (injectable/get_it, riverpod codegen):
  annotation-driven indirection where call edges route through generated
  registries — an aggressive resolution test if desired.
- **Analyzer plugin / custom_lint ecosystem**: third-party diagnostics;
  affects diagnostics surface, not program semantics.
- **Obfuscation and source maps** (`--obfuscate`, dart2js maps): affect
  symbol observability in artifacts, not source semantics.
- **Hot reload semantics** (VM/DDC): runtime tooling behavior, out of
  static-graph scope.
- **Embedding APIs** (`dart_api.h`, custom embedders): beyond ordinary
  project scope.
- **Legacy pre-null-safety corpus**: unconsumable by Dart 3 toolchains;
  only representable via the language-version mechanism down to 2.12
  (PKG-07).
- **String/collection standard-library breadth**: deliberately excluded per
  the brief (no API enumeration); only members with semantic machinery
  (`Enum`, `Function.apply`, `identical`, `fromEnvironment`) appear.
- **Comments/whitespace/trivia fidelity** (beyond doc comments and
  suppression comments): included only where semantically load-bearing
  (`// @dart=`, `// ignore:`).

No further category families surfaced on this pass that change semantic
entities, identities, relationships, resolution, types, visibility,
dispatch, source inclusion, or diagnostics beyond those cataloged above.
