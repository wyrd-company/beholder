# Dart semantic coverage research checklist

## Scope and baseline assumptions

This is a candidate checklist for a broad, idiomatic, self-contained Dart project. It targets semantic distinctions that can change declared entities, identity, relationships, name and type resolution, visibility, dispatch, source membership, build output, or diagnostics. It does not prescribe a fixture structure or graph representation.

- **Baseline:** Dart SDK 3.5, with sound null safety, the Common Front End (CFE), analyzer, `dart` CLI, and Pub. Dart 3.5 is a conservative reference point, not a claim about the current latest SDK.
- **Language versioning:** each package's effective language version is derived from its SDK lower constraint unless a file has an explicit language-version comment. Dependencies may therefore use different language versions in one resolution.
- **Execution surfaces:** Dart VM ahead-of-time (AOT) and just-in-time (JIT), JavaScript compilation, and analyzer-only source are relevant. Flutter-specific framework behavior is outside the baseline, but Flutter packages can expose ordinary Dart and generated-code conditions.
- **Package assumptions:** one root package, path-hosted local dependencies where multiple packages are needed, conventional `lib/`, `bin/`, `test/`, and generated-source usage through common Pub tooling.
- **Terminology:** “library” means a Dart library, which may consist of one defining compilation unit and `part` units. It does not mean a Pub package.

## 1. Libraries, units, imports, and visibility

### DART-001 — Library identity across compilation units

- **Behavior:** a library is the privacy and namespace boundary; a file is not necessarily a library when it is a `part`.
- **Evidence:** a defining unit with two `part` directives, cross-part references, and one private declaration shared between parts.
- **Variants/failures:** standalone files, named `library` directives, duplicate declarations across parts, and a part incorrectly analyzed as an independent library.
- **Constraints/provenance:** language-defined; modern code may omit the `library` directive, but annotations on a library require one.
- **Confidence/source:** high — Dart language specification; Dart language documentation on libraries and imports.

### DART-002 — `part` / `part of` membership and URI matching

- **Behavior:** a part contributes declarations to exactly one library and has no import namespace of its own.
- **Evidence:** URI-form `part of`, name-form `part of`, and declarations referenced between defining and part units.
- **Variants/failures:** missing `part of`, mismatched library name or URI, part included by two libraries, nested `part`, imports in a part, and orphan part.
- **Constraints/provenance:** language-defined; analyzer and CFE diagnose malformed membership.
- **Confidence/source:** high — Dart language specification.

### DART-003 — Relative, package, and SDK URI imports

- **Behavior:** URI form affects library identity and resolution; the same source should not acquire two identities through inconsistent URIs.
- **Evidence:** `dart:` import, `package:` import, and relative import with cross-library references.
- **Variants/failures:** importing into `lib/src`, path normalization, percent encoding, case-sensitive filesystems, missing URI, and illegal access outside package roots.
- **Constraints/provenance:** language defines import directives; SDK and Pub define URI resolution details.
- **Confidence/source:** high — Dart language specification; Pub package layout documentation.

### DART-004 — Import prefixes

- **Behavior:** `as` creates a prefix namespace and changes how imported declarations are resolved.
- **Evidence:** two libraries exporting the same public name, both imported under distinct prefixes and used in type and expression positions.
- **Variants/failures:** prefix colliding with local declarations, same prefix reused, deferred prefix, and an unused prefix.
- **Constraints/provenance:** language-defined.
- **Confidence/source:** high — Dart language specification.

### DART-005 — Import combinators

- **Behavior:** `show` and `hide` alter the imported namespace without altering the source library.
- **Evidence:** imports using `show`, `hide`, and chained combinators with both admitted and excluded names referenced.
- **Variants/failures:** private names, nonexistent combinator names, `show`/`hide` order, prefixed imports, and name ambiguity after filtering.
- **Constraints/provenance:** language-defined; some nonexistent-name diagnostics may be analyzer warnings rather than compile errors.
- **Confidence/source:** high — Dart language specification.

### DART-006 — Exports and transitive public APIs

- **Behavior:** `export` re-exposes another library's namespace and produces references whose declaring and exposing libraries differ.
- **Evidence:** barrel library exporting two implementation libraries, including an export combinator and a transitive export.
- **Variants/failures:** export cycles, conflicting exported names, local declaration shadowing an export, private declarations not exported, and imports that do not re-export.
- **Constraints/provenance:** language-defined.
- **Confidence/source:** high — Dart language specification.

### DART-007 — Namespace conflicts and ambiguity

- **Behavior:** multiple imports or exports can introduce the same name; resolution depends on local declarations, prefixes, and combinators.
- **Evidence:** ambiguous unprefixed reference plus corrected prefixed/combinator forms.
- **Variants/failures:** same declaration reached through multiple export paths versus two distinct declarations; type/value namespace contexts; extension conflicts.
- **Constraints/provenance:** language-defined; exact diagnostic text is implementation-specific.
- **Confidence/source:** high — Dart language specification; analyzer diagnostic documentation.

### DART-008 — Underscore privacy

- **Behavior:** identifiers beginning with `_` are private to a library, including private types, members, constructors, named parameters, and extension names.
- **Evidence:** private declarations used across parts but rejected from another library; public API exposing a private type.
- **Variants/failures:** private names in metadata, private enum members, private named parameters, and public members inherited from a private superclass.
- **Constraints/provenance:** language-defined; package boundaries do not create privacy.
- **Confidence/source:** high — Dart language specification.

### DART-009 — Conditional imports and exports

- **Behavior:** environment declarations select one library implementation at compile time, changing source inclusion and resolved declaration identity.
- **Evidence:** default implementation plus `if (dart.library.io)` and `if (dart.library.js_interop)` alternatives sharing an API.
- **Variants/failures:** unavailable condition keys, incompatible branch APIs, analyzer treatment of unselected branches, and mutually true conditions whose first match wins.
- **Constraints/provenance:** language/SDK-defined; recognized `dart.library.*` values and target availability are implementation-defined.
- **Confidence/source:** high — Dart language documentation on conditional imports and exports.

### DART-010 — Configurable URI environment values

- **Behavior:** `bool.fromEnvironment`, `int.fromEnvironment`, and `String.fromEnvironment` observe compile-time environment declarations and can influence constants and tree shaking.
- **Evidence:** const values with defaults and build invocations supplying `-D` values.
- **Variants/failures:** absent keys, invalid integer/bool text, non-const calls, and different VM versus JavaScript build values.
- **Constraints/provenance:** SDK API and implementation-defined compiler configuration; consistency is only guaranteed for const use.
- **Confidence/source:** high — Dart core API documentation.

## 2. Declarations and object-model structure

### DART-011 — Classes, abstract classes, and implicit interfaces

- **Behavior:** every class defines both a class and an implicit interface; `extends` inherits implementation while `implements` requires interface conformance.
- **Evidence:** concrete and abstract classes, subclassing, interface implementation, inherited abstract members, and overridden members.
- **Variants/failures:** missing implementation, abstract instantiation, covariant override, and concrete class with inherited unresolved abstract member.
- **Constraints/provenance:** language-defined.
- **Confidence/source:** high — Dart language specification.

### DART-012 — Class modifiers (`base`, `interface`, `final`, `sealed`)

- **Behavior:** modifiers constrain where and how a type may be extended, implemented, or mixed in, and can induce restrictions on subtypes.
- **Evidence:** declarations in one library and legal/illegal subtype relations from another library for every modifier.
- **Variants/failures:** modifier combinations (`abstract base`, `abstract interface`, `sealed`), transitive `base` obligations, same-library exceptions, and `final` blocking both extension and implementation.
- **Constraints/provenance:** language-defined since Dart 3; restrictions are library-sensitive.
- **Confidence/source:** high — Dart language documentation on class modifiers; Dart language specification.

### DART-013 — Mixins and `with` application

