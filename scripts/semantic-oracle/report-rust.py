#!/usr/bin/env python3
# ---
# relationships:
#   implements: semantic-oracle-model
#   references: semantic-oracle-profile-rust
# ---
"""Writes a Rust compiler report for `semantic-oracle audit --report`.

Copies the project outside its tree, reads rustdoc JSON for each `lib` target,
then compiles a generated probe crate that depends on the copy, to verify
`auto-traits`. The project tree gets no build output and no lockfile. One compile
reports every failing probe, by line. A line that fails for a reason other than the
trait is set aside and the compile repeats, at most five times.

A build that fails is an error: this script exits 2 and writes no report. A report
never states what a failed build did not check.

  report-rust.py --project DIR --context ID --out FILE
      [--features a,b] [--no-default-features] [--all-features] [--cfg NAME]...

All of these use one toolchain, the one the project selects.

Not stated, so `unchecked` in the audit: an opaque occurrence of a callable the
probe cannot name or call by shape, and a condition that is not the same trait
on a callable's type parameters, up to three of them. Rustdoc's JSON format is
unstable, so this
script reads it through RUSTC_BOOTSTRAP and reports the format version it read.
"""
import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import time

LIBRARY_KINDS = {"lib", "rlib", "dylib", "cdylib", "staticlib", "proc-macro"}
TRAITS = ["RefUnwindSafe", "Send", "Sync", "Unpin", "UnwindSafe"]
BOUND = {
    "RefUnwindSafe": "::std::panic::RefUnwindSafe",
    "Send": "::std::marker::Send",
    "Sync": "::std::marker::Sync",
    "Unpin": "::std::marker::Unpin",
    "UnwindSafe": "::std::panic::UnwindSafe",
}
# What the probe reads: every other member kind has no `auto-traits`.
COVERS = [
    "parameters[].name",
    "parameters[].type",
    "parameters[].mode",
    "returns",
    "abi",
    "type",
    "value",
    "modifiers[]",
    "auto-traits[]",
    "bases[].bindings[].auto-traits[]",
    "type-parameters[].name",
    "type-parameters[].kind",
]


# The toolchain the project selects, for every command, so that a probe and its
# report's version come from the same compiler as rustdoc's.
TOOLCHAIN = {}


def run(command, cwd, env=None):
    merged = {**os.environ, **TOOLCHAIN, **(env or {})}
    return subprocess.run(command, cwd=cwd, env=merged, capture_output=True, text=True)


def fail(message):
    """A build that failed stops the report: nothing it would state was checked."""
    print(f"report-rust: {message}", file=sys.stderr)
    sys.exit(2)


def select_toolchain(project):
    if not shutil.which("rustup"):
        return
    active = subprocess.run(["rustup", "show", "active-toolchain"], cwd=project, capture_output=True, text=True)
    name = active.stdout.split()
    if active.returncode == 0 and name:
        TOOLCHAIN["RUSTUP_TOOLCHAIN"] = name[0]


def cargo_metadata(project):
    found = run(["cargo", "metadata", "--format-version", "1", "--no-deps", "--offline"], project)
    if found.returncode:
        fail(f"cargo metadata failed:\n{found.stderr}")
    return json.loads(found.stdout)


def rustdoc_json(project, target_dir, selection):
    command = ["cargo", "doc", "--offline", "--no-deps", "--lib", "--workspace", "--target-dir", target_dir]
    if selection.features:
        command += ["--features", selection.features]
    if selection.no_default_features:
        command.append("--no-default-features")
    if selection.all_features:
        command.append("--all-features")
    env = {
        "RUSTC_BOOTSTRAP": "1",
        "RUSTDOCFLAGS": f"-Z unstable-options --output-format json --document-private-items -A warnings {cfg_flags(selection)}",
    }
    built = run(command, project, env)
    if built.returncode:
        fail(f"rustdoc failed:\n{built.stderr}")
    return os.path.join(target_dir, "doc")


def cfg_flags(selection):
    return " ".join(f"--cfg {name}" for name in selection.cfg)


def dependency_line(package, selection):
    """A probe's dependency on a package, in the context's feature selection."""
    wanted = [name for name in selection.features.split(",") if name]
    if selection.all_features:
        chosen = sorted(package["features"])
    else:
        chosen = sorted(name for name in wanted if name in package["features"])
    parts = [f"path = {json.dumps(os.path.dirname(package['manifest_path']))}"]
    if chosen:
        parts.append(f"features = {json.dumps(chosen)}")
    if selection.no_default_features:
        parts.append("default-features = false")
    return f"{json.dumps(package['name'])} = {{ {', '.join(parts)} }}"


