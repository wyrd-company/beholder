#!/usr/bin/env python3
# ---
# relationships:
#   references: semantic-oracle-model
# ---
"""Reports whether the installed toolchains can check each fixture project.

The language profiles name no exact corpus toolchain. Each asks for a toolchain
that supports what the project declares. This script reads those declarations
for every `<language>/projects/<name>` directory under the fixtures directory
and compares them with the installed toolchains:

  csharp      global.json sdk.version and rollForward, and every TargetFramework(s),
              against `dotnet --list-sdks`
  dart        each pubspec's environment.sdk against `dart --version`
  go          each go.mod's go and toolchain directives against `go version`
  rust        rust-toolchain.toml's channel and targets, and each Cargo.toml's
              rust-version and edition, against `rustc --version`
  python      each requires-python against the python3.N interpreters on PATH
  typescript  the typescript installed in the project's node_modules, or its
              package.json pin, and Node.js 22.6 or later

Manifests under vendor/, node_modules/, obj/, bin/, target/ and other build or
dependency directories belong to dependencies, and are not read.

One line per project: its state, its name and what decided the state. States,
worst first:

  missing-toolchain  a toolchain the project needs is not installed
  unsupported        the installed toolchain does not satisfy a declaration
  pin-not-honoured   an exact pin cannot select its version, because nothing
                     installed switches toolchains (rustc without rustup); the
                     installed version is used instead
  ok                 the installed toolchain satisfies every declaration

Each context of a project's semantic-oracle.yml names the toolchain release its
meaning depends on: `release` (C#, Dart, Go, Rust), `python` or `compiler`
(TypeScript). A line `<language>/<project>#<context>` reports it:

  release-mismatch   no installed release equals the parameter or extends it
                     with dotted components, or the context has no parameter
  ok                 the release the project runs under matches it

Exits 1 when any project is unsupported or missing a toolchain, or any context
is release-mismatch.

  toolchains.py [--fixtures DIR]
"""
import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import tomllib

STATES = ("missing-toolchain", "unsupported", "pin-not-honoured", "ok")
FAILING = {"missing-toolchain", "unsupported"}
# The context parameter that names the toolchain release a dataset's meaning
# depends on. A report's release must equal it or extend it with dotted components.
RELEASE_PARAMETER = {"csharp": "release", "dart": "release", "go": "release",
                     "python": "python", "rust": "release", "typescript": "compiler"}
LANGUAGES = ("csharp", "dart", "go", "python", "rust", "typescript")
SKIPPED = {".git", ".dart_tool", "bin", "build", "node_modules", "obj", "target", "vendor"}
NODE_MINIMUM = (22, 6, 0)
RUST_EDITIONS = {"2015": (1, 0, 0), "2018": (1, 31, 0), "2021": (1, 56, 0), "2024": (1, 85, 0)}


class Result:
    def __init__(self):
        self.state = "ok"
        self.notes = []
        # The releases a context's release parameter can match: what the
        # installed toolchain runs for this project.
        self.releases = []

    def add(self, state, note):
        if STATES.index(state) < STATES.index(self.state):
            self.state = state
        self.notes.append(f"{state}: {note}" if state != "ok" else note)

    def line(self):
        # Problems first, so that the reason for the state leads the line.
        notes = sorted(self.notes, key=lambda note: next(
            (index for index, state in enumerate(STATES[:-1]) if note.startswith(state + ":")),
            len(STATES)))
        return "; ".join(notes)


# Versions -----------------------------------------------------------------

def numbers(text):
    """The leading dotted integers of a version, as a tuple."""
    match = re.match(r"\s*v?(\d+(?:\.\d+)*)", text)
    if not match:
        raise ValueError(f"not a version: {text!r}")
    return tuple(int(part) for part in match.group(1).split("."))


def padded(version, length=3):
    return tuple(version) + (0,) * (length - len(version))


def shown(version):
    return ".".join(str(part) for part in version)


def run(command, cwd=None, env=None):
    """The command's stdout, or None when it is not installed or fails."""
    if shutil.which(command[0]) is None:
        return None
    try:
        done = subprocess.run(command, capture_output=True, text=True, cwd=cwd, env=env, timeout=120)
    except (OSError, subprocess.TimeoutExpired):
        return None
    if done.returncode != 0:
        return None
    return done.stdout + done.stderr


