# Python semantic coverage candidate checklist

## Scope and baseline

This report identifies candidate behaviors for a broad, idiomatic, self-contained Python project intended to expose semantically distinct entities, identities, relationships, resolution rules, types, visibility, dispatch, source inclusion, build behavior, and diagnostics. It does not prescribe a project layout, graph schema, implementation strategy, or scoring model.

- **Baseline language:** Python 3.13 language semantics.
- **Baseline implementation:** CPython 3.13, using its normal bytecode interpreter and import machinery.
- **Baseline packaging:** a standards-based `pyproject.toml` build through a PEP 517-compatible frontend and backend, with no assumption that the project is installed editable.
- **Baseline host:** a conventional desktop/server operating system. Platform-specific cases are explicit checklist variants.
- **Compatibility envelope:** source may deliberately contain guarded constructs for Python 3.8 through 3.14, alternative Python implementations, and multiple operating systems. Python 2 is out of baseline scope, but compatibility shims that materially affect Python 3 semantics remain candidates.
- **Meaning of authority:** “language” means behavior specified by the Python language reference or an accepted Python Enhancement Proposal (PEP); “CPython” means implementation behavior documented or embodied by the reference implementation; “ecosystem” means interoperable packaging, typing, framework, or tooling convention rather than runtime language semantics.

## 1. Compilation units, modules, packages, and imports

### PY-MOD-001 — Script, importable module, and `__main__` identity

- **Distinct behavior:** The same source file can execute as the top-level module named `__main__` or load under its import name. That changes module identity, package context, relative-import behavior, and guarded startup effects.
- **Demonstration:** Include an importable module with declarations plus top-level behavior guarded by its runtime module name, and exercise it both by importing it and by executing it as a module or script.
- **Variants and failures:** Direct file execution differs from module execution with `-m`; importing after direct execution can produce two module objects; relative imports can fail when package context is absent; a package may supply its own `__main__.py`.
- **Constraints / authority / confidence / verification:** Python 3; language and CPython launcher behavior; **high**. Verify in the Python Language Reference and Python command-line documentation.

### PY-MOD-002 — Regular package initialization

- **Distinct behavior:** A directory containing `__init__.py` defines a regular package whose initializer executes once per interpreter module cache and may create or re-export its public surface.
- **Demonstration:** Include a package initializer that defines a name, imports a child, and re-exports another name used through the package.
- **Variants and failures:** Import order can expose partially initialized packages; package attributes for children appear after import; initializer exceptions leave nuanced `sys.modules` state; empty and side-effectful initializers differ.
- **Constraints / authority / confidence / verification:** Python 3; language/import-system behavior; **high**. Verify in the Python Language Reference and `importlib` documentation.

### PY-MOD-003 — Implicit namespace packages

- **Distinct behavior:** A package without `__init__.py` can span multiple search-path portions and has dynamic path composition rather than one defining source file.
- **Demonstration:** Provide at least two distribution or source-root portions contributing different children to one namespace package.
- **Variants and failures:** A regular package of the same name can take precedence; path order changes discovered portions; zip and installed portions interact; adding `__init__.py` changes identity and initialization semantics.
- **Constraints / authority / confidence / verification:** Python 3.3+; language/import system defined by PEP 420; **high**. Verify PEP 420, “Implicit Namespace Packages,” and `importlib` documentation.

### PY-MOD-004 — Absolute, explicit-relative, and parent-relative imports

- **Distinct behavior:** Import spelling selects resolution from `sys.path` or from the current package hierarchy, with dots encoding package ancestry rather than filesystem traversal alone.
- **Demonstration:** Include sibling and parent imports using absolute and explicit-relative forms, with a same-named top-level module that makes the distinction observable.
- **Variants and failures:** Relative import beyond the top-level package fails; direct script execution may have no known parent package; aliases bind different local names; refactoring package depth changes relative resolution.
- **Constraints / authority / confidence / verification:** Python 3; language, substantially clarified by PEP 328; **high**. Verify the Python Language Reference and PEP 328, “Imports: Multi-Line and Absolute/Relative.”

### PY-MOD-005 — `from` imports, aliases, and submodule fallback

- **Distinct behavior:** A `from package import name` may retrieve an existing package attribute or cause lookup/import of a submodule, then bind an independently aliased local name.
- **Demonstration:** Include imported values, imported submodules, aliases, and a package attribute whose name overlaps a possible child module.
- **Variants and failures:** Missing attributes raise `ImportError`; a package initializer can shadow a child; alias identity differs from qualified access only by binding; cyclic import timing changes which attribute exists.
- **Constraints / authority / confidence / verification:** Python 3; language/import-system behavior; **high**. Verify the import statement and import system in the Python Language Reference.

### PY-MOD-006 — Star import and declared export surface

- **Distinct behavior:** `from module import *` derives imported bindings from `__all__`, or otherwise from names not beginning with an underscore. Package `__all__` can trigger selected child imports.
- **Demonstration:** Include a module and package with explicit `__all__`, hidden names, re-exported names, and a consumer using star import at module scope.
- **Variants and failures:** `__all__` can name absent attributes and fail; without it underscore filtering is conventional visibility; star import is syntax-restricted outside module scope; static resolution must not assume every defined name is exported.
- **Constraints / authority / confidence / verification:** Python 3; language; **high**. Verify the import statement in the Python Language Reference.

### PY-MOD-007 — Import caching, cycles, and partial initialization

- **Distinct behavior:** Imports consult `sys.modules`, insert a module before executing it, and can expose a partially initialized object during a cycle.
- **Demonstration:** Include two modules importing each other, with one safe late access and one deliberately failing early access or diagnostic case.
- **Variants and failures:** Whole-module imports often survive cycles where immediate `from` imports fail; deleting cache entries can create duplicate identities; failed imports and re-imports differ by failure point.
- **Constraints / authority / confidence / verification:** Python 3; import system with CPython diagnostics; **high**. Verify the Python Language Reference import system and `sys.modules` documentation.

### PY-MOD-008 — Conditional and local imports

- **Distinct behavior:** Imports may occur only under runtime conditions or inside functions, so dependency existence and binding lifetime depend on control flow rather than source presence alone.
- **Demonstration:** Include imports guarded by platform, optional-dependency, and `TYPE_CHECKING` conditions, plus a function-local import used to avoid a cycle.
- **Variants and failures:** A missing optional dependency may be caught narrowly or accidentally masked; local imports still use the global module cache; names guarded only for type checking are absent at runtime.
- **Constraints / authority / confidence / verification:** Python 3; language plus typing ecosystem; **high**. Verify the Python Language Reference, `typing.TYPE_CHECKING`, and import system documentation.

### PY-MOD-009 — Dynamic import APIs and computed module names

- **Distinct behavior:** `importlib.import_module`, `__import__`, loaders, and computed strings introduce dependencies not expressed by import statements.
- **Demonstration:** Include a registry or configuration-derived module name loaded dynamically, and access a known symbol from the loaded module.
- **Variants and failures:** Relative dynamic imports require a package anchor; returned objects differ between `__import__` forms; invalid names and missing modules raise different exceptions; arbitrary loaders may synthesize modules.
- **Constraints / authority / confidence / verification:** Python 3; CPython standard-library import API consistent with language import semantics; **high**. Verify `importlib` and built-in `__import__` documentation.

### PY-MOD-010 — Module-level dynamic attributes

- **Distinct behavior:** A module can define `__getattr__` and `__dir__` so unresolved attribute access and discoverability are computed rather than limited to stored globals.
- **Demonstration:** Include a module that lazily produces or imports one documented attribute through module-level `__getattr__`, and declares a custom directory listing.
- **Variants and failures:** Direct global lookup inside the module bypasses module `__getattr__`; missing dynamic names must raise `AttributeError`; typing stubs may declare a broader or narrower surface.
- **Constraints / authority / confidence / verification:** Python 3.7+; language behavior defined by PEP 562; **high**. Verify PEP 562, “Module `__getattr__` and `__dir__`.”

### PY-MOD-011 — Custom finders, loaders, module specs, and non-file modules

- **Distinct behavior:** Meta-path finders and loaders can resolve modules from sources other than ordinary `.py` files, control creation/execution, and attach origin and package metadata through a module spec.
- **Demonstration:** Include a small custom finder/loader or a generated/imported module whose spec and origin make its non-file provenance observable.
- **Variants and failures:** Namespace loaders, zip importers, frozen modules, extension modules, and source loaders differ; a loader may return a pre-existing module; malformed specs cause package and relative-import failures.
- **Constraints / authority / confidence / verification:** Python 3.4+ modern protocol; language import system and CPython `importlib`; **high**. Verify PEP 451, “A ModuleSpec Type for the Import System,” and `importlib.abc` documentation.

### PY-MOD-012 — Search path, shadowing, and source-root configuration

- **Distinct behavior:** Module resolution depends on ordered search locations, execution mode, installation state, and tool configuration. A project module can shadow a standard-library or dependency module.
- **Demonstration:** Include two resolvable candidates for one name in distinct roots or a deliberate shadowing diagnostic, with project metadata that declares intended package roots.
- **Variants and failures:** Current-directory precedence differs by launcher mode; editable installs add indirection; environment `PYTHONPATH`, `.pth` files, virtual environments, and test runners mutate paths; case-insensitive filesystems collapse names.
- **Constraints / authority / confidence / verification:** Python 3; CPython startup/import behavior plus ecosystem configuration; **high**. Verify Python command-line, `site`, `sys.path`, and packaging-tool documentation.

### PY-MOD-013 — Source, bytecode-only, extension, frozen, and built-in modules

- **Distinct behavior:** Imported modules need not have Python source. Their callable/type surfaces and provenance can come from bytecode, native extension binaries, the interpreter, or frozen artifacts.
- **Demonstration:** Reference at least one normal source module and one built-in or extension module; optionally package a bytecode-only or frozen variant as a controlled case.
- **Variants and failures:** `__file__` may be absent; source locations may not exist; bytecode cache compatibility uses interpreter magic; extension suffixes and application binary interface (ABI) tags are platform-specific.
- **Constraints / authority / confidence / verification:** CPython and packaging behavior; **high**. Verify `importlib.machinery`, `marshal`, and extension-module documentation.

### PY-MOD-014 — Zip imports and package resources

- **Distinct behavior:** Modules may load from archives, where paths are not ordinary filesystem paths, while resource APIs address package resources through loader abstractions.
- **Demonstration:** Supply or build an archive containing importable Python plus non-code package data accessed with `importlib.resources` rather than path assumptions.
- **Variants and failures:** Native extensions generally cannot import directly from zip archives; namespace-package resource behavior varies by reader support; `__file__`-relative file opening fails for non-filesystem loaders.
- **Constraints / authority / confidence / verification:** CPython standard library; **high**. Verify `zipimport` and `importlib.resources` documentation.

### PY-MOD-015 — Mutable package search paths

- **Distinct behavior:** A package’s `__path__` controls child-module search and can be extended or replaced at runtime, allowing regular packages and legacy namespace mechanisms to span locations.
- **Demonstration:** Include a package that intentionally adds a second controlled location before importing a child found only there.
- **Variants and failures:** `pkgutil.extend_path`, `pkg_resources` legacy namespaces, direct list mutation, and implicit namespaces differ; path mutation after a failed import may require cache invalidation; static tools may ignore runtime changes.
- **Constraints / authority / confidence / verification:** Python import system plus ecosystem legacy mechanisms; **medium-high**. Verify package `__path__`, `pkgutil.extend_path`, and `importlib.invalidate_caches` documentation.

### PY-MOD-016 — Module reload and stale object references

- **Distinct behavior:** `importlib.reload` re-executes code in an existing module dictionary, retaining some old names while creating new function/class objects; external direct references are not rebound.
- **Demonstration:** Import a module, retain aliases and an old instance, change or select a second source state, reload, and compare qualified names, object identities, retained names, and behavior.
- **Variants and failures:** Reload is not thread-safe; extension initialization behavior varies; old instances keep old class identity; removed definitions can remain in the dictionary; recursive reload dependencies are not automatic.
- **Constraints / authority / confidence / verification:** CPython standard library/import semantics; **high**. Verify `importlib.reload` documentation.

## 2. Binding, scope, namespaces, and execution

### PY-BIND-001 — Local, enclosing, global, and built-in name resolution

- **Distinct behavior:** Unqualified names follow lexical local/enclosing/global/built-in resolution, while assignments classify names as local unless declared otherwise.
- **Demonstration:** Include nested functions reading names from all four levels, local shadowing, and an assignment that makes an earlier read an unbound-local error.
- **Variants and failures:** Class bodies do not form ordinary enclosing function scopes for methods; built-ins can be shadowed; compile-time binding classification precedes runtime branch execution.
- **Constraints / authority / confidence / verification:** Python 3; language; **high**. Verify execution model and naming/binding rules in the Python Language Reference.

### PY-BIND-002 — `global` and `nonlocal` rebinding

- **Distinct behavior:** Declarations redirect assignment and deletion to a module global or nearest enclosing function binding.
- **Demonstration:** Include closures that mutate enclosing state with `nonlocal` and functions that replace module state with `global`.
- **Variants and failures:** `nonlocal` requires an existing enclosing binding and cannot target module scope; declaration after use or conflicting parameters is a syntax error; mutation of an object needs neither declaration.
- **Constraints / authority / confidence / verification:** Python 3; language; **high**. Verify `global` and `nonlocal` statements in the Python Language Reference.

### PY-BIND-003 — Closures, cells, and late binding

- **Distinct behavior:** Nested functions capture variable cells, not frozen values, so callbacks created in a loop usually observe the final binding unless a value is captured separately.
- **Demonstration:** Include closures produced in a loop showing late binding and a contrasted form capturing each iteration value through a default or factory scope.
- **Variants and failures:** `nonlocal` mutates the shared cell; deleting a captured binding can cause a free-variable error; comprehension scopes change common closure examples.
- **Constraints / authority / confidence / verification:** Python 3; language with CPython cell representation; **high**. Verify the execution model in the Python Language Reference.

### PY-BIND-004 — Class-body namespace and method scope boundary

- **Distinct behavior:** A class body executes as a namespace-building block, but names defined there are not lexically visible as unqualified enclosing variables inside methods.
- **Demonstration:** Include class-body computed attributes, a method requiring qualified instance/class access, and a nested function or comprehension that exposes class-scope rules.
- **Variants and failures:** Zero-argument `super()` and `__class__` use a compiler-created cell; annotations and type-parameter scopes have version-specific exceptions; metaclasses can replace the prepared namespace.
- **Constraints / authority / confidence / verification:** Python 3; language; **high**. Verify class definitions, execution model, and data model in the Python Language Reference.

### PY-BIND-005 — Comprehension and generator-expression scopes

- **Distinct behavior:** Comprehension iteration variables are isolated from the containing scope in Python 3, while expressions can capture outer bindings and assignment expressions have special binding rules.
- **Demonstration:** Include list/set/dict comprehensions and a generator expression with nested clauses, an outer capture, and an assignment expression used in a permitted position.
- **Variants and failures:** The leftmost iterable evaluates in the enclosing scope; class-body comprehensions cannot freely read class locals; assignment expressions are forbidden in several comprehension positions; generator expressions are lazy.
- **Constraints / authority / confidence / verification:** Python 3, with assignment expressions in 3.8+; language; **high**. Verify displays, comprehensions, generator expressions, and PEP 572.

### PY-BIND-006 — Exception-handler target lifetime

- **Distinct behavior:** The name bound by an `except ... as name` clause is cleared after the handler to break reference cycles, unlike an ordinary block-local binding.
- **Demonstration:** Include a pre-existing name shadowed by an exception target, then access it after the handler as a deliberate failure or guarded diagnostic.
- **Variants and failures:** Exception groups bind subgroup objects per clause; captured traceback references can still retain frames elsewhere; absence of block scope should not be generalized from this special cleanup.
- **Constraints / authority / confidence / verification:** Python 3; language; **high**. Verify the `try` statement in the Python Language Reference.

### PY-BIND-007 — Structural-pattern capture scope and irrefutability

- **Distinct behavior:** Capture patterns bind names in the surrounding local scope rather than a nested case scope, and successful alternatives must bind compatible name sets.
- **Demonstration:** Include a `match` with literal, capture, wildcard, OR, and guarded cases, then use a captured name after a successful match path.
- **Variants and failures:** A failed match may leave partial bindings unspecified; duplicate captures and inconsistent OR bindings are syntax errors; a bare name is a capture, not a constant; unreachable cases after an irrefutable pattern are invalid.
- **Constraints / authority / confidence / verification:** Python 3.10+; language defined by PEP 634; **high**. Verify PEP 634, “Structural Pattern Matching: Specification,” and the compound statements reference.

### PY-BIND-008 — Dynamic execution with `exec`, `eval`, and namespace mappings

- **Distinct behavior:** Runtime-compiled text can read and create bindings in supplied global/local mappings, but optimized function locals and closure access constrain whether changes become normal local variables.
- **Demonstration:** Include dynamic evaluation against explicit dictionaries and execution that adds a discoverable global; contrast with an attempted function-local injection.
- **Variants and failures:** Separate globals and locals approximate class-body lookup; built-ins may be inserted automatically; compilation modes differ; untrusted input is unsafe; Python 3.13 refines default-locals semantics.
- **Constraints / authority / confidence / verification:** Python 3, with material 3.13 changes from PEP 667; language and CPython execution behavior; **high**. Verify built-in `exec`/`eval`, execution model, and PEP 667.

### PY-BIND-009 — Introspection of globals, locals, frames, and code objects

- **Distinct behavior:** Runtime namespaces and compiled metadata are observable through `globals()`, `locals()`, frames, code objects, and inspection APIs, but snapshot/write-through guarantees differ by scope.
- **Demonstration:** Inspect module, function, and class locals; expose a function’s qualified name, annotations, defaults, closure, and code metadata without depending on exact bytecode.
- **Variants and failures:** Optimized scopes return defined mappings with 3.13 semantics; tracing may affect frames; alternative implementations need not share CPython bytecode or frame details.
- **Constraints / authority / confidence / verification:** Language APIs plus CPython implementation details; **high** for namespace contracts, **medium** for frames. Verify built-ins, `inspect`, data model, and PEP 667.

### PY-BIND-010 — Deletion and binding absence

- **Distinct behavior:** `del` removes a binding or delegates to container/attribute deletion protocols; it does not itself destroy an object that remains referenced.
- **Demonstration:** Delete a local/global name, an object attribute, and a container item, then observe the distinct missing-name, missing-attribute, and missing-key/index behavior.
- **Variants and failures:** Deleting a free variable affects closure access; descriptors can implement deletion; `__slots__` members can become unset; finalization timing is separate from name deletion.
- **Constraints / authority / confidence / verification:** Python 3; language/data model; **high**. Verify the `del` statement and object model in the Python Language Reference.

### PY-BIND-011 — Private-name mangling and underscore conventions

