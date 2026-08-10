---
relationships:
  references:
    - ground-rules
    - reports/go/synthesis/checklist
---

# Go fixture corpus plan overview

This overview records the selected compiler, available build contexts, the
project list, the primary owner of every in-scope canonical checklist item, the
scope exceptions, and the independence audit. Each project has a standalone brief
under `fixtures/go/plan/projects/`.

## Selected compiler and build contexts

- **Compiler.** Official Go `gc` toolchain `go1.26.5`, module mode, on 64-bit x86
  Linux. This is the single installed release the corpus builds with. No
  historical toolchain is provisioned.
- **Language-version baseline.** The checklist baseline is Go 1.25. A newer
  installed toolchain does not erase semantics selected by a module or file
  language version, so version-sensitive items select their language version
  through `go` directives and release build constraints, not through a different
  toolchain. Default module language version is `go1.25`; boundary items raise or
  lower it explicitly.

### Default build context

- `GOOS=linux`, `GOARCH=amd64`, module mode, `CGO_ENABLED=1` with the installed
  system C toolchain available.

### Additional available build contexts

Every context below was confirmed to build with the installed toolchain. A
project declares only the contexts its assigned coverage requires.

- **Alternate architecture:** `GOARCH=386` (32-bit), for width-boundary,
  atomic-alignment, and two-architecture layout coverage (compile context).
- **Alternate operating systems:** `GOOS=windows` and `GOOS=darwin`, for
  filename and expression platform selection and platform-dependent API surface
  (compile context).
- **WebAssembly:** `GOOS=wasip1`/`GOARCH=wasm` and `GOOS=js`/`GOARCH=wasm`, for
  WebAssembly directive coverage (compile context).
- **C interoperability:** `CGO_ENABLED=1` with the system C toolchain, and its
  `CGO_ENABLED=0` pure-Go counterpart.
- **Assembly:** `gc` `amd64` assembly ABI with bodyless Go declarations.
- **Architecture feature levels:** `GOAMD64` feature settings (for example `v1`
  and a higher level) with matching feature build tags.
- **Instrumentation:** `-race`, `-cover`, profile-guided optimization (`-pgo`),
  and `checkptr`.
- **Build modes:** `pie`, `c-archive`, `c-shared`, and `plugin`.
- **Source-identity flags:** `-trimpath` and build-cache on/off.
- **Module resolution modes:** workspace on/off (`GOWORK`), vendor mode
  (`-mod=vendor`) versus module mode (`-mod=mod`), `GOTOOLCHAIN=local`, and a
  hermetic local module proxy or prepopulated module cache (no network).
- **Per-file / per-module language versions:** `go` directives and release build
  constraints across the `go1.17`–`go1.25` boundaries used by version-sensitive
  items.

## Projects

| Project | Purpose | Directory | Difficulty |
| --- | --- | --- | --- |
| `package-structure` | Package formation, scopes, declarations, initialization, and lexical structure | `fixtures/go/projects/package-structure/` | complex |
| `collections-toolkit` | Types, methods, interfaces, generics, expressions, and control flow | `fixtures/go/projects/collections-toolkit/` | complex |
| `pipeline-scheduler` | Concurrency, the memory model, atomics, context, errors, and runtime lifecycle | `fixtures/go/projects/pipeline-scheduler/` | complex |
| `plugin-registry` | Multiple commands, init registration, reflection, struct tags, templates, plugins, generated bindings | `fixtures/go/projects/plugin-registry/` | complex |
| `platform-bridge` | Build selection, C interoperability, assembly, low-level directives, and unsafe layout | `fixtures/go/projects/platform-bridge/` | complex |
| `module-federation` | Module and dependency identity, versioning, workspaces, vendoring, and layout | `fixtures/go/projects/module-federation/` | complex |
| `codegen-assets` | Documentation, `go:generate`, generated markers, and embedded files | `fixtures/go/projects/codegen-assets/` | routine |
| `behavior-suite` | Test, example, fuzz, and `testdata` mechanics, plus package-graph validity | `fixtures/go/projects/behavior-suite/` | complex |

## Primary assignment of in-scope canonical identifiers

Each identifier below has exactly one primary project. Features may recur in
other projects; natural overlap is not accounted here.

