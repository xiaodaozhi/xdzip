#!/usr/bin/env python3
"""Fail-closed source patching, SDK identity and release provenance contracts."""
import copy
import contextlib
import hashlib
import io
import json
import importlib.util
import os
from pathlib import Path, PureWindowsPath
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import qt_network_security as security
import qt_network_materials as materials


class SecurityBuildTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.config, self.source, self.inputs = security.configuration()

    def record(self):
        return dict(schema_version=1, variant=self.config['variant'], inputs_sha256=self.inputs,
                    source_sha256=self.source['sha256'], patches=self.config['patches'],
                    core_sha256='1'*64, network_sha256='2'*64, features={'http':'1','ssl':'1'},
                    core_features={'system_zlib':'1'},
                    verification={'status':'passed','network_sha256':'2'*64})

    def test_official_patch_identity_and_order_are_pinned(self):
        self.assertEqual(len(self.config['patches']), 5)
        self.assertIn('116337-qtbase-6.8-0002', self.config['patches'][0]['file'])
        self.assertIn('116337-qtbase-6.8-0001', self.config['patches'][1]['file'])
        self.assertEqual(self.source['module'], 'qtbase')
        self.assertEqual(len(self.inputs), 64)

    def test_exact_patch_rejects_ambiguous_reapplied_and_out_of_scope_inputs(self):
        change = b'diff --git a/src/network/test.cpp b/src/network/test.cpp\n--- a/src/network/test.cpp\n+++ b/src/network/test.cpp\n@@ -1,2 +1,2 @@\n context\n-old\n+new\n'
        before = b'context\nold\n'
        files = {'src/network/test.cpp': before}
        security.apply_patch_files(files, change)
        self.assertEqual(files['src/network/test.cpp'], b'context\nnew\n')
        for invalid_files, invalid_patch in [
            (files, change), ({'src/network/test.cpp': before*2}, change),
            ({'src/network/test.cpp': before}, change.replace(b'-1,2', b'-1,3')),
            ({'src/corelib/test.cpp': before}, change.replace(b'src/network/', b'src/corelib/')),
            ({'src/network/test.cpp': before}, change.replace(b'src/network/test.cpp', b'src/network/../test.cpp')),
        ]:
            with self.subTest(patch=invalid_patch[:60]), self.assertRaises(ValueError):
                security.apply_patch_files(dict(invalid_files), invalid_patch)

    def test_receipt_binds_inputs_features_core_network_and_executed_probe(self):
        path = self.root/security.RECEIPT
        original = self.record()
        with patch.object(security, 'library', side_effect=lambda _, module: module), \
                patch.object(security, 'digest', side_effect=lambda module: {'Core':'1'*64,'Network':'2'*64}[module]), \
                patch.object(security, 'features', return_value=original['features']), \
                patch.object(security, 'core_features', return_value=original['core_features']), \
                patch.object(security, 'configuration', return_value=(self.config, self.source, self.inputs)), \
                patch.object(security.sys, 'platform', 'linux'):
            path.write_text(json.dumps(original))
            self.assertEqual(security.identity(self.root), original)
            for key in ('variant', 'inputs_sha256', 'source_sha256', 'core_sha256', 'network_sha256',
                        'features', 'core_features', 'patches', 'verification'):
                data = copy.deepcopy(original); data[key] = None
                path.write_text(json.dumps(data))
                with self.subTest(key=key), self.assertRaises(ValueError):
                    security.identity(self.root)
            data = copy.deepcopy(original); data['verification']['network_sha256'] = '3'*64
            path.write_text(json.dumps(data))
            with self.assertRaises(ValueError):
                security.identity(self.root)

    def test_missing_or_symlink_receipt_cannot_approve_an_sdk(self):
        self.assertIsNone(security.identity(self.root, required=False))
        with self.assertRaisesRegex(ValueError, 'Prepare'):
            security.identity(self.root)
        original = self.root/'owned.json'; original.write_text(json.dumps(self.record()))
        try:
            (self.root/security.RECEIPT).symlink_to(original)
        except OSError:
            self.skipTest('This host does not permit test symlinks')
        with self.assertRaisesRegex(ValueError, 'regular file'):
            security.identity(self.root)

    def test_native_commands_never_receive_production_signing_secrets(self):
        with patch.dict(os.environ, {'XDZIP_UPDATE_SIGNING_KEY_HEX':'owned-test-only',
                                    'XDZIP_SPARKLE_SIGNING_KEY_HEX':'owned-test-only'}), \
                patch.object(security.subprocess, 'run', return_value=subprocess.CompletedProcess([], 0, b'ok', b'')) as run:
            security.run(['owned-compiler'], self.root/'command.log')
        self.assertNotIn('XDZIP_UPDATE_SIGNING_KEY_HEX', run.call_args.kwargs['env'])
        self.assertNotIn('XDZIP_SPARKLE_SIGNING_KEY_HEX', run.call_args.kwargs['env'])

    def test_macos_sdk_tool_replacement_invalidates_receipt(self):
        original = self.record()
        original['sdk_tools'] = []
        actual_digest = security.digest
        for relative in security.MACOS_NETWORK_TOOLS:
            tool = self.root/relative; tool.parent.mkdir(parents=True, exist_ok=True)
            tool.write_bytes(b'owned sealed SDK tool')
            original['sdk_tools'].append(dict(sdk_path=relative, sha256=actual_digest(tool)))
        path = self.root/security.RECEIPT; path.write_text(json.dumps(original))
        def digest(value):
            return {'Core':'1'*64,'Network':'2'*64}[value] if isinstance(value, str) else actual_digest(value)
        with patch.object(security, 'library', side_effect=lambda _, module: module), \
                patch.object(security, 'digest', side_effect=digest), \
                patch.object(security, 'features', return_value=original['features']), \
                patch.object(security, 'core_features', return_value=original['core_features']), \
                patch.object(security, 'configuration', return_value=(self.config, self.source, self.inputs)), \
                patch.object(security.sys, 'platform', 'darwin'):
            security.identity(self.root)
            (self.root/security.MACOS_NETWORK_TOOLS[0]).write_bytes(b'replaced tool')
            with self.assertRaisesRegex(ValueError, 'tool changed'):
                security.identity(self.root)

    def test_authentication_failure_log_is_sanitized_before_raising(self):
        spec = importlib.util.spec_from_file_location('network_check', security.ROOT/'scripts/check-qt-network.py')
        check = importlib.util.module_from_spec(spec); spec.loader.exec_module(check)
        output = self.root/'verification'
        def failed(*args, **kwargs):
            (output/'authentication.log').write_text('FAIL! Loc: ['+str(security.ROOT/'build/owned.cpp')+'(12)]')
            return subprocess.CompletedProcess([], 1, b'', b'')
        with patch.object(check, 'run'), patch.object(check.subprocess, 'run', side_effect=failed):
            with self.assertRaisesRegex(ValueError, 'authentication regression failed'):
                check.check(self.root/'sdk', self.root/'source', output)
        self.assertNotIn(str(security.ROOT), (output/'authentication.log').read_text())
        self.assertIn('<workspace>', (output/'authentication.log').read_text())

    def test_windows_paths_reach_cmake_without_backslash_escapes(self):
        spec = importlib.util.spec_from_file_location('network_check_paths', security.ROOT/'scripts/check-qt-network.py')
        check = importlib.util.module_from_spec(spec); spec.loader.exec_module(check)
        with patch.object(check, 'run', side_effect=ValueError('captured configure')) as run:
            with self.assertRaisesRegex(ValueError, 'captured configure'):
                check.check(self.root/'sdk', PureWindowsPath('C:/a/owned/qtbase'), self.root/'probe-output')
        self.assertIn('-DQTBASE_SOURCE:PATH=C:/a/owned/qtbase', run.call_args.args[0])

    def test_failed_commands_redact_both_windows_path_forms(self):
        raw = b'C:\\Users\\owned\\xdzip\\src.cpp C:/Users/owned/xdzip/src.cpp C:/Users/owned/tmp'
        with patch.object(security, 'ROOT', PureWindowsPath('C:/Users/owned/xdzip')), \
                patch.object(security.Path, 'home', return_value=PureWindowsPath('C:/Users/owned')), \
                patch.object(security.subprocess, 'run', return_value=subprocess.CompletedProcess([], 1, raw, b'')), \
                contextlib.redirect_stderr(io.StringIO()) as console:
            with self.assertRaisesRegex(ValueError, 'command failed'):
                security.run(['owned-command'], self.root/'failure.log')
        for content in [(self.root/'failure.log').read_text(), console.getvalue()]:
            self.assertNotIn('owned', content)
            self.assertIn('<workspace>', content)
            self.assertIn('<user>', content)

    def test_abi_rejects_removed_exports_and_new_unavailable_core_imports(self):
        sdk, candidate = self.root/'sdk', self.root/'candidate'
        def library(prefix, module):
            return (prefix.name, module)
        def symbols(key, *, undefined=False):
            if undefined:
                return {'required-core'}
            if key == ('sdk', 'Core'):
                return {'original-core'}
            if key == ('candidate', 'Core'):
                return {'required-core'}
            return {'network-export'}
        with patch.object(security, 'library', side_effect=library), patch.object(security, 'symbol_names', side_effect=symbols):
            with self.assertRaisesRegex(ValueError, 'missing_core'):
                security.check_abi(sdk, candidate)
        with patch.object(security, 'library', side_effect=library), \
                patch.object(security, 'symbol_names', side_effect=[{'old'}, set(), set(), set(), set()]):
            with self.assertRaisesRegex(ValueError, 'missing_network'):
                security.check_abi(sdk, candidate)

    def package(self):
        original = self.record()
        with patch.object(materials, 'identity', return_value=original):
            materials.collect(self.root/'sdk', self.root/'notices')
        audit = dict(files=[{'path':'lib/libQt6Network.so.6','sha256':'3'*64}], external_files=[])
        attribution = dict(network_security=original,
            audit_sha256=hashlib.sha256(json.dumps(audit, sort_keys=True, separators=(',', ':')).encode()).hexdigest(),
            files=[dict(path='artifact/lib/libQt6Network.so.6', sha256='3'*64, sdk_sha256='2'*64,
                        network_security_variant=self.config['variant'], evidence='deployment-copy-and-rewrite-record')])
        (self.root/'deployment-audit.json').write_text(json.dumps(audit))
        (self.root/'runtime-attribution.json').write_text(json.dumps(attribution))
        return original, attribution

    def test_release_requires_patches_and_actual_deployed_provenance(self):
        original, attribution = self.package()
        self.assertEqual(materials.verify_package(self.root), original)
        for field, bad in [('sdk_sha256', '4'*64), ('sha256', '4'*64), ('evidence', 'filename-only-unverified-rewrite')]:
            changed = copy.deepcopy(attribution); changed['files'][0][field] = bad
            (self.root/'runtime-attribution.json').write_text(json.dumps(changed))
            with self.subTest(field=field), self.assertRaises(ValueError):
                materials.verify_package(self.root)
        (self.root/'runtime-attribution.json').write_text(json.dumps(attribution))
        patch_file = self.root/'notices/Qt-network-security'/Path(self.config['patches'][0]['file']).name
        patch_file.write_text('changed')
        with self.assertRaisesRegex(ValueError, 'patch missing or changed'):
            materials.verify_package(self.root)


if __name__ == '__main__':
    unittest.main()
