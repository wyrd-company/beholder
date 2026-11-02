# Authoring semantic-oracle datasets

A semantic-oracle dataset is the `semantic-oracle.yml` file at a fixture
project's root. It states what the project's source means, by the rules of
`docs/specifications/semantic-oracle-model.yml` and the profile of the
project's language. This page describes the commands that capture locators
and check a dataset, and the loop an author follows with them.

The commands work offline. They read the project's files, the repository
index and the specifications that are built into the binary. They never run
an indexer or read index output: a dataset states what the author derived
from source, build configuration and the language specification, and the
tools only compute coordinates and digests and check the result.

## The fixture corpus

The fixture projects and their datasets live in the repository
`wyrd-company/beholder-fixtures`, linked as the submodule `fixtures/`. A
normal clone does not check it out; `task fixtures:init` does, at the
commit this repository pins. Its projects are
under `fixtures/fixtures/<language>/projects/<project>/`.

## Commands

The binary is `semantic-oracle`, from the `beholder-semantic-oracle-cli`
crate. Each command has a Taskfile task that runs it from the directory you
call `task` in.

| Task | Command | Does |
| --- | --- | --- |
| `task semantic-oracle:inventory PROJECT=<dir>` | `semantic-oracle inventory <dir>` | Prints the `documents` block for every tracked file, with current digests. |
| `task semantic-oracle:locate PROJECT=<dir> FILE=<path> TEXT=<text> [LINE=<n>] [COLUMN=<n>]` | `semantic-oracle locate <path> <text> [--line <n>] [--column <n>] --project <dir>` | Prints the locator of one span. |
| `task semantic-oracle:validate PROJECT=<dir>` | `semantic-oracle validate <dir>` | Validates the dataset against the model, its profile and the project's files. |
| `task semantic-oracle:audit PROJECT=<dir>` | `semantic-oracle audit <dir>` | Validates, then audits the dataset against its profile. |
| `task test:oracle-schema` | | Tests the schemas, the validator and the commands. |

`PROJECT` defaults to the current directory. `validate` and `audit` also
accept the path of the dataset file itself.

### inventory

```console
$ semantic-oracle inventory fixtures/tally
documents:
  - path: Cargo.toml
    sha256: 3c9d…
    disposition: configuration
  - path: NOTES.md
    sha256: 81f2…
    # new: needs a disposition
  - path: src/lib.rs
    sha256: 5a07…
    disposition: indexed
    origin: source
    members:
      - unit: tally-lib
```

The tracked files are the entries of the git index under the project root,
except the dataset file. Digests and locators describe the bytes in the
working tree, so a project's files must check out unconverted: mark the
project `-text` in `.gitattributes` so that no platform rewrites its line
endings. An entry the dataset already has keeps its
disposition, reason, origin and members. A new entry has none, so the
dataset fails validation until you classify it. Untracked files are not
documents.

### locate

```console
$ semantic-oracle locate src/lib.rs Counter --line 3 --project fixtures/tally
{path: src/lib.rs, start: [3, 18], end: [3, 25], text: Counter}
$ semantic-oracle locate src/lib.rs Counter --project fixtures/tally
semantic-oracle: src/lib.rs: the text occurs at [1, 12], [3, 18], [4, 5]
```

The text must occur exactly once where `--line` and `--column` look. When it
occurs more than once, the command lists the positions and exits 1; repeat it
with the line, and the column if the line holds the text twice. Coordinates
follow the model: lines end at LF, CR LF or CR, and columns count Unicode
scalar values from 1, so the output is correct for any file encoding and
line ending the model admits.

### validate and audit

Output is JSON Lines: one object per finding, then one summary object.
Here a comment line was added above line 4 of `src/lib.rs`:

```console
$ semantic-oracle validate fixtures/tally
{"at":"/documents/2/sha256","message":"the file's SHA-256 is 771c7f80966f3c9da50078f3fc1e93a88a4323f2ca2264e9d730bc16da0174ec; the 4 locator(s) in it are stale","rule":"model:locator-stale","severity":"error"}
{"summary":{"errors":1,"warnings":0}}
```

