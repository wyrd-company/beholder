---
docs: true
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
