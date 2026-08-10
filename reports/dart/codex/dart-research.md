# Dart semantic and project-feature research checklist

## Scope and baseline assumptions

This is a candidate checklist for a broad, idiomatic, self-contained Dart project. It focuses on features that can change declaration identity, name or type resolution, visibility, dispatch, source inclusion, build output, runtime behavior, or diagnostics. It does not prescribe a project layout, graph schema, analyzer, or scoring model.

- **Baseline language and SDK:** Dart 3.8, with sound null safety, unless a checklist item explicitly selects another language version or experiment. Dart 3.8 is chosen as a concrete recent Dart 3 baseline from model knowledge; the exact currently supported stable SDK must be verified.
- **Primary native implementation:** the Dart VM in both just-in-time (JIT) development and ahead-of-time (AOT) compilation modes.
- **Other official targets:** JavaScript through the official web compilers, and WebAssembly where supported. Target differences are part of the checklist rather than assumed away.
- **Package resolution:** Pub with a committed `pubspec.yaml`, a generated package configuration, and an explicit SDK constraint. Lock-file treatment differs for applications and reusable packages.
- **Analysis:** the SDK analyzer using `analysis_options.yaml`. Analyzer diagnostics and lints are distinguished from language compile-time errors.
- **Framework scope:** Flutter-specific widget behavior is not part of the Dart language baseline. Flutter, build systems, generators, and interop belong only where they materially change Dart source inclusion or semantics.
- **Authority labels:** **Language** means specified by the Dart language. **Official implementation/toolchain** means behavior of the Dart SDK, analyzer, compilers, Pub, or official platform libraries. **Ecosystem convention** means a widely used package or layout practice that is not part of the language.

## 1. Libraries, source units, and names

### DART-LIB-001 — Library identity and multi-file `part` composition

- **Distinct behavior:** A Dart library is the privacy and declaration namespace boundary. A library may be one compilation unit or combine a root with `part` files; declarations in every part share private names and one library identity.
- **Observable content:** A root file with a library directive or implicit library, two `part` directives, matching `part of` forms, cross-part references, and duplicate/private declarations.
- **Variants and failures:** Exercise URI-based and named `part of` where accepted; a missing part, mismatched owner, a part imported directly, duplicate parts, directives in the wrong order, and two roots claiming one part. Contrast separate imported libraries containing the same `_private` spelling.
- **Constraints / authority:** Parts are statically included and are not packages or modules in their own right. Exact restrictions on named libraries and `part of` syntax are language-version-sensitive. **Language.**
- **Confidence / likely source:** High. Dart language specification; Dart language documentation for libraries and imports.

### DART-LIB-002 — Imports, prefixes, combinators, and namespace collisions

- **Distinct behavior:** Imports add selected names to a library scope. Prefixes create a separate lookup namespace; `show` and `hide` change which declarations participate in resolution.
- **Observable content:** Multiple libraries exporting identical public names, unprefixed and prefixed imports, chained combinators, and references whose meaning changes with `show` or `hide`.
- **Variants and failures:** Ambiguous unprefixed imports, hidden names, duplicate prefixes, a local declaration shadowing an import, prefix use in type and expression positions, and imports that expose extension methods. Importing the same library through different URI spellings should test canonical identity and diagnostics.
- **Constraints / authority:** Private names are never imported. URI canonicalization and duplicate-URI warnings may involve the toolchain as well as language rules. **Language plus official toolchain.**
- **Confidence / likely source:** High. Dart language specification; libraries and imports documentation; analyzer diagnostic catalog.

### DART-LIB-003 — Exports and transitive public API

- **Distinct behavior:** `export` exposes another library's public namespace without importing it for local use. Combinators shape the transitive API, and conflicting re-exports may make a name ambiguous for clients.
- **Observable content:** A facade library exporting two implementation libraries with `show`/`hide`, a client importing only the facade, and a facade attempting to use an exported-only name.
- **Variants and failures:** Export cycles, duplicate exports, conflicting names, a private declaration behind an export, and a locally declared name colliding with a re-export. Include an export whose target changes under conditional selection.
- **Constraints / authority:** Export namespaces and client resolution are language-defined; some ambiguity reporting is analyzer/compiler-facing. **Language plus official toolchain diagnostics.**
- **Confidence / likely source:** High. Dart language specification; libraries and imports documentation.

### DART-LIB-004 — URI forms and package identity

- **Distinct behavior:** `dart:`, `package:`, and relative URIs resolve through different authorities. Package URIs depend on the generated package configuration, while relative URIs depend on the containing source URI.
- **Observable content:** Imports using all applicable URI schemes; a public `package:` import; an internal relative import; and two paths that might reach the same file.
- **Variants and failures:** Unknown packages, paths outside the package, case differences, percent encoding, symlinks, non-canonical `lib/../lib` paths, and `package:` access to `lib/src`. Demonstrate that `lib/src` is technically importable but conventionally private.
- **Constraints / authority:** URI resolution is language- and toolchain-defined; package mapping comes from Pub. Filesystem case and symlink behavior can be platform-dependent. **Language, official toolchain, and ecosystem convention.**
- **Confidence / likely source:** High. Dart language specification; Pub package configuration and package layout documentation.

### DART-LIB-005 — Conditional imports and exports

- **Distinct behavior:** A conditional URI selects exactly one implementation from compile-time environment booleans such as platform-library availability. The selected library changes declarations, types, and dependencies present in a build.
- **Observable content:** One interface-facing library with native, browser, and fallback implementations selected by conditional import or export, then referenced through a common API.
- **Variants and failures:** No condition true, more than one true with first-match behavior, incompatible selected APIs, unsupported condition keys, a missing unselected file, and nesting conditional export behind a facade. Test analysis versus compilation for several targets.
- **Constraints / authority:** Conditions and recognized environment keys are constrained; arbitrary runtime conditions are not allowed. Supported libraries differ by target and SDK. **Language plus official toolchain.**
- **Confidence / likely source:** High for selection semantics; medium for the full current set of supported keys. Dart language documentation for libraries and imports; SDK platform-library documentation.

### DART-LIB-006 — Deferred imports and load boundaries

- **Distinct behavior:** A `deferred as` import creates a prefix whose members cannot be used until `loadLibrary()` completes. Web builds can use it for deferred code loading; native implementations may load eagerly while preserving program semantics.
- **Observable content:** A deferred import, awaited `prefix.loadLibrary()`, a type or value access after loading, and repeated loads.
- **Variants and failures:** Access before load, using a deferred type where compile-time availability is required, const references, multiple deferred prefixes to one library, load failure, and target-specific chunking. Compare deferred and ordinary imports of the same library.
- **Constraints / authority:** Runtime loading strategy is implementation-defined. Language restrictions on deferred prefixes are static. **Language plus official compilers.**
- **Confidence / likely source:** Medium. Dart language specification; libraries and imports documentation; web compiler documentation.

### DART-LIB-007 — Library-private identifiers

- **Distinct behavior:** An identifier beginning with `_` is private to its library, not to a class or file. Parts share access; imports and subtyping do not grant access.
- **Observable content:** Private top-level, class, constructor, getter, setter, extension, and named-parameter declarations referenced from the same file, a part, and a different library.
- **Variants and failures:** Public subclasses cannot override or invoke another library's private member by spelling; the same `_name` in two libraries denotes different identities. Test private named constructors and private types exposed accidentally through public signatures.
- **Constraints / authority:** Some analyzer diagnostics enforce public-API hygiene beyond language accessibility. **Language plus official analyzer.**
- **Confidence / likely source:** High. Dart language specification; language documentation for libraries.

### DART-LIB-008 — Declaration namespaces, shadowing, and ambiguous references

- **Distinct behavior:** Dart lookup spans lexical scopes, instance members, type declarations, imports, prefixes, labels, and type parameters. The same spelling can denote different entities by context, and local declarations can hide outer or imported names.
- **Observable content:** Nested functions and blocks with shadowing, a type parameter matching a top-level type, a local matching an instance member, labels, and type/expression uses of the same spelling.
- **Variants and failures:** Use `this` to bypass local shadowing, qualified import names, static-versus-instance access, setter/getter namespace interactions, forward references, and genuinely ambiguous imports. Include unreachable but still statically resolved references.
- **Constraints / authority:** Exact scoping and declaration-order rules vary by declaration kind. **Language.**
- **Confidence / likely source:** High. Dart language specification.

### DART-LIB-009 — Metadata annotations

- **Distinct behavior:** Metadata attaches constant expressions to libraries, directives, declarations, parameters, type uses, and other supported syntactic targets. Tools and generators can interpret annotations even when runtime behavior is unchanged.
- **Observable content:** Built-in annotations and custom `const` annotation classes applied to several target kinds, including annotations imported through a prefix.
- **Variants and failures:** Non-constant metadata, an invalid target, duplicate annotations, private annotation classes, annotations on types versus declarations, and annotations consumed by a generator. Include tool-recognized annotations whose effect is diagnostic only.
- **Constraints / authority:** Constant-expression validity and legal positions are language-defined; meaning assigned by analyzer, compiler, or packages is tool-specific. **Language plus official toolchain/ecosystem consumers.**
- **Confidence / likely source:** High. Dart language specification; metadata documentation; relevant tool annotation documentation.

## 2. Types, inference, and flow analysis

### DART-TYPE-001 — Sound null safety and nullable type structure

- **Distinct behavior:** `T` and `T?` have distinct subtyping and member-access rules. `Null`, `Never`, `Object`, `Object?`, `dynamic`, and `void` occupy special positions, and soundness is enforced across assignments, returns, overrides, and generic instantiations.
- **Observable content:** Nullable and non-nullable variables, nullable member access, `!`, `?.`, `??`, `??=`, nullable generics, and functions returning `Never` and `void`.
- **Variants and failures:** Null assertion failure, assigning `null` to non-nullable types, `dynamic` postponing checks, `void` value restrictions, `Never` reachability, `T?` where `T` may already be nullable, and promotion after null tests.
- **Constraints / authority:** Dart 3 requires sound null safety for supported language versions; old pre-null-safety code has sharply reduced or no support depending on SDK and command. **Language plus official runtime.**
- **Confidence / likely source:** High. Dart null-safety documentation; Dart language specification; type-system documentation.