class Crate:
    def __init__(self, data):
        self.data = data
        self.index = data["index"]
        self.paths = data["paths"]

    def item_path(self, item_id):
        known = self.paths.get(str(item_id))
        return "::".join(known["path"]) if known else None

    def name(self, item_id):
        path = self.item_path(item_id)
        return f"@{{{path}}}" if path else None

    def path_ref(self, path_value, self_type):
        target = self.name(path_value["id"]) or path_value["path"]
        args = path_value.get("args")
        return target + self.generic_args(args, self_type)

    def generic_args(self, args, self_type):
        if not args:
            return ""
        if "angle_bracketed" in args:
            bracketed = args["angle_bracketed"]
            parts = []
            for arg in bracketed.get("args", []):
                if "lifetime" in arg:
                    parts.append(arg["lifetime"])
                elif "type" in arg:
                    parts.append(self.ty(arg["type"], self_type))
                elif "const" in arg:
                    parts.append(arg["const"]["expr"])
            for constraint in bracketed.get("constraints", []):
                binding = constraint["binding"]
                if "equality" in binding and "type" in binding["equality"]:
                    parts.append(f"{constraint['name']} = {self.ty(binding['equality']['type'], self_type)}")
            return f"<{', '.join(parts)}>" if parts else ""
        if "parenthesized" in args:
            call = args["parenthesized"]
            inputs = ", ".join(self.ty(t, self_type) for t in call["inputs"])
            output = call.get("output")
            return f"({inputs})" + (f" -> {self.ty(output, self_type)}" if output else "")
        return ""

    def bound(self, bound, self_type):
        if "trait_bound" in bound:
            trait = bound["trait_bound"]
            modifier = "?" if trait.get("modifier") == "maybe" else ""
            return modifier + self.path_ref(trait["trait"], self_type)
        if "outlives" in bound:
            return bound["outlives"]
        if "use" in bound:
            names = [n.get("lifetime") or n.get("param") for n in bound["use"]]
            return f"use<{', '.join(names)}>"
        return ""

    def ty(self, t, self_type=None):
        if t is None:
            return "()"
        if "resolved_path" in t:
            return self.path_ref(t["resolved_path"], self_type)
        if "generic" in t:
            name = t["generic"]
            return self_type if name == "Self" and self_type else name
        if "primitive" in t:
            return t["primitive"]
        if "tuple" in t:
            items = [self.ty(item, self_type) for item in t["tuple"]]
            return f"({items[0]},)" if len(items) == 1 else f"({', '.join(items)})"
        if "slice" in t:
            return f"[{self.ty(t['slice'], self_type)}]"
        if "array" in t:
            return f"[{self.ty(t['array']['type'], self_type)}; {t['array']['len']}]"
        if "raw_pointer" in t:
            mutability = "mut" if t["raw_pointer"]["is_mutable"] else "const"
            return f"*{mutability} {self.ty(t['raw_pointer']['type'], self_type)}"
        if "borrowed_ref" in t:
            ref = t["borrowed_ref"]
            lifetime = f"{ref['lifetime']} " if ref.get("lifetime") else ""
            mutability = "mut " if ref["is_mutable"] else ""
            return f"&{lifetime}{mutability}{self.ty(ref['type'], self_type)}"
        if "impl_trait" in t:
            return "impl " + " + ".join(self.bound(b, self_type) for b in t["impl_trait"])
        if "dyn_trait" in t:
            dyn = t["dyn_trait"]
            parts = [self.path_ref(trait["trait"], self_type) for trait in dyn["traits"]]
            if dyn.get("lifetime"):
                parts.append(dyn["lifetime"])
            return "dyn " + " + ".join(parts)
        if "function_pointer" in t:
            pointer = t["function_pointer"]
            sig = pointer["sig"]
            inputs = ", ".join(self.ty(arg[1], self_type) for arg in sig["inputs"])
            output = f" -> {self.ty(sig['output'], self_type)}" if sig.get("output") else ""
            abi = pointer["header"].get("abi")
            prefix = "" if abi == "Rust" or abi is None else f'extern "{abi_name(abi)}" '
            return f"{prefix}fn({inputs}){output}"
        if "qualified_path" in t:
            qualified = t["qualified_path"]
            subject = self.ty(qualified["self_type"], self_type)
            trait = qualified.get("trait")
            if trait:
                # The associated item by its own path, as the profile writes it.
                owner = self.item_path(trait["id"])
                name = qualified["name"]
                item = f"@{{{owner}::{name}}}" if owner else name
                return f"<{subject} as {self.path_ref(trait, self_type)}>::{item}"
            return f"{subject}::{qualified['name']}"
        return json.dumps(t)


def abi_name(abi):
    if isinstance(abi, str):
        return abi
    return next(iter(abi))


def callable_slots(crate, function, self_type):
    slots = generics_slots(function["generics"])
    signature = function["sig"]
    for i, (name, ty) in enumerate(signature["inputs"]):
        is_self = name == "self"
        if name not in ("_", "") and not name.startswith("__arg"):
            slots[f"parameters[{i}].name"] = name
        if is_self and "borrowed_ref" in ty and ty["borrowed_ref"]["type"].get("generic") == "Self" and not ty["borrowed_ref"].get("lifetime"):
            slots[f"parameters[{i}].mode"] = "self-mut-ref" if ty["borrowed_ref"]["is_mutable"] else "self-ref"
        elif is_self and ty.get("generic") == "Self":
            pass
        else:
            slots[f"parameters[{i}].type"] = crate.ty(ty, self_type)
    if signature.get("is_c_variadic"):
        slots[f"parameters[{len(signature['inputs'])}].mode"] = "c-variadic"
    output = signature.get("output")
    if output is not None and output != {"tuple": []}:
        slots["returns"] = crate.ty(output, self_type)
    header = function["header"]
    abi = header.get("abi")
    if abi and abi != "Rust":
        slots["abi"] = abi_name(abi)
    modifiers = [word for word, on in [("async", header.get("is_async")), ("const", header.get("is_const")), ("unsafe", header.get("is_unsafe"))] if on]
    for word in modifiers:
        slots[f"modifiers[{word}]"] = word
    return slots


