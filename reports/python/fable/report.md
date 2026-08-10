# Python — Research Checklist for a Code-Graph Reference Project

Independent research pass. Language: **Python**. Produced from model knowledge only;
no external sources, repositories, or downstream context were consulted.

## 1. Purpose and Method

This report enumerates Python language, toolchain, build, packaging, and ecosystem
features that a broad, idiomatic, self-contained reference project should exercise so
that a code-graph generator's accuracy and completeness can be evaluated against it.

Inclusion criterion: an item is listed only if omitting it would leave materially
distinct behavior unrepresented — distinct semantic entities, entity identity rules,
relationships (definition, reference, import, inheritance, dispatch, override,
re-export), name resolution, types, visibility, source inclusion, or diagnostics.
Grammar productions and standard-library API breadth are deliberately not enumerated.

Categories are derived from how Python itself partitions semantics (import system,
execution model, data model, typing system, packaging specs, ecosystem conventions),
not from a language-neutral taxonomy. Identifiers (e.g., `IMP-03`) are report-local
and stable within this document only.

Each item records: behavior; observable project content that would demonstrate it;
variants/interactions/failure cases; constraints; whether it is defined by the
language, an official implementation, or ecosystem convention; confidence; and a
likely primary source for verification (named conservatively; no invented sections).

## 2. Baseline Version and Implementation Assumptions

- **Implementation:** CPython. Alternative implementations (PyPy, GraalPy,
  MicroPython) are treated as variance, not baseline.
- **Language baseline:** Python 3.12, with deltas noted across 3.8–3.14. A reference
  project can reasonably require ≥3.10 or ≥3.12 and gate newer syntax per file.
- **Packaging baseline:** `pyproject.toml`-driven builds per PyPA specifications
  (PEP 517/518/621), with setuptools or hatchling as representative backends and
  pip/uv as representative installers.
- **Typing baseline:** the standalone Python typing specification and `typing`
  module semantics; no specific type checker is assumed.
- **Source encoding:** UTF-8 (default since Python 3).
- **Environment:** one virtual environment per project; single-platform execution
  with platform-conditional code present in source.

---

## 3. Checklist

### 3.1 Modules, Packages, and the Import System (IMP)

**IMP-01. Regular packages and `__init__.py` execution**
- Behavior: a directory with `__init__.py` is a package; the init module executes on
  first import; names it defines are attributes of the package entity itself.
- Evidence: a multi-level package tree where `__init__.py` files define names,
  execute statements, and aggregate submodule APIs.
- Variants/failures: empty `__init__.py` vs. code-bearing; init code raising on
  import; importing `pkg.sub` also sets attribute `sub` on `pkg` as a side effect.
- Constraints: none beyond Python 3.
- Defined by: language (import system). Confidence: high.
- Source: Python Language Reference, "The import system".

**IMP-02. Implicit namespace packages (PEP 420)**
- Behavior: directories without `__init__.py` form namespace packages; one import
  package may be assembled from multiple directories ("portions") across `sys.path`.
- Evidence: a namespace package whose portions live under two different source
  roots; both contribute submodules to the same package identity.
- Variants/failures: a regular package with the same name shadows all namespace
  portions; namespace packages have no `__file__`; tools frequently misresolve them.
- Constraints: Python ≥3.3.
- Defined by: language (PEP 420). Confidence: high.
- Source: PEP 420; Language Reference, "The import system".

**IMP-03. Import statement binding semantics**
- Behavior: `import a.b.c` initializes `a`, `a.b`, and `a.b.c` but binds only `a`
  in the importer's namespace; `import a.b.c as m` binds `m` to the leaf module;
  `from a.b import c` binds `c` only.
- Evidence: modules exercising all three forms against the same package, plus
  usages that only resolve through the dotted chain (`a.b.c.f()`).
- Variants/failures: `import a.b as m` resolving via attribute lookup (tolerates
  partially initialized modules since 3.7); aliasing creating multiple names for
  one module entity.
- Constraints: none.
- Defined by: language. Confidence: high.
- Source: Language Reference, "The import statement".

**IMP-04. Relative imports and execution mode**
- Behavior: `from . import x` / `from ..pkg import y` resolve against `__package__`;
  they fail when a module is executed as a top-level script.
- Evidence: intra-package relative imports at several depths; a module that is
  runnable via `python -m pkg.mod` but not via direct path execution.
- Variants/failures: relative import beyond the top-level package raises
  ImportError; `python -m` vs. direct-file execution changes resolvability.
- Constraints: Python 3 absolute-import default (PEP 328).
- Defined by: language (PEP 328, PEP 366). Confidence: high.
- Source: PEP 328; Language Reference, "The import statement".

**IMP-05. `from pkg import name`: attribute vs. submodule ambiguity**
- Behavior: `from pkg import name` first tries an attribute of the loaded package,
  then falls back to importing submodule `pkg.name`; an `__init__.py` attribute can
  shadow a same-named submodule, and outcome depends on init execution order.
- Evidence: a package containing both an attribute and a submodule with the same
  name; imports that resolve to each depending on form.
- Variants/failures: shadowing flips the resolved entity kind (object vs. module);
  a graph asserting "submodule" when the attribute wins is wrong.
- Constraints: none.
- Defined by: language + CPython import details. Confidence: high.
- Source: Language Reference, "The import system".

**IMP-06. Wildcard imports and `__all__`**
- Behavior: `from mod import *` imports `__all__` if defined, else all names not
  starting with underscore; `__all__` also declares the conventional public API.
- Evidence: one module with explicit `__all__` (including a name defined only
  dynamically), one without; a consumer using star import.
- Variants/failures: `__all__` computed or extended at runtime; `__all__` listing a
  missing name makes star import raise AttributeError; underscore-prefixed names
  excluded without `__all__`; star import permitted only at module scope.
- Constraints: none.
- Defined by: language; public-API meaning is ecosystem convention. Confidence: high.
- Source: Language Reference, "The import statement".

**IMP-07. Re-export and API-facade patterns**
- Behavior: names imported into `__init__.py` (or a facade module) become part of
  that module's surface; `from .impl import name as name` is the typing-spec
  convention for explicit re-export.
- Evidence: an `__init__.py` aggregating from private submodules (`_impl.py`);
  consumers importing only from the package root; redundant-alias re-exports.
- Variants/failures: same object reachable under multiple qualified names (identity
  vs. binding); underscore-module convention hiding implementation; re-export of a
  third-party name.
- Constraints: redundant-alias meaning is significant to type checkers.
- Defined by: language mechanics; convention from typing spec/PEP 484. Confidence: high.
- Source: typing specification (import/re-export rules); PEP 484.

**IMP-08. Conditional and guarded imports**
- Behavior: imports inside `try/except ImportError` (optional dependency with
  fallback), or gated on `sys.version_info` / `sys.platform`, make module-level
  bindings environment-dependent.
- Evidence: an optional-dependency import with a pure-Python fallback binding the
  same name; version-gated import of a module that moved between versions.
- Variants/failures: both branches binding the same name to different entities; the
  fallback defining a subset API; static resolution must represent alternatives.
- Constraints: none.
- Defined by: language mechanics; pattern is ecosystem convention. Confidence: high.
- Source: Language Reference, "The import statement"; stdlib usage patterns.

**IMP-09. Function-local (deferred) imports**
- Behavior: an import statement inside a function binds locally at call time; used
  for cycle-breaking, optional heavy dependencies, and startup cost.
- Evidence: a function importing a sibling module inside its body, where module
  scope has no corresponding import.
- Variants/failures: the imported name invisible at module scope; repeated calls
  reuse `sys.modules`; interaction with circular imports (IMP-10).
- Constraints: none.
- Defined by: language. Confidence: high.
- Source: Language Reference, "The import system".

**IMP-10. Circular imports and partial initialization**
- Behavior: modules in an import cycle observe each other partially initialized;
  `import a` in a cycle can succeed where `from a import f` raises ImportError
  because `f` is not yet bound.
- Evidence: two modules with a deliberate cycle resolved via deferred access or
  function-local import; a comment-free structural demonstration of both forms.
