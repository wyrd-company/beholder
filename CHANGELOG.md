# Changelog

Every released version has a section here, and the release workflow reads the
section matching the tag as that release's notes. A tag with no section does not
release.

## 0.1.0

First version.

- Symbol-level index for Rust, Go and TypeScript, expressed as tree-sitter
  queries and a language table rather than per-language code.
- Cognitive complexity per symbol, and tier 0 indentation density for every file
  the walk visits.
- A heuristic reference graph with fan-in and fan-out kept separate, measured
  against `rust-analyzer scip` and carrying that error bar on every edge.
- Delta mode: any two revisions in, changed symbols out, matched by an identity
  that survives reformatting.
- Risk ranking by complexity delta weighted by fan-in, ranked as a percentile
  within each language and merged on percentile alone.
- Review surfaces: a GitHub Action posting one sticky comment, SARIF for inline
  annotations, and the same report against a local gitpr snapshot.
- Generated results stored in the `refs/beholder/index` git ref, portable
  through ordinary fetch and push.
