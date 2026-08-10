---
relationships:
  references:
    - ../../ground-rules
    - reports/csharp/synthesis/checklist
---

# Project brief: assemblies-and-references

## Purpose

An idiomatic multi-assembly solution that exercises assembly and type identity and
the reference graph among assemblies: project references, friend access, extern
and reference aliases, type forwarding, reference-versus-implementation
assemblies, and binary-versus-source compatibility across recompiled producers and
consumers.

## Exclusive directory

`fixtures/csharp/projects/assemblies-and-references/`

## Difficulty

complex

## Assigned canonical identifiers

- CS-CAN-047 — source, reference, and duplicate-reference type conflicts.
- CS-CAN-089 — assembly and type identity.
- CS-CAN-090 — project-reference graph.
- CS-CAN-091 — friend assemblies.
- CS-CAN-092 — extern aliases and reference aliases.
- CS-CAN-093 — type forwarding and facades.
- CS-CAN-094 — reference-versus-implementation assemblies and framework packs.
- CS-CAN-113 — binary-versus-source compatibility.

Valid variants named by these items are in scope, including a source type winning
over a reference type with a warning; assemblies differing by name, namespace,
version, culture, and signing that declare equal qualified type names; an acyclic
project-reference graph with a transitive public type and a build-only
`ReferenceOutputAssembly=false` edge; `InternalsVisibleTo` friend versus
non-friend access; two assemblies exporting one qualified type resolved by
`extern alias` and reference aliases; a public type moved between assemblies with
forwarding metadata consumed by binaries built before and after the move;
compilation against the `net10.0` reference pack running on the implementation
pack and a produced project reference assembly; and versioned producer and
consumer pairs whose recompilation changes observed binding. Every assembly,
signing key, and reference in these arrangements is internal to this project. The
unresolved-ambiguity, circular-reference, and missing-target counterexamples those
rows also list are invalid source or diagnostic states and are not fixture
content.

## Declared build contexts

- Default `net10.0`, using the installed reference and implementation packs and
  `ProduceReferenceAssembly` for CS-CAN-094.

## Dependency needs

None third-party. All producer, consumer, friend, forwarding, alias, and version
assemblies are project-local. Any strong-name key used for the signed-producer
variant of CS-CAN-091 is committed to this project.

## Generated-source needs

The MSBuild-generated `InternalsVisibleTo` form of CS-CAN-091 is produced by the
build in-process. No committed generated output and no `generate` task.

## Planned tests

none

## Required interfaces

`Taskfile.yml` and `coverage.md` as defined in `fixtures/csharp/ground-rules.md`.
