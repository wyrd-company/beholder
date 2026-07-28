# beholder

Beholder is a structural index of a codebase that answers two questions: what is risky, and what depends on what. Metrics are a view over the index, not the point of it.

- Rust
- Library + CLI
- Publish
  - crates.io beholder / beholder-cli
  - wyrd-company/homebrew-tools
  - npm - @wyrd-company/beholder

## Languages

Rust, Go, TypeScript, C#, Python, Dart.

Language support is data, not code. Each language is a set of tree-sitter queries plus a metadata table: which node kinds are symbols, which node kinds increment complexity and which nest, and how imports and module declarations are written. Adding a language is a directory of query files and a table entry. No language has a code path of its own.

`docs/adding-a-language.md` is the procedure, down to which files a new language adds.

## Analysis tiers

Every result declares the tier that produced it. Beholder degrades across tiers, never silently.

- **Tier 0 — indentation density.** No grammar required. Applies to every file that exists, including formats with no tree-sitter support.
- **Tier 1 — tree-sitter.** Real symbol boundaries, line ranges, and nesting-weighted cognitive complexity.
- **Tier 2 — resolved references.** Heuristic resolution from imports and identifier occurrences. An external index may supply resolved references in place of the heuristic resolver.

## Analysis phases

Analysis runs in two phases with a hard boundary between them. The boundary determines what parallelizes, what is addressable by content hash, and what must be recomputed when anything in the file set changes.

### Phase 1 — per file

A pure function of one file's repo-relative path and its content, with no knowledge of any other file.

Both inputs matter. The path is what a qualified name is built against and what a symbol's identity is anchored to, so it cannot be dropped. What phase 1 must never depend on is machine-local state: an absolute path, the checkout location, the wall clock, or anything about the machine that ran it. A repo-relative path carries none of that — it is the same string in every clone.

The same rule decides which files are analyzed at all. Only ignore rules that live in the repository count: its own `.gitignore` files and `beholder.toml`. A global excludes file, `.git/info/exclude`, and `.gitignore` files above the repository are all machine-local and invisible to a committed tree, so beholder does not read them. Otherwise the same commit would analyze differently on a laptop and in CI.

Produces:

- Symbols: kind, name, qualified path, line range
- Cognitive complexity per symbol
- Tier 0 density for the file
- Intra-file reference pairs
- Import and module declarations
- Identifier occurrences

Phase 1 is embarrassingly parallel and deterministic: the same path and the same content always yield the same result, on any machine, in any checkout location. That determinism is what makes a phase 1 result safe to store and reuse, keyed on the path together with the content hash.

### Phase 2 — whole set

A function of the complete phase 1 output for every file in scope.

Produces:

- Resolved reference edges between symbols
- Fan-in and fan-out per symbol, kept as separate values
- Cohesion components per file
- Within-language percentile ranks
- Risk ranking

Adding, removing, or changing any file can change phase 2 results for files that did not change. No phase 2 result is attributable to a single file, and none is addressable by file content.

Delta mode runs both phases over two revisions and compares phase 2 output by symbol identity.

## Stored results

Generated phase results live in a custom git ref, `refs/beholder/index`. They are not workspace files and there is no service behind them. A larger repository cold-starts from work someone else already did, and a differential recomputes only what actually changed.

### The ref

`refs/beholder/index` is an ordinary ref outside `refs/heads/*`. It needs no branch, no checked-out worktree, no GitHub-specific API and no artifact store. It travels through ordinary git:

```sh
git push origin +refs/beholder/*:refs/beholder/*
git fetch origin +refs/beholder/*:refs/beholder/*
```

A fresh clone fetches the ref, resolves the source commit an index refers to, and reuses the stored phase output for every file whose content is unchanged.

### The index commit

Each index commit carries two parents, in this order:

0. the source commit whose analysis it stores
1. the previous index commit

