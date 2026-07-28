# Resolver accuracy

Beholder's reference graph is produced by a heuristic resolver: it reads
imports, module paths, qualifiers and identifier occurrences, and it has no type
information. A graph built that way is useful only if its error bar is known, so
it is measured rather than asserted.

Measured figures live in `crates/beholder/accuracy.toml`. Every edge in
`edges.jsonl` carries the language's observed range and its own resolution
confidence.

## What is compared

An external resolved index supplies ground truth. Beholder neither produces nor
requires one.

```sh
# Generate and compare in one operation, so the two cannot describe
# different code.
cargo run --release -p beholder-oracle -- /path/to/repo --generate
```

Both sides reduce to the same unit: a **directed pair of beholder symbol ids**,
`(referring symbol, referenced symbol)`. Beholder's edges are deduplicated to
distinct pairs — one pair may carry a call and a type mention and counts once.
Edge kind is not compared: the question is whether the dependency was found, not
how it was labelled.

```text
precision = |beholder ∩ oracle| / |beholder|
recall    = |beholder ∩ oracle| / |oracle|
```

### Projecting SCIP onto beholder's symbols

Every SCIP occurrence is a (file, range, symbol) triple. Ranges are compared as
(line, column) pairs, because two declarations can share a line and a nested one
sits inside another's span.

- A **definition** occurrence contributes only when its range is exactly some
  beholder symbol's own declared name.
- A **reference** occurrence contributes only when it falls inside some beholder
  symbol's span *and* names a symbol that cleared the same bar.

Requiring an exact declaration match is what keeps the measurement honest.
Mapping a definition to whatever symbol merely *contains* it would fold fields,
enum variants, associated constants, statics, macros and nested declarations
onto their enclosing function or type. References to things beholder does not
model would then become apparently-correct edges, flattering precision and
recall together.

### The exclusion set

Excluded from **both** sides, counted separately so the set is auditable.

| Reason | `tagver` | `intentional` |
| --- | ---: | ---: |
| External dependency — referent defined outside the analyzed files | 1564 | 12013 |
| Local binding — a SCIP `local N`, never a symbol | 671 | 6461 |
| Unmodeled referent — a field, variant, associated constant, static or macro | 392 | 3398 |
| No modeled referrer — an `impl` header or `use`, outside any symbol | 339 | 1240 |
| Recursion — neither side emits a self-edge | 0 | 11 |
| Malformed range | 0 | 0 |
| **Reference occurrences seen** | **3293** | **26155** |

The exclusions dwarf the compared set. That is expected — most of what a
codebase mentions is standard library, dependency, or something finer-grained
than a function or type — but it means these figures describe beholder's graph
over the symbols beholder models, and nothing wider.

SCIP columns are read in the encoding the index declares; both measurements
below used UTF-8. The oracle warns when the working tree is dirty and when a
SCIP document names a file the analysis never saw.

## Pair-level results

Measured against `rust-analyzer scip 1.97.1`.

| Repository | Beholder edges | High confidence | Oracle edges | Agreed | Precision | Recall |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `tagver` | 286 | 102 | 257 | 249 | 0.871 | 0.969 |
| `intentional` | 2155 | 802 | 1945 | 1699 | 0.788 | 0.874 |

`accuracy.toml` records both repositories and the observed range. It does not
record a pooled figure: the spread is structural — `tagver` is small and mostly
free functions, `intentional` leans on traits and generic containers — and a
pool dominated by the larger repository would read as a per-edge probability
that it is not.

## Fan-in, which is what ranking consumes

Pair-level precision and recall say whether individual edges are right. They do
not say whether the *number* ranking consumes is right, because errors can
cancel or concentrate. Three bases were measured against oracle fan-in over
every beholder symbol.

Top-k overlap is a range, because fan-in is heavily tied and which symbols
occupy a top-k list is often undetermined. The lower bound counts only symbols
that must be in any top k; the upper bound counts every symbol that could be.

| Repository | Basis | Spearman | Top-10 | Top-25 | Mean abs. error | Mean rel. error |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `tagver` | raw | **0.954** | 0.70–0.90 | 0.84–1.00 | 0.39 | 0.16 |
| `tagver` | high confidence only | 0.548 | 0.50–0.50 | 0.40–1.00 | 1.59 | 0.74 |
| `tagver` | high + capped low | 0.951 | 0.50–0.60 | 0.84–1.00 | 0.80 | 0.23 |
| `intentional` | raw | 0.757 | 0.50–0.50 | 0.72–0.76 | 1.32 | 0.37 |
| `intentional` | high confidence only | 0.405 | 0.60–0.70 | 0.40–0.44 | 2.24 | 0.88 |
| `intentional` | high + capped low | **0.763** | 0.60–0.70 | 0.40–0.44 | **1.18** | **0.26** |

"High confidence" means the source states where the reference points: an
explicit import, a path qualifier, or `Self`. "Capped low" adds low-confidence
edges but stops any one target absorbing more than five of them.

### Stratified by symbol class, raw basis

| Class | `tagver` n / Spearman | `intentional` n / Spearman |
| --- | --- | --- |
| free function | 65 / 0.925 | 243 / 0.953 |
| inherent method | 24 / 0.983 | 181 / 0.716 |
| trait method | 9 / 1.000 | 12 / **−0.363** |
| type | 17 / 0.935 | 91 / 0.662 |

