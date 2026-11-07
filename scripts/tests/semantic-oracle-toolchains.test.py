#!/usr/bin/env python3
# ---
# relationships:
#   references: semantic-oracle-model
# ---
"""Runs scripts/semantic-oracle/toolchains.py on a small corpus in a temporary
directory, with stand-in toolchain commands on PATH, and checks each project's
state and the exit status. Needs only python3."""
import importlib.util
import os
import re
import stat
import subprocess
import sys
import tempfile

sys.dont_write_bytecode = True

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
SCRIPT = os.path.join(ROOT, "scripts", "semantic-oracle", "toolchains.py")

spec = importlib.util.spec_from_file_location("toolchains", SCRIPT)
toolchains = importlib.util.module_from_spec(spec)
spec.loader.exec_module(toolchains)


def write(root, files):
    for name, text in files.items():
        path = os.path.join(root, name)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w") as out:
            out.write(text)


def command(directory, name, script):
    path = os.path.join(directory, name)
    with open(path, "w") as out:
        out.write("#!/bin/sh\n" + script)
    os.chmod(path, os.stat(path).st_mode | stat.S_IXUSR)


def tools(directory, sysroot, omit=()):
    os.makedirs(directory, exist_ok=True)
    stand_ins = {
        "dotnet": 'echo "8.0.204 [/sdk]"; echo "9.0.100 [/sdk]"\n',
        "dart": 'echo "Dart SDK version: 3.5.0 (stable) (Tue Jul 30 2024) on \\"linux_x64\\""\n',
        "go": 'echo "go version go1.23.1 linux/amd64"\n',
        "rustc": f'if [ "$1" = --print ]; then echo {sysroot}; '
                 'else echo "rustc 1.82.0 (f6e511eec 2024-10-15) (Homebrew)"; fi\n',
        "python3.11": 'echo "Python 3.11.9"\n',
        "python3.13": 'echo "Python 3.13.1"\n',
        "node": 'echo v22.11.0\n',
    }
    for name, script in stand_ins.items():
        if name not in omit:
            command(directory, name, script)


CORPUS = {
    # C#
    "csharp/projects/rolls/global.json": '{"sdk": {"version": "8.0.100", "rollForward": "latestFeature"}}',
    "csharp/projects/rolls/a/A.csproj": "<Project><PropertyGroup><TargetFrameworks>net8.0;netstandard2.0"
                                        "</TargetFrameworks></PropertyGroup></Project>",
    "csharp/projects/rolls/vendor/global.json": '{"sdk": {"version": "7.0.100", "rollForward": "disable"}}',
    "csharp/projects/exact/global.json": '{"sdk": {"version": "8.0.100", "rollForward": "disable"}}',
    "csharp/projects/exact/A.csproj": "<Project><PropertyGroup><TargetFramework>net8.0</TargetFramework>"
                                      "</PropertyGroup></Project>",
    "csharp/projects/future/A.csproj": "<Project><PropertyGroup><TargetFramework>net10.0</TargetFramework>"
                                       "</PropertyGroup></Project>",
    # Dart
    "dart/projects/legacy-upper/pubspec.yaml": "name: a\nenvironment:\n  sdk: '>=2.12.0 <3.0.0'\n"
                                               "dependencies:\n  flutter:\n    sdk: flutter\n",
    "dart/projects/legacy-upper/vendor/old/pubspec.yaml": "name: b\nenvironment:\n  sdk: '>=2.9.0 <3.0.0'\n",
    "dart/projects/pre-null-safety/pubspec.yaml": "name: a\nenvironment:\n  sdk: \">=2.9.0-18.0 <3.0.0\"\n",
    "dart/projects/too-new/pubspec.yaml": "name: a\nenvironment:\n  sdk: ^3.6.0 # newer\n",
    "dart/projects/caret/pubspec.yaml": "name: a\n\nenvironment:\n  sdk: ^3.4.0\n",
    # Go
    "go/projects/toolchain/go.mod": "module a\n\ngo 1.22\n\ntoolchain go1.22.5\n",
    "go/projects/too-new/go.mod": "module a\n\ngo 1.21\n",
    "go/projects/too-new/inner/go.mod": "module b\n\ngo 1.24rc1\n",
    # Rust
    "rust/projects/pinned/rust-toolchain.toml": '[toolchain]\nchannel = "1.80.0"\n',
    "rust/projects/pinned/Cargo.toml": '[package]\nname = "a"\nedition = "2021"\nrust-version = "1.70"\n',
    "rust/projects/stable/rust-toolchain.toml": '[toolchain]\nchannel = "stable"\n'
                                                'targets = ["x86_64-unknown-linux-gnu"]\n',
    "rust/projects/stable/Cargo.toml": '[workspace]\nmembers = ["b"]\n[workspace.package]\nedition = "2021"\n',
    "rust/projects/stable/b/Cargo.toml": '[package]\nname = "b"\nedition.workspace = true\n',
    "rust/projects/stable/vendor/c/Cargo.toml": '[package]\nname = "c"\nrust-version = "1.99"\n',
    "rust/projects/edition/Cargo.toml": '[package]\nname = "a"\nedition = "2024"\n',
    "rust/projects/msrv/Cargo.toml": '[package]\nname = "a"\nedition = "2021"\nrust-version = "1.90"\n',
    "rust/projects/target/rust-toolchain.toml": '[toolchain]\ntargets = ["wasm32-unknown-unknown"]\n',
    "rust/projects/target/Cargo.toml": '[package]\nname = "a"\nedition = "2021"\n',
    # Python
    "python/projects/fits/pyproject.toml": '[project]\nname = "a"\nrequires-python = ">=3.12"\n',
    "python/projects/fits/sub/pyproject.toml": '[project]\nname = "b"\nrequires-python = "~=3.11.0"\n',
    "python/projects/too-new/pyproject.toml": '[project]\nname = "a"\nrequires-python = ">=3.14, <4"\n',
    "python/projects/undeclared/setup.py": "",
    # TypeScript
    "typescript/projects/installed/package.json": '{"devDependencies": {"typescript": "5.9.3"}}',
    "typescript/projects/installed/node_modules/typescript/package.json": '{"version": "5.9.3"}',
    "typescript/projects/not-installed/package.json": '{"devDependencies": {"typescript": "5.4.5"}}',
}