def const_value(value):
    if value is None:
        return None
    text = value.replace("_", "")
    for suffix in ("u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize", "f32", "f64"):
        if text.endswith(suffix):
            text = text[: -len(suffix)]
    return text


def generics_slots(generics):
    slots = {}
    for i, param in enumerate(generics.get("params", [])):
        kind = next(iter(param["kind"]))
        if kind == "type" and param["kind"]["type"].get("is_synthetic"):
            continue
        slots[f"type-parameters[{i}].name"] = param["name"]
        slots[f"type-parameters[{i}].kind"] = kind
    return slots


def add_member(members, path, kind):
    """A member, by its path and common kind: Rust's namespaces let one path name a
    module, a type and a function."""
    return members.setdefault(f"{kind}:{path}", {"path": path, "kind": kind, "slots": {}, "unstated": []})


class Walker:
    """Lists the crate's items by the profile's canonical entity paths."""

    def __init__(self, crate):
        self.crate = crate
        self.members = {}

    def add(self, path, slots, kind):
        add_member(self.members, path, kind)["slots"].update(slots)

    def walk_module(self, item_id, prefix):
        crate = self.crate
        module = crate.index[str(item_id)]["inner"]["module"]
        for child in module["items"]:
            item = crate.index.get(str(child))
            if not item:
                continue
            self.walk_item(item, prefix)

    def walk_item(self, item, prefix):
        crate = self.crate
        kind = next(iter(item["inner"]))
        inner = item["inner"][kind]
        name = item.get("name")
        path = f"{prefix}::{name}" if name else prefix
        if kind == "module":
            self.walk_module(item["id"], path)
        elif kind == "function":
            self.add(path, callable_slots(crate, inner, None), "callable")
        elif kind == "constant":
            slots = {"type": crate.ty(inner["type"])}
            value = const_value(inner["const"].get("value") or inner["const"].get("expr"))
            if value and re.fullmatch(r"-?[0-9.]+|true|false", value):
                slots["value"] = value
            self.add(path, slots, "constant")
        elif kind == "static":
            self.add(path, {"type": crate.ty(inner["type"])}, "variable")
        elif kind == "type_alias":
            self.add(path, {"type": crate.ty(inner["type"])}, "type-alias")
        elif kind == "struct":
            self.walk_struct(item, inner, path)
        elif kind == "enum":
            self.walk_enum(item, inner, path)
        elif kind == "union":
            self.add(path, {}, "type")
            for field in inner.get("fields", []):
                self.field(field, path)
            self.inherent(inner, path)
        elif kind == "trait":
            self.walk_trait(inner, path)

    def field(self, field_id, owner):
        item = self.crate.index.get(str(field_id))
        if item and "struct_field" in item["inner"]:
            self.add(f"{owner}::{item['name']}", {"type": self.crate.ty(item["inner"]["struct_field"])}, "field")

    def walk_struct(self, item, inner, path):
        self.add(path, {**self.attributes(item), **generics_slots(inner["generics"])}, "type")
        struct_kind = inner["kind"]
        fields = []
        if "plain" in struct_kind:
            fields = struct_kind["plain"]["fields"]
        elif "tuple" in struct_kind:
            for index, field in enumerate(struct_kind["tuple"]):
                if field is not None:
                    found = self.crate.index.get(str(field))
                    if found:
                        self.add(f"{path}::{index}", {"type": self.crate.ty(found["inner"]["struct_field"])}, "field")
        for field in fields:
            self.field(field, path)
        self.inherent(inner, path)

    def walk_enum(self, item, inner, path):
        self.add(path, self.attributes(item), "type")
        # A variant's discriminant, stated or implicit, for an enum of only units
        # or one that states a discriminant.
        variants = [self.crate.index.get(str(v)) for v in inner["variants"]]
        variants = [v for v in variants if v]
        states_values = all(v["inner"]["variant"]["kind"] == "plain" for v in variants) or any(
            v["inner"]["variant"].get("discriminant") for v in variants
        )
        next_value = 0
        for variant_id in inner["variants"]:
            variant = self.crate.index.get(str(variant_id))
            if not variant:
                continue
            variant_path = f"{path}::{variant['name']}"
            slots = {}
            discriminant = variant["inner"]["variant"].get("discriminant")
            if discriminant:
                slots["value"] = discriminant["value"]
                next_value = int(discriminant["value"]) + 1
            elif states_values and next_value is not None:
                slots["value"] = str(next_value)
                next_value += 1
            self.add(variant_path, slots, "enum-member")
            shape = variant["inner"]["variant"]["kind"]
            if isinstance(shape, dict) and "struct" in shape:
                for field in shape["struct"]["fields"]:
                    self.field(field, variant_path)
        self.inherent(inner, path)

    def attributes(self, item):
        attributes = json.dumps(item.get("attrs", []))
        return {"modifiers[non_exhaustive]": "non_exhaustive"} if "non_exhaustive" in attributes else {}

    def inherent(self, inner, path):
        crate = self.crate
        for impl_id in inner.get("impls", []):
            implementation = crate.index.get(str(impl_id))
            if not implementation:
                continue
            block = implementation["inner"]["impl"]
            if block.get("trait"):
                continue
            self_type = crate.ty(block["for"])
            for member in block["items"]:
                item = crate.index.get(str(member))
                if not item:
                    continue
                kind = next(iter(item["inner"]))
                child = f"{path}::{item['name']}"
                if kind == "function":
                    self.add(child, callable_slots(crate, item["inner"][kind], self_type), "callable")
                elif kind == "assoc_const":
                    self.add(child, {"type": crate.ty(item["inner"][kind]["type"], self_type)}, "constant")

    def walk_trait(self, inner, path):
        crate = self.crate
        self.add(path, {"modifiers[auto]": "auto"} if inner.get("is_auto") else {}, "type")
        for member in inner["items"]:
            item = crate.index.get(str(member))
            if not item:
                continue
            kind = next(iter(item["inner"]))
            child = f"{path}::{item['name']}"
            body = item["inner"][kind]
            if kind == "function":
                slots = callable_slots(crate, body, None)
                if body.get("has_body"):
                    slots["modifiers[provided]"] = "provided"
                self.add(child, slots, "callable")
            elif kind == "assoc_const":
                slots = {"type": crate.ty(body["type"])}
                if body.get("value") is not None:
                    slots["modifiers[provided]"] = "provided"
                self.add(child, slots, "constant")
            elif kind == "assoc_type":
                slots = {}
                if body.get("type") is not None:
                    slots["type"] = crate.ty(body["type"])
                self.add(child, slots, "type-alias")