- Variants/failures: cycle through package `__init__`; cycle broken by
  `TYPE_CHECKING` import (TYP-07); error message attribution ("partially
  initialized module", 3.8+).
- Constraints: none.
- Defined by: language + CPython behavior details. Confidence: high.
- Source: Language Reference, "The import system".

**IMP-11. Module-level `__getattr__` and `__dir__` (PEP 562)**
- Behavior: a module may define `__getattr__(name)` and `__dir__()`; attributes can
  be produced lazily or virtually, with no static definition site.
- Evidence: a module exposing a lazily loaded submodule attribute and a deprecated
  alias via module `__getattr__`.
- Variants/failures: names reachable at runtime but absent from static analysis;
  `__all__` interplay; deprecation warnings on access.
- Constraints: Python ≥3.7.
- Defined by: language (PEP 562). Confidence: high.
- Source: PEP 562.

**IMP-12. Dynamic import with computed names**
- Behavior: `importlib.import_module(name)` and `__import__` resolve modules from
  runtime strings; bindings appear via assignment, not import statements.
- Evidence: a driver that imports plugin modules from a computed dotted string and
  reads an attribute off the result.
- Variants/failures: string built by concatenation/f-string (unresolvable
  statically); import into `globals()`; failure paths caught and logged.
- Constraints: none.
- Defined by: stdlib (importlib), official implementation. Confidence: high.
- Source: importlib documentation.

**IMP-13. Import-time side effects**
- Behavior: module top-level code runs exactly once per process at first import;
  projects use this for registration, monkey-patching, and configuration.
- Evidence: a module whose import registers a class into a registry defined in
  another module; observable ordering dependency between imports.
- Variants/failures: side effects skipped if the module is never imported;
  double-execution when a module is imported under two names (IMP-14).
- Constraints: none.
- Defined by: language execution model. Confidence: high.
- Source: Language Reference, "Execution model"; "The import system".

**IMP-14. Script/module dual identity: `__main__`, `-m`, `__main__.py`**
- Behavior: an executed file becomes module `__main__`; the same file imported by
  dotted name creates a *second* module object with duplicate class/function
  entities; `python -m pkg` runs `pkg/__main__.py`; `__name__ == "__main__"`
  guards entry points.
- Evidence: a package with `__main__.py`; a module with a main guard invoking a
  CLI function; `__package__`/`__spec__` reliance.
- Variants/failures: isinstance failures across the two identities; runpy
  execution; zipapp `__main__.py`.
- Constraints: none.
- Defined by: language + CPython (runpy). Confidence: high.
- Source: Language Reference, "Top-level components"; runpy documentation.

**IMP-15. Module shadowing and name collisions**
- Behavior: `sys.path` order decides resolution; a local file named like a stdlib
  or third-party module (e.g., a local `json.py`) shadows it for the whole process.
- Evidence: a deliberately named local module that shadows a stdlib name in one
  source root, with a consumer importing it; a package/module pair with colliding
  names in different roots.
- Variants/failures: script-directory prepending to `sys.path` makes shadowing
  execution-mode dependent; regular package beats namespace package.
- Constraints: CPython path initialization details.
- Defined by: language mechanics + implementation path setup. Confidence: high.
- Source: Language Reference, "The import system"; `sys.path` documentation.

**IMP-16. Import hooks, custom finders/loaders, `.pth` files**
- Behavior: `sys.meta_path` finders and path hooks (PEP 302/451) can materialize
  modules with no on-disk source, rewrite sources, or redirect resolution; `.pth`
  files in site-packages extend paths and may execute code at startup.
- Evidence: a small meta-path finder that serves a generated module; a `.pth` file
  contributed by an editable install (PKG-06).
- Variants/failures: pytest's assertion-rewriting hook as the mainstream in-the-wild
  example; loaders creating modules whose `__file__` is virtual.
- Constraints: Python ≥3.4 for ModuleSpec model.
- Defined by: language (PEPs 302/451) + CPython (site module). Confidence: high.
- Source: PEP 451; importlib documentation; site module documentation.

### 3.2 Name Binding and Scoping (SCO)

**SCO-01. LEGB resolution and compile-time locals**
- Behavior: a name assigned anywhere in a function is local throughout the function
  body; reading it before assignment raises UnboundLocalError even if a global of
  the same name exists.
- Evidence: a function shadowing a module global by later assignment; the
  same name resolving globally in a sibling function without assignment.
- Variants/failures: augmented assignment triggering localness; conditional
  assignment paths; the same identifier meaning different entities in adjacent
  functions.
- Constraints: none.
- Defined by: language (execution model). Confidence: high.
- Source: Language Reference, "Execution model" (naming and binding).

**SCO-02. `global` and `nonlocal` declarations**
- Behavior: `global` rebinding a module-level name from a function; `nonlocal`
  rebinding an enclosing-function local from a nested function.
- Evidence: a counter/accumulator pair demonstrating both; a nested closure that
  mutates enclosing state.
- Variants/failures: `nonlocal` with no matching enclosing binding is a
  SyntaxError; `global` for a name never assigned at module scope creates it on
  first write.
- Constraints: none.
- Defined by: language. Confidence: high.
- Source: Language Reference, "Simple statements" (`global`, `nonlocal`).

**SCO-03. Closures, cell variables, and late binding**
- Behavior: nested functions capture variables (cells), not values; loop-created
  closures observe the final loop value unless a default-argument snapshot or a
  factory function is used.
- Evidence: a list of closures created in a loop showing late binding, alongside
  the default-arg idiom capturing per-iteration values.
- Variants/failures: `__closure__`/free variables as identity evidence; closures
  escaping their defining function; shared mutable cell between sibling closures.
- Constraints: none.
- Defined by: language. Confidence: high.
- Source: Language Reference, "Execution model".

**SCO-04. Comprehension scopes**
- Behavior: comprehensions and generator expressions have their own function-like
  scope; only the outermost iterable is evaluated in the enclosing scope; in class
  bodies, inner parts of a comprehension cannot see class-level names.
- Evidence: a comprehension in a class body that references a class attribute in
  the outermost iterable (legal) contrasted with a form that would fail;
  nested comprehensions.
- Variants/failures: loop variable does not leak (unlike `for` statements); walrus
  inside a comprehension binds in the *enclosing* scope (SCO-07); CPython 3.12
  inlines comprehension frames (PEP 709) without changing scoping.
- Constraints: Python 3 semantics.
- Defined by: language. Confidence: high.
- Source: Language Reference, "Displays for lists, sets and dictionaries".

**SCO-05. Class body is not an enclosing scope for methods**
- Behavior: names defined in a class body are not visible as bare names inside
  method bodies; access requires `self.`, `cls.`, or `ClassName.`.
- Evidence: a class attribute used correctly via qualified access, and a
  same-named module global demonstrating what a bare name in a method resolves to.
- Variants/failures: a method's bare-name reference resolving to a module global
  rather than the class attribute — a classic mis-resolution trap for graphs.
- Constraints: none.
- Defined by: language. Confidence: high.
- Source: Language Reference, "Execution model".

**SCO-06. Private name mangling (`__name`)**
- Behavior: identifiers with two leading underscores (and at most one trailing) in
  a class context are rewritten to `_ClassName__name`, including in nested code in
  the class body; mangling makes "private" attributes per-class entities.
- Evidence: base and subclass each defining `__attr` with distinct storage; an
  external access via the mangled name; a mangled method reference.
- Variants/failures: mangling applies to any identifier position in the class body
  (including imports); no mangling for `__dunder__` names; subclass cannot
  accidentally override a mangled parent attribute.
- Constraints: none.
- Defined by: language. Confidence: high.
- Source: Language Reference, "Expressions" (identifiers/private name mangling).

**SCO-07. Assignment expressions (walrus, PEP 572)**
- Behavior: `:=` binds a name inside an expression; within comprehensions it binds
  in the containing scope, not the comprehension scope.
- Evidence: walrus in a `while` condition, in an `if`, and inside a comprehension
  whose binding is used after the comprehension.
- Variants/failures: illegal as comprehension loop-variable target; parenthesization
  requirements; creates bindings a purely statement-oriented scanner misses.
- Constraints: Python ≥3.8.
- Defined by: language (PEP 572). Confidence: high.
- Source: PEP 572.

**SCO-08. Unbinding and scope-limited bindings**
- Behavior: `del x` removes a binding; `except E as e:` binds `e` only within the
  handler and deletes it at block exit; `match` capture patterns bind names that
  persist after the statement.
- Evidence: an exception handler whose alias is provably absent after the block; a
  `del` of a module-level name; match captures used after the `match`.
- Variants/failures: reading a deleted name raises NameError/UnboundLocalError;
  `_` wildcard in match binds nothing.
- Constraints: except-alias deletion is Python 3 behavior.
- Defined by: language. Confidence: high.
- Source: Language Reference, "The try statement"; "The match statement".

**SCO-09. Shadowing builtins**
- Behavior: module-, class-, or function-level names may shadow builtins (`list`,
  `id`, `type`); resolution falls back to `builtins` only when no other binding
  exists.
- Evidence: a function parameter named `type` used alongside a genuine builtin use
  elsewhere; a module-level `id` variable.
- Variants/failures: annotations shadowed by local names; a graph must resolve the
  same identifier to different entities per scope.
- Constraints: none.
- Defined by: language. Confidence: high.
- Source: Language Reference, "Execution model".

**SCO-10. Annotation-only declarations (PEP 526)**
- Behavior: `x: T` without a value creates no runtime binding; at module/class
  level the annotation is recorded in `__annotations__` (and, pre-3.14, its
  expression is evaluated); inside functions, local annotations are neither
  evaluated nor stored.
- Evidence: a class with annotated-but-unassigned attributes consumed by a
  dataclass-like mechanism; a module-level annotated declaration.
- Variants/failures: annotated name absent from runtime namespace but present as a
  declared entity; `ClassVar` marking; interplay with PEP 563/649 (TYP-01).
- Constraints: Python ≥3.6.
- Defined by: language (PEP 526). Confidence: high.
- Source: PEP 526.

### 3.3 Functions and Callables (FUN)

**FUN-01. Parameter kinds and default evaluation**
- Behavior: positional-only (`/`), keyword-only (`*`), `*args`, `**kwargs`, and
  defaults; default expressions are evaluated once at `def` time in the defining
  scope and shared across calls.
- Evidence: a signature using every parameter kind; a default referencing a module
  constant; the mutable-default idiom (`None` sentinel) next to a deliberate
  shared-default case.
- Variants/failures: call-site compatibility errors (TypeError) as diagnostics;
  defaults referencing earlier parameters is invalid; keyword arguments matching
  by name creates name-level references at call sites.
- Constraints: positional-only syntax requires ≥3.8 (PEP 570).
- Defined by: language. Confidence: high.
- Source: Language Reference, "Function definitions"; PEP 570.

**FUN-02. Lambdas and inline callables**
- Behavior: `lambda` creates anonymous function entities, typically bound via
  assignment or passed inline; `__name__` is `<lambda>` while `__qualname__`
  reflects nesting.
- Evidence: lambdas passed as `key=` arguments, stored in dicts, and assigned to
  module names.
- Variants/failures: identity/naming for anonymous entities; a lambda closing over
  loop variables (SCO-03).
- Constraints: none.
- Defined by: language. Confidence: high.
- Source: Language Reference, "Expressions" (lambda).

**FUN-03. Nested functions and factories**
- Behavior: functions defined inside functions are distinct per invocation of the
  enclosing function; factories return closures whose identity is dynamic though
  their definition site is static.
- Evidence: a factory returning configured closures; a helper function invisible
  outside its enclosing function.
- Variants/failures: `__qualname__` with `<locals>` segments; a returned inner
  function attached to a class after the fact (CLS-15).
- Constraints: none.
- Defined by: language. Confidence: high.
- Source: Language Reference, "Function definitions".

**FUN-04. Decorators: wrapping, replacing, metadata, arbitrary expressions**
- Behavior: decorator expressions are evaluated at definition time and applied
  bottom-up; the bound entity is whatever the decorator returns — a wrapper, the
  original, or an unrelated object; `functools.wraps` copies identity metadata and
  sets `__wrapped__`.
- Evidence: stacked decorators; a parameterized decorator factory; a registering
  decorator returning the function unchanged; a replacing decorator returning a
  class or non-callable; use of `functools.wraps` and one wrapper deliberately
  without it.
- Variants/failures: since 3.9 (PEP 614) any expression may be a decorator (e.g.,
  subscripted or attribute-chained); decorated name's runtime type differing from
  the `def`; `inspect.unwrap` chain; decorators altering signatures.
- Constraints: PEP 614 requires ≥3.9.
- Defined by: language; `wraps` by stdlib. Confidence: high.
- Source: Language Reference, "Function definitions"; PEP 614; functools docs.

**FUN-05. `typing.overload` sets**
- Behavior: multiple `@overload`-decorated stub bodies followed by one
  implementation; at runtime only the last definition is bound, while statically
  the overloads define the call signatures; `typing.get_overloads` (3.11+) exposes
  them at runtime.
- Evidence: an overloaded function in a `.py` file with `...`-bodied overloads and
  a real implementation; call sites exercising distinct overloads.
- Variants/failures: overloads defined only in a stub file (TYP-08); repeated
  same-name defs that are *not* overloads (ERR-04) as the contrast case.
- Constraints: `get_overloads` requires ≥3.11.
- Defined by: stdlib typing + typing spec. Confidence: high.
- Source: typing module documentation; typing specification (overloads).

**FUN-06. `functools.partial` and `partialmethod`**
- Behavior: derived callables that reference an underlying callable via the `func`
  attribute rather than via a definition; `partialmethod` participates in
  descriptor binding.
- Evidence: module-level partials used as handlers; a class using `partialmethod`
  to derive variants of a method.
- Variants/failures: call-graph edges pass through an object attribute; partials of
  bound methods; introspection differences from real functions.
- Constraints: none.
- Defined by: stdlib. Confidence: high.
- Source: functools documentation.

**FUN-07. `functools.singledispatch` / `singledispatchmethod`**
- Behavior: runtime dispatch on the first argument's type; `@f.register`
  implementations (via annotation or explicit type) form a dispatch family whose
  membership is established by decorator calls, possibly across modules.
- Evidence: a generic function with registrations in the defining module and in a
  separate module; a class using `singledispatchmethod`.
- Variants/failures: registration by annotation vs. explicit argument; dispatch on
  ABCs/virtual subclasses (CLS-07); registrations executed only if their module is
  imported (IMP-13).
- Constraints: annotation-based register ≥3.7; method form ≥3.8.
- Defined by: stdlib. Confidence: high.
- Source: functools documentation.

**FUN-08. Generators and `yield from`**
- Behavior: a `def` containing `yield` defines a generator function; calling it
  creates a generator without executing the body (deferred call-graph semantics);
  `yield from` delegates; return values surface via StopIteration; StopIteration
  raised inside a generator becomes RuntimeError (PEP 479).
- Evidence: generator functions, generator expressions, a delegating generator,
  and `send`/`close` usage.
- Variants/failures: generator vs. normal function distinguished only by body
  content; infinite generators; generator-based context managers (FLW-04).
- Constraints: PEP 479 semantics ≥3.7 unconditionally.
- Defined by: language (PEP 255/342/380/479). Confidence: high.
- Source: Language Reference, "Yield expressions"; PEP 380; PEP 479.

**FUN-09. Callable objects (`__call__`)**
- Behavior: instances of classes defining `__call__` are used where functions are
  expected, including as decorators; call sites dispatch through the type's
  `__call__`, not a function entity.
- Evidence: a callable-instance decorator; a callable strategy object passed as a
  callback.
- Variants/failures: classes themselves are callables (instantiation as implicit
  `__call__` on the metaclass); distinguishing instantiation from function calls
  is a core graph requirement.
- Constraints: none.
- Defined by: language (data model). Confidence: high.
- Source: Language Reference, "Data model" (`__call__`).

**FUN-10. First-class function references**
- Behavior: functions and methods referenced without call syntax — stored in dicts
  and lists (dispatch tables), passed as callbacks, compared, and re-exported;
  `obj.method` creates a bound-method object each access.
- Evidence: a dispatch-table dict mapping strings to functions; a callback passed
  to a higher-order function in another module; a bound method stored and invoked
  later.
- Variants/failures: reference-only edges (no call) vs. call edges; identity of
  bound methods (fresh object per access); `Class.method` vs. `instance.method`.
- Constraints: none.
- Defined by: language. Confidence: high.
- Source: Language Reference, "Data model".

**FUN-11. Caching and wrapper factories (`lru_cache`, `cache`, `cached_property`)**
- Behavior: stdlib decorators replace functions with wrapper objects carrying the
  original via `__wrapped__`; `cached_property` converts a method into a non-data
  descriptor that replaces itself with an instance attribute on first access.
- Evidence: an `lru_cache`-decorated module function; a class with
  `functools.cached_property`.
- Variants/failures: the runtime entity type differs from the source `def`;
  `cached_property` interacts with `__slots__` (requires `__dict__`) and with
  data-descriptor precedence (CLS-09).
- Constraints: `cache` ≥3.9; `cached_property` ≥3.8.
- Defined by: stdlib. Confidence: high.
- Source: functools documentation.

**FUN-12. Function metadata and attributes**
- Behavior: `__name__`, `__qualname__`, `__module__`, `__doc__`, `__defaults__`,
  and arbitrary user-set function attributes (`f.marker = ...`) carry identity and
  metadata; docstrings are the first statement expression.
- Evidence: functions with docstrings; a framework-style pattern setting attributes
  on functions from a decorator; qualname reflecting nesting.
- Variants/failures: metadata divergence when `wraps` is omitted (FUN-04);
  `-OO` stripping docstrings (TOL-03).
- Constraints: none.
- Defined by: language + CPython object model. Confidence: high.
- Source: Language Reference, "Data model" (callable types).

### 3.4 Classes and the Object Model (CLS)

**CLS-01. Class definitions; class vs. instance attributes**
- Behavior: class bodies execute top-to-bottom at definition time; names bound
  there become class attributes; instance attributes are created dynamically
  (typically in `__init__`) and shadow class attributes per instance.
- Evidence: a class with class-level defaults, instance attributes assigned in
  `__init__`, and a method reading both; an instance shadowing a class attribute.
- Variants/failures: class attribute mutated via the class vs. rebound via an
  instance; attributes assigned in methods other than `__init__`; attributes only
  ever set externally on instances.
- Constraints: none.
- Defined by: language. Confidence: high.
- Source: Language Reference, "Class definitions"; "Data model".

**CLS-02. Multiple inheritance and C3 MRO**
- Behavior: method/attribute resolution follows C3 linearization across all bases;
  diamond hierarchies resolve deterministically; an unlinearizable hierarchy raises
  TypeError at class creation.
- Evidence: a diamond hierarchy where the resolved method differs from naive
  depth-first expectation; `__mro__` consumed somewhere.
- Variants/failures: MRO conflict as a class-creation failure (ERR-05); mixin
  ordering changing behavior; `object` at the root.
- Constraints: none.
- Defined by: language (new-style classes / C3). Confidence: high.
- Source: Language Reference, "Data model"; the Python 2.3 MRO description
  (C3 linearization) as historically documented.

**CLS-03. `super()` semantics**
- Behavior: zero-argument `super()` works via the compiler-provided `__class__`
  cell in methods defined lexically in a class body; it dispatches to the *next
  class in the instance's MRO*, not the textual parent; two-argument form is
  explicit.
- Evidence: cooperative `__init__` chains across a diamond; a `super()` call whose
  target depends on the dynamic type; one explicit `super(Type, self)` use.
- Variants/failures: zero-arg `super()` failing in functions defined outside the
  class and attached later (CLS-15); `super()` in classmethods; missing
  cooperative call breaking the chain.
- Constraints: zero-arg form is CPython compiler magic specified for the language.
- Defined by: language + CPython compiler detail. Confidence: high.
- Source: built-in functions documentation (`super`); Language Reference.

**CLS-04. `classmethod` and `staticmethod`**
- Behavior: descriptor-based method kinds; classmethods receive the class (enabling
  alternate constructors honoring subclasses); staticmethods bind nothing.
- Evidence: an alternate constructor classmethod returning `cls(...)`; a static
  utility method; calls through both class and instance.
- Variants/failures: staticmethods becoming plain callables when accessed (3.10+
  made them directly callable); decorator stacking with `property`/`abstractmethod`.
- Constraints: none.
- Defined by: language/stdlib built-ins. Confidence: high.
- Source: built-in functions documentation; Language Reference, "Data model".

**CLS-05. Properties and multi-definition single entities**
- Behavior: `@property` plus `@x.setter` / `@x.deleter` produce *three* `def`
  statements with the same name forming one class-level descriptor entity;
  attribute reads/writes at call sites become method invocations.
- Evidence: a full getter/setter/deleter property; a read-only property; call
  sites using plain attribute syntax.
- Variants/failures: the same-name-multiple-defs pattern must not be flagged as
  redefinition (contrast ERR-04); property inherited and overridden partially in a
  subclass; abstract properties.
- Constraints: none.
- Defined by: language/stdlib. Confidence: high.
- Source: built-in functions documentation (`property`).

**CLS-06. `__slots__`**
- Behavior: `__slots__` replaces per-instance `__dict__` with fixed descriptors;
  declared slot names become class-level data descriptors; assigning undeclared
  attributes raises AttributeError.
- Evidence: a slotted class with declared attributes and a demonstration that
  arbitrary attributes are rejected; a subclass adding slots.
- Variants/failures: inheritance where one base lacks slots restores `__dict__`;
  `__weakref__` slot; conflict between a slot and a class attribute of the same
  name (ValueError at class creation); dataclass `slots=True` (CLS-17).
- Constraints: none.
- Defined by: language (data model). Confidence: high.
- Source: Language Reference, "Data model" (`__slots__`).

**CLS-07. Abstract base classes and virtual subclasses**
- Behavior: `abc.ABC`/`ABCMeta` with `@abstractmethod` blocks instantiation of
  incomplete classes at runtime; `SomeABC.register(Cls)` makes `Cls` a *virtual*
  subclass — `isinstance`/`issubclass` succeed with no inheritance edge in source.
- Evidence: an ABC with abstract and concrete methods; a conforming subclass; a
  registered virtual subclass; an attempted instantiation of an abstract class as
  a failure case.
- Variants/failures: abstract properties/classmethods; `__subclasshook__`;
  stdlib `collections.abc` interfaces implemented implicitly.
- Constraints: none.
- Defined by: stdlib (PEP 3119). Confidence: high.
- Source: abc module documentation; PEP 3119.

**CLS-08. Metaclasses and class-creation hooks**
- Behavior: `class C(Base, metaclass=M)` routes class creation through `M`;
  `__init_subclass__` runs in the base for each new subclass;
  `__set_name__` notifies descriptor instances of their owner and name;
  `__prepare__` can change the class-body namespace.
- Evidence: a metaclass that registers or validates subclasses; a base using
  `__init_subclass__(cls, **kwargs)` with class-keyword arguments in subclass
  headers; a descriptor relying on `__set_name__`.
- Variants/failures: metaclass conflict TypeError when bases have incompatible
  metaclasses (ERR-05); implicit metaclass inheritance; keyword arguments in the
  class statement header.
- Constraints: `__init_subclass__`/`__set_name__` ≥3.6 (PEP 487).
- Defined by: language (PEP 3115, PEP 487). Confidence: high.
- Source: Language Reference, "Data model" (metaclasses); PEP 487.

**CLS-09. Custom descriptors; data vs. non-data precedence**
- Behavior: objects with `__get__`/`__set__`/`__delete__` mediate attribute access
  from the class; data descriptors override instance `__dict__`, non-data
  descriptors are overridden by it.
- Evidence: a validating data descriptor (typed field) and a caching non-data
  descriptor, each with observable precedence vs. instance attributes.
- Variants/failures: functions themselves are non-data descriptors (method
  binding); `__set_name__` integration; descriptor on a slotted class.
- Constraints: none.
- Defined by: language (data model). Confidence: high.
- Source: Language Reference, "Data model" (implementing descriptors);
  the official Descriptor HowTo guide.

**CLS-10. Dynamic attribute hooks on classes**
- Behavior: `__getattr__` (fallback on miss), `__getattribute__` (intercepts all),
  and `__setattr__`/`__delattr__` make attribute existence a runtime question;
  proxies and delegation patterns produce attributes with no definition site.
- Evidence: a delegating proxy class forwarding unknown attributes to a wrapped
  object; a class synthesizing attributes from a schema dict.
- Variants/failures: static "attribute missing" conclusions invalidated;
  `hasattr` probing; `__getattr__` raising AttributeError to signal absence;
  infinite recursion pitfalls in `__getattribute__`.
- Constraints: none.
- Defined by: language (data model). Confidence: high.
- Source: Language Reference, "Data model" (customizing attribute access).

**CLS-11. Special-method dispatch and operator protocols**
- Behavior: operators, iteration, containment, context management, hashing,
  truthiness, and formatting compile to *implicit* special-method dispatch that is
  looked up on the *type* (bypassing instance dicts and instance `__getattr__`);
  defining `__eq__` without `__hash__` sets `__hash__` to None (unhashability as a
  synthesized change).
- Evidence: a value class implementing arithmetic (`__add__`/`__radd__`),
  comparison, `__iter__`, `__contains__`, `__enter__`/`__exit__`, `__repr__`,
  `__bool__`; operator call sites with no explicit method call.
- Variants/failures: reflected-operand fallback; NotImplemented protocol;
  `functools.total_ordering` synthesizing comparison methods; `__index__`,
  `__getitem__` for both subscription and legacy iteration fallback.
- Constraints: none.
- Defined by: language (data model). Confidence: high.
- Source: Language Reference, "Data model" (special method lookup).

**CLS-12. Class decorators**
- Behavior: decorators applied to classes may register, mutate, wrap, or replace
  the class object; the bound name may end up referring to a different class than
  the one in the `class` statement.
- Evidence: a registration decorator; `functools.total_ordering`;
  `typing.runtime_checkable` on a Protocol; a decorator returning a subclass or
  wrapper.
- Variants/failures: `@dataclass(slots=True)` returning a *new* class object
  (identity change, CLS-17); stacked class decorators.
- Constraints: none.
- Defined by: language. Confidence: high.
- Source: Language Reference, "Class definitions".

**CLS-13. Nested and function-local classes**
- Behavior: classes may be defined inside classes or functions; local classes are
  created per call; `__qualname__` encodes lexical nesting including `<locals>`.
- Evidence: an inner configuration/Meta class inside another class; a class
  defined inside a factory function and returned.
- Variants/failures: inner classes are attributes, not scopes, for method name
  resolution; per-call class identities defeating global uniqueness assumptions.
- Constraints: none.
- Defined by: language. Confidence: high.
- Source: Language Reference, "Class definitions".

**CLS-14. Dynamic class creation**
- Behavior: `type(name, bases, ns)` and `types.new_class` create classes with no
  `class` statement; `enum`'s functional API and `collections.namedtuple` do this
  under the hood.
- Evidence: a `type(...)` call building a class from a spec dict; a
  `namedtuple("Point", "x y")`; an `Enum("Color", "RED GREEN")`.
- Variants/failures: the class's `__module__`/`__qualname__` may not match any
  source location; string-derived member/field names.
- Constraints: none.
- Defined by: language + stdlib. Confidence: high.
- Source: built-in functions documentation (`type`); types module documentation.

**CLS-15. Post-definition mutation (monkey-patching)**
- Behavior: methods and attributes may be added to or replaced on classes,
  instances, and modules after definition; `Cls.method = func` retroactively
  changes dispatch for all instances.
- Evidence: a module that attaches an externally defined function as a method; an
  instance-level method override; a patch applied at import time (IMP-13).
- Variants/failures: zero-arg `super()` broken in externally attached functions
  (CLS-03); patched entities' definition site vs. attachment site; patches applied
  conditionally.
- Constraints: none.
- Defined by: language mechanics; pattern is ecosystem convention. Confidence: high.
- Source: Language Reference, "Data model".

**CLS-16. Enumerations**
- Behavior: `Enum` subclasses convert class attributes into singleton member
  instances via the metaclass; equal values create aliases (one member, extra
  names); enums with members cannot be subclassed further; `auto()` synthesizes
  values.
- Evidence: an `Enum` with explicit values, an alias, `auto()`, methods on the
  enum, and a `Flag` used with bitwise operators; iteration over members.
- Variants/failures: members are instances of the class, not plain attributes;
  `IntEnum`/`StrEnum` mixing (StrEnum ≥3.11); `_ignore_`; functional API
  (CLS-14); notable str()/format() behavior changes across 3.10→3.12.
- Constraints: StrEnum ≥3.11; behavior details shifted in 3.11/3.12.
- Defined by: stdlib. Confidence: high (medium on cross-version formatting details).
- Source: enum module documentation.

**CLS-17. Dataclasses**
- Behavior: `@dataclass` reads `__annotations__` to synthesize `__init__`,
  `__repr__`, `__eq__`, ordering, and optionally `__hash__`; `field()` controls
  defaults/factories; `ClassVar` annotations are excluded; `InitVar` params flow to
  `__post_init__`; fields inherit and re-order across dataclass bases.
- Evidence: dataclasses with `field(default_factory=...)`, `frozen=True`,
  `kw_only=True`, `slots=True`, inheritance between dataclasses, and
  `__post_init__`.
- Variants/failures: synthesized methods have no source definition (generated-entity
  representation); `slots=True` returns a distinct class object; mutable default
  without factory raises ValueError at decoration; frozen + inheritance
  constraints.
- Constraints: `kw_only`/`slots` ≥3.10.
- Defined by: stdlib (PEP 557). Confidence: high.
- Source: dataclasses module documentation.

**CLS-18. NamedTuples and TypedDicts**
- Behavior: `typing.NamedTuple` class syntax builds tuple subclasses with named
  fields, defaults, and methods; `collections.namedtuple` builds them from
  strings; `TypedDict` classes describe dict shapes — instances are plain dicts
  and `isinstance` against the TypedDict is rejected.
- Evidence: a class-syntax NamedTuple with defaults and a method; a functional
  namedtuple; a TypedDict with `total=False` and its functional syntax;
  `_replace`/`_fields` usage.
- Variants/failures: TypedDict as a static-only shape with a runtime callable
  facade; NamedTuple field access both by name and index; inheritance rules for
  TypedDicts (merging keys).
- Constraints: `Required`/`NotRequired` ≥3.11; `ReadOnly` ≥3.13.
- Defined by: stdlib + typing spec (PEP 589 et al.). Confidence: high.
- Source: typing module documentation; collections documentation; PEP 589.

### 3.5 Static Typing Surface (TYP)

**TYP-01. Annotations: runtime representation and evaluation regime**
- Behavior: annotations on functions, classes, and modules are stored in
  `__annotations__`; by default (≤3.13) they are evaluated eagerly at definition
  time; `from __future__ import annotations` (PEP 563) stringizes them per-module;
  3.14 adopts lazy evaluation (PEP 649/749) with `annotationlib` formats.
- Evidence: one module using the future import and one not; annotations consumed
  at runtime (dataclass-like); `typing.get_type_hints` resolving strings.
- Variants/failures: string annotations referencing names not imported at runtime
  (TYP-07) break `get_type_hints`; per-module regime differences within one
  project; three regimes across versions is a core version-sensitivity.
- Constraints: PEP 563 opt-in 3.7+; PEP 649 default in 3.14.
- Defined by: language (PEPs 526/563/649/749). Confidence: high (3.14 details: medium).
- Source: PEP 563; PEP 649.

**TYP-02. Core typing constructs breadth**
- Behavior: `Optional`/`Union` and PEP 604 `X | Y`; PEP 585 builtin generics
  (`list[int]`); `Callable`; `Literal`; `Final`; `ClassVar`; `Annotated` (metadata
  that runtime consumers may read); `Never`/`NoReturn`; `Self`; `LiteralString`;
  `Unpack` with a TypedDict for `**kwargs` (PEP 692).
- Evidence: signatures and attribute annotations exercising each construct where
  it changes meaning — e.g., `Final` module constants, `ClassVar` in a dataclass,
  `Annotated` consumed by a runtime validator, `Self` in a fluent API.
- Variants/failures: `X | Y` at runtime requires ≥3.10 (or stringized
  annotations); deprecated `typing.List` aliases coexisting with builtin generics;
  `typing_extensions` as the backport source for older baselines.
- Constraints: per-construct minimums: 585/604 ≥3.9/3.10; `Self`, `LiteralString`
  ≥3.11; `Unpack`-kwargs ≥3.12.
- Defined by: stdlib + typing spec (many PEPs). Confidence: high.
- Source: typing module documentation; the typing specification.

**TYP-03. Type aliases: implicit, `TypeAlias`, and the `type` statement**
- Behavior: a module-level assignment of a type expression is an implicit alias;
  PEP 613 `TypeAlias` marks it explicitly; PEP 695 `type X = ...` (3.12) creates a
  lazily evaluated `TypeAliasType` object — a distinct entity kind that can be
  generic (`type Pair[T] = tuple[T, T]`).
- Evidence: all three alias forms, including a generic PEP 695 alias and a
  forward-referencing alias, with consumers in other modules.
- Variants/failures: distinguishing an alias from an ordinary assignment requires
  type-expression awareness; lazy evaluation means the alias body may reference
  names defined later.
- Constraints: `type` statement ≥3.12.
- Defined by: language/stdlib (PEP 613, PEP 695). Confidence: high.
- Source: PEP 695; typing module documentation.

**TYP-04. Type variables: legacy and PEP 695 scoped syntax**
- Behavior: legacy `TypeVar("T")` objects are module-level entities shared across
  uses; PEP 695 syntax (`def f[T](...)`, `class C[T]:`) introduces compiler-managed
  lexical scopes for type parameters and records them in `__type_params__`;
  bounds, constraints, variance (explicit legacy vs. inferred in 695), `ParamSpec`
  (PEP 612), `TypeVarTuple` (PEP 646), and defaults (PEP 696, 3.13).
- Evidence: a module using legacy TypeVars with bounds/constraints and another
  using PEP 695 generics for the same shapes; a ParamSpec-preserving decorator; a
  variadic generic container.
- Variants/failures: the same conceptual parameter as module entity vs. scoped
  syntax entity; annotation-scope name resolution for bounds (lazy); mixing styles.
- Constraints: 695 ≥3.12; 696 ≥3.13.
- Defined by: language + typing spec. Confidence: high.
- Source: PEP 695; PEP 612; PEP 646; PEP 696.

**TYP-05. Generic classes at runtime**
- Behavior: `Generic[T]` bases and PEP 695 headers make classes subscriptable;
  `C[int]` at runtime produces `types.GenericAlias`/`_GenericAlias` objects, not
  new classes; `__class_getitem__` customizes subscription; `__orig_bases__`
  preserves generic bases.
- Evidence: a generic container class defined and *used subscripted* in
  annotations, base-class lists (`class IntBox(Box[int])`), and runtime
  expressions.
- Variants/failures: subscripted base classes creating inheritance edges through
  alias objects; instantiation of a subscripted alias; erasure at runtime
  (instances know nothing of the parameter).
- Constraints: PEP 560 machinery ≥3.7.
- Defined by: language + stdlib. Confidence: high.
- Source: Language Reference, "Data model" (`__class_getitem__`); PEP 560.

**TYP-06. Protocols and structural subtyping**
- Behavior: `typing.Protocol` classes define structural interfaces; conforming
  classes need no inheritance edge; `@runtime_checkable` enables (method-presence
  only) `isinstance` checks.
- Evidence: a protocol with methods and attributes; a conforming class with no
  relationship in source; a runtime-checkable protocol used in `isinstance`; a
  class explicitly inheriting the protocol as the contrast case.
- Variants/failures: conformance is a derived (checker-computed) relationship, not
  a declared one — a key graph-representation decision; protocol members with
  default implementations; non-method member checks not enforced at runtime.
- Constraints: ≥3.8 (PEP 544).
- Defined by: stdlib + typing spec (PEP 544). Confidence: high.
- Source: PEP 544; typing module documentation.

**TYP-07. `TYPE_CHECKING` imports and forward references**
- Behavior: imports guarded by `if TYPE_CHECKING:` exist for static analysis only;
  at runtime the names are unbound, so annotations using them must be strings or
  the module must use the annotations future import; commonly used to break
  circular imports.
- Evidence: two modules with a type-only mutual dependency resolved via
  TYPE_CHECKING guards plus string annotations; a runtime consumer showing
  `get_type_hints` failure or success depending on namespace provisioning.
- Variants/failures: a graph must decide whether type-only imports are edges of a
  different kind; guard spelled via `typing.TYPE_CHECKING` or constant-folded
  equivalents.
- Constraints: none.
- Defined by: stdlib constant + typing spec convention. Confidence: high.
- Source: typing module documentation.

**TYP-08. Stub files and typed-package markers (PEP 561)**
- Behavior: `.pyi` stubs override `.py` sources for type checkers but are inert at
  runtime; `py.typed` marks an installed package as typed; stub-only distributions
  (`foo-stubs`) supply types for untyped packages; partial stubs are permitted.
- Evidence: a package shipping `py.typed`; one module accompanied by a `.pyi`
  whose declared surface deliberately differs in detail from runtime; a stub for a
  compiled extension (PKG-11).
- Variants/failures: dual sources of truth for one module identity (stub vs.
  runtime divergence as a failure case); stubs containing overloads absent from
  the `.py`; `.pyi` files must be included in source-inclusion decisions.
- Constraints: PEP 561 resolution order is checker-implemented.
- Defined by: typing spec (PEP 561); ecosystem (typeshed). Confidence: high.
- Source: PEP 561; the typing specification (distributing type information).

**TYP-09. `NewType`, `cast`, and narrowing functions**
- Behavior: `NewType("UserId", int)` is a distinct static type but a plain callable
  at runtime; `typing.cast` is a runtime no-op with static meaning; `TypeGuard`
  (PEP 647) and `TypeIs` (PEP 742, 3.13) mark user functions as narrowing
  predicates.
- Evidence: a NewType used across modules; a cast at an interface boundary; a
  narrowing predicate consumed in a conditional.
- Variants/failures: static-only entities/relationships with trivial runtime
  footprint; `isinstance`-based narrowing as the built-in contrast.
- Constraints: TypeIs ≥3.13 (or typing_extensions).
- Defined by: stdlib + typing spec. Confidence: high.
- Source: typing module documentation; PEP 647; PEP 742.

**TYP-10. Checker-directive decorators: `@override`, `@final`, `@deprecated`**
- Behavior: `@typing.override` (3.12, PEP 698) asserts an override relationship;
  `@typing.final`/`typing.Final` prohibit override/reassignment;
  `@warnings.deprecated` (3.13, PEP 702) marks entities deprecated with both
  static and runtime effect.
- Evidence: an overriding method marked `@override` (and one mismatch case as a
  diagnostic); a `@final` class and method; a deprecated function still called
  somewhere.
- Variants/failures: these encode relationship *assertions* that can disagree with
  structure — valuable for diagnostics evaluation.
- Constraints: version minimums as noted; typing_extensions backports.
- Defined by: stdlib + typing spec (PEPs 698, 702, 591). Confidence: high.
- Source: PEP 698; PEP 702; PEP 591.

### 3.6 Control-Flow Constructs with Entity Semantics (FLW)

**FLW-01. Structural pattern matching (`match`)**
- Behavior: class patterns reference class entities and use `__match_args__` for
  positional sub-patterns via `isinstance`; *bare names are capture patterns that
  bind*, while *dotted names are value lookups* — an asymmetric resolution rule;
  or-patterns, guards, mapping/sequence patterns, and `as` captures all bind names.
- Evidence: a `match` over a small class hierarchy with `__match_args__`, a
  constant-vs-capture contrast (`case Color.RED` vs. `case red`), guards, and
  nested patterns.
- Variants/failures: `match`/`case` are soft keywords (also valid identifiers
  elsewhere — parser-ambiguity test); or-pattern alternatives must bind identical
  names; wildcard `_` binds nothing.
- Constraints: ≥3.10 (PEPs 634–636).
- Defined by: language. Confidence: high.
- Source: PEP 634; Language Reference, "The match statement".

**FLW-02. Exception hierarchies, raising, and handling**
- Behavior: user exception classes form inheritance-driven handling relationships;
  `except (A, B) as e` matches by subclass; `raise X from Y` sets explicit cause;
  handlers create implicit control-flow edges from raise sites to handlers of
  matching types.
- Evidence: a project exception hierarchy with a base error; raises and handlers
  across module boundaries; re-raise, bare `raise`, and `raise ... from`.
- Variants/failures: catching a base class handles subclasses (relationship via
  MRO); custom exceptions with extra attributes; `finally` interaction.
- Constraints: none.
- Defined by: language. Confidence: high.
- Source: Language Reference, "The try statement"; "The raise statement".

**FLW-03. Exception groups and `except*`**
- Behavior: `ExceptionGroup` aggregates exceptions; `except*` clauses match and
  split groups by type, with multiple clauses potentially all executing.
- Evidence: code raising an ExceptionGroup handled by multiple `except*` clauses;
  interaction with a plain `except` contrast.
- Variants/failures: mixing `except` and `except*` in one `try` is a SyntaxError;
  asyncio TaskGroup as the mainstream producer.
- Constraints: ≥3.11 (PEP 654).
- Defined by: language. Confidence: high.
- Source: PEP 654.

**FLW-04. Context managers**
- Behavior: `with` dispatches implicitly to `__enter__`/`__exit__`;
  `@contextlib.contextmanager` turns a generator function into a context-manager
  factory (entity-kind transformation); `ExitStack` composes managers dynamically;
  parenthesized multi-item `with` groups (3.10+).
- Evidence: a class-based manager, a generator-based manager, `async with`
  (CON-01), and a multi-item parenthesized `with`.
- Variants/failures: `__exit__` swallowing exceptions changes flow; context
  managers returned from functions (dynamic manager identity);
  `contextlib.suppress`/`closing` wrappers.
- Constraints: parenthesized form ≥3.10 (grammar guaranteed ≥3.10).
- Defined by: language + stdlib. Confidence: high.
- Source: Language Reference, "The with statement"; contextlib documentation.

**FLW-05. Iteration protocol and unpacking**
- Behavior: `for` compiles to implicit `__iter__`/`__next__` dispatch; iterable
  unpacking (`a, *rest = ...`), starred call arguments, and dict unpacking
  (`**kwargs`) create multi-name bindings and implicit protocol references.
- Evidence: a custom iterable class consumed by `for`; starred unpacking in
  assignments and calls; nested-target unpacking in a loop header.
- Variants/failures: unpacking arity errors at runtime; `zip`/`enumerate` chains;
  fallback iteration via `__getitem__` for classes without `__iter__`.
- Constraints: none.
- Defined by: language. Confidence: high.
- Source: Language Reference, "The for statement"; "Assignment statements".

**FLW-06. Assignment target varieties and augmented assignment**
- Behavior: assignment targets include names, attributes (`obj.x = ...`, an
  implicit `__setattr__`), subscripts (`d[k] = ...`, `__setitem__`), and chained
  targets; augmented assignment dispatches to `__iadd__`-style methods with
  fallback and *rebinds the target*.
- Evidence: attribute and subscript writes on classes with custom setters;
  `x += 1` on immutables vs. an `__iadd__`-defining class.
- Variants/failures: augmented assignment on an attribute both reads and writes;
  tuple-target chained assignment; writes as reference kinds distinct from reads.
- Constraints: none.
- Defined by: language. Confidence: high.
- Source: Language Reference, "Assignment statements"; "Data model".

**FLW-07. f-strings and template strings**
- Behavior: f-string interpolations are full expressions referencing names inside
  string literals; PEP 701 (3.12) formalizes the grammar (nested same quotes,
  multiline, arbitrary nesting); the `=` debug specifier embeds source text;
  3.14 adds t-strings (PEP 750) producing Template objects — a new literal kind.
- Evidence: f-strings referencing locals/attributes/calls, nested format specs,
  the `=` form, and (if the baseline allows) a t-string.
- Variants/failures: name references inside literals are easily missed by lexical
  scanners; implicit adjacent-literal concatenation mixing f- and plain strings.
- Constraints: PEP 701 ≥3.12; t-strings ≥3.14.
- Defined by: language (PEP 498, 701, 750). Confidence: high (t-strings: medium).
- Source: PEP 498; PEP 701; PEP 750.

**FLW-08. Module-level executable code and definition order**
- Behavior: a module is a statement sequence; definitions are effects of
  execution, so order matters (a decorator, base class, or default must already be
  bound); module-level loops/conditionals can generate bindings.
- Evidence: module code computing constants, building lookup tables, and
  conditionally defining names before dependent definitions.
- Variants/failures: forward reference at module level raises NameError at import;
  names created inside module-level loops.
- Constraints: none.
- Defined by: language. Confidence: high.
- Source: Language Reference, "Top-level components"; "Execution model".

**FLW-09. Conditional and alternative definitions**
- Behavior: the same name may be given different `def`/`class` definitions under
  `if/else` branches keyed on version, platform, or feature detection — one name,
  multiple candidate definition sites, at most one live per environment.
- Evidence: a function defined two ways under a `sys.version_info` branch; a
  platform-conditional class; a fast-path/slow-path pair keyed on an optional
  import (IMP-08).
- Variants/failures: graphs must represent alternative definitions without
  declaring a redefinition error; dead branch for the analyzed environment.
- Constraints: none.
- Defined by: language mechanics; pattern is ecosystem convention. Confidence: high.
- Source: Language Reference, "Compound statements".

### 3.7 Runtime Dynamism and String-Mediated References (DYN)

**DYN-01. `exec`, `eval`, and `compile`**
- Behavior: code in strings can define or reference entities in supplied
  namespaces; definitions materialize with no source location in the file's AST.
- Evidence: a controlled `exec` building a function into a dict namespace which is
  then published as a module attribute; an `eval` of a stored expression.
- Variants/failures: namespace-dict plumbing (`globals()`/custom dicts);
  `namedtuple`-style stdlib internals as precedent; security-adjacent linters flag
  these — good diagnostics fodder.
- Constraints: none.
- Defined by: language. Confidence: high.
- Source: built-in functions documentation (`exec`, `eval`, `compile`).

**DYN-02. String-based attribute and namespace access**
- Behavior: `getattr`/`setattr`/`delattr`/`hasattr` with computed names, and
  mutation of `globals()`/`vars()` dicts, create and reference entities without
  identifier syntax.
- Evidence: config-driven `getattr(module, name)` dispatch; a loop `setattr`-ing
  fields from a schema; a `globals()[name] = value` export.
- Variants/failures: constant-string arguments (statically resolvable) vs. computed
  (not); `hasattr` feature probing steering control flow.
- Constraints: none.
- Defined by: language/stdlib. Confidence: high.
- Source: built-in functions documentation.

**DYN-03. Dotted-path string references**
- Behavior: many mechanisms reference entities by dotted strings resolved at
  runtime: entry points (`pkg.mod:func`), `unittest.mock.patch("pkg.mod.Name")`,
  logging config class paths, plugin registries, serialization hooks.
- Evidence: at least three distinct dotted-string reference forms in project
  content — packaging metadata, a mock patch in tests, and a config-driven factory.
- Variants/failures: stale strings after refactors (the central rename hazard);
  colon vs. dot separators per mechanism; strings assembled at runtime.
- Constraints: resolution semantics are per-mechanism.
- Defined by: ecosystem convention (mechanism-specific). Confidence: high.
- Source: PyPA entry-points specification; unittest.mock documentation;
  logging.config documentation.

**DYN-04. Generated source files**
- Behavior: build steps write Python sources consumed as ordinary modules —
  version files (setuptools-scm style), protocol/schema compiler outputs
  (protobuf-style `_pb2` modules), templating; generated modules may be heavily
  dynamic internally.
- Evidence: a generated `_version.py` referenced by the package `__init__`; a
  build script that writes a module into the source tree or build output.
- Variants/failures: source present only after build (source-inclusion decision);
  generated code style diverging from hand-written; regeneration divergence.
- Constraints: generator-specific.
- Defined by: ecosystem convention. Confidence: high.
- Source: setuptools-scm documentation; protobuf Python generated-code docs.

**DYN-05. Plugin discovery via entry points**
- Behavior: `importlib.metadata.entry_points()` enumerates entities declared in
  *other distributions'* metadata; the reference graph crosses package boundaries
  through metadata, not imports.
- Evidence: a host package defining an entry-point group and loading plugins; a
  sibling package in the project declaring an entry point into that group.
- Variants/failures: group-name filtering API changes across 3.9→3.12
  (dict-like vs. `select`); plugins absent until installed; dangling entry points.
- Constraints: `importlib.metadata` stable API ≥3.10-ish; backport
  `importlib_metadata` common.
- Defined by: stdlib + PyPA spec. Confidence: high.
- Source: importlib.metadata documentation; PyPA entry-points specification.

**DYN-06. Registry patterns**
- Behavior: decorators, `__init_subclass__`, and metaclasses append entities into
  module-level registries; lookup sites reference entities only through the
  registry data structure.
- Evidence: a decorator-populated handler registry consumed by a dispatcher in
  another module; an `__init_subclass__`-based auto-registration hierarchy.
- Variants/failures: registration depends on import having happened (IMP-13);
  keys as strings vs. types; duplicate-registration errors.
- Constraints: none.
- Defined by: ecosystem convention on language mechanics. Confidence: high.
- Source: Language Reference (decorators, PEP 487); common framework practice.

**DYN-07. Pickling and qualified-name identity**
- Behavior: pickle serializes classes/functions by module + qualname; only
  importable top-level (or stably qualified) entities round-trip; renames break
  persisted data; `__reduce__`/`copyreg` customize.
- Evidence: a picklable dataclass round-tripped in tests; a local class shown to
  fail pickling as the contrast.
- Variants/failures: multiprocessing spawn inherits this constraint (CON-03);
  `__main__`-defined entities pickling against the dual-identity problem (IMP-14).
- Constraints: none.
- Defined by: stdlib. Confidence: high.
- Source: pickle module documentation.

### 3.8 Concurrency and Async (CON)

**CON-01. Coroutines: `async def` / `await`**
- Behavior: `async def` defines a coroutine function; *calling it does not run the
  body* — it returns a coroutine awaited later (deferred call-graph edges, like
  generators); `await` points reference awaitables; `asyncio.run` bridges sync
  entry to async.
- Evidence: an async call chain across modules with a sync entry point; `await`
  on library coroutines and on custom `__await__` objects.
- Variants/failures: forgetting `await` (RuntimeWarning: coroutine never awaited)
  as a diagnostic; async and sync functions with the same name in one API;
  `asyncio.gather`/TaskGroup fan-out.
- Constraints: ≥3.5; TaskGroup ≥3.11.
- Defined by: language (PEP 492) + stdlib asyncio. Confidence: high.
- Source: PEP 492; Language Reference, "Coroutines".

**CON-02. Async generators, comprehensions, and async context/iteration**
- Behavior: `async def` with `yield` defines async generators; `async for` /
  `async with` dispatch to `__aiter__`/`__anext__` and `__aenter__`/`__aexit__`;
  async comprehensions embed awaits in displays.
- Evidence: an async generator consumed by `async for`; a class implementing the
  async context protocol; an async comprehension.
- Variants/failures: sync/async protocol pairs on one class; `yield` inside
  `async def` flipping the entity kind.
- Constraints: ≥3.6 (PEPs 525/530).
- Defined by: language. Confidence: high.
- Source: PEP 525; PEP 530.

**CON-03. Threads, processes, and start-method constraints**
- Behavior: `threading.Thread(target=f)` and executors reference callables as
  data; multiprocessing under spawn/forkserver pickles targets, requiring
  importable top-level functions and an `if __name__ == "__main__"` guard; default
  start method varies by platform (and changed away from fork on Linux in 3.14).
- Evidence: a worker function at module top level used by a process pool behind a
  main guard; a thread with a bound-method target.
- Variants/failures: lambdas/local functions failing under spawn; platform-varying
  behavior of identical source; free-threading interaction (TOL-07).
- Constraints: platform-dependent defaults; 3.14 default change (medium confidence).
- Defined by: stdlib + platform behavior. Confidence: high (3.14 detail: medium).
- Source: multiprocessing documentation.

### 3.9 Project Layout, Packaging, and Build (PKG)

**PKG-01. `pyproject.toml` core metadata (PEP 621)**
- Behavior: `[project]` declares name, version, `requires-python`, dependencies,
  and dynamic fields; `[build-system]` selects the backend; this file is the root
  artifact tying import packages to a distribution.
- Evidence: a complete `pyproject.toml` with static metadata, one `dynamic` field,
  and dependency specifiers.
- Variants/failures: dynamic version resolved by backend hooks; name
  normalization (PEP 503/508) differing from the import name (PKG-07).
- Constraints: PEP 517/518/621 tooling.
- Defined by: PyPA specifications. Confidence: high.
- Source: pyproject.toml specification (PyPA specs); PEP 621.

**PKG-02. Build backends and package discovery**
- Behavior: setuptools, hatchling, flit-core, and poetry-core discover/include
  packages differently (auto-discovery vs. explicit `packages`/`only-include`);
  what lands in the wheel — and thus what is importable when installed — is
  backend-configuration dependent.
- Evidence: backend configuration that includes one package and excludes a
  sibling directory; a data file included via backend config.
- Variants/failures: legacy `setup.py`/`setup.cfg` coexistence; a module present
  in the repo but excluded from the distribution (source-inclusion divergence
  between "project on disk" and "installed package").
- Constraints: backend-specific configuration tables.
- Defined by: ecosystem (per-backend), on PEP 517. Confidence: high.
- Source: setuptools documentation; hatchling documentation.

**PKG-03. src layout vs. flat layout**
- Behavior: `src/pkg/...` keeps the import package off the repo root so tests run
  against the installed/editable package; flat layout puts `pkg/` at the root and
  on `sys.path` when running from the repo — changing which copy resolves.
- Evidence: a src-layout project (and, if multiple roots exist in the fixture, a
  flat contrast); test runs that only work with the package installed.
- Variants/failures: accidental import of the repo copy vs. installed copy;
  `pythonpath` settings in test config papering over layout.
- Constraints: convention, enforced only by tooling defaults.
- Defined by: ecosystem convention (PyPA guidance). Confidence: high.
- Source: Python Packaging User Guide (src layout discussion).

**PKG-04. Console scripts and entry-point groups**
- Behavior: `[project.scripts]` / `[project.gui-scripts]` /
  `[project.entry-points."group"]` map names to `pkg.mod:attr` object references;
  installers synthesize executables that import and call the referenced entity.
- Evidence: a console script pointing at a `main` function; a custom entry-point
  group used for plugins (DYN-05).
- Variants/failures: dangling references not validated at build time; the
  referenced attribute being nested (`mod:Class.method`).
- Constraints: PyPA entry-points spec.
- Defined by: PyPA specifications. Confidence: high.
- Source: PyPA entry-points specification.

**PKG-05. Extras and environment markers**
- Behavior: `[project.optional-dependencies]` defines extras; PEP 508 markers
  (`python_version`, `sys_platform`, `platform_machine`, `extra`) make the
  dependency set environment-conditional — the resolvable third-party surface
  varies per environment.
- Evidence: an extra guarding an optional-import feature (IMP-08); a
  platform-marked dependency; a python-version-marked backport
  (`typing_extensions; python_version < "3.12"` shape).
- Variants/failures: code importing an extra's package unconditionally (latent
  ImportError); marker-driven differences between analyzed and executed
  environments. `[dependency-groups]` (PEP 735) as the newer dev-dependency shape.
