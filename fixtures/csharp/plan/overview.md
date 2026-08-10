---
relationships:
  references:
    - ground-rules
    - reports/csharp/synthesis/checklist
---

# C# fixture corpus plan overview

## Selected toolchain

- **.NET SDK:** 10.0.110 (the only SDK installed; no `global.json` pin present).
- **MSBuild:** 18.0.11 bundled with the SDK.
- **Roslyn:** the C# compiler bundled in SDK 10.0.110.
- **Runtime / CLR:** CoreCLR, host and shared framework 10.0.10
  (`Microsoft.NETCore.App` 10.0.10, `Microsoft.AspNetCore.App` 10.0.10).
- **Runtime identifier:** `ubuntu.26.04-x64` (64-bit x86 Linux).
- **Language version:** the SDK's default C# language version for the current
  target framework, with earlier versions selected through `LangVersion` where a
  checklist item's behavior is version-gated.

The corpus does not provision historical .NET SDKs or Roslyn compilers, non-x64
processes, Windows- or macOS-only frameworks, or external services.

## Build contexts

The default build context is `net10.0` on CoreCLR 10.0.10, `ubuntu.26.04-x64`,
SDK-default language version, safe (non-unsafe) compilation. Each context below
is locally available and confirmed to build offline with no package source. A
project declares only the contexts its assignments require.

| Context | Availability | Owning projects |
| --- | --- | --- |
| `net10.0` default | Installed SDK/ref/runtime packs | all |
| Earlier `LangVersion` selections | Single Roslyn, no extra packs | source-and-binding, packages-and-build |
| Unsafe compilation (`AllowUnsafeBlocks`) | Roslyn intrinsic | unsafe-and-lowlevel |
| Native C companion library (`clang`/`gcc`) | `clang` and `gcc` present | unsafe-and-lowlevel |
| ASP.NET Core shared framework (Web SDK / `FrameworkReference`) | `Microsoft.AspNetCore.App.Ref` 10.0.10 installed | framework-and-builtin-generators |
| Trimming, single-file, Native Ahead-of-Time (AOT) publish | `Microsoft.NET.ILLink.Tasks` and `Microsoft.DotNet.ILCompiler` 10.0.10 in the SDK offline source; `clang` present for AOT link | deployment-and-platform |
| Second target framework for multi-targeting | Requires a vendored reference pack (see packages-and-build dependency need) | packages-and-build |
| Non-C# CLI producer (Visual Basic and F#) | Both compile offline on `net10.0` | interop-languages-and-testing |

## Projects

Each project occupies an exclusive directory under `fixtures/csharp/projects/`.

| # | Slug | Purpose | Difficulty |
| --- | --- | --- | --- |
| 1 | `source-and-binding` | Source form, declaration identity, and name binding across a coherent multi-file, multi-assembly library and application. | complex |
| 2 | `types-and-values` | Declared type kinds, records, value-type construction and copying, tuples, arrays, and nullability. | complex |
| 3 | `members-and-operators` | Fields, properties, indexers, events, constructors, accessibility, delegates, and user-defined operators and conversions. | routine |
| 4 | `generics-and-dispatch` | Generics, constraints, variance, interface members, overload resolution, conversions, and virtual and interface dispatch. | complex |
| 5 | `expressions-and-flow` | Definite assignment, patterns, switch, initializers, collection expressions, deconstruction, ranges, interpolation, checked arithmetic, expression trees, and query syntax. | complex |
| 6 | `functions-and-async` | Lambdas, local functions, async methods, iterators, `foreach`, async streams, exception handling, disposal, and lock lowering. | complex |
| 7 | `unsafe-and-lowlevel` | Native-sized integers, ref-like and inline-array structs, unsafe pointers and function pointers, pinning, layout metadata, and native interoperation. | complex |
| 8 | `attributes-and-metadata` | Attributes and recognized-attribute protocols, module initializers, caller information, reflection over emitted shape, and unspecified-shape observation. | complex |
| 9 | `assemblies-and-references` | Assembly and type identity, project and friend references, extern aliases, type forwarding, reference-versus-implementation assemblies, and binary compatibility. | complex |
| 10 | `packages-and-build` | SDK compile items, generated build inputs, packages, central package management, imported build policy, deterministic builds, resources, multi-targeting, and solution composition. | complex |
| 11 | `roslyn-components` | Roslyn source generators, analyzers and suppressors, and diagnostic policy configuration. | complex |
| 12 | `framework-and-builtin-generators` | Shared-framework and SDK-family compilation, built-in source generators, markup generation, and framework-mediated request handling. | complex |
| 13 | `deployment-and-platform` | Platform-compatibility attributes and analyzers, trimming, single-file, and Native AOT publishing, and reflection- and string-mediated linkage. | complex |
| 14 | `interop-languages-and-testing` | Consumption of non-C# CLI metadata and framework-mediated test discovery and invocation. | complex |

