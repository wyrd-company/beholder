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
  go          each go.mod's go and toolchain directives against `go version`, and
              the toolchain the effective GOTOOLCHAIN selects for it
  rust        rust-toolchain.toml's channel and targets, and each Cargo.toml's
              rust-version and edition, against `rustc -vV`
  python      each requires-python against the python3.N interpreters on PATH
  typescript  the typescript installed in the project's node_modules against its
              package.json version or range, and Node.js 22.6 or later

Manifests under vendor/, node_modules/, obj/, bin/, target/ and other build or
dependency directories belong to dependencies, and are not read.

One line per project: its state, its name and what decided the state. States,
worst first:

  missing-toolchain  a toolchain the project needs is not installed, or a Rust
                     target it lists has no standard library
  unsupported        the installed toolchain does not satisfy a declaration
  pin-not-honoured   an exact pin cannot select its version, and the installed
                     one is used instead: a rust-toolchain.toml channel that
                     rustup does not select (or there is no rustup), or a
                     typescript version node_modules does not hold
  ok                 the installed toolchain satisfies every declaration

Each context of a project's semantic-oracle.yml names the toolchain release its
meaning depends on: `release` (C#, Dart, Go, Rust), `python` or `compiler`
(TypeScript). A line `<language>/<project>#<context>` reports it:

  release-mismatch   no installed release equals the parameter or extends it
                     with dotted numeric components, or the context has no
                     parameter. A prerelease, such as Python 3.14.0rc1, extends
                     no release: it matches only a parameter naming it exactly
  ok                 the release the project runs under matches it

A line `<language>/<project>#<path>` marks a semantic-oracle.yml whose contexts
cannot be read, or that lists none:

  unresolved         part of its `contexts` is in a form this script does not
                     read, so none of its contexts is checked

Exits 1 when any project is unsupported or missing a toolchain, any context is
release-mismatch, or any semantic-oracle.yml is unresolved.

  toolchains.py [--fixtures DIR]
"""
import argparse
import datetime
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
NODE_MINIMUM = "22.6.0"
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


def typed(value, kind, what):
    """value, or ValueError when a manifest gives it another shape."""
    names = {str: "a string", dict: "a table", list: "a list"}
    if not isinstance(value, kind) or (kind is list and not all(isinstance(item, str) for item in value)):
        raise ValueError(f"{what} is not {names[kind]}" + (" of strings" if kind is list else "")
                         + f": {value!r}")
    return value


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
            found = [line.split()[0] for line in out.splitlines() if re.match(r"\s*\d", line)]
            for sdk in found:
                dart_version(sdk)
            return sorted(found, key=dart_version)
        return self.probe("dotnet", find)

    def dart(self):
        def find():
            out = run(["dart", "--version"], cwd=self.neutral)
            match = out and re.search(r"Dart SDK version: (\S+)", out)
            return match.group(1) if match else None
        return self.probe("dart", find)

    def go(self):
        """The go on PATH's own version, and the effective GOTOOLCHAIN.

        GOTOOLCHAIN=local only identifies the binary on PATH. Which toolchain
        a module selects follows the effective GOTOOLCHAIN: the environment,
        then the go env file, then the default `auto`."""
        def find():
            out = run(["go", "version"], cwd=self.neutral, env=dict(os.environ, GOTOOLCHAIN="local"))
            match = out and re.search(r"go version (go\S+)", out)
            if not match:
                return None
            setting = os.environ.get("GOTOOLCHAIN")
            if not setting:
                # None when the command fails: the switching policy is unknown.
                # An empty value is Go's default, auto.
                out = run(["go", "env", "GOTOOLCHAIN"], cwd=self.neutral)
                setting = None if out is None else (out.strip() or "auto")
            return match.group(1), setting
        return self.probe("go", find)

    def rustup(self):
        return self.probe("rustup", lambda: shutil.which("rustup") is not None)

    def rustc(self, cwd):
        """rustc as selected in cwd: version, channel, commit date, sysroot, and
        rustup's active toolchain name, or None without rustup."""
        def find():
            out = run(["rustc", "-vV"], cwd=cwd)
            match = out and re.search(r"^release: ((\d+\.\d+\.\d+)(?:-(\w+)\S*)?)[ \t]*$", out, re.M)
            if not match:
                return None
            date = re.search(r"^commit-date: (\d{4}-\d{2}-\d{2})", out, re.M)
            sysroot = (run(["rustc", "--print", "sysroot"], cwd=cwd) or "").strip()
            active = None
            if self.rustup():
                active = (run(["rustup", "show", "active-toolchain"], cwd=cwd) or "").split(maxsplit=1)
                active = active[0] if active else None
            return {"release": match.group(1), "version": numbers(match.group(2)),
                    "channel": match.group(3) or "stable",
                    "date": date.group(1) if date else None, "sysroot": sysroot, "active": active}
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
                    # The whole version, with any prerelease suffix: 3.14.0rc1
                    # is not 3.14.0. A trailing + marks a build past it.
                    match = out and re.search(r"Python (\d+\.\d+\.\d+(?:(?:a|b|rc)\d+)?\+?)(?:\s|$)", out)
                    if match:
                        found[name] = match.group(1)
            return found
        return self.probe("python", find)

    def node(self):
        """The version node reports, without its v, prerelease included."""
        def find():
            out = run(["node", "--version"], cwd=self.neutral)
            return out.strip().removeprefix("v") if out else None
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
    """(major, minor, feature band) of an SDK version string."""
    major, minor, patch = padded(numbers(version))
    return major, minor, patch // 100


def sdk_prerelease(version):
    return dart_version(version)[3] == 0


# How far each policy may roll: the leading components of (major, minor,
# feature band) that a selected SDK shares with the requested one.
ROLL_SCOPE = {"patch": 3, "feature": 2, "minor": 1, "major": 0,
              "latestPatch": 3, "latestFeature": 2, "latestMinor": 1, "latestMajor": 0}


def roll_forward(requested, policy, installed, allow_prerelease=True):
    """The SDK version string global.json selects, or None. Follows the .NET
    SDK's rules, on full versions including prerelease identifiers.

    allowPrerelease defaults to true outside Visual Studio; false removes every
    prerelease SDK before rolling forward. `patch`, the default, takes the
    requested version, or else the latest patch in its feature band. `feature`,
    `minor` and `major` take the latest patch in the requested band, or else in
    the nearest higher feature band, minor or major they allow. The `latest*`
    policies take the highest SDK they allow."""
    want = dart_version(requested)
    candidates = [sdk for sdk in installed if allow_prerelease or not sdk_prerelease(sdk)]
    policy = policy or "patch"
    if policy == "disable":
        exact = [sdk for sdk in candidates if dart_version(sdk) == want]
        return exact[0] if exact else None
    if policy not in ROLL_SCOPE:
        raise ValueError(f"unknown rollForward {policy!r}")
    scope = ROLL_SCOPE[policy]
    allowed = sorted((sdk for sdk in candidates
                      if dart_version(sdk) >= want and sdk_band(sdk)[:scope] == sdk_band(requested)[:scope]),
                     key=dart_version)
    if not allowed:
        return None
    if policy.startswith("latest"):
        return allowed[-1]
    if policy == "patch" and any(dart_version(sdk) == want for sdk in allowed):
        return next(sdk for sdk in allowed if dart_version(sdk) == want)
    nearest = min(sdk_band(sdk) for sdk in allowed)
    return [sdk for sdk in allowed if sdk_band(sdk) == nearest][-1]


def framework_major(framework):
    """The SDK major a target framework needs, or 0 when any SDK builds it."""
    match = re.fullmatch(r"net(?:coreapp)?(\d+)\.\d+(?:-[\w.]+)?", framework.strip())
    return int(match.group(1)) if match else 0


def check_csharp(project, tools, result):
    sdks = tools.dotnet_sdks()
    if not sdks:
        result.add("missing-toolchain", "no .NET SDK (dotnet --list-sdks)")
        return
    selected = sdks[-1]
    for path in manifests(project, "global.json"):
        where = relative(path, project)
        sdk = typed(typed(json.loads(read(path)), dict, where).get("sdk", {}), dict, f"{where} sdk")
        allow = sdk.get("allowPrerelease", True)
        if not isinstance(allow, bool):
            raise ValueError(f"{where} sdk.allowPrerelease is not a boolean: {allow!r}")
        if "version" in sdk:
            version = typed(sdk["version"], str, f"{where} sdk.version")
            policy = sdk.get("rollForward")
            if policy is not None:
                typed(policy, str, f"{where} sdk.rollForward")
            chosen = roll_forward(version, policy, sdks, allow)
            asked = f"SDK {version} (rollForward {policy or 'patch'}" + ("" if allow else ", no prerelease") + ")"
        elif "allowPrerelease" in sdk:
            # No version: the highest SDK allowPrerelease allows, as latestMajor.
            chosen = next((candidate for candidate in reversed(sdks) if allow or not sdk_prerelease(candidate)), None)
            asked = "the highest SDK" + ("" if allow else " that is not a prerelease")
        else:
            continue
        if chosen is None:
            result.add("unsupported", f"{where} selects {asked}; installed {', '.join(sdks)}")
        else:
            result.add("ok", f"{where} selects SDK {chosen}")
            if os.path.dirname(path) == project:
                selected = chosen
    frameworks = set()
    for path in manifests(project, suffix=(".csproj", ".props", ".targets")):
        for match in re.finditer(r"<TargetFrameworks?>([^<]*)<", read(path)):
            frameworks.update(item.strip() for item in match.group(1).split(";")
                              if item.strip() and "$" not in item)
    if not frameworks:
        result.add("ok", "no TargetFramework")
    major = numbers(selected)[0]
    for framework in sorted(frameworks):
        if framework_major(framework) > major:
            result.add("unsupported", f"{framework} needs SDK {framework_major(framework)}, have {selected}")
    result.releases = [selected]
    supported = sorted(framework for framework in frameworks if framework_major(framework) <= major)
    if supported:
        result.add("ok", f"SDK {selected} targets {', '.join(supported)}")


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
    text = text.strip().strip("'\"").strip()
    if text == "any":
        return []
    if not text:
        raise ValueError("empty SDK constraint")
    bounds = []
    token = re.compile(r"\s*(\^|>=|<=|>|<)?\s*([0-9][0-9A-Za-z.+-]*)\s*")
    position = 0
    while position < len(text):
        match = token.match(text, position)
        if not match:
            raise ValueError(f"cannot parse SDK constraint {text!r} at {text[position:]!r}")
        position = match.end()
        operator, version = match.groups()
        dart_version(version)
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
        try:
            bounds = None if constraint is None else dart_constraint(constraint)
        except ValueError as error:
            result.add("unsupported", f"{where}: {error}")
            continue
        if constraint is None:
            result.add("unsupported", f"{where} declares no environment.sdk")
        elif not satisfies(installed, bounds, dart_version):
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


def go_setting(setting):
    """(base, mode) of a GOTOOLCHAIN value: auto is local+auto, path local+path."""
    setting = {"auto": "local+auto", "path": "local+path"}.get(setting, setting)
    base, _, mode = setting.partition("+")
    if base != "local":
        go_version(base)
    if mode not in ("", "auto", "path"):
        raise ValueError(f"unknown GOTOOLCHAIN {setting!r}")
    return base, mode


def go_select(local, setting, go, toolchain):
    """(release, note, state) of the toolchain a module selects.

    The go directive is a minimum. A toolchain directive, or a go directive newer
    than the base toolchain, selects a newer release only when GOTOOLCHAIN
    allows switching (+auto, the default, or +path)."""
    base, mode = go_setting(setting)
    selected = local if base == "local" else base
    if toolchain == "default":
        # `toolchain default` keeps the base toolchain: no switching, and the go
        # line is still a minimum.
        if go and go_version(go) > go_version(selected):
            return selected, f"go {go} is newer than {selected}, and go.mod sets toolchain default", "unsupported"
        return selected, None, "ok"
    if mode:
        if toolchain and go_version(toolchain) > go_version(selected):
            selected = toolchain
        if go and go_version(go) > go_version(selected):
            selected = "go" + go
        return selected, None, "ok"
    if go and go_version(go) > go_version(selected):
        return selected, f"go {go} is newer than {selected}, and GOTOOLCHAIN={setting} does not switch", \
            "unsupported"
    if toolchain and go_version(toolchain) > go_version(selected):
        return selected, f"toolchain {toolchain} is a suggestion GOTOOLCHAIN={setting} does not follow", "ok"
    return selected, None, "ok"


def check_go(project, tools, result):
    probed = tools.go()
    if probed is None:
        result.add("missing-toolchain", "no Go (go version)")
        return
    local, setting = probed
    if setting is None:
        result.releases = [local]
        result.add("unsupported", f"GOTOOLCHAIN is unresolved: `go env GOTOOLCHAIN` failed, so which toolchain "
                   f"each go.mod selects is unknown (local {local})")
        return
    releases = []
    passed = 0
    for path in manifests(project, "go.mod"):
        where = relative(path, project)
        text = read(path)
        directives = {}
        for directive in ("go", "toolchain"):
            match = re.search(rf"^{directive}\s+(\S+)", text, re.M)
            directives[directive] = match.group(1) if match else None
        selected, note, state = go_select(local, setting, directives["go"], directives["toolchain"])
        if state != "ok":
            result.add(state, f"{where} {note}")
            continue
        if note:
            result.add("ok", f"{where} {note}")
        if selected != local:
            _, mode = go_setting(setting)
            if mode == "path" and shutil.which(selected) is None:
                result.add("missing-toolchain", f"{where} selects {selected}, not on PATH (GOTOOLCHAIN={setting})")
                continue
            result.add("ok", f"{where} selects {selected} (GOTOOLCHAIN={setting}"
                       f"{', downloaded on first use' if mode != 'path' else ''})")
        if selected not in releases:
            releases.append(selected)
        passed += 1
    result.releases = releases or [local]
    if passed:
        result.add("ok", f"{passed} go.mod(s) supported (local {local}, GOTOOLCHAIN={setting})")


# Rust ---------------------------------------------------------------------

def rust_pin_matches(pin, rustc):
    """Whether the selected rustc is the toolchain a channel pin names.

    With rustup, its active toolchain's name. Without, the version and channel,
    and a dated channel's date against the commit date: a nightly is built from
    the previous day's last commit, so either day matches."""
    if rustc["active"]:
        return rustc["active"] == pin or rustc["active"].startswith(pin + "-")
    named = re.fullmatch(r"(stable|beta|nightly)(?:-(\d{4})-(\d{2})-(\d{2}))?", pin)
    if named:
        if named.group(1) != rustc["channel"]:
            return False
        if not named.group(2):
            return True
        if not rustc["date"]:
            return False
        day = datetime.date(*(int(part) for part in named.group(2, 3, 4)))
        return rustc["date"] in (day.isoformat(), (day - datetime.timedelta(days=1)).isoformat())
    if re.fullmatch(r"\d+\.\d+(?:\.\d+)?", pin):
        length = len(pin.split("."))
        return rustc["channel"] == "stable" and numbers(pin) == rustc["version"][:length]
    return False


def rust_toolchain_file(project):
    """(file name, [toolchain] table) of the project's toolchain file, or None.

    rustup prefers rust-toolchain.toml. The legacy rust-toolchain holds either
    the same TOML or a single line naming the channel."""
    for name in ("rust-toolchain.toml", "rust-toolchain"):
        path = os.path.join(project, name)
        if not os.path.isfile(path):
            continue
        text = read(path)
        if name == "rust-toolchain" and "[toolchain]" not in text:
            lines = [line.strip() for line in text.splitlines() if line.strip()]
            if len(lines) != 1 or len(lines[0].split()) != 1:
                raise ValueError(f"rust-toolchain is neither TOML nor a single channel name: {text.strip()!r}")
            return name, {"channel": lines[0]}
        return name, typed(tomllib.loads(text).get("toolchain", {}), dict, f"{name} [toolchain]")
    return None


def check_rust(project, tools, result):
    rustc = tools.rustc(project)
    if rustc is None:
        result.add("missing-toolchain", "no rustc (rustc -vV)")
        return
    installed = rustc["version"]
    # The full release: a beta or nightly, such as 1.84.0-nightly, matches no
    # context release a stable one would.
    result.releases = [rustc["release"]]
    identity = rustc["active"] or f"rustc {shown(installed)} {rustc['channel']}" + (
        f" ({rustc['date']})" if rustc["date"] else "")
    found = rust_toolchain_file(project)
    if found:
        name, table = found
        pin = table.get("channel")
        if pin is not None:
            typed(pin, str, f"{name} channel")
            if not rust_pin_matches(pin, rustc):
                how = "rustup is not installed" if not tools.rustup() else "rustup did not select it"
                result.add("pin-not-honoured", f"{name} channel {pin}, {how}; {identity} is used")
            else:
                result.add("ok", f"{name} channel {pin} is {identity}")
        for target in typed(table.get("targets", []), list, f"{name} targets"):
            if rustc["sysroot"] and not os.path.isdir(os.path.join(rustc["sysroot"], "lib", "rustlib", target)):
                result.add("missing-toolchain", f"{name} target {target} has no standard library installed")
    passed = 0
    for path in manifests(project, "Cargo.toml"):
        where = relative(path, project)
        manifest = tomllib.loads(read(path))
        failed = False
        workspace = typed(manifest.get("workspace", {}), dict, f"{where} [workspace]")
        for name, table in (("package", manifest.get("package", {})), ("workspace.package", workspace.get("package", {}))):
            typed(table, dict, f"{where} [{name}]")
            minimum = table.get("rust-version")
            # A dict is `rust-version.workspace = true`, read at the workspace root.
            if not isinstance(minimum, (str, dict, type(None))):
                typed(minimum, str, f"{where} {name}.rust-version")
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

PEP440 = re.compile(r"(\d+(?:\.\d+)*)(?:[-_.]?(a|alpha|b|beta|c|rc|pre|preview)[-_.]?(\d*))?"
                    r"(?:[-_.]?(post|rev|r)[-_.]?(\d*))?(?:[-_.]?(dev)[-_.]?(\d*))?", re.I)
PRE_PHASE = {"a": 0, "alpha": 0, "b": 1, "beta": 1, "c": 2, "rc": 2, "pre": 2, "preview": 2}


def pep440_key(text, size):
    """A comparable PEP 440 version: dev < a < b < rc < final < post."""
    match = PEP440.fullmatch(text)
    if not match:
        raise ValueError(f"unsupported requires-python version {text!r}")
    release, pre, pre_number, post, post_number, dev, dev_number = match.groups()
    if pre:
        phase = (PRE_PHASE[pre.lower()], int(pre_number or 0))
    elif dev and not post:
        phase = (-1, 0)
    else:
        phase = (3, 0)
    return (padded(numbers(release), size), phase, int(post_number or 0) if post else -1,
            int(dev_number or 0) if dev else float("inf"))


def pep440_satisfies(installed, specifier):
    """Whether an interpreter version satisfies a requires-python specifier,
    in PEP 440 order: 3.14.0rc1 is older than 3.14.0, so `>=3.14.0` excludes
    it. A prerelease interpreter is eligible for every clause.

    Clauses may name pre-, post- and dev-releases. Epochs and local versions
    are not supported. A trailing + on the interpreter, a build past the
    version it names, compares as that version."""
    installed = installed.removesuffix("+")
    version = numbers(installed)
    for clause in filter(None, (part.strip() for part in specifier.split(","))):
        match = re.fullmatch(r"(~=|===|==|!=|>=|<=|>|<)\s*v?([0-9][0-9A-Za-z._-]*?)(\.\*)?", clause)
        if not match:
            raise ValueError(f"unsupported requires-python clause {clause!r}")
        operator, text, wildcard = match.groups()
        if wildcard and not re.fullmatch(r"\d+(?:\.\d+)*", text):
            raise ValueError(f"unsupported requires-python clause {clause!r}")
        want = numbers(text)
        size = max(len(want), len(version))
        have, full = pep440_key(installed, size), pep440_key(text, size)
        if operator in ("==", "!=", "===") and wildcard:
            equal = tuple(version[:len(want)]) == want
            ok = equal if operator == "==" else not equal
        elif operator in ("==", "==="):
            ok = have == full
        elif operator == "!=":
            ok = have != full
        elif operator == "~=":
            if len(want) < 2:
                raise ValueError(f"~= needs two release components: {clause!r}")
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
    result.releases = list(interpreters.values())
    declared = {}
    for path in manifests(project, "pyproject.toml"):
        where = relative(path, project)
        table = typed(tomllib.loads(read(path)).get("project", {}), dict, f"{where} [project]")
        specifier = table.get("requires-python")
        if specifier is not None:
            typed(specifier, str, f"{where} requires-python")
            declared.setdefault(specifier, []).append(relative(path, project))
    for specifier, paths in declared.items():
        where = paths[0] if len(paths) == 1 else f"{len(paths)} pyproject.toml"
        fitting = [name for name, version in interpreters.items() if pep440_satisfies(version, specifier)]
        if fitting:
            result.add("ok", f"requires-python {specifier!r} ({where}): {', '.join(fitting)}")
        else:
            result.add("unsupported", f"requires-python {specifier!r} ({', '.join(paths)}) matches none of "
                       f"{', '.join(f'{name} ({version})' for name, version in interpreters.items())}")
    if not declared:
        result.add("ok", f"no requires-python; {', '.join(interpreters)} available")


# TypeScript ---------------------------------------------------------------

def npm_comparators(text):
    """(bounds, prereleases) for one space-separated npm comparator set: the
    (operator, version) bounds, and the major.minor.patch of each comparator
    written with a prerelease tag."""
    text = re.sub(r"(>=|<=|>|<|=|\^|~)\s+", r"\1", text.strip())
    hyphen = re.fullmatch(r"(\S+)\s+-\s+(\S+)", text)
    if hyphen:
        low, high = npm_comparators(">=" + hyphen.group(1)), npm_comparators("<=" + hyphen.group(2))
        return low[0] + high[0], low[1] | high[1]
    bounds = []
    prereleases = set()
    for item in text.split():
        match = re.fullmatch(r"(>=|<=|>|<|=|\^|~)?v?(?:(\d+|[xX*])(?:\.(\d+|[xX*]))?(?:\.(\d+|[xX*]))?"
                             r"(-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?)", item)
        if not match:
            raise ValueError(f"cannot compare with the npm range {item!r}")
        operator, *parts, pre = match.groups()
        parts = [None if part is None or part in "xX*" else int(part) for part in parts]
        given = len(parts) if None not in parts else parts.index(None)
        if given == 0:
            continue
        major, minor, patch = (part or 0 for part in parts)
        if pre:
            prereleases.add((major, minor, patch))
        low = f"{major}.{minor}.{patch}{pre or ''}"
        if given == 1:
            upper = f"{major + 1}.0.0-0"
        elif given == 2:
            upper = f"{major}.{minor + 1}.0-0"
        else:
            upper = None
        if operator == "^":
            upper = (f"{major + 1}.0.0-0" if major or given == 1 else
                     f"0.{minor + 1}.0-0" if minor or given == 2 else f"0.0.{patch + 1}-0")
            bounds += [(">=", low), ("<", upper)]
        elif operator == "~":
            bounds += [(">=", low), ("<", f"{major}.{minor + 1}.0-0" if given > 1 else f"{major + 1}.0.0-0")]
        elif operator in (None, "="):
            bounds += [(">=", low), ("<", upper)] if upper else [("==", low)]
        elif operator == ">=":
            bounds.append((">=", low))
        elif operator == "<":
            bounds.append(("<", f"{low}-0" if given < 3 and not pre else low))
        elif operator == ">":
            bounds.append((">=", upper) if upper else (">", low))
        elif operator == "<=":
            bounds.append(("<", upper) if upper else ("<=", low))
    return bounds, prereleases


def npm_satisfies(version, declared):
    """Whether an installed version satisfies an npm version or range.

    As npm does, a prerelease is eligible for a comparator set only when a
    comparator in that set names a prerelease of the same major.minor.patch;
    ordering is checked separately."""
    release = dart_version(version)
    for part in declared.split("||"):
        bounds, prereleases = npm_comparators(part)
        if release[3] == 0 and tuple(release[:3]) not in prereleases:
            continue
        if satisfies(version, bounds, dart_version):
            return True
    return False


def check_typescript(project, tools, result):
    node = tools.node()
    if node is None:
        result.add("missing-toolchain", "no Node.js (node --version)")
    elif dart_version(node) < dart_version(NODE_MINIMUM):
        result.add("unsupported", f"Node.js {node} is older than {NODE_MINIMUM}")
    else:
        result.add("ok", f"Node.js {node}")
    pin = None
    package = os.path.join(project, "package.json")
    if os.path.exists(package):
        manifest = typed(json.loads(read(package)), dict, "package.json")
        for field in ("devDependencies", "dependencies"):
            declared = typed(manifest.get(field, {}), dict, f"package.json {field}").get("typescript")
            if declared is not None and pin is None:
                pin = typed(declared, str, f"package.json {field}.typescript")
    installed = os.path.join(project, "node_modules", "typescript", "package.json")
    if os.path.exists(installed):
        found = typed(json.loads(read(installed)), dict, "node_modules/typescript/package.json")
        if "version" not in found:
            raise ValueError("node_modules/typescript/package.json has no version")
        version = typed(found["version"], str, "node_modules/typescript/package.json version")
        dart_version(version)
        result.releases = [version]
        if pin is None:
            result.add("ok", f"typescript {version} installed, none declared")
            return
        try:
            fits = npm_satisfies(version, pin)
        except ValueError as error:
            result.add("ok", f"typescript {version} installed; package.json declares {pin!r}: {error}")
            return
        exact = re.fullmatch(r"=?v?\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?", pin.strip())
        if fits:
            result.add("ok", f"typescript {version} installed, satisfies package.json {pin!r}")
        elif exact:
            result.add("pin-not-honoured", f"package.json pins typescript {pin}, node_modules has {version}, "
                       "which is used (npm ci)")
        else:
            result.add("unsupported", f"typescript {version} in node_modules is outside package.json {pin!r} (npm ci)")
    elif pin:
        result.add("missing-toolchain", f"typescript {pin} is declared and not installed in node_modules (npm ci)")
    else:
        result.add("missing-toolchain", "no typescript declared or installed")


class Flow:
    """A YAML flow collection or scalar: mappings and sequences become dicts
    and lists, and every scalar a string. Anchors, aliases, tags and anything
    else it cannot read raise ValueError."""

    def __init__(self, text):
        self.text = text
        self.position = 0

    def fail(self, what):
        raise ValueError(f"{what} at {self.text[self.position:self.position + 20]!r}")

    def skip(self):
        while self.position < len(self.text) and self.text[self.position].isspace():
            self.position += 1

    def peek(self):
        self.skip()
        return self.text[self.position] if self.position < len(self.text) else ""

    def whole(self):
        value = self.value()
        if self.peek():
            self.fail("unexpected text after a value")
        return value

    def value(self):
        first = self.peek()
        if first == "{":
            return self.collection("}", {})
        if first == "[":
            return self.collection("]", [])
        if first in ("'", '"'):
            return self.quoted()
        if not first or first in ",:]}#*&!|>%@`":
            self.fail("an unsupported YAML value")
        start = self.position
        while self.position < len(self.text):
            here = self.text[self.position]
            after = self.text[self.position + 1:self.position + 2]
            if here in ",[]{}" or (here == ":" and (not after or after.isspace() or after in ",[]{}")):
                break
            if here == "#" and self.text[self.position - 1].isspace():
                self.fail("a comment inside a flow collection")
            self.position += 1
        return self.text[start:self.position].strip()

    def quoted(self):
        quote = self.text[self.position]
        self.position += 1
        found = []
        while self.position < len(self.text):
            here = self.text[self.position]
            if quote == "'" and here == "'":
                if self.text[self.position + 1:self.position + 2] == "'":
                    found.append("'")
                    self.position += 2
                    continue
                self.position += 1
                return "".join(found)
            if quote == '"' and here == "\\":
                escaped = self.text[self.position + 1:self.position + 2]
                if escaped not in ('"', "\\", "/"):
                    self.fail("an unsupported escape")
                found.append(escaped)
                self.position += 2
                continue
            if quote == '"' and here == '"':
                self.position += 1
                return "".join(found)
            found.append(here)
            self.position += 1
        self.fail("an unterminated string")

    def collection(self, end, into):
        self.position += 1
        while True:
            if self.peek() == end:
                self.position += 1
                return into
            if isinstance(into, dict):
                key = self.value()
                if not isinstance(key, str):
                    self.fail("a key that is not a scalar")
                if key in into:
                    self.fail(f"a repeated key {key!r}")
                if self.peek() != ":":
                    self.fail("a key without a value")
                self.position += 1
                into[key] = None if self.peek() in (",", end) else self.value()
            else:
                into.append(self.value())
            separator = self.peek()
            if separator == ",":
                self.position += 1
            elif separator != end:
                self.fail("a missing `,`")


def uncommented(line):
    """The line without a trailing comment, outside quoted scalars."""
    quote = None
    index = 0
    while index < len(line):
        here = line[index]
        before = line[index - 1] if index else " "
        if quote == "'" and here == "'":
            if line[index + 1:index + 2] == "'":
                index += 1
            else:
                quote = None
        elif quote == '"' and here == "\\":
            index += 1
        elif quote == '"' and here == '"':
            quote = None
        elif quote is None and here in "'\"" and (before.isspace() or before in "[{,:"):
            quote = here
        elif quote is None and here == "#" and before.isspace():
            return line[:index].rstrip()
        index += 1
    return line.rstrip()


def indent(line):
    return len(line) - len(line.lstrip(" "))


KEY = re.compile(r"""(?:([^\s'"#\[\]{},:][^:#]*?)|'([^']*)'|"([^"\\]*)")[ \t]*:(?:[ \t]+(.*)|)$""")


def block_mapping(lines, what):
    """{key: (inline value, [deeper lines])} for block mapping lines, which
    must all be keys at the first line's indent or lines deeper than it."""
    found = {}
    if not lines:
        return found
    column = indent(lines[0])
    key = None
    for line in lines:
        if indent(line) > column and key is not None:
            found[key][1].append(line)
            continue
        match = indent(line) == column and KEY.match(line.strip())
        if not match:
            raise ValueError(f"{what}: cannot read {line.strip()!r}")
        key = next(part for part in match.groups()[:3] if part is not None)
        if key in found:
            raise ValueError(f"{what}: repeated key {key!r}")
        found[key] = ((match.group(4) or "").strip(), [])
    return found


def block_scalar(value, deeper, what):
    """A block mapping value: a string, a dict or list, or None when empty."""
    if deeper:
        if value:
            raise ValueError(f"{what}: a value continued on another line")
        return []  # A nested block collection.
    if not value:
        return None
    if value[0] in "[{'\"":
        return Flow(value).whole()
    if value[0] in "*&!|>%@`":
        raise ValueError(f"{what}: unsupported YAML value {value!r}")
    return value


def context_entry(entry, number):
    """(id, scalar parameters) of one context, from a dict or block mapping."""
    what = f"context {number}"
    if isinstance(entry, dict):
        identifier, parameters = entry.get("id"), entry.get("parameters")
    else:
        fields = block_mapping(entry, what)
        identifier = block_scalar(*fields["id"], f"{what} id") if "id" in fields else None
        parameters = None
        if "parameters" in fields:
            value, deeper = fields["parameters"]
            if value and deeper:
                raise ValueError(f"{what} parameters: a value continued on another line")
            if value:
                parameters = block_scalar(value, [], f"{what} parameters")
            else:
                parameters = {}
                for key, (inner, nested) in block_mapping(deeper, f"{what} parameters").items():
                    parameters[key] = block_scalar(inner, nested, f"{what} parameter {key}")
    if not isinstance(identifier, str) or not identifier:
        raise ValueError(f"{what} has no id")
    if not isinstance(parameters, dict):
        raise ValueError(f"context {identifier} has no parameters mapping")
    return identifier, {key: value for key, value in parameters.items() if isinstance(value, str)}


def contexts(text):
    """(id, parameters) for each entry of a dataset's top-level `contexts`.

    Reads block and flow sequences of block or flow mappings without a YAML
    parser, and keeps only scalar parameters, as strings. Raises ValueError
    when there is no context, or any part of the list it cannot read: a
    context must never be skipped."""
    lines = text.splitlines()
    starts = [index for index, line in enumerate(lines)
              if re.match(r"""(contexts|'contexts'|"contexts")[ \t]*:(?:[ \t]|$)""", line)]
    if not starts:
        raise ValueError("no top-level `contexts`")
    if len(starts) > 1:
        raise ValueError("more than one top-level `contexts`")
    head = uncommented(lines[starts[0]]).split(":", 1)[1].strip()
    block = []
    for line in lines[starts[0] + 1:]:
        if line.strip() and not line.lstrip().startswith("#"):
            if not line[0].isspace() and not re.match(r"-(?:[ \t]|$)", line):
                break
            if "\t" in line[:indent(line) + 1]:
                raise ValueError("a tab in the indentation of `contexts`")
            block.append(uncommented(line))
    if head:
        if head[0] not in "[":
            raise ValueError(f"`contexts` is not a list: {head!r}")
        # A flow sequence may continue on the lines after the key.
        entries = Flow(" ".join([head] + [line.strip() for line in block])).whole()
        if not isinstance(entries, list):
            raise ValueError("`contexts` is not a list")
        found = []
        for number, entry in enumerate(entries, 1):
            if not isinstance(entry, dict):
                raise ValueError(f"context {number} is not a mapping")
            found.append(context_entry(entry, number))
    else:
        found = []
        column = indent(block[0]) if block else 0
        entries = []
        for line in block:
            if indent(line) == column and re.match(r"-(?:[ \t]|$)", line.lstrip()):
                entries.append([line])
            elif indent(line) > column and entries:
                entries[-1].append(line)
            else:
                raise ValueError(f"cannot read {line.strip()!r} in `contexts`")
        for number, entry in enumerate(entries, 1):
            first = entry[0][column + 1:]
            content = first.lstrip(" ")
            if content.startswith("{"):
                body = Flow(" ".join([content] + [line.strip() for line in entry[1:]])).whole()
                found.append(context_entry(body, number))
                continue
            if content[:1] in ("&", "*", "!", "|", ">", "%", "@", "`", "?"):
                raise ValueError(f"context {number}: unsupported YAML value {content!r}")
            if content.startswith("- ") or content == "-":
                raise ValueError(f"context {number} is not a mapping")
            # Replace the dash with a space, so the entry's keys share a column.
            mapping = ([" " * (column + 1) + first] if content else []) + entry[1:]
            found.append(context_entry(mapping, number))
    if not found:
        raise ValueError("`contexts` lists no context")
    return found


def extends(release, wanted):
    """Whether release is wanted, or wanted extended with dotted numeric
    components: 3.14.2 extends 3.14, and 3.14.0rc1 and 5.9.0-beta extend nothing."""
    rest = release[len(wanted):] if release.startswith(wanted) else None
    return release == wanted or (bool(rest) and re.fullmatch(r"(?:\.\d+)+", rest) is not None)


def check_releases(language, project, result):
    """One (state, context, note) for each context of the project's datasets."""
    key = RELEASE_PARAMETER[language]
    lines = []
    for path in manifests(project, "semantic-oracle.yml"):
        where = relative(path, project)
        prefix = os.path.dirname(where)
        try:
            found = contexts(read(path))
        except (ValueError, OSError, UnicodeDecodeError) as error:
            lines.append(("unresolved", where, f"cannot read its contexts: {error}"))
            continue
        for context, parameters in found:
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
        except (ValueError, OSError, UnicodeDecodeError, tomllib.TOMLDecodeError, json.JSONDecodeError) as error:
            result.add("unsupported", f"cannot read a declaration: {error}")
        counts.setdefault(language, dict.fromkeys(STATES, 0))[result.state] += 1
        print(f"{result.state:<17} {language}/{name}: {result.line()}")
        try:
            releases = check_releases(language, project, result)
        except (ValueError, OSError) as error:
            releases = [("unresolved", "semantic-oracle.yml", f"cannot read its contexts: {error}")]
        for state, context, note in releases:
            tally = context_counts.setdefault(language, {"unresolved": 0, "release-mismatch": 0, "ok": 0})
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
    unresolved = sum(tally["unresolved"] for tally in context_counts.values())
    print(f"{len(found)} projects, {failing} cannot be checked with the installed toolchains; "
          f"{sum(tally['release-mismatch'] + tally['ok'] for tally in context_counts.values())} dataset contexts, "
          f"{mismatched} release-mismatch"
          + (f"; {unresolved} semantic-oracle.yml unresolved" if unresolved else ""))
    return 1 if failing or mismatched or unresolved else 0


if __name__ == "__main__":
    sys.exit(main())
