---
relationships:
  references:
    - ../../ground-rules
    - reports/csharp/synthesis/checklist
---

# Project brief: framework-and-builtin-generators

## Purpose

An idiomatic web-capable solution that exercises SDK-family and shared-framework
compilation and the source generators the SDK and framework ship: an
`Microsoft.AspNetCore.App` shared-framework reference, Web SDK and Razor markup
generation, System.Text.Json, regular-expression, and logging generators, and
framework-mediated request handling.

## Exclusive directory

`fixtures/csharp/projects/framework-and-builtin-generators/`

## Difficulty

complex

## Assigned canonical identifiers

- CS-CAN-098 — framework references and shared frameworks (ASP.NET Core variant).
- CS-CAN-104 — SDK family and workload-generated compilation (Web SDK and Razor
  markup variant).
- CS-CAN-118 — System.Text.Json generated-versus-reflection serialization.
- CS-CAN-119 — generated regular expressions.
- CS-CAN-120 — framework convention and request-delegate-mediated calls (ASP.NET
  Core variant).
- CS-CAN-122 — code generation from non-C# inputs (markup variant).
- CS-CAN-123 — attribute-driven member-synthesis toolkit (the SDK-supplied logging
  message generator).

Valid variants named by these items are in scope, including an explicit or
SDK-implicit ASP.NET Core shared-framework reference versus a plain library
without it; base-SDK versus Web-SDK compilation inputs and a Razor markup-
generated partial class with `x:`-style name linkage; a generated
`JsonSerializerContext` for a closed type graph contrasted with a reflection-based
call; a generated regular-expression partial member contrasted with a runtime-
constructed regex; minimal endpoint handlers, controller discovery, parameter
binding, dependency injection, and the request-delegate-generated path; C#
generated from markup connected to handwritten code with checked-in-versus-build-
generated contrast; and a source-generated logging method whose generated surface
is consumed. The Windows Desktop shared-framework variant of CS-CAN-098, the
Windows markup workloads of CS-CAN-104, and the interceptor variant of
CS-CAN-120 are excepted in the overview. The missing-registration, unsupported-
member, and invalid-pattern counterexamples those rows also list are diagnostic
states, not valid fixture source.

## Declared build contexts

- Default `net10.0`.
- ASP.NET Core shared framework through the Web SDK and `FrameworkReference`
  (`Microsoft.AspNetCore.App` 10.0.10, installed).

## Dependency needs

None third-party. The shared framework, the System.Text.Json, regular-expression,
Razor, request-delegate, and logging generators, and `Microsoft.Extensions`
logging abstractions are all part of the installed SDK and ASP.NET Core shared
framework and require no download.

## Generated-source needs

The JSON, regular-expression, Razor markup, request-delegate, and logging
generators emit their compilation units in-process during the build. For
CS-CAN-122, the checked-in-versus-build-generated contrast commits the checked-in
generated file alongside the build-generated one; the required tasks never rewrite
either. No `generate` task.

## Planned tests

none

## Required interfaces

`Taskfile.yml` and `coverage.md` as defined in `fixtures/csharp/ground-rules.md`.