- **Distinct behavior:** Identifiers beginning with two underscores inside a class definition are lexically mangled with the class name to reduce accidental subclass collisions; single-leading underscores and public/private exports are conventions, not access control.
- **Demonstration:** Include base and subclass attributes with the same double-underscore spelling, access through methods and explicit mangled names, and contrast with a single-underscore name and `__all__`.
- **Variants and failures:** Names ending with two underscores are not mangled; long class names are truncated by implementation rules; mangling occurs in nested expressions compiled in class context; reflective access can always bypass it.
- **Constraints / authority / confidence / verification:** Python 3; language lexical rules plus convention; **high**. Verify identifiers/private-name mangling and style conventions in official Python documentation.

## 3. Callables, parameters, decorators, and suspended execution

### PY-CALL-001 — Parameter kinds and call binding

- **Distinct behavior:** Positional-only, positional-or-keyword, variadic positional, keyword-only, and variadic keyword parameters participate in a defined argument-binding algorithm.
- **Demonstration:** Define and call a function containing every parameter kind, including valid unpacked calls and invalid duplicate, missing, positional, and unexpected arguments.
- **Variants and failures:** Built-ins may expose positional-only signatures; parameter names before `/` are not callable keywords; `*` can introduce keyword-only parameters without a variadic binding.
- **Constraints / authority / confidence / verification:** Positional-only syntax Python 3.8+; language, PEP 570; **high**. Verify function definitions/calls and PEP 570.

### PY-CALL-002 — Default values and definition-time evaluation

- **Distinct behavior:** Parameter defaults evaluate once when the function definition executes and remain objects attached to the function, so mutable defaults retain state.
- **Demonstration:** Include a mutable-default behavior case and a contrasted sentinel-based case, plus a default referring to an outer name later rebound.
- **Variants and failures:** Defaults must follow non-defaults within applicable parameter groups; decorators see the already-created function; annotations may evaluate under a different policy from defaults.
- **Constraints / authority / confidence / verification:** Python 3; language; **high**. Verify function definitions and calls in the Python Language Reference.

### PY-CALL-003 — Call unpacking and evaluation order

- **Distinct behavior:** Calls can merge positional and keyword unpackings, with left-to-right expression evaluation and runtime detection of duplicate or nonconforming arguments.
- **Demonstration:** Include multiple iterable and mapping unpackings whose evaluation has observable order, plus duplicate-key and non-string-key failure cases.
- **Variants and failures:** Grammar permits interleaving forms subject to ordering rules; mapping subclasses can customize iteration; duplicate explicit/unpacked keywords raise `TypeError` before entering the callee.
- **Constraints / authority / confidence / verification:** Generalized unpacking Python 3.5+; language, PEP 448; **high**. Verify calls and PEP 448.

### PY-CALL-004 — First-class functions, lambdas, and qualified identity

- **Distinct behavior:** Function definitions and lambdas create objects that can be aliased, stored, returned, and introspected independently of the binding that first received them.
- **Demonstration:** Alias a named function, return a nested function, and store both a lambda and function in a registry; observe name and qualified-name metadata.
- **Variants and failures:** Rebinding does not change function metadata; lambdas permit expressions only; nested functions with identical simple names differ by qualified name and object identity.
- **Constraints / authority / confidence / verification:** Python 3; language/data model; **high**. Verify function definitions, lambda expressions, and callable types.

### PY-CALL-005 — Decorator evaluation and application order

- **Distinct behavior:** Decorator expressions evaluate when a function or class is defined, and multiple decorators wrap the created object in a specified bottom-up application order.
- **Demonstration:** Use stacked function and class decorators, including one preserving wrapped metadata and one replacing the object with a distinct callable or class.
- **Variants and failures:** Decorators may return any object; metadata preservation by `functools.wraps` is convention, not automatic; descriptor behavior changes when decorators are ordered around `classmethod`, `staticmethod`, or `property`.
- **Constraints / authority / confidence / verification:** Python 3; language plus standard-library convention; **high**. Verify function/class definitions and `functools.wraps` documentation.

### PY-CALL-006 — Bound methods and method descriptor forms

- **Distinct behavior:** Accessing a normal function through an instance produces a bound method carrying the instance; static methods suppress binding; class methods bind the class.
- **Demonstration:** Define all three forms, access them through both class and instance, and inspect or compare the resulting callable objects and receivers.
- **Variants and failures:** Retrieving the raw class dictionary bypasses descriptor binding; each normal method access can create a new bound-method object; assigning a plain function to an instance does not invoke class descriptor binding.
- **Constraints / authority / confidence / verification:** Python 3; language data model; **high**. Verify descriptor invocation and standard type hierarchy in the Python Language Reference.

### PY-CALL-007 — Callable instances and signature indirection

- **Distinct behavior:** Instances become callable through a class-defined `__call__`, so call relationships can target an object’s type-level protocol rather than a function binding.
- **Demonstration:** Include a stateful callable class, pass an instance where a callback is expected, and contrast its runtime call with its constructor call.
- **Variants and failures:** Metaclass `__call__` controls calling a class; an instance attribute named `__call__` does not normally satisfy special-method lookup; introspected signatures can be customized.
- **Constraints / authority / confidence / verification:** Python 3; language data model; **high**. Verify special method lookup and emulating callable objects.

### PY-CALL-008 — Generators, `yield`, and generator return values

- **Distinct behavior:** A function containing `yield` returns a generator without executing its body; iteration resumes suspended frames, and `return value` becomes `StopIteration.value`.
- **Demonstration:** Include a generator with multiple yields, sent values, exception injection, finalization cleanup, and an explicit return value consumed manually or via delegation.
- **Variants and failures:** Re-entering a running generator fails; unhandled `StopIteration` escaping generator code is transformed; closing injects `GeneratorExit`; exhausted generators stay exhausted.
- **Constraints / authority / confidence / verification:** Python 3; language, including PEP 342 and PEP 479; **high**. Verify generator expressions/functions and generator object methods.

### PY-CALL-009 — Delegating generators with `yield from`

- **Distinct behavior:** `yield from` delegates iteration, send, throw, close, and final return-value capture to a subiterator through a defined protocol.
- **Demonstration:** Include an outer generator delegating to an inner generator and to a plain iterable, capturing the inner generator’s return value.
- **Variants and failures:** Plain iterators lack send/throw features; close propagation differs by available methods; exceptions can be handled by either layer; manual loops are not semantically equivalent.
- **Constraints / authority / confidence / verification:** Python 3.3+; language, PEP 380; **high**. Verify PEP 380, “Syntax for Delegating to a Subgenerator.”

### PY-CALL-010 — Native coroutines and `await`

- **Distinct behavior:** `async def` creates coroutine functions whose calls return coroutine objects; `await` suspends through the awaitable protocol and is syntax-restricted to asynchronous contexts.
- **Demonstration:** Include coroutine creation, awaiting another coroutine and a custom awaitable, task scheduling, and a deliberately un-awaited coroutine diagnostic.
- **Variants and failures:** Calling does not run the body; awaiting the same coroutine twice fails; event-loop policy is library-specific; cancellation arrives as an exception at suspension points.
- **Constraints / authority / confidence / verification:** Python 3.5+; language, PEP 492, plus `asyncio`; **high**. Verify coroutines in the Python Language Reference and `asyncio` documentation.

### PY-CALL-011 — Asynchronous iteration and generators

- **Distinct behavior:** `async for` uses `__aiter__`/`__anext__`, while an asynchronous generator combines `async def` and `yield` with asynchronous finalization semantics.
- **Demonstration:** Include a custom asynchronous iterator, an asynchronous generator, comprehension over it, early termination, and explicit or context-managed cleanup.
- **Variants and failures:** `__anext__` ends with `StopAsyncIteration`; asynchronous generators cannot return a value; finalization may depend on loop hooks; older `__aiter__` awaitable behavior is obsolete.
- **Constraints / authority / confidence / verification:** Python 3.6+ for async generators; language, PEP 525 and PEP 530; **high**. Verify asynchronous iterators/generators in the Python Language Reference.

### PY-CALL-012 — Context propagation and task-local state

- **Distinct behavior:** `contextvars` values follow copied execution context across asynchronous tasks while remaining isolated from unrelated contexts, unlike ordinary globals or thread locals.
- **Demonstration:** Set a context variable, spawn concurrent tasks with differing values, and show propagation into nested coroutines and restoration after reset.
- **Variants and failures:** Thread transitions need explicit context handling; task creation captures current context; token reset order matters; framework scheduling can mediate propagation.
- **Constraints / authority / confidence / verification:** Python 3.7+; standard-library/CPython API from PEP 567; **high**. Verify PEP 567, “Context Variables,” and `contextvars` documentation.

## 4. Classes, inheritance, descriptors, and object identity

### PY-OBJ-001 — Class creation protocol and namespace preparation

- **Distinct behavior:** Class definition evaluates bases and keywords, chooses a metaclass, asks it to prepare a namespace, executes the body, and invokes the metaclass to create the class.
- **Demonstration:** Include a metaclass with custom namespace preparation and creation hooks, a class keyword, and body declarations whose order is observable.
- **Variants and failures:** Incompatible metaclasses cause conflict; `__mro_entries__` can replace non-type bases; ordered namespace behavior is guaranteed in modern Python; a class decorator runs after metaclass creation.
- **Constraints / authority / confidence / verification:** Python 3; language data model, PEP 3115 and PEP 560 refinements; **high**. Verify “Customizing class creation” in the Python Language Reference.

### PY-OBJ-002 — Construction through `__new__`, `__init__`, and metaclass `__call__`

- **Distinct behavior:** Calling a class is mediated by its metaclass, normally invoking allocation with `__new__` and initialization with `__init__`; returning a foreign type from `__new__` skips that class’s initializer.
- **Demonstration:** Include an immutable-style subclass customizing `__new__`, a normal initializer, and a controlled case returning or caching an existing instance.
- **Variants and failures:** `__init__` must return `None`; cooperative multiple inheritance affects both hooks; metaclass `__call__` can bypass ordinary construction; pickling may reconstruct differently.
- **Constraints / authority / confidence / verification:** Python 3; language data model; **high**. Verify basic customization and object creation in the Python Language Reference.

### PY-OBJ-003 — Single and multiple inheritance with C3 method resolution order

- **Distinct behavior:** Attribute and method lookup across multiple bases follows the computed C3 method resolution order (MRO), preserving local precedence and monotonicity.
- **Demonstration:** Include a diamond hierarchy with one overridden method per level, inspect the MRO, and exercise cooperative dispatch.
- **Variants and failures:** Inconsistent base order rejects class creation; changing base order changes resolution; virtual subclassing does not enter MRO; extension types may impose layout conflicts.
- **Constraints / authority / confidence / verification:** Python 3; language/CPython type system; **high**. Verify the Python data model and the Python MRO documentation.

### PY-OBJ-004 — Zero-argument and explicit `super`

- **Distinct behavior:** `super()` performs lookup after a designated class in an object’s MRO; zero-argument form depends on an implicit `__class__` cell and first argument.
- **Demonstration:** Use cooperative methods in a diamond and both zero-argument and explicit `super` forms, including a class method.
- **Variants and failures:** Nested functions and comprehensions may not inherit the needed first-argument context; hard-coded base calls bypass cooperative order; assigning `__class__` changes closure-sensitive behavior only within language constraints.
- **Constraints / authority / confidence / verification:** Python 3; language data model/built-in; **high**. Verify built-in `super` and method resolution documentation.

### PY-OBJ-005 — Instance, class, and metaclass attribute lookup

- **Distinct behavior:** Attribute lookup traverses instance storage, class MRO, descriptors, and for class objects the metaclass MRO, with different precedence for data and non-data descriptors.
- **Demonstration:** Include same-named values at instance, class, base-class, and metaclass levels, plus both descriptor kinds.
- **Variants and failures:** Special-method lookup often bypasses instance storage; class assignment can replace descriptors; `__getattribute__` can intercept ordinary access; metaclass descriptors affect class attributes.
- **Constraints / authority / confidence / verification:** Python 3; language data model; **high**. Verify attribute access and descriptor invocation in the Python Language Reference.

### PY-OBJ-006 — Data and non-data descriptors

- **Distinct behavior:** Objects defining `__get__` mediate attribute reads; defining `__set__` or `__delete__` makes a data descriptor that takes precedence over instance attributes.
- **Demonstration:** Include a validating data descriptor and a cached/non-data descriptor, assign a colliding instance value, and access each through class and instance.
- **Variants and failures:** `__set_name__` receives the owner/name at class creation; descriptor reuse across owners needs care; raising `AttributeError` can trigger fallback; properties, functions, slots, and many built-ins are descriptor applications.
- **Constraints / authority / confidence / verification:** Python 3; language data model; **high**. Verify the Descriptor HowTo Guide and data model.

### PY-OBJ-007 — Properties and managed attributes

- **Distinct behavior:** `property` combines getter, setter, and deleter functions into a data descriptor, separating public attribute syntax from backing state and validation.
- **Demonstration:** Include read-only, read-write, and deletable properties, inheritance override of an accessor, and an accessor that raises `AttributeError` deliberately.
- **Variants and failures:** Setter/deleter decorators create a new property object; property access through class returns the descriptor; abstract properties interact with abstract base classes; cached properties are non-data descriptors with different shadowing.
- **Constraints / authority / confidence / verification:** Python 3; language-standard built-in; **high**. Verify built-in `property` and descriptor documentation.

### PY-OBJ-008 — `__getattribute__`, `__getattr__`, and `__setattr__`

- **Distinct behavior:** Custom attribute interception can override all ordinary reads, provide fallback for missing attributes, and mediate writes or deletes.
- **Demonstration:** Include a proxy-like class forwarding selected missing attributes, validating writes, and delegating internally to `object` to avoid recursion.
- **Variants and failures:** `__getattr__` runs only after normal lookup raises `AttributeError`; accidental self-access recurses; special-method lookup can bypass these hooks; modules have a distinct `__getattr__` protocol.
- **Constraints / authority / confidence / verification:** Python 3; language data model; **high**. Verify customizing attribute access in the Python Language Reference.

### PY-OBJ-009 — `__slots__`, object layout, and weak references

- **Distinct behavior:** `__slots__` creates member descriptors and can omit per-instance `__dict__` and weak-reference support, changing storage, permitted attributes, and inheritance layout.
- **Demonstration:** Include a slotted class, a subclass adding slots, deliberate assignment of an undeclared attribute, and weak-reference behavior with and without the required slot.
- **Variants and failures:** An unslotted subclass regains `__dict__`; multiple slotted bases can conflict; repeating a slot name can hide base storage; slot descriptors are not ordinary defaults.
- **Constraints / authority / confidence / verification:** Python 3; language with CPython layout implications; **high**. Verify `__slots__` in the Python data model.

### PY-OBJ-010 — Abstract base classes and virtual subclasses

- **Distinct behavior:** Abstract methods prevent ordinary instantiation until concretely implemented, while registration and `__subclasshook__` can make unrelated classes pass subclass/instance checks without entering inheritance or MRO.
- **Demonstration:** Include an abstract base class with abstract and concrete methods, a concrete subclass, a registered virtual subclass, and a structural subclass hook.
- **Variants and failures:** Abstract decorators must be ordered correctly with descriptors; virtual subclasses do not inherit implementations; caches may need invalidation; protocols offer a static alternative.
- **Constraints / authority / confidence / verification:** Python 3; standard-library semantics; **high**. Verify `abc` documentation and PEP 3119.

### PY-OBJ-011 — Class initialization hooks and subclass registration

- **Distinct behavior:** `__init_subclass__` runs on subclass creation with class keywords, and `__set_name__` informs attributes/descriptors of their assigned names.
- **Demonstration:** Include a base class that records or validates subclasses and a descriptor using `__set_name__`, with cooperative forwarding of unknown class keywords.
- **Variants and failures:** Multiple bases require cooperative hooks; assigning a descriptor after class creation does not automatically call `__set_name__`; metaclasses may duplicate or supersede registry behavior.
- **Constraints / authority / confidence / verification:** Python 3.6+; language, PEP 487; **high**. Verify PEP 487, “Simpler customisation of class creation.”

### PY-OBJ-012 — Runtime mutation and monkey patching

- **Distinct behavior:** Module globals and most class attributes can be added, replaced, or deleted after definition, changing behavior of existing instances and imported aliases depending on what was bound.
- **Demonstration:** Replace a class method after creating an instance, add an attribute to a module, and contrast qualified lookup with a previously imported direct alias.
- **Variants and failures:** Extension and immutable built-in types may reject mutation; cached bound methods retain their original function; patch timing matters; descriptors assigned later may miss creation hooks.
- **Constraints / authority / confidence / verification:** Python 3; language object model plus implementation restrictions; **high**. Verify attribute assignment and type-object documentation.

### PY-OBJ-013 — Dataclass-generated methods and fields

- **Distinct behavior:** `@dataclass` interprets annotated class attributes as fields and generates initialization, representation, comparison, hashing, pattern-matching, and optionally slots according to configuration.
- **Demonstration:** Include ordinary/default/default-factory fields, excluded class variables and init-only variables, inheritance, frozen behavior, keyword-only fields, and a post-init hook.
- **Variants and failures:** Mutable literal defaults are rejected under evolving checks; field order across inheritance can invalidate constructors; equality is same-type by default; unsafe hash choices interact with mutability; `slots` returns a replacement class.
- **Constraints / authority / confidence / verification:** Python 3.7+, with keyword-only/match args/slots additions in later versions; standard library, PEP 557; **high**. Verify `dataclasses` documentation.

### PY-OBJ-014 — Enumerations and member identity

- **Distinct behavior:** An `Enum` class transforms class-body values into singleton members, distinguishes names from values, and changes construction, iteration, comparison, and subclassing rules.
- **Demonstration:** Include aliases, automatically generated values, an integer-mixing enum or flag enum, custom methods, and value-to-member lookup.
- **Variants and failures:** Aliases are omitted from normal iteration; unique verification can reject them; `IntEnum` compares like integers and can lose enum type in operations; members cannot normally be extended with new members by subclassing.
- **Constraints / authority / confidence / verification:** Python 3; standard library with version additions such as `StrEnum` in 3.11; **high**. Verify `enum` documentation and PEP 435.

### PY-OBJ-015 — Named tuples and tuple subclasses with declared fields

- **Distinct behavior:** `collections.namedtuple` and `typing.NamedTuple` generate tuple subclasses whose fields are both indexed positions and descriptors, with generated constructors and metadata.
- **Demonstration:** Include both factory and class syntax, defaults, methods, annotations, unpacking, and immutable field-assignment failure.
- **Variants and failures:** Renaming invalid fields is factory-specific; default ordering matters; equality remains tuple-compatible unlike many record classes; generic named tuples require newer typing syntax.
- **Constraints / authority / confidence / verification:** Python 3; standard library and typing ecosystem; **high**. Verify `collections.namedtuple` and `typing.NamedTuple` documentation.

