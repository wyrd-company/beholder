# C# Semantic Coverage Research Checklist

## Scope and baseline assumptions

This report identifies candidate coverage for a broad, idiomatic, self-contained C# project whose source and build conditions expose materially distinct semantic facts. It does not prescribe a project layout, graph model, scoring method, or analysis implementation.

- **Baseline language:** C# 13, with selected earlier-version and preview/version-gated cases called out explicitly.
- **Baseline implementation:** Microsoft Roslyn compiler as shipped with the .NET 9 SDK.
- **Baseline runtime:** .NET 9 on CoreCLR, using an SDK-style project and PackageReference where packages are needed.
- **Compatibility assumptions:** The checklist also covers behavior relevant to .NET Standard, .NET Framework, alternate target frameworks, and non-Microsoft CLI implementations where that changes compilation, reference resolution, metadata, or runtime behavior.
- **Compilation model:** A C# compilation consumes source trees, metadata references, analyzer/configuration inputs, parse options, compilation options, and generated source. Assembly identity and target framework reference assemblies are part of the meaning of the program.
- **Authority labels:** “Language” means C# specification or adopted feature specification; “implementation” means Roslyn, MSBuild, .NET SDK, CLR, or platform behavior; “ecosystem” means a widely used convention rather than a language guarantee.

## 1. Source text, declarations, and identity

### CS-SRC-001 — Source text and lexical edge cases

- **Distinct behavior:** Unicode identifiers, escaped identifiers, contextual keywords, verbatim identifiers, comments, preprocessor-disabled text, raw/verbatim/interpolated strings, UTF-8 literals, and numeric literal forms can make tokenization differ from appearance.
- **Observable content:** Declarations and references using Unicode and `@` identifiers; a contextual keyword used as an identifier; raw and interpolated raw strings with differing dollar/quote counts; `u8` string data; literals with separators and checked range edges.
- **Variants and failures:** Unicode escape normalization versus spelling; invalid escape sequences; unterminated raw strings; keyword changes under another language version; interpolation brace-count errors; identical-looking but distinct Unicode code points.
- **Constraints:** Raw strings and UTF-8 literals require C# 11; contextual-keyword status is language-version dependent.
- **Authority:** Language.
- **Confidence:** High.
- **Likely primary source:** C# language specification lexical structure; Microsoft C# feature specifications for raw string and UTF-8 literals.

### CS-SRC-002 — Preprocessor symbols and source mapping

- **Distinct behavior:** `#if`, `#define`, `#undef`, `#nullable`, `#pragma warning`, `#line`, and diagnostic directives change source inclusion, nullable context, diagnostics, and reported locations without being runtime control flow.
- **Observable content:** Mutually exclusive declarations under build constants; hidden and remapped lines; warning suppression/restoration; `#error` and `#warning`; local nullable-context transitions.
- **Variants and failures:** Duplicate types only in one symbol set; inactive malformed-looking code; warning IDs versus categories; generated-code line mapping; symbols supplied by project rather than source.
- **Constraints:** Symbols and available directives vary by compiler/language version; `#line` enhanced forms are version-gated.
- **Authority:** Language plus compiler implementation for diagnostic IDs and mapped locations.
- **Confidence:** High.
- **Likely primary source:** C# language specification pre-processing directives; Roslyn compiler documentation.

### CS-SRC-003 — Compilation units, top-level statements, and synthesized entry point

- **Distinct behavior:** Top-level statements synthesize a containing program and entry point, constrain declaration ordering and multiplicity, and interact with explicit `Main` methods and generated members.
- **Observable content:** One compilation unit with top-level statements plus namespace/type declarations; an explicit `Main`; top-level local functions, variables, `await`, and returned exit code.
- **Variants and failures:** Multiple files with top-level statements; explicit entry point ignored with a warning; executable versus library output; generated type/member names are implementation details.
- **Constraints:** Top-level statements require C# 9; only one compilation unit may contain them.
- **Authority:** Language, with synthesized metadata naming implementation-defined.
- **Confidence:** High.
- **Likely primary source:** Microsoft C# feature specification for top-level statements.

### CS-SRC-004 — Namespace forms and using scope

- **Distinct behavior:** Block-scoped and file-scoped namespaces produce the same namespace identity but different lexical using scope; namespaces are open across files and assemblies.
- **Observable content:** The same namespace contributed by several files; nested namespaces; file-scoped namespace with usings before and after it; global-namespace types.
- **Variants and failures:** Mixing file-scoped and block-scoped namespace declarations in one compilation unit; namespace/type name collisions; namespace aliases shadowing roots.
- **Constraints:** File-scoped namespaces require C# 10.
- **Authority:** Language.
- **Confidence:** High.
- **Likely primary source:** C# language specification namespaces; file-scoped namespace feature specification.

### CS-SRC-005 — Partial type identity across files

- **Distinct behavior:** Partial declarations merge into one type while attributes, base interfaces, constraints, members, and nested declarations accumulate across parts.
- **Observable content:** A partial class/struct/interface/record split across files, with members and attributes on different parts and a nested partial type.
- **Variants and failures:** Conflicting accessibility, base class, type parameter names, constraints, record kind, or modifiers; generated and handwritten parts; declaration order is generally not semantic.
- **Constraints:** All parts must be in one assembly/module and match arity and containing identity.
- **Authority:** Language.
- **Confidence:** High.
- **Likely primary source:** C# language specification partial declarations.

### CS-SRC-006 — Partial methods, properties, and indexers

- **Distinct behavior:** Defining and implementing declarations combine into one member; optional partial methods may be erased when unimplemented, while members with observable signatures require implementations.
- **Observable content:** Implemented and unimplemented partial methods; partial members split between handwritten and generated files; calls and attributes on both parts.
- **Variants and failures:** Accessibility, return type, `out`, virtuality, and non-void rules; signature mismatch; duplicate implementations; partial properties/indexers in C# 13.
- **Constraints:** Extended partial methods require C# 9; partial properties and indexers require C# 13.
- **Authority:** Language.
- **Confidence:** High.
- **Likely primary source:** C# partial method and C# 13 partial member feature specifications.

### CS-SRC-007 — File-local types

- **Distinct behavior:** A `file` type is visible only within its declaring source file and has a compiler-generated metadata name that avoids cross-file identity collisions.
- **Observable content:** Same simple file-local type name in two files; use within each file; a non-file type with the same simple name.
- **Variants and failures:** Exposure through public member signatures; use from another file; nesting/modifier restrictions; generated metadata names should not be treated as source identity.
- **Constraints:** Requires C# 11.
- **Authority:** Language, with metadata name mangling implementation-defined.
- **Confidence:** High.
- **Likely primary source:** Microsoft C# feature specification for file-local types.

### CS-SRC-008 — Nested types, arity, and constructed identity

- **Distinct behavior:** Nested type identity includes containing type identity and generic arity; nested types have their own accessibility and may capture containing type parameters.
- **Observable content:** Generic outer and inner types; same simple name at namespace and nested scopes; nested types with differing arity and accessibility.
- **Variants and failures:** Open versus constructed nested types; reflection metadata `+` naming; inaccessible containing type limiting effective exposure.
- **Constraints:** CLI metadata represents nested types and generic parameters independently of C# display syntax.
- **Authority:** Language plus ECMA-335 metadata.
- **Confidence:** High.
- **Likely primary source:** C# language specification types and declarations; ECMA-335.

## 2. Type forms and type relationships

### CS-TYP-001 — Classes, structs, interfaces, enums, and delegates

- **Distinct behavior:** Fundamental declared type kinds differ in inheritance, allocation/copy semantics, permitted members, default values, boxing, and callable identity.
- **Observable content:** Each declared kind with conversions and representative members; enum with explicit underlying type and aliases; multicast delegate invocation.
- **Variants and failures:** Struct boxing/copy mutation; enum duplicate values and invalid constant range; delegate variance and method-group conversion; class single inheritance versus interface multiple inheritance.
- **Constraints:** Runtime representation follows CLI rules; newer language versions allow more interface and struct members.
- **Authority:** Language and CLI.
- **Confidence:** High.
- **Likely primary source:** C# language specification type system; ECMA-335.

### CS-TYP-002 — Records and synthesized members

- **Distinct behavior:** Record class and record struct declarations synthesize equality, printing, cloning/copying, positional properties, deconstruction, and inheritance-related members.
- **Observable content:** Positional and nominal records; record inheritance; `with` expressions; user-declared member replacing a synthesized member; readonly record struct.
- **Variants and failures:** Equality contract across derived records; shallow copy; mutable record struct; primary-constructor parameter/property collisions; forbidden class/record inheritance mixing.
- **Constraints:** Record classes require C# 9; record structs require C# 10.
- **Authority:** Language, with some synthesized naming/metadata details implemented by Roslyn.
- **Confidence:** High.
- **Likely primary source:** Microsoft C# record feature specifications.

### CS-TYP-003 — Primary constructors

- **Distinct behavior:** Primary constructor parameters are in scope throughout a type body, may be captured into hidden storage, and do not automatically become properties except for positional records.
- **Observable content:** Class and struct primary constructors; parameters used in field initializers, methods, base arguments, and record declarations; a parameter shadowed by a member.
- **Variants and failures:** Storage elided versus captured; duplicated storage when also assigned to a field; constructor chaining requirements; struct initialization warnings; attributes targeted to parameters or properties.
- **Constraints:** Non-record primary constructors require C# 12.
- **Authority:** Language, with hidden storage names implementation-defined.
- **Confidence:** High.
- **Likely primary source:** Microsoft C# 12 primary constructors feature specification.

### CS-TYP-004 — Arrays and runtime array covariance

- **Distinct behavior:** Single-, multidimensional-, and jagged arrays have distinct types; reference-type arrays are covariant with runtime store checks.
- **Observable content:** All array shapes; covariance assignment followed by a failing store; array creation with inferred and explicit bounds; `System.Array` use.
- **Variants and failures:** Zero-based rectangular arrays versus non-zero-bound arrays created by reflection; element nullability; stack allocation is not an array conversion in every context.
- **Constraints:** Array covariance and exceptions are CLR behavior reflected by C#.
- **Authority:** Language and CLI/runtime.
- **Confidence:** High.
- **Likely primary source:** C# language specification arrays; ECMA-335 array types.

