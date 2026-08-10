---
relationships:
  references:
    - ground-rules
    - reports/go/synthesis/checklist
    - fixtures/go/plan/overview
---

# Project brief: `package-structure`

## Purpose

A coherent, idiomatic library deliberately structured to demonstrate how Go forms
packages from multiple files, resolves scopes and declarations, and orders
initialization. It is the primary owner of the source, import, scope, and
declaration semantics that recur throughout the corpus.

## Exclusive directory

`fixtures/go/projects/package-structure/`

## Difficulty

complex

## Assigned canonical identifiers and distinct valid variants

- **GO-CAN-SRC-001** — multi-file package formation. Valid variants: distinct
  per-file import sets and aliases; a declaration used before its textual point;
  contrast with an external `_test` package.
- **GO-CAN-SRC-002** — package identity versus import path and directory name.
  Valid variants: an import path whose last element differs from the package
  clause, imported by default name and by alias; two same-named packages
  distinguished by alias.
- **GO-CAN-SRC-003** — scopes and declaration points. Valid variants: universe,
  package, file, function, label, and nested block scopes; legal shadowing;
  parameter and named-result scope.
- **GO-CAN-SRC-004** — universe identifiers and shadowing. Valid variants:
  shadowing of representative predeclared type, function, constant, and
  `nil`-adjacent names, including `any`, `comparable` (go1.18), and `clear`,
  `min`, `max` (go1.21), with unshadowed uses retained elsewhere.
- **GO-CAN-SRC-005** — exportedness and Unicode. Valid variants: exported and
  unexported types, values, functions, fields, and methods; a non-ASCII first
  rune; distinct unexported members across packages.
- **GO-CAN-SRC-006** — blank identifier. Valid variants: `_` in imports,
  assignments, returns, fields, range clauses, and compile-time interface
  assertions.
- **GO-CAN-SRC-007** — labels and their separate namespace. Valid variants:
  labels with `break`, `continue`, and `goto`, alongside a same-spelled variable.
- **GO-CAN-SRC-008** — unused-name validity. Valid variants: valid unused package
  declarations, parameters, and results; `_ = x` suppression; an assignment-only
  local that remains unused.
- **GO-CAN-SRC-009** — lexing, comments, semicolon insertion, literals, and
  source positions. Valid variants: ordinary and doc comments; numeric literal
  variants including digit separators and octal spellings; automatic semicolons;
  a near-miss line comment that stays an ordinary comment; valid `//line` and
  `/*line*/` directives that remap logical positions while physical positions are
  preserved (both position kinds).
- **GO-CAN-IMP-001** — import declaration forms and file namespace. Valid
  variants: default-name, explicit-alias, dot, and blank import forms across
  separate files, each file-scoped and participating in cycle checks.
- **GO-CAN-DEC-001** — declaration forms and dependency visibility. Valid
  variants: grouped and ungrouped `const`, `var`, `type`, and `func`; forward and
  mutual references; whole-package order independence contrasted with lexical
  local scope.
- **GO-CAN-DEC-002** — package-variable initialization dependencies. Valid
  variants: direct and through-function dependencies across files with observable
  side effects; independent variables; a legal cycle broken by assignment in
  `init`.
- **GO-CAN-DEC-003** — multiple `init` functions and imported-package
  initialization. Valid variants: several local `init` functions; a diamond
  import graph with a blank-imported registrar; imported packages initialized
  once before importers.
- **GO-CAN-DEC-005** — short declarations and partial redeclaration. Valid
  variants: `:=` with new and existing names, multi-value expressions, at least
  one new non-blank variable per block, and initializer scope.
- **GO-CAN-DEC-006** — function-local named types. Valid variants: local defined
  types used in identity and conversions; contrast with a package-defined
  equivalent and an alias.

## Declared build contexts

- Default context only.
- Language-version selection through `go` directives for the predeclared
  identifiers introduced at go1.18 and go1.21 (GO-CAN-SRC-004).

## Dependency needs

Standard library and project-local packages only. No third-party dependency.

## Generated-source needs

None.

## Planned tests

None. Coverage is created by library source. The external `_test` package contrast
named under GO-CAN-SRC-001 is a package-clause demonstration, not a behavioral
test; primary test-mechanics coverage belongs to `behavior-suite`.

## Required `Taskfile.yml` and `coverage.md` interfaces

- `Taskfile.yml` defines `build`, `lint`, and `test` as thin wrappers over
  ordinary Go commands. `build` compiles all fixture-owned packages in the
  declared context. `lint` checks `gofmt` formatting and runs `go vet` on
  fixture-owned packages. `test` runs Go tests and succeeds with no test files.
  All three pass and write no logs, evidence, manifests, or hashes.
- `coverage.md` is a Markdown table that records, for each assigned identifier,
  the canonical checklist identifier, a stable source path, the named
  declaration, directive, package, module, or test that creates the coverage, the
  build context when it differs from the default, and additional locators when
  distinct valid variants require them. It uses stable names, not line numbers.