### DART-TYPE-002 — Per-library language versions and SDK constraints

- **Distinct behavior:** The package SDK constraint selects a default language version, while a file language-version comment can opt a library down within supported bounds. Syntax, inference, errors, and feature availability can therefore differ between libraries in one package graph.
- **Observable content:** A `pubspec.yaml` SDK range and libraries with explicit `// @dart =` versions using a feature introduced after the opted-down version.
- **Variants and failures:** Opting above the SDK, below the minimum supported version, malformed comments, a part with a conflicting version, generated files carrying stale markers, and dependencies with older language versions.
- **Constraints / authority:** Minimum supported versions move with SDK releases. Parts inherit or must agree with the owning library under specific rules. **Official SDK/analyzer implementing language versioning.**
- **Confidence / likely source:** High in concept, medium on current lower bounds. Dart language-versioning documentation; Pub SDK-constraint documentation.

### DART-TYPE-003 — `dynamic`, implicit dynamic, and runtime invocation

- **Distinct behavior:** `dynamic` suppresses most static member checking and inserts runtime dispatch/checks. It differs from `Object?`, which permits few members, and from inferred concrete types.
- **Observable content:** Explicit and implicit `dynamic`, dynamic calls, property reads/writes, generic values involving `dynamic`, casts, and a failed runtime method lookup.
- **Variants and failures:** `dynamic` as a top or bottom-like type in assignability, dynamic tear-offs, `noSuchMethod` interception, runtime argument-shape errors, and analyzer strict modes that reject implicit dynamic. Contrast `var`, `Object?`, and `dynamic` initialized from the same expression.
- **Constraints / authority:** Core semantics are language/runtime-defined; strict diagnostics are analyzer configuration. **Language plus official analyzer/runtime.**
- **Confidence / likely source:** High. Dart language specification; type-system and analyzer strictness documentation.

### DART-TYPE-004 — Generic classes, bounds, and instantiate-to-bound

- **Distinct behavior:** Generic types have reified type arguments and bounded type parameters. Raw or omitted type arguments are completed through instantiate-to-bound rules rather than simple textual defaults.
- **Observable content:** Generic classes and aliases with upper bounds, recursive/F-bounds, omitted arguments, nested bounds, and runtime type tests preserving arguments.
- **Variants and failures:** Bound violations, `dynamic` and nullable arguments, mutually dependent bounds, legacy-style raw types, constructor inference, and generic subtype relationships. Include `C` versus `C<Object?>` where completion may differ.
- **Constraints / authority:** Exact instantiate-to-bound rules have evolved and are language-version-sensitive. **Language.**
- **Confidence / likely source:** High overall, medium for obscure recursive cases. Dart language specification; generics documentation.

### DART-TYPE-005 — Generic methods and generic function types

- **Distinct behavior:** Functions and methods can declare type parameters, and function types can themselves be generic. Instantiation may be explicit, inferred, or represented by a generic tear-off.
- **Observable content:** Generic top-level/local functions, generic instance/static methods, explicit type arguments, inferred calls, and variables typed as generic function types.
- **Variants and failures:** A generic function assigned to nongeneric and generic function types, bounds at tear-off and invocation, higher-order inference, partial-looking instantiations that are illegal, and dynamic generic invocation.
- **Constraints / authority:** Generic function subtyping and inference are language-defined. **Language.**
- **Confidence / likely source:** High. Dart language specification; generics documentation.

### DART-TYPE-006 — Function-type subtyping and parameter shapes

- **Distinct behavior:** Function types encode return type, positional arity, optionality, named parameter names, `required`, and parameter types. Assignability is contravariant in parameters and covariant in results, subject to Dart's precise rules.
- **Observable content:** Assignments among function values with required/optional positional and named parameters, callable objects, methods, and closures.
- **Variants and failures:** Renamed named parameters, missing or extra named parameters, `void` result adaptation, nullable functions, generic functions, covariant runtime checks, and overriding methods with altered shapes.
- **Constraints / authority:** Details involving `void`, `dynamic`, and required named parameters are easy to misstate and should be checked against the current specification. **Language.**
- **Confidence / likely source:** High for main behavior, medium for edge cases. Dart language specification; type-system documentation.

### DART-TYPE-007 — Type aliases

- **Distinct behavior:** `typedef` can name function types and general type expressions. A non-class alias generally preserves the aliased type's identity rather than creating a nominal wrapper.
- **Observable content:** Generic aliases for functions, records, nullable types, and parameterized classes; aliases used in constructors, type tests, and public APIs.
- **Variants and failures:** Cyclic aliases, aliases with bounds, old function-typedef syntax, alias constructor invocation, equality of runtime types, and an alias mistaken for a distinct nominal type. Contrast with an extension type or wrapper class.
- **Constraints / authority:** Generalized non-function aliases require a sufficiently recent Dart 2 language version. **Language.**
- **Confidence / likely source:** High. Dart language specification; typedef documentation.

### DART-TYPE-008 — Contextual type inference and inference failures

- **Distinct behavior:** Dart inference flows both downward from context and upward from expressions. It determines local variable types, generic call arguments, collection element types, closure parameter types, record types, and constructor invocations.
- **Observable content:** The same literal, closure, and generic invocation with and without a contextual type; `var`, `final`, and explicit declarations; empty collections; and chained generic calls.
- **Variants and failures:** Inference choosing `dynamic`, `Never`, least upper bounds, or downwards constraints; ambiguous closures; cascades; conditional expressions; collection control flow; and invalid inference due to bounds. Include strict-inference analyzer settings.
- **Constraints / authority:** Core inference is language-defined and has changed across language versions; strict-inference diagnostics are analyzer policy. **Language plus official analyzer.**
- **Confidence / likely source:** High for categories, medium for individual least-upper-bound corner cases. Dart language specification; type inference documentation; analyzer strict-mode documentation.

### DART-TYPE-009 — Flow analysis, promotion, and reachability

- **Distinct behavior:** Flow analysis promotes local variables and eligible fields after tests, assignments, null checks, pattern matches, and terminating control flow. Writes, captures, and aliasing can prevent or demote promotion.
- **Observable content:** Type and null promotions across `if`, loops, early returns, `switch`, local functions, closures, and private final fields.
- **Variants and failures:** Promotion blocked by mutation, closure capture, an overriding getter, public fields, external implementation risk, or an unrelated write; promotion of private final fields in supported language versions; `Never`-returning calls and unreachable branches.
- **Constraints / authority:** Field promotion and the reasons reported for non-promotion changed materially during Dart 3. Language-version and analyzer behavior matter. **Language plus official analyzer diagnostics.**
- **Confidence / likely source:** High. Dart language specification; flow-analysis and non-promotion documentation.

### DART-TYPE-010 — Explicit casts, type tests, and reified generics

- **Distinct behavior:** `is`, `is!`, and `as` perform runtime type checks and feed promotion. Dart preserves enough generic type information for many runtime checks, unlike fully erased generic systems.
- **Observable content:** Successful and failed casts, generic `is` checks, promoted branches, nullable targets, function types, records, and extension-type values.
- **Variants and failures:** `dynamic` casts, checks involving `Never` or `Null`, covariant generic positions, JavaScript-target representation limits, and patterns using cast or type checks. Contrast a type alias with a nominal class.
- **Constraints / authority:** Semantic checks are language-defined; exact runtime representation and messages differ by backend. **Language plus official implementations.**
- **Confidence / likely source:** High. Dart language specification; type-system documentation.

### DART-TYPE-011 — `FutureOr<T>` special type semantics

- **Distinct behavior:** `FutureOr<T>` is not an ordinary union class. The type system gives it special subtyping and normalization behavior, affecting async APIs and generic inference.
- **Observable content:** APIs accepting and returning `FutureOr<T>`, assignments of both `T` and `Future<T>`, nested or nullable forms, and async flattening contexts.
- **Variants and failures:** `FutureOr<Object>`, `FutureOr<dynamic>`, `FutureOr<Never>`, nested `FutureOr`, runtime type tests, and confusion with a nominal union. Include generic substitution where `T` itself is a future.
- **Constraints / authority:** Defined jointly by language rules and the SDK core library declaration. **Language plus official SDK.**
- **Confidence / likely source:** Medium-high. Dart language specification; `dart:async` API documentation.

### DART-TYPE-012 — Records and structural record types

- **Distinct behavior:** Records are immutable, structurally typed products. Shape includes positional arity and named-field names; named-field order is not identity, while positional field labels in type annotations are documentation only.
- **Observable content:** Record literals and types, named and positional fields, destructuring, returns, generic fields, equality, and runtime type checks.
- **Variants and failures:** One-element record syntax, empty-looking parentheses, reordered named fields, different named labels, positional documentation names, nullable records versus nullable fields, and record-shape mismatch.
- **Constraints / authority:** Records became stable in Dart 3. **Language plus official core runtime behavior.**
- **Confidence / likely source:** High. Dart records documentation; Dart language specification.

## 3. Object model and member dispatch

### DART-OBJ-001 — Class inheritance, interfaces, and implicit interfaces

- **Distinct behavior:** Every ordinary class declaration also defines an interface. `extends` inherits implementation, while `implements` requires an interface contract without inheriting bodies; a class may implement multiple interfaces.
- **Observable content:** A superclass, subclasses, `implements` clauses, inherited and reimplemented members, abstract members, and subtype assignments.
- **Variants and failures:** Missing implementations, incompatible overrides, repeated interfaces through different paths, generic substitution, diamond-shaped interface graphs, static members that are not inherited as instance interface members, and private members across libraries.
- **Constraints / authority:** Class modifiers can restrict which relationships are legal across libraries. **Language.**
- **Confidence / likely source:** High. Dart language specification; classes and class-modifier documentation.

