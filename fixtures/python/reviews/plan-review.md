VERDICT: REJECT

# Python fixture plan review

## Blocking findings

### P1-001 — Required annotation regime is unavailable in declared interpreter

- Plan anchors: `fixtures/python/plan/overview.md:L14`,
  `fixtures/python/plan/overview.md:L24`, and
  `fixtures/python/plan/projects/typed-toolkit.md:L35`.
- Checklist anchors: `reports/python/synthesis/checklist.md:L7`,
  `reports/python/synthesis/checklist.md:L141`, and
  `reports/python/synthesis/checklist.md:L328`.
- Ground-rule anchors: `fixtures/python/ground-rules.md:L22` and
  `fixtures/python/ground-rules.md:L34`.

The plan selects only CPython 3.14.6, assigns the eager and future-stringized
3.13 annotation regimes, and excludes the interpreter's 3.14 lazy regime as a
research gap. Local inspection confirms that un-futured annotations on this
interpreter are deferred through `__annotate__`; resolving an undefined
annotation raises when `__annotations__` is accessed rather than when the
function is defined. The selected context therefore cannot create the claimed
eager-evaluation variant. Record the eager regime as an unavailable historical-
interpreter variant or select an available context that can represent it.

### P1-002 — Packaging context disables assigned build-isolation coverage

- Plan anchors: `fixtures/python/plan/projects/distributable-suite.md:L37` and
  `fixtures/python/plan/projects/distributable-suite.md:L65`.
- Checklist anchor: `reports/python/synthesis/checklist.md:L226`.
- Ground-rule anchors: `fixtures/python/ground-rules.md:L34` and
  `fixtures/python/ground-rules.md:L89`.

`PY-CAN-PKG-002` requires a build with a constrained backend absent from the
caller environment. The assignment promises that isolated build, but the only
declared packaging context says build isolation is disabled. Offline operation
does not require disabling isolation when backend artifacts are vendored. Make
the declared context support the assigned isolated build without downloads.

### P1-003 — Absent ecosystem tools are still claimed as supported contexts

- Plan anchors: `fixtures/python/plan/overview.md:L46`,
  `fixtures/python/plan/projects/typed-toolkit.md:L49`,
  `fixtures/python/plan/projects/typed-toolkit.md:L51`,
  `fixtures/python/plan/projects/typed-toolkit.md:L58`,
  `fixtures/python/plan/projects/diagnostics-toolkit.md:L43`, and
  `fixtures/python/plan/projects/distributable-suite.md:L57`.
- Checklist anchors: `reports/python/synthesis/checklist.md:L155`,
  `reports/python/synthesis/checklist.md:L157`,
  `reports/python/synthesis/checklist.md:L246`, and
  `reports/python/synthesis/checklist.md:L291`.
- Ground-rule anchors: `fixtures/python/ground-rules.md:L25`,
  `fixtures/python/ground-rules.md:L37`, and
  `fixtures/python/ground-rules.md:L44`.

The overview declares type-checker, linter, and formatter contexts unavailable.
The briefs nevertheless assign conditional checker/tool rows using unnamed
configuration artifacts under only the default runtime. Those checklist rows
require a fixed or selected checker/tool and version or a per-tool schema. An
unnamed configuration cannot establish those contracts. Name a locally
available selected tool context and version, or account for these rows or
variants as unavailable conditional contexts.

### P1-004 — Third-party build backend is not named

- Plan anchor: `fixtures/python/plan/projects/distributable-suite.md:L72`.
- Ground-rule anchors: `fixtures/python/ground-rules.md:L77` and
  `fixtures/python/ground-rules.md:L82`.

The only dependency declaration says one Python Packaging Authority (PyPA)
backend will be chosen and gives setuptools or hatchling as examples. The
ground rules require the reviewed brief to name every third-party distribution;
the implementor may pin and vendor the named dependency, but may not select it
after plan review. Name the backend distribution while retaining the stated
pinning, provenance, licensing, and offline-artifact requirements.

## Validation

- Canonical accounting: 216 identifiers equal 206 unique primary assignments
  plus the 10 listed exceptions. Overview and project-brief ownership agree.
- Project boundaries: 14 distinct exclusive directories; no cross-project
  dependency is declared.
- Planned tests: each planned test names checklist coverage created by test
  source. Other briefs record `None`.
- Source-design boundary: no blocking source-file, nested-layout, declaration,
  signature, example, scenario, code, pseudocode, call-sequence, validator, or
  implementation-step prescription found.
- Generated-source policy: committed inputs and outputs are required where
  generation creates coverage; required validation does not regenerate source.
- Installed toolchain: CPython 3.14.6, standard Global Interpreter Lock (GIL)
  build, GCC 15.2, CPython headers, pip 26.1.2, wheel 0.47.0, uv 0.11.21, and
  `spawn`/`fork`/`forkserver` are present. No selected checker, linter,
  formatter, pytest, Cython, documentation builder, or freezer command was
  found.
- Markdown lint: clean for the bounded planning and review artifacts with
  MD041 disabled for this report's mandated verdict first line.
