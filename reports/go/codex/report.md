# Go Semantic Coverage Research Checklist

## Scope and baseline

This report is a candidate checklist for a broad, idiomatic, self-contained Go project intended to expose materially distinct language, package, toolchain, build, and ecosystem behavior. It describes coverage, not a fixture design, graph schema, scoring model, or analyzer strategy.

- **Baseline language:** Go 1.23 language semantics, with the module declaring an explicit `go` language version.
- **Baseline implementation:** the official `gc` toolchain distributed by the Go project, using module-aware builds.
- **Baseline host matrix:** at least one Unix-like target plus a second `GOOS` or `GOARCH` combination. Cgo-enabled and cgo-disabled builds are distinct configurations.
- **Version policy:** syntax and semantics are interpreted according to the module or file language version, not merely the installed toolchain version. Experimental features require their named `GOEXPERIMENT` setting.
- **Dependency policy:** direct, indirect, replaced, vendored, workspace-local, standard-library, and tool-only dependencies are distinct forms even when they resolve to identical source.
- **Knowledge boundary:** confidence is strongest through Go 1.23. Generic type aliases and later toolchain behavior must be checked against release notes for the exact installed Go version.

## Checklist

### Source files, packages, names, and visibility

#### GO-SRC-001 — Multi-file package formation

- **Distinct behavior:** Non-test `.go` files selected for one directory and configuration contribute declarations to one package scope even though each file has its own file scope.
- **Observable content:** A package split across several files, with declarations in one file referring to package-level declarations in another.
- **Variants and failures:** File order must not create name-resolution dependencies. Selected files with different package clauses fail package loading; `_test` packages are handled separately.
- **Constraints:** Only files selected for the active build configuration participate. Directory layout and module boundaries affect package identity.
- **Authority:** Language definition plus official `go` command behavior.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*; `go help packages`; `go help buildconstraint`.

#### GO-SRC-002 — Package identity versus package name

- **Distinct behavior:** An imported package is identified by import path, while source selectors normally use its declared package name or an explicit import name; these strings need not match.
- **Observable content:** A package whose final import-path element differs from its package clause, imported both normally and with an alias.
- **Variants and failures:** Two imported packages with the same declared name require aliases. The special `main` package name determines executable behavior, not an import-path suffix.
- **Constraints:** Import paths resolve through modules, the standard library, vendoring, or legacy modes.
- **Authority:** Language definition and official toolchain.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*; *Go Modules Reference*.

#### GO-SRC-003 — Package, file, block, and lexical scopes

- **Distinct behavior:** Go has universe, package, file, function, and nested block scopes with precise declaration points; a nearer declaration shadows an outer one without changing its identity.
- **Observable content:** Package names, file-scoped imports, parameters, named results, short declarations, loop variables, and nested locals that intentionally reuse a spelling.
- **Variants and failures:** A declaration may be in scope only after part of its declaration. Labels have function-wide label scope. Illegal redeclarations differ from legal shadowing.
- **Constraints:** Scope and declaration point are language-version-sensitive for changed loop rules.
- **Authority:** Language definition.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*.

#### GO-SRC-004 — Universe identifiers and shadowing

- **Distinct behavior:** Predeclared types, constants, functions, and `nil` live in the universe block and can usually be shadowed by user declarations.
- **Observable content:** Local or package declarations shadowing representative predeclared identifiers, alongside unshadowed uses.
- **Variants and failures:** Shadowing changes resolution rather than mutating a built-in. Some predeclared names were added in recent versions and are unavailable under older language versions.
- **Constraints:** `any` and `comparable` arrived with generics; `clear`, `min`, and `max` arrived in Go 1.21.
- **Authority:** Language definition.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*; Go release notes.

#### GO-SRC-005 — Import declaration forms

- **Distinct behavior:** Imports can use the package's default name, an explicit alias, the blank identifier for side effects, or dot import to inject exported names into file scope.
- **Observable content:** Separate files demonstrating all four forms and cross-package references or initialization effects.
- **Variants and failures:** Imports are file-scoped. Dot imports can cause collisions. Blank imports create dependency and initialization edges without selector references. Unused non-blank imports are errors.
- **Constraints:** Import cycles are forbidden. Some import paths are restricted by `internal` and module rules.
- **Authority:** Language definition plus official package loader.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*; `go help packages`.

#### GO-SRC-006 — Exported identifiers and Unicode case

- **Distinct behavior:** Package visibility is spelling-based: an identifier is exported when its first Unicode character is an uppercase letter and the identifier is declared in a package block or is a field or method name.
- **Observable content:** Exported and unexported package declarations, fields, methods, embedded names, and at least one non-ASCII identifier.
- **Variants and failures:** Export does not bypass `internal` import restrictions. Unexported members of different packages are never identical merely because their spelling matches.
- **Constraints:** Unicode categories and source encoding follow the language specification.
- **Authority:** Language definition.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*.

#### GO-SRC-007 — Blank identifier semantics

- **Distinct behavior:** `_` discards values and suppresses otherwise required bindings, but it does not introduce a usable declaration or ordinary symbol identity.
- **Observable content:** Blank imports, blank fields, ignored assignment and return values, compile-time interface assertions, and blank range variables.
- **Variants and failures:** Blank fields affect struct layout and comparability but cannot be selected. Assigning an untyped value to `_` can still require type checking.
- **Constraints:** Context determines whether `_` is a declaration target, assignment target, import name, or field name.
- **Authority:** Language definition.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*.

#### GO-SRC-008 — Labels and their separate namespace

- **Distinct behavior:** Labels are scoped to an entire function body and do not conflict with variables, types, functions, or package names.
- **Observable content:** Labeled loops and statements with `break`, `continue`, and `goto`, including a variable sharing the label spelling.
- **Variants and failures:** An unused label is invalid. `goto` cannot jump over variable declarations that would enter scope, nor into a block.
- **Constraints:** Labels do not cross function literal boundaries.
- **Authority:** Language definition.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*.

#### GO-SRC-009 — Compile-time unused-name diagnostics

- **Distinct behavior:** Unused imports and declared local variables are compile errors, while unused package-level declarations, parameters, and results are permitted.
- **Observable content:** Isolated invalid files or packages for unused import and local cases, plus valid unused package members and parameters.
- **Variants and failures:** Assigning to `_` can make an otherwise unused value acceptable. Whether an import is used is determined per file, not per package.
- **Constraints:** Diagnostics and exact wording are implementation-specific; validity is language-defined.
- **Authority:** Language definition; diagnostic presentation by compiler.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*; `go` compiler documentation.

#### GO-SRC-010 — Comments, semicolon insertion, and source positions

- **Distinct behavior:** Newlines can insert semicolons, comments can affect token separation, and line directives can remap reported file names and positions without changing physical source identity.
- **Observable content:** Formatting-sensitive valid and invalid constructs, ordinary comments, and both forms of line directive with diagnostics after remapping.
- **Variants and failures:** A newline before a selector dot is invalid because of semicolon insertion. Malformed line directives remain comments or produce implementation-dependent diagnostics.
- **Constraints:** Source must be UTF-8 with implementation restrictions permitted for some code points; line-directive interpretation is specified by the language.
- **Authority:** Language definition.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*.

### Declarations, initialization, and executable entry

#### GO-DEC-001 — Declaration groups and dependent declarations

- **Distinct behavior:** Parenthesized declaration groups and individual declarations introduce constants, variables, types, and functions with category-specific declaration points and dependency rules.
- **Observable content:** Grouped and ungrouped declarations whose initializers and types refer to earlier and later package declarations.
- **Variants and failures:** Package-level declaration order is mostly irrelevant to resolution, but initialization dependencies determine execution order. Duplicate top-level names are invalid apart from special `init` functions.
- **Constraints:** Package scope spans selected files.
- **Authority:** Language definition.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*.

#### GO-DEC-002 — Package variable initialization dependencies

- **Distinct behavior:** Package variables initialize in dependency order determined by references in initialization expressions and function or method bodies involved in those expressions.
- **Observable content:** Several package variables across files with direct and indirect initialization dependencies and observable side effects.
- **Variants and failures:** Independent variables follow presentation order chosen by the compiler. Hidden dependencies through interfaces or dynamic calls may not count as specification dependencies.
- **Constraints:** The compiler is encouraged, but not required by the language, to present files in lexical file-name order.
- **Authority:** Language definition with an intentionally implementation-sensitive ordering edge.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*.

#### GO-DEC-003 — Multiple `init` functions

- **Distinct behavior:** A package can declare multiple parameterless, resultless functions named `init`; they are not ordinary package members and cannot be referenced.
- **Observable content:** Multiple `init` declarations in one file and across files, each producing an observable registration or state change.
- **Variants and failures:** A declared `init` identifier cannot be called. Incorrect signatures are invalid. Ordering follows package initialization rules and file presentation.
- **Constraints:** Test binaries may add generated initialization around user packages.
- **Authority:** Language definition and official test tool behavior.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*; `go help test`.

#### GO-DEC-004 — Imported-package initialization order

- **Distinct behavior:** A program initializes imported packages before the importing package, once per package instance, in dependency order before invoking local `init` functions.
- **Observable content:** A diamond-shaped package dependency with state changes in package initializers and a blank-imported registration package.
- **Variants and failures:** Import cycles are rejected. Multiple paths to the same package do not cause repeated initialization.
- **Constraints:** Package identity and selected module version determine whether imports denote the same package instance.
- **Authority:** Language definition and official linker/toolchain.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*.

#### GO-DEC-005 — Initialization cycles within a package

- **Distinct behavior:** Cycles among package variable initialization dependencies are invalid even though ordinary recursive function calls are allowed.
- **Observable content:** An invalid package with direct and indirect variable cycles, contrasted with valid mutually recursive functions.
- **Variants and failures:** Dependencies can be induced through referenced function or method bodies under the specification's rules. Diagnostics may choose a different cycle path.
- **Constraints:** Cycle determination is compile-time and package-local.
- **Authority:** Language definition.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*.

#### GO-DEC-006 — `main` package and entry function

- **Distinct behavior:** An executable program requires package `main` and a top-level parameterless, resultless function named `main`; that function is not invoked as an ordinary exported entry point by the runtime.
- **Observable content:** A command package with `main`, an importable library package, and invalid command variants with missing or wrong-signature entry functions.
- **Variants and failures:** Package `main` may have any import path. Its `main` function and `init` functions cannot be referenced from another package in the normal model.
- **Constraints:** Build modes can alter artifact form but not the source-level signature rule.
- **Authority:** Language definition and official linker.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*; `go help buildmode`.

### Constants and core type identity

#### GO-TYP-001 — Untyped constants and default types

- **Distinct behavior:** Constants may have arbitrary precision and remain untyped until context supplies a type; otherwise they receive a category-specific default type.
- **Observable content:** Untyped Boolean, rune, integer, floating-point, complex, and string constants used in assignments, calls, comparisons, and generic inference.
- **Variants and failures:** The same constant may be representable in one destination and overflow another. Constant expressions can behave differently from runtime expressions.
- **Constraints:** Compiler resource limits may bound implementation capacity while preserving required precision behavior.
- **Authority:** Language definition with implementation resource limits.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*.

#### GO-TYP-002 — `iota` and implicit constant expressions