### DART-OBJ-002 — Dart 3 class modifiers and closed-world guarantees

- **Distinct behavior:** `base`, `interface`, `final`, `sealed`, `mixin`, and combinations such as `abstract interface` constrain external extension, implementation, mixing-in, construction, and exhaustiveness. Restrictions differ inside and outside the declaring library.
- **Observable content:** Separate libraries attempting permitted and forbidden `extends`, `implements`, and `with` relationships for each meaningful modifier, plus a sealed hierarchy used in a switch.
- **Variants and failures:** Modifier propagation requirements on subtypes, `abstract base`, `abstract final`, `mixin class`, sealed direct versus indirect subtypes, private subtypes, and subtype declarations in parts. Avoid treating `sealed` as runtime sealing.
- **Constraints / authority:** Stable from Dart 3.0; exact legal modifier combinations and downstream obligations are language-defined. **Language.**
- **Confidence / likely source:** High. Dart class-modifier documentation; Dart language specification.

### DART-OBJ-003 — Mixins, `on` constraints, and mixin application

- **Distinct behavior:** Mixins contribute instance members to a generated mixin-application class. `on` constraints state required superclass interfaces and affect `super` resolution inside the mixin.
- **Observable content:** A mixin declaration with fields, methods, an `on` constraint, and `super` calls; classes applying one and several mixins in order.
- **Variants and failures:** Conflicting members, later mixins overriding earlier ones, unsatisfied constraints, generic mixins, `mixin class` used both ways, constructors excluded from mixin contribution, and modifiers restricting application.
- **Constraints / authority:** Application order and superclass linearization are language-defined. **Language.**
- **Confidence / likely source:** High. Dart mixins and class-modifier documentation; Dart language specification.

### DART-OBJ-004 — Override checking, `covariant`, and runtime parameter checks

- **Distinct behavior:** Overrides must satisfy interface subtyping rules. A `covariant` parameter permits a narrower implementation-facing type by inserting a runtime check at dynamically dispatched entry.
- **Observable content:** Base and override methods/getters/setters, generic substitution, covariant parameters declared in a superclass and locally, and calls through base-typed and subtype-typed references.
- **Variants and failures:** Unsound-looking narrowing without `covariant`, runtime failure through a base reference, named parameter shape changes, return covariance, field-induced getter/setter overrides, multiple interface constraints, and analyzer `@override` diagnostics.
- **Constraints / authority:** Override validity and runtime checks are language-defined; `@override` itself mainly assists tools. **Language plus official analyzer.**
- **Confidence / likely source:** High. Dart language specification; type-system documentation.

### DART-OBJ-005 — Instance, static, top-level, and extension member resolution

- **Distinct behavior:** Instance dispatch is based on receiver runtime class after static resolution admits the call. Static and top-level members are lexically selected and not virtual. Extension members are statically selected and do not override instance members.
- **Observable content:** Same-named instance/static/top-level/extension members invoked through variables with different static and runtime types.
- **Variants and failures:** Static access through instances, hidden statics, an extension losing to an instance member, dynamic receivers ignoring extensions, null-aware invocation, generic receiver types, and callable members.
- **Constraints / authority:** Language-defined; compiler optimizations must preserve observable dispatch. **Language.**
- **Confidence / likely source:** High. Dart language specification; extension-method documentation.

### DART-OBJ-006 — Extension methods and ambiguity resolution

- **Distinct behavior:** Extensions add statically resolved members based on the receiver's static type and extension applicability/specificity. They have names/import visibility, can be unnamed, generic, and explicitly invoked.
- **Observable content:** Competing extensions from separate imports, a more-specific extension, explicit extension override syntax, an unnamed local extension, and a dynamic receiver.
- **Variants and failures:** Equal-specificity ambiguity, hidden or prefixed extensions, nullable receivers, generic inference, instance-member precedence, extension getters/setters/operators, and the same extension re-exported through multiple paths.
- **Constraints / authority:** Extensions do not modify runtime classes. Import rules and specificity determine availability. **Language.**
- **Confidence / likely source:** High. Dart extension-method documentation; Dart language specification.

### DART-OBJ-007 — Extension types and representation-type semantics

- **Distinct behavior:** An extension type is a compile-time abstraction over a representation type. It can expose members and implement allowed interfaces without necessarily allocating a wrapper; representation and abstraction boundaries affect assignability and dispatch.
- **Observable content:** Generic and nongeneric extension types, representation constructors, declared members, interface implementation, explicit access to representation behavior where allowed, and runtime checks.
- **Variants and failures:** Two extension types over one representation, extension type versus type alias, nullable representation, member name collision, invalid interface implementation, representation exposure, dynamic calls, and backend erasure effects.
- **Constraints / authority:** Extension types became stable in Dart 3.3. Runtime identity and `is` behavior require careful verification against the current specification and backend. **Language plus official implementations.**
- **Confidence / likely source:** Medium-high. Dart extension-types documentation; Dart language specification.

### DART-OBJ-008 — Constructors, initializer order, and initializing formals

- **Distinct behavior:** Generative constructors create instances and initialize fields through defaults, initializing formals, initializer lists, superclass invocation, and constructor bodies in a defined order.
- **Observable content:** Named and unnamed constructors, field formals, initializer-list expressions, assertions, explicit and implicit `super` calls, and field initializers with observable side effects.
- **Variants and failures:** Reading `this` too early, duplicate field initialization, uninitialized final fields, redirect versus body, constructor inheritance misconception, private named constructors, and initialization failure. Include parameter defaults and required named constructor parameters.
- **Constraints / authority:** Exact evaluation order is language-defined; observable side effects should avoid relying on unspecified optimizer details. **Language.**
- **Confidence / likely source:** High. Dart constructors documentation; Dart language specification.

### DART-OBJ-009 — Super parameters and constructor redirection

- **Distinct behavior:** Super-initializer parameters forward arguments to a superclass constructor. Redirecting generative constructors delegate within a class; redirecting factories may target another class and affect apparent construction identity.
- **Observable content:** Positional and named super parameters, explicit super arguments, generative redirects, factory redirects, and generic target constructors.
- **Variants and failures:** Duplicate forwarding, renamed or incompatible super parameters, redirects with cycles, redirects to abstract/inaccessible constructors, mixed field and super formals, and const redirect chains.
- **Constraints / authority:** Super parameters require Dart 2.17 or later; redirect rules are language-defined. **Language.**
- **Confidence / likely source:** High. Dart constructors documentation; Dart language specification.

### DART-OBJ-010 — Factory constructors, caching, and subtype returns

- **Distinct behavior:** A factory constructor need not allocate a fresh instance and may return an instance of a subtype or cached object. It has no access to a newly allocated `this`.
- **Observable content:** A factory returning a private implementation, a cache returning identical instances, and a factory redirect.
- **Variants and failures:** Return type incompatibility, null return, generic factories, const factories where legal, constructor tear-offs, abstract classes with factories, and clients assuming runtime type equals the invoked class.
- **Constraints / authority:** Constructor interface and return restrictions are language-defined; caching strategy is program behavior. **Language.**
- **Confidence / likely source:** High. Dart constructors documentation; Dart language specification.

### DART-OBJ-011 — `const` constructors and canonical instances

- **Distinct behavior:** Const construction evaluates at compile time and canonicalizes equal constant objects within the language's constant semantics. A const-capable constructor can still be invoked non-const outside a const context.
- **Observable content:** Const classes with final fields, repeated equal const invocations tested with `identical`, implicit const contexts, and a non-const invocation of the same constructor.
- **Variants and failures:** Non-final fields, non-constant arguments, const subclass requirements, redirecting const factories, generic const instances, canonicalization across libraries, and compile-time errors inside constant evaluation.
- **Constraints / authority:** Constant identity is language-defined; object layout is implementation-defined. **Language plus official runtime.**
- **Confidence / likely source:** High. Dart language specification; constructors and constant documentation.

### DART-OBJ-012 — Fields, getters, setters, and property syntax

- **Distinct behavior:** Property reads and writes resolve to fields or accessor methods. A field induces a getter and sometimes a setter; overriding and interface checks operate on those accessor signatures.
- **Observable content:** Mutable/final fields, explicit getters/setters, abstract accessors, static accessors, compound assignments, and overrides that replace a field with accessors or vice versa.
- **Variants and failures:** Getter-only assignment, setter-only reads, mismatched getter/setter types, compound assignment reading then writing, null-aware assignment, field shadowing, and interface conflicts from multiple supertypes.
- **Constraints / authority:** Resolution and evaluation order are language-defined. **Language.**
- **Confidence / likely source:** High. Dart language specification; classes documentation.

### DART-OBJ-013 — `late` variables and lazy initialization

- **Distinct behavior:** `late` delays initialization or initialization checking. A `late final` variable permits one successful assignment; reading an uninitialized late variable or assigning twice fails at runtime. Late instance/top-level initializers can access `this` or run lazily.
- **Observable content:** Local, instance, static, and top-level late variables; lazy initializers with counters; `late final` assignment; and error cases.
- **Variants and failures:** Re-entrant or throwing initializers, cyclic late initialization, initializer retried or not after failure as specified, writes before first read, promoted late locals, and backend-specific error text.
- **Constraints / authority:** Core behavior is language-defined; exception types/messages and optimization are implementation details. **Language plus official runtime.**
- **Confidence / likely source:** High overall, medium on failure/retry corner cases. Dart null-safety documentation; Dart language specification.

### DART-OBJ-014 — Top-level and static initialization