# ------------------------------------------------------------------- probes


def source_text(text):
    """A report's type text as Rust source: `@{a::b}` to `::a::b`."""
    return re.sub(r"@\{([^}]*)\}", lambda m: "::" + rust_path(public_path(m.group(1))), text)


def public_path(path):
    """A standard library name by the path a consumer can write. Rustdoc gives the
    defining path, whose inner modules are private: `core::future::future::Future`
    is public as `core::future::Future`."""
    parts = path.split("::")
    if parts[0] in ("core", "alloc", "std") and len(parts) > 3:
        return "::".join([parts[0], parts[1], parts[-1]])
    return path


def rust_path(path):
    keywords = {"type", "match", "fn", "mod", "in", "ref", "use", "move", "impl", "dyn", "as", "loop", "try"}
    return "::".join(f"r#{part}" if part in keywords else part for part in path.split("::"))


def declared_bounds(crate, generics):
    """`{parameter: [bound text]}` the declaration itself requires."""
    bounds = {}
    for param in generics.get("params", []):
        kind = param["kind"].get("type") if isinstance(param["kind"], dict) else None
        if kind:
            bounds.setdefault(param["name"], []).extend(
                crate.bound(b, None) for b in kind.get("bounds", [])
            )
    for predicate in generics.get("where_predicates", []):
        bound = predicate.get("bound_predicate")
        if bound and "generic" in bound["type"]:
            bounds.setdefault(bound["type"]["generic"], []).extend(
                crate.bound(b, None) for b in bound["bounds"]
            )
    return bounds


def head_and_clause(crate, generics, extra=()):
    """Renders generics as a probe function's head and where clause. Each type
    parameter outlives each lifetime, which a type with a reference field needs."""
    lifetimes = [p["name"] for p in generics.get("params", []) if "lifetime" in p["kind"]]
    types = [p["name"] for p in generics.get("params", []) if "type" in p["kind"] and not p["kind"]["type"].get("is_synthetic")]
    consts = [(p["name"], p["kind"]["const"]["type"]) for p in generics.get("params", []) if "const" in p["kind"]]
    head = lifetimes + types + [f"const {name}: {source_text(crate.ty(ty))}" for name, ty in consts]
    clauses = []
    for name, bounds in declared_bounds(crate, generics).items():
        for bound in bounds:
            clauses.append(f"{name}: {source_text(bound)}")
    for lifetime in lifetimes:
        clauses.extend(f"{name}: {lifetime}" for name in types)
    clauses.extend(extra)
    return (
        f"<{', '.join(head)}>" if head else "",
        f" where {', '.join(clauses)}" if clauses else "",
        ", ".join(lifetimes + types + [name for name, _ in consts]),
    )


def conditions_of(crate, inner, declared):
    """Rustdoc's auto-trait conditions per trait, without what the declaration
    already requires. A trait with no implementation is absent."""
    conditions = {}
    for impl_id in inner.get("impls", []):
        implementation = crate.index.get(str(impl_id))
        if not implementation:
            continue
        block = implementation["inner"]["impl"]
        trait = block.get("trait")
        name = trait["path"].split("::")[-1] if trait else None
        if name not in TRAITS or block.get("is_negative"):
            continue
        predicates = {}
        pairs = []
        for predicate in block["generics"].get("where_predicates", []):
            bound = predicate.get("bound_predicate")
            if bound and "generic" in bound["type"]:
                pairs.append((bound["type"]["generic"], bound["bounds"]))
        for param in block["generics"].get("params", []):
            kind = param["kind"].get("type") if isinstance(param["kind"], dict) else None
            if kind:
                pairs.append((param["name"], kind.get("bounds", [])))
        for subject, bounds in pairs:
            for bound in bounds:
                if "trait_bound" not in bound or bound["trait_bound"].get("modifier") == "maybe":
                    continue
                written = bound["trait_bound"]["trait"]["path"].split("::")[-1]
                text = written if written in TRAITS else crate.bound(bound, None)
                if written == "Sized" or text in declared.get(subject, []):
                    continue
                predicates.setdefault(subject, set()).add(text)
        conditions[name] = sorted(f"{subject}: {' + '.join(sorted(names))}" for subject, names in predicates.items())
    return conditions


