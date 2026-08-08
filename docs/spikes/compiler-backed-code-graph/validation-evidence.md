---
relationships:
  references:
    - compiler-backed-code-graph
    - scip-tree-sitter-spike
---

# Compiler-backed code graph validation evidence

## Reproduction boundary

The six provider inputs came from the related SCIP and tree-sitter spike at
commit `c98d422`. Regenerate one target from that checkout with:

```sh
RUN=after LANGUAGE=<rust|go|typescript|python|dart|csharp> scripts/harness-run.sh
```

The harness writes `oracle.scip`, `spike.scip`, `manifest.json`, and
`report.json` below `.work/harness/after/<language>/`. Its fixture roots are
`tagver` plus `open-oscar-server`, `dub`, `fastmcp`, `dart-data`, and
`community.nservicebus.transport.nats` in
`/workspaces/references/scip-references/fixture-repos/`.

Verify an input before import:

```sh
sha256sum <oracle.scip>
scip lint <oracle.scip>
cargo run -p beholder-code-graph-spike -- import \
  --index <oracle.scip> \
  --output <graph.json> \
  --repository <fixture-identity> \
  --root .
```

`scip lint` is expected to fail for all six pinned inputs. The import succeeds
and records the provider digest, diagnostics, and uncertainty independently.
The hashes, byte sizes, graph counts, and lint issue counts are in
[`findings.md`](findings.md).

The 638-node Rust vertical slice is a separate fresh index of `tagver` revision
`45fb5465b421a7b72b654cc3d30a3dce9a54eea3`. Its input is 372,852 bytes with
SHA-256
`3ee399a85d9e442d858f710f7d0f91e889650948e30152a0a8c8adb9b17fe03c`.
The six-language Rust row uses the pinned 368,943-byte oracle input and has 604
nodes. The provider output and repository state differ, so the node counts are
not expected to match.

## Deliberate mutations

Each mutation was applied alone to the committed prototype. The named focused
test failed. The reverse patch restored the source before the next mutation.

| Guard changed | Assertion that killed the mutation |
| --- | --- |
| Provider input digest comparison | `provider_input_digest_is_load_bearing` |
| Local symbol document key | `local_symbols_are_document_scoped` |
| Import role mapping | `import_role_creates_an_import_edge` |
| Ambiguous outcome projection | `ambiguous_target_emits_no_edge` |
| Unsafe path classification | `document_path_outside_root_is_opaque_external_scope` |
| Snapshot-local node identifier | `node_ids_are_snapshot_local` |
| Capsule integrity comparison | `graph_integrity_rejects_build_identity_tampering` |
| Dependency direction | `direct_dependency_and_dependant_directions_are_opposites` |
| Breadth-first shortest-path selection | `transitive_reach_uses_a_deterministic_shortest_path` |
| Strongly connected component cycle test | `strongly_connected_cycles_are_reported_with_evidence` |
| Bridge removal comparison | `bridge_position_is_proved_by_node_removal` |
| Weak-region traversal | `weak_regions_ignore_edge_direction_and_include_isolates` |
| Duplicate occurrence evidence retention | `duplicate_occurrences_retain_every_evidence_record` |
| Provider diagnostic retention | `provider_diagnostics_retain_location_and_provenance` |
| Requested-root canonicalization | `capsule_identity_canonicalizes_requested_root_order` |
| External definition-alias outcome | `definition_alias_to_external_symbol_stays_external` |
| Mandatory edge evidence | `graph_integrity_requires_edge_evidence` |

## Validation commands

Run the focused prototype gate:

```sh
cargo fmt --all -- --check
cargo clippy -p beholder-code-graph-spike --all-targets -- -D warnings
cargo test -p beholder-code-graph-spike
```

Run the repository gate with the Homebrew `libgit2` shared library available:

```sh
LD_LIBRARY_PATH=/home/linuxbrew/.linuxbrew/opt/libgit2/lib task ci
```

The repository gate covers all workspace unit, identity, store, action,
formatting, Clippy, actionlint, and ShellCheck checks.