- **Distinct behavior:** `iota` is an untyped integer index within a constant declaration and omitted expression lists repeat the previous non-empty list with a new `iota` value.
- **Observable content:** A grouped constant declaration using explicit expressions, omitted expressions, multiple values, shifts, and `_` gaps.
- **Variants and failures:** `iota` resets for each constant declaration and is zero inside a nested constant declaration's expression context according to its own declaration.
- **Constraints:** Constant representability rules apply when values receive types.
- **Authority:** Language definition.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*.

#### GO-TYP-003 — Defined types versus type aliases

- **Distinct behavior:** A type definition creates a new, distinct named type; an alias denotes the identical target type and does not create a new type identity.
- **Observable content:** A defined type and alias over the same underlying type, with assignments, conversions, methods, reflection, and cross-package uses.
- **Variants and failures:** Methods can generally be declared only on a locally defined receiver type. Alias receiver restrictions are stricter, especially when the target is instantiated or generic.
- **Constraints:** Alias declarations require Go 1.9; aliases with type parameters are version- and experiment-sensitive.
- **Authority:** Language definition.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*; Go release notes.

#### GO-TYP-004 — Underlying types, assignability, and conversions

- **Distinct behavior:** Type identity, identical underlying types, assignability, and explicit convertibility are different relations; named values often require conversion where unnamed values do not.
- **Observable content:** Assignments and conversions among defined and unnamed scalar, composite, pointer, channel, and generic types.
- **Variants and failures:** Assignment can also depend on untyped representability, interface implementation, channel direction, `nil`, and type-parameter type sets. Conversion may allocate or reinterpret depending on the types.
- **Constraints:** Unsafe conversions are governed separately from ordinary conversions.
- **Authority:** Language definition.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*.

#### GO-TYP-005 — Implementation-sized numeric types

- **Distinct behavior:** `int`, `uint`, and `uintptr` have implementation-specific width, while the explicitly sized integer and floating types have fixed widths and distinct identities.
- **Observable content:** Constants at width boundaries, conversions, shifts, `unsafe.Sizeof`, and build variants for 32-bit and 64-bit targets.
- **Variants and failures:** A constant may compile for one architecture and overflow on another. `byte` and `rune` are aliases, not new defined types.
- **Constraints:** Width is target-dependent; integer overflow at runtime follows language rules.
- **Authority:** Language definition plus target implementation.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*; `go help environment`.

#### GO-TYP-006 — Array length as type identity

- **Distinct behavior:** Array length is part of type identity and must be a non-negative representable constant; arrays have value-copy semantics.
- **Observable content:** Arrays with differing lengths, inferred-length literals, named and unnamed arrays, assignments, comparisons, and pointer-to-array slicing.
- **Variants and failures:** Comparable element types make arrays comparable. Lengths inferred by composite literals depend on indexed elements. Generic array terms can constrain permitted operations.
- **Constraints:** Very large arrays can exceed implementation limits despite a valid constant length.
- **Authority:** Language definition with implementation resource limits.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*.

#### GO-TYP-007 — Slice identity, aliasing, and capacity

- **Distinct behavior:** A slice is a descriptor over an underlying array; slicing and assignment can create aliases, while append may reuse or replace backing storage.
- **Observable content:** Nil and non-nil empty slices, two- and three-index slicing, overlapping copy, append with and without capacity growth, and array-to-slice derivation.
- **Variants and failures:** Index and slice bounds can fail at compile time for constants or panic at runtime. Append growth strategy is not specified.
- **Constraints:** `unsafe` and cgo can impose extra lifetime and pointer restrictions.
- **Authority:** Core semantics are language-defined; allocation and growth are implementation-defined.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*.

#### GO-TYP-008 — Map key constraints and map identity

- **Distinct behavior:** Map key types must be comparable; map values are not addressable, and lookup has one-value and comma-ok forms.
- **Observable content:** Maps keyed by scalar, array, struct, pointer, and interface types, with insert, lookup, delete, clear, nil-map reads, and invalid key types.
- **Variants and failures:** An interface key can panic during hashing or comparison when its dynamic value is not comparable. Writes to a nil map panic. Iteration order is unspecified.
- **Constraints:** `clear` requires Go 1.21. Concurrent unsynchronized access has separate runtime and memory-model consequences.
- **Authority:** Language definition; some runtime diagnostics are implementation behavior.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*; Go 1.21 release notes.

#### GO-TYP-009 — Struct fields, tags, and identity

- **Distinct behavior:** Field names, ordering, types, tags, and embedding contribute to struct identity under exact rules; tags are ignored for some conversion rules but visible through reflection.
- **Observable content:** Named and anonymous struct types with exported and unexported fields, blank fields, distinct tags, and keyed and positional literals.
- **Variants and failures:** Unkeyed literals of another package's struct with unexported fields are invalid. Duplicate non-blank field names are invalid. Tags can change ecosystem behavior without changing field selection.
- **Constraints:** Layout and padding are implementation-dependent subject to size and alignment guarantees.
- **Authority:** Language definition; tag meaning is package convention.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*; `reflect` package documentation.

#### GO-TYP-010 — Embedded fields and selector promotion

- **Distinct behavior:** Embedded fields promote fields and methods to outer selector and method sets according to depth, uniqueness, and pointer/value rules.
- **Observable content:** Multi-level value and pointer embedding, promoted fields and methods, explicit paths, and same-depth name collisions.
- **Variants and failures:** An ambiguous promoted selector is invalid only when selected; the containing type can otherwise remain valid. Promotion does not change declaration ownership or export status.
- **Constraints:** An embedded field must be a permitted type name or pointer form under the active language version.
- **Authority:** Language definition.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*.

#### GO-TYP-011 — Pointer types and addressability

- **Distinct behavior:** Addressability controls use of `&`, assignment through selectors and indexes, implicit method-call addressing, and which composite values may be mutated.
- **Observable content:** Addressable variables, fields, array elements, slice elements, non-addressable map elements and temporary results, plus pointer composite literals.
- **Variants and failures:** The compiler may implicitly take an address for a method call on an addressable value, but not for a method value obtained from a non-addressable value. Pointers to distinct named types remain distinct.
- **Constraints:** Pointer arithmetic is absent outside `unsafe`; cgo pointer-passing rules add runtime constraints.
- **Authority:** Language definition; cgo rules by official toolchain.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*; cgo command documentation.

#### GO-TYP-012 — Function types, signatures, and variadic parameters

- **Distinct behavior:** Function type identity depends on parameter and result sequences and variadic status, but not parameter names; functions support multiple results and are otherwise comparable only to `nil`.
- **Observable content:** Named and anonymous function types, variadic and slice-taking functions, multiple and named results, callbacks, closures, and nil function values.
- **Variants and failures:** Variadic expansion requires an assignable slice of the final element type. Generic functions require instantiation or inference before use as ordinary function values in many contexts.
- **Constraints:** Calls across cgo or assembly boundaries obey additional application binary interface constraints.
- **Authority:** Language definition; application binary interface behavior by implementation.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*.

#### GO-TYP-013 — Channel direction and assignability

- **Distinct behavior:** Bidirectional, send-only, and receive-only channel types are distinct, with special assignability rules allowing direction restriction.
- **Observable content:** Directional parameters and results, conversions or assignments from bidirectional channels, sends, receives, close, and invalid opposite-direction operations.
- **Variants and failures:** Defined versus unnamed channel types affect assignability. Receive has single-value and comma-ok forms. Closing or sending on a closed channel panics.
- **Constraints:** Channel operations participate in the memory model and scheduler.
- **Authority:** Language definition and Go memory model.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*; *The Go Memory Model*.

#### GO-TYP-014 — Recursive type validity

- **Distinct behavior:** Recursive types are valid only when recursion passes through an indirection that prevents infinite size; aliases and generic instantiations can expose less obvious cycles.
- **Observable content:** Valid recursion through pointers, slices, maps, channels, interfaces, or functions, plus invalid direct array or struct cycles.
- **Variants and failures:** Mutually recursive declarations may be valid or invalid depending on the complete path. Generic instantiation cycles can fail by infinite expansion or implementation restrictions.
- **Constraints:** Exact generic cycle restrictions can be sensitive to compiler version.
- **Authority:** Language definition with compiler diagnostics.
- **Confidence:** Medium.
- **Likely primary source:** *The Go Programming Language Specification*; compiler release notes and issue documentation if needed.

### Methods, interfaces, and generic types

#### GO-MET-001 — Receiver declarations and method ownership

- **Distinct behavior:** A method is declared on a local defined receiver base type, or pointer to it, and belongs to that receiver's method namespace rather than package function namespace.
- **Observable content:** Value and pointer receiver methods, a package function with the same name, receiver parameter renaming, and invalid methods on foreign or disallowed alias types.
- **Variants and failures:** Receiver base types must be defined in the same package and cannot be pointer or interface types. Generic receiver declarations implicitly declare receiver type parameters corresponding to the base type.
- **Constraints:** Alias and instantiated-type receiver restrictions changed as generic aliases evolved.
- **Authority:** Language definition.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*.

#### GO-MET-002 — Value and pointer method sets

- **Distinct behavior:** The method set of a defined value type includes value-receiver methods, while the pointer method set includes both pointer- and value-receiver methods.
- **Observable content:** Interface assignments, calls through values and pointers, addressable implicit calls, and non-addressable values that fail pointer-method calls.
- **Variants and failures:** Syntactic call shorthand does not enlarge the value type's method set for interface satisfaction. Embedded fields add separate promotion rules.
- **Constraints:** Receiver type arguments and constraints can affect generic method sets.
- **Authority:** Language definition.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*.

#### GO-MET-003 — Method values and method expressions

- **Distinct behavior:** A method value binds a receiver and yields a function value, while a method expression yields a function whose first explicit parameter is the receiver.
- **Observable content:** Value- and pointer-receiver methods used as direct calls, bound method values, method expressions, callbacks, and nil receiver cases.
- **Variants and failures:** Receiver evaluation occurs when a method value is formed. Method expressions follow method-set rules. A valid call can still panic inside a method on a nil receiver.
- **Constraints:** Generic receiver instantiation must be resolved before the resulting function signature is known.
- **Authority:** Language definition.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*.

#### GO-MET-004 — Implicit interface implementation

- **Distinct behavior:** Types implement interfaces structurally and implicitly through method sets; no declaration names the implemented interface.
- **Observable content:** Value and pointer types assigned to multiple interfaces, compile-time assertion idioms, cross-package unexported method cases, and a deliberate missing-method failure.
- **Variants and failures:** Pointer-receiver methods can make only the pointer implement an interface. Unexported method identity includes its declaring package. Embedding can promote implementation or create ambiguity.
- **Constraints:** Constraint-only interfaces have restrictions on use as runtime value types.
- **Authority:** Language definition.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*.

#### GO-MET-005 — Interface value dynamic type and typed nil

- **Distinct behavior:** An interface value carries a dynamic type and dynamic value; an interface holding a typed nil pointer is not itself nil.
- **Observable content:** Nil interfaces, interfaces containing typed nil pointers, equality checks, method calls, type assertions, formatting, and reflection.
- **Variants and failures:** Calling a promoted or concrete method through a typed nil may succeed or panic depending on method code. Comparing interfaces can panic if dynamic values are not comparable.
- **Constraints:** Representation is implementation-defined, but observable equality and dispatch semantics are language-defined.
- **Authority:** Language definition.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*.