### CS-TYP-005 — Nullable value types and lifted operators

- **Distinct behavior:** `T?` for non-nullable value `T` is `Nullable<T>`, with lifted conversions/operators and special boxing behavior.
- **Observable content:** Nullable arithmetic/comparison, null coalescing, `bool?` logical operators, pattern matching, boxing of null and non-null values.
- **Variants and failures:** Lifted-to-null versus lifted returning non-nullable `bool`; `GetValueOrDefault`; nested nullable prohibition; generic `T?` meaning depends on constraints/context.
- **Constraints:** Core language and runtime library support.
- **Authority:** Language and runtime.
- **Confidence:** High.
- **Likely primary source:** C# language specification nullable value types and lifted operators.

### CS-TYP-006 — Nullable reference type annotations and flow state

- **Distinct behavior:** Reference nullability is primarily compile-time annotation and flow analysis, not a distinct CLR type; context and attributes alter warnings and public metadata.
- **Observable content:** Nullable enabled/disabled scopes; annotated and oblivious APIs; assignments, dereferences, null tests, guards, and generic nullability; nullability attributes.
- **Variants and failures:** Override/interface mismatch; `!` suppression changes warnings but not value; unconstrained `T?`; constructor member initialization; interop with unannotated metadata; warnings-as-errors.
- **Constraints:** Requires C# 8 or later and suitable annotations; analysis has evolved across compiler versions.
- **Authority:** Language feature plus Roslyn flow-analysis implementation and runtime attributes.
- **Confidence:** High.
- **Likely primary source:** Microsoft C# nullable reference types specification and compiler documentation.

### CS-TYP-007 — Tuples and tuple element names

- **Distinct behavior:** Tuple syntax maps to `System.ValueTuple` constructions; element names affect source binding and metadata attributes but generally not runtime type identity.
- **Observable content:** Named and unnamed tuples; assignment with name warnings; deconstruction; nested eight-or-more-element tuple; tuple conversion.
- **Variants and failures:** Name inference, erased names in identity, dynamic/nullability annotations on elements, malformed metadata, comparison and equality.
- **Constraints:** Requires C# 7 and compatible `ValueTuple` types/reference assemblies.
- **Authority:** Language plus library/metadata encoding.
- **Confidence:** High.
- **Likely primary source:** C# tuple feature specification; ECMA-335 custom attribute encoding.

### CS-TYP-008 — Anonymous types

- **Distinct behavior:** Anonymous object creation synthesizes internal immutable structural types whose identity is shared by shape within a compilation but unavailable by source name.
- **Observable content:** Repeated identical shapes, reordered/differently named shapes, inferred member names, equality and projection.
- **Variants and failures:** Cross-assembly identity differs; property type/nullability changes shape; anonymous types escaping through `object` or inference.
- **Constraints:** Compiler-generated implementation detail subject to observable language guarantees.
- **Authority:** Language with naming and sharing strategy implementation-defined.
- **Confidence:** High.
- **Likely primary source:** C# language specification anonymous object initializers.

### CS-TYP-009 — `dynamic` binding

- **Distinct behavior:** `dynamic` is encoded largely as `object` plus metadata annotations, deferring member, conversion, overload, and operator binding to runtime.
- **Observable content:** Dynamic invocation, member access, indexing, conversion, overload selection, and propagation; dynamic nested in generics/tuples.
- **Variants and failures:** Runtime binder exceptions; extension methods not discovered dynamically as extension methods; explicit interface members; missing Microsoft.CSharp/runtime binder support; `dynamic` versus `object` identity.
- **Constraints:** Requires dynamic runtime binder and applicable runtime metadata.
- **Authority:** Language plus Microsoft runtime binder implementation.
- **Confidence:** High.
- **Likely primary source:** C# language specification dynamic binding; runtime binder documentation.

### CS-TYP-010 — Native-sized integers

- **Distinct behavior:** `nint` and `nuint` have platform-sized range and conversions, with constant and overload behavior that has changed between language versions.
- **Observable content:** Native integer declarations, casts, arithmetic, `sizeof`, overloads against `IntPtr`/`UIntPtr`, 32/64-bit conditional results.
- **Variants and failures:** Checked overflow varies by process architecture; metadata identity is `System.IntPtr`/`UIntPtr`; pre-C# 11 special compiler handling versus newer operator support.
- **Constraints:** Syntax requires C# 9; runtime/library capabilities and architecture matter.
- **Authority:** Language and runtime.
- **Confidence:** High.
- **Likely primary source:** Microsoft C# native-sized integer feature specifications.

### CS-TYP-011 — Ref-like, readonly, and inline-array structs

- **Distinct behavior:** `ref struct`, `readonly struct`, `readonly ref struct`, and inline arrays impose escape, storage, mutation, boxing, generic, async, and interface restrictions distinct from ordinary structs.
- **Observable content:** Span-backed ref struct; readonly members and defensive copies; `[InlineArray]` type used through indexing/spans; attempted heap capture/boxing.
- **Variants and failures:** Ref fields; `scoped` escape rules; interface implementation and generic `allows ref struct`; use in iterator/async code where value does not cross suspension; invalid class field or lambda capture.
- **Constraints:** Ref structs require C# 7.2; ref fields/scoped refinements C# 11; inline arrays C# 12; interface/generic relaxations C# 13 and compatible runtime.
- **Authority:** Language plus runtime/compiler support.
- **Confidence:** High.
- **Likely primary source:** Microsoft C# ref safety, inline arrays, and C# 13 ref struct feature specifications.

### CS-TYP-012 — Pointer, function-pointer, and fixed-buffer types

- **Distinct behavior:** Unsafe pointer types, managed/unmanaged function pointers, fixed buffers, address-taking, and pinning bypass ordinary managed-reference rules.
- **Observable content:** Pointer arithmetic and conversions; `fixed` statement; fixed-size buffer field; `delegate*` invocation with calling convention; `stackalloc` conversion.
- **Variants and failures:** Managed versus unmanaged calling convention; platform architecture; pointer to managed type error; lifetime bugs not always diagnosed; unsafe context placement.
- **Constraints:** Requires `AllowUnsafeBlocks`; function pointers require C# 9 and runtime support.
- **Authority:** Language, CLI, and platform application binary interface (ABI).
- **Confidence:** High.
- **Likely primary source:** C# language specification unsafe code; function pointer feature specification; ECMA-335.

## 3. Members, initialization, and access

### CS-MEM-001 — Fields, constants, and static initialization

- **Distinct behavior:** Instance/static fields, constants, `readonly`, `volatile`, and type initialization differ in storage, metadata, allowed expressions, ordering, and memory semantics.
- **Observable content:** Const consumed across an assembly boundary; readonly writes in constructors; volatile field; static field initializers and explicit static constructor; `beforefieldinit` contrast.
- **Variants and failures:** Inlined stale constants after dependency rebuild mismatch; textual partial-part ordering caveats; circular initialization; unsupported volatile types; default versus explicit initialization.
- **Constraints:** CLI controls metadata and type-initializer execution; memory-model details are runtime/platform-sensitive.
- **Authority:** Language and CLI/runtime.
- **Confidence:** High.
- **Likely primary source:** C# language specification variables and classes; ECMA-335 type initialization.

### CS-MEM-002 — Properties and indexers

- **Distinct behavior:** Properties/indexers are accessor methods with source-level member semantics, including auto-property storage, expression bodies, accessor-specific visibility, `init`, `required`, `ref` returns, and the C# 13 `field` preview/feature trajectory.
- **Observable content:** Auto and custom accessors; asymmetric access; init-only and required properties; indexer overloads; ref-return property; accessor attributes.
- **Variants and failures:** Property is not a variable except ref-return; init-only metadata via `modreq`; required-member inheritance/constructor contracts; property hiding/overriding; `field` contextual keyword version dependency.
- **Constraints:** Init-only requires C# 9; required members C# 11; partial properties/indexers C# 13; `field` keyword availability depends on post-C# 13 language version/preview.
- **Authority:** Language plus metadata conventions.
- **Confidence:** High, except medium for post-C# 13 `field` status.
- **Likely primary source:** C# language specification members; init-only and required-members feature specifications.

### CS-MEM-003 — Events

- **Distinct behavior:** Field-like and custom events expose add/remove accessors, restrict invocation to the declaring type, and have distinct interface/override behavior.
- **Observable content:** Field-like event, explicit add/remove event, interface event, virtual override, subscription/unsubscription, event invocation.
- **Variants and failures:** Event versus delegate field access; static events; explicit interface events; thread safety of synthesized accessors; partial-event support in newer language versions.
- **Constraints:** Custom accessor lowering and thread-safety details are compiler/runtime behavior; partial events are post-C# 13/version-gated.
- **Authority:** Language plus compiler lowering.
- **Confidence:** High for established events; medium for partial events.
- **Likely primary source:** C# language specification events; relevant newer C# feature specification.

### CS-MEM-004 — Constructors, finalizers, and initialization order

- **Distinct behavior:** Instance, static, primary, copy-like record, and struct constructors govern initialization; finalizers lower to `Finalize` overrides with runtime scheduling.
- **Observable content:** Base/this constructor chains; field/property initializers; explicit parameterless struct constructor; finalizer; record copy constructor; constructor attributes.
- **Variants and failures:** Implicit constructors suppressed by declarations; `new T()` under generic constraints; `default(T)` bypassing struct constructor; exception during initialization; finalizer nondeterminism.
- **Constraints:** Parameterless struct constructor support changed in C# 10 and needs compatible runtime behavior for some activation paths.
- **Authority:** Language and CLR runtime.
- **Confidence:** High.
- **Likely primary source:** C# language specification constructors/destructors; ECMA-335 object construction.

### CS-MEM-005 — Required members and initialization contracts

- **Distinct behavior:** `required` fields/properties create compile-time object-initialization obligations propagated through inheritance and affected by constructor annotations.
- **Observable content:** Required members on base and derived classes/records; satisfying object initializer; constructor with `SetsRequiredMembers`; generic `new()` use.
- **Variants and failures:** Inaccessible required setter/member; default values do not satisfy caller obligation; copy/with behavior; metadata consumer from older compiler; hidden required member.
- **Constraints:** C# 11 and required-member attributes in target framework or polyfills.
- **Authority:** Language and metadata convention.
- **Confidence:** High.
- **Likely primary source:** Microsoft C# required members feature specification.

