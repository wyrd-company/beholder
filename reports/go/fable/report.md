# Go — Code-Graph Fixture Research Checklist

Independent research pass on the Go programming language. The checklist below
enumerates language, toolchain, build, package, and ecosystem features that a
broad, idiomatic, self-contained reference project should exercise so that a
code-graph generator's accuracy and completeness can be evaluated against it.
Items are included when omitting them would leave materially distinct semantic
entities, identities, relationships, resolution rules, type behavior,
visibility, dispatch, source inclusion, or diagnostics unrepresented.

---

## 1. Baseline Version and Implementation Assumptions

- **Language/toolchain baseline:** Go 1.25 (released August 2025), the `gc`
  toolchain distributed by the Go project (`cmd/compile`, `cmd/link`, the `go`
  command). Module mode is assumed (`GO111MODULE` is effectively always `on`
  since Go 1.16; GOPATH build mode is legacy).
- **Alternative implementations:** `gccgo` and TinyGo exist and diverge in
  pragmas, supported platforms, unsafe details, and some runtime behavior.
  This report assumes `gc` unless stated.
- **Language versioning model:** Go's semantics are versioned per-module via
  the `go` directive in `go.mod`, and (since Go 1.21) per-file via
  `//go:build go1.N` constraints. The same source text can have different
  semantics under different declared language versions (see DECL-09, MOD-01).
- **Platform assumption:** linux/amd64 as the default target, with
  cross-compilation via `GOOS`/`GOARCH` treated as first-class configuration
  variance (see the PLAT category).
- **Compatibility promise:** Go 1.x source compatibility is strong; almost all
  behavior below is stable across 1.18–1.25 except where flagged in §4.

---

## 2. Category Overview

| Prefix | Category |
| --- | --- |
| MOD | Modules, workspaces, and dependency resolution |
| PKG | Packages, directories, and source-file organization |
| BLD | Build constraints and conditional compilation |
| DECL | Declarations, scope, and name resolution |
| TYPE | Types, type identity, and conversions |
| IFACE | Interfaces, method sets, and dispatch |
| GEN | Generics (type parameters) |
| FUNC | Functions, closures, control flow, entry points |
| IMP | Imports and inter-package edges |
| TEST | Testing, benchmarks, fuzzing, examples |
| GENC | Generated code and embedded assets |
| DIR | Compiler and linker directives |
| CGO | Cgo and foreign-function surface |
| PLAT | Platform and configuration variance |
| DIAG | Invalid programs and diagnostics |
| DYN | Dynamic and reflective escape hatches |
| ECO | Ecosystem conventions |

---

## 3. Checklist

### MOD — Modules, workspaces, and dependency resolution

#### MOD-01 · Module identity and the `go` directive

- **Behavior:** The `module` line in `go.mod` defines the import-path prefix
  and identity of every package in the module. The `go` directive sets the
  module's language version, which changes actual semantics (e.g.,
  per-iteration loop variables apply only at `go >= 1.22`).
- **Observable content:** A `go.mod` with `module`, `go`, and `require`
  directives; packages imported by paths derived from the module path.
- **Variants & failure cases:** Module path that does not match any hosting
  URL (self-contained fixture); `go` directive older than the toolchain
  (semantics downgrade); missing `go` directive (implied old version, pre-1.16
  behavior assumptions).
- **Constraints:** Module mode; go command ≥ 1.16 for current defaults.
- **Defined by:** Official implementation (go command); language semantics
  coupling defined in the spec since 1.21.
- **Confidence:** High.
- **Primary source:** Go Modules Reference (go.dev/ref/mod).

#### MOD-02 · Semantic import versioning (`/v2+` major-version suffix)

- **Behavior:** Major versions ≥ 2 carry the version in the module path and
  therefore in every import path. `example.com/m` and `example.com/m/v2` are
  distinct modules with distinct package identities that can coexist in one
  build, even when both packages share the same package name.
- **Observable content:** A dependency (or nested module) at `/v2`, imported
  alongside or instead of v1; standard-library analogue `math/rand` vs
  `math/rand/v2` imported in the same file with an alias.
- **Variants & failure cases:** `go.mod` module path missing the `/v2` suffix
  while tagged v2+ is invalid; importing `/v2` path but referring to the
  unsuffixed package name in code.
- **Constraints:** Module mode; go 1.22+ for `math/rand/v2`.
- **Defined by:** Official implementation (go command).
- **Confidence:** High.
- **Primary source:** Go Modules Reference (go.dev/ref/mod).

#### MOD-03 · `replace` directives

- **Behavior:** `replace` redirects a module path (optionally at a version) to
  another module path/version or to a local filesystem directory, changing
  which source files back an import path.
- **Observable content:** A `replace example.com/dep => ./local/dep` entry
  with a sibling directory containing its own `go.mod`.
- **Variants & failure cases:** Filesystem replace without version; replace of
  a transitive dependency; replaces are only honored in the main module (not
  when the module is consumed as a dependency).
- **Constraints:** Main-module-only effect; ignored under some `go install
  pkg@version` flows.
- **Defined by:** Official implementation.
- **Confidence:** High.
- **Primary source:** Go Modules Reference (go.dev/ref/mod).

#### MOD-04 · Multi-module workspaces (`go.work`)

- **Behavior:** A `go.work` file with `use` directives builds several modules
  together, overriding normal module resolution for the used modules; it can
  also carry workspace-level `replace` directives.
- **Observable content:** Umbrella directory with `go.work` and two or more
  modules that import each other without `replace` lines in their `go.mod`s.
- **Variants & failure cases:** Build behavior differs with `GOWORK=off`; a
  package resolving to different sources inside vs outside the workspace;
  `go work vendor` (1.22+).
- **Constraints:** Go 1.18+.
- **Defined by:** Official implementation.
- **Confidence:** High.
- **Primary source:** Go Modules Reference (go.dev/ref/mod); Go 1.18 release notes.

#### MOD-05 · Vendoring

- **Behavior:** A `vendor/` tree plus `vendor/modules.txt` redirects source
  inclusion: with `-mod=vendor` (automatic when `vendor/` exists and the `go`
  version is ≥ 1.14), dependency packages are read from `vendor/`, not the
  module cache. Vendored file sets can be pruned (test files omitted).
- **Observable content:** A committed `vendor/` directory whose contents are
  the actual compiled sources for dependencies.
- **Variants & failure cases:** Inconsistent `modules.txt` vs `go.mod` is an
  error; `-mod=mod` ignores vendor; graphs that scan the repo see dependency
  code twice (cache path vs vendor path) unless they model the switch.
- **Constraints:** go command behavior; not part of the language.
- **Defined by:** Official implementation.
- **Confidence:** High.
- **Primary source:** Go Modules Reference (go.dev/ref/mod).

#### MOD-06 · Nested modules exclude subtrees

- **Behavior:** A subdirectory containing its own `go.mod` is not part of the
  enclosing module; its packages are unreachable from the parent unless
  required/replaced as a separate module.
- **Observable content:** `./sub/go.mod` inside the main module; parent build
  ignores `./sub` packages; a `require` + `replace` (or `go.work use`) makes
  them visible again.
- **Variants & failure cases:** Import of a nested-module package without
  wiring it up fails resolution even though the source is physically present.
- **Constraints:** Module mode.
- **Defined by:** Official implementation.
- **Confidence:** High.
- **Primary source:** Go Modules Reference (go.dev/ref/mod).

#### MOD-07 · Minimal version selection, `require`, `// indirect`, `go.sum`

- **Behavior:** The go command selects dependency versions by minimal version
  selection over the module graph. `// indirect` marks requirements not
  imported directly. `go.sum` pins content hashes; mismatches abort builds.
  Test-only dependencies appear in `go.mod` like ordinary ones.
- **Observable content:** `go.mod` with direct and indirect requirements; a
  dependency used only from `_test.go` files.
- **Variants & failure cases:** Module-graph pruning differences between `go`
  directive versions (1.17+ pruned graphs); missing `go.sum` entries fail with
  a distinctive diagnostic.
- **Constraints:** Module mode; network or module cache availability.
- **Defined by:** Official implementation.
- **Confidence:** High.
- **Primary source:** Go Modules Reference (go.dev/ref/mod).

#### MOD-08 · `toolchain` directive and automatic toolchain switching

- **Behavior:** Since Go 1.21, `go.mod` may name a `toolchain`; a go command
  older than the required version can download and re-exec the newer
  toolchain. Which toolchain runs changes available language features.
- **Observable content:** `toolchain go1.25.x` line in `go.mod`.
- **Variants & failure cases:** `GOTOOLCHAIN=local` disables switching; a
  `go` directive newer than the installed toolchain refuses to build under
  `local`.
- **Constraints:** Go 1.21+.
- **Defined by:** Official implementation.
- **Confidence:** High.
- **Primary source:** Go toolchain documentation (go.dev/doc/toolchain); Go 1.21 release notes.

#### MOD-09 · `exclude` and `retract` directives