def condition_source(predicate):
    subject, bounds = predicate.split(": ", 1)
    parts = [BOUND.get(bound, source_text(bound)) for bound in bounds.split(" + ")]
    return ", ".join(f"{subject}: {part}" for part in parts)


class Probes:
    """The generated crate. A line holds one claim, so that one compile checks
    them all and each failure names its claim."""

    def __init__(self):
        self.lines = ["#![allow(warnings)]", "extern crate alloc;"]
        for trait, bound in BOUND.items():
            self.lines.append(f"fn need_{trait}<T: ?Sized + {bound}>() {{}}")
        self.claims = {}

    def add(self, source, claim):
        self.lines.append(source)
        self.claims[len(self.lines)] = claim


def plan_type(probes, crate, path, consumer, inner, generics):
    declared = declared_bounds(crate, generics)
    conditions = conditions_of(crate, inner, declared)
    for trait in TRAITS:
        wanted = conditions.get(trait, [])
        extra = [condition_source(c) for c in wanted]
        head, clause, args = head_and_clause(crate, generics, extra)
        subject = "::" + rust_path(consumer) + (f"<{args}>" if args else "")
        name = f"p{len(probes.lines)}"
        probes.add(
            f"pub fn {name}{head}(){clause} {{ need_{trait}::<{subject}>(); }}",
            {"path": f"type:{path}", "label": trait, "conditions": wanted, "kind": "holds"},
        )
        # With a condition dropped, the claim fails to compile.
        for dropped in wanted:
            kept = [condition_source(c) for c in wanted if c != dropped]
            head, clause, args = head_and_clause(crate, generics, kept)
            name = f"p{len(probes.lines)}"
            probes.add(
                f"pub fn {name}{head}(){clause} {{ need_{trait}::<{subject}>(); }}",
                {"path": f"type:{path}", "label": trait, "conditions": wanted, "kind": "needs"},
            )


def rewrite_opaque(crate, output, first):
    """The return with each opaque type replaced by a parameter, numbered from
    `first` in source order, and those parameters' bounds. Returns None for a
    form this script does not bind."""
    bounds = {}
    counter = [first]

    def walk(t):
        if "impl_trait" in t:
            name = f"O{counter[0]}"
            counter[0] += 1
            bounds[name] = []
            for bound in t["impl_trait"]:
                if "trait_bound" not in bound:
                    continue
                trait = bound["trait_bound"]["trait"]
                args = (trait.get("args") or {}).get("angle_bracketed") or {}
                constraints = []
                for constraint in args.get("constraints", []):
                    equal = constraint["binding"].get("equality") or {}
                    if "type" in equal:
                        constraints.append(f"{constraint['name']} = {walk(equal['type'])}")
                if constraints:
                    # The path as written: a trait's defining path can be private.
                    written = trait["path"]
                    written = written if "::" not in written else "::" + written
                    bounds[name].append(f"{written}<{', '.join(constraints)}>")
            return name
        if "tuple" in t:
            items = [walk(item) for item in t["tuple"]]
            return f"({items[0]},)" if len(items) == 1 else f"({', '.join(items)})"
        if "resolved_path" in t:
            path = t["resolved_path"]
            target = crate.item_path(path["id"])
            args = (path.get("args") or {}).get("angle_bracketed") or {}
            if not target:
                raise LookupError
            parts = [
                arg["lifetime"] if "lifetime" in arg else walk(arg["type"])
                for arg in args.get("args", [])
                if "lifetime" in arg or "type" in arg
            ]
            return f"::{rust_path(target)}<{', '.join(parts)}>" if parts else f"::{rust_path(target)}"
        if "borrowed_ref" in t:
            ref = t["borrowed_ref"]
            return f"&'static {'mut ' if ref['is_mutable'] else ''}{walk(ref['type'])}"
        return source_text(crate.ty(t))

    try:
        return walk(output), bounds, counter[0]
    except LookupError:
        return None


def condition_subsets(names):
    """Every subset of a callable's type parameters, smallest first."""
    subsets = [[]]
    for name in names:
        subsets += [subset + [name] for subset in subsets]
    return sorted(subsets, key=len)


