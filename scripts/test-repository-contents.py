#!/usr/bin/env python3
"""Use real staging, including forced ignored files, to enforce content boundaries."""
from pathlib import Path
import subprocess
import tempfile
import unittest
from repository_contents import MAX_BLOB_BYTES, violations


class ContentsTests(unittest.TestCase):
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
