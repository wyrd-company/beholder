---
relationships:
  references:
    - ../../ground-rules
    - reports/csharp/synthesis/checklist
---

# Project brief: deployment-and-platform

## Purpose

An idiomatic application and library that exercise platform-targeting and
publish-shape behavior: platform-compatibility attributes and analyzers; trimming,
single-file, and Native Ahead-of-Time publishing with reachability and dynamic-
code annotations; and reflection- and string-mediated linkage across a plugin
boundary.

## Exclusive directory

`fixtures/csharp/projects/deployment-and-platform/`

## Difficulty

complex

## Assigned canonical identifiers

- CS-CAN-110 — platform-specific APIs and analyzers.
- CS-CAN-111 — trimming, single-file, and Native AOT.
- CS-CAN-121 — reflection- and string-mediated linkage.

Valid variants named by these items are in scope, including platform-specific APIs
behind recognized guards with `SupportedOSPlatform` and `UnsupportedOSPlatform`
attributes and sources and assets conditioned on operating system and
architecture; reflection and dynamic activation annotated for trimming, single-
file, and Native AOT with reachability and dynamic-code attributes and the
analyzer warnings they drive; and type and member resolution by raw string and by
`nameof`, dynamic instantiation, configuration binding, and loading through an
assembly-load-context plugin boundary. The runtime-only failure counterexamples
those rows also list are observed runtime behavior of valid source, not invalid
source.

## Declared build contexts

- Default `net10.0`.
- Trimming and single-file publish using `Microsoft.NET.ILLink.Tasks` 10.0.10
  (installed).
- Native AOT publish using `Microsoft.DotNet.ILCompiler` 10.0.10 (installed) with
  the locally installed `clang` for native linking.

The trimming, single-file, and AOT analyzers surface their warnings during the
default build of the annotated source; the publish contexts exercise the
end-to-end trim, single-file, and AOT shapes. The required `build`, `lint`, and
`test` tasks remain offline and perform no download.

## Dependency needs

None third-party. The trimming and AOT toolchains are in the SDK's offline package
source and the native linker is locally installed.

## Generated-source needs

Where the AOT toolchain replaces reflection with source generation for
CS-CAN-111, that generation runs in-process during publish. No committed generated
output and no `generate` task.

## Planned tests

none

## Required interfaces

`Taskfile.yml` and `coverage.md` as defined in `fixtures/csharp/ground-rules.md`.