class Toolchains:
    """Probes each toolchain once, on first use."""

    def __init__(self):
        self.cache = {}
        self.neutral = tempfile.mkdtemp(prefix="toolchains-")

    def probe(self, name, function):
        if name not in self.cache:
            self.cache[name] = function()
        return self.cache[name]

    def dotnet_sdks(self):
        def find():
            out = run(["dotnet", "--list-sdks"], cwd=self.neutral)
            if out is None:
                return None
            return sorted(numbers(line) for line in out.splitlines() if re.match(r"\s*\d", line))
        return self.probe("dotnet", find)

    def dart(self):
        def find():
            out = run(["dart", "--version"], cwd=self.neutral)
            match = out and re.search(r"Dart SDK version: (\S+)", out)
            return match.group(1) if match else None
        return self.probe("dart", find)

    def go(self):
        def find():
            env = dict(os.environ, GOTOOLCHAIN="local")
            out = run(["go", "version"], cwd=self.neutral, env=env)
            match = out and re.search(r"go version go(\d+(?:\.\d+)*)", out)
            return numbers(match.group(1)) if match else None
        return self.probe("go", find)

    def rustup(self):
        return self.probe("rustup", lambda: shutil.which("rustup") is not None)

    def rustc(self, cwd):
        """rustc's version and sysroot. With rustup, as selected in cwd."""
        def find():
            out = run(["rustc", "--version"], cwd=cwd)
            match = out and re.search(r"rustc (\d+\.\d+\.\d+)(-\S+)?", out)
            if not match:
                return None
            sysroot = (run(["rustc", "--print", "sysroot"], cwd=cwd) or "").strip()
            return numbers(match.group(1)), (match.group(2) or "").lstrip("-"), sysroot
        return self.probe(("rustc", cwd if self.rustup() else None), find)

    def pythons(self):
        def find():
            found = {}
            for directory in os.environ.get("PATH", "").split(os.pathsep):
                if not os.path.isdir(directory):
                    continue
                for name in sorted(os.listdir(directory)):
                    if name in found or not re.fullmatch(r"python3\.\d+", name):
                        continue
                    path = os.path.join(directory, name)
                    if not os.access(path, os.X_OK):
                        continue
                    out = run([path, "--version"])
                    match = out and re.search(r"Python (\d+\.\d+\.\d+)", out)
                    if match:
                        found[name] = numbers(match.group(1))
            return found
        return self.probe("python", find)

    def node(self):
        def find():
            out = run(["node", "--version"], cwd=self.neutral)
            return numbers(out) if out else None
        return self.probe("node", find)


def manifests(project, *names, suffix=None):
    """Manifests of the project itself, outside dependency and build trees."""
    found = []
    for directory, subdirectories, files in os.walk(project):
        subdirectories[:] = sorted(name for name in subdirectories if name not in SKIPPED)
        for name in sorted(files):
            if name in names or (suffix and name.endswith(suffix)):
                found.append(os.path.join(directory, name))
    return found


def relative(path, project):
    return os.path.relpath(path, project)


def read(path):
    with open(path, encoding="utf-8") as source:
        return source.read()


# C# -----------------------------------------------------------------------

def sdk_band(version):
    major, minor, patch = padded(version)
    return major, minor, patch // 100


def roll_forward(requested, policy, installed):
    """The SDK global.json selects, or None. Follows the .NET SDK's rules."""
    requested = padded(requested)
    policy = policy or "latestPatch"
    if policy == "disable":
        candidates = [sdk for sdk in installed if padded(sdk) == requested]
    elif policy in ("patch", "latestPatch"):
        candidates = [sdk for sdk in installed
                      if sdk_band(sdk) == sdk_band(requested) and padded(sdk) >= requested]
    elif policy in ("feature", "latestFeature"):
        candidates = [sdk for sdk in installed if padded(sdk)[:2] == requested[:2] and padded(sdk) >= requested]
    elif policy in ("minor", "latestMinor"):
        candidates = [sdk for sdk in installed if padded(sdk)[0] == requested[0] and padded(sdk) >= requested]
    elif policy in ("major", "latestMajor"):
        candidates = [sdk for sdk in installed if padded(sdk) >= requested]
    else:
        raise ValueError(f"unknown rollForward {policy!r}")
    return max(candidates) if candidates else None


