# Changelog

Every released version has a section here, and the release workflow reads the
section matching the tag as that release's notes. A tag with no section does not
release.

## 0.1.0

### Breaking

- Analyze every repository file with tier 0 indentation density, with tier 1 symbol analysis for Rust, Go, and TypeScript. Report cognitive complexity, references, cohesion, and change-scoped risk through portable Git-backed reuse and JSONL, SARIF, and gitpr output. Rust reference edges carry a measured precision and recall error bar; Go and TypeScript report accuracy as not measured.

### Features

- Install beholder from crates.io, npm, the Homebrew tap, or platform archives.
- Report pull request risk through a GitHub Action with one updated comment, SARIF output, threshold-based silence, and shared index reuse across runs.

### Fixes

- Publish Beholder documentation on wyrd.foo with every release.