#### GO-MET-006 — Interface embedding, method identity, and collisions

- **Distinct behavior:** Basic interfaces define intersections of method sets; embedding combines requirements, and methods with the same name must have identical signatures.
- **Observable content:** Direct and embedded interface methods across packages, duplicate identical methods, conflicting signatures, and unexported methods.
- **Variants and failures:** An interface can embed a named basic interface or type elements when used as a constraint. Duplicate method declarations with mismatched signatures are invalid.
- **Constraints:** General interfaces containing type terms cannot be used freely as ordinary value types.
- **Authority:** Language definition.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*.

#### GO-MET-007 — Type assertions and type switches

- **Distinct behavior:** Type assertions refine an interface value to a concrete type or another interface and can panic or return comma-ok; type switches introduce per-case dynamic-type knowledge.
- **Observable content:** Successful and failing assertions, comma-ok assertions, assertions between interfaces, nil-interface cases, and type switches with concrete, interface, nil, and default cases.
- **Variants and failures:** In a single-type case the switch variable has that case type; in multi-type and default cases it retains the interface type. Duplicate or impossible cases can be invalid.
- **Constraints:** Assertions on type parameters are restricted; conversion to an interface first may be needed.
- **Authority:** Language definition.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*.

#### GO-GEN-001 — Generic type and function declarations

- **Distinct behavior:** Type parameters introduce a separate parameter list with constraints; generic declarations denote families that must be instantiated before most ordinary use.
- **Observable content:** Generic functions and defined types with one and several type parameters, cross-parameter constraints, explicit instantiations, and uses across packages.
- **Variants and failures:** Methods may use receiver type parameters but cannot declare their own additional method type parameters. Generic declarations cannot be used as uninstantiated runtime values.
- **Constraints:** Generics require Go 1.18 or later and the package's declared language version governs acceptance.
- **Authority:** Language definition.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*; Go 1.18 release notes.

#### GO-GEN-002 — Constraint type sets

- **Distinct behavior:** Interface constraints can describe type sets using methods, exact type terms, underlying-type terms with `~`, unions, intersections, and `comparable`.
- **Observable content:** Constraints covering named types with common underlying types, union terms, embedded constraints, methods plus type terms, and empty type sets.
- **Variants and failures:** Overlapping non-interface union terms and illegal uses of `comparable` or method-bearing interfaces in unions are rejected. A constraint can be syntactically valid but have an empty type set.
- **Constraints:** General constraint interfaces are not ordinary basic interface value types.
- **Authority:** Language definition.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*.

#### GO-GEN-003 — Instantiation and constraint satisfaction

- **Distinct behavior:** Instantiation substitutes type arguments, checks constraint satisfaction, and creates concrete function or type identities tied to the generic declaration and arguments.
- **Observable content:** Several valid instantiations of one declaration, nested generic types, an alias or field referring to an instantiation, and invalid arguments.
- **Variants and failures:** Go 1.20 changed `comparable` constraint satisfaction so some comparable interfaces satisfy it even though their type set is not strictly comparable. Runtime comparison can still panic for unsuitable dynamic values.
- **Constraints:** Requires Go 1.18; exact satisfaction rules depend on declared language version.
- **Authority:** Language definition and versioned language changes.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*; Go 1.20 release notes.

#### GO-GEN-004 — Type inference

- **Distinct behavior:** Function arguments, assignment context, constraints, and relationships among type parameters can infer omitted type arguments.
- **Observable content:** Full and partial explicit type argument lists, argument-based inference, assignment of generic functions to typed variables, interface-method inference, and intentionally ambiguous calls.
- **Variants and failures:** Untyped constants use representative types during inference. Inference capabilities expanded in Go 1.21 and Go 1.22; code can fail under an older module language version.
- **Constraints:** Inference does not apply in every syntactic context and never guesses among unresolved alternatives.
- **Authority:** Language definition and release-specific changes.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*; Go 1.21 and Go 1.22 release notes.

#### GO-GEN-005 — Operations on type parameters

- **Distinct behavior:** An operation is permitted on a type parameter only when it is valid with a compatible type and result across the constraint's entire type set.
- **Observable content:** Arithmetic, comparison, indexing, slicing, channel, field, conversion, and method operations under narrow and broad constraints.
- **Variants and failures:** A union whose members support superficially similar operations may still not supply identical operand or result requirements. Field selection on structural type sets is notably limited.
- **Constraints:** Rules have received clarifications and implementation fixes since Go 1.18.
- **Authority:** Language definition; edge acceptance may be compiler-version-sensitive.
- **Confidence:** Medium.
- **Likely primary source:** *The Go Programming Language Specification*; Go compiler release notes.

#### GO-GEN-006 — Generic named types and receiver type parameters

- **Distinct behavior:** Methods on an instantiated generic defined type use receiver-declared type parameters corresponding to the base type's parameters, with constraints inherited from the base declaration.
- **Observable content:** A generic type with value and pointer methods, renamed receiver parameters, instantiated method calls, promoted generic methods, and interface satisfaction.
- **Variants and failures:** Receiver parameter names need not match the base declaration. Receiver forms that denote aliases, instantiated aliases, or foreign types are invalid.
- **Constraints:** Generic receiver syntax requires Go 1.18 or later.
- **Authority:** Language definition.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*.

#### GO-GEN-007 — Generic type aliases

- **Distinct behavior:** A generic alias introduces parameters while preserving identity with an instantiated target rather than creating a defined type.
- **Observable content:** A parameterized alias to a local generic type used in assignments, signatures, embedding, and reflection, plus receiver-declaration failures.
- **Variants and failures:** Go 1.23 exposed generic aliases behind `GOEXPERIMENT=aliastypeparams`, with known package-boundary limitations in that preview; later releases materially changed support.
- **Constraints:** Exact stable availability and cross-package behavior must be verified against the selected toolchain's release notes.
- **Authority:** Language definition when enabled or stabilized; experiment behavior by official implementation.
- **Confidence:** Medium.
- **Likely primary source:** Go 1.23 and subsequent Go release notes; *The Go Programming Language Specification* for the chosen version.

### Expressions, calls, and control flow

#### GO-EXP-001 — Selector resolution

- **Distinct behavior:** The same selector syntax can resolve a package member, direct field or method, promoted field or method, or method on an implicitly addressed receiver.
- **Observable content:** Qualified identifiers, direct and promoted selectors, pointer dereference shorthand, same-depth ambiguity, and selectors on instantiated generic types.
- **Variants and failures:** Package-qualified selectors expose only exported names outside the declaring package. Ambiguous promoted selectors fail at use. A field and method cannot coexist at the same direct selector level with one name.
- **Constraints:** Selection depends on the static type and addressability of the left operand.
- **Authority:** Language definition.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*.

#### GO-EXP-002 — Composite literals

- **Distinct behavior:** Composite literals construct struct, array, slice, and map values with keyed or positional elements, nested type elision, and special address-taking behavior.
- **Observable content:** Keyed and unkeyed struct literals, indexed arrays and slices, map literals, nested omitted element types, pointer literals, empty literals, and literals of named generic instantiations.
- **Variants and failures:** Duplicate keys or indexes, mixed keyed and unkeyed struct elements, unexported foreign fields, excess positional fields, and non-constant array indexes are invalid as applicable.
- **Constraints:** Taking the address of an empty slice or map literal has different allocation and nil behavior from `new` of the corresponding type.
- **Authority:** Language definition.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*.

#### GO-EXP-003 — Indexing and slicing across types

- **Distinct behavior:** Indexing and slicing have distinct typing, addressability, bounds, and multi-value behavior for arrays, pointers to arrays, slices, strings, maps, and constrained type parameters.
- **Observable content:** Each indexable category, full slices, constant and runtime bounds, map comma-ok lookup, UTF-8 string byte indexing, and generic index operations.
- **Variants and failures:** String indexing yields bytes and is not addressable. Map indexes are not addressable. Constant out-of-range indexes are compile errors; runtime violations panic.
- **Constraints:** Generic indexing requires compatible element and key types across all types in the type set.
- **Authority:** Language definition.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*.

#### GO-EXP-004 — Calls, conversions, and instantiations sharing syntax

- **Distinct behavior:** Parenthesized and bracketed expression forms are disambiguated by the resolved operand as calls, conversions, indexing, slicing, or generic instantiation.
- **Observable content:** A type conversion, ordinary call, call of a function value, generic instantiation and call, and expressions where parentheses are required to remove ambiguity.
- **Variants and failures:** A conversion has one argument except special string or slice cases, while calls follow function signatures. Generic type arguments can be partly inferred only for functions.
- **Constraints:** Parser and type checker must cooperate; language-version gates change which parses are meaningful.
- **Authority:** Language definition.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*.

#### GO-EXP-005 — Multiple assignment and tuple-producing expressions

- **Distinct behavior:** Assignment evaluates operands under ordered phases and can distribute a multi-valued call, map lookup, receive, or assertion result to several destinations.
- **Observable content:** Parallel swaps, assignments to indexed or dereferenced destinations with side effects, multiple-return calls, and each comma-ok form.
- **Variants and failures:** Assignment count and assignability are checked after tuple expansion. Earlier evaluation of destination operands can make results differ from sequential assignments.
- **Constraints:** Short declarations add the requirement that at least one non-blank name be new in the current block.
- **Authority:** Language definition.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*.

#### GO-EXP-006 — Evaluation order and unspecified ordering

- **Distinct behavior:** Go specifies lexical left-to-right evaluation for function calls, method calls, and communication operands in an expression, but leaves some relative evaluation and initialization details unspecified.
- **Observable content:** Side-effecting operands in calls, map assignments, composite literals, multiple assignment, and package initialization where all permitted outcomes are visible.
- **Variants and failures:** Dependence on unspecified ordering is valid but non-portable behavior. Constant evaluation and dependency-based initialization are separate from runtime operand order.
- **Constraints:** Compiler optimization must preserve specified effects but may choose among unspecified orders.
- **Authority:** Language definition.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*.

#### GO-EXP-007 — Short variable declarations

- **Distinct behavior:** `:=` can redeclare existing variables only when they were declared earlier in the same block, have the same type, and at least one non-blank variable is new.
- **Observable content:** A short declaration introducing all-new names, mixed new and same-block names, a shadowing declaration in an inner block, and invalid no-new-name cases.
- **Variants and failures:** Function parameters count as declarations in the function body's block. Multi-valued expressions can determine several types simultaneously.
- **Constraints:** Range and type-switch clauses have special implicit declaration behavior.
- **Authority:** Language definition.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*.

#### GO-EXP-008 — Named results and naked returns

- **Distinct behavior:** Result parameters are variables scoped to the function body and can be changed by deferred functions; a bare return returns their current values.
- **Observable content:** Functions with unnamed and named results, explicit and naked returns, shadowing near a return, and a defer that mutates a named result.
- **Variants and failures:** A naked return is invalid when a result name is shadowed at the return point in a way that makes meaning unclear. Multiple results form a tuple only in permitted contexts.
- **Constraints:** Exact compiler diagnostic wording is implementation-specific.
- **Authority:** Language definition.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*.

