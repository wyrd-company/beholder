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
        "dotnet": 'echo "8.0.204 [/sdk]"; echo "9.0.100 [/sdk]"; echo "9.0.200-preview.1.24101.2 [/sdk]"\n',
        "dart": 'echo "Dart SDK version: 3.5.0 (stable) (Tue Jul 30 2024) on \\"linux_x64\\""\n',
        "go": 'if [ "$1" = env ]; then echo "${GOTOOLCHAIN:-auto}"; '
              'else echo "go version go1.23.1 linux/amd64"; fi\n',
        "rustc": f'if [ "$1" = --print ]; then echo {sysroot}; '
                 'else printf "rustc 1.82.0 (f6e511eec 2024-10-15) (Homebrew)\\nbinary: rustc\\n'
                 'commit-date: 2024-10-15\\nrelease: 1.82.0\\n"; fi\n',
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
    "csharp/projects/null-sdk/global.json": '{"sdk": null}',
    "csharp/projects/no-prerelease/global.json": '{"sdk": {"version": "8.0.100", "rollForward": "latestMajor", '
                                                 '"allowPrerelease": false}}',
    "csharp/projects/stable-only/global.json": '{"sdk": {"allowPrerelease": false}}',
    "csharp/projects/future/A.csproj": "<Project><PropertyGroup><TargetFramework>net10.0</TargetFramework>"
                                       "</PropertyGroup></Project>",
    # Dart
    "dart/projects/legacy-upper/pubspec.yaml": "name: a\nenvironment:\n  sdk: '>=2.12.0 <3.0.0'\n"
                                               "dependencies:\n  flutter:\n    sdk: flutter\n",
    "dart/projects/legacy-upper/vendor/old/pubspec.yaml": "name: b\nenvironment:\n  sdk: '>=2.9.0 <3.0.0'\n",
    "dart/projects/pre-null-safety/pubspec.yaml": "name: a\nenvironment:\n  sdk: \">=2.9.0-18.0 <3.0.0\"\n",
    "dart/projects/too-new/pubspec.yaml": "name: a\nenvironment:\n  sdk: ^3.6.0 # newer\n",
    "dart/projects/caret/pubspec.yaml": "name: a\n\nenvironment:\n  sdk: ^3.4.0\n",
    "dart/projects/garbled/pubspec.yaml": "name: a\nenvironment:\n  sdk: '>=3.0.0 <4.0.0 or so'\n",
    # Go
    "go/projects/toolchain/go.mod": "module a\n\ngo 1.22\n\ntoolchain go1.22.5\n",
    "go/projects/too-new/go.mod": "module a\n\ngo 1.21\n",
    "go/projects/too-new/inner/go.mod": "module b\n\ngo 1.24rc1\n",
    "go/projects/default/go.mod": "module a\n\ngo 1.99.0\n\ntoolchain default\n",
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
    "rust/projects/legacy/rust-toolchain": "1.80.0\n",
    "rust/projects/legacy-toml/rust-toolchain": '[toolchain]\nchannel = "1.82.0"\n',
    "rust/projects/both/rust-toolchain.toml": '[toolchain]\nchannel = "1.82"\n',
    "rust/projects/both/rust-toolchain": "1.80.0\n",
    "rust/projects/legacy-garbled/rust-toolchain": "1.80.0\nstable\n",
    "rust/projects/nightly/rust-toolchain.toml": '[toolchain]\nchannel = "nightly-2024-10-16"\n',
    "rust/projects/numeric-channel/rust-toolchain.toml": '[toolchain]\nchannel = 1.82\n',
    "go/projects/toolchain/semantic-oracle.yml": "profile: go/1\ncontexts:\n  - id: default\n    parameters:\n"
                                                 "      release: go1.23\n      tags: []\n  - id: old\n"
                                                 "    parameters:\n      release: \"go1.22\"\n"
                                                 "public-api:\n  members: []\n",
    "go/projects/too-new/semantic-oracle.yml": "contexts:\n  - id: default\n    parameters:\n"
                                               "      release: go1.24rc1\n",
    "rust/projects/stable/semantic-oracle.yml": "contexts:\n- id: build\n  parameters:\n    mode: build\n"
                                                "    cfg: []\n- id: pinned\n  parameters:\n    release: '1.82'\n",
    # Python
    "python/projects/fits/semantic-oracle.yml": "contexts:\n  - id: runtime # the import root\n    parameters:\n"
                                                "      python: '3.13'\n  - id: patch\n    parameters:\n"
                                                "      python: 3.11.8\n",
    "python/projects/fits/pyproject.toml": '[project]\nname = "a"\nrequires-python = ">=3.12"\n',
    "python/projects/fits/sub/pyproject.toml": '[project]\nname = "b"\nrequires-python = "~=3.11.0"\n',
    "python/projects/too-new/pyproject.toml": '[project]\nname = "a"\nrequires-python = ">=3.14, <4"\n',
    "python/projects/undeclared/setup.py": "",
    # TypeScript
    "typescript/projects/installed/package.json": '{"devDependencies": {"typescript": "5.9.3"}}',
    "typescript/projects/installed/node_modules/typescript/package.json": '{"version": "5.9.3"}',
    "typescript/projects/installed/semantic-oracle.yml": "contexts:\n  - id: default\n    parameters:\n"
                                                         "      compiler: \"5.9\"\n      options: {\"strict\": true}\n",
    "typescript/projects/not-installed/package.json": '{"devDependencies": {"typescript": "5.4.5"}}',
    "typescript/projects/other-pin/package.json": '{"devDependencies": {"typescript": "5.4.5"}}',
    "typescript/projects/other-pin/node_modules/typescript/package.json": '{"version": "5.9.3"}',
    "typescript/projects/out-of-range/package.json": '{"devDependencies": {"typescript": ">=5.0 <5.5"}}',
    "typescript/projects/out-of-range/node_modules/typescript/package.json": '{"version": "5.9.3"}',
    "typescript/projects/in-range/package.json": '{"dependencies": {"typescript": "^5.9.0 || ~4.9"}}',
    "typescript/projects/in-range/node_modules/typescript/package.json": '{"version": "5.9.3"}',
    "typescript/projects/no-version/package.json": '{"devDependencies": {"typescript": "5.9.3"}}',
    "typescript/projects/no-version/node_modules/typescript/package.json": '{"name": "typescript"}',
}