After `inventory` refreshes the digest, each moved locator is reported with
the positions where its text now occurs:

```console
$ semantic-oracle validate fixtures/tally
{"at":"/references/1/at","message":"stale: the text at these coordinates is \"// the \"; the stored text occurs at [1, 12], [3, 18], [5, 5]","rule":"model:locator-stale","severity":"error"}
{"summary":{"errors":1,"warnings":0}}
```

| Member | Content |
| --- | --- |
| `severity` | `error` fails the dataset; `warning` does not. |
| `rule` | The rule the finding enforces: `schema:<keyword>` for the dataset schema, `model:<rule>` for the model, `<language>:<rule>` for a profile. |
| `at` | The JSON Pointer of the dataset location. |
| `message` | One line. |

Findings are sorted by location, then rule, so the same dataset and files
give the same output. The exit status is 0 when there is no error, 1 when
there is at least one, and 2 when the command cannot run, such as when the
dataset file is missing or the project is not in a git working tree.

A schema error stops the later checks, and so does a dataset of another
model or profile version.

## The authoring loop

1. Run `inventory` and put its output in the dataset. Give every document a
   disposition by the profile's file-class table, and give each indexed
   document its origin and its unit and context members.
2. Declare the contexts the project's `build` and `test` tasks define, then
   the units and distributions, by the profile's rules.
3. Write entities, references, relations, visibility, public API members and
   extension facts. Use `locate` for every site; type the meaning, let the
   tool type the coordinates.
4. Write the signature of each public API member under `public-api`
   `signatures`, as the next section describes.
5. Run `validate` and fix every error. Run it after each batch of facts, so
   that a mistake is found near where it was made.
6. Run `audit`. Fix every error, then read each warning: a `model:coverage`
   warning names an indexed document that holds no fact site in a context,
   which is either an empty file or a file whose facts are still missing.
7. When the source changes, run `inventory` again to refresh the digests,
   then `validate`. Every locator whose text moved is reported with the
   positions where its text now occurs; recapture each with `locate`.

The dataset is reviewed independently once it validates and audits clean.
The tools check structure and consistency; a reviewer confirms each fact
against source, build configuration and the language specification.

## Writing signatures

The public API of a distribution is `public-api` `members`, one membership for
each member, and `signatures`, the facts a consumer depends on beyond a member's
name. A member has a record when it has at least one slot:

```yaml
public-api:
  members:
    - {distribution: tally, entity: tally.make, paths: ['tally::make'], access: all}
  signatures:
    - entity: tally.make
      returns: "@{tally.Counter}"
```

The model's Signatures section states the slots, and the profile's Public API
section states which slots each `language-kind` has, its modes and modifiers, and
its canonical spelling. Write each type in the language's own syntax with `@{id}`
for each name that denotes an entity. Write each value as one string, quoted where
YAML would read another type, such as `"null"`, `"True"` or `"1"`.

| Step | Who | Writes |
| --- | --- | --- |
| Declared slots | The author, from source and the language specification | Types, modes, modifiers, defaults and values |
| Inferred slots with a specification | The author, applying the specification's rule: Go and Dart | `{inferred: <type text>}` |
| Inferred slots the compiler defines | `tsc --declaration`, outside the project tree: TypeScript | `{inferred: <type text>}`, in canonical spelling |
| Check | A compiler report from a per-language script | Findings to settle against the language's rule |

Do not read a compiler report or a compiler draft of a declared slot while
writing it, so that a disagreement shows. Settle each disagreement against the
language's rule, and never by copying the compiler's value. SCIP, Beholder and
every evaluated indexer or language service are never an input.

## What the checks cover

`validate` applies the model's validation criteria 1 to 7, and checks a
signature's schema and ids:

