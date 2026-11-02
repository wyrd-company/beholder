#!/usr/bin/env python3
# ---
# relationships:
#   implements: semantic-oracle-model
#   references: semantic-oracle-profile-python
# ---
"""Writes a Python compiler report for `semantic-oracle audit --report`.

Imports each module under `--search-path` and reads it with `inspect.signature`:
parameter names, kinds, defaults and annotations, and return annotations. Importing
runs the project's module-level code, so run it on a project you trust. It writes no
bytecode into the project tree.

  report-python.py --search-path DIR[:DIR] --context ID --out FILE --python VERSION

VERSION is the context's interpreter, such as `3.14`. The script runs under
`pythonVERSION`, and stops with exit 2 and no report when that interpreter is absent,
so that a report never states one version's view for another's context. The report
states what it ran under.

Python has no reference type checker, so this covers the runtime's view only, and is
for a `runtime` context. A slot it cannot state is `unchecked` in the audit. A module that fails to
import is listed under `failed`, as its project may intend, and its slots stay
`unchecked`.
"""
import argparse
import builtins
import enum
import importlib
import inspect
import json
import os
import pkgutil
import platform
import re
import shutil
import sys
import types
import typing

sys.dont_write_bytecode = True

COVERS = [
    "parameters[].name",
    "parameters[].mode",
    "parameters[].type",
    "parameters[].default",
    "returns",
    "get.type",
    "set.type",
    "value",
]
MODES = {
    inspect.Parameter.POSITIONAL_ONLY: "positional-only",
    inspect.Parameter.KEYWORD_ONLY: "keyword-only",
    inspect.Parameter.VAR_POSITIONAL: "var-positional",
    inspect.Parameter.VAR_KEYWORD: "var-keyword",
}
BARE = {"bool", "bytearray", "bytes", "complex", "dict", "float", "frozenset", "int", "list", "object", "set", "str", "tuple", "type"}


# The runtime expands an alias to its value, and the profile writes the alias. Each
# value a module binds that is not a class is one, by identity.
ALIASES = {}

try:
    import annotationlib

    STRING_FORMAT = annotationlib.Format.STRING
except ImportError:
    STRING_FORMAT = None


def remember_aliases(module):
    for name, value in vars(module).items():
        if not inspect.isclass(value) and (typing.get_origin(value) is not None or isinstance(value, types.UnionType)):
            ALIASES.setdefault(id(value), f"{module.__name__}.{name}")


class Expanded(Exception):
    """The runtime replaced an annotation's name by its value, so its text is lost."""


def resolved_name(dotted, namespace):
    """A written name, as type text. Returns None for a name it cannot find."""
    head, *rest = dotted.split(".")
    if head == "None":
        return "None"
    if head in namespace:
        value = namespace[head]
    elif hasattr(builtins, head):
        value = getattr(builtins, head)
    else:
        return None
    for part in rest:
        value = getattr(value, part, None)
        if value is None:
            return None
    if id(value) in ALIASES:
        return f"@{{{ALIASES[id(value)]}}}"
    if isinstance(value, typing.TypeVar) or type(value).__name__ in ("ParamSpec", "TypeVarTuple"):
        return value.__name__
    module = getattr(value, "__module__", None)
    name = getattr(value, "__qualname__", None) or getattr(value, "__name__", None) or getattr(value, "_name", None)
    if module == "builtins" and name in BARE:
        return name
    if module and name:
        return f"@{{{module}.{name}}}"
    return None


NAME = re.compile(r"""("(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*')|([A-Za-z_][A-Za-z0-9_.]*)""")


def written_text(annotation, namespace):
    """An annotation's written text, with each name resolved. Returns None for a name
    it cannot find."""
    failed = []

    def replace(match):
        if match.group(1):
            return match.group(1)
        found = resolved_name(match.group(2), namespace)
        if found is None:
            failed.append(match.group(2))
            return match.group(2)
        return found

    spelled = NAME.sub(replace, annotation.strip())
    return None if failed else spelled


