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

The leading `+` lets Git update a non-branch ref. To publish without
overwriting another writer, push the index without `+`:

```bash
git push origin refs/beholder/index:refs/beholder/index
```

Beholder can fetch the ref itself. It keeps local index commits on top of what
it fetched:

```bash
beholder store --fetch origin
```

Beholder does not push. Its fetch uses the repository's Git configuration, but
not URL-scoped `http.<url>.*` keys. If your credential is stored as
`http.<url>.extraHeader`, as `actions/checkout` stores its token, fetch with Git.

## Concurrent writers

Local ref updates use compare-and-swap. A rejected push means another writer
published first. Fetch their tip, rebuild your index commits on it, and push
again:

```bash
git fetch origin refs/beholder/index
beholder store --rebuild-onto "$(git rev-parse FETCH_HEAD)"
git push origin refs/beholder/index:refs/beholder/index
```

The Action does this for you. Its push is never forced, and only a rejected ref
starts a rebuild. Other push failures stop the publish step.

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