- **Distinct behavior:** Top-level and static variables with initializers are generally initialized lazily on first access, creating observable ordering and cyclic-initialization behavior across libraries.
- **Observable content:** Cross-library initializers with side effects and dependencies, const versus final variables, and a deliberate initialization cycle.
- **Variants and failures:** Initializer throws, recursive read during initialization, unused variable never initialized, multiple isolates each initializing state, and compiler tree shaking. Contrast eager local initialization.
- **Constraints / authority:** Semantics are language-defined, while scheduling and storage are implementation-defined. **Language plus official runtime.**
- **Confidence / likely source:** Medium-high. Dart language specification.

### DART-OBJ-015 — Operators, `==`, `hashCode`, indexing, and `call`

- **Distinct behavior:** User-defined operators are statically admitted then dynamically dispatched as instance methods. `[]`, `[]=`, unary/binary operators, relational operators, `==`, and `call` have distinct invocation shapes; hash consistency is a library contract.
- **Observable content:** A class defining representative operators, callable instances, equality/hash behavior in a set or map, and calls through supertype/dynamic references.
- **Variants and failures:** Unsupported operator declarations, asymmetric equality across types, null comparison rules, compound indexing evaluation order, `identical` versus `==`, and callable-object tear-offs or invocation. Include extension operators for contrast.
- **Constraints / authority:** Operator mapping and equality invocation are language-defined; `hashCode` consistency is an official core-library contract. **Language plus official SDK.**
- **Confidence / likely source:** High. Dart language specification; operators documentation; `Object` API documentation.

### DART-OBJ-016 — `noSuchMethod` and dynamic forwarding

- **Distinct behavior:** Failed dynamic instance invocations can reach `noSuchMethod`. A concrete `noSuchMethod` implementation can also satisfy some otherwise missing interface members under specified conditions, with runtime forwarding.
- **Observable content:** A class implementing an interface through `noSuchMethod`, dynamic missing getter/setter/method calls, and inspection of `Invocation` data.
- **Variants and failures:** Statically known missing members, correct versus incorrect argument shapes, private symbols, named arguments, invocation through an interface, and targets/backends that optimize forwarding differently.
- **Constraints / authority:** Exact forwarding eligibility and `Invocation` contents are language/core-library-defined; error messages are implementation-specific. **Language plus official SDK/runtime.**
- **Confidence / likely source:** Medium-high. Dart language specification; `Object.noSuchMethod` and `Invocation` API documentation.

### DART-OBJ-017 — Enhanced enums

- **Distinct behavior:** Enhanced enum declarations create a sealed set of canonical instances and may declare fields, methods, interfaces, mixins, const constructors, and per-value arguments or bodies.
- **Observable content:** An enum with fields, a const constructor, overridden method, interface implementation, mixin, `values`, `index`, and switch exhaustiveness.
- **Variants and failures:** Duplicate names, attempts to instantiate or extend, overriding reserved members, generic-looking limitations, identity and serialization assumptions, enum value-specific behavior, and adding a value breaking exhaustive switches.
- **Constraints / authority:** Enhanced enums require Dart 2.17 or later. **Language plus official core behavior.**
- **Confidence / likely source:** High. Dart enum documentation; Dart language specification.

## 4. Functions, closures, asynchronous execution, and errors

### DART-FUN-001 — Parameter kinds, defaults, and invocation shape

- **Distinct behavior:** Functions distinguish required positional, optional positional, named, and required named parameters. Default values are compile-time constants, and named argument identity is part of the API and function type.
- **Observable content:** Top-level, local, method, and constructor declarations using every parameter kind, defaults, named argument reordering, and invocations through typed function values.
- **Variants and failures:** Missing required arguments, extra/unknown names, positional/named syntax confusion, nullable optional parameters without defaults, default-value type mismatch, duplicate names, and override shape compatibility.
- **Constraints / authority:** Language-defined. **Language.**
- **Confidence / likely source:** High. Dart functions documentation; Dart language specification.

### DART-FUN-002 — Lexical closures, capture, and local declarations

- **Distinct behavior:** Closures capture lexical variables by reference-like variable identity, not by copying each value. Local functions and anonymous functions have their own scopes, type inference, and recursion behavior.
- **Observable content:** Closures created in loops and nested scopes, mutation after capture, recursive local functions, escaping closures, and captured type-promoted variables.
- **Variants and failures:** Loop-variable capture semantics, closure capture blocking promotion, `this`/type-parameter capture, async closure lifetime, shadowed captures, and identity/equality of separate closures.
- **Constraints / authority:** Language semantics define capture results; allocation and closure representation are implementation-defined. **Language plus official runtimes.**
- **Confidence / likely source:** High. Dart language specification; functions documentation.

### DART-FUN-003 — Function, method, and constructor tear-offs

- **Distinct behavior:** Referring to a function-like declaration without invoking it creates a function object. Instance tear-offs bind a receiver; static/top-level and constructor tear-offs do not. Generic tear-offs may be instantiated.
- **Observable content:** Tear-offs of top-level, static, instance, extension, generic, unnamed/named constructor, and callable members, assigned to explicit function types.
- **Variants and failures:** Equality of repeated tear-offs, a receiver expression evaluated once, generic instantiation, nullable receivers, factory constructor tear-offs, overloaded-looking named constructors, and method extraction from `dynamic`.
- **Constraints / authority:** Constructor tear-offs require Dart 2.15 or later. Function-object identity details should be verified. **Language plus official runtime.**
- **Confidence / likely source:** High for resolution, medium for equality identity. Dart language specification; functions and constructors documentation.

### DART-FUN-004 — `async`, `await`, and future flattening

- **Distinct behavior:** An `async` function returns a `Future`, suspends at `await`, and converts returned values/errors according to async flattening rules. Evaluation before the first suspension and resumption ordering are observable.
- **Observable content:** Async functions returning values, futures, and errors; multiple awaits; `try/finally` across suspension; and calls that observe synchronous setup versus later completion.
- **Variants and failures:** Awaiting non-futures, nested future-like generic types, synchronous throw before/inside async transformation, `void` async callbacks, unawaited futures, cancellation misconception, and stack traces.
- **Constraints / authority:** Language defines transformation semantics; event-loop scheduling and stack-trace presentation involve `dart:async` and runtime implementations. **Language plus official SDK/runtime.**
- **Confidence / likely source:** High. Dart asynchronous-programming documentation; Dart language specification; `dart:async` API.

### DART-FUN-005 — Synchronous and asynchronous generators

- **Distinct behavior:** `sync*` returns an `Iterable` and uses `yield`/`yield*`; `async*` returns a `Stream`. Execution is lazy and interacts with iteration, pausing, cancellation, errors, and delegation.
- **Observable content:** Both generator kinds, delegated generators, side effects before and after yields, early consumer termination, and errors/finally blocks.
- **Variants and failures:** Yield type mismatch, recursive generators, `yield*` over sync versus async sources, stream pause/cancel behavior, multiple iteration/listen attempts, and generator return restrictions.
- **Constraints / authority:** Core transformation is language-defined; `Stream` subscription behavior comes from the official library. **Language plus official SDK.**
- **Confidence / likely source:** High overall, medium for cancellation edge cases. Dart language specification; asynchronous-programming and `dart:async` documentation.

### DART-FUN-006 — Exceptions, typed catches, rethrow, and finally

- **Distinct behavior:** Any non-null object can be thrown. `on` clauses filter by runtime type, `catch` binds exception and stack trace, `rethrow` preserves the active exception, and `finally` runs across normal, exceptional, return, and async exits.
- **Observable content:** Custom thrown objects, ordered typed catches, stack binding, rethrow through nested functions, finally with return/throw interactions, and async errors.
- **Variants and failures:** Throwing `null` under null safety, catch-order shadowing, bare `rethrow` outside catch, finally overriding an earlier return/error, VM versus web stack traces, and uncaught isolate errors.
- **Constraints / authority:** Control behavior is language-defined; stack trace format and uncaught handling are implementation/platform-specific. **Language plus official runtime.**
- **Confidence / likely source:** High. Dart error-handling documentation; Dart language specification.

### DART-FUN-007 — Assertions and build-mode behavior

- **Distinct behavior:** `assert` conditions and optional messages are checked only when assertions are enabled. Their expressions may not execute in production configurations, so side effects are not reliable program behavior.
- **Observable content:** Assertions with lazy message expressions and side-effect counters, compiled/run with assertions enabled and disabled.
- **Variants and failures:** Constant-false assertions, assertion in const construction, custom messages, web/native differences, and optimizer removal. Include analyzer diagnostics independent of runtime assertion settings.
- **Constraints / authority:** Enablement defaults and command-line flags differ by tool and build mode; AOT production builds commonly disable assertions. **Language plus official toolchain.**
- **Confidence / likely source:** High. Dart language specification; SDK command/build-mode documentation.

### DART-FUN-008 — Isolates, entry points, and message transfer

- **Distinct behavior:** Isolates have separate mutable heaps and communicate by messages. Spawned entry points, closure capture, sendability, error ports, exit ports, and transfer of immutable or transferable data materially affect reachable code and runtime behavior.
- **Observable content:** `Isolate.spawn` or equivalent, top-level and closure entry points where supported, `SendPort`/`ReceivePort`, valid messages, an unsendable object, error/exit listeners, and `TransferableTypedData`.
- **Variants and failures:** `spawn` versus `spawnUri`, same-code versus separate-code isolates, sending closures, native resources, isolate termination, per-isolate static initialization, web support limitations, and AOT tree-shaking entry-point preservation.
- **Constraints / authority:** Official `dart:isolate` behavior differs by platform; browser JavaScript does not expose the same isolate model. **Official SDK/runtime, grounded in language concurrency model.**
- **Confidence / likely source:** Medium-high. Dart concurrency documentation; `dart:isolate` API; compiler entry-point documentation.

