VERDICT: REJECT

# Rust fixture plan review

## Blocking findings

### RUST-PLAN-001 — P1 — Installed target is classified as unavailable

- **Plan anchors:** `fixtures/rust/plan/overview.md:61-68` and
  `fixtures/rust/plan/overview.md:207-209`.
- **Checklist anchor:** `reports/rust/synthesis/checklist.md:1144-1150`.
- **Violated ground rule:** `fixtures/rust/ground-rules.md:22-35` requires build
  contexts and exceptions to reflect the selected installed toolchain.
- **Finding:** The plan says every non-host `rustup` target is uninstalled and
  excludes RS-CAN-ECO-008 on that basis. Local inspection finds a stable Rust
  1.97.1 rustup toolchain at the same compiler commit as the selected Homebrew
  build, with `wasm32-unknown-unknown` and `aarch64-unknown-linux-gnu` installed.
  The current unavailable-context accounting is therefore false. Reconcile the
  selected toolchain, declared context matrix, and RS-CAN-ECO-008 disposition
  with the installed state.

### RUST-PLAN-002 — P1 — Compile-fail doctest is planned as fixture content

- **Plan anchor:**
  `fixtures/rust/plan/projects/tests-and-documentation.md:29-33`.
- **Checklist anchor:** `reports/rust/synthesis/checklist.md:1078-1084`.
- **Violated ground rule:** `fixtures/rust/ground-rules.md:41-50` requires all
  corpus source to be valid and keeps expected-failure programs and
  intentionally invalid examples out of fixture content.
- **Finding:** The brief says a compile-fail documentation block and
  failure-expecting rustdoc annotations remain corpus content when the enclosing
  crate and test command pass. A `compile_fail` doctest is intentionally invalid
  synthetic Rust source, so harness success does not make it valid fixture
  source. Keep this invalid-only variant as documentation or exception rather
  than planned test source.

## Accounting and policy checks

- Canonical accounting: 141 identifiers; 136 unique primary assignments; five
  allowed whole-item exceptions; no duplicate or missing primary assignment
  outside those exceptions.
- Overview assignments and project-brief assignments match exactly.
- Sixteen exclusive project directories are distinct, and no brief depends on
  another top-level fixture project.
- Planned tests other than RUST-PLAN-002 name checklist coverage created by test
  source.
- Third-party dependencies name their coverage purpose and require pinning,
  vendoring, provenance, licensing, notices, and offline metadata.
- Generated-source handling keeps tracked source unchanged during required
  validation tasks.

## Installed-toolchain observations

- PATH toolchain: Homebrew `rustc` 1.97.1 commit `8bab26f4f`, Cargo 1.97.1,
  rustfmt 1.9.0, and Clippy 0.1.97 on `x86_64-unknown-linux-gnu`.
- rustup stable toolchain: `rustc` 1.97.1 commit `8bab26f4f`, with host,
  `aarch64-unknown-linux-gnu`, and `wasm32-unknown-unknown` targets installed.
- No nightly toolchain is installed.