EXPECTED = {
    "csharp/rolls": ("ok", "global.json selects SDK 8.0.204"),
    "csharp/exact": ("unsupported", "selects SDK 8.0.100 (rollForward disable)"),
    "csharp/future": ("unsupported", "net10.0 needs SDK 10, have 9.0.200-preview.1.24101.2"),
    "csharp/null-sdk": ("unsupported", "cannot read a declaration: global.json sdk is not a table: None"),
    "csharp/no-prerelease": ("ok", "global.json selects SDK 9.0.100"),
    "csharp/stable-only": ("ok", "global.json selects SDK 9.0.100"),
    "go/default": ("unsupported", "go.mod go 1.99.0 is newer than go1.23.1, and go.mod sets toolchain default"),
    "rust/legacy": ("pin-not-honoured", "rust-toolchain channel 1.80.0, rustup is not installed"),
    "rust/legacy-toml": ("ok", "rust-toolchain channel 1.82.0 is rustc 1.82.0 stable (2024-10-15)"),
    "rust/both": ("ok", "rust-toolchain.toml channel 1.82 is rustc 1.82.0 stable"),
    "rust/legacy-garbled": ("unsupported", "rust-toolchain is neither TOML nor a single channel name"),
    "dart/garbled": ("unsupported", "cannot parse SDK constraint '>=3.0.0 <4.0.0 or so' at 'or so'"),
    "dart/caret": ("ok", "Dart 3.5.0 satisfies 1 pubspec"),
    "dart/legacy-upper": ("ok", "Dart 3.5.0 satisfies 1 pubspec"),
    "dart/pre-null-safety": ("unsupported", "excludes Dart 3.5.0"),
    "dart/too-new": ("unsupported", "sdk '^3.6.0' excludes Dart 3.5.0"),
    "go/toolchain": ("ok", "1 go.mod(s) supported (local go1.23.1, GOTOOLCHAIN=auto)"),
    "go/too-new": ("ok", "inner/go.mod selects go1.24rc1 (GOTOOLCHAIN=auto, downloaded on first use)"),
    "python/fits": ("ok", "requires-python '~=3.11.0' (sub/pyproject.toml): python3.11"),
    "python/too-new": ("unsupported", "matches none of python3.11 (3.11.9), python3.13 (3.13.1)"),
    "python/undeclared": ("ok", "no requires-python"),
    "rust/edition": ("unsupported", "edition 2024 needs a newer rustc than 1.82.0"),
    "rust/msrv": ("unsupported", "rust-version 1.90 is newer than rustc 1.82.0"),
    "rust/pinned": ("pin-not-honoured", "channel 1.80.0, rustup is not installed; rustc 1.82.0 stable (2024-10-15) is used"),
    "rust/stable": ("ok", "rustc 1.82.0 satisfies 2 Cargo.toml"),
    "rust/target": ("missing-toolchain", "target wasm32-unknown-unknown has no standard library"),
    "rust/nightly": ("pin-not-honoured", "channel nightly-2024-10-16, rustup is not installed; "
                     "rustc 1.82.0 stable (2024-10-15) is used"),
    "rust/numeric-channel": ("unsupported", "rust-toolchain.toml channel is not a string: 1.82"),
    "typescript/installed": ("ok", "typescript 5.9.3 installed, satisfies package.json '5.9.3'"),
    "typescript/other-pin": ("pin-not-honoured", "package.json pins typescript 5.4.5, node_modules has 5.9.3"),
    "typescript/out-of-range": ("unsupported", "typescript 5.9.3 in node_modules is outside package.json '>=5.0 <5.5'"),
    "typescript/in-range": ("ok", "satisfies package.json '^5.9.0 || ~4.9'"),
    "typescript/no-version": ("unsupported", "node_modules/typescript/package.json has no version"),
    "typescript/not-installed": ("missing-toolchain", "typescript 5.4.5 is declared and not installed"),
    "go/toolchain#default": ("ok", "release go1.23; installed go1.23.1"),
    "go/too-new#default": ("ok", "release go1.24rc1; installed go1.23.1, go1.24rc1"),
    "go/toolchain#old": ("release-mismatch", "release go1.22; installed go1.23.1"),
    "rust/stable#build": ("release-mismatch", "declares no `release` parameter; installed 1.82.0"),
    "rust/stable#pinned": ("ok", "release 1.82; installed 1.82.0"),
    "python/fits#runtime": ("ok", "python 3.13; installed 3.11.9, 3.13.1"),
    "python/fits#patch": ("release-mismatch", "python 3.11.8; installed 3.11.9, 3.13.1"),
    "typescript/installed#default": ("ok", "compiler 5.9; installed 5.9.3"),
}


