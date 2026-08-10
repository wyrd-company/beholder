# Project brief: async-runtime-and-registration

## Purpose

Demonstrate ecosystem attribute-macro rewriting and link-time registration: how
a runtime entry or test attribute and a body-wrapping instrumentation attribute
rewrite async mains, tests, and functions into wrappers, runtimes, and spans; and
how a registration crate collects distributed values through linker sections
without direct source references.

## Exclusive directory

`fixtures/rust/projects/async-runtime-and-registration/`

## Difficulty

complex

## Assigned coverage

- **RS-CAN-ECO-003** — one runtime entry or test attribute and one body-wrapping
  instrumentation attribute, comparing the source and the compiled semantic shape.
- **RS-CAN-ECO-007** — a registration crate with producers in separate modules or
  crates and a consumer iterating the collected set.

Named counterexamples (a `Send + 'static` spawn requirement, a dead or unlinked
producer omitted from the set, a duplicate registration) are preserved as
documentation beside their valid demonstrations. Version-specific hidden macro
output is documentation-level.

## Declared build contexts

- Default: `x86_64-unknown-linux-gnu`, edition 2024, stable 1.97.1.
- The host ELF linker sections used by the registration crate.
- A dependency-feature-gated runtime context.

## Dependency needs

Three third-party dependencies:

- `tokio`, restricted to the runtime and macro features it needs, for the runtime
  entry and test attribute in RS-CAN-ECO-003.
- `tracing`, for the instrumentation attribute in RS-CAN-ECO-003.
- `inventory`, for the link-time registration in RS-CAN-ECO-007.

Each is pinned, vendored with source location, license, notices, and offline
metadata, including any transitive crates required for an offline build. No
vendored crate is shared with another project.

## Generated-source needs

None as tracked source. The attribute macros expand during compilation.

## Planned tests

One planned test. A runtime test-attribute target supplies the test-attribute
portion of RS-CAN-ECO-003 coverage, whose rewritten entry only exists under the
test compilation. No other test source is planned.

## Required interfaces

Provide `Taskfile.yml` with `build`, `lint`, and `test` as defined in the ground
rules, where `build` resolves the vendored dependencies offline, and `coverage.md`
mapping each assigned identifier to its stable source location.
