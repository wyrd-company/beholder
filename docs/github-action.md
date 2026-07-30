---
docs: true
title: GitHub Action
order: 4
---

The Beholder action compares a pull request head with its merge base, uploads
SARIF annotations, and maintains one sticky pull request comment.

```yaml
permissions:
  contents: write
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

`contents: write` lets the action publish `refs/beholder/index`,
`pull-requests: write` lets it update the sticky comment, and
`security-events: write` lets GitHub accept the SARIF file.

The action stays silent when no changed symbol crosses the risk threshold. If a
comment from an earlier push exists, it updates that comment so it no longer
describes stale code. Beholder reports findings but does not gate the build.

The action fetches stored results before analysis and publishes newly generated
results afterward. A repository with no stored index simply performs a normal
cold start.
