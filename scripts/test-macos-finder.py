#!/usr/bin/env python3
"""Owned fixtures for Finder bundle identity and signing order, no OS registration."""
import importlib.util
from pathlib import Path
import plistlib
import tempfile
import unittest
from unittest.mock import patch
from macos_finder import ENTITLEMENTS, sign_finder

spec = importlib.util.spec_from_file_location('macdmg', Path(__file__).with_name('build-macos-dmg.py'))
macdmg = importlib.util.module_from_spec(spec)
spec.loader.exec_module(macdmg)


class FinderSigningTests(unittest.TestCase):
    def fixture(self, root):
        app = root/'xdzip.app'
        contents = app/'Contents'
        contents.mkdir(parents=True)
        (contents/'Info.plist').write_bytes(plistlib.dumps({'XdzipFinderSyncProtocol': 1}))
        extension = contents/'PlugIns/xdzip-finder-sync.appex'
        relay = contents/'Helpers/xdzip-shell-relay.app'
        for bundle, identity, executable in (
                (extension, 'org.xdzip.finder-sync', 'xdzip-finder-sync'),
                (relay, 'org.xdzip.shell-relay', 'xdzip-shell-relay')):
            (bundle/'Contents/MacOS').mkdir(parents=True)
            (bundle/'Contents/Info.plist').write_bytes(plistlib.dumps({
                'CFBundleIdentifier': identity, 'CFBundleExecutable': executable,
                'NSExtension': {'NSExtensionPointIdentifier': 'com.apple.FinderSync'}}))
            (bundle/'Contents/MacOS'/executable).write_bytes(b'owned Mach-O fixture')
        (extension/'Contents/Resources').mkdir()
        entitlements = extension/'Contents/Resources/Finder.entitlements'
        entitlements.write_bytes(plistlib.dumps(ENTITLEMENTS))
        return app, extension, relay, entitlements

    def test_reapply_exact_entitlements_after_images_before_outer_seal(self):
        with tempfile.TemporaryDirectory() as tmp:
            app, extension, relay, entitlement = self.fixture(Path(tmp))
            commands = []
            with patch.object(macdmg, 'native', return_value=True), \
                    patch.object(macdmg, 'checked', side_effect=commands.append):
                macdmg.sign_local(app)
            extension_sign = next(i for i, c in enumerate(commands)
                                  if c[-1] == extension and '--entitlements' in c)
            for i, c in enumerate(commands):
                if c[-1] == extension/'Contents/MacOS/xdzip-finder-sync':
                    self.assertLess(i, extension_sign)
            self.assertEqual(commands[extension_sign][-3:], ['--entitlements', entitlement, extension])
            self.assertEqual(commands[-2][-1], app)
            self.assertEqual(commands[-1], ['codesign', '--verify', '--deep', '--strict', app])
            self.assertTrue(any(c[-1] == relay and '--sign' in c for c in commands))

    def test_unknown_identity_or_broader_entitlements_refuse_before_signing(self):
        with tempfile.TemporaryDirectory() as tmp:
            app, extension, relay, entitlement = self.fixture(Path(tmp))
            entitlement.write_bytes(plistlib.dumps({**ENTITLEMENTS, 'com.apple.security.network.client': True}))
            commands = []
            with self.assertRaisesRegex(ValueError, 'entitlements'):
                sign_finder(app, commands.append)
            self.assertEqual(commands, [])
            entitlement.write_bytes(plistlib.dumps(ENTITLEMENTS))
            metadata = relay/'Contents/Info.plist'
            metadata.write_bytes(plistlib.dumps({'CFBundleIdentifier': 'foreign'}))
            with self.assertRaisesRegex(ValueError, 'identity'):
                sign_finder(app, commands.append)
            self.assertEqual(commands, [])

    def test_old_bundle_skips_and_unknown_protocol_refuses(self):
        with tempfile.TemporaryDirectory() as tmp:
            app, _, _, _ = self.fixture(Path(tmp))
            info = app/'Contents/Info.plist'
            info.write_bytes(plistlib.dumps({}))
            commands = []
            sign_finder(app, commands.append)
            self.assertEqual(commands, [])
            info.write_bytes(plistlib.dumps({'XdzipFinderSyncProtocol': 2}))
            with self.assertRaisesRegex(ValueError, 'protocol'):
                sign_finder(app, commands.append)

    def test_entitlements_match_checked_in_native_contract(self):
        source = Path(__file__).resolve().parents[1]/'apps/desktop/shell/Finder.entitlements'
        self.assertEqual(plistlib.loads(source.read_bytes()), ENTITLEMENTS)
        paths = ENTITLEMENTS['com.apple.security.temporary-exception.files.home-relative-path.read-only']
        self.assertEqual(paths, ['/Library/Application Support/xdzip/finder-sync.json'])


if __name__ == '__main__':
    unittest.main()