### PY-OBJ-016 — Object identity, equality, hashing, and mutability contract

- **Distinct behavior:** `is` compares identity, `==` dispatches rich comparison, and hashing must remain stable and consistent with equality for mapping/set membership.
- **Demonstration:** Include value-equal distinct objects, singleton identity checks, a correctly hashable immutable value type, and an unhashable mutable value type.
- **Variants and failures:** Interning makes identity of some immutable literals implementation-dependent; overriding equality commonly disables inherited hashing; hash randomization changes cross-process values; NaN-like non-reflexive equality is legal.
- **Constraints / authority / confidence / verification:** Language contracts with CPython optimization/randomization; **high**. Verify comparisons and `object.__hash__` in the Python Language Reference.

### PY-OBJ-017 — Weak references and finalization

- **Distinct behavior:** Weak references observe objects without keeping them alive, and callbacks/finalizers run when collection occurs subject to lifetime and shutdown constraints.
- **Demonstration:** Create weak references, a weak collection, and a `weakref.finalize` callback for an eligible object, then remove strong references without requiring exact timing.
- **Variants and failures:** Some built-ins and slotted instances lack weak-reference support; callbacks cannot safely recover the referent; cycles and interpreter shutdown affect timing; resurrection complicates `__del__` behavior.
- **Constraints / authority / confidence / verification:** Python standard library with implementation-dependent collection timing; **high** for API, **medium** for timing. Verify `weakref` and data-model finalization documentation.

### PY-OBJ-018 — Class subscription and custom generic aliases

- **Distinct behavior:** Subscribing a class invokes metaclass `__getitem__` or class `__class_getitem__`, enabling runtime generic aliases or domain-specific class-level indexing distinct from instance subscription.
- **Demonstration:** Include a class implementing `__class_getitem__`, inspect the received key and returned alias-like object, and contrast with instance `__getitem__` and a metaclass override.
- **Variants and failures:** Metaclass `__getitem__` takes precedence; built-in/typing aliases implement richer contracts; custom use outside typing may confuse static tools; subscription expressions can carry tuples and slices.
- **Constraints / authority / confidence / verification:** Python 3.7+ protocol, PEP 560; language data model; **high**. Verify `__class_getitem__` and PEP 560.

### PY-OBJ-019 — Customized instance and subclass checks

- **Distinct behavior:** Metaclass `__instancecheck__` and `__subclasscheck__` can make runtime checks succeed without ordinary inheritance, introducing dynamic type relationships.
- **Demonstration:** Include a metaclass or abstract base class with a controlled structural check and contrast runtime membership with actual MRO and inherited attributes.
- **Variants and failures:** Results may be cached by `abc`; recursive checks can loop; parameterized generics remain restricted; static type checkers do not generally infer arbitrary runtime hooks.
- **Constraints / authority / confidence / verification:** Python 3; language data model and `abc`; **high**. Verify customizing instance/subclass checks and `abc` documentation.

## 5. Data model protocols and dispatch

### PY-PROT-001 — Truth testing and Boolean short-circuit values

- **Distinct behavior:** Truth tests call `__bool__` or fall back to `__len__`; `and` and `or` short-circuit and return an operand rather than coercing to `bool`.
- **Demonstration:** Include custom truthy/falsy objects with both protocols, short-circuit expressions with observable skipped effects, and operand-return behavior.
- **Variants and failures:** `__bool__` must return actual `bool`; negative `__len__` fails; array/dataframe ecosystems may deliberately reject ambiguous truth; `not` always returns `bool`.
- **Constraints / authority / confidence / verification:** Python 3; language data model; **high**. Verify truth-value testing and Boolean operations.

### PY-PROT-002 — Rich comparisons, reflection, and chaining

- **Distinct behavior:** Comparison operators dispatch to left and reflected right methods with subclass priority and `NotImplemented` fallback; chained comparisons evaluate shared operands once and short-circuit.
- **Demonstration:** Include two interacting operand classes, `NotImplemented`, equality and ordering, and a chained comparison with observable evaluation count.
- **Variants and failures:** `==` ultimately falls back to identity-based behavior while ordering generally raises `TypeError`; `NotImplemented` is not the same as raising; `functools.total_ordering` synthesizes methods.
- **Constraints / authority / confidence / verification:** Python 3; language data model; **high**. Verify value comparisons and rich comparison methods.

### PY-PROT-003 — Binary arithmetic, reflected, and in-place dispatch

- **Distinct behavior:** Binary operators try left and reflected methods under subtype rules, while augmented assignment first tries an in-place method and then falls back to binary result plus rebinding.
- **Demonstration:** Include operand classes implementing primary, reflected, and in-place methods, including `NotImplemented` and an alias that exposes mutation versus rebinding.
- **Variants and failures:** Sequence `+=` may mutate despite later assignment failure; matrix multiplication has its own hooks; augmented assignment to attributes/items performs read-operation-write phases; built-in numeric coercions vary.
- **Constraints / authority / confidence / verification:** Python 3; language data model; **high**. Verify emulating numeric types and augmented assignment.

### PY-PROT-004 — Container length, indexing, slicing, and membership

- **Distinct behavior:** Container syntax dispatches through `__len__`, `__getitem__`, `__setitem__`, `__delitem__`, and `__contains__`; slicing passes a `slice` object and membership has iteration/indexing fallbacks.
- **Demonstration:** Include a custom container supporting scalar and slice access, mutation/deletion, negative-index normalization, and explicit membership behavior.
- **Variants and failures:** Missing `__contains__` falls back to iteration then legacy integer indexing; `__index__` supplies exact integer conversion; extended slices carry a step; mappings and sequences interpret keys differently.
- **Constraints / authority / confidence / verification:** Python 3; language data model; **high**. Verify emulating container types and expression subscriptions/slicings.

### PY-PROT-005 — Iteration protocol and iterator identity

- **Distinct behavior:** `iter` obtains an iterator from `__iter__` or sequence fallback; iterators return themselves from `__iter__`, advance with `__next__`, and remain exhausted once done by convention.
- **Demonstration:** Include a re-iterable container and separate one-shot iterator, manual `next` with default, and simultaneous iterations showing independent or shared state.
- **Variants and failures:** A broken iterator that resumes after `StopIteration` violates the protocol; two-argument `iter` creates a sentinel iterator from a callable; mutation during iteration has container-specific behavior.
- **Constraints / authority / confidence / verification:** Python 3; language data model; **high**. Verify iterator types and `iter`/`next` built-ins.

### PY-PROT-006 — Context manager protocol

- **Distinct behavior:** `with` evaluates a manager, calls enter, binds its result, and calls exit on normal or exceptional completion; exit can suppress exceptions by returning true.
- **Demonstration:** Include class-based and generator-based context managers, multiple managers, suppression and propagation cases, and cleanup when body setup partially fails.
- **Variants and failures:** Multiple managers behave like nested statements; `contextlib.ExitStack` supports dynamic composition; return value from enter need not be manager; exceptions in exit replace or chain prior exceptions.
- **Constraints / authority / confidence / verification:** Python 3; language plus `contextlib`, PEP 343; **high**. Verify the `with` statement and context manager types.

### PY-PROT-007 — Asynchronous context manager protocol

- **Distinct behavior:** `async with` awaits `__aenter__` and `__aexit__`, enabling asynchronous acquisition, cleanup, and exception suppression.
- **Demonstration:** Include a class-based asynchronous manager and an `asynccontextmanager`, with normal completion, raised exception, and cancellation during the body.
- **Variants and failures:** It is valid only in asynchronous function bodies; cleanup itself can be cancelled unless protected; sync and async manager protocols are not interchangeable; multiple managers nest asynchronously.
- **Constraints / authority / confidence / verification:** Python 3.5+; language plus `contextlib`; **high**. Verify asynchronous context managers and PEP 492.

### PY-PROT-008 — Representation, string, format, and bytes conversion

- **Distinct behavior:** `repr`, `str`, formatted values, and `bytes` dispatch through distinct protocols with defined fallbacks and conversion flags.
- **Demonstration:** Include an object implementing `__repr__`, `__str__`, and `__format__`, then exercise ordinary formatting, explicit conversions, and a bytes-producing object.
- **Variants and failures:** `__repr__` and `__str__` must return strings; f-string `!r`, `!s`, and `!a` convert before formatting; `object.__format__` accepts limited specs; recursive representations need guarding.
- **Constraints / authority / confidence / verification:** Python 3; language data model; **high**. Verify string conversion, formatted string literals, and special methods.

### PY-PROT-009 — Numeric conversion and index protocol

- **Distinct behavior:** Exact integer contexts use `__index__`, while explicit numeric conversions use `__int__`, `__float__`, or `__complex__` with documented fallbacks.
- **Demonstration:** Include a numeric-like object with distinct index and integer conversion behavior, used in slicing, range, conversion, and a byte operation.
- **Variants and failures:** Returning a strict non-integer is rejected; `int` fallback rules have changed over releases; `bool` is an integer subtype; truncation-related fallbacks are being deprecated or removed.
- **Constraints / authority / confidence / verification:** Python 3; language/standard built-ins with version changes; **high**. Verify numeric emulation and built-in conversion functions.

### PY-PROT-010 — Special-method lookup bypass

- **Distinct behavior:** Implicit syntax generally looks up special methods on the type, bypassing an instance dictionary and often ordinary `__getattribute__`, to preserve consistent operator behavior.
- **Demonstration:** Assign a special-method-named callable to one instance and contrast explicit attribute calling with operator or built-in syntax; then define it on the class.
- **Variants and failures:** Exact lookup paths differ among operations and metaclass cases; normal methods do bind through descriptor behavior; monkey patching the type can affect existing instances.
- **Constraints / authority / confidence / verification:** Python 3; language data model with CPython details; **high**. Verify “Special method lookup” in the Python Language Reference.

### PY-PROT-011 — Structural pattern matching protocol surface

- **Distinct behavior:** Matching dispatches differently for sequence, mapping, class, literal, OR, AS, and wildcard patterns; class patterns use type checks plus `__match_args__` or named attributes.
- **Demonstration:** Include each major pattern family, a class with explicit positional match metadata, a mapping with extra keys, a sequence, nested patterns, guards, and value patterns using qualified constants.
- **Variants and failures:** Strings/bytes are excluded from sequence patterns; mappings use two-argument `get` semantics; duplicate mapping keys can be syntax or runtime errors; guards may have side effects; pattern protocol call counts are partly unspecified.
- **Constraints / authority / confidence / verification:** Python 3.10+; language, PEP 634; **high**. Verify PEP 634 and the `match` statement reference.

### PY-PROT-012 — Copying, replacement, and pickling protocols

- **Distinct behavior:** Shallow/deep copying and serialization discover reduction, state, construction, and memoization hooks rather than merely traversing visible attributes.
- **Demonstration:** Include an object graph with aliasing/cycles, custom copy behavior, custom pickle state, and successful round-trip that preserves selected identity relationships.
- **Variants and failures:** Local functions, lambdas, locks, open resources, and dynamically defined classes are often unpicklable; pickle refers to importable qualified names; protocol versions differ; loading untrusted pickle is unsafe.
- **Constraints / authority / confidence / verification:** CPython standard library and ecosystem persistence behavior; **high**. Verify `copy`, `pickle`, and “Pickling Class Instances” documentation.

### PY-PROT-013 — Buffer protocol and zero-copy views

- **Distinct behavior:** Binary-producing objects can export a buffer consumed by `memoryview` and native interfaces without copying, with shape, format, mutability, and lifetime constraints.
- **Demonstration:** Use a mutable binary exporter and sliced/cast memory views, showing shared mutation and release behavior; optionally include a native exporter.
- **Variants and failures:** Contiguity and format restrict casts; resizing an exporter with active views can fail; Python-level buffer export becomes available through newer language support; alternative implementations vary.
- **Constraints / authority / confidence / verification:** CPython data model/C API, with Python-level protocol additions in 3.12 via PEP 688; **medium**. Verify Buffer Protocol documentation and PEP 688.

## 6. Types, annotations, generics, and static contracts

Python annotations are mostly runtime metadata plus an ecosystem of static rules. A comprehensive project should distinguish what the interpreter enforces, what typing specifications require, and what individual type checkers infer or diagnose.

### PY-TYPE-001 — Annotation storage and evaluation policy

- **Distinct behavior:** Function, class, and module annotations may be evaluated when the annotated statement executes, stored as strings under postponed evaluation, or obtained through newer deferred-evaluation machinery, depending on version and future imports.
- **Demonstration:** Include annotations referring to already defined names, forward names, expressions with observable evaluation, and introspection through `__annotations__` and `typing.get_type_hints` under the baseline policy.
- **Variants and failures:** `from __future__ import annotations` stringizes most annotations through 3.13; unresolved forward references can fail during type-hint resolution; class and nested scopes affect evaluation; Python 3.14 is expected to materially change defaults through PEP 649/749.
- **Constraints / authority / confidence / verification:** Python 3.7+ policy variants; language plus typing/runtime APIs; **high** through 3.13, **medium** for 3.14. Verify annotations in the Python Language Reference, `__future__`, `typing`, PEP 563, PEP 649, and PEP 749.

### PY-TYPE-002 — Variable annotations and annotation-only bindings

- **Distinct behavior:** Annotating a variable records metadata at module/class scope but does not necessarily bind a runtime value; local annotations affect compiler classification without a runtime annotations mapping.
- **Demonstration:** Include annotated assignments with and without values at module, class, instance, and function scopes, then distinguish annotation presence from name/attribute existence.
- **Variants and failures:** Instance annotations are not collected on the instance automatically; annotations on targets other than simple names have different storage behavior; future/deferred policies affect value representation.
- **Constraints / authority / confidence / verification:** Python 3.6+; language, PEP 526; **high**. Verify annotated assignment statements and PEP 526.

### PY-TYPE-003 — Built-in generic aliases and union types

- **Distinct behavior:** Parameterizing built-in collections produces runtime generic alias objects, and `|` constructs union type objects, while static tools interpret them as type expressions.
- **Demonstration:** Annotate equivalent container and optional unions using modern syntax, introspect origin/arguments, and use a union in permitted runtime instance/subclass checks.
- **Variants and failures:** Parameterized generics are generally invalid as the second argument of `isinstance`; metaclasses can override `__or__`; typing aliases and built-in aliases have version-specific runtime equality/pickling behavior.
- **Constraints / authority / confidence / verification:** Built-in generics Python 3.9+ (PEP 585), unions Python 3.10+ (PEP 604); language/typing; **high**. Verify PEP 585, PEP 604, and `types.UnionType`.

### PY-TYPE-004 — Type aliases, including explicit `type` statements

- **Distinct behavior:** An ordinary assignment can be interpreted statically as a type alias, `TypeAlias` can disambiguate it, and Python 3.12’s `type` statement creates a distinct lazy `TypeAliasType` with its own parameter scope.
- **Demonstration:** Include legacy, explicitly marked, generic, recursive, and `type`-statement aliases, then inspect the runtime object produced by the new form.
- **Variants and failures:** `type` statements are syntax errors before 3.12; aliases are not classes and generally cannot substitute into runtime class checks; forward/lazy value evaluation differs by form.
- **Constraints / authority / confidence / verification:** Python 3.12+ for `type`; language/typing, PEP 613 and PEP 695; **high**. Verify type statements in the Python Language Reference and the typing specification.

### PY-TYPE-005 — Type variables, bounds, constraints, variance, and defaults

- **Distinct behavior:** Generic parameters can be bounded, constrained, covariant, contravariant, invariant, inferred for variance, or defaulted, shaping static substitution while producing runtime metadata.
- **Demonstration:** Include generic functions and classes with bounded and constrained parameters, variance-sensitive producer/consumer positions, and both legacy `TypeVar` and 3.12 parameter-list syntax.
- **Variants and failures:** Bounds and constraints have different solving behavior; runtime calls do not enforce substitutions; parameter syntax introduces lexical type-parameter scopes; defaults require recent versions and ordering rules.
- **Constraints / authority / confidence / verification:** Core generics Python 3.5+; syntax/inferred variance 3.12+ via PEP 695; defaults 3.13-era via PEP 696; typing specification; **medium-high**. Verify `typing`, PEP 484, PEP 695, and PEP 696.

### PY-TYPE-006 — Generic classes, generic functions, and runtime erasure

- **Distinct behavior:** Generic specialization describes static types but usually constructs the same runtime class; instances ordinarily do not retain concrete type arguments without custom support.
- **Demonstration:** Define generic containers/functions in old and new syntax, specialize them, construct instances, and inspect `__orig_bases__`, parameters, and any instance `__orig_class__` behavior.
- **Variants and failures:** User-defined generic aliases and built-ins differ; `__orig_class__` is best-effort and timing-sensitive; metaclasses can conflict with legacy generic machinery; subclass specialization can bind or propagate parameters.
- **Constraints / authority / confidence / verification:** Python 3.5+, new syntax 3.12+; typing ecosystem/runtime support; **high** conceptually, **medium** for introspection details. Verify `typing` generic classes and PEP 695.

### PY-TYPE-007 — Protocols and runtime-checkable structural typing

- **Distinct behavior:** Protocols describe static structural subtyping independent of inheritance; selected protocols can support shallow runtime instance/subclass checks by member presence.
- **Demonstration:** Include a protocol with methods, properties, class/static members, a generic protocol, an implicit implementer, an explicit implementer, and a runtime-checkable case.
- **Variants and failures:** Runtime checks ignore signatures and may freeze member sets in newer versions; data protocols restrict `issubclass`; unsafe overlap affects narrowing; `hasattr` semantics can involve dynamic attributes.
- **Constraints / authority / confidence / verification:** Python 3.8+ in stdlib, PEP 544; typing specification and `typing` runtime; **high**. Verify PEP 544 and `typing.Protocol` documentation.

### PY-TYPE-008 — Callable types, callback protocols, `ParamSpec`, and `Concatenate`

- **Distinct behavior:** Callable annotations can express positional signatures, ellipsis, overload-like callback protocols, and decorators that preserve or transform parameter lists.
- **Demonstration:** Include a higher-order function, a callable protocol with keyword-sensitive parameters, and a decorator typed with `ParamSpec`/`Concatenate` that adds or removes a leading context argument.
- **Variants and failures:** `Callable[..., R]` intentionally accepts unknown parameters; parameter names and kinds matter for protocols; runtime callable checks do not verify signatures; `ParamSpec.args`/`.kwargs` have constrained valid use.
- **Constraints / authority / confidence / verification:** `ParamSpec` Python 3.10+ stdlib, PEP 612; typing specification; **high**. Verify PEP 612 and callable typing documentation.

### PY-TYPE-009 — Overloads and implementation relationships

