---
product: beholder
title: Stored results
order: 5
---

Beholder stores generated analysis in `refs/beholder/index`, an ordinary custom
Git ref outside `refs/heads/*`. The results are not workspace files, and no
service or hosted artifact store is required.

## Transport the ref

Move the ref between clones with Git:

```bash
git fetch origin +refs/beholder/*:refs/beholder/*
git push origin +refs/beholder/*:refs/beholder/*
```

Beholder does not replace those commands. The leading `+` lets Git update a
non-branch ref; the Action's own index push is never forced. Concurrent writers
use compare-and-swap, and a rejected writer fetches the winning tip, rebuilds on
it, and retries.

## Index history

An index commit names the source commit it describes as its first parent and
the preceding index commit as its second parent. The first index commit has
only the source parent. Walking the second-parent chain gives the index history.

Its tree contains:

```text
meta.json                 fingerprint and source commit
phase1/<repo path>.json   one phase-one result per analyzed file
phase2.json               whole-set analysis
```

## Derived cache, not authority

Stored results are reusable work, not source of truth. Beholder checks the
source commit, schema and tool versions, configuration, language metadata,
paths, file set, and content hashes before reuse. Missing results are generated
and appended; stale or mismatched results are ignored and recomputed.

## Action workflows

The Beholder action compares a pull request head with its merge base, uploads
SARIF annotations, and maintains one sticky pull request comment.

```yaml
permissions:
  contents: read
  pull-requests: write
  security-events: write

jobs:
  report:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - id: beholder
        uses: wyrd-company/beholder@main
      - uses: github/codeql-action/upload-sarif@v3
        with:
          sarif_file: ${{ steps.beholder.outputs.sarif }}
```

`contents: read` lets the action fetch source and stored results,
`pull-requests: write` lets it update the sticky comment, and
`security-events: write` lets GitHub accept the SARIF file.

The action stays silent when no changed symbol crosses the risk threshold. If a
comment from an earlier push exists, it updates that comment so it no longer
describes stale code. Beholder reports findings but does not gate the build.

The action fetches stored results before analysis. Publishing is opt-in:
`publish-index` defaults to `false`. `mode` defaults to `report`. A repository with no stored index simply performs a normal
cold start.

For fork PRs, the token is read-only. Set `comment: false` and skip SARIF upload
when write permissions are unavailable; JSON and SARIF files are still generated.
Do not use `pull_request_target` to execute untrusted PR code with write access.

## Publish after main CI

The user chooses where publication runs. This workflow requires only
`contents: write`, so that it can fetch and push the index. It checks out the
successful main push SHA, rather than the workflow's default checkout SHA.

```yaml
name: Publish Beholder index
on:
  workflow_run:
    workflows: [CI]
    types: [completed]
    branches: [main]
permissions:
  contents: write
concurrency:
  group: beholder-index
  cancel-in-progress: false
jobs:
  index:
    if: >-
      github.event.workflow_run.conclusion == 'success' &&
      github.event.workflow_run.event == 'push' &&
      github.event.workflow_run.head_branch == 'main' &&
      github.event.workflow_run.head_repository.full_name == github.repository
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7
        with:
          ref: ${{ github.event.workflow_run.head_sha }}
          fetch-depth: 0
      - uses: wyrd-company/beholder@main
        with:
          mode: index
          publish-index: 'true'
```

`mode: index` runs `beholder index --store --use-store` on the checked-out commit.
It needs no `base` or `head`, creates no report or comment, and leaves report
outputs empty. Valid file results remain reusable when earlier CI runs failed
or were skipped. A repository without an index starts cold.

The action script owns `git push`. After a rejection it saves the local tip,
fetches the remote tip, and asks `beholder store --reconcile <saved-tip>` to
rebuild local index commits onto it. Beholder does not start a git-push subprocess.