- **Behavior:** mixin application synthesizes class behavior with ordered superclass chains and member conflict resolution.
- **Evidence:** two mixins applied in order, overridden member using `super`, fields/accessors contributed by mixins, and an explicit mixin application class.
- **Variants/failures:** duplicate members, repeated mixin, constructors not inherited from mixins, `on` constraints, and application to prohibited class modifiers.
- **Constraints/provenance:** language-defined.
- **Confidence/source:** high — Dart language specification.

### DART-014 — Mixin declarations and `on` constraints

- **Behavior:** `mixin` declares reusable implementation; `on` supplies superclass constraints and a valid `super` interface.
- **Evidence:** a mixin with multiple `on` types, `super` member calls, legal application, and rejected application.
- **Variants/failures:** `mixin class`, `base mixin`, transitive constraint satisfaction, abstract members, and conflicting constraints.
- **Constraints/provenance:** language-defined; modifier combinations changed in Dart 3.
- **Confidence/source:** high — Dart language documentation on mixins and class modifiers.

### DART-015 — Mixin classes

- **Behavior:** `mixin class` may be constructed or used as a mixin subject to restrictions that ordinary classes/mixins do not share.
- **Evidence:** one declaration used both with `new`/implicit construction and in a `with` clause.
- **Variants/failures:** generative constructors, superclass other than `Object`, `on` clauses, and modifier combinations such as `base mixin class`.
- **Constraints/provenance:** language-defined since Dart 3.
- **Confidence/source:** high — Dart language documentation on mixins and class modifiers.

### DART-016 — Extension declarations

- **Behavior:** extensions add statically resolved members without changing runtime type or interface conformance.
- **Evidence:** named and unnamed extensions, generic extension, extension methods/getters/operators, explicit extension override, and use on a nullable/non-null receiver.
- **Variants/failures:** equally specific extensions causing ambiguity, instance member taking precedence, `dynamic` receiver bypassing extension lookup, imported extension visibility, and private/unnamed extensions.
- **Constraints/provenance:** language-defined.
- **Confidence/source:** high — Dart language documentation on extension methods; Dart language specification.

### DART-017 — Extension types

- **Behavior:** an extension type has a representation type, a distinct static interface, optional implemented interfaces, and mostly representation-level runtime behavior.
- **Evidence:** generic extension type with primary constructor, representation field, members, interface implementation, casts, and representation access.
- **Variants/failures:** nullable extension types, representation type subtyping, boxing situations, member conflicts, constructor redirects, and prohibited representation cycles.
- **Constraints/provenance:** language-defined, stable from Dart 3.3; platform/interop use may add restrictions.
- **Confidence/source:** high — Dart language documentation on extension types; feature specification.

### DART-018 — Enums and enhanced enums

- **Behavior:** enums create a sealed finite class with canonical constant instances, generated `values`, `index`, and `name`; enhanced enums add fields, methods, and const constructors.
- **Evidence:** simple and enhanced enum, interface implementation, const field, method override, switch exhaustiveness, and access to generated members.
- **Variants/failures:** duplicate const arguments do not merge enum values, forbidden `values`/`index` declarations, non-final fields, generative factory restrictions, and exhaustive versus guarded cases.
- **Constraints/provenance:** language-defined; enhanced enums require Dart 2.17.
- **Confidence/source:** high — Dart language documentation on enums.

### DART-019 — Type aliases

- **Behavior:** `typedef` creates an alias for function or non-function types; aliases preserve most underlying type relations but have their own declaration identity and metadata location.
- **Evidence:** generic function-type alias, generic class-type alias, alias used in constructors, casts, bounds, and runtime type tests.
- **Variants/failures:** alias cycles, legacy function typedef syntax, instantiate-to-bound, aliases of nullable/record types, and whether runtime display preserves alias spelling.
- **Constraints/provenance:** language-defined; non-function aliases require Dart 2.13.
- **Confidence/source:** high — Dart language specification; Dart language documentation on typedefs.

### DART-020 — Top-level variables and functions

- **Behavior:** top-level declarations belong to library namespaces; variables synthesize getters and, unless final, setters, with lazy initialization semantics.
- **Evidence:** mutable, `final`, `const`, `late`, and inferred top-level variables plus functions referencing them across libraries.
- **Variants/failures:** initialization cycles, throwing initializers retried on later access, private accessors, and getter/function name conflicts.
- **Constraints/provenance:** language-defined.
- **Confidence/source:** high — Dart language specification.

### DART-021 — Instance, static, and external members

- **Behavior:** instance and static namespaces resolve differently; `external` declares an implementation supplied outside the Dart body or by augmentation/tooling.
- **Evidence:** same conceptual operation as instance/static members, static access through declaring type, external function/member with a supported implementation boundary.
- **Variants/failures:** static members are not inherited as instance members, instance access from static context, external body present, and missing external implementation.
- **Constraints/provenance:** core semantics are language-defined; external linkage is implementation/platform/tool-defined.
- **Confidence/source:** high — Dart language specification; platform interop documentation.

## 3. Fields, accessors, constructors, and initialization

### DART-022 — Fields and synthesized accessors

- **Behavior:** field declarations induce getter/setter members; final, const, late, static, and covariant fields differ in write and initialization rules.
- **Evidence:** each field kind, accesses through interface types, overrides with explicit accessors, and tear-offs where legal.
- **Variants/failures:** field/getter conflicts, final setter assignment, covariant write checks, inferred public field types, and abstract fields.
- **Constraints/provenance:** language-defined.
- **Confidence/source:** high — Dart language specification.

### DART-023 — Explicit getters and setters

- **Behavior:** getter and setter with the same base name are distinct declarations that can form one property interface and override independently.
- **Evidence:** paired accessor, getter-only and setter-only properties, asymmetric parameter/return types, and inherited accessor completion.
- **Variants/failures:** invalid setter arity/return declaration, getter-method collision, override variance, and a setter referenced using the `name=` identity.
- **Constraints/provenance:** language-defined.
- **Confidence/source:** high — Dart language specification.

### DART-024 — Generative constructors and initialization order

- **Behavior:** instance initialization follows field initializers, initializing formals/list, superclass constructor invocation, and body according to language order.
- **Evidence:** unnamed and named generative constructors with initializer list, assertions, redirect/super invocation, and observable initialization dependencies.
- **Variants/failures:** reading `this` too early, duplicate field initialization, final field omitted, redirecting constructor with body, and super-parameter interactions.
- **Constraints/provenance:** language-defined.
- **Confidence/source:** high — Dart language specification.

### DART-025 — Initializing formals and super parameters

- **Behavior:** `this.x` initializes a field; `super.x` forwards to a superclass constructor, creating parameter-to-field/constructor relationships.
- **Evidence:** positional and named initializing formals, super parameters mixed with explicit arguments, defaults, and required names.
- **Variants/failures:** use in factory constructors, duplicate super arguments, incompatible types, private super named parameters across libraries, and redirecting constructors.
- **Constraints/provenance:** language-defined; super parameters require Dart 2.17.
- **Confidence/source:** high — Dart language documentation on constructors.

### DART-026 — Named and redirecting constructors

- **Behavior:** constructors have class-qualified identities; redirecting generative constructors delegate to another constructor and do not initialize separately.
- **Evidence:** unnamed/named constructors, `this.named(...)` redirect, and cross-class factory redirect.
- **Variants/failures:** redirect cycles, const redirect consistency, type argument forwarding, target privacy, and redirect to incompatible signature.
- **Constraints/provenance:** language-defined.
- **Confidence/source:** high — Dart language specification.

### DART-027 — Factory constructors

- **Behavior:** factories may return cached instances or subtype instances and do not allocate/initialize `this` directly.
- **Evidence:** factory returning an existing instance, redirecting factory, generic factory, and interface-typed factory returning an implementation.
- **Variants/failures:** accessing `this`, return type incompatibility, const factories, external factories, and constructor tear-offs.
- **Constraints/provenance:** language-defined; external factory implementation can be platform-defined.
- **Confidence/source:** high — Dart language specification.

### DART-028 — Constructor tear-offs

