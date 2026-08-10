VERDICT: REJECT

# TypeScript fixture plan review

## Blocking findings

### TS-PLAN-001 — P1: Compiler-version context is not pinned

`fixtures/typescript/plan/projects/compiler-version-boundary.md:38-48` declares
only “one earlier” release somewhere in the TypeScript 4.7–5.8 range. The exact
release and therefore the exact version boundary remain undecided. This conflicts
with `reports/typescript/synthesis/checklist.md:21-27`, which makes exact compiler
identity and an explicit transition case part of TS-CAN-001, and with
`fixtures/typescript/ground-rules.md:22-35`, which requires declared compilation
contexts to identify the pinned compiler version. Select the exact earlier
release and state the exact two-release context so feasibility and variant
coverage can be reviewed.

### TS-PLAN-002 — P1: Deferred-import context defers feasibility to implementation

`fixtures/typescript/plan/projects/module-graph-edges.md:56-70` assigns
TS-CAN-033 but declares only an unspecified TypeScript 5.9 module mode “that
supports” the syntax and directs the implementor to confirm it. The reviewed plan
must establish a valid, available compilation context before implementation. The
supported-module condition is at
`reports/typescript/synthesis/checklist.md:279-285`. Ground rules require the
overview and brief to declare contexts at
`fixtures/typescript/ground-rules.md:168-185`. Name a verified supported mode, or
record the item/variant under an allowed research-gap or unavailable-context
exception.

### TS-PLAN-003 — P1: Case-insensitive coverage has no declared available context

`fixtures/typescript/plan/projects/module-resolution-and-formats.md:44-55` claims
the case-insensitive-filesystem variant of TS-CAN-012, but its only file-identity
context is an unspecified casing and committed-symlink layout. The corpus default
is 64-bit x86 Linux (`fixtures/typescript/plan/overview.md:26-30`), and the plan
does not declare an available case-insensitive filesystem or compiler-host
context. This leaves the conditional filesystem variant in
`reports/typescript/synthesis/checklist.md:109-115` neither feasible nor excepted.
Declare an actually available context or list the variant as an unsupported
conditional context.

### TS-PLAN-004 — P1: Invalid-source variants remain assigned

The following assignments contradict their own valid-source exclusions and
`fixtures/typescript/ground-rules.md:47-56`:

- `fixtures/typescript/plan/projects/program-solution-workspace.md:29-46` excludes
  error variants, then assigns the TS-CAN-005 error-removal state. That state
  requires a preceding compiler error rather than controlled valid source.
- `fixtures/typescript/plan/projects/type-level-programming.md:28-31` excludes
  invalid forms, but `fixtures/typescript/plan/projects/type-level-programming.md:55-59`
  assigns an interface extending a union, which the checklist identifies as the
  failure variant at `reports/typescript/synthesis/checklist.md:621-627`.
- `fixtures/typescript/plan/projects/type-relations-and-compatibility.md:50-54`
  assigns a write rejection through a readonly view, which is a compiler-error
  case rather than valid-source coverage (`reports/typescript/synthesis/checklist.md:541-547`).

Remove invalid-source coverage from the briefs. Retain only valid structural or
type-changing residue.

### TS-PLAN-005 — P1: TS-CAN-038 omits its valid mismatch obligation

`fixtures/typescript/plan/projects/package-publishing-ecosystem.md:29-33`
excludes load-time mismatches. Its TS-CAN-038 assignment at lines 57-58 has no
runtime/type entry-point mismatch. The checklist obligation at
`reports/typescript/synthesis/checklist.md:321-327` requires an intentional
mismatch that can still be valid TypeScript source and package metadata. Runtime
failure does not make the source compiler-invalid. Assign this distinct valid
variant without turning it into an application behavior test.

### TS-PLAN-006 — P1: Valid JSDoc variant is incorrectly excluded

`fixtures/typescript/plan/projects/checked-javascript-inputs.md:25-36` excludes
unsupported or ignored JSDoc tags as though they were invalid source. They are a
named valid TS-CAN-013 variant at
`reports/typescript/synthesis/checklist.md:117-123`; an ignored tag remains valid
JavaScript/TypeScript input. Assign the variant, or provide an allowed exception.

### TS-PLAN-007 — P1: Distinct valid module variants are missing

The module-edge brief does not complete the required valid-variant accounting:

- `fixtures/typescript/plan/projects/module-graph-edges.md:33-37` assigns an
  anonymous default class **or** function. TS-CAN-018 requires both distinct
  declaration forms at `reports/typescript/synthesis/checklist.md:159-165`.
- `fixtures/typescript/plan/projects/module-graph-edges.md:41-43` omits the valid
  cyclic-barrel, identical-origin-through-two-stars, CommonJS-source, and
  default-not-forwarded-by-star variants named for TS-CAN-020 at
  `reports/typescript/synthesis/checklist.md:175-181`.

Complete the brief's distinct valid variants and re-run the same audit across all
assigned canonical items.

## Validation

- Identifier accounting: 92 canonical identifiers appear exactly once in the
  overview. The 90 in-scope identifiers appear exactly once across project
  briefs; TS-CAN-088 and TS-CAN-092 are the two item exceptions.
- Independence: 19 exclusive project directories are declared. No brief depends
  on another top-level fixture project.
- Tests: all 19 briefs plan `none` and state that test source creates no assigned
  coverage.
- Dependency and generated-source policy: reviewed plans keep named generation
  inside project boundaries and keep required validation tasks non-mutating.
- Toolchain observations: declared Node.js `v26.5.1` and npm `11.17.0` are locally
  available through the Homebrew installation; Bun `1.3.14` is locally
  available; TypeScript `5.9.3` runs from the local npm cache in offline mode.
- Markdown: `rumdl check` passed for the 22 review-readable TypeScript ground-rule,
  reviewer-prompt, overview, and project-brief files.

## Severity counts

- P0: 0
- P1: 7
- P2: 0
- P3: 0
- Q: 0