#### GO-EXP-009 — Function literals, closures, and captured variables

- **Distinct behavior:** Function literals create closures that capture variables by reference to storage rather than copying current values automatically.
- **Observable content:** Escaping and non-escaping closures, mutation shared by several closures, captures of parameters and named results, and closures created in loops.
- **Variants and failures:** Captured storage may move to the heap, but escape behavior is not semantic identity. Loop-variable capture changed materially in Go 1.22.
- **Constraints:** Lifetime and allocation are implementation decisions; observable capture semantics are language-defined.
- **Authority:** Language definition; allocation by compiler implementation.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*; Go 1.22 release notes.

#### GO-EXP-010 — Loop variable lifetime across language versions

- **Distinct behavior:** For modules declaring Go 1.22 or later, iteration variables declared by a `for` clause or range clause are distinct per iteration in the cases specified; older language versions reuse variables.
- **Observable content:** Closures and stored addresses created during classic and range loops, built once under pre-1.22 semantics and once under Go 1.22-or-later semantics.
- **Variants and failures:** Preexisting variables assigned with `=` remain reused. The `loopvar` experiment previewed the change before it became versioned language behavior.
- **Constraints:** The module or file language version, not only installed toolchain, selects semantics.
- **Authority:** Versioned language definition and official toolchain transition rules.
- **Confidence:** High.
- **Likely primary source:** Go 1.22 release notes; Go blog material on loop variables; *The Go Programming Language Specification*.

#### GO-CTL-001 — Basic `for` forms and range over core values

- **Distinct behavior:** Go has condition-only, three-clause, infinite, and range loops; range produces category-specific iteration values for arrays, slices, strings, maps, and channels.
- **Observable content:** Every loop form and every core range operand, with zero, one, and two iteration variables where permitted.
- **Variants and failures:** String range decodes UTF-8 and yields replacement runes for invalid encodings. Map order is unspecified. Channel range ends only on close. Nil channel range blocks forever.
- **Constraints:** Number and types of range variables depend on operand category and language version.
- **Authority:** Language definition.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*.

#### GO-CTL-002 — Range over integers

- **Distinct behavior:** Since Go 1.22, ranging over an integer expression iterates from zero to one less than its value, with iteration-variable typing determined by declaration context.
- **Observable content:** Untyped and typed integer operands, zero and negative values, declared versus preexisting iteration variables, and invalid non-integer operands.
- **Variants and failures:** A constant range value must be representable in the iteration variable's type. No iterations occur for non-positive values.
- **Constraints:** Requires language version Go 1.22 or later.
- **Authority:** Language definition.
- **Confidence:** High.
- **Likely primary source:** Go 1.22 release notes; *The Go Programming Language Specification*.

#### GO-CTL-003 — Range over iterator functions

- **Distinct behavior:** Since Go 1.23, range can invoke iterator functions with a synthesized yield callback and support zero, one, or two yielded values.
- **Observable content:** Iterator functions of all supported signatures, early loop exit, nested iteration, and an iterator that violates the yield protocol.
- **Variants and failures:** `break`, `return`, or panic must stop accepting yielded values; an iterator that calls yield again after it returned false triggers defined or implementation-enforced failure behavior. Iterator closures interact with capture and defer.
- **Constraints:** Requires language version Go 1.23 or later; standard iterator conventions may use the `iter` package but the syntax is language-defined.
- **Authority:** Language definition and official runtime/compiler implementation.
- **Confidence:** High.
- **Likely primary source:** Go 1.23 release notes; *The Go Programming Language Specification*; `iter` package documentation.

#### GO-CTL-004 — Expression and type switches

- **Distinct behavior:** Expression switches use equality and optional short initialization; type switches dispatch on an interface's dynamic type with case-sensitive variable typing.
- **Observable content:** Tagged and tagless switches, multiple expressions per case, default in different positions, fallthrough, and the full set of type-switch case forms.
- **Variants and failures:** Only expression switches allow `fallthrough`. Cases need not be compile-time constants, so duplicate runtime-equal cases may be permitted except where specifically prohibited. Duplicate type cases are invalid.
- **Constraints:** Comparison must be valid for the switch expression and case expressions.
- **Authority:** Language definition.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*.

#### GO-CTL-005 — `if` and switch initialization scopes

- **Distinct behavior:** `if` and switch statements can include a simple initialization statement whose declarations remain in scope across condition, cases, and associated branches under statement-specific rules.
- **Observable content:** Short declarations in `if`, expression switch, and type-switch headers, with branch-local shadowing and references after the statement that fail.
- **Variants and failures:** Type-switch guard variables have different static types in single-type versus multi-type cases. `else if` nesting creates related but distinct scopes.
- **Constraints:** Scope follows the language's implicit-block model.
- **Authority:** Language definition.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*.

#### GO-CTL-006 — `break`, `continue`, `goto`, and `fallthrough`

- **Distinct behavior:** Branch statements have enclosing-statement, label, and placement restrictions that create non-local control-flow edges.
- **Observable content:** Labeled and unlabeled loop exits, labeled switch or select breaks, loop continues, valid and invalid gotos, and legal fallthrough.
- **Variants and failures:** `continue` labels must denote loops. `fallthrough` is forbidden in type switches and final cases. `goto` cannot bypass declarations or cross function boundaries.
- **Constraints:** Static legality is language-defined; unreachable code is generally allowed except where another rule forbids it.
- **Authority:** Language definition.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*.

#### GO-CTL-007 — `defer` argument timing and stack order

- **Distinct behavior:** A defer statement evaluates the function value and arguments immediately, schedules the call for function exit, and executes deferred calls in last-in, first-out order.
- **Observable content:** Several defers with side-effecting arguments, a deferred method call, a closure observing later mutation, named-result mutation, and panic-driven unwinding.
- **Variants and failures:** A nil deferred function value panics when invoked at exit, not when deferred. Defers in loops accumulate until the surrounding function returns unless enclosed in another function.
- **Constraints:** Open-coded defer optimization must preserve semantics.
- **Authority:** Language definition; optimization by compiler.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*.

#### GO-CTL-008 — Panic and recover

- **Distinct behavior:** Panic begins stack unwinding, runs deferred calls, and can be stopped only by a qualifying direct recover call in a deferred function on the panicking goroutine.
- **Observable content:** Explicit panic, runtime panic, successful recover, indirect or wrong-goroutine recover that fails, re-panic, and nested deferred calls.
- **Variants and failures:** `panic(nil)` behavior changed in Go 1.21 to ensure a panicking call is distinguishable unless the compatibility setting requests older behavior. Recovered value type affects type assertions.
- **Constraints:** `GODEBUG=panicnil` can select compatibility behavior in relevant toolchains.
- **Authority:** Language definition plus official runtime compatibility behavior.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*; Go 1.21 release notes; runtime documentation for `GODEBUG`.

#### GO-EXP-011 — Built-in functions and context-sensitive typing

- **Distinct behavior:** Predeclared built-ins are not ordinary first-class functions; their admissible operands and result types depend on context, constantness, and type sets.
- **Observable content:** Representative uses of `append`, `cap`, `clear`, `close`, `complex`, `copy`, `delete`, `imag`, `len`, `make`, `max`, `min`, `new`, `panic`, `print`, `println`, `real`, and `recover`.
- **Variants and failures:** Some results are constants. Some built-ins accept type parameters only with compatible core types. `print` and `println` are implementation aids with unspecified formatting. Built-ins can be shadowed.
- **Constraints:** `clear`, `min`, and `max` require Go 1.21; `min` and `max` have ordered-type and NaN details.
- **Authority:** Language definition, with implementation-defined output for debugging built-ins.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*; Go 1.21 release notes.

#### GO-EXP-012 — String and byte/rune conversions

- **Distinct behavior:** Conversions among strings, byte slices, rune slices, and integers have special element-copy and Unicode semantics unlike general type conversion.
- **Observable content:** String-to-byte and string-to-rune conversions, reverse conversions, integer-to-string conversion, invalid UTF-8, and mutation proving whether storage aliases.
- **Variants and failures:** Integer-to-string yields the Unicode replacement character for invalid code points. Compiler optimizations may avoid allocation where no mutation is observable.
- **Constraints:** `unsafe.String` and `unsafe.Slice` follow different lifetime and aliasing contracts.
- **Authority:** Language definition; allocation optimization by implementation.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*; `unsafe` package documentation.

### Concurrency and the memory model

#### GO-CON-001 — Goroutine creation and lifetime

- **Distinct behavior:** A `go` statement evaluates a call's function and arguments in the current goroutine, then starts the call independently without return values to the caller.
- **Observable content:** Goroutines launched from functions, methods, closures, and interface calls, with argument side effects and synchronization for completion.
- **Variants and failures:** Program exit does not wait for other goroutines. Scheduling and start timing are unspecified. A panic in any unrecovered goroutine terminates the program after runtime handling.
- **Constraints:** Scheduler behavior depends on runtime, target, and `GOMAXPROCS`.
- **Authority:** Language definition and official runtime.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*; runtime package documentation.

#### GO-CON-002 — Channel states and communication

- **Distinct behavior:** Send, receive, and close behavior depends on whether a channel is nil, open, closed, buffered, or unbuffered; communication also establishes synchronization.
- **Observable content:** Buffered and unbuffered channels, nil channel operations in controlled contexts, close, drain, comma-ok receives, and deliberate send or close panics.
- **Variants and failures:** Receive from closed yields the zero value after buffered data drains. Closing nil or already closed channels panics. Nil sends and receives block forever.
- **Constraints:** Only a sender conventionally closes; the language enforces type direction but not ownership.
- **Authority:** Language definition and Go memory model.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*; *The Go Memory Model*.

#### GO-CON-003 — `select` readiness and nondeterminism

- **Distinct behavior:** A select evaluates channel operands on entry, chooses pseudo-randomly among ready communications, uses default only when none are ready, and blocks if it has no default and no ready case.
- **Observable content:** Send and receive cases, comma-ok receive, default, multiple simultaneously ready cases, nil-channel disabling, and empty select.
- **Variants and failures:** Side effects in channel and send expressions occur even for unchosen cases according to select evaluation rules. Choice among ready cases is intentionally nondeterministic.
- **Constraints:** Scheduler and randomization details are implementation-defined; selection semantics are language-defined.
- **Authority:** Language definition and runtime implementation.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*.

#### GO-CON-004 — Happens-before synchronization

- **Distinct behavior:** Goroutine creation, channel communication and close, mutexes, one-time initialization, atomics, and other documented operations create specific synchronized-before relationships.
- **Observable content:** Paired examples using channels, locks, `Once`, `WaitGroup`, and atomics to publish data, contrasted with an unsynchronized access pattern.
- **Variants and failures:** Mere goroutine completion, sleep, or observing a value does not necessarily synchronize. Buffered channels have counting-rule nuances.
- **Constraints:** Library synchronization contracts supplement the formal memory model.
- **Authority:** Go memory model and standard-library documentation.
- **Confidence:** High.
- **Likely primary source:** *The Go Memory Model*; `sync` and `sync/atomic` package documentation.

#### GO-CON-005 — Data races and race-enabled behavior

