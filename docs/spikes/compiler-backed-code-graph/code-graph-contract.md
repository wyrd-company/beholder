---
relationships:
  references: compiler-backed-code-graph
---

# Candidate compiler-backed code graph contract

## Purpose

The compiler-backed code graph records semantic evidence from one configured
build. It separates build capture, semantic extraction, graph projection, and
analysis so each target language can use its own compiler architecture while
Beholder consumes one contract.

```text
configured build
    |
    v
provider run ---- diagnostics and scope accounting
    |
    v
evidence snapshot ---- resolved, ambiguous, external, unknown
    |
    v
code graph ---- nodes, typed directed edges, supporting evidence
    |
    +---- direct and transitive paths
    +---- coupling measurements
    +---- cycles, regions, and bridge positions
```

Provider output contains observed semantic facts. Beholder derives paths and
measurements from those facts. A derived value retains the nodes and edges that
reproduce it.

## Build capsule

Every snapshot belongs to one immutable build capsule. The capsule contains:

- repository identity and source revision;
- working-tree state when the source is not a clean revision;
- compiler, language service, provider, and transport versions;
- the provider command as an argument vector;
- configured targets, features, profiles, and language options;
- manifest and lockfile digests used to configure the build;
- provider output digest;
- requested and observed source roots; and
- compilation units reported by the provider or build system.

The capsule identifier is a Secure Hash Algorithm 256-bit (SHA-256) digest of
its canonical fields. Machine-local paths and capture time are not identity
inputs. Environment values are absent unless a provider declares a specific
non-secret value as build configuration.

The provider output digest binds the capsule to the exact semantic evidence the
adapter reads. An adapter refuses a digest mismatch.

The build capsule identifier and snapshot identifier are distinct. The
snapshot identifier hashes the complete neutral projection, including producer
and adapter identity, nodes, facts, edges, diagnostics, and validation records.
Reports name the snapshot identifier. Two adapter projections of one provider
input therefore cannot share report-cache identity unless their stored content
is byte-identical.

## Knowledge and provenance

Every field that can be incomplete carries one of three knowledge states:

- `observed`: the provider or build system emitted the value;
- `declared`: the invocation wrapper supplied the value; or
- `unknown`: the provider contract does not establish the value.

Every node, relationship, unresolved reference, diagnostic, and validation
record names its provider evidence. Projection never upgrades declared or
unknown evidence into observed evidence.

Evidence from compiler-backed providers, SCIP Code Intelligence Protocol
(SCIP) indexes, tree-sitter extraction, and future runtime sources can coexist.
Each evidence item retains its producer and method. Consumers select an
explicit evidence policy when several sources describe the same fact.

## Analyzed scope

Analyzed scope contains the configured compilation units and a disposition for
every document known to the provider boundary:

- `included`: semantic evidence was emitted;
- `excluded`: a named configuration rule excludes the document;
- `missing`: the configured build requested the document and the provider did
  not emit it;
- `generated`: the build produced the document; or
- `external`: the document belongs to a dependency or toolchain.

Scope also contains provider diagnostics, validation results, definition and
reference counts, and explicit provider limits. A missing document is not an
empty document. A reference with no target is not proof that no dependency
exists.

Coverage accounting starts from an independent inventory appropriate to the
provider. A syntax provider uses a named-identifier census. A build provider
uses configured targets and source inputs. Each inventory item receives one
disposition, and unaccounted items remain visible.

## Symbols

A symbol contains:

- a snapshot-local opaque identifier;
- an opaque semantic key supplied by the provider adapter;
- language and broad language-neutral kind;
- provider-native kind as evidence;
- display name;
- workspace, external, local, or unknown scope;
- zero or more definition locations;
- optional enclosing symbol; and
- provenance references.

Broad kinds are `package`, `module`, `namespace`, `type`, `callable`, `member`,
`variable`, `parameter`, `macro`, and `other`.

Snapshot-local identifiers prevent accidental joins between configured builds.
Equivalence across builds or providers is a separate evidence-bearing claim.
Document-local semantic keys include document identity. Multiple definition
locations are valid and do not imply ambiguity.

## Reference facts and edges

A reference fact records the source location, optional enclosing source symbol,
raw provider target, edge kind, provenance, and outcome. Each source location
names its UTF-8, UTF-16, UTF-32, or unknown column encoding:

- `resolved`: exactly one target is established;
- `ambiguous`: a bounded candidate set is established;
- `external`: the target is established outside analyzed workspace scope; or
- `unknown`: no target is established, with a typed reason.

Only resolved and external facts create graph edges. Ambiguous facts preserve
their candidates without choosing one. Unknown facts preserve their reason
without inventing a node.

Edge kinds are `reference`, `import`, `implementation`, `type-definition`,
`definition-alias`, and `provider-specific`. A provider-specific edge includes
the provider's stable kind name.

Repeated occurrences between the same source, target, and kind form exactly one
edge. The edge retains every supporting evidence location. Every edge-eligible
fact appears in exactly one matching edge. Every edge endpoint names a stored
node.

## Provider boundary

A provider owns build-system and compiler-specific work. Its output crosses the
boundary as the language-neutral snapshot.

```rust
pub trait CodeGraphProvider {
    fn capture(&self, request: BuildRequest) -> Result<EvidenceSnapshot>;
}
```

`BuildRequest` identifies repository state and requested build configuration.
`EvidenceSnapshot` contains only the contract above. Compiler object models,
language-server handles, database entity types, and transport messages remain
inside the provider adapter.

SCIP is one transport adapter. It supplies symbols, occurrences, locations,
selected symbol relationships, and producer metadata. The invocation wrapper
supplies source revision, build configuration, requested scope, diagnostics,
and output validation because SCIP does not carry those facts.

Tree-sitter is a separate syntax provider. Its evidence keeps syntax provenance
and measured accuracy. It does not inherit compiler-backed authority when its
facts share a graph with compiler evidence.

## Queries

Dependency direction is explicit:

- dependencies follow outgoing edges from a selected symbol;
- dependants follow incoming edges to a selected symbol.

Direct results return one-edge evidence paths. Transitive results return one
deterministic shortest path for every reachable symbol. Stable node and edge
ordering makes repeated queries byte-identical for one snapshot and policy.

Queries accept an evidence policy. The result names the policy and build
capsule. Unknown and ambiguous reference counts accompany exact paths.

The spike implements one explicit policy, `all_stored_edges`, and includes it
in every report. Selecting among configured policies remains a production
schema and application programming interface requirement.

## Coupling analysis

Coupling analysis derives these values from stored nodes and edges:

- fan-in: distinct incoming neighbours;
- fan-out: distinct outgoing neighbours;
- transitive incoming and outgoing reach;
- dependency cycles as strongly connected components;
- connected regions from the graph viewed as undirected; and
- bridge position when removing a node increases the number of connected
  regions.

Cycle and region identifiers derive from stable member ordering. A bridge
result includes the neighbour partitions created by removal. Coupling output
contains the snapshot, policy, node set, and edge set needed to reproduce every
value. The graph contract defines no combined risk classification.

## Storage boundary

Provider snapshots and derived reports are separate versioned artifacts. A
provider snapshot is append-only evidence for one build capsule. A report is a
rebuildable projection over one snapshot and evidence policy.

Schema version, build capsule identifier, provider identity, and evidence
policy are required inputs to reuse. A mismatch causes recomputation instead of
implicit migration.