### DART-FUN-009 — Event queue, microtask queue, Futures, Streams, and Zones

- **Distinct behavior:** Dart asynchronous execution distinguishes event and microtask scheduling. Futures and Streams propagate values/errors through this scheduling, while Zones can intercept scheduling, error handling, and zone-local values.
- **Observable content:** Ordered `scheduleMicrotask`, `Future` callbacks, timers, sync and async stream controllers, and `runZonedGuarded` with zone-local data.
- **Variants and failures:** Starving the event queue with microtasks, synchronous stream reentrancy, unhandled async errors, broadcast versus single-subscription streams, callback registered in one zone and completed in another, and target-specific event sources.
- **Constraints / authority:** APIs are official; integration with host event loops is platform/implementation-defined. **Official SDK/runtime.**
- **Confidence / likely source:** Medium-high. `dart:async` API documentation; Dart asynchronous-programming documentation.

## 5. Expressions, control flow, constants, and collections

### DART-EXPR-001 — Evaluation order and short-circuit operators

- **Distinct behavior:** Operand, receiver, argument, assignment, and collection-element evaluation order determines visible side effects. `&&`, `||`, `??`, null-aware access, and conditionals selectively skip evaluation.
- **Observable content:** Small side-effecting functions used as receivers, operands, positional/named arguments, index expressions, and right-hand sides.
- **Variants and failures:** Named argument source order versus parameter order, compound assignments evaluating a receiver/index once, cascade receivers, exceptions mid-expression, and optimizer preservation. Do not infer unspecified order where the specification does not guarantee it.
- **Constraints / authority:** Mostly language-defined; any unspecified subcases should be recorded rather than normalized to one backend. **Language.**
- **Confidence / likely source:** High for common operators, medium for obscure argument/cascade details. Dart language specification.

### DART-EXPR-002 — Cascades and null-shorting cascades

- **Distinct behavior:** Cascades perform several operations on one evaluated receiver while the overall expression yields that receiver. Null-shorting cascades skip the entire cascade section chain when the receiver is null, distinct from isolated null-aware member calls.
- **Observable content:** `..`, `?..`, cascade assignments, indexing, method calls, nested cascades, and a receiver with an evaluation counter.
- **Variants and failures:** Cascade precedence, cascades on `void`, nullable receiver analysis, `?..` placement, sections containing ordinary `..`, and recent syntax changes permitting more null-aware cascade forms.
- **Constraints / authority:** Core cascades are longstanding; null-aware cascade syntax/semantics have had recent refinements and need baseline verification. **Language.**
- **Confidence / likely source:** High for ordinary cascades, medium for newest syntax. Dart operators documentation; Dart language specification.

### DART-EXPR-003 — Switch statements, switch expressions, guards, and exhaustiveness

- **Distinct behavior:** Modern switches use patterns, optional guards, and exhaustiveness analysis. Switch expressions must be exhaustive; statements have their own completion/fall-through rules. Sealed types, enums, booleans, nullable types, and records can produce closed spaces.
- **Observable content:** Exhaustive and non-exhaustive switches over enums, a sealed hierarchy, nullable values, and records; guarded cases; grouped labels; switch expressions and statements.
- **Variants and failures:** Guarded cases do not necessarily cover the unguarded space, unreachable/dominated cases, newly added enum/sealed subtypes, default/wildcard cases, case-variable scope, and legacy switch syntax. Include invalid exhaustiveness and unreachable-case diagnostics.
- **Constraints / authority:** Pattern switches became stable in Dart 3; analyzer exhaustiveness algorithms may improve across SDKs. **Language plus official analyzer.**
- **Confidence / likely source:** High. Dart patterns and branches documentation; Dart language specification.

### DART-EXPR-004 — Pattern forms, matching, and destructuring

- **Distinct behavior:** Patterns can test and destructure values and bind variables in declarations, assignments, `if-case`, switches, loops, and collection elements. Object, record, list, map, logical, relational, cast, null-check, and null-assert patterns have different refutability and access behavior.
- **Observable content:** Representative nested patterns in each supported context, including typed variables, rest elements, object getters, map keys, logical alternatives, and destructuring assignment.
- **Variants and failures:** Irrefutable patterns required in declarations, failed refutable matches, variable sets/types across `||` alternatives, side-effecting getters, duplicate bindings, list length mismatch, missing map keys, cast failure, null assertion failure, and assignment to existing variables.
- **Constraints / authority:** Stable from Dart 3 with additions in later Dart 3 releases. Precise allowed pattern/context combinations are language-version-sensitive. **Language.**
- **Confidence / likely source:** High overall, medium for newest pattern additions. Dart patterns documentation; Dart language specification.

### DART-EXPR-005 — Loop semantics and pattern iteration

- **Distinct behavior:** `for`, `for-in`, `while`, and `do` differ in scope, evaluation, capture, and control targets. Pattern `for-in` destructures each element and can fail if a refutable pattern is used where not permitted.
- **Observable content:** Classic and collection `for`, synchronous `for-in`, `await for`, labeled `break`/`continue`, loop-variable closures, and destructuring loop variables.
- **Variants and failures:** Mutating a collection during iteration, break/continue through finally, async cancellation, loop-variable capture identity, nullable iteration, and invalid pattern shapes.
- **Constraints / authority:** Core control flow is language-defined; iterator concurrent-modification behavior is collection implementation-specific. **Language plus official SDK collections.**
- **Confidence / likely source:** High. Dart loops and patterns documentation; Dart language specification.

### DART-EXPR-006 — Collection literals, control-flow elements, spreads, and null-aware elements

- **Distinct behavior:** List, set, and map literals infer types and evaluate elements in order. `if`, `for`, spread, null-aware spread, and newer null-aware element forms conditionally contribute entries and affect promotion/inference.
- **Observable content:** Typed and inferred literals with nested collection `if`/`for`, `...`, `...?`, null-aware elements/entries where supported, duplicate set elements and map keys, and const collections.
- **Variants and failures:** Empty `{}` defaults to a map absent context, set/map ambiguity, wrong spread type, null spread, duplicate const map keys, pattern-based collection elements, side effects, and baseline rejection of newer syntax under an older language version.
- **Constraints / authority:** Null-aware collection elements were added in a recent Dart 3 release and need exact version verification; other collection control flow is older. **Language plus official core collections.**
- **Confidence / likely source:** High except exact recent-version boundary, which is medium. Dart collections documentation; Dart language specification.

### DART-EXPR-007 — Constant expressions and constant contexts

- **Distinct behavior:** Constant expressions form a restricted compile-time language. Const contexts can make nested constructors and collection literals implicitly const, and canonicalization gives constants identity semantics.
- **Observable content:** Const arithmetic, symbols, type literals, records, collections, constructor calls, environment values, and nested implicit const contexts.
- **Variants and failures:** Non-const invocation, runtime-dependent values, invalid operator calls, duplicate map keys, cyclic constants, overflow/backend numeric differences, const evaluation throwing, and `const` versus `final`.
- **Constraints / authority:** Language-defined, with compiler diagnostics and possible backend-specific limits. **Language plus official compilers.**
- **Confidence / likely source:** High. Dart language specification; constants documentation.

### DART-EXPR-008 — Compile-time environment declarations

- **Distinct behavior:** `bool.fromEnvironment`, `int.fromEnvironment`, `String.fromEnvironment`, and `bool.hasEnvironment` read compile-time configuration in const contexts. Compilers can use them to include, specialize, or remove code.
- **Observable content:** Const environment reads with and without defaults, build commands supplying declarations, and branches whose reachability/output changes by configuration.
- **Variants and failures:** Runtime invocation expectations, invalid integer values, absent versus empty string, inconsistent declarations across tools, conditional imports using a different condition mechanism, and tree-shaken branches.
- **Constraints / authority:** Values are supplied by each compiler/tool invocation; not all host environment variables are visible. **Official core library and toolchain.**
- **Confidence / likely source:** High. Core-library API documentation; SDK compiler/run command documentation.

### DART-EXPR-009 — Numeric semantics across native, JavaScript, and WebAssembly

- **Distinct behavior:** Dart specifies `int` and `double` APIs, but integer range/precision and representation differ across backends. Native integers and JavaScript-number compilation can diverge for large values, bit operations, identity, and interoperability.
- **Observable content:** Values around 53-bit precision and native integer limits, shifts, unsigned shifts, overflow-sensitive arithmetic, `double.nan`, infinities, negative zero, integer/double equality, and parsing.
- **Variants and failures:** Constant versus runtime arithmetic, JavaScript target precision loss, BigInt as a separate class, `identical` behavior, hash/equality, and WebAssembly compiler behavior.
- **Constraints / authority:** Some semantics are specified with target allowances; representation and performance are implementation-defined. **Language plus official core library/backends.**
- **Confidence / likely source:** Medium. Dart numbers documentation; core-library API; compiler platform documentation; Dart language specification.

### DART-EXPR-010 — Strings, interpolation, symbols, and canonical identity

- **Distinct behavior:** Adjacent string literals concatenate, interpolation evaluates expressions, raw and multiline strings alter escape handling, and `Symbol` values encode names with restrictions that matter for mirrors and `Invocation`.
- **Observable content:** Compile-time and runtime strings, interpolation with side effects, raw/multiline/Unicode escapes, adjacent literals, const symbols, and symbol equality.
- **Variants and failures:** Unicode code units versus runes/graphemes, malformed escapes, `$name` lookup, platform string representation, private-name symbols, and runtime versus canonical const identity.
- **Constraints / authority:** Lexical/string semantics are language-defined; Unicode segmentation requires libraries beyond base string indexing. **Language plus official core libraries.**
- **Confidence / likely source:** High. Dart language specification; built-in types and `Symbol` API documentation.

### DART-EXPR-011 — Wildcard variables and non-binding `_`