- **Behavior:** referring to a constructor without invoking it yields a function object, including generic constructor instantiation syntax.
- **Evidence:** unnamed/named constructor tear-offs, redirecting factory tear-off, generic class instantiated and uninstantiated forms.
- **Variants/failures:** const context, inference of type arguments, private constructors, ambiguous parser contexts, and extension-type constructors.
- **Constraints/provenance:** language-defined; constructor tear-offs require Dart 2.15.
- **Confidence/source:** high — Dart language documentation on constructor tear-offs.

### DART-029 — `late` initialization

- **Behavior:** `late` defers initialization and can add runtime checks for reads-before-write or repeated writes to `late final`.
- **Evidence:** late local, instance, static, top-level, late-with-initializer referencing `this`, and late final assignment.
- **Variants/failures:** uninitialized read, repeated final assignment, lazy initializer side effects/throws, promotion limitations, and nullable late values.
- **Constraints/provenance:** language-defined under null safety.
- **Confidence/source:** high — Dart null-safety documentation; Dart language specification.

## 4. Type system, generics, and null safety

### DART-030 — Sound nullability and `Null`/`Never`/`Object?`

- **Behavior:** nullable and non-nullable types have distinct subtype relations; `Never` is bottom and `Object?` is top, while `Null` inhabits nullable types.
- **Evidence:** nullable declarations, assignments across the lattice, throwing expressions typed `Never`, and APIs accepting `Object?`.
- **Variants/failures:** `void`, `dynamic`, legacy types from opted-out dependencies, unreachable code, and nullable type parameters.
- **Constraints/provenance:** language-defined; sound null safety requires Dart 2.12+ and all runtime entry packages opted in.
- **Confidence/source:** high — Dart null-safety documentation; Dart language specification.

### DART-031 — Null-aware operations

- **Behavior:** `?.`, `?..`, `?[]`, `??`, `??=`, and null-shorting chains control evaluation and static nullability.
- **Evidence:** side-effecting receiver/index/right operand to show skipped evaluation, chained nullable access, and null-aware cascade.
- **Variants/failures:** unnecessary null-aware use after promotion, assignment targets, parentheses breaking shorting, and extension/member resolution on nullable receivers.
- **Constraints/provenance:** language-defined; some null-aware syntax was expanded in later 2.x releases.
- **Confidence/source:** high — Dart language documentation on operators and null safety.

### DART-032 — Null assertion and explicit casts

- **Behavior:** postfix `!` changes static type and throws on null; `as` imposes a runtime subtype check unless statically eliminated.
- **Evidence:** successful and failing `!`, nullable-to-non-null cast, generic cast, and cast affecting subsequent member resolution.
- **Variants/failures:** `dynamic`, `Never`, redundant casts, promoted values, and JavaScript representation differences in error details.
- **Constraints/provenance:** language-defined; exception class/message details are implementation-defined.
- **Confidence/source:** high — Dart language specification.

### DART-033 — Flow analysis and promotion

- **Behavior:** control flow promotes local variables and eligible fields based on tests, assignments, reachability, and captured writes.
- **Evidence:** `is`/null tests, early return, logical operators, switch patterns, private final-field promotion, and deliberate demotion.
- **Variants/failures:** public or overridden getters not promoted, write/capture invalidation, promotion failure reasons, `this` properties, and language-version changes to field promotion.
- **Constraints/provenance:** language-defined with evolving algorithms; field promotion became available in Dart 3.2.
- **Confidence/source:** high — Dart language documentation on type promotion and non-promotion reasons.

### DART-034 — Generic classes, functions, and methods

- **Behavior:** type parameters introduce scoped type identities; type arguments may be explicit or inferred at calls and constructor invocations.
- **Evidence:** nested generic scopes, generic top-level/local/instance functions, generic constructor use, and inferred versus explicit arguments.
- **Variants/failures:** shadowed type parameter names, wrong arity, raw types, generic tear-offs, and inference using context/downwards and arguments/upwards.
- **Constraints/provenance:** language-defined.
- **Confidence/source:** high — Dart language specification.

### DART-035 — Bounds and instantiate-to-bound

- **Behavior:** `extends` bounds constrain type arguments; omitted type arguments are completed using instantiate-to-bound rules, including recursive bounds.
- **Evidence:** simple and F-bounded parameter, omitted type arguments in type annotations/tear-offs, valid and invalid explicit arguments.
- **Variants/failures:** mutually recursive bounds, nullable bounds, `dynamic`/`Never`, super-bounded types, and different inference in legacy libraries.
- **Constraints/provenance:** language-defined; algorithm details changed alongside null safety.
- **Confidence/source:** medium — Dart language specification; language feature specifications.

### DART-036 — Generic covariance and runtime checks

- **Behavior:** Dart generic type parameters are covariant by default, so writes through widened generic views can require runtime checks.
- **Evidence:** `List<Sub>` viewed as `List<Base>` followed by an incompatible insertion, plus custom generic setter behavior.
- **Variants/failures:** function types use variance by position, `covariant` parameters, `dynamic` bypass, and compiler optimization preserving soundness.
- **Constraints/provenance:** language-defined; runtime error details depend on implementation.
- **Confidence/source:** high — Dart language specification.

### DART-037 — Function types and variance

- **Behavior:** function types encode return, positional/named parameter, requiredness, and generic parameters; subtyping is covariant in returns and contravariant in parameters with Dart-specific optional rules.
- **Evidence:** assignments among function types, generic function types, named/optional differences, and call through typedef/interface.
- **Variants/failures:** parameter names for named functions are semantic, positional names are not, required named changes compatibility, and `void` return has special assignability.
- **Constraints/provenance:** language-defined.
- **Confidence/source:** high — Dart language specification.

### DART-038 — `dynamic`, `Object`, and `void`

- **Behavior:** `dynamic` defers member checking to runtime, `Object` exposes only its interface, and `void` suppresses use of a value in most expression contexts.
- **Evidence:** identical receiver values typed each way, dynamic invocation success/failure, and callback returning a value where `void` is expected.
- **Variants/failures:** implicit downcasts from dynamic, dynamic generic arguments, `Function`, nullable forms, and compiler diagnostic differences.
- **Constraints/provenance:** language-defined; runtime missing-member error shape is implementation-defined.
- **Confidence/source:** high — Dart language specification.

### DART-039 — Type tests and reified generic types

- **Behavior:** `is`/`is!` perform runtime subtype tests, including most generic type arguments, and feed promotion.
- **Evidence:** tests over class hierarchies, generic instances, records, functions, aliases, and nullable values.
- **Variants/failures:** type variables, `dynamic`, erased/interop representations, extension types, and tests that are statically always true/false.
- **Constraints/provenance:** language-defined, with backend-specific representation constrained to preserve observable tests except documented interop cases.
- **Confidence/source:** high — Dart language specification.

### DART-040 — Records and record types

- **Behavior:** records are immutable structural values; shape includes positional arity and named-field names, while named-field order is not significant.
- **Evidence:** positional, named, and mixed records; destructuring; getters `$1` and named fields; type annotations and equality.
- **Variants/failures:** one-element record syntax, different named shapes, typedef aliases, generic fields, nullable records, and record-pattern shape mismatch.
- **Constraints/provenance:** language-defined since Dart 3.
- **Confidence/source:** high — Dart language documentation on records; Dart language specification.

### DART-041 — Type inference for variables, collections, and expressions

- **Behavior:** `var`, omitted generic arguments, collection literals, conditionals, and closures infer types from context and operands using constraint solving and least-upper-bound rules.
- **Evidence:** context-free and context-typed literals/closures, mixed numeric collection, empty collection, conditional expression, and inferred public member.
- **Variants/failures:** `dynamic` fallback, `Never` branches, downwards/upwards inference, analyzer `strict-inference`, and language-version-dependent inference.
- **Constraints/provenance:** core inference is language-defined; strictness lint/analyzer modes are tool-defined.
- **Confidence/source:** medium — Dart language specification; analyzer documentation.

## 5. Calls, dispatch, operators, and member resolution

