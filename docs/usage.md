---
docs: true
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

This command reports local state. Transporting the ref between clones remains
an ordinary Git operation; see [Stored results](stored-results).