- **Distinct behavior:** Consecutive `@overload` declarations describe multiple static call contracts but are replaced at runtime by one implementation; calling an undecorated overload stub is invalid behavior.
- **Demonstration:** Include overloads differentiated by literal, arity, or correlated parameter/return types followed by one implementation that handles all runtime paths.
- **Variants and failures:** Overlapping or unreachable overloads are checker diagnostics; implementation compatibility rules are static; `get_overloads` exposes registered stubs in recent Python; singledispatch is a different runtime mechanism.
- **Constraints / authority / confidence / verification:** Typing ecosystem, PEP 484, with runtime introspection additions in Python 3.11; **high**. Verify `typing.overload` and the typing specification.

### PY-TYPE-010 — `TypedDict` record shapes

- **Distinct behavior:** `TypedDict` gives mappings a static per-key schema while creating no distinct runtime instance type; totality and per-key requiredness affect static construction and access.
- **Demonstration:** Include class and functional forms, inheritance, required and optional keys, read-only keys if the supported version allows, and a runtime plain dictionary instance.
- **Variants and failures:** `isinstance` checks are unsupported; functional syntax handles invalid identifiers; openness/extra items and read-only semantics are evolving in the typing specification; runtime metadata exposes required/optional keys.
- **Constraints / authority / confidence / verification:** Python 3.8+ stdlib; PEP 589, PEP 655, and newer typing-spec additions; **high** for core, **medium** for latest extensions. Verify `typing.TypedDict` and relevant PEPs.

### PY-TYPE-011 — Literal, final, class-variable, and read-only intent

- **Distinct behavior:** `Literal` restricts values, `Final` restricts reassignment/override, and `ClassVar` separates class state from instance fields for static tools and dataclass processing; most are not runtime-enforced.
- **Demonstration:** Include literal-discriminated branches, module/class final names, class variables in a dataclass, and intentional static violations that still have defined runtime behavior where valid.
- **Variants and failures:** `Final` and `ClassVar` nesting rules evolved; enum members can behave as literals; `LiteralString` tracks safe string composition; `ReadOnly` applies to newer typed-dictionary fields.
- **Constraints / authority / confidence / verification:** Typing ecosystem with version-dependent syntax; **high**. Verify `typing` documentation, PEP 586, PEP 591, and PEP 675.

### PY-TYPE-012 — `NewType`, nominal wrappers, and type guards

- **Distinct behavior:** `NewType` creates a static nominal distinction with an identity-like runtime call, while `TypeGuard` and `TypeIs` allow user predicates to narrow types under different consistency rules.
- **Demonstration:** Include a new type passed through its constructor, a type-guard predicate narrowing a heterogeneous value, and a `TypeIs` predicate when available.
- **Variants and failures:** `NewType` values are not runtime wrapper instances; invalid narrowing can make a checker unsound; `TypeGuard` narrows only the positive branch and can permit non-subtype targets; `TypeIs` is newer and narrows both branches under subtype constraints.
- **Constraints / authority / confidence / verification:** `NewType`/`TypeGuard` in modern Python; `TypeIs` Python 3.13 via PEP 742; typing specification; **medium-high**. Verify `typing.NewType`, PEP 647, and PEP 742.

### PY-TYPE-013 — Self types, overrides, and method-generic relationships

- **Distinct behavior:** `Self` ties method results/parameters to the most specific subclass, while `@override` marks an intended base-member relationship for static checking without changing runtime dispatch.
- **Demonstration:** Include fluent instance and class methods returning `Self`, subclass specialization, valid override, misspelled/nonexistent override diagnostic, and runtime inspection of decorator metadata where available.
- **Variants and failures:** `Self` is not equivalent to the enclosing class in all positions; explicit self-type type variables remain useful; `@override` can fail to attach metadata to some descriptors; decorators affect correct ordering.
- **Constraints / authority / confidence / verification:** `Self` Python 3.11 (PEP 673); `override` Python 3.12 (PEP 698); typing ecosystem; **high**. Verify PEP 673, PEP 698, and `typing` documentation.

### PY-TYPE-014 — Variadic generics and unpacked type tuples

- **Distinct behavior:** `TypeVarTuple` and type-level unpacking express an arbitrary sequence of type arguments, enabling shape- or tuple-sensitive generic APIs.
- **Demonstration:** Include a variadic generic class or function, tuple annotations with unpacked types, and a specialization with zero, one, and multiple elements.
- **Variants and failures:** Only one type-variable tuple is permitted in a parameter list in core rules; unpacking syntax differs across versions; interactions with ordinary type variables and defaults have ordering constraints.
- **Constraints / authority / confidence / verification:** Python 3.11+ stdlib, PEP 646; typing specification; **medium-high**. Verify PEP 646 and `typing.TypeVarTuple`.

### PY-TYPE-015 — Type-checker directives and deliberate static diagnostics

- **Distinct behavior:** Comments and special forms such as `type: ignore`, error-code-qualified ignores, `cast`, `assert_type`, `reveal_type`, `no_type_check`, and unreachable assertions alter diagnostics or expose inferred types without necessarily changing runtime values.
- **Demonstration:** Include valid narrowing/inference probes, one targeted suppression, an incorrect assertion as a negative case, and runtime-safe handling of checker-only helpers.
- **Variants and failures:** Error-code syntax is checker-specific; `reveal_type` has a runtime implementation in newer Python but traditionally was checker-only; broad ignores can mask unrelated errors; reachability depends on checker configuration.
- **Constraints / authority / confidence / verification:** Typing specification plus tool conventions; **high** for standard forms, **medium** across checker-specific comments. Verify `typing` documentation and chosen checker manuals.

### PY-TYPE-016 — Stub files, inline typing, and typed-distribution markers

- **Distinct behavior:** `.pyi` stubs can replace or augment the statically visible interface of runtime modules, while a `py.typed` marker declares that a distribution supports inline or partial typing.
- **Demonstration:** Provide a runtime module with a corresponding stub whose public surface includes overloads or precise types, plus package data marking inline or partial typing as appropriate.
- **Variants and failures:** Stub and runtime surfaces can drift; stub-only packages have naming conventions; partial stub packages merge with runtime sources; source annotations typically lose to a co-located stub for static analysis.
- **Constraints / authority / confidence / verification:** Typing ecosystem, PEP 484 and PEP 561; **high**. Verify PEP 561, “Distributing and Packaging Type Information,” and typing specification.

### PY-TYPE-017 — Checker configuration and import resolution modes

- **Distinct behavior:** Strictness, target Python version/platform, missing-import policy, namespace-package rules, plugin use, and per-module overrides materially change inferred types and diagnostics.
- **Demonstration:** Include explicit static-tool configuration with at least one scoped override and source that differs under target version/platform or strictness.
- **Variants and failures:** Different checkers implement extensions and inference differently; configuration can live in `pyproject.toml` or tool-specific files; an installed dependency, source checkout, and stub package may resolve differently.
- **Constraints / authority / confidence / verification:** Ecosystem/tool-defined; **high** that variation exists, **medium** on cross-tool equivalence. Verify the selected type checker’s official configuration and import-resolution documentation.

### PY-TYPE-018 — Dataclass-like transforms and framework-generated members

- **Distinct behavior:** `dataclass_transform` tells static tools that a decorator, base class, or metaclass synthesizes dataclass-like constructors and field semantics even when runtime generation is framework-specific.
- **Demonstration:** Include a small transform-decorated factory/base and resulting class with field specifiers, generated initializer expectations, and runtime behavior matching the advertised transform.
- **Variants and failures:** Static defaults can be overridden by decorators; field specifier parameters affect init/default/alias behavior; runtime and advertised semantics can drift; individual checkers may differ at edges.
- **Constraints / authority / confidence / verification:** Python 3.11+, PEP 681; typing ecosystem; **high**. Verify PEP 681 and `typing.dataclass_transform`.

### PY-TYPE-019 — Annotated metadata and runtime consumers

- **Distinct behavior:** `Annotated` attaches ordered metadata to an underlying static type, allowing frameworks to interpret validators, dependencies, serialization aliases, or documentation without changing the base assignability rule.
- **Demonstration:** Include nested/repeated annotated metadata, retrieve hints with extras preserved, and pass the same declaration through a small runtime consumer.
- **Variants and failures:** Default hint resolution strips extras unless requested; alias boundaries affect metadata flattening; metadata equality is order-sensitive; consumers can assign incompatible meanings to the same object.
- **Constraints / authority / confidence / verification:** Python 3.9+ stdlib, PEP 593; typing ecosystem/runtime API; **high**. Verify PEP 593 and `typing.Annotated`.

### PY-TYPE-020 — Narrowing by control flow and exhaustiveness

- **Distinct behavior:** Static analyzers narrow unions through `is None`, `isinstance`, discriminant literals, assertions, user predicates, and pattern matching, then may diagnose unreachable or non-exhaustive branches.
- **Demonstration:** Include one discriminated union processed by conditions and patterns, an assertion-based narrow, and an explicit never-reached assertion for exhaustive handling.
- **Variants and failures:** Exhaustiveness is checker-dependent; mutation and aliasing can invalidate narrowing; negative `isinstance` narrowing depends on class relationships; `Any` and dynamic attributes weaken conclusions.
- **Constraints / authority / confidence / verification:** Typing specification and tool behavior; **medium-high**. Verify typing narrowing guidance, `assert_never`, and selected checker documentation.

## 7. Control flow, exceptions, and resource lifetime

### PY-FLOW-001 — Conditional, loop, and `else` control flow

- **Distinct behavior:** `if` selects by truth testing, while `for`/`while` `else` suites run only on normal loop exhaustion, not after `break`.
- **Demonstration:** Include loops ending by exhaustion, `break`, `continue`, `return`, and exception, with nested loops to disambiguate which `else` belongs to which loop.
- **Variants and failures:** `continue` still executes enclosing `finally`; iterator exceptions other than `StopIteration` abort; static reachability may differ for constant conditions.
- **Constraints / authority / confidence / verification:** Python 3; language; **high**. Verify compound statements in the Python Language Reference.

### PY-FLOW-002 — `try`, `except`, `else`, and `finally` precedence

- **Distinct behavior:** Handlers match exceptions by class, `else` runs only when the try suite completes without exception, and `finally` always runs and can replace pending return values or exceptions.
- **Demonstration:** Include normal completion, matched/unmatched exceptions, handler-raised exceptions, return through `finally`, and a deliberate `finally` return/raise overriding prior control flow.
- **Variants and failures:** Bare `except` catches `BaseException` subclasses including exits/interrupts; handler order matters; control flow in `finally` can hide failures and is increasingly diagnosed; exception target cleanup is special.
- **Constraints / authority / confidence / verification:** Python 3; language; **high**. Verify the `try` statement.

### PY-FLOW-003 — Exception chaining, context, cause, and traceback

- **Distinct behavior:** Raising while handling another exception sets implicit context; `raise ... from ...` sets an explicit cause or suppresses displayed context; bare `raise` re-raises with traceback state.
- **Demonstration:** Include implicit chaining, explicit cause, `from None`, bare re-raise, and traceback inspection without relying on exact line formatting.
- **Variants and failures:** Modifying traceback via `with_traceback` changes presentation; re-raising outside a handler fails; Python 3.11 changed behavior after traceback mutation in handlers; custom exceptions can carry notes.
- **Constraints / authority / confidence / verification:** Python 3; language/data model, PEP 3134 and PEP 409; **high**. Verify exception context and the `raise` statement.

### PY-FLOW-004 — Custom exception hierarchies, arguments, and notes

- **Distinct behavior:** User exceptions participate in class-based matching, can carry structured attributes, inherit common behavior, and accept diagnostic notes separate from the main message.
- **Demonstration:** Define a small hierarchy rooted in `Exception`, raise/catch at specific and base levels, preserve structured fields, and attach a note.
- **Variants and failures:** Multiple inheritance among built-in exceptions is discouraged because layouts can conflict; subclassing `BaseException` bypasses common catches; constructors and pickling must agree for transported exceptions.
- **Constraints / authority / confidence / verification:** Python 3; language/CPython standard exception model; notes added in 3.11 via PEP 678; **high**. Verify built-in exceptions and PEP 678.

### PY-FLOW-005 — Exception groups and `except*`

- **Distinct behavior:** Exception groups carry multiple failures in a tree; `except*` splits matching subgroups so multiple clauses can handle disjoint portions and unhandled/raised portions merge afterward.
- **Demonstration:** Raise a nested exception group with multiple exception types, handle subgroups in separate `except*` clauses, and leave or raise a residual subgroup.
- **Variants and failures:** Ordinary `except` cannot mix with `except*` in one try; `return`, `break`, and `continue` are forbidden in `except*`; group subclasses customize derivation; control-flow frameworks may originate groups concurrently.
- **Constraints / authority / confidence / verification:** Python 3.11+, language, PEP 654; **high**. Verify PEP 654, “Exception Groups and `except*`.”

### PY-FLOW-006 — Assertions and optimization-dependent removal

- **Distinct behavior:** `assert` condition and message evaluation can be removed when optimization is requested, so assertions are not reliable runtime validation.
- **Demonstration:** Include an assertion whose condition/message has observable evaluation and exercise source under normal and optimized interpreter modes.
- **Variants and failures:** `__debug__` is compile-time constant and false under optimization; assigning to it is invalid; static tools may use assertions for narrowing even if runtime can remove them.
- **Constraints / authority / confidence / verification:** Python 3; language and CPython launcher/compile optimization; **high**. Verify simple statements and command-line `-O`/`-OO` documentation.

### PY-FLOW-007 — Iteration mutation and deterministic order guarantees

- **Distinct behavior:** Dictionaries preserve insertion order as a language guarantee, sets do not promise a stable order, and mutation during iteration has container-specific semantics and diagnostics.
- **Demonstration:** Exercise ordered dictionary insertion/replacement/deletion/reinsertion, set traversal without expected order, list mutation behavior, and dictionary-size mutation failure.
- **Variants and failures:** Hash randomization changes set and some collision-derived behavior across processes; alternative implementations must preserve dict order but not CPython layout; same-size dict value replacement is permitted.
- **Constraints / authority / confidence / verification:** Dict order guaranteed Python 3.7+; language for dict order, implementation/container-specific elsewhere; **high**. Verify mapping and set type documentation.

### PY-FLOW-008 — Resource cleanup, destructors, and cyclic garbage collection

- **Distinct behavior:** Deterministic cleanup should use context management; `__del__`, reference counting, cyclic collection, weak finalizers, and interpreter shutdown have distinct lifetime behavior.
- **Demonstration:** Include an object in a reference cycle with a finalizer, explicit resource management through `with`, and observation that deletion of a name is not a portable destruction trigger.
- **Variants and failures:** CPython often finalizes promptly by reference count while other implementations do not; finalizers can resurrect objects; shutdown globals may be cleared; uncollectable cases have changed since PEP 442.
- **Constraints / authority / confidence / verification:** Language plus CPython garbage collector; **high** for nondeterminism, **medium** for exact ordering. Verify data-model `__del__`, `gc`, `weakref`, and PEP 442.

### PY-FLOW-009 — Threading, atomicity assumptions, and synchronization

- **Distinct behavior:** Threads share objects and module state; language-level race safety requires synchronization, while CPython’s Global Interpreter Lock (GIL) and operation-level behavior are implementation details rather than a general data-race contract.
- **Demonstration:** Include shared mutable state updated with and without a lock, thread-local state, exception propagation/reporting, and coordination that avoids timing-dependent pass criteria.
- **Variants and failures:** CPython 3.13 offers an experimental free-threaded build; C extensions may release the GIL; seemingly atomic compound operations race; alternative implementations differ; daemon threads may be abandoned at shutdown.
- **Constraints / authority / confidence / verification:** Standard library plus CPython implementation, materially build-dependent in 3.13; **high**. Verify `threading`, CPython free-threading documentation, and PEP 703.

### PY-FLOW-010 — Multiprocessing start methods and importability

- **Distinct behavior:** Process creation by fork, spawn, or forkserver changes state inheritance, startup import behavior, callable picklability, and the need for top-level startup guards.
- **Demonstration:** Run a top-level worker through at least spawn and one available alternative, passing serializable data and avoiding accidental recursive startup.
- **Variants and failures:** Platform defaults differ and are changing across Python releases; local functions/lambdas fail under spawn; fork with threads is hazardous; frozen executables need support hooks; queues and pools transport exceptions imperfectly.
- **Constraints / authority / confidence / verification:** CPython standard library and operating-system dependent; **high**. Verify `multiprocessing` contexts and start methods.

### PY-FLOW-011 — Async task groups, cancellation, and grouped failures

- **Distinct behavior:** Structured asynchronous concurrency ties child-task lifetime to a lexical scope, propagates cancellation, and reports multiple failures as exception groups.
- **Demonstration:** Include sibling tasks where one fails, another is cancelled and cleans up, and the surrounding task group raises a handled exception group.
- **Variants and failures:** Swallowing cancellation can break structured-concurrency components; cancellation counts/behavior changed in recent maintenance releases; creating tasks outside an active group fails or closes coroutines depending on version.
- **Constraints / authority / confidence / verification:** `asyncio.TaskGroup` Python 3.11+; standard library built atop language exception groups; **medium-high** because details evolve. Verify `asyncio` task and task-group documentation.

### PY-FLOW-012 — Signals, interrupts, and asynchronous exceptions

- **Distinct behavior:** Signal handlers run according to interpreter/main-thread rules and may cause exceptions such as `KeyboardInterrupt` between apparently adjacent operations.
- **Demonstration:** Install and restore a benign signal handler on supported platforms, record main-thread execution, and keep platform guards for unsupported signal names/APIs.
- **Variants and failures:** Windows and POSIX signal sets differ; handlers are main-thread-only; blocking native code delays Python handler execution; wakeup descriptors and subprocess signals are platform-specific.
- **Constraints / authority / confidence / verification:** CPython standard library and platform; **medium-high**. Verify `signal` documentation.

## 8. Source text, expressions, literals, and compilation modes

### PY-SRC-001 — Source encoding, Unicode identifiers, and normalization

- **Distinct behavior:** Python source defaults to UTF-8, can declare an encoding, and accepts many Unicode identifier characters after normalization; identifier spelling and string contents do not share identical normalization rules.
- **Demonstration:** Include non-ASCII identifiers and text, an explicit valid encoding declaration in a compatible file, and two visually or canonically confusable cases kept distinguishable and documented.
- **Variants and failures:** A byte-order mark is compatible only with UTF-8; invalid encoding declarations or undecodable bytes are compile errors; identifiers use Normalization Form KC (NFKC); tools and filesystems can normalize filenames differently.
- **Constraints / authority / confidence / verification:** Python 3; language lexical rules, PEP 3120 and PEP 3131; **high**. Verify lexical analysis in the Python Language Reference.

### PY-SRC-002 — Physical/logical lines, indentation, and tabs