def plan_opaque(probes, crate, path, function, call, owner_generics, self_type, key, members):
    """Adds one line per occurrence, trait and set of conditions. Each line calls
    the callable with typed arguments and binds its return to a helper's
    signature. `key` is the slot prefix: `auto-traits` or a binding's."""
    signature = function["sig"]
    output = signature.get("output")
    is_async = bool(function["header"].get("is_async"))
    if not is_async and "impl_trait" not in json.dumps(output):
        return
    # A binding's slot is on its implementing type.
    member = f"{'type' if key.startswith('bases[') else 'callable'}:{path}"
    merged = {
        "params": owner_generics.get("params", []) + function["generics"].get("params", []),
        "where_predicates": owner_generics.get("where_predicates", []) + function["generics"].get("where_predicates", []),
    }
    types = [p["name"] for p in merged["params"] if "type" in p["kind"] and not p["kind"]["type"].get("is_synthetic")]
    synthetic = any("type" in p["kind"] and p["kind"]["type"].get("is_synthetic") for p in merged["params"])
    if len(types) > 3 or synthetic:
        members[member]["unstated"].append(key)
        return
    rewritten = rewrite_opaque(crate, output, 1 if is_async else 0) if output is not None else ("()", {}, 1 if is_async else 0)
    if rewritten is None:
        members[member]["unstated"].append(key)
        return
    returned, nested, total = rewritten
    whole = total == 1 and (is_async or "impl_trait" in (output or {}))
    arguments = []
    parameters = []
    for i, (_, ty) in enumerate(signature["inputs"]):
        parameters.append(f"x{i}: {source_text(crate.ty(ty, self_type))}")
        arguments.append(f"x{i}")
    invoke = f"{call}({', '.join(arguments)})"
    occurrences = [f"O{i}" for i in range(total)]
    for ordinal, occurrence in enumerate(occurrences):
        for trait in TRAITS:
            label = trait if whole else f"#{ordinal} {trait}"
            for subset in condition_subsets(types):
                extra = [f"{name}: {BOUND[trait]}" for name in subset]
                head, clause, _ = head_and_clause(crate, merged, extra)
                name = f"p{len(probes.lines)}"
                wanted = sorted(f"{item}: {trait}" for item in subset)
                claim = {"path": member, "label": label, "conditions": wanted, "kind": "opaque", "key": key, "generic": bool(types)}
                helper_params = [
                    f"{o}" + (f": {' + '.join(nested[o])}" if nested.get(o) else "")
                    for o in occurrences
                    if o != "O0" or not is_async
                ]
                if is_async:
                    await_params = ", ".join(helper_params)
                    head_text = head
                    if ordinal == 0:
                        body = f"fn h<F: {BOUND[trait]}>(_: &F) {{}} h(&{invoke});"
                    else:
                        constrained = ", ".join(
                            f"{o}: {BOUND[trait]}" + (" + " + " + ".join(nested[o]) if nested.get(o) else "")
                            if o == occurrence
                            else o + (": " + " + ".join(nested[o]) if nested.get(o) else "")
                            for o in occurrences[1:]
                        )
                        body = (
                            f"fn h<{constrained}>(_: &{returned}) {{}} "
                            f"let f = {invoke}; let _ = async move {{ let out = f.await; h(&out); }};"
                        )
                else:
                    constrained = ", ".join(
                        f"{o}: {BOUND[trait]}" + (" + " + " + ".join(nested[o]) if nested.get(o) else "")
                        if o == occurrence
                        else o + (": " + " + ".join(nested[o]) if nested.get(o) else "")
                        for o in occurrences
                    )
                    body = f"fn h<{constrained}>(_: &{returned}) {{}} h(&{invoke});"
                probes.add(f"pub fn {name}{head}({', '.join(parameters)}){clause} {{ {body} }}", claim)


PROBE_TARGET = "semantic_oracle_probe"


def compile_probe(directory, target_dir):
    command = ["cargo", "check", "--offline", "--message-format=json", "--target-dir", target_dir]
    built = run(command, directory)
    return read_check(built.returncode, built.stdout, built.stderr)


def read_check(returncode, stdout, stderr):
    """A check's `E0277` lines and its lines that fail for another reason. Stops the
    report when the check says nothing about them: a dependency that fails, an error
    outside the generated source, or a failure with no diagnostic to attribute."""
    failed, other = set(), set()
    checked = False
    for line in stdout.splitlines():
        try:
            message = json.loads(line)
        except json.JSONDecodeError:
            continue
        target = (message.get("target") or {}).get("name")
        if message.get("reason") == "compiler-artifact" and target == PROBE_TARGET:
            checked = True
        if message.get("reason") != "compiler-message":
            continue
        body = message["message"]
        if body.get("level") != "error":
            continue
        if target != PROBE_TARGET:
            fail(f"`{target}` failed to compile:\n{body.get('rendered', body.get('message'))}")
        spans = [s for s in body["spans"] if s.get("is_primary") and s["file_name"] == "src/lib.rs"]
        code = (body.get("code") or {}).get("code")
        if not spans:
            # `aborting due to N previous errors`, which has no code, follows the others.
            if code is None and (failed or other):
                continue
            fail(f"the probe failed with no generated line:\n{body.get('rendered', body.get('message'))}")
        for span in spans:
            (failed if code == "E0277" else other).add(span["line_start"])
    if returncode == 0:
        if not checked:
            fail(f"cargo check reported success with no check of the probe:\n{stderr}")
    elif not failed and not other:
        fail(f"cargo check failed with no diagnostic for the probe:\n{stderr}")
    return failed, other


