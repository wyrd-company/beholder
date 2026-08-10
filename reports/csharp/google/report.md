# Comprehensive C# Feature Coverage for a Code-Graph Reference Project

**Key Points:**
*   Research suggests that evaluating code-graph generators for C# requires modeling complex, multi-phase compilation features such as Roslyn Source Generators and Compile-Time Interceptors to accurately capture modern semantic boundaries.
*   It seems likely that edge-case resolution mechanics, such as handling distinct namespace collisions using `extern alias` directives, represent a significant blind spot in rudimentary graph implementations.
*   The evidence leans toward MSBuild conditional constructs (`Choose`, `When`) introducing profound hidden complexity, as they conditionally alter compilation inputs, dependencies, and constant definitions prior to the language parser's execution.
*   Recent C# language evolutions (spanning C# 10 through C# 13) have introduced paradigm-shifting structural constructs—including file-local types, primary constructors, and `ref struct` generic constraints—that fundamentally alter symbol identity, visibility, and dispatch resolution in a correct code graph.

**Introduction**
The evaluation of code-graph generators necessitates a rigorously constructed reference project that exercises the full spectrum of a target language's semantic and structural capabilities. For C#, a language characterized by rapid evolution and deep integration with the .NET ecosystem, simple abstract syntax tree (AST) extraction is insufficient. A correct code graph must accurately represent semantic entities, identities, relationships, resolution mechanisms, types, visibility boundaries, dispatch targets, source inclusions, and diagnostic states across a diverse array of compilation contexts. This research report compiles a comprehensive, source-grounded checklist of C# language, toolchain, build, package, and ecosystem features that a broad, idiomatic, self-contained reference project must cover.

**Methodology and Scope**
The investigation focuses on materially distinct C# behaviors and project conditions whose omission would leave a meaningful gap in code-graph coverage. The scope spans language-defined semantics, official compiler (`csc.exe`) behaviors, Roslyn workspace boundaries, MSBuild configuration intricacies, and multi-location program elements (such as partial classes and source-generated artifacts). The research prioritizes primary sources, including official language specifications, compiler documentation, and release notes across supported implementations (.NET Standard, .NET Framework, .NET Core, up to .NET 8/9 and C# 13). 

---

## 1. Generated, Synthesized, and Multi-Location Program Elements

The representation of entities that span multiple physical files or exist entirely in memory during the compilation process poses one of the most severe challenges for code-graph generators.

### CS-FEAT-001: Roslyn Source Generators
*   **Feature or behavior name**: Compile-Time Metaprogramming via Source Generators.
*   **Description of the materially distinct behavior**: Roslyn Source Generators enable the compiler to inspect user code via a `SemanticModel` and dynamically inject additional C# source code into the compilation process [cite: 1], [cite: 2]. The generated code does not exist on disk prior to the build but contributes structurally and semantically to the final assembly [cite: 1], [cite: 3]. A code graph must represent these synthesized files, their semantic connections to the original source, and the specific generator that produced them.
*   **Observable project content**:
    ```csharp
    // Generator project (must be .NET Standard 2.0)
    [Generator]
    public class CustomGenerator : ISourceGenerator {
        public void Execute(GeneratorExecutionContext context) {
            context.AddSource("Generated.cs", SourceText.From("public partial class Target { public void GenMethod() {} }", Encoding.UTF8));
        }
        public void Initialize(GeneratorInitializationContext context) {}
    }
    ```
*   **Important variants, interactions, counterexamples, and failure cases**: Requires the generating project to target `.NET Standard 2.0` and explicitly set `LangVersion` [cite: 1], [cite: 4]. Incremental generators (introduced in .NET 6) represent a performance optimization variant but semantically achieve the same outcome [cite: 5]. If the graph relies purely on static disk scanning, generated methods will appear as unresolved references, causing massive graph breakage.
*   **Constraints**: Requires C# 9.0+, `.NET 5+` SDK, and `.NET Standard 2.0` for the generator library itself [cite: 1], [cite: 1].
*   **Classification**: Normative / Official compiler behavior.
*   **Confidence**: High.

### CS-FEAT-002: Compile-Time Method Interceptors
*   **Feature or behavior name**: Method Interceptors (Experimental).
*   **Description of the materially distinct behavior**: Interceptors allow the rerouting of specific method calls to substitute code at compile time, bypassing runtime reflection [cite: 6], [cite: 7]. The interception is driven by the `InterceptsLocationAttribute`, which specifies the exact file path, line, and character of the invocation to be hijacked [cite: 8]. Code graphs must track both the original intended call edge and the substituted interceptor edge.
*   **Observable project content**:
    ```xml
    <PropertyGroup>
        <InterceptorsPreviewNamespaces>$(InterceptorsPreviewNamespaces);MyProject.Interceptors</InterceptorsPreviewNamespaces>
    </PropertyGroup>
    ```
    ```csharp
    namespace System.Runtime.CompilerServices {
        [AttributeUsage(AttributeTargets.Method, AllowMultiple = true)]
        public sealed class InterceptsLocationAttribute(string filePath, int line, int character) : Attribute {}
    }
    
    namespace MyProject.Interceptors {
        public static class MyInterceptor {
            [InterceptsLocation("Program.cs", 10, 15)]
            public static void InterceptedMethod() { /* New behavior */ }
        }
    }
    ```
*   **Important variants, interactions, counterexamples, and failure cases**: Interceptors do not support properties, constructors, generic methods, or `ref`/`out` parameters [cite: 9]. A graph generator parsing only syntax will emit edges to the originally called method; a semantically accurate generator must emit edges to the interceptor method for the specific coordinate defined.
*   **Constraints**: C# 12, .NET 8 Preview / .NET 9. Requires explicit MSBuild opt-in via `<InterceptorsPreviewNamespaces>` [cite: 6], [cite: 10].
*   **Classification**: Implementation-defined / Experimental.
*   **Confidence**: High.