| Project | Primary canonical identifiers |
| --- | --- |
| `package-structure` | GO-CAN-SRC-001, GO-CAN-SRC-002, GO-CAN-SRC-003, GO-CAN-SRC-004, GO-CAN-SRC-005, GO-CAN-SRC-006, GO-CAN-SRC-007, GO-CAN-SRC-008, GO-CAN-SRC-009, GO-CAN-IMP-001, GO-CAN-DEC-001, GO-CAN-DEC-002, GO-CAN-DEC-003, GO-CAN-DEC-005, GO-CAN-DEC-006 |
| `collections-toolkit` | GO-CAN-TYP-001, GO-CAN-TYP-002, GO-CAN-TYP-003, GO-CAN-TYP-004, GO-CAN-TYP-005, GO-CAN-TYP-006, GO-CAN-TYP-007, GO-CAN-TYP-008, GO-CAN-TYP-009, GO-CAN-TYP-010, GO-CAN-TYP-011, GO-CAN-TYP-012, GO-CAN-TYP-013, GO-CAN-TYP-014, GO-CAN-MET-001, GO-CAN-MET-002, GO-CAN-MET-003, GO-CAN-IFC-001, GO-CAN-IFC-002, GO-CAN-IFC-003, GO-CAN-IFC-004, GO-CAN-GEN-001, GO-CAN-GEN-002, GO-CAN-GEN-003, GO-CAN-GEN-004, GO-CAN-GEN-005, GO-CAN-EXP-001, GO-CAN-EXP-002, GO-CAN-EXP-003, GO-CAN-EXP-004, GO-CAN-EXP-005, GO-CAN-EXP-006, GO-CAN-EXP-007, GO-CAN-EXP-008, GO-CAN-EXP-009, GO-CAN-EXP-010, GO-CAN-CTL-001, GO-CAN-CTL-002, GO-CAN-CTL-003, GO-CAN-CTL-004, GO-CAN-CTL-005, GO-CAN-CTL-006, GO-CAN-CTL-007, GO-CAN-CTL-008, GO-CAN-CTL-009, GO-CAN-DIA-008 |
| `pipeline-scheduler` | GO-CAN-CON-001, GO-CAN-CON-002, GO-CAN-CON-003, GO-CAN-CON-004, GO-CAN-CON-005, GO-CAN-CON-006, GO-CAN-DYN-005, GO-CAN-DYN-009, GO-CAN-DIA-009 |
| `plugin-registry` | GO-CAN-DEC-004, GO-CAN-DYN-001, GO-CAN-DYN-002, GO-CAN-DYN-003, GO-CAN-DYN-004, GO-CAN-DYN-007, GO-CAN-DYN-008 |
| `platform-bridge` | GO-CAN-BLD-001, GO-CAN-BLD-002, GO-CAN-BLD-003, GO-CAN-BLD-004, GO-CAN-BLD-005, GO-CAN-BLD-006, GO-CAN-BLD-007, GO-CAN-BLD-008, GO-CAN-BLD-009, GO-CAN-BLD-010, GO-CAN-BLD-011, GO-CAN-CGO-001, GO-CAN-CGO-002, GO-CAN-CGO-003, GO-CAN-CGO-004, GO-CAN-ASM-001, GO-CAN-DIR-001, GO-CAN-DIR-002, GO-CAN-DIR-003, GO-CAN-DIR-004, GO-CAN-DYN-006 |
| `module-federation` | GO-CAN-MOD-001, GO-CAN-MOD-002, GO-CAN-MOD-003, GO-CAN-MOD-004, GO-CAN-MOD-005, GO-CAN-MOD-006, GO-CAN-MOD-007, GO-CAN-MOD-008, GO-CAN-MOD-009, GO-CAN-MOD-010, GO-CAN-MOD-012, GO-CAN-MOD-013, GO-CAN-ECO-001, GO-CAN-ECO-002 |
| `codegen-assets` | GO-CAN-SRC-010, GO-CAN-GENR-001, GO-CAN-GENR-002, GO-CAN-GENR-003 |
| `behavior-suite` | GO-CAN-TST-001, GO-CAN-TST-002, GO-CAN-TST-003, GO-CAN-TST-004, GO-CAN-TST-005, GO-CAN-TST-006, GO-CAN-MOD-011 |