### DART-042 — Positional, optional, and named parameters

- **Behavior:** call compatibility depends on positional arity, named labels, defaults, `required`, and parameter types.
- **Evidence:** functions/methods/constructors covering required positional, optional positional, named optional, and required named parameters.
- **Variants/failures:** omitted defaults, duplicate/unknown names, named ordering, private named parameters, const defaults, and override compatibility.
- **Constraints/provenance:** language-defined; non-nullable optional parameters generally need defaults or `required`.
- **Confidence/source:** high — Dart language specification.

### DART-043 — `covariant` parameters and checked dispatch

- **Behavior:** a covariant override or declaration permits narrowing an input type while inserting a runtime check at dynamically dispatched entry.
- **Evidence:** base-typed receiver invoking subclass override with both compatible and incompatible values.
- **Variants/failures:** implicit covariance from field setters, generic covariance, interface inheritance conflicts, and direct call statically rejected versus widened call failing at runtime.
- **Constraints/provenance:** language-defined.
- **Confidence/source:** high — Dart language specification.

### DART-044 — Override resolution and inheritance conflicts

- **Behavior:** class hierarchy, mixin order, declared overrides, getters/setters, and interface members determine the effective member and override relationships.
- **Evidence:** diamond-shaped interface graph, mixin-provided implementation, explicit resolution override, and `@override` metadata.
- **Variants/failures:** incompatible inherited signatures, concrete/abstract conflict, static member non-override, extension member non-override, and `noSuchMethod` satisfaction.
- **Constraints/provenance:** language-defined; `@override` checking is analyzer-supported SDK metadata.
- **Confidence/source:** high — Dart language specification; analyzer documentation.

### DART-045 — Super invocations and superclass member access

- **Behavior:** `super` bypasses ordinary virtual lookup at the current class and resolves in the superclass/mixin application chain.
- **Evidence:** method/getter/setter/operator calls through `super`, including a mixin's constrained super call.
- **Variants/failures:** abstract super member, static context, extension members, late-bound call inside closures, and multiple mixin ordering.
- **Constraints/provenance:** language-defined; abstract-super-forwarding details have evolved.
- **Confidence/source:** high — Dart language specification.

### DART-046 — Callable objects and `call`

- **Behavior:** an instance with a `call` method can be invoked with function-call syntax while retaining its class type.
- **Evidence:** generic callable class, tear-off of `call`, assignment to compatible function type, and dynamic invocation.
- **Variants/failures:** extension `call`, nullable callable, named arguments, `noSuchMethod`, and difference between object identity and method tear-off identity.
- **Constraints/provenance:** language-defined.
- **Confidence/source:** high — Dart language documentation on callable objects.

### DART-047 — Method and function tear-offs

- **Behavior:** property access to a method/function yields a closure; instance method tear-offs bind a receiver and generic tear-offs may await instantiation.
- **Evidence:** top-level, static, instance, local, generic, and extension-member tear-offs, invoked and compared where meaningful.
- **Variants/failures:** getter returning a function, callable objects, constructor tear-offs, `super` tear-offs, and backend-dependent closure identity.
- **Constraints/provenance:** language-defined except object identity/canonicalization details not guaranteed.
- **Confidence/source:** high — Dart language specification.

### DART-048 — Operator declarations and desugared dispatch

- **Behavior:** user-defined operators map syntax to instance members; compound assignment, equality, indexing, and unary operators have special evaluation/dispatch rules.
- **Evidence:** arithmetic, comparison, unary, `[]`, `[]=`, `==`, and compound assignment with side-effecting receiver/index.
- **Variants/failures:** non-overloadable operators, return-type restrictions for `==`, `!=` derived from `==`, null equality shortcut, and assignment fallback from getter/operator/setter.
- **Constraints/provenance:** language-defined.
- **Confidence/source:** high — Dart language specification; Dart language documentation on operators.

### DART-049 — Equality, identity, and `hashCode`

- **Behavior:** `==` is dynamically dispatched with null handling, while `identical` tests identity; const canonicalization and primitive backends affect observed identity.
- **Evidence:** overridden equality/hash, const equal instances, non-const equal instances, enum/record equality, and identity checks.
- **Variants/failures:** asymmetric/bad equality, mutable hash keys, NaN and signed zero, JavaScript number representation, and canonicalization boundaries.
- **Constraints/provenance:** language and core-library contracts; some primitive identity behavior is backend-defined/documented.
- **Confidence/source:** high — Dart language specification; `Object` and `identical` API documentation.

### DART-050 — `noSuchMethod` forwarding

- **Behavior:** unresolved dynamic invocations can reach `noSuchMethod`; a non-abstract override can also allow a class to satisfy some otherwise missing interface members through forwarders.
- **Evidence:** class overriding `noSuchMethod`, invocation through `dynamic`, and interface implementation without explicit bodies.
- **Variants/failures:** statically unresolved call rejected before runtime, private member names across libraries, invocation metadata, null receiver, and compiler-generated forwarders.
- **Constraints/provenance:** language-defined with VM/compiler implementation support; mirrors/call metadata can differ.
- **Confidence/source:** medium — Dart language specification; `Object.noSuchMethod` API documentation.

### DART-051 — Cascades

- **Behavior:** cascades evaluate one target once and route sections to it while the overall expression retains the target value.
- **Evidence:** method/property/index cascade, assignment, nested cascade, null-shorting cascade, and cascade within a larger expression.
- **Variants/failures:** cascade section static context, `..` versus `...` spread confusion, void-returning sections, and target promotion/nullability.
- **Constraints/provenance:** language-defined.
- **Confidence/source:** high — Dart language documentation on operators.

## 6. Control flow, patterns, and destructuring

### DART-052 — Pattern kinds and refutability

- **Behavior:** variable, wildcard, constant, relational, logical, cast, null-check/assert, list, map, record, and object patterns differ in matching, binding, and failure.
- **Evidence:** each materially distinct pattern kind in declarations, switch cases, and `if-case` where legal.
- **Variants/failures:** duplicate/inconsistent bindings across alternatives, irrefutable context using refutable pattern, getter evaluation/throws in object patterns, and static type mismatch.
- **Constraints/provenance:** language-defined since Dart 3, with later incremental pattern refinements.
- **Confidence/source:** high — Dart language documentation on patterns; Dart language specification.

### DART-053 — Pattern variable scope and assignment

- **Behavior:** pattern declarations create variables; pattern assignments update existing variables and have different irrefutability and scope rules.
- **Evidence:** destructuring `var`/`final`, typed patterns, parenthesized pattern assignment, swaps, and nested bindings.
- **Variants/failures:** assigning final variables, declaration/assignment ambiguity, partial match failure, duplicate variables, and variable scope in guards/case bodies.
- **Constraints/provenance:** language-defined since Dart 3.
- **Confidence/source:** high — Dart language documentation on patterns.

### DART-054 — Pattern switches, exhaustiveness, and guards

- **Behavior:** switch statements/expressions use pattern coverage over sealed types, enums, booleans, nullable types, and records; guards affect reachability but generally not exhaustiveness coverage.
- **Evidence:** exhaustive switch expression over sealed subtype family, guarded case, OR pattern, overlapping/unreachable case, and non-exhaustive invalid switch.
- **Variants/failures:** subtype added in same library, default/wildcard, nullable scrutinee, legacy switch syntax, and analyzer/CFE differences in diagnostic severity.
- **Constraints/provenance:** language-defined since Dart 3; exhaustiveness algorithm has evolved.
- **Confidence/source:** high — Dart language documentation on branches and patterns.

### DART-055 — Object patterns and getter-based destructuring

- **Behavior:** object patterns test a type and invoke named getters; field shorthand binds names based on getter identifiers rather than physical fields.
- **Evidence:** object pattern over class with getter, inherited getter, renamed subpattern, nested null-check, and a throwing getter.
- **Variants/failures:** missing/private getter, extension getter not used as interface member, side effects/order, generic type test, and extension type representation.
- **Constraints/provenance:** language-defined.
- **Confidence/source:** high — Dart language documentation on patterns.

