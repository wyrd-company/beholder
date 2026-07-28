# Resolver accuracy

Beholder's reference graph is produced by a heuristic resolver: it reads
imports, module paths, qualifiers and identifier occurrences, and it has no type
information. A graph built that way is useful only if its error bar is known, so
it is measured rather than asserted.

The measured figures live in `crates/beholder/accuracy.toml` and are carried on
every edge in `edges.jsonl`.

## What is compared

An external resolved index supplies ground truth. Beholder neither produces nor
requires one; `beholder-oracle` reads an index that already exists.

```sh
rust-analyzer scip /path/to/repo
beholder-oracle /path/to/repo --index /path/to/repo/index.scip
```

Both sides are reduced to the same unit: a **directed pair of beholder symbol
ids**, `(referring symbol, referenced symbol)`.

- Beholder's edges are taken from `graph::build`, deduplicated to distinct pairs.
  One pair may carry several edge kinds — a call and a type mention between the
  same two symbols — and counts once.
- The oracle's edges are projected onto beholder's symbols. Every SCIP occurrence
  is a (file, range, symbol) triple. A definition occurrence maps its SCIP symbol
  to the beholder symbol whose line range contains it. A reference occurrence
  maps the same way to the symbol it was written inside. An edge exists when both
  ends land on beholder symbols.

An edge matches when both ends agree. Edge kind is not compared: the question is
whether beholder found the dependency, not how it labelled it.

```text
precision = |beholder ∩ oracle| / |beholder|
recall    = |beholder ∩ oracle| / |oracle|
```

## What is excluded

Excluded from **both** sides, so neither is credited or penalised:

- **References with an endpoint beholder does not model.** Constants, statics,
  macros, closures, local bindings, struct fields and enum variants are not
  symbols in beholder's index. On the corpus below this is the large majority of
  SCIP reference occurrences — 2008 of 2622 in `tagver`, 13387 of 19694 in
  `intentional`.
- **References into dependencies.** A symbol defined outside the analyzed file
  set has no beholder id, so no edge is expected in either direction.
- **SCIP `local N` symbols**, which are function-local bindings.
- **Recursion.** Neither side emits an edge from a symbol to itself.

Excluded from the oracle only:

- **Occurrences outside any symbol.** An `impl` header or a `use` declaration
  sits at file scope, so there is no referring symbol to attach an edge to.

## Results

Measured against `rust-analyzer scip 1.97.1` at each repository's `HEAD`.

| Repository | Beholder edges | Oracle edges | Agreed | Precision | Recall |
| --- | ---: | ---: | ---: | ---: | ---: |
| `tagver` | 286 | 288 | 267 | 0.934 | 0.927 |
| `intentional` | 2155 | 2312 | 1725 | 0.800 | 0.746 |
| **pooled** | **2441** | **2600** | **1992** | **0.816** | **0.766** |

`accuracy.toml` records the pooled figure, because that is the number a consumer
of an arbitrary repository should expect. The spread between the two
repositories is itself informative: `tagver` is small and mostly free functions,
`intentional` is larger and leans on traits and generic containers, and accuracy
tracks that difference.

## Error classes

### Method dispatch without receiver types

The dominant precision failure, and inherent to a type-free resolver. A method
call gives beholder a name and nothing else. When exactly one symbol in the
project carries that name, beholder resolves to it — even when the real receiver
is a type from the standard library or a dependency.

| Spurious edges | Target | What the call really was |
| ---: | --- | --- |
| 111 | `JsonParser::expect` | `Option::expect`, `Result::expect` |
| 78 | `<Bump as FromStr>::Err` | the `Err` variant of `Result` |
| 46 | `DiscoveryConfig::is_empty` | `Vec::is_empty`, `str::is_empty` |
| 36 | `JsonNode::find` | `Iterator::find` |
| 37 | `<Bump as FromStr>::from_str` | `from_str` on other types |

Every one of these is a project symbol whose name collides with a ubiquitous
standard-library method. Constraining a method call to targets that are declared
inside something already stops it landing on free functions; nothing short of
type information will stop it landing on the wrong type's method.

### External-crate name collisions

An explicit import of a name from a dependency does not stop that name
resolving to a project symbol of the same name. `use anyhow::Result` in a
project that also defines `Result` produces edges to the project's alias.

Beholder cannot currently tell `intentional_core::Config` — a workspace crate,
and a real project symbol — from `anyhow::Result` — a dependency. Both are
import paths whose leading segment names no file-derived module. Distinguishing
them needs the build manifest, which is deliberately outside what tier 2 reads.

### Referrer attribution mismatches

A smaller class where both sides agree an edge exists but disagree about which
symbol it came from, so it appears as a false positive and a false negative at
once. On `intentional`, `Projection` (15 spurious, 16 missed) and `git` (13 and
13) are the visible cases.

### Remaining recall gaps

After resolving `Self` against the referring symbol's own scope, and recording a
path's qualifier as a reference in its own right — `Error::io` refers to `Error`
as well as to `io` — the largest remaining misses are references to types
(`ReleaseUnitConfig` 46, `DiscoveryCandidate` 35, `Config` 26). These are
partially resolved already, so the gap is in specific syntactic positions rather
than in the type being unreachable.

## Reproducing

```sh
rust-analyzer scip /workspaces/tools/tagver
cargo run --release -p beholder-oracle -- /workspaces/tools/tagver \
  --index /workspaces/tools/tagver/index.scip
```

`--json` emits the full comparison including every disagreeing pair. `--revision`
measures a committed tree instead of the working tree.