- Constraints: PEP 508; PEP 735 support is tool-dependent (medium confidence).
- Defined by: PyPA specifications. Confidence: high.
- Source: PEP 508; pyproject specification.

**PKG-06. Editable installs (PEP 660)**
- Behavior: editable installs expose the working tree through `.pth` entries,
  import hooks, or proxy modules, so the "installed" package resolves to
  in-repo files; mechanism differs by backend.
- Evidence: a project designed to be installed editable, with tests importing the
  installed name.
- Variants/failures: editable hooks confusing path-based analysis (module
  `__file__` pointing into the repo); strict vs. lenient editable modes in
  setuptools.
- Constraints: PEP 660 backends.
- Defined by: PyPA specifications + backend implementations. Confidence: high.
- Source: PEP 660.

**PKG-07. Distribution name vs. import name**
- Behavior: the installable name (normalized, dash-friendly) and the import
  package name are independent; one distribution may ship multiple top-level
  import packages or modules.
- Evidence: a project whose dist name differs from its import name; a dist
  shipping both a package and a standalone top-level module.
- Variants/failures: `importlib.metadata` mapping from dist to modules;
  graphs conflating the two namespaces.
- Constraints: PEP 503 normalization.
- Defined by: PyPA specifications. Confidence: high.
- Source: Python Packaging User Guide; importlib.metadata documentation.