### CS-MEM-006 — Access modifiers and effective accessibility

- **Distinct behavior:** `public`, `private`, `protected`, `internal`, combined protected/internal forms, and `file` visibility are checked through containing types and signature types.
- **Observable content:** Members at all access levels across derived/non-derived types and another assembly; public member with less-accessible signature as an error.
- **Variants and failures:** `protected internal` is union while `private protected` is intersection; nested-type access; accessor-specific access; friend assemblies; reflection bypass.
- **Constraints:** Cross-assembly tests require distinct assembly identity; protected rules depend on receiver expression.
- **Authority:** Language and assembly metadata.
- **Confidence:** High.
- **Likely primary source:** C# language specification declared accessibility and member access.

### CS-MEM-007 — Extension methods and extension binding

- **Distinct behavior:** Extension methods are static methods considered by special lookup only when instance lookup finds no applicable member; receiver conversions and import scope control candidates.
- **Observable content:** Competing extension namespaces; generic and ref receiver extensions; instance member taking precedence; invocation as ordinary static method; null receiver.
- **Variants and failures:** Ambiguity; inaccessible extension container; dynamic receiver; extension on interface versus concrete type; extension blocks/members in post-C# 13 versions.
- **Constraints:** Classic extension methods require static non-generic containing class; extension-member syntax is newer/version-gated.
- **Authority:** Language.
- **Confidence:** High for classic form; medium for post-C# 13 extension blocks.
- **Likely primary source:** C# language specification extension methods; newer extension members feature specification.

### CS-MEM-008 — User-defined operators and conversions

- **Distinct behavior:** Declared operators and implicit/explicit conversions participate in overload resolution with restrictions on declaring/operand types; checked operator variants and truth operators affect expression semantics.
- **Observable content:** Arithmetic/comparison operator pairs; implicit and explicit conversions; `true`/`false`; checked and unchecked operators; nullable operands.
- **Variants and failures:** Ambiguous conversion chains; no chaining of two user conversions; lifted operators; checked declaration fallback; equality warnings; interface static abstract operators.
- **Constraints:** Checked user-defined operators require C# 11; some operator declaration rules evolved.
- **Authority:** Language.
- **Confidence:** High.
- **Likely primary source:** C# language specification operators and user-defined conversions.

## 4. Generics and abstraction constraints

### CS-GEN-001 — Generic type and method construction

- **Distinct behavior:** Generic definitions, constructed types, open types, method type inference, and reification in CLI metadata create identities and binding distinct from erased generic systems.
- **Observable content:** Generic classes, nested generics, generic methods with inferred/explicit arguments, reflection over open and closed constructions, static fields per construction.
- **Variants and failures:** Arity overloads; inference failure; metadata versus source parameter names; code sharing is runtime implementation and does not erase identity.
- **Constraints:** CLI generic constraints and reflection shape apply.
- **Authority:** Language and CLI/runtime.
- **Confidence:** High.
- **Likely primary source:** C# language specification generics; ECMA-335 generics.

### CS-GEN-002 — Generic constraints and constraint ordering

- **Distinct behavior:** Class, struct, unmanaged, notnull, base/interface, constructor, default, enum/delegate, and ref-safety constraints change valid substitutions and available operations.
- **Observable content:** Types/methods using each meaningful constraint; operations enabled by constraints; constrained calls; nullable type parameter annotations.
- **Variants and failures:** Mutually exclusive or wrongly ordered constraints; `class` versus `class?`; `notnull` warning behavior; `new()` with abstract/inaccessible constructors; constraint satisfaction through type parameters.
- **Constraints:** `unmanaged` requires C# 7.3; nullable constraints C# 8; `default` constraint C# 9.
- **Authority:** Language and CLI metadata where representable.
- **Confidence:** High.
- **Likely primary source:** C# language specification generic constraints and relevant feature specifications.

### CS-GEN-003 — Variance

- **Distinct behavior:** Interface and delegate type parameters may be covariant or contravariant, enabling reference conversions while restricting parameter use in declarations.
- **Observable content:** Variant interface/delegate definitions; successful and failing conversions; variant generic delegate method-group use.
- **Variants and failures:** Value-type substitutions do not gain variance conversions; invariant nested positions; ambiguity from multiple variant conversions; runtime cast behavior.
- **Constraints:** Only interfaces and delegates declare variance; only reference-type arguments participate.
- **Authority:** Language and CLI.
- **Confidence:** High.
- **Likely primary source:** C# language specification variance; ECMA-335.

### CS-GEN-004 — Static abstract and virtual interface members

- **Distinct behavior:** Static abstract/virtual interface members permit type-parameter-qualified dispatch and operator abstraction, unlike ordinary runtime instance dispatch.
- **Observable content:** Interface with static abstract property/method/operator; implementations; generic algorithm calling `T.Member`; inherited default static implementation.
- **Variants and failures:** Direct interface invocation restrictions; ambiguous most-specific implementation; checked operators; explicit static implementation; type inference limitations.
- **Constraints:** Requires C# 11 and .NET runtime/tooling support appropriate to static interface members.
- **Authority:** Language and runtime.
- **Confidence:** High.
- **Likely primary source:** Microsoft C# static abstract interface members feature specification.

### CS-GEN-005 — Ref-struct generic anti-constraint

- **Distinct behavior:** `allows ref struct` admits ref-like type arguments while imposing ref-safety obligations on generic code and interacting with other constraints.
- **Observable content:** Generic interface/method accepting both ordinary and ref structs; safe operations; rejected capture/boxing; implementation by a ref struct.
- **Variants and failures:** Constraint ordering; conflict with `class`; use through virtual/interface dispatch; runtime or target support; substitution into older metadata consumers.
- **Constraints:** C# 13 and suitable runtime/compiler support.
- **Authority:** Language and runtime.
- **Confidence:** High.
- **Likely primary source:** Microsoft C# 13 feature specification for ref struct interfaces and generic anti-constraints.

## 5. Lookup, overload resolution, and conversions

### CS-BND-001 — Using directives, aliases, global usings, and ambiguity

- **Distinct behavior:** Namespace/type imports, static imports, aliases, `global::`, and global usings alter name lookup without changing declared identity.
- **Observable content:** Same simple type name from two namespaces; namespace/type and generic aliases; `using static`; global using in another file; root-qualified resolution.
- **Variants and failures:** Duplicate aliases; alias shadowing; alias to tuple/pointer/array with unsafe requirements; global-using scope and ordering; implicit SDK usings adding ambiguity.
- **Constraints:** Global usings require C# 10; aliases to broader type forms require C# 12.
- **Authority:** Language plus SDK convention for generated implicit usings.
- **Confidence:** High.
- **Likely primary source:** C# language specification using directives; global using and alias-any-type feature specifications.

### CS-BND-002 — Member lookup, hiding, and `new`

- **Distinct behavior:** Member lookup across base types can hide by name or signature; `new` acknowledges hiding but does not create virtual overriding.
- **Observable content:** Derived members hiding fields, methods, properties, nested types, and overload groups; qualified base access; calls through base and derived static types.
- **Variants and failures:** Warning with/without `new`; method overloads introduced in derived type; inaccessible members affecting lookup; generic-base substitutions.
- **Constraints:** Compile-time receiver type controls non-virtual binding.
- **Authority:** Language.
- **Confidence:** High.
- **Likely primary source:** C# language specification member lookup and hiding.

### CS-BND-003 — Overload resolution

- **Distinct behavior:** Candidate discovery, applicability, generic inference, conversions, receiver kind, parameter form, and tie-breakers select a single callable member or diagnose ambiguity.
- **Observable content:** Overloads differing by numeric/reference conversion, genericity, `params`, optional parameters, ref kind, inheritance, and extension status.
- **Variants and failures:** Null/default literal ambiguity; lambda natural type; user-defined conversions; better function member version changes; overload-resolution-priority metadata in newer compilers.
- **Constraints:** Rules have accumulated across C# versions; referencing assemblies built with newer compiler attributes can affect selection.
- **Authority:** Language, with compatibility details in compiler feature specs.
- **Confidence:** High.
- **Likely primary source:** C# language specification overload resolution; Microsoft C# breaking-change and feature specifications.

### CS-BND-004 — Named, optional, and parameter-array arguments

- **Distinct behavior:** Named argument mapping, compile-time optional defaults, `params` expansion, and newer params-collection builders change call shape and dependency semantics.
- **Observable content:** Reordered named arguments; omitted optional values; normal and expanded `params` calls; `params` array, span, and collection forms.
- **Variants and failures:** Caller embeds optional default; source/binary version mismatch; duplicate/out-of-position names; empty expanded form; overload competition; invalid collection builder.
- **Constraints:** Named/optional arguments C# 4; non-array params collections require C# 13 and compatible collection construction pattern.
- **Authority:** Language and metadata conventions.
- **Confidence:** High.
- **Likely primary source:** C# language specification arguments; C# 13 params collections feature specification.

### CS-BND-005 — Ref argument and return kinds

- **Distinct behavior:** `ref`, `out`, `in`, `ref readonly`, ref returns, ref locals, conditional refs, and ref assignment preserve variable identity and impose exact signature/escape rules.
- **Observable content:** Each argument kind; ref-returning indexer/method; ref local reassignment; readonly alias; overloads differing where permitted.
- **Variants and failures:** Temporary passed to `in`; call-site modifier requirements and warnings; `ref`/`out` metadata signature collision; defensive copies; returning local/reference with insufficient lifetime.
- **Constraints:** Feature availability spans C# 7 through C# 12; CLR signatures encode some distinctions through modifiers/attributes.
- **Authority:** Language and CLI metadata.
- **Confidence:** High.
- **Likely primary source:** C# language specification variables and parameter passing; ref readonly parameters feature specification.

### CS-BND-006 — Built-in and user-defined conversions

- **Distinct behavior:** Identity, numeric, reference, boxing, unboxing, nullable, tuple, dynamic, pointer, span, interpolated-string-handler, and user conversions determine assignment and overload applicability.
- **Observable content:** Representative implicit/explicit conversions and failed casts; checked numeric conversion; boxing/unboxing; reference downcast; conversion operators.
- **Variants and failures:** Constant-expression conversions; null literal; variance/reference ambiguity; invalid unbox despite convertible numeric value; newer span conversions can change overload selection.
- **Constraints:** Conversion set and tie-breakers vary by language version and target library surface.
- **Authority:** Language plus runtime type checks.
- **Confidence:** High.
- **Likely primary source:** C# language specification conversions; relevant feature specifications.

