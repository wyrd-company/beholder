---
relationships:
  references:
    - ../../ground-rules
    - reports/csharp/synthesis/checklist
---

# Project brief: roslyn-components

## Purpose

An idiomatic solution that hosts Roslyn compiler components and their consumer:
source generators, analyzers and suppressors, and the diagnostic policy that
configures how the resulting diagnostics are treated.

## Exclusive directory

`fixtures/csharp/projects/roslyn-components/`

## Difficulty

complex

## Assigned canonical identifiers

- CS-CAN-015 — source generators.
- CS-CAN-016 — analyzers, suppressors, and generated-code diagnostics.
- CS-CAN-102 — diagnostic policy configuration.

Valid variants named by these items are in scope, including a classic or
incremental generator that consumes syntax, symbols, `AdditionalFiles`, analyzer
config, and metadata and emits a partial counterpart and a diagnostic with
recorded provenance; analyzer and suppressor inputs that inspect semantic state
without changing emitted program semantics, with configured generated-code
treatment and severity; and combined nullable settings, warning and analysis
levels, `.editorconfig`, global config, `NoWarn`, promotion and demotion, pragmas,
suppression attributes, and a suppressor showing precedence and the same source
passing or failing by configuration. The analyzer-crash, nondeterministic-output,
and compiler-error-not-downgradable counterexamples those rows also list are
diagnostic states, not valid fixture source.

## Declared build contexts

- Default `net10.0` for the consumer project.
- `netstandard2.0` for the generator, analyzer, and suppressor assemblies. This
  is the standard Roslyn component target framework and it loads in the SDK
  10.0.110
  Roslyn host: that host is `csc` 5.0 running on CoreCLR 10.0.10, and a component
  compiled against Roslyn API 4.14.0 targeting `netstandard2.0` is at or below the
  host's Roslyn version and within a framework the host implements, so it loads.
  `netstandard2.0` is not among the installed packs and is made available by a
  vendored pack named in dependency needs.

## Dependency needs

The following are not among the installed SDK packs and are vendored to compile
the generator, analyzer, and suppressor assemblies:

- **`Microsoft.CodeAnalysis.CSharp` version 4.14.0** (MIT-licensed Roslyn API),
  with its transitive **`Microsoft.CodeAnalysis.Common` 4.14.0**. Version 4.14.0
  is the current stable Roslyn API release and is at or below the host's `csc` 5.0,
  so components built against it load in the SDK 10.0.110 compiler.
- **`Microsoft.CodeAnalysis.Analyzers` version 3.11.0** (MIT-licensed), the
  analyzer and generator authoring support these APIs require.
- **`NETStandard.Library` version 2.0.3** (MIT-licensed), the `netstandard2.0`
  reference assemblies the component target framework requests offline.

The implementor pins these exact versions, records their source and license and
notice files, and preserves the NuGet and MSBuild metadata needed for offline
restore and build.

## Generated-source needs

The generator emits its compilation units in-process during the consumer's build;
this is part of the build, not a regeneration step, and requires no committed
generated output and no `generate` task. Provenance the generator records is its
own emitted output, not a checked-in file.

## Planned tests

none

## Required interfaces

`Taskfile.yml` and `coverage.md` as defined in `fixtures/csharp/ground-rules.md`.
