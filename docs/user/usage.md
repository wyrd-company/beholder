---
product: beholder
title: Command usage
order: 3
---

Run Beholder inside a Git working tree. Use `beholder <command> --help` for the
complete options accepted by a command.

## `index`

Analyze the working tree and write `symbols.jsonl`, `files.jsonl`, and
`edges.jsonl`:

```bash
beholder index
```

Choose another output directory with `--out-dir`. Add `--use-store` to reuse
valid stored phase-one results, or `--store` to write the completed analysis to
Beholder's custom Git ref. Storing requires a clean working tree because the
result is filed against `HEAD`.

## `delta`

Compare two revisions and emit changed symbols as JSON Lines:

```bash
beholder delta main HEAD
```

Add `--use-store` to reuse and append valid stored analysis.

## `report`

Rank the symbols touched between two revisions, with the riskiest first:

```bash
beholder report main HEAD
```

The default output is text. Use `--format json` to produce the
surface-independent report, `--threshold` to choose the minimum percentile to
surface, and `--basis raw` to audit raw rather than damped fan-in weighting.

One analysis can feed multiple review surfaces:

```bash
beholder report main HEAD --format json > report.json
beholder render --from report.json --format sarif > beholder.sarif
beholder render --from report.json --format markdown > comment.md
```

## `store`

Inspect the local stored-index ref and its recent history:

```bash
beholder store
beholder store --limit 10
```

Fetch the ref from a remote, keeping local index commits on top of it:

```bash
beholder store --fetch origin
```

After a rejected push, rebuild local index commits on the tip that won:

```bash
git fetch --refmap= origin refs/beholder/index
beholder store --rebuild-onto "$(git rev-parse FETCH_HEAD)"
```

Beholder does not push. Publish with `git push`; see
[Stored results](stored-results).

## Select files for indexing

Repeat `--only` to select exact repository-relative files or directory subtrees:

```sh
beholder index --only src/lib --only tests/example.rs --out-dir /tmp/beholder-selected
jq -r '[.density, .path] | @tsv' /tmp/beholder-selected/files.jsonl |
  sort -n --reverse | awk '$1 >= 50'
```

This is the equivalent of `complexity --only <prefix> | sort -n --reverse |
awk '$1 >= 50'`: use the vendored standard `density` score, not cognitive
complexity or risk percentile. JSONL remains sorted by repository-relative path.

Selectors are relative to the repository root, including when invoked from a
subdirectory. `./`, repeated separators, and trailing slashes are normalized.
Absolute paths, backslashes, `..`, empty selectors, and root-only selectors are
errors. Matching is case-sensitive and uses complete path components:
`src/lib` matches that file or its descendants, never `src/library`.
Selectors form a union; no match succeeds with empty output. Without `--only`,
indexing retains its existing scope. Git ignores, infrastructure exclusions,
and configured exclusions (including `only_paths`) still apply.

`--only` cannot be combined with `--store`. Selected analysis is local JSONL;
it cannot replace the full stored index. Selection is part of the configuration
fingerprint, so `--use-store` only reuses results with the same selection.
Library callers using the same configuration get identical selection in revision
and worktree reads. Whole-set graph edges, fan counts, percentiles, and risk are
computed over the selected set; references outside it can be unresolved. Density
remains a per-file score independent of the selected set.