### CS-BND-007 — Method groups, target typing, and natural function types

- **Distinct behavior:** A method group has no ordinary value until converted or target-typed; lambdas and method groups can acquire delegate or expression-tree types, with natural types in newer versions.
- **Observable content:** Overloaded method group assigned to delegates, passed generically, converted to `var` where allowed; lambda with inferred/explicit parameter and return types.
- **Variants and failures:** Ambiguous group; static versus instance receiver capture; extension candidate; variance; ref-return/parameter mismatch; version-dependent inferred natural delegate type.
- **Constraints:** Natural types for lambdas/method groups require C# 10 refinements; conversions continue to evolve.
- **Authority:** Language.
- **Confidence:** High.
- **Likely primary source:** C# language specification anonymous functions and method groups; C# 10 lambda improvements specification.

## 6. Inheritance, interfaces, and dispatch

### CS-DSP-001 — Virtual, abstract, sealed, and override dispatch

- **Distinct behavior:** Virtual slots dispatch by runtime receiver type; abstract members require implementation; sealed overrides terminate further overriding; covariant returns preserve slot identity with refined source type.
- **Observable content:** Multi-level hierarchy with virtual calls through each static type; abstract base; sealed override; covariant-return override.
- **Variants and failures:** Calling virtual member during construction; override accessibility/signature/nullability mismatch; `new virtual` creates a separate slot; base-qualified call.
- **Constraints:** Covariant returns require C# 9 and runtime support for metadata pattern.
- **Authority:** Language and CLI runtime dispatch.
- **Confidence:** High.
- **Likely primary source:** C# language specification classes and virtual methods; ECMA-335.

### CS-DSP-002 — Interface implementation and explicit members

- **Distinct behavior:** Implicit implementation uses compatible public members; explicit implementation is accessible only through the interface and can distinguish otherwise colliding interface contracts.
- **Observable content:** One type implementing multiple interfaces implicitly and explicitly; interface mapping; inherited implementation; reimplementation in derived type.
- **Variants and failures:** Return/ref/nullability mismatch; default implementation interaction; property/event accessor differences; boxing of struct receiver; static abstract implementation.
- **Constraints:** Runtime interface maps and compiler rules both matter.
- **Authority:** Language and CLI.
- **Confidence:** High.
- **Likely primary source:** C# language specification interfaces; ECMA-335 method implementation metadata.

### CS-DSP-003 — Default interface implementations

- **Distinct behavior:** Interfaces can supply instance member bodies; runtime resolution chooses the most specific implementation and can report ambiguity.
- **Observable content:** Default method/property; derived interface override; class without implementation; diamond interface inheritance; call through interface reference.
- **Variants and failures:** Class member wins; explicit reabstraction; ambiguous unrelated defaults; runtime support absent; interface member not callable as a class member solely because default exists.
- **Constraints:** Requires C# 8 and supporting runtime; older .NET Framework targets may not support execution.
- **Authority:** Language and CLR.
- **Confidence:** High.
- **Likely primary source:** Microsoft C# default interface methods feature specification; runtime documentation.

### CS-DSP-004 — Boxing and constrained dispatch for value types

- **Distinct behavior:** Converting structs to object/interface boxes a copy, while constrained generic/interface calls may avoid boxing; mutation can target different storage.
- **Observable content:** Mutable struct boxed to interface; calls before/after original mutation; generic constrained call; overridden `ToString` versus inherited object member.
- **Variants and failures:** Nullable boxing; readonly receiver defensive copy; explicit interface member; enum boxing; identity/reference comparisons.
- **Constraints:** Optimization is implementation-specific, but observable copy and dispatch semantics are defined.
- **Authority:** Language and CLI/runtime.
- **Confidence:** High.
- **Likely primary source:** C# language specification boxing; ECMA-335 constrained calls.

### CS-DSP-005 — Delegate creation, combination, and invocation

- **Distinct behavior:** Delegates bind target plus method, support variance, multicast combination, equality, and invocation-list exception behavior.
- **Observable content:** Static/open/closed instance delegates where expressible; multicast add/remove; generic variant delegate; async-returning delegate.
- **Variants and failures:** Removal uses last matching subsequence; invocation stops on exception; returned value from last handler; method group versus lambda identity; struct target boxing.
- **Constraints:** Reflection APIs can create delegate forms not directly inferred by syntax.
- **Authority:** Language and runtime library.
- **Confidence:** High.
- **Likely primary source:** C# language specification delegates; .NET delegate runtime documentation.

## 7. Expressions, patterns, and flow analysis

### CS-EXP-001 — Definite assignment, reachability, and variable scope

- **Distinct behavior:** Compile-time dataflow governs reads, out assignments, constructor field initialization, pattern/local scope, unreachable code, and return completeness.
- **Observable content:** Branch/loop/try assignments; out variables; pattern variables across boolean operators; invalid unassigned read and unreachable statement.
- **Variants and failures:** Struct fields versus class fields; captured variables; switch exhaustiveness differs from definite assignment; null-state is separate analysis.
- **Constraints:** Compiler diagnostics may evolve while core rules are language-defined.
- **Authority:** Language plus diagnostic implementation.
- **Confidence:** High.
- **Likely primary source:** C# language specification variables, statements, and definite assignment.

### CS-EXP-002 — Pattern matching and narrowing

- **Distinct behavior:** Declaration, type, constant, relational, logical, property, positional, recursive, list, slice, and `var` patterns perform tests, bind variables, narrow types, and affect flow state.
- **Observable content:** `is` and switch patterns across the pattern families; custom `Deconstruct`; list-pattern-compatible type; null and guarded cases.
- **Variants and failures:** Subsumed/unreachable arms; non-exhaustive switch expression; evaluation order and repeated property access; `and`/`or` variable restrictions; nullable narrowing; user operators generally not used for relational pattern semantics in the same way as expressions.
- **Constraints:** Feature families span C# 7–11; list patterns require C# 11.
- **Authority:** Language.
- **Confidence:** High.
- **Likely primary source:** C# language specification patterns and pattern-matching feature specifications.

### CS-EXP-003 — Switch statement/expression exhaustiveness and guards

- **Distinct behavior:** Switch selection, guard evaluation, exhaustiveness warnings, runtime failure for unmatched expressions, and subsumption depend on patterns and input type.
- **Observable content:** Statement and expression switches over enum, nullable, closed-looking hierarchy, tuple, and guarded patterns.
- **Variants and failures:** Enums are not closed; guard prevents subsumption certainty; null arm; warning versus compile error; runtime `SwitchExpressionException`; optimized decision DAG is not semantic ordering beyond specified tests.
- **Constraints:** Switch expressions require C# 8; diagnostics vary by compiler version/settings.
- **Authority:** Language plus compiler diagnostics/lowering.
- **Confidence:** High.
- **Likely primary source:** C# switch expression and pattern matching specifications.

### CS-EXP-004 — Null operators and conditional access

- **Distinct behavior:** `??`, `??=`, `?.`, `?[]`, null-forgiving `!`, and lifted access short-circuit evaluation and affect type/null flow.
- **Observable content:** Side-effecting receiver/index/argument; chained conditional access; nullable value access; coalescing assignment; throw expression.
- **Variants and failures:** Parentheses interrupt conditional chain; conditional access to extension/ref members; unconstrained generic receiver; null-conditional assignment is post-C# 13/version-gated.
- **Constraints:** Null-conditional/coalescing features span multiple versions; nullable warnings depend on context.
- **Authority:** Language.
- **Confidence:** High for established operators; medium for null-conditional assignment.
- **Likely primary source:** C# language specification null operators; relevant newer feature specification.

### CS-EXP-005 — Object, collection, and `with` initialization

- **Distinct behavior:** Object/collection initializers, nested member initializers, index initializers, `with`, and implicit index-from-end initialization invoke different construction, assignment, and mutation paths.
- **Observable content:** Object initializer with required/init members; collection `Add` overloads; dictionary index initializer; nested initializer; record and struct `with`; `[^n]` initializer.
- **Variants and failures:** Collection initializer requires enumerable shape plus applicable `Add`; nested initializer does not replace object; exceptions leave partially initialized objects; C# 13 implicit index access; anonymous-type `with` invalid.
- **Constraints:** Implicit indexer initialization requires C# 13; `with` coverage expanded after C# 9.
- **Authority:** Language.
- **Confidence:** High.
- **Likely primary source:** C# language specification object/collection initializers; relevant `with` and C# 13 feature specs.

### CS-EXP-006 — Collection expressions and builders

- **Distinct behavior:** `[ ... ]` collection expressions are target-typed and can lower to arrays, spans, constructible collections, or builder methods; spread elements enumerate/copy sources.
- **Observable content:** Collection expressions targeting array, span, interface, list-like custom builder; empty expression; spread with side effects; overload competition.
- **Variants and failures:** No target type; capacity/count availability; evaluation order; builder attribute validity; ref-safety for spans; conversion changes overload choice.
- **Constraints:** Requires C# 12 and suitable target framework attributes/types for custom builders.
- **Authority:** Language plus library/compiler conventions.
- **Confidence:** High.
- **Likely primary source:** Microsoft C# 12 collection expressions feature specification.

### CS-EXP-007 — Range, index, and slicing patterns

- **Distinct behavior:** `Index`/`Range` syntax can bind through language-recognized patterns or indexers, with from-end length calculations and target-dependent slicing.
- **Observable content:** Array/string/span slicing; custom type with `Length`/`Count`, integer/index/range indexers, or `Slice`; boundary cases.
- **Variants and failures:** Copy versus view; from-end zero; out-of-range runtime error; pattern member accessibility; overload resolution between indexer forms.
- **Constraints:** Requires C# 8 and target framework types/support.
- **Authority:** Language plus target library implementation.
- **Confidence:** High.
- **Likely primary source:** Microsoft C# indices and ranges feature specification.

### CS-EXP-008 — Interpolated strings and handlers

