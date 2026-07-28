#!/usr/bin/env python3

import argparse
import json
from pathlib import Path
import sys
import tomllib


def fail(message: str) -> None:
    print(f"release version check failed: {message}", file=sys.stderr)
    raise SystemExit(1)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--expected")
    args = parser.parse_args()

    root = Path(__file__).resolve().parents[2]
    workspace = tomllib.loads((root / "Cargo.toml").read_text())
    npm = json.loads((root / "npm/package.json").read_text())

    cargo_version = workspace["workspace"]["package"]["version"]
    # The CLI depends on the library through the workspace table, so this pin is
    # the one place a published beholder-cli says which beholder it needs.
    library_dependency = workspace["workspace"]["dependencies"]["beholder"]["version"]
    npm_version = npm["version"]
    expected = args.expected or cargo_version

    versions = {
        "Cargo workspace": cargo_version,
        "beholder library pin": library_dependency,
        "npm package": npm_version,
    }
    for source, version in versions.items():
        if version != expected:
            fail(f"{source} is {version}, expected {expected}")

    if npm["name"] != "@wyrd-company/beholder":
        fail("npm package name is not @wyrd-company/beholder")
    if npm["publishConfig"] != {
        "access": "public",
        "registry": "https://registry.npmjs.org",
    }:
        fail("npm publishConfig must bind public npmjs publication")
    if npm["bin"] != {"beholder": "bin/beholder.js"}:
        fail("npm executable mapping is not exact")

    print(f"release versions agree at {expected}")


if __name__ == "__main__":
    main()