def framework_major(framework):
    """The SDK major a target framework needs, or 0 when any SDK builds it."""
    match = re.fullmatch(r"net(?:coreapp)?(\d+)\.\d+(?:-[\w.]+)?", framework.strip())
    return int(match.group(1)) if match else 0


def check_csharp(project, tools, result):
    sdks = tools.dotnet_sdks()
    if not sdks:
        result.add("missing-toolchain", "no .NET SDK (dotnet --list-sdks)")
        return
    selected = max(sdks)
    for path in manifests(project, "global.json"):
        sdk = json.loads(read(path)).get("sdk", {})
        if "version" not in sdk:
            continue
        policy = sdk.get("rollForward")
        chosen = roll_forward(numbers(sdk["version"]), policy, sdks)
        where = relative(path, project)
        if chosen is None:
            result.add("unsupported", f"{where} selects SDK {sdk['version']} "
                       f"(rollForward {policy or 'latestPatch'}); installed {', '.join(map(shown, sdks))}")
        else:
            result.add("ok", f"{where} selects SDK {shown(chosen)}")
            if os.path.dirname(path) == project:
                selected = chosen
    frameworks = set()
    for path in manifests(project, suffix=(".csproj", ".props", ".targets")):
        for match in re.finditer(r"<TargetFrameworks?>([^<]*)<", read(path)):
            frameworks.update(item.strip() for item in match.group(1).split(";")
                              if item.strip() and "$" not in item)
    if not frameworks:
        result.add("ok", "no TargetFramework")
    for framework in sorted(frameworks):
        if framework_major(framework) > selected[0]:
            result.add("unsupported", f"{framework} needs SDK {framework_major(framework)}, have {shown(selected)}")
    result.releases = [shown(selected)]
    supported = sorted(framework for framework in frameworks if framework_major(framework) <= selected[0])
    if supported:
        result.add("ok", f"SDK {shown(selected)} targets {', '.join(supported)}")


# Dart ---------------------------------------------------------------------

def dart_version(text):
    """A comparable semver: pre-releases sort before their release."""
    match = re.fullmatch(r"(\d+)\.(\d+)\.(\d+)(?:-([0-9A-Za-z.-]+))?(?:\+[0-9A-Za-z.-]+)?", text.strip())
    if not match:
        raise ValueError(f"not a Dart version: {text!r}")
    release = tuple(int(part) for part in match.group(1, 2, 3))
    pre = match.group(4)
    if pre is None:
        return release + (1, ())
    parts = tuple((0, int(part), "") if part.isdigit() else (1, 0, part) for part in pre.split("."))
    return release + (0, parts)


def dart_constraint(text):
    """A list of (operator, version) bounds, with the Dart 3 reading of `<3.0.0`."""
    text = text.strip().strip("'\"")
    if text == "any":
        return []
    bounds = []
    for token in re.findall(r"(\^|>=|<=|>|<)?\s*([0-9][0-9A-Za-z.+-]*)", text):
        operator, version = token
        if operator == "^":
            major, minor, patch = (int(part) for part in re.match(r"(\d+)\.(\d+)\.(\d+)", version).groups())
            upper = f"{major + 1}.0.0" if major > 0 else f"0.{minor + 1}.0"
            bounds += [(">=", version), ("<", upper)]
        else:
            bounds.append((operator or "==", version))
    # Dart 3 reads an upper bound of <3.0.0 as <4.0.0 when the lower bound is
    # at least 2.12.0, the first null-safe release.
    lower = [version for operator, version in bounds if operator in (">=", ">")]
    if lower and dart_version(lower[0]) >= dart_version("2.12.0-0"):
        bounds = [(operator, "4.0.0" if (operator, dart_version(version)) == ("<", dart_version("3.0.0"))
                   else version) for operator, version in bounds]
    return bounds


def satisfies(version, bounds, parse):
    have = parse(version) if isinstance(version, str) else version
    tests = {
        ">=": lambda want: have >= want, ">": lambda want: have > want,
        "<=": lambda want: have <= want, "<": lambda want: have < want,
        "==": lambda want: have == want, "!=": lambda want: have != want,
    }
    return all(tests[operator](parse(want) if isinstance(want, str) else want) for operator, want in bounds)


