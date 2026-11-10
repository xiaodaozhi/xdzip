#!/usr/bin/env python3
"""Exercise application-bundle comparisons using small, owned fixtures."""
import importlib.util
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch


spec = importlib.util.spec_from_file_location(
    'verify_macos_app_zip', Path(__file__).with_name('verify-macos-app-zip.py'))
verify = importlib.util.module_from_spec(spec)
spec.loader.exec_module(verify)


@unittest.skipUnless(os.name == 'posix', 'POSIX symlink and mode semantics required')
class BundleComparisonTests(unittest.TestCase):
    def test_disposable_extraction_is_removed_on_failure_and_never_reuses_a_directory(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp).resolve()
            (root/'build').mkdir()
            output = root/'build/output'
            with patch.object(verify, 'ROOT', root):
                with self.assertRaisesRegex(RuntimeError, 'fixture failure'):
                    with verify.temporary_extraction_directory(output):
                        (output/'wrong.app').mkdir()
                        raise RuntimeError('fixture failure')
                self.assertFalse(output.exists())
                output.mkdir()
                with self.assertRaises(ValueError):
                    with verify.temporary_extraction_directory(output):
                        pass
                self.assertTrue(output.is_dir())

    def create_bundle(self, root):
        app = root/'xdzip.app'
        executable = app/'Contents/MacOS/xdzip'
        executable.parent.mkdir(parents=True)
        executable.write_bytes(b'executable')
        executable.chmod(0o755)
        resources = app/'Contents/Resources'
        resources.mkdir()
        (resources/'data').write_bytes(b'signed resource')
        (resources/'data').chmod(0o644)
        (app/'Contents/alias').symlink_to('Resources/data')
        return app

    def test_matches_payloads_links_and_safe_file_modes(self):
        for failure in (None, 'content', 'link', 'extra', 'missing', 'mode', 'root-link'):
            with self.subTest(failure=failure), tempfile.TemporaryDirectory() as temp:
                root = Path(temp)
                reference = self.create_bundle(root/'reference')
                actual = self.create_bundle(root/'actual')
                if failure == 'content':
                    (actual/'Contents/Resources/data').write_bytes(b'altered')
                elif failure == 'link':
                    link = actual/'Contents/alias'
                    link.unlink()
                    link.symlink_to('MacOS/xdzip')
                elif failure == 'extra':
                    (actual/'Contents/Resources/extra').write_bytes(b'extra')
                elif failure == 'missing':
                    (actual/'Contents/Resources/data').unlink()
                elif failure == 'mode':
                    (actual/'Contents/MacOS/xdzip').chmod(0o700)
                elif failure == 'root-link':
                    actual.rename(root/'actual/real.app')
                    actual.symlink_to('real.app')
                if failure:
                    with self.assertRaises(ValueError):
                        verify.compare_bundles(reference, actual)
                else:
                    self.assertEqual(verify.compare_bundles(reference, actual), 6)


if __name__ == '__main__':
    unittest.main()