- **Distinct behavior:** In recent Dart, declarations and patterns using `_` can denote non-binding wildcard variables rather than one reusable variable. This changes identity, duplicate-declaration behavior, reads, and pattern binding.
- **Observable content:** Multiple `_` declarations/parameters in one scope, wildcard patterns, attempted reads of `_`, and comparison with a named unused variable under different language versions.
- **Variants and failures:** Private identifiers beginning with `_` are not wildcards, older language versions may treat some underscores differently, setter parameter conventions, and generated code that assumed `_` was bindable.
- **Constraints / authority:** Introduced or broadened in a recent Dart 3 release; exact version and syntactic positions require verification. **Language plus official analyzer.**
- **Confidence / likely source:** Medium. Dart language evolution documentation; Dart language specification; analyzer diagnostics.

## 6. Package graph, analysis, generation, and build configuration

### DART-PKG-001 — Pubspec identity, SDK constraints, and dependency graph

- **Distinct behavior:** `pubspec.yaml` declares package name, version, SDK constraints, dependencies, dev dependencies, and dependency sources. Resolution determines which libraries and language versions exist in the graph.
- **Observable content:** A valid pubspec with explicit Dart SDK range, direct runtime and dev dependencies, then generated lock and package-configuration state.
- **Variants and failures:** Missing/incompatible SDK constraints, transitive version conflicts, prereleases, hosted/path/Git/SDK dependencies, invalid package names, package rename, and dev dependency visibility to dependents.
- **Constraints / authority:** Pub semantics and hosted repository policies can change independently of the language. **Official Pub toolchain.**
- **Confidence / likely source:** High. Pub pubspec and dependency documentation.

### DART-PKG-002 — Lock files and reproducible resolution

- **Distinct behavior:** `pubspec.lock` pins a complete resolution for applications; reusable packages normally do not rely on a committed lock for their consumers. The chosen versions can change APIs, language versions, generated content, and diagnostics.
- **Observable content:** Resolution with and without an existing lock, an intentionally changed constraint, and output showing direct/transitive dependency selections.
- **Variants and failures:** Application versus package convention, stale lock, offline cache, SDK-pinned packages, platform-specific dependency constraints, and `pub upgrade` versus `pub get`.
- **Constraints / authority:** Lock format and command behavior are Pub-defined; commit policy is ecosystem convention documented by Pub. **Official Pub plus ecosystem convention.**
- **Confidence / likely source:** High. Pub package dependency and lock-file documentation.

### DART-PKG-003 — Dependency overrides and local replacement

- **Distinct behavior:** Dependency overrides can replace the source or version selected for a package across the resolution, making analyzed code differ from published constraints. Separate override files may keep local changes out of the package manifest.
- **Observable content:** A dependency constrained to one range and overridden to a local path or other version, with a referenced API available only in the override.
- **Variants and failures:** Multiple packages trying to override the same dependency, override incompatibility, override files, transitive assumptions, unpublished local paths, and consumers not inheriting a package's override.
- **Constraints / authority:** Exact override-file support and workspace interaction are Pub-version-dependent. **Official Pub.**
- **Confidence / likely source:** Medium-high. Pub dependency-overrides documentation.

### DART-PKG-004 — Pub workspaces and multi-package resolution

- **Distinct behavior:** Pub workspaces resolve several packages together and can share resolution state while retaining distinct package/library identities, SDK constraints, public APIs, and analysis roots.
- **Observable content:** A workspace root and member package manifests, cross-member dependencies, shared resolution data, and a member excluded or misdeclared.
- **Variants and failures:** Duplicate package names, incompatible member SDK constraints, nested workspaces, path dependency versus workspace membership, package-local overrides, and commands run at root versus member.
- **Constraints / authority:** Workspaces are a comparatively recent Pub feature; manifest keys, lock placement, and minimum SDK must be verified for the baseline. **Official Pub.**
- **Confidence / likely source:** Medium. Pub workspace documentation.

### DART-PKG-005 — Conventional package source surfaces and entry points

- **Distinct behavior:** Pub recognizes packages through `lib/`, while executables commonly live in `bin/`; tests, examples, tools, and web entry points have conventional roles. Only files under `lib/` are addressable by other packages through `package:` URIs.
- **Observable content:** Public libraries in `lib/`, internal libraries in `lib/src/`, executable entry points, tests, and non-library assets/configuration with imports demonstrating allowed reach.
- **Variants and failures:** Importing another package's `bin` or `test`, reaching `lib/src` despite convention, multiple executable declarations, package root relative imports, and files omitted from publication.
- **Constraints / authority:** `lib` package addressing is toolchain behavior; `lib/src` privacy is convention plus lints, not language privacy. **Official Pub/toolchain plus ecosystem convention.**
- **Confidence / likely source:** High. Pub package-layout convention documentation.

### DART-PKG-006 — Analyzer options, strictness, lints, and exclusions

- **Distinct behavior:** `analysis_options.yaml` can include shared configs, enable language experiments, exclude files, change diagnostic severity, and activate strict casts/inference/raw-types. The same source can therefore be accepted, warned, or omitted by different analysis configurations.
- **Observable content:** A base included options file, local overrides, strict modes, lint rules, severity changes, and excluded source containing a known issue.
- **Variants and failures:** Include cycles or missing include, unknown diagnostics/lints, analyzer versus compiler differences, generated-file exclusions, nested packages with separate roots, and command-line options overriding files.
- **Constraints / authority:** Analyzer behavior is official toolchain policy, not all diagnostics are language errors, and options evolve with SDK releases. **Official analyzer plus lint ecosystem.**
- **Confidence / likely source:** High. Analyzer configuration and diagnostic documentation; official lint-package documentation.

### DART-PKG-007 — Compile-time errors, warnings, lints, and runtime failures

- **Distinct behavior:** Dart distinguishes language compile-time errors from analyzer warnings/hints/lints and from runtime failures caused by `dynamic`, casts, `late`, covariance, or assertions. Different tools may continue after non-error diagnostics but must reject true errors.
- **Observable content:** Separate minimal files or configurations demonstrating each class, with diagnostics whose spans can be tied to declarations and references.
- **Variants and failures:** Analyzer-only errors versus compiler behavior, severity promotion/demotion, unreachable code, invalid overrides, inferred dynamic under strictness, and backend-specific runtime exception classes/messages.
- **Constraints / authority:** Diagnostic names and messages are not stable language API; error conditions are specified more durably than codes. **Language plus official analyzer/compilers/runtimes.**
- **Confidence / likely source:** High. Dart language specification; analyzer diagnostic catalog; compiler documentation.

### DART-PKG-008 — Generated sources using `part`

- **Distinct behavior:** Common generators emit `.g.dart` or similar parts that become full members of the owning library, with access to private declarations. Their presence, staleness, and language-version marker can change resolution and diagnostics.
- **Observable content:** An annotated input library, declared generated part, generator configuration, and generated declarations referenced by the input. The report does not prescribe generator code or a fixture layout.
- **Variants and failures:** Missing output, stale output, duplicate declarations, wrong `part of`, generated source excluded from analysis, conflicting language versions, and checked-in versus ephemeral output.
- **Constraints / authority:** Parts are language-defined; naming and workflows are ecosystem conventions, often implemented through `build_runner` and source-generation packages. **Language plus ecosystem tooling.**
- **Confidence / likely source:** High. Dart parts documentation; `build_runner` and source-generation package documentation.

### DART-PKG-009 — Build-system asset graph and builder configuration

- **Distinct behavior:** The common Dart build ecosystem treats source and generated files as assets transformed by configured builders. `build.yaml`, builder ordering, target filters, and generated-output visibility determine which Dart libraries exist for analysis or compilation.
- **Observable content:** Builder configuration with target include/exclude patterns, an input annotation, generated Dart output, and a downstream builder or compiler consuming it.
- **Variants and failures:** Conflicting outputs, hidden versus source outputs, stale build cache, builder phase ordering, optional builders, build filters, nested packages, and source output absent from version control.
- **Constraints / authority:** This is not Dart language behavior. Details depend on `build`, `build_config`, and `build_runner` versions. **Ecosystem convention/tooling.**
- **Confidence / likely source:** Medium-high. Official package documentation for `build_runner`, `build`, and `build_config`.

### DART-PKG-010 — Experimental language features and augmentation/macros history

- **Distinct behavior:** SDK experiment flags can enable syntax or semantics not available in the package's normal language version. Augmentation and macro proposals/tooling have changed significantly and may be experimental, replaced, or unavailable in a given stable SDK.
- **Observable content:** Analyzer/compiler configuration attempting a known experiment in an isolated library, paired with the same syntax without the flag and with a stable alternative where one exists.
- **Variants and failures:** Analyzer accepts while compiler rejects, experiment renamed/removed, dependency language version mismatch, generated augmentations not discoverable, and source written for an abandoned macro prototype.
- **Constraints / authority:** Never assume experimental features are portable or stable. Exact available flags must be queried from the selected SDK during verification. **Official toolchain experiments, not stable language unless incorporated.**
- **Confidence / likely source:** Low-medium. Dart language evolution and SDK experiment documentation; current analyzer/compiler help.

### DART-PKG-011 — Native assets and build hooks

- **Distinct behavior:** Recent Dart package tooling can coordinate native-code assets and build hooks, changing which dynamic/static libraries accompany a package and how FFI bindings resolve at runtime.
- **Observable content:** A package manifest and hook/native-asset configuration that produces or declares a native artifact consumed from Dart FFI, plus a platform without that artifact.
- **Variants and failures:** Host versus target architecture, cross-compilation, missing symbols, transitive native assets, link mode, packaging, hook failure, and unsupported SDK versions.
- **Constraints / authority:** APIs and manifest/build-hook protocols are recent and may still evolve; exact stable status at Dart 3.8 must be verified. **Official toolchain/package ecosystem.**
- **Confidence / likely source:** Low-medium. Dart native-assets and hooks documentation; `dart:ffi` documentation.

