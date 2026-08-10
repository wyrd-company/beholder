# C# Research Report: Candidate Checklist for a Code-Graph Reference Project

Independent research pass. Language: C#. Produced from model knowledge only; no
web, repository, or fixture inspection. The checklist enumerates language,
toolchain, build, package, and ecosystem features whose omission would leave
materially distinct semantic behavior unrepresented in a code graph: entity
kinds, identities, relationships, name resolution, types, visibility, dispatch,
source inclusion, and diagnostics.

---

## 1. Baseline Version and Implementation Assumptions

- **Language baseline:** C# 13 as the center of gravity (shipped with .NET 9,
  November 2024), with C# 14 / .NET 10 (November 2025) features included and
  flagged. C# 8–12 features are treated as fully mainstream.
- **Compiler:** Roslyn (`csc` as distributed inside the .NET SDK). Roslyn is
  the only production C# compiler in current use; several checklist items are
  Roslyn behavior rather than specified language behavior and are labeled as
  such.
- **Build system:** MSBuild with SDK-style projects (`Microsoft.NET.Sdk` and
  its derivatives), invoked through the `dotnet` CLI. Legacy (non-SDK)
  `.csproj` and `packages.config` are treated as legacy variants, noted where
  they change behavior.
- **Package manager:** NuGet via `PackageReference`.
- **Runtime baseline:** CoreCLR (.NET 8 LTS / .NET 9 / .NET 10 LTS).
  .NET Framework 4.x and `netstandard2.0` appear as multi-targeting variants
  because they change available language/runtime features.
