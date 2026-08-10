---
relationships:
  references:
    - ../../ground-rules
    - reports/csharp/synthesis/checklist
---

# Project brief: interop-languages-and-testing

## Purpose

An idiomatic solution that exercises C# consumption of artifacts produced or
invoked by another toolchain: a non-C# CLI assembly consumed from C#, and a test
project whose tests are discovered and invoked by a test framework rather than
called directly.

## Exclusive directory

`fixtures/csharp/projects/interop-languages-and-testing/`

## Difficulty

complex

## Assigned canonical identifiers

- CS-CAN-116 — cross-language CLI consumption.
- CS-CAN-117 — test discovery and framework-mediated invocation.

Valid variants named by these items are in scope, including a small non-C# CLI
assembly whose module, static, indexer, default-member, optional, and functional
metadata shapes are consumed from C# and preserve types with no direct C#
spelling; and, in one test framework, ordinary and data-driven tests, member data
bound by `nameof`, fixtures and lifecycle, and the runner package or entry
behavior contrasted with direct calls. The invalid-data-source and
missing-linkage counterexamples those rows also list are diagnostic states, not
valid fixture source.

## Declared build contexts

- Default `net10.0` for the C# consumer and the test project.
- A non-C# CLI producer compiled on `net10.0`; both Visual Basic and F# compile
  offline with the installed SDK, and the implementor selects one.

## Dependency needs

**One test framework** and its runner are vendored for CS-CAN-117, because
framework-mediated discovery and invocation cannot be represented by local code
alone. The implementor names the framework, pins its package versions, records
their source and license and notice files, and preserves the NuGet and MSBuild
metadata needed for offline restore, build, and `test`. The non-C# CLI producer
of CS-CAN-116 is project-local source in a supported companion language and needs
no
third-party dependency.

## Generated-source needs

None.

## Planned tests

One test project, required by CS-CAN-117: its test source is the coverage for
CS-CAN-117, because framework-mediated discovery and invocation exist only as test
source that the framework runner discovers and invokes. No other project plans
tests.

## Required interfaces

`Taskfile.yml` and `coverage.md` as defined in `fixtures/csharp/ground-rules.md`.
