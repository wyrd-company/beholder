# Project brief: macros-and-procedural-macros

## Purpose

Demonstrate Rust's macro systems and the entities they create: declarative
matching, scope, hygiene, and `$crate`; macro-generated named items; the
procedural-macro crate role with function-like, derive, and attribute entry
points; derive helper attributes; procedural spans and resolution; compiler
built-in macros; entry-point synthesis and rewriting; and a hermetic
macro-backed embedded DSL.

## Exclusive directory

`fixtures/rust/projects/macros-and-procedural-macros/`

## Difficulty

complex

## Assigned coverage

- **RS-CAN-PROJ-013** — a procedural-macro crate exporting function-like, derive,
  and attribute entry points with host-only dependencies, used from a target
  crate.
- **RS-CAN-PROJ-018** — unit-returning and `Result`-returning mains, `no_main`,
  a generated test main, and one macro-rewritten main.
- **RS-CAN-MAC-001** — declarative arms, fragment families, nested repetitions,
  and item, expression, statement, and pattern output.
- **RS-CAN-MAC-002** — declarative scope, export, `macro_use` import, re-export,
  and shadowing.
- **RS-CAN-MAC-003** — declarative hygiene, mixed-site resolution, and `$crate`
  through a downstream re-export.
- **RS-CAN-MAC-004** — macro-generated structs, modules, impls, statics, and
  companion items referenced from ordinary code.
- **RS-CAN-MAC-005** — derive macros and registered helper attributes over generic
  input, consumed through a bound.
- **RS-CAN-MAC-006** — attribute and function-like procedural transformation
  producing item and expression output.
- **RS-CAN-MAC-007** — span-carrying generated identifiers and one caller
  resolution distinction that does not depend on diagnostic coordinates.
- **RS-CAN-MAC-008** — compiler built-in and contextual macros for source
  location, environment presence and absence, formatting, `cfg!`, and inclusion.
- **RS-CAN-ECO-009** — a hermetic function-like macro that parses embedded non-Rust
  syntax over deterministic offline data and produces typed output.

Named counterexamples (no matching arm, follow-set and recursion violations,
duplicate generated output, a private-helper failure, an invalid generated
program, an expansion-time invalid input) are preserved as documentation beside
their valid demonstrations.

## Declared build contexts

- Default: `x86_64-unknown-linux-gnu`, edition 2024, stable 1.97.1.
- Editions 2015, 2018, and 2021 for legacy `macro_use`, cross-edition macro
  invocation, and fragment-grammar differences.
- Host procedural-macro crates (`proc-macro = true`) compiled for the host during
  the same build.

## Dependency needs

No third-party dependency. Procedural-macro members build against the standard
`proc_macro` crate; the embedded-DSL macro parses its input and reads its
deterministic offline data without an external parser crate. All macro data files
are checked in.

## Generated-source needs

None as tracked source. Declarative and procedural macros expand during
compilation and write no committed generated files.

## Planned tests

One planned test. A test target whose libtest-generated main supplies the
generated-test-main portion of RS-CAN-PROJ-018 coverage, which only exists under
the test compilation. No other test source is planned.

## Required interfaces

Provide `Taskfile.yml` with `build`, `lint`, and `test` as defined in the ground
rules, and `coverage.md` mapping each assigned identifier to its stable source
location.