- **Specification status:** ECMA-334 (the C# standard) trails the shipping
  language by several versions. Features beyond the standardized subset are
  specified by Microsoft's feature specifications maintained in the
  `dotnet/csharplang` repository and documented on Microsoft Learn. "Defined
  by: language" below means ECMA-334 and/or a csharplang feature spec.

---

## 2. Checklist

Legend per item: **Behavior** (what is semantically distinct), **Demonstrated
by** (observable project content), **Variants & failure cases**,
**Constraints** (language/toolchain/platform/package-manager),
**Defined by / Confidence / Source**.

### 2.1 Source Inclusion and Compilation Units (SRC)

#### SRC-01 — SDK-style implicit compile item globbing

- **Behavior:** SDK projects compile `**/*.cs` under the project directory by
  default, excluding `bin/` and `obj/`. The set of compiled sources is a build
  artifact, not a directory listing: `Compile Remove`/`Compile Include`,
  `EnableDefaultCompileItems=false`, and `Link`ed files outside the project
  cone all change which files contribute symbols.
- **Demonstrated by:** A project containing a `.cs` file excluded via
  `<Compile Remove>`, a file included from outside the project directory with
  `Link` metadata, and stale `.cs` files under `obj/` that must not be
  scanned as user source.
- **Variants & failure cases:** Duplicate include (file matched by glob and
  explicit `Compile Include`) causes build error NETSDK1022 unless defaults
  disabled; legacy csproj lists every file explicitly.
- **Constraints:** MSBuild SDK-style projects; `dotnet` CLI or MSBuild.
- **Defined by:** Build system (MSBuild SDK). **Confidence:** high.
  **Source:** Microsoft Learn .NET SDK/MSBuild documentation.

#### SRC-02 — Partial types and partial members across files

- **Behavior:** One type declared in multiple files (and multiple parts per
  file) forms a single symbol whose members, base list, attributes, and
  interfaces are merged. Partial methods (declaration + optional or required
  implementation), partial properties/indexers (C# 13), and partial
  constructors/events (C# 14) split a single member's identity across files.
- **Demonstrated by:** A class split across three files plus a
  generator-completed partial member; a C# 9 "extended" partial method with
  return value and accessibility; a partial property with defining and
  implementing declarations in separate files.
- **Variants & failure cases:** Original (void, private, no-out) partial
  methods with no implementation are removed from output entirely — calls to
  them vanish; missing implementation for an extended partial member is an
  error; conflicting modifiers across parts are errors; attribute lists from
  all parts merge.
- **Constraints:** Partial properties/indexers require C# 13; partial
  constructors/events require C# 14.
- **Defined by:** Language. **Confidence:** high. **Source:** ECMA-334 and
  csharplang feature specs.

#### SRC-03 — Top-level statements and the synthesized entry point

- **Behavior:** A single file with top-level statements synthesizes a
  `Program` class in the global namespace containing a `<Main>$` method; local
  functions and type declarations after the statements attach to that scope;
  the implicit `args` parameter and awaited expressions change the synthesized
  signature. `partial class Program` elsewhere augments the synthesized type.
- **Demonstrated by:** A `Program.cs` with top-level statements, an `await`,
  usage of `args`, a local function, and a separate file declaring
  `partial class Program` adding members.
- **Variants & failure cases:** Top-level statements in two files is an error;
  combining with an explicit `Main` entry point is an error (duplicate entry
  point); return values (`return 5;`) change the synthesized return type.
- **Constraints:** C# 9+; only one file per compilation.
- **Defined by:** Language (synthesized names are Roslyn convention).
  **Confidence:** high. **Source:** csharplang top-level statements spec.

#### SRC-04 — Explicit entry points and StartupObject selection

- **Behavior:** The entry point is a static `Main` with one of several
  permitted signatures (including `async Task`/`Task<int>` forms). Multiple
  candidate `Main` methods require `<StartupObject>` to disambiguate — build
  configuration, not source, decides which method is the root.
- **Demonstrated by:** Two classes with valid `Main` methods and a csproj
  `StartupObject` naming one; an `async Task<int> Main`.
- **Variants & failure cases:** No entry point in an `Exe` project is an
  error; `Main` in a generic type is not a candidate; `async void Main` is
  not a valid entry point.
- **Constraints:** `OutputType=Exe`; async Main requires C# 7.1+.
- **Defined by:** Language + build system. **Confidence:** high.
  **Source:** ECMA-334; Microsoft Learn C# docs.

#### SRC-05 — Conditional compilation

- **Behavior:** `#if/#elif/#else/#endif` with symbols from `#define`,
  `<DefineConstants>`, configuration defaults (`DEBUG;TRACE` in Debug, `TRACE`
  in Release), target-framework symbols (`NET9_0`, `NET8_0_OR_GREATER`,
  `NETSTANDARD2_0`, `NETFRAMEWORK`), and OS symbols for OS-specific TFMs
  (`WINDOWS`, `ANDROID`, versioned `_OR_GREATER` forms). Different builds of
  the same file contain different declarations, members, and call sites.
- **Demonstrated by:** A file whose class has an extra member under
  `#if DEBUG`, and a different method body per TFM in a multi-targeted
  project; a custom symbol defined only in one configuration.
- **Variants & failure cases:** `#error`/`#warning` directives; excluded
  regions may contain non-parsing text; symbols are boolean-only (no values);
  `#define` must precede all tokens in the file.
- **Constraints:** Symbol sets are SDK/TFM-dependent; `true`/`false` literals
  allowed in conditions.
- **Defined by:** Language (directives) + build system (symbol injection).
  **Confidence:** high. **Source:** ECMA-334; Microsoft Learn SDK docs.

#### SRC-06 — Namespace declaration forms and file organization

- **Behavior:** Block-scoped namespaces (nestable, multiple per file, reopened
  across files) versus file-scoped namespaces (one per file, C# 10). The same
  namespace spans arbitrary files and assemblies; a file can contribute types
  to several namespaces; nested namespace declarations (`namespace A.B`)
  create the full chain.
- **Demonstrated by:** One file with two block namespaces plus a nested one; a
  file-scoped namespace file; the same namespace reopened in two projects.
- **Variants & failure cases:** Mixing file-scoped with block namespaces in
  one file is an error; types outside any namespace live in the global
  namespace.
- **Constraints:** File-scoped namespaces require C# 10.
- **Defined by:** Language. **Confidence:** high. **Source:** ECMA-334.

#### SRC-07 — Global usings and ImplicitUsings

- **Behavior:** `global using` directives (in any file) apply to the whole
  compilation. `<ImplicitUsings>enable</ImplicitUsings>` causes the SDK to
  generate `obj/.../*.GlobalUsings.g.cs` with an SDK-family-dependent set
  (e.g., `System`, `System.Linq`, `System.Threading.Tasks` for the base SDK;
  ASP.NET Core namespaces added by the Web SDK). Name resolution in every file
  depends on generated, non-checked-in source.
- **Demonstrated by:** A project with ImplicitUsings on, one explicit
  `global using` and one `global using static`, plus a file that compiles only
  because of those directives; `<Using Include/Remove>` items in the csproj.
- **Variants & failure cases:** Duplicate global usings warn; `global using`
  must precede non-global usings in its file; alias forms
  (`global using X = ...`) participate.
- **Constraints:** C# 10+; implicit set differs per SDK
  (`Microsoft.NET.Sdk` vs `.Web` vs `.Worker` vs `.WindowsDesktop`).
- **Defined by:** Language (global using) + build system (implicit set).
  **Confidence:** high. **Source:** csharplang spec; Microsoft Learn SDK docs.

#### SRC-08 — Build-generated sources in obj/

- **Behavior:** The compilation includes generated files never present in the
  source tree: `AssemblyInfo.g.cs` (assembly attributes from MSBuild
  properties), `*.AssemblyAttributes.cs` (`TargetFrameworkAttribute`),
  `GlobalUsings.g.cs`, and source-generator output (on disk only when
  `EmitCompilerGeneratedFiles=true`, under
  `obj/generated/<assembly>/<generator>/`).
- **Demonstrated by:** Assembly-level attributes observable in the graph that
  exist in no user file; a project setting `GenerateAssemblyInfo=false` with a
  hand-written `AssemblyInfo.cs` (the alternative path); duplicate-attribute
  errors when both exist.
- **Variants & failure cases:** Version properties (`Version`,
  `AssemblyVersion`, `InformationalVersion`) surface as attributes;
  deterministic paths differ per TFM in multi-targeted builds.
- **Constraints:** SDK-style projects.
- **Defined by:** Build system. **Confidence:** high. **Source:** Microsoft
  Learn .NET SDK documentation.

#### SRC-09 — Line, pragma, and nullable directives

- **Behavior:** `#line` (renumber, `#line hidden`, and C# 10 fine-grained
  span form) remaps diagnostics and sequence points to other files — generated
  code (e.g., Razor) claims positions in original sources. `#pragma warning
  disable/restore` toggles diagnostics per region. `#nullable
  enable/disable/restore` (with `warnings`/`annotations` variants) changes
  nullability semantics mid-file.
- **Demonstrated by:** A file using `#nullable disable` around one member in
  an otherwise nullable-enabled project; a generated-style file with `#line`
  pointing at another file; region-scoped pragma suppression.
- **Variants & failure cases:** Diagnostics reported "in" a file that is not
  the syntactic file; `#nullable` overrides the project setting only within
  its span.
- **Constraints:** Fine-grained `#line` requires C# 10.
- **Defined by:** Language. **Confidence:** high. **Source:** ECMA-334;
  csharplang enhanced-line-directive spec.

#### SRC-10 — File-local types (`file` modifier)

- **Behavior:** `file class`/`struct`/etc. (C# 11) is visible only within its
  declaring file. Two files in one project may declare `file class Helper`
  with the same source name and namespace — distinct types whose metadata
  names are mangled with a file identifier. Identity cannot be derived from
  (namespace, name) alone.
- **Demonstrated by:** Two files each declaring `file class Impl` in the same
  namespace, each used by a non-file type in its own file.
- **Variants & failure cases:** File-local types cannot be member types of
  non-file types' signatures that are more visible; commonly emitted by source
  generators to avoid collisions.
- **Constraints:** C# 11+.
- **Defined by:** Language (mangling is Roslyn convention).
  **Confidence:** high. **Source:** csharplang file-local types spec.

#### SRC-11 — One source file compiled into multiple projects

- **Behavior:** Via `Compile Include` with `Link`, shared projects
  (`.shproj`), or plain multi-targeting, a single physical file produces
  distinct symbols in distinct assemblies/compilations, potentially with
  different `#if` shapes and different resolved references per context.
- **Demonstrated by:** A `Shared/Util.cs` linked into two projects where a
  `#if` symbol differs, giving the "same" class different members per project.
- **Variants & failure cases:** File-to-symbol mapping is one-to-many; IDE
  "current context" selection is a convention, not semantics.
- **Constraints:** MSBuild.
- **Defined by:** Build system. **Confidence:** high. **Source:** Microsoft
  Learn MSBuild documentation.

#### SRC-12 — Scripting dialect and file-based programs

- **Behavior:** C# script files (`.csx`) form a different dialect: `#r` and
  `#load` directives, global statements anywhere, no namespace declarations,
  submission-chained scoping (Roslyn scripting/`dotnet-script`). Separately,
  .NET 10 "file-based apps" allow `dotnet run app.cs` with `#:package`,
  `#:sdk`, and `#:property` directives in a plain `.cs` file — project
  metadata embedded in source.
- **Demonstrated by:** A `.csx` with `#load` of another script and `#r` of a
  package/assembly; a single-file app with `#:package` directives.
- **Variants & failure cases:** `.csx` semantics (member lookup on the script
  type, top-level variables as fields) differ from ordinary C#; file-based
  apps convert to full projects (`dotnet project convert`).
- **Constraints:** `.csx` requires a scripting host (csi, dotnet-script);
  file-based apps require .NET 10 SDK.
- **Defined by:** Compiler/ecosystem (csx); .NET SDK (file-based apps).
  **Confidence:** medium (directive details). **Source:** Roslyn scripting
  docs; Microsoft Learn .NET 10 SDK documentation.

### 2.2 Assemblies, Projects, and Cross-Assembly Identity (ASM)

#### ASM-01 — Assembly identity independent of source layout

- **Behavior:** Assembly name, root namespace, project file name, and folder
  name are four independent things. `AssemblyName` and `RootNamespace` can
  each differ from the project name; namespaces need not match either.
  Assembly identity comprises name, version, culture, and optional public key
  token (strong naming).
- **Demonstrated by:** A project whose `AssemblyName` and `RootNamespace`
  both differ from the `.csproj` name and folder, with namespaces unrelated to
  the root namespace; assembly-level attributes (`[assembly: ...]`) placed in
  an arbitrary source file.
- **Variants & failure cases:** Strong-named assembly (`SignAssembly` +
  `.snk`); `AssemblyVersion` vs `FileVersion` vs `InformationalVersion`
  diverge; two projects producing the same assembly name in one dependency
  graph cause conflicts.
- **Constraints:** MSBuild properties; strong naming optional on .NET Core.
- **Defined by:** Build system + runtime. **Confidence:** high.
  **Source:** Microsoft Learn assembly documentation.

#### ASM-02 — ProjectReference graphs

- **Behavior:** `ProjectReference` creates compile-time symbol visibility
  across projects (public/protected surface, plus internal with friendship).
  References are acyclic; transitive project references flow by default
  (compile-time transitivity of `ProjectReference` assets). Metadata like
  `ReferenceOutputAssembly=false` creates a build dependency with no symbol
  edge.
- **Demonstrated by:** Three projects A→B→C where A uses a C type through
  transitive flow; a reference with `ReferenceOutputAssembly=false` whose
  types are not resolvable.
- **Variants & failure cases:** Circular ProjectReference is a build error
  (while circular type references within one assembly are fine); TFM
  compatibility checks (NU1201-style errors) when referencing an incompatible
  target.
- **Constraints:** MSBuild/NuGet restore.
- **Defined by:** Build system. **Confidence:** high. **Source:** Microsoft
  Learn MSBuild documentation.

#### ASM-03 — InternalsVisibleTo (friend assemblies)

- **Behavior:** `[assembly: InternalsVisibleTo("Friend")]` makes `internal`
  (and `private protected` via inheritance rules) members resolvable from a
  named other assembly. Accessibility is thus not derivable from the member
  alone — it depends on assembly-level attributes of the *declaring*
  assembly and the identity of the consumer.
- **Demonstrated by:** A library exposing internals to a test project that
  calls an internal method and implements an internal interface; the attribute
  supplied either in source or via `<InternalsVisibleTo>` MSBuild item.
- **Variants & failure cases:** Strong-named assemblies require the friend
  declaration to include the full public key; a non-friend assembly consuming
  the same member is a compile error (CS0122).
- **Constraints:** None beyond strong-name key rules.
- **Defined by:** Language/runtime attribute honored by compiler.
  **Confidence:** high. **Source:** Microsoft Learn API docs for
  InternalsVisibleToAttribute.

#### ASM-04 — Type forwarding

- **Behavior:** `[assembly: TypeForwardedTo(typeof(T))]` makes an assembly
  that no longer defines `T` redirect references to the assembly that does.
  Compile-time references recorded against one assembly resolve at runtime in
  another; the BCL uses this pervasively (`netstandard` facades,
  `System.Runtime` vs `System.Private.CoreLib`).
- **Demonstrated by:** Consuming a BCL type whose reference assembly forwards
  to the implementation; a small library refactoring that moves a type and
  forwards it to preserve binary compatibility.
- **Variants & failure cases:** Graph identity of a type must be stable across
  the forward; missing forward target is a runtime `TypeLoadException`.
- **Constraints:** Runtime + compiler support.
- **Defined by:** Runtime/metadata (CLI standard). **Confidence:** high.
  **Source:** ECMA-335; Microsoft Learn documentation.

#### ASM-05 — Reference assemblies vs implementation assemblies

- **Behavior:** Compilation binds against *reference assemblies* (API-only,
  bodies stripped, sometimes with internals removed) from targeting packs and
  NuGet `ref/` folders; the runtime loads different implementation assemblies.
  Builds also emit their own `ref` assembly when
  `ProduceReferenceAssembly=true` (default in recent SDKs) to enable
  incremental compilation.
- **Demonstrated by:** A project whose external symbol information comes from
  the .NET targeting pack; observing that private members of BCL types are
  absent from the compile-time view.
- **Variants & failure cases:** Internal members visible at runtime but not in
  ref assemblies; mismatched ref/impl surface is possible with hand-built
  packages.
- **Constraints:** .NET SDK; NuGet package conventions (`ref/`, `lib/`).
- **Defined by:** Build system + NuGet conventions. **Confidence:** high.
  **Source:** Microsoft Learn documentation on reference assemblies.

#### ASM-06 — extern alias

- **Behavior:** Two references can define the same fully qualified type name.
  `Aliases` metadata on a reference plus `extern alias X;` in source creates a
  named root (`X::Some.Type`) so both can be consumed in one file. Resolution
  becomes reference-identity-sensitive, not just name-based.
- **Demonstrated by:** Two versions/forks of a library referenced with
  aliases, one file consuming a same-named type from each via `A::` / `B::`.
- **Variants & failure cases:** Without aliases, using the duplicate name is
  CS0433 (type exists in both); `global::` is the implicit default alias.
- **Constraints:** `Aliases` metadata on PackageReference/ProjectReference/
  Reference items.
- **Defined by:** Language + build system. **Confidence:** high.
  **Source:** ECMA-334 (extern alias); Microsoft Learn.

#### ASM-07 — Multi-targeting: one project, several compilations

- **Behavior:** `<TargetFrameworks>net9.0;netstandard2.0</TargetFrameworks>`
  produces one compilation per TFM from the same sources, with different
  defines, different references, different default LangVersion, and possibly
  different item sets (`Condition="'$(TargetFramework)'=='...'"`). One source
  declaration maps to N distinct assembly-level symbols whose members can
  differ.
- **Demonstrated by:** A multi-targeted library where a method exists only for
  one TFM under `#if`, a PackageReference conditioned on TFM, and a public API
  whose parameter type differs per TFM.
- **Variants & failure cases:** `netstandard2.0` caps default LangVersion at
  7.3 (modern features error without explicit LangVersion override, and some
  need runtime support regardless); OS-specific TFMs (`net9.0-windows`) add
  platform analyzers and symbols.
- **Constraints:** MSBuild inner/outer build model.
- **Defined by:** Build system. **Confidence:** high. **Source:** Microsoft
  Learn multi-targeting documentation.

#### ASM-08 — Compiler-generated metadata names vs source names

- **Behavior:** Metadata identity diverges from source identity: property
  accessors `get_X`/`set_X`, event accessors `add_E`/`remove_E`, operators
  `op_Addition` etc., constructors `.ctor`/`.cctor`, backing fields
  `<X>k__BackingField`, closures `<>c__DisplayClass*`/`<>c`, lambda and local
  function methods (`<M>b__*`, `<M>g__Name|*`), iterator/async state machines
  `<M>d__N` (linked to source methods by `IteratorStateMachine`/
  `AsyncStateMachine` attributes), record clone `<Clone>$`, top-level
  `<Main>$`, anonymous types `<>f__AnonymousType*`, generic arity suffixes
  (`` List`1 ``), and nested-type `+` separators.
- **Demonstrated by:** Any project using properties, lambdas with captures,
  iterators, async methods, records, and nested generics — the graph must
  either unify or deliberately distinguish source symbols and their metadata
  shadows.
- **Variants & failure cases:** A graph built from IL sees synthesized types
  absent from source; one built from syntax misses them; `CompilerGenerated`
  attribute marks most synthesized members.
- **Constraints:** Name mangling is unspecified by the language; stable in
  practice per Roslyn version.
- **Defined by:** Compiler (Roslyn convention) over CLI metadata rules.
  **Confidence:** high. **Source:** Roslyn repository documentation; ECMA-335
  for special names.

#### ASM-09 — Consuming assemblies produced by other languages

- **Behavior:** C# projects routinely reference VB.NET and F# assemblies. The
  consumer sees CLI metadata: F# modules appear as static classes, F# union
  cases as nested types, curried functions as `FSharpFunc`-typed members; VB
  case-insensitive declarations arrive with fixed casing; VB default
  properties surface as indexers via `DefaultMemberAttribute`.
- **Demonstrated by:** A solution with a small F# or VB library referenced by
  the C# project, calling into a module function and a class member.
- **Variants & failure cases:** Language-specific constructs with no C#
  spelling (F# unit, byref-like conventions, VB `Module`s) still occupy the
  symbol graph; optional-parameter and attribute encodings differ subtly.
- **Constraints:** Multi-language solution; single-language graphs must still
  represent the foreign assembly's surface.
- **Defined by:** CLI metadata + each language's compiler.
  **Confidence:** medium. **Source:** ECMA-335; language documentation.

### 2.3 Name Resolution and Identifiers (RES)

#### RES-01 — The full family of using directives

- **Behavior:** `using N;` (namespace import), `using static T;` (imports
  static members and nested types — including making contained extension
  methods available), `using X = Some.Type;` (alias), `global using` variants,
  and C# 12 "alias any type" (`using Point = (int X, int Y);`,
  `using unsafe P = int*;`). Aliases create alternate names for the same
  symbol; tuple aliases carry element names.
- **Demonstrated by:** One file exercising each form, including an alias to a
  generic instantiation (`using IntList = List<int>;`), a tuple alias, and a
  `using static` that enables both a constant and an extension method.
- **Variants & failure cases:** Usings are per-file (except global); an alias
  is not usable in its own definition; alias to pointer types requires
  `unsafe` modifier on the alias; namespace-vs-type name collisions resolved
  by context.
- **Constraints:** Alias-any-type requires C# 12.
- **Defined by:** Language. **Confidence:** high. **Source:** ECMA-334;
  csharplang alias-any-type spec.

#### RES-02 — Scoped lookup, shadowing, and the global:: qualifier

- **Behavior:** Lookup proceeds through nested scopes (locals, parameters,
  type members, base types, enclosing types, namespaces, usings). Inner
  declarations shadow outer ones (locals over fields, type parameters over
  outer types, nested types over usings). `global::` bypasses all imports to
  the root namespace — required when a user type named `System` or a
  same-named namespace hijacks resolution.
- **Demonstrated by:** A field shadowed by a local and disambiguated with
  `this.`; a type parameter `T` shadowing an outer class `T`; a user namespace
  segment that collides with `System` fixed via `global::System`.
- **Variants & failure cases:** Two imported namespaces exposing the same type
  name make the simple name ambiguous (CS0104) — qualification or alias
  required; "closer" using scope in nested namespaces wins over outer.
- **Constraints:** None.
- **Defined by:** Language. **Confidence:** high. **Source:** ECMA-334.

#### RES-03 — Source vs referenced-assembly type conflicts

- **Behavior:** When a compilation defines a type with the same fully
  qualified name as one in a referenced assembly, the source definition wins
  with warning CS0436. When two *referenced* assemblies both define the name,
  use is an error (CS0433) unless disambiguated by extern alias. Identity is
  therefore (assembly, namespace, name, arity), not name alone.
- **Demonstrated by:** A project that redefines a type also present in a
  referenced library and uses it; a second scenario with the conflict between
  two references.
- **Variants & failure cases:** Common in practice with polyfills
  (`Index`/`Range`/`IsExternalInit` recompiled for downlevel TFMs);
  `System.Private.CoreLib` vs `System.Runtime` unification handled by
  compiler.
- **Constraints:** None.
- **Defined by:** Compiler behavior over language rules.
  **Confidence:** high. **Source:** Roslyn documentation; compiler
  diagnostics reference.

#### RES-04 — Extension method lookup scope

- **Behavior:** Classic extension method invocation resolves by searching
  enclosing namespace declarations outward, then using-imported namespaces per
  scope — the *call site's* imports decide which extension wins, so identical
  call syntax binds differently per file. Instance members always beat
  extensions.
- **Demonstrated by:** Two namespaces each defining `Widget.Frob()`
  extensions; two files calling `w.Frob()` with different usings binding to
  different methods; a third case where an instance `Frob` preempts both.
- **Variants & failure cases:** Ambiguity when two applicable extensions are
  in the same scope tier; extension invocation on `dynamic` receivers is not
  supported; extensions on `this` in structs take `ref this` only via
  `ref this` extension methods on structs (C# 7.2 `ref`/`in` extension
  receivers for value types).
- **Constraints:** None (classic form since C# 3).
- **Defined by:** Language. **Confidence:** high. **Source:** ECMA-334.

#### RES-05 — Symbol references outside executable code: nameof, typeof, cref

- **Behavior:** `nameof(x)` compiles to a constant string but *binds* its
  argument (a real reference edge with no runtime dependency); C# 11 extends
  scope so method attributes can name parameters; C# 14 allows unbound generic
  types (`nameof(List<>)`). `typeof` references types including open generics
  (`typeof(Dictionary<,>)`). XML documentation `cref` attributes bind symbols
  and emit warnings when unresolvable (with `GenerateDocumentationFile`).
- **Demonstrated by:** `nameof` on members, parameters, and (C# 14) unbound
  generics; `typeof` of open and closed generics; `///` docs with valid and
  deliberately invalid `cref` targets and the resulting CS1574 warnings.
- **Variants & failure cases:** `nameof` yields only the final identifier
  (`nameof(A.B.C)` == "C"); crefs use a distinct ID string format in emitted
  XML (`T:`, `M:` prefixes with encoded signatures).
- **Constraints:** Doc warnings require documentation generation enabled;
  unbound-generic nameof requires C# 14.
- **Defined by:** Language. **Confidence:** high (medium for C# 14 detail).
  **Source:** ECMA-334; csharplang specs.

#### RES-06 — Identifier spellings: @-verbatim and Unicode escapes

- **Behavior:** `@class` and `class` name the same identifier text with the
  `@` allowing keyword collision; Unicode escape sequences in identifiers
  (`class` is invalid as keyword evasion, but `_`-style escapes in
  ordinary identifiers are the same identifier as the literal spelling).
  Multiple source spellings map to one symbol name.
- **Demonstrated by:** A type named `@event` consumed elsewhere as `@event`;
  an identifier written once literally and once with a Unicode escape,
  referring to the same symbol.
- **Variants & failure cases:** Contextual keywords (`var`, `record`,
  `nameof`, `field`) remain valid identifiers, creating parse-context
  sensitivity — e.g., a user type named `var` changes what `var x` means.
- **Constraints:** None.
- **Defined by:** Language. **Confidence:** high. **Source:** ECMA-334
  lexical grammar.

### 2.4 Type Declarations and Members (TY)

#### TY-01 — The full set of type declaration kinds and modifiers

- **Behavior:** `class`, `struct`, `interface`, `enum`, `delegate`,
  `record` (class), `record struct`, `readonly record struct`, plus modifiers
  `static`, `abstract`, `sealed`, `partial`, `ref` (structs), `readonly`
  (structs), `file`. Each kind has distinct member capabilities, default base
  types (`object`, `ValueType`, `Enum`, `MulticastDelegate`), and identity
  rules.
- **Demonstrated by:** At least one of each kind, including a static class
  (only static members, no instantiation), an abstract class, and a delegate
  type used as a field/parameter type.
- **Variants & failure cases:** Static classes cannot be type arguments or
  base types; enums/delegates cannot be declared partial? (they cannot);
  structs cannot inherit; interfaces cannot hold instance state.
- **Constraints:** Record structs require C# 10.
- **Defined by:** Language. **Confidence:** high. **Source:** ECMA-334.

#### TY-02 — Records and their synthesized members

- **Behavior:** Records synthesize: value-based `Equals(R)`/`Equals(object)`,
  `GetHashCode`, `operator ==`/`!=`, `ToString` + virtual protected
  `PrintMembers`, an `EqualityContract` property (record classes), a copy
  constructor plus `<Clone>$` enabling `with` expressions, and — for
  positional records — public init-only properties and a `Deconstruct` per
  primary-constructor parameter. Record inheritance is record-to-record only.
- **Demonstrated by:** A positional record, a record with a body member
  overriding a synthesized one (user-defined `ToString` suppressing
  synthesis), a derived record, a `with` expression, `record struct`
  (mutable properties by default, no clone/EqualityContract) and
  `readonly record struct`.
- **Variants & failure cases:** `sealed override ToString` (C# 10); attribute
  targeting on positional parameters (`[property: X] [param: Y]`); manual
  members with matching signatures replace synthesis; positional parameter
  used as a field instead when a member of that name exists.
- **Constraints:** C# 9 (classes) / C# 10 (structs); `IsExternalInit` needed
  on downlevel TFMs.
- **Defined by:** Language. **Confidence:** high. **Source:** csharplang
  records spec.

#### TY-03 — Primary constructors on non-record classes and structs

- **Behavior:** C# 12 primary constructor parameters are in scope throughout
  the type body; when used in member bodies they are captured into
  compiler-synthesized private fields (not properties, unlike records). Other
  constructors must chain to the primary via `this(...)`. Base-clause
  arguments (`class D(int x) : B(x)`) create constructor-call edges in the
  base list.
- **Demonstrated by:** A class with a primary constructor whose parameter is
  used in a method (capture) and also passed to the base class; a manual
  member shadowing the parameter name.
- **Variants & failure cases:** "Double storage" warning when a parameter is
  both passed to base and captured; parameter shadowed by a member — lookup
  prefers member in some positions (initializers use the parameter);
  struct primary constructors interact with definite assignment.
- **Constraints:** C# 12.
- **Defined by:** Language. **Confidence:** high. **Source:** csharplang
  primary constructors spec.

#### TY-04 — Property forms

- **Behavior:** Auto-properties (with synthesized backing fields), full
  properties, expression-bodied accessors, `init` accessors (metadata modreq
  `IsExternalInit`), `required` members, accessor-level accessibility
  (`public int X { get; private set; }`), static/virtual/abstract/override
  properties, and the `field` contextual keyword (C# 14; preview in C# 13)
  which introduces a semi-synthesized backing field inside accessor bodies.
- **Demonstrated by:** Each form, plus a property whose getter and setter have
  different accessibility, and one using `field`.
- **Variants & failure cases:** `field` changes meaning for code that also has
  a real field named `field` (breaking-change dance, escape via `@field` /
  `this.field`); init-only assignable in object initializers, constructors,
  and `with`.
- **Constraints:** `field` requires C# 14 (or C# 13 preview LangVersion).
- **Defined by:** Language. **Confidence:** high (medium on exact C# 13
  preview gating). **Source:** csharplang field-keyword spec.

#### TY-05 — Events

- **Behavior:** Field-like events synthesize add/remove accessors and a
  hidden delegate field; custom events declare explicit `add`/`remove`.
  Invocation is only permitted inside the declaring type (outside, only
  `+=`/`-=` binds). Metadata members `add_E`/`remove_E` and the event member
  itself are distinct entities.
- **Demonstrated by:** A field-like event raised via `E?.Invoke(...)`, a
  custom-accessor event, an interface event implemented explicitly, and an
  external subscriber using `+=` with a method group and a lambda.
- **Variants & failure cases:** Abstract/interface events; static events;
  partial events (C# 14); `+=` with method group creates a delegate
  reference edge to the handler.
- **Constraints:** Partial events require C# 14.
- **Defined by:** Language. **Confidence:** high. **Source:** ECMA-334.

#### TY-06 — Indexers, operators, and user-defined conversions

- **Behavior:** Indexers (`this[...]`, metadata `Item`/`get_Item` unless
  renamed by `IndexerName`), overloaded operators (`op_Addition` etc.),
  `true`/`false` operators enabling `&&`/`||` lifting, implicit/explicit
  conversion operators (which then silently participate at call sites and in
  overload resolution), and C# 11 `checked` operator variants
  (`op_CheckedAddition`) selected by `checked` context.
- **Demonstrated by:** A type with an indexer (plus an overloaded indexer), an
  arithmetic operator pair (checked and unchecked forms), and an implicit
  conversion exercised implicitly at an assignment and a call argument.
- **Variants & failure cases:** Conversion operators create invisible call
  edges; operators must be public static on the declaring type; C# 14 adds
  instance user-defined compound assignment operators (`operator +=`) and
  extension operators.
- **Constraints:** Checked operators C# 11; compound-assignment operators
  C# 14.
- **Defined by:** Language. **Confidence:** high (medium for C# 14 operator
  details). **Source:** ECMA-334; csharplang specs.

#### TY-07 — Nested types and the full accessibility lattice

- **Behavior:** Types nest arbitrarily (including generic-in-generic);
  accessibility spans `public`, `internal`, `protected`, `private`,
  `protected internal` (union), `private protected` (intersection), with
  `private`/`protected` only valid on nested types. Effective accessibility
  is the intersection with all enclosing types'.
- **Demonstrated by:** A `private protected` member consumed from a derived
  type in the same assembly and rejected from a derived type in a friend-less
  other assembly; a public type nested inside an internal type (effectively
  internal); generic outer with generic nested type used as
  `Outer<int>.Inner<string>`.
- **Variants & failure cases:** Nested type accessing enclosing type's
  privates; name lookup preferring nested types over inherited/imported names.
- **Constraints:** `private protected` requires C# 7.2.
- **Defined by:** Language. **Confidence:** high. **Source:** ECMA-334.

#### TY-08 — Struct-specific semantics

- **Behavior:** Structs: implicit parameterless construction (`default`),
  C# 10 explicit parameterless constructors and field initializers, C# 11
  auto-default of unassigned fields in constructors, `readonly struct` and
  per-member `readonly`, `ref struct` (stack-only; cannot box, cannot be a
  field of a class, historically no interfaces), C# 11 `ref` fields with
  `scoped`/`[UnscopedRef]` lifetime annotations, `fixed`-size buffers
  (unsafe), and C# 12 `[InlineArray]` types with element-access sugar.
- **Demonstrated by:** A `readonly struct`, a `ref struct` holding a
  `Span<T>` and a `ref` field, an `[InlineArray(8)]` buffer indexed and
  sliced, a struct with a field initializer and explicit parameterless ctor
  (`new S()` vs `default(S)` differing).
- **Variants & failure cases:** C# 13 lets ref structs implement interfaces
  and be constrained-generic via `allows ref struct` (boxing still
  prohibited; interface dispatch restrictions); defensive copies on
  non-readonly member access through `in` params/readonly fields.
- **Constraints:** Version gates as noted; ref fields need runtime support
  (.NET 7+).
- **Defined by:** Language. **Confidence:** high. **Source:** csharplang
  low-level struct specs.

#### TY-09 — Enums

- **Behavior:** Named constant sets over an underlying integral type
  (default `int`, any integral including `byte`/`ulong` allowed). Members are
  constants (compile-time folded), values may repeat, `[Flags]` is
  convention-only, explicit numeric conversion both directions, and any
  underlying-typed value converts to the enum even without a named member.
- **Demonstrated by:** An enum with explicit underlying type, duplicate
  values, use in a `switch` (with exhaustiveness diagnostics behavior), a
  flags enum combined with `|`, and a cast of an out-of-range value.
- **Variants & failure cases:** Enum constants referenced cross-assembly are
  baked in like consts; `System.Enum` and `System.ValueType` as base-type
  edges; enum constraints (`where T : struct, Enum` / `where T : Enum`).
- **Constraints:** Enum constraint requires C# 7.3.
- **Defined by:** Language. **Confidence:** high. **Source:** ECMA-334.

#### TY-10 — Delegates, method group conversions, and function pointers

- **Behavior:** Delegate declarations create full types with `Invoke`/
  `BeginInvoke` members; method group → delegate conversion creates a
  reference edge without invocation (C# 11 caches such conversions);
  `Delegate`/`MulticastDelegate` are special base types. Unsafe function
  pointers (`delegate* managed<int, void>`, `delegate* unmanaged[Cdecl]<...>`)
  take `&Method` references; `[UnmanagedCallersOnly]` methods are callable
  only via pointer.
- **Demonstrated by:** A custom delegate type, assignment `d = SomeMethod;`,
  multicast `+=`, a `delegate*` field initialized with `&StaticMethod`, and
  an `UnmanagedCallersOnly` export.
- **Variants & failure cases:** Overload selection during method-group
  conversion (target signature drives resolution); C# 10 natural type of
  method groups (`var f = M;` when unique); variance in delegate conversion.
- **Constraints:** Function pointers require C# 9 + `unsafe`;
  UnmanagedCallersOnly requires .NET 5+.
- **Defined by:** Language. **Confidence:** high. **Source:** csharplang
  function-pointers spec; ECMA-334.

#### TY-11 — Anonymous types and tuples

- **Behavior:** Anonymous types (`new { X = 1, Y = "a" }`) are synthesized
  internal types, structurally reused within an assembly for identical
  member name/type/order sequences; they cannot be named in source. Tuples
  (`(int A, string B)`) erase to `System.ValueTuple<...>`; element names live
  only in `TupleElementNamesAttribute` metadata and flow structurally through
  conversions (name mismatches produce warnings, not errors).
- **Demonstrated by:** Two methods creating the same anonymous shape; tuple
  types in a public API with named elements consumed from another project;
  a tuple conversion that drops/renames names; tuples > 7 elements
  (nested `Rest`).
- **Variants & failure cases:** Anonymous types in expression-tree contexts;
  tuple equality operators (C# 7.3) and deconstruction interplay.
- **Constraints:** ValueTuple available in-box from net47/netcoreapp2.0;
  package `System.ValueTuple` for older TFMs.
- **Defined by:** Language + Roslyn (anonymous type naming).
  **Confidence:** high. **Source:** ECMA-334; csharplang tuples spec.

#### TY-12 — const vs static readonly and cross-assembly constant baking

- **Behavior:** `const` values (and enum members, and default parameter
  values) are baked into *consuming* assemblies at compile time — updating
  the defining library without recompiling consumers leaves stale values, and
  no runtime field-access edge exists. `static readonly` produces a real
  field reference resolved at runtime.
- **Demonstrated by:** A library `const` and a `static readonly` consumed from
  another project; a default parameter value defined by a const from a third
  assembly.
- **Variants & failure cases:** `const` limited to compile-time-constant
  types (numeric, bool, char, string, null, enums; `decimal` const is
  compiler-encoded via attribute, not a CLI literal); const folding in
  expressions.
- **Constraints:** None.
- **Defined by:** Language. **Confidence:** high. **Source:** ECMA-334.

#### TY-13 — required members

- **Behavior:** `required` fields/properties (C# 11) impose obligations on
  object creation sites: every `new` must set them in an object initializer
  unless the constructor carries `[SetsRequiredMembers]`. Requiredness is
  part of the API contract, encoded via `RequiredMemberAttribute` +
  `CompilerFeatureRequired` modreq on constructors, and affects which
  constructors are usable cross-assembly from downlevel compilers.
- **Demonstrated by:** A type with required properties constructed with and
  without initializers (error case), and a `[SetsRequiredMembers]`
  constructor bypassing the requirement.
- **Variants & failure cases:** Interaction with `init`; inheritance
  (derived types inherit obligations); older compilers cannot construct such
  types at all.
- **Constraints:** C# 11; attribute types must exist (polyfill on downlevel
  TFMs).
- **Defined by:** Language. **Confidence:** high. **Source:** csharplang
  required-members spec.

### 2.5 Generics (GEN)

#### GEN-01 — Generic arity as part of identity

- **Behavior:** `Foo`, `Foo<T>`, and `Foo<T,U>` are distinct types that can
  coexist in one namespace; metadata names carry arity (`` Foo`1 ``). Generic
  methods overload on arity too. Nested generics accumulate type parameters
  (`Outer<T>.Inner<U>` has combined generic context). Open generic references
  (`typeof(Foo<>)`) versus constructed types (`Foo<int>`) versus the
  definition are distinct graph entities.
- **Demonstrated by:** Same-named types of arities 0/1/2 all used; a nested
  generic instantiated with mixed arguments; `typeof` of both open and
  constructed forms.
- **Variants & failure cases:** Partial declarations must agree on type
  parameter names; arity in `nameof` (C# 14 unbound form).
- **Constraints:** None.
- **Defined by:** Language + CLI metadata. **Confidence:** high.
  **Source:** ECMA-334; ECMA-335.

#### GEN-02 — Constraint clauses

- **Behavior:** `where T : class / class? / struct / notnull / unmanaged /
  new() / BaseClass / IInterface / U / default / allows ref struct` each
  change what members and conversions are available on `T`, which types may
  instantiate it, and nullability analysis. Constraints propagate through
  overriding/implementation (with `default` disambiguating override
  candidates) and are part of the method/type signature semantics.
- **Demonstrated by:** One generic type or method per interesting constraint,
  including `unmanaged` used with pointers/`sizeof`, `new()` invoked, a
  type-parameter-dependent constraint (`where U : T`), and C# 13
  `allows ref struct` instantiated with a `Span<T>`.
- **Variants & failure cases:** Violating a constraint at the use site
  (CS0311-style errors); constraints on local functions and lambdas
  (lambdas: none); `struct` implies `new()`.
- **Constraints:** `notnull` C# 8; `unmanaged` C# 7.3; `allows ref struct`
  C# 13 + .NET 9 runtime.
- **Defined by:** Language. **Confidence:** high. **Source:** ECMA-334;
  csharplang specs.

#### GEN-03 — Variance

- **Behavior:** `out` (covariant) and `in` (contravariant) type parameters on
  interfaces and delegates create assignability edges between different
  constructed types (`IEnumerable<string>` → `IEnumerable<object>`;
  `Action<object>` → `Action<string>`), affecting conversion graphs, overload
  resolution, and pattern matching.
- **Demonstrated by:** A custom covariant interface and contravariant
  delegate exercised by implicit conversions; variance-enabled method group
  and delegate conversions.
- **Variants & failure cases:** Variance is invalid for structs' type
  arguments (no variance conversion involving value types); variant
  parameters restricted to input/output positions (compile errors otherwise);
  C# 9 default interface members interact with variance validity.
- **Constraints:** C# 4+.
- **Defined by:** Language + runtime. **Confidence:** high.
  **Source:** ECMA-334.

#### GEN-04 — Static abstract/virtual interface members and generic math

- **Behavior:** Interfaces may declare `static abstract`/`static virtual`
  members (C# 11), including operators; calls dispatch through a type
  parameter (`T.Parse(...)`, `x + y` where `T : INumber<T>`). This is a
  distinct dispatch mechanism: the receiver is a *type*, resolved per
  instantiation, invisible to naive call graphs.
- **Demonstrated by:** A generic algorithm constrained to
  `IAdditionOperators<T,T,T>`/`IParsable<T>` invoking static interface
  members and operators; a user interface with a `static abstract` factory
  implemented by two types.
- **Variants & failure cases:** Not callable on the interface itself (must go
  through a type parameter); `static virtual` with default bodies; requires
  runtime support (no .NET Framework/netstandard2.0).
- **Constraints:** C# 11 + .NET 7+ runtime.
- **Defined by:** Language + runtime. **Confidence:** high.
  **Source:** csharplang static-abstracts spec; Microsoft Learn generic math
  documentation.

#### GEN-05 — Type inference and target-typing

- **Behavior:** Generic method type arguments are inferred from arguments
  (including lambdas and method groups); target-typed constructs
  (`new()` C# 9, target-typed conditional, collection expressions, `default`)
  take their type from context — the same token sequence binds to different
  symbols depending on the assignment target or parameter type.
- **Demonstrated by:** Calls relying on inference incl. a lambda-shaped
  inference; `Widget w = new(arg);` binding a specific constructor;
  a ternary whose branches only type-check via target-typing; an inference
  failure requiring explicit type arguments.
- **Variants & failure cases:** Inference failures (CS0411); `var` local
  inference; `dynamic` arguments defer inference to runtime.
- **Constraints:** Target-typed `new` C# 9.
- **Defined by:** Language. **Confidence:** high. **Source:** ECMA-334
  type inference chapter; csharplang specs.

### 2.6 Inheritance, Interfaces, and Dispatch (INH)

#### INH-01 — virtual/override/sealed/new slot semantics

- **Behavior:** `virtual` + `override` share one dispatch slot;
  `sealed override` closes it; `new` (or implicit hiding, with warning
  CS0108) starts a *new* slot so both members coexist and are reachable —
  which one a call binds to depends on the static receiver type. A correct
  graph distinguishes overriding (same slot) from hiding (parallel members).
- **Demonstrated by:** A three-level hierarchy with an override chain, a
  `sealed override`, and a `new` method invoked through both base-typed and
  derived-typed receivers showing divergent targets.
- **Variants & failure cases:** Hiding fields/nested types (not just
  methods); `base.M()` non-virtual call edge; overriding with covariant
  return (see INH-05).
- **Constraints:** None.
- **Defined by:** Language. **Confidence:** high. **Source:** ECMA-334.

#### INH-02 — Abstract members and implementation obligations

- **Behavior:** Abstract methods/properties/events/indexers must be overridden
  by non-abstract derived classes; failure is a compile error. Abstract
  members create must-implement relationship edges distinct from virtual
  ones.
- **Demonstrated by:** An abstract base with each abstract member kind, one
  concrete subclass implementing all, one intermediate abstract subclass
  implementing some.
- **Variants & failure cases:** Abstract override (re-abstracting a virtual);
  calls to abstract members are always virtual.
- **Constraints:** None.
- **Defined by:** Language. **Confidence:** high. **Source:** ECMA-334.

#### INH-03 — Interface implementation mapping

- **Behavior:** Interface members map to implementations implicitly (public
  member name/signature match — including a member inherited from a *base
  class* that never mentions the interface) or explicitly
  (`void IFoo.M()`, metadata name includes the interface, member has no
  simple-name lookup and is callable only through the interface). Derived
  classes can *re-implement* an interface, remapping members.
- **Demonstrated by:** A class implementing one interface implicitly and
  another explicitly; a base-class method satisfying a derived class's
  interface declaration; a derived class re-listing the interface to remap a
  member; two interfaces with the same member signature implemented once
  (shared) and separately (two explicit implementations).
- **Variants & failure cases:** Explicit implementations cannot be virtual or
  abstract? (they cannot be declared virtual; sealed by nature, but can be
  arranged with protected virtual helpers); casting is required to invoke;
  interface mapping recomputed per class declaration.
- **Constraints:** None.
- **Defined by:** Language. **Confidence:** high. **Source:** ECMA-334
  interface mapping rules.

#### INH-04 — Default interface members

- **Behavior:** Interfaces may provide method bodies, static members
  (including fields), and accessibility modifiers (C# 8). Implementing
  classes that omit the member inherit the default via *interface* dispatch
  only (not on the class's own surface — a cast to the interface is needed).
  "Most specific override" rules resolve diamonds; a truly ambiguous diamond
  is a compile error at the use/declaration site. Interfaces can explicitly
  implement members of their base interfaces, including re-abstraction.
- **Demonstrated by:** An interface with a default method one class inherits
  and another overrides; a diamond (`IA` → `IB`,`IC` → class) resolved by a
  most-specific override; a re-abstracted member forcing implementers.
- **Variants & failure cases:** Requires .NET Core 3.0+ runtime — the same
  code fails to compile for netstandard2.0/.NET Framework targets; private
  and static interface members; DIMs never appear on the implementing class's
  member list.
- **Constraints:** C# 8 + runtime support.
- **Defined by:** Language + runtime. **Confidence:** high.
  **Source:** csharplang default-interface-methods spec.

#### INH-05 — Covariant return types

- **Behavior:** An override may narrow the return type to a derived class
  (C# 9), including on properties' getters. The runtime enforces this via
  `PreserveBaseOverridesAttribute`; the override participates in the same
  slot yet has a different signature than its base.
- **Demonstrated by:** `abstract Animal Create();` overridden as
  `override Dog Create();` and both invoked through base and derived
  receivers.
- **Variants & failure cases:** Not available for interface implementations
  (interface mapping still needs exact/variant match); unavailable on
  .NET Framework targets.
- **Constraints:** C# 9 + .NET 5+ runtime.
- **Defined by:** Language + runtime. **Confidence:** high.
  **Source:** csharplang covariant-returns spec.

#### INH-06 — Constructor chaining and initialization order

- **Behavior:** `this(...)` and `base(...)` initializers create explicit
  constructor-to-constructor call edges; instance field initializers run
  before the base constructor call in derived-to-base order; static
  constructors and static field initializers run per type with `beforefieldinit`
  relaxations. Virtual calls from a base constructor dispatch to derived
  overrides before the derived constructor body runs.
- **Demonstrated by:** A hierarchy with field initializers, chained
  constructors, and a (deliberately dubious) virtual call from a base
  constructor observed to hit the derived override.
- **Variants & failure cases:** Implicit default `base()` call; struct
  constructors and `this = default` assignment; static constructor presence
  toggling `beforefieldinit`.
- **Constraints:** None.
- **Defined by:** Language (timing partly implementation-relaxed).
  **Confidence:** high. **Source:** ECMA-334; ECMA-335 (beforefieldinit).

### 2.7 Callables and Invocation Semantics (FN)

#### FN-01 — Overloading and overload resolution

- **Behavior:** Candidate gathering plus betterness rules decide targets
  among overloads differing by parameter types, arity, `params`,
  optional parameters, and named arguments (which permit reordering and
  change evaluation order). C# 13 adds `params` collections
  (`params ReadOnlySpan<T>`, `params IEnumerable<T>`, builder types) with
  new betterness preferring span forms, and
  `[OverloadResolutionPriority]` to force tie-breaks. Default parameter
  values are baked at call sites (like consts).
- **Demonstrated by:** An overload set exercising: exact vs convertible
  match, `params` expanded vs normal form, named-argument reordering,
  optional parameter omission, a C# 13 `params ReadOnlySpan<T>` overload
  winning over `params T[]`, and a priority-attributed overload.
- **Variants & failure cases:** Ambiguity CS0121; adding an overload as a
  silent behavior change for existing call sites; tuple/nullable conversions
  in betterness; interpolated string handler overloads beating `string`.
- **Constraints:** Params collections + priority attribute require C# 13.
- **Defined by:** Language. **Confidence:** high. **Source:** ECMA-334
  overload resolution; csharplang C# 13 specs.

#### FN-02 — Classic extension methods

- **Behavior:** Static methods with a `this` first parameter invoked as
  instance members; binding depends on using scope (RES-04); receiver may
  bind by conversion (extension on `IEnumerable<T>` applying to arrays);
  `ref this`/`in this` extensions mutate value-type receivers; extensions are
  also plain static methods callable by full name and convertible to
  delegates.
- **Demonstrated by:** An extension used in instance form and static form and
  as a method-group delegate; a generic extension with inference from the
  receiver; a `ref this` extension on a struct.
- **Variants & failure cases:** Not applicable when an instance member
  matches; nullable-receiver extensions run on null receivers (no NRE at
  dispatch); LINQ as the canonical consumer.
- **Constraints:** C# 3+.
- **Defined by:** Language. **Confidence:** high. **Source:** ECMA-334.

#### FN-03 — Extension members (C# 14 extension blocks)

- **Behavior:** `extension(Receiver r) { ... }` blocks inside static classes
  declare extension *properties*, methods, and static extension members (and
  extension operators) for a receiver type. Lookup makes them appear as
  members of the receiver; metadata encoding uses generated skeleton
  types/marker methods so downlevel compilers see plain statics. A new
  member-kind and lookup path distinct from classic extensions.
- **Demonstrated by:** An extension property and a static extension member
  consumed as if declared on the receiver; a classic extension method
  coexisting in the same static class.
- **Variants & failure cases:** Disambiguation syntax when conflicting;
  compatibility of classic-vs-new forms with identical shapes; consumption
  from older LangVersion.
- **Constraints:** C# 14 / .NET 10 SDK.
- **Defined by:** Language. **Confidence:** medium (metadata encoding
  details). **Source:** csharplang extensions feature spec.

#### FN-04 — Lambdas, anonymous methods, local functions, closures

- **Behavior:** Lambdas/anonymous methods capture variables (synthesized
  display classes; `static` lambdas forbid capture); C# 10 gives lambdas
  natural delegate types, attributes, and explicit return types; C# 12 adds
  default parameter values and `params` in lambdas; C# 14 allows modifiers
  (`ref`/`out`) on implicitly typed lambda parameters. Local functions
  (optionally `static`, generic, iterator, async) are named symbols nested in
  methods with mangled metadata names; unlike lambdas they support
  `yield`/definite-assignment-friendly usage before declaration.
- **Demonstrated by:** A capturing lambda, a `static` lambda, a lambda with
  attribute + explicit return type, a generic local function called before
  its declaration, an async and an iterator local function, a lambda stored
  with `var` (natural type `Func<...>`).
- **Variants & failure cases:** Capture of `this`, loop-variable capture
  semantics (foreach per-iteration variable since C# 5); conversion of the
  same lambda to `Expression<T>` vs delegate; recursion requiring named local
  functions.
- **Constraints:** Version gates as noted.
- **Defined by:** Language. **Confidence:** high. **Source:** ECMA-334;
  csharplang lambda-improvements spec.

#### FN-05 — Iterators (yield)

- **Behavior:** Methods/accessors/local functions with `yield return`/
  `yield break` compile into state-machine classes implementing
  `IEnumerable<T>`/`IEnumerator<T>`; the body does not execute at call time
  (deferred execution); `IteratorStateMachineAttribute` links method to
  generated type.
- **Demonstrated by:** An iterator method consumed lazily; an iterator local
  function; `yield` inside `try/finally`; C# 13 allowing `ref` locals and
  `unsafe` blocks inside iterators (no `yield` in unsafe regions).
- **Variants & failure cases:** Iterators cannot have `ref`/`out` parameters;
  return type must be one of the four enumerable/enumerator shapes;
  side-effect timing surprises (exceptions thrown at first MoveNext).
- **Constraints:** C# 2+; C# 13 relaxations.
- **Defined by:** Language. **Confidence:** high. **Source:** ECMA-334.

#### FN-06 — async/await and the awaitable pattern

- **Behavior:** `async` methods compile to state machines
  (`AsyncStateMachineAttribute` link); `await` binds pattern members —
  `GetAwaiter()` (instance or *extension*), `IsCompleted`,
  `GetResult`, `INotifyCompletion` — so any type can be made awaitable.
  Return types `Task`, `Task<T>`, `ValueTask(<T>)`, `void`, and custom
  task-like types via `[AsyncMethodBuilder]` (overridable per-method in
  C# 10). `await` in `catch`/`finally`; C# 13 permits `ref` locals/`unsafe`
  contexts inside async methods under restrictions.
- **Demonstrated by:** Async methods with each mainstream return type; a
  custom awaitable via extension `GetAwaiter`; a task-like type with a
  custom builder; `await` inside catch; async lambda; async entry point.
- **Variants & failure cases:** `async void` (event-handler convention,
  distinct error propagation); missing awaiter members produce binding
  errors; overload sets differing by sync/async shape.
- **Constraints:** C# 5 baseline; builder override C# 10.
- **Defined by:** Language. **Confidence:** high. **Source:** ECMA-334
  (async), csharplang task-types spec.

#### FN-07 — Async streams and async disposal

- **Behavior:** `await foreach` binds `IAsyncEnumerable<T>` or a pattern
  (`GetAsyncEnumerator`/`MoveNextAsync`/`Current`); async iterator methods
  (`async` + `yield`) produce combined state machines;
  `[EnumeratorCancellation]` wires `WithCancellation` tokens to iterator
  parameters. `await using` binds `IAsyncDisposable.DisposeAsync` (or
  pattern).
- **Demonstrated by:** An async iterator consumed with `await foreach` with
  cancellation; a type implementing both `IDisposable` and `IAsyncDisposable`
  used in both statement forms.
- **Variants & failure cases:** `ConfigureAwait(false)` wrappers changing the
  bound enumerator type; netstandard2.0 needs Microsoft.Bcl.AsyncInterfaces
  package.
- **Constraints:** C# 8; interfaces in-box .NET Core 3.0+.
- **Defined by:** Language. **Confidence:** high. **Source:** csharplang
  async-streams spec.

#### FN-08 — Byref parameters, returns, and locals

- **Behavior:** `ref`, `out`, `in`, and C# 12 `ref readonly` parameters
  change signatures (overloads may differ only by ref-kind vs by-value —
  restricted), call sites (explicit `ref`/`out`; `in` optional at call site),
  and aliasing semantics. `out var` declares variables at call sites;
  discards `_` bind nothing. `ref` returns/locals alias storage; ref
  conditional (`ref (c ? ref a : ref b)`) and ref reassignment; `scoped`
  locals/parameters constrain escape analysis.
- **Demonstrated by:** Methods with all four parameter ref-kinds and calls to
  each; a `ref` return from an array/span consumed via `ref` local and
  mutated; `out var` in a condition; a `scoped` parameter.
- **Variants & failure cases:** Overloading by-value vs `in` is allowed with
  betterness rules; `ref readonly` argument warnings for rvalues; defensive
  copies through `in`.
- **Constraints:** `in` C# 7.2; `ref readonly` parameters C# 12.
- **Defined by:** Language. **Confidence:** high. **Source:** ECMA-334;
  csharplang specs.

#### FN-09 — dynamic binding

- **Behavior:** Expressions of type `dynamic` defer member lookup, overload
  resolution, and conversions to runtime (DLR call sites); compile-time
  member references are absent by design. `dynamic` in signatures erases to
  `object` + `DynamicAttribute` in metadata; `dynamic` and `object` are an
  identity conversion apart. Static receivers with dynamic *arguments* also
  become dynamically bound (unless the candidate set is statically unique in
  newer compilers).
- **Demonstrated by:** A dynamic receiver calling a method that exists on the
  runtime type only; a statically-typed call made dynamic by a dynamic
  argument; `dynamic` in a public API signature consumed from another
  project.
- **Variants & failure cases:** Extension methods and dynamic dispatch don't
  mix (runtime binder failure); runtime binder exceptions where compile-time
  would have errored; requires Microsoft.CSharp assembly on some targets.
- **Constraints:** C# 4+.
- **Defined by:** Language + runtime binder. **Confidence:** high.
  **Source:** ECMA-334 dynamic binding chapter.

#### FN-10 — Expression trees

- **Behavior:** A lambda converted to `Expression<TDelegate>` compiles to
  factory-method calls (`Expression.Lambda`, `Expression.Call` with
  `MethodInfo`s) instead of executable IL — member references become
  *reflection object* references; many newer language constructs are illegal
  inside expression lambdas (e.g., `?.`, async, pattern matching,
  string interpolation handlers, tuples in some positions).
- **Demonstrated by:** The same lambda assigned to both `Func<T,bool>` and
  `Expression<Func<T,bool>>`; a LINQ query against `IQueryable<T>` (binds
  `Queryable` extensions taking expression trees, not `Enumerable`).
- **Variants & failure cases:** CS0834-family errors for unsupported
  constructs; provider libraries (EF Core) interpreting trees — semantic
  meaning delegated to ecosystem.
- **Constraints:** C# 3+.
- **Defined by:** Language. **Confidence:** high. **Source:** ECMA-334;
  Microsoft Learn expression trees documentation.

#### FN-11 — Caller-information and argument-expression injection

- **Behavior:** `[CallerMemberName]`, `[CallerFilePath]`, `[CallerLineNumber]`
  fill optional parameters with call-site facts; C# 10
  `[CallerArgumentExpression("param")]` injects the *source text* of another
  argument. Call sites thus pass values that appear nowhere in their source.
- **Demonstrated by:** A logging/guard helper using all four, called from
  several places; `ArgumentNullException.ThrowIfNull` style usage.
- **Variants & failure cases:** Explicitly supplied arguments override
  injection; interaction with `nameof`; `#line` directives change injected
  file/line values.
- **Constraints:** C# 5 / C# 10.
- **Defined by:** Language. **Confidence:** high. **Source:** csharplang
  caller-argument-expression spec.

#### FN-12 — Interceptors

- **Behavior:** A generator-emitted method with `[InterceptsLocation(...)]`
  *replaces the target of a specific call site* (identified by an opaque
  location token) at compile time — the syntactic callee and the semantic
  callee differ. Used in production by ASP.NET Core's request-delegate
  generator and the configuration-binder generator, and by Native AOT
  scenarios.
- **Demonstrated by:** A project enabling an interceptor-based generator
  (e.g., configuration binder) where a call to a normal API is intercepted;
  with `EmitCompilerGeneratedFiles` the interceptor source is observable.
- **Variants & failure cases:** Only specific call-site locations are
  intercepted (other calls to the same method are untouched); feature gated
  by `<InterceptorsNamespaces>`/preview settings; still evolving —
  experimental status.
- **Constraints:** C# 12+ Roslyn experimental feature; .NET 8+ SDK usage.
- **Defined by:** Compiler (Roslyn experimental). **Confidence:** medium.
  **Source:** Roslyn interceptors design documentation.

### 2.8 Pattern-Based (Duck-Typed) Language Protocols (PAT)

#### PAT-01 — foreach and its lookup pattern

- **Behavior:** `foreach` binds a `GetEnumerator()` method (interface not
  required), then `MoveNext()`/`Current`; C# 9 allows *extension*
  `GetEnumerator`. Ref-struct enumerators (e.g., `Span<T>`) and
  `ref`/`ref readonly` iteration variables; the loop variable is implicitly
  converted (potential hidden cast per iteration with explicit typing).
- **Demonstrated by:** foreach over: an array (special-cased), a `List<T>`
  (struct enumerator), a custom pattern-only enumerable, a type made
  enumerable purely via an extension `GetEnumerator`, and a `Span<T>`.
- **Variants & failure cases:** Dispose called when enumerator is
  IDisposable or a disposable ref struct; missing pattern members produce
  binding errors; `dynamic` collection defers everything.
- **Constraints:** Extension GetEnumerator requires C# 9.
- **Defined by:** Language. **Confidence:** high. **Source:** ECMA-334
  foreach statement rules.

#### PAT-02 — using and pattern-based disposal

- **Behavior:** `using` statements and C# 8 using *declarations* (scope to
  end of block) bind `IDisposable.Dispose`; ref structs qualify via a
  pattern `Dispose()` with no interface. `await using` binds
  `IAsyncDisposable`/pattern. Resource cleanup edges differ per form.
- **Demonstrated by:** Classic using statement, a using declaration, a ref
  struct with pattern Dispose used in `using`, and `await using`.
- **Variants & failure cases:** Multiple declarations in one using; using on
  a null value; declaration ordering determines reverse-disposal order.
- **Constraints:** Using declarations + pattern disposal C# 8.
- **Defined by:** Language. **Confidence:** high. **Source:** ECMA-334;
  csharplang using-declarations spec.

#### PAT-03 — Object and collection initializers

- **Behavior:** Object initializers bind property/field setters (including
  `init`) and nested initializers; collection initializers bind an applicable
  `Add` method (instance or *extension*) per element with the target
  implementing `IEnumerable`; dictionary/index initializers bind indexer
  setters (`[k] = v`); C# 13 permits `^`/implicit-index access inside object
  initializers.
- **Demonstrated by:** A nested object initializer; a custom collection
  initialized via an extension `Add`; an index initializer on a dictionary;
  combined object+collection initialization.
- **Variants & failure cases:** `Add` overload resolution per element (mixed
  element shapes hitting different overloads); required members enforced
  here; initializer runs after constructor.
- **Constraints:** Extension Add C# 6; index initializers C# 6.
- **Defined by:** Language. **Confidence:** high. **Source:** ECMA-334.

#### PAT-04 — Collection expressions and CollectionBuilder

- **Behavior:** `[a, b, .. rest]` (C# 12) is target-typed: it constructs
  arrays, spans, `List<T>`, immutable collections, and any type carrying
  `[CollectionBuilder(typeof(Builder), "Create")]` — binding a builder method
  the source never names. Spread elements enumerate their operands. No
  natural type: the same expression means different construction per target.
- **Demonstrated by:** One collection expression assigned to `int[]`,
  `Span<int>`, `List<int>`, `ImmutableArray<int>`, and a custom
  CollectionBuilder type; a spread of another collection.
- **Variants & failure cases:** Empty `[]` needing a target; interface
  targets (`IEnumerable<int>`) choose compiler-selected concrete types;
  C# 13/14 refinements to conversions and `params` synergy.
- **Constraints:** C# 12; builder attribute in .NET 8+ (polyfillable).
- **Defined by:** Language. **Confidence:** high. **Source:** csharplang
  collection-expressions spec.

#### PAT-05 — Deconstruction

- **Behavior:** `var (a, b) = expr;` binds an instance or *extension*
  `Deconstruct(out ...)` (tuples deconstruct natively); positional patterns
  (`expr is (1, _)`) bind the same methods. Overloaded Deconstructs by arity;
  deconstruction into existing variables and mixed declaration forms
  (C# 10).
- **Demonstrated by:** A type with two Deconstruct arities, an extension
  Deconstruct for a foreign type, deconstructing assignment plus a positional
  pattern hitting the same method.
- **Variants & failure cases:** Ambiguous same-arity Deconstructs error;
  nested deconstruction; discards.
- **Constraints:** C# 7+.
- **Defined by:** Language. **Confidence:** high. **Source:** ECMA-334
  deconstruction; csharplang specs.

#### PAT-06 — LINQ query syntax translation

- **Behavior:** Query expressions translate to chained invocations of
  `Select`/`Where`/`SelectMany`/`Join`/`GroupBy`/`OrderBy`/`Cast` — bound as
  *ordinary methods* on the source (instance or extension), so any type
  supplying the query pattern participates. `let` and multiple `from`
  clauses introduce transparent identifiers (compiler-invented anonymous
  types); an explicitly typed range variable inserts a `Cast<T>` call.
- **Demonstrated by:** A query with `let`, `join ... into`, `group ... by
  ... into`, and `orderby` over `IEnumerable<T>`; the same query over
  `IQueryable<T>` (binding `Queryable`, expression trees); a custom type
  with its own instance `Select`/`Where` consumed by query syntax without
  LINQ usings.
- **Variants & failure cases:** Degenerate `select x` elision; missing
  `using System.Linq` making queries unbindable; query over `dynamic`
  unsupported.
- **Constraints:** C# 3+.
- **Defined by:** Language (translation is specified, binding is ordinary).
  **Confidence:** high. **Source:** ECMA-334 query expression translation.

#### PAT-07 — Index and Range implicit support

- **Behavior:** `x[^1]` and `x[1..3]` bind either real
  `Index`/`Range` indexers or the *implicit pattern*: a `Length`/`Count`
  property plus an int indexer (for `^`) and a `Slice(int,int)` method (for
  ranges). Arrays get compiler-generated helpers. Members the source never
  names (`Length`, `Slice`) enter the call graph.
- **Demonstrated by:** `^`/ranges over an array, a string (`Substring`
  special-casing), a `Span<T>`, and a custom type with only
  `Count` + indexer + `Slice`.
- **Variants & failure cases:** C# 13 implicit index access in object
  initializers; missing pattern members error.
- **Constraints:** C# 8; Index/Range types in-box .NET Core 3.0+.
- **Defined by:** Language. **Confidence:** high. **Source:** csharplang
  ranges spec.

#### PAT-08 — Interpolated string handlers and string interpolation targets

- **Behavior:** `$"..."` may compile to: `string.Format`-style calls, a
  `DefaultInterpolatedStringHandler` append sequence, a constant string
  (C# 10, when all holes are constant), `FormattableString`/`IFormattable`
  (target-typed), or a *custom handler* struct chosen by parameter attributes
  (`[InterpolatedStringHandlerArgument]`), where appends can be conditionally
  skipped based on a bool out-parameter from the handler constructor —
  argument evaluation itself becomes conditional.
- **Demonstrated by:** The same interpolation passed to `string`, to
  `FormattableString`, and to a logging-style API with a custom handler that
  short-circuits; a const interpolated string.
- **Variants & failure cases:** Overloads differing string vs handler;
  culture behavior differences (`FormattableString.Invariant`).
- **Constraints:** Handlers C# 10.
- **Defined by:** Language. **Confidence:** high. **Source:** csharplang
  improved-interpolated-strings spec.

### 2.9 Expressions, Flow Analysis, and Type-State (EXP)

#### EXP-01 — Pattern matching suite

- **Behavior:** `is`/`switch` patterns: constant (incl. matching a
  `Span<char>`/`ReadOnlySpan<char>` against string constants, C# 11), type,
  declaration, `var`, discard, property (incl. extended dotted forms
  `{ A.B: 1 }`, C# 10), positional (Deconstruct binding), relational,
  logical (`and`/`or`/`not`), parenthesized, list patterns `[1, _, ..var
  rest]` (C# 11) binding Length/indexer/slice pattern members. Switch
  *expressions* are exhaustiveness-checked (CS8509 warnings with example
  counterexamples); pattern variables have flow-scoped definite assignment.
- **Demonstrated by:** A switch expression over a record hierarchy combining
  most pattern kinds; a list pattern with slice capture; an `is` pattern in a
  condition with the variable used after; a non-exhaustive switch expression
  producing the warning.
- **Variants & failure cases:** Subsumption diagnostics (unreachable arm);
  patterns binding user Deconstruct and Length/indexer members; `not null`
  idiom interacting with nullability flow.
- **Constraints:** Versions as noted (C# 7–11 accretion).
- **Defined by:** Language. **Confidence:** high. **Source:** csharplang
  patterns specs; ECMA-334.

#### EXP-02 — Nullable reference types

- **Behavior:** With nullable context enabled, `string` vs `string?` are
  distinct declared types driving flow analysis (warnings, not errors, by
  default); metadata encodes annotations via `NullableAttribute`/
  `NullableContextAttribute` so warnings flow across assemblies; the
  null-forgiving `!` operator, nullable flow attributes
  (`[NotNullWhen]`, `[MaybeNullWhen]`, `[MemberNotNull]`, `[DoesNotReturn]`,
  `[AllowNull]`, `[DisallowNull]`, `[NotNullIfNotNull]`) alter analysis at
  call sites; unconstrained generics use `T?` with `default` semantics and
  the `notnull`/`class?` constraints interact.
- **Demonstrated by:** A nullable-enabled project consuming both an annotated
  and an *oblivious* (nullable-disabled) library; APIs using the flow
  attributes exercised so warnings appear/disappear; `#nullable` region
  overrides; a `T?` on an unconstrained generic.
- **Variants & failure cases:** Same code warns or not depending on
  `<Nullable>` setting (configuration-dependent diagnostics); annotations are
  erased at runtime (no cast semantics); `!` suppression sites.
- **Constraints:** C# 8+; project or directive opt-in.
- **Defined by:** Language. **Confidence:** high. **Source:** csharplang
  nullable-reference-types specs.

#### EXP-03 — Conversion taxonomy

- **Behavior:** Identity, implicit/explicit numeric, enum, nullable lifting,
  reference conversions (up/down-cast, variance), boxing/unboxing,
  user-defined implicit/explicit operators (with lifted forms), tuple
  conversions, method group and lambda conversions, `dynamic` conversions,
  and C# 14 first-class span conversions (`T[]` → `Span<T>`/
  `ReadOnlySpan<T>`, `Span<T>` → `ReadOnlySpan<T>` as language-level
  implicit conversions rather than operator methods). `is`/`as`/casts differ
  in which conversions they consider (as/is: reference-ish only, no
  user-defined).
- **Demonstrated by:** Sites exercising each family, especially a
  user-defined implicit conversion silently applied at an argument, a lifted
  operator on `int?`, a variance conversion, and (C# 14) span conversions
  changing overload choice.
- **Variants & failure cases:** `as` with user-defined conversion is a
  compile error; boxing of struct through interface; conversion-driven
  overload ambiguity.
- **Constraints:** Span conversions C# 14.
- **Defined by:** Language. **Confidence:** high (medium for C# 14 span
  rules). **Source:** ECMA-334 conversions chapter.

#### EXP-04 — checked/unchecked contexts

- **Behavior:** Overflow behavior of integral arithmetic and conversions
  depends on context: `checked`/`unchecked` blocks and expressions, the
  project-wide `<CheckForOverflowUnderflow>` switch, and constant expressions
  (always checked at compile time unless explicitly unchecked). C# 11
  checked operator variants get *selected* by context — same `+` token,
  different `op_` method.
- **Demonstrated by:** The same arithmetic in checked and unchecked blocks; a
  project with overflow checking globally enabled; a user type with paired
  checked/unchecked operators observed to bind differently.
- **Variants & failure cases:** Constant overflow as compile error;
  `unchecked((int)0xFFFFFFFF)` idiom.
- **Constraints:** Checked operators C# 11.
- **Defined by:** Language + build switch. **Confidence:** high.
  **Source:** ECMA-334.

#### EXP-05 — Definite assignment and declaration scoping in expressions

- **Behavior:** `out var` declarations, pattern variables, and deconstruction
  declarations introduce variables whose scope extends beyond their textual
  expression (to the enclosing statement's block, with condition-dependent
  definite assignment); using-declaration scope runs to block end; `for`/
  `foreach` variable scoping; struct auto-default (C# 11) changed older
  definite-assignment errors into implicit zeroing.
- **Demonstrated by:** `if (int.TryParse(s, out var n)) use(n); else use(n)`
  (error in else branch); pattern variable used after the `if`; a
  variable-shadowing rejection (no shadowing of locals in nested scopes).
- **Variants & failure cases:** Variables from `is` patterns in boolean
  expressions with `!`/`||`; `goto` interacting with assignment analysis.
- **Constraints:** C# 7+ features.
- **Defined by:** Language. **Confidence:** high. **Source:** ECMA-334
  definite assignment.

#### EXP-06 — lock statement forms

- **Behavior:** `lock (obj)` lowers to `Monitor.Enter/Exit` with try/finally
  — an invisible BCL dependency; C# 13: when the operand's static type is
  `System.Threading.Lock`, the statement instead binds `Lock.EnterScope()`
  and a ref-struct scope disposal — the same syntax produces a different
  call graph based on operand type.
- **Demonstrated by:** One lock over an `object` field and one over a
  `System.Threading.Lock` field; a warning case where a `Lock` is converted
  to `object` and locked (defeats the fast path, compiler warns).
- **Variants & failure cases:** Locking on `this`/strings (analyzer
  territory); `await` prohibited inside lock (until the C# 14-era relaxation
  discussions — as of baseline, prohibited).
- **Constraints:** `Lock` requires .NET 9 + C# 13.
- **Defined by:** Language. **Confidence:** high. **Source:** csharplang
  lock-object spec.

#### EXP-07 — String literal forms with semantic impact

- **Behavior:** Regular, verbatim `@""`, raw `"""..."""` (C# 11, lexical
  only), interpolated combinations, and UTF-8 literals `"..."u8` whose *type*
  is `ReadOnlySpan<byte>` — a different type and overload surface, not just
  different escaping. Constant string concatenation and `const` interpolated
  strings fold at compile time (cross-assembly baking per TY-12).
- **Demonstrated by:** A `u8` literal flowing into a span-based API; raw
  strings containing quotes/braces; a const built from interpolation of
  consts.
- **Variants & failure cases:** `u8` not a `string` (no constant folding into
  string APIs); raw interpolated strings with multiple `$` for brace
  escaping.
- **Constraints:** Raw and u8 literals C# 11.
- **Defined by:** Language. **Confidence:** high. **Source:** csharplang
  raw-string/u8 specs.

#### EXP-08 — Unsafe code

- **Behavior:** `unsafe` contexts enable pointer types (`T*`, `void*`),
  address-of, dereference, pointer arithmetic, `fixed` statements (pinning
  via array/string/`GetPinnableReference` pattern), `stackalloc` (also usable
  safely as `Span<T>` without `unsafe`), `sizeof` on user types, and
  fixed-size buffers in structs. Gated by `<AllowUnsafeBlocks>` — a build
  property that makes source legality configuration-dependent.
- **Demonstrated by:** An unsafe block with pointer arithmetic over a fixed
  array; a `stackalloc` into `Span<int>` in safe code; a custom
  `GetPinnableReference` used by `fixed`; the project failing to compile with
  the flag off.
- **Variants & failure cases:** Unsafe contexts on types vs methods vs
  blocks; function pointers (TY-10); C# 13 unsafe in iterators/async.
- **Constraints:** Compiler flag; C# 7.3 pattern-based fixed.
- **Defined by:** Language + build switch. **Confidence:** high.
  **Source:** ECMA-334 unsafe code annex.

### 2.10 Attributes, Metadata, and Compile-Time Codegen (MET)

#### MET-01 — Attribute application breadth

- **Behavior:** Attributes attach to assemblies, modules, types, members,
  parameters, return values (`[return: ...]`), generic type parameters,
  fields-behind-properties (`[field: ...]`), and events' accessors; targets
  disambiguate (`[method:]`, `[property:]`, `[param:]` on positional record
  parameters). Generic attributes (C# 11) instantiate attribute classes with
  type arguments. Arguments are constants, `typeof`, arrays, and named
  properties; `AttributeUsage` controls multiplicity/inheritance.
- **Demonstrated by:** Each target position exercised once, including
  assembly-level in a dedicated file, `[field:]` on an auto-property, a
  generic attribute, and an attribute using `typeof(Open<>)`.
- **Variants & failure cases:** Attributes referencing types create real
  edges; string-typed type references (`[TypeConverter("Ns.T, Asm")]`)
  create *unresolvable-by-compiler* references (ecosystem-resolved);
  conditional attribute classes elided per symbols.
- **Constraints:** Generic attributes C# 11.
- **Defined by:** Language. **Confidence:** high. **Source:** ECMA-334
  attributes chapter.

#### MET-02 — [Conditional] call elision

- **Behavior:** Calls to `[Conditional("SYM")]` methods (and attribute
  applications of conditional attribute classes) are *removed at the call
  site* when the symbol is undefined in the calling file's context — the call
  edge exists in one configuration and not another; arguments are not
  evaluated when elided.
- **Demonstrated by:** `Debug.Assert` calls present under Debug and absent in
  Release; a user conditional method invoked from a file that defines the
  symbol and one that doesn't.
- **Variants & failure cases:** Conditional methods must return void and not
  be used via delegates; elision is per-call-site compilation context.
- **Constraints:** None.
- **Defined by:** Language. **Confidence:** high. **Source:** ECMA-334.

#### MET-03 — Obsolete, Experimental, and API lifecycle diagnostics

- **Behavior:** `[Obsolete]` (message, `error: true`, `DiagnosticId`,
  `UrlFormat`) turns member *references* into warnings or errors —
  reference-sensitive diagnostics; `[Experimental("DIAG")]` (C# 12/.NET 8)
  produces errors unless suppressed per diagnostic ID;
  `[EditorBrowsable]` hides members from tooling without semantic effect
  (a useful negative case).
- **Demonstrated by:** Calls to obsolete-warning, obsolete-error, and
  experimental members, with one suppression via `#pragma` and one via
  `NoWarn`.
- **Variants & failure cases:** Obsolete on types vs members vs accessors;
  obsoleting interface implementations; `ObsoleteAttribute` used internally
  by compiler for ref-struct downlevel protection.
- **Constraints:** Experimental requires .NET 8+ attribute.
- **Defined by:** Language/compiler. **Confidence:** high.
  **Source:** Microsoft Learn API docs; csharplang experimental spec.

#### MET-04 — Source generators

- **Behavior:** Roslyn (incremental) source generators add compilation units
  during build — typically completing user-declared `partial` types/members
  keyed off marker attributes; they can read `AdditionalFiles` (non-C#
  inputs like JSON/txt) so non-source files determine generated symbols.
  Generated code is part of semantics but absent from the source tree unless
  `EmitCompilerGeneratedFiles` is set. Generators cannot modify existing
  code (interceptors, FN-12, are the carve-out).
- **Demonstrated by:** A project consuming at least one real generator
  (e.g., `System.Text.Json` context, `GeneratedRegex`, or `LibraryImport`)
  where a user partial type gains generated members; an `AdditionalFiles`
  item feeding a generator; emitted-files inspection enabled.
- **Variants & failure cases:** Generator diagnostics; ordering/isolation
  (generators can't see each other's output); IDE vs build discrepancies;
  generator delivered via NuGet `analyzers/dotnet/cs` asset.
- **Constraints:** Roslyn 4.x for incremental generators; SDK builds.
- **Defined by:** Compiler (Roslyn API) + ecosystem.
  **Confidence:** high. **Source:** Roslyn source-generators documentation.

#### MET-05 — Diagnostic configuration and suppression layers

- **Behavior:** Effective diagnostics are computed from: compiler defaults,
  warning waves (`AnalysisLevel`/`WarningLevel`), `.editorconfig` and
  `.globalconfig` severity mappings (per-diagnostic, per-path),
  `<NoWarn>`/`<WarningsAsErrors>`/`<TreatWarningsAsErrors>`,
  `#pragma warning`, `[SuppressMessage]`, and analyzer *suppressors*.
  Identical source yields different diagnostic sets — and pass/fail builds —
  per configuration.
- **Demonstrated by:** A repo with `.editorconfig` escalating one rule to
  error and silencing another; a project-level NoWarn; a source-level pragma
  and `[SuppressMessage]`; `TreatWarningsAsErrors` with a targeted
  `WarningsNotAsErrors`.
- **Variants & failure cases:** Nullable warnings promoted via
  `<WarningsAsErrors>nullable</WarningsAsErrors>`; analyzers shipped in the
  SDK (NETAnalyzers) vs packages; `AnalysisMode` presets.
- **Constraints:** SDK 5+ for NETAnalyzers/waves.
- **Defined by:** Compiler + build system. **Confidence:** high.
  **Source:** Microsoft Learn code-analysis configuration docs.

#### MET-06 — Semantics-bearing well-known attributes

- **Behavior:** Attributes the compiler/runtime treat specially:
  `[ModuleInitializer]` (method invoked at module load — an invisible call
  edge from nothing), `[SkipLocalsInit]`, `[MethodImpl(AggressiveInlining |
  NoInlining | Synchronized)]`, `[StructLayout]`/`[FieldOffset]` (unions),
  `[UnscopedRef]`, `[DoesNotReturn]`/`[MemberNotNull]` (flow),
  `[DynamicallyAccessedMembers]`/`[RequiresUnreferencedCode]`/
  `[RequiresDynamicCode]` (trimming/AOT diagnostics that make *reflection
  reachability* explicit), `[DebuggerDisplay]` (none — negative case).
- **Demonstrated by:** A module initializer; an explicit-layout union struct;
  a trimming-annotated API consumed with and without warnings under
  `IsTrimmable`/`PublishTrimmed` analysis.
- **Variants & failure cases:** Module initializer ordering across multiple
  initializers unspecified; trimming attributes only diagnose under enabled
  analyzers.
- **Constraints:** ModuleInitializer C# 9; trimming analyzers .NET 6+.
- **Defined by:** Compiler + runtime + SDK analyzers. **Confidence:** high.
  **Source:** Microsoft Learn API documentation.

#### MET-07 — Metadata encodings of language-level type information

- **Behavior:** Several C# type distinctions exist only as attribute/modifier
  encodings on `object`/primitive metadata: `dynamic` (`DynamicAttribute`),
  tuple names (`TupleElementNamesAttribute`), nullable annotations
  (`NullableAttribute`, `NullableContextAttribute`), native ints pre-.NET 7
  (`NativeIntegerAttribute`), plus modreq/modopt markers: `IsExternalInit`
  (init accessors), `IsVolatile` (volatile fields), `InAttribute` +
  `IsReadOnlyAttribute` (in params, readonly returns/members),
  `RequiresLocationAttribute` (ref readonly params),
  `IsByRefLikeAttribute` plus guard `Obsolete` (ref structs), `ScopedRefAttribute`,
  `IsUnmanagedAttribute` (constraint), `ExtensionAttribute`. A graph reading
  metadata must decode these to recover source-level types; one reading
  source must know they round-trip.
- **Demonstrated by:** A public API using `dynamic`, named tuples, nullable
  annotations, `in`/`ref readonly` params, init setters, volatile fields, and
  a ref struct — consumed from a second project so the encodings are
  actually traversed.
- **Variants & failure cases:** Polyfilled attribute types embedded per
  assembly (CS0436 interplay); modreqs breaking downlevel consumers by
  design.
- **Constraints:** Encodings vary by compiler/TFM era.
- **Defined by:** Compiler conventions over CLI metadata.
  **Confidence:** high. **Source:** Roslyn documentation; ECMA-335.

#### MET-08 — Native and COM interop surfaces

- **Behavior:** `[DllImport]` declares extern methods bound at runtime to
  native exports (string-named edges leaving the managed graph);
  `[LibraryImport]` (source-generated P/Invoke, .NET 7+) replaces runtime
  marshalling with generated C# — a `partial` method completed by a
  generator. COM interop: `[ComImport]`/interop assemblies, and *embedded
  interop types* (`EmbedInteropTypes`, NoPIA) where interface types are
  copied into consuming assemblies and unified at runtime by
  `[TypeIdentifier]` equivalence — same "type" as multiple embedded copies.
- **Demonstrated by:** A DllImport and a LibraryImport of the same native
  function (with `MarshalAs` on one); (Windows-gated variant) a COM interface
  consumed with interop-type embedding.
- **Variants & failure cases:** LibraryImport requires `partial` +
  allow-unsafe in some marshalling shapes; charset/entry-point resolution
  variants; NoPIA type equivalence breaks nominal identity assumptions.
- **Constraints:** LibraryImport .NET 7+; COM/NoPIA effectively
  Windows-oriented.
- **Defined by:** Runtime + compiler + generator. **Confidence:** high
  (medium on NoPIA details). **Source:** Microsoft Learn interop
  documentation.

### 2.11 Build and Configuration (BLD)

#### BLD-01 — LangVersion gating

- **Behavior:** `<LangVersion>` (default derived from TFM: net8.0→C# 12,
  net9.0→C# 13, net10.0→C# 14; netstandard2.0/net4x→7.3) decides which
  syntax/semantics are legal. The same file compiles or errors
  (CS8370-family: "feature requires language version X") purely by project
  configuration; `preview` unlocks in-flight features and marks assemblies.
- **Demonstrated by:** A multi-targeted or two-project setup where a modern
  feature compiles under one LangVersion and errors under another; an
  explicit `<LangVersion>latest</LangVersion>`.
- **Variants & failure cases:** Language features needing runtime *and*
  LangVersion (DIMs, static abstracts) vs syntax-only features usable
  downlevel with polyfills (records on netstandard2.0 via IsExternalInit).
- **Constraints:** SDK defaults change per release.
- **Defined by:** Compiler + SDK defaults. **Confidence:** high.
  **Source:** Microsoft Learn C# language versioning documentation.

#### BLD-02 — Semantics-changing MSBuild properties

- **Behavior:** A small set of properties alter language semantics or the
  compiled surface, not just packaging: `DefineConstants`, `Nullable`,
  `ImplicitUsings`, `AllowUnsafeBlocks`, `CheckForOverflowUnderflow`,
  `LangVersion`, `TreatWarningsAsErrors`, `SignAssembly`,
  `GenerateAssemblyInfo`, `InterceptorsNamespaces`, `Deterministic`,
  `Features`. The project file is a semantic input on par with source.
- **Demonstrated by:** A csproj exercising several of these with
  source that depends on each (unsafe block, overflow behavior, nullable
  warnings, custom define).
- **Variants & failure cases:** Same source, different `Configuration` or
  property → different symbols/diagnostics; properties set outside the
  csproj (BLD-03) invisible to file-local inspection.
- **Constraints:** MSBuild.
- **Defined by:** Build system. **Confidence:** high. **Source:** Microsoft
  Learn compiler-options/MSBuild property reference.

#### BLD-03 — Directory.Build.props/targets and import hierarchy

- **Behavior:** MSBuild auto-imports the nearest `Directory.Build.props`
  (before the project) and `Directory.Build.targets` (after), walking up the
  directory tree; SDK props/targets and NuGet-delivered `build/` files also
  import. Effective properties/items for a project are assembled from files
  the project never references — repo-root files can flip `LangVersion`,
  `Nullable`, defines, or add source files for every project underneath.
- **Demonstrated by:** A repo-root Directory.Build.props setting
  `Nullable` and a custom constant consumed by `#if` in a leaf project;
  a nested override file demonstrating nearest-wins (with explicit chaining
  via `Import` of the parent).
- **Variants & failure cases:** Only the nearest file is auto-imported
  (chaining is manual); ordering between props/targets matters for
  property evaluation.
- **Constraints:** MSBuild 15+.
- **Defined by:** Build system. **Confidence:** high. **Source:** Microsoft
  Learn MSBuild customization documentation.

#### BLD-04 — global.json and SDK version selection

- **Behavior:** `global.json` pins the .NET SDK (version + roll-forward
  policy) for a directory subtree. The SDK version determines default
  LangVersion mapping, implicit usings content, bundled analyzers, and
  available TFMs — so two checkouts of identical sources can build with
  different language rules based on a JSON file and installed SDKs.
- **Demonstrated by:** A `global.json` at repo root with a rollForward
  policy; documentation of the SDK-dependent behavior the fixture relies on.
- **Variants & failure cases:** Missing pinned SDK → build failure;
  `rollForward: latestFeature` etc. altering resolution.
- **Constraints:** .NET SDK.
- **Defined by:** SDK. **Confidence:** high. **Source:** Microsoft Learn
  global.json documentation.

#### BLD-05 — SDK families and their compilation deltas

- **Behavior:** `Microsoft.NET.Sdk` vs `.Web` vs `.Razor` vs `.Worker` vs
  `.WindowsDesktop` change: implicit `using` sets, implicit
  `FrameworkReference`s (ASP.NET Core shared framework), included source
  generators (Razor), extra item types (`.razor`, `.cshtml`, `.xaml` with
  `UseWPF`/`UseWindowsForms`), and default content globs. Project semantics
  depend on the one-line `Sdk="..."` attribute.
- **Demonstrated by:** A web project using ASP.NET types with no
  PackageReference (shared framework) plus implicit web usings; a class
  library that fails to see those types.
- **Variants & failure cases:** `FrameworkReference` added manually to a
  plain library; WindowsDesktop TFM gating (`net9.0-windows`).
- **Constraints:** SDK availability; some families OS-restricted.
- **Defined by:** SDK. **Confidence:** high. **Source:** Microsoft Learn
  project SDK documentation.

#### BLD-06 — Resources, resx codegen, and satellite assemblies

- **Behavior:** `.resx` `EmbeddedResource` items generate strongly typed
  accessor classes (design-time generated `*.Designer.cs`, checked in, tied
  by `DependentUpon` metadata) and embed `.resources` blobs; per-culture
  `.resx` files build *satellite assemblies* (`<culture>/App.resources.dll`)
  — assemblies with no code. Resource lookup via `ResourceManager` is
  string-keyed (statically invisible), while the Designer class exposes
  compiled properties per resource.
- **Demonstrated by:** A default + one culture-specific resx, the checked-in
  Designer file, and code consuming a resource both via the typed class and a
  raw string key.
- **Variants & failure cases:** Manifest resource naming rules
  (RootNamespace + folder path); `GenerateResource` behavior; missing
  culture fallback chain at runtime.
- **Constraints:** MSBuild; single-file designer generation is an IDE/build
  convention.
- **Defined by:** Build system + ecosystem convention. **Confidence:** high.
  **Source:** Microsoft Learn resources documentation.

#### BLD-07 — Markup-generated partial classes (XAML, WinForms designer)

- **Behavior:** WPF/WinUI/MAUI XAML compiles `.xaml` into generated partial
  class halves (`*.g.cs` with `InitializeComponent`, field declarations for
  named elements) merged with user code-behind; WinForms uses checked-in
  `*.Designer.cs` partials. Cross-file member references flow between markup
  (`x:Name`, event wiring) and C#.
- **Demonstrated by:** (Windows-gated variant) a WPF window whose code-behind
  references an `x:Name`d element and handles an event declared in XAML; a
  WinForms form with its Designer partial.
- **Variants & failure cases:** Event handler references from markup are
  name-based; `x:Class` mismatch errors; MAUI equivalents are cross-platform.
- **Constraints:** `net*-windows` TFMs + `UseWPF`/`UseWindowsForms`, or MAUI
  workloads — platform-restricted; may be a documented exclusion for a
  self-contained cross-platform fixture.
- **Defined by:** Build system/toolchain. **Confidence:** medium-high.
  **Source:** Microsoft Learn WPF/WinForms documentation.

#### BLD-08 — Deterministic builds, PDBs, SourceLink, and embedded sources

- **Behavior:** `Deterministic` builds normalize paths (`PathMap`); portable
  PDBs map IL to source (sequence points honoring `#line`); SourceLink embeds
  repository URLs; `EmbedAllSources`/`EmbedUntrackedSources` put source text
  inside PDBs. These decide whether and how compiled artifacts can be traced
  back to source identities.
- **Demonstrated by:** A project producing portable PDBs with SourceLink
  configuration and one embedded (generated) source file.
- **Variants & failure cases:** `DebugType` embedded vs portable; PathMap
  making recorded paths differ from disk paths.
- **Constraints:** SDK/NuGet packages for SourceLink providers (in-box in
  recent SDKs).
- **Defined by:** Compiler + SDK. **Confidence:** high. **Source:** Roslyn
  deterministic-builds documentation; Microsoft Learn SourceLink docs.

#### BLD-09 — Output kinds, trimming/AOT, and publish-time semantics

- **Behavior:** `OutputType` (Exe/Library/WinExe) selects entry-point
  requirements; `PublishTrimmed`/`PublishAot` activate analyzers that make
  reflection-based reachability an *error surface*
  (`IL2026`/`IL3050`-family warnings on annotated APIs), changing which
  programs are "valid"; single-file publish alters resource/assembly loading
  assumptions.
- **Demonstrated by:** A console app with AOT-incompatible reflection use
  producing trim/AOT warnings under `IsAotCompatible`, and an annotated
  library consumed by it.
- **Variants & failure cases:** Warnings only appear with the analyzers
  enabled (configuration-dependent diagnostics); `InvariantGlobalization`
  and feature switches removing code paths.
- **Constraints:** .NET 7+/8+ for AOT.
- **Defined by:** SDK + runtime. **Confidence:** high. **Source:** Microsoft
  Learn trimming/Native AOT documentation.

### 2.12 Packages and Dependency Resolution (PKG)

#### PKG-01 — PackageReference and transitive graph resolution

- **Behavior:** Direct `PackageReference`s pull transitive dependencies;
  NuGet resolves each package ID to a *single* version per project via
  nearest-wins and lowest-applicable-version rules; conflicts surface as
  NU1605 (downgrade) or NU1107 (version conflict) diagnostics. Floating
  versions (`8.*`) and version ranges make the resolved graph
  restore-time-dependent.
- **Demonstrated by:** Two direct packages sharing a transitive dependency at
  different versions, with the unified version observable in
  `obj/project.assets.json`; a floating version; a pinned exact version.
- **Variants & failure cases:** `ExcludeAssets`/`IncludeAssets`/
  `PrivateAssets` metadata changing which assets flow (compile vs runtime vs
  analyzers vs buildTransitive) and whether they flow to consumers.
- **Constraints:** NuGet restore precedes compilation; `nuget.config`
  source configuration affects results.
- **Defined by:** Package manager. **Confidence:** high. **Source:** NuGet
  documentation on dependency resolution.

#### PKG-02 — Central Package Management and lock files

- **Behavior:** `Directory.Packages.props` with
  `ManagePackageVersionsCentrally` moves versions out of csproj files
  (`PackageVersion` items, per-project `VersionOverride`), so the csproj no
  longer states the resolved version. `RestorePackagesWithLockFile` produces
  `packages.lock.json`, pinning the full graph (with locked mode enforcing
  it).
- **Demonstrated by:** CPM across two projects with one override; a lock file
  committed and restored in locked mode.
- **Variants & failure cases:** CPM + floating versions restrictions;
  transitive pinning (`CentralPackageTransitivePinningEnabled`).
- **Constraints:** NuGet 6+ tooling.
- **Defined by:** Package manager. **Confidence:** high. **Source:** NuGet
  Central Package Management documentation.

#### PKG-03 — Package asset selection: TFM folders, RIDs, and asset types

- **Behavior:** A package carries per-TFM compile/runtime assets
  (`lib/net8.0`, `lib/netstandard2.0`, `ref/`), RID-specific native assets
  (`runtimes/<rid>/native`), and build logic (`build/`, `buildTransitive/`
  props/targets that inject properties/items into consumers). Which
  assemblies (and thus which API surface/version) a project compiles against
  depends on nearest-TFM selection — the *same package* provides different
  symbols to different TFMs of one multi-targeted project.
- **Demonstrated by:** A dependency consumed from net9.0 and netstandard2.0
  targets of one library where the package's API differs per TFM (`#if`
  needed); a package whose buildTransitive targets define a constant.
- **Variants & failure cases:** `contentFiles` delivering source (PKG-07);
  RID selection only at publish/runtime for natives; asset-selection
  failures (NU1202 incompatibility).
- **Constraints:** NuGet conventions.
- **Defined by:** Package manager. **Confidence:** high. **Source:** NuGet
  package-structure documentation.

#### PKG-04 — Packages delivering analyzers and source generators

- **Behavior:** Assets under `analyzers/dotnet/cs` load into the compiler for
  consuming projects — a package changes diagnostics and *adds generated
  code* without contributing any referenced assembly
  (`DevelopmentDependency`/`PrivateAssets=all` packages have compile-time
  effect but no runtime reference).
- **Demonstrated by:** A generator package (e.g., a toolkit generator)
  producing members consumed by user code, with no runtime dependency in the
  output; an analyzer package raising a custom diagnostic.
- **Variants & failure cases:** Generator version vs Roslyn version
  compatibility (`analyzers/dotnet/roslyn4.x` folders); disabling via
  `ExcludeAssets=analyzers`.
- **Constraints:** Roslyn-compatible packages.
- **Defined by:** Package manager + compiler. **Confidence:** high.
  **Source:** NuGet analyzer-package conventions documentation.

#### PKG-05 — FrameworkReference and shared frameworks

- **Behavior:** `FrameworkReference` (e.g., `Microsoft.AspNetCore.App`)
  references a *shared framework* — a large assembly set resolved from the
  installed runtime, not NuGet packages. The Web SDK adds it implicitly.
  Symbols arrive without any PackageReference; versioning follows the TFM
  and installed runtime, not the package graph.
- **Demonstrated by:** A class library with an explicit
  `FrameworkReference` to ASP.NET Core using its types; contrast with a
  project lacking it where the same usings fail.
- **Variants & failure cases:** Windows Desktop framework references;
  runtime-pack targeting during publish.
- **Constraints:** .NET Core 3.0+ model.
- **Defined by:** SDK. **Confidence:** high. **Source:** Microsoft Learn
  shared-framework documentation.

#### PKG-06 — In-box vs package type overlap and unification

- **Behavior:** Some types ship both in the framework and as packages
  (`System.Text.Json` downlevel, `System.Memory` for netstandard2.0,
  `Microsoft.Bcl.AsyncInterfaces`); polyfill packages plus type forwarding
  keep identity unified — or fail to, yielding duplicate-type errors/warnings
  when both source polyfills and package types exist (interacts with
  RES-03). The `NETStandard.Library` facade set resolves netstandard
  references to per-platform implementations.
- **Demonstrated by:** A netstandard2.0 target consuming `Span<T>` via
  `System.Memory` while the net9.0 target uses the in-box type — same source
  symbol name, different defining assemblies per TFM.
- **Variants & failure cases:** Polyfill attribute classes
  (`IsExternalInit`, `RequiredMemberAttribute`) compiled into multiple
  assemblies as internal types.
- **Constraints:** Multi-targeting.
- **Defined by:** Ecosystem + package manager. **Confidence:** high.
  **Source:** Microsoft Learn / NuGet documentation.

#### PKG-07 — Source-distributing packages (contentFiles)

- **Behavior:** Packages can inject *source files* into consumers via
  `contentFiles/cs/<tfm>/` with `buildAction=Compile` — the consuming
  project compiles code it does not contain, with the package (not a
  referenced assembly) as origin. Common for internal-visibility polyfills
  and unit-test helpers.
- **Demonstrated by:** A consumed source-only package whose type appears in
  the consumer's own assembly.
- **Variants & failure cases:** `PrivateAssets` defaults include
  contentfiles; language-specific folders (`cs`, `any`).
- **Constraints:** PackageReference-only feature.
- **Defined by:** Package manager. **Confidence:** medium-high.
  **Source:** NuGet contentFiles documentation.

### 2.13 Invalid Programs and Diagnostic Semantics (DIA)

#### DIA-01 — Error-bearing compilations still have (partial) semantics

- **Behavior:** Roslyn produces a full symbol table and partial semantic
  model in the presence of errors: unresolved names become error types,
  other members of an erroneous type still bind, and unaffected files are
  fully analyzable. A graph's behavior on broken code (missing using,
  missing package, typo'd type) is a distinct observable dimension from its
  behavior on valid code.
- **Demonstrated by:** A deliberately broken variant/configuration: one file
  referencing a nonexistent type (CS0246) while sibling declarations remain
  resolvable; a project with a removed PackageReference.
- **Variants & failure cases:** Cascading errors; syntax errors (worse
  recovery) vs binding errors (good recovery); IDE-style analysis vs strict
  build.
- **Constraints:** Compiler behavior, not language.
- **Defined by:** Compiler (Roslyn). **Confidence:** high. **Source:**
  Roslyn documentation/API behavior.

#### DIA-02 — Ambiguity and duplicate-definition diagnostics

- **Behavior:** Distinct, well-defined failure modes: ambiguous simple name
  across usings (CS0104), ambiguous overload (CS0121), ambiguous type across
  references (CS0433), source/reference conflict warning (CS0436), duplicate
  type in one namespace without `partial` (CS0101), duplicate member
  (CS0111). Each implies a different graph condition (two candidates, no
  winner vs. shadowing with winner).
- **Demonstrated by:** Minimal broken examples of each, isolated so the rest
  of the fixture still compiles (e.g., in an excluded-by-default variant
  configuration).
- **Variants & failure cases:** Ambiguities resolvable by alias/qualification
  shown alongside their fixes.
- **Constraints:** None.
- **Defined by:** Language (conditions) + compiler (codes).
  **Confidence:** high. **Source:** ECMA-334; compiler diagnostics
  reference.

#### DIA-03 — Feature-gate and capability errors

- **Behavior:** Programs that are invalid only under some configurations:
  LangVersion too low (CS8370-family), `unsafe` without AllowUnsafeBlocks
  (CS0227), DIMs/static abstracts on runtimes lacking support,
  ref structs where boxing would occur, `await` in sync methods, `yield` in
  invalid contexts, entry-point conflicts. Validity is a function of
  (source, LangVersion, TFM, properties).
- **Demonstrated by:** The same source included in two build configurations
  where exactly one fails, with the failure mode documented.
- **Variants & failure cases:** Errors vs warnings for near-miss cases;
  `RequiresPreviewFeatures` diagnostics.
- **Constraints:** As per feature.
- **Defined by:** Compiler + SDK. **Confidence:** high. **Source:**
  Microsoft Learn compiler messages documentation.

#### DIA-04 — Configuration-dependent warning sets on valid code

- **Behavior:** Fully valid programs whose diagnostic output differs by
  configuration: nullable warnings on/off, unused-using (and doc-comment
  CS1591 only when XML docs enabled), unreachable code under `#if`,
  warning-wave additions by AnalysisLevel, platform-compatibility analyzer
  warnings (CA1416) only for OS-specific TFMs. Diagnostics are part of
  observable semantics but not stable across configurations.
- **Demonstrated by:** One file whose warning list is documented under two
  project configurations.
- **Variants & failure cases:** Warnings promoted to errors flipping build
  success; suppressed-by-generator diagnostics.
- **Constraints:** SDK analyzers.
- **Defined by:** Compiler + SDK. **Confidence:** high. **Source:**
  Microsoft Learn code-analysis documentation.

### 2.14 Ecosystem Conventions with Semantic Weight (ECO)

#### ECO-01 — Test frameworks and attribute-driven discovery

- **Behavior:** xUnit/NUnit/MSTest locate tests via attributes
  (`[Fact]`/`[Theory]`+`[InlineData]`/`[MemberData]`, `[Test]`/`[TestCase]`,
  `[TestMethod]`) — call sites exist only in the runner; data sources
  reference members by *string name* (`[MemberData(nameof(Cases))]`).
  Test projects are ordinary libraries with `Microsoft.NET.Test.Sdk` (VSTest)
  or generate their own entry point (xUnit v3, Microsoft.Testing.Platform) —
  OutputType and entry-point synthesis differ by framework generation.
  Lifecycle interfaces (`IClassFixture<T>`, constructor injection in xUnit)
  create framework-mediated instantiation edges.
- **Demonstrated by:** A test project with facts, theories with inline and
  member data, a fixture class, and the runner package references; ideally
  one classic VSTest-style and a note on MTP/xunit-v3 entry-point synthesis.
- **Variants & failure cases:** `IsTestProject`/auto-added properties;
  parallelism attributes; MSTest source-generator mode (recent versions).
- **Constraints:** Framework packages; runner integration.
- **Defined by:** Ecosystem convention. **Confidence:** high (medium for
  newest runner generations). **Source:** xUnit/NUnit/MSTest documentation.

#### ECO-02 — System.Text.Json source-generated serialization

- **Behavior:** `[JsonSerializable(typeof(T))]` on a partial
  `JsonSerializerContext` triggers a generator producing metadata/serialization
  members for the closed type graph of `T` — an attribute argument closes
  over an entire type universe, and serialization member access
  (property getters/setters) happens through generated code rather than
  reflection. Contrast with reflection-mode `JsonSerializer` (invisible
  member access).
- **Demonstrated by:** A partial context with a couple of types, options
  attributes (`JsonSourceGenerationOptions`), and call sites using the
  context; one reflection-mode call for contrast.
- **Variants & failure cases:** AOT interplay (reflection mode warns);
  polymorphism attributes (`[JsonDerivedType]`) declaring type hierarchies
  in metadata.
- **Constraints:** .NET 6+.
- **Defined by:** Ecosystem (BCL generator). **Confidence:** high.
  **Source:** Microsoft Learn System.Text.Json source-generation docs.

#### ECO-03 — GeneratedRegex

- **Behavior:** `[GeneratedRegex("pattern")]` on a `partial` method (or
  C# 13+ partial property) makes a generator emit an optimized `Regex`
  implementation — pattern strings become compile-time inputs producing
  members and diagnostics (invalid pattern = build-time error via analyzer).
- **Demonstrated by:** A partial method with the attribute, its call sites,
  and an intentionally invalid pattern variant for the diagnostic.
- **Variants & failure cases:** Options arguments; culture-sensitive
  variants.
- **Constraints:** .NET 7+.
- **Defined by:** Ecosystem (BCL generator). **Confidence:** high.
  **Source:** Microsoft Learn regular expression source generator docs.

#### ECO-04 — ASP.NET Core structural conventions

- **Behavior:** Minimal APIs pass lambdas/method groups to `MapGet`-style
  methods where *parameter binding* is convention-driven (by type/name/
  attributes) and may be rewritten by the request-delegate generator via
  interceptors under AOT; MVC controllers are discovered by naming/attribute
  conventions and invoked reflectively; Razor components/pages compile from
  markup files into classes whose base types and namespaces follow folder
  conventions (`_Imports.razor` cascading usings); DI containers construct
  types from constructor signatures registered by lambdas or assembly scans.
- **Demonstrated by:** A minimal-API app with a few endpoints (lambda +
  method group), one controller, one Razor component with `@code`, DI
  registrations consumed via constructor injection.
- **Variants & failure cases:** Route string ↔ handler parameter coupling;
  `[FromServices]`/`[FromBody]` overrides; conventions vs explicit
  attributes.
- **Constraints:** Web SDK + shared framework.
- **Defined by:** Ecosystem convention (framework). **Confidence:** high.
  **Source:** Microsoft Learn ASP.NET Core documentation.

#### ECO-05 — Reflection- and string-mediated linkage

- **Behavior:** Idiomatic .NET creates edges no static analysis of calls can
  see: `Type.GetType("Ns.T, Asm")`, `Activator.CreateInstance`,
  `GetMethod("Name")`, DI registrations by scanning, configuration binding
  (`IConfiguration.Bind`/`GetSection("Key")` mapping strings to property
  names), and attribute-string type references. Trimming annotations
  (MET-06/BLD-09) are the ecosystem's own acknowledgment of this gap.
- **Demonstrated by:** A small plugin-style load via `Type.GetType`, a
  config-bound options class, and a reflection call to a private member —
  each with the target member present so the "invisible" edge is checkable.
- **Variants & failure cases:** `nameof`-based reflection (checkable) vs raw
  strings (not); `AssemblyLoadContext` dynamic loading.
- **Constraints:** None.
- **Defined by:** Ecosystem convention. **Confidence:** high.
  **Source:** Microsoft Learn reflection documentation.

#### ECO-06 — Codegen from non-C# inputs (protobuf, OpenAPI, EF migrations)

- **Behavior:** Build-integrated generators compile foreign sources into C#:
  `Grpc.Tools` turns `Protobuf` items (.proto) into partial message/service
  classes at build time (in obj/); OpenAPI client generators (NSwag/Kiota)
  and EF Core scaffolding/migrations produce *checked-in* generated C#
  (`*.Designer.cs`, snapshot classes) that user code extends via partials.
  The origin of a symbol may be a non-C# file or a tool run, with
  regeneration drift as a real failure mode.
- **Demonstrated by:** A project with a .proto compiled via Grpc.Tools and a
  service implementation deriving from the generated base; an EF-style
  checked-in generated file paired with a partial extension.
- **Variants & failure cases:** Generated-at-build (invisible in tree) vs
  generated-and-committed (visible, marked `<auto-generated/>`);
  `<auto-generated>` header suppressing analyzers/style rules.
- **Constraints:** Tool packages; MSBuild integration.
- **Defined by:** Ecosystem convention. **Confidence:** high (medium on
  specific tool behaviors). **Source:** gRPC for .NET and EF Core
  documentation.

#### ECO-07 — Attribute-driven member synthesis toolkits

- **Behavior:** Widely used generators synthesize members from annotated
  ones: CommunityToolkit.Mvvm turns `[ObservableProperty]` fields into
  generated public properties (the *used* symbol never appears in user
  source), `[RelayCommand]` methods into command properties; similar
  patterns exist for mapping, logging (`[LoggerMessage]` partial methods),
  and DI registration. User code references generated members; generated
  code references user members.
- **Demonstrated by:** A viewmodel-style class with an
  `[ObservableProperty]` field and a `[LoggerMessage]` partial method, with
  consumers using the generated surface.
- **Variants & failure cases:** Partial-class requirement; naming
  transformations (field `_name` → property `Name`) as identity mapping
  challenges; generator diagnostics for misuse.
- **Constraints:** Toolkit packages; Roslyn 4 generators.
- **Defined by:** Ecosystem convention. **Confidence:** high.
  **Source:** CommunityToolkit.Mvvm and .NET logging-generator docs.

#### ECO-08 — Solutions and workspace composition

- **Behavior:** `.sln` (classic) and `.slnx` (XML, newer SDKs) group projects
  with solution folders (virtual, not directories) and per-configuration
  project mappings; multiple solutions may include overlapping project sets;
  projects build fine without any solution (`dotnet build` on csproj or
  folder). Which projects are "in" the codebase is a workspace choice with
  no single source of truth.
- **Demonstrated by:** A solution including all fixture projects with one
  solution folder; a project intentionally left out of the solution but
  referenced by ProjectReference (still builds).
- **Variants & failure cases:** Solution build-order/config mappings;
  `dotnet sln` manipulation; `.slnf` solution filters.
- **Constraints:** `.slnx` requires recent SDK/VS tooling.
- **Defined by:** Toolchain convention. **Confidence:** high (medium for
  .slnx specifics). **Source:** Microsoft Learn solution-file
  documentation.

---

## 3. Features with Material Changes Between Recent Versions

| Version (ships with) | Graph-relevant additions/changes |
| --- | --- |
| C# 8 (.NET Core 3.0) | Nullable reference types; default interface members; async streams/`await foreach`/`await using`; indices/ranges + implicit patterns; using declarations; switch expressions and recursive patterns; static local functions |
| C# 9 (.NET 5) | Records + synthesized members; init accessors (`IsExternalInit` modreq); top-level statements; covariant returns; function pointers; module initializers; extended partial methods; target-typed `new`; extension `GetEnumerator`; source generators (Roslyn v1) |
| C# 10 (.NET 6) | `global using` + ImplicitUsings; file-scoped namespaces; record structs; struct parameterless ctors/field initializers; lambda natural types/attributes/return types; interpolated string handlers; `CallerArgumentExpression`; enhanced `#line`; incremental generators |
| C# 11 (.NET 7) | `file` types; generic attributes; static abstract/virtual interface members; required members; list patterns; `ref` fields + `scoped`; raw strings; UTF-8 literals; checked operators; auto-default structs; extended `nameof` scope |
| C# 12 (.NET 8) | Primary constructors (non-record); collection expressions + `CollectionBuilder`; alias any type; `ref readonly` params; inline arrays; lambda default params; interceptors (experimental); `[Experimental]` |
| C# 13 (.NET 9) | `params` collections; `System.Threading.Lock`-aware `lock`; partial properties/indexers; `allows ref struct` + ref-struct interfaces; `[OverloadResolutionPriority]`; ref/unsafe in iterators & async; implicit index in initializers; `field` (preview) |
| C# 14 (.NET 10) | Extension members (`extension` blocks incl. properties/static/operators); `field` keyword final; null-conditional assignment; `nameof` unbound generics; lambda param modifiers w/o types; partial constructors/events; user-defined compound assignment; first-class span conversions; SDK: file-based apps (`#:` directives) |

Toolchain shifts worth flagging: SDK-style projects and PackageReference
(vs packages.config) as the modern baseline; Roslyn incremental generators
replacing v1 generators; `.slnx` solution format; Central Package
Management; Microsoft.Testing.Platform emerging beside VSTest; trimming/AOT
analyzers becoming mainstream validity surfaces.

---

## 4. Implementation-Defined or Unspecified Behavior

- **Compiler-generated names** (closures, state machines, backing fields,
  anonymous types, file-local mangling, `<Main>$`, `<Clone>$`): unspecified
  by the language; stable Roslyn conventions that have changed historically.
- **Static initialization timing:** `beforefieldinit` permits relaxed timing
  of static field initialization for types without static constructors;
  observable ordering differs by runtime and optimization.
- **Field initializer order across `partial` parts:** within one part it is
  textual; across parts it depends on the order files are given to the
  compiler — effectively implementation/build-defined.
- **Module initializer ordering** across multiple `[ModuleInitializer]`
  methods: unspecified.
- **Floating-point excess precision:** the spec permits operations to be
  performed at higher precision than the nominal type; results can differ by
  platform/JIT.
- **Anonymous type reuse and naming:** structural reuse within an assembly is
  compiler behavior, not spec.
- **Reflection member ordering** (`GetMembers`) and metadata token order:
  unspecified; graphs must not depend on it.
- **String literal interning:** guaranteed for literals per spec, but
  runtime-wide interning behavior of computed strings is implementation
  detail.
- **GC/finalization timing** (`~T()` execution, weak references): runtime
  behavior, non-deterministic.
- **Default struct member layout:** C# emits sequential layout for structs
  and auto for classes by default; actual field placement is runtime
  territory (`StructLayout` overrides).
- **Warning sets:** which warnings exist (and warning waves) is
  compiler-version-defined, not language-defined.
- **`Environment`-dependent lexing edge:** none for line endings semantically,
  but raw-string indentation trimming rules are precisely specified —
  a good positive test rather than an ID case.

---

## 5. Areas Where My Knowledge May Be Incomplete

- **C# 14 fine print:** exact metadata encoding of extension members,
  the final shipped set of extension operators and compound-assignment
  operator rules, and the precise scope of first-class span conversions.
  Confidence medium; verify against csharplang specs for the C# 14 release.
- **Interceptors:** the location-token format (checksum-based) and
  stabilization status past .NET 9/10; gating property names have changed
  during previews.
- **File-based apps (`dotnet run app.cs`):** exact directive set and
  semantics (`#:package`, `#:sdk`, `#:property`) — medium confidence.
- **xUnit v3 / Microsoft.Testing.Platform:** entry-point generation details
  and project-shape changes are recent and evolving.
- **`.slnx`** schema details and SDK version gates.
- **Exact default implicit-usings lists per SDK family** — directionally
  right, itemization may be off.
- **NuGet edge rules:** precise tie-breaks in nearest-wins across diamond
  graphs with ranges, and CPM transitive pinning corner cases.
- **MSTest/NUnit newest source-generator-based modes.**
- **Legacy surfaces** (non-SDK csproj, packages.config, binding redirects,
  .NET Framework-specific behaviors like AppDomains): known shape, but I
  treated them as out of the modern baseline; a fixture targeting legacy
  would need dedicated research.
- **WinUI 3 / MAUI XAML compilation** specifics (XamlC vs reflection-based
  loading) — medium confidence.

---

## 6. Final Audit: Categories or Families Possibly Still Missing

Reviewed against the checklist; candidates a fixture designer may still want
to weigh:

1. **Legacy project system variants** — non-SDK csproj, packages.config,
   app.config binding redirects, AssemblyInfo-only attribute flow. Excluded
   from baseline; include only if legacy support is in scope.
2. **T4 text templates** (`.tt` → generated `.cs`, design-time vs runtime
   templates) — older but still-encountered codegen family not covered by a
   dedicated item.
3. **Cross-language *solutions*** are covered for consumption (ASM-09), but
   C# projects *exposing* APIs shaped for VB/F# consumers (case-collisions,
   optional/ref conventions) are not separately itemized.
4. **AssemblyLoadContext plugin architectures** (runtime assembly graph
   diverging from compile-time graph) — touched in ECO-05, may deserve its
   own item if runtime-loading fidelity matters.
5. **Runtime code generation** (`System.Reflection.Emit`,
   `DynamicMethod`, Roslyn-as-a-library compiling code at runtime) —
   deliberately out of static-graph scope, but worth an explicit exclusion
   decision.
6. **Workload-gated platforms** (MAUI, Blazor WASM, Unity's compilation
   pipeline and asmdefs) — Unity in particular has its own assembly-definition
   and IL-postprocessing world that a general fixture likely excludes.
7. **Style/formatting-only surfaces** (EditorConfig code-style rules,
   `dotnet format`) — intentionally omitted as non-semantic beyond the
   diagnostics channel already covered in MET-05.
8. **Security/signing details** (delay signing, public sign, ILLink
   substitutions files) — niche build inputs with minor semantic surface.
9. **Documentation pipeline** beyond cref (DocFX, `<inheritdoc/>` semantics —
   `<inheritdoc/>` resolution is a real cross-symbol relationship handled by
   tooling; consider an item if doc-graph fidelity matters).
10. **Interactive/REPL submissions** (csi chained submissions) — partially
    covered in SRC-12; full submission-chain semantics omitted.
11. **Preprocessor-driven API diffs in *published packages*** (same package
    ID whose surface differs per TFM) — covered via PKG-03/PKG-06 jointly;
    no standalone item.
12. **Analyzer *suppressors* and severity escalation in CI vs local** —
    covered inside MET-05; standalone treatment optional.

No further whole categories identified beyond the above; the checklist's 14
sections plus these audit candidates are believed to span the semantic
surface a C# code-graph fixture should consider.