**PKG-08. Namespace package spanning distributions**
- Behavior: multiple distributions each contribute portions to one namespace
  package (`company.tools`, `company.data`); the package entity's contents depend
  on the installed set.
- Evidence: two sub-projects in the fixture workspace contributing to a shared
  namespace root without `__init__.py` at the shared level.
- Variants/failures: a stray `__init__.py` in one portion breaking the merge;
  legacy pkgutil/pkg_resources-style namespaces as historical variants.
- Constraints: PEP 420; packaging guidance for namespace packages.
- Defined by: language + PyPA convention. Confidence: high.
- Source: PEP 420; Python Packaging User Guide (namespace packages).

**PKG-09. Package data and `importlib.resources`**
- Behavior: non-Python files inside packages are accessed via
  `importlib.resources` (files()/as_file); inclusion depends on backend config;
  resources are part of the package's content surface.
- Evidence: a data file (template, JSON) inside a package read via
  importlib.resources; backend config including it.
- Variants/failures: `__file__`-relative access as the fragile contrast (breaks in
  zips); data missing from wheels when config omits it.
- Constraints: files() API ≥3.9 (stable ≥3.12 for some traversable APIs).
- Defined by: stdlib + backend config. Confidence: high.
- Source: importlib.resources documentation.

**PKG-10. Version single-sourcing**
- Behavior: version may live as `__version__` in source, be derived from VCS
  (setuptools-scm writing a file — DYN-04), or be read at runtime via
  `importlib.metadata.version(dist)`; each choice changes what entity carries the
  version and how it's referenced.