def report(corpus, path, **env):
    done = subprocess.run([sys.executable, SCRIPT, "--fixtures", corpus], capture_output=True, text=True,
                          env={"PATH": path, **env})
    states = {}
    for line in done.stdout.splitlines():
        match = re.match(r"(\S+)\s+(\S+?): (.*)", line)
        if match and match.group(1) in toolchains.STATES + ("release-mismatch", "unresolved"):
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
    assert "rust       missing-toolchain 1 unsupported 4 pin-not-honoured 3 ok 3; "\
        "contexts release-mismatch 1 ok 1" in done.stdout, done.stdout
    assert "34 projects, 16 cannot be checked with the installed toolchains; 8 dataset contexts, "\
        "3 release-mismatch" in done.stdout, done.stdout


def test_gotoolchain(scratch):
    corpus = os.path.join(scratch, "fixtures")
    write(corpus, {key: value for key, value in CORPUS.items() if key.startswith("go/")})
    tools(os.path.join(scratch, "bin"), scratch)
    done, states = report(corpus, os.path.join(scratch, "bin"), GOTOOLCHAIN="local")
    assert done.returncode == 1, done.stdout
    assert states["go/too-new"] == ("unsupported", "unsupported: inner/go.mod go 1.24rc1 is newer than go1.23.1, "
                                    "and GOTOOLCHAIN=local does not switch; 1 go.mod(s) supported "
                                    "(local go1.23.1, GOTOOLCHAIN=local)"), states["go/too-new"]
    assert states["go/too-new#default"][0] == "release-mismatch", states
    done, states = report(corpus, os.path.join(scratch, "bin"), GOTOOLCHAIN="path")
    assert states["go/too-new"][0] == "missing-toolchain", states
    assert "selects go1.24rc1, not on PATH (GOTOOLCHAIN=path)" in states["go/too-new"][1], states
    command(os.path.join(scratch, "bin"), "go1.24rc1", "echo go version go1.24rc1 linux/amd64\n")
    done, states = report(corpus, os.path.join(scratch, "bin"), GOTOOLCHAIN="path")
    assert states["go/too-new"][0] == "ok" and states["go/too-new#default"][0] == "ok", states