## Primary assignment of in-scope canonical identifiers

Every in-scope canonical identifier has exactly one primary project below.
Checklist overlap into other projects is natural and not accounted here.

| Project | Assigned canonical identifiers |
| --- | --- |
| `source-and-binding` | CS-CAN-001, CS-CAN-002, CS-CAN-003, CS-CAN-004, CS-CAN-005, CS-CAN-006, CS-CAN-007, CS-CAN-008, CS-CAN-009, CS-CAN-010, CS-CAN-011, CS-CAN-046, CS-CAN-048, CS-CAN-049 |
| `types-and-values` | CS-CAN-019, CS-CAN-020, CS-CAN-021, CS-CAN-022, CS-CAN-023, CS-CAN-024, CS-CAN-025, CS-CAN-026, CS-CAN-031, CS-CAN-032, CS-CAN-058 |
| `members-and-operators` | CS-CAN-033, CS-CAN-034, CS-CAN-035, CS-CAN-036, CS-CAN-037, CS-CAN-038, CS-CAN-039, CS-CAN-040 |
| `generics-and-dispatch` | CS-CAN-027, CS-CAN-041, CS-CAN-042, CS-CAN-043, CS-CAN-044, CS-CAN-045, CS-CAN-050, CS-CAN-051, CS-CAN-052, CS-CAN-053, CS-CAN-054, CS-CAN-055, CS-CAN-056, CS-CAN-057 |
| `expressions-and-flow` | CS-CAN-059, CS-CAN-060, CS-CAN-061, CS-CAN-062, CS-CAN-063, CS-CAN-064, CS-CAN-065, CS-CAN-066, CS-CAN-067, CS-CAN-068, CS-CAN-069, CS-CAN-070 |
| `functions-and-async` | CS-CAN-071, CS-CAN-072, CS-CAN-073, CS-CAN-074, CS-CAN-075, CS-CAN-076, CS-CAN-077, CS-CAN-078, CS-CAN-079 |
| `unsafe-and-lowlevel` | CS-CAN-028, CS-CAN-029, CS-CAN-030, CS-CAN-080, CS-CAN-107, CS-CAN-108 |
| `attributes-and-metadata` | CS-CAN-081, CS-CAN-082, CS-CAN-083, CS-CAN-084, CS-CAN-085, CS-CAN-086, CS-CAN-087, CS-CAN-088, CS-CAN-115, CS-CAN-125, CS-CAN-126, CS-CAN-127, CS-CAN-128 |
| `assemblies-and-references` | CS-CAN-047, CS-CAN-089, CS-CAN-090, CS-CAN-091, CS-CAN-092, CS-CAN-093, CS-CAN-094, CS-CAN-113 |
| `packages-and-build` | CS-CAN-012, CS-CAN-013, CS-CAN-014, CS-CAN-095, CS-CAN-096, CS-CAN-097, CS-CAN-099, CS-CAN-100, CS-CAN-101, CS-CAN-103, CS-CAN-105, CS-CAN-106, CS-CAN-124 |
| `roslyn-components` | CS-CAN-015, CS-CAN-016, CS-CAN-102 |
| `framework-and-builtin-generators` | CS-CAN-098, CS-CAN-104, CS-CAN-118, CS-CAN-119, CS-CAN-120, CS-CAN-122, CS-CAN-123 |
| `deployment-and-platform` | CS-CAN-110, CS-CAN-111, CS-CAN-121 |
| `interop-languages-and-testing` | CS-CAN-116, CS-CAN-117 |

123 in-scope canonical identifiers are assigned. The 5 excepted canonical items
and the 13 research-gap items below complete the 128 canonical rows and 13 gap
rows of the checklist.

