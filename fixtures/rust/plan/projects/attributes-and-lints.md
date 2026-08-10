# Project brief: attributes-and-lints

## Purpose

Demonstrate Rust's attribute and lint surface: inner, outer, active, inert,
helper, and tool attributes; conditional-compilation removal and its subparts;
built-in target cfg and checked custom cfg; diagnostic and API-shaping
attributes; and lint levels, scopes, expectations, and configuration.

## Exclusive directory

`fixtures/rust/projects/attributes-and-lints/`

## Difficulty

routine

## Assigned coverage

- **RS-CAN-ATTR-001** — crate and module inner attributes and outer item, field,
  and variant attributes across cfg, `cfg_attr`, derive helper, lint, repr, doc,
  and a registered tool namespace.
- **RS-CAN-ATTR-002** — `cfg` on modules, items, fields, variants, arms, impls,
  and supported subexpression positions, and `cfg_attr` injection of path,
  derive, repr, and no-std attributes.
- **RS-CAN-ATTR-003** — mutually exclusive shipped target branches, major target
  cfg families, a declared custom cfg, and a retained unexpected-cfg warning
  (excluding the literal `cfg(true)`/`cfg(false)` research-gap variant).
- **RS-CAN-ATTR-004** — scoped lint levels, deprecated and must-use sites,
  cross-crate non-exhaustive types, `track_caller`, and semantic versus hint
  codegen attributes.
- **RS-CAN-DIAG-001** — allow, warn, deny, forbid, and expect levels at crate,
  module, and item scopes, fulfilled and unfulfilled expectations, workspace lint
  inheritance, and command-line deny and cap.

Named counterexamples (an invalid attribute position, an unknown tool namespace,
a `forbid` override, a `cfg!` wrong-platform reference) are preserved as
documentation beside their valid demonstrations, and only host cfg branches
compile.

## Declared build contexts

- Default: `x86_64-unknown-linux-gnu`, edition 2024, stable 1.97.1.
- `cfg`-selected non-host target branches, compiled only on the host edge.
- A cross-crate boundary (a local downstream crate) for cross-crate non-exhaustive
  and lint-cap effects.
- A workspace context for lint-table inheritance.

## Dependency needs

No third-party dependency. Cross-crate and workspace effects use local crates.

## Generated-source needs

None.

## Planned tests

None.

## Required interfaces

Provide `Taskfile.yml` with `build`, `lint`, and `test` as defined in the ground
rules, where `lint` remains consistent with the intentional lint-level source, and
`coverage.md` mapping each assigned identifier to its stable source location.