- **Distinct behavior:** Interpolated strings may convert to string, formattable forms, or custom handlers that conditionally skip formatting and argument evaluation.
- **Observable content:** Ordinary, verbatim, raw interpolation; alignment/format; custom handler with receiver/argument forwarding and conditional append; overloads competing by target.
- **Variants and failures:** Culture differences; evaluation skipped by handler; malformed handler shape; ref struct handler lifetime; compile-time constant interpolated string.
- **Constraints:** Custom handlers and constant interpolation require C# 10; raw interpolation C# 11.
- **Authority:** Language plus formatting library behavior.
- **Confidence:** High.
- **Likely primary source:** Microsoft C# interpolated string handler feature specification.

### CS-EXP-009 — Checked and unchecked arithmetic

- **Distinct behavior:** Checked context controls overflow for integral operations and explicit conversions, including user-defined checked operators, while floating-point behavior differs.
- **Observable content:** Compile-time constant overflow; runtime overflow in checked/unchecked scopes; `/checked` project option; checked user operator.
- **Variants and failures:** Decimal always throws for overflow; floating-point does not; native integer architecture; explicit local context overrides project setting; dependency-compiled bodies retain their semantics.
- **Constraints:** User-defined checked operators require C# 11.
- **Authority:** Language and runtime numeric implementation.
- **Confidence:** High.
- **Likely primary source:** C# language specification checked and unchecked operators/statements.

### CS-EXP-010 — Expression trees

- **Distinct behavior:** A lambda converted to `Expression<TDelegate>` becomes a data structure with a restricted language subset rather than executable delegate code.
- **Observable content:** Equivalent lambda converted to delegate and expression tree; quoted nested lambda; captured variable; provider inspection/compilation.
- **Variants and failures:** Many newer constructs are prohibited or lowered differently; optional/named/dynamic/ref arguments and statement bodies restrictions; captured closure represented through constants/member access; compiler version expands restrictions slowly.
- **Constraints:** Depends on `System.Linq.Expressions`; supported node model may lag language features.
- **Authority:** Language conversion rules plus runtime library API.
- **Confidence:** High.
- **Likely primary source:** C# language specification anonymous functions; .NET expression tree documentation.

## 8. Functions, capture, suspension, and enumeration

### CS-FUN-001 — Lambdas, anonymous methods, and capture

- **Distinct behavior:** Anonymous functions infer signatures from target types and capture variables into compiler-generated closure storage with shared-variable semantics.
- **Observable content:** Capturing and noncapturing lambdas; loop-variable capture; mutable captured local; `static` lambda; anonymous method; lambda attributes and explicit return type.
- **Variants and failures:** Capture lifetime extends scope; `foreach` capture semantics differ from old compiler versions; static lambda capture error; delegate identity/cache behavior implementation-specific; ref-like capture forbidden.
- **Constraints:** Static lambdas C# 9; lambda attributes/explicit return types C# 10; default lambda parameters C# 12.
- **Authority:** Language, with closure class shape/caching implementation-defined.
- **Confidence:** High.
- **Likely primary source:** C# language specification anonymous functions; lambda improvement feature specs.

### CS-FUN-002 — Local functions

- **Distinct behavior:** Local functions have lexical scope, can be generic/recursive/iterator/async, capture variables, and differ from lambdas in definite assignment and allocation opportunities.
- **Observable content:** Generic recursive local function; static local function; captured local; iterator or async local function; attributes.
- **Variants and failures:** Forward use; overload limitations; capture of ref-like values; generated method/closure identities implementation-defined.
- **Constraints:** Local functions require C# 7; static local functions C# 8.
- **Authority:** Language plus compiler lowering.
- **Confidence:** High.
- **Likely primary source:** Microsoft C# local functions feature specification.

### CS-FUN-003 — Async methods and awaitable pattern

- **Distinct behavior:** `async` methods lower to state machines; `await` binds an awaitable pattern, splits execution, captures context depending on awaiter/library behavior, and changes exception delivery.
- **Observable content:** `Task`, `Task<T>`, `ValueTask<T>`, `void`, and custom task-like returns; custom awaiter; multiple awaits; exception before/after suspension; async `Main`.
- **Variants and failures:** `async void` exception/caller semantics; no-suspension warning; ref-like locals allowed only when not crossing await under newer rules; synchronization-context behavior is library/runtime, not `await` itself; builder attributes.
- **Constraints:** Custom task-like returns C# 7; ref/unsafe relaxation in C# 13; runtime/library support required.
- **Authority:** Language plus compiler lowering and task/awaiter libraries.
- **Confidence:** High.
- **Likely primary source:** C# language specification async functions; task-like and C# 13 feature specifications.

### CS-FUN-004 — Iterators and enumeration pattern

- **Distinct behavior:** `yield` methods lower to state machines with deferred execution and disposal; `foreach` can bind a pattern without `IEnumerable` and has special array/string/ref semantics.
- **Observable content:** Generic and nongeneric iterator; `try/finally` around yield; custom enumerator pattern; ref/ref readonly iteration variable; extension `GetEnumerator`.
- **Variants and failures:** Exceptions deferred until enumeration; mutation invalidation comes from collection; boxing of struct enumerators; disposal paths; ref/unsafe values not crossing yield under C# 13; invalid `yield` in catch/finally contexts.
- **Constraints:** Ref extension enumeration and ref iteration are newer additions; C# 13 relaxes some restrictions.
- **Authority:** Language plus compiler lowering/library conventions.
- **Confidence:** High.
- **Likely primary source:** C# language specification iterators and `foreach`; C# 13 ref/unsafe iterator feature specification.

### CS-FUN-005 — Async iterators and `await foreach`

- **Distinct behavior:** Async iterators combine async and iterator state machines, use cancellation/disposal patterns, and defer execution through `IAsyncEnumerable<T>`-like shapes.
- **Observable content:** `async IAsyncEnumerable<T>` with `yield` and `await`; `await foreach`; `await using`; cancellation token forwarding attribute; custom async enumerator pattern.
- **Variants and failures:** Cancellation token source selection; disposal exception; configure-await extension behavior; break/exception disposal; multiple enumeration; ref-like restrictions.
- **Constraints:** Requires C# 8 and supporting async interfaces/library.
- **Authority:** Language plus runtime library conventions.
- **Confidence:** High.
- **Likely primary source:** Microsoft C# async streams feature specification.

## 9. Statements, exceptions, and lifetime

### CS-CTL-001 — Exception handling and filters

- **Distinct behavior:** Catch selection, filters, rethrow, finally execution, and exception propagation have precise evaluation and stack-preservation semantics.
- **Observable content:** Typed catches with filters and side effects; `throw;` versus `throw ex;`; nested try/finally; exception during filter/finally; unreachable catch error.
- **Variants and failures:** Filter exceptions treated as false by runtime; asynchronous/iterator exception timing; non-CLS exceptions historically wrapped depending runtime setting; stack trace differences.
- **Constraints:** Some exception behavior belongs to CLR and runtime configuration.
- **Authority:** Language and CLR.
- **Confidence:** High.
- **Likely primary source:** C# language specification exception handling; ECMA-335 exception model.

### CS-CTL-002 — `using`, disposal patterns, and `await using`

- **Distinct behavior:** Using statements/declarations lower to guaranteed disposal, bind interface or pattern-based disposal, handle nullable/resource copies, and dispose in reverse declaration order.
- **Observable content:** Statement and declaration forms; multiple resources; struct disposer; pattern-only `Dispose`; `await using`; early return/exception.
- **Variants and failures:** Declared variable readonly restrictions; boxing of disposer; dynamic resource; disposal exception replacing body exception; scope difference for declaration; ref struct pattern.
- **Constraints:** Using declarations and pattern-based async disposal require C# 8.
- **Authority:** Language plus disposal interfaces/library.
- **Confidence:** High.
- **Likely primary source:** C# language specification using statement; C# 8 using declaration/async streams specifications.

### CS-CTL-003 — `lock` semantics and `System.Threading.Lock`

- **Distinct behavior:** Traditional `lock` lowers through `Monitor`; C# 13 recognizes `System.Threading.Lock` and uses its scope pattern, creating type-directed lowering.
- **Observable content:** Lock on ordinary reference and on `System.Threading.Lock`; exception/return inside lock; warning-worthy lock targets; attempted await inside body.
- **Variants and failures:** Boxing a value as lock identity trap; locking `this`, type, or interned string ecosystem hazards; conversion of new lock type to object changes semantics; mutual exclusion/runtime behavior.
- **Constraints:** Specialized lock requires C# 13 and .NET 9 library type.
- **Authority:** Language/compiler plus runtime library.
- **Confidence:** High.
- **Likely primary source:** Microsoft C# 13 lock object feature specification; .NET Monitor/Lock documentation.

### CS-CTL-004 — Fixed, stack allocation, and escape lifetime

- **Distinct behavior:** `fixed`, `stackalloc`, span conversions, and ref escape analysis control whether stack or movable managed storage can be referenced safely.
- **Observable content:** Fixed string/array/span/pattern pinnable object; stackalloc pointer and span; conditional stackalloc; returned/local/captured reference attempts.
- **Variants and failures:** Zero-length/null pinning; custom `GetPinnableReference`; compiler stack/heap optimizations not guaranteed; stack overflow; safe-context and scoped propagation.
- **Constraints:** Stackalloc span conversion requires C# 7.2; safe-context refinements changed in C# 11 and later.
- **Authority:** Language, compiler, and runtime garbage collector.
- **Confidence:** High.
- **Likely primary source:** C# unsafe-code and ref-safety specifications.

## 10. Metadata, attributes, and compiler-recognized protocols

### CS-MET-001 — Attributes and target selection

- **Distinct behavior:** Attributes attach metadata to assemblies, modules, types, parameters, return values, accessors, backing fields, generic parameters, and synthesized record members; usage rules govern multiplicity/inheritance.
- **Observable content:** Custom attribute with positional/named arguments; all important targets; inherited and non-inherited attributes; conditional attribute class.
- **Variants and failures:** Attribute arguments limited to metadata constants; short-name resolution; duplicate disallowed; attribute inheritance is reflection API behavior and differs by target; record parameter target forwarding.
- **Constraints:** Metadata readers may expose synthesized targets differently.
- **Authority:** Language, CLI metadata, and reflection library.
- **Confidence:** High.
- **Likely primary source:** C# language specification attributes; ECMA-335 custom attributes.

