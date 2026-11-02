# Beholder

Beholder is a structural index of a codebase that answers two questions: what
is risky, and what depends on what. It owns code analysis in the Wyrd Company
toolkit.

## Who Beholder is for

Weigh every design decision against all of these. The CLI is one consumer of
the `beholder` crate, not the only one.

| Consumer | State | What it needs from Beholder |
| --- | --- | --- |
| Beholder CLI and GitHub Action | Current | Complexity and risk reports that understand code, beyond indent and line counts. |
| Refinery | Needed | Relationship graphs between code, so that users do not hand-author every relationship. |
| Intentional | Needed | A programmatic way to validate that a change is breaking, from the public API. An author still sets major, minor or patch. |
| Dependency analysis | Future | The exact call stacks exercised in the libraries a project uses, so that a version upgrade or a CVE can be evaluated before acting. |

## Vocabulary

| Term | Meaning |
| --- | --- |
| tier | The depth that produced a result: 0 indentation density, 1 tree-sitter symbols and complexity, 2 resolved references. Every result declares its tier. |
| phase 1 | A pure function of one file's repo-relative path and content. |
| phase 2 | A function of the complete phase 1 output for every file in scope. |
| public API | The members of a distribution in a context and the signature of each member. |
| public API member | An entity that an external consumer can name. |
| signature | The facts about a member's declaration that a consumer depends on, beyond its name. |
| fixture project | A project in `wyrd-company/beholder-fixtures`, the `fixtures/` submodule. |
| semantic-oracle dataset | A fixture project's `semantic-oracle.yml`: the facts a correct indexer finds in it. |
| contract | `docs/specifications/semantic-oracle-model.yml`, the language profiles beside it, and their schemas. |
| pilot | A dataset used to calibrate the contract. |
| evaluator | The comparison of an indexer's output with the datasets. |

The contract and `docs/technical-designs/semantic-oracle-signatures.yml` hold
the full terms.

## Commands

`task --list` shows every task. `task ci` is what CI runs.

- `task test:oracle-schema` tests the contract's schemas and tools.
- `task fixtures:init` fetches the fixture corpus. A normal clone does not.
- `task semantic-oracle:validate PROJECT=<dir>` and
  `task semantic-oracle:audit PROJECT=<dir>` check a dataset.

## Conventions an agent gets wrong

- Phase 1 never depends on machine-local state or on another file, so that one
  file's result is addressable by content.
- Beholder degrades across tiers, never silently.
- Language support is data: query files plus a table entry. A language gets no
  code path of its own. `docs/adding-a-language.md` is the procedure.
- The CLI crate holds CLI concerns only. Behavior goes in the library.
- Oracle facts come from source, build configuration and language
  specifications. They never come from SCIP, Beholder or another indexer. The
  signature design states the only use of a compiler.
- Do not change a fixture project's source to make validation pass.
- The `beholder-semantic-oracle` crates are unpublished. They are not release
  units and take no intent.

## Do not touch

- `CHANGELOG.md`. Intentional writes it. A pull request with a user-facing
  change adds an intent with `intentional add`.
- Tags and manifest versions. Intentional owns them, without a `v`.

## Where things are

| Path | Contents |
| --- | --- |
| `crates/beholder` | The library: index, analysis, storage. |
| `crates/beholder-cli` | The CLI. |
| `crates/beholder-oracle` | Measures resolver accuracy against an external reference index. Unrelated to the semantic-oracle crates. |
| `crates/beholder-semantic-oracle`, `-cli` | Dataset validation and authoring tools. |
| `docs/user/` | Public documentation, published to wyrd.foo. |
| `docs/specifications/`, `docs/technical-designs/` | Refinery artifacts. |
| `fixtures/` | Submodule, initialized on demand. |
