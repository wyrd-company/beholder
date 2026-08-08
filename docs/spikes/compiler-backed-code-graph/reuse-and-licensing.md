---
relationships:
  references: compiler-backed-code-graph
---

# Reuse and licensing assessment

This is engineering evidence, not a legal conclusion. `Concept` and `behavior`
mean design learning and black-box comparison. `Schema`, `library`, and `code`
mean implementation reuse subject to the listed notices and a locked transitive
dependency audit.

## Selected transport and providers

| Candidate | Declared terms and dependency shape | Distribution implications | Reuse posture |
| --- | --- | --- | --- |
| [SCIP schema and tools](https://github.com/scip-code/scip/blob/f7c0b174aea88b51dbeef1b583844577efb989e0/LICENSE) | Apache-2.0. Go tools depend on Protocol Buffers, compression, SQLite, language detection, and command-line packages. Checked-in bindings are generated from the Apache schema. | Shipping copied schema, bindings, or tools requires Apache and transitive notices. No separate generated-output terms were found. | Concept, behavior, schema, tests, libraries, and code are suitable. |
| [Rust `scip` crate](https://github.com/scip-code/scip/blob/e8ee0ae6038f8298e2195812eea9d7b1196748ae/bindings/rust/Cargo.toml) | Apache-2.0. Runtime dependency is MIT-licensed `protobuf` 3.7.2. Generated Rust bindings are included. | Small dependency shape already present in `beholder-oracle`. Preserve Apache and MIT notices. | Library and generated bindings are suitable. |
| [rust-analyzer SCIP](https://github.com/rust-lang/rust-analyzer/blob/478b8936bb221e84718ba2aa90906c3b32dfd3c8/crates/rust-analyzer/src/cli/scip.rs) | Dual MIT or Apache-2.0. The command depends on the rust-analyzer workspace and unstable internal crates. | Use a versioned subprocess. Embedding internal HIR crates couples Beholder to a large unstable graph. | Concept, behavior, fixtures, and subprocess use are suitable. Direct library reuse is conditional. |
| [scip-go](https://github.com/scip-code/scip-go/blob/18c307a4ccef5654c866addb42a5e1f579dc4684/LICENSE) | Apache-2.0 Go command using `go/packages`, `go/types`, `x/tools`, SCIP bindings, and Protocol Buffers. `fingerprint.go` carries a Go Authors BSD-style header while the referenced license text is absent from the repository root. | Static binary distribution needs dependency notices. Preserve the Go notice and resolve its exact terms before copying that file. | Subprocess behavior and tests are suitable. Source copying is conditional. |
| [scip-typescript](https://github.com/sourcegraph/scip-typescript/blob/891eb4293709a6a587bf4468dfa1b45a85182fd9/LICENSE) | Apache-2.0 Node.js command using TypeScript compiler API, `google-protobuf`, Commander, and Progress. SCIP bindings are generated. | Shipping carries the JavaScript dependency tree and notices. | Subprocess use, tests, and source are suitable. Direct library use is conditional on dependency fit. |
| [scip-python](https://github.com/sourcegraph/scip-python/blob/8b60bbce1f2a4c7a517776cb395bbafb2e731e4f/LICENSE.txt) | MIT Pyright fork. Published command is a Webpack bundle containing Pyright, Node dependencies, and copied typeshed data. Typeshed fallback contains Apache-2.0 and MIT material. | Ship the bundle only with its embedded notices and dependency inventory. | Subprocess behavior is suitable. Importing or copying the fork is conditional. |
| [scip-dart](https://github.com/Workiva/scip-dart/blob/22a5bdb3c1cf6215117d2f406ae4cf0da685571e/LICENSE) | Apache-2.0 Dart command using analyzer, Protocol Buffers, package configuration, pubspec parsers, and utility packages. Generated Dart bindings are checked in. The pinned source has no lockfile. | Exact transitive versions and licenses require a restored lock audit before redistribution. | Subprocess behavior is suitable. Library or code reuse is conditional. |
| [scip-dotnet](https://github.com/sourcegraph/scip-dotnet/blob/47884461a79839fb74c99e6a0a7978cd7eb62476/LICENSE) | Apache-2.0 .NET tool using Google.Protobuf, Roslyn workspaces/features, MSBuild locator, System.CommandLine, and ConfigurationManager. Its [NOTICE](https://github.com/sourcegraph/scip-dotnet/blob/47884461a79839fb74c99e6a0a7978cd7eb62476/NOTICE) records LsifDotnet-derived command and MSBuild loading code. | Preserve NOTICE and NuGet package notices. | Subprocess use, tests, and source are suitable. Direct library use is conditional. |
| [tree-sitter](https://github.com/tree-sitter/tree-sitter/blob/4f7ab225840f05e080851cc839a1fa92bbf46a36/LICENSE) | MIT Rust/C library. Unicode tables carry separate Unicode and International Components for Unicode notices. Each generated parser follows its grammar repository's license. | Audit every grammar and preserve Unicode notices. Optional WebAssembly support adds Wasmtime. | Library use is suitable. Copied runtime or parser code is conditional on local notices. |

## Wider implementation candidates

| Candidate | Declared terms and dependency shape | Reuse posture |
| --- | --- | --- |
| [Kythe](https://github.com/kythe/kythe/blob/26056edfc953b5d4ea0ed8e94db072caa7f7d4c7/LICENSE) | Apache-2.0 plus University of Illinois/National Center for Supercomputing Applications terms for copied LLVM extractor code. Bazel build spans C++, Go, Java, TypeScript, Protocol Buffers, compilers, and a large third-party tree. | Schema reuse is suitable. Tests, libraries, binaries, and source require component-level notice review. |
| [Glean](https://github.com/facebookincubator/Glean/blob/33154355ec4454034efd588be22c7beb7b00ab9b/LICENSE) | BSD-3-Clause Haskell/C++ system using Thrift, Folly, RocksDB, compression, cryptography, and native libraries. Vendored LMDB uses OpenLDAP Public License 2.8. LSP code preserves MIT and Apache notices. | Concepts and Angle schemas are suitable. Runtime or source distribution needs a full native dependency inventory. |
| [SemanticDB](https://github.com/scalameta/scalameta/blob/256d830d8a710ae1fded9e779a518b49a0996206/LICENSE.md) | BSD-3-Clause Protocol Buffers schema and Scala bindings. NOTICE records BSD, MIT, and Apache-derived benchmark/build material. Scala 2 producer is tied to compiler versions. | Schema and readers are suitable. Compiler plugin is Scala-specific and carries full build inventory. |
| [BSP](https://github.com/build-server-protocol/build-server-protocol/blob/9097c78224910a4a285ba3207e55dd33c9e9da97/LICENSE) | Apache-2.0 Smithy model and generated support libraries. [`bsp4j` NOTICE](https://github.com/build-server-protocol/build-server-protocol/blob/9097c78224910a4a285ba3207e55dd33c9e9da97/NOTICE.md) records Eclipse Public License 2.0 LSP4J-derived code. | Use the Apache schema and JSON-RPC behavior. Treat `bsp4j` source and binaries separately. |
| [Clang tooling](https://github.com/llvm/llvm-project/blob/0aeb516dccab806616070123bd504bdd544cc39d/clang/LICENSE.TXT) | Apache-2.0 with LLVM exception. Native C++ distribution includes Clang/LLVM libraries, built-in headers, and locally licensed third-party components. | Concepts, behavior, and tests are suitable. Libraries are suitable with a matching component inventory. Source copying is conditional. |
| [Roslyn](https://github.com/dotnet/roslyn/blob/821002e9fbca8cbcff74533bdb25400cee299dd8/License.txt) | MIT source with extensive [third-party notices](https://github.com/dotnet/roslyn/blob/821002e9fbca8cbcff74533bdb25400cee299dd8/THIRD-PARTY-NOTICES.txt). Official Microsoft.CodeAnalysis NuGet packages have smaller declared dependency surfaces. | Prefer official packages. Concepts, behavior, tests, and package APIs are suitable. Source-tree copying is conditional. |
| [Joern](https://github.com/joernio/joern/blob/8a73ec09be8fa59dba3cfed5959690c003d7ca52/LICENSE) | Apache-2.0 Scala/JDK platform. Bundled Eclipse CDT in `c2cpg` is Eclipse Public License 2.0. Runtime includes Code Property Graph, FlatGraph, semantic/data-flow modules, parsers, and many frontends. | Concepts and Apache schema code are suitable. Full frontend or binary distribution needs component review. |
| [CodeQL](https://github.com/github/codeql/blob/c9142680f5b6409dbe0944350321c54e8c801e61/LICENSE) | Query libraries and tests are MIT. CLI and engine use restrictive [GitHub CodeQL Terms](https://github.com/github/codeql-cli-binaries/blob/1c54fd8be7e39ed654fdd1f5999aa3320c7edb34/LICENSE.md), including redistribution and use limits. Database distribution terms are not established here. | Public design is suitable conceptually. Behavioral use requires an expressly permitted context. CLI, engine, and database distribution are excluded without a separate license decision. |
| [Sourcetrail](https://github.com/CoatiSoftware/Sourcetrail/blob/master/LICENSE) | GPL-3.0 archived application with language indexers and a custom extension database. | Conceptual and behavioral use only. No implementation dependency. |

## Implementation posture

- Beholder reads SCIP through the existing Apache-2.0 Rust crate and MIT
  Protocol Buffers runtime.
- Language indexers remain versioned subprocess providers. Their compiler and
  package ecosystems do not enter Beholder's runtime dependency graph.
- Permissive schemas and focused fixtures are reusable with their notices.
  Whole compiler and indexing implementations are not vendored.
- A shipped upstream executable requires a software bill of materials and
  notice bundle derived from its locked dependency graph.
- CodeQL CLI is outside the implementation candidate set without a separate
  commercial-license decision.

## Unresolved license evidence

- scip-go does not include the license text referenced by one Go Authors
  BSD-style source header.
- scip-dart's pinned source does not lock its full transitive dependency graph.
- Full source-build inventories are not established for rust-analyzer, Kythe,
  Glean, Joern, LLVM, or Roslyn.
- No open candidate states separate ownership terms for emitted semantic index
  files. CodeQL database distribution terms remain unclear.