### DART-PKG-012 — Publication inclusion and ignored files

- **Distinct behavior:** `dart pub publish` selects a package archive using Pub rules and ignore files. A local build can see files that consumers never receive, producing missing-library, missing-part, or missing-asset failures after publication.
- **Observable content:** A dry-run publication manifest involving source, generated files, platform assets, ignored files, and an import/part that would fail if excluded.
- **Variants and failures:** `.gitignore` versus `.pubignore`, oversized or forbidden files, symlinks, generated sources not checked in, path dependencies, and files outside the package root.
- **Constraints / authority:** Publication rules are Pub/host policy and can change. **Official Pub and hosted-service policy.**
- **Confidence / likely source:** Medium-high. Pub publishing and package-layout documentation.

## 7. Platforms, compilation modes, and interoperation

### DART-PLAT-001 — VM JIT versus AOT compilation

- **Distinct behavior:** VM JIT supports dynamic development features and runtime compilation strategies; AOT produces a closed-world executable with tree shaking and stricter entry-point reachability. Both must preserve language semantics but differ in reflection, code loading, startup, and retained declarations.
- **Observable content:** One program compiled/run in both modes, with dynamic invocation, deferred loading, assertions, isolates, and intentionally indirectly reached entry points.
- **Variants and failures:** Tree-shaken callbacks, `@pragma` entry-point preservation, mirrors, native symbols, generic reification, stack traces, and build-mode constants. Performance differences alone are out of scope unless they change observable behavior.
- **Constraints / authority:** Official VM/compiler behavior; pragma contracts require their own documentation. **Official implementation/toolchain.**
- **Confidence / likely source:** Medium-high. Dart native compiler and VM documentation; pragma/API documentation.

### DART-PLAT-002 — JavaScript web compilation and development compiler differences

- **Distinct behavior:** Official web compilers translate Dart into JavaScript for production or development. Closed-world optimization, runtime type representation, numeric behavior, deferred chunks, source maps, and dynamic interoperability can differ from the VM and from each other.
- **Observable content:** A browser entry point compiled in development and production modes, using large integers, runtime type checks, deferred imports, assertions, and stack traces.
- **Variants and failures:** Unsupported `dart:io`/FFI/mirrors, CSP restrictions, minified names, tree shaking, JS exception translation, hot reload/restart development behavior, and browser API availability.
- **Constraints / authority:** Compiler names and supported modes can change; verify against the chosen SDK. **Official compilers/platform libraries.**
- **Confidence / likely source:** Medium-high. Dart web compiler and platform documentation.

### DART-PLAT-003 — WebAssembly target

- **Distinct behavior:** The WebAssembly target has its own supported library set, JavaScript interop boundary, runtime representation, browser requirements, and compiler restrictions. Code valid for JavaScript or VM may not compile or behave identically.
- **Observable content:** A web-compatible entry point compiled to WebAssembly, conditional platform selection, interop calls, exceptions, numeric edge cases, and a deliberately unsupported feature.
- **Variants and failures:** Browser Wasm garbage-collection support, fallback strategy, dynamic code/reflection restrictions, source maps, JS interop value conversion, and compiler maturity across SDK versions.
- **Constraints / authority:** Support is recent and version/browser-dependent. **Official compiler and host platform.**
- **Confidence / likely source:** Medium. Dart WebAssembly compilation documentation; supported-platform documentation.

### DART-PLAT-004 — Platform library availability and conditional API surfaces

- **Distinct behavior:** Libraries such as `dart:io`, browser libraries, `dart:ffi`, `dart:isolate`, and `dart:mirrors` are not uniformly available. Importing one can make an otherwise pure Dart library target-specific.
- **Observable content:** Target-specific leaf libraries behind a conditional facade, plus direct invalid imports compiled for the wrong target.
- **Variants and failures:** Analyzer context configured for one target while build uses another, transitive unsupported imports, package metadata claiming platforms, stubs with mismatched APIs, and tests run only on one platform.
- **Constraints / authority:** Availability is SDK/platform-defined and changes over time; some older browser libraries are being replaced by interop packages. **Official SDK/toolchain.**
- **Confidence / likely source:** High in concept, medium on current library matrix. SDK library and platform-support documentation.

### DART-PLAT-005 — Native FFI types, layouts, callbacks, and reachability

- **Distinct behavior:** `dart:ffi` maps Dart declarations to native ABI types, structures, unions, pointers, functions, and callbacks. ABI layout, ownership, symbol lookup, and callback entry points affect type correctness and runtime safety beyond ordinary Dart semantics.
- **Observable content:** Bindings for primitive and aggregate native types, a dynamic-library symbol, a Dart-to-native call, a native callback, allocation/free ownership, and platform-conditional library loading.
- **Variants and failures:** 32/64-bit layout, alignment/packing, signedness, calling convention, null pointers, use-after-free, leaf calls, callback thread/isolate restrictions, AOT entry-point retention, and generated versus handwritten bindings.
- **Constraints / authority:** Native-only, ABI- and OS-dependent; annotation/API restrictions change with SDK. **Official SDK plus host ABI.**
- **Confidence / likely source:** Medium-high. `dart:ffi` API and official FFI documentation; target ABI documentation.

### DART-PLAT-006 — JavaScript interop declarations and conversions

- **Distinct behavior:** Modern static JavaScript interop uses annotated/external declarations and constrained interop types. Calls, properties, constructors, callbacks, nullability, promise conversion, and object identity cross a boundary not modeled as ordinary Dart dynamic dispatch.
- **Observable content:** Static interop declarations, extension members exposing JS APIs, exported/callback Dart functions where supported, promise/future conversion, and conditional isolation from non-web targets.
- **Variants and failures:** `dart:js_interop` versus older interop libraries/packages, missing JS members detected only at runtime, type erasure, nullable/undefined distinctions, minification, Wasm versus JS backend, and unsupported Dart values crossing the boundary.
- **Constraints / authority:** Interop APIs changed materially in Dart 3 and continue evolving; backend restrictions are official implementation behavior. **Official SDK/toolchain and host JavaScript.**
- **Confidence / likely source:** Medium. `dart:js_interop` API and official JavaScript interop documentation.

### DART-PLAT-007 — Mirrors and reflective reachability

- **Distinct behavior:** `dart:mirrors` can discover and invoke declarations dynamically on supported VM configurations, defeating ordinary static reachability assumptions. It is unavailable on important targets and often incompatible with tree-shaken deployment.
- **Observable content:** Reflective enumeration and invocation of a declaration with no static call, plus compilation attempts on VM JIT, AOT, and web targets.
- **Variants and failures:** Private-name access, metadata reflection, generic types, minification, AOT rejection or limited support, and replacing reflection with generated registration.
- **Constraints / authority:** Platform support is deliberately limited and must be verified per compiler. **Official SDK/runtime.**
- **Confidence / likely source:** Medium-high. `dart:mirrors` API and platform/compiler documentation.

### DART-PLAT-008 — Entrypoints, tree shaking, and pragma-preserved declarations

- **Distinct behavior:** Executables begin at recognized entry points, while AOT/web compilers remove unreachable code under closed-world assumptions. Native callbacks, VM embedding, serialization registries, and other external callers may require explicit entry-point preservation pragmas.
- **Observable content:** Standard `main` variants, an indirectly reached declaration, pragma-annotated callbacks or classes, and production builds showing retained versus removed reachability.
- **Variants and failures:** Async `main`, command-line arguments, web main, isolate entry points, native callback names, reflective access, misspelled/unsupported pragmas, and development builds masking production removal.
- **Constraints / authority:** Main signature rules are language/tool-defined; individual `@pragma` meanings are implementation contracts and can change. **Language plus official compilers/runtime.**
- **Confidence / likely source:** Medium-high. Dart program-entry documentation; VM/compiler pragma documentation.

### DART-PLAT-009 — File, process, socket, and environment behavior

- **Distinct behavior:** `dart:io` APIs expose OS-dependent paths, encodings, processes, signals, sockets, standard streams, and environment variables. These affect runtime graph edges to resources and platform-specific failures even though they are not core language semantics.
- **Observable content:** Conditional native code using paths, a subprocess or socket, environment reads, and file encodings, paired with an unsupported web target.
- **Variants and failures:** Windows versus POSIX paths/signals, locale/encoding, permissions, sandboxing, symlinks, asynchronous ordering, and process exit behavior.
- **Constraints / authority:** Native platforms only; behavior combines official SDK contracts with host OS semantics. **Official SDK plus platform.**
- **Confidence / likely source:** High. `dart:io` API documentation.

## 8. Cross-feature interactions worth explicit coverage

### DART-X-001 — Sealed hierarchy plus patterns plus library boundaries

- **Distinct behavior:** Exhaustiveness depends on the sealed declaration's known subtype space and library rules, while object/type patterns bind subtype-specific data. Moving or adding declarations can change both legal subtype relationships and switch diagnostics.
- **Observable content:** A sealed generic hierarchy with direct and indirect subtypes, some in parts, and exhaustive switches using guarded and unguarded patterns.
- **Variants and failures:** Private subtype, indirect subtype outside the library where legal, missing generic instantiation case, guard mistaken for coverage, and a new enum/sealed case.
- **Constraints / authority:** Dart 3 language semantics and analyzer exhaustiveness. **Language plus official analyzer.**
- **Confidence / likely source:** High. Class-modifier and patterns documentation; Dart language specification.

### DART-X-002 — Null safety plus generics plus promotion