The first index for a repository has only parent 0, because there is no previous index. Walking parent 1 from the ref tip is the index history, newest first. Reading parent 0 of any index names the source it describes. Naming the source commit as a parent is also what makes the ref self-contained: pushing the index carries the analyzed objects with it.

The tree mirrors the source layout, so a diff between two index commits reads as a diff of the analysis.

```text
meta.json                 fingerprint plus the source commit id
phase1/<repo path>.json   one phase 1 result per analyzed file
phase2.json               the whole-set phase 2 result
```

### Derived cache, not source of truth

Nothing stored is authoritative, and beholder never trusts it on sight. Metadata is checked first: source commit, schema version, tool version, analysis configuration, language table and path rules. Then the payload is checked against the tree it claims to describe — the same set of files, every path repo-relative, and each file's content hash matching the blob that is actually there. Anything else is recomputed. Results that are missing are generated and appended; results that are stale are ignored.

`index --store` refuses to file an analysis of a dirty working tree against `HEAD`, because a stored result that is quietly wrong is worse than no stored result: every later run and every fresh clone would trust it.

### Concurrent writers

Ref updates are compare-and-swap against the tip the commit was built on. A writer that loses fetches the tip that beat it, rebuilds against that new index parent, and retries.

Pushes are never forced, for the same reason. A force push would make the compare-and-swap meaningless the moment two machines were involved, because the loser would never learn it had lost. A rejected push is the signal to fetch the winning tip and rebuild on it. No existing result is rewritten, and no history is discarded.

## Core features

1. **Symbol-level index.** Every file, function, and type with path, kind, name, and line range. Symbol identity is stable across line shifts, so a reformat is not a change. Anonymous constructs such as closures are not symbols — nothing about them survives a later revision to match on — and their complexity belongs to the symbol that contains them. Everything else is a query over this.

2. **Cognitive complexity per symbol.** Nesting-weighted, not cyclomatic. Attached to a line range, so a consumer reads exactly the span that scored.

3. **The reference graph.** Fan-in and fan-out kept separate — they mean opposite things. Queryable in both directions: what calls this, what does this call, what breaks if this signature changes.

4. **Cohesion by connected components.** Intra-file symbol references partitioned into components. One component is cohesive at any size; three components are three files in a trenchcoat. This is the honest form of "too many types in a file."

5. **Delta mode.** Any two revisions in, changed symbols out with before and after metrics.

6. **Risk ranking.** Complexity delta weighted by fan-in, ranked by percentile within a language and merged across languages. Scores from different languages are not comparable; percentile ranks are. Ranking decides what surfaces and what stays quiet, so it is a core calculation rather than a presentation concern.

7. **Machine-first output.** JSONL primary, SARIF second, human format last. `symbols.jsonl` carries one record per function and type; `files.jsonl` carries one record per file the walk visits, including files no grammar claims. Output is deterministic, stably sorted, and repo-relative.

8. **Review surfaces.** GitHub Action and gitpr. One sticky comment edited in place, SARIF annotations inline on the risky lines, and silence when nothing is risky.

9. **Resolver validation.** Heuristic resolution is measured against an external resolved index, reported as precision and recall per language, so the graph carries a known error bar. Every edge in `edges.jsonl` carries it. Method calls are the weak case — without receiver types, a name that exists once in a project attracts every call that shares it — and `docs/resolver-accuracy.md` has the measurement and the error classes.

## Not in scope

- **No aggregate score.** No letter grade, no maintainability index. Aggregates invite gaming and turn an instrument into a gate.
- **No gating.** Beholder reports. Anyone who wants a build gate can build one on the JSONL and own that decision.
- **No index service.** Generated results live in a git ref and travel through ordinary fetch and push. Nothing in beholder requires a server, a database, or a hosted artifact store.
- **No intent analysis.** Comparing a change against the description its author wrote is a separate, non-deterministic consumer of beholder's output, not part of this binary.