Trait methods on `intentional` are anti-correlated. With twelve symbols that is
a small sample, but it is the one stratum where fan-in actively misleads.

### Concentration of error

Overstatement is extremely concentrated. On `tagver` the ten worst symbols
absorb 100% of it; on `intentional`, 84%. The worst cases are exactly the
low-confidence method-name collisions:

| Overstated by | Symbol | What the calls really were |
| ---: | --- | --- |
| +111 | `JsonParser::expect` | `Option::expect`, `Result::expect` |
| +78 | `<Bump as FromStr>::Err` | the `Err` variant of `Result` |
| +46 | `DiscoveryConfig::is_empty` | `Vec::is_empty`, `str::is_empty` |

Capping low-confidence contributions cuts `intentional`'s concentration from 84%
to 56% and its mean relative error from 0.37 to 0.26 while leaving rank
correlation intact.

## Error classes

### Method dispatch without receiver types

The dominant precision failure and inherent to a type-free resolver. A method
call supplies a name and nothing else; when exactly one project symbol carries
that name, beholder resolves to it even though the real receiver is a standard
library type. Constraining method calls to targets declared inside something
already stops them landing on free functions. Nothing short of type information
stops them landing on the wrong type's method.

### External-crate name collisions

An explicit import from a dependency does not stop the name resolving to a
project symbol of the same name: `use anyhow::Result` in a project that also
defines `Result`. Beholder cannot tell `intentional_core::Config` — a workspace
crate, a real project symbol — from `anyhow::Result`. Both are import paths
whose leading segment names no file-derived module, and separating them needs
the build manifest, deliberately outside what tier 2 reads.

### Type references over-attributed

On `tagver` the largest remaining false positives are type mentions: `Version`
(+15) and `Result` (+11). Beholder counts a distinct referrer for mentions the
oracle attributes elsewhere or does not model.

### `Self` resolution is outermost-first

`Self` resolves against the referring symbol's own scope by scanning scope words
left to right and taking the first that names exactly one symbol. For a trait
impl the scope reads `<Type as Trait>`, so `Type` is scanned before `Trait` and
wins — correct, but only because of how that segment is spelled. Under module
nesting the order is wrong: in `mod ledger { impl Tally { … } }` the scope reads
`ledger::Tally`, and a type named `ledger` anywhere in the project captures
`Self` before `Tally` does. A scope word naming several symbols is skipped
rather than treated as ambiguous, so the scan continues outward past it.
`self_under_module_nesting_takes_the_outermost_naming_scope` pins this so a
future change to innermost-first is a deliberate one.

## Micro-decision: fan-in is confidence-aware, not confidence-filtered

**Context.** The Definition of Done ranks risk by complexity delta weighted by
fan-in. Measurement showed raw fan-in is inflated by low-confidence method-name
collisions, so the question was how to make that weight trustworthy.

**Decision.** `Degree` reports `fan_in` and `fan_in_high_confidence` separately,
and every edge carries its own `confidence`. Ranking will combine them rather
than filter to high confidence alone.

**Why not high-confidence only.** It was measured and it is worse. High
confidence covers 102 of 286 edges on `tagver` and 802 of 2155 on `intentional`,
and most of what it discards is correct. Rank correlation against the oracle
falls from 0.954 to 0.548 and from 0.757 to 0.405. Filtering buys precision on
individual edges at the cost of the ordering, which is the thing ranking needs.

**What the evidence supports instead.** Keep every edge and damp the
low-confidence contribution. Capping at five per target preserved rank
correlation (0.951 and 0.763) while cutting mean relative error and roughly
halving the concentration of overstatement.

The shipped damping is `high + sqrt(low)` rather than that cap. A cap makes two
symbols either side of it indistinguishable and moves the ordering
discontinuously as code changes; a square root keeps every low-confidence
reference contributing while stopping a hundred of them from dominating. Read
retroactively over fourteen merged changes it removes exactly the promotions
this measurement predicts are wrong — see `FanInBasis` and the gate 5 judgment
recorded on `crate::risk`.

**Consequence.** `fan_in_high_confidence` is a ranking input, not a better
fan-in. Presenting it as the graph's answer to "what depends on this" would
understate every dependency the source did not spell out.

## Micro-decision: damped is the default basis

**Context.** Two fan-in bases survived measurement. Raw reconstructs the
oracle's whole ordering best; damped removes the promotions the measurement
predicts are wrong. Statistics alone could not choose between them, so both were
run over fourteen already-merged changes.

**Decision.** `beholder report` defaults to the damped basis. Raw remains
selectable with `--basis raw`.

**Why.** Where the two disagree, damping is right. A three-line predicate ranked
second in its whole change under raw, on 27 references none of which the source
stated. Across the corpus, damping cut such promotions from three to one at no
observed cost, and no report got worse. Raw's advantage is in reconstructing the
full distribution; a report only needs the top of it not to be wrong.

**Why raw stays.** It is the benchmark the resolver is measured against, and
keeping it selectable means a future change to the damping function can be
compared rather than asserted. Both fan-ins remain in `edges.jsonl` and in every
report either way, so a consumer can always audit the weighting.

**Evidence.** `docs/gate5-evaluation.toml`.

## Reproducing

```sh
cargo run --release -p beholder-oracle -- /workspaces/tools/tagver --generate
```

`--index` uses an existing index instead of generating one. `--json` emits the
full comparison including every disagreeing pair and all fan-in statistics.
`--revision` measures a committed tree instead of the working tree.
