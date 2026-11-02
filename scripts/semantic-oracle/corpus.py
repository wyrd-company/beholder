#!/usr/bin/env python3
# ---
# relationships:
#   references: semantic-oracle-model
# ---
"""Runs `semantic-oracle validate` and `audit` on the corpus's adopted datasets.

A dataset has adopted this model version when its `public-api` holds `members`.
`adopted-datasets.txt`, beside this script, lists the datasets that must have: a
listed dataset in the old shape fails, so that a dataset cannot leave the check
unnoticed. A dataset in neither is skipped, and named. Compiler reports are not
part of this check, so every signature slot is `unchecked` here.

  corpus.py --fixtures DIR --binary PATH
"""
import argparse
import json
import os
import re
import subprocess
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
ADOPTED = re.compile(r"^public-api:\s*\n(?:\s*#[^\n]*\n)*\s+members:", re.M)


def required():
    listed = set()
    with open(os.path.join(HERE, "adopted-datasets.txt")) as source:
        for line in source:
            line = line.split("#", 1)[0].strip()
            if line:
                listed.add(line)
    return listed


def datasets(root):
    for directory, names, files in os.walk(root):
        names[:] = [name for name in names if name not in (".git", "node_modules", "target")]
        if "semantic-oracle.yml" in files:
            yield directory


def check(binary, command, project):
    started = time.monotonic()
    done = subprocess.run([binary, command, project], capture_output=True, text=True)
    lines = done.stdout.splitlines()
    summary = {}
    if lines:
        try:
            summary = json.loads(lines[-1]).get("summary", {})
        except json.JSONDecodeError:
            pass
    findings = [line for line in lines if '"severity":"error"' in line]
    return done.returncode, summary, findings, round(time.monotonic() - started, 2), done.stderr


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fixtures", required=True)
    parser.add_argument("--binary", required=True)
    arguments = parser.parse_args()
    root = os.path.abspath(arguments.fixtures)
    listed = required()
    seen = set()
    failed = 0
    checked = 0
    for project in sorted(datasets(root)):
        name = os.path.relpath(project, root)
        seen.add(name)
        with open(os.path.join(project, "semantic-oracle.yml")) as source:
            text = source.read()
        if not ADOPTED.search(text):
            if name in listed:
                print(f"FAIL {name}: listed as adopted, and its public-api is in the old shape")
                failed += 1
            else:
                print(f"skip {name}: has not adopted the public-api shape")
            continue
        checked += 1
        for command in ("validate", "audit"):
            code, summary, findings, seconds, errors = check(arguments.binary, command, project)
            print(f"{command} {name}: exit {code}, {json.dumps(summary, sort_keys=True)}, {seconds}s")
            if code:
                failed += 1
                for line in findings[:50]:
                    print(f"  {line}")
                if errors:
                    print(f"  {errors.strip()}")
    for name in sorted(listed - seen):
        print(f"FAIL {name}: listed as adopted, and the fixtures have no such dataset")
        failed += 1
    print(f"checked {checked} dataset(s), {failed} failure(s)")
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
