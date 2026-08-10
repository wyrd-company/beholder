VERDICT: REJECT

# Dart fixture plan review

## Blocking findings

### DART-PLAN-001 — P1: `dart test` dependencies are absent

Plan anchors:

- `fixtures/dart/plan/overview.md:L259-L262`
- `fixtures/dart/plan/projects/analysis-and-docs.md:L60` and `L82`
- `fixtures/dart/plan/projects/async-and-isolates.md:L58` and `L75`
- `fixtures/dart/plan/projects/code-generation.md:L53` and `L81`
- `fixtures/dart/plan/projects/configuration-and-versions.md:L53` and `L70`
- `fixtures/dart/plan/projects/construction-and-constants.md:L68` and `L85`
- `fixtures/dart/plan/projects/functions-and-dispatch.md:L58` and `L75`
- `fixtures/dart/plan/projects/libraries-and-visibility.md:L73` and `L90`
- `fixtures/dart/plan/projects/null-safety-flow-patterns.md:L74` and `L90`
- `fixtures/dart/plan/projects/platforms-and-interop.md:L77` and `L96`
- `fixtures/dart/plan/projects/primitive-values.md:L50` and `L67`
- `fixtures/dart/plan/projects/type-model.md:L90` and `L107`

Ground-rule anchor: `fixtures/dart/ground-rules.md:L116-L117` requires every
project's `test` task to run `dart test`, including projects with no test files.
The installed command reports that its flags and options are supplied by the
project's `package:test` dependency. Only `packages-and-workspace` plans that
dependency. The other eleven briefs require `dart test` while omitting the
package that supplies it, and the overview explicitly confines `package:test`
to `packages-and-workspace`. Their required `test` task therefore has no
feasible committed offline resolution.

Name and vendor a pinned `package:test` development dependency in every project
that invokes `dart test`, or change the ground-rule interface and briefs to a
feasible no-test command while preserving the required successful `test` task.

### DART-PLAN-002 — P1: distinct valid variants lack plan assignments

The overview assigns every canonical identifier, but several owning briefs do
not assign distinct valid variants named by their checklist rows. These are not
invalid-source cases or unavailable contexts:

| Plan anchor | Checklist anchor | Missing valid variants |
|---|---|---|
| `fixtures/dart/plan/projects/libraries-and-visibility.md:L36` | `reports/dart/synthesis/checklist.md:L64` | A hidden extension no longer participating in extension resolution. |
| `fixtures/dart/plan/projects/libraries-and-visibility.md:L46` | `reports/dart/synthesis/checklist.md:L67` | A foreign subclass's same-text private member being unrelated to the original library's member. |
| `fixtures/dart/plan/projects/libraries-and-visibility.md:L50` | `reports/dart/synthesis/checklist.md:L68` | First-matching conditional-import selection. |
| `fixtures/dart/plan/projects/libraries-and-visibility.md:L53` | `reports/dart/synthesis/checklist.md:L69` | Repeated `loadLibrary()` on a deferred import. |
| `fixtures/dart/plan/projects/type-model.md:L26` | `reports/dart/synthesis/checklist.md:L77` | Implementing a concrete class as an interface without inheriting its implementation. |
| `fixtures/dart/plan/projects/type-model.md:L32` | `reports/dart/synthesis/checklist.md:L79` | Later-mixin precedence and the same mixin over different bases having different `super` targets. |
| `fixtures/dart/plan/projects/type-model.md:L38` | `reports/dart/synthesis/checklist.md:L81` | Instance-member precedence, most-specific extension selection, `dynamic` bypass, and a nullable receiver including `null`. |
| `fixtures/dart/plan/projects/type-model.md:L41` | `reports/dart/synthesis/checklist.md:L82` | Nullable representation, casts, and contrast with class wrappers or extension methods. |
| `fixtures/dart/plan/projects/type-model.md:L47` | `reports/dart/synthesis/checklist.md:L84` | Record equality and named-field order versus names. |
| `fixtures/dart/plan/projects/type-model.md:L56` | `reports/dart/synthesis/checklist.md:L87` | Implementing a field with computed accessors, overriding one accessor half, and a final collection remaining mutable. |
| `fixtures/dart/plan/projects/type-model.md:L74` | `reports/dart/synthesis/checklist.md:L97` | Setter or field covariance and covariance across multiple interfaces. |
| `fixtures/dart/plan/projects/construction-and-constants.md:L42` | `reports/dart/synthesis/checklist.md:L113` | Specified single evaluation of side-effecting receivers and indexes during implicit operator lowering. |
| `fixtures/dart/plan/projects/null-safety-flow-patterns.md:L29` | `reports/dart/synthesis/checklist.md:L123` | Mutation, capture, loop, conflicting getter, non-final field, `noSuchMethod`, and public-property promotion defeats. |
| `fixtures/dart/plan/projects/null-safety-flow-patterns.md:L32` | `reports/dart/synthesis/checklist.md:L124` | Runtime read before `late` initialization, second write to `late final`, and initializer recursion or exception. |
| `fixtures/dart/plan/projects/null-safety-flow-patterns.md:L35` | `reports/dart/synthesis/checklist.md:L125` | Runtime failure from postfix `!`, property getter/setter interaction, and the cascade boundary. |
| `fixtures/dart/plan/projects/async-and-isolates.md:L31` | `reports/dart/synthesis/checklist.md:L141` | A second listener on a single-subscription stream failing at runtime. |
| `fixtures/dart/plan/projects/async-and-isolates.md:L39-L49` | `reports/dart/synthesis/checklist.md:L143-L144` | Native `spawnUri`, cyclic sendable data where supported, and `TransferableTypedData` reuse failure. The brief excludes only the web `spawnUri` variant. |
| `fixtures/dart/plan/projects/packages-and-workspace.md:L30` | `reports/dart/synthesis/checklist.md:L155` | Asset or data lookup across package roots. |
| `fixtures/dart/plan/projects/platforms-and-interop.md:L44` | `reports/dart/synthesis/checklist.md:L198` | A native Foreign Function Interface callback and its callback-thread relationship. |

Ground-rule anchors: `fixtures/dart/ground-rules.md:L32-L34` requires every
distinct valid variant that creates distinct structure or a semantic
relationship, and `fixtures/dart/ground-rules.md:L162-L167` requires each brief
to record those variants. Assign each listed variant to its primary project, or
record a permitted exception with a concrete unavailable context. Keep the
repair at project boundaries without prescribing source design.

## Installed-toolchain observations

- `dart --version`: Dart Software Development Kit (SDK) 3.12.2 stable on
  `linux_x64`.
- `dart compile` provides `js`, `jit-snapshot`, `kernel`, `exe`,
  `aot-snapshot`, and `wasm`.
- `gcc`, `cc`, and `clang` are installed. Flutter and common Chromium/Chrome
  browser commands are absent.
- `/usr/lib/dart/lib/_internal/allowed_experiments.json` has an empty default
  experiment set and defines `macros` plus `enhanced-parts` in its macros set.
- `dart test --help` states that command options come from the project's
  `package:test` dependency.

## Validation

- Canonical identifier accounting: 102 identifiers; 100 primary assignments;
  two item-level exclusions; no duplicate primary identifier.
- Project directories are pairwise exclusive, and no brief declares a
  dependency on another top-level fixture project.
- Declared Dart compilation targets are present in the installed SDK.
- `rumdl check` passed for all 16 authored planning artifacts.
- The review report passes `rumdl check --disable MD041`; MD041 is disabled
  because the required verdict must be the exact first line.