### DART-056 — Collection patterns and rest elements

- **Behavior:** list/map patterns destructure collection-like values using prescribed operations, with list rest patterns capturing or discarding a middle segment.
- **Evidence:** fixed/rest list patterns, map key/value patterns, missing-key failure, typed subpatterns, and custom compatible collection objects if supported.
- **Variants/failures:** duplicate/non-constant map keys, multiple rests, length mismatch, null values versus absent keys, and getter/operator side effects.
- **Constraints/provenance:** language-defined; exact accepted matched interfaces should be checked against the current specification.
- **Confidence/source:** medium — Dart language documentation on pattern types.

### DART-057 — `if-case` and pattern `for` loops

- **Behavior:** patterns can conditionally bind in `if-case` or irrefutably destructure each iteration element in pattern-based loops.
- **Evidence:** `if (value case pattern when guard)`, `for (final (a,b) in records)`, and collection-if/for equivalents.
- **Variants/failures:** binding scope in `else`, refutable loop pattern, guard promotion, asynchronous `await for`, and nested destructuring.
- **Constraints/provenance:** language-defined since Dart 3.
- **Confidence/source:** high — Dart language documentation on patterns and loops.

### DART-058 — Traditional switch semantics and labels

- **Behavior:** statement switches, case labels, `continue label`, empty-case sharing, `break`, and fall-through restrictions form control-flow relationships distinct from pattern selection.
- **Evidence:** legacy constant cases, labeled continuation, shared body, default, and deliberate illegal fall-through.
- **Variants/failures:** switch expression versus statement, enum exhaustiveness, cases ending in throw/return, duplicate constants, and pattern cases mixed with constants.
- **Constraints/provenance:** language-defined; switch behavior changed with Dart 3 patterns.
- **Confidence/source:** high — Dart language specification.

### DART-059 — Loops, labels, break, and continue

- **Behavior:** lexical labels select loop/switch targets; `for-in` introduces iteration variables and invokes iteration protocol.
- **Evidence:** nested labeled loops, `break`/`continue`, C-style loop scopes, synchronous `for-in`, and closure capture of loop variables.
- **Variants/failures:** invalid target kind, label shadowing, mutation during iteration, per-iteration capture semantics, and `await for` cancellation.
- **Constraints/provenance:** core control flow is language-defined; iterator concurrent-modification behavior is library-specific.
- **Confidence/source:** high — Dart language specification.

### DART-060 — Exceptions, rethrow, and stack traces

- **Behavior:** any non-null object may be thrown; `try`/`on`/`catch` dispatches by runtime type, `finally` always participates in completion, and `rethrow` preserves the active exception.
- **Evidence:** typed and untyped catches, exception plus stack trace binding, rethrow, nested finally overriding return/throw, and custom exception.
- **Variants/failures:** throwing null (prohibited under current Dart), catch ordering, async errors, `Error` versus `Exception` convention, and backend-specific stack format.
- **Constraints/provenance:** language-defined control flow; stack trace contents/format and some error types are implementation-defined.
- **Confidence/source:** high — Dart language specification; Dart language documentation on error handling.

## 7. Functions, closures, async, and concurrency

### DART-061 — Lexical closures and capture

- **Behavior:** local/anonymous functions capture lexical variables by variable, not frozen value, and introduce nested declaration scopes.
- **Evidence:** mutable capture, escaping closure, loop-variable capture, nested generic/local function, and `this`/type-parameter capture.
- **Variants/failures:** late captured variable, promotion lost due to writes, recursive local function declaration form, and closure identity.
- **Constraints/provenance:** language-defined; allocation strategy is implementation-defined.
- **Confidence/source:** high — Dart language specification.

### DART-062 — Arrow, block, anonymous, and local functions

- **Behavior:** different syntactic forms create function declarations or expressions with distinct names/scopes but compatible function types.
- **Evidence:** expression-bodied member/top-level function, anonymous closure, named local function, and immediately invoked closure.
- **Variants/failures:** `async` arrow, return inference, recursive reference, statement forbidden in arrow body, and metadata on parameters/local declarations.
- **Constraints/provenance:** language-defined.
- **Confidence/source:** high — Dart language specification.

### DART-063 — `async`, `await`, and Future flattening

- **Behavior:** an async function returns a `Future`; `await` suspends and unwraps futures/future-like values according to static/runtime rules; returned futures are flattened.
- **Evidence:** async functions returning value and future, try/finally across await, awaited dynamic value, and async closure.
- **Variants/failures:** `async void`, synchronous prefix before first suspension, error propagation, `FutureOr<T>`, and invalid await outside async context.
- **Constraints/provenance:** language-defined plus `dart:async`; scheduling details use implementation/library event-loop contracts.
- **Confidence/source:** high — Dart language specification; `dart:async` API documentation.

### DART-064 — Synchronous generators

- **Behavior:** `sync*` functions lazily produce an `Iterable`, with `yield` and `yield*` controlling generator delegation and suspension.
- **Evidence:** side effects proving laziness, `yield`, `yield*`, early iteration termination, and finally cleanup.
- **Variants/failures:** return with value, yielded type mismatch, re-iteration creating new execution, thrown errors, and mutation during traversal.
- **Constraints/provenance:** language-defined plus core Iterable contracts.
- **Confidence/source:** high — Dart language specification.

### DART-065 — Asynchronous generators and streams

- **Behavior:** `async*` functions produce a `Stream`; `yield`/`yield*`, pauses, cancellation, and errors interact with asynchronous control flow.
- **Evidence:** async generator with awaits, delegated stream, cancellation triggering finally, and error emission.
- **Variants/failures:** single-subscription versus broadcast source, yield during cancellation, return value prohibition, and backpressure timing.
- **Constraints/provenance:** syntax semantics are language-defined; detailed stream scheduling/cancellation is `dart:async` library-defined.
- **Confidence/source:** high — Dart language specification; Stream API documentation.

### DART-066 — Isolates and message boundaries

- **Behavior:** isolates do not share ordinary mutable memory; spawned entry points and messages establish cross-isolate references with sendability restrictions.
- **Evidence:** `Isolate.spawn`, `ReceivePort`/`SendPort`, structured messages, error/exit ports, and a rejected unsendable value.
- **Variants/failures:** same-code versus `spawnUri`, transferable typed data, closures and native resources, platform support, isolate groups, and web behavior.
- **Constraints/provenance:** official SDK implementation/API; sendability and availability vary by runtime/backend.
- **Confidence/source:** high — `dart:isolate` API documentation; Dart concurrency documentation.

### DART-067 — Zones and asynchronous error/context propagation

- **Behavior:** zones provide dynamically scoped async context and hooks that can alter scheduling/error handling without changing lexical declarations.
- **Evidence:** `runZoned`/`runZonedGuarded`, zone values across awaits, uncaught async error handling, and overridden scheduling hook.
- **Variants/failures:** error-zone boundaries, root zone, isolate boundaries, callback registration versus execution zone, and performance/backend differences.
- **Constraints/provenance:** `dart:async` library-defined with runtime support.
- **Confidence/source:** medium — Zone API documentation.

## 8. Constants, literals, metadata, and collections

### DART-068 — Constant expressions and canonicalization

- **Behavior:** const evaluation occurs at compile time under a restricted expression set; equivalent const objects are canonicalized.
- **Evidence:** const constructors, nested const collections, implicit const contexts, identical checks, and compile-time expression references.
- **Variants/failures:** non-const call/operand, const evaluation throws, environment constants, generic const objects, and backend primitive identity nuances.
- **Constraints/provenance:** language-defined; diagnostic presentation and representation are implementation-defined.
- **Confidence/source:** high — Dart language specification.

### DART-069 — Constant constructor invariants

- **Behavior:** const generative constructors require final instance state and const-valid initialization; invocations may still be non-const when `const` is omitted outside const context.
- **Evidence:** same constructor invoked const and non-const, assertions, redirecting const constructor, and inherited fields.
- **Variants/failures:** non-final field, non-const initializer, const subclass of non-const superclass, assertion failure, and factory const redirect.
- **Constraints/provenance:** language-defined.
- **Confidence/source:** high — Dart language specification.