- **Distinct behavior:** Programs with data races have sharply limited guarantees, while the race detector instruments supported builds and may define the `race` build tag.
- **Observable content:** A deliberate race isolated from a synchronized equivalent, and source selected only in race-enabled builds.
- **Variants and failures:** Map races can cause runtime fatal errors; other races may produce variable results. Race detection support varies by operating system, architecture, cgo, and build mode.
- **Constraints:** `go test -race` or `go build -race` uses official instrumentation but is not part of the language definition.
- **Authority:** Go memory model and official toolchain.
- **Confidence:** High.
- **Likely primary source:** *The Go Memory Model*; official race detector documentation; `go help build`.

#### GO-CON-006 — Atomic operations and alignment

- **Distinct behavior:** `sync/atomic` operations provide sequentially consistent atomic access and synchronization; typed atomic wrappers carry no-copy and alignment expectations.
- **Observable content:** Atomic load/store, compare-and-swap, read-modify-write, pointer operations, and typed atomic values alongside invalid-copy or misalignment-sensitive patterns.
- **Variants and failures:** On 32-bit systems, 64-bit atomic operands have placement and alignment requirements for older APIs. Copying an atomic value after first use violates its contract.
- **Constraints:** Architecture support and alignment details are implementation and library specific.
- **Authority:** Standard-library contract and Go memory model.
- **Confidence:** High.
- **Likely primary source:** `sync/atomic` package documentation; *The Go Memory Model*.

### Modules, package loading, and dependency identity

#### GO-MOD-001 — Module path, package import path, and `go.mod`

- **Distinct behavior:** A module supplies the path prefix and dependency metadata used to map import paths to package directories and versions.
- **Observable content:** A root `go.mod`, packages at the module root and subdirectories, and imports whose paths derive from the module path.
- **Variants and failures:** A repository can contain several modules. A package directory is not necessarily a module. A mismatched module path can resolve to the wrong identity or fail fetching.
- **Constraints:** Module-aware behavior is the modern default; exact fallback to legacy `GOPATH` mode depends on toolchain version and environment.
- **Authority:** Official `go` command.
- **Confidence:** High.
- **Likely primary source:** *Go Modules Reference*; `go help modules`.

#### GO-MOD-002 — `go` and `toolchain` directives

- **Distinct behavior:** The `go` directive sets the module's language and module-graph semantics and, since Go 1.21, is a required minimum toolchain version; the `toolchain` directive can suggest a newer toolchain.
- **Observable content:** Modules or controlled variants with differing `go` lines, a `toolchain` line, and syntax accepted or rejected due to the declared version.
- **Variants and failures:** The active toolchain can be selected or downloaded according to `GOTOOLCHAIN`. Dependency modules can force a higher minimum `go` version.
- **Constraints:** `toolchain` and automatic switching arrived in Go 1.21 and behavior has evolved in later releases.
- **Authority:** Official `go` command and versioned language policy.
- **Confidence:** High.
- **Likely primary source:** *Go Toolchains* documentation; *Go Modules Reference*; Go 1.21 release notes.

#### GO-MOD-003 — Minimal version selection and module graph pruning

- **Distinct behavior:** The build list selects one version per module path using Minimal Version Selection, while graph pruning and lazy loading change which transitive requirements must be read.
- **Observable content:** Dependencies that request different versions of a shared module, direct and indirect requirements, and an import that reveals the selected version.
- **Variants and failures:** Go 1.17 changed `go.mod` requirements and pruning behavior. `go mod tidy` can add or remove indirect requirements based on all relevant package configurations.
- **Constraints:** Module `go` versions affect graph pruning. Version queries and proxy availability affect acquisition but not Minimal Version Selection's core rule.
- **Authority:** Official `go` command.
- **Confidence:** High.
- **Likely primary source:** *Go Modules Reference*; official module graph pruning documentation.

#### GO-MOD-004 — Semantic import versioning

- **Distinct behavior:** Module major versions v2 and above normally require a `/vN` suffix in the module and package import paths, making major versions distinct package identities.
- **Observable content:** Two major versions of a dependency imported into one package with aliases, plus an invalid or mismatched path/version case.
- **Variants and failures:** `gopkg.in` has a recognized special convention. `+incompatible` versions apply to older repositories without modern module layout.
- **Constraints:** Rules vary for module paths ending in `/vN`, module zip contents, and legacy modules.
- **Authority:** Official `go` command convention.
- **Confidence:** High.
- **Likely primary source:** *Go Modules Reference*; official module compatibility guidance.

#### GO-MOD-005 — `replace`, `exclude`, and `retract`

- **Distinct behavior:** `replace` changes source resolution without changing the imported module path identity, `exclude` prevents a version from the build list, and `retract` advertises unsuitable versions to version queries.
- **Observable content:** Version and local-directory replacements, excluded versions, retractions in a dependency's own `go.mod`, and resolution evidence.
- **Variants and failures:** A local replacement need not have a version but generally needs a matching module declaration. A replacement alone does not add a module to the graph. Retraction does not automatically prevent an explicitly required version from building.
- **Constraints:** Workspace `replace` directives can override or conflict with module directives.
- **Authority:** Official `go` command.
- **Confidence:** High.
- **Likely primary source:** *Go Modules Reference*.

#### GO-MOD-006 — Workspaces and `go.work`

- **Distinct behavior:** A workspace combines several main modules for local resolution and can supply workspace-level `go`, `toolchain`, and replacement directives.
- **Observable content:** At least two modules listed by `use`, with one importing the other, plus workspace-only and module-only build comparisons.
- **Variants and failures:** `GOWORK=off` disables workspace use. Conflicting replacements across workspace modules must be resolved. Workspace builds can differ from reproducible single-module builds.
- **Constraints:** Workspaces require Go 1.18 or later; directives available in `go.work` depend on toolchain version.
- **Authority:** Official `go` command.
- **Confidence:** High.
- **Likely primary source:** *Go Modules Reference*; `go help work`; Go 1.18 release notes.

#### GO-MOD-007 — Nested modules and module boundaries

- **Distinct behavior:** A nested directory containing its own `go.mod` belongs to a separate module and is excluded from parent module package patterns and normal parent package traversal.
- **Observable content:** A parent module and nested child module with distinct paths, dependencies, and similarly named packages.
- **Variants and failures:** A workspace may reunite them for local development without merging identities. Parent `./...` behavior does not imply traversal into the nested module.
- **Constraints:** Repository layout alone does not define import identity; each active module root does.
- **Authority:** Official `go` command.
- **Confidence:** High.
- **Likely primary source:** *Go Modules Reference*; `go help packages`.

#### GO-MOD-008 — `internal` package import restrictions

- **Distinct behavior:** A package beneath an `internal` directory may be imported only by code rooted within the parent tree specified by the `internal` rule.
- **Observable content:** A valid importing sibling under the permitted root and an invalid importer outside it, ideally across module boundaries.
- **Variants and failures:** The restriction is based on import-path or filesystem ancestry as resolved by the toolchain, not exported identifier visibility.
- **Constraints:** The `go` command enforces the convention during package loading.
- **Authority:** Official toolchain convention.
- **Confidence:** High.
- **Likely primary source:** `go help packages`; `cmd/go` documentation.

#### GO-MOD-009 — Vendoring

- **Distinct behavior:** Vendor mode resolves eligible dependency imports from a main module's `vendor` tree and validates metadata in `vendor/modules.txt`.
- **Observable content:** A consistent vendor directory, its metadata, a dependency source distinguishable from cache source, and a deliberately inconsistent variant.
- **Variants and failures:** Automatic vendor mode depends on the `go` version and existence of a vendor directory. `-mod=vendor`, `-mod=mod`, and workspace vendoring select different behavior.
- **Constraints:** Vendored package identity remains its canonical import path; packages within vendor trees have placement and nested-vendor rules.
- **Authority:** Official `go` command.
- **Confidence:** High.
- **Likely primary source:** *Go Modules Reference*; `go help mod vendor`; `go help build`.

#### GO-MOD-010 — Canonical module source and checksum state

- **Distinct behavior:** Module resolution can use proxies, direct version-control access, the checksum database, private-module patterns, and the module cache while preserving canonical module path and version identity.
- **Observable content:** A dependency represented in `go.sum`, an intentionally local or private-pattern substitute, and module-cache or vendor resolution metadata without relying on network access.
- **Variants and failures:** `GOPROXY`, `GONOSUMDB`, `GOPRIVATE`, `GONOPROXY`, `GOSUMDB`, and `GOVCS` change acquisition and verification. Missing checksums can be a diagnostic rather than a source-language failure.
- **Constraints:** Environment and network policy control availability; checksum behavior is not part of language semantics.
- **Authority:** Official `go` command and service protocol.
- **Confidence:** High.
- **Likely primary source:** *Go Modules Reference*; `go help environment`; official module proxy and checksum database documentation.

#### GO-MOD-011 — Import cycles and package graph validity

- **Distinct behavior:** The package import graph must be acyclic, including cycles induced through test variants; initialization order and buildability depend on that graph.
- **Observable content:** A valid directed acyclic package graph, a direct import cycle, an indirect cycle, and a test-only cycle involving internal test compilation.
- **Variants and failures:** External test packages can sometimes avoid a cycle that same-package tests introduce. Build constraints may expose a cycle only on one platform.
- **Constraints:** Cycle diagnostics are produced after active-file selection and import resolution.
- **Authority:** Language and official toolchain.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*; `go help packages`; `go help test`.

#### GO-MOD-012 — Legacy `GOPATH` and relative-import behavior

- **Distinct behavior:** Legacy package resolution can derive import identity from `GOPATH`, vendor ancestry, and filesystem layout rather than a module file; local relative imports are limited to non-module contexts.
- **Observable content:** A quarantined legacy-mode package set and equivalent module-aware failure or identity difference.
- **Variants and failures:** Modern releases have narrowed or removed automatic legacy fallbacks. Relative imports are rejected in module mode and are unsuitable for installed packages.
- **Constraints:** Exact support should be verified for the selected toolchain; this is compatibility coverage rather than baseline practice.
- **Authority:** Official `go` command legacy behavior.
- **Confidence:** Medium.
- **Likely primary source:** `go help gopath`; `go help modules`; release notes for the selected version.

### Build selection and non-Go inputs

#### GO-BLD-001 — Build constraint expressions

- **Distinct behavior:** A leading `//go:build` expression includes a file only when Boolean expressions over satisfied build tags evaluate true.
- **Observable content:** Files selected by conjunction, disjunction, negation, parentheses, and a caller-supplied tag, with mutually exclusive declarations across variants.
- **Variants and failures:** A misplaced or malformed directive is ignored or diagnosed. Multiple `//go:build` lines are invalid. Selected variants can cause duplicates, missing symbols, or package-name conflicts.
- **Constraints:** The directive must appear near the file start and be separated from the package clause as documented.
- **Authority:** Official `go` command plus language-defined comment syntax.
- **Confidence:** High.
- **Likely primary source:** `go help buildconstraint`.

#### GO-BLD-002 — Legacy `+build` constraints