- Evidence: `__version__` in the package root plus a metadata-based lookup in
  another component.
- Variants/failures: drift between metadata and attribute; `dynamic = ["version"]`
  in pyproject.
- Constraints: none.
- Defined by: ecosystem convention. Confidence: high.
- Source: Python Packaging User Guide (single-sourcing the version).

**PKG-11. Compiled extension modules**
- Behavior: `.so`/`.pyd` extensions are modules with no Python source; Cython
  transpiles `.pyx` to C at build time; cffi builds bindings; a common idiom pairs
  an accelerated extension with a pure-Python fallback under one name; `.pyi`
  stubs supply the static surface for extensions.
- Evidence: a small extension (or Cython module) built by the backend, imported
  behind a try/except fallback to a pure module; a stub for it.
- Variants/failures: platform wheel tags; source-inclusion for `.pyx`/generated C;
  extension entities discoverable only via stubs or runtime introspection.
- Constraints: compiler toolchain at build; CPython ABI.
- Defined by: CPython C API + ecosystem tooling. Confidence: high.
- Source: CPython "Extending and Embedding" documentation; Cython documentation.

**PKG-12. Multi-project workspaces and local dependencies**
- Behavior: monorepos hold several distributions with cross-dependencies expressed
  as path dependencies or workspace members (uv/poetry/pdm); resolution differs
  between "workspace dev" and "published" modes; lock files pin the environment.