def settle(probes, failed, unusable, members, times):
    """Turns compile results into slots. A claim whose line failed for another
    reason states nothing, so its member's slot is unstated."""
    best = {}
    for line, claim in probes.claims.items():
        path = claim["path"]
        key = claim.get("key", "auto-traits")
        if line in unusable:
            if key not in members[path]["unstated"]:
                members[path]["unstated"].append(key)
            times["unusable"] += 1
            continue
        passed = line not in failed
        if claim["kind"] == "needs":
            # The condition is needed, so this line fails. If it compiles, the
            # type holds the trait with less than rustdoc's conditions state.
            if passed and key not in members[path]["unstated"]:
                members[path]["unstated"].append(key)
            continue
        if not passed:
            continue
        label = claim["label"]
        current = best.get((path, key, label))
        if current is None or len(claim["conditions"]) < len(current):
            best[(path, key, label)] = claim["conditions"]
    # A generic callable can hold a trait under a condition these lines do not try,
    # so no passing line does not show that the trait is absent.
    for claim in probes.claims.values():
        wanted = (claim["path"], claim.get("key", "auto-traits"), claim["label"])
        if claim.get("generic") and wanted not in best:
            element = f"{wanted[1]}[{wanted[2]}]"
            if element not in members[claim["path"]]["unstated"]:
                members[claim["path"]]["unstated"].append(element)
    for (path, key, label), conditions in best.items():
        written = label + (" when " + ", ".join(conditions) if conditions else "")
        members[path]["slots"][f"{key}[{label}]"] = written
    for member in members.values():
        for key in member["unstated"]:
            for slot in [slot for slot in member["slots"] if slot.startswith(key + "[")]:
                del member["slots"][slot]


def probe_auto_traits(crate_data, packages, selection, members, target_dir):
    directory = tempfile.mkdtemp(prefix="semantic-oracle-probe-")
    times = {"compiles": 0, "seconds": 0.0, "probes": 0, "unusable": 0}
    try:
        probes = Probes()
        for crate, plans in crate_data:
            for path, consumer, inner, generics in plans["types"]:
                plan_type(probes, crate, path, consumer, inner, generics)
            for path, function, call, owner, self_type, key in plans["callables"]:
                plan_opaque(probes, crate, path, function, call, owner, self_type, key, members)
        times["probes"] = len(probes.claims)
        dependencies = "\n".join(dependency_line(package, selection) for package in packages)
        os.makedirs(os.path.join(directory, "src"), exist_ok=True)
        with open(os.path.join(directory, "Cargo.toml"), "w") as manifest:
            manifest.write(
                "[package]\nname = \"semantic-oracle-probe\"\nversion = \"0.0.0\"\nedition = \"2021\"\n"
                f"[dependencies]\n{dependencies}\n[workspace]\n"
            )
        unusable, failed = set(), set()
        settled = False
        for _ in range(5):
            with open(os.path.join(directory, "src", "lib.rs"), "w") as source:
                for i, line in enumerate(probes.lines, 1):
                    source.write(("// " + line if i in unusable else line) + "\n")
            started = time.monotonic()
            failed, other = compile_probe(directory, target_dir)
            times["seconds"] += time.monotonic() - started
            times["compiles"] += 1
            if not other - unusable:
                settled = True
                break
            unusable |= other
        if not settled:
            fail(f"the probe still had unattributed failures after {times['compiles']} compiles")
        settle(probes, failed, unusable, members, times)
    finally:
        shutil.rmtree(directory, ignore_errors=True)
    return times