def pubspec_sdk(text):
    """environment.sdk of a pubspec, read without a YAML parser."""
    block = re.search(r"^environment:[ \t]*(?:#.*)?\n((?:[ \t]+.*\n?|[ \t]*\n)*)", text, re.M)
    if not block:
        return None
    match = re.search(r"^[ \t]+sdk:[ \t]*(.+?)[ \t]*(?:#.*)?$", block.group(1), re.M)
    return match.group(1).strip().strip("'\"") if match else None


def check_dart(project, tools, result):
    installed = tools.dart()
    if installed is None:
        result.add("missing-toolchain", "no Dart SDK (dart --version)")
        return
    result.releases = [installed]
    passed = 0
    for path in manifests(project, "pubspec.yaml"):
        constraint = pubspec_sdk(read(path))
        where = relative(path, project)
        if constraint is None:
            result.add("unsupported", f"{where} declares no environment.sdk")
        elif not satisfies(installed, dart_constraint(constraint), dart_version):
            result.add("unsupported", f"{where} sdk {constraint!r} excludes Dart {installed}")
        else:
            passed += 1
    if passed:
        result.add("ok", f"Dart {installed} satisfies {passed} pubspec(s)")
    elif not result.notes:
        result.add("ok", "no pubspec.yaml")


# Go -----------------------------------------------------------------------

def go_version(text):
    """go1.21rc1 and 1.21 sort before 1.21.0; release candidates by number."""
    match = re.fullmatch(r"(?:go)?(\d+)\.(\d+)(?:\.(\d+))?(?:(rc|beta)(\d+))?", text.strip())
    if not match:
        raise ValueError(f"not a Go version: {text!r}")
    major, minor, patch, kind, number = match.groups()
    if patch is not None:
        return (int(major), int(minor), int(patch), 2, 0)
    if kind:
        return (int(major), int(minor), 0, {"beta": 0, "rc": 1}[kind], int(number))
    return (int(major), int(minor), -1, 2, 0)


def check_go(project, tools, result):
    installed = tools.go()
    if installed is None:
        result.add("missing-toolchain", "no Go (go version)")
        return
    have = go_version(shown(padded(installed)))
    result.releases = [f"go{shown(installed)}"]
    passed = 0
    for path in manifests(project, "go.mod"):
        where = relative(path, project)
        text = read(path)
        for directive in ("go", "toolchain"):
            match = re.search(rf"^{directive}\s+(\S+)", text, re.M)
            if match and go_version(match.group(1)) > have:
                result.add("unsupported", f"{where} {directive} {match.group(1)} is newer than go{shown(installed)}")
                break
        else:
            passed += 1
    if passed:
        result.add("ok", f"go{shown(installed)} satisfies {passed} go.mod(s)")


# Rust ---------------------------------------------------------------------

def check_rust(project, tools, result):
    rustc = tools.rustc(project)
    if rustc is None:
        result.add("missing-toolchain", "no rustc (rustc --version)")
        return
    installed, channel, sysroot = rustc
    result.releases = [shown(installed)]
    channel = channel.split("-")[0] or "stable"
    toolchain = os.path.join(project, "rust-toolchain.toml")
    if os.path.exists(toolchain):
        table = tomllib.loads(read(toolchain)).get("toolchain", {})
        pin = table.get("channel")
        if pin:
            matches = (pin == channel if not re.match(r"\d", pin)
                       else padded(numbers(pin))[:len(pin.split("."))] == installed[:len(pin.split("."))]
                       and channel == "stable")
            if not matches:
                how = "rustup is not installed" if not tools.rustup() else "rustup did not select it"
                result.add("pin-not-honoured",
                           f"rust-toolchain.toml channel {pin}, {how}; rustc {shown(installed)} {channel} is used")
        for target in table.get("targets", []):
            if sysroot and not os.path.isdir(os.path.join(sysroot, "lib", "rustlib", target)):
                result.add("pin-not-honoured", f"rust-toolchain.toml target {target} has no standard library installed")
    passed = 0
    for path in manifests(project, "Cargo.toml"):
        where = relative(path, project)
        manifest = tomllib.loads(read(path))
        failed = False
        for table in (manifest.get("package", {}), manifest.get("workspace", {}).get("package", {})):
            minimum = table.get("rust-version")
            if isinstance(minimum, str) and padded(numbers(minimum)) > installed:
                result.add("unsupported", f"{where} rust-version {minimum} is newer than rustc {shown(installed)}")
                failed = True
            edition = table.get("edition")
            if isinstance(edition, str) and RUST_EDITIONS.get(edition, (99, 0, 0)) > installed:
                result.add("unsupported", f"{where} edition {edition} needs a newer rustc than {shown(installed)}")
                failed = True
        passed += not failed
    if passed:
        result.add("ok", f"rustc {shown(installed)} satisfies {passed} Cargo.toml(s)")