- Evidence: two fixture sub-projects where one depends on the other by path/
  workspace reference; a lock file present.
- Variants/failures: import resolution across sibling projects without
  installation; editable+workspace combinations; requirements.txt as legacy shape.
- Constraints: tool-specific (uv workspaces, poetry, pip-tools); fast-moving.
- Defined by: ecosystem convention. Confidence: medium.
- Source: uv documentation; poetry documentation.

**PKG-13. Single-file scripts with inline metadata (PEP 723)**
- Behavior: a standalone script may embed a `# /// script` metadata block with
  dependencies and requires-python, executed by runners (uv, pipx) in ephemeral
  environments — a compilation unit outside any package/pyproject.
- Evidence: one runnable script in the project carrying an inline metadata block
  and importing a third-party dependency declared only there.
- Variants/failures: script both standalone and imported (dual role); tools that
  ignore the block treat it as a comment.
- Constraints: PEP 723; runner support required.
- Defined by: PyPA specification (PEP 723). Confidence: high.
- Source: PEP 723.

**PKG-14. Tool configuration co-located in `pyproject.toml`**
- Behavior: `[tool.*]` tables configure type checkers, linters, formatters, and
  test runners; settings like mypy's namespace-package handling or pytest's
  `pythonpath` materially change how sources are resolved and interpreted.
- Evidence: `[tool.pytest.ini_options]`, a type-checker table, and a lint table
  with per-file ignores that reference project paths.
- Variants/failures: equivalent settings in `setup.cfg`/`tox.ini`/`.ini` files;
  config that changes effective import roots for tests.
- Constraints: per-tool schemas.
- Defined by: ecosystem convention. Confidence: high.
- Source: respective tool documentation (pytest, mypy, ruff).

### 3.10 Testing Conventions (TST)

**TST-01. pytest discovery and import modes**
- Behavior: files matching `test_*.py`/`*_test.py`, classes `Test*` (no
  `__init__`), and functions `test_*` are collected by convention, not by
  registration; rootdir/conftest logic and import mode (prepend/importlib) decide
  how test modules are imported; pytest's assertion rewriting is an import hook
  transforming test bytecode.
- Evidence: a tests tree exercising naming conventions, including one
  deliberately non-collected helper module.
- Variants/failures: tests importable under two identities depending on
  `__init__.py` presence in test dirs; rewritten asserts as an example of
  loaded-code ≠ on-disk-source.
