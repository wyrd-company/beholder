#!/usr/bin/env python3
# ---
# relationships:
#   validates: stored-results
# ---
"""Verify action publication boundaries and the successful-main checkout."""
import json
from pathlib import Path
import subprocess
import unittest

ROOT = Path(__file__).resolve().parents[2]


def load(path):
    return json.loads(subprocess.check_output(['yq', '-o=json', '.', str(ROOT / path)], text=True))


class IndexWorkflowTests(unittest.TestCase):
    def test_fork_defaults_do_not_publish(self):
        action = load('action.yml')
        self.assertEqual(action['inputs']['publish-index']['default'], 'false')
        self.assertEqual(action['inputs']['mode']['default'], 'report')
        steps = action['runs']['steps']
        publish = next(s for s in steps if s['name'] == 'Publish the stored index')
        self.assertEqual(publish['if'], "inputs.publish-index == 'true'")
        index = next(s for s in steps if s['name'] == 'Update the checked-out index')
        self.assertEqual(index['if'], "inputs.mode == 'index'")
        self.assertEqual(index['run'], '"$BEHOLDER" index --store --use-store')
        report = next(s for s in steps if s['name'] == 'Report')
        self.assertEqual(report['if'], "inputs.mode == 'report'")
        self.assertEqual(load('.github/workflows/beholder.yml')['permissions']['contents'], 'read')

    def test_successful_main_sha(self):
        workflow = load('.github/workflows/beholder-index.yml')
        self.assertEqual(workflow['on']['workflow_run']['workflows'], ['CI'])
        self.assertEqual(workflow['on']['workflow_run']['branches'], ['main'])
        job = workflow['jobs']['index']
        for guard in ["conclusion == 'success'", "event == 'push'", "head_branch == 'main'",
                      'head_repository.full_name == github.repository']:
            self.assertIn(guard, job['if'])
        self.assertEqual(job['steps'][0]['with']['ref'], '${{ github.event.workflow_run.head_sha }}')
        self.assertEqual(job['steps'][1]['with'], {'mode': 'index', 'publish-index': 'true'})


if __name__ == '__main__':
    unittest.main()