def annotation_text(annotation, namespace):
    """A slot's text: an annotation's written text when it is a string, and otherwise
    the evaluated annotation. An evaluated alias stops, because its name is lost."""
    if isinstance(annotation, str):
        return written_text(annotation, namespace)
    return type_text(annotation)


def type_text(annotation):
    """An annotation as type text. Returns None for one this cannot spell."""
    if annotation is inspect.Signature.empty:
        return None
    if id(annotation) in ALIASES:
        raise Expanded
    if annotation is None or annotation is type(None):
        return "None"
    if annotation is Ellipsis:
        return "..."
    if isinstance(annotation, str):
        return None
    if isinstance(annotation, typing.TypeVar) or getattr(type(annotation), "__name__", "") in ("ParamSpec", "TypeVarTuple"):
        return annotation.__name__
    origin = typing.get_origin(annotation)
    if origin is typing.Union or origin is types.UnionType:
        parts = [type_text(arg) for arg in typing.get_args(annotation)]
        return None if None in parts else " | ".join(parts)
    if origin is typing.Literal:
        values = [json.dumps(v) if isinstance(v, str) else repr(v) for v in typing.get_args(annotation)]
        return f"@{{typing.Literal}}[{', '.join(values)}]"
    if origin is not None:
        head = type_text(origin)
        parts = []
        for arg in typing.get_args(annotation):
            if isinstance(arg, list):
                inner = [type_text(item) for item in arg]
                if None in inner:
                    return None
                parts.append(f"[{', '.join(inner)}]")
            else:
                parts.append(type_text(arg))
        return None if head is None or None in parts else f"{head}[{', '.join(parts)}]"
    module = getattr(annotation, "__module__", None)
    name = getattr(annotation, "__qualname__", None) or getattr(annotation, "__name__", None) or getattr(annotation, "_name", None)
    if module == "builtins" and name in BARE:
        return name
    if module and name:
        return f"@{{{'builtins.' if False else ''}{module}.{name}}}" if module != "builtins" else f"@{{{name}}}"
    return None


def default_text(value):
    if value is None:
        return "None"
    if value is True:
        return "True"
    if value is False:
        return "False"
    if isinstance(value, bool):
        return repr(value)
    if isinstance(value, int):
        return str(value)
    if isinstance(value, float):
        return repr(value)
    if isinstance(value, str):
        return json.dumps(value)
    return "present"