- **Distinct behavior:** Older `// +build` lines express source inclusion with a different line-and-option Boolean syntax and may coexist with an equivalent `//go:build` line for older tools.
- **Observable content:** Correctly paired modern and legacy constraints and a deliberately mismatched or malformed pair.
- **Variants and failures:** `gofmt` can add equivalent legacy lines in some toolchain eras. Modern tools diagnose disagreement. Very old tools ignore `//go:build` but understand `+build`.
- **Constraints:** Support and automatic formatting are toolchain-version-sensitive.
- **Authority:** Official `go` command compatibility behavior.
- **Confidence:** High.
- **Likely primary source:** `go help buildconstraint`; release notes introducing `//go:build`.

#### GO-BLD-003 — File-name platform constraints

- **Distinct behavior:** Recognized suffixes before the extension implicitly constrain a file by operating system, architecture, or their combination.
- **Observable content:** Generic, OS-specific, architecture-specific, and combined-suffix files that supply alternate implementations.
- **Variants and failures:** Only exact recognized suffix patterns apply; arbitrary underscore suffixes do not. Filename constraints combine with explicit build directives.
- **Constraints:** The known `GOOS` and `GOARCH` sets are toolchain-defined and expand over time.
- **Authority:** Official `go` command.
- **Confidence:** High.
- **Likely primary source:** `go help buildconstraint`; `go tool dist list` documentation.

#### GO-BLD-004 — Implicit toolchain and release tags

- **Distinct behavior:** Build constraints can test target OS, architecture, compiler, cgo availability, release tags such as `go1.N`, and selected feature tags.
- **Observable content:** Files gated on `gc`, `gccgo`, `cgo`, at least two release tags, and a user tag supplied through `-tags`.
- **Variants and failures:** Release tags are cumulative through the installed release, not necessarily the package language version. User tags can accidentally select conflicting implementations.
- **Constraints:** Tags such as `race`, `msan`, `asan`, and platform aliases are enabled only by documented build modes or target rules.
- **Authority:** Official `go` command and toolchain implementations.
- **Confidence:** High.
- **Likely primary source:** `go help buildconstraint`; `go help build`.

#### GO-BLD-005 — Architecture feature tags and environment

- **Distinct behavior:** Architecture tuning variables such as `GOAMD64`, `GOARM`, `GOARM64`, `GOMIPS`, `GOPPC64`, `GORISCV64`, and `GOWASM` can define feature tags and alter generated code.
- **Observable content:** A package with a portable implementation and feature-tagged alternatives for a supported architecture level.
- **Variants and failures:** Feature tags can imply lower levels. The available variables, allowed values, and tags differ by architecture and toolchain release.
- **Constraints:** Cross-compilation may build successfully yet be impossible to run on the host; cgo cross-builds need a C cross-compiler.
- **Authority:** Official `go` command and compiler.
- **Confidence:** Medium.
- **Likely primary source:** `go help buildconstraint`; `go help environment`; architecture-specific Go documentation.

#### GO-BLD-006 — Ignored and specially named files

- **Distinct behavior:** Files beginning with `_` or `.`, files with unsupported extensions, and files excluded by constraints do not contribute ordinary Go declarations; `_test.go` files contribute only during test builds.
- **Observable content:** Each ignored-name form, a constrained-out file, an ordinary source file, and a test-only file containing otherwise colliding declarations.
- **Variants and failures:** A directory can report no buildable Go files in one configuration. Editor or generic filesystem enumeration may see files the `go` command excludes.
- **Constraints:** Non-Go files of recognized kinds can still participate through cgo, assembly, embedding, or linking rules.
- **Authority:** Official `go` command.
- **Confidence:** High.
- **Likely primary source:** `go help packages`; `go help buildconstraint`; `go help test`.

#### GO-BLD-007 — Per-file language version from release constraints

- **Distinct behavior:** In modern toolchains, a build constraint mentioning a Go release can raise the language version used for that file within the module's allowed context.
- **Observable content:** Two selected files in one package whose permitted syntax differs because one has a qualifying `go1.N` constraint.
- **Variants and failures:** Negated or complex version terms affect the inferred file version differently. Toolchain release still bounds features available to compile.
- **Constraints:** This behavior was introduced alongside stricter version handling in Go 1.21 and should be verified for exact releases.
- **Authority:** Official compiler and `go` command.
- **Confidence:** Medium.
- **Likely primary source:** `go help buildconstraint`; Go 1.21 release notes.

#### GO-BLD-008 — Generated-source convention and `go generate`

- **Distinct behavior:** Generated `.go` files are ordinary selected source, while the standardized generated-code marker labels provenance and `go generate` runs explicit source directives outside normal build dependency analysis.
- **Observable content:** A checked-in generated Go file with the canonical marker and a source file containing generation directives that name a local generator.
- **Variants and failures:** `go build` does not run generators. Directive commands, environment expansion, ordering, and tool availability can change generated output. A malformed marker loses conventional generated status without excluding the file.
- **Constraints:** `go generate` is intentionally not a general build system and provides no automatic dependency tracking.
- **Authority:** Official `go` command convention.
- **Confidence:** High.
- **Likely primary source:** `go help generate`; official generated-code convention documentation.

#### GO-BLD-009 — Embedded files with `//go:embed`

- **Distinct behavior:** Embed directives associate filesystem files with a package variable of type string, byte slice, or `embed.FS`, creating source-to-resource relations at build time.
- **Observable content:** Single-file and pattern embeds into each supported variable type, an imported `embed` package, hidden-file behavior, and an invalid unmatched pattern.
- **Variants and failures:** `all:` changes hidden-name matching. Patterns cannot escape the package module's permitted tree or match invalid paths. The blank import form can make the directive legal without a selector use.
- **Constraints:** Requires Go 1.16 or later. Directive placement and variable declaration shape are strict.
- **Authority:** Official toolchain and standard-library contract.
- **Confidence:** High.
- **Likely primary source:** `embed` package documentation.

#### GO-BLD-010 — Cgo preamble and pseudo-package `C`

- **Distinct behavior:** Importing pseudo-package `C` causes cgo to derive Go-visible names and generated bridge code from the immediately preceding C preamble and auxiliary native files.
- **Observable content:** A cgo Go file referencing a C type, variable, and function, plus local C source and platform-specific linker or compiler flags.
- **Variants and failures:** The import cannot be used like an ordinary package. `CGO_ENABLED=0` excludes cgo files. C unions, bitfields, macros, callbacks, multiple return values with `errno`, and pointer passing each have special mappings.
- **Constraints:** Requires a working C toolchain and target-compatible compiler; cross-compilation is configuration-sensitive.
- **Authority:** Official cgo implementation and documented Go/C interoperability contract.
- **Confidence:** High.
- **Likely primary source:** cgo command documentation.

#### GO-BLD-011 — Cgo pointer and lifetime rules

- **Distinct behavior:** Passing pointers between Go and C is constrained by whether referenced memory contains Go pointers, whether C retains it, pinning, and runtime pointer checks.
- **Observable content:** Valid transient pointer passing, pinned memory where supported, C-allocated memory, callback handles, and isolated invalid retention or nested-pointer examples.
- **Variants and failures:** `runtime/cgo.Handle`, `runtime.Pinner`, finalizers, and `cgocheck` settings offer distinct ownership paths. Violations may panic only at runtime.
- **Constraints:** Rules and dynamic checks have evolved; exact behavior depends on Go version and `GOEXPERIMENT=cgocheck2` or related settings.
- **Authority:** Official cgo and runtime implementation.
- **Confidence:** Medium.
- **Likely primary source:** cgo command documentation; `runtime/cgo` and `runtime` package documentation.

#### GO-BLD-012 — Go assembly and bodyless declarations

- **Distinct behavior:** A Go function declaration without a body can be implemented in a package's assembly file, with symbols and calling convention connected by the assembler and linker rather than Go syntax.
- **Observable content:** A bodyless Go declaration, matching architecture-specific assembly implementation, portable fallback, and an intentionally mismatched signature or absent symbol variant.
- **Variants and failures:** Assembly syntax, pseudo-registers, frame declarations, pointer maps, and application binary interface wrappers are architecture- and compiler-specific. Compiler intrinsics may replace some declarations.
- **Constraints:** Go assembly is not defined by the language and differs from platform-native assembler syntax.
- **Authority:** Official assembler, compiler, and linker implementation.
- **Confidence:** Medium.
- **Likely primary source:** *A Quick Guide to Go's Assembler*; compiler application binary interface documentation.

#### GO-BLD-013 — Compiler and linker directives

- **Distinct behavior:** `//go:` directives such as `noinline`, `nosplit`, `noescape`, `linkname`, `wasmimport`, or `wasmexport` can alter compilation, calling, visibility, or symbol linkage outside ordinary language semantics.
- **Observable content:** Only directives supported by the selected toolchain, attached to valid declarations, with ordinary counterparts and a malformed or unauthorized case.
- **Variants and failures:** `linkname` can connect otherwise unrelated or unexported symbols and requires `unsafe`; newer toolchains restrict unsafe external linkname use. Directive sets and acceptance change by release and target.
- **Constraints:** Many directives are reserved for runtime or low-level packages and are unsupported application contracts.
- **Authority:** Official implementation, not the Go language unless separately documented.
- **Confidence:** Medium.
- **Likely primary source:** Compiler command documentation; `unsafe` package documentation; relevant Go release notes.

#### GO-BLD-014 — Build modes, linking, and plugin packages

- **Distinct behavior:** Build modes can produce executables, archives, shared libraries, C archives or shared objects, plugins, or position-independent packages, changing required entry points and link-time reachability.
- **Observable content:** Package sets valid under ordinary executable, archive, and one supported alternate build mode, with link metadata captured separately.
- **Variants and failures:** `plugin` dynamically resolves exported symbols by string and requires closely matching toolchain, dependency, and build settings. C-shared and C-archive modes use cgo export directives and platform-specific naming.
- **Constraints:** Mode availability varies by `GOOS`, `GOARCH`, compiler, cgo, and external linker.
- **Authority:** Official toolchain implementation and standard `plugin` package.
- **Confidence:** Medium.
- **Likely primary source:** `go help buildmode`; `plugin` package documentation; cgo command documentation.

#### GO-BLD-015 — Instrumented build variants

- **Distinct behavior:** Race, memory-sanitizer, address-sanitizer, coverage, fuzz, and profiling builds can insert generated code, synthetic functions, extra dependencies, or implicit build tags absent from normal source builds.
- **Observable content:** Normal and supported instrumented builds of the same package, plus tag-gated source for a documented instrumentation tag.
- **Variants and failures:** Support matrices differ by platform and require cgo or external tools in some modes. Coverage can be package-scoped or include selected dependent packages.
- **Constraints:** Instrumentation is official toolchain behavior, not language semantics.
- **Authority:** Official `go` command, compiler, and runtime.
- **Confidence:** High.
- **Likely primary source:** `go help build`; `go help testflag`; official race detector and coverage documentation.

#### GO-BLD-016 — Build cache, trimming, and source identity

- **Distinct behavior:** The build cache reuses actions keyed by inputs and configuration, while flags such as `-trimpath` alter recorded source paths and debug metadata without changing package import identity.
- **Observable content:** Equivalent builds with and without trimmed paths and with a changed build tag or generated input that invalidates the relevant action.
- **Variants and failures:** `-buildvcs` can embed version-control metadata. Reproducibility can differ with cgo, external linkers, timestamps, or generators.
- **Constraints:** Cache key and object metadata details are implementation-specific and change between toolchain releases.
- **Authority:** Official `go` command implementation.
- **Confidence:** Medium.
- **Likely primary source:** `go help build`; `go help cache`; `runtime/debug` package documentation for build information.

