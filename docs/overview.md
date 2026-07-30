---
docs: true
title: Overview
order: 1
---

Beholder is a structural index of a codebase. It answers two questions: what is
risky, and what depends on what. Metrics are views over that index, not an
aggregate grade or a build gate.

The index records symbols, nesting-weighted cognitive complexity, reference
relationships, and cohesion. Delta mode compares two revisions by stable symbol
identity, and risk ranking combines complexity change with fan-in to identify
the changes most worth reviewing.

## Analysis tiers

Every result declares the tier that produced it:

- **Tier 0 — indentation density.** Works for every file without a grammar.
- **Tier 1 — tree-sitter.** Adds symbol boundaries, line ranges, and cognitive
  complexity for Rust, Go, and TypeScript.
- **Tier 2 — resolved references.** Uses heuristic resolution from imports and
  identifier occurrences, or accepts an external resolved index.

Resolver accuracy has been measured for Rust. Go and TypeScript resolution have
not yet been measured, so Beholder reports their accuracy as unknown.

## What Beholder does not do

Beholder does not produce a maintainability grade, fail a build, or require an
index service. Its primary output is machine-readable, and review surfaces
render the same report for people.