### CS-MET-002 — Compiler-recognized attributes

- **Distinct behavior:** Attributes such as conditional, obsolete, caller-info, nullable-flow, required-member, module-initializer, interpolated-handler, collection-builder, and method-builder attributes alter compilation or analysis rather than merely annotate.
- **Observable content:** Representative recognized attributes and affected calls/diagnostics/lowering; same-looking unrecognized custom attribute for contrast.
- **Variants and failures:** Attribute namespace/type identity must match; malformed metadata; attributes consumed at declaration versus call site; conditional calls omitted with argument side effects; obsolete warning/error modes.
- **Constraints:** Availability depends on language and target framework; polyfills are sometimes valid.
- **Authority:** Language/compiler conventions and runtime libraries.
- **Confidence:** High.
- **Likely primary source:** C# language specification attributes and individual Microsoft C# feature specifications.

### CS-MET-003 — Caller information and expression capture

- **Distinct behavior:** Omitted optional arguments can be filled from caller file, line, member name, or argument expression text at the call site.
- **Observable content:** Calls using caller-member/file/line and caller-argument-expression parameters, including wrapped helper and explicit supplied argument.
- **Variants and failures:** Path mapping/deterministic build changes file text; argument-name mismatch; explicit argument suppresses substitution; generated and line-mapped source; expression spelling rather than semantic normalization.
- **Constraints:** CallerArgumentExpression requires C# 10 and framework attribute or polyfill.
- **Authority:** Language/compiler and metadata attribute convention.
- **Confidence:** High.
- **Likely primary source:** C# caller information and CallerArgumentExpression feature specifications.

### CS-MET-004 — Module initializers

- **Distinct behavior:** Attributed static methods are emitted to run when a module initializes, independently of ordinary type constructor calls.
- **Observable content:** Valid module initializer plus observable static state; multiple initializer methods; initializer in referenced module.
- **Variants and failures:** Ordering among multiple initializers is implementation-defined or constrained by compiler emission rather than source contract; invalid signature/accessibility/generic containment; trimming/linking.
- **Constraints:** C# 9; runtime module initialization semantics apply.
- **Authority:** Language feature plus CLR/compiler implementation.
- **Confidence:** High.
- **Likely primary source:** Microsoft C# module initializers feature specification; ECMA-335.

### CS-MET-005 — Reflection, metadata names, and constructed members

- **Distinct behavior:** Reflection exposes emitted rather than purely source concepts: backing fields, accessors, state machines, generic definitions, custom modifiers, explicit-interface names, and hidden compiler types.
- **Observable content:** Runtime reflection over representative source declarations and generated constructs; binding flags; generic member construction; tuple/dynamic/nullability metadata decoding.
- **Variants and failures:** Trimming or Native AOT can remove metadata; reflection order is not generally source order; private generated names are unstable; inherited member lookup differs by member kind.
- **Constraints:** Runtime, linker, and deployment model materially affect results.
- **Authority:** Runtime implementation and CLI metadata, with some source mappings defined by conventions.
- **Confidence:** High.
- **Likely primary source:** ECMA-335; .NET reflection documentation; compiler feature specifications.

## 11. Assembly, dependency, and package boundaries

### CS-ASM-001 — Assembly identity, references, and type identity

- **Distinct behavior:** Type identity includes assembly and namespace/nesting/name/arity; compilation references a chosen metadata surface, not arbitrary runtime assemblies.
- **Observable content:** Two assemblies defining same fully qualified type; consumer references; alias/disambiguation; runtime load/version contrast.
- **Variants and failures:** Simple-name conflicts; reference assembly versus implementation assembly; assembly unification/binding on .NET Framework; load contexts allowing multiple identities; missing transitive reference.
- **Constraints:** Resolution differs among .NET SDK/CoreCLR, .NET Framework MSBuild, and custom hosts.
- **Authority:** CLI, build toolchain, and runtime implementation.
- **Confidence:** High.
- **Likely primary source:** ECMA-335 assembly/type identity; .NET assembly loading documentation.

### CS-ASM-002 — `InternalsVisibleTo` and strong-named friend assemblies

- **Distinct behavior:** Friend assembly metadata expands `internal` accessibility to a named assembly and changes accessibility checks without making APIs public.
- **Observable content:** Producer with internal type/member; friend and non-friend consumers; friend declaration; signed-key form where applicable.
- **Variants and failures:** Assembly-name/key mismatch; unsigned versus strong-named producer; protected-internal combinations; compiler-generated internals used by tests/generators.
- **Constraints:** Strong-name rules and signing support differ by target/toolchain.
- **Authority:** Compiler/runtime attribute convention and CLI assembly identity.
- **Confidence:** High.
- **Likely primary source:** .NET InternalsVisibleToAttribute and strong naming documentation.

### CS-ASM-003 — Extern aliases and reference aliases

- **Distinct behavior:** `extern alias` plus build reference aliases allows simultaneous use of otherwise colliding assembly/type names.
- **Observable content:** Two referenced assemblies exporting the same qualified type; aliased references; `alias::Namespace.Type` usage; global alias inclusion/exclusion.
- **Variants and failures:** Alias missing from project metadata; `global` versus custom aliases; transitive dependencies cannot always be addressed through the same alias; alias conflicts.
- **Constraints:** MSBuild/Roslyn reference alias configuration required.
- **Authority:** Language plus compiler/build implementation.
- **Confidence:** High.
- **Likely primary source:** C# language specification extern aliases; MSBuild reference metadata documentation.

### CS-ASM-004 — Type forwarding and facade assemblies

- **Distinct behavior:** Type-forwarding metadata preserves a public type’s apparent assembly-facing compatibility while implementation moves to another assembly.
- **Observable content:** Facade assembly with `TypeForwardedTo`, destination assembly, and consumer compiled before/after movement.
- **Variants and failures:** Forwarding cycles/missing target; nested types; type identity and reflection assembly reporting; forwarding versus duplicate type definition; package version mismatch.
- **Constraints:** CLI/runtime loader behavior; build references must include resolvable destination.
- **Authority:** CLI and .NET runtime/compiler conventions.
- **Confidence:** High.
- **Likely primary source:** ECMA-335 exported types; .NET type forwarding documentation.

### CS-ASM-005 — PackageReference assets and compile/runtime separation

- **Distinct behavior:** NuGet selects compile, runtime, analyzer, build, native, and content assets separately; transitivity controls which APIs and files participate in a consumer project.
- **Observable content:** Package references with include/exclude/private assets; package providing ref/lib/runtime/analyzers/buildTransitive assets; lock/restore output as build input.
- **Variants and failures:** Compile succeeds but runtime asset missing; transitive package version resolution; central package management; package source mapping; stale/no restore; package downgrade/conflict diagnostics.
- **Constraints:** NuGet/MSBuild/.NET SDK behavior and package layout conventions, not C# language.
- **Authority:** Official NuGet/MSBuild implementation.
- **Confidence:** High.
- **Likely primary source:** NuGet PackageReference and package asset documentation.

### CS-ASM-006 — Framework references and reference assemblies

- **Distinct behavior:** Target framework supplies a curated compile-time API surface via reference packs while runtime packs/host supply implementation; the same C# can bind differently per target.
- **Observable content:** API present only in one target framework; FrameworkReference; compile against reference assembly and execute against runtime implementation.
- **Variants and failures:** Missing targeting pack; accidental implementation-assembly reference; platform-specific reference; API exists at runtime but not target compile surface; compatibility warning.
- **Constraints:** .NET SDK target framework monikers and installed packs.
- **Authority:** Official SDK/runtime implementation.
- **Confidence:** High.
- **Likely primary source:** .NET target framework and reference assembly documentation.

## 12. Project system, build configurations, and generated input

### CS-BLD-001 — SDK default items and explicit source inclusion

- **Distinct behavior:** SDK-style projects implicitly include matching source files; remove/include/update/link metadata and generated intermediate files determine compilation membership and logical paths.
- **Observable content:** Default-included file, excluded file, explicitly linked external file, conditional compile item, generated source in intermediate output.
- **Variants and failures:** Duplicate Compile items when defaults remain enabled; case sensitivity by filesystem; glob results; linked logical path versus physical path; design-time build differences.
- **Constraints:** MSBuild and selected SDK version; filesystem/platform behavior.
- **Authority:** Official .NET SDK/MSBuild implementation.
- **Confidence:** High.
- **Likely primary source:** .NET SDK default items and MSBuild item documentation.

### CS-BLD-002 — Language-version selection and preview features

- **Distinct behavior:** `LangVersion` changes parsing, binding, diagnostics, contextual keywords, and sometimes overload resolution independently of target framework.
- **Observable content:** Version-gated syntax/semantics compiled under explicit older, current, latest, and preview settings.
- **Variants and failures:** `latest` changes when SDK changes; `preview` instability; compiler may require framework types/attributes; generated code uses same or separately supplied parse options.
- **Constraints:** Available versions are bounded by installed compiler; SDK defaults derive language version from target framework.
- **Authority:** Official compiler/SDK implementation grounded in language versions.
- **Confidence:** High.
- **Likely primary source:** Microsoft C# language versioning and configure-language-version documentation.

### CS-BLD-003 — Conditional properties and compilation constants

- **Distinct behavior:** Configuration, platform, target framework, runtime identifier, and arbitrary MSBuild properties can alter source membership, constants, references, output kind, and compiler switches.
- **Observable content:** Conditional PropertyGroup/ItemGroup and `DefineConstants`; distinct Debug/Release or target-specific declarations.
- **Variants and failures:** Property evaluation order; semicolon constant merging; configuration name not intrinsically semantic; IDE design-time versus command-line global properties.
- **Constraints:** MSBuild evaluation model and project imports.
- **Authority:** Official MSBuild/.NET SDK implementation.
- **Confidence:** High.
- **Likely primary source:** MSBuild condition, property, item, and evaluation documentation.

### CS-BLD-004 — Multi-targeting

