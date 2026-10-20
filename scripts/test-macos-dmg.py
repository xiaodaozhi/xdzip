#!/usr/bin/env python3
"""Fail-closed DMG verification tests without deployment, signing or mounting."""
from contextlib import nullcontext, redirect_stdout
import importlib.util
import io
import json
import os
from pathlib import Path
import plistlib
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch
import zipfile
import macos_launch_services as ls


spec = importlib.util.spec_from_file_location('verify_macos_dmg', Path(__file__).with_name('verify-macos-dmg.py'))
verify = importlib.util.module_from_spec(spec)
spec.loader.exec_module(verify)
IDENTITY = {'version': '0.1.0-dev.9', 'build': 9, 'revision': '0123456789ab', 'dirty': False}


class DmgVerificationTests(unittest.TestCase):
    def test_native_environment_discards_sdk_style_and_probe_overrides(self):
        inherited = {'QT_PLUGIN_PATH': '/private/sdk/plugins', 'QML_IMPORT_PATH': '/private/sdk/qml',
                     'QML2_IMPORT_PATH': '/private/sdk/qml', 'DYLD_FRAMEWORK_PATH': '/private/sdk/lib',
                     'QT_STYLE_OVERRIDE': 'Material', 'QT_QUICK_CONTROLS_STYLE': 'Material',
                     'QT_QUICK_BACKEND': 'software', 'QSG_RHI_BACKEND': 'opengl',
                     'XDZIP_PROBE_OPERATION': 'unrelated', 'XDZIP_EXPECT_QT_SOURCE_PROOF': 'unrelated'}
        with patch.dict(os.environ, inherited):
            env = verify.clean_environment(Path('/owned/output'))
        for key in inherited.keys()-{'QSG_RHI_BACKEND'}:
            self.assertNotIn(key, env)
        self.assertEqual(env['QSG_RHI_BACKEND'], 'metal')
        self.assertEqual(env['QT_QPA_PLATFORM'], 'cocoa')
        self.assertEqual(env['PATH'], '/usr/bin:/bin:/usr/sbin:/sbin')
        self.assertEqual(env['XDZIP_SETTINGS_PATH'], '/owned/output/isolated/settings.ini')

    def test_every_image_and_bundle_requires_strict_signature_success(self):
        app = Path('/owned/xdzip.app')
        images = [app/'Contents/MacOS/xdzip', app/'Contents/PlugIns/unreferenced.dylib']
        with patch.object(verify, 'mach_o_files', return_value=images), \
             patch.object(verify.macdmg, 'checked') as checked:
            report = verify.verify_signatures(app)
            self.assertEqual(report['native_binary_count'], 2)
            self.assertTrue(report['bundle_deep_strict'])
            self.assertEqual(checked.call_count, 3)
            for call in checked.call_args_list:
                self.assertIn('--strict', call.args[0])
                self.assertIn('--all-architectures', call.args[0])
            self.assertIn('--deep', checked.call_args.args[0])
            checked.side_effect = ValueError('codesign failed')
            with self.assertRaisesRegex(ValueError, 'codesign failed'):
                verify.verify_signatures(app)
        with patch.object(verify, 'mach_o_files', return_value=[]):
            with self.assertRaisesRegex(ValueError, 'No native code'):
                verify.verify_signatures(app)

    def test_dirty_or_unknown_identity_is_not_accepted(self):
        valid = 'xdzip 0.1.0-dev.9 (build 9, 0123456789ab)'
        with patch.object(verify.macdmg, 'checked', return_value=valid):
            self.assertEqual(verify.product_identity(Path('/owned/xdzip'), {})[1], IDENTITY)
        for text in [valid[:-1]+', dirty)', valid.replace('0123456789ab', 'unknown'), 'xdzip 0.1.0']:
            with patch.object(verify.macdmg, 'checked', return_value=text):
                with self.assertRaisesRegex(ValueError, 'clean verifiable'):
                    verify.product_identity(Path('/owned/xdzip'), {})

    def test_product_failure_preserves_sanitized_local_log(self):
        with tempfile.TemporaryDirectory() as temp:
            output = Path(temp)
            text = str(verify.ROOT)+'/cache/lib\n'+str(Path.home())+'/private/file'
            result = subprocess.CompletedProcess([], 3, '', text)
            with patch.object(verify.subprocess, 'run', return_value=result):
                with self.assertRaisesRegex(ValueError, 'owned-check failed'):
                    verify.run_product(Path('/owned/xdzip'), ['--smoke-test'], {}, output, 'owned-check')
            log = (output/'owned-check.log').read_text()
            self.assertNotIn(str(verify.ROOT), log)
            self.assertNotIn(str(Path.home()), log)
            self.assertIn('<workspace>', log)

    def probe(self, _binary, arguments, environment, _output, _label, **_):
        fixture = Path(arguments[-1])
        target = Path(environment['XDZIP_PROBE_EXTRACT_TARGET'])
        with zipfile.ZipFile(fixture) as archive:
            archive.extractall(target)
            records = len(archive.infolist())
            size = sum(item.file_size for item in archive.infolist())
        events = [{'event': 'first_frame', **IDENTITY, 'runtime_available': True,
                   'backend': 'metal', 'architecture': 'arm64'},
                  {'event': 'open_ready', 'records': records},
                  {'event': 'search_frame', 'rows': 2}, {'event': 'sort_frame'},
                  {'event': 'browse_finished'}, {'event': 'extract_requested'},
                  {'event': 'extract_finished', 'status': 'succeeded', 'committed_bytes': str(size)},
                  {'event': 'quit_requested'}]
        return events, target

    def test_extraction_requires_native_frames_and_independently_correct_bytes(self):
        for failure in [None, 'wrong-payload', 'extra-file', 'missing-directory', 'software', 'identity', 'missing-event']:
            with self.subTest(failure=failure), tempfile.TemporaryDirectory() as temp:
                output = Path(temp)
                def product(*args, **kwargs):
                    events, target = self.probe(*args, **kwargs)
                    if failure == 'wrong-payload': (target/'item-000.txt').write_bytes(b'corrupted')
                    elif failure == 'extra-file': (target/'unexpected').write_bytes(b'')
                    elif failure == 'missing-directory': (target/'empty-directory').rmdir()
                    elif failure == 'software': events[0]['backend'] = 'software'
                    elif failure == 'identity': events[0]['revision'] = 'a'*12
                    elif failure == 'missing-event': events.pop()
                    return ''.join('XDZIP_PROBE '+json.dumps(event)+'\n' for event in events)
                with patch.object(verify, 'run_product', side_effect=product):
                    if failure:
                        with self.assertRaises(ValueError):
                            verify.verify_extraction(Path('/owned/xdzip'), {}, output, IDENTITY)
                    else:
                        report = verify.verify_extraction(Path('/owned/xdzip'), {}, output, IDENTITY)
                        self.assertTrue(report['independent_byte_check'])
                        self.assertEqual(report['records'], 5)
                        self.assertEqual(report['files']['empty.txt']['bytes'], 0)
                        self.assertEqual(report['directories'], ['empty-directory', '目录'])

    def test_failed_architecture_style_or_signature_gate_removes_the_new_copy(self):
        for failure in ['optimization', 'signature']:
            with self.subTest(failure=failure), tempfile.TemporaryDirectory() as temp:
                root = Path(temp).resolve()
                mount = root/'mounted';source = mount/'xdzip.app'
                (source/'Contents').mkdir(parents=True)
                info = source/'Contents/Info.plist'
                info.write_bytes(plistlib.dumps({'CFBundleIdentifier': ls.BUNDLE_ID}))
                original = info.read_bytes()
                (mount/'Applications').symlink_to('/Applications')
                dmg = root/'input.dmg';dmg.write_bytes(b'owned DMG')
                output = root/'build/verification';copy = output/'中文 space/xdzip.app'
                arguments = ['verify-macos-dmg', '--dmg', str(dmg), '--output', str(output)]
                mounted = plistlib.dumps({'system-entities': [{'mount-point': str(mount)}]})
                with patch.object(ls, 'ROOT', root), patch.object(verify, 'ROOT', root), \
                     patch.object(verify, 'transient_registrations', side_effect=lambda _: nullcontext()), \
                     patch.object(verify.macdmg, 'validate_bundle', return_value=copy/'Contents/MacOS/xdzip'), \
                     patch.object(verify, 'validate_optimized_bundle', return_value={'status': 'passed'}) as optimized, \
                     patch.object(verify, 'verify_signatures', return_value={'status': 'passed'}) as signed, \
                     patch.object(verify, 'audit') as audit, patch.object(verify, 'run_product') as product, \
                     patch.object(verify.subprocess, 'check_output', return_value=mounted), \
                     patch.object(verify.subprocess, 'run'), patch.object(sys, 'argv', arguments), \
                     redirect_stdout(io.StringIO()):
                    (optimized if failure == 'optimization' else signed).side_effect = ValueError('strict gate failed')
                    with self.assertRaisesRegex(ValueError, 'strict gate failed'):
                        verify.main()
                    audit.assert_not_called();product.assert_not_called()
                self.assertFalse(copy.exists())
                self.assertFalse((output/'verification.json').exists())
                self.assertEqual(info.read_bytes(), original)
                self.assertEqual(dmg.read_bytes(), b'owned DMG')


if __name__ == '__main__':
    unittest.main()
