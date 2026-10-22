# scripts/test-sparkle-bundle.py

"""Exercise Mac packaging boundaries without claiming native updater execution."""
import copy
import hashlib
import json
import os
from pathlib import Path
import plistlib
import runpy
import tempfile
import unittest
from unittest.mock import patch

import sparkle_bundle as sparkle

image = runpy.run_path(str(Path(__file__).with_name('test-macos-attribution.py')))['image']


class SparkleBundleTests(unittest.TestCase):
    def setUp(self):
        temporary_root = sparkle.ROOT/'build/sparkle-tests'
        temporary_root.mkdir(parents=True, exist_ok=True)
        temporary = tempfile.TemporaryDirectory(dir=temporary_root)
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.sdk = self.root/'sdk'
        self.app = self.root/'xdzip.app'
        self.spec = copy.deepcopy(sparkle.configuration())
        self.spec['license_sha256'] = hashlib.sha256(b'fixture legal notice').hexdigest()
        self.addCleanup(patch.stopall)
        patch('sparkle_bundle.configuration', return_value=self.spec).start()
        self.write(self.sdk/'LICENSE', b'fixture legal notice')
        framework = self.sdk/'Sparkle.framework/Versions/B'
        self.write(framework/'Resources/Info.plist', plistlib.dumps({'CFBundleShortVersionString': '2.10.0'}))
        self.write(framework/'Headers/SPUUpdater.h', b'fixture SDK header')
        for relative in ('Sparkle', 'Autoupdate', 'Updater.app/Contents/MacOS/Updater',
                         'XPCServices/Downloader.xpc/Contents/MacOS/Downloader'):
            self.write(framework/relative, image())
        self.write(self.app/'Contents/Info.plist', plistlib.dumps({'SUPublicEDKey': 'fixture-only-public-key'}))

    def write(self, path, data):
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)

    def test_deployment_identity_and_unused_service_removal(self):
        framework = sparkle.embed(self.sdk, self.app, installable=True)
        info = plistlib.loads((self.app/'Contents/Info.plist').read_bytes())
        self.assertTrue(info['XdzipUpdateInstallable'])
        self.assertEqual(info['XdzipUpdateInstallId'], sparkle.INSTALL_ID)
        self.assertFalse((framework/'Versions/B/XPCServices').exists())
        self.assertTrue((framework/'Versions/B/Autoupdate').is_file())
        self.assertTrue((self.sdk/'Sparkle.framework/Versions/B/XPCServices').is_dir())
        sparkle.embed(self.sdk, self.app, installable=False)
        self.assertFalse(plistlib.loads((self.app/'Contents/Info.plist').read_bytes())['XdzipUpdateInstallable'])

    def test_missing_public_key_cannot_create_installable_marker(self):
        self.write(self.app/'Contents/Info.plist', plistlib.dumps({}))
        sparkle.embed(self.sdk, self.app, installable=True)
        self.assertFalse(plistlib.loads((self.app/'Contents/Info.plist').read_bytes())['XdzipUpdateInstallable'])

    def test_notice_identity_is_pinned(self):
        files = sparkle.write_notice(self.sdk, self.root/'notices')
        self.assertEqual(files[0].read_bytes(), b'fixture legal notice')
        self.write(self.sdk/'LICENSE', b'changed notice')
        with self.assertRaisesRegex(ValueError, 'identity differs'):
            sparkle.write_notice(self.sdk, self.root/'other')

    def test_helper_runtime_flags_and_entitlements_are_separate(self):
        framework = sparkle.embed(self.sdk, self.app, installable=False)
        entitlements = {'com.apple.security.cs.disable-library-validation': True}
        metadata = sparkle.signing_metadata(framework, lambda command: plistlib.dumps(entitlements).decode())
        calls = []
        sparkle.sign_helpers(framework, metadata, lambda command: calls.append(command), self.root)
        self.assertEqual(len(calls), 4)
        for command in calls[:3]:
            self.assertIn('runtime', command)
            self.assertNotIn('--deep', command)
        self.assertEqual(plistlib.loads((self.root/'sparkle-helper-0.entitlements').read_bytes()), entitlements)
        with self.assertRaisesRegex(ValueError, 'debugger'):
            sparkle.signing_metadata(framework, lambda command: plistlib.dumps({'com.apple.security.get-task-allow': True}).decode())

    def test_native_attribution_rejects_changed_code(self):
        sparkle.embed(self.sdk, self.app, installable=False)
        relative = sparkle.PREFIX+'Versions/B/Sparkle'
        path = self.app/relative
        report = {'files': [], 'components': [], 'dependencies': [],
                  'gaps': [{'kind': 'native-outside-qt-attribution', 'file': relative}]}
        def audit():
            return {'files': [{'path': relative, 'sha256': sparkle.checksum(path)}]}
        sparkle.attribute(report, self.app, self.sdk, audit())
        self.assertEqual(report['gaps'], [])
        self.assertEqual(report['files'][0]['owners'], ['sparkle:Sparkle@2.10.0'])
        self.assertNotIn(str(self.sdk), json.dumps(report))
        path.write_bytes(image(b'changed executable'))
        with self.assertRaisesRegex(ValueError, 'differ'):
            sparkle.attribute(copy.deepcopy(report), self.app, self.sdk, audit())

    def test_signing_uses_final_asset_and_fails_on_missing_package(self):
        with patch.dict(os.environ, {'XDZIP_SPARKLE_SIGNING_KEY_HEX': ''}):
            self.assertFalse(sparkle.sign_staged_archive(self.root, '0.1.0-alpha.7', 20))
        with patch.dict(os.environ, {'XDZIP_SPARKLE_SIGNING_KEY_HEX': 'fixture-only'}):
            with self.assertRaisesRegex(ValueError, 'missing'):
                sparkle.sign_staged_archive(self.root, '0.1.0-alpha.7', 20)
            package = self.root/'xdzip-0.1.0-alpha.7-macos-arm64.app.zip'
            package.write_bytes(b'fixture package')
            calls = []
            def signer(arguments, tool):
                calls.append(arguments)
                if arguments[0] == 'sign-archive':
                    (self.root/'macos-update.json').write_text(json.dumps({'ed_signature': 'fixture-signature'}))
            with patch('release_updates.run_tool', side_effect=signer):
                self.assertTrue(sparkle.sign_staged_archive(self.root, '0.1.0-alpha.7', 20))
            self.assertEqual(calls[0][calls[0].index('--archive')+1], str(package))
            self.assertEqual(calls[1][0], 'verify-archive')
            self.assertNotIn('fixture-only', repr(calls))


if __name__ == '__main__':
    unittest.main()
