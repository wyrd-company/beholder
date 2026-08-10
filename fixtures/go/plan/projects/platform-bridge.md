---
relationships:
  references:
    - ground-rules
    - reports/go/synthesis/checklist
    - fixtures/go/plan/overview
---

# Project brief: `platform-bridge`

## Purpose

A portable, low-level library that adapts one logical interface across operating
systems, architectures, and a C boundary. It is the primary owner of build
selection, C interoperability, assembly, low-level compiler and linker
directives, WebAssembly directives, and unsafe layout.

## Exclusive directory

`fixtures/go/projects/platform-bridge/`

## Difficulty

complex

## Assigned canonical identifiers and distinct valid variants

- **GO-CAN-BLD-001** — modern `//go:build` constraints with conjunction,
  disjunction, negation, parentheses, platform tags, release tags, and custom
  tags.
- **GO-CAN-BLD-002** — legacy `+build` constraints paired with equivalent modern
  lines that agree, including a preserved pre-1.17 variant.
- **GO-CAN-BLD-003** — filename platform constraints: `_GOOS.go`, `_GOARCH.go`,
  and combined suffixes alongside generic files, plus a noncanonical suffix that
  is not constrained.
- **GO-CAN-BLD-004** — implicit tags, release tags, and GOOS implication,
  including compiler, cgo on/off, and a GOOS-derived tag such as `unix`.
- **GO-CAN-BLD-005** — architecture feature tags selecting code at two `GOAMD64`
  feature settings, distinguished from `GOARCH`.
- **GO-CAN-BLD-006** — ignored and specially named files and directories:
  `testdata`, dot and underscore paths, `_test.go`, and a conventional `ignore`
  tag seen differently by build, test, file command, and directory walk.
- **GO-CAN-BLD-007** — per-file language version raised or lowered by release
  constraints, with positive, negated, and compound release-tag expressions at a
  loop-semantics or syntax boundary.
- **GO-CAN-BLD-008** — conditional duplicate definitions of one logical interface
  in mutually exclusive platform or tag files with distinct bodies or types, each
  configuration compiling alone.
- **GO-CAN-BLD-009** — instrumented builds with race, coverage, profile-guided
  optimization, and `checkptr`, with semantic source declarations as the
  reference.
- **GO-CAN-BLD-010** — build cache, trimming, and source identity: rebuild
  identical sources with cache on and off and `-trimpath`, preserving logical
  source positions.
- **GO-CAN-BLD-011** — platform-dependent API surface: one import graph built for
  at least two `GOOS` or `GOARCH` targets and one non-platform setting, recording
  declarations and standard-library availability that differ, without unioning
  mutually exclusive APIs.
- **GO-CAN-CGO-001** — the Cgo preamble and pseudo-package `C`, referencing C
  types, functions, constants, and allocation from the comment immediately before
  `import "C"`, across multiple Cgo files.
- **GO-CAN-CGO-002** — Cgo directives and callbacks: `#cgo` flags, including
  platform-conditioned flags, and a valid `//export` callback with C and Go sides.
- **GO-CAN-CGO-003** — the Cgo source-inclusion switch: a Cgo implementation and
  a `!cgo` pure-Go fallback exposing the same interface, built at `CGO_ENABLED=1`
  and `0`.
- **GO-CAN-CGO-004** — Go pointer and C lifetime rules: permitted transient
  pointer passing, C allocation, `runtime.KeepAlive`, and pinning or handles.
  `cgocheck`-detected misuse is excluded (see overview exceptions).
- **GO-CAN-ASM-001** — assembly and bodyless declarations: Go declarations without
  bodies paired with matching `.s` implementations for the default architecture
  and called, including architecture suffixes and build constraints.
- **GO-CAN-DIR-001** — compiler directives and lexical placement for documented
  pragmas such as `noescape`, `nosplit`, `noinline`, and `norace`, only where
  accepted.
- **GO-CAN-DIR-002** — `go:linkname` visibility: a local declaration bound to a
  self-contained external object symbol with the required unsafe import, using
  allowed symbols under the post-1.23 restrictions and contrasting an ordinary
  exported alternative.
- **GO-CAN-DIR-003** — link-time data and build modes: an eligible string variable
  injected with `-ldflags -X`, built as an ordinary executable plus a relevant
  `pie`, `c-archive`, or `c-shared` mode.
- **GO-CAN-DIR-004** — WebAssembly directives: valid `go:wasmimport` and
  `go:wasmexport` declarations preserved for a WebAssembly target at the language
  versions where each is accepted.
- **GO-CAN-DYN-006** — unsafe operations and layout: size, alignment, and offset;
  valid pointer conversion and arithmetic within an object; documented slice and
  string helpers; exercised at two architectures.

## Declared build contexts

- Default context (`linux/amd64`, `CGO_ENABLED=1`).
- `CGO_ENABLED=0` pure-Go counterpart for GO-CAN-CGO-003 and GO-CAN-BLD-004.
- `GOOS=windows` and `GOOS=darwin` for GO-CAN-BLD-003, GO-CAN-BLD-008, and
  GO-CAN-BLD-011 (compile context).
- `GOARCH=386` alongside `GOARCH=amd64` for GO-CAN-DYN-006 two-architecture
  layout and GO-CAN-BLD-011 (compile context).
- `GOAMD64` feature settings at two levels for GO-CAN-BLD-005.
- WebAssembly target `GOOS=wasip1`/`GOARCH=wasm` (and `GOOS=js`/`GOARCH=wasm`
  where a variant needs it) for GO-CAN-DIR-004 (compile context).
- `gc` `amd64` assembly ABI for GO-CAN-ASM-001.
- Instrumentation `-race`, `-cover`, `-pgo`, and `checkptr` for GO-CAN-BLD-009.
- Build modes `pie`, `c-archive`, and `c-shared` for GO-CAN-DIR-003.
- `-trimpath` and build-cache on/off for GO-CAN-BLD-010.
- Per-file language-version release constraints for GO-CAN-BLD-007.

## Dependency needs

Standard library and project-local packages only. No third-party dependency. C
sources for the Cgo preamble and the `.s` assembly files are fixture-owned and
local to this project.

## Generated-source needs

None committed by a fixture generator. Cgo bridge artifacts and instrumentation
outputs are produced by the toolchain at build time and are not committed
generator output.

## Planned tests

None. Build selection, foreign-code, directive, and unsafe coverage is created by
source, directives, filenames, and build contexts; no test source is required to
create assigned coverage.

## Required `Taskfile.yml` and `coverage.md` interfaces

- `Taskfile.yml` defines `build`, `lint`, and `test` as thin wrappers over
  ordinary Go commands. `build` compiles all fixture-owned packages in every
  declared build context — the alternate operating systems, architectures,
  feature levels, WebAssembly target, cgo on/off, assembly, instrumentation,
  build modes, and trim and language-version variants. `lint` checks `gofmt`
  formatting and runs `go vet` on fixture-owned packages. `test` runs Go tests and
  succeeds with no test files. All three pass and write no logs, evidence,
  manifests, or hashes.
- `coverage.md` is a Markdown table recording, per assigned identifier, the
  canonical identifier, a stable source path, the named declaration, directive,
  package, module, or test that creates coverage, the build context when non-
  default, and additional locators for distinct valid variants. It uses stable
  names, not line numbers.
