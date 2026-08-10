VERDICT: REJECT

# Python fixture plan verification

## Blocking finding

### P1-004 — Hook-returned dependencies remain unnamed and unpinned

- Plan anchor:
  `fixtures/python/plan/projects/distributable-suite.md:L74-L80`.
- Ground-rule anchors: `fixtures/python/ground-rules.md:L77-L80` and
  `fixtures/python/ground-rules.md:L82-L87`.

The repair names and pins the backend itself as `setuptools==84.0.0`, with
Python Package Index provenance, license, notice, wheel, and offline-wheelhouse
requirements. It then authorizes the implementor to acquire and vendor “any
build requirements” returned by the backend hooks without naming or pinning
those distributions in the reviewed brief. Those are third-party dependencies,
and the ground rules permit network access only for dependencies named by the
reviewed plan. Local toolchain inspection cannot close the set because
setuptools is neither installed nor cached. P1-004 therefore remains blocking.

The sole repair pass is consumed. Continued rejection requires human
reassessment.

## Prior-finding verification

- **P1-001 — Addressed.**
  `fixtures/python/plan/projects/typed-toolkit.md:L39` and
  `fixtures/python/plan/overview.md:L134-L139` now classify the eager,
  definition-time annotation regime as an unavailable historical-interpreter
  variant. The future-stringized regime remains assigned, and deferred Python
  Enhancement Proposal (PEP) 649/749 details remain research gap
  `PY-GAP-001`.
- **P1-002 — Addressed.**
  `fixtures/python/plan/projects/distributable-suite.md:L37` and
  `fixtures/python/plan/projects/distributable-suite.md:L65-L68` now keep PEP
  517 isolation enabled and supply its backend offline through a local
  wheelhouse.
- **P1-003 — Addressed.** `fixtures/python/plan/overview.md:L121-L124` records
  `PY-CAN-TYPE-015`, `PY-CAN-TYPE-017`, `PY-CAN-DIAG-008`, and
  `PY-CAN-PKG-022` as unavailable conditional contexts. Their briefs no longer
  claim unsupported checker, linter, formatter, or per-tool-schema contexts.
- **P1-004 — Not addressed.** Backend identity is fixed, but the dependency
  clause above still delegates selection of additional third-party build
  requirements to the implementor.

## Validation

- Canonical accounting: 216 identifiers equal 202 unique primary assignments
  plus 14 unique exceptions. The sets are disjoint and complete.
- Overview and project briefs: all 202 ownership entries agree exactly.
- Project independence: 14 distinct exclusive directories; no cross-project
  dependency was introduced.
- Planned tests: the standard-library test source still names
  `PY-CAN-DIAG-009`, `PY-CAN-DIAG-013`, `PY-CAN-DIAG-014`, and the test-runner
  main-identity variant of `PY-CAN-INT-007`. Other projects record `None`.
- Source-design boundary: the repair adds no fixture source file, nested source
  layout, declaration, signature, example, domain scenario, code, pseudocode,
  case, call sequence, validator internal, or implementation step.
- Generated-source policy: unchanged and compliant; committed generation
  inputs and outputs remain required, and required validation does not
  regenerate tracked source.
- Toolchain: CPython 3.14.6, standard Global Interpreter Lock build, GCC 15.2,
  headers, pip 26.1.2, wheel 0.47.0, and uv 0.11.21 remain available.
  Setuptools is not installed and no setuptools artifact is present in the pip
  cache.
- Markdown lint: clean for all 18 bounded planning and review artifacts with
  MD041 disabled for this report's mandated verdict first line.