### DART-070 — Metadata annotations

- **Behavior:** annotations attach const objects or const constructor invocations to libraries, declarations, directives, parameters, and other supported targets.
- **Evidence:** custom annotation class used on library, class, member, parameter, import, and deprecated/override SDK annotations.
- **Variants/failures:** non-const annotation, target restrictions enforced by analyzer conventions, multiple annotations/order, and metadata on parts versus library.
- **Constraints/provenance:** metadata syntax/evaluation is language-defined; most target validation such as `@Target` is analyzer/ecosystem-defined.
- **Confidence/source:** high — Dart language specification; `package:meta` documentation.

### DART-071 — Collection literals and inferred literal type

- **Behavior:** list, set, and map literals select collection kind and infer element/key/value types from elements and context.
- **Evidence:** typed/untyped/empty literals, ambiguous `{}` (map), const literals, and mixed element types.
- **Variants/failures:** duplicate const set/map keys, mutable versus const, contextual inference, and map/set disambiguation by entries/elements.
- **Constraints/provenance:** language-defined; collection class behavior is core-library-defined.
- **Confidence/source:** high — Dart language specification.

### DART-072 — Collection `if`, `for`, and spread elements

- **Behavior:** control-flow and spread elements conditionally/iteratively contribute values during literal evaluation; `...?` skips null.
- **Evidence:** nested collection-if/for, sync iterable spread, nullable spread, map spread, and side effects demonstrating order.
- **Variants/failures:** wrong spread element type, null with non-null-aware spread, duplicate keys, pattern-based for, and const restrictions.
- **Constraints/provenance:** language-defined; collection-if/for/spread require Dart 2.3, with later pattern integration.
- **Confidence/source:** high — Dart language documentation on collections.

### DART-073 — String interpolation, symbols, and raw/multiline literals

- **Behavior:** interpolation invokes string conversion and evaluates embedded expressions; raw strings suppress escapes; symbols encode identifier-like names.
- **Evidence:** adjacent strings, interpolation with side effects/custom `toString`, raw and multiline forms, const symbol literal, and escaped identifiers.
- **Variants/failures:** malformed interpolation/escape, `$identifier` scope resolution, const interpolation restrictions, and symbol privacy/minification implications.
- **Constraints/provenance:** language-defined; Symbol utility/retention differs by backend and reflection support.
- **Confidence/source:** high — Dart language specification; core Symbol API documentation.

### DART-074 — Numeric literals and arithmetic backend differences

- **Behavior:** integer/double literals, hexadecimal/exponent forms, division operators, overflow/precision, bit operations, and equality can differ in representable range by backend.
- **Evidence:** boundary-sized integers, shifts/bitwise operations, `~/`, NaN/infinity/signed zero, and compile/run on VM and JavaScript target.
- **Variants/failures:** compile-time integer range, JS safe-integer precision, `int`/`double` runtime types, division by zero, and web BigInt interop.
- **Constraints/provenance:** language/core-library semantics with documented platform differences between native and JavaScript implementations.
- **Confidence/source:** high — Dart core numeric API documentation; Dart web numeric representation documentation.

## 9. Package resolution, analyzer, generation, and source selection

### DART-075 — Pub package identity and `pubspec.yaml`

- **Behavior:** package name, SDK constraint, dependencies, dev dependencies, dependency overrides, and workspace/resolution settings determine available libraries and versions.
- **Evidence:** root package plus local path dependency, SDK range, regular/dev dependency separation, and generated package configuration.
- **Variants/failures:** invalid name/version constraint, missing dependency, override conflict, cyclic path dependency, and package used without direct declaration.
- **Constraints/provenance:** Pub tool-defined; some workspace features require newer Dart SDK versions than the baseline.
- **Confidence/source:** high — Pubspec documentation; Pub package documentation.

### DART-076 — Version solving and lockfiles

- **Behavior:** Pub resolves a mutually compatible dependency graph; applications normally pin resolutions in `pubspec.lock`, while packages expose constraints.
- **Evidence:** transitive dependency, constrained alternatives, lockfile, and resolution change after constraint edit.
- **Variants/failures:** unsatisfiable constraints, prereleases, hosted source changes, SDK incompatibility, `dependency_overrides`, and platform-specific package availability.
- **Constraints/provenance:** Pub implementation-defined algorithm and ecosystem convention; lockfile policy differs for applications and reusable packages.
- **Confidence/source:** high — Pub versioning and package dependency documentation.

### DART-077 — Package layout and public-versus-internal import convention

- **Behavior:** only files below `lib/` are exposed through `package:` URIs; `lib/src/` is conventionally internal but not language-private.
- **Evidence:** public entry library exporting selected `lib/src` declarations, direct `package:pkg/src/...` import, and code under `bin/`/`test/`.
- **Variants/failures:** importing another package outside `lib`, relative imports crossing roots, absent public barrel, and analyzer lints against implementation imports.
- **Constraints/provenance:** Pub layout/tool constraints plus ecosystem convention; `src` privacy is not language-enforced privacy.
- **Confidence/source:** high — Pub package layout conventions.

### DART-078 — Per-library language-version selection

- **Behavior:** the package SDK lower bound establishes a default language version, and a leading `// @dart=` marker can select an older supported version per library.
- **Evidence:** two libraries in one package with different versions and syntax/semantics whose availability differs.
- **Variants/failures:** marker placement/format, version newer than SDK or package default, parts with inconsistent markers, opted-out legacy null safety, and dependency libraries at other versions.
- **Constraints/provenance:** Dart toolchain-defined and language-feature-gating behavior; supported old versions are bounded by SDK policy.
- **Confidence/source:** high — Dart language versioning documentation.

### DART-079 — Analyzer configuration and severity

- **Behavior:** `analysis_options.yaml` can include shared configs, enable language analyzer modes, exclude files, activate plugins, and change diagnostic severities/lints.
- **Evidence:** included lint set, one overridden severity, excludes, `strict-casts`/`strict-inference`/`strict-raw-types`, and analyzer output changes.
- **Variants/failures:** include cycle/missing include, package include resolution, analyzer versus compiler diagnostics, deprecated options, and nested package boundaries.
- **Constraints/provenance:** analyzer tool-defined; lints often come from SDK or ecosystem packages.
- **Confidence/source:** high — Dart analyzer configuration documentation.

### DART-080 — Compiler errors, analyzer errors, warnings, and lints

- **Behavior:** invalid source can be rejected by both CFE and analyzer, while static warnings/lints may not prevent execution; diagnostic codes and recovery graphs differ.
- **Evidence:** one syntax error, compile-time semantic error, static warning, lint, and legal runtime type failure in isolated files/targets.
- **Variants/failures:** cascading diagnostics, error suppression, fatal-warnings flags, unreachable/dead code, and backend-only errors.
- **Constraints/provenance:** error conditions are often language-defined; code, wording, severity, recovery, and lint are tool-defined.
- **Confidence/source:** high — Dart language specification; analyzer diagnostic documentation; `dart analyze` documentation.

### DART-081 — Generated sources and build graph

- **Behavior:** ecosystem builders consume annotated/configured inputs and emit Dart assets, usually `*.g.dart`, whose declarations and `part` membership become ordinary source semantics after generation.
- **Evidence:** builder dependency/config, annotated input with `part` directive, generated `part of` output, and code referencing emitted declarations.
- **Variants/failures:** stale/missing output, conflicting outputs, builder ordering, source-gen errors, checked-in versus ephemeral output, and analyzer before generation.
- **Constraints/provenance:** `build_runner`, `build`, and generator-package conventions; generated Dart itself is language-defined.
- **Confidence/source:** high — `build_runner`, `build`, and `source_gen` package documentation.

### DART-082 — Code generation through `dart run` and snapshots