def collect_probe_plans(crate, members):
    """What a consumer can reach: its types and the callables with opaque returns,
    each by the consumer path a probe can write, keyed by canonical path."""
    plans = {"types": [], "callables": []}
    seen = set()
    root = crate.index[str(crate.data["root"])]

    def plan_impls(inner, path, consumer):
        for impl_id in inner.get("impls", []):
            implementation = crate.index.get(str(impl_id))
            if not implementation:
                continue
            if implementation["inner"]["impl"].get("is_synthetic") or implementation["inner"]["impl"].get("blanket_impl"):
                continue
            block = implementation["inner"]["impl"]
            self_type = crate.ty(block["for"])
            subject = source_text(self_type)
            trait = block.get("trait")
            if trait and not crate.item_path(trait["id"]):
                continue
            base = crate.name(trait["id"]) + crate.generic_args(trait.get("args"), None) if trait else None
            for member in block["items"]:
                method = crate.index.get(str(member))
                if not method or "function" not in method["inner"]:
                    continue
                if not trait and method.get("visibility") != "public":
                    continue
                name = method["name"]
                if trait:
                    call = f"<{subject} as {source_text(base)}>::{name}"
                    plans["callables"].append((path, method["inner"]["function"], call, block["generics"], self_type, f"bases[{base}].bindings[{name}].auto-traits"))
                else:
                    child = f"{path}::{name}"
                    add_member(members, child, "callable")
                    call = f"{subject}::{name}" if block["generics"].get("params") else f"::{rust_path(consumer)}::{name}"
                    plans["callables"].append((child, method["inner"]["function"], call, block["generics"], self_type, "auto-traits"))

    def plan(item, canonical, consumer):
        kind = next(iter(item["inner"]))
        if kind in ("struct", "enum", "union") and canonical not in seen:
            seen.add(canonical)
            inner = item["inner"][kind]
            add_member(members, canonical, "type")
            plans["types"].append((canonical, consumer, inner, inner["generics"]))
            plan_impls(inner, canonical, consumer)
        elif kind == "function" and canonical not in seen:
            seen.add(canonical)
            add_member(members, canonical, "callable")
            call = "::" + rust_path(consumer)
            plans["callables"].append((canonical, item["inner"]["function"], call, {}, None, "auto-traits"))

    visited = set()

    def visit(module_id, prefix):
        if (module_id, prefix) in visited:
            return
        visited.add((module_id, prefix))
        for child in crate.index[str(module_id)]["inner"]["module"]["items"]:
            item = crate.index.get(str(child))
            if not item or item.get("visibility") != "public":
                continue
            kind = next(iter(item["inner"]))
            if kind == "use":
                use = item["inner"]["use"]
                target = crate.index.get(str(use.get("id")))
                if not target:
                    continue
                target_kind = next(iter(target["inner"]))
                if target_kind == "module":
                    visit(target["id"], prefix if use.get("is_glob") else f"{prefix}::{use['name']}")
                elif not use.get("is_glob"):
                    canonical = crate.item_path(target["id"]) or f"{prefix}::{use['name']}"
                    plan(target, canonical, f"{prefix}::{use['name']}")
            elif kind == "module":
                visit(item["id"], f"{prefix}::{item['name']}")
            elif item.get("name"):
                path = f"{prefix}::{item['name']}"
                plan(item, crate.item_path(item["id"]) or path, path)

    visit(crate.data["root"], root["name"])
    return plans


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--project", required=True)
    parser.add_argument("--context", required=True)
    parser.add_argument("--out", required=True)
    parser.add_argument("--features", default="")
    parser.add_argument("--no-default-features", action="store_true")
    parser.add_argument("--all-features", action="store_true")
    parser.add_argument("--cfg", action="append", default=[])
    arguments = parser.parse_args()
    original = os.path.abspath(arguments.project)
    out = os.path.abspath(arguments.out)
    if out == original or out.startswith(original + os.sep):
        fail(f"--out {out} lies inside the project tree")
    if os.path.exists(out):
        os.remove(out)
    work = tempfile.mkdtemp(prefix="semantic-oracle-rust-")
    started = time.monotonic()
    try:
        project = os.path.join(work, "project")
        shutil.copytree(original, project, symlinks=True, ignore=shutil.ignore_patterns("target", ".git"))
        target_dir = os.path.join(work, "target")
        select_toolchain(project)
        # A context's `cfg` reaches rustdoc and the probe's dependencies alike.
        if arguments.cfg:
            TOOLCHAIN["RUSTFLAGS"] = cfg_flags(arguments)
        metadata = cargo_metadata(project)
        doc_dir = rustdoc_json(project, target_dir, arguments)
        doc_seconds = time.monotonic() - started
        members = {}
        crate_data = []
        packages = []
        format_version = None
        for package in metadata["packages"]:
            for target in package["targets"]:
                if not LIBRARY_KINDS & set(target["kind"]):
                    continue
                name = target["name"].replace("-", "_")
                path = os.path.join(doc_dir, f"{name}.json")
                if not os.path.exists(path):
                    continue
                with open(path) as source:
                    crate = Crate(json.load(source))
                format_version = crate.data.get("format_version")
                walker = Walker(crate)
                walker.walk_module(crate.data["root"], crate.index[str(crate.data["root"])]["name"])
                for member in walker.members.values():
                    member["unit"] = {"package": package["name"], "package-version": package["version"]}
                members.update(walker.members)
                if "proc-macro" not in target["kind"]:
                    crate_data.append((crate, collect_probe_plans(crate, members)))
                    packages.append(package)
        # A type or callable that no public path reaches has nothing to probe.
        planned = set()
        for _, plans in crate_data:
            planned |= {f"type:{plan[0]}" for plan in plans["types"]}
            planned |= {f"callable:{plan[0]}" for plan in plans["callables"] if plan[5] == "auto-traits"}
        for key, member in members.items():
            if key not in planned and key.split(":")[0] in ("type", "callable"):
                member["unstated"].append("auto-traits")
        probe_target = os.path.join(work, "probe-target")
        times = probe_auto_traits(crate_data, packages, arguments, members, probe_target)
        version = run(["rustc", "--version"], project).stdout.strip()
        report = {
            "format": "semantic-oracle-compiler-report/1",
            "language": "rust",
            "context": arguments.context,
            "tool": {"name": "rustdoc-json", "version": f"{version}, format {format_version}"},
            "covers": COVERS,
            "members": sorted(members.values(), key=lambda member: (member["path"], member["kind"])),
        }
        with open(out, "w") as written:
            json.dump(report, written, indent=1, sort_keys=True)
        print(json.dumps({
            "members": len(members),
            "rustdoc_seconds": round(doc_seconds, 2),
            "probe_seconds": round(times["seconds"], 2),
            "probe_compiles": times["compiles"],
            "probes": times["probes"],
            "probes_unusable": times["unusable"],
            "total_seconds": round(time.monotonic() - started, 2),
        }))
    finally:
        shutil.rmtree(work, ignore_errors=True)


if __name__ == "__main__":
    main()
