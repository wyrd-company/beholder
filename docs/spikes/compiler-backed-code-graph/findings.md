---
relationships:
  derives-from: compiler-backed-code-graph
  references:
    - compiler-backed-code-graph-contract
    - compiler-backed-code-graph-reference-landscape
    - compiler-backed-code-graph-reuse-and-licensing
    - compiler-backed-code-graph-validation-evidence
    - scip-tree-sitter-spike
---

# Compiler-backed code graph findings

## Answer

A build-aware provider boundary and language-neutral evidence graph are viable.
Providers must retain their compiler-native build model and uncertainty before
projecting facts into the shared graph. SCIP Code Intelligence Protocol (SCIP)
is a useful transport adapter. It is not a sufficient build capsule, coverage
model, or universal coupling graph.

The working Rust prototype proves the boundary against a real Cargo build. Its
consumer imports compiler-backed evidence without rust-analyzer types, returns
direct and transitive paths in both directions, and derives distinct degree,
cycles, weak regions, and bridge positions from stored nodes and edges alone.

## Selected vertical slice

The end-to-end slice uses `rust-analyzer scip` as a versioned subprocess and
the Apache-2.0 Rust `scip` crate as the transport reader. The adapter emits the
versioned neutral model defined in the contract. The consumer has no compiler
or SCIP types outside the adapter module.

The real `tagver` build produced:

- 20 documents, 638 nodes, 992 edges, and 3,271 reference facts;
- 1,495 resolved, 22 ambiguous, and 1,754 unknown outcomes;
- a direct query with 51 outgoing neighbours for a selected `main` symbol;
- one dependency cycle, 76 weak regions, and 99 bridge positions; and
- a snapshot-local build identifier bound to the exact provider output digest.

The selected `main` semantic key had definition locations in both
`crates/cli/build.rs` and `crates/cli/src/main.rs`. This is an upstream symbol
identity collision across Cargo targets. A provider wrapper must preserve
target and compilation-unit identity rather than accepting the SCIP symbol as
globally unique.

## Six-language challenge

The adapter consumed the pinned compiler-backed oracle output for every target
language. `R`, `A`, `E`, and `U` mean resolved, ambiguous, external, and unknown
reference outcomes.

| Target | Provider | Documents | External documents | Nodes | Edges | Reference facts | R / A / E / U | Diagnostics |
| --- | --- | ---: | ---: | ---: | ---: | ---: | --- | ---: |
| Rust | rust-analyzer | 20 | 0 | 604 | 992 | 3,271 | 1,495 / 22 / 0 / 1,754 | 0 |
| Go | scip-go | 272 | 11 | 47,473 | 83,463 | 264,733 | 213,671 / 0 / 0 / 51,062 | 2 |
| TypeScript | scip-typescript | 4,100 | 0 | 132,129 | 131,515 | 433,828 | 223,921 / 1,208 / 0 / 208,699 | 85 |
| Python | scip-python | 821 | 0 | 65,518 | 108,894 | 225,425 | 145,547 / 0 / 27,574 / 52,304 | 0 |
| Dart | scip-dart | 252 | 0 | 8,837 | 26,196 | 51,901 | 38,615 / 0 / 13,096 / 190 | 130 |
| C# | scip-dotnet | 22 | 0 | 553 | 0 | 2,381 | 1,165 / 0 / 0 / 1,216 | 0 |

The counts are evidence about each provider and fixture. They are not comparable
language quality scores. The fixtures differ greatly in size, dependency
closure, configuration, and generated code.

The 638-node vertical slice used a fresh index of `tagver` revision
`45fb5465b421a7b72b654cc3d30a3dce9a54eea3`. The
604-node Rust matrix row used the pinned six-language oracle. Their exact
revision, input digests, sizes, and reproduction commands are recorded in the
[validation evidence](validation-evidence.md).