- **Distinct behavior:** Indentation tokens define suites after explicit/implicit line joining, and mixed tabs/spaces can be ambiguous or invalid under tab expansion rules.
- **Demonstration:** Include nested suites, implicit continuation inside delimiters, explicit line continuation in a controlled case, comments/blank lines, and separate invalid indentation/tab fixtures.
- **Variants and failures:** Backslash continuation cannot carry a trailing comment and interacts with string lexing; formfeed has special leading-indent handling; inconsistent dedentation and ambiguous tab use raise different compile diagnostics.
- **Constraints / authority / confidence / verification:** Python 3; language lexical rules; **high**. Verify lexical analysis in the Python Language Reference.

### PY-SRC-003 — Future statements and compile-unit feature flags

- **Distinct behavior:** A `__future__` statement changes compiler semantics for its containing module and must appear near the beginning; features have optional and mandatory release metadata.
- **Demonstration:** Include a module using postponed annotations and a dynamic `compile` call with explicit future-flag inheritance or suppression.
- **Variants and failures:** Misplaced future imports are syntax errors; imported modules do not inherit caller flags; interactive compilation differs; obsolete future features remain accepted for compatibility.
- **Constraints / authority / confidence / verification:** Python 3; language and `__future__` module, PEP 236; **high**. Verify future statements and `__future__` documentation.

### PY-SRC-004 — Literal concatenation, prefixes, escapes, raw strings, and bytes

- **Distinct behavior:** Adjacent literals concatenate at compile time; prefixes select text, bytes, raw, formatted, and combinations; escape handling and permitted contents differ.
- **Demonstration:** Include adjacent string and bytes literals, raw/path-like text, escaped Unicode text, invalid-escape diagnostic, and bytes restricted to ASCII source characters except escapes.
- **Variants and failures:** Raw literals still cannot end with an odd backslash and still process quote delimiters; text and bytes cannot concatenate; unrecognized escapes have progressed from warnings toward errors across versions; prefix combinations are constrained.
- **Constraints / authority / confidence / verification:** Python 3; language lexical rules; **high**. Verify string and bytes literals in lexical analysis.

### PY-SRC-005 — Formatted string literals and nested expression parsing

- **Distinct behavior:** F-strings interleave literal text, evaluated expressions, conversions, nested format specifications, self-documenting expressions, and evaluation order.
- **Demonstration:** Include conversion flags, dynamic nested format specs, debug `=`, escaped braces, multiline/nested expressions allowed by the baseline grammar, and a custom `__format__` target.
- **Variants and failures:** Python 3.12’s PEP 701 makes expression parsing much more regular and allows constructs rejected earlier; nesting has parser/token limits; malformed braces/conversions are syntax errors; expressions execute at runtime.
- **Constraints / authority / confidence / verification:** Python 3.6+, materially changed in 3.12 by PEP 701; language; **high**. Verify formatted string literals and PEP 701.

### PY-SRC-006 — Numeric literal and arithmetic edge semantics

- **Distinct behavior:** Integer literals have arbitrary precision and multiple bases; floating and complex literals follow host numeric representations; floor division, modulo, exponentiation, and negative operands have Python-specific relationships and precedence.
- **Demonstration:** Include large integers, digit separators, base prefixes, float infinities/NaNs produced at runtime, complex arithmetic, negative floor/modulo, and exponentiation versus unary minus.
- **Variants and failures:** Leading-zero decimal integers are invalid; integer string conversion has a configurable digit limit in modern CPython; floating results depend on platform C semantics at edges; decimal and fraction types use separate libraries.
- **Constraints / authority / confidence / verification:** Language plus CPython security limit; **high**. Verify numeric literals, arithmetic conversions, built-in numeric types, and `sys.set_int_max_str_digits`.

### PY-SRC-007 — Collection displays, unpacking, and duplicate handling

- **Distinct behavior:** List/set/dict displays evaluate components in order; starred elements unpack iterables; double-starred mappings merge with later duplicate keys winning in displays.
- **Demonstration:** Include all display types, multiple unpackings with duplicate dictionary keys, unhashable set/dict-key failure, and expression side effects exposing evaluation order.
- **Variants and failures:** Dictionary comprehensions have key-before-value evaluation since modern Python; call keyword duplicates fail instead of overwriting; set display order is not guaranteed; a single empty brace is a dict.
- **Constraints / authority / confidence / verification:** Generalized unpacking Python 3.5+; language, PEP 448; **high**. Verify displays and PEP 448.

### PY-SRC-008 — Assignment unpacking and starred targets

- **Distinct behavior:** Iterable unpacking assigns by position, permits one starred target, and can recursively destructure targets; assignment evaluates the right side before target writes, with specified target order.
- **Demonstration:** Include nested unpacking, starred capture, chained assignment identity, overlapping item/attribute targets, and too-few/too-many failure cases.
- **Variants and failures:** Starred targets collect a new list; assignment to multiple targets can observe earlier writes; starred expressions are broader in expression lists in newer Python; pattern matching resembles but does not use assignment semantics.
- **Constraints / authority / confidence / verification:** Python 3; language; **high**. Verify assignment statements and expression lists.

### PY-SRC-009 — Assignment expressions and binding position

- **Distinct behavior:** The walrus operator binds a name while yielding a value, with intentionally low precedence and special scoping in comprehensions.
- **Demonstration:** Use assignment expressions in a conditional loop, a larger expression requiring parentheses, and a comprehension where a result is bound in the containing scope.
- **Variants and failures:** Only simple names are targets; several comprehension placements and class-scope uses are invalid; binding in a comprehension can conflict with iteration variables; before Python 3.8 syntax is invalid.
- **Constraints / authority / confidence / verification:** Python 3.8+, language, PEP 572; **high**. Verify assignment expressions and PEP 572.

### PY-SRC-010 — Expression evaluation order and chained access/calls

- **Distinct behavior:** Python generally evaluates expressions left to right, while assignment targets, short-circuit operators, comparisons, subscriptions, attributes, and calls impose distinct lookup and timing boundaries.
- **Demonstration:** Use small side-effect-recording operands to expose order in arithmetic, calls, displays, attribute/subscript assignment, chained comparison, and conditional expressions.
- **Variants and failures:** Operator precedence affects grouping, not operand evaluation order; augmented assignment evaluates its target once; exceptions stop remaining evaluations; optimizer transformations must preserve observable semantics.
- **Constraints / authority / confidence / verification:** Python 3; language; **high**. Verify evaluation order and operator precedence in the Python Language Reference.

### PY-SRC-011 — Compile, abstract syntax trees, and optimization

- **Distinct behavior:** Source can compile in execution, evaluation, or single-statement modes into code objects or abstract syntax trees (ASTs), with flags controlling future features, optimization, and top-level await.
- **Demonstration:** Parse and compile a short generated expression/module, transform or inspect its AST, fix locations, execute with explicit namespaces, and compile a deliberate syntax error.
- **Variants and failures:** AST node shapes change between versions; malformed manually built trees fail validation; optimizer levels can remove assertions/docstrings; exact bytecode and constants are CPython implementation details.
- **Constraints / authority / confidence / verification:** Python 3; standard library/CPython compiler APIs; **high**. Verify `ast`, `compile`, and built-in code-object documentation.

### PY-SRC-012 — Top-level await and interactive compilation

- **Distinct behavior:** Interactive shells and compiler APIs can permit top-level asynchronous constructs that are invalid in ordinary module compilation, returning a coroutine-like code result for a host to run.
- **Demonstration:** Compile a top-level await expression using the dedicated compiler flag and contrast it with ordinary module compilation failure.
- **Variants and failures:** Notebook and shell behavior is host-specific; the source is not generally importable as a normal module; auto-await integration and event-loop ownership differ across environments.
- **Constraints / authority / confidence / verification:** CPython 3.8+ compiler flag plus ecosystem shells; **medium-high**. Verify `ast.PyCF_ALLOW_TOP_LEVEL_AWAIT`, `compile`, and host documentation.

### PY-SRC-013 — Docstrings and optimization

- **Distinct behavior:** A leading string expression becomes a module, class, function, or method docstring and is stored on the object, while higher optimization can remove it.
- **Demonstration:** Include docstrings at several declaration levels, attribute them through introspection, and compare compilation under normal and `-OO` optimization.
- **Variants and failures:** Additional string expressions can be recognized by documentation tools but are not standard `__doc__`; decorators can replace/preserve docstrings; indentation cleaning is an inspection convention.
- **Constraints / authority / confidence / verification:** Language plus CPython optimization/tool convention; **high**. Verify string literal/docstring rules, `inspect.getdoc`, and command-line optimization options.

### PY-SRC-014 — Soft keywords and grammar-version dependence

- **Distinct behavior:** Tokens such as `match`, `case`, `_`, and `type` act as keywords only in specific grammar contexts, remaining valid identifiers elsewhere.
- **Demonstration:** Use each as an ordinary identifier where permitted and as syntax in its contextual form, with version-gated negative parsing cases.
- **Variants and failures:** `match`/`case` begin in 3.10, `_` is wildcard only in patterns, and `type` is soft for the 3.12 type statement; parser target version controls acceptance.
- **Constraints / authority / confidence / verification:** Python 3.10+/3.12+; language lexical/grammar rules; **high**. Verify lexical analysis and compound/simple statements.

### PY-SRC-015 — Syntactically valid but context-invalid constructs

- **Distinct behavior:** Some token/grammar forms compile only in permitted semantic contexts: `return`, `yield`, `await`, `break`, `continue`, `nonlocal`, starred targets, and asynchronous comprehensions are context-checked by the compiler.
- **Demonstration:** Maintain isolated negative source cases for each context restriction, alongside a nearest valid enclosing-context example.
- **Variants and failures:** Diagnostics and exact source spans improve across CPython releases; some restrictions are grammar errors while others arise in symbol-table/compiler passes; dynamic compilation exposes the phase.
- **Constraints / authority / confidence / verification:** Python 3; language with implementation-defined diagnostic wording; **high**. Verify relevant simple/compound statement references.

### PY-SRC-016 — Template string literals

- **Distinct behavior:** Python 3.14 template string literals are expected to produce structured template/interpolation objects for later processing rather than immediately formatting to text like f-strings.
- **Demonstration:** In a version-gated source unit, include literal and interpolated portions, conversion/format metadata, and a consumer that inspects structure before rendering.
- **Variants and failures:** Syntax is invalid before 3.14; exact runtime types, concatenation constraints, and interpolation semantics require current-version verification; applications can interpret the same template data differently.
- **Constraints / authority / confidence / verification:** Python 3.14+, language via PEP 750; **medium** because baseline is 3.13 and no live verification was performed. Verify PEP 750, “Template Strings,” and Python 3.14 language/standard-library documentation.

## 9. Runtime dispatch frameworks and dynamic ecosystem conventions

### PY-DYN-001 — Single-dispatch generic functions

- **Distinct behavior:** `functools.singledispatch` chooses an implementation from the runtime type of the first argument using registered classes and their MRO, separate from static overload declarations.
- **Demonstration:** Include default, explicit, annotation-inferred, abstract-base, union, and subclass registrations, then inspect the registry and chosen implementation.
- **Variants and failures:** Dispatch ignores later arguments; ambiguous abstract-base relationships use defined resolution/error behavior; registration can return the undecorated implementation; union registration is supported only in newer Python.
- **Constraints / authority / confidence / verification:** Python 3.4+, evolving standard library; **high**. Verify `functools.singledispatch` documentation and PEP 443.

### PY-DYN-002 — Single-dispatch methods and descriptor ordering

- **Distinct behavior:** `singledispatchmethod` dispatches on the first non-`self`/non-`cls` argument and must cooperate with descriptor decorators and registration attributes.
- **Demonstration:** Include instance/class method variants with registrations and place the dispatch decorator in the required order relative to other method decorators.
- **Variants and failures:** Wrong decorator order hides `.register`; staticmethod/classmethod combinations bind differently; inheritance and later registration can change all instances’ dispatch.
- **Constraints / authority / confidence / verification:** Python 3.8+; standard library; **high**. Verify `functools.singledispatchmethod` documentation.

### PY-DYN-003 — Plugin discovery through distribution entry points

- **Distinct behavior:** Installed distributions can advertise importable objects under named entry-point groups, and consumers discover/load them through package metadata rather than direct source imports.
- **Demonstration:** Declare a private entry-point group in project metadata with more than one loadable target, then enumerate/select/load it through `importlib.metadata` in an installed environment.
- **Variants and failures:** Entry points exist only after metadata installation/build integration; editable installs vary; duplicate names across distributions require consumer policy; load strings may point to missing modules/attributes; API selection changed across Python versions.
- **Constraints / authority / confidence / verification:** Packaging ecosystem standardized by PyPA specifications and `importlib.metadata`; **high**. Verify the PyPA Entry Points specification and `importlib.metadata` documentation.

### PY-DYN-004 — Console and graphical script wrappers

- **Distinct behavior:** Script entry points cause installers to generate platform-specific launchers that import and call a declared object, so the executable artifact is generated rather than project source.
- **Demonstration:** Declare one console script, build/install the distribution, and invoke the generated wrapper with a callable returning a process status.
- **Variants and failures:** Windows launchers differ from POSIX scripts; direct invocation versus `python -m` can set `sys.argv[0]` differently; malformed targets fail after installation; graphical scripts have platform-specific console behavior.
- **Constraints / authority / confidence / verification:** Packaging ecosystem; **high**. Verify the PyPA Entry Points specification and build-backend documentation.

### PY-DYN-005 — Runtime metadata, decorators, and annotations consumed by frameworks

- **Distinct behavior:** Frameworks commonly discover handlers, routes, fields, tests, or commands by inspecting decorators, class inheritance, naming, annotations, or module globals, creating semantic relationships absent from direct calls.
- **Demonstration:** Include one small self-contained registry/decorator convention that records decorated objects and later invokes or inspects them without hard-coded direct references.
- **Variants and failures:** Import side effects may be required to populate registries; decorator ordering changes registered objects; discovery by filesystem/name is tool-specific; annotation evaluation policy can break consumers.
- **Constraints / authority / confidence / verification:** Ecosystem convention; **high** for the pattern, source depends on selected framework or local registry. Verify the official documentation for any chosen framework; otherwise the project’s registry contract is primary.

### PY-DYN-006 — Dependency injection by annotations or signatures

- **Distinct behavior:** Runtime frameworks may inspect signatures, defaults, and annotated metadata to construct arguments and choose providers, turning metadata into call edges.
- **Demonstration:** Include a minimal local injector or common framework feature where one callable’s annotated/defaulted parameter is resolved from a registered provider.
- **Variants and failures:** Decorators that do not preserve signatures break discovery; postponed annotations require resolution namespaces; qualifiers may use `Annotated`; providers can be scoped, lazy, cyclic, or ambiguous.
- **Constraints / authority / confidence / verification:** Ecosystem convention built on `inspect`/`typing`; **medium-high**. Verify the chosen framework’s official dependency-injection documentation or the local contract.

### PY-DYN-007 — Proxies, wrappers, and transparent delegation

- **Distinct behavior:** Proxy objects can forward attribute access, calls, iteration, context management, or operators to a wrapped target, but Python’s special-method lookup prevents complete transparency from `__getattr__` alone.
- **Demonstration:** Include a proxy forwarding ordinary attributes and explicitly implementing at least one special protocol, plus wrapped metadata on a callable wrapper.
- **Variants and failures:** `isinstance`, identity, pickling, weak references, and descriptor binding expose proxy identity; special methods need type-level forwarding; `functools.update_wrapper` affects introspection only.
- **Constraints / authority / confidence / verification:** Language data model plus ecosystem pattern; **high**. Verify special-method lookup and `functools` wrapper documentation.

### PY-DYN-008 — Lazy imports and lazy values

- **Distinct behavior:** Applications and libraries defer import or object construction through module attribute hooks, proxies, cached descriptors, or lazy loaders, making dependency/use timing distinct from declaration.
- **Demonstration:** Include a lazy module export and a cached per-instance computed attribute, showing first-access creation and subsequent cached identity.
- **Variants and failures:** Failure moves from import time to use time; circularity can improve or worsen; thread safety differs; `importlib.util.LazyLoader` has loader/class mutation constraints; static declarations may live in stubs.
- **Constraints / authority / confidence / verification:** Standard library plus ecosystem convention; **medium-high**. Verify `importlib.util.LazyLoader`, PEP 562, and `functools.cached_property` documentation.

## 10. Distribution, dependency, and build semantics

### PY-PKG-001 — Import package versus distribution identity

- **Distinct behavior:** Installed distribution names, normalized project names, import package names, and top-level modules are separate namespaces with many-to-many relationships.
- **Demonstration:** Declare project metadata whose normalized distribution name differs from its import package spelling, and expose more than one importable top-level package or module from one distribution.
- **Variants and failures:** Hyphens/underscores/dots normalize for distribution comparison but not imports; multiple distributions can contribute to a namespace package; import-to-distribution mapping is heuristic metadata, not a language rule.
- **Constraints / authority / confidence / verification:** Packaging ecosystem; **high**. Verify the PyPA Names and normalization specification and `importlib.metadata.packages_distributions` documentation.

### PY-PKG-002 — `pyproject.toml` build-system isolation

- **Distinct behavior:** The `[build-system]` table selects a build backend and declares build requirements installed in an isolated environment before project metadata or artifacts are built.
- **Demonstration:** Provide valid build-system metadata with a constrained backend requirement, then build through a standards-compatible frontend from an environment lacking that backend globally.
- **Variants and failures:** Missing/invalid backend, unsatisfiable requirements, offline resolution, self-hosting backends, and `backend-path` change outcomes; frontends can offer non-isolated escape hatches.
- **Constraints / authority / confidence / verification:** Packaging standards, PEP 517 and PEP 518; **high**. Verify PEP 517, PEP 518, and the PyPA Build System Interface specification.

### PY-PKG-003 — Static and dynamic core metadata

- **Distinct behavior:** Project metadata can be declared statically under `[project]` or marked dynamic for the backend to compute; fields control identity, compatibility, dependencies, entry points, and published metadata.
- **Demonstration:** Declare most fields statically and one legitimately dynamic field whose value is derived by the backend, then inspect built artifact metadata.
- **Variants and failures:** A field cannot be both statically specified and dynamic; backend support for dynamic sources varies; metadata versions gate fields; readme/content-type and license expression formats have evolved.
- **Constraints / authority / confidence / verification:** Packaging ecosystem, PEP 621 and Core Metadata specifications; **high**. Verify PEP 621 and PyPA Core Metadata.

### PY-PKG-004 — Source distribution and wheel content divergence

- **Distinct behavior:** Source distributions (sdists) contain build inputs, while wheels contain installed files; their file sets, metadata placement, and build-time generation can differ materially.
- **Demonstration:** Build both formats, with one build input present only in the sdist and one generated runtime artifact present in the wheel, then inspect manifests without prescribing exact paths.
- **Variants and failures:** Building a wheel from a source tree versus from its sdist can reveal missing inputs; backend defaults differ; reproducibility depends on timestamps/order/environment; pure and platform wheels have distinct tags.
- **Constraints / authority / confidence / verification:** Packaging ecosystem, PEP 517 and binary/source distribution specifications; **high**. Verify PyPA Source Distribution Format and Binary Distribution Format specifications.

