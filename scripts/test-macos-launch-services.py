#!/usr/bin/env python3
"""Verify registration ownership boundaries and cleanup on failed validations."""
import plistlib
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import macos_launch_services as ls


class RegistrationTests(unittest.TestCase):
    def app(self, root, name, identity=ls.BUNDLE_ID):
        app = root/name
        (app/'Contents').mkdir(parents=True)
        (app/'Contents/Info.plist').write_bytes(plistlib.dumps({'CFBundleIdentifier': identity}))
        return app

    def test_dump_selects_exact_identity_and_handles_unicode_spaces(self):
        data = ('path: /tmp/中文 space/xdzip.app (0x123)\nidentifier: org.xdzip.desktop\n'
                '--------------------\npath: /tmp/other.app (0x124)\nidentifier: org.xdzip.desktop.other\n')
        with patch.object(ls.subprocess, 'check_output', return_value=data):
            self.assertEqual(ls.registered_apps(), {Path('/tmp/中文 space/xdzip.app')})

    def test_temporary_bundle_preserves_inputs_and_removes_failed_new_copy(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp).resolve();build = root/'build';build.mkdir()
            source = self.app(build, 'primary.app')
            original = (source/'Contents/Info.plist').read_bytes()
            target = build/'copy.app'
            with patch.object(ls, 'ROOT', root):
                with self.assertRaisesRegex(RuntimeError, 'probe failed'):
                    with ls.temporary_bundle_path(target):
                        self.app(build, 'copy.app')
                        raise RuntimeError('probe failed')
                self.assertFalse(target.exists())
                self.assertEqual((source/'Contents/Info.plist').read_bytes(), original)
                with ls.temporary_bundle_path(target):
                    self.app(build, 'copy.app')
                self.assertFalse(target.exists())

    def test_temporary_bundle_rejects_existing_and_outside_paths(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp).resolve();build = root/'build';build.mkdir()
            source = self.app(build, 'primary.app')
            external = self.app(root, 'external.app')
            alias = build/'alias.app';alias.symlink_to(external)
            (build/'parent').symlink_to(root)
            with patch.object(ls, 'ROOT', root):
                for target in [source, alias, root/'new.app', build/'parent/new.app', build/'copy']:
                    with self.subTest(target=target.name):
                        with self.assertRaises(ValueError):
                            with ls.temporary_bundle_path(target):
                                self.fail('Invalid temporary destination accepted')
                self.assertTrue(source.is_dir());self.assertTrue(external.is_dir())

    def test_temporary_bundle_does_not_follow_a_new_leaf_symlink(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp).resolve();build = root/'build';build.mkdir()
            external = self.app(root, 'external.app');target = build/'copy.app'
            with patch.object(ls, 'ROOT', root):
                with ls.temporary_bundle_path(target):
                    target.symlink_to(external)
            self.assertFalse(target.is_symlink());self.assertTrue(external.is_dir())

    def test_copies_cannot_escape_or_claim_another_products_bundle(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp).resolve();build = root/'build';build.mkdir()
            owned = self.app(build, 'xdzip.app')
            other = self.app(build, 'other.app', 'another.product')
            external = self.app(root, 'external.app')
            (build/'alias.app').symlink_to(external)
            self.assertTrue(ls.build_copy(owned, build))
            self.assertTrue(ls.build_copy(build/'gone.app', build))
            for path in [other, external, build/'alias.app', build/'../external.app']:
                self.assertFalse(ls.build_copy(path, build))

    def test_preview_has_no_mutations_and_keep_must_be_valid(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp).resolve();build = root/'build';build.mkdir()
            keep = self.app(build, 'primary.app');old = self.app(build, 'old.app')
            with patch.object(ls, 'ROOT', root), patch.object(ls, 'registered_apps', return_value={keep, old}), \
                 patch.object(ls, 'unregister') as unregister, patch.object(ls.subprocess, 'run') as run:
                result = ls.cleanup(keep)
                self.assertEqual(result['candidate_count'], 1)
                unregister.assert_not_called();run.assert_not_called()
                with self.assertRaises(OSError):
                    ls.cleanup(build/'missing.app', True)

    def test_apply_keeps_primary_and_does_not_touch_external_installations(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp).resolve();build = root/'build';build.mkdir()
            keep = self.app(build, 'primary.app');old = self.app(build, 'old.app')
            external = self.app(root, 'installed.app')
            with patch.object(ls, 'ROOT', root), \
                 patch.object(ls, 'registered_apps', side_effect=[{keep, old, external}, {keep, external}]), \
                 patch.object(ls, 'unregister') as unregister, patch.object(ls.subprocess, 'run') as run:
                run.return_value.returncode = 0
                result = ls.cleanup(keep, True)
                unregister.assert_called_once_with(old)
                self.assertEqual(result['outside_build_untouched'], 1)
                self.assertEqual(result['remaining_build_duplicates'], 0)
                self.assertTrue((old/'Contents/Info.plist').exists())

    def test_failed_validation_removes_only_new_owned_registration(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp).resolve();build = root/'build';build.mkdir()
            prior = self.app(build, 'prior.app');new = self.app(build, 'new.app')
            external = self.app(root, 'external.app')
            with patch.object(ls, 'ROOT', root), patch.object(ls.sys, 'platform', 'darwin'), \
                 patch.object(ls, 'registered_apps', side_effect=[{prior}, {prior, new, external}, {prior, external}]), \
                 patch.object(ls, 'unregister') as unregister:
                with self.assertRaisesRegex(RuntimeError, 'probe failed'):
                    with ls.transient_registrations(build):
                        raise RuntimeError('probe failed')
                unregister.assert_called_once_with(new)

    def test_cleanup_failure_does_not_claim_a_pass(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp).resolve();build = root/'build';build.mkdir()
            app = self.app(build, 'new.app')
            with patch.object(ls, 'ROOT', root), patch.object(ls.sys, 'platform', 'darwin'), \
                 patch.object(ls, 'registered_apps', return_value={app}), patch.object(ls.subprocess, 'run') as run:
                run.return_value.returncode = 0
                keep = self.app(root, 'primary.app')
                with self.assertRaisesRegex(RuntimeError, 'converge'):
                    ls.cleanup(keep, True)

    def test_transient_cleanup_checks_actual_removal(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp).resolve();build = root/'build';build.mkdir()
            app = self.app(build, 'new.app')
            with patch.object(ls, 'ROOT', root), patch.object(ls.sys, 'platform', 'darwin'), \
                 patch.object(ls, 'registered_apps', side_effect=[set(), {app}, {app}]), \
                 patch.object(ls, 'unregister'):
                with self.assertRaisesRegex(RuntimeError, 'converge'):
                    with ls.transient_registrations(build):
                        pass


if __name__ == '__main__':
    unittest.main()
