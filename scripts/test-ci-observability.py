#!/usr/bin/env python3
"""CI instrumentation must preserve failure, source identity and bounded output."""
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('ci_timing', ROOT / 'scripts/ci-timing.py')
timing = importlib.util.module_from_spec(spec)
spec.loader.exec_module(timing)


class TimingTests(unittest.TestCase):
    def setUp(self):
        parent = ROOT / 'build/test-ci-observability';parent.mkdir(parents=True, exist_ok=True)
        self.temp = tempfile.TemporaryDirectory(dir=parent);self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.assertTrue(self.root.resolve().is_relative_to(parent.resolve()))

    def test_failed_and_successful_commands_keep_original_status(self):
        target = self.root / 'timing.json'
        timing.append(target, 'anchors', 'owned.py', 1000000, 3000000, 7, 'a' * 40)
        timing.append(target, 'body', 'other.py', 3000000, 4000000, 0, 'a' * 40)
        rows = json.loads(target.read_text(encoding='utf-8'))['commands']
        self.assertEqual([(r['status'], r['exit_code']) for r in rows], [('failed', 7), ('passed', 0)])
        self.assertEqual(rows[0]['elapsed_ms'], 2)

    def test_other_source_and_private_labels_cannot_be_mixed(self):
        target = self.root / 'timing.json'
        timing.append(target, 'anchors', 'owned.py', 1, 2, 0, 'a' * 40)
        with self.assertRaisesRegex(ValueError, 'another source'):
            timing.append(target, 'anchors', 'owned.py', 1, 2, 0, 'b' * 40)
        for label in ['private/path.py', 'name\nsecret', 'C:\\private.py']:
            with self.assertRaisesRegex(ValueError, 'label'):
                timing.append(target, 'anchors', label, 1, 2, 0, 'a' * 40)

    def powershell(self, body):
        executable = shutil.which('pwsh')
        if not executable:self.skipTest('pwsh unavailable')
        source = self.root / 'owned.ps1'
        quoted = str(ROOT / 'scripts/ci-observability.ps1').replace("'", "''")
        source.write_text("$ErrorActionPreference='Stop'\n. '" + quoted + "'\n" + body, encoding='utf-8')
        return subprocess.run([executable, '-NoProfile', '-NonInteractive', '-File', str(source)],
                              cwd=ROOT, capture_output=True, timeout=30)

    def test_native_failure_is_not_hidden(self):
        stub = self.root / 'owned-child.py';stub.write_text('raise SystemExit(7)\n', encoding='utf-8')
        target = self.root / 'ps-timing.json'
        rel = target.relative_to(ROOT).as_posix()
        body = "Initialize-CiTiming -Scope fixture -OutputPath '" + rel + "'\nSet-CiPhase 'owned'\n"
        body += "try { Invoke-CiCommand -Program '" + sys.executable.replace("'", "''") + "' -Arguments @('" + str(stub).replace("'", "''") + "'); exit 91 } catch { if ($_.Exception.Message -notmatch 'exit code 7') { throw }; exit 0 }\n"
        result = self.powershell(body)
        self.assertEqual(result.returncode, 0, result.stderr.decode(errors='replace'))
        row = json.loads(target.read_text(encoding='utf-8'))['commands'][0]
        self.assertEqual((row['status'], row['exit_code'], row['command']), ('failed', 7, 'owned-child.py'))
        self.assertNotIn(str(self.root), target.read_text(encoding='utf-8'))

    def test_success_postcondition_failure_and_later_success_keep_separate_results(self):
        stub = self.root / 'owned-child.py';stub.write_text('raise SystemExit(0)\n', encoding='utf-8')
        target = self.root / 'ps-timing.json'
        body = "Initialize-CiTiming -Scope fixture -OutputPath '" + target.relative_to(ROOT).as_posix() + "'\n"
        command = "Invoke-CiCommand -Program '" + sys.executable.replace("'", "''") + "' -Arguments @('" + str(stub).replace("'", "''") + "')"
        body += "Set-CiPhase 'postcondition'\ntry { " + command + " -AfterSuccess { throw 'owned-postcondition' }; exit 93 } catch { if ($_.Exception.Message -ne 'owned-postcondition') { throw } }\n"
        body += "Set-CiPhase 'complete'\n" + command + "\n"
        result = self.powershell(body)
        self.assertEqual(result.returncode, 0, result.stderr.decode(errors='replace'))
        rows = json.loads(target.read_text(encoding='utf-8'))['commands']
        self.assertEqual([(row['phase'], row['status'], row['exit_code']) for row in rows],
                         [('postcondition', 'failed', 0), ('complete', 'passed', 0)])

    def test_source_output_is_rejected_before_writing(self):
        result = self.powershell("try { Initialize-CiTiming -Scope fixture -OutputPath 'scripts/forbidden-timing.json'; exit 92 } catch { exit 0 }\n")
        self.assertEqual(result.returncode, 0)
        self.assertFalse((ROOT / 'scripts/forbidden-timing.json').exists())


if __name__ == '__main__':unittest.main()