def test_unresolved_gotoolchain(scratch):
    corpus = os.path.join(scratch, "fixtures")
    write(corpus, {key: value for key, value in CORPUS.items() if key.startswith("go/projects/toolchain/")})
    bin = os.path.join(scratch, "bin")
    tools(bin, scratch)
    command(bin, "go", 'if [ "$1" = env ]; then exit 1; else echo "go version go1.23.1 linux/amd64"; fi\n')
    done, states = report(corpus, bin)
    assert done.returncode == 1, done.stdout
    assert states["go/toolchain"][0] == "unsupported", states
    assert "GOTOOLCHAIN is unresolved: `go env GOTOOLCHAIN` failed" in states["go/toolchain"][1], states
    command(bin, "go", 'if [ "$1" = env ]; then echo; else echo "go version go1.23.1 linux/amd64"; fi\n')
    done, states = report(corpus, bin)
    assert states["go/toolchain"] == ("ok", "1 go.mod(s) supported (local go1.23.1, GOTOOLCHAIN=auto)"), states


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
    passing = {key: value for key, value in CORPUS.items() if "/pinned/" in key or "/caret/" in key}
    passing["rust/projects/pinned/semantic-oracle.yml"] = "contexts:\n  - id: a\n    parameters:\n      release: 1.82.0\n"
    write(corpus, passing)
    tools(os.path.join(scratch, "bin"), scratch)
    done, states = report(corpus, os.path.join(scratch, "bin"))
    assert done.returncode == 0, done.stdout
    assert states["rust/pinned"][0] == "pin-not-honoured"
    assert states["rust/pinned#a"][0] == "ok", states


def test_context_forms(scratch):
    """Every context is read whatever its key order or style, and a dataset
    whose contexts cannot all be read is unresolved, never zero contexts."""
    corpus = os.path.join(scratch, "fixtures")
    bin = os.path.join(scratch, "bin")
    os.makedirs(bin)
    command(bin, "python3.14", 'echo "Python 3.14.2"\n')
    write(corpus, {
        "python/projects/forms/semantic-oracle.yml":
            "contexts:\n  - parameters:\n      python: \"3.99\"\n    id: reordered\n    command: none\n"
            "  - {id: flow, parameters: {python: '3.99', options: [a, b]}, command: none}\n"
            "  -\n    id: bare\n    parameters: {python: '3.14'}\n",
        "python/projects/flow-list/semantic-oracle.yml":
            "contexts: [{id: listed, parameters: {python: \"3.99\"}, command: none},\n"
            "           {parameters: {python: \"3.14\"}, id: second, command: none}]\n",
        "python/projects/alias/semantic-oracle.yml":
            "contexts:\n  - id: a\n    parameters:\n      python: *py\n",
        "python/projects/no-id/semantic-oracle.yml":
            "contexts:\n  - parameters:\n      python: '3.14'\n    command: none\n",
        "python/projects/empty/semantic-oracle.yml": "contexts: []\n",
        "python/projects/absent/semantic-oracle.yml": "profile: python/1\n",
        "python/projects/scalar/semantic-oracle.yml": "contexts:\n  - default\n",
    })
    done, states = report(corpus, bin)
    assert done.returncode == 1, done.stdout
    assert states["python/forms#reordered"] == ("release-mismatch", "python 3.99; installed 3.14.2"), states
    assert states["python/forms#flow"] == ("release-mismatch", "python 3.99; installed 3.14.2"), states
    assert states["python/forms#bare"] == ("ok", "python 3.14; installed 3.14.2"), states
    assert states["python/flow-list#listed"][0] == "release-mismatch", states
    assert states["python/flow-list#second"][0] == "ok", states
    for project, reason in (("alias", "unsupported YAML value '*py'"), ("no-id", "context 1 has no id"),
                            ("empty", "`contexts` lists no context"), ("absent", "no top-level `contexts`"),
                            ("scalar", "cannot read 'default'")):
        line = states[f"python/{project}#semantic-oracle.yml"]
        assert line[0] == "unresolved" and reason in line[1], (project, line)
    assert "5 dataset contexts, 3 release-mismatch; 5 semantic-oracle.yml unresolved" in done.stdout, done.stdout