## Exceptions

### Invalid-only items

Their distinctive coverage exists only as invalid source, a compiler-error state,
or recovered malformed syntax, which ground rules exclude from fixture content.

| Identifier | Reason |
| --- | --- |
| CS-CAN-017 | Recoverable syntax trees are parser recovery over malformed source. |
| CS-CAN-018 | Unresolved and erroneous symbols are compiler-error states, not valid source. |
| CS-CAN-112 | Ambiguous names, calls, conversions, and defaults are compile-time ambiguity diagnostics; each valid disambiguated form is already covered by its primary item (CS-CAN-004, CS-CAN-047, CS-CAN-050, CS-CAN-053, CS-CAN-057, CS-CAN-060). |

### Unavailable conditional contexts

| Identifier | Reason |
| --- | --- |
| CS-CAN-109 | COM interop and embedded type equivalence are Windows-only; the corpus host is `ubuntu.26.04-x64`. |
| CS-CAN-114 | Compiler breaking-change boundaries require pinned historical compiler and SDK pairs; the corpus provisions only SDK 10.0.110 and no moving-`latest` evidence is permitted. |

### Excluded variants of otherwise in-scope items

The named item keeps its primary project for its available variants; only the
listed variant is excepted.

| Item and variant | Category | Reason |
| --- | --- | --- |
| CS-CAN-028, 32-bit-process native-integer range | Unavailable context | No 32-bit runtime is installed; the 64-bit demonstration is owned by `unsafe-and-lowlevel`. |
| CS-CAN-098, Windows Desktop shared framework | Unavailable context | Only the ASP.NET Core shared framework is installed; the Web variant is owned by `framework-and-builtin-generators`. |
| CS-CAN-104, Windows markup workloads (WPF, WinForms, MAUI, XAML) | Unavailable context | Those workloads are Windows-only; the Web SDK and Razor markup variant is owned by `framework-and-builtin-generators`. |
| CS-CAN-120, interceptor-mediated call variant | Research gap | Interceptors are unresolved in CS-GAP-004; the ASP.NET Core request-delegate variant is owned by `framework-and-builtin-generators`. |
| CS-CAN-124, `.slnx` XML solution format | Research gap | `.slnx` is unresolved in CS-GAP-010; the classic `.sln` variant is owned by `packages-and-build`. |
| CS-CAN-128, non-x64 architecture and endianness observations | Unavailable context | Only `ubuntu.26.04-x64` is installed; the x64 non-invariant observation is owned by `attributes-and-metadata`. |

### Research gaps

The checklist keeps these outside the required set until verified against the
exact stable compiler, SDK, runtime, and primary specification. They are not
planned.

| Identifier | Subject |
| --- | --- |
| CS-GAP-001 | C# 14 extension blocks and members. |
| CS-GAP-002 | C# 14 `field`-backed property syntax. |
| CS-GAP-003 | Other claimed C# 14 language changes. |
| CS-GAP-004 | Experimental interceptors. |
| CS-GAP-005 | Script submissions and file-based apps. |
| CS-GAP-006 | Exact source-generator host requirements. |
| CS-GAP-007 | Legacy project, package, and runtime surfaces. |
| CS-GAP-008 | Rare or foreign-authored CLI metadata. |
| CS-GAP-009 | COM/NoPIA and alternate-platform depth. |
| CS-GAP-010 | Framework, workload, and runner variants. |
| CS-GAP-011 | Intermediate Language weaving and runtime code generation. |
| CS-GAP-012 | Concurrency and memory-model litmus behavior. |
| CS-GAP-013 | T4, documentation inheritance, and other generators. |

## Independence audit

- Each project occupies one exclusive directory under `fixtures/csharp/projects/`
  named by its slug. No two projects share a directory.
- No project depends on another top-level fixture project. Every project contains
  the local projects and assemblies it needs, including any multi-assembly,
  friend, forwarding, versioned-producer, or cross-language arrangements internal
  to that project.
- Third-party dependency needs are internal to their owning project and vendored:
  `roslyn-components` vendors the Roslyn API packages; `packages-and-build`
  vendors locally authored packages and a second-target-framework reference pack;
  `interop-languages-and-testing` vendors one test framework. No vendored
  dependency crosses a project boundary.
- No project's required `build`, `lint`, or `test` restores from the network.