### CS-FEAT-003: Partial Classes, Methods, and Properties
*   **Feature or behavior name**: Partial Type and Member Declarations.
*   **Description of the materially distinct behavior**: The `partial` modifier allows the definition of classes, structs, interfaces, methods, and (as of C# 13) properties and indexers to be split across multiple physical files [cite: 11], [cite: 12]. The code graph must aggregate these dispersed syntax nodes into a singular semantic symbol. Furthermore, C# 9 updated partial methods to allow return types and `out` parameters, provided an implementation is strictly supplied [cite: 3].
*   **Observable project content**:
    ```csharp
    // File 1: Declaration
    public partial class Employee {
        public partial string Name { get; set; } // C# 13 partial property
        partial void CalculatePay();
    }
    // File 2: Implementation
    public partial class Employee {
        private string _name;
        public partial string Name {
            get => _name;
            set => _name = value;
        }
        partial void CalculatePay() { /* Implementation */ }
    }
    ```
*   **Important variants, interactions, counterexamples, and failure cases**: C# 13 partial properties require an exact signature match and cannot utilize auto-property syntax [cite: 13], [cite: 14]. Different parts of a partial class can inherit from different interfaces, merging into a unified inheritance graph [cite: 11].
*   **Constraints**: Core C# 2.0 feature [cite: 12]. Extended in C# 9 (methods) and C# 13 (properties) [cite: 13], [cite: 3].
*   **Classification**: Normative.
*   **Confidence**: High.

---

## 2. Modules, Packages, Workspaces, and Dependencies

Dependency boundaries and workspace configurations dictate how semantic resolution occurs across disparate assemblies.

### CS-FEAT-004: Extern Alias Directive
*   **Feature or behavior name**: `extern alias` for Assembly Disambiguation.
*   **Description of the materially distinct behavior**: When two referenced assemblies define a type with the exact same fully qualified name (matching namespace and class name), `extern alias` resolves the collision by isolating the namespaces into custom root-level namespaces defined at the build level [cite: 15], [cite: 16]. The code graph must properly isolate the type resolution paths according to the alias mapped in the `.csproj` file.
*   **Observable project content**:
    ```xml
    <ItemGroup>
      <Reference Include="Grid.v1.dll">
        <Aliases>GridV1</Aliases>
      </Reference>
      <Reference Include="Grid.v2.dll">
        <Aliases>GridV2</Aliases>
      </Reference>
    </ItemGroup>
    ```
    ```csharp
    extern alias GridV1;
    extern alias GridV2;
    // Resolving ambiguous types:
    GridV1::ThirdParty.Grid myV1Grid = new GridV1::ThirdParty.Grid();
    GridV2::ThirdParty.Grid myV2Grid = new GridV2::ThirdParty.Grid();
    ```
*   **Important variants, interactions, counterexamples, and failure cases**: The `::` operator (namespace alias qualifier) must be modeled [cite: 17], [cite: 18]. If a graph generator relies strictly on global namespace resolution, it will encounter fatal ambiguity errors or incorrectly collapse the two distinct types into a single entity.
*   **Constraints**: Introduced in C# 2.0 [cite: 16]. Applicable to assemblies with matching fully qualified namespaces [cite: 19].
*   **Classification**: Normative.
*   **Confidence**: High.

### CS-FEAT-005: Global and Implicit Usings
*   **Feature or behavior name**: Project-Wide Namespace Imports.
*   **Description of the materially distinct behavior**: C# 10 introduced `global using` directives, which export a namespace import to every source file in the current compilation [cite: 20], [cite: 21]. Furthermore, MSBuild `<ImplicitUsings>enable</ImplicitUsings>` dynamically synthesizes global using statements based on the project SDK type (e.g., Web, Console) [cite: 21].
*   **Observable project content**:
    ```csharp
    // GlobalUsings.cs
    global using System.Text;
    global using static System.Console;
    global using Env = System.Environment;
    ```
*   **Important variants, interactions, counterexamples, and failure cases**: Global usings interact heavily with `extern alias` and traditional local usings, potentially causing hidden ambiguities [cite: 21]. Code graphs must inject these global scopes into the resolution chain of every file in the project. Omitting the implicit MSBuild generated usings will result in massive "type not found" diagnostic errors across the graph.
*   **Constraints**: C# 10, .NET 6+ SDK [cite: 21], [cite: 22].
*   **Classification**: Normative / Build System Integration.
*   **Confidence**: High.

---

## 3. Build Systems, Conditional Inclusion, and Platform Variation

### CS-FEAT-006: MSBuild Choose/When Conditional Constructs
*   **Feature or behavior name**: MSBuild Either/Or Processing.
*   **Description of the materially distinct behavior**: The MSBuild `<Choose>`, `<When>`, and `<Otherwise>` elements allow for dynamic, conditional evaluation of project properties and items based on configuration, platform, or environmental variables [cite: 23], [cite: 24]. This can conditionally include entire sets of `.cs` source files or alter preprocessor directives (`DefineConstants`).
*   **Observable project content**:
    ```xml
    <Project>
      <Choose>
        <When Condition=" '$(Configuration)'=='Debug' ">
          <PropertyGroup>
            <DefineConstants>DEBUG;TRACE;DEV_ENV</DefineConstants>
          </PropertyGroup>
          <ItemGroup>
            <Compile Include="UnitTesting\*.cs" />
          </ItemGroup>
        </When>
        <Otherwise>
          <PropertyGroup>
            <DefineConstants>TRACE;PROD_ENV</DefineConstants>
          </PropertyGroup>
          <ItemGroup>
            <Compile Remove="UnitTesting\*.cs" />
          </ItemGroup>
        </Otherwise>
      </Choose>
    </Project>
    ```
*   **Important variants, interactions, counterexamples, and failure cases**: The first `<When>` element that evaluates to `true` is executed, and all subsequent branches are skipped [cite: 23], [cite: 25]. A code graph must model a specific configuration matrix (e.g., Debug|x64) and correctly evaluate these MSBuild conditions to determine which files even exist in the compilation context [cite: 23], [cite: 26].
*   **Constraints**: Standard MSBuild feature. Cannot be used inside a `<Target>` element [cite: 27].
*   **Classification**: Build System Behavior.
*   **Confidence**: High.

### CS-FEAT-007: Command-Line Compiler Options (csc.exe) overrides
*   **Feature or behavior name**: Core Compiler Options.
*   **Description of the materially distinct behavior**: Properties defined in MSBuild (`.csproj`) translate directly to `csc.exe` compiler flags. Key flags that radically alter semantic graphs include `LangVersion` (altering grammar parsing), `DefineConstants` (altering `#if` macro inclusion), `AllowUnsafeBlocks` (allowing pointer operations), and `PathMap` (modifying diagnostic and macro output paths) [cite: 28], [cite: 29].
*   **Observable project content**:
    ```xml
    <PropertyGroup>
      <LangVersion>preview</LangVersion>
      <AllowUnsafeBlocks>true</AllowUnsafeBlocks>
      <PathMap>C:\local\path=X:\build\path</PathMap>
      <CheckForOverflowUnderflow>true</CheckForOverflowUnderflow>
    </PropertyGroup>
    ```
*   **Important variants, interactions, counterexamples, and failure cases**: `LangVersion=latest` resolves to the most recent minor version installed, whereas `LangVersion=default` is strictly bound to the Target Framework Moniker (TFM) [cite: 30], [cite: 29]. The `PathMap` argument alters how the `CallerFilePathAttribute` resolves strings in the AST [cite: 28], requiring the code graph to trace mapped paths rather than literal disk paths.
*   **Constraints**: Affects all C# versions. Behavior changes depending on SDK version [cite: 30], [cite: 31].
*   **Classification**: Official compiler behavior.
*   **Confidence**: High.

---

## 4. Language-Defined Semantics and Constructs

Recent iterations of the C# language introduce highly complex mechanics that bridge the gap between memory layout, asynchronous control flow, and semantic visibility. 

### CS-FEAT-008: File-Local Types
*   **Feature or behavior name**: File-Scoped Visibility Boundary.
*   **Description of the materially distinct behavior**: Introduced in C# 11, the `file` access modifier restricts the visibility of a top-level type to the specific physical file in which it is declared [cite: 32], [cite: 33]. This prevents naming collisions across a project. Code graphs must encode this distinct visibility scope, ensuring that references in other files to a type with the same name resolve independently.
*   **Observable project content**:
    ```csharp
    // FileA.cs
    file class HiddenHelper { public void DoWork() {} }
    
    // FileB.cs
    file class HiddenHelper { public void DoOtherWork() {} }
    ```
*   **Important variants, interactions, counterexamples, and failure cases**: A file-local type cannot be exposed in the public API of the assembly [cite: 33]. If a graph generator flattens all types into a project-wide namespace bucket, it will incorrectly merge `FileA.HiddenHelper` and `FileB.HiddenHelper`, corrupting the graph structure.
*   **Constraints**: C# 11 and later [cite: 32].
*   **Classification**: Normative.
*   **Confidence**: High.

### CS-FEAT-009: ref struct Generics and Interfaces
*   **Feature or behavior name**: `allows ref struct` Generic Constraint and Interfaces.
*   **Description of the materially distinct behavior**: In C# 13, `ref struct` types (which are confined to the stack) gained the ability to implement interfaces [cite: 14], [cite: 34]. Concurrently, generic type parameters can now specify the `allows ref struct` anti-constraint, indicating that the generic type is permitted to be a stack-only struct [cite: 35], [cite: 34].
*   **Observable project content**:
    ```csharp
    public interface IStackLoggable { void Log(); }
    
    public ref struct LogSpan : IStackLoggable {
        public void Log() { }
    }

    // Generic anti-constraint
    public void ProcessLog<T>(T item) where T : IStackLoggable, allows ref struct {
        item.Log();
    }
    ```
*   **Important variants, interactions, counterexamples, and failure cases**: While a `ref struct` can implement an interface, it cannot be boxed or cast to that interface type (as that requires heap allocation) [cite: 34]. The interface can only be used safely as a generic constraint. A code-graph must accurately map interface implementation relationships without implying valid castability in all contexts.
*   **Constraints**: C# 13, .NET 9 [cite: 35], [cite: 14].
*   **Classification**: Normative.
*   **Confidence**: High.

### CS-FEAT-010: Enhanced Params Collections
*   **Feature or behavior name**: Variable arguments with Spans and Enumerables.
*   **Description of the materially distinct behavior**: Prior to C# 13, the `params` keyword was strictly confined to arrays (`params int[]`) [cite: 35]. C# 13 extends this to support `Span<T>`, `ReadOnlySpan<T>`, and types implementing `IEnumerable<T>` [cite: 13], [cite: 36].
*   **Observable project content**:
    ```csharp
    public void ProcessData(params ReadOnlySpan<int> data) { }
    public void ProcessList(params IEnumerable<string> names) { }

    // Invocation
    ProcessData(1, 2, 3);
    ```
*   **Important variants, interactions, counterexamples, and failure cases**: Overload resolution is fundamentally altered. If a class contains both `Method(params int[])` and `Method(params ReadOnlySpan<int>)`, the C# 13 compiler utilizes overload resolution priority rules to favor the non-allocating `Span` overload [cite: 34]. Graph generators must emulate this exact dispatch logic to draw the correct call edge.
*   **Constraints**: C# 13, .NET 9 [cite: 13], [cite: 36].
*   **Classification**: Normative.
*   **Confidence**: High.

### CS-FEAT-011: Primary Constructors (Classes and Structs)
*   **Feature or behavior name**: Inline Type Declarations via Primary Constructors.
*   **Description of the materially distinct behavior**: C# 12 expanded primary constructors (previously exclusive to `record` types) to all standard classes and structs [cite: 37], [cite: 38]. Parameters declared in a primary constructor are placed in scope for the entirety of the class body [cite: 37], [cite: 39].
*   **Observable project content**:
    ```csharp
    public class DependencyService(ILogger logger, IRepository repo) {
        public void Execute() {
            logger.Log("Executing..."); // 'logger' is in scope here
        }
    }
    ```
*   **Important variants, interactions, counterexamples, and failure cases**: Unlike `record` types, primary constructor parameters on regular classes do *not* automatically synthesize public properties [cite: 38]. They operate similarly to hidden private backing fields. Code graphs must capture these parameters as distinct scoped entities and correctly bind member references to them.
*   **Constraints**: C# 12, .NET 8 [cite: 37], [cite: 40].
*   **Classification**: Normative.
*   **Confidence**: High.

### CS-FEAT-012: Generic Attributes
*   **Feature or behavior name**: Attributes with Generic Type Parameters.
*   **Description of the materially distinct behavior**: C# 11 introduced the ability for classes inheriting from `System.Attribute` to take generic type parameters, replacing the older `typeof()` constructor pattern [cite: 32], [cite: 41].
*   **Observable project content**:
    ```csharp
    public class ValidatorAttribute<T> : Attribute where T : IValidator {}

    [Validator<StringValidator>]
    public class UserDto { }
    ```
*   **Important variants, interactions, counterexamples, and failure cases**: Graph implementations must connect the attribute usage directly to the generic argument's type symbol. Type constraints applied to the generic attribute (`where T : IValidator`) must be evaluated to confirm graph validity [cite: 41].
*   **Constraints**: C# 11, .NET 7 [cite: 32], [cite: 41].
*   **Classification**: Normative.
*   **Confidence**: High.

### CS-FEAT-013: Overload Resolution Priority
*   **Feature or behavior name**: `OverloadResolutionPriorityAttribute`.
*   **Description of the materially distinct behavior**: In C# 13, library authors can dictate the compiler's preference between competing method overloads using the `[OverloadResolutionPriority(int)]` attribute [cite: 13], [cite: 34]. A higher integer value forces the compiler to select that overload, even if a legacy overload might otherwise be considered a closer match.
*   **Observable project content**:
    ```csharp
    public static class MathOps {
        public static void Sum(int[] array) { }
        
        [OverloadResolutionPriority(1)]
        public static void Sum(ReadOnlySpan<int> span) { }
    }
    // Invocation: MathOps.Sum(new int[] { 1, 2, 3 });
    // Resolves to the ReadOnlySpan overload due to priority.
    ```
*   **Important variants, interactions, counterexamples, and failure cases**: If a code graph uses legacy standard dispatch logic, it will incorrectly link the `int[]` invocation to the `int[]` overload. This breaks edge relationships in the graph.
*   **Constraints**: C# 13, .NET 9 [cite: 13], [cite: 34].
*   **Classification**: Normative.
*   **Confidence**: High.

### CS-FEAT-014: Static Abstract Members in Interfaces
*   **Feature or behavior name**: Generic Math and Static Interfaces.
*   **Description of the materially distinct behavior**: C# 11 introduced the ability for interfaces to declare `static abstract` members, allowing types to enforce the implementation of operators (e.g., `+`, `-`) and other static methods [cite: 32], [cite: 41]. This facilitates generic math operations without boxing or reflection [cite: 33].
*   **Observable project content**:
    ```csharp
    public interface IAddable<T> where T : IAddable<T> {
        static abstract T operator +(T left, T right);
    }
    
    public struct Point : IAddable<Point> {
        public static Point operator +(Point left, Point right) => new Point();
    }
    ```
*   **Important variants, interactions, counterexamples, and failure cases**: Dispatch of static virtual methods occurs based on the generic type parameter at runtime (or compile-time generation). A code graph must model the implementation of static interface members and the resolution of constrained generic operator invocations.
*   **Constraints**: C# 11, .NET 7 [cite: 32], [cite: 33].
*   **Classification**: Normative.
*   **Confidence**: High.

### CS-FEAT-015: Record Types and Record Structs
*   **Feature or behavior name**: Immutable Value and Reference Semantics.
*   **Description of the materially distinct behavior**: `record` (C# 9) and `record struct` (C# 10) declare types with built-in value-equality semantics and synthesized methods (`ToString`, `GetHashCode`, `Equals`, and a hidden `<Clone>$` method) [cite: 20], [cite: 21]. 
*   **Observable project content**:
    ```csharp
    public record class Person(string Name);
    public readonly record struct Point(int X, int Y);
    ```
*   **Important variants, interactions, counterexamples, and failure cases**: C# 10 allows making `ToString()` sealed on record classes [cite: 21]. Record primary constructor parameters automatically synthesize public auto-properties [cite: 21], [cite: 38]. Graph generators must synthesize these properties into the graph structure; otherwise, downstream nodes attempting to read `person.Name` will map to nonexistent edges.
*   **Constraints**: C# 9 (records), C# 10 (record structs) [cite: 20], [cite: 42].
*   **Classification**: Normative.
*   **Confidence**: High.

---

## 5. Completeness Review

### Recommended Baseline Version and Implementation Assumptions
To capture a sufficiently robust and forward-looking codebase that evaluates modern code-graph generators, the reference project must establish **.NET 8.0 SDK (Long Term Support)** as its baseline, targeting the **C# 12** language specification by default [cite: 37]. However, the `.csproj` must conditionally include a cross-targeted or isolated component targeting **.NET 9.0 Preview** and `<LangVersion>preview</LangVersion>` to exercise cutting-edge C# 13 features like `params Span<T>` [cite: 35], [cite: 13], [cite: 34] and `ref struct` generic capabilities [cite: 14].

### Material Changes Across Recent Supported Versions
*   **Language-Level Syntactic Sugar to Deep Semantics**: Over the past four years, C# has transitioned from simple syntax sugar (e.g., expression-bodied members) to features that dramatically shift type systems and memory layouts. The introduction of `record structs` [cite: 20], inline arrays [cite: 37], and `allows ref struct` constraints [cite: 14] necessitate code graphs that are intimately aware of stack versus heap allocations and value-equality synthesis [cite: 21].
*   **Compile-Time Dominance**: The paradigm has decisively shifted toward compile-time metaprogramming. Roslyn Source Generators [cite: 1] and Interceptors [cite: 43], [cite: 6] replace runtime Reflection [cite: 44], [cite: 10]. Consequently, a code graph built purely from static file parsing is fundamentally obsolete; the graph generator must invoke or simulate the Roslyn compiler pipeline to accurately map source identity and method execution flows.

### Behaviors That Cannot Coexist in One Configuration
*   **Abstract and Extern Modifiers**: It is a compiler error to apply both `abstract` and `extern` modifiers to the same member. `extern` signifies implementation outside C# (e.g., a native C library via P/Invoke), while `abstract` mandates implementation in derived classes [cite: 45].
*   **Conflicting Implicit Usings**: If a library targets multiple frameworks via MSBuild cross-targeting, the implicit usings generated by the SDK (e.g., ASP.NET Core vs. standard Console) will vary [cite: 21]. Code graphs must evaluate each target framework as a separate analytical pass.

### Features Commonly Omitted from Language Demonstrations
*   **Extern Alias Resolution**: Because namespace collisions are generally resolved by standard refactoring, `extern alias` represents a highly advanced, obscure feature [cite: 17]. Graph generators often skip this, resulting in catastrophic graph collapses in enterprise environments utilizing deeply versioned legacy assemblies.
*   **MSBuild Conditional Branching**: Rudimentary AST tools parse `.cs` files indiscriminately. Code graphs must execute the underlying MSBuild `<Choose>` / `<When>` semantics to avoid including files designated for alternative target architectures or environmental configurations [cite: 23], [cite: 24].

### Areas Where Authoritative Sources Disagree or Remain Unclear
*   **Interceptors in Production**: C# 12/13 Experimental Interceptors exist in a state of high ambiguity. Microsoft documentation stringently dictates that they are subject to breaking changes and should *not* be used in production code [cite: 6], [cite: 7]. However, prominent frameworks (like ASP.NET Core Minimal APIs) are already relying on them for AOT compilation optimization [cite: 10]. Code graph builders face ambiguity regarding whether to invest resources in mapping `InterceptsLocationAttribute` behavior, given its unstable specification.

### Unresolved Research Gaps
After the primary investigation, several likely categories, features, and interactions require further deep-dive research to guarantee 100% ecosystem representation:
1.  **Native AOT (Ahead-of-Time) Trimming Annotations**: How attributes like `[RequiresUnreferencedCode]` and `[DynamicallyAccessedMembers]` interact with the code graph to designate paths that might be stripped at link time.
2.  **COM Interop and Dynamic Dispatch**: The interaction between the `dynamic` keyword (C# 4.0 [cite: 46]) and explicit COM type libraries imported via MSBuild `<COMReference>`, modifying dispatch chains outside of normal Roslyn boundaries.
3.  **IL Weaving vs. Source Generation**: While Roslyn Source Generators append C# files [cite: 1], older ecosystem tools (like Fody/PostSharp) rewrite Intermediate Language (IL) directly post-compilation. Ensuring a graph generator defines a clear operational boundary distinguishing C# source-semantics from IL-semantics remains an open design challenge.

**Sources:**
1. [todaysoftmag.com](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQGYW2GDDvEWuTRsCwU3Hcafwnle61a1bYxxtt09ND7UCC36WTtMwJ_6f1r9Rbq29Bnk-tdkokKQLS32iulWZTqE0M8Uff0LXdG_ITrRu2uNO2jkLGdHptFa5gKb5y8dqLUCpgMKftaP_5-lothY18UyfnIqrYaKbo8=)
2. [medium.com](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQHuih59uXFTvEagUi6zjVux2t-IYj-7h8Dn99WUdwBLeyampDhPxCFqbDJI5PuMWuQFLda6ABU8J_6tIeTwIAxx6lpAdGJRplgZN1OYfOZSaxoNFxqjR2AaN_Su0r9SLSC1vF5VvJvVWGuFAoTvkY6d3SKGBCaWrMGHFIROT9H1sJPzAhOFM_QU83Vv6R7_2O5NRigP-_FNr5fElmiAAQ==)
3. [youtube.com](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQFaC9a5s5wt3Mc9DD4b-4BNsHG7YwPUfvJBsHuDLBr-olcLfZ517tABOpWep9eGA8YQGMGnwC4-K0GBXCa92W9OVwSYNRJil6tGLU7dszgIAW6--zkekvn5N02YkfAQPiHn)
4. [sieraglobal.com](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQFzyL5IhNoDnVvfxtv4iqgqzu5mLg4c09f2LIzRJVP9cILk35uPF6SimnnmWeajsdiZ__JX3bKmnNakOqfq2qO0EUhXa-GQ3eW65J9g69-brz2QBYjDwIK51zY0refufWVagd6_-koYgxVHVn8PAykhwg==)
5. [thinktecture.com](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQH48dB-cjC0Xv3JMat0RZyhZYswvESQVVDgfrTl760usEVCFvX5uhZq1bTcfdwru62wf901ydaF5nW6wytIR_88S5JxC-yOabvMtwkBdyt-amDoH-Wt8NqI7-a7d2Z0EueAExEGGprSfmzqthCBg_gX-7U2G2riUne_mb3Jew==)
6. [microsoft.com](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQHDRFBnHboXcYlPagXXHnWoiVkZPpxtfR3VXV9XDq4sDknpzeccqd1W1Qq1xgW3uBwbDPVzwz62fwa7UEwksPB3wAEY3LPEo6PDgh795FLT0rEfAvIwBctRjBp4fM3G5BoLkg2aHxSmDjhgVVmZ7h4dezEoSRiRU1mI)
7. [microsoft.com](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQFaOO4KQarrRwy7doFCGu4HGbZqdTfrfIVSlPGqw0-QsEjRVFnskWHl3-QruzCzgSz75eeoVy0JbvbWbI9rqKqKhQ_PQdXzhYDeUZSen3Twm1fdY6OrS3MoDK42BbjQhc1pS6CC_y5VJyHjHnTGYtc_vIehLKIJ6Mvh61E=)
8. [medium.com](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQEjXc8gySHwRXd6RqaHrZVyqj0uzF8U-z9B867u7lV8AWXWd1qHi8YnjyLuEf4Sk8bm4m5aJmazmRF2xM-3O04DbriUBQhyDBwmfS2k9Vai0XuK_xzpG_LnOgrOa4fkscjdO-U856gJ0rV0VXvUGdN4hIhV46b6MJw-tiBj)
9. [medium.com](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQE9efEHDZyu0ku5_EOXzYl8KjC-yM62r84LppzKPjsjR455rk2GYEjr7dfFL5L1gXULat1JDeJr4GO9MsprFgoZI8TPMpCa6YxlOKCZ8i5xmxqjFAMQzhZT8qG3dgzlsaIvCwz_k2snwaHwqi5BY_thPfkTdaS9lUpPu8n844k=)
10. [jetbrains.com](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQFIKlIfrELm2vmTHJkPoy-CEWGwAF6Svx3WFv-xlGK2uXn41xs-ShjxnRvaX9D7L6i3kn2Bd7EYTKjmv3okEJrg5J9rKqqXv7ft8ELC4Hi1I5smlKNfEVKbTdS0hMLgRc_gXhE0daWkbEVNqM8CY5YB8v9BhxqHW9ZLc9L39tG-Z_PCKEWj2zaasi2InQMhpp3D6Zs0tglE)
11. [tutorialsteacher.com](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQE58WGCPuUagIk_NUnGBs39Ci9F_oDs1_IgXJYEpvQzpvJF5srT7E0gfOBldu3CGqD-8CD5D-1fnvqrPFTdRw6srQYc1fp4jRCL2Dhc5B8Iml1lscXrdrMDMKJALY8P9y4985KgZj-CS7CcsY7W6m3dSH0=)
12. [c-sharptutorial.com](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQEDAoDCEojCufPu8ajjIZxhx305GQw2H6Bg0w5euWDUyEwIQ9xajX0palqvI5SGo2eCOa5-jytpJUYVbFPqJ3KQeRZUDbNCou-sHxLL8Q2ScP1U3p44lhCCLVsXI_9OQj_-As6WpGi7WMzgF_1AtGNH5soYsj2sIJjnmaf5mXI=)
13. [elmah.io](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQGm_KAx2hU7E-7NMGjFiXd_kt0Ro0dxhdf1CU7YQLyh5bXRn3geVfEtP8q6dO_DBDaR3vnr7JucMcDL5J7nSJSJ-O9d0rLjCslSR4ggkiFd6b44zhWND-iXd0GzZre37UanC5pCbK9tft1_Mj4OHmFBs8Kp6hYT)
14. [medium.com](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQGjL4Gp72DND3_iih8Whp2ibHdJ4n3Ja8zT9u6BoUNFz7BND50_6kJf0X7FBm93u8Fdfei--R2Fjjg3phNXJhIgJtxNP_Px3WJoU0MEqqbcx2ywY-ZUpBuhl7dljcxqXwNxqecpFo0UF30R3d3Ylc3Di3-iTTxqZNNukyAakZZHmhJRH-UYlN_0x9QN49cVmQAgeBouWg==)
15. [theburningmonk.com](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQHvfUtKg0mnp2j4QLwON8ipIWcI6HLuaXYysNfwsnlzQ0zh4lYYOlrLzzHH8_U_vEGT9zJttJ9vAZKWP75tUWSWA6rfBB-KDZyzJGDbRNF_mK1maa1SIJmvljGRXaSQVErxpDBoQtdXt1ozIYGD6h673FiAa-5dbMrRcgGN-qiq1w==)
16. [csharp-evolution.com](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQFxVPsFy1vgncssQTVjtwypFs2vFYpNc4bXxwQJl8n0v8fzM20dIYtC2cAjh26S3H7gfK5h61h4wL3ab2zy4u7ORLM590O-_y1lDYfYim0CydVM9_2cjywPlCWL3KH_V6BBgTw=)
17. [csharpplayersguide.com](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQEfVKwObSPmLPsylFKgLsa_Nb76UOqX8gWs-2AmYzYGMF8OQQ1opdndar2i_a4HmRX0ZqveGwyl3BSonhOPTvMg5cf69nUrQvQn6t58k7XkXnY7BIUPeQxK2B5xGU96j1HTiApdFzHolfivewGzcY-C7w==)
18. [github.com](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQHbXSL5KglmTbffrHY1-Po8Vuowu7dPiKHH9DsV_mBDdx9K17lYN0-vnQhkpRnppCRsqMiC6ETfkoNAJ-3ggXo9RB_f4G6c-N9RmBuBjRVuKfIXe9Bvm0qXeNfkIr8zMXkxxaUC4-gh13fWwMwlTmnWMYSgg8zL89q2OqpxBf9WrGbQcQQLMS7ksArxTkXN3fPos_Z8xhQZe3ixdbJPCpGv)
19. [microsoft.com](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQEoqkqBqhXQ-MmwJFE8E6bnSxSMTWDvNL1eu8StFTM1n1qepzVzaBI3f7BEuHARK71_zM2Z_OX6WbpscDZII9MrXumJ4ifDAMDmlxJPAwwZfwVd8a0K6c_Ffyu5Ms4ro5cqUEuHp-GXi5DhLuMoPUOXTUN2D-d9udinBzc0TSg3GFFiwRdKsiW0j7cFaiju)
20. [code-maze.com](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQFwYXquAl3R6v8wk0mLM5zy1m1EFaKFqOlbpCzet3uljnPmxQYNex2kj4IP-XVDlH103BYEafOkZEFhHBROX1jtgzKyVNYM96asXFZ4kb32Kgrb-ZiAUcyjbhvhc4TqJdfH27g=)
21. [microsoft.com](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQG_1eEidDy_c8Ga_bLg9-_ZlUbr_vDmDV5fPN6u8gU4QuBqNykPn-cJQsY38-Zq0tvfNvYQikI0yO1jNMY7E9eBuIAPIgjDofgINwvoKavlNXQZPdvuGY9Gf5VKii3HYxxNj5E4eBQJtmyTa9ldyl6ToA==)
22. [github.com](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQFo1jWaNWzTaq2An65g3OeQYq23qu3PcFp8oO62u1LpCk-Z02xkfg2mAVuUiZ2gCjo_nrqq_Q-gILG0TVCigYbHBf4TGVVMyIlfLaTGGqCCOTNFUzBlyvUVcOajTqfS7VgEBoKPex4mV8Pn5QXah1Iu2ebR7k6ADI0=)
23. [microsoft.com](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQFWnQ_sEYZqb_Uy6hpL879MIAghNlYmOq_TXJ8jpqV6f9fJps-zbnr19LkSy3aBem-OXs7E8obyS78RKXOYz0zspDZ-wTXjZ_WT_dffMVleNfrMvaTjVWmDy9AvsYGSNyKxu7Oige7QofsavaASIuNxERsj4STlh9Bso8eRMuc5lPjZ2KzteZ2jmh7AbRI_uJVkECJCK_YJW9xObM72)
24. [microsoft.com](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQGKPuiCh4GoDJxomqbciNHUvBNhV-AzkpuYPUEa8SLu6my471yPvE3O3turi9ZWRDUDDOadTW4_KBjxtMpsQDndLr2UO2eYmBAiVLuAfkJbNB6e0lSnBiv2W0SygCXVE6JXQgks4wRLHL_09EWl5GZ8tXNZjm6F6StxRL6NixFPlTYhUifmuKAgCrS5AGXANMD3ZKCgUQ==)
25. [github.com](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQGgYkj5xJCA8yiJE4f1se-sHQFJSo-wC9jRCL_RsE12cswVROSo97bEt2gmMYi4pKv78qSsbvLzGsuZ4oiUMHZkz3QbdOtcYznry7onhJg6RxnQoz8DThS69CXh9ZO0ULyJzrVgsvMRtz5Ho4-xS96E_mrnRH5mCZ2c0dnAV9RsjXk08EtV4IeB04fKAvkmFzaSBpz8UEU2)
26. [medium.com](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQGvxbTHSFDSsG_9zNbxuUU7QgtcYSttvZfpcubttm5iva88NmWy0J9GJ2yCkopzzx4KG2fkoeNCZTM5v7HOcAt3L9jDgCRy9STaSKpMK_0s45_j2ROI55jHljB5msmZWLyd8ddzIteje70gCgGaMLu7WY5oXU1zDJd4z2ac2ZjM82OUdzDj4u_Y9Hc=)
27. [stackoverflow.com](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQF4AkbMyrkXmrEbCySii3oLfuqpAU_8PilKpC0YTlgAHDz5TV8RgLj40POyvYYvw-agHRcu1zhizUPSaXdrzK3iP-fOuBWIMARTywmHDdU0S0Tt_ETwL4ic-86hylTAi-w2uELbsnzU0v5ocbWOEwYugM4XQ70Keya47Hk4wQxTpFoEjDWHd08n_bhTwyTd-M8xDSs=)
28. [github.com](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQGhoocDveMgCEQeVFpyPVKnyct21b8qjd_cUegy9mjMzLcQoXdYZM_kszCS6--amqq_nfNCXI62cQgZZlJLEFEajGu1ow9jLn89mRoVQON4mWUA53KzGP2tezbM1m5nkWv2gVL4oynsMgEiRl41yp_O2CNW4srN_rFKQFUfTeOmA65oSkpuo7bnPPbTlu2kYhvCzrfC7YNdLdlm)
29. [github.com](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQHQAFAyTl2lkkkE5ocjzTehPQmjAoLTSbdXFrHg69-xf5LMMeo-clxpPBEKDeyJWQxZhGsstaq8BjoPqQwXxqG9d08FakuXvSR_SJwAxQiAT8kx8SJbL6d5Yu2igysFPkgtvjHi0nPvqGw2bIwwP--HjpyxBzibCMZVEHMksTaU9dLA_oddxX2nRx3RpFuhhpe7nyWGC-gM-zgv)
30. [microsoft.com](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQEVl_ngVfnWEE4RqJKqOc32_jCU31x81xPJ23vnfU9Wy1bQ-KkkjkzTpEI7bqA4cuPKdD53IV15xnTEUZ97aMAhN2-CKod_o-FYmHcHru0emwxb0AgKgpJekSi7xHolvnj7ul6-Pcra6wsNI4eQe3utw6qW_i9nLINog3qPIbHH75h9IOlzl2GQ_N5Uew==)
31. [microsoft.com](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQH1CtuT90Eia8zw4NIaUnMlMlinReNsOMtdX0vJkO8SU5eRy0bj5nNTzKIjk6bDtx9VfT4gEr3IjL_p7JQv9smPSKB1uVG03BuJBrr1E_78mQ0w_dbQTPxMaaDj8q4sD1IQ1pceZvX5OhazCe3r5KXXUTXIVlewCBEpc2_AhEUvugK4kK6msPOzWYQ=)
32. [medium.com](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQGuVqfv37RO4dCwBVrBoN4kgXF0zcyLPgwgd6rR7vOijwT78KUIxfrTqBVMK_GbhTKv8NJQxeQ1Ztt3b1BUz2NoQnTyRqbFAxdJD-5H8rzLWG1sG-sXnsjD6mGyJEDj8mRgglTjHWwga0ATIpEUNUhCMFS2F67Vsg==)
33. [elmah.io](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQGHqGvx0SDPArbYF0Gcu3LyWC1NJ2i-4seMoIbSOir9IBAdbFHhQY6sXQW8qlWIFmzaeL6L6u4COr-rlPCmuxkXWQJbiHAzssR5JgAeYvefG1p4pDXvdqPPQsm8OzWmxHu9nPtHfNw0IWdNVTYOjrrcIkaWxFjLfM-zyMnp8r-dDQ==)
34. [redhat.com](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQE_y45VOo5E6MRcgTWwLJeoAdV2mdrpflHmuDrjSGrbuTqjIuPqp87irrpnfzhjA7iIgxKb2J_zArFBr6gpsHFe7c9bf3Be5mkbAHCjHf1mY7Sps40xMPzk2giO2X1bhy7bJBJtxsYuEbRWYP4GBzIFVCxCu9TL7wkcNf6lteg=)
35. [infoworld.com](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQGqIn4dVB7_krgrgvxGKIKbmtDVf82RH9HI7vU6xjcxh2a1TV7ojZywHPVFYauZrhkrzHGeneJHwiPpuazvhiH8V0h3Ufwu-ze33RmoB793l-7us5drxIENjqlbYi359NiK7GYswLjyE3peuvoc-Nx80XdpCybNfEy9EdxYQ_KkEQZuk8hbOtoo)
36. [medium.com](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQEKKGm9irLW0YibhX29lHHmACOHLU_RdvC0jkEbiCMETfhPgi-CIQgDPmwK0bG6pCfspDfDSirwXOi-jchNEVGqvhzREhScWuY-L5fQI2boAubSN6-sMjI6EHsYLP6AkpZdhnq3U9iO_KSn3wb3mQ8hpDQPFRYPsNDiwf6OWrYS)
37. [infoworld.com](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQEz4SPHzKDUdByjLCYy0H-YNciI4DpdXz3iAy1vArECRKqA8mE4VbR2MWaMtvHnIktxpZTm8AAQQOCKhhMjiWZ8YVVUy486XGGnyvrZX9CLUpt45FzHsUz6J1W6WsB0V96WoAuouj3IhUyMkY97iboUogi80WeTx4SX2eDByedbNCBHk6j2l68B)
38. [laurentkempe.com](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQG505sIeRw4u1GGuE_YSILRzBpa4UPDnpdaef4JdLGXHgQz4xz0O5F99_HVH-k_OBc3iEoJ99qHtOBfuGdufhlemTDfFQsT3uLoIxEe8NujUy5VmElL-JNQMCqeecyuxTbYb2R-0F7bPOmFj9CnCWDIPQ==)
39. [c-sharpcorner.com](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQEVqo4lileN1RNE97IHgghUeXLuHUS_jC3j_oYTKwC-B3qj2z3g1IaomTxSjhIYj8N4vU1lLkf_e63z1jaB6PDBguug6DtDXlmX1DwaJjsiTGjfw4gx4rIkpvdbascThiX1LWJoLHobqCue5OIW-E4zo0WJaxru1w==)
40. [dev.to](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQEwjbas72fVLxWDN-69TO9olxc3-KeH_bNWbYyjnKUP_I-QN8kfQF_lgRi7Pb_oC34BM0YP0aTGSEWY5qwFKI-x6j7kAD00hcWSdYYeMu8a_9FmARAb9lJjNFDJTr5MO4geWdDGmCi0FK2WXyPSbbfFSFmgNJfX9he_B0PCaxBFdUxdWWqOb8-h7222)
41. [visualstudiomagazine.com](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQFmy0nL_1l1OWWc5Vvt0GnbOWTMXWsQxdVJvxstvv_CoZOPXk0xmNh32k2XpZ6pbemLYxX3swFUFrJPRVjlahkmnmnkHhLWv16u7m4vUojGjOoB6kz1J_AJoy0n9TJmeeM6NHivur07f4IEXk8fnzZC54ipccXsUHfZnovSRAxhvEHN_LAvfY4INg==)
42. [medium.com](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQEiCCQCh77NqO3HW-cHx4kwnvu988_oKmFYxAy_-6oMnhIzvptH6R812kzQkoD9f3L35LUj6_azLlKMiIdpMRbNlWO6YVO8t3598f036Bh764kR8KSzn7-rlRuSDOUroegf_rjBKoHAeYXhs1HP1x23ky8dAAsz_Z3Wo7DSCgELNUJLhMgS2wH_ScH5j5pYDB9TsQhYcFkf9ZycMfEDz3ZNnfxWV8qQniBz)
43. [databeautifier.com](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQGZouNnVOvqW3P6A1RQVVuFtXEMtRKwyBnBRdSCE9-AD0enpQkIbCcWAQucAUf40PIrT8j_83ZvA8ZqoBv0VeOr5oopLTJkyoI88LpYSXw2S-nG5WqvGF3nHG_1xs6J0DX4mCljpiZC0-JeCgn6YoqXxRdv5pk-zb3NqzghHzI0wfNA3WNhWrD1WY91fwrMvm_d9rNbzwiTg0YrSXo=)
44. [yaseerarafat.com](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQEL1BNKYJ01wFM9lnJzdxprNr9RykpsnTvMPyftnpo0DVMf4sz-r68hZLYFfuOyWe1Efhh4z0elqGW_VB24kyZxGlxEifTVdvkL5iIIVNbf14C3jmNZZ_iqp0WOwDGiezTIAcBNuTXe25GQha-5LjpVCZwDRYhHGowJkym94q3b9kh-4VlDn_5uBWHJKZsX4F1dTCXTyj8Ja3vQTIiQNE35ZBBKtVuWqA==)
45. [microsoft.com](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQHO3RyuMEVuTuAfL6ia-0AQIVBgQx-bAkVAfFViDtQPm9aH_9EcWOxMP_-gUPK1P-IzxmRI5WNmB2hSGkc6mnJxcuNlErZDXOPAReqX2iFmnpJlesGzUvcPQocAWHR5D9kA8CgmDpmVtQJM-mtFYUQelcr3x2qYtcFaX4HBPFSKq9kYfNJuEjqD)
46. [scribd.com](https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQFFg5GqPPmlmYEFHndX13JsF-8OK3Il2I_PrOoAJpc2V0DvzLb8yeJp4oAkRiQgPZczDUo_1vy9YpmuPrbnzhLHj3Zb0r-cH2TanvCKwjUfcyddWj5NVeqD5D0PG384tO7-U7yDYOkBD1UfrA==)

Citations:
  [1] https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQGYW2GDDvEWuTRsCwU3Hcafwnle61a1bYxxtt09ND7UCC36WTtMwJ_6f1r9Rbq29Bnk-tdkokKQLS32iulWZTqE0M8Uff0LXdG_ITrRu2uNO2jkLGdHptFa5gKb5y8dqLUCpgMKftaP_5-lothY18UyfnIqrYaKbo8=
  [2] https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQHuih59uXFTvEagUi6zjVux2t-IYj-7h8Dn99WUdwBLeyampDhPxCFqbDJI5PuMWuQFLda6ABU8J_6tIeTwIAxx6lpAdGJRplgZN1OYfOZSaxoNFxqjR2AaN_Su0r9SLSC1vF5VvJvVWGuFAoTvkY6d3SKGBCaWrMGHFIROT9H1sJPzAhOFM_QU83Vv6R7_2O5NRigP-_FNr5fElmiAAQ==
  [3] https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQFaC9a5s5wt3Mc9DD4b-4BNsHG7YwPUfvJBsHuDLBr-olcLfZ517tABOpWep9eGA8YQGMGnwC4-K0GBXCa92W9OVwSYNRJil6tGLU7dszgIAW6--zkekvn5N02YkfAQPiHn
  [4] https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQFzyL5IhNoDnVvfxtv4iqgqzu5mLg4c09f2LIzRJVP9cILk35uPF6SimnnmWeajsdiZ__JX3bKmnNakOqfq2qO0EUhXa-GQ3eW65J9g69-brz2QBYjDwIK51zY0refufWVagd6_-koYgxVHVn8PAykhwg==
  [5] https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQH48dB-cjC0Xv3JMat0RZyhZYswvESQVVDgfrTl760usEVCFvX5uhZq1bTcfdwru62wf901ydaF5nW6wytIR_88S5JxC-yOabvMtwkBdyt-amDoH-Wt8NqI7-a7d2Z0EueAExEGGprSfmzqthCBg_gX-7U2G2riUne_mb3Jew==
  [6] https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQHDRFBnHboXcYlPagXXHnWoiVkZPpxtfR3VXV9XDq4sDknpzeccqd1W1Qq1xgW3uBwbDPVzwz62fwa7UEwksPB3wAEY3LPEo6PDgh795FLT0rEfAvIwBctRjBp4fM3G5BoLkg2aHxSmDjhgVVmZ7h4dezEoSRiRU1mI
  [7] https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQFaOO4KQarrRwy7doFCGu4HGbZqdTfrfIVSlPGqw0-QsEjRVFnskWHl3-QruzCzgSz75eeoVy0JbvbWbI9rqKqKhQ_PQdXzhYDeUZSen3Twm1fdY6OrS3MoDK42BbjQhc1pS6CC_y5VJyHjHnTGYtc_vIehLKIJ6Mvh61E=
  [8] https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQEjXc8gySHwRXd6RqaHrZVyqj0uzF8U-z9B867u7lV8AWXWd1qHi8YnjyLuEf4Sk8bm4m5aJmazmRF2xM-3O04DbriUBQhyDBwmfS2k9Vai0XuK_xzpG_LnOgrOa4fkscjdO-U856gJ0rV0VXvUGdN4hIhV46b6MJw-tiBj
  [9] https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQE9efEHDZyu0ku5_EOXzYl8KjC-yM62r84LppzKPjsjR455rk2GYEjr7dfFL5L1gXULat1JDeJr4GO9MsprFgoZI8TPMpCa6YxlOKCZ8i5xmxqjFAMQzhZT8qG3dgzlsaIvCwz_k2snwaHwqi5BY_thPfkTdaS9lUpPu8n844k=
  [10] https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQFIKlIfrELm2vmTHJkPoy-CEWGwAF6Svx3WFv-xlGK2uXn41xs-ShjxnRvaX9D7L6i3kn2Bd7EYTKjmv3okEJrg5J9rKqqXv7ft8ELC4Hi1I5smlKNfEVKbTdS0hMLgRc_gXhE0daWkbEVNqM8CY5YB8v9BhxqHW9ZLc9L39tG-Z_PCKEWj2zaasi2InQMhpp3D6Zs0tglE
  [11] https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQE58WGCPuUagIk_NUnGBs39Ci9F_oDs1_IgXJYEpvQzpvJF5srT7E0gfOBldu3CGqD-8CD5D-1fnvqrPFTdRw6srQYc1fp4jRCL2Dhc5B8Iml1lscXrdrMDMKJALY8P9y4985KgZj-CS7CcsY7W6m3dSH0=
  [12] https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQEDAoDCEojCufPu8ajjIZxhx305GQw2H6Bg0w5euWDUyEwIQ9xajX0palqvI5SGo2eCOa5-jytpJUYVbFPqJ3KQeRZUDbNCou-sHxLL8Q2ScP1U3p44lhCCLVsXI_9OQj_-As6WpGi7WMzgF_1AtGNH5soYsj2sIJjnmaf5mXI=
  [13] https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQGm_KAx2hU7E-7NMGjFiXd_kt0Ro0dxhdf1CU7YQLyh5bXRn3geVfEtP8q6dO_DBDaR3vnr7JucMcDL5J7nSJSJ-O9d0rLjCslSR4ggkiFd6b44zhWND-iXd0GzZre37UanC5pCbK9tft1_Mj4OHmFBs8Kp6hYT
  [14] https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQGjL4Gp72DND3_iih8Whp2ibHdJ4n3Ja8zT9u6BoUNFz7BND50_6kJf0X7FBm93u8Fdfei--R2Fjjg3phNXJhIgJtxNP_Px3WJoU0MEqqbcx2ywY-ZUpBuhl7dljcxqXwNxqecpFo0UF30R3d3Ylc3Di3-iTTxqZNNukyAakZZHmhJRH-UYlN_0x9QN49cVmQAgeBouWg==
  [15] https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQHvfUtKg0mnp2j4QLwON8ipIWcI6HLuaXYysNfwsnlzQ0zh4lYYOlrLzzHH8_U_vEGT9zJttJ9vAZKWP75tUWSWA6rfBB-KDZyzJGDbRNF_mK1maa1SIJmvljGRXaSQVErxpDBoQtdXt1ozIYGD6h673FiAa-5dbMrRcgGN-qiq1w==
  [16] https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQFxVPsFy1vgncssQTVjtwypFs2vFYpNc4bXxwQJl8n0v8fzM20dIYtC2cAjh26S3H7gfK5h61h4wL3ab2zy4u7ORLM590O-_y1lDYfYim0CydVM9_2cjywPlCWL3KH_V6BBgTw=
  [17] https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQEfVKwObSPmLPsylFKgLsa_Nb76UOqX8gWs-2AmYzYGMF8OQQ1opdndar2i_a4HmRX0ZqveGwyl3BSonhOPTvMg5cf69nUrQvQn6t58k7XkXnY7BIUPeQxK2B5xGU96j1HTiApdFzHolfivewGzcY-C7w==
  [18] https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQHbXSL5KglmTbffrHY1-Po8Vuowu7dPiKHH9DsV_mBDdx9K17lYN0-vnQhkpRnppCRsqMiC6ETfkoNAJ-3ggXo9RB_f4G6c-N9RmBuBjRVuKfIXe9Bvm0qXeNfkIr8zMXkxxaUC4-gh13fWwMwlTmnWMYSgg8zL89q2OqpxBf9WrGbQcQQLMS7ksArxTkXN3fPos_Z8xhQZe3ixdbJPCpGv
  [19] https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQEoqkqBqhXQ-MmwJFE8E6bnSxSMTWDvNL1eu8StFTM1n1qepzVzaBI3f7BEuHARK71_zM2Z_OX6WbpscDZII9MrXumJ4ifDAMDmlxJPAwwZfwVd8a0K6c_Ffyu5Ms4ro5cqUEuHp-GXi5DhLuMoPUOXTUN2D-d9udinBzc0TSg3GFFiwRdKsiW0j7cFaiju
  [20] https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQFwYXquAl3R6v8wk0mLM5zy1m1EFaKFqOlbpCzet3uljnPmxQYNex2kj4IP-XVDlH103BYEafOkZEFhHBROX1jtgzKyVNYM96asXFZ4kb32Kgrb-ZiAUcyjbhvhc4TqJdfH27g=
  [21] https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQG_1eEidDy_c8Ga_bLg9-_ZlUbr_vDmDV5fPN6u8gU4QuBqNykPn-cJQsY38-Zq0tvfNvYQikI0yO1jNMY7E9eBuIAPIgjDofgINwvoKavlNXQZPdvuGY9Gf5VKii3HYxxNj5E4eBQJtmyTa9ldyl6ToA==
  [22] https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQFo1jWaNWzTaq2An65g3OeQYq23qu3PcFp8oO62u1LpCk-Z02xkfg2mAVuUiZ2gCjo_nrqq_Q-gILG0TVCigYbHBf4TGVVMyIlfLaTGGqCCOTNFUzBlyvUVcOajTqfS7VgEBoKPex4mV8Pn5QXah1Iu2ebR7k6ADI0=
  [23] https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQFWnQ_sEYZqb_Uy6hpL879MIAghNlYmOq_TXJ8jpqV6f9fJps-zbnr19LkSy3aBem-OXs7E8obyS78RKXOYz0zspDZ-wTXjZ_WT_dffMVleNfrMvaTjVWmDy9AvsYGSNyKxu7Oige7QofsavaASIuNxERsj4STlh9Bso8eRMuc5lPjZ2KzteZ2jmh7AbRI_uJVkECJCK_YJW9xObM72
  [24] https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQGKPuiCh4GoDJxomqbciNHUvBNhV-AzkpuYPUEa8SLu6my471yPvE3O3turi9ZWRDUDDOadTW4_KBjxtMpsQDndLr2UO2eYmBAiVLuAfkJbNB6e0lSnBiv2W0SygCXVE6JXQgks4wRLHL_09EWl5GZ8tXNZjm6F6StxRL6NixFPlTYhUifmuKAgCrS5AGXANMD3ZKCgUQ==
  [25] https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQGgYkj5xJCA8yiJE4f1se-sHQFJSo-wC9jRCL_RsE12cswVROSo97bEt2gmMYi4pKv78qSsbvLzGsuZ4oiUMHZkz3QbdOtcYznry7onhJg6RxnQoz8DThS69CXh9ZO0ULyJzrVgsvMRtz5Ho4-xS96E_mrnRH5mCZ2c0dnAV9RsjXk08EtV4IeB04fKAvkmFzaSBpz8UEU2
  [26] https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQGvxbTHSFDSsG_9zNbxuUU7QgtcYSttvZfpcubttm5iva88NmWy0J9GJ2yCkopzzx4KG2fkoeNCZTM5v7HOcAt3L9jDgCRy9STaSKpMK_0s45_j2ROI55jHljB5msmZWLyd8ddzIteje70gCgGaMLu7WY5oXU1zDJd4z2ac2ZjM82OUdzDj4u_Y9Hc=
  [27] https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQF4AkbMyrkXmrEbCySii3oLfuqpAU_8PilKpC0YTlgAHDz5TV8RgLj40POyvYYvw-agHRcu1zhizUPSaXdrzK3iP-fOuBWIMARTywmHDdU0S0Tt_ETwL4ic-86hylTAi-w2uELbsnzU0v5ocbWOEwYugM4XQ70Keya47Hk4wQxTpFoEjDWHd08n_bhTwyTd-M8xDSs=
  [28] https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQGhoocDveMgCEQeVFpyPVKnyct21b8qjd_cUegy9mjMzLcQoXdYZM_kszCS6--amqq_nfNCXI62cQgZZlJLEFEajGu1ow9jLn89mRoVQON4mWUA53KzGP2tezbM1m5nkWv2gVL4oynsMgEiRl41yp_O2CNW4srN_rFKQFUfTeOmA65oSkpuo7bnPPbTlu2kYhvCzrfC7YNdLdlm
  [29] https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQHQAFAyTl2lkkkE5ocjzTehPQmjAoLTSbdXFrHg69-xf5LMMeo-clxpPBEKDeyJWQxZhGsstaq8BjoPqQwXxqG9d08FakuXvSR_SJwAxQiAT8kx8SJbL6d5Yu2igysFPkgtvjHi0nPvqGw2bIwwP--HjpyxBzibCMZVEHMksTaU9dLA_oddxX2nRx3RpFuhhpe7nyWGC-gM-zgv
  [30] https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQEVl_ngVfnWEE4RqJKqOc32_jCU31x81xPJ23vnfU9Wy1bQ-KkkjkzTpEI7bqA4cuPKdD53IV15xnTEUZ97aMAhN2-CKod_o-FYmHcHru0emwxb0AgKgpJekSi7xHolvnj7ul6-Pcra6wsNI4eQe3utw6qW_i9nLINog3qPIbHH75h9IOlzl2GQ_N5Uew==
  [31] https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQH1CtuT90Eia8zw4NIaUnMlMlinReNsOMtdX0vJkO8SU5eRy0bj5nNTzKIjk6bDtx9VfT4gEr3IjL_p7JQv9smPSKB1uVG03BuJBrr1E_78mQ0w_dbQTPxMaaDj8q4sD1IQ1pceZvX5OhazCe3r5KXXUTXIVlewCBEpc2_AhEUvugK4kK6msPOzWYQ=
  [32] https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQGuVqfv37RO4dCwBVrBoN4kgXF0zcyLPgwgd6rR7vOijwT78KUIxfrTqBVMK_GbhTKv8NJQxeQ1Ztt3b1BUz2NoQnTyRqbFAxdJD-5H8rzLWG1sG-sXnsjD6mGyJEDj8mRgglTjHWwga0ATIpEUNUhCMFS2F67Vsg==
  [33] https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQGHqGvx0SDPArbYF0Gcu3LyWC1NJ2i-4seMoIbSOir9IBAdbFHhQY6sXQW8qlWIFmzaeL6L6u4COr-rlPCmuxkXWQJbiHAzssR5JgAeYvefG1p4pDXvdqPPQsm8OzWmxHu9nPtHfNw0IWdNVTYOjrrcIkaWxFjLfM-zyMnp8r-dDQ==
  [34] https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQE_y45VOo5E6MRcgTWwLJeoAdV2mdrpflHmuDrjSGrbuTqjIuPqp87irrpnfzhjA7iIgxKb2J_zArFBr6gpsHFe7c9bf3Be5mkbAHCjHf1mY7Sps40xMPzk2giO2X1bhy7bJBJtxsYuEbRWYP4GBzIFVCxCu9TL7wkcNf6lteg=
  [35] https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQGqIn4dVB7_krgrgvxGKIKbmtDVf82RH9HI7vU6xjcxh2a1TV7ojZywHPVFYauZrhkrzHGeneJHwiPpuazvhiH8V0h3Ufwu-ze33RmoB793l-7us5drxIENjqlbYi359NiK7GYswLjyE3peuvoc-Nx80XdpCybNfEy9EdxYQ_KkEQZuk8hbOtoo
  [36] https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQEKKGm9irLW0YibhX29lHHmACOHLU_RdvC0jkEbiCMETfhPgi-CIQgDPmwK0bG6pCfspDfDSirwXOi-jchNEVGqvhzREhScWuY-L5fQI2boAubSN6-sMjI6EHsYLP6AkpZdhnq3U9iO_KSn3wb3mQ8hpDQPFRYPsNDiwf6OWrYS
  [37] https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQEz4SPHzKDUdByjLCYy0H-YNciI4DpdXz3iAy1vArECRKqA8mE4VbR2MWaMtvHnIktxpZTm8AAQQOCKhhMjiWZ8YVVUy486XGGnyvrZX9CLUpt45FzHsUz6J1W6WsB0V96WoAuouj3IhUyMkY97iboUogi80WeTx4SX2eDByedbNCBHk6j2l68B
  [38] https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQG505sIeRw4u1GGuE_YSILRzBpa4UPDnpdaef4JdLGXHgQz4xz0O5F99_HVH-k_OBc3iEoJ99qHtOBfuGdufhlemTDfFQsT3uLoIxEe8NujUy5VmElL-JNQMCqeecyuxTbYb2R-0F7bPOmFj9CnCWDIPQ==
  [39] https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQEVqo4lileN1RNE97IHgghUeXLuHUS_jC3j_oYTKwC-B3qj2z3g1IaomTxSjhIYj8N4vU1lLkf_e63z1jaB6PDBguug6DtDXlmX1DwaJjsiTGjfw4gx4rIkpvdbascThiX1LWJoLHobqCue5OIW-E4zo0WJaxru1w==
  [40] https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQEwjbas72fVLxWDN-69TO9olxc3-KeH_bNWbYyjnKUP_I-QN8kfQF_lgRi7Pb_oC34BM0YP0aTGSEWY5qwFKI-x6j7kAD00hcWSdYYeMu8a_9FmARAb9lJjNFDJTr5MO4geWdDGmCi0FK2WXyPSbbfFSFmgNJfX9he_B0PCaxBFdUxdWWqOb8-h7222
  [41] https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQFmy0nL_1l1OWWc5Vvt0GnbOWTMXWsQxdVJvxstvv_CoZOPXk0xmNh32k2XpZ6pbemLYxX3swFUFrJPRVjlahkmnmnkHhLWv16u7m4vUojGjOoB6kz1J_AJoy0n9TJmeeM6NHivur07f4IEXk8fnzZC54ipccXsUHfZnovSRAxhvEHN_LAvfY4INg==
  [42] https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQEiCCQCh77NqO3HW-cHx4kwnvu988_oKmFYxAy_-6oMnhIzvptH6R812kzQkoD9f3L35LUj6_azLlKMiIdpMRbNlWO6YVO8t3598f036Bh764kR8KSzn7-rlRuSDOUroegf_rjBKoHAeYXhs1HP1x23ky8dAAsz_Z3Wo7DSCgELNUJLhMgS2wH_ScH5j5pYDB9TsQhYcFkf9ZycMfEDz3ZNnfxWV8qQniBz
  [43] https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQGZouNnVOvqW3P6A1RQVVuFtXEMtRKwyBnBRdSCE9-AD0enpQkIbCcWAQucAUf40PIrT8j_83ZvA8ZqoBv0VeOr5oopLTJkyoI88LpYSXw2S-nG5WqvGF3nHG_1xs6J0DX4mCljpiZC0-JeCgn6YoqXxRdv5pk-zb3NqzghHzI0wfNA3WNhWrD1WY91fwrMvm_d9rNbzwiTg0YrSXo=
  [44] https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQEL1BNKYJ01wFM9lnJzdxprNr9RykpsnTvMPyftnpo0DVMf4sz-r68hZLYFfuOyWe1Efhh4z0elqGW_VB24kyZxGlxEifTVdvkL5iIIVNbf14C3jmNZZ_iqp0WOwDGiezTIAcBNuTXe25GQha-5LjpVCZwDRYhHGowJkym94q3b9kh-4VlDn_5uBWHJKZsX4F1dTCXTyj8Ja3vQTIiQNE35ZBBKtVuWqA==
  [45] https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQHO3RyuMEVuTuAfL6ia-0AQIVBgQx-bAkVAfFViDtQPm9aH_9EcWOxMP_-gUPK1P-IzxmRI5WNmB2hSGkc6mnJxcuNlErZDXOPAReqX2iFmnpJlesGzUvcPQocAWHR5D9kA8CgmDpmVtQJM-mtFYUQelcr3x2qYtcFaX4HBPFSKq9kYfNJuEjqD
  [46] https://vertexaisearch.cloud.google.com/grounding-api-redirect/AUZIYQFFg5GqPPmlmYEFHndX13JsF-8OK3Il2I_PrOoAJpc2V0DvzLb8yeJp4oAkRiQgPZczDUo_1vy9YpmuPrbnzhLHj3Zb0r-cH2TanvCKwjUfcyddWj5NVeqD5D0PG384tO7-U7yDYOkBD1UfrA==

