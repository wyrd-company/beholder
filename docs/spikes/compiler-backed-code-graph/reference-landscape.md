---
relationships:
  references: compiler-backed-code-graph
---

# Compiler-backed code graph reference landscape

## Reading the matrix

The landscape is organized by the part of the problem each reference solves.
`Conceptual` means its design informs Beholder. `Behavioral` means it supplies a
comparison case. `Implementation` means an interface, schema, library, test, or
program is a candidate for direct use after the accompanying dependency and
license assessment.

No reference supplies all four required boundaries: configured-build capture,
semantic extraction, evidence transport, and explainable coupling analysis.

## Build capture

| Reference | Inputs and build integration | Scope and update behavior | Extension and implementation | Influence |
| --- | --- | --- | --- | --- |
| [Clang JSON Compilation Database](https://clang.llvm.org/docs/JSONCompilationDatabase.html) and [LibTooling](https://clang.llvm.org/docs/LibTooling.html) | One directory, source file, and exact compiler argument vector per translation unit. Multiple commands for one file preserve build variants. LibTooling runs the Clang frontend, Abstract Syntax Tree (AST) visitors, and [matchers](https://clang.llvm.org/docs/LibASTMatchers.html). | The database is authoritative only for listed commands. The format has no update protocol. A consumer reruns affected units. | C++ frontend actions, AST consumers, visitors, matchers, and compiler plugins. | **Implementation** precedent for explicit compilation-unit identity and replayable build inputs. |
| [Build Server Protocol (BSP)](https://build-server-protocol.github.io/docs/specification.html) | JavaScript Object Notation Remote Procedure Call (JSON-RPC) exposes build targets, sources, dependency sources and modules, compiler options, inverse sources, and compile/test/run operations. | Capability negotiation states what a server exposes. `buildTarget/didChange` invalidates cached target data. The protocol describes build topology and diagnostics, not semantic completeness. | Language-specific data fields, protocol extensions, and new build-server implementations. | **Conceptual** provider boundary; possible **implementation** input when a target ecosystem has a reliable server. |
| [Kythe extraction](https://kythe.io/docs/kythe-overview.html) | Extractors capture hermetic compilation units, source inputs, arguments, and dependencies into `.kzip` archives. Integrations include Bazel, Java compiler, Maven, CMake, and Go. | Compilation-unit claiming supports parallel deduplication. Coverage remains the union of captured compilation units. | New build extractors and language indexers feed the [Kythe schema](https://kythe.io/docs/schema/). | **Conceptual** extract/index/serve separation; selected extractor patterns are **implementation** candidates. |
| [CodeQL database creation](https://docs.github.com/en/code-security/reference/code-scanning/codeql/codeql-cli-manual/database-create) | `none`, `autobuild`, and manual modes either inspect source or trace a configured build. Language extractors create one database per language and source snapshot. | Official [compiled-language guidance](https://docs.github.com/en/code-security/how-tos/find-and-fix-code-vulnerabilities/manage-your-configuration/codeql-for-compiled-languages) makes build-mode coverage differences explicit. Databases are analyzed snapshots, not a public incremental update protocol. | Extractor options, query packs, model packs, and extensible predicates. | **Behavioral** benchmark for build-mode provenance and coverage claims; **conceptual** query influence. |

## Semantic interchange and graph systems

| Reference | Semantic engine and model | Queries and derived data | Completeness and incrementality | Extension, runtime, and influence |
| --- | --- | --- | --- | --- |
| [Semantic Code Intelligence Protocol (SCIP)](https://github.com/scip-code/scip/blob/main/scip.proto) | Producer-neutral Protocol Buffers documents contain occurrences, symbols, diagnostics, external symbols, and navigation relationships. Package-qualified symbol grammar is language-neutral. | Supports definitions, references, implementations, type definitions, hover, and diagnostics. Enclosing ranges can support containment-based call hierarchy. | A complete workspace snapshot with no update protocol. The [design](https://github.com/scip-code/scip/blob/main/docs/DESIGN.md) permits compiler-precise and heuristic producers. It has no configured-build identity, analyzed-scope ledger, ambiguity candidates, confidence, or general unresolved record. | Go reference tools and generated language bindings. **Implementation** transport and symbol grammar, wrapped by Beholder evidence records. |
| [Language Server Index Format (LSIF)](https://microsoft.github.io/language-server-protocol/specifications/lsif/0.6.0/specification/) | Newline-delimited JSON graph connects source ranges and result sets to precomputed Language Server Protocol (LSP) answers. Projects and monikers support cross-project navigation. | Definitions, references, hover, semantic tokens, diagnostics, and package navigation. | Snapshot stream with no update or completeness model. The ecosystem site is archived and the TypeScript indexer recommends SCIP. | JSON graph producers and consumers. **Conceptual** and **behavioral** predecessor only. |
| [SemanticDB](https://scalameta.org/docs/semanticdb/specification.html) with [Metals BSP integration](https://metalsmith.io/metals/docs/integrations/new-build-tool/) | Scala and Java compiler plugins emit document checksums, symbols, signatures, types, definition/reference occurrences, synthetics, diagnostics, and optional build-target identity. | Navigation, refactoring, semantic rewrites, and documentation tools. | Compiler emits document snapshots while build server and Metals manage workspace updates. No general ambiguity or coverage record. | Scala/JVM compiler plugins and consumers. **Conceptual** precedent for checksums, target identity, signatures, and synthetics. |
| [Kythe graph](https://kythe.io/docs/schema/) | Language-neutral VNames identify nodes. Facts and typed edges describe definitions, references, calls, inheritance, types, containment, and documentation. | Graph store plus cross-reference serving and exploration. | Output completeness is extractor/indexer-specific. Optional and advisory edges make evidence strength non-uniform. | C++, Java, Go, and selected build integrations. **Conceptual** model; selective **implementation** reuse. |
| [Glean](https://glean.software/docs/introduction) | Language-native typed facts live behind versioned schemas. Stable fact identifiers, ownership, and the language-neutral `codemarkup` layer preserve native detail and shared views. | Angle queries plus stored and on-demand [derived predicates](https://glean.software/docs/derived/). | [Stacked databases](https://glean.software/docs/implementation/incrementality/) hide changed units and add replacement facts. Ownership propagates through derivation. Schemas can model uncertainty but do not create it automatically. | Haskell/C++ service, language indexers, schemas, and query clients. **Conceptual** strongest precedent for native evidence, ownership, and rebuildable neutral projections. |
| [Joern Code Property Graph (CPG)](https://docs.joern.io/code-property-graph) | Language frontends produce a property graph combining AST, control flow, call graph, and program dependence data. Overlays add semantic layers to robust or incomplete parses. | Scala-based traversal and data-flow query language over the [CPG specification](https://cpg.joern.io/). | Snapshot-oriented. Optional semantic edges and later overlays make partial evidence explicit in the construction process, but there is no general configured-build completeness contract. | Scala, FlatGraph, X2CPG frontends, overlays, and plugins. **Conceptual** analysis vocabulary and **behavioral** query comparator. |

## Target-language semantic producers

Each producer emits a bulk SCIP snapshot. None adds configured-build identity,
analyzed-scope accounting, ambiguity candidates, or a completeness proof to the
SCIP schema. A Beholder provider therefore owns those records around the index.

| Target | Producer and build context | Semantic engine, known boundary, and update model | Implementation and influence |
| --- | --- | --- | --- |
| Rust | [`rust-analyzer scip`](https://github.com/rust-lang/rust-analyzer/blob/master/crates/rust-analyzer/src/cli/scip.rs) loads Cargo/crate graph, configuration flags, build scripts, and procedural macros. | rust-analyzer High-level Intermediate Representation (HIR) provides name resolution, macro expansion, and type inference. Salsa is incremental inside the interactive engine; SCIP is a bulk snapshot. Official discussion describes current SCIP dependency output as limited. | Rust. **Implementation** prototype candidate through its process boundary; native HIR crates are not Beholder's shared interface. |
| Go | [scip-go](https://github.com/scip-code/scip-go) runs Go tooling and `golang.org/x/tools/go/packages` for modules and workspaces. Alternate builds can use Go Packages Driver Protocol. | Compiler type information and package variants. README documents cross-repository and non-module limits. Go caches help extraction, while output remains a snapshot. | Go. **Implementation** candidate with an explicit Go build/environment capsule and package-variant scope. |
| TypeScript | [scip-typescript](https://github.com/sourcegraph/scip-typescript) reads `tsconfig.json`, JavaScript projects, and Yarn/pnpm workspaces. | TypeScript compiler symbols and types. Project-level symbol caching improves bulk indexing. Project references, overloads, declaration merging, and structural types remain required fixture challenges rather than completeness claims. | TypeScript/Node.js. **Implementation** candidate. |
| Python | [scip-python](https://github.com/sourcegraph/scip-python) reads project configuration and active Python/pip environment or an explicit package manifest. | Pyright parsing, import resolution, type checking, and inference. General runtime behavior is outside static resolution. Output is a memory-intensive bulk snapshot. | TypeScript/Node.js plus Python environment. **Implementation** candidate with explicit dynamic/unknown evidence. |
| Dart | [scip-dart](https://github.com/Workiva/scip-dart) requires restored pub packages and reads package configuration, pubspec, and lock data. | Dart analyzer symbols and relationships. Package-local indexes and Flutter build context require explicit compilation-unit boundaries. | Dart. **Implementation** candidate after declared terms and package-boundary behavior are verified. |
| C# | [scip-dotnet](https://github.com/sourcegraph/scip-dotnet) loads repository, solution, or project state through Microsoft Build Engine (MSBuild). | Roslyn workspace, compiler symbols, and type information. Roslyn [`SymbolInfo`](https://learn.microsoft.com/en-us/dotnet/api/microsoft.codeanalysis.symbolinfo) exposes candidate symbols and reasons that SCIP cannot retain without augmentation. | C#/.NET. **Implementation** candidate; a direct Roslyn adapter preserves more ambiguity evidence than SCIP. |

## Query and analysis products

| Reference | Graph and analysis behavior | Evidence boundary | Influence |
| --- | --- | --- | --- |
| [Sourcegraph precise navigation](https://sourcegraph.com/docs/code-search/code-navigation/precise_code_navigation) | Consumes SCIP/LSIF from manual jobs, Continuous Integration (CI), or auto-indexing. Answers definitions, references, implementations, and cross-repository package navigation for repository revisions. Search navigation is a lower-accuracy fallback. | Proves SCIP deployment at scale. It does not expose a general typed dependency/coupling graph contract. | **Behavioral** production benchmark. |
| [SciTools Understand](https://docs.scitools.com/help/dependencies/what-are-dependencies.html) | Proprietary parsers create entities and references at file, type, function, and variable boundaries. Product exposes callers/callees, dependency, butterfly, call, control-flow, and architecture graphs plus metrics and scripting APIs. | Public behavior defines dependency from references. Public docs make no machine-readable completeness, ambiguity, or incremental guarantee. | **Behavioral** comparator for dependency exploration and explainable graph views. |
| [Teamscale architecture analysis](https://docs.teamscale.com/reference/ui/architecture/) | Maps code into nested components and evaluates allowed, tolerated, and denied dependencies. The [incremental engine](https://docs.teamscale.com/introduction/why-teamscale/) analyzes each commit and tracks findings through history. | Proprietary model. Supported architecture-language list covers several targets but not the full set. No public uncertainty contract. | **Behavioral** benchmark for incremental coupling and architecture conformance. |
| [Sourcetrail](https://github.com/CoatiSoftware/Sourcetrail) | Offline graph explorer for C/C++, Java, and Python with interactive caller/callee and dependency views. [SourcetrailDB](https://github.com/CoatiSoftware/SourcetrailDB) enabled custom indexers. | Original project is archived and GPL-3.0. Language coverage excludes most targets. | **Conceptual** and **behavioral** only. |

## Resulting boundaries

- Build capture, semantic extraction, evidence transport, graph projection, and
  derived analysis remain separate responsibilities.
- SCIP is the first transport adapter. It is not the foundational graph or
  coverage contract.
- Compiler-native ambiguity is preserved before transport flattening. Roslyn's
  resolved symbol, candidate set, and candidate reason provide the concrete
  model.
- Compilation units and source ownership are first-class. Kythe extraction,
  BSP build targets, and Glean fact ownership provide independent precedent.
- Language-native facts project into a shared graph. Glean demonstrates why a
  shared view need not erase native evidence.