- Constraints: pytest-specific.
- Defined by: ecosystem convention (pytest). Confidence: high.
- Source: pytest documentation (test discovery; import mechanisms).

**TST-02. Fixtures and `conftest.py`**
- Behavior: fixtures are resolved *by parameter name* against a hierarchy of
  `conftest.py` files and plugins — a name-resolution system parallel to imports,
  with no import statements linking definition and use; fixtures can override
  same-named fixtures from outer scopes; `conftest.py` is loaded specially.
- Evidence: fixtures defined in nested conftests, one overriding an outer one; an
  autouse fixture; a yield fixture with teardown; a test consuming fixtures from
  multiple levels.
- Variants/failures: fixture-name collisions; fixtures provided by installed
  plugins via entry points (DYN-05); request/factory fixtures.
- Constraints: pytest-specific.
- Defined by: ecosystem convention (pytest). Confidence: high.
- Source: pytest documentation (fixtures; conftest.py).

**TST-03. Parametrization and markers**
- Behavior: `@pytest.mark.parametrize` multiplies one test function into many test
  items with generated identities; markers (`skip`, `skipif`, `xfail`, custom)
  attach metadata that alters execution and reporting.
- Evidence: parametrized tests with explicit ids; a version-`skipif` tied to a
  version-gated feature module (TOL-04); a custom marker registered in config.
- Variants/failures: indirect parametrization through fixtures; one source entity
  ↔ many runtime test identities.
- Constraints: pytest-specific.
- Defined by: ecosystem convention. Confidence: high.
- Source: pytest documentation (parametrize; markers).

**TST-04. unittest-style tests**
- Behavior: `unittest.TestCase` subclasses with `test_*` methods, `setUp`
  /`tearDown`, and class-level fixtures form the stdlib testing model; discovery
  by module/class/method naming; pytest can collect them too.
- Evidence: one TestCase-based module alongside pytest-style tests, including
  inherited test methods from a shared base TestCase (tests acquired via MRO).
- Variants/failures: a base class with test methods and `Mixin` pattern to avoid
  double-collection; `subTest`.
- Constraints: stdlib.
- Defined by: stdlib. Confidence: high.
- Source: unittest documentation.

**TST-05. Mock patching by dotted string**
- Behavior: `unittest.mock.patch("pkg.mod.Name")` replaces attributes located by
  string during a scope; the correct target is *where the name is looked up*, not
  where it is defined — a from-import in the consumer means patching the
  consumer's module attribute.
- Evidence: tests patching the same underlying function at two different dotted
  locations to show the from-import distinction; `patch.object`;
  a `create=True` patch of a nonexistent attribute as a failure-mode probe.
- Variants/failures: autospec; patch as decorator vs. context manager; stale
  string targets after refactoring (rename hazard for graphs).
- Constraints: stdlib.
- Defined by: stdlib (mock). Confidence: high.
- Source: unittest.mock documentation ("where to patch").

**TST-06. Doctests**
- Behavior: executable examples inside docstrings (and text files) are code in
  string literals — references to project entities embedded in documentation,
  runnable by doctest/pytest.
- Evidence: a function whose docstring contains a passing doctest; a README-style
  text file with doctest examples wired into the test config.
- Variants/failures: doctests referencing renamed APIs (silent staleness if never
  run); ELLIPSIS/whitespace directives.
- Constraints: stdlib + runner integration.
- Defined by: stdlib. Confidence: high.
- Source: doctest documentation.

### 3.11 Interpreter and Toolchain Variance (TOL)

**TOL-01. CPython vs. alternative implementations**
- Behavior: the language reference leaves room for implementation differences
  (GC/finalization timing, object identity details, performance-motivated
  behaviors); PyPy et al. differ in refcounting semantics and C-extension support.
- Evidence: project docs/config declaring CPython; any reliance on prompt
  finalization (`__del__`) isolated and flagged.
- Variants/failures: code depending on refcount-immediate `__del__` breaks on
  PyPy; `sys.implementation` probing.
- Constraints: baseline assumption (Section 2).
- Defined by: language vs. implementation split. Confidence: high.
- Source: Language Reference (implementation notes); sys module documentation.

**TOL-02. Bytecode caching and source-less modules**
- Behavior: CPython caches bytecode in `__pycache__` (PEP 3147); modules can be
  distributed as `.pyc`-only (sourceless) and still import; cache invalidation is
  timestamp- or hash-based (PEP 552).
- Evidence: presence of `__pycache__` handling in ignores; optionally one
  sourceless module in a controlled corner as a source-inclusion probe.
- Variants/failures: graphs keyed on `.py` files miss sourceless modules; stale
  cache anomalies.
- Constraints: CPython-specific.
- Defined by: official implementation (PEPs 3147/552). Confidence: high.
- Source: PEP 3147; PEP 552.

**TOL-03. Optimization flags: `-O`, `-OO`, `__debug__`**
- Behavior: `-O` removes `assert` statements and sets `__debug__` False; `-OO`
  additionally strips docstrings — the same source yields different runtime
  entities/behavior per invocation flags.
- Evidence: an `assert`-guarded invariant and an `if __debug__:` block; a
  docstring-dependent feature (doctest, help text) noted as `-OO`-fragile.
- Variants/failures: asserts as control flow (anti-pattern) breaking under `-O`.
- Constraints: CPython flags.
- Defined by: official implementation. Confidence: high.
- Source: CPython command-line documentation.

**TOL-04. Syntax-version gating of source files**
- Behavior: a file using newer syntax (match ≥3.10, `except*` ≥3.11, PEP 695
  ≥3.12) raises SyntaxError at *import/compile time* on older interpreters; a
  project can carry per-version modules selected at runtime, so parseability is
  per-file, per-version.
- Evidence: version-gated modules imported conditionally (FLW-09/IMP-08), each
  using syntax of its minimum version; `requires-python` consistency in metadata.
- Variants/failures: a graph generator's own parser version limiting what it can
  read — the central "which grammar" question; `from __future__` imports as
  per-module dialect switches.
- Constraints: interpreter version.
- Defined by: language versioning. Confidence: high.
- Source: per-version "What's New in Python" documents.

**TOL-05. Platform-conditional code**
- Behavior: `sys.platform`/`os.name` branches select definitions; some stdlib
  modules exist only per-platform (`msvcrt`, `fcntl`, `termios`); the resolvable
  import set is platform-dependent.
- Evidence: a module with POSIX/Windows implementations selected at import; an
  import of a platform-only stdlib module inside a guard.
- Variants/failures: analyzing on one platform vs. representing all platforms;
  `os.path` being an alias to `posixpath`/`ntpath` (module identity aliasing in
  the stdlib itself).
- Constraints: platform.
- Defined by: stdlib/platform. Confidence: high.
- Source: sys and os module documentation.

**TOL-06. Environment and path resolution**
- Behavior: virtual environments, `PYTHONPATH`, site-packages, and `sitecustomize`
  determine which module a name resolves to; the same import can resolve to
  different installed versions per environment.
- Evidence: environment declaration (lock file, requires-python); a documented
  assumption of one active venv; optionally a vendored copy of a dependency
  shadowing the installed one.
- Variants/failures: vendoring (`pkg/_vendor/...`) duplicating third-party
  entities under new identities; `-S`/isolated mode altering startup.
- Constraints: CPython site machinery.
- Defined by: official implementation + ecosystem practice. Confidence: high.
- Source: site module documentation; venv documentation.

**TOL-07. Free-threaded and JIT builds (3.13+)**
- Behavior: CPython 3.13 introduced an experimental no-GIL ("free-threaded") build
  (PEP 703) and an experimental JIT; free-threading is an ABI/build variant —
  extension compatibility and concurrency semantics differ; support matured in
  3.14 (medium confidence on exact status).
- Evidence: none required in source beyond thread-safety-relevant code (CON-03);
  optionally a marker/classifier acknowledging build variants.
- Variants/failures: extensions requiring the free-threaded ABI variant; behavior
  differences under true parallelism.
- Constraints: CPython ≥3.13 special builds.
- Defined by: official implementation (PEP 703). Confidence: medium.
- Source: PEP 703; CPython 3.13 release notes.

**TOL-08. Source encoding and identifier normalization**
- Behavior: source defaults to UTF-8 with optional encoding declarations
  (PEP 263); identifiers may be non-ASCII (PEP 3131) and are normalized to NFKC —
  two visually distinct spellings can denote the *same* entity.
- Evidence: one module with non-ASCII identifiers used across a module boundary;
  (optionally) a pair of NFKC-equivalent spellings referring to one binding.
- Variants/failures: tools comparing identifiers byte-wise mis-model NFKC
  equivalence; an explicit encoding declaration on one file.
- Constraints: Python 3 lexer rules.
- Defined by: language (PEPs 263/3131). Confidence: high (NFKC edge: medium).
- Source: PEP 3131; Language Reference, "Lexical analysis".

### 3.12 Invalid, Broken, and Ambiguous Program States (ERR)

**ERR-01. Syntactically invalid file within the project**
- Behavior: a file that fails to parse (future-version syntax, or plain breakage)
  raises SyntaxError only when compiled/imported; the rest of the project remains
  valid — analysis must degrade per-file, not per-project.
- Evidence: one quarantined file with next-version-only syntax, excluded from
  runtime imports but present on disk.
- Variants/failures: files intentionally excluded from packaging but present in
  the repo (PKG-02); templates with placeholder syntax stored as `.py`.
- Constraints: keep it unimported to preserve project runnability.
- Defined by: language. Confidence: high.
- Source: Language Reference, "Lexical analysis"/compilation behavior.

**ERR-02. Unresolvable imports**
- Behavior: imports of missing optional dependencies, or of stdlib modules removed
  by version (`distutils` and `imp` removed in 3.12; PEP 594 "dead battery"
  modules removed in 3.13), fail at runtime with ImportError/ModuleNotFoundError.
- Evidence: a guarded import of a removed stdlib module (IMP-08) and one
  deliberately unguarded import of a nonexistent module in a never-imported
  module, as a dangling-edge probe.
- Variants/failures: typo'd import names; imports valid only under an extra
  (PKG-05); resolution differing between analyzed environment and target.
- Constraints: version-dependent stdlib surface.
- Defined by: language + stdlib evolution. Confidence: high.
- Source: "What's New in Python 3.12/3.13" (removals); PEP 594.

**ERR-03. Latent attribute errors vs. dynamic provision**
- Behavior: attribute references that no static definition satisfies may be typos
  (latent AttributeError) or may be satisfied dynamically (CLS-10, IMP-11,
  CLS-15) — the discrimination is a core accuracy test.
- Evidence: one true dangling attribute reference in unreachable/never-run code,
  adjacent to dynamic-provision cases that look identical lexically.
- Variants/failures: `hasattr`-guarded access; `getattr` with defaults;
  attribute added by a different module before use.
- Constraints: none.
- Defined by: language semantics. Confidence: high.
- Source: Language Reference, "Data model".

**ERR-04. Redefinition and rebinding within one scope**
- Behavior: a second `def`/`class`/assignment of the same name silently replaces
  the first — legal, but usually a defect; contrast with sanctioned same-name
  patterns (property setters CLS-05, overloads FUN-05, conditional definitions
  FLW-09).
- Evidence: one accidental duplicate `def` in a clearly marked corner; the
  sanctioned patterns present elsewhere for contrast.
- Variants/failures: a name first imported then shadowed by a local definition;
  class attribute overwritten later in the class body.
- Constraints: none.
- Defined by: language. Confidence: high.
- Source: Language Reference, "Execution model".

**ERR-05. Class-construction failures**
- Behavior: inconsistent MRO (TypeError at class creation), metaclass conflicts,
  duplicate base classes, and slot/attribute conflicts fail when the `class`
  statement executes — the entity never comes into being.
