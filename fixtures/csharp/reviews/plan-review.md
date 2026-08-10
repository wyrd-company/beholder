VERDICT: REJECT

# C# fixture plan review

## Blocking findings

### CSHARP-PLAN-001 — P1

- **Plan anchors:** `fixtures/csharp/plan/overview.md:27-40` and
  `fixtures/csharp/plan/projects/packages-and-build.md:65-86`
- **Checklist anchor:** `reports/csharp/synthesis/checklist.md:136` (CS-CAN-101)
- **Violated ground rule:** `fixtures/csharp/ground-rules.md:27-29` requires an
  additional target framework context to be locally available, and
  `fixtures/csharp/ground-rules.md:156-169` requires the plan and brief to record
  available and declared build contexts.
- **Condition:** The overview says every listed context is locally available,
  but the second target framework is neither selected nor installed and instead
  requires an unnamed reference pack to be acquired later. Without an exact
  target framework moniker and corresponding reference-pack identity, the
  CS-CAN-101 context cannot be checked for availability, offline feasibility, or
  distinct reference selection. Select the second target framework and name the
  pack that makes it available.

### CSHARP-PLAN-002 — P1

- **Plan anchor:** `fixtures/csharp/plan/projects/roslyn-components.md:42-59`
- **Checklist anchors:** `reports/csharp/synthesis/checklist.md:30-31`
  (CS-CAN-015 and CS-CAN-016)
- **Violated ground rule:** `fixtures/csharp/ground-rules.md:156-169` requires the
  overview and each brief to record available and declared build contexts.
- **Condition:** The brief delegates selection of the generator and analyzer
  target framework to the implementor and conditionally introduces another
  unnamed reference pack. The plan therefore does not declare the component
  compilation context or establish that SDK 10.0.110 can load it. Select the
  target framework and identify any pack required to compile it so compatibility
  and offline availability can be reviewed before implementation.

### CSHARP-PLAN-003 — P1

- **Plan anchor:**
  `fixtures/csharp/plan/projects/interop-languages-and-testing.md:45-54`
- **Checklist anchor:** `reports/csharp/synthesis/checklist.md:159` (CS-CAN-117)
- **Violated ground rule:** `fixtures/csharp/ground-rules.md:81-91` requires a
  reviewed brief to name every third-party dependency and permits network access
  only to acquire those named dependencies.
- **Condition:** "One test framework" does not name the framework or runner, and
  the brief defers that choice to the implementor. The implementor therefore has
  no reviewed dependency it is authorized to acquire, and framework/runner
  compatibility cannot be checked. Name the selected test framework and runner;
  retain the existing pinning, provenance, licensing, and offline-build
  requirements.

## Verification

- Checklist accounting: 123 identifiers have exactly one primary project; five
  identifiers are whole-item exceptions; all 128 canonical identifiers are
  accounted for; all 13 research gaps are listed; overview and brief assignments
  match.
- Project boundaries: 14 unique exclusive directories are declared, and no brief
  depends on another top-level fixture project.
- Planned tests: only `interop-languages-and-testing` plans test source, and it
  assigns that source to CS-CAN-117.
- Markdown lint: `rumdl check` passed for all 18 authored planning artifacts.

## Installed-toolchain observations

- Installed SDK: .NET SDK 10.0.110 with MSBuild 18.0.11 and CoreCLR 10.0.10 on
  `ubuntu.26.04-x64`; no other SDK or architecture is installed.
- Installed shared frameworks: `Microsoft.NETCore.App` 10.0.10 and
  `Microsoft.AspNetCore.App` 10.0.10, with their 10.0.10 reference packs.
- Native AOT and trimming inputs are present as
  `Microsoft.DotNet.ILCompiler` 10.0.10 and `Microsoft.NET.ILLink.Tasks` 10.0.10;
  `clang` and `gcc` are installed.
- Visual Basic and F# compilers are bundled with SDK 10.0.110.
- No second target-framework reference pack or vendored Roslyn/test-framework
  package is present in the installed SDK packs.
