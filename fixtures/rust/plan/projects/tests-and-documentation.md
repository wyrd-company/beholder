# Project brief: tests-and-documentation

## Purpose

Demonstrate Rust's test and documentation compilation model, where the test and
documentation source is itself the coverage: unit and integration test crate
boundaries; harness attributes, generated mains, examples, and benches; and
documentation tests with documentation source inclusion.

## Exclusive directory

`fixtures/rust/projects/tests-and-documentation/`

## Difficulty

complex

## Assigned coverage

- **RS-CAN-TEST-001** — a unit test accessing a private ancestor, an integration
  test crate seeing only public API, multiple integration targets, and a shared
  support module.
- **RS-CAN-TEST-002** — passing, ignored, expected-panic, and `Result`-returning
  tests, a required-feature example, and harness-less test and bench targets with
  explicit mains.
- **RS-CAN-TEST-003** — runnable, no-run, ignored, should-panic, hidden-line, and
  edition-tagged documentation tests and crate-level documentation inclusion.

Named counterexamples (a private item rejected from an integration test, a
compile-fail documentation block, an accidental helper target, a panic-abort
harness conflict) are preserved as documentation beside their valid
demonstrations. Failure-expecting rustdoc annotations remain valid corpus content
because the enclosing crate compiles and `test` passes.

## Declared build contexts

- Default: `x86_64-unknown-linux-gnu`, edition 2024, stable 1.97.1.
- Library builds with and without the test configuration.
- Integration, example, bench, and documentation-test compilations.
- An edition-tagged documentation-test context.

## Dependency needs

No third-party dependency. Harness-less bench and test targets provide their own
entry points; no benchmark framework crate is required.

## Planned tests

Planned. The unit, integration, example, bench, and documentation test source is
the assigned coverage: unit and integration test source creates the RS-CAN-TEST-001
crate-boundary coverage; test, example, and bench target source creates the
RS-CAN-TEST-002 harness and generated-main coverage; documentation-test and
documentation-inclusion source creates the RS-CAN-TEST-003 coverage.

## Generated-source needs

None as tracked source. Libtest and rustdoc synthesize their mains and synthetic
crates during `test`; no committed generated files result.

## Required interfaces

Provide `Taskfile.yml` with `build`, `lint`, and `test` as defined in the ground
rules, where `test` compiles and runs the unit, integration, and documentation
test source, and `coverage.md` mapping each assigned identifier to its stable
source location.