def test_prerelease_python(scratch):
    """A prerelease interpreter satisfies neither a final requires-python
    bound nor a context naming the final release."""
    corpus = os.path.join(scratch, "fixtures")
    bin = os.path.join(scratch, "bin")
    os.makedirs(bin)
    command(bin, "python3.14", 'echo "Python 3.14.0rc1"\n')
    write(corpus, {
        "python/projects/final/pyproject.toml": '[project]\nname = "a"\nrequires-python = ">=3.14.0"\n',
        "python/projects/final/semantic-oracle.yml":
            "contexts:\n  - id: patch\n    parameters:\n      python: \"3.14.0\"\n"
            "  - id: minor\n    parameters:\n      python: \"3.14\"\n"
            "  - id: exact\n    parameters:\n      python: \"3.14.0rc1\"\n",
        "python/projects/pre/pyproject.toml": '[project]\nname = "a"\nrequires-python = ">=3.14.0rc1"\n',
    })
    done, states = report(corpus, bin)
    assert done.returncode == 1, done.stdout
    assert states["python/final"] == ("unsupported", "unsupported: requires-python '>=3.14.0' "
                                      "(pyproject.toml) matches none of python3.14 (3.14.0rc1)"), states
    assert states["python/pre"][0] == "ok", states
    assert states["python/final#patch"] == ("release-mismatch", "python 3.14.0; installed 3.14.0rc1"), states
    assert states["python/final#minor"] == ("release-mismatch", "python 3.14; installed 3.14.0rc1"), states
    assert states["python/final#exact"] == ("ok", "python 3.14.0rc1; installed 3.14.0rc1"), states


def test_empty(scratch):
    done = subprocess.run([sys.executable, SCRIPT, "--fixtures", scratch], capture_output=True, text=True)
    assert done.returncode == 2 and "fixtures:init" in done.stdout, done.stdout