### PY-PKG-005 — Package discovery and source-root mapping

- **Distinct behavior:** Build backends discover packages/modules according to explicit configuration or conventions; import layout in a checkout can differ from installed layout.
- **Demonstration:** Configure package discovery across a nontrivial source root, include/exclude patterns, and at least one standalone module or namespace portion as supported by the backend.
- **Variants and failures:** Flat and `src` layouts expose different accidental-import risks; implicit namespaces may need a discovery flag; data/test directories can be misclassified; backend behavior is not standardized by PEP 621.
- **Constraints / authority / confidence / verification:** Build-backend ecosystem convention; **high**. Verify the chosen backend’s official package-discovery documentation.

### PY-PKG-006 — Runtime dependencies and environment markers

- **Distinct behavior:** Dependency declarations combine normalized names, version specifiers, extras, direct references, and environment markers evaluated for the target environment.
- **Demonstration:** Declare a baseline dependency, a Python-version-gated dependency, and an operating-system- or implementation-gated dependency with non-overlapping marker intent.
- **Variants and failures:** Marker variables describe the install environment; quoting and Boolean precedence matter; direct URLs affect reproducibility/security; invalid or contradictory constraints can make resolution fail.
- **Constraints / authority / confidence / verification:** Packaging ecosystem, PEP 508 and dependency-specifier specifications; **high**. Verify PEP 508 and PyPA Dependency Specifiers.

### PY-PKG-007 — Optional dependencies and extras

- **Distinct behavior:** Extras name optional dependency sets selected by consumers and can also appear on dependency edges, so an installed feature surface depends on requested extras.
- **Demonstration:** Declare at least two extras with one shared and one marker-guarded dependency, and include code that handles the optional import explicitly.
- **Variants and failures:** Extra names normalize; self-referential extras may aggregate features with backend/frontend support; installed metadata records `Provides-Extra`; absence must not be confused with a broken mandatory dependency.
- **Constraints / authority / confidence / verification:** Packaging ecosystem; **high**. Verify PEP 508, PEP 621, and Core Metadata `Provides-Extra`/`Requires-Dist`.

### PY-PKG-008 — Version specifiers, prereleases, and local versions

- **Distinct behavior:** Public versions have defined normalization and ordering, while specifiers choose compatible candidates with special prerelease and local-version rules.
- **Demonstration:** Use a valid normalized project version and dependencies exercising compatible-release, exclusion, and bounded ranges; separately validate representative prerelease/post/dev/local versions.
- **Variants and failures:** Arbitrary equality is discouraged; local identifiers are prohibited or ignored in some public-index/specifier contexts; prerelease selection depends on available candidates and frontend policy; epochs alter order.
- **Constraints / authority / confidence / verification:** Packaging ecosystem, PEP 440; **high**. Verify PEP 440 and PyPA Version Specifiers.

### PY-PKG-009 — `Requires-Python` and syntax compatibility

- **Distinct behavior:** Distribution metadata can constrain interpreter versions independently from dependency markers, while source syntax may impose an even stronger unexpressed minimum if metadata is wrong.
- **Demonstration:** Declare an accurate Python range, include a guarded dependency marker at a boundary, and test metadata/syntax under at least one accepted and one rejected interpreter version.
- **Variants and failures:** Installers may use `Requires-Python` before downloading/building; sdists can contain backend code with a different minimum; universal-looking wheel tags do not override metadata; alternative implementations also report Python language versions.
- **Constraints / authority / confidence / verification:** Packaging ecosystem plus language grammar; **high**. Verify Core Metadata `Requires-Python` and wheel compatibility tags.

### PY-PKG-010 — Wheel compatibility tags and platform specificity

- **Distinct behavior:** Wheel filenames encode interpreter, ABI, and platform compatibility, which determines whether installers consider an artifact usable.
- **Demonstration:** Build a pure wheel and, if native code is included, a platform wheel; inspect their tags and internal `WHEEL` metadata.
- **Variants and failures:** `py3-none-any`, CPython ABI tags, stable ABI tags, manylinux/musllinux/macOS/Windows tags, and free-threaded tags represent distinct compatibility sets; retagging without true compatibility is invalid.
- **Constraints / authority / confidence / verification:** Packaging ecosystem and platform policy; **high**. Verify the PyPA Platform Compatibility Tags and Binary Distribution Format specifications.

### PY-PKG-011 — Editable installation protocol

- **Distinct behavior:** Editable builds expose a working source tree through an installed wheel/proxy/path mechanism while still installing metadata and generated entry points; behavior need not match a regular wheel in file provenance.
- **Demonstration:** Install the project editable through a compatible backend, import it, modify a source declaration without reinstalling, and distinguish changes that still require rebuild such as native code or entry metadata.
- **Variants and failures:** Backends implement editable exposure differently; added packages may or may not appear without reinstall; metadata should match a normal wheel except allowed dependencies; namespace and import-hook approaches differ.
- **Constraints / authority / confidence / verification:** Packaging ecosystem, PEP 660; **high**. Verify PEP 660, “Editable installs for pyproject.toml based builds.”

### PY-PKG-012 — Package data and installed resource inclusion

- **Distinct behavior:** Non-Python resources must be intentionally included in built artifacts and accessed through package-aware APIs; source-control presence alone does not imply installation.
- **Demonstration:** Include textual and binary package data, configure backend inclusion, verify both wheel/sdist contents, and read resources from an installed package.
- **Variants and failures:** Namespace packages and non-filesystem loaders need traversable resource APIs; editable installs can hide missing wheel configuration; data outside packages may install to platform-dependent locations.
- **Constraints / authority / confidence / verification:** Packaging backend convention plus `importlib.resources`; **high**. Verify backend package-data documentation and `importlib.resources`.

### PY-PKG-013 — Distribution metadata and runtime version lookup

- **Distinct behavior:** Runtime code can query installed distribution metadata, files, requirements, entry points, and version, but such metadata may be absent when running directly from an uninstalled source tree.
- **Demonstration:** Derive a public runtime version through `importlib.metadata` with a controlled fallback or generated source for source-tree execution, and compare it to built metadata.
- **Variants and failures:** Distribution and import names differ; editable installs supply metadata; vendoring/frozen apps may omit it; multiple installed versions/paths create ambiguous or inconsistent environments.
- **Constraints / authority / confidence / verification:** Standard library plus packaging ecosystem; **high**. Verify `importlib.metadata` and Core Metadata.

### PY-PKG-014 — Direct URL origin and provenance metadata

- **Distinct behavior:** Installations from version-control or archive references can record origin and requested revision separately from ordinary index-based version metadata.
- **Demonstration:** Use a local self-contained direct reference or inspect a prepared direct-URL metadata example without requiring external network access.
- **Variants and failures:** Editable version-control installs include different fields; credentials must not be retained; mutable branch names weaken reproducibility; hashes apply to archive URLs.
- **Constraints / authority / confidence / verification:** Packaging ecosystem, PEP 610; **medium-high**. Verify PEP 610, “Recording the Direct URL Origin of installed distributions.”

### PY-PKG-015 — Reproducible builds and environment-sensitive outputs

- **Distinct behavior:** Wheels and sdists can vary with time, locale, filesystem order, absolute paths, compiler flags, generated versions, or dependency versions unless the build deliberately normalizes inputs.
- **Demonstration:** Build twice from equivalent clean inputs with a fixed reproducibility epoch/environment, compare normalized artifacts, and include one documented environment-derived metadata or native-build variant as a countercase.
- **Variants and failures:** Archive timestamps and compression metadata differ; native binaries embed toolchain paths/IDs; build isolation does not guarantee locked dependencies; generated file ordering can be nondeterministic.
- **Constraints / authority / confidence / verification:** Ecosystem/toolchain convention; **medium-high**. Verify backend reproducible-build guidance and the reproducible-builds specification/practices used by the selected toolchain.

### PY-PKG-016 — Lock files and dependency groups

- **Distinct behavior:** Resolved lock data and development dependency groups describe environment construction beyond published runtime metadata, but standards and tool formats are recent or tool-specific.
- **Demonstration:** Include the selected tool’s lock/configuration with at least runtime and development groups, environment-specific resolution, hashes where supported, and a reproducible sync/install operation.
- **Variants and failures:** Lock formats differ and new standard lock proposals/specifications may supersede tool formats; dependency groups are not necessarily published extras; cross-platform universal versus per-environment locks trade size for determinism.
- **Constraints / authority / confidence / verification:** Ecosystem, rapidly evolving around Python 3.13-era packaging standards; **low-medium**. Verify current PyPA specifications and selected package manager documentation.

### PY-PKG-017 — Virtual environments and installation scheme isolation

- **Distinct behavior:** Virtual environments alter interpreter prefixes, executable selection, import paths, and script installation while optionally exposing system site packages.
- **Demonstration:** Create an isolated environment, install the built project, record prefix/base-prefix and import provenance, and contrast with direct source-tree execution.
- **Variants and failures:** Environments are not portable by directory copy; activation is shell convenience rather than required; user-site behavior and externally managed base environments vary; symlink/copy modes differ by platform.
- **Constraints / authority / confidence / verification:** Python standard library plus installer behavior, PEP 405; **high**. Verify `venv`, `site`, and PEP 405.

### PY-PKG-018 — Externally managed base environments

- **Distinct behavior:** A base Python installation can declare itself externally managed so Python-specific installers refuse or require explicit override for global modifications, redirecting users toward virtual environments.
- **Demonstration:** Represent or test installer behavior against an environment bearing the marker without mutating the actual system installation.
- **Variants and failures:** Frontend override flags and distro guidance differ; virtual environments are exempt; marker location follows interpreter installation schemes; behavior is installer policy rather than import semantics.
- **Constraints / authority / confidence / verification:** Packaging ecosystem, PEP 668; **medium-high**. Verify PEP 668 and the Externally Managed Environments specification.

### PY-PKG-019 — Build hooks, metadata preparation, and config settings

- **Distinct behavior:** A build frontend invokes backend hooks for requirements, metadata, sdists, regular wheels, and editable wheels, passing frontend-defined configuration settings and potentially taking optimized paths.
- **Demonstration:** Use a backend or thin local backend exposing observable hook selection, one dynamic build requirement or metadata-preparation path, and a namespaced config setting with deterministic effect.
- **Variants and failures:** Frontends need not call optional hooks; metadata prepared early must match the eventual wheel; config-setting spelling/semantics are backend-defined; in-process and subprocess execution must not be assumed.
- **Constraints / authority / confidence / verification:** Packaging ecosystem, PEP 517 and PEP 660; **high**. Verify the PyPA Build System Interface and selected backend/frontend documentation.

### PY-PKG-020 — Resolver environment and conflicting dependency constraints

- **Distinct behavior:** Installation computes a compatible dependency graph from versions, markers, extras, Python constraints, and available artifacts; source imports alone do not reveal the selected or rejected versions.
- **Demonstration:** Model a local, offline set of candidate distributions with one satisfiable branched resolution and one genuine version conflict, then inspect the resolved installed metadata or diagnostic.
- **Variants and failures:** Resolver algorithms and diagnostics are frontend-specific; already-installed packages, constraints files, yanked releases, prereleases, and build failures influence results; lock/sync tools may refuse re-resolution.
- **Constraints / authority / confidence / verification:** Packaging ecosystem; **medium-high**. Verify selected installer/resolver documentation plus PyPA version/dependency specifications.

## 11. Native code, foreign interfaces, generated sources, and deployment

### PY-NATIVE-001 — CPython extension module initialization

- **Distinct behavior:** A native shared library exports an interpreter-discovered initialization symbol that creates a module and exposes functions/types without Python source definitions.
- **Demonstration:** Build a minimal extension module with one function and one heap type, import it, and retain native sources/build declarations as the semantic origin.
- **Variants and failures:** Single-phase and multi-phase initialization differ; subinterpreter safety and per-module state require modern slots; filename suffix and linker rules are platform-specific; initialization-name mismatch produces import failure.
- **Constraints / authority / confidence / verification:** CPython C API; **high**. Verify “Defining Extension Modules,” PEP 489, and CPython extension documentation.

### PY-NATIVE-002 — Stable ABI, limited API, and CPython-version ABI

- **Distinct behavior:** Extensions can target a specific CPython ABI or restrict themselves to the limited API for a stable `abi3` compatibility range, changing available symbols and wheel tags.
- **Demonstration:** Configure one extension for the limited API or document a controlled build matrix comparing limited and version-specific binaries.
- **Variants and failures:** Some API/features are unavailable under the limited surface; compile-time lower-bound macros matter; free-threaded builds have distinct compatibility concerns; third-party native dependencies can still constrain portability.
- **Constraints / authority / confidence / verification:** CPython and packaging; **high**. Verify CPython Stable Application Binary Interface documentation and wheel tag specifications.

### PY-NATIVE-003 — Native ownership, reference counting, and exception translation

- **Distinct behavior:** CPython C extensions explicitly manage owned/borrowed references and report Python exceptions through a thread-local error indicator and sentinel return values.
- **Demonstration:** Include successful and failing native calls, object ownership across container insertion, and a translated native validation failure, with debug-build or leak checks where feasible.
- **Variants and failures:** Missing decrements leak; extra decrements corrupt; returning a non-error result with an exception set triggers `SystemError`; alternative bindings automate ownership differently; immortal objects alter implementation internals, not extension contracts.
- **Constraints / authority / confidence / verification:** CPython C API implementation contract; **high**. Verify C API reference-counting and exception-handling documentation.

### PY-NATIVE-004 — GIL management and free-threaded extension declarations

- **Distinct behavior:** Native code may release/reacquire the GIL around blocking work, attach external threads, and in free-threaded CPython builds must meet additional thread-safety and module-declaration requirements.
- **Demonstration:** Include a native operation that safely releases the GIL without touching Python objects, plus build/runtime guards describing free-threaded support.
- **Variants and failures:** C API calls without attached thread state are invalid; subinterpreters complicate thread attachment; a free-threaded interpreter may re-enable the GIL for incompatible extensions; exact APIs evolve.
- **Constraints / authority / confidence / verification:** CPython, materially changed experimentally in 3.13 via PEP 703; **medium**. Verify CPython C API thread-state/free-threading documentation and PEP 703.

### PY-NATIVE-005 — Foreign function calls through `ctypes` or equivalent

- **Distinct behavior:** Runtime foreign-function interfaces load shared libraries and translate declared native signatures without an importable extension module, making correctness depend on ABI declarations.
- **Demonstration:** Call a platform-provided or project-built tiny native library with explicit argument/result types, one structure or callback, and error handling.
- **Variants and failures:** Calling convention, type width, structure packing, library naming, symbol visibility, callback lifetime, and thread-local error capture vary by platform; wrong signatures can crash the process.
- **Constraints / authority / confidence / verification:** CPython standard library plus platform ABI; **medium-high**. Verify `ctypes` documentation and target platform ABI documentation.

### PY-NATIVE-006 — Generated Python modules

- **Distinct behavior:** Build tools can generate importable `.py` or `.pyi` files from schemas, templates, grammar, or metadata, so semantic source provenance crosses a generation boundary.
- **Demonstration:** Include a deterministic generator input and build step producing a module or stub whose declarations are imported by handwritten code; validate freshness without prescribing a particular generator.
- **Variants and failures:** Generated files may be committed, sdist-only inputs, or wheel-only outputs; stale checked-in output drifts; line/source maps may be absent; generation may depend on host tools unavailable in isolated builds.
- **Constraints / authority / confidence / verification:** Ecosystem/build convention; **high**. Verify the selected generator and build backend documentation.

### PY-NATIVE-007 — Code generation at import or runtime

- **Distinct behavior:** Programs can construct classes/functions through `type`, closures, `exec`, AST compilation, factories, or metaprogramming, creating entities without one direct declaration site.
- **Demonstration:** Include a factory generating named classes or functions with stable metadata and a consumer resolving them through a registry; preserve input-to-output provenance.
- **Variants and failures:** Generated qualified names/modules affect pickling; tracebacks point to synthetic filenames/lines; closures can share state accidentally; static tools may see only declared protocols/stubs.
- **Constraints / authority / confidence / verification:** Language runtime behavior; **high**. Verify built-in `type`, `compile`/`exec`, AST, and data-model documentation.

### PY-NATIVE-008 — Cython or alternative source-to-extension compilation

- **Distinct behavior:** Common transpilers accept Python-like source with extra type/build constructs and emit C/C++ extension code, producing runtime entities whose authoritative source is neither plain Python nor hand-written C.
- **Demonstration:** If included, compile a minimal typed module with both Python-callable and native-only declarations and expose generated-source provenance.
- **Variants and failures:** Pure-Python and extension modes differ; language levels change Python semantics; generated C is usually not the authored source; annotation meanings can be compiler directives; tool versions affect ABI/output.
- **Constraints / authority / confidence / verification:** Ecosystem convention, optional; **medium**. Verify the chosen compiler’s official language and build documentation.

### PY-NATIVE-009 — Frozen or bundled applications

- **Distinct behavior:** Freezers bundle an interpreter, modules, metadata, and resources after static/dynamic import discovery, changing origins, filesystem assumptions, and sometimes module inclusion.
- **Demonstration:** Build one standalone artifact, exercise package resources, dynamic imports, distribution metadata, and subprocess/self-location behavior from the bundle.
- **Variants and failures:** Hidden imports and data need hooks; one-file extraction differs from directory bundles; native libraries need collection; `sys.frozen`, `_MEIPASS`, or equivalents are tool-specific; platform builds are usually not cross-platform.
- **Constraints / authority / confidence / verification:** Ecosystem/tool-defined; **medium**. Verify selected freezer’s official documentation.

### PY-NATIVE-010 — Zip applications and executable archives

- **Distinct behavior:** A Python zip application starts from archive-root `__main__.py` and imports bundled pure Python through zip import semantics.
- **Demonstration:** Build an executable archive containing package code and resources, then invoke it and compare module origins to an installed run.
- **Variants and failures:** Native extensions cannot normally load in-place; shebang/interpreter selection is platform-sensitive; dependencies may be bundled or preinstalled; archive timestamp/reproducibility concerns apply.
- **Constraints / authority / confidence / verification:** Python standard library, PEP 441; **high**. Verify `zipapp`, `zipimport`, and PEP 441.

### PY-NATIVE-011 — Alternative Python implementations

- **Distinct behavior:** PyPy, GraalPy, MicroPython, and others implement different subsets and performance/foreign-interface behavior while aiming at varying degrees of language compatibility.
- **Demonstration:** Keep implementation-guarded behavior isolated, identify a capability via `sys.implementation`, and run common language-semantic cases on any included alternative.
- **Variants and failures:** C-extension compatibility, garbage collection, frame/bytecode introspection, recursion, standard-library availability, small-device limitations, and JIT behavior differ; `platform.python_implementation` is descriptive, not a capability check.
- **Constraints / authority / confidence / verification:** Implementation-defined; **high** that differences exist, **low-medium** on current details. Verify each implementation’s official compatibility documentation.