EXPECTED = {
    "csharp/rolls": ("ok", "global.json selects SDK 8.0.204"),
    "csharp/exact": ("unsupported", "selects SDK 8.0.100 (rollForward disable)"),
    "csharp/future": ("unsupported", "net10.0 needs SDK 10, have 9.0.100"),
    "dart/caret": ("ok", "Dart 3.5.0 satisfies 1 pubspec"),
    "dart/legacy-upper": ("ok", "Dart 3.5.0 satisfies 1 pubspec"),
    "dart/pre-null-safety": ("unsupported", "excludes Dart 3.5.0"),
    "dart/too-new": ("unsupported", "sdk '^3.6.0' excludes Dart 3.5.0"),
    "go/toolchain": ("ok", "go1.23.1 satisfies 1 go.mod"),
    "go/too-new": ("unsupported", "inner/go.mod go 1.24rc1 is newer than go1.23.1"),
    "python/fits": ("ok", "requires-python '~=3.11.0' (sub/pyproject.toml): python3.11"),
    "python/too-new": ("unsupported", "matches none of python3.11 (3.11.9), python3.13 (3.13.1)"),
    "python/undeclared": ("ok", "no requires-python"),
    "rust/edition": ("unsupported", "edition 2024 needs a newer rustc than 1.82.0"),
    "rust/msrv": ("unsupported", "rust-version 1.90 is newer than rustc 1.82.0"),
    "rust/pinned": ("pin-not-honoured", "channel 1.80.0, rustup is not installed; rustc 1.82.0 stable is used"),
    "rust/stable": ("ok", "rustc 1.82.0 satisfies 2 Cargo.toml"),
    "rust/target": ("pin-not-honoured", "target wasm32-unknown-unknown has no standard library"),
    "typescript/installed": ("ok", "typescript 5.9.3 installed"),
    "typescript/not-installed": ("missing-toolchain", "typescript 5.4.5 is declared and not installed"),
}


def report(corpus, path):
    done = subprocess.run([sys.executable, SCRIPT, "--fixtures", corpus], capture_output=True, text=True,
                          env={"PATH": path})
    states = {}
    for line in done.stdout.splitlines():
        match = re.match(r"(\S+)\s+(\S+?): (.*)", line)
        if match and match.group(1) in toolchains.STATES:
            states[match.group(2)] = (match.group(1), match.group(3))
    return done, states


