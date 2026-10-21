#!/usr/bin/env python3
# ---
# relationships:
#   validates: publishing
# ---
"""Exercise the release event boundary and the jobs that consume it."""
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[2]
VERSION = tomllib.loads((ROOT / "Cargo.toml").read_text())["workspace"]["package"]["version"]


def workflow(name):
    return json.loads(subprocess.check_output(
        ["yq", "-o=json", ".", str(ROOT / ".github/workflows" / name)], text=True
    ))


class ReleaseEventTests(unittest.TestCase):
    def run_event(self, event, ref_type="branch", ref_name="main", npm_version=None):
        with tempfile.TemporaryDirectory() as directory:
            cwd = ROOT
            if npm_version is not None:
                cwd = Path(directory) / "source"
                (cwd / "npm").mkdir(parents=True)
                shutil.copytree(ROOT / "scripts/release", cwd / "scripts/release")
                for name in ("Cargo.toml", "CHANGELOG.md"):
                    shutil.copyfile(ROOT / name, cwd / name)
                package = json.loads((ROOT / "npm/package.json").read_text())
                package["version"] = npm_version
                (cwd / "npm/package.json").write_text(json.dumps(package))
            output = Path(directory) / "output"
            env = {**os.environ, "GITHUB_EVENT_NAME": event,
                   "GITHUB_REF_TYPE": ref_type, "GITHUB_REF_NAME": ref_name,
                   "RUNNER_TEMP": directory, "GITHUB_OUTPUT": str(output)}
            result = subprocess.run(
                ["bash", "scripts/release/validate-event.sh"], cwd=cwd,
                env=env, capture_output=True, text=True
            )
            return result, output.read_text() if output.exists() else ""

    def test_manual_branch_build_uses_manifest_version_without_credentials(self):
        result, output = self.run_event("workflow_dispatch")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(output, f"version={VERSION}\n")

    def test_manual_tag_dispatch_is_still_a_build(self):
        result, output = self.run_event("workflow_dispatch", "tag", "9.9.9")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(output, f"version={VERSION}\n")

    def test_version_tag_push_checks_manifests_and_changelog(self):
        result, output = self.run_event("push", "tag", VERSION)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(output, f"version={VERSION}\n")

    def test_branch_push_cannot_release_even_with_version_shaped_name(self):
        result, output = self.run_event("push", "branch", VERSION)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("plain Semantic Versioning tag", result.stderr)
        self.assertEqual(output, "")

    def test_malformed_tag_cannot_release(self):
        for tag in (f"v{VERSION}", "01.2.3", "1.2.3-rc.1"):
            with self.subTest(tag=tag):
                result, output = self.run_event("push", "tag", tag)
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(output, "")

    def test_different_release_version_is_rejected(self):
        result, output = self.run_event("push", "tag", "9.9.9")
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(output, "")

    def test_manifest_disagreement_rejects_otherwise_valid_release(self):
        result, output = self.run_event("push", "tag", VERSION, npm_version="9.9.9")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("npm package is 9.9.9", result.stderr)
        self.assertEqual(output, "")

    def test_other_event_cannot_enter_release_workflow(self):
        result, output = self.run_event("pull_request", "tag", VERSION)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("Unsupported", result.stderr)
        self.assertEqual(output, "")


class WorkflowBoundaryTests(unittest.TestCase):
    def test_every_publication_job_requires_a_tag_push_and_github_release(self):
        jobs = workflow("cd.yml")["jobs"]
        channels = {"publish-crates", "publish-npm", "publish-homebrew-tap"}
        for name in channels | {"release", "verify-cargo-client", "verify-npm-clients",
                                "verify-homebrew-clients", "update-major-tag"}:
            with self.subTest(job=name):
                self.assertEqual(jobs[name]["if"],
                                 "github.event_name == 'push' && github.ref_type == 'tag'")
        for name in channels:
            self.assertEqual(jobs[name]["needs"], "release")
        self.assertEqual(set(jobs["release"]["needs"]), {"build", "verify-arm64-archive"})
        self.assertEqual(jobs["verify-arm64-archive"]["needs"], "build")
        self.assertEqual(jobs["verify-arm64-archive"]["runs-on"], "ubuntu-24.04-arm")
        archive_steps = jobs["verify-arm64-archive"]["steps"]
        self.assertTrue(any(step.get("with", {}).get("name") == "beholder-linux-arm64"
                            for step in archive_steps))
        self.assertTrue(any("./beholder-linux-arm64/beholder --version" in step.get("run", "")
                            for step in archive_steps))
        self.assertTrue(any("ensure-github-release.sh" in step.get("run", "")
                            for step in jobs["release"]["steps"]))
        self.assertEqual(jobs["build"]["needs"], "validate")
        self.assertEqual(jobs["validate"]["needs"], "credential-preflight")
        self.assertEqual(jobs["validate"]["uses"], "./.github/workflows/ci.yml")
        self.assertTrue(any(step.get("run", "").strip() == "scripts/release/validate-event.sh"
                            for step in jobs["build"]["steps"]))

    def test_manual_preflight_does_not_require_publication_credentials(self):
        preflight = workflow("cd.yml")["jobs"]["credential-preflight"]
        self.assertNotIn("env", preflight)
        credential_steps = [s for s in preflight["steps"] if "secrets." in str(s)]
        self.assertEqual(len(credential_steps), 1)
        self.assertEqual(credential_steps[0]["if"], "github.event_name == 'push'")

    def test_releases_for_same_ref_are_serialized(self):
        self.assertEqual(workflow("cd.yml")["concurrency"],
                         {"group": "release-${{ github.ref }}", "cancel-in-progress": False})

    def test_docs_publish_from_main_without_release_dependency(self):
        docs = workflow("publish-docs.yml")
        self.assertEqual(docs["on"]["push"], {"branches": ["main"], "paths": ["docs/**"]})
        self.assertIn("workflow_dispatch", docs["on"])
        self.assertEqual(docs["concurrency"],
                         {"group": "publish-docs", "cancel-in-progress": False})
        self.assertNotIn("needs", docs["jobs"]["publish"])
        self.assertNotIn("publish-docs", workflow("cd.yml")["jobs"])

    def test_hosted_ci_runs_full_gate_and_supported_platform_tests(self):
        ci = workflow("ci.yml")
        self.assertIn("pull_request", ci["on"])
        self.assertIn("workflow_call", ci["on"])
        self.assertEqual(ci["on"]["push"]["branches"], ["main"])
        self.assertIn("task ci", [s.get("run") for s in ci["jobs"]["ci"]["steps"]])
        platforms = ci["jobs"]["platforms"]
        self.assertEqual(set(platforms["strategy"]["matrix"]["os"]),
                         {"ubuntu-latest", "ubuntu-24.04-arm", "macos-latest", "windows-latest"})
        self.assertIn("cargo test --workspace --locked", [s.get("run") for s in platforms["steps"]])
        self.assertIn("npm test --prefix npm", [s.get("run") for s in platforms["steps"]])

    def test_external_actions_are_commit_pinned(self):
        files = [*ROOT.glob(".github/workflows/*.yml"), ROOT / "action.yml"]
        count = 0
        for path in files:
            for action in re.findall(r"^\s*(?:- )?uses:\s*([^\s#]+)", path.read_text(), re.MULTILINE):
                if action.startswith("./"):
                    continue
                count += 1
                self.assertRegex(action, r"@[0-9a-f]{40}$", str(path))
        self.assertGreater(count, 10)


if __name__ == "__main__":
    unittest.main()
