---
relationships:
  references:
    - ground-rules
    - reports/go/synthesis/checklist
    - fixtures/go/plan/overview
---

# Project brief: `plugin-registry`

## Purpose

A multi-command application with a runtime provider registry. It is the primary
owner of executable-entry semantics, initialization-time registration,
reflection-based construction, struct-tag-driven behavior, template name-driven
access, plugin loading, and a generated schema binding.

## Exclusive directory

`fixtures/go/projects/plugin-registry/`

## Difficulty

complex

## Assigned canonical identifiers and distinct valid variants

- **GO-CAN-DEC-004** — executable entry and multiple commands: several
  `package main` directories with valid `func main()` and shared libraries, plus
  a legal test of a main package.
- **GO-CAN-DYN-001** — reflection-based construction and member access using
  `Type`, `Value`, `TypeFor`, `New`, `FieldByName`, `MethodByName`, conversion,
  interface tests, and mutation, across exported, unexported, addressable, and
  settable cases, with dynamic name input.
- **GO-CAN-DYN-002** — struct-tag-driven behavior: one struct with multiple
  consumer tags reflected through valid `StructTag` syntax, including rename,
  omit, and embedded-field promotion, with tags affecting type identity.
- **GO-CAN-DYN-003** — initialization-time registration of two providers in `init`
  selected by runtime key, one provider enabled only by blank import or build
  tag, contrasted with explicit registration.
- **GO-CAN-DYN-004** — plugins: on the supported host, a plugin built and loaded
  with function and variable symbols resolved by string, with duplicate-open and
  init-once behavior.
- **GO-CAN-DYN-007** — generated schemas and protocol bindings: a small
  checked-in generated binding paired with its source schema, used through
  generated and reflective entry points, with generator provenance.
- **GO-CAN-DYN-008** — template and name-driven access: a template resolving
  exported fields, methods, and registered functions by string, including method
  error return and embedded promotion, with dynamic template text.

## Declared build contexts

- Default context.
- `plugin` build mode with `CGO_ENABLED=1` on the default operating system and
  architecture for GO-CAN-DYN-004.
- A build tag that enables the tag-selected provider for GO-CAN-DYN-003.

## Dependency needs

Standard library and project-local packages only. No third-party dependency. The
plugin and registry providers are local packages within this project's directory.

## Generated-source needs

Required for GO-CAN-DYN-007. The project commits the generator source, the non-Go
source schema, the standard generated-code marker, and the generated Go output.
A separate `generate` task may drive generator invocation; the required validation
tasks never regenerate source. GO-CAN-DYN-004 may also require a `plugin` build
step that is separate from the validation tasks.

## Planned tests

- One test creates the GO-CAN-DEC-004 coverage for "a legal test of a main
  package," demonstrating that a `package main` directory can carry a valid test.

Other coverage is created by command, library, generator-output, and plugin
source rather than by test source.

## Required `Taskfile.yml` and `coverage.md` interfaces

- `Taskfile.yml` defines `build`, `lint`, and `test` as thin wrappers over
  ordinary Go commands, plus an optional non-validation `generate` task and any
  separate plugin build step. `build` compiles all fixture-owned packages in every
  declared build context, including the plugin build mode and the tag-selected
  provider. `lint` checks `gofmt` formatting and runs `go vet` on fixture-owned
  packages; generated output remains subject to `gofmt` and `go vet`. `test` runs
  Go tests and succeeds with no test files. The three validation tasks pass, write
  no logs, evidence, manifests, or hashes, and never regenerate source.
- `coverage.md` is a Markdown table recording, per assigned identifier, the
  canonical identifier, a stable source path, the named declaration, directive,
  package, module, or test that creates coverage, the build context when non-
  default, and additional locators for distinct valid variants. It uses stable
  names, not line numbers.