- **Distinct behavior:** One project can produce separate compilations per target framework with different references, constants, APIs, generated source, and outputs.
- **Observable content:** `TargetFrameworks`; target-conditioned source/reference/package; `#if` target symbols; API surface varying by target.
- **Variants and failures:** Cross-target asset selection; outer versus inner builds; incompatible package assets; same source symbol resolving to different definitions; target-specific nullable/platform warnings.
- **Constraints:** .NET SDK/MSBuild/NuGet versions and installed targeting packs.
- **Authority:** Official SDK/MSBuild/NuGet implementation.
- **Confidence:** High.
- **Likely primary source:** .NET SDK multi-targeting documentation.

### CS-BLD-005 — Output kind, entry point, and startup object

- **Distinct behavior:** Library, console, Windows executable, module, and other output settings change entry-point requirements, predefined symbols/attributes, and emitted artifact semantics.
- **Observable content:** Explicit/static `Main` overloads, top-level entry point, StartupObject selection, library build, platform-targeted executable.
- **Variants and failures:** Multiple valid entry points; async entry point; invalid signature; top-level statements precedence; exit-code conversion; platform subsystem distinction.
- **Constraints:** Supported output types vary with SDK and target.
- **Authority:** Language entry-point rules plus compiler/SDK implementation.
- **Confidence:** High.
- **Likely primary source:** C# language specification application startup; compiler option documentation.

### CS-BLD-006 — Nullable, warning, and diagnostic policy configuration

- **Distinct behavior:** Project/editor configuration controls nullable contexts, warning levels, severity promotion/suppression, analyzer diagnostics, and whether the same semantic issue fails the build.
- **Observable content:** `Nullable`, `WarningsAsErrors`, `NoWarn`, warning level, analysis level, `.editorconfig` severity, and source pragmas with conflicting precedence.
- **Variants and failures:** Diagnostic ID-specific versus all warnings; generated-code analyzer suppression; command-line/global analyzer config; newer SDK analysis level introduces diagnostics; compiler error cannot be downgraded like warning.
- **Constraints:** Roslyn/MSBuild/.NET SDK version-dependent.
- **Authority:** Official compiler/SDK implementation.
- **Confidence:** High.
- **Likely primary source:** C# compiler options and Roslyn analyzer configuration documentation.

### CS-BLD-007 — Implicit usings and generated global imports

- **Distinct behavior:** SDK implicit usings inject target/project-type-dependent global imports not present in handwritten source, affecting name resolution and ambiguity.
- **Observable content:** Enabled/disabled implicit usings; generated global-usings file; code that binds only when enabled; custom removal/addition.
- **Variants and failures:** Web versus console SDK sets; target framework/SDK version differences; collision with user type; stale IDE generated view.
- **Constraints:** .NET 6-era and later SDK templates/features; exact set is SDK-specific.
- **Authority:** Official .NET SDK convention.
- **Confidence:** High.
- **Likely primary source:** .NET SDK implicit using documentation and SDK targets.

### CS-BLD-008 — Source generators

- **Distinct behavior:** Incremental or classic generators add source to a compilation based on syntax, symbols, metadata, additional files, analyzer config, and references; generated declarations participate in later binding but not recursively in generator discovery in unrestricted ways.
- **Observable content:** Generator assembly, emitted source, handwritten partial counterpart, generator diagnostic, AdditionalFiles and analyzer options input.
- **Variants and failures:** Duplicate hint names/declarations; generator exception; nondeterministic output; generated-source ordering/encoding; generator cannot generally observe output of other generators in the same run as ordinary new input; IDE/design-time differences.
- **Constraints:** Roslyn compiler API and host version; generator target framework/loading rules.
- **Authority:** Official Roslyn compiler platform implementation.
- **Confidence:** High.
- **Likely primary source:** Roslyn source generator and incremental generator documentation.

### CS-BLD-009 — Analyzers, suppressors, and code-generation diagnostics

- **Distinct behavior:** Analyzers and suppressors inspect semantic compilations and add or suppress diagnostics without changing emitted program semantics directly; build policy can make them fatal.
- **Observable content:** Analyzer reference; source and project-option diagnostics; suppression attribute/pragma/editorconfig; analyzer-generated code classification.
- **Variants and failures:** Analyzer crash diagnostic; concurrent execution; severity precedence; compiler versus analyzer diagnostic; package transitivity/private assets; version skew.
- **Constraints:** Roslyn host API and selected analysis level.
- **Authority:** Official Roslyn/.NET SDK implementation plus package ecosystem.
- **Confidence:** High.
- **Likely primary source:** Roslyn analyzer and diagnostic configuration documentation.

### CS-BLD-010 — MSBuild code generation and build ordering

- **Distinct behavior:** Targets/tasks may create source before compilation, embed generated assembly attributes, or alter items; target dependency and incremental inputs/outputs decide whether generated files are current.
- **Observable content:** A pre-compile target generating a Compile item; generated assembly-info/global-usings files; declared inputs/outputs; clean versus incremental build.
- **Variants and failures:** File created after item evaluation but not added; stale output; parallel target race; design-time build skipping target; duplicate assembly attributes when auto-generation and manual source coexist.
- **Constraints:** MSBuild/.NET SDK targets and host environment.
- **Authority:** Official build implementation and ecosystem convention.
- **Confidence:** High.
- **Likely primary source:** MSBuild target/task and .NET SDK generated assembly info documentation.

### CS-BLD-011 — Deterministic, path-mapped, and embedded-source builds

- **Distinct behavior:** Deterministic compilation, path mapping, source embedding, checksums, and portable Program Database (PDB) settings affect emitted identities, caller-file strings, sequence points, and reproducibility.
- **Observable content:** Deterministic/path-map/debug settings; caller-file-path use; mapped source paths; embedded source or Source Link metadata; repeated-build comparison.
- **Variants and failures:** Absolute paths leak without mapping; timestamps/tool versions/resources break reproducibility; `#line` interaction; PDB omitted or platform-specific.
- **Constraints:** Roslyn/MSBuild and debugger metadata formats.
- **Authority:** Official compiler/SDK implementation.
- **Confidence:** High.
- **Likely primary source:** Roslyn deterministic build, PathMap, and portable PDB documentation.

### CS-BLD-012 — Resources, settings, and generated strongly typed accessors

- **Distinct behavior:** Embedded resources and `.resx` processing add manifest data and may generate C# accessors whose identity, culture fallback, and visibility are build-controlled.
- **Observable content:** Embedded resource, localized satellite resource, generated strongly typed resource class, manifest name customization.
- **Variants and failures:** Logical-name mismatch; culture fallback; duplicate resource names; designer/custom-tool source inclusion; trimming/single-file packaging effects.
- **Constraints:** MSBuild resource tasks, runtime ResourceManager, and platform tooling.
- **Authority:** Official SDK/runtime implementation and ecosystem convention.
- **Confidence:** Medium.
- **Likely primary source:** MSBuild resource and .NET ResourceManager documentation.

## 13. Platform interoperation and deployment-sensitive semantics

### CS-INT-001 — Platform invocation and source-generated interop

- **Distinct behavior:** P/Invoke declarations or generated marshalling bind managed signatures to native libraries, entry points, calling conventions, character sets, and marshalling rules.
- **Observable content:** `DllImport` and `LibraryImport` declarations; blittable and marshalled parameter types; explicit entry point/calling convention; last-error handling.
- **Variants and failures:** Library/entry-point not found; 32/64-bit ABI mismatch; string/boolean/struct layout; callback lifetime; source-generated marshaller diagnostics; Native AOT restrictions.
- **Constraints:** Operating system, architecture, runtime, native dependency, and unsafe settings.
- **Authority:** .NET runtime/SDK implementation and platform ABI.
- **Confidence:** High.
- **Likely primary source:** .NET native interoperability, DllImport, and LibraryImport documentation.

### CS-INT-002 — Explicit/sequential layout and marshalling metadata

- **Distinct behavior:** Struct/class layout attributes, field offsets, packing, fixed buffers, and marshal attributes determine unmanaged representation independently of ordinary source member order assumptions.
- **Observable content:** Sequential and explicit-layout structs; unions; pack/size; `MarshalAs`; size/offset checks.
- **Variants and failures:** Overlapping managed references rejected or unsafe; architecture-dependent padding; auto layout; generic layout restrictions; endianness is platform-defined.
- **Constraints:** CLR/runtime and ABI-specific.
- **Authority:** CLI/runtime and platform ABI.
- **Confidence:** High.
- **Likely primary source:** ECMA-335 layout metadata; .NET interop marshalling documentation.

### CS-INT-003 — COM and optional/named interop behavior

- **Distinct behavior:** COM interop can embed interop types, omit ref modifiers in special contexts, use dynamic dispatch, optional/named arguments, and runtime-callable wrappers.
- **Observable content:** Interop reference with embedded types; imported COM interface; named/optional invocation; event or dynamic member access.
- **Variants and failures:** Windows-only availability; type equivalence versus assembly identity; missing registration; apartment/thread behavior; trimming/AOT limitations.
- **Constraints:** Primarily Windows/.NET runtime and MSBuild reference metadata.
- **Authority:** Compiler/runtime implementation and COM platform specification.
- **Confidence:** Medium.
- **Likely primary source:** Microsoft C# COM interop and .NET COM interop documentation.

### CS-INT-004 — Platform compatibility and conditional APIs

- **Distinct behavior:** OS/architecture-specific target frameworks, runtime identifiers, and platform annotations change available references, analyzer flow, publish assets, and runtime validity.
- **Observable content:** Platform-specific API guarded by runtime test; supported/unsupported-platform attributes; OS-conditioned build item or target framework.
- **Variants and failures:** Analyzer understands recognized guards only; compile-time availability versus runtime support; browser/mobile/AOT restrictions; architecture-specific native asset.
- **Constraints:** SDK analyzers, target packs, runtime identifier graph, and deployment platform.
- **Authority:** Official SDK/runtime implementation and platform contracts.
- **Confidence:** High.
- **Likely primary source:** .NET platform compatibility analyzer and runtime identifier documentation.

### CS-INT-005 — Trimming, single-file, and Native AOT effects

