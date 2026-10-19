#!/usr/bin/env python3
"""Use real staging, including forced ignored files, to enforce content boundaries."""
from pathlib import Path
import hashlib
import runpy
import subprocess
import tempfile
import unittest
from repository_contents import MAX_BLOB_BYTES, violations


class ContentsTests(unittest.TestCase):
    def test_historical_source_bytes_survive_windows_style_checkout(self):
        project = Path(__file__).resolve().parents[1]
        contract = runpy.run_path(str(project/'scripts/check-settings-compat.py'))['SOURCES']
        prefix = Path('apps/desktop/tests/preferences-compat/previous')
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)/'repo';root.mkdir()
            output = Path(directory)/'checkout';output.mkdir()
            subprocess.run(['git', 'init', '-q', str(root)], check=True)
            subprocess.run(['git', 'config', 'core.autocrlf', 'true'], cwd=root, check=True)
            (root/'.gitattributes').write_bytes((project/'.gitattributes').read_bytes())
            for name in contract:
                destination = root/prefix/name;destination.parent.mkdir(parents=True, exist_ok=True)
                destination.write_bytes((project/prefix/name).read_bytes())
            subprocess.run(['git', 'add', '.gitattributes', prefix.as_posix()], cwd=root, check=True)
            subprocess.run(['git', 'checkout-index', '--all', '--prefix='+output.as_posix()+'/'],
                           cwd=root, check=True)
            for name, expected in contract.items():
                self.assertEqual(hashlib.sha256((output/prefix/name).read_bytes()).hexdigest(), expected)

    def test_force_added_outputs_fail_while_durable_inputs_pass(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            subprocess.run(['git', 'init', '-q', str(root)], check=True)
            (root/'.gitignore').write_text('build/\ndocs/html/\n')
            good = ['crates/core/src/lib.rs', 'tests/fixtures/owned.zip',
                    'design/hifi/screens/original.png', 'design/html/app.js',
                    'docs/qa/acceptance.md', 'docs/design/architecture-decisions.md']
            bad = ['build/test/report.md', 'docs/html/index.html', 'docs/qa/results.json',
                   'docs/qa/private-test-readiness.md', 'design/assets/preview/contact.png']
            for name in good + bad:
                path = root/name;path.parent.mkdir(parents=True, exist_ok=True);path.write_bytes(b'owned input')
            subprocess.run(['git', 'add', '-f', *good], cwd=root, check=True)
            self.assertEqual(violations(root), [])
            subprocess.run(['git', 'add', '-f', *bad], cwd=root, check=True)
            failures = violations(root)
            for name in bad:
                self.assertTrue(any(f.startswith(name+':') for f in failures), name)
            for name in good:
                self.assertFalse(any(f.startswith(name+':') for f in failures), name)

    def test_large_blob_is_checked_in_index_even_after_worktree_truncation(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            subprocess.run(['git', 'init', '-q', str(root)], check=True)
            path = root/'large.bin';path.write_bytes(b'x' * (MAX_BLOB_BYTES + 1))
            subprocess.run(['git', 'add', 'large.bin'], cwd=root, check=True)
            path.write_bytes(b'small working file')
            self.assertTrue(any('large.bin: blob exceeds' in f for f in violations(root)))


if __name__ == '__main__':
    unittest.main()
