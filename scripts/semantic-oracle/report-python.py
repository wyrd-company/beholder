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

  report-python.py --search-path DIR[:DIR] --context ID --out FILE

Python has no reference type checker, so this covers the runtime's view only, and is
for a `runtime` context. A slot it cannot state is `unchecked` in the audit.
"""
import argparse
import enum
import importlib
import inspect
import json
import os
import pkgutil
import platform
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


def type_text(annotation):
    """An annotation as type text. Returns None for one this cannot spell."""
    if annotation is inspect.Signature.empty:
        return None
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

    def callable(self, path, function):
        try:
            signature = inspect.signature(function, eval_str=True)
        except (ValueError, TypeError, NameError, AttributeError, SyntaxError):
            try:
                signature = inspect.signature(function)
            except (ValueError, TypeError):
                self.unstated(path, "parameters")
                self.unstated(path, "returns")
                return
        slots = {}
        for i, parameter in enumerate(signature.parameters.values()):
            slots[f"parameters[{i}].name"] = parameter.name
            if parameter.kind in MODES:
                slots[f"parameters[{i}].mode"] = MODES[parameter.kind]
            text = type_text(parameter.annotation)
            if text is not None:
                slots[f"parameters[{i}].type"] = text
            elif parameter.annotation is not inspect.Parameter.empty:
                self.unstated(path, f"parameters[].type")
            if parameter.default is not inspect.Parameter.empty:
                slots[f"parameters[{i}].default"] = default_text(parameter.default)
        text = type_text(signature.return_annotation)
        if text is not None:
            slots["returns"] = text
        elif signature.return_annotation is not inspect.Signature.empty:
            self.unstated(path, "returns")
        self.add(path, slots)

    def property(self, path, prop):
        slots = {}
        for key, function, target in (("get", prop.fget, "return"), ("set", prop.fset, None)):
            if function is None:
                continue
            try:
                hints = inspect.get_annotations(function, eval_str=True)
            except Exception:
                hints = getattr(function, "__annotations__", {})
            if target == "return":
                annotation = hints.get("return", inspect.Signature.empty)
            else:
                names = [name for name in hints if name != "return"]
                annotation = hints[names[0]] if names else inspect.Signature.empty
            text = type_text(annotation)
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


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--search-path", required=True)
    parser.add_argument("--context", required=True)
    parser.add_argument("--out", required=True)
    arguments = parser.parse_args()
    search_path = [os.path.abspath(p) for p in arguments.search_path.split(os.pathsep)]
    sys.path[:0] = search_path
    report = Report()
    failed = []
    for name in sorted(set(modules_under(search_path))):
        try:
            report.module(importlib.import_module(name))
        except BaseException as cause:  # A project module can raise anything on import.
            failed.append(f"{name}: {type(cause).__name__}: {cause}")
    with open(arguments.out, "w") as out:
        json.dump(
            {
                "format": "semantic-oracle-compiler-report/1",
                "language": "python",
                "context": arguments.context,
                "tool": {"name": "inspect.signature", "version": platform.python_version()},
                "covers": COVERS,
                "members": sorted(report.members.values(), key=lambda member: member["path"]),
                "failed": failed,
            },
            out,
            indent=1,
            sort_keys=True,
        )
    print(json.dumps({"members": len(report.members), "failed": len(failed)}))


if __name__ == "__main__":
    main()