def test_corpus(scratch):
    corpus = os.path.join(scratch, "fixtures")
    sysroot = os.path.join(scratch, "sysroot")
    os.makedirs(os.path.join(sysroot, "lib", "rustlib", "x86_64-unknown-linux-gnu"))
    write(corpus, CORPUS)
    tools(os.path.join(scratch, "bin"), sysroot)
    done, states = report(corpus, os.path.join(scratch, "bin"))
    assert done.returncode == 1, done.stdout + done.stderr
    assert set(states) == set(EXPECTED), sorted(set(states) ^ set(EXPECTED))
    for project, (state, note) in EXPECTED.items():
        assert states[project][0] == state, f"{project}: {states[project]}, not {state}"
        assert note in states[project][1], f"{project}: {states[project][1]!r} lacks {note!r}"
    assert "rust       unsupported 2 pin-not-honoured 2 ok 1" in done.stdout, done.stdout
    assert "19 projects, 9 cannot be checked" in done.stdout, done.stdout


def test_missing(scratch):
    corpus = os.path.join(scratch, "fixtures")
    write(corpus, {key: value for key, value in CORPUS.items() if key.startswith(("go/", "typescript/projects/installed/"))})
    tools(os.path.join(scratch, "bin"), scratch, omit=("go", "node"))
    done, states = report(corpus, os.path.join(scratch, "bin"))
    assert done.returncode == 1
    assert states["go/toolchain"] == ("missing-toolchain", "missing-toolchain: no Go (go version)"), states
    assert states["typescript/installed"][0] == "missing-toolchain", states


def test_passing(scratch):
    corpus = os.path.join(scratch, "fixtures")
    write(corpus, {key: value for key, value in CORPUS.items() if "/pinned/" in key or "/caret/" in key})
    tools(os.path.join(scratch, "bin"), scratch)
    done, states = report(corpus, os.path.join(scratch, "bin"))
    assert done.returncode == 0, done.stdout
    assert states["rust/pinned"][0] == "pin-not-honoured"


def test_empty(scratch):
    done = subprocess.run([sys.executable, SCRIPT, "--fixtures", scratch], capture_output=True, text=True)
    assert done.returncode == 2 and "fixtures:init" in done.stdout, done.stdout


def test_versions(_):
    roll = toolchains.roll_forward
    installed = [(8, 0, 204), (8, 0, 110), (9, 0, 100)]
    assert roll((8, 0, 100), None, installed) == (8, 0, 110)
    assert roll((8, 0, 100), "patch", installed) == (8, 0, 110)
    assert roll((8, 0, 100), "latestFeature", installed) == (8, 0, 204)
    assert roll((8, 0, 300), "feature", installed) is None
    assert roll((8, 0, 300), "latestMajor", installed) == (9, 0, 100)
    assert roll((8, 0, 110), "disable", installed) == (8, 0, 110)
    assert toolchains.framework_major("net8.0-windows") == 8
    assert toolchains.framework_major("netcoreapp3.1") == 3
    assert toolchains.framework_major("net472") == 0
    satisfied = lambda version, constraint: toolchains.satisfies(
        version, toolchains.dart_constraint(constraint), toolchains.dart_version)
    assert satisfied("3.5.0", ">=2.12.0-0 <3.0.0")
    assert not satisfied("3.5.0", ">=2.9.0-18.0 <3.0.0")
    assert satisfied("3.5.0", "^3.0.0") and not satisfied("4.0.0", "^3.0.0")
    assert not satisfied("3.5.0-1.0.dev", ">=3.5.0")
    assert satisfied("0.3.1", "^0.3.0") and not satisfied("0.4.0", "^0.3.0")
    assert toolchains.go_version("1.21") < toolchains.go_version("go1.21rc2") < toolchains.go_version("1.21.0")
    pep = toolchains.pep440_satisfies
    assert pep((3, 11, 9), ">=3.10, <3.12") and not pep((3, 12, 0), ">=3.10, <3.12")
    assert pep((3, 11, 2), "==3.11.*") and not pep((3, 12, 0), "==3.11.*")
    assert pep((3, 11, 2), "~=3.11") and pep((3, 13, 0), "~=3.11") and not pep((3, 12, 0), "~=3.11.1")
    assert pep((3, 13, 0), "!=3.12.*")


def main():
    failed = 0
    for test in (test_versions, test_corpus, test_missing, test_passing, test_empty):
        with tempfile.TemporaryDirectory() as scratch:
            try:
                test(scratch)
                print(f"ok   {test.__name__}")
            except (AssertionError, KeyError) as error:
                failed += 1
                print(f"FAIL {test.__name__}: {error}")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