- **Distinct behavior:** A type parameter's bound and actual argument determine whether `T`, `T?`, and null tests promote usefully. Generic covariance, runtime checks, and inference can expose unsound assumptions hidden by `dynamic`.
- **Observable content:** Generic functions/classes with nullable and non-nullable bounds, null checks on `T`, promoted locals/fields, and calls with `Never`, `Null`, `dynamic`, and nullable concrete types.
- **Variants and failures:** `T extends Object`, `T extends Object?`, nullable type parameters, `T?` normalization, failed promotion after mutation, and casts inserted by covariance.
- **Constraints / authority:** Language-defined; obscure normalization/inference rules merit specification verification. **Language.**
- **Confidence / likely source:** Medium-high. Null-safety, generics, flow-analysis documentation; Dart language specification.

### DART-X-003 — Mixins plus modifiers plus interface conflicts

- **Distinct behavior:** Applying mixins changes superclass chains and member implementations, while Dart 3 modifiers restrict legal use and multiple interfaces impose a combined member contract.
- **Observable content:** Cross-library base/mixin/mixin-class declarations, ordered applications, `on` constraints, and conflicting generic interface members.
- **Variants and failures:** Modifier propagation, later-mixin precedence, incompatible getters/setters, private members, `super` calls, and a generated mixin-application alias if supported.
- **Constraints / authority:** Language-defined and Dart-version-sensitive. **Language.**
- **Confidence / likely source:** High. Mixins and class-modifier documentation; Dart language specification.

### DART-X-004 — Generated parts plus language versions plus analyzer exclusion

- **Distinct behavior:** A generated part shares its owner's library and privacy, but generation timing, file markers, and analyzer exclusions can make the generator, analyzer, and compiler see different source sets.
- **Observable content:** An owner referencing generated private/public declarations under explicit package and file language versions, with analysis both including and excluding the output.
- **Variants and failures:** Missing/stale output, conflicting part ownership, new syntax emitted for an old owner, generated diagnostics suppressed, and production compilation without running the generator.
- **Constraints / authority:** Combines language part rules, SDK versioning, analyzer configuration, and ecosystem build behavior. **Mixed authority.**
- **Confidence / likely source:** High. Parts and language-versioning documentation; analyzer and build-system documentation.

### DART-X-005 — Conditional sources plus package/platform compilation matrix

- **Distinct behavior:** Package resolution is shared at one level, but conditional imports, available SDK libraries, compile-time defines, and backend restrictions select distinct program graphs per target/configuration.
- **Observable content:** The same public entry library built for VM JIT, native AOT, JavaScript, and WebAssembly where available, with target implementations and configuration branches.
- **Variants and failures:** Unselected source with syntax errors, selected source with API drift, unsupported transitive imports, environment declarations confused with conditional-import keys, and analyzer target mismatch.
- **Constraints / authority:** Language conditional semantics plus official platform/compiler behavior and Pub resolution. **Mixed authority.**
- **Confidence / likely source:** Medium-high. Libraries/imports, compiler, platform, and Pub documentation.

### DART-X-006 — Dynamic invocation plus covariance plus `noSuchMethod`

- **Distinct behavior:** A statically admitted interface call may fail a covariant runtime parameter check, while a truly missing dynamic member may invoke `noSuchMethod`; these are different dispatch failures with different declaration relationships.
- **Observable content:** Calls through concrete, interface, and dynamic references to real, covariant, and missing members, including getters/setters and named arguments.
- **Variants and failures:** Wrong receiver member, wrong argument type, wrong named-argument shape, interface satisfied by forwarding, extension method unavailable dynamically, and backend-specific exception details.
- **Constraints / authority:** Language and runtime behavior. **Language plus official runtime.**
- **Confidence / likely source:** High. Dart language specification; `Object.noSuchMethod` documentation.

### DART-X-007 — Constants plus environment defines plus tree shaking

- **Distinct behavior:** Compile-time environment constants can select constant branches and make declarations unreachable to a closed-world compiler, while JIT analysis may still retain them.
- **Observable content:** Const environment values used in conditionals, object/collection constants, and target builds where otherwise valid branches or dependencies disappear.
- **Variants and failures:** Runtime environment mistaken for compile-time configuration, differing define values between generation and compilation, assertions, conditional imports, and reflection/FFI callbacks requiring preservation.
- **Constraints / authority:** Official core API/compiler behavior layered on language constants. **Language plus official toolchain.**
- **Confidence / likely source:** High. Environment-constructor APIs; compiler and constants documentation.

## Material changes across recent Dart versions

The exact release boundaries below should be checked against the selected SDK's language-version table before treating them as authoritative.

- **Dart 3.0:** sound null safety became mandatory for supported Dart 3 programs; records, patterns, switch expressions, enhanced exhaustiveness, and class modifiers became stable. Older packages can carry earlier language versions only within the SDK's supported floor.
- **Dart 3.3:** extension types became stable. Their representation and interface rules are materially different from extension methods, aliases, and wrapper classes.
- **Later Dart 3 releases:** flow analysis added promotion of eligible private final fields and improved non-promotion diagnostics. Exact eligibility and version boundary must be verified.
- **Recent Dart 3 releases:** wildcard variables broadened the treatment of `_`; null-aware collection elements and null-aware cascade syntax were expanded; newer shorthand/member-access syntax may exist after the stated 3.8 baseline. Each feature must be guarded by a per-library language version when comparing SDKs.
- **Pub evolution:** workspace resolution, local override files, native assets, and package build hooks are recent enough that minimum SDK and manifest formats must be verified.
- **Web evolution:** static JavaScript interop and WebAssembly support changed the supported types, annotations, and library surface. Older `dart:html`, `dart:js`, or package-based interop code may remain valid only on selected targets or may be discouraged rather than immediately invalid.
- **Experimental macros/augmentations:** the design and implementation path changed materially. No stable support should be assumed solely from an old proposal, experiment flag, or generated source.

## Implementation-defined, platform-dependent, or unspecified areas

- Object, closure, record, generic type, and extension-type runtime representation; allocation elision; and optimizer strategy.
- Function tear-off identity in edge cases, error object text, stack-trace shape, symbol/minified names, and source-map fidelity.
- Deferred-loading chunk boundaries and whether native targets load eagerly.
- Scheduling integration between Dart microtasks/events and browser or operating-system event loops.
- Integer precision/range allowances and low-level numeric representation across native, JavaScript, and WebAssembly backends.
- Reflection availability, tree-shaking retention, pragma interpretation, and externally invoked entry-point retention.
- Isolate implementation, supported spawn forms, message-copy optimizations, and platform-specific sendability restrictions.
- FFI ABI layout, native symbol visibility, callback threading, dynamic library naming, and memory ownership.
- JavaScript interop conversions, JavaScript `undefined` versus Dart null, promise handling, identity, and backend-specific restrictions.
- Filesystem case, URI/symlink canonicalization, paths, process/signal behavior, locale, networking, and sandbox restrictions.
- Analyzer diagnostic codes, messages, severities, recovery behavior, and whether a non-language diagnostic blocks a particular command.
- Build cache invalidation, generated-source visibility, publication filters, hosted-package policy, and availability of recent Pub/build features.

Where the language specification deliberately leaves behavior open, examples should demonstrate the allowance without asserting one backend's result as portable Dart semantics.

## Known knowledge gaps requiring primary-source verification

- The exact current stable Dart version in August 2026 and the lower language-version floor supported by that SDK.
- Exact release numbers and syntactic coverage for private-field promotion, wildcard variables, null-aware collection elements, newer cascade forms, and any dot-shorthand/member-shorthand feature after Dart 3.8.
- Current stable status and final APIs for Pub workspaces, native assets, and build hooks.
- Current status of macro and augmentation work, including which experiments were removed, renamed, or incorporated elsewhere.
- Full conditional-import environment-key set and its differences among JavaScript and WebAssembly web compilers.
- WebAssembly restrictions, browser requirements, fallback behavior, and parity with the JavaScript interop libraries.
- Precise extension-type runtime type tests, interface restrictions, erasure behavior, and backend differences.
- Edge cases in instantiate-to-bound, function subtyping involving `void`/`dynamic`, least-upper-bound inference, and `FutureOr` normalization.
- Exact `noSuchMethod` forwarding eligibility and invocation-symbol behavior for private or named members.
- Lazy top-level/late initializer behavior after re-entrant failure or thrown initialization.
- Support matrix for `dart:mirrors`, deferred imports, isolate spawning forms, and pragmas under current VM JIT, native AOT, JavaScript, and WebAssembly compilers.

## Final audit for possibly missing feature families

The checklist covers library composition and privacy; import/export selection; lexical/name resolution; null-safe and generic typing; inference and promotion; records, patterns, and sealed exhaustiveness; classes, modifiers, mixins, constructors, extensions, extension types, enums, and dynamic dispatch; closures, async behavior, generators, exceptions, isolates, and queues; expression order, constants, collections, cascades, and numeric target differences; Pub resolution/workspaces/publication; analyzer configuration and diagnostics; generated sources and common builders; native, JavaScript, WebAssembly, FFI, mirrors, and platform-library boundaries.

Possible residual families to verify before declaring completeness:

- New stable language features released after Dart 3.8, especially shorthand syntax, variance annotations, or augmentation-related constructs that change reference resolution or type identity.
- Less common declaration forms or legacy syntax still accepted by the baseline, including old function typedefs and mixin-application class aliases.
- Precise lexical edge cases involving Unicode identifiers, comments containing language-version markers, raw strings, and parser recovery, where they affect entity identity or diagnostics rather than grammar coverage alone.
- SDK-specific pragmas beyond entry-point preservation and optimizer directives that materially change reachability or dispatch.
- Flutter-only build variants, asset generation, and plugin registrant generation, if the eventual scope explicitly includes Flutter rather than Dart alone.
- Package ecosystems that synthesize runtime registries without `part` files, such as command routers, serializers, dependency injection, or test discovery, if common convention rather than language coverage becomes a goal.

No source code, project theme, fixture layout, scoring system, or analysis-tool recommendation is included.