class Report:
    def __init__(self):
        self.members = {}

    def add(self, path, slots):
        self.members.setdefault(path, {"path": path, "slots": {}, "unstated": []})["slots"].update(slots)

    def unstated(self, path, shape):
        self.add(path, {})
        self.members[path]["unstated"].append(shape)

    def text_or_unstated(self, path, slot, annotation, namespace):
        """A slot's text, or None. A form it cannot spell and an alias the runtime
        expanded are unstated, so that they are neither an error nor agreed."""
        try:
            text = annotation_text(annotation, namespace)
        except Expanded:
            text = None
        if text is None:
            self.unstated(path, slot)
        return text

    def callable(self, path, function):
        namespace = getattr(function, "__globals__", {})
        try:
            signature = (
                inspect.signature(function, annotation_format=STRING_FORMAT)
                if STRING_FORMAT is not None
                else inspect.signature(function)
            )
        except (ValueError, TypeError, NameError, AttributeError, SyntaxError):
            self.unstated(path, "parameters")
            self.unstated(path, "returns")
            return
        slots = {}
        for i, parameter in enumerate(signature.parameters.values()):
            slots[f"parameters[{i}].name"] = parameter.name
            if parameter.kind in MODES:
                slots[f"parameters[{i}].mode"] = MODES[parameter.kind]
            if parameter.annotation is not inspect.Parameter.empty:
                text = self.text_or_unstated(path, f"parameters[{i}].type", parameter.annotation, namespace)
                if text is not None:
                    slots[f"parameters[{i}].type"] = text
            if parameter.default is not inspect.Parameter.empty:
                slots[f"parameters[{i}].default"] = default_text(parameter.default)
        if signature.return_annotation is not inspect.Signature.empty:
            text = self.text_or_unstated(path, "returns", signature.return_annotation, namespace)
            if text is not None:
                slots["returns"] = text
        self.add(path, slots)

    def property(self, path, prop):
        slots = {}
        for key, function, target in (("get", prop.fget, "return"), ("set", prop.fset, None)):
            if function is None:
                continue
            namespace = getattr(function, "__globals__", {})
            try:
                hints = (
                    inspect.get_annotations(function, format=STRING_FORMAT)
                    if STRING_FORMAT is not None
                    else inspect.get_annotations(function)
                )
            except Exception:
                self.unstated(path, f"{key}.type")
                continue
            if target == "return":
                annotation = hints.get("return", inspect.Signature.empty)
            else:
                names = [name for name in hints if name != "return"]
                annotation = hints[names[0]] if names else inspect.Signature.empty
            if annotation is inspect.Signature.empty:
                continue
            text = self.text_or_unstated(path, f"{key}.type", annotation, namespace)
            if text is not None:
                slots[f"{key}.type"] = text
        self.add(path, slots)

    def klass(self, path, cls):
        self.add(path, {})
        for name, value in vars(cls).items():
            child = f"{path}.{name}"
            if isinstance(value, (staticmethod, classmethod)):
                self.callable(child, value.__func__)
            elif isinstance(value, property):
                self.property(child, value)
            elif inspect.isfunction(value):
                self.callable(child, value)
            elif inspect.isclass(value) and value.__qualname__.startswith(cls.__qualname__ + "."):
                self.klass(child, value)
            elif isinstance(cls, enum.EnumMeta) and isinstance(value, cls):
                literal = default_text(value.value)
                if literal != "present":
                    self.add(child, {"value": literal})

    def module(self, module):
        for name, value in vars(module).items():
            if getattr(value, "__module__", None) != module.__name__:
                continue
            path = f"{module.__name__}.{name}"
            if inspect.isfunction(value):
                self.callable(path, value)
            elif inspect.isclass(value):
                self.klass(path, value)


def modules_under(search_path):
    for directory in search_path:
        for found in pkgutil.walk_packages([directory]):
            yield found.name


def run_under(version):
    """Continues under the context's interpreter, whichever one started this script."""
    running = f"{sys.version_info.major}.{sys.version_info.minor}"
    if running == version:
        return
    interpreter = shutil.which(f"python{version}")
    if interpreter is None:
        print(f"report-python: the context's interpreter python{version} is absent", file=sys.stderr)
        sys.exit(2)
    os.execv(interpreter, [interpreter, "-B", os.path.abspath(__file__), *sys.argv[1:]])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--search-path", required=True)
    parser.add_argument("--context", required=True)
    parser.add_argument("--out", required=True)
    parser.add_argument("--python", required=True)
    arguments = parser.parse_args()
    run_under(arguments.python)
    search_path = [os.path.abspath(p) for p in arguments.search_path.split(os.pathsep)]
    out = os.path.abspath(arguments.out)
    if any(out.startswith(directory + os.sep) for directory in search_path):
        sys.exit(f"report-python: --out {out} lies inside the project tree")
    if os.path.exists(out):
        os.remove(out)
    sys.path[:0] = search_path
    report = Report()
    failed = []
    modules = []
    for name in sorted(set(modules_under(search_path))):
        try:
            module = importlib.import_module(name)
            remember_aliases(module)
            modules.append(module)
        except BaseException as cause:  # A project module can raise anything on import.
            failed.append(f"{name}: {type(cause).__name__}: {cause}")
    # Every alias is known before any annotation is read.
    for module in modules:
        report.module(module)
    with open(out, "w") as written:
        json.dump(
            {
                "format": "semantic-oracle-compiler-report/1",
                "language": "python",
                "context": arguments.context,
                "tool": {"name": "inspect.signature", "version": platform.python_version(), "interpreter": sys.executable},
                "covers": COVERS,
                "members": sorted(report.members.values(), key=lambda member: member["path"]),
                "failed": failed,
            },
            written,
            indent=1,
            sort_keys=True,
        )
    print(json.dumps({"members": len(report.members), "failed": len(failed), "python": platform.python_version(), "interpreter": sys.executable}))


if __name__ == "__main__":
    main()