- Evidence: a guarded (try/except) demonstration or a never-imported module
  containing an unlinearizable hierarchy.
- Variants/failures: abstract instantiation failure (CLS-07) as the runtime-stage
  contrast; failures occurring at import time of the defining module.
- Constraints: keep isolated from the healthy import graph.
- Defined by: language. Confidence: high.
- Source: Language Reference, "Data model" (class creation).

**ERR-06. Wrong or unresolvable annotations tolerated at runtime**
- Behavior: annotations are not enforced; an annotation can name a nonexistent
  type (string form), disagree with the actual value, or be arbitrary
  expressions — programs run anyway; only `get_type_hints`/checkers object.
- Evidence: a deliberately wrong annotation on a working function; a string
  annotation referencing a name that exists only under TYPE_CHECKING.
- Variants/failures: graphs that trust annotations for typing must record
  provenance (declared vs. inferred vs. runtime-observed).
- Constraints: pre-3.14 eager evaluation makes *expression* errors raise at def
  time unless stringized — regime-dependent (TYP-01).
- Defined by: language. Confidence: high.
- Source: PEP 526; PEP 563.

**ERR-07. Star-import collisions and last-writer-wins**
- Behavior: multiple `from x import *` statements can silently rebind names;
  final binding depends on statement order; provenance of a used name becomes
  ambiguous without `__all__` discipline.
- Evidence: two star imports with one overlapping name, plus a use of that name.
- Variants/failures: linters flag shadowing; `__all__` on the sources changes the
  collision set.
- Constraints: module scope only.
- Defined by: language. Confidence: high.
- Source: Language Reference, "The import statement".

### 3.13 Ecosystem Framework Patterns (ECO)

These generalize widely used framework behaviors without requiring a specific
framework; a fixture can implement miniature in-project versions.

**ECO-01. Decorator-based registration of handlers/routes/commands**
- Behavior: Flask/click-style `@app.route(...)`/`@cli.command()` decorators bind
  entities into an application object; the call graph from dispatcher to handler
  exists only through the registration data.
- Evidence: an in-project mini-router where decorated functions are invoked by
  string key lookup, with registrations spread across modules.
- Variants/failures: registration order and import-dependency (IMP-13); handler
  name vs. registered key divergence.
- Constraints: none.
- Defined by: ecosystem convention. Confidence: high.
- Source: Flask/click documentation as representative precedent.

**ECO-02. Declarative ORM-style models**
- Behavior: Django/SQLAlchemy-style models use metaclasses/`__init_subclass__` so
  class-level field descriptors define per-instance attributes whose access type
  differs by receiver (class → descriptor/column, instance → value); relationships
  are declared by *string* class references; reverse accessors are injected at
  runtime with no source definition.
- Evidence: an in-project mini-ORM base exercising descriptor fields, a
  string-referenced related model, and one runtime-injected reverse attribute.
- Variants/failures: attributes existing only after "app registry" setup;
  checker plugins normally paper over this — a graph must decide what to model.
- Constraints: none.
- Defined by: ecosystem convention. Confidence: high.
- Source: Django/SQLAlchemy documentation as representative precedent.

**ECO-03. Annotation-driven runtime models and `dataclass_transform`**
- Behavior: pydantic-style libraries consume `__annotations__` to generate
  validation/serialization behavior; PEP 681 `dataclass_transform` lets a
  third-party decorator/base be treated as dataclass-like by checkers —
  synthesized `__init__` signatures derived from annotations.
- Evidence: an in-project mini-model base reading annotations to build fields,
  marked with `dataclass_transform`, with call sites using the synthesized
  constructor shape.
- Variants/failures: `Annotated` metadata consumed at runtime (TYP-02); default
  vs. field-specifier semantics.
- Constraints: `dataclass_transform` in typing ≥3.11 (PEP 681).
- Defined by: typing spec + ecosystem convention. Confidence: high.
- Source: PEP 681.

**ECO-04. Module-name coupling in logging**
- Behavior: `logging.getLogger(__name__)` keys logger hierarchies to module dotted
  paths; renames/moves silently change logger identity; string logger names in
  config reference the module namespace.
- Evidence: per-module loggers via `__name__`; a logging config referencing one
  logger by dotted string.
- Variants/failures: `__name__` becoming `"__main__"` under script execution
  (IMP-14) changing the logger tree.
- Constraints: none.
- Defined by: stdlib + convention. Confidence: high.
- Source: logging documentation.

**ECO-05. Documentation cross-references to code entities**
- Behavior: docstrings carry structured references (Sphinx roles like
  `:class:`/`:func:`, or plain dotted names in Google/NumPy styles) that name
  entities; autodoc-style tooling resolves them against the import system.
- Evidence: docstrings in one style referencing project classes/functions by
  dotted path, including one stale reference as a staleness probe.
- Variants/failures: references into third-party namespaces; `__doc__` absent
  under `-OO` (TOL-03).
- Constraints: ecosystem tooling.
- Defined by: ecosystem convention. Confidence: high.
- Source: Sphinx documentation (cross-referencing).

**ECO-06. CLI argument frameworks binding callables**
- Behavior: argparse subcommands conventionally bind handlers via
  `set_defaults(func=...)` and dispatch through `args.func(...)`; typer/click
  derive CLI parameters from function signatures and annotations — signatures as
  external interface (parameter names become user-facing flags).
- Evidence: an argparse-based CLI with subcommands dispatching through stored
  callables; entry point wiring (PKG-04).
- Variants/failures: handler reference stored as data (FUN-10); signature changes
  silently changing the CLI surface.
- Constraints: none.
- Defined by: stdlib + ecosystem convention. Confidence: high.
- Source: argparse documentation.

---

## 4. Features Whose Support Changed Materially Between Recent Versions

| Feature | Version | Nature of change |
| --- | --- | --- |
| Assignment expressions (`:=`), positional-only params, f-string `=` | 3.8 | New syntax (PEP 572, 570) |
| `typing.Protocol`, `Literal`, `Final`, `TypedDict` in stdlib | 3.8 | Typing surface additions |
| Builtin generics `list[int]` (PEP 585); arbitrary decorator expressions (PEP 614) | 3.9 | New runtime/typing + grammar relaxation |
| `match` statement; `X \| Y` unions; `ParamSpec`; `TypeAlias`; `TypeGuard`; parenthesized `with` | 3.10 | Major syntax/typing additions |
| `except*` / ExceptionGroup; `Self`; `TypeVarTuple`; `Required/NotRequired`; `StrEnum`; `tomllib`; `typing.get_overloads`; `dataclass_transform` | 3.11 | Syntax + typing + stdlib |
| PEP 695 generics & `type` statement; PEP 701 f-strings; `@override`; `Unpack` kwargs; comprehension inlining (PEP 709); **`distutils`/`imp` removed** | 3.12 | Syntax, typing, semantics-adjacent, stdlib removals |
| TypeVar defaults (PEP 696); `TypeIs`; `ReadOnly` TypedDict; `warnings.deprecated`; free-threaded & JIT builds (experimental); PEP 594 module removals; docstring leading-whitespace stripping | 3.13 | Typing + build variants + removals (dedent detail: medium confidence) |
| Deferred annotation evaluation by default (PEP 649/749, `annotationlib`); t-strings (PEP 750); subinterpreters in stdlib (PEP 734); free-threading officially supported; multiprocessing default start-method change on Linux | 3.14 | Semantic regime change for annotations; new literal kind (several details: medium confidence) |

Cross-cutting regime note: **annotations have three regimes** — eager (default
≤3.13), stringized (`from __future__ import annotations`, per-module), lazy
(3.14 default). A fixture spanning these is the single highest-leverage
version-sensitivity test for typing-aware graphs.

## 5. Implementation-Defined or Unspecified Behavior

- Object identity: `is`, `id()` reuse, small-integer caching, string interning —
  CPython details; code relying on them is nonportable.
- Finalization: `__del__` timing, refcounting vs. tracing GC; destruction order at
  interpreter shutdown.
- `dict` insertion order is a language guarantee since 3.7 (was CPython-specific
  in 3.6); `set` iteration order remains unspecified and hash-randomized
  (`PYTHONHASHSEED`).
- `sys.path` initialization (script dir prepending, site processing) —
  implementation/startup-mode dependent.
- Bytecode format, `__pycache__` layout, and frame/traceback details (including
  the 3.12 comprehension inlining) — CPython internals observable via
  introspection.
- Exact wording of error messages (much improved 3.10–3.13) — diagnostics that
  match on text are version-fragile.
- Recursion limits, stack behavior, GIL scheduling, and free-threaded-build
  memory-model details.
- Import-lock granularity and thread-safety of import side effects.
- Default filesystem/locale encodings on non-UTF-8 platforms (UTF-8 mode and its
  planned default-ness — PEP 686 — is version-contingent; medium confidence).

## 6. Areas Where My Knowledge May Be Incomplete

- Exact 3.14 finalized behavior (annotation semantics edge cases under PEP 749,
  t-string API surface, subinterpreter stdlib API, multiprocessing default
  changes) — post-release refinements may differ from my snapshot.
- Fast-moving packaging tooling: uv workspace semantics, PEP 735 dependency
  groups adoption, poetry 2.x PEP 621 alignment, editable-install mechanics per
  backend version.
- Enum behavioral details across 3.10→3.12 (str/format/repr changes) — direction
  known, specifics fuzzy.
- Typing-spec conformance details that shifted after the spec was consolidated at
  typing.python.org (e.g., precise Protocol member rules, `__all__`/re-export
  visibility rules per checker).
- pytest import-mode edge cases (`importlib` mode identity behavior) and rootdir
  determination corner cases.
- Free-threaded build semantics and which stdlib/C-extension guarantees hold.
- Windows-specific launcher/py.exe and path-resolution behaviors.

## 7. Final Audit — Categories or Feature Families Possibly Still Missing

Reviewed against the checklist; candidates a fixture designer may still weigh:

- **Notebooks**: `.ipynb` as a source form (cells, execution order, magics) —
  excluded as outside a standard package, but common in real repos.
- **Embedding and the C API from the host side** (Python embedded in another
  program) — inverse of PKG-11; likely out of scope for a self-contained fixture.
- **`ctypes`/`cffi` ABI-mode FFI without build steps** — partially covered by
  PKG-11; a pure-`ctypes` binding is a distinct no-build variant.
- **Internationalization** (`gettext` catalogs, `_()` conventions) — string-level
  entity references via message ids.
- **Weak references and finalizers** (`weakref`, `__del__`-adjacent liveness) —
  behavioral, rarely graph-relevant.
- **Audit hooks / `sys.settrace` / instrumentation** — runtime observability, not
  program structure.
- **`__future__` breadth** — only `annotations` remains materially active in
  supported versions; covered in TYP-01.
- **Alternative distribution ecosystems** (conda, OS packages, vendored wheels in
  `wheels/` dirs) — environment provisioning variance beyond PKG-12.
- **WASM/Pyodide, iOS/Android tiers (3.13+)** — platform variants beyond TOL-05.
- **Type-checker-specific config semantics** (mypy plugins, pyright strictness
  tiers) — deliberately excluded per no-tool-assumption rule, but plugin-driven
  type synthesis (e.g., ORM plugins) parallels ECO-02/ECO-03.
- **Task runners and CI** (tox/nox/Makefile/Taskfile, GitHub Actions matrices) —
  they select interpreter versions and environments (interacts with TOL-04/06)
  but add no new language semantics.
- **`typing_extensions` as a parallel source of typing entities** — mentioned in
  TYP-02 constraints; a fixture targeting older baselines would import newer
  constructs from it, giving one conceptual entity two provider modules.
- **Docs builds (Sphinx autodoc) executing imports** — import-side-effect surface
  during documentation generation; adjacent to ECO-05.

No further first-class semantic families were identified beyond the 13 categories
and 126 items above.