- **Distinct behavior:** Deployment transforms can remove code/metadata, alter assembly locations, reject dynamic code, and require annotations, making reflection and loading behavior configuration-dependent.
- **Observable content:** Reflection/dynamic activation with trimming annotations; publish settings for trimming, single-file, and Native Ahead-of-Time (AOT); warning-producing call chain.
- **Variants and failures:** Missing members only after publish; `Assembly.Location`/files differ in bundle; source generation replaces reflection; unsupported runtime features; analyzer warnings versus runtime failures.
- **Constraints:** .NET publish toolchain, target platform, and runtime version.
- **Authority:** Official .NET SDK/runtime implementation.
- **Confidence:** High.
- **Likely primary source:** .NET trimming, single-file deployment, and Native AOT documentation.

## 14. Invalid, ambiguous, and compatibility cases

### CS-DIA-001 — Parse errors with recoverable declarations

- **Distinct behavior:** Roslyn constructs syntax trees and may expose declared/referenced symbols around missing or malformed tokens, while diagnostics identify incomplete source.
- **Observable content:** Carefully isolated missing delimiter/token, malformed generic, and unterminated construct adjacent to valid declarations.
- **Variants and failures:** Parser recovery changes by compiler version; cascading diagnostics; script versus regular parse kind; language version changes interpretation.
- **Constraints:** Compiler implementation behavior beyond the specification’s valid-program rules.
- **Authority:** Roslyn implementation.
- **Confidence:** High.
- **Likely primary source:** Roslyn syntax API and compiler diagnostic documentation.

### CS-DIA-002 — Unresolved, inaccessible, and erroneous symbols

- **Distinct behavior:** A compilation may retain error-type/error-symbol structure for missing references, wrong arity, inaccessible members, or invalid constructions so later diagnostics can proceed.
- **Observable content:** Missing type/reference; wrong generic arity; inaccessible type; member on error receiver; unresolved alias.
- **Variants and failures:** Candidate-symbol sets for overload/access failures; cascading errors; metadata reference missing versus namespace missing; IDE speculative/incomplete state.
- **Constraints:** Roslyn semantic model implementation; invalid programs have limited language guarantees.
- **Authority:** Compiler implementation.
- **Confidence:** High.
- **Likely primary source:** Roslyn semantic model APIs and compiler diagnostics.

### CS-DIA-003 — Ambiguous names, calls, and conversions

- **Distinct behavior:** Ambiguity can arise during namespace/type lookup, member lookup, extension lookup, overload resolution, conversion, default-interface resolution, and pattern applicability, with distinct candidate sets.
- **Observable content:** One minimal case for each major ambiguity family and an explicit qualification/cast that resolves it.
- **Variants and failures:** Adding a dependency/global using/overload creates ambiguity; ambiguity may occur only under one target or language version; runtime dynamic ambiguity differs from compile-time ambiguity.
- **Constraints:** Diagnostic text/IDs and candidate reporting are compiler-version dependent.
- **Authority:** Language for invalidity; compiler for diagnostics.
- **Confidence:** High.
- **Likely primary source:** C# language specification name binding and overload resolution; Roslyn diagnostics.

### CS-DIA-004 — Binary/source compatibility mismatch

- **Distinct behavior:** Recompilation can change behavior or fail even when an existing binary consumer still runs, and dependency replacement can fail at runtime despite source compatibility.
- **Observable content:** Dependency/API pairs illustrating inlined constants/optional defaults, added overload ambiguity, changed virtual/member kind, removed member, type movement/forwarding, and default-interface addition.
- **Variants and failures:** `MissingMethodException`, `TypeLoadException`, changed overload after rebuild, stale caller defaults, strong-name/version binding, covariant return runtime support.
- **Constraints:** Requires multiple compilation versions and controlled runtime references.
- **Authority:** Language binding plus CLI/runtime and compiler implementation.
- **Confidence:** High.
- **Likely primary source:** C# versioning guidance; ECMA-335; .NET compatibility documentation.

### CS-DIA-005 — Compiler and language breaking-change boundaries

- **Distinct behavior:** Corrected compiler behavior and new tie-breakers/contextual keywords can change diagnostics or binding for the same text under a newer compiler even with the same declared language version.
- **Observable content:** Version-pinned cases known to differ, alongside explicit disambiguation; record actual compiler/SDK versions with results.
- **Variants and failures:** Specification correction versus feature-version change; warning-only change; runtime-library overload additions; `latest` versus fixed `LangVersion`.
- **Constraints:** Exact historical compiler toolsets must be available to demonstrate each boundary.
- **Authority:** Official compiler implementation and published compatibility notes.
- **Confidence:** Medium.
- **Likely primary source:** Microsoft C# compiler breaking-change documentation and Roslyn release notes.

## Recent language-version changes requiring deliberate coverage

The following changes are especially likely to alter parsing, symbol shape, lowering, overload resolution, or diagnostics in projects that otherwise look similar:

| Version | Material feature families |
|---|---|
| C# 8 | Nullable reference types, async streams/async disposal, default interface implementations, switch expressions and expanded patterns, indices/ranges, using declarations, static local functions, unmanaged constructed types |
| C# 9 | Records, init-only setters, top-level statements, native-sized integers, function pointers, module initializers, covariant returns, target-typed `new`, static lambdas, expanded partial methods |
| C# 10 | Global/file-scoped namespaces, lambda improvements, record structs, parameterless struct constructors, interpolated-string handlers, caller argument expressions, extended property patterns, constant interpolated strings |
| C# 11 | Required members, static abstract interface members, generic attributes, raw/UTF-8 strings, list patterns, checked user operators, ref fields and scoped ref safety, unsigned right shift, auto-default structs, file-local types |
| C# 12 | Primary constructors, collection expressions/builders, inline arrays, aliases to broader type forms, default lambda parameters, `ref readonly` parameters, experimental interceptors in SDK-controlled preview contexts |
| C# 13 | Params collections, specialized `System.Threading.Lock`, ref/unsafe relaxation in async and iterators, ref-struct interface implementation and `allows ref struct`, partial properties/indexers, implicit index access in object initializers, newer escape refinements |
| Post-C# 13 / preview-dependent | Extension blocks/members, `field`-backed property syntax, null-conditional assignment, partial constructors/events, additional span conversions and overload tie-breakers; exact released version and final rules must be verified |

Projects should pin `LangVersion` and compiler SDK when a case relies on any row above. “Latest” and “preview” are moving inputs, not reproducible baselines.

## Implementation-defined, unspecified, or deliberately unstable areas

- Names and exact shapes of synthesized types, fields, methods, closure classes, async/iterator state machines, anonymous types, top-level containers, and file-local metadata names.
- Whether noncapturing delegates are cached, how generic code is shared, whether constrained calls box, and most optimization/inlining/tail-call decisions, provided observable language behavior is preserved.
- Reflection enumeration order, generated member ordering, metadata token assignment, local-slot allocation, and exact diagnostic wording/order.
- Static initialization timing where `beforefieldinit` permits latitude; module initializer ordering beyond guaranteed constraints; finalizer timing and thread.
- Thread scheduling, synchronization-context/task scheduler behavior, memory reordering allowed by the CLI/runtime memory model, and process architecture effects.
- Stack versus heap allocation optimizations for spans/closures/state machines where escape semantics remain valid.
- Filesystem case sensitivity, path forms, newline/encoding handling outside specified compiler decoding, environment-dependent glob and native-library resolution.
- Native ABI layout details, endianness, pointer size, calling convention details, COM availability, and platform-specific exception behavior.
- NuGet dependency choice when ranges/floating versions or multiple sources are used; MSBuild target ordering not constrained by declared dependencies; package scripts/build targets supplied by third parties.
- Analyzer and source-generator scheduling, concurrency, internal caching, and diagnostic order. Generator outputs should be deterministic by contract if reproducibility matters, but hosts do not assign semantic meaning to incidental execution order.
- Runtime reflection, loading, and dynamic-binding behavior after trimming, single-file bundling, Native AOT, or custom AssemblyLoadContext policies.

## Known knowledge limits and verification priorities

- C# features finalized after C# 13 need verification against the exact SDK chosen for the project. In particular, extension blocks, `field`, null-conditional assignment, partial constructors/events, and newer implicit Span conversions may have changed syntax or release status.
- The detailed C# 13 ref-safety rules, especially safe-context joins and ref-like use around suspension points, are intricate and should be checked against the adopted feature specifications and Roslyn conformance tests.
- Historical breaking changes in overload resolution and nullable-flow analysis are too numerous to enumerate reliably from memory. Compiler release notes and compatibility documents should identify high-value version-pair cases.
- Expression-tree prohibitions evolve as the language grows. Verify the current diagnostic matrix rather than assuming every newer expression is rejected or represented.
- COM interop, strong naming on non-Windows platforms, older .NET Framework binding, and alternate CLI implementation behavior are lower-confidence areas here.
- MSBuild design-time build behavior and project-system-specific generated items can differ between command-line SDK builds and integrated development environments. Verify with the intended host if such behavior is in scope.
- The exact SDK-supplied implicit using set, target-framework predefined constants, analyzer defaults, and reference-pack contents are versioned implementation inputs.
- Script/submission (`.csx`) compilation, interactive directives, and notebook hosts are omitted from the baseline project model. Add them only if the intended meaning of “project” includes host-defined scripting semantics.

## Final coverage audit

The checklist covers source/tokenization and preprocessing; declaration and type identity; type forms; nullable and ref safety; members and initialization; generics and constraints; lookup, conversions, and overload resolution; inheritance/interface/dispatch; expressions, patterns, and dataflow; closures, async, iterators, and lifetime; exceptions and resource handling; attributes/metadata/reflection; assembly and package boundaries; SDK/MSBuild configuration; generated sources and diagnostics; interop and deployment transforms; invalid, ambiguous, and compatibility cases.

Feature families that may still warrant additions after primary-source verification are:

- Host-specific script/notebook semantics, alternate compilers/runtimes, multi-module assemblies, and legacy non-SDK project systems.
- Detailed concurrency/memory-model litmus cases, serialization-framework conventions, dependency-injection conventions, and framework-specific code generation. These are excluded unless common ecosystem behavior, rather than C# semantics, is an explicit coverage goal.
- Rare metadata authored outside C#—custom modifiers, malformed metadata, varargs, function-pointer signatures, type-equivalence metadata, and members C# can consume but cannot declare. These can materially test metadata consumption but should be bounded as interoperability cases.
- Experimental interceptors deserve a separate version-pinned section if enabled. Their source-location-based call substitution is intentionally preview/tooling-sensitive and should not be treated as stable C# 12 behavior.

