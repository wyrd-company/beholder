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
- The generator and analyzer assemblies target the framework required to load in
  the SDK 10.0.110 Roslyn host; the implementor selects that framework and, if its
  reference pack is not among the installed packs, vendors it under the dependency
  need below.

## Dependency needs

**Roslyn compiler API packages** (`Microsoft.CodeAnalysis.CSharp` and its
`Microsoft.CodeAnalysis.Common` dependency, and the analyzer and generator SDK
support they require) are not among the offline packs and are vendored. Their
purpose is to compile the generator, analyzer, and suppressor assemblies. The
implementor pins the package versions, records their source and license and notice
files, and preserves the NuGet and MSBuild metadata needed for offline restore and
build. If the chosen component target framework's reference pack is likewise
absent offline, it is vendored on the same terms.

## Generated-source needs

The generator emits its compilation units in-process during the consumer's build;
this is part of the build, not a regeneration step, and requires no committed
generated output and no `generate` task. Provenance the generator records is its
own emitted output, not a checked-in file.

## Planned tests

none

## Required interfaces

`Taskfile.yml` and `coverage.md` as defined in `fixtures/csharp/ground-rules.md`.