def test_versions(_):
    roll = toolchains.roll_forward
    installed = ["8.0.204", "8.0.110", "9.0.100"]
    assert roll("8.0.100", None, installed) == "8.0.110"
    assert roll("8.0.100", "patch", installed) == "8.0.110"
    assert roll("8.0.100", "feature", installed) == "8.0.110"
    assert roll("8.0.100", "latestFeature", installed) == "8.0.204"
    assert roll("8.0.300", "feature", installed) is None
    assert roll("8.0.300", "major", installed) == "9.0.100"
    assert roll("8.0.300", "latestMajor", installed) == "9.0.100"
    assert roll("8.0.110", "disable", installed) == "8.0.110"
    patches = ["8.0.110", "8.0.115", "8.0.120"]
    assert roll("8.0.115", None, patches) == "8.0.115"
    assert roll("8.0.115", "latestPatch", patches) == "8.0.120"
    assert roll("8.0.112", "patch", patches) == "8.0.120"
    assert roll("8.0.115", "feature", patches) == "8.0.120"
    spread = ["8.1.100", "8.1.104", "8.1.200", "8.2.300", "9.0.100", "9.0.105", "10.0.100"]
    assert roll("8.0.100", "feature", spread) is None
    assert roll("8.0.100", "minor", spread) == "8.1.104"
    assert roll("8.0.100", "latestMinor", spread) == "8.2.300"
    assert roll("8.3.100", "minor", spread) is None
    assert roll("8.3.100", "major", spread) == "9.0.105"
    assert roll("8.3.100", "latestMajor", spread) == "10.0.100"
    previews = ["8.0.204-preview.1.24101.1"]
    assert roll("8.0.100", "latestFeature", previews) == "8.0.204-preview.1.24101.1"
    assert roll("8.0.100", "latestFeature", previews, allow_prerelease=False) is None
    mixed = ["8.0.110", "8.0.204-preview.1", "8.0.204"]
    assert roll("8.0.200", "latestFeature", mixed) == "8.0.204"
    assert roll("8.0.204-preview.1", "disable", mixed) == "8.0.204-preview.1"
    assert roll("8.0.204-preview.1", "disable", mixed, allow_prerelease=False) is None
    assert roll("8.0.200", "patch", ["8.0.204-rc.1", "8.0.203"]) == "8.0.204-rc.1"
    assert roll("8.0.200", "patch", ["8.0.204-rc.1", "8.0.203"], allow_prerelease=False) == "8.0.203"
    select = toolchains.go_select
    assert select("go1.23.1", "local", "1.24", None)[2] == "unsupported"
    assert select("go1.23.1", "local", "1.22", "go1.24.0") == (
        "go1.23.1", "toolchain go1.24.0 is a suggestion GOTOOLCHAIN=local does not follow", "ok")
    assert select("go1.23.1", "auto", "1.22", "go1.24.0") == ("go1.24.0", None, "ok")
    assert select("go1.23.1", "auto", "1.22", "go1.22.5") == ("go1.23.1", None, "ok")
    assert select("go1.23.1", "go1.22.5+auto", "1.22", None) == ("go1.22.5", None, "ok")
    assert select("go1.23.1", "go1.25.0", "1.24", "go1.26.0") == (
        "go1.25.0", "toolchain go1.26.0 is a suggestion GOTOOLCHAIN=go1.25.0 does not follow", "ok")
    assert select("go1.23.1", "local+path", "1.24.2", "default") == (
        "go1.23.1", "go 1.24.2 is newer than go1.23.1, and go.mod sets toolchain default", "unsupported")
    assert select("go1.27.3", "auto", "1.99.0", "default")[2] == "unsupported"
    assert select("go1.27.3", "auto", "1.22", "default") == ("go1.27.3", None, "ok")
    stable = {"version": (1, 82, 0), "channel": "stable", "date": "2024-10-15", "active": None}
    nightly = dict(stable, version=(1, 84, 0), channel="nightly", date="2024-10-15")
    matches = toolchains.rust_pin_matches
    assert matches("1.82", stable) and matches("1.82.0", stable) and matches("stable", stable)
    assert not matches("1.82.1", stable) and not matches("nightly", stable)
    assert matches("nightly-2024-10-16", nightly) and matches("nightly-2024-10-15", nightly)
    assert not matches("nightly-2024-10-18", nightly) and not matches("nightly-2024-10-16", stable)
    assert not matches("nightly-2024-10-16", dict(nightly, date=None))
    assert matches("nightly-2024-10-16", dict(stable, active="nightly-2024-10-16-x86_64-unknown-linux-gnu"))
    assert not matches("nightly-2024-10-16", dict(stable, active="nightly-2024-10-17-x86_64-unknown-linux-gnu"))
    npm = toolchains.npm_satisfies
    assert npm("5.9.3", "5.9.3") and npm("5.9.3", "^5.4.0") and npm("5.9.3", "~5.9.1") and npm("5.9.3", "5.x")
    assert npm("5.9.3", ">= 5.0.0") and npm("5.9.3", "5.0 - 5.9") and npm("5.9.3", "*")
    assert not npm("5.9.3", "~5.8") and not npm("5.9.3", "<5.9") and not npm("5.9.3", "^0.5.0")
    assert not npm("5.4.0", "5.4.0-dev.20240119") and npm("5.4.0-dev.20240119", "5.4.0-dev.20240119")
    assert not npm("5.9.0-dev.20240101", "^5.4.0") and not npm("5.9.0-dev.20240101", ">=5.0.0 <6")
    assert npm("5.9.0-dev.20240101", ">=5.9.0-dev.20231201 <6") and npm("5.9.0-beta", "^5.9.0-alpha")
    assert not npm("5.9.1-beta", "^5.9.0-alpha") and npm("5.9.1-beta", "^5.9.0-alpha || >=5.9.1-alpha")
    assert not npm("6.0.0-dev.1", "5.0 - 6.0.0")
    assert toolchains.framework_major("net8.0-windows") == 8
    assert toolchains.framework_major("netcoreapp3.1") == 3
    assert toolchains.framework_major("net472") == 0
    satisfied = lambda version, constraint: toolchains.satisfies(
        version, toolchains.dart_constraint(constraint), toolchains.dart_version)
    assert satisfied("3.5.0", ">=2.12.0-0 <3.0.0")
    assert not satisfied("3.5.0", ">=2.9.0-18.0 <3.0.0")
    assert satisfied("3.5.0", "^3.0.0") and not satisfied("4.0.0", "^3.0.0")
    assert not satisfied("3.5.0-1.0.dev", ">=3.5.0")
    assert toolchains.dart_constraint("any") == [] and toolchains.dart_constraint("'>=3.0.0'") == [(">=", "3.0.0")]
    for garbled in ("", "''", ">=3.0.0 <4.0.0 junk", ">=3.0 <4.0.0", "^3", "3.0.0 || 4.0.0"):
        try:
            toolchains.dart_constraint(garbled)
        except ValueError:
            continue
        raise AssertionError(f"dart_constraint accepted {garbled!r}")
    assert satisfied("0.3.1", "^0.3.0") and not satisfied("0.4.0", "^0.3.0")
    assert toolchains.go_version("1.21") < toolchains.go_version("go1.21rc2") < toolchains.go_version("1.21.0")
    pep = toolchains.pep440_satisfies
    assert pep("3.11.9", ">=3.10, <3.12") and not pep("3.12.0", ">=3.10, <3.12")
    assert pep("3.11.2", "==3.11.*") and not pep("3.12.0", "==3.11.*")
    assert pep("3.11.2", "~=3.11") and pep("3.13.0", "~=3.11") and not pep("3.12.0", "~=3.11.1")
    assert pep("3.13.0", "!=3.12.*")
    assert pep("3.13.0", ">=3.13.0rc1") and pep("3.13.0", ">=3.13.0b2") and pep("3.13.0", ">=3.13.0.dev0")
    assert not pep("3.12.9", ">=3.13.0rc1") and not pep("3.13.0", "==3.13.0rc1")
    assert not pep("3.13.0", "<3.13.0rc1") and pep("3.13.0", "<3.13.0.post1")
    assert not pep("3.13.0", ">3.13.0.post1") and pep("3.13.0", "~=3.13.0a1") and pep("3.13.0", ">=3.13a1")
    assert not pep("3.14.0rc1", ">=3.14.0") and not pep("3.14.0rc1", ">=3.14") and not pep("3.14.0b2", "==3.14.0")
    assert pep("3.14.0rc1", ">=3.14.0rc1") and pep("3.14.0rc1", ">=3.13") and pep("3.14.0rc1", "<3.14")
    assert pep("3.14.0rc1", "==3.14.*") and pep("3.14.0rc1", "~=3.13") and not pep("3.14.0rc1", ">=3.14.0rc2")
    assert pep("3.14.0+", ">=3.14.0")
    assert toolchains.extends("go1.27.3", "go1.27") and toolchains.extends("10.0.403", "10.0.403")
    assert not toolchains.extends("1.990", "1.99") and not toolchains.extends("1.9", "1.99")
    assert not toolchains.extends("3.14.0rc1", "3.14") and not toolchains.extends("3.14.0rc1", "3.14.0")
    assert not toolchains.extends("5.9.0-beta", "5.9") and not toolchains.extends("1.84.0-nightly", "1.84")
    assert not toolchains.extends("go1.24rc1", "go1.24") and toolchains.extends("3.14.0rc1", "3.14.0rc1")
    assert toolchains.contexts("contexts:\n- id: a\n  parameters:\n    release: '3.13'\n    options: []\n"
                               "- id: \"b\"\n  parameters: {}\nrelease: 9\n") == [
        ("a", {"release": "3.13"}), ("b", {})]
    assert toolchains.contexts("contexts:\n  - id: a\n    parameters:\n      options:\n        strict: true\n"
                               "      'release': '1.2' # pinned\n    description: >\n      text\n") == [
        ("a", {"release": "1.2"})]
    for garbled in ("contexts:\n  - id: a\n    parameters:\n      release: 1.2\n        3\n",
                    "contexts:\n\t- id: a\n    parameters: {}\n",
                    "contexts:\n  - id: a\n    id: b\n    parameters: {}\n",
                    "contexts:\n  - id: a\n    command: x\n", "contexts:\n  - &x {id: a, parameters: {}}\n",
                    "contexts:\n  - id: a\n    parameters: {}\n   - id: b\n    parameters: {}\n",
                    "contexts:\n  - {id: a, parameters: {}\n", "contexts: [] \ncontexts: []\n"):
        try:
            toolchains.contexts(garbled)
        except ValueError:
            continue
        raise AssertionError(f"contexts read {garbled!r}")


def main():
    failed = 0
    for test in (test_versions, test_corpus, test_gotoolchain, test_unresolved_gotoolchain, test_missing,
                 test_passing, test_context_forms, test_prerelease_python, test_empty):
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