# Python -------------------------------------------------------------------

def pep440_satisfies(version, specifier):
    """A requires-python specifier on a release version, without pre-releases."""
    for clause in filter(None, (part.strip() for part in specifier.split(","))):
        match = re.fullmatch(r"(~=|===|==|!=|>=|<=|>|<)\s*([0-9][0-9.]*)(\.\*)?", clause)
        if not match:
            raise ValueError(f"unsupported requires-python clause {clause!r}")
        operator, text, wildcard = match.groups()
        want = numbers(text)
        size = max(len(want), len(version))
        have, full = padded(version, size), padded(want, size)
        if operator in ("==", "!=", "===") and wildcard:
            equal = tuple(version[:len(want)]) == want
            ok = equal if operator == "==" else not equal
        elif operator in ("==", "==="):
            ok = have == full
        elif operator == "!=":
            ok = have != full
        elif operator == "~=":
            ok = have >= full and tuple(version[:len(want) - 1]) == want[:-1]
        else:
            ok = {">=": have >= full, "<=": have <= full, ">": have > full, "<": have < full}[operator]
        if not ok:
            return False
    return True


def check_python(project, tools, result):
    interpreters = tools.pythons()
    if not interpreters:
        result.add("missing-toolchain", "no python3.N interpreter on PATH")
        return
    result.releases = [shown(version) for version in interpreters.values()]
    declared = {}
    for path in manifests(project, "pyproject.toml"):
        specifier = tomllib.loads(read(path)).get("project", {}).get("requires-python")
        if specifier is not None:
            declared.setdefault(specifier, []).append(relative(path, project))
    for specifier, paths in declared.items():
        where = paths[0] if len(paths) == 1 else f"{len(paths)} pyproject.toml"
        fitting = [name for name, version in interpreters.items() if pep440_satisfies(version, specifier)]
        if fitting:
            result.add("ok", f"requires-python {specifier!r} ({where}): {', '.join(fitting)}")
        else:
            result.add("unsupported", f"requires-python {specifier!r} ({', '.join(paths)}) matches none of "
                       f"{', '.join(f'{name} ({shown(version)})' for name, version in interpreters.items())}")
    if not declared:
        result.add("ok", f"no requires-python; {', '.join(interpreters)} available")


# TypeScript ---------------------------------------------------------------

def check_typescript(project, tools, result):
    node = tools.node()
    if node is None:
        result.add("missing-toolchain", "no Node.js (node --version)")
    elif padded(node) < NODE_MINIMUM:
        result.add("unsupported", f"Node.js {shown(node)} is older than {shown(NODE_MINIMUM)}")
    else:
        result.add("ok", f"Node.js {shown(node)}")
    pin = None
    package = os.path.join(project, "package.json")
    if os.path.exists(package):
        manifest = json.loads(read(package))
        for field in ("devDependencies", "dependencies"):
            pin = pin or manifest.get(field, {}).get("typescript")
    installed = os.path.join(project, "node_modules", "typescript", "package.json")
    if os.path.exists(installed):
        version = json.loads(read(installed))["version"]
        result.releases = [version]
        note = f"typescript {version} installed"
        if pin and pin != version and re.fullmatch(r"\d+\.\d+\.\d+", pin):
            note += f" (package.json pins {pin})"
        result.add("ok", note)
    elif pin:
        result.add("missing-toolchain", f"typescript {pin} is declared and not installed in node_modules (npm ci)")
    else:
        result.add("missing-toolchain", "no typescript declared or installed")