### PY-NATIVE-012 — Subinterpreters and interpreter-local state

- **Distinct behavior:** Multiple interpreters in one process have distinct module dictionaries and much runtime state, while extension modules and some process-global resources may be shared or constrained.
- **Demonstration:** If supported, execute an import/state mutation in two subinterpreters and verify separation, including a native extension’s declared compatibility.
- **Variants and failures:** Public high-level APIs are evolving; objects generally cannot be shared directly; extension singletons/global C state can violate isolation; per-interpreter GIL work is version-specific.
- **Constraints / authority / confidence / verification:** CPython implementation, rapidly evolving around 3.12–3.14; **low-medium**. Verify current CPython subinterpreter documentation, PEP 554 status, and PEP 684.

## 12. Platform, locale, filesystem, and environment variants

### PY-PLAT-001 — Filesystem encoding and `str`/`bytes` paths

- **Distinct behavior:** Filesystem APIs accept text and often bytes paths, encode/decode through platform settings and surrogate escapes, and may return path values in the representation supplied.
- **Demonstration:** Create a filename representable through the baseline filesystem, exercise `pathlib`/`os` text paths, and on POSIX include a bytes-name case not valid UTF-8 with guarded expectations.
- **Variants and failures:** Windows path APIs are Unicode-centric and bytes support is limited; UTF-8 mode changes defaults; unencodable paths fail; normalization and case behavior are filesystem-specific.
- **Constraints / authority / confidence / verification:** CPython standard library/platform, PEP 383 and PEP 540; **medium-high**. Verify filesystem encoding, `os`, and UTF-8 mode documentation.

### PY-PLAT-002 — Case sensitivity and import-name collisions

- **Distinct behavior:** Filesystem case rules affect whether differently cased module/package filenames collide or import, while Python identifiers and import strings remain case-sensitive.
- **Demonstration:** Represent a guarded collision case and avoid assuming both candidates can coexist on every filesystem.
- **Variants and failures:** Windows/macOS configurations are commonly case-insensitive, Linux commonly sensitive, but volumes can differ; cached bytecode and version control can mask case-only renames.
- **Constraints / authority / confidence / verification:** Platform/filesystem and CPython importer; **high**. Verify `importlib` path-finder behavior and target filesystem documentation.

### PY-PLAT-003 — Path syntax, symlinks, and canonical identity

- **Distinct behavior:** Drive letters, UNC paths, separators, reserved names, symlinks, hard links, and case normalization mean lexical paths and filesystem object identity differ.
- **Demonstration:** Use `pathlib` for lexical operations and guarded real filesystem links, comparing absolute/resolved paths and same-file checks.
- **Variants and failures:** Symlink creation may require privileges; resolve can fail or loop; Windows and POSIX path classes have incompatible semantics; import caches may see the same source through multiple path spellings.
- **Constraints / authority / confidence / verification:** Standard library/platform; **high**. Verify `pathlib` and `os.path` documentation.

### PY-PLAT-004 — Newlines, text encoding, and locale

- **Distinct behavior:** Text I/O decodes bytes, translates newlines, and chooses defaults from locale/UTF-8 mode unless encoding/newline are explicit.
- **Demonstration:** Read/write controlled UTF-8 data with explicit settings, exercise universal newline input, and include a subprocess run under a distinct locale or UTF-8 mode.
- **Variants and failures:** Default encoding changes under UTF-8 mode and may change in future Python; Windows output translation differs; invalid bytes depend on error handler; locale availability varies.
- **Constraints / authority / confidence / verification:** CPython standard library/platform, PEP 540 and PEP 597; **high**. Verify `open`, `io`, locale, and UTF-8 mode documentation.

### PY-PLAT-005 — Environment variables and process startup configuration

- **Distinct behavior:** Environment variables affect Python startup, encoding, import paths, warnings, hashing, allocator, and application configuration before or during module import.
- **Demonstration:** Launch child interpreters with controlled `PYTHONHASHSEED`, UTF-8/warning settings, and an application variable, recording effects without relying on the parent environment.
- **Variants and failures:** Isolated mode ignores many Python variables; empty/unset values differ; environment key case sensitivity is platform-specific; values are process-global and race-prone to mutate in threads.
- **Constraints / authority / confidence / verification:** CPython startup/platform; **high**. Verify Python command-line and environment variable documentation.

### PY-PLAT-006 — Time zones, clocks, and timestamp ambiguity

- **Distinct behavior:** Naive and aware datetimes, local timezone rules, daylight-saving folds, monotonic clocks, wall clocks, and filesystem timestamp precision have distinct semantics.
- **Demonstration:** Include an aware datetime across an ambiguous local-time fold using packaged/system zone data, compare monotonic versus wall-clock use, and inspect file timestamp resolution without asserting exact precision.
- **Variants and failures:** System time-zone database availability differs; `tzdata` can be an optional dependency; leap seconds are generally not represented; environment `TZ` support is platform-specific.
- **Constraints / authority / confidence / verification:** Standard library/platform, PEP 495 and PEP 615; **medium-high**. Verify `datetime`, `time`, and `zoneinfo` documentation.

### PY-PLAT-007 — Subprocess execution, quoting, and descriptors/handles

- **Distinct behavior:** Process creation takes an argument vector or shell command, with platform-specific executable lookup, quoting, signal, descriptor inheritance, and text encoding.
- **Demonstration:** Spawn the current interpreter with an explicit argument list, exchange text and bytes, propagate an exit code, set an environment, and separately guard any shell invocation.
- **Variants and failures:** Windows command-line reconstruction differs from POSIX `exec`; `shell=True` changes injection and lookup risks; close-on-exec/handle inheritance differs; deadlock can occur without `communicate` or async handling.
- **Constraints / authority / confidence / verification:** Standard library/platform; **high**. Verify `subprocess` documentation.

### PY-PLAT-008 — Platform-conditional APIs and dependency fallbacks

- **Distinct behavior:** Modules, attributes, constants, or dependencies may exist only on selected operating systems or interpreter builds, requiring guarded imports and capability checks.
- **Demonstration:** Include one POSIX-only and one Windows-only branch or harmless stand-in, with a shared public wrapper and explicit unsupported-path behavior.
- **Variants and failures:** Checking `os.name`/`sys.platform` can be less precise than feature detection; type checkers prune branches using target platform; documentation builders may mock imports; dead branches must still parse on the target Python version.
- **Constraints / authority / confidence / verification:** Language plus platform/typing ecosystem; **high**. Verify `sys.platform`, `os`, and relevant platform module documentation.

### PY-PLAT-009 — Hash randomization and nondeterministic identities

- **Distinct behavior:** CPython salts hashes for selected immutable types per process, affecting hash values and iteration order of unordered containers while preserving equality and mapping correctness.
- **Demonstration:** Run child interpreters with random and fixed hash seeds, compare hash/order-sensitive output, and keep deterministic serialization explicitly sorted.
- **Variants and failures:** A seed of zero disables randomization; custom hashes may not be salted; dict insertion order can remain stable despite differing hashes; hash width differs by build architecture.
- **Constraints / authority / confidence / verification:** CPython implementation/startup, PEP 456 background; **high**. Verify `PYTHONHASHSEED`, `sys.hash_info`, and data-model hashing notes.

### PY-PLAT-010 — Architecture and build-flag differences

- **Distinct behavior:** Pointer width, byte order, debug/optimized/free-threaded builds, Unicode internals, and optional modules affect limits, binary formats, available APIs, and compatibility.
- **Demonstration:** Record capabilities through `sysconfig`, `sys.maxsize`, `sys.byteorder`, ABI flags/tags, and guarded optional imports rather than asserting one architecture.
- **Variants and failures:** 32-bit sizes overflow sooner; debug allocators expose bugs; free-threaded builds use distinct ABI indications; distributor builds can omit standard extension modules or apply patches.
- **Constraints / authority / confidence / verification:** CPython implementation/platform; **high**. Verify `sys`, `sysconfig`, and CPython build configuration documentation.

## 13. Diagnostics, warnings, tests, and analysis configuration

### PY-DIAG-001 — Tokenization, parsing, symbol-table, and compile diagnostics

- **Distinct behavior:** Invalid source can fail during decoding/tokenization, parsing, contextual name analysis, or bytecode compilation, with distinct exception types, offsets, end offsets, and messages.
- **Demonstration:** Keep isolated negative inputs for invalid bytes/encoding, unterminated literal/delimiter, grammar error, indentation error, tab error, invalid `nonlocal`, duplicate parameters/captures, and context-invalid control statements.
- **Variants and failures:** Exact wording, caret ranges, suggestions, and recovery improve across CPython releases; some errors move phases as grammar changes; tools parsing with a target grammar may disagree with the running interpreter.
- **Constraints / authority / confidence / verification:** Language invalidity with CPython diagnostic representation; **high** for rejection, **medium** for message stability. Verify lexical/grammar references and built-in `SyntaxError` attributes.

### PY-DIAG-002 — Import failure taxonomy

- **Distinct behavior:** Resolution can fail because no module spec exists, a requested imported name is absent, module execution raised an unrelated exception, or a binary dependency could not load.
- **Demonstration:** Include controlled cases producing `ModuleNotFoundError`, `ImportError` for a missing `from` name, an exception from module initialization, and a guarded native-loader failure.
- **Variants and failures:** Catching broad `ImportError` around an optional dependency can mask its transitive/internal failure; exception `name` and `path` attributes aid attribution; cyclic-import messages are implementation-specific.
- **Constraints / authority / confidence / verification:** Python 3; language/CPython exceptions and loader behavior; **high**. Verify built-in exceptions and import system documentation.

### PY-DIAG-003 — Attribute, name, key, index, and type errors

- **Distinct behavior:** Similar “missing” operations report distinct exceptions based on namespace, attribute protocol, mapping key, sequence index, or invalid operand/call contract.
- **Demonstration:** Trigger each category in a minimal controlled operation and inspect structured attributes where defined, including enhanced name/attribute suggestions only as non-stable output.
- **Variants and failures:** User protocols can translate or misuse exception types; `__getattr__` depends specifically on `AttributeError`; iteration sequence fallback depends on `IndexError`; exact CPython messages are not portable APIs.
- **Constraints / authority / confidence / verification:** Language/data model with CPython messages; **high**. Verify built-in exception documentation and relevant protocols.

### PY-DIAG-004 — Warning categories, filters, registries, and stack levels

- **Distinct behavior:** Warnings are categorized, filtered by ordered rules, deduplicated through per-module registries, and attributed to a source location chosen by `stacklevel` or skip logic.
- **Demonstration:** Emit custom and built-in-category warnings through a wrapper, configure capture/error/ignore filters, reset registries, and verify caller attribution without exact formatted text.
- **Variants and failures:** Default filters differ for developer mode, debug builds, tests, and `__main__`; environment/command-line filters apply before application code; concurrent filter mutation is process-global; newer APIs add skip-prefix support.
- **Constraints / authority / confidence / verification:** CPython standard library; **high**. Verify `warnings`, warning-control command-line options, and development mode documentation.

### PY-DIAG-005 — Deprecation lifecycle and compatibility shims

- **Distinct behavior:** Deprecated syntax/APIs can remain valid while emitting warnings, later become errors, and often require version- or capability-selected replacement paths.
- **Demonstration:** Include one local deprecated API emitting an attributed `DeprecationWarning`, a replacement, and compatibility code selected without executing an invalid syntax branch on older versions.
- **Variants and failures:** `DeprecationWarning` is hidden by default outside `__main__`; `FutureWarning` targets application users; importing removed modules fails; warning-as-error configurations expose otherwise hidden paths.
- **Constraints / authority / confidence / verification:** Language/standard library plus project convention; **high**. Verify `warnings` and the deprecation notice for any selected standard feature.

### PY-DIAG-006 — Runtime unraisable exceptions and thread exceptions

- **Distinct behavior:** Exceptions from destructors, weakref callbacks, and some cleanup hooks cannot propagate normally and go to `sys.unraisablehook`; uncaught thread failures go to `threading.excepthook`.
- **Demonstration:** Install temporary hooks, trigger a controlled unraisable callback and thread failure, record structured fields, then restore hooks.
- **Variants and failures:** Holding referenced objects in hooks can resurrect or leak them; shutdown behavior differs; subprocess isolation prevents polluting the main test run; exact stderr formatting is not stable.
- **Constraints / authority / confidence / verification:** CPython/Python standard library, hooks added in modern Python; **high**. Verify `sys.unraisablehook` and `threading.excepthook`.

### PY-DIAG-007 — Static type errors versus valid runtime behavior

- **Distinct behavior:** Many typing violations remain syntactically and operationally valid at runtime, while other annotations can fail at runtime during evaluation or introspection.
- **Demonstration:** Separate deliberate checker-negative cases—bad assignment, bad override, non-exhaustive assumptions, invalid overload—from runtime-negative cases, with expected tool configuration explicit.
- **Variants and failures:** Different checkers accept extensions or infer differently; `Any` suppresses propagation of errors; untyped function bodies may receive limited checking; runtime annotation policy changes outcomes.
- **Constraints / authority / confidence / verification:** Typing ecosystem; **high**. Verify the typing specification and selected checker documentation.

### PY-DIAG-008 — Linter/parser target-version and configuration effects

- **Distinct behavior:** Linters and formatters parse and diagnose according to configured target Python versions, enabled rule families, per-file exclusions, and generated/vendor exclusions.
- **Demonstration:** Supply explicit tool configuration, one version-dependent source construct, one narrow per-file exception for a negative case, and clear inclusion/exclusion of generated source.
- **Variants and failures:** Tools may parse syntax newer than the running interpreter; configuration discovery depends on working directory and file hierarchy; plugin versions alter rules; auto-fixes can change semantics at edge cases.
- **Constraints / authority / confidence / verification:** Ecosystem/tool-defined; **high**. Verify selected linter/formatter official configuration documentation.

### PY-DIAG-009 — Standard test discovery and load behavior

- **Distinct behavior:** `unittest` discovers modules/classes/methods by names and imports test modules, then fixture hooks and cleanup stacks determine setup/teardown relationships.
- **Demonstration:** Include discovered tests, class/module fixtures, subtests, skips/expected failures, and a load failure represented as a test error.
- **Variants and failures:** Namespace-package discovery support changed across versions; top-level directory affects import names; import side effects happen during collection; cleanup hooks run after partial setup in defined circumstances.
- **Constraints / authority / confidence / verification:** Python standard library; **high**. Verify `unittest` discovery and fixture documentation.

### PY-DIAG-010 — Pytest-style collection, fixtures, parametrization, and plugins

- **Distinct behavior:** A common external test ecosystem creates test instances and dependency edges through naming, fixture parameters, scopes, markers, hooks, and plugin loading rather than explicit calls.
- **Demonstration:** If this ecosystem is selected, include fixture dependency/injection, parametrized cases with stable IDs, scoped/autouse fixtures, one marker, and local plugin/conftest discovery.
- **Variants and failures:** Import modes change module identity/path; same-named test modules can collide; plugin autoload makes environments non-hermetic unless controlled; fixture cycles/missing fixtures are collection errors; async tests need a plugin.
- **Constraints / authority / confidence / verification:** Common ecosystem convention, optional external dependency; **high**. Verify pytest’s official fixtures, parametrization, collection, import modes, and plugin documentation.

### PY-DIAG-011 — Doctest discovery and execution context

- **Distinct behavior:** Examples embedded in docstrings or text files can be parsed and executed as tests in a shared example namespace with output-matching rules.
- **Demonstration:** Include one docstring example with state shared across prompts, exception output, and an option directive, then run it through standard discovery.
- **Variants and failures:** Prompt/whitespace/ellipsis normalization options change outcomes; optimization can remove docstrings; nondeterministic repr/order is fragile; imported versus script module names affect examples.
- **Constraints / authority / confidence / verification:** Python standard library/ecosystem documentation convention; **high**. Verify `doctest` documentation.

### PY-DIAG-012 — Coverage, tracing, profiling, and monitoring effects

- **Distinct behavior:** Tracing/profiling/monitoring APIs observe calls, lines, returns, exceptions, opcodes, and generated code; they can change timing and expose code objects lacking ordinary source.
- **Demonstration:** Install a minimal trace or monitoring callback for selected project code, exercise normal/generator/async/exception paths, and remove it reliably.
- **Variants and failures:** `sys.settrace` is per-thread and slow; Python 3.12 introduced `sys.monitoring` through PEP 669; branch coverage tools define arcs differently; optimized/generated/native code may be partially invisible.
- **Constraints / authority / confidence / verification:** CPython APIs and ecosystem tools, materially changed in 3.12; **medium-high**. Verify `sys.settrace`, `sys.setprofile`, `sys.monitoring`, and PEP 669.

### PY-DIAG-013 — Logging hierarchy and configuration-time binding

- **Distinct behavior:** Loggers form a dotted-name hierarchy with propagation, effective levels, handlers, filters, and deferred message formatting, usually configured outside call sites.
- **Demonstration:** Create module-named child loggers, configure parent/child handlers and propagation, use structured extras and exception information, and capture output without global leakage.
- **Variants and failures:** Duplicate handlers duplicate records; import-time configuration conflicts with applications; formatting style and encoding differ; lazy `%` formatting avoids work but f-strings evaluate immediately; multiprocessing needs coordination.
- **Constraints / authority / confidence / verification:** Python standard library/ecosystem convention; **high**. Verify `logging` documentation.

### PY-DIAG-014 — Security-sensitive dynamic features

- **Distinct behavior:** `eval`/`exec`, pickle loading, shell invocation, dynamic imports, path extraction, and unsafe deserialization cross trust boundaries and can execute code even when presented as data operations.
- **Demonstration:** Include only benign negative/guard cases showing that trusted and untrusted inputs take different APIs; do not embed an exploit payload.
- **Variants and failures:** Removing built-ins does not make `eval` a secure sandbox; pickle authenticity is required; shell quoting is platform-specific; archive path traversal defenses vary by version/API; audit hooks observe but are not a full sandbox.
- **Constraints / authority / confidence / verification:** Language/standard-library security contracts; **high**. Verify security warnings in `eval`, `pickle`, `subprocess`, archive, and import documentation.

### PY-DIAG-015 — Audit hooks

- **Distinct behavior:** CPython/Python audit events expose security-relevant operations to registered hooks, with event names/arguments forming a documented runtime observation surface.
- **Demonstration:** Register a narrow audit hook in an isolated process, cause controlled import/file/compile events, and record event categories without assuming every implementation emits identical internals.
- **Variants and failures:** Python-level hooks added after startup are not a sandbox and can be bypassed by malicious native code; native hooks can install earlier; hooks cannot normally be removed; events and arguments evolve compatibly.
- **Constraints / authority / confidence / verification:** CPython/runtime API, PEP 578; **medium-high**. Verify PEP 578 and `sys.addaudithook`/audit events documentation.