- **Behavior:** executable package entry points can transform files before analysis/build; Pub resolves the generator separately from its emitted program, and VM snapshots may execute compiled entry points.
- **Evidence:** dev dependency exposing a `bin` command, declared input/output contract, generated Dart consumed by main sources, and repeatable invocation.
- **Variants/failures:** generator version drift, non-hermetic environment inputs, executable name resolution, snapshot invalidation, and generated file absent on clean checkout.
- **Constraints/provenance:** Dart CLI/Pub implementation plus ecosystem convention.
- **Confidence/source:** medium — Dart CLI and Pub executable-package documentation.

### DART-083 — Tests as separately resolved entry libraries

- **Behavior:** each test file is an entry library with dev dependencies and may use annotations/reflection-like registration from `package:test`; test-only code still participates in static resolution.
- **Evidence:** unit test importing public and internal APIs, setup/group/test closures, async test, and test-only support library.
- **Variants/failures:** skipped/tags/platform selectors, compile-time test failure, timeout, generated tests, and browser versus VM test platform.
- **Constraints/provenance:** `package:test` ecosystem convention/tool behavior; ordinary imports and closures are language-defined.
- **Confidence/source:** high — `package:test` documentation.

### DART-084 — Formatting and machine-readable static output

- **Behavior:** formatter does not change semantics but language-version parsing can change whether source is accepted/formatted; analyzer/compiler may expose machine-readable diagnostics consumed by tooling.
- **Evidence:** source requiring current syntax, format check, analyzer output, and excluded/generated files.
- **Variants/failures:** formatter version changes, syntax errors preventing format, trailing commas affecting style, and diagnostic location recovery.
- **Constraints/provenance:** `dart format`/analyzer implementation-defined; no semantic entity should depend on formatting.
- **Confidence/source:** high — Dart CLI formatter/analyzer documentation.

## 10. Platforms, interoperation, and target-specific semantics

### DART-085 — SDK library availability by platform

- **Behavior:** `dart:io`, `dart:html`, `dart:ffi`, `dart:js_interop`, and other SDK libraries are not uniformly available across VM/native/web targets.
- **Evidence:** platform-specific libraries behind conditional imports and separate target entry points.
- **Variants/failures:** analyzer accepts source but target compiler rejects unavailable library, deprecated web libraries, test platform, and transitive import of unavailable library.
- **Constraints/provenance:** official SDK implementation/platform contract.
- **Confidence/source:** high — Dart SDK library documentation and platform documentation.

### DART-086 — Native FFI declarations and ABI types

- **Behavior:** `dart:ffi` maps Dart declarations to native symbols, native function signatures, pointers, structs/unions, allocators, and application binary interface (ABI)-dependent layouts.
- **Evidence:** dynamic library lookup, native/Dart function typedef pair, `lookupFunction`, struct fields, pointer ownership, callback, and `@Native` external binding if supported.
- **Variants/failures:** unsupported signature/type, symbol missing, calling convention, ABI-specific integer widths/alignment, leaf calls, callback threading, resource lifetime, and AOT restrictions.
- **Constraints/provenance:** official SDK implementation/API; available only on native targets and constrained by OS/architecture ABI.
- **Confidence/source:** high — `dart:ffi` API documentation; Dart FFI documentation.

### DART-087 — JavaScript static interop

- **Behavior:** `dart:js_interop` external extension types and annotations bind Dart members to JavaScript properties/functions with representation and conversion restrictions unlike normal Dart dispatch.
- **Evidence:** annotated external library/type/member, JS object literal/constructor/function call, extension members, export to JavaScript, and explicit conversion APIs.
- **Variants/failures:** nullable values, `undefined` versus null, promise/future conversion, generic restrictions, unsupported Dart values, minification, wasm versus JavaScript backend, and missing JS member.
- **Constraints/provenance:** official Dart web implementation/API; modern static interop varies by minimum SDK and backend.
- **Confidence/source:** medium — Dart JavaScript interop documentation; `dart:js_interop` API documentation.

### DART-088 — Legacy JavaScript and browser interop surfaces

- **Behavior:** older `dart:html`, `dart:js`, `package:js`, and related APIs create different external declarations and target constraints from static interop.
- **Evidence:** only if compatibility coverage is required, a dependency or library using one legacy surface alongside a modern wrapper boundary.
- **Variants/failures:** deprecated APIs, JS compiler only, incompatible annotations, migration to extension-type interop, and analyzer deprecation diagnostics.
- **Constraints/provenance:** official SDK plus ecosystem packages; availability/deprecation depends on SDK release.
- **Confidence/source:** medium — Dart web interop migration documentation and relevant API docs.

### DART-089 — Native assets and package hooks

- **Behavior:** newer Dart tooling can build/bundle native assets for packages and make them available to FFI bindings; hook output changes the build graph beyond Dart source.
- **Evidence:** package hook/configuration declaring a native build asset and a Dart binding that resolves it.
- **Variants/failures:** unsupported SDK, host/target cross-compilation, asset naming collision, missing toolchain, cached stale asset, and platform filtering.
- **Constraints/provenance:** evolving official Dart/Pub native-assets tooling; support level and file contracts are version-dependent.
- **Confidence/source:** low — Dart native assets documentation and SDK repository specifications should be verified.

### DART-090 — VM pragmas and implementation-specific annotations

- **Behavior:** annotations such as `@pragma(...)` can affect entry-point retention, inlining, exact-result assumptions, or other compiler behavior without changing ordinary language semantics.
- **Evidence:** recognized entry-point pragma on otherwise tree-shakable declarations and a harmless unknown pragma for contrast.
- **Variants/failures:** VM-only names, compiler/version changes, malformed options, AOT versus JIT, web compiler pragmas, and reflection/native callback reachability.
- **Constraints/provenance:** official implementation-defined; most pragma names are not language contracts.
- **Confidence/source:** medium — Dart SDK `pragma` API documentation and compiler/VM documentation.

### DART-091 — Tree shaking and reflective/dynamic reachability

- **Behavior:** AOT/web compilers may omit unreachable declarations, while dynamic invocations, interop exports, native callbacks, and pragmas can alter retained code.
- **Evidence:** release compilation with unused declarations, dynamically reachable entry point, interop/native callback, and size/symbol observation.
- **Variants/failures:** JIT retains more code, `dart:mirrors` availability, constructor names, deferred loading, and compiler conservatism.
- **Constraints/provenance:** official compiler implementation behavior; observable program semantics must be preserved, but retained symbol set is not language-defined.
- **Confidence/source:** medium — Dart compiler documentation; VM entry-point pragma documentation.

### DART-092 — Deferred imports and loading

- **Behavior:** `deferred as` separates a library's load unit; members are accessed through a prefix after `loadLibrary`, which is an implicitly available operation.
- **Evidence:** deferred prefix, awaited `loadLibrary`, then constructor/static/top-level access from the deferred library.
- **Variants/failures:** access before loading, repeated loads, imports of same library deferred and immediate, const references, platform/backend load-unit support, and transitive dependencies.
- **Constraints/provenance:** language-defined surface with compiler/backend-defined loading behavior; most useful on web builds.
- **Confidence/source:** high — Dart language documentation on deferred loading; compiler documentation.

### DART-093 — Reflection and mirrors

- **Behavior:** `dart:mirrors` can discover and invoke declarations dynamically on supported VM configurations, creating reachability and identity not visible in static call sites.
- **Evidence:** VM-only reflective lookup/invocation isolated behind a platform boundary if included.
- **Variants/failures:** unavailable on web/Flutter AOT, private symbol access/library mirrors, minification, metadata discovery, and tree shaking.
- **Constraints/provenance:** official SDK library but platform-limited and discouraged in many ecosystems.
- **Confidence/source:** high — `dart:mirrors` API documentation.

## 11. Intentionally invalid and edge-condition corpus

### DART-094 — Syntax recovery and incomplete source