| Rule | Checks |
| --- | --- |
| `model:version`, `model:profile-version`, `schema:*` | The model and profile versions, and the dataset schema of the profile. |
| `model:id-unique`, `model:id-reference` | Ids are unique across the dataset, and every id names a record of the required kind, including `scope` and `within` members and extension members. |
| `model:document-inventory`, `model:document-unique`, `model:document-digest`, `model:document-utf8` | One entry per tracked file and none for anything else, each digest equal to the file's SHA-256, and every indexed document valid UTF-8. |
| `model:locator-stale`, `model:locator-range`, `model:locator-order`, `model:locator-document`, `model:locator-empty` | Each locator's document is in the inventory, its coordinates lie in the document, end does not precede start, and its text equals the source. A digest that differs makes every locator in that document stale. |
| `model:locator-ambiguous` | A definition site listed twice for one entity, or two references with one site, `from` and role in a shared context. |
| `model:site-membership` | Each site is in an indexed document that is a member of the fact's unit in every context of the fact. Only a profile with an augmentation form, TypeScript, allows a site in another unit's document. |
| `model:context-declared`, `model:context-scope`, `model:context-partition`, `model:context-distinct` | Context ids are declared. A fact without `contexts` holds in every context its unit is built in, and listed contexts lie within those. A definition site's `contexts` lie within its entity's. The entities a fact names exist in each of its contexts. Context-variant lists partition their record's contexts, and contexts differ in a parameter. |
| `model:unit-key`, `model:unit-root` | Unit keys, by the profile's key members, name one unit in the dataset; a unit root is a module or namespace of that unit. |
| `model:effective-visibility` | An entity whose unit has no distribution has `external: none`. |
| `model:entity-identity`, `model:entity-parent`, `model:entity-origin` | Unit, path and kind identify one entity per context; parent chains end; generated entities outside generated documents have `anchor` and `generated-by`; entities defined in generated documents are generated. |
| `model:external-key`, `model:external-unused` | External keys are unique, and every external is a target of a reference, relation, extension fact or `@{id}` of a signature. |
| `schema:*`, `model:id-reference`, `model:context-*` on `public-api` `signatures` | A signature has the record form and vocabulary of its profile, names a project entity, and lists contexts where that entity exists. An external that a `@{id}` names counts as used. |
| `model:relation-duplicate`, `model:public-api-duplicate` | No relation or public API membership is stated twice in a context. |
| `model:public-api-access` | A member's `access` agrees with the entity's effective `external` access in every context. |
| `model:public-api-membership-recomputed` | The membership the profile's public API rule gives, recomputed in every context from `parent`, `exports`, visibility and the profile's entry facts, equals the `public-api` `members` entries: the same members, each with the same complete path list and access. A member may be an entity of another unit, or an external, that a re-export reaches. For Rust, `project` and `external` access are `all` exactly for entities with a consumer path; trait implementation items are exempt. |
| `model:extension-key`, `<language>:extension-target` | Extension keys, by the profile's key members, are unique per context; an extension member names a record of the kind the profile states. An extension fact without `contexts` holds where the unit of its first entity or unit member is built. |
| `<language>:sorted` | Lists the profile states are sorted are sorted. |

`audit` adds the mechanical part of criterion 9:

| Rule | Checks |
| --- | --- |
| `<language>:document-class` | Files of the profile language are `indexed`, `inactive` or `external`; other files are not `indexed`; known configuration files are `configuration` (a warning). The profiles' exceptions apply: a Python `setup.py` may be `configuration`, a TypeScript `.json` file may be `indexed`, and TypeScript `.js`, `.jsx`, `.mjs` and `.cjs` files are not checked, because whether they are source depends on `allowJs`. |
| `model:out-of-profile` | No entity declared inside a callable body, and no reference to a parameter or type parameter. |
| `model:coverage` | Every indexed document holds a fact site in every context it is compiled in (a warning). |
| `go:generated-origin` | A Go file with a generated-code header before its package clause has `origin: generated`. |
| `rust:visibility-path` | Every `pub(in P)` names a module entity. |

Not checked mechanically, and left to review: that each context derives
from a project command, that the inventory dispositions and members follow
the build, that the dataset is exhaustive for the required categories, the
model's `model:signature-*` rules and `profile:signature-compiler`, that
a TypeScript site in another unit's document is an augmentation, and the
inputs of public API recomputation: that each `exports` relation's
`visibility` is the binding's, and that each `typescript:entry` fact names
the entry module that the package configuration selects.