## Exceptions

Only exclusions are listed. Every excluded item falls in exactly one allowed
category: invalid-only, unsupported conditional context, or research gap.

| Canonical identifier | Category | Reason |
| --- | --- | --- |
| GO-CAN-MOD-014 | Research gap | Go 1.25 `ignore` directive is singly sourced and marked unresolved; the plan does not resolve research gaps. |
| GO-CAN-TST-007 | Invalid-only | Its coverage requires source that triggers analyzer diagnostics; the required `lint` gate runs `go vet` on fixture-owned packages and must pass, so vet-triggering source cannot be committed and only quiet near-neighbors — which do not create the coverage — would remain. |
| GO-CAN-DIA-001 | Invalid-only | Lexical/parse failure with partial syntax is realized only through compiler rejection and parser recovery, which are not fixture content. |
| GO-CAN-DIA-002 | Invalid-only | Name and declaration failures are realized only through compiler rejection; valid near-neighbors are owned by the positive declaration and import items. |
| GO-CAN-DIA-003 | Invalid-only | The type-error taxonomy is realized only through compiler rejection; each legal near-neighbor is owned by its positive type, method, interface, or generic item. |
| GO-CAN-DIA-004 | Invalid-only | Package and load failures are realized only through loader rejection; valid resolution is owned by the module and package items. |
| GO-CAN-DIA-005 | Invalid-only | The distinct coverage is a quarantined ambiguous selector use (a compile error); the legal equal-depth structure and explicit-path resolution are owned by GO-CAN-TYP-010. |
| GO-CAN-DIA-006 | Invalid-only | Interface-satisfaction diagnostics are realized only through compile-time rejection; valid satisfaction is owned by GO-CAN-IFC-001. |
| GO-CAN-DIA-007 | Invalid-only | The distinct coverage is an unprovided body or missing linked symbol (a compile or link failure); provided bodyless declarations are owned by GO-CAN-ASM-001 and the Cgo and directive items. |

Excluded variants inside otherwise in-scope items are not re-enumerated. Every
"counterexample and failure" a checklist item names — no-files-match, malformed
expression, wrong or missing signature, missing symbol, illegal redeclaration,
jump-over-declaration, uncomparable key, and the like — is uniformly excluded as
invalid source per the ground rules. Named excluded variants worth calling out:

- GO-CAN-DIA-008: the fatal, non-recoverable runtime-state variant is excluded as
  abnormal termination that would break `test`; recoverable panic-and-recover
  coverage is retained.
- GO-CAN-MOD-002: the automatic toolchain-download variant is excluded as an
  external service; `GOTOOLCHAIN=local` selection and language-version raising are
  retained.
- GO-CAN-MOD-010: external proxy, sumdb, and network resolution variants are
  excluded as external services; hermetic `go.sum`, local-proxy, and vendor
  resolution are retained.
- GO-CAN-MOD-012: relative-import behavior and a GOPATH baseline are excluded as
  invalid or unsupported legacy configuration; the inert canonical import-path
  comment that module mode ignores is retained.
- GO-CAN-CON-005: whether a race manifests under `-race` is not asserted; the
  race source and the race build context are retained.
- GO-CAN-CGO-004: `cgocheck`-detected misuse is excluded as a runtime error;
  permitted transient pointer passing, allocation, `KeepAlive`, and handles are
  retained.

## Independence audit

- Each project occupies one exclusive directory directly under
  `fixtures/go/projects/`; the eight directories are disjoint and named by the
  project slugs above.
- No project imports another top-level fixture project. Every project builds from
  the standard library and its own local modules only.
- No third-party dependency is required anywhere in the corpus; the module,
  workspace, and vendoring coverage is created with local modules and a hermetic
  local proxy or cache, so no implementor needs network access.
- `module-federation` contains several local modules and a workspace; all live
  inside its own directory and none is importable by another project.
- Each project independently defines `build`, `lint`, and `test`, and passes them
  within its declared build contexts. The corpus root defines the same three
  tasks and invokes each project's corresponding task.