### Testing, examples, fuzzing, and tooling-only source

#### GO-TST-001 — Same-package and external test packages

- **Distinct behavior:** `_test.go` files may compile into the package under test or into a separate package conventionally named with `_test`, producing distinct visibility and import relationships.
- **Observable content:** Both internal and external test files for one package, with the internal test using unexported names and the external test using only the public API.
- **Variants and failures:** The test command may compile a package variant augmented with internal test files, then link external tests against that variant. Test-only imports can introduce cycles or dependencies absent from normal builds.
- **Constraints:** Test files are excluded from ordinary `go build` package source.
- **Authority:** Official `go` command convention.
- **Confidence:** High.
- **Likely primary source:** `go help test`; `go help packages`.

#### GO-TST-002 — Test binary synthesis and discovery conventions

- **Distinct behavior:** `go test` discovers correctly named test, benchmark, fuzz, and example functions and synthesizes a test main that registers and invokes them.
- **Observable content:** Valid discovered functions, similarly named but undiscovered functions, optional `TestMain`, and package initialization visible before test execution.
- **Variants and failures:** Signature and naming rules differ by function kind. `TestMain` can control exit and setup. Generated test-main identities do not exist as source declarations.
- **Constraints:** Testing APIs and discovery are standard-library and `go` command conventions, not core language.
- **Authority:** Official `go` command and `testing` package.
- **Confidence:** High.
- **Likely primary source:** `testing` package documentation; `go help test`.

#### GO-TST-003 — Subtests, sub-benchmarks, and parallel execution

- **Distinct behavior:** Tests and benchmarks can register nested functions dynamically, use slash-separated names, and opt into coordinated parallel execution.
- **Observable content:** Nested `Run` calls, duplicate subtest names, cleanup callbacks, parallel children, and state captured from an enclosing loop.
- **Variants and failures:** Names are sanitized and made unique. Parallel tests pause and resume under the test scheduler. Cleanup order and scope differ from ordinary defer.
- **Constraints:** Loop capture behavior depends on the package language version; testing behavior depends on standard-library version.
- **Authority:** `testing` package convention and implementation.
- **Confidence:** High.
- **Likely primary source:** `testing` package documentation.

#### GO-TST-004 — Executable examples

- **Distinct behavior:** Example functions can be associated by naming with a package, type, function, or method; output comments can make them executable tests and unordered output changes comparison rules.
- **Observable content:** Package-, type-, function-, and method-associated examples, one with ordered output, one with unordered output, and one documentation-only example.
- **Variants and failures:** Naming must map to an existing exported declaration except recognized suffixes. Output matching, not return values, determines success.
- **Constraints:** Discovery and association are `go test` and documentation-tool conventions.
- **Authority:** Official `testing` package and documentation tooling.
- **Confidence:** High.
- **Likely primary source:** `testing` package documentation; `go doc` documentation.

#### GO-TST-005 — Fuzz targets and seed corpus

- **Distinct behavior:** Fuzz functions register a typed fuzz target, consume seed inputs from code and `testdata/fuzz`, and run under generated-input instrumentation distinct from ordinary tests.
- **Observable content:** A valid fuzz function, multiple supported seed types, seed corpus files, a discovered failing input, and invalid target signatures in isolated failures.
- **Variants and failures:** Fuzzing and ordinary seed execution have different runtime modes. Only documented primitive argument types are supported. Failing inputs may be written back to the corpus cache or package directory.
- **Constraints:** Native fuzzing requires Go 1.18 or later; supported platforms and instrumentation evolve.
- **Authority:** Official `go` command and `testing` package.
- **Confidence:** High.
- **Likely primary source:** `testing` package documentation; official fuzzing documentation; Go 1.18 release notes.

#### GO-TST-006 — `testdata` and package-relative data

- **Distinct behavior:** The `go` command ignores directories named `testdata` when expanding package patterns, while tests and generators can access their contents by package-relative filesystem paths.
- **Observable content:** Test fixtures under `testdata`, a nested Go-looking file that is not a package input, and tests or fuzzing that read selected data.
- **Variants and failures:** Runtime working-directory assumptions are tool behavior, not language semantics. Embed patterns and fuzz corpus conventions interact differently with `testdata`.
- **Constraints:** Filesystem access can be affected by sandboxing, cross-compilation, or test execution environment.
- **Authority:** Official `go` command convention.
- **Confidence:** High.
- **Likely primary source:** `go help packages`; `go help test`; `testing` package documentation.

#### GO-TST-007 — Tool dependencies and build-tag conventions

- **Distinct behavior:** Projects commonly retain module requirements for generators or linters through a source file excluded from normal builds, often using a dedicated build tag and blank imports.
- **Observable content:** A tooling-only package or file with blank imports, corresponding module requirements, and proof that normal builds exclude it while dependency maintenance includes it when requested.
- **Variants and failures:** `go mod tidy` considers a broad set of build tags but treats some tags specially; emerging tool directives in later Go versions may replace or supplement the convention.
- **Constraints:** This pattern is ecosystem convention and exact module-command treatment changes by release.
- **Authority:** Ecosystem convention plus official module loader behavior.
- **Confidence:** Medium.
- **Likely primary source:** *Go Modules Reference*; `go help mod tidy`; release notes for any module tool directive in the selected version.

### Dynamic, reflective, and ecosystem-mediated relationships

#### GO-DYN-001 — Interface dispatch at call sites

- **Distinct behavior:** A method call through an interface resolves statically to an interface method but dispatches at runtime to the dynamic concrete type's implementation.
- **Observable content:** Several concrete implementations, value and pointer implementers, assignments through wider and narrower interfaces, and calls whose possible targets differ by control flow.
- **Variants and failures:** Type assertions and switches can narrow possible targets. Nil interfaces and typed nil receivers create distinct failure paths. Generic constraints can use interface methods without creating runtime interface values.
- **Constraints:** Closed-world target enumeration is not guaranteed by the language, especially across plugins or reflection.
- **Authority:** Language definition.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*.

#### GO-DYN-002 — Reflection-based type and member access

- **Distinct behavior:** `reflect` exposes runtime types, values, fields, methods, tags, dynamic calls, and dynamic construction that may not have direct source selectors or calls.
- **Observable content:** Type and value acquisition, field and method lookup by string, dynamic call, dynamic type construction, addressability and settability checks, and panic-producing misuse.
- **Variants and failures:** Exported and unexported members have different interface and set permissions. Method availability can be affected by linker reachability rules. `reflect.TypeFor` offers a generic type path in newer releases.
- **Constraints:** Reflection APIs evolve with the standard library; representation details remain implementation-defined.
- **Authority:** Standard-library contract and runtime implementation.
- **Confidence:** High.
- **Likely primary source:** `reflect` package documentation.

#### GO-DYN-003 — Struct-tag-driven behavior

- **Distinct behavior:** Struct tags are opaque language-level strings interpreted by packages to rename, omit, validate, serialize, map, or inject fields without ordinary call edges at each field.
- **Observable content:** One struct with distinct tags consumed by at least two conventional packages, embedded fields, conflicting tag names, and malformed tag syntax observable through reflection.
- **Variants and failures:** Each consumer defines its own grammar, field promotion, case, conflict, and unexported-field rules. `go vet` may diagnose malformed conventional tags that the compiler accepts.
- **Constraints:** Tag strings affect struct type identity, while their meanings are ecosystem-specific.
- **Authority:** Language for tag presence and reflection; package convention for semantics.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*; `reflect.StructTag` documentation; documentation of each consuming package.

#### GO-DYN-004 — Registration through `init` and global state

- **Distinct behavior:** Packages often register implementations by mutating global registries during initialization, so blank imports or transitive imports can create runtime dispatch relationships without direct calls.
- **Observable content:** A registry package, several provider packages registering in `init`, a consumer selecting by key, and one provider activated only by blank import or build tag.
- **Variants and failures:** Duplicate keys, initialization order, hidden dynamic keys, and omitted side-effect imports change targets. Registration may occur through explicit calls instead of `init`.
- **Constraints:** This is ecosystem architecture built on language-defined initialization.
- **Authority:** Language for initialization; ecosystem convention for registry semantics.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*; documentation of the chosen registry pattern's standard package if any.

#### GO-DYN-005 — Plugin symbol lookup

- **Distinct behavior:** The `plugin` package loads a separately built module at runtime and looks up exported variables or functions by string, creating package and symbol relationships unavailable during ordinary static linking.
- **Observable content:** A supported host and plugin pair, successful function and variable lookup, failed lookup, and failed type assertion for an unexpected symbol type.
- **Variants and failures:** Plugin initialization runs once on first open. Build settings or dependency versions that do not match closely can prevent loading. Platform support is limited.
- **Constraints:** Official `gc` implementation feature, not portable language behavior; race detection has additional limitations.
- **Authority:** Standard-library and linker implementation.
- **Confidence:** High.
- **Likely primary source:** `plugin` package documentation; `go help buildmode`.

#### GO-DYN-006 — Runtime callbacks, finalizers, and cleanup hooks

- **Distinct behavior:** Runtime and library APIs can retain and later invoke function values supplied as finalizers, cleanups, signal handlers, timers, or callbacks, with no direct syntactic call at the invocation point.
- **Observable content:** At least one lifecycle callback with explicit reachability control and one deterministic library callback, separated from nondeterministic finalizer behavior.
- **Variants and failures:** Finalizer timing and execution are not guaranteed, object reachability can be extended with `KeepAlive`, and newer releases may offer cleanup APIs with different guarantees.
- **Constraints:** Runtime lifecycle APIs are version-sensitive and unsuitable for asserting deterministic execution.
- **Authority:** Standard-library and runtime contract.
- **Confidence:** Medium.
- **Likely primary source:** `runtime` package documentation; documentation of the selected callback API.

#### GO-DYN-007 — `unsafe` types and operations

- **Distinct behavior:** `unsafe.Pointer`, `uintptr`, layout queries, offset arithmetic, and newer slice/string helpers permit relationships that bypass ordinary type identity, pointer, bounds, and lifetime checks.
- **Observable content:** Size, alignment, and offset queries; valid pointer reinterpretation; pointer arithmetic within one object; slice and string construction; and quarantined invalid lifetime or bounds cases.
- **Variants and failures:** Converting a pointer through stored `uintptr` can lose garbage-collector tracking. Layout changes by architecture. `checkptr` instrumentation can detect some invalid uses.
- **Constraints:** The package explicitly falls outside Go 1 compatibility guarantees except as documented; operations depend on `gc`, architecture, cgo, and instrumentation.
- **Authority:** Official `unsafe` package and implementation.
- **Confidence:** High.
- **Likely primary source:** `unsafe` package documentation; compiler documentation for `checkptr`.

#### GO-DYN-008 — Serialization and generated protocol bindings

