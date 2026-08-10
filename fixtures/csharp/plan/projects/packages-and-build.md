---
relationships:
  references:
    - ../../ground-rules
    - reports/csharp/synthesis/checklist
---

# Project brief: packages-and-build

## Purpose

An idiomatic solution that exercises how the SDK, MSBuild, and NuGet assemble a
compilation: default and explicit compile items, generated build inputs, package
references and their asset separation, central package management, imported build
policy, deterministic builds, resources, multi-targeting, and solution
composition.

## Exclusive directory

`fixtures/csharp/projects/packages-and-build/`

## Difficulty

complex

## Assigned canonical identifiers

- CS-CAN-012 — SDK default compile items and explicit inclusion.
- CS-CAN-013 — one physical source in multiple compilations.
- CS-CAN-014 — SDK- and MSBuild-generated source.
- CS-CAN-095 — PackageReference graph and asset separation.
- CS-CAN-096 — central package management and lock files.
- CS-CAN-097 — source-distributing packages.
- CS-CAN-099 — language-version and compiler-version selection.
- CS-CAN-100 — conditional MSBuild properties and items and compiler constants.
- CS-CAN-101 — multi-targeting.
- CS-CAN-103 — imported build policy and SDK selection.
- CS-CAN-105 — deterministic, path-mapped, debug, and embedded-source builds.
- CS-CAN-106 — resources and strongly typed accessors.
- CS-CAN-124 — solution and workspace composition.

Valid variants named by these items are in scope, including default-globbed,
removed, linked, and conditional compile items; one linked source compiled into
two targets; SDK-generated assembly attributes, target-framework attributes, and
global usings and a pre-compile target-created source as real compilation units;
direct and transitive package references separating compile, runtime, analyzer,
build, buildTransitive, and content assets with include, exclude, and private
variations; package versions moved to `Directory.Packages.props` with one
override and a generated lock file restored in locked mode; a package whose
`contentFiles` C# asset is emitted into the consumer; version-gated syntax under
fixed, `latest`, and `preview` language versions with the compiler and SDK
version recorded; configuration, platform, and custom-property conditions with
`Choose` and `When`; `Directory.Build.props` and `targets`, nested imports, and
a `global.json` SDK pin; deterministic, path-mapped, portable-PDB, Source Link,
and embedded-source builds; default and localized resources with a typed accessor
and satellite assembly; two target frameworks producing distinct compilation
inputs; and classic-solution membership with virtual folders, configuration
mapping, and a direct no-solution build. CS-CAN-099 records the single installed
compiler and SDK version; its language-version axis is covered by the fixed,
`latest`, and `preview` selections above. The `.slnx` XML-solution variant of
CS-CAN-124 is excepted in the overview; the duplicate-item and missing-pin
counterexamples those rows also list are invalid source or diagnostic states and
are not fixture content.

## Declared build contexts

- Default `net10.0`.
- Earlier and `preview` `LangVersion` selections for CS-CAN-099.
- A second target framework for the multi-targeting inner and outer builds of
  CS-CAN-101, provided by a vendored reference pack (see dependency needs).
- Conditional configuration, platform, and custom-property contexts for
  CS-CAN-100.

## Dependency needs

- **Locally authored packages** for CS-CAN-095, CS-CAN-096, and CS-CAN-097. These
  are produced from project-local source and vendored into a local package feed
  so the package graph, central management, lock file, and content-file
  distribution
  are exercised without any third-party download. Not third-party; no external
  provenance required.
- **One additional target-framework reference pack** for CS-CAN-101, vendored so
  the second target framework compiles offline. Its purpose is to make one
  additional target framework locally available; the implementor pins the pack
  version, records its source and license and notice files, and preserves the
  NuGet and MSBuild metadata needed for offline restore and build.

## Generated-source needs

CS-CAN-014 requires SDK- and MSBuild-generated compilation units. Generator source
and any committed pre-compile target output are committed; the required `build`,
`lint`, and `test` tasks never rewrite them. In-process SDK generation of assembly
attributes and global usings is part of the build.

## Planned tests

none

## Required interfaces

`Taskfile.yml` and `coverage.md` as defined in `fixtures/csharp/ground-rules.md`.