- **Behavior:** parsers/analyzers recover from malformed or incomplete syntax and may still expose surrounding declarations/references with synthetic nodes.
- **Evidence:** separate invalid units with missing delimiter, incomplete declaration, malformed type argument list, and valid declarations before/after the error.
- **Variants/failures:** cascading diagnostics, part files, new syntax under old language version, and analyzer versus compiler recovery.
- **Constraints/provenance:** grammar errors are language-related; recovery nodes and continued semantic model are tool implementation-defined.
- **Confidence/source:** high — analyzer implementation/diagnostic documentation; Dart grammar.

### DART-095 — Unresolved, ambiguous, and inaccessible names

- **Behavior:** failures differ between no declaration found, multiple declarations found, and a known private/inaccessible declaration.
- **Evidence:** isolated examples of each at top level, type position, member access, constructor, prefix, and extension invocation.
- **Variants/failures:** typo suggestions, dynamic receiver deferring failure, import combinators, generated source absent, and conditional branch mismatch.
- **Constraints/provenance:** core validity is language-defined; diagnostic classification/recovery is analyzer/CFE-defined.
- **Confidence/source:** high — Dart language specification; analyzer diagnostic documentation.

### DART-096 — Invalid override and inheritance combinations

- **Behavior:** subtype relationships may exist syntactically while inherited member signatures, modifiers, mixin constraints, or constructors make the declaration invalid.
- **Evidence:** incompatible return/parameter override, missing interface member, conflicting inherited members, illegal external subtype of modified class, and unsatisfied mixin `on` constraint.
- **Variants/failures:** getter/setter pairs, covariant escape hatch, noSuchMethod forwarders, private members, generic substitution, and abstract classes.
- **Constraints/provenance:** language-defined; analyzer may report several diagnostics per root cause.
- **Confidence/source:** high — Dart language specification.

### DART-097 — Invalid type arguments and inference failures

- **Behavior:** wrong type-argument arity, bound failure, non-type used as a type, and constraint-solving failure are distinct resolution/type errors.
- **Evidence:** each failure on class, alias, generic function, constructor tear-off, extension, and pattern type.
- **Variants/failures:** inferred versus explicit failure, raw type accepted, instantiate-to-bound, cyclic aliases, and `dynamic` masking constraints.
- **Constraints/provenance:** language-defined with analyzer/CFE diagnostic differences.
- **Confidence/source:** high — Dart language specification; analyzer diagnostic documentation.

### DART-098 — Compile-time constant failures

- **Behavior:** a syntactically const context can fail because an operand is non-constant, evaluation throws, constructor is not const, type argument is invalid, or collection contents conflict.
- **Evidence:** separate cases for each root cause plus a valid near-neighbor.
- **Variants/failures:** implicit const context, environment values, assertion failure, redirect chain, cyclic evaluation, and duplicate constant collection keys.
- **Constraints/provenance:** language-defined; evaluation limits and diagnostic details may be implementation-defined.
- **Confidence/source:** high — Dart language specification.

### DART-099 — Runtime type, initialization, and dispatch failures

- **Behavior:** legal source can fail at runtime through cast/check failure, null assertion, late initialization, dynamic no-such-method, concurrent modification, or platform interop.
- **Evidence:** isolated executable cases for type cast, covariant check, `late` read/write, dynamic missing member, and one target-specific failure.
- **Variants/failures:** AOT/JIT/web error class/message/stack, caught versus uncaught, async error channel, and optimizer effects.
- **Constraints/provenance:** triggering conditions are language/core-library/platform contracts; diagnostic representation is implementation-defined.
- **Confidence/source:** high — Dart language specification and core API documentation.

### DART-100 — Cycles across declarations, libraries, and initialization

- **Behavior:** some cycles are legal (imports, types, recursive functions), some are rejected (aliases, redirects, constants), and some fail only when lazy initialization is evaluated.
- **Evidence:** legal import cycle, mutually recursive classes/functions, illegal typedef/constructor redirect/const cycle, and top-level initialization cycle.
- **Variants/failures:** cycle through exports/parts, generic bounds, late variables breaking cycles, conditional imports, and generated sources.
- **Constraints/provenance:** language-defined categories with tool-defined cycle reporting.
- **Confidence/source:** medium — Dart language specification; analyzer diagnostic documentation.

## Material recent-version changes to verify

The following changes materially affect what source is valid or what semantic relationships exist. Exact patch/minor availability should be checked against the selected SDK release notes and feature specifications.

- **Dart 2.12:** sound null safety, nullable type syntax, `late`, required named parameters, and flow analysis.
- **Dart 2.13–2.19:** non-function type aliases; constructor tear-offs; enhanced enums; super parameters; improved enum/mixin behavior; language-version and null-safety refinements.
- **Dart 3.0:** records, patterns, switch expressions, exhaustiveness, sealed classes, and the class-modifier system. Dart 3 also removed support for running unsound/null-safety-opted-out applications, while old-version dependency libraries remain a compatibility concern subject to SDK support.
- **Dart 3.2:** promotion of eligible private final fields and related flow-analysis changes.
- **Dart 3.3:** extension types and modernized JavaScript interop surface.
- **After baseline:** Pub workspaces, native-assets hooks, augmentation/macros experiments, analyzer options, JS/Wasm interop, and language-feature refinements may differ materially. Treat them as version-gated until verified against the chosen current SDK.

## Implementation-defined, platform-defined, or unspecified areas

- Exact diagnostic wording, number, order, recovery nodes, synthetic elements, warning severity, and analyzer-versus-CFE differences.
- Runtime type/error object details, stack trace format, source-map naming, closure/tear-off identity, allocation, inlining, and optimization.
- Integer representation and some identity behavior across native, JavaScript, and WebAssembly backends.
- Isolate availability, spawn behavior, message sendability edge cases, and scheduling details across runtimes.
- Deferred-load partitioning, tree-shaken symbol retention, code size, compilation artifacts, and pragma effects.
- Native ABI layout, symbol lookup, asset building, callback threading, and ownership/lifetime outside managed Dart.
- JavaScript representation, `undefined`/null conversion, exported name retention, and unsupported interop shapes.
- Filesystem URI case/symlink normalization, host OS behavior, package cache location, and network-backed Pub resolution.
- Generator ordering/cache behavior and generated-output lifecycle where ecosystem tools, not the language, own the contract.

## Known uncertainty and verification priorities

- The exact current status and stable syntax of native assets, build hooks, augmentations, and any macro replacement after Dart 3.5 is low-confidence.
- Static JavaScript interop and WebAssembly restrictions have changed quickly; verify supported annotations, extension-type forms, conversions, and target matrix.
- Fine points of instantiate-to-bound, super-bounded types, pattern collection matching, abstract-super forwarding, and `noSuchMethod` forwarder synthesis should be checked in the normative specification/feature specs.
- Exact language-version gates for minor null-aware, pattern, mixin, and analyzer-flow improvements need release-note verification.
- Flutter-specific source generation, asset/plugin registration, platform channels, and kernel transformations are intentionally not expanded because they are framework/build-system behavior rather than baseline Dart. A project whose stated scope includes Flutter would need a separate framework checklist.

## Final audit for possibly missing feature families

The checklist covers library composition, privacy, imports/exports, declarations, class modifiers, inheritance/mixins/extensions, constructors and initialization, generics/nullability/inference, dispatch/operators, records/patterns/control flow, closures/async/generators/isolates, constants/metadata/collections, Pub/analyzer/generation, native/web interop, backend selection, and invalid-program recovery.

Potential gaps to re-check before declaring coverage complete:

- newer-than-baseline language experiments or stabilized features, especially augmentation-related syntax;
- WebAssembly-specific interop and runtime type/number distinctions;
- obscure grammar/semantic interactions involving symbols, comments that set language versions, or legacy libraries;
- SDK internal annotations and compiler pragmas that are materially used by ordinary ecosystem packages;
- package workspace and hook behavior introduced after Dart 3.5;
- target-specific entry-point and snapshot constraints for command-line, server, web, and embedded/native builds.

These gaps are verification flags, not recommendations to include experimental or framework-specific material without a stable, reproducible toolchain contract.