- **Distinct behavior:** Common generators create Go types, methods, descriptors, registration code, and source references from non-Go schemas; runtime serialization may use both generated interfaces and reflection.
- **Observable content:** A small checked-in generated binding and its source schema, generator provenance marker, descriptors, and consumer code using both generated and reflective entry points.
- **Variants and failures:** Generator editions and versions can produce materially different application programming interfaces. Hand editing generated output is unstable. Build success can depend on generated files already existing.
- **Constraints:** Ecosystem-specific; inclusion is warranted only when such generated bindings are part of the project under evaluation.
- **Authority:** Generator and ecosystem convention, with generated Go governed normally by the language.
- **Confidence:** Medium.
- **Likely primary source:** Official documentation for the selected schema compiler and Go runtime library.

#### GO-DYN-009 — Contextual cancellation and error wrapping conventions

- **Distinct behavior:** `context.Context` and wrapped errors carry dynamic control and causal relationships through ordinary interfaces and conventions rather than dedicated language constructs.
- **Observable content:** Context propagation with cancellation and deadline callbacks, sentinel and typed errors, wrapping, unwrapping, `errors.Is`, `errors.As`, and multi-error joins.
- **Variants and failures:** Custom `Is`, `As`, or `Unwrap` methods alter traversal and matching. Cancellation causes are version-dependent library features. String equality is not error identity.
- **Constraints:** These are standard-library contracts widely used to express call and failure semantics.
- **Authority:** Standard-library convention.
- **Confidence:** High.
- **Likely primary source:** `context` and `errors` package documentation.

### Invalid, partial, and ambiguous project states

#### GO-DIA-001 — Parse and lexical failures with partial declarations

- **Distinct behavior:** A source file can contain recoverable syntax errors after which parsers expose partial abstract syntax and declarations, while the compiler rejects the package.
- **Observable content:** Isolated malformed files covering an unterminated literal or comment, missing delimiter, illegal token, and semicolon-related parse failure after valid declarations.
- **Variants and failures:** Different parsers recover at different points and synthesize nodes or positions differently. A build constraint can hide the same invalid file from one configuration.
- **Constraints:** Error recovery is tool-specific; lexical and syntactic validity are language-defined.
- **Authority:** Language definition for validity, implementation for recovery and diagnostics.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*; `go/parser` and `go/scanner` package documentation.

#### GO-DIA-002 — Name-resolution and declaration failures

- **Distinct behavior:** Undefined names, duplicate declarations, invalid shadow assumptions, unused locals or imports, and package-name conflicts can leave a partially resolvable package graph.
- **Observable content:** One isolated case of each failure, including valid declarations and references before and after the error.
- **Variants and failures:** Dot imports and embedding can create ambiguity rather than a direct duplicate. Duplicate `init` functions are the important allowed top-level spelling exception.
- **Constraints:** Active-file selection determines which collisions exist.
- **Authority:** Language definition; diagnostics by compiler and package loader.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*; `go/types` package documentation.

#### GO-DIA-003 — Type-checking failures

- **Distinct behavior:** Invalid assignments, conversions, calls, operations, assertions, receiver declarations, interface implementations, and generic instantiations can coexist with valid typed regions.
- **Observable content:** Small isolated failures for each major type-relation family with neighboring valid counterexamples.
- **Variants and failures:** Untyped constants, aliases, method sets, channel direction, and constraints often turn superficially similar cases into opposite outcomes. Compiler and `go/types` error text may differ.
- **Constraints:** The declared language version must be supplied consistently to any type checker.
- **Authority:** Language definition; diagnostics by implementation.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*; `go/types` package documentation.

#### GO-DIA-004 — Load failures from configuration and dependencies

- **Distinct behavior:** A package can fail before parsing because no files match, multiple package names are selected, an import is unavailable or forbidden, checksums are missing, module metadata is inconsistent, or native tools are absent.
- **Observable content:** Separate configurations causing each class of load failure while retaining enough source and metadata to identify intended packages.
- **Variants and failures:** `go list -e` can return partial package data with errors. Workspace, vendor, proxy, cgo, and build-tag settings change which error appears first.
- **Constraints:** Failure reporting is official tool behavior and may vary by release.
- **Authority:** Official `go` command.
- **Confidence:** High.
- **Likely primary source:** `go help list`; `go help packages`; *Go Modules Reference*.

#### GO-DIA-005 — Runtime panics versus compile-time errors

- **Distinct behavior:** Many invalid runtime states are legal to compile and fail only when executed, including nil dereference, bounds failure, failed assertion, closed-channel operation, map misuse, divide by zero with nonconstant operands, and explicit panic.
- **Observable content:** Isolated, recoverable functions for each panic family and paired compile-time-constant cases where the language rejects the operation earlier.
- **Variants and failures:** Panic value and message types can be implementation-specific. Fatal runtime failures such as concurrent map corruption or stack exhaustion may not be recoverable.
- **Constraints:** Compiler optimization cannot remove required observable panics, but exact diagnostics and stack traces vary.
- **Authority:** Language definition for most failure conditions; runtime implementation for presentation and fatal cases.
- **Confidence:** High.
- **Likely primary source:** *The Go Programming Language Specification*; runtime package documentation.

#### GO-DIA-006 — Static-analysis diagnostics beyond compilation

- **Distinct behavior:** `go vet` and analyzers report suspicious but often legal constructs such as formatting mismatches, copied locks, unreachable patterns, malformed tags, lost cancellation, and version-specific loop closures.
- **Observable content:** Valid source with representative vet findings, explicit analyzer configuration, and nearby counterexamples that should remain quiet.
- **Variants and failures:** The analyzer set used by `go test` is curated and differs from invoking all vet analyzers. Findings and defaults change by toolchain version.
- **Constraints:** Analyzer diagnostics are not language validity and should not be merged with compiler errors.
- **Authority:** Official tool convention and individual analyzer contracts.
- **Confidence:** High.
- **Likely primary source:** `go help vet`; `go vet help`; `golang.org/x/tools/go/analysis` package documentation.

## Recent version changes requiring explicit coverage

| Version | Material support change | Coverage consequence |
| --- | --- | --- |
| Go 1.17 | Module graph pruning and lazy module loading became the normal shape for modules declaring Go 1.17 or later. | Preserve an older module-graph variant or at least metadata that distinguishes pruned from unpruned requirements. |
| Go 1.18 | Type parameters, constraints, instantiation, fuzzing, workspaces, and core generics-aware library and tool behavior arrived. | Treat pre-generics and generics-aware parsing, typing, loading, and tests as distinct. |
| Go 1.19 | The memory model was revised around atomics and synchronization, with typed atomic APIs in the library. | Verify concurrency expectations against the current memory-model document, not older folklore. |
| Go 1.20 | Constraint satisfaction for `comparable` changed, and multiple standard APIs changed reflective or error-mediated behavior. | Include an interface that newly satisfies `comparable` while retaining a possible runtime comparison panic. |
| Go 1.21 | The `go` line became a strict minimum, toolchain selection was added, type inference expanded, `clear`, `min`, and `max` became predeclared, and `panic(nil)` compatibility changed. | Run a language-version boundary variant and record toolchain/environment selection separately. |
| Go 1.22 | Each iteration can have fresh loop variables, integer range was added, and inference expanded again. | Compile closure/address and integer-range cases under both sides of the language-version boundary. |
| Go 1.23 | Range-over-function iterators became language behavior; generic aliases were available as an experiment with limitations. | Exercise iterator protocol and label generic-alias evidence as experimental unless the selected later release stabilizes it. |
| Later than Go 1.23 | Generic alias support and tool/module directives are known areas of active change. | Verify exact syntax, cross-package support, directives, and defaults in the installed release notes before claiming coverage. |

## Implementation-defined, unspecified, and environment-dependent behavior

The following should be represented as configurable or nondeterministic facts rather than a single portable outcome:

- Width of `int`, `uint`, and `uintptr`; size and alignment of implementation-dependent types; struct padding; native endianness exposed through low-level code.
- Package-file presentation order for otherwise independent initialization, despite the compiler recommendation to use lexical file-name order.
- Relative evaluation where the specification leaves order open, map iteration order, select choice among ready cases, goroutine scheduling, and timing of finalizers.
- Allocation location, escape analysis, stack growth, inlining, devirtualization, monomorphization or shape sharing for generics, append capacity growth, and linker dead-code elimination.
- Exact panic values, fatal runtime messages, stack traces, synthetic wrapper names, application binary interface symbols, object files, and debug/source-path metadata.
- `unsafe`, cgo, assembly, compiler directives, external linking, plugins, sanitizers, and cross-compilation support.
- Active tags, `GOOS`, `GOARCH`, architecture feature level, compiler (`gc` or `gccgo`), `CGO_ENABLED`, `GOEXPERIMENT`, build mode, test mode, workspace selection, vendor mode, and toolchain selection.
- Module proxy, checksum, private-module, version-control, cache, and network policy; these can change resolvability without changing source text.

## Known uncertainty and verification priorities

- **Generic aliases after Go 1.23:** Verify the exact release that stabilizes them, cross-package support, receiver restrictions, and interaction with reflection and export data. Confidence: medium.
- **Per-file language-version rules:** Verify how complex positive and negative release-tag expressions set the file version in the target toolchain. Confidence: medium.
- **Architecture feature tags:** Verify the complete current tag and environment matrix. New targets and feature levels are added over time. Confidence: medium.
- **Compiler directives and linkname restrictions:** Verify only documented, accepted directives for the chosen toolchain. Many are internal contracts. Confidence: medium.
- **Generic recursive declarations and operations over mixed type sets:** Compiler fixes and specification clarifications can move edge cases between rejection and acceptance. Confidence: medium.
- **Post-1.23 module and tooling directives:** Check release notes and `go help` from the exact installed toolchain before adding later behavior. Confidence: low without that check.

## Final completeness audit

Coverage represented in this report includes:

- lexical and scope identity; package/file boundaries; imports, export, aliases, and initialization;
- constants; all core composite type families; named-type identity; assignment, conversion, comparability, addressability, and nil;
- receiver and method sets; embedding and promotion; interface implementation and dispatch; generic declarations, constraints, inference, instantiation, and version-sensitive aliases;
- expressions, built-ins, closures, control flow, defer, panic, range forms, goroutines, channels, synchronization, races, and atomics;
- modules, versions, workspaces, nested modules, vendoring, internal packages, source selection, platform variants, generators, embedding, cgo, assembly, directives, linking, and instrumentation;
- internal and external tests, generated test mains, examples, subtests, fuzzing, data files, dynamic registration, reflection, tags, plugins, unsafe behavior, and partial or invalid projects.

Feature families that may still need project-specific additions are:

- **Target-specific low-level facilities:** operating-system syscalls, WebAssembly imports and exports, mobile bindings, or compiler intrinsics, when the project actually targets those environments.
- **Ecosystem-specific generated and declarative links:** database query generators, remote procedure call schemas, dependency-injection generators, template compilation, or framework routing, when those artifacts exist.
- **Newer release behavior:** language, module, telemetry, tool dependency, testing, and runtime changes after Go 1.23.
- **Alternate implementation behavior:** `gccgo`, TinyGo, or other Go-like toolchains, if portability beyond the official `gc` implementation is in scope.

No other major core-language feature family is knowingly absent. The highest residual risk is release drift in post-Go-1.23 toolchain behavior, not missing baseline Go semantics.
