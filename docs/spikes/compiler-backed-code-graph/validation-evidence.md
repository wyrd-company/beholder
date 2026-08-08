---
relationships:
  references:
    - compiler-backed-code-graph
    - scip-tree-sitter-spike
---

# Compiler-backed code graph validation evidence

## Reproduction boundary

The six provider inputs came from the related SCIP and tree-sitter spike at
commit `c98d422fc22b3bb93936c6d69bb888ae93efac30`. Regenerate one target from
that checkout with:

```sh
RUN=after LANGUAGE=<rust|go|typescript|python|dart|csharp> scripts/harness-run.sh
```

The harness writes `oracle.scip`, `spike.scip`, `manifest.json`, and
`report.json` below `.work/harness/after/<language>/`. Its fixture roots are
`/workspaces/tools/tagver` plus `open-oscar-server`, `dub`, `fastmcp`,
`dart-data`, and `community.nservicebus.transport.nats` below
`/workspaces/references/scip-references/fixture-repos/`.

The scratch manifests were not retained. This table preserves their commit
identity fields so the pinned bytes can be regenerated from clean checkouts.

| Target | Corpus commit | Oracle source commit |
| --- | --- | --- |
| Rust | `45fb5465b421a7b72b654cc3d30a3dce9a54eea3` | `478b8936bb221e84718ba2aa90906c3b32dfd3c8` |
| Go | `ab70f80a630cf36c931ad174d51396e07e0103a0` | `18c307a4ccef5654c866addb42a5e1f579dc4684` |
| TypeScript | `b0836571a7dfb0c25e648764e47d97636b419fe8` | `891eb4293709a6a587bf4468dfa1b45a85182fd9` |
| Python | `8a1820f1c38401fa02c8996a7a30e864561b80d3` | `8b60bbce1f2a4c7a517776cb395bbafb2e731e4f` |
| Dart | `7161db1bb0be31f8b9bd4f68f575b80680988936` | `22a5bdb3c1cf6215117d2f406ae4cf0da685571e` |
| C# | `2f4fc124dc42cd4f118c223015d5e716c0f08542` | `47884461a79839fb74c99e6a0a7978cd7eb62476` |

The validator source is pinned at
`f7c0b174aea88b51dbeef1b583844577efb989e0`. The related findings record the
Rust, Go, Node.js, npm, pnpm, Python, uv, Dart, .NET Software Development Kit,
and Flutter versions used by the harness.

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
It used `rust-analyzer 1.97.1 (8bab26f 2026-07-14)` and this exact invocation:

```sh
rust-analyzer scip /workspaces/tools/tagver \
  --output <tagver.scip> \
  --exclude-vendored-libraries
```

The six-language Rust row uses the pinned 368,943-byte oracle input and has 604
nodes. The provider output and repository state differ, so the node counts are
not expected to match.

## Exploratory mutations

Each mutation was applied alone to the committed prototype. The named focused
test failed. The reverse patch restored the source before the next mutation.
The exact patches and failing command output were not retained. This table is
an observation about guard sensitivity, not independently reproducible
validation evidence. The committed regression tests above are the durable
replacement.

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