All six source indexes failed upstream `scip lint`. The observed issue-line
counts were Rust 1,754, Go 51,964, TypeScript 302,319, Python 79,321, Dart
1,055, and C# 1,264. Common causes were missing external `SymbolInformation`,
relationships to absent symbols, repeated document paths, and local symbols in
invalid scopes. Adapter graph-integrity validation therefore remains distinct
from upstream transport lint.

### Language-shaped limits

| Target | Viable acquisition path | Build context and explicit limit |
| --- | --- | --- |
| Rust | `rust-analyzer scip` subprocess, with a future native rust-analyzer provider only if SCIP gaps block required queries | Capture Cargo workspace, target, features, conditional compilation, build scripts, and procedural macros. SCIP omitted external symbols and left 53.6% of facts unknown in the fixture. Package symbols collided across Cargo targets. |
| Go | `scip-go` subprocess over `go/packages` | Capture modules, workspaces, build tags, environment, and package variants. The index included 11 compiler-cache documents outside repository root; the adapter retained only opaque external dispositions. Unknown outcomes were 19.3%. |
| TypeScript | `scip-typescript` subprocess for the first provider; direct TypeScript compiler augmentation when project identity or candidates are required | Capture every `tsconfig`, project reference, package manager, and declaration input. Repeated paths from several projects require document-instance evidence. The provider omitted its language field and left 48.1% of facts unknown. |
| Python | `scip-python` subprocess backed by Pyright; direct Pyright evidence when dynamic or candidate detail is required | Capture interpreter, environment, installed distributions, stubs, and configuration. The provider omitted its language field. Unknown outcomes were 23.2%; established external outcomes were 12.2%. Runtime dispatch remains unknown. |
| Dart | `scip-dart` subprocess after dependency-lock and notice audit | Capture pub workspace, package configuration, Dart or Flutter mode, conditional imports, and generated sources. The fixture had 0.4% unknown outcomes and 25.2% established external outcomes. Redistribution remains blocked on the missing dependency lock audit. |
| C# | Direct Roslyn provider or an augmented scip-dotnet wrapper | Capture solution and project graph, target frameworks, properties, references, generators, and compilations. scip-dotnet emitted resolved targets but no enclosing ranges, so the adapter could not establish source symbols and emitted no dependency edges. |

## Model result

The shared model remained viable across the six outputs after four provider
assumptions became explicit:

- document-local keys include relative document identity and repeated provider
  documents retain distinct evidence identifiers;
- paths outside repository root become opaque external dispositions rather
  than machine-local graph locations;
- absent language metadata becomes `unknown`; and
- resolved, ambiguous, external, and typed unknown outcomes remain distinct.

Snapshot-local node identifiers include the canonical build capsule identifier.
Changing repository, revision, configuration, provider input, producer, or
requested roots changes the capsule. The adapter rejects an input digest
mismatch. Graph validation rejects dangling nodes, dangling or empty evidence,
empty ambiguity, outcome-to-scope contradictions, duplicate evidence
identifiers, evidence that does not support its edge, contradictory knowledge
or scope states, edge-eligible evidence with no projected edge, and capsule
identity tampering. Source locations retain each provider document's UTF-8,
UTF-16, UTF-32, or unknown position encoding.

SCIP repeated some TypeScript paths for several configured projects without a
compilation-unit identifier. The adapter retains distinct evidence records but
projects same-path local symbols together. The production schema must link each
definition, reference, and local symbol to a compilation unit before it can
compare configuration variants. Until then, repeated-path evidence supports a
workspace union only.

## Query and coupling result

Direct dependency and dependant queries return edge-bearing paths. Transitive
queries use deterministic breadth-first search and return one shortest evidence
path per reachable node. Query and coupling reports name the snapshot, the
`all_stored_edges` evidence policy, uncertainty counts, and the stored node and
edge sets required by the report. Arbitrary policy selection remains a
production requirement; the spike implements only the named all-stored policy.
Coupling output includes:

- distinct incoming and outgoing neighbours plus their evidence edges;
- transitive reach as inspectable paths;
- strongly connected cycles from Tarjan analysis;
- weakly connected regions, including isolated nodes; and
- bridge positions with before-and-after region counts, neighbour partitions,
  and incident evidence.

These values are projections. The provider snapshot remains the evidence source.
No combined risk score is part of the graph contract. The proof ran full
coupling analysis on the 638-node Rust graph. Its per-node transitive paths are
an all-pairs expansion and were not run on the 47,473-node Go or 132,129-node
TypeScript graphs. Production analysis must query reach on demand or store a
compact reachability representation.

## Validation

The prototype has 40 focused unit and integrity tests. Seventeen exploratory
mutations were each killed by their named assertion. The killed guards covered
provider digest binding, local identity, imports, ambiguity, external paths,
capsule derivation and validation, query direction, shortest paths, cycles,
weak regions, bridges, diagnostic retention, repeated evidence, external alias
classification, and mandatory edge evidence.

The mutation patches and failing logs were not retained. This observation is
not independently reproducible and is not completion proof. The committed
regression suite and repository gate are the durable validation evidence.

The Rust snapshot was 3.49 MB from a 369 KB SCIP input. The Go snapshot was
287.60 MB from a 32.24 MB SCIP input. Pretty JSON expansion was 9.5 times and
8.9 times respectively. JSON is suitable inspectable spike evidence. A
production store needs compact encoding, streaming ingestion, and derived-report
caching keyed by snapshot and algorithm version.

## Reuse and licensing result

The implementation reuses only the Apache-2.0 SCIP Rust binding and its MIT
Protocol Buffers runtime. Compiler-backed indexers remain versioned subprocesses.
No candidate compiler or indexer source was copied.

SCIP schema and bindings are suitable implementation inputs. CodeQL remains a
conceptual or expressly permitted behavioral reference because its command-line
engine has restrictive terms. scip-go's unbundled Go Authors BSD notice,
Build Server Protocol's Eclipse Public License 2.0 bsp4j lineage, Joern's
Eclipse CDT code, and scip-dart's unlocked dependency graph require separate
review before code or binary distribution.

## Disposition

Retain SCIP as one transport adapter with build-capsule augmentation. Retain
tree-sitter as a syntax inventory, unaccounted-identifier census, source-shape
witness, and explicit fallback. Neither source defines the foundational graph,
and evidence from either source never gains compiler provenance through
projection.

Implement one build-capsule runner and provider contract. Land the proven Rust
vertical slice first. Use Go as the first portability and build-variant test.
Require each provider to publish an independent input inventory, document
dispositions, diagnostics, uncertainty counts, and provider-specific validation
before its coupling answers are trusted.

## Evidence inputs

| Target | Pinned index SHA-256 | Bytes |
| --- | --- | ---: |
| Rust | `bbf519df55a9a2626a4a9ed9d6d86e330f03c15af9c3613e370aeebd137cb6e3` | 368,943 |
| Go | `a947beb54ef0c88efc7f45969e015750c1544ecbc6bf53bddc27cc2219d89339` | 32,238,435 |
| TypeScript | `7b8344d3eb9d6f7fbf69e53f205bb42ae4131a95b298e2a5563c576b68765913` | 68,245,465 |
| Python | `9ca184fcf14b4514c8f254857c804c020df0e18283411e5b29d2928c924d813f` | 38,001,306 |
| Dart | `26fb908ba5312abf80a27476e535ab1604db54917e5c53c11397596f8ae11a36` | 4,577,649 |
| C# | `7cfa3ca0fb9a1f2a15c8f3fa6a91a1d587701e5b474de01f5908954d0f2dcd99` | 363,448 |

The source-to-oracle comparison method and syntax completeness census are
defined by the related SCIP and tree-sitter findings. The
[validation evidence](validation-evidence.md) records regeneration inputs and
the Dart lock gap and mutation evidence limit. The reference landscape,
contract, and reuse assessment contain the primary sources and dependency
details behind these conclusions.