def contexts(text):
    """(id, parameters) for each entry of a dataset's top-level `contexts`.

    Reads the block style the datasets use, without a YAML parser, and keeps
    only scalar parameters."""
    found = []
    inside = False
    for line in text.splitlines():
        if re.match(r"contexts:\s*(#.*)?$", line):
            inside = True
            continue
        if not inside or not line.strip() or line.lstrip().startswith("#"):
            continue
        if not line[0].isspace() and not line.startswith("-"):
            break
        entry = re.match(r"\s*-\s+id:\s*(['\"]?)([^'\"#]+?)\1\s*(?:#.*)?$", line)
        if entry:
            found.append((entry.group(2), {}))
            continue
        parameter = re.match(r"\s+([\w-]+):\s*(['\"]?)([^'\"{\[#]*?)\2\s*(?:#.*)?$", line)
        if found and parameter and parameter.group(3):
            found[-1][1].setdefault(parameter.group(1), parameter.group(3))
    return found


def extends(release, wanted):
    return release == wanted or release.startswith(wanted + ".")


def check_releases(language, project, result):
    """One (state, context, note) for each context of the project's datasets."""
    key = RELEASE_PARAMETER[language]
    lines = []
    for path in manifests(project, "semantic-oracle.yml"):
        prefix = os.path.dirname(relative(path, project))
        for context, parameters in contexts(read(path)):
            name = f"{prefix}/{context}" if prefix else context
            wanted = parameters.get(key)
            installed = ", ".join(result.releases) or "none installed"
            if wanted is None:
                lines.append(("release-mismatch", name, f"declares no `{key}` parameter; installed {installed}"))
            elif any(extends(release, wanted) for release in result.releases):
                lines.append(("ok", name, f"{key} {wanted}; installed {installed}"))
            else:
                lines.append(("release-mismatch", name, f"{key} {wanted}; installed {installed}"))
    return lines


CHECKS = {
    "csharp": check_csharp, "dart": check_dart, "go": check_go,
    "python": check_python, "rust": check_rust, "typescript": check_typescript,
}


def projects(root):
    for language in LANGUAGES:
        directory = os.path.join(root, language, "projects")
        if not os.path.isdir(directory):
            continue
        for name in sorted(os.listdir(directory)):
            if os.path.isdir(os.path.join(directory, name)):
                yield language, name, os.path.join(directory, name)


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--fixtures", default="fixtures")
    arguments = parser.parse_args()
    root = os.path.abspath(arguments.fixtures)
    found = list(projects(root))
    if not found:
        print(f"no <language>/projects/<name> directories under {root}; `task fixtures:init` checks out the corpus")
        return 2
    tools = Toolchains()
    counts = {}
    context_counts = {}
    for language, name, project in found:
        result = Result()
        try:
            CHECKS[language](project, tools, result)
        except (ValueError, OSError, tomllib.TOMLDecodeError, json.JSONDecodeError) as error:
            result.add("unsupported", f"cannot read a declaration: {error}")
        counts.setdefault(language, dict.fromkeys(STATES, 0))[result.state] += 1
        print(f"{result.state:<17} {language}/{name}: {result.line()}")
        try:
            releases = check_releases(language, project, result)
        except (ValueError, OSError) as error:
            releases = [("release-mismatch", "semantic-oracle.yml", f"cannot read its contexts: {error}")]
        for state, context, note in releases:
            tally = context_counts.setdefault(language, {"release-mismatch": 0, "ok": 0})
            tally[state] += 1
            print(f"{state:<17} {language}/{name}#{context}: {note}")
    shutil.rmtree(tools.neutral, ignore_errors=True)
    print()
    failing = 0
    for language, tally in counts.items():
        failing += sum(tally[state] for state in FAILING)
        line = " ".join(f"{state} {tally[state]}" for state in STATES if tally[state])
        if language in context_counts:
            line += "; contexts " + " ".join(f"{state} {count}" for state, count in context_counts[language].items()
                                             if count)
        print(f"{language:<10} {line}")
    mismatched = sum(tally["release-mismatch"] for tally in context_counts.values())
    print(f"{len(found)} projects, {failing} cannot be checked with the installed toolchains; "
          f"{sum(sum(tally.values()) for tally in context_counts.values())} dataset contexts, "
          f"{mismatched} release-mismatch")
    return 1 if failing or mismatched else 0


if __name__ == "__main__":
    sys.exit(main())
