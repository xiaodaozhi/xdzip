#!/usr/bin/env python3
"""Check the cold-start wrapper's bounded logging and owned process cleanup."""
import importlib.util
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import unittest

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('x11_startup', ROOT/'scripts/check-x11-startup.py')
startup = importlib.util.module_from_spec(spec)
spec.loader.exec_module(startup)


@unittest.skipUnless(sys.platform.startswith('linux'), 'Requires Linux process groups')
class X11SupervisionTests(unittest.TestCase):
    def setUp(self):
        (ROOT/'build').mkdir(exist_ok=True)
        self.temp = tempfile.TemporaryDirectory(dir=ROOT/'build')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)

    def test_exit_status_and_output_are_preserved(self):
        for code in (0, 7):
            with self.subTest(code=code):
                log = self.root/(str(code)+'.log')
                command = [sys.executable, '-c', 'import sys; print("owned output", flush=True); sys.exit(int(sys.argv[1]))', str(code)]
                self.assertEqual(startup.supervise(command, log, timeout=5), code)
                self.assertIn('owned output', log.read_text(encoding='utf-8'))

    def test_timeout_kills_owned_child_tree_and_preserves_unrelated_process(self):
        marker = self.root/'child.pid'
        log = self.root/'timeout.log'
        program = ('import pathlib,subprocess,sys,time; '
                   'p=subprocess.Popen([sys.executable,"-c","import time; time.sleep(30)"]); '
                   'pathlib.Path(sys.argv[1]).write_text(str(p.pid)); '
                   'print("child started",flush=True); time.sleep(30)')
        unrelated = subprocess.Popen([sys.executable, '-c', 'import time; time.sleep(30)'], start_new_session=True)
        try:
            with self.assertRaises(subprocess.TimeoutExpired):
                startup.supervise([sys.executable, '-c', program, str(marker)], log, timeout=2)
            self.assertIn('child started', log.read_text(encoding='utf-8'))
            child = int(marker.read_text())
            stat = Path('/proc')/str(child)/'stat'
            deadline = time.monotonic()+5
            while stat.exists() and stat.read_text().split()[2] != 'Z' and time.monotonic() < deadline:
                time.sleep(.02)
            self.assertTrue(not stat.exists() or stat.read_text().split()[2] == 'Z')
            self.assertIsNone(unrelated.poll())
        finally:
            unrelated.terminate()
            unrelated.wait(timeout=5)


if __name__ == '__main__':
    unittest.main()