- **Behavior:** `exclude` removes specific dependency versions from version
  selection in the main module. `retract` (in a dependency's `go.mod`) marks
  published versions as withdrawn, steering resolution away from them.
- **Observable content:** `exclude` entry in the main `go.mod`; a retracted
  version in a local replaced dependency.
- **Variants & failure cases:** Retractions only take effect via the latest
  version's `go.mod`; excluded versions may still appear in `go.sum` history.
- **Constraints:** Module mode; go 1.16+ for retract.
- **Defined by:** Official implementation.
- **Confidence:** High.
- **Primary source:** Go Modules Reference (go.dev/ref/mod).

#### MOD-10 · Import path ≠ package name

- **Behavior:** The package name used in code comes from the imported
  package's `package` clause, not from the import path's last element.
  Resolution of qualified identifiers therefore requires reading the target
  package's source.
- **Observable content:** A dependency whose directory/base path differs from
  its package name (e.g., a path ending in `client-go` declaring
  `package clientgo`, or a `yaml.v2`-style path declaring `package yaml`);
  call sites using the declared name without an alias.
- **Variants & failure cases:** Tools that guess package name from path
  mis-resolve every selector; aliasing at import restores an explicit local
  name.
- **Constraints:** None.
- **Defined by:** Language.
- **Confidence:** High.
- **Primary source:** Go Language Specification (go.dev/ref/spec), Import declarations.

#### MOD-11 · `tool` directive (tracked tool dependencies)

- **Behavior:** Go 1.24 added `tool` directives in `go.mod` so executable
  dependencies (code generators, linters) are version-locked and runnable via
  `go tool <name>`, replacing the `tools.go` blank-import convention.
- **Observable content:** `tool example.com/cmd/gen` in `go.mod`; a
  `go:generate` line invoking `go tool gen`.
- **Variants & failure cases:** Pre-1.24 projects use a build-tagged
  `tools.go` with blank imports instead (see ECO-04); both patterns create
  module-graph edges without normal import edges from library code.
- **Constraints:** Go 1.24+.
- **Defined by:** Official implementation.
- **Confidence:** High.
- **Primary source:** Go 1.24 release notes; Go Modules Reference (go.dev/ref/mod).

#### MOD-12 · `ignore` directive

- **Behavior:** Go 1.25 added an `ignore` directive to `go.mod` that tells the
  go command to skip listed directories entirely (useful for large non-Go
  asset trees), changing which source directories are even considered.
- **Observable content:** `ignore ./assets` in `go.mod` with `.go` files under
  that path that must not become packages.
- **Variants & failure cases:** Without the directive, stray `.go` files in
  asset trees become (possibly broken) packages during `./...` walks.
- **Constraints:** Go 1.25+.
- **Defined by:** Official implementation.
- **Confidence:** Medium (recent feature; details of matching semantics).
- **Primary source:** Go 1.25 release notes; Go Modules Reference (go.dev/ref/mod).

---

### PKG — Packages, directories, and source-file organization

#### PKG-01 · Package clause vs directory name

- **Behavior:** A package's import path is its directory path; its name in
  code is the `package` clause. All non-test files in a directory must declare
  the same package name, which may differ from the directory name.
- **Observable content:** A directory `stringutil2/` declaring
  `package stringutil`; importers referencing `stringutil.X`.
- **Variants & failure cases:** Two different package clauses in one directory
  is a build error ("found packages A and B"); the sole sanctioned exception
  is the external test package (TEST-02).
- **Constraints:** None.
- **Defined by:** Language + official implementation (directory mapping is the
  go command's).
- **Confidence:** High.
- **Primary source:** Go Language Specification; go command documentation (pkg.go.dev/cmd/go).

#### PKG-02 · `internal/` visibility enforcement

- **Behavior:** A package under a path element `internal` is importable only
  by packages rooted at the parent of that `internal` directory. This is an
  import-graph visibility rule enforced by the go command, orthogonal to
  identifier exportedness.
- **Observable content:** `internal/` package imported from within the module;
  a nested `a/internal/b` importable from `a/...` but not from `c/...`.
- **Variants & failure cases:** Violating import fails with "use of internal
  package ... not allowed"; multiple `internal` elements nest the rule; the
  rule also shapes which standard-library packages are reachable.
- **Constraints:** Enforced by the go command (not the compiler alone).
- **Defined by:** Official implementation.
- **Confidence:** High.
- **Primary source:** go command documentation (pkg.go.dev/cmd/go, "Internal Directories").

#### PKG-03 · Ignored files and directories (`testdata`, `_*`, `.*`)

- **Behavior:** The go command ignores directories named `testdata` and any
  file or directory whose name starts with `_` or `.`. Contents there are
  never compiled but are commonly read at run time by tests.
- **Observable content:** `testdata/` holding fixtures (including `.go` files
  that must not join the package); an `_attic/old.go` file excluded from the
  build.
- **Variants & failure cases:** A graph that globs `**/*.go` will include
  symbols the toolchain never compiles; `testdata` Go files may be
  intentionally invalid.
- **Constraints:** go command behavior.
- **Defined by:** Official implementation.
- **Confidence:** High.
- **Primary source:** go command documentation (pkg.go.dev/cmd/go).

#### PKG-04 · `package main` and multiple commands

- **Behavior:** `package main` with `func main()` defines an executable; main
  packages cannot be imported. A module commonly holds several main packages
  (conventionally under `cmd/<name>/`), each producing a distinct binary that
  links a distinct subset of the package graph.
- **Observable content:** Two or more `cmd/*/main.go` programs sharing library
  packages; an attempted import of a main package failing.
- **Variants & failure cases:** `main` without `func main` fails at link;
  tests inside `package main` are legal (TEST-01 variant).
- **Constraints:** None.
- **Defined by:** Language (program execution) + implementation (import ban).
- **Confidence:** High.
- **Primary source:** Go Language Specification, Program execution.

#### PKG-05 · File-scoped imports across a multi-file package

- **Behavior:** Imports are declared per file, not per package. Different
  files of one package may import different packages, use different aliases
  for the same package, or use the same alias for different packages.
- **Observable content:** A package split across ≥3 files where a type is
  declared in one file, its methods in another, and each file has a distinct
  import block (including conflicting aliases across files).
- **Variants & failure cases:** Resolution of a qualified identifier must be
  per-file; package-level identifiers resolve across all files of the package.
- **Constraints:** None.
- **Defined by:** Language.
- **Confidence:** High.
- **Primary source:** Go Language Specification, Declarations and scope.

#### PKG-06 · Package documentation and `doc.go`

- **Behavior:** The doc comment preceding a `package` clause documents the
  package; convention places package docs in a dedicated `doc.go`. Only one
  file's package comment is conventionally authoritative.
- **Observable content:** `doc.go` containing only a package comment and the
  package clause.
- **Variants & failure cases:** Multiple files with package comments (gets
  concatenated by doc tools in file order); `//go:build`-excluded doc files.
- **Constraints:** Convention; doc extraction defined by go/doc.
- **Defined by:** Ecosystem convention + official tooling.
- **Confidence:** High.
- **Primary source:** Go Doc Comments (go.dev/doc/comment).

#### PKG-07 · Exportedness by identifier case, including Unicode

- **Behavior:** An identifier is exported iff its first character is an
  uppercase letter (Unicode class Lu) and it is declared at package scope or
  is a field/method name. Identifiers beginning with characters from
  caseless scripts (e.g., CJK) can never be exported.
- **Observable content:** Exported/unexported pairs of every entity kind
  (type, func, method, field, const, var); a Unicode identifier (e.g., a
  Greek-letter unexported name) demonstrating the Lu rule.
- **Variants & failure cases:** Unexported fields block cross-package unkeyed
  composite literals and reflection-based setting; unexported methods still
  participate in interface satisfaction (IFACE-07).
- **Constraints:** None.
- **Defined by:** Language.
- **Confidence:** High.
- **Primary source:** Go Language Specification, Exported identifiers.

#### PKG-08 · Order-independent package-level declarations

- **Behavior:** Package-scope identifiers may be used before their
  declaration, across files. There is no forward-declaration mechanism and no
  header/order constraint; resolution is whole-package.
- **Observable content:** A function in file A calling a function declared
  later in file B; mutually recursive functions and types split across files.
- **Variants & failure cases:** Local (block) scope is strictly lexical by
  contrast — using a local before its declaration is an error; initializer
  dependency cycles among package vars are errors (DIAG-03).
- **Constraints:** None.
- **Defined by:** Language.
- **Confidence:** High.
- **Primary source:** Go Language Specification, Declarations and scope.

---

### BLD — Build constraints and conditional compilation

#### BLD-01 · `//go:build` expressions (and legacy `// +build`)

- **Behavior:** A `//go:build` line before the package clause includes or
  excludes the whole file per target/tag expression using `&&`, `||`, `!`, and
  parentheses. Legacy `// +build` lines are still honored and must agree when
  both are present (gofmt keeps them in sync).
- **Observable content:** Files with constraints like
  `//go:build linux && amd64`, `//go:build !windows`, and a file carrying both
  syntaxes.
- **Variants & failure cases:** A constraint placed after the package clause
  is inert (vet's `buildtag` check flags it); mutually exclusive files
  declaring the same symbol (BLD-08).
- **Constraints:** `//go:build` requires Go 1.17+; blank line must separate
  the constraint from the package clause.
- **Defined by:** Official implementation (go/build).
- **Confidence:** High.
- **Primary source:** go command documentation, Build constraints (pkg.go.dev/cmd/go and pkg.go.dev/go/build).

#### BLD-02 · Filename-suffix constraints

- **Behavior:** File names of the form `name_GOOS.go`, `name_GOARCH.go`,
  `name_GOOS_GOARCH.go` (and `_test` before those) impose implicit
  constraints. The suffix must follow an underscore; a file named `linux.go`
  is unconstrained.
- **Observable content:** `file_linux.go`, `file_windows.go`,
  `file_linux_amd64.go`, plus a decoy `linux.go` with no constraint.
- **Variants & failure cases:** Suffix order is fixed (GOOS then GOARCH);
  `_test` combines (`x_linux_test.go`); unknown suffixes are not constraints.
- **Constraints:** go command file-selection rules.
- **Defined by:** Official implementation.
- **Confidence:** High.
- **Primary source:** go/build package documentation (pkg.go.dev/go/build).

#### BLD-03 · Custom and toolchain-defined build tags

- **Behavior:** `-tags` defines arbitrary tags selecting alternative file
  sets. The toolchain predefines tags: `cgo` (when cgo is enabled), `gc` /
  `gccgo` (compiler), `unix` (1.17+ umbrella for Unix-like GOOS), release
  tags `go1.N`, and instrumentation tags such as `race`, `msan`, `asan`.
- **Observable content:** A feature toggled by a custom tag (two files:
  `//go:build featurex` and `//go:build !featurex`); a `//go:build unix`
  file; a `//go:build race`-guarded helper.
- **Variants & failure cases:** GOOS implication chains (`android` also
  satisfies `linux`; `ios` also satisfies `darwin`); `GOEXPERIMENT` values
  surface as `goexperiment.*` tags.
- **Constraints:** Tag set depends on build invocation → the compiled program
  is configuration-dependent.
- **Defined by:** Official implementation.
- **Confidence:** High (medium on the full implication list).
- **Primary source:** go/build package documentation; go command documentation.

#### BLD-04 · Release tags and per-file language version

- **Behavior:** `//go:build go1.N` selects files by toolchain release. Since
  Go 1.21 it additionally sets that file's *language version*, so one package
  can mix files with different semantics (e.g., pre- and post-1.22 loop-variable
  scoping).
- **Observable content:** A file constrained `//go:build go1.22` alongside the
  module's `go` directive at a different version.
- **Variants & failure cases:** Downgrading a file below the version a used
  feature requires produces compile errors mentioning the language version.
- **Constraints:** Go 1.21+ for the language-version coupling.
- **Defined by:** Official implementation + spec appendix on language versions.
- **Confidence:** High.
- **Primary source:** Go 1.21 release notes; Go toolchain documentation (go.dev/doc/toolchain).

#### BLD-05 · `//go:build ignore` convention

- **Behavior:** An impossible tag (conventionally `ignore`) excludes a file
  from every ordinary build; such files are run explicitly (`go run gen.go`)
  as generators or one-off tools while living inside a package directory.
- **Observable content:** A `gen.go` with `//go:build ignore` and its own
  `package main`, referenced by a `//go:generate go run gen.go` line.
- **Variants & failure cases:** Any never-satisfied tag works; the file still
  parses as Go and defines symbols a naive scanner would merge into the
  directory's package.
- **Constraints:** Convention only — `ignore` is not special to the toolchain.
- **Defined by:** Ecosystem convention.
- **Confidence:** High.
- **Primary source:** go command documentation (generate section shows the pattern).

#### BLD-06 · Assembly files and body-less Go declarations

- **Behavior:** A `func` declared in Go without a body is legal when
  implemented elsewhere — typically a `.s` assembly file in the same package
  (symbols spelled `·Name` / `TEXT ·Name(SB)`), selected by GOARCH filename
  suffix.
- **Observable content:** `add_amd64.s` implementing `func Add(a, b int64) int64`
  declared body-less in a Go file, with a pure-Go fallback file for other
  architectures.
- **Variants & failure cases:** Missing implementation surfaces at link time;
  `//go:noescape` commonly annotates such declarations (DIR-02); without any
  provider the compiler rejects the body-less func unless assembly is present
  in the package.
- **Constraints:** gc toolchain assembler syntax; per-arch files.
- **Defined by:** Official implementation.
- **Confidence:** High.
- **Primary source:** Go assembler documentation (go.dev/doc/asm).

#### BLD-07 · `_test.go` suffix as a build-mode constraint

- **Behavior:** Files ending `_test.go` are compiled only by `go test`,
  forming a superset package (or a sibling external package, TEST-02). Symbols
  there are invisible to ordinary builds.
- **Observable content:** A helper declared only in `_test.go` used by tests;
  the same identifier free for other purposes in non-test builds.
- **Variants & failure cases:** Combining with platform suffixes and
  `//go:build` tags; test files of main packages.
- **Constraints:** go command behavior.
- **Defined by:** Official implementation.
- **Confidence:** High.
- **Primary source:** testing package documentation (pkg.go.dev/testing); go command documentation.

#### BLD-08 · One symbol, several conditional definitions

- **Behavior:** Mutually exclusive files may each define the same package-level
  identifier (function, const, type) with per-platform or per-tag bodies —
  a single logical entity with multiple conditional definitions and possibly
  different signatures per configuration.
- **Observable content:** `const pathSep` or `func defaultConfigDir()` defined
  differently in `_windows.go` and `_unix.go` files.
- **Variants & failure cases:** If two included files both define it →
  duplicate declaration error; if none does → undefined identifier; a correct
  graph must either pick a configuration or represent the variance.
- **Constraints:** File-selection rules above.
- **Defined by:** Emergent from language + build constraints.
- **Confidence:** High.
- **Primary source:** Go Language Specification (declarations) + go/build documentation.

---

### DECL — Declarations, scope, and name resolution

#### DECL-01 · Declaration kinds and grouped declarations

- **Behavior:** Package-level `const`, `var`, `type`, and `func` declarations,
  including parenthesized groups, create the package's named entities. Groups
  are purely syntactic but `const` groups carry `iota`/implicit-RHS semantics
  (DECL-06).
- **Observable content:** All four kinds at package level, grouped and
  ungrouped; multi-name specs (`var a, b, c int`); function/method decls.
- **Variants & failure cases:** Local declarations of const/var/type inside
  functions (no local funcs except literals; no local methods).
- **Constraints:** None.
- **Defined by:** Language.
- **Confidence:** High.
- **Primary source:** Go Language Specification, Declarations and scope.

#### DECL-02 · Shadowing, including predeclared identifiers and package names

- **Behavior:** Inner scopes may shadow anything: predeclared identifiers
  (`len`, `int`, `true`, `error`, `any`, `nil`), imported package names (a
  parameter `url` hides package `url` in that scope), and outer variables.
  Resolution is innermost-scope-first.
- **Observable content:** A function that shadows `len` and `error`; a
  parameter shadowing an imported package name with a later (failing or
  restructured) attempted use of that package in the same scope.
- **Variants & failure cases:** `if`/`for`/`switch` init statements introduce
  implicit scopes; `vet -shadow` is off by default so this is legal and
  silent; universe-scope entities are shadowable because they are ordinary
  identifiers, not keywords.
- **Constraints:** None.
- **Defined by:** Language.
- **Confidence:** High.
- **Primary source:** Go Language Specification, Declarations and scope; Predeclared identifiers.

#### DECL-03 · Blank identifier declarations

- **Behavior:** `_` discards a declaration or value: `var _ Iface = (*T)(nil)`
  is the standard compile-time interface-compliance assertion (a real
  reference edge to both the type and the interface with no named entity);
  blank parameters, blank fields, blank loop variables, and blank assignments
  all exist.
- **Observable content:** Compliance assertions; a struct with a `_` padding
  field; `for range` with no variables.
- **Variants & failure cases:** `_` cannot be read; `func _()` is legal but
  uncallable (declared, never referenceable) — same for `type _ struct{}` and
  `var _ = ...` initializer side effects at package init.
- **Constraints:** None.
- **Defined by:** Language.
- **Confidence:** High.
- **Primary source:** Go Language Specification, Blank identifier.

#### DECL-04 · `init` functions

- **Behavior:** Any file may declare any number of `func init()` — entities
  with no name binding (cannot be called or referenced), executed after
  package-level variable initialization, per package, in dependency order.
- **Observable content:** Two `init` funcs in one file plus another in a
  second file; observable side-effect ordering; an `init` performing
  registration into a package-level map (DYN-04 pattern).
- **Variants & failure cases:** `init` is not declared in package scope: you
  cannot reference `init`, and `init` cannot be shadowed away from this role;
  declaring `init` with parameters/results is an error.
- **Constraints:** None.
- **Defined by:** Language.
- **Confidence:** High.
- **Primary source:** Go Language Specification, Package initialization.

#### DECL-05 · Package initialization order

- **Behavior:** Package-level variables initialize in dependency order, not
  source order; ties break by declaration order within a file and file order
  across files. Since Go 1.21 the cross-package init order is fully specified
  (imports first, deterministic order).
- **Observable content:** Package vars whose initializers reference each other
  out of source order; cross-file initializer dependencies.
- **Variants & failure cases:** Initialization cycles are compile errors
  (DIAG-03); file order presented to the compiler is a go-command detail
  (sorted by filename) — see §5.
- **Constraints:** Deterministic cross-package order specified at 1.21+.
- **Defined by:** Language (order rules) + implementation (file ordering).
- **Confidence:** High.
- **Primary source:** Go Language Specification, Package initialization; Go 1.21 release notes.

#### DECL-06 · `const` blocks, `iota`, and implicit repetition

- **Behavior:** Within a grouped const declaration, omitted expressions repeat
  the previous spec, and `iota` counts specs from zero — the standard enum
  idiom. Each const is an untyped or typed constant entity.
- **Observable content:** An enum block with `iota`, skipped values (`_`),
  bit-shift patterns (`1 << iota`), and a typed enum (`type Level int`) later
  given a `String` method (GENC-05 tie-in).
- **Variants & failure cases:** `iota` outside a const group is just zero…
  actually only usable inside const declarations; multi-identifier specs
  advance `iota` once per spec, not per identifier.
- **Constraints:** None.
- **Defined by:** Language.
- **Confidence:** High.
- **Primary source:** Go Language Specification, Iota; Constant declarations.

#### DECL-07 · Short variable declarations and partial redeclaration

- **Behavior:** `:=` declares new variables, but in a multi-assignment it may
  *redeclare* names already declared in the same scope (assigning instead of
  shadowing) as long as at least one name is new. In a nested scope the same
  spelling shadows instead.
- **Observable content:** `f, err := open(); g, err := open()` (err reused);
  an `if v, ok := m[k]; ok { ... }` scoped declaration; a deliberate shadow
  bug pattern (`err :=` inside a block).
- **Variants & failure cases:** All-names-old is "no new variables" error;
  identity of the variable (same entity vs new entity) differs between the
  two forms — graph-relevant.
- **Constraints:** None.
- **Defined by:** Language.
- **Confidence:** High.
- **Primary source:** Go Language Specification, Short variable declarations.

#### DECL-08 · Method declarations and receiver-locality

- **Behavior:** Methods attach to a named (defined) type declared in the same
  package. Receivers are value or pointer; the base type cannot itself be a
  pointer or an interface. Methods on non-struct defined types (e.g.,
  `type Celsius float64`, `type HandlerFunc func(...)`) are idiomatic and
  enable interface adaptation.
- **Observable content:** Value- and pointer-receiver methods on a struct;
  methods on a defined slice/func/map type; a failed (commented or
  diagnostic-fixture) attempt to define a method on a non-local type.
- **Variants & failure cases:** "cannot define new methods on non-local type";
  receiver may be unnamed or `_`; a generic type's methods declare receiver
  type parameters (GEN-04).
- **Constraints:** None.
- **Defined by:** Language.
- **Confidence:** High.
- **Primary source:** Go Language Specification, Method declarations.

#### DECL-09 · Per-iteration loop variables (Go 1.22 semantics shift)

- **Behavior:** At language version ≥ 1.22, `for` loop variables are freshly
  bound each iteration; before, one variable was shared across iterations.
  Closures capturing loop variables mean *different entities* under the two
  versions — same text, different semantics, gated by MOD-01/BLD-04.
- **Observable content:** A loop launching goroutines/closures over the loop
  variable; module `go` directive at ≥1.22; optionally a `//go:build go1.21`
  file preserving old semantics.
- **Variants & failure cases:** The classic captured-loop-variable bug is
  correct code post-1.22; `GOEXPERIMENT=loopvar` predated it in 1.21.
- **Constraints:** Go 1.22+; controlled by declared language version, not
  toolchain version.
- **Defined by:** Language (versioned).
- **Confidence:** High.
- **Primary source:** Go 1.22 release notes; Go Language Specification, For statements.

#### DECL-10 · Labels, `goto`, and labeled `break`/`continue`

- **Behavior:** Labels live in a separate namespace, scoped to the function.
  `break`/`continue` with labels target enclosing loops/switch/select; `goto`
  cannot jump into a block or over variable declarations.
- **Observable content:** A labeled outer loop broken from an inner loop; a
  `goto` to a cleanup label; a label shadowing nothing despite matching a
  variable name.
- **Variants & failure cases:** Unused labels are compile errors; illegal
  `goto` over declarations is a distinct diagnostic.
- **Constraints:** None.
- **Defined by:** Language.
- **Confidence:** High.
- **Primary source:** Go Language Specification, Labeled statements; Goto statements.

#### DECL-11 · Function-local named types

- **Behavior:** Types (including structs and interfaces) may be declared
  inside functions. They are distinct entities per declaration site, cannot
  have methods declared, and can still satisfy interfaces via embedded fields
  or be used as type-assertion targets.
- **Observable content:** A local `type row struct{...}` used with a map; a
  local interface used in a type assertion.
- **Variants & failure cases:** Two local types of identical structure in
  different functions are distinct named types; a local type referencing a
  type parameter of an enclosing generic function (allowed with restrictions).
- **Constraints:** None.
- **Defined by:** Language.
- **Confidence:** High.
- **Primary source:** Go Language Specification, Type declarations.

---

### TYPE — Types, type identity, and conversions

#### TYPE-01 · Defined types vs type aliases

- **Behavior:** `type A B` creates a new distinct type with its own method set
  and identity; `type A = B` creates an alternate name for the *same* type.
  Aliases add no methods and follow the aliased type's identity everywhere
  (assignability, satisfaction, reflection name).
- **Observable content:** A defined wrapper (`type UserID string`) that
  rejects plain-string assignment, alongside an alias
  (`type ID = string`) that accepts it; methods on the defined type only.
- **Variants & failure cases:** Alias chains; alias to a type in another
  package (the classic gradual-refactor use); predeclared aliases `byte`,
  `rune`, `any` (TYPE-12); generic aliases (GEN-05).
- **Constraints:** Aliases since Go 1.9.
- **Defined by:** Language.
- **Confidence:** High.
- **Primary source:** Go Language Specification, Type declarations; Type identity.

#### TYPE-02 · Underlying types, assignability, and conversions

- **Behavior:** Every type has an underlying type; assignability permits
  mixing a named type and an unnamed type with identical underlying type
  without conversion, while two differently named types require explicit
  conversion. Struct conversion ignores field tags (Go 1.8+).
- **Observable content:** `type Meters float64` converted to/from `float64`;
  two structurally identical named structs converted explicitly; a tag-only
  difference converted implicitly-by-conversion.
- **Variants & failure cases:** Conversion legality vs assignability
  distinction; unexported fields require same-package identity for struct
  conversion.
- **Constraints:** None.
- **Defined by:** Language.
- **Confidence:** High.
- **Primary source:** Go Language Specification, Assignability; Conversions.

#### TYPE-03 · Struct embedding and promotion

- **Behavior:** An embedded (anonymous) field promotes its fields and methods
  to the outer type, at increasing depth; the shallowest declaration wins,
  and same-depth duplicates make the selector ambiguous (error only at use).
  Embedding `*T` vs `T` changes method-set propagation; embedding an
  interface stores a value and delegates dynamically.
- **Observable content:** Multi-level embedding with promotion; a name that is
  shadowed at a shallower depth; an embedded `*T`; a struct embedding an
  interface and satisfying it by delegation; ambiguity fixture (DIAG-05).
- **Variants & failure cases:** Promoted fields cannot be set via composite
  literal on the outer type (must nest, TYPE-13); nil embedded interface
  panics at call time; promotion feeds interface satisfaction (IFACE-01).
- **Constraints:** None.
- **Defined by:** Language.
- **Confidence:** High.
- **Primary source:** Go Language Specification, Struct types; Selectors.

#### TYPE-04 · Method sets and addressability

- **Behavior:** The method set of `T` contains value-receiver methods; `*T`
  has both value- and pointer-receiver methods. Interface satisfaction uses
  method sets exactly; the shorthand `v.PtrMethod()` on an addressable `v`
  is sugar for `(&v).PtrMethod()` and does not change method sets.
- **Observable content:** A type whose pointer satisfies an interface while
  its value does not; a map-element method-call failure (unaddressable);
  compliance assertions for both `T` and `*T`.
- **Variants & failure cases:** Calling a pointer method on an unaddressable
  value is a compile error; embedding interacts (TYPE-03).
- **Constraints:** None.
- **Defined by:** Language.
- **Confidence:** High.
- **Primary source:** Go Language Specification, Method sets.

#### TYPE-05 · Struct field tags

- **Behavior:** String tags on struct fields are part of the type (affecting
  type identity, but not conversions since 1.8) and carry
  runtime-readable metadata in the conventional `key:"value"` format that
  drives marshaling, ORM mapping, and validation.
- **Observable content:** JSON/YAML-tagged structs with `omitempty`, renames,
  `-` exclusions; a struct read via `reflect.StructTag.Get`.
- **Variants & failure cases:** Malformed tags are silently inert (vet's
  `structtag` check flags them); tags create semantic relationships (field ↔
  wire name) invisible to pure call-graph analysis (DYN-02).
- **Constraints:** Tag syntax convention defined by reflect.
- **Defined by:** Language (tags exist) + std convention (format).
- **Confidence:** High.
- **Primary source:** Go Language Specification, Struct types; reflect.StructTag documentation.

#### TYPE-06 · Anonymous (unnamed) composite types

- **Behavior:** Struct, interface, func, map, slice, channel types may appear
  unnamed at use sites. Identity is structural: two identical unnamed types
  are the same type across packages.
- **Observable content:** A `[]struct{ Name string }` literal; an anonymous
  interface in a variable declaration or assertion
  (`x.(interface{ Close() error })`); a func-typed field.
- **Variants & failure cases:** Structural identity across packages fails if
  unexported field names are involved (identity then requires same package).
- **Constraints:** None.
- **Defined by:** Language.
- **Confidence:** High.
- **Primary source:** Go Language Specification, Type identity.

#### TYPE-07 · Recursive and mutually recursive types

- **Behavior:** Named types may reference themselves through pointers,
  slices, maps, or function types; mutual recursion may span files of the
  package. Direct self-containment by value is an invalid-cycle error.
- **Observable content:** A tree node (`Children []*Node`), an AST-style
  mutual recursion split across two files; an invalid `type T struct{ t T }`
  diagnostic fixture.
- **Variants & failure cases:** Recursion through interfaces; generic
  recursive constraints (GEN-07).
- **Constraints:** None.
- **Defined by:** Language.
- **Confidence:** High.
- **Primary source:** Go Language Specification, Type declarations.

#### TYPE-08 · Untyped constants and constant arithmetic

- **Behavior:** Constants are arbitrary-precision and "untyped" until used;
  they convert implicitly where representable and take default types
  (`int`, `float64`, `rune`, `string`, `bool`, `complex128`) at binding.
  Constant expressions are evaluated at compile time with exactness rules.
- **Observable content:** An untyped const used as several distinct types; a
  constant-overflow compile error (`const big = 1 << 200` bound to int); rune
  and imaginary literals; constant expressions crossing package boundaries.
- **Variants & failure cases:** Representability failures at use sites, not
  declaration; platform-dependent overflow with `int` on 32-bit targets
  (PLAT-02).
- **Constraints:** Minimum implementation precision requirements; gc exceeds
  them.
- **Defined by:** Language.
- **Confidence:** High.
- **Primary source:** Go Language Specification, Constants.

#### TYPE-09 · String/slice/array conversion family

- **Behavior:** Conversions between `string`, `[]byte`, `[]rune`; slice to
  array pointer (Go 1.17); slice to array (Go 1.20). Each has distinct
  semantics (copying vs aliasing, length panics).
- **Observable content:** All four conversion forms; a
  length-mismatch runtime panic fixture for slice→array.
- **Variants & failure cases:** Version gating: slice→array under `go < 1.20`
  is a compile error — a good language-version probe.
- **Constraints:** Language version.
- **Defined by:** Language (versioned).
- **Confidence:** High.
- **Primary source:** Go Language Specification, Conversions; Go 1.20 release notes.

#### TYPE-10 · Channel types and directionality

- **Behavior:** `chan T`, `chan<- T`, `<-chan T` are distinct types with
  implicit conversion from bidirectional to directional only. Direction
  restricts available operations at compile time.
- **Observable content:** A function taking `<-chan Event` fed from a
  `chan Event`; a compile-error fixture sending on a receive-only channel.
- **Variants & failure cases:** Direction in struct fields and interfaces;
  `close` on receive-only channel is a compile error.
- **Constraints:** None.
- **Defined by:** Language.
- **Confidence:** High.
- **Primary source:** Go Language Specification, Channel types.

#### TYPE-11 · The `unsafe` package as compiler magic

- **Behavior:** `unsafe.Pointer`, `Sizeof`, `Offsetof`, `Alignof` (constant
  results), `Add`, `Slice`, `SliceData`, `String`, `StringData` are
  compiler-implemented; importing `unsafe` also gates other features
  (`go:linkname`). `unsafe.Pointer` ↔ `uintptr` conversions have special
  validity patterns.
- **Observable content:** A struct-offset computation; a `unsafe.Slice` use;
  the `unsafe` import enabling a linkname (DIR-01).
- **Variants & failure cases:** `Sizeof` results are implementation-defined
  (§5); misuse patterns are vet-checked (`unsafeptr`).
- **Constraints:** gc-specific results; portability hazards.
- **Defined by:** Language (package exists) + implementation (values).
- **Confidence:** High.
- **Primary source:** unsafe package documentation (pkg.go.dev/unsafe).

#### TYPE-12 · Predeclared aliases and names: `byte`, `rune`, `any`, `error`

- **Behavior:** `byte`/`uint8` and `rune`/`int32` are identical types;
  `any` = `interface{}` (alias, 1.18+); `error` is a predeclared non-alias
  interface type. Graphs must unify aliases and treat `error` as an ordinary
  interface that user types satisfy.
- **Observable content:** Interchange of `byte` and `uint8` in signatures; an
  `any`-typed API; a custom error type with `Error() string`.
- **Variants & failure cases:** Shadowing `any` or `error` locally (DECL-02);
  `errors.Is/As` chains are library convention, not language.
- **Constraints:** `any` requires Go 1.18+.
- **Defined by:** Language.
- **Confidence:** High.
- **Primary source:** Go Language Specification, Predeclared identifiers; Errors.

#### TYPE-13 · Composite literals as reference sites

- **Behavior:** Keyed struct literals reference field identifiers; unkeyed
  literals bind positionally and break silently on field reordering; map/
  slice/array literals may nest and elide inner type names. Cross-package
  unkeyed literals of structs with unexported fields are illegal.
- **Observable content:** Keyed and unkeyed literals of the same struct;
  nested literals with elided element types; a cross-package unkeyed-literal
  compile error fixture; `&T{...}` pointer literals.
- **Variants & failure cases:** Cannot initialize promoted fields directly in
  an outer literal (must write the embedded field's own literal); duplicate
  keys are errors.
- **Constraints:** None.
- **Defined by:** Language.
- **Confidence:** High.
- **Primary source:** Go Language Specification, Composite literals.

---

### IFACE — Interfaces, method sets, and dispatch

#### IFACE-01 · Implicit (structural) interface satisfaction

- **Behavior:** A type satisfies an interface purely by method-set inclusion —
  no declaration links them. The implements-relation is computed, may cross
  packages that never import each other, and includes promoted methods.
- **Observable content:** A concrete type in package A satisfying an interface
  in package B with no reference between them; satisfaction achieved solely
  via an embedded type's methods; the `var _ I = (*T)(nil)` assertion idiom.
- **Variants & failure cases:** Near-misses (wrong receiver kind → only `*T`
  satisfies, TYPE-04; wrong signature); satisfaction of standard-library
  interfaces (`io.Reader`, `fmt.Stringer`, `error`, `sort.Interface`).
- **Constraints:** None.
- **Defined by:** Language.
- **Confidence:** High.
- **Primary source:** Go Language Specification, Interface types; Implementing an interface.

#### IFACE-02 · Interface embedding and overlapping method sets

- **Behavior:** Interfaces embed other interfaces to union method sets. Since
  Go 1.14, embedded interfaces may contribute duplicate methods if the
  signatures are identical; conflicting signatures are errors.
- **Observable content:** `io.ReadWriteCloser`-style composition of local
  interfaces; an overlap (`ReadCloser` + `Closer`) legal at 1.14+; a
  conflicting-signature error fixture.
- **Variants & failure cases:** Pre-1.14 modules reject overlap — version
  sensitive; embedding `error` or `fmt.Stringer` into local interfaces.
- **Constraints:** Go 1.14+ for overlap.
- **Defined by:** Language (versioned).
- **Confidence:** High.
- **Primary source:** Go Language Specification, Interface types; Go 1.14 release notes.

#### IFACE-03 · Type assertions and type switches

- **Behavior:** `x.(T)` asserts a dynamic type (single-value form panics;
  two-value form reports). `switch v := x.(type)` binds `v` with a *different
  static type in each case* — one syntactic variable, several
  per-case entities; a `nil` case and multi-type cases exist.
- **Observable content:** Both assertion forms; a type switch with concrete
  cases, an interface case, a multi-type case (where `v` keeps the interface
  type), and a `default`.
- **Variants & failure cases:** Assertion to a type that cannot possibly
  satisfy the interface is a compile error (impossible assertion); assertions
  to other interfaces (dynamic re-check).
- **Constraints:** None.
- **Defined by:** Language.
- **Confidence:** High.
- **Primary source:** Go Language Specification, Type assertions; Type switches.

#### IFACE-04 · Static call vs dynamic dispatch

- **Behavior:** A method call on a concrete type resolves statically; a call
  through an interface dispatches on the dynamic type at run time. A correct
  graph distinguishes the concrete edge from the interface-call edge whose
  callee set is "all satisfiers".
- **Observable content:** The same method invoked once via the concrete type
  and once via an interface variable; a function accepting the interface and
  called with several implementations.
- **Variants & failure cases:** Devirtualization (incl. PGO-driven) is a gc
  optimization and must not change modeled semantics; nil-interface calls
  panic at run time.
- **Constraints:** None.
- **Defined by:** Language (semantics); implementation (optimization).
- **Confidence:** High.
- **Primary source:** Go Language Specification, Method values / Calls.

#### IFACE-05 · Method values and method expressions

- **Behavior:** `x.M` (method value) closes over the receiver, yielding a
  `func(args)`; `T.M` and `(*T).M` (method expressions) yield functions with
  the receiver as an explicit first parameter. Both convert methods into
  first-class function values — distinct reference forms to the same method
  entity.
- **Observable content:** A method value passed as a callback; `T.M` used with
  `sort.Slice`-style helpers or stored in a map of handlers.
- **Variants & failure cases:** Method value on a pointer receiver via an
  addressable value; method expression on an interface type
  (`I.M` — receiver becomes the interface).
- **Constraints:** None.
- **Defined by:** Language.
- **Confidence:** High.
- **Primary source:** Go Language Specification, Method values; Method expressions.

#### IFACE-06 · Function-type adapters (`http.HandlerFunc` pattern)

- **Behavior:** A defined function type with a method on itself lets bare
  functions satisfy an interface via conversion — an idiomatic dispatch bridge
  (`type HandlerFunc func(...)` with `ServeHTTP` calling the receiver).
- **Observable content:** A local adapter type mirroring the pattern; a plain
  function converted to it and used where the interface is required.
- **Variants & failure cases:** The conversion site is the only link between
  the function and the interface — easy for graphs to miss.
- **Constraints:** None.
- **Defined by:** Ecosystem convention built on language features.
- **Confidence:** High.
- **Primary source:** net/http package documentation (HandlerFunc).

#### IFACE-07 · Unexported interface methods restrict satisfiers

- **Behavior:** An interface with an unexported method can only be satisfied
  by types in the interface's package (or embedders of such a type) — a
  sealed-interface technique. Satisfaction rules consider package identity of
  unexported names.
- **Observable content:** A sealed interface with `isVariant()`; external type
  failing to satisfy it; internal types satisfying it; an external type
  embedding an internal implementation to inherit satisfaction.
- **Variants & failure cases:** The embedding loophole is the notable
  counterexample.
- **Constraints:** None.
- **Defined by:** Language.
- **Confidence:** High.
- **Primary source:** Go Language Specification, Interface types; Uniqueness of identifiers.

#### IFACE-08 · Basic vs constraint-only interfaces

- **Behavior:** Interfaces containing type sets (`~T`, unions) or `comparable`
  are valid only as type constraints, not as variable types. `comparable`
  itself is predeclared and constraint-only.
- **Observable content:** A constraint interface with a union used in a
  generic function; a compile-error fixture using it as a variable type.
- **Variants & failure cases:** Go 1.20 loosened satisfaction of `comparable`
  (ordinary comparable-but-panicky types like interfaces now satisfy it);
  version-sensitive.
- **Constraints:** Go 1.18+; 1.20 for the `comparable` loosening.
- **Defined by:** Language (versioned).
- **Confidence:** High.
- **Primary source:** Go Language Specification, Interface types; Go 1.20 release notes.

---

### GEN — Generics (type parameters)

#### GEN-01 · Generic functions and generic types

- **Behavior:** Functions and named types take type-parameter lists with
  constraints. An instantiation (explicit or inferred) produces a concrete
  entity; identical type arguments yield the identical instantiated type.
- **Observable content:** A generic container type and generic functions,
  instantiated with several arguments across packages; equality of two
  same-argument instantiations demonstrated by assignment.
- **Variants & failure cases:** Un-instantiated generic names cannot be used
  as values or (mostly) types; instantiation of an exported generic with an
  unexported local type crosses visibility domains (GEN-08).
- **Constraints:** Go 1.18+.
- **Defined by:** Language.
- **Confidence:** High.
- **Primary source:** Go Language Specification, Type parameter declarations; Instantiations.

#### GEN-02 · Type inference

- **Behavior:** Call-site inference (and, since 1.21, inference in more
  contexts: generic functions used as arguments to generic functions,
  assignment to function-typed variables) determines type arguments without
  spelling them. Inferred vs explicit instantiation must resolve to the same
  entity.
- **Observable content:** The same generic function called with and without
  explicit arguments; a generic function passed to another generic function
  relying on 1.21 inference.
- **Variants & failure cases:** Inference failure diagnostics; untyped
  constant arguments influencing inferred types.
- **Constraints:** Go 1.18+; 1.21 for expanded inference.
- **Defined by:** Language (versioned).
- **Confidence:** High.
- **Primary source:** Go Language Specification, Type inference; Go 1.21 release notes.

#### GEN-03 · Constraint interfaces: type sets, `~`, unions, `comparable`

- **Behavior:** Constraints combine method requirements with type sets:
  exact terms, `~T` underlying-type terms, and unions. `comparable` allows
  `==`. Operations available inside a generic body are those valid for the
  entire type set.
- **Observable content:** A `Number` constraint (`~int | ~float64`), a
  constraint mixing a method with a type set, use of `==` under `comparable`,
  and an error fixture invoking an operation not shared by the set.
- **Variants & failure cases:** Unions cannot contain `comparable` or
  method-bearing interfaces; `~T` requires T to be its own underlying type;
  the spec's "core type" formulation was removed/reworked in Go 1.25 without
  intended semantic change.
- **Constraints:** Go 1.18+.
- **Defined by:** Language.
- **Confidence:** High (medium on 1.25 spec-rewording detail).
- **Primary source:** Go Language Specification, Interface types (type sets); Go 1.25 release notes.

#### GEN-04 · Methods on generic types; no type-parameterized methods

- **Behavior:** Methods on a generic type redeclare receiver type parameters
  (`func (s *Stack[T]) Push(v T)`); methods may not introduce their *own*
  type parameters. This shapes what dispatch/graph structures are possible
  (no generic-method vertices).
- **Observable content:** A generic type with several methods; a
  compile-error fixture attempting `func (r R) M[U any](...)`.
- **Variants & failure cases:** Interface satisfaction by instantiated
  generic types (a `Stack[int]` satisfying an interface); receiver
  type-parameter names may differ from the declaration's.
- **Constraints:** Go 1.18+.
- **Defined by:** Language.
- **Confidence:** High.
- **Primary source:** Go Language Specification, Method declarations.

#### GEN-05 · Generic type aliases

- **Behavior:** Go 1.24 made type aliases parameterizable
  (`type Set[E comparable] = map[E]struct{}`) — alias identity now follows
  substitution, and alias declarations themselves carry type parameters.
- **Observable content:** A generic alias used interchangeably with its RHS
  instantiation; an alias of an instantiated generic type.
- **Variants & failure cases:** Pre-1.24 this is a compile error (1.23 had it
  behind `GOEXPERIMENT=aliastypeparams`); tooling based on older go/types may
  misread it.
- **Constraints:** Go 1.24+.
- **Defined by:** Language (versioned).
- **Confidence:** High.
- **Primary source:** Go 1.24 release notes; Go Language Specification, Type declarations.

#### GEN-06 · Explicit and partial instantiation as values

- **Behavior:** Instantiating a generic function without calling it
  (`f := Map[int, string]`) yields an ordinary function value — an entity
  derived from the generic declaration plus arguments.
- **Observable content:** An instantiated generic function stored in a
  variable/struct field and called later.
- **Variants & failure cases:** Partial instantiation (fewer arguments than
  parameters) is allowed only when the rest are inferable at that point.
- **Constraints:** Go 1.18+.
- **Defined by:** Language.
- **Confidence:** High.
- **Primary source:** Go Language Specification, Instantiations.

#### GEN-07 · Recursive generic types and self-referential constraints

- **Behavior:** Generic types may reference their own instantiations
  (`type Tree[T any] struct { kids []Tree[T] }` via pointer/slice), and
  constraints may reference the type parameter
  (`type Ordered[T Lesser[T]]`-style, the "curiously recurring" pattern).
- **Observable content:** A recursive generic container; a
  self-referential constraint (`interface{ CompareTo(T) int }` used as
  `[T Comparable[T]]`).
- **Variants & failure cases:** Illegal constraint cycles produce dedicated
  diagnostics; anonymous recursion limits exist for marshalable structures.
- **Constraints:** Go 1.18+.
- **Defined by:** Language.
- **Confidence:** High.
- **Primary source:** Go Language Specification, Type parameter declarations.

#### GEN-08 · Cross-package instantiation

- **Behavior:** A generic declared in one package instantiated in another with
  the *consumer's* (possibly unexported) types creates entities whose parts
  live in two visibility domains; the instantiation exists at the use site.
- **Observable content:** A library generic (`Cache[K,V]`) instantiated in a
  consumer package with an unexported local struct as `V`.
- **Variants & failure cases:** Where instantiation entities "belong" is a
  genuine modeling question; compiled-code sharing (GC shape stenciling) is an
  implementation detail that must not leak into semantics.
- **Constraints:** Go 1.18+.
- **Defined by:** Language (semantics); implementation (stenciling).
- **Confidence:** High.
- **Primary source:** Go Language Specification, Instantiations.

---

### FUNC — Functions, closures, control flow, entry points

#### FUNC-01 · Program entry and lifecycle

- **Behavior:** Execution begins by initializing imported packages
  (transitively), then the main package, then calling `main.main`. `os.Exit`
  skips deferred functions; returning from `main` ends the process without
  waiting for goroutines.
- **Observable content:** A main package whose imports demonstrate observable
  init ordering; a goroutine outliving `main` (never runs to completion).
- **Variants & failure cases:** Multiple mains per module (PKG-04); test
  binaries synthesize their own entry (TEST-05).
- **Constraints:** None.
- **Defined by:** Language.
- **Confidence:** High.
- **Primary source:** Go Language Specification, Program execution.

#### FUNC-02 · Function literals and closures

- **Behavior:** Function literals are anonymous function entities that capture
  enclosing variables *by reference*; captured variables' identity (shared vs
  per-iteration, DECL-09) is semantic. Closures may be immediately invoked,
  stored, or returned.
- **Observable content:** A counter-factory returning a closure; closures
  sharing one captured variable; an IIFE; a closure stored in a struct field.
- **Variants & failure cases:** Escape of captured locals (heap allocation) is
  implementation detail; recursion by a named local variable assignment.
- **Constraints:** None.
- **Defined by:** Language.
- **Confidence:** High.
- **Primary source:** Go Language Specification, Function literals.

#### FUNC-03 · `defer` and `go` statements as call edges

- **Behavior:** `defer f(args)` and `go f(args)` evaluate the callee and
  arguments immediately but call later/concurrently — call edges with
  distinct temporal semantics. Deferred calls run LIFO on return or panic.
- **Observable content:** Deferred close/unlock idioms; a deferred method
  value; `go` launching a named function, a closure, and a method value;
  deferred closure mutating a named result (FUNC-05).
- **Variants & failure cases:** Defer inside loops (per-iteration
  accumulation); argument-evaluation timing surprises.
- **Constraints:** None.
- **Defined by:** Language.
- **Confidence:** High.
- **Primary source:** Go Language Specification, Defer statements; Go statements.

#### FUNC-04 · Variadic functions and argument spreading

- **Behavior:** `...T` final parameters accept zero or more values or an
  existing slice via `s...`. `append` and `fmt`-style APIs rely on it;
  spreading vs element-passing are distinct call shapes.
- **Observable content:** A local variadic function called both ways;
  forwarding a variadic (`f(xs...)`) through a wrapper.
- **Variants & failure cases:** Mixing spread and additional elements is
  illegal; `append([]byte, string...)` special case.
- **Constraints:** None.
- **Defined by:** Language.
- **Confidence:** High.
- **Primary source:** Go Language Specification, Passing arguments to ... parameters.

#### FUNC-05 · Named results and bare returns

- **Behavior:** Named result parameters are declared variables in the function
  scope; a bare `return` returns their current values, and deferred functions
  may modify them after `return` assigns them — including replacing a
  returned `error` in a `defer`/`recover` idiom.
- **Observable content:** A function with named results and a bare return; a
  deferred function converting a panic into a returned error.
- **Variants & failure cases:** Shadowing a named result inside a block breaks
  the deferred-modification idiom silently.
- **Constraints:** None.
- **Defined by:** Language.
- **Confidence:** High.
- **Primary source:** Go Language Specification, Return statements; Defer statements.

#### FUNC-06 · `panic`/`recover` control flow

- **Behavior:** `panic` unwinds through deferred calls; `recover` inside a
  deferred function stops unwinding. This creates non-local control-flow
  edges from any statement to enclosing deferred handlers.
- **Observable content:** A recover-based error boundary; a re-panic; runtime
  panics (nil dereference, index out of range) reaching a recover.
- **Variants & failure cases:** `recover` outside a deferred call returns nil
  and does nothing; goroutine panics are not recoverable across goroutines.
- **Constraints:** None.
- **Defined by:** Language.
- **Confidence:** High.
- **Primary source:** Go Language Specification, Handling panics.

#### FUNC-07 · Built-in functions are not first-class

- **Behavior:** `len`, `cap`, `make`, `new`, `append`, `copy`, `delete`,
  `panic`, `recover`, `print`, `println`, `min`, `max`, `clear` (1.21+) are
  compiler-known and cannot be used as function values; several are generic-ish
  in ways user code cannot express and some (`len` of constant expressions)
  produce constants.
- **Observable content:** Ordinary uses; an error fixture taking `f := len`;
  a constant `len` of a fixed array; shadowing `len` (DECL-02 tie-in).
- **Variants & failure cases:** `min`/`max`/`clear` are version-gated (1.21);
  `print`/`println` are implementation-provided and not guaranteed.
- **Constraints:** Language version for the newer builtins.
- **Defined by:** Language ( `print`/`println`: implementation-guaranteed only informally).
- **Confidence:** High.
- **Primary source:** Go Language Specification, Built-in functions.

#### FUNC-08 · `for range` forms, including int and function ranges

- **Behavior:** `range` iterates arrays/slices/strings (runes!), maps,
  channels, integers (1.22+), and functions (`func(yield func(...) bool)`,
  1.23+). Range-over-func inverts the call relationship: the loop body becomes
  a synthesized closure passed to the iterator.
- **Observable content:** All range forms; a custom iterator in `iter.Seq`
  style consumed by `for x := range seq`; `break` inside a func-range
  (yield returning false).
- **Variants & failure cases:** String range yields byte-index/rune pairs;
  map range order is unspecified (§5); range-over-func is a strong probe of
  whether a graph models the desugaring.
- **Constraints:** Go 1.22 (int), 1.23 (func).
- **Defined by:** Language (versioned).
- **Confidence:** High.
- **Primary source:** Go Language Specification, For statements (range clause); Go 1.23 release notes.

#### FUNC-09 · `select` and channel operations

- **Behavior:** `select` chooses among channel communications, pseudo-randomly
  when several are ready; `default` makes it non-blocking. Send/receive
  statements and the two-value receive (`v, ok := <-ch`) are distinct
  operations on channel entities.
- **Observable content:** A select over two channels plus default; a closed
  channel drained with two-value receives.
- **Variants & failure cases:** Empty `select{}` blocks forever; nil-channel
  arms block; randomization is specified as uniform pseudo-random.
- **Constraints:** None.
- **Defined by:** Language.
- **Confidence:** High.
- **Primary source:** Go Language Specification, Select statements.

---

### IMP — Imports and inter-package edges

#### IMP-01 · Named (aliased) imports

- **Behavior:** `import alias "path"` binds the package to a file-scoped
  alias, required when two imports share a base package name and to resolve
  MOD-10 mismatches.
- **Observable content:** Two same-named packages imported with aliases in one
  file; different aliases for the same package in different files (PKG-05).
- **Variants & failure cases:** Alias shadowing by locals (DECL-02); alias `.`
  and `_` are the special forms below.
- **Constraints:** None.
- **Defined by:** Language.
- **Confidence:** High.
- **Primary source:** Go Language Specification, Import declarations.

#### IMP-02 · Blank imports and init-only dependencies

- **Behavior:** `import _ "path"` links a package solely for its side effects
  (init functions, registrations). It creates a real dependency edge with no
  named references — the mechanism behind `database/sql` drivers and
  `image/*` codec registration.
- **Observable content:** A blank import whose init registers into a registry
  consulted elsewhere (a local plugin-registry fixture mirroring the
  `image.RegisterFormat` pattern).
- **Variants & failure cases:** Removing the blank import changes runtime
  behavior with zero reference-level diffs — the canonical "invisible edge".
- **Constraints:** None.
- **Defined by:** Language.
- **Confidence:** High.
- **Primary source:** Go Language Specification, Import declarations.

#### IMP-03 · Dot imports

- **Behavior:** `import . "path"` merges the package's exported names into the
  file's scope; references then look like locals. Name collisions with local
  declarations are errors.
- **Observable content:** One file using a dot import (idiomatic mainly in
  some test DSLs); an identifier that would be ambiguous resolved per rules.
- **Variants & failure cases:** Dot import of a package whose exported name
  collides with another dot import; disallowed inside files that declare the
  same name.
- **Constraints:** Legal but lint-discouraged.
- **Defined by:** Language.
- **Confidence:** High.
- **Primary source:** Go Language Specification, Import declarations.

#### IMP-04 · Import cycles are errors; the external-test escape

- **Behavior:** Package import cycles are compile-time errors. External test
  packages (`foo_test`) may import packages that themselves import `foo`,
  enabling test-only dependency shapes impossible for ordinary packages.
- **Observable content:** A commented/diagnostic cycle fixture; a `foo_test`
  external package importing a helper that imports `foo`.
- **Variants & failure cases:** Cycles via type-checking only (no calls) are
  still errors; test-induced cycles through in-package `_test.go` files are
  still illegal — only the external package escapes.
- **Constraints:** None.
- **Defined by:** Language + toolchain (test-package handling).
- **Confidence:** High.
- **Primary source:** Go Language Specification, Import declarations; go command documentation (go test).

#### IMP-05 · Unused imports are errors

- **Behavior:** An imported-and-unused package is a compile error; the blank
  alias is the sanctioned suppression. Same for unused local variables (but
  not unused package-level vars, params, or results).
- **Observable content:** A diagnostic fixture; a `_`-aliased import retained
  for side effects.
- **Variants & failure cases:** Use in only one build configuration makes an
  import unused in others (interacts with BLD-01); goimports-style tooling
  rewrites are ecosystem response.
- **Constraints:** None.
- **Defined by:** Language.
- **Confidence:** High.
- **Primary source:** Go Language Specification, Import declarations.

#### IMP-06 · Standard library resolution and `math/rand/v2`-style coexistence

- **Behavior:** Standard-library import paths resolve without module
  requirements; module paths require a dotted first element, so `foo/bar`
  space belongs to std. `math/rand` and `math/rand/v2` (and `encoding/json`
  vs the experimental `encoding/json/v2`) are distinct packages with equal
  package names.
- **Observable content:** Both rand packages imported (one aliased) in one
  file; a local package named the same as a std package (e.g., a local
  `errors`) disambiguated purely by import path.
- **Variants & failure cases:** `GOEXPERIMENT=jsonv2` changes std surface
  (1.25, §4); vendored std is not a thing — std comes from the toolchain.
- **Constraints:** Toolchain version determines std contents.
- **Defined by:** Official implementation.
- **Confidence:** High.
- **Primary source:** go command documentation; Go 1.22/1.25 release notes.

---

### TEST — Testing, benchmarks, fuzzing, examples

#### TEST-01 · In-package test files

- **Behavior:** `_test.go` files in `package foo` extend the package under
  `go test` with test-only declarations and full access to unexported names.
- **Observable content:** White-box tests touching unexported functions;
  test-only helper types; tests inside a `package main`.
- **Variants & failure cases:** Symbol collisions between test and non-test
  files are errors under `go test` only — a configuration-dependent
  diagnostic.
- **Constraints:** go test build mode.
- **Defined by:** Official implementation.
- **Confidence:** High.
- **Primary source:** testing package documentation; go command documentation (go test).

#### TEST-02 · External test package (`foo_test`)

- **Behavior:** `_test.go` files may instead declare `package foo_test` — a
  second package in the same directory, importing `foo` like any consumer.
  The `export_test.go` idiom (an in-package test file re-exporting internals)
  bridges the two.
- **Observable content:** A directory with `foo`, in-package tests, external
  tests importing `foo`, and an `export_test.go` alias like
  `var ParseInternal = parseInternal`.
- **Variants & failure cases:** The only legal two-packages-one-directory
  case; external tests may import packages that import `foo` (IMP-04).
- **Constraints:** go test build mode.
- **Defined by:** Official implementation.
- **Confidence:** High.
- **Primary source:** go command documentation (go test); testing package documentation.

#### TEST-03 · Test/benchmark/fuzz function discovery by signature and name

- **Behavior:** `func TestXxx(*testing.T)`, `func BenchmarkXxx(*testing.B)`,
  `func FuzzXxx(*testing.F)` are discovered by name pattern + signature; the
  character after the prefix must not be a lowercase letter. These are
  entry-point entities invoked reflectively by the generated test main.
- **Observable content:** Conforming and deliberately non-conforming
  (`Testsomething`) functions; a fuzz target with `f.Add` seeds and a
  registered fuzz function.
- **Variants & failure cases:** Wrong signatures are diagnosed by vet
  (`tests` analyzer) rather than the compiler; subtests via `t.Run` with
  *runtime-computed names* are unenumerable statically.
- **Constraints:** go test; fuzzing needs Go 1.18+ and supported platforms.
- **Defined by:** Official implementation.
- **Confidence:** High.
- **Primary source:** testing package documentation.

#### TEST-04 · Example functions bind documentation to identifiers by name

- **Behavior:** `ExampleFoo`, `ExampleType_Method`, `Example_suffix` naming
  conventions attach runnable examples to package/type/method/function
  identifiers; a trailing `// Output:` comment makes them asserted tests, and
  `// Unordered output:` relaxes ordering. Whole-file examples exist.
- **Observable content:** Examples at each binding level with and without
  output comments; a whole-file example (a file whose only test-relevant decl
  is one example plus supporting decls).
- **Variants & failure cases:** Example with no output comment compiles but
  never executes as a test; wrong name → binds to nothing (dangling doc).
- **Constraints:** go test + godoc conventions.
- **Defined by:** Official implementation + doc-tool convention.
- **Confidence:** High.
- **Primary source:** testing package documentation (Examples); Go Doc Comments.

#### TEST-05 · `TestMain` and the synthesized test binary

- **Behavior:** `func TestMain(m *testing.M)` replaces the default test
  driver for a package. The test binary itself is a generated main package
  linking the package, its test files, and external test package together.
- **Observable content:** A `TestMain` doing setup/teardown around `m.Run()`.
- **Variants & failure cases:** Only one `TestMain` per test package;
  in-package and external test packages each may have one.
- **Constraints:** go test.
- **Defined by:** Official implementation.
- **Confidence:** High.
- **Primary source:** testing package documentation.

#### TEST-06 · `testdata` fixtures and fuzz corpora

- **Behavior:** Runtime-read fixtures live in `testdata/` (never compiled,
  PKG-03); fuzzing writes/reads corpus files under
  `testdata/fuzz/FuzzName/` in a defined text format.
- **Observable content:** Tests opening `testdata/` files by relative path
  (tests run with the package directory as working directory); checked-in
  corpus entries.
- **Variants & failure cases:** Intentionally invalid Go source as test input
  under `testdata` (a parser fixture) — must not join the graph as code.
- **Constraints:** go test conventions.
- **Defined by:** Official implementation.
- **Confidence:** High.
- **Primary source:** go command documentation; testing package documentation (fuzzing).

#### TEST-07 · Vet runs during `go test`

- **Behavior:** `go test` runs a high-confidence subset of `go vet` (printf,
  tests-analyzer, buildtag, etc.) and fails the build on findings — diagnostics
  that appear only in the test configuration.
- **Observable content:** A printf-verb mismatch fixture that builds under
  `go build` but fails `go test` (or is annotated as such).
- **Variants & failure cases:** `-vet=off` disables; the analyzer set has
  grown across releases (e.g., new `tests`/`stringintconv` additions).
- **Constraints:** go command behavior; version-varying analyzer set.
- **Defined by:** Official implementation.
- **Confidence:** High.
- **Primary source:** go command documentation (go test, go vet).

---

### GENC — Generated code and embedded assets

#### GENC-01 · `//go:generate` directives

- **Behavior:** `go generate` scans for `//go:generate cmd args` comments and
  runs them; it is never automatic. Generated outputs are ordinary sources
  that must be committed (or regenerated) for the package to build.
- **Observable content:** Generate lines invoking `go run` on an
  ignore-tagged generator (BLD-05) and/or `go tool` (MOD-11), plus the
  committed outputs.
- **Variants & failure cases:** `$GOFILE`, `$GOPACKAGE`, `-run` filtering;
  drift between directive and committed output is a real-world hazard graphs
  may want to surface.
- **Constraints:** go command.
- **Defined by:** Official implementation.
- **Confidence:** High.
- **Primary source:** go command documentation (go generate).

#### GENC-02 · Generated-file marker convention

- **Behavior:** The canonical marker `^// Code generated .* DO NOT EDIT\.$`
  (a whole line before the package clause) identifies generated files; tools
  (including go/build-adjacent tooling and linters) treat marked files
  specially.
- **Observable content:** Generated files bearing the exact marker; one file
  with a near-miss marker (wrong position) as a counterexample.
- **Variants & failure cases:** The regexp is precise — trailing text,
  wrong casing, or placement after the package clause does not count.
- **Constraints:** Convention with official blessing.
- **Defined by:** Official convention (documented format).
- **Confidence:** High.
- **Primary source:** go generate design documentation / go command documentation.

#### GENC-03 · `//line` directives remap positions

- **Behavior:** `//line file:line[:col]` and `/*line ...*/` comments make the
  compiler attribute subsequent tokens to other files/positions — used by
  code generators (yacc/goyacc, templating compilers) so diagnostics point at
  the source template. Position-bearing graphs must decide which coordinate
  system they report.
- **Observable content:** A generated file with line directives pointing into
  a sibling non-Go source; an error fixture whose diagnostic cites the
  remapped file.
- **Variants & failure cases:** Column-precise `/*line*/` forms; bogus targets
  (nonexistent files) are accepted silently.
- **Constraints:** gc compiler feature.
- **Defined by:** Official implementation.
- **Confidence:** High.
- **Primary source:** cmd/compile documentation (pkg.go.dev/cmd/compile).

#### GENC-04 · `//go:embed`

- **Behavior:** `//go:embed pattern...` immediately preceding a package-level
  `var` of type `string`, `[]byte`, or `embed.FS` embeds file contents at
  build time. Requires importing `embed` (blank import suffices for
  string/[]byte). Patterns match package-relative paths; `all:` includes
  `_`/`.`-prefixed entries otherwise skipped; `..`, absolute paths, and files
  outside the module are rejected.
- **Observable content:** All three variable types; a directory embed with
  `all:`; the blank `embed` import; embedded content referenced at run time.
- **Variants & failure cases:** Compile errors for local-variable embeds,
  missing files, or bad patterns — resource-inclusion edges from source to
  non-source files.
- **Constraints:** Go 1.16+.
- **Defined by:** Official implementation.
- **Confidence:** High.
- **Primary source:** embed package documentation (pkg.go.dev/embed).

#### GENC-05 · Ecosystem generators: stringer, protobuf, mocks

- **Behavior:** Common generators add semantic entities keyed to hand-written
  ones: `stringer` emits `x_string.go` with a `String()` method for a const
  enum (making it a `fmt.Stringer`); `protoc-gen-go` emits `.pb.go` files
  whose types register into a global protobuf registry at init; mock
  generators emit interface implementations.
- **Observable content:** A stringer-style generated file for a DECL-06 enum
  (checked in); a generated-file with init-time registration mirroring the
  `.pb.go` shape.
- **Variants & failure cases:** Generated methods change interface
  satisfaction (the enum becomes a Stringer only when generation ran);
  registry access is stringly-typed (DYN tie-in).
- **Constraints:** External tools; outputs are ordinary Go.
- **Defined by:** Ecosystem convention.
- **Confidence:** High.
- **Primary source:** golang.org/x/tools/cmd/stringer documentation.

---

### DIR — Compiler and linker directives

#### DIR-01 · `//go:linkname`

- **Behavior:** `//go:linkname localname [importpath.name]` aliases a local
  symbol to another package's (possibly unexported) symbol at link level,
  bypassing the visibility system; requires `import _ "unsafe"`. Creates
  hidden cross-package reference edges invisible to normal resolution.
- **Observable content:** A body-less func pulled from another package in the
  fixture via linkname; the required unsafe import.
- **Variants & failure cases:** Push vs pull forms; Go 1.23 tightened pulls
  into runtime/std internals (`-checklinkname` default on) — version- and
  target-sensitive legality.
- **Constraints:** gc-only; unsafe import; linker checks (1.23+).
- **Defined by:** Official implementation (gc).
- **Confidence:** High (medium on exact 1.23 enforcement scope).
- **Primary source:** cmd/compile documentation; Go 1.23 release notes.

#### DIR-02 · Performance/codegen pragmas

- **Behavior:** `//go:noinline`, `//go:nosplit`, `//go:noescape` (on body-less
  decls), `//go:norace`, `//go:uintptrescapes` alter gc compilation of the
  adjacent declaration. They do not change language semantics but are
  attached metadata a faithful graph may carry; `noescape` changes legality
  expectations around assembly stubs.
- **Observable content:** Annotated functions, including a `//go:noescape` on
  the BLD-06 assembly declaration.
- **Variants & failure cases:** Misplaced pragmas are silently ignored;
  gccgo/TinyGo differ.
- **Constraints:** gc-specific.
- **Defined by:** Official implementation (gc).
- **Confidence:** High.
- **Primary source:** cmd/compile documentation (pkg.go.dev/cmd/compile).

#### DIR-03 · Link-time variable injection (`-ldflags -X`)

- **Behavior:** `go build -ldflags "-X importpath.Var=value"` sets a
  package-level `string` variable at link time — a value with no source
  expression, commonly used for version stamping.
- **Observable content:** A `var version = "dev"` consumed at run time and
  documented as ldflags-set (build script or Makefile line in the repo).
- **Variants & failure cases:** Only settable strings (not consts, not
  initialized from function calls); wrong path silently no-ops.
- **Constraints:** cmd/link behavior.
- **Defined by:** Official implementation.
- **Confidence:** High.
- **Primary source:** cmd/link documentation (pkg.go.dev/cmd/link).

#### DIR-04 · WASM directives (`go:wasmimport`, `go:wasmexport`)

- **Behavior:** On wasm targets, `//go:wasmimport module name` binds a
  body-less func to a host import; `//go:wasmexport` (Go 1.24) exposes a Go
  function to the host — FFI edges without cgo.
- **Observable content:** A `//go:build wasip1`-constrained file with a
  wasmimport declaration and a pure-Go fallback elsewhere.
- **Variants & failure cases:** Type restrictions on signatures; target-gated
  (js/wasm vs wasip1 differences).
- **Constraints:** GOOS=js/wasip1, GOARCH=wasm; 1.21+/1.24+.
- **Defined by:** Official implementation.
- **Confidence:** Medium.
- **Primary source:** Go 1.21 and 1.24 release notes; cmd/compile documentation.

#### DIR-05 · Directive placement and lexical rules

- **Behavior:** `//go:` directives must be line comments with no space after
  `//`, immediately preceding the declaration they modify (build constraints:
  before the package clause, blank-line-separated). Misplacement silently
  produces an ordinary comment.
- **Observable content:** Correct directives plus near-miss fixtures
  (`// go:noinline` with a space; a build tag after the package clause).
- **Variants & failure cases:** Vet's `buildtag`/`directive` analyzers catch
  some misplacements; graphs treating directives as comments lose semantics.
- **Constraints:** gc lexical convention.
- **Defined by:** Official implementation.
- **Confidence:** High.
- **Primary source:** cmd/compile documentation; go vet documentation.

---

### CGO — Cgo and foreign-function surface

#### CGO-01 · `import "C"` and the preamble

- **Behavior:** The pseudo-package `C` exposes identifiers from the C
  preamble (the comment block immediately preceding `import "C"`). `C.foo`,
  `C.int`, `C.CString`, `C.GoString` etc. are entities that exist in no Go
  source file; the import is per-file and the preamble is file-local.
- **Observable content:** A cgo file with a small static C function in the
  preamble, called from Go; conversions between C and Go strings/slices.
- **Variants & failure cases:** `C` cannot be aliased or dot-imported; two
  files have independent preambles; exported Go funcs via `//export` impose
  preamble restrictions (declarations only in that file's preamble).
- **Constraints:** Requires a C toolchain and CGO_ENABLED=1; implies the
  `cgo` build tag.
- **Defined by:** Official implementation (cmd/cgo).
- **Confidence:** High.
- **Primary source:** cgo documentation (pkg.go.dev/cmd/cgo).

#### CGO-02 · `#cgo` directives and `//export`

- **Behavior:** `#cgo CFLAGS:/LDFLAGS:/pkg-config:` lines in the preamble
  configure compilation/linking, optionally per-GOOS/GOARCH; `//export Name`
  makes a Go function callable from C (generating a C header at build).
- **Observable content:** `#cgo` flags with a platform qualifier; one
  exported Go function referenced from preamble C code.
- **Variants & failure cases:** Security-filtered flag allowlist; Go 1.24
  added `#cgo noescape/nocallback` annotations for C function declarations.
- **Constraints:** cgo builds only.
- **Defined by:** Official implementation.
- **Confidence:** High (medium on the 1.24 annotation detail).
- **Primary source:** cgo documentation (pkg.go.dev/cmd/cgo); Go 1.24 release notes.

#### CGO-03 · CGO_ENABLED as a source-inclusion switch

- **Behavior:** With `CGO_ENABLED=0` (default when cross-compiling), cgo files
  are excluded and `!cgo` fallbacks compile instead; standard-library
  packages (`net`, `os/user`) switch between cgo and pure-Go resolvers the
  same way. The package's entity set differs per configuration.
- **Observable content:** A package with a cgo implementation and a
  `//go:build !cgo` pure-Go fallback of the same API (BLD-08 shape);
  optionally `netgo`/`osusergo` tag usage.
- **Variants & failure cases:** Behavior differences between the two
  implementations are a classic bug source; `purego` is a widespread
  *ecosystem* tag with the same intent (not toolchain-defined).
- **Constraints:** Environment-dependent builds.
- **Defined by:** Official implementation (+ ecosystem for `purego`).
- **Confidence:** High.
- **Primary source:** cgo documentation; go command documentation (environment variables).

---

### PLAT — Platform and configuration variance

#### PLAT-01 · GOOS/GOARCH API-surface variance

- **Behavior:** The compiled declaration set of a package can differ per
  target (via BLD-01/02): different types (e.g., syscall structures),
  different functions, different constants. Std packages (`syscall`,
  `os/signal`, `path/filepath` behaviors) model this too.
- **Observable content:** A fixture package whose exported surface differs by
  GOOS (one extra function on one platform), consumed by conditional callers.
- **Variants & failure cases:** A graph built for one target silently omits
  other targets' entities; `go vet` and `gopls` analyze per-configuration.
- **Constraints:** Cross-compilation is first-class (`GOOS=windows go build`).
- **Defined by:** Emergent from build constraints (official implementation).
- **Confidence:** High.
- **Primary source:** go command documentation; go/build documentation.

#### PLAT-02 · Platform-dependent numeric sizes

- **Behavior:** `int`, `uint`, `uintptr` are 32- or 64-bit depending on
  architecture (implementation-chosen). Constant representability and
  overflow diagnostics therefore vary by target.
- **Observable content:** A constant that fits `int` on 64-bit but is a
  compile error for a 32-bit target (documented in the fixture); `math.MaxInt`
  usage.
- **Variants & failure cases:** Alignment/size via `unsafe.Sizeof` differs
  too (TYPE-11); wasm is 64-bit-int on 32-bit-pointer nuance territory.
- **Constraints:** Target architecture.
- **Defined by:** Language (allows either) + implementation (chooses).
- **Confidence:** High.
- **Primary source:** Go Language Specification, Numeric types.

#### PLAT-03 · GOOS implication hierarchy and the `unix` tag

- **Behavior:** Build-tag satisfaction is hierarchical: `android` builds also
  satisfy `linux`; `ios` satisfies `darwin`; many GOOSes satisfy `unix`
  (1.17+). Files tagged at different levels of the hierarchy interact.
- **Observable content:** A `_linux.go` file, an `//go:build unix` file, and
  an `//go:build android` file with defined override relationships among
  their symbols.
- **Variants & failure cases:** Assuming `linux` excludes `android` is a
  common tooling bug.
- **Constraints:** go/build rules.
- **Defined by:** Official implementation.
- **Confidence:** High (medium on the complete implication table).
- **Primary source:** go/build package documentation.

#### PLAT-04 · Environment/config-dependent builds beyond GOOS

- **Behavior:** `GOFLAGS`, `-tags`, `GOEXPERIMENT`, `-race`, and `CGO_ENABLED`
  all change the selected file set or injected tags; two builds of the same
  tree are different programs. A fixture should pin its intended
  configuration(s) visibly (Makefile/taskfile/CI matrix committed as data).
- **Observable content:** Documented build matrix; tag-varied files exercised
  by it.
- **Variants & failure cases:** `GOEXPERIMENT` tags (`goexperiment.*`) gate
  std-library variants (e.g., jsonv2 in 1.25).
- **Constraints:** go command environment handling.
- **Defined by:** Official implementation.
- **Confidence:** High.
- **Primary source:** go command documentation (build modes, environment).

---

### DIAG — Invalid programs and diagnostics

#### DIAG-01 · Unused locals and imports (compile errors)

- **Behavior:** Unused local variables and unused imports are hard compile
  errors — unusual among mainstream languages and central to what "valid
  program" means in Go. Unused package-level vars, parameters, and constants
  are fine.
- **Observable content:** Error fixtures (as clearly quarantined non-building
  files, e.g., under `testdata`, or documented snippets) plus the legal
  contrasts.
- **Variants & failure cases:** `_ = x` suppression idiom; assignment-only
  use does not count as "use" for locals ("declared and not used" still
  fires on `x := 1; x = 2`).
- **Constraints:** None.
- **Defined by:** Language.
- **Confidence:** High.
- **Primary source:** Go Language Specification (implementation restriction notes).

#### DIAG-02 · Import cycle diagnostics

- **Behavior:** `import cycle not allowed` with the cycle path — a
  program-level structural error a graph should be able to predict from its
  own import edges.
- **Observable content:** A quarantined two-package cycle fixture.
- **Variants & failure cases:** Cycles through test files vs the IMP-04
  external-test escape.
- **Constraints:** None.
- **Defined by:** Language.
- **Confidence:** High.
- **Primary source:** Go Language Specification, Import declarations.

#### DIAG-03 · Initialization cycles

- **Behavior:** Package-level variables whose initializers form a dependency
  cycle (possibly through functions) are compile errors, distinct from import
  cycles.
- **Observable content:** A quarantined `var a = b; var b = a` fixture and a
  subtler one through a function call.
- **Variants & failure cases:** Breaking a cycle by moving assignment into
  `init` is the standard fix — same shape, now legal.
- **Constraints:** None.
- **Defined by:** Language.
- **Confidence:** High.
- **Primary source:** Go Language Specification, Package initialization.

#### DIAG-04 · Redeclaration and duplicate symbols

- **Behavior:** Redeclaring a name in the same scope is an error, including
  across files at package scope, across included build-constrained files
  (BLD-08 failure mode), and between test and non-test files under `go test`.
- **Observable content:** Quarantined duplicate-declaration fixtures for the
  same-file, cross-file, and cross-configuration cases.
- **Variants & failure cases:** Method redeclaration on the same receiver
  type; a field and method with the same name on one struct is also an error.
- **Constraints:** None.
- **Defined by:** Language.
- **Confidence:** High.
- **Primary source:** Go Language Specification, Declarations and scope.

#### DIAG-05 · Ambiguous promoted selectors

- **Behavior:** Two embedded types at equal depth providing the same name make
  `x.Name` an "ambiguous selector" error — but only at a use site; the type
  declaration itself is legal.
- **Observable content:** The legal type plus a quarantined use-site error and
  a legal explicit path (`x.A.Name`).
- **Variants & failure cases:** Depth-tiebreak legality when depths differ.
- **Constraints:** None.
- **Defined by:** Language.
- **Confidence:** High.
- **Primary source:** Go Language Specification, Selectors.

#### DIAG-06 · Interface-satisfaction error reporting at use sites

- **Behavior:** "T does not implement I (missing method M)" (or wrong
  signature / pointer-receiver notes) is reported where a value is assigned or
  passed, not where the type is declared — implements-relations are
  demand-checked.
- **Observable content:** A quarantined near-miss (pointer-receiver-only
  method assigned as value type) exercising the distinctive diagnostic.
- **Variants & failure cases:** The pointer-receiver hint text is a
  gc-specific diagnostic nicety.
- **Constraints:** None.
- **Defined by:** Language (rule); implementation (message).
- **Confidence:** High.
- **Primary source:** Go Language Specification, Interface types.

#### DIAG-07 · Missing function bodies

- **Behavior:** A body-less func without an assembly implementation or
  authorized directive fails ("missing function body") — the legality of
  BLD-06/DIR-01/DIR-04 declarations is conditional on their providers.
- **Observable content:** The legal assembly-backed decl beside a quarantined
  provider-less one.
- **Variants & failure cases:** Link-time vs compile-time surfacing differs
  by how the symbol is (not) provided.
- **Constraints:** gc behavior.
- **Defined by:** Official implementation.
- **Confidence:** High.
- **Primary source:** Go Language Specification, Function declarations; Go assembler documentation.

#### DIAG-08 · `go vet` findings vs compiler errors

- **Behavior:** Vet analyzers (printf, structtag, buildtag, copylocks,
  unreachable, loopclosure, tests, …) flag legal-but-suspect code — a second
  diagnostic tier with its own catalogue, partially enforced during `go test`
  (TEST-07).
- **Observable content:** One fixture per representative analyzer (printf
  mismatch, mutex copy, inert build tag), documented as vet-level.
- **Variants & failure cases:** Analyzer set varies by release; `loopclosure`
  findings largely evaporated at language ≥1.22 (DECL-09) — a
  version-sensitive diagnostic.
- **Constraints:** Tool, not language.
- **Defined by:** Official implementation.
- **Confidence:** High.
- **Primary source:** go vet documentation (pkg.go.dev/cmd/vet).

#### DIAG-09 · Type-error taxonomy probes

- **Behavior:** Distinctive Go type errors worth representing: assignment
  mismatch counts, non-boolean conditions, impossible type assertions,
  comparison of uncomparable types (slices/maps/funcs, `==` only vs `nil`),
  map-element addressability, string-immutability writes.
- **Observable content:** A quarantined set of one-liner error fixtures, one
  per class, each with the legal near-neighbor.
- **Variants & failure cases:** Uncomparable keys in map literals; struct
  comparability being contingent on all fields.
- **Constraints:** None.
- **Defined by:** Language.
- **Confidence:** High.
- **Primary source:** Go Language Specification, Comparison operators; Assignability.

---

### DYN — Dynamic and reflective escape hatches

#### DYN-01 · Reflection-based member access

- **Behavior:** `reflect.Value.MethodByName/FieldByName`, `reflect.New`, and
  `Type.Implements` perform string/runtime-driven member access and
  satisfaction checks that static graphs cannot resolve; linker dead-code
  elimination is disabled for methods when such calls are present with
  non-constant names.
- **Observable content:** A dispatcher invoking methods by name from a config
  string; a reflective interface check.
- **Variants & failure cases:** Unexported fields readable but not settable;
  reflect breaks the "unused = removable" assumption for methods.
- **Constraints:** None.
- **Defined by:** Std library (official).
- **Confidence:** High.
- **Primary source:** reflect package documentation (pkg.go.dev/reflect).

#### DYN-02 · Tag- and name-driven serialization/templating

- **Behavior:** `encoding/json` (tags + exported-field names),
  `text/template`/`html/template` (field/method access by string at run
  time) create data-flow relationships expressed in strings and struct tags
  rather than identifiers.
- **Observable content:** A struct round-tripped through JSON with renames; a
  template calling a method and reading fields by name.
- **Variants & failure cases:** Template references to missing members fail at
  execution; json omits unexported fields silently — visibility interacting
  with runtime behavior.
- **Constraints:** Std library semantics.
- **Defined by:** Std library (official) + tag conventions.
- **Confidence:** High.
- **Primary source:** encoding/json and text/template package documentation.

#### DYN-03 · `plugin` package

- **Behavior:** `plugin.Open` loads a `-buildmode=plugin` shared object at run
  time and looks up exported symbols by string — late-bound inter-binary
  edges. Heavily restricted: effectively linux/darwin, cgo required,
  identical toolchain/dependency versions.
- **Observable content:** A plugin main-package plus a host using
  `plugin.Lookup` (documented as platform-conditional).
- **Variants & failure cases:** Version-mismatch load failures; most projects
  use registries (IMP-02) or RPC instead — worth representing as the
  contrast.
- **Constraints:** Platform- and buildmode-restricted.
- **Defined by:** Official implementation.
- **Confidence:** Medium (exact platform support list).
- **Primary source:** plugin package documentation (pkg.go.dev/plugin).

#### DYN-04 · Init-time registries

- **Behavior:** The ecosystem-standard inversion: packages register factories
  into a shared map during `init`, consumers select by string key at run time
  (`database/sql.Register`, `image.RegisterFormat`, protobuf registry). The
  true "implements/provides" edges are only visible by tracing init side
  effects plus blank imports.
- **Observable content:** A local registry package, two provider packages
  registering via init, a consumer using blank imports and string lookup.
- **Variants & failure cases:** Missing blank import = runtime "unknown
  driver" error with zero compile-time signal.
- **Constraints:** None.
- **Defined by:** Ecosystem convention (std-modeled).
- **Confidence:** High.
- **Primary source:** database/sql package documentation (Register).

---

### ECO — Ecosystem conventions

#### ECO-01 · Doc comments, doc links, and deprecation

- **Behavior:** Doc comments immediately precede declarations; since Go 1.19
  they support structured links (`[Name]`, `[pkg.Name]`), lists, and
  headings — machine-readable references between docs and symbols. A
  paragraph beginning `Deprecated:` marks a symbol deprecated; tooling
  (gopls, staticcheck) surfaces uses.
- **Observable content:** Doc comments with intra- and cross-package `[...]`
  links; a deprecated function still referenced somewhere.
- **Variants & failure cases:** Link resolution depends on the file's imports;
  a `Deprecated:` mid-paragraph does not count.
- **Constraints:** Go 1.19+ for link syntax.
- **Defined by:** Official convention/tooling.
- **Confidence:** High.
- **Primary source:** Go Doc Comments (go.dev/doc/comment).

#### ECO-02 · Repository layout conventions

- **Behavior:** `cmd/<app>` for binaries, `internal/` for private code, no
  mandated `src/`; a repo may host multiple modules (tagged
  `subdir/vX.Y.Z`). Layout is convention except where the toolchain assigns
  meaning (`internal`, `testdata`, `vendor`, nested `go.mod`).
- **Observable content:** The conventional layout exercised together with its
  toolchain-meaningful subset.
- **Variants & failure cases:** A `pkg/` directory is convention-only; tools
  ascribing semantics to it are wrong.
- **Constraints:** None.
- **Defined by:** Ecosystem convention (partially toolchain-backed).
- **Confidence:** High.
- **Primary source:** go command documentation; Go Modules Reference.

#### ECO-03 · Module distribution: proxies, checksum DB, vanity imports

- **Behavior:** `GOPROXY`, `GONOSUMCHECK`-era → `GOSUMDB`, `GOPRIVATE`
  configure where modules come from and how they are verified; custom import
  paths resolve via `<meta name="go-import">` redirects. All affect which
  bytes back an import path — environment-dependent resolution.
- **Observable content:** For a self-contained fixture: documented
  `GOFLAGS=-mod=vendor` or fully replace-wired local deps so resolution is
  hermetic; notes for the networked variants.
- **Variants & failure cases:** `GOPROXY=off`/`direct`; private-module
  checksum exemptions.
- **Constraints:** go command environment.
- **Defined by:** Official implementation.
- **Confidence:** High.
- **Primary source:** Go Modules Reference (go.dev/ref/mod).

#### ECO-04 · Tool-dependency patterns (`tools.go` legacy)

- **Behavior:** Pre-1.24 convention: a build-tag-excluded `tools.go`
  (`//go:build tools`) blank-imports tool main packages so `go.mod` tracks
  them — intentional "imports" of main packages that never compile
  (main-package import ban is bypassed because the file is never built).
- **Observable content:** A legacy `tools.go` alongside (or contrasted with)
  MOD-11 `tool` directives.
- **Variants & failure cases:** If the tag ever matched, the build would fail
  on the main-package import — a nice consistency probe.
- **Constraints:** Convention; superseded at 1.24.
- **Defined by:** Ecosystem convention.
- **Confidence:** High.
- **Primary source:** Go Modules Reference / Go wiki convention (widely documented pattern).

#### ECO-05 · Canonical import-path comments (legacy)

- **Behavior:** `package foo // import "example.com/foo"` once enforced a
  canonical import path in GOPATH mode; module mode ignores it, but the
  syntax still appears in older code and tools may still parse it.
- **Observable content:** One file carrying the comment, documented as inert
  under modules.
- **Variants & failure cases:** Mismatch with actual path was an error in
  GOPATH mode only.
- **Constraints:** Legacy.
- **Defined by:** Official implementation (historical).
- **Confidence:** Medium (current tooling treatment).
- **Primary source:** go command documentation (historical sections).

---

## 4. Features Whose Support Changed Materially Between Recent Versions

| Version | Change | Checklist ties |
| --- | --- | --- |
| 1.16 | `//go:embed`; module mode default | GENC-04, MOD-01 |
| 1.17 | `//go:build` syntax; slice→array-pointer conversions; `unix`-adjacent groundwork | BLD-01, TYPE-09 |
| 1.18 | Generics; fuzzing; `go.work`; `any`/`comparable` predeclared | GEN-*, TEST-03, MOD-04, TYPE-12 |
| 1.19 | Doc-comment links and structure | ECO-01 |
| 1.20 | Slice→array conversions; `comparable` satisfied by non-strictly-comparable types | TYPE-09, IFACE-08 |
| 1.21 | `min`/`max`/`clear`; specified package-init order; toolchain management; per-file language versions; expanded inference; `go:wasmimport` | FUNC-07, DECL-05, MOD-08, BLD-04, GEN-02, DIR-04 |
| 1.22 | Per-iteration loop variables (language-version-gated); range over int; `math/rand/v2`; `go work vendor` | DECL-09, FUNC-08, IMP-06, MOD-04 |
| 1.23 | Range over functions (`iter`); generic-alias experiment; `//go:linkname` pull restrictions | FUNC-08, GEN-05, DIR-01 |
| 1.24 | Generic type aliases GA; `tool` directive; `go:wasmexport`; cgo `noescape`/`nocallback`; Swiss-map runtime (impl detail) | GEN-05, MOD-11, DIR-04, CGO-02 |
| 1.25 | "Core types" removed from spec prose (no intended semantic change); `go.mod` `ignore` directive; `GOEXPERIMENT=jsonv2`; container-aware GOMAXPROCS (runtime) | GEN-03, MOD-12, IMP-06 |

Version-gating mechanism itself (module `go` directive + per-file
`//go:build go1.N`) is a first-class fixture concern: identical text, different
graphs (BLD-04, DECL-09).

---

## 5. Implementation-Defined or Unspecified Behavior

- **`int`/`uint`/`uintptr` width** — implementation-chosen 32/64-bit (PLAT-02).
- **`unsafe.Sizeof/Alignof/Offsetof` results and struct layout** — implementation-defined (TYPE-11).
- **Map iteration order** — unspecified; gc randomizes deliberately per iteration.
- **`select` choice among ready cases** — specified only as uniform pseudo-random.
- **Evaluation order** — function calls and communications in one expression
  are ordered left-to-right, but other operand-ordering (e.g., between
  operands of a binary op, assignment LHS/RHS interleavings) is partially
  unspecified.
- **Order source files are presented to the compiler** — spec leaves it open;
  the go command sorts by filename, which fixes `init`-within-package order in
  practice (DECL-05).
- **Constant arithmetic precision** — spec sets minimums; gc implements
  effectively arbitrary precision.
- **Goroutine scheduling, GOMAXPROCS effects, stack growth** — runtime
  behavior, not semantics.
- **Inlining, escape analysis, devirtualization, PGO** — gc optimizations;
  must not affect a semantic graph (IFACE-04).
- **String/`[]byte` conversion copying elisions** — observable-behavior-preserving
  optimizations.
- **Diagnostic message texts** — gc-specific (DIAG-06); gccgo differs.
- **`print`/`println` builtins** — implementation-provided, output format and
  destination unspecified.
- **Hash seeds** (map, `hash/maphash` default) — randomized per process.

---

## 6. Areas Where My Knowledge May Be Incomplete

- **Go 1.25 specifics** (the `ignore` directive's exact matching rules, final
  jsonv2 surface, spec rewording scope) and anything in the Go 1.26 cycle
  (expected February 2026) — post-cutoff details.
- **Exact `//go:linkname` enforcement matrix** after the 1.23 tightening
  (which std internals remain pull-able, `-checklinkname` interactions).
- **Complete GOOS implication table and `unix` membership list** in current
  go/build.
- **The precise, current `go vet` analyzer set** and which subset runs under
  `go test` per release.
- **cgo generated-file internals** (`_cgo_gotypes.go` naming and contents) —
  I treat them as build artifacts, but tools that scan build caches may see
  them.
- **gccgo/TinyGo divergences** beyond pragma support — I have not detailed
  them.
- **`plugin` package platform support list** as of 1.25.
- **Fuzz corpus file format details** (encoding of corpus entries).
- **GOEXPERIMENT catalogue** (e.g., status of arenas, greenteagc) — moving
  target.

---

## 7. Final Audit — Categories or Families Possibly Still Missing

Reviewed against the checklist; candidates a fixture designer may still want
to weigh:

1. **Runtime-only observability constructs** — `runtime.SetFinalizer` /
   `runtime.AddCleanup` (1.24), `weak` pointers (1.24), `unique` interning
   (1.23): they create lifecycle relationships (callbacks held by the
   runtime) but arguably sit below code-graph altitude. Not itemized.
2. **Dynamically named test subtests** (`t.Run(fmt.Sprintf(...))`) — noted in
   TEST-03 but could be its own item: statically unenumerable test entities.
3. **Build info stamping** (`runtime/debug.ReadBuildInfo`, VCS metadata
   embedding) — source-external data compiled into binaries; adjacent to
   DIR-03 but not itemized.
4. **`-buildmode` variants** beyond plugin (c-archive, c-shared, pie) — they
   change exported-symbol surfaces for FFI consumers; only plugin (DYN-03)
   and cgo `//export` (CGO-02) are covered.
5. **`arena`-style experimental APIs and `GOEXPERIMENT` std variance** —
   mentioned (PLAT-04, §6) but not itemized per-experiment.
6. **`gopls`/`go/packages` load modes** — how tools (not the compiler)
   partition packages (test variants, overlays); deliberately excluded as
   tool-specific, but any graph builder consuming `go/packages` inherits
   these variants (notably the *four* package variants per directory under
   test: package, package-under-test, external test, synthesized main).
7. **Comment-position subtleties** — doc-comment association rules when
   directives and blank lines interleave; partially covered (DIR-05, ECO-01).
8. **Deprecated language corners** — e.g., `string(int)` conversion vet
   warning (`stringintconv`), octal-literal styles (`0o` vs legacy `0`),
   underscore digit separators: lexical variety worth sprinkling through any
   fixture but too fine-grained to itemize here.
9. **Symbol identity across build configurations** — BLD-08 raises it; a
   dedicated treatment of "what is *the* entity when N conditional
   definitions exist" may deserve first-class status in any evaluation
   rubric.
10. **Error-wrapping conventions** (`fmt.Errorf` with `%w`, `errors.Is/As`) —
    semantic chains encoded in format strings; library-level, adjacent to
    DYN-02, not itemized.

These are judged lower-yield or tool-specific relative to the itemized set,
but none are unreasonable additions.