## 14. Cross-feature interaction candidates

These items are retained separately because exercising each constituent feature in isolation would not expose the combined behavior.

### PY-INT-001 — Decorated, overloaded, generic descriptor

- **Distinct behavior:** A method can simultaneously be a descriptor, wrapped callable, generic static contract, overload implementation, and runtime registration target; decorator order controls which object each layer observes.
- **Demonstration:** Include a property/class method or singledispatch method with annotations/overloads and metadata-preserving wrapper, then inspect binding, static surface, and runtime target.
- **Variants and failures:** Wrong order loses `.register`, abstractness, setter, signature, or binding; stub declarations can model a surface that wrappers obscure; runtime dispatch and static overload selection can disagree.
- **Constraints / authority / confidence / verification:** Language plus typing/standard-library conventions; **high**. Verify descriptor, decorator, `functools`, `abc`, and typing documentation.

### PY-INT-002 — Dataclass inheritance with descriptors, slots, and pattern matching

- **Distinct behavior:** Dataclass field discovery and generated methods interact with inherited field order, descriptor defaults, frozen writes, slot generation, weak references, and `__match_args__` consumed by patterns.
- **Demonstration:** Include a base/subclass pair using these features and match instances positionally and by keyword while exercising generated initialization and a managed field.
- **Variants and failures:** Slot-name overlap is skipped in newer versions; descriptor default probing invokes class access; keyword-only fields are excluded from positional matching; frozen behavior can be bypassed by low-level object methods.
- **Constraints / authority / confidence / verification:** Python 3.10+ for full interaction; standard library plus language; **high**. Verify `dataclasses`, descriptor behavior, and pattern matching.

### PY-INT-003 — Circular imports with annotations and runtime introspection

- **Distinct behavior:** Type-only imports and forward annotations can break runtime import cycles, but resolving annotations later can reintroduce imports or fail from missing namespaces.
- **Demonstration:** Include two mutually referring modules using guarded type imports and forward annotations, then resolve hints with explicit correct namespaces and show a controlled failure without them.
- **Variants and failures:** Future/deferred annotations change storage; dataclasses/frameworks may resolve annotations during class creation; base classes and runtime decorators still require real imports; aliases can be lost from namespaces.
- **Constraints / authority / confidence / verification:** Python 3; typing plus import semantics, version-sensitive; **high**. Verify `typing.TYPE_CHECKING`, `get_type_hints`, and annotation PEPs.

### PY-INT-004 — Namespace package split across distributions with resources and entry points

- **Distinct behavior:** Independent distributions can contribute modules/resources to one namespace and advertise plugins, so package identity, resource origin, and distribution ownership are not one-to-one.
- **Demonstration:** Build/install two local distributions contributing separate portions and entry points, then discover imports, resources, and owning metadata.
- **Variants and failures:** Regular-package collision hides portions; uninstalling one distribution removes only its files but can damage shared directories under poor installers; resource APIs and editable installs may compose portions differently.
- **Constraints / authority / confidence / verification:** Import language plus packaging ecosystem; **high**. Verify PEP 420, `importlib.resources`, `importlib.metadata`, and entry-point specifications.

### PY-INT-005 — Generated version/source under isolated and editable builds

- **Distinct behavior:** Version-control-derived or otherwise generated modules must be available during isolated wheel/sdist builds and source-tree/editable execution without creating inconsistent identities.
- **Demonstration:** Generate one metadata-bearing source from deterministic local inputs, build wheel and sdist, install regular/editable, and compare the exposed value/provenance.
- **Variants and failures:** Source archives may lack version-control metadata; stale generated files drift; editable hooks can bypass a build step; importing the package to obtain build metadata can create dependency cycles.
- **Constraints / authority / confidence / verification:** Build-backend ecosystem; **medium-high**. Verify the selected backend/versioning tool documentation and PEP 517 isolation rules.

### PY-INT-006 — Async generator cleanup under cancellation and context management

- **Distinct behavior:** Cancellation can interrupt an async consumer while an async generator and async context manager both have pending cleanup, requiring correct exception propagation and finalization order.
- **Demonstration:** Consume an async generator inside an async manager/task group, cancel during suspension, and record that both cleanup layers run while cancellation is not silently lost.
- **Variants and failures:** Suppressing `CancelledError` without uncancelling can corrupt structured-concurrency assumptions; loop shutdown handles unfinished async generators; cleanup can raise and form groups.
- **Constraints / authority / confidence / verification:** Python 3.11+ full interaction; language plus `asyncio`/`contextlib`; **medium-high**. Verify async generator finalization, task cancellation, and TaskGroup documentation.

### PY-INT-007 — Multiprocessing with package entry modes and pickling

- **Distinct behavior:** Spawned children re-import the main module under special startup conditions, so package execution mode, startup guards, top-level callable identity, and pickle-qualified names jointly determine correctness.
- **Demonstration:** Launch a top-level worker from both an importable package entry and a console/module entry path using spawn, with no import-time recursive side effects.
- **Variants and failures:** Interactive/local definitions fail; frozen applications require support; direct scripts can have different package context; test runners replace main-module identity.
- **Constraints / authority / confidence / verification:** CPython standard library/platform; **high**. Verify `multiprocessing` programming guidelines and `__main__` documentation.

### PY-INT-008 — Optional native dependency with typed fallback

- **Distinct behavior:** A package may expose one stable public interface backed by a native accelerator when importable and a pure-Python fallback otherwise, while stubs/static types describe both.
- **Demonstration:** Include two internal implementations selected by narrow import/capability handling, parity checks, a shared public re-export, and build metadata for the optional native artifact.
- **Variants and failures:** Catching broad import errors masks broken accelerators; behavior/performance can drift; platform wheels may omit unsupported targets; static analyzers can resolve a different branch from runtime configuration.
- **Constraints / authority / confidence / verification:** Ecosystem convention plus import/packaging semantics; **high**. Verify extension/import and packaging documentation; implementation parity is project-defined.

## Recent version deltas

The checklist baseline is CPython 3.13. The following deltas are material because they alter parseability, runtime entities/protocols, source inclusion, diagnostics, or configuration. Patch-release diagnostic wording and bug fixes are not enumerated.

| Version | Material changes relevant to this checklist | Coverage consequence | Confidence and primary verification |
| --- | --- | --- | --- |
| 3.8 | Assignment expressions; positional-only parameters; f-string debug expressions; `typing.Final`, `Protocol`, and typed dictionaries in the standard library; audit hooks; pickle protocol 5; shared-memory multiprocessing. | Establishes a practical lower compatibility boundary for several syntax and typing candidates. Files containing `/` parameters or `:=` cannot even parse on older interpreters. | **High.** Verify “What’s New In Python 3.8” and PEPs 570, 572, 578, and 590 where vectorcall internals matter. |
| 3.9 | Built-in collection generic aliases; dictionary union operators; flexible decorator expressions; `zoneinfo`; standard-library removals/deprecations and parser transition. | Modern annotations and mapping operators need version-separated parse cases; time-zone behavior gains a standard source. | **High.** Verify “What’s New In Python 3.9” and PEPs 585, 584, and 615. |
| 3.10 | Structural pattern matching; `X | Y` unions; `ParamSpec`; explicit type aliases in the standard typing surface; parenthesized context managers; `zip(strict=...)`; improved syntax diagnostics. | Adds a major new binding/dispatch family and runtime union entity; static call-shape coverage expands. | **High.** Verify “What’s New In Python 3.10” and PEPs 604, 612, 613, 634, 635, and 636. |
| 3.11 | Exception groups and `except*`; exception notes; `asyncio.TaskGroup`; `tomllib`; `Self`; variadic generics; required/not-required typed-dictionary keys; `StrEnum`; substantial interpreter/traceback changes; integer-string conversion limits. | Adds grouped control flow and structured concurrency; typing metadata expands; exact code objects/bytecode/tracebacks must be version-qualified. | **High.** Verify “What’s New In Python 3.11” and PEPs 654, 678, 673, 646, and 655. |
| 3.12 | Type parameter and `type` alias syntax; fully specified f-string grammar; `@override`; Python-level buffer protocol; low-impact monitoring; per-interpreter GIL infrastructure; `distutils` removal. | Adds syntax that will not parse on 3.11, new runtime type-alias/parameter entities, changed f-string tokenization, monitoring relationships, and build-tool migration requirements. | **High.** Verify “What’s New In Python 3.12” and PEPs 695, 701, 698, 688, 669, 684, and 632. |
| 3.13 | Defined `locals()` mutation/snapshot semantics; experimental free-threaded CPython build; experimental just-in-time compilation build mode; `TypeIs`; type-parameter defaults; typing/runtime refinements and removals. | Baseline must state build flags. Concurrency/ABI assumptions bifurcate. Dynamic-execution and local-introspection expectations become more portable. New typing syntax/objects need target 3.13. | **High** for PEP 667/703/742/696, **medium** for experimental implementation details. Verify “What’s New In Python 3.13,” PEPs 667, 703, 742, 696, and CPython build documentation. |
| 3.14 | Deferred annotation evaluation is expected to become default through PEP 649/749; template strings arrive through PEP 750; multiple-interpreter APIs and additional grammar simplifications are expected to mature. | Annotation storage/evaluation and runtime consumers can change without source edits. Template strings add a new structured literal entity. Baseline-3.13 tools must reject or isolate 3.14-only syntax. | **Medium**; this report did not verify final 3.14 status. Verify “What’s New In Python 3.14,” PEPs 649, 749, 750, 734, and the final 3.14 language reference before inclusion. |

Version interactions that deserve explicit paired cases:

- Annotation expressions under baseline eager behavior, `from __future__ import annotations`, and Python 3.14 deferred defaults.
- Identical f-string intent parsed under pre-3.12 restrictions and the PEP 701 grammar.
- A class/function generic in legacy `typing` syntax and PEP 695 syntax, including the different parameter scope/runtime metadata.
- Concurrent code under ordinary GIL-enabled CPython and a 3.13 free-threaded build, without treating either result as a language guarantee.
- Type-checker target versions that accept new typing features independently of runtime availability through backports.
- Packaging under a backend that previously depended on removed `distutils` and its supported modern replacement.
- Diagnostic snapshots separated by interpreter minor version, especially parser locations, import-cycle hints, tracebacks, and specialized call errors.

## Implementation-defined, unspecified, and deliberately unstable behavior

The following should be represented as qualified observations or variants, never as portable Python guarantees:

### CPython-specific representation and execution

- Exact bytecode instructions, specialization/adaptive caches, code-object private fields, line-table encoding, marshalled bytecode, and `.pyc` format/magic.
- Object memory layout, `sys.getsizeof`, allocator behavior, reference counts, immortal-object internals, small-object freelists, and whether `id()` resembles an address.
- Interning/caching of strings, integers, tuples, code constants, and bound objects. Identity of equal literals is not a portable semantic relationship.
- Prompt destruction caused by reference counting, cyclic garbage-collection schedule, finalizer order, weakref callback timing, and work performed at interpreter shutdown.
- GIL scheduling, operation-level atomicity, bytecode switching points, experimental just-in-time compiler decisions, and free-threaded performance or safety of unqualified extensions.
- Availability/content of frames, trace events, monitoring events beyond documented contracts, recursion limits, and stack-overflow handling.

### Language-specified freedom or partial specification

- Set/frozenset iteration order and order changes under hash randomization.
- Partial name bindings after a failed structural pattern match; code must not depend on preservation or cleanup.
- Number and order of some protocol method calls during pattern matching, beyond documented semantic outcomes.
- Finalization time for unreachable objects and relative finalization of unrelated objects.
- Exact representations that include identity/address, such as default `repr` output.
- Floating-point edge behavior inherited from the platform, including NaN payloads, signed zero details in some conversions, and overflow/underflow characteristics not fixed by a higher-level API.
- Scheduling and fairness among threads, tasks, processes, locks, queues, and asynchronous callbacks unless a specific library contract states otherwise.
- Resource-limit values, recursion depth at failure, file-descriptor/handle numbers, process identifiers, timestamps, and filesystem timestamp resolution.

### Host, tool, and ecosystem variability

- Filesystem case sensitivity, Unicode filename normalization, symlink privileges, path maximums, atomic rename details, advisory locking, and directory traversal order.
- Default encodings/locales/time zones, signal availability, subprocess quoting, multiprocessing default start method, shared-library lookup, and native ABI.
- Installed module/search path ordering after launchers, environment variables, virtual environments, editable installs, `.pth` processing, test runners, notebook kernels, or embedded hosts modify startup.
- Resolver choice, selected dependency version, build hook call sequence when hooks are optional, wheel choice among compatible artifacts, editable exposure mechanism, and exact generated script wrapper.
- Type-checker inference, plugin behavior, error codes/messages, linter rules, coverage arcs, formatter output across major tool versions, and IDE/notebook execution models.
- Exception, warning, parser, and import diagnostic wording and suggestion text. Exception class and structured fields are more stable than messages.
- Alternative interpreter support for CPython C extensions, weak/finalization timing, frames, tracing, standard-library modules, resource limits, and performance.

## Known knowledge limits and verification priorities

No live sources were consulted. These areas have the highest chance of incompleteness or stale detail:

1. **Python 3.14 final semantics.** PEP 649/749 annotation behavior, PEP 750 template objects, PEP 734 multiple-interpreter APIs, grammar changes, and precise transition/deprecation timing need verification against final 3.14 documentation.
2. **Free-threaded CPython.** Extension compatibility declarations, wheel/ABI tags, GIL fallback, standard-library thread safety, and debugger/profiler support are experimental and fast-moving.
3. **Typing specification evolution.** Type parameter defaults, `TypeIs`, typed-dictionary openness/read-only/extra-item rules, `Annotated` alias behavior, protocol runtime checks, and checker convergence may have advanced since baseline knowledge.
4. **Packaging standards.** Lock-file standards, dependency groups, license expression/file metadata, editable behavior, attestations, externally managed environments, and wheel tags evolve outside the language release cadence.
5. **Alternative implementations.** Current compatibility for PyPy, GraalPy, MicroPython, and other runtimes is not detailed enough here for implementation-specific expected outcomes.
6. **Native interfaces.** Stable ABI coverage, subinterpreter/free-thread safety, limited API availability, and platform toolchains require current CPython and target-platform documentation.
7. **Framework metaprogramming.** ORMs, validation/model libraries, web routing, serialization, scientific array dispatch, and command frameworks each add distinctive generated entities. The checklist captures their common mechanisms, not every framework-specific contract.
8. **Operating-system edges.** Windows application execution aliases, macOS framework builds, POSIX fork/thread changes, musl versus glibc, mobile/WASI builds, and exotic filesystems need environment-specific verification.
9. **Warnings and removals.** Precise version in which an invalid escape, legacy import hook, typing alias, C API, or standard-library module moves from warning to error/removal should be checked per supported version.

Likely primary-source hierarchy for verification:

- Python Language Reference and Python Standard Library documentation for language and standard runtime behavior.
- “What’s New In Python X.Y” plus the cited PEP for version transitions and design-level normative detail.
- CPython C API documentation and CPython build/free-threading documentation for native and implementation contracts.
- Python Packaging Authority (PyPA) specifications for core metadata, names, dependencies, versions, build interfaces, distributions, tags, entry points, and direct URLs.
- Typing specification first, then `typing` documentation and relevant typing PEPs; individual checker documentation only for tool-specific behavior.
- Official documentation for the selected build backend, frontend/resolver, test framework, generator/transpiler, freezer, and alternative interpreter.

## Final coverage audit

| Feature family | Covered by | Residual risk or possible omission |
| --- | --- | --- |
| Source decoding, grammar, compile contexts, AST/code compilation | `PY-SRC-*`, `PY-DIAG-001` | Incremental/interactive parser recovery and every minor-version diagnostic are intentionally not enumerated. |
| Module identity, package topology, resolution, loading, cache, resources | `PY-MOD-*`, `PY-INT-003`, `PY-INT-004` | Embedded-host import configuration and legacy loader APIs receive only indirect coverage. |
| Lexical binding, scopes, dynamic namespaces, introspection | `PY-BIND-*` | Exact symbol-table internals and debugger mutation of frames remain implementation details. |
| Functions, decorators, descriptors, generators, coroutines | `PY-CALL-*`, `PY-OBJ-*`, `PY-INT-001` | C-level vectorcall and framework-specific wrapper signature tricks are not deeply specified. |
| Classes, metaclasses, inheritance, layout, identity, lifetime | `PY-OBJ-*`, `PY-PROT-*`, `PY-FLOW-008` | Exotic extension-type layout and object resurrection sequences require native cases. |
| Operator and protocol dispatch | `PY-PROT-*`, `PY-DYN-*` | Domain protocols such as NumPy array coercion/ufunc dispatch are ecosystem-specific and not selected here. |
| Type annotations, static relationships, stubs, generated members | `PY-TYPE-*`, `PY-INT-001`–`PY-INT-003` | Latest checker-specific inference and Python 3.14 annotations are high-priority verification gaps. |
| Exceptions, control flow, concurrency, cancellation | `PY-FLOW-*`, `PY-INT-006`, `PY-INT-007` | Distributed concurrency and third-party event-loop semantics are outside self-contained core coverage. |
| Dynamic discovery, registries, plugins, injection, proxies | `PY-DYN-*`, `PY-PKG-003`, `PY-PKG-004` | Individual web, object-relational mapping (ORM), command-line interface (CLI), and validation frameworks can add contracts beyond the generic mechanisms. |
| Packaging metadata, builds, artifacts, resolution, environments | `PY-PKG-*`, `PY-INT-004`, `PY-INT-005` | Emerging lock/attestation/license standards and frontend-specific resolution remain fluid. |
| Native/generated/bundled sources and alternative runtimes | `PY-NATIVE-*`, `PY-INT-008` | One project cannot practically exercise every compiler, freezer, ABI, and implementation; each retained variant needs current official verification. |
| Platform, filesystem, locale, subprocess, architecture | `PY-PLAT-*`, `PY-FLOW-010`, `PY-FLOW-012` | Mobile, WebAssembly System Interface (WASI), embedded, and unusual filesystem targets are only acknowledged, not itemized. |
| Diagnostics, warnings, testing, instrumentation, security | `PY-DIAG-*` | Fuzzing, property-based testing, mutation testing, and security scanners are analysis processes rather than Python semantic feature families, so they are not checklist items. |
| Cross-feature composition | `PY-INT-*` | Further combinations are combinatorial; retained cases target interactions where constituent-only coverage would miss a distinct result. |

No obvious core Python feature family remains wholly absent. The main unresolved boundary is framework-specific metaprogramming: scientific computing, object-relational mapping, validation/models, web routing, serialization, and command frameworks each warrant additional candidates only if those ecosystems enter the intended project scope. The current report captures the language and packaging mechanisms those frameworks commonly build upon without selecting a project theme or third-party stack.
