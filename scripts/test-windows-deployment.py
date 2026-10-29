#!/usr/bin/env python3
"""Windows deployment policy regressions using owned fixtures, no native execution."""
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch
import deploy_windows as deployment
import deployment_audit as audit
from installer_payload import HISTORICAL_PRODUCT_REVISION
import verify_windows_runtime as verification


class WindowsDeploymentTests(unittest.TestCase):
    def test_helper_omission_is_limited_to_the_clean_pinned_historical_product(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); (root/'xdzip.exe').write_bytes(b'MZowned')
            pin = HISTORICAL_PRODUCT_REVISION
            cli = f'xdzip 0.1.0-dev.1 (build 1, {pin[:12]})\n'
            with patch.dict(os.environ, {'CI': 'true'}), patch.object(deployment, 'run', return_value=cli) as run:
                with self.assertRaisesRegex(ValueError, 'requires the standalone'):
                    deployment.update_helper_deployment(root)
                run.assert_not_called()
                record = deployment.update_helper_deployment(root, pin)
                self.assertIsNone(record['update_helper_imports'])
                self.assertEqual(record['historical_product_qa']['revision'], pin)
                self.assertIs(record['historical_product_qa']['update_helper_in_historical_source'], False)
                run.assert_called_once_with(str((root/'xdzip.exe').resolve()), '--version')
            with patch.dict(os.environ, {'CI': 'false'}), patch.object(deployment, 'run') as run:
                with self.assertRaisesRegex(ValueError, 'disposable CI'):
                    deployment.update_helper_deployment(root, pin)
                run.assert_not_called()
            with patch.dict(os.environ, {'CI': 'true'}):
                for invalid in [pin[:12], 'a'*40, '']:
                    with self.subTest(revision=invalid), patch.object(deployment, 'run') as run:
                        with self.assertRaisesRegex(ValueError, 'fixed full revision'):
                            deployment.update_helper_deployment(root, invalid)
                        run.assert_not_called()
                for invalid in [cli.replace(')', ', dirty)'), cli.replace('build 1', 'build 2'),
                                cli.replace(pin[:12], 'a'*12), cli.replace('dev.1', 'alpha.1')]:
                    with self.subTest(identity=invalid), patch.object(deployment, 'run', return_value=invalid):
                        with self.assertRaisesRegex(ValueError, 'pinned clean'):
                            deployment.update_helper_deployment(root, pin)
                (root/'xdzip-update-helper.exe').write_bytes(b'MZowned helper')
                with self.assertRaisesRegex(ValueError, 'must not be supplied a newer'):
                    deployment.update_helper_deployment(root, pin)
                with patch.object(deployment, 'run', return_value='    KERNEL32.dll\n'):
                    self.assertEqual(deployment.update_helper_deployment(root), {'update_helper_imports': ['KERNEL32.dll']})

    def test_update_helper_does_not_depend_on_the_replaced_runtime(self):
        with patch.object(deployment,'run',return_value='    KERNEL32.dll\n    api-ms-win-core-synch-l1-2-0.dll\n'):
            self.assertEqual(len(deployment.standalone_update_helper(Path('owned.exe'))),2)
        for name in ['Qt6Core.dll','VCRUNTIME140.dll','MSVCP140.dll','third-party.dll']:
            with self.subTest(import_name=name), patch.object(deployment,'run',return_value='    '+name+'\n'):
                with self.assertRaisesRegex(ValueError,'static CRT'):
                    deployment.standalone_update_helper(Path('owned.exe'))

    def test_graphics_use_explicit_redist_with_notices_and_never_path(self):
        with tempfile.TemporaryDirectory() as temp:
            base = Path(temp); sdk = base/'sdk'; root = base/'product'
            root.mkdir()
            for name in deployment.FALLBACKS:
                path = sdk/'Redist/D3D/x64'/name
                path.parent.mkdir(parents=True, exist_ok=True); path.write_bytes(b'owned SDK '+name.encode())
            license_file = sdk/'Licenses/10.0.1.0/sdk_license.rtf'
            license_file.parent.mkdir(parents=True); license_file.write_bytes(b'owned license')
            result = deployment.copy_graphics(sdk, '10.0.1.0', root)
            self.assertEqual({item['file'] for item in result['files']}, set(deployment.FALLBACKS))
            self.assertTrue(all(item['supplier'] == 'Microsoft Windows SDK REDIST' for item in result['files']))
            self.assertFalse((root/'opengl32sw.dll').exists())
            self.assertTrue((root/'share/notices/Windows-SDK-graphics/sdk_license.rtf').is_file())
            self.assertNotIn(str(base), json.dumps(result))
            (sdk/'Redist/D3D/x64/dxil.dll').unlink()
            with self.assertRaisesRegex(ValueError, 'Missing bounded'):
                deployment.copy_graphics(sdk, '10.0.1.0', root)
            with self.assertRaisesRegex(ValueError, 'Explicit Windows SDK'):
                deployment.copy_graphics(sdk, '', root)

    def test_incomplete_sdk_graphics_are_rejected_before_copy(self):
        with tempfile.TemporaryDirectory() as temp:
            base = Path(temp); sdk = base/'sdk'; root = base/'product'; root.mkdir()
            path = sdk/'Redist/D3D/x64'/deployment.FALLBACKS[0]
            path.parent.mkdir(parents=True); path.write_bytes(b'owned SDK')
            (sdk/'license.rtf').write_bytes(b'owned license')
            with self.assertRaisesRegex(ValueError, 'Missing bounded'):
                deployment.copy_graphics(sdk, '10.0.1.0', root)
            self.assertEqual(list(root.iterdir()), [])

    def test_deploy_skips_and_removes_software_opengl(self):
        with tempfile.TemporaryDirectory() as temp:
            base = Path(temp); root = base/'build/runtime'; root.mkdir(parents=True)
            (base/'scripts').mkdir()
            (base/'scripts/windows-runtime.json').write_text('{}')
            (root/'xdzip.exe').write_bytes(b'owned')
            (root/'share').mkdir()
            for relative in deployment.FALLBACKS + ('styles/qmodernwindowsstyle.dll',
                    'platforms/qwindows.dll', 'qml/QtQuick/Controls/Basic/qmldir',
                    'translations/qt_zh_CN.qm'):
                path = root/relative; path.parent.mkdir(parents=True, exist_ok=True); path.write_bytes(b'owned')
            (root/'opengl32sw.dll').write_bytes(b'leftover from earlier deployment')
            arguments = ['deploy_windows.py', '--root', str(root), '--qt-prefix', str(base/'qt'),
                         '--qml-dir', str(base/'qml'), '--qml-import', str(base/'imports'),
                         '--runtime', str(base/'runtime-input')]
            with patch.object(deployment, 'ROOT', base), patch.object(sys, 'argv', arguments), \
                 patch.dict(os.environ, {'WindowsSdkDir': str(base/'sdk'), 'WindowsSDKVersion': '10.0.1.0'}), \
                 patch.object(deployment.subprocess, 'run') as deploy, \
                 patch.object(deployment, 'copy_runtime'), patch.object(deployment, 'copy_graphics'), \
                 patch.object(deployment, 'update_helper_deployment', return_value={}):
                deployment.main()
            self.assertIn('--no-opengl-sw', deploy.call_args.args[0])
            self.assertFalse((root/'opengl32sw.dll').exists())
            self.assertNotIn('opengl32sw.dll', json.loads((root/'share/deployment.json').read_text())['graphics_fallbacks'])

    def test_nested_software_opengl_is_rejected(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); nested = root/'nested/OpenGL32SW.dll'
            nested.parent.mkdir(); nested.write_bytes(b'owned')
            with self.assertRaisesRegex(ValueError, 'must not be present'):
                deployment.require_no_software_opengl(root)

    def test_renderer_requires_selected_software_adapter_for_warp(self):
        prefix = ('qt.scenegraph.general: Creating QRhi with backend D3D11 for window\n'
                  'qt.scenegraph.general: Created QRhi for window\n')
        hardware = ("qt.rhi.general: Adapter 0: 'hardware' (vendor 0x8086 device 0x1234 flags 0x0)\n"
                    'qt.rhi.general:   using this adapter\n')
        warp = ("qt.rhi.general: Adapter 1: 'Microsoft Basic Render Driver' "
                '(vendor 0x1414 device 0x8C flags 0x2)\nqt.rhi.general:   using this adapter\n')
        self.assertFalse(verification.renderer_observation(prefix+hardware, False)['software_device'])
        self.assertTrue(verification.renderer_observation(prefix+hardware+warp, True)['software_device'])
        with self.assertRaisesRegex(ValueError, 'WARP'):
            verification.renderer_observation(prefix+hardware, True)
        with self.assertRaisesRegex(ValueError, 'selected adapter'):
            verification.renderer_observation(prefix, False)
        with self.assertRaisesRegex(ValueError, 'initialization'):
            verification.renderer_observation(warp, True)

    def test_renderer_environment_clears_inherited_backend_overrides(self):
        inherited = {'PATH': 'system', 'QT_QUICK_BACKEND': 'software',
                     'QSG_RHI_BACKEND': 'opengl', 'QT_D3D_ADAPTER_INDEX': '0',
                     'QSG_RHI_PREFER_SOFTWARE_RENDERER': '1', 'XDZIP_ASSOCIATION_PROBE': '1'}
        default = verification.renderer_environment(inherited, Path('runtime'), Path('output'), False)
        for key in ('QT_QUICK_BACKEND', 'QSG_RHI_BACKEND', 'QT_D3D_ADAPTER_INDEX',
                    'QSG_RHI_PREFER_SOFTWARE_RENDERER', 'XDZIP_ASSOCIATION_PROBE'):
            self.assertNotIn(key, default)
        self.assertEqual(default['PATH'], 'system')
        warp = verification.renderer_environment(inherited, Path('runtime'), Path('output'), True)
        self.assertEqual(warp['QSG_RHI_PREFER_SOFTWARE_RENDERER'], '1')

    def test_prune_preserves_basic_widgets_translations_and_graphics(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            removed = [p+'/owned.qml' for p in deployment.MODULES] + list(deployment.LIBRARIES) + ['qml/core.qmltypes']
            kept = list(deployment.FALLBACKS) + ['styles/qmodernwindowsstyle.dll',
                'qml/QtQuick/Controls/Basic/qmldir', 'qml/QtQuick/Controls/qmldir',
                'Qt6Widgets.dll', 'translations/qt_zh_CN.qm']
            for relative in removed+kept:
                path = root/relative; path.parent.mkdir(parents=True, exist_ok=True); path.write_bytes(b'owned')
            records = deployment.prune(root)
            self.assertEqual({p['path'] for p in records}, set(removed))
            self.assertTrue(all((root/p).is_file() for p in kept))
            self.assertEqual(deployment.prune(root), [])

    def test_crt_cannot_be_satisfied_by_host_system32(self):
        with tempfile.TemporaryDirectory() as temp:
            base = Path(temp)
            (base/'intermediate').mkdir()
            root = base/'intermediate/../product'; root.mkdir()
            app = root/'xdzip.exe'; app.write_bytes(b'MZowned')
            system = base/'Windows/System32'; system.mkdir(parents=True)
            (system/'msvcp140.dll').write_bytes(b'MZhost')
            def imports(*command):
                return '    MSVCP140.dll\n' if Path(command[-1]).resolve() == app.resolve() else ''
            with patch.dict(os.environ, {'SystemRoot': str(base/'Windows')}), patch.object(audit, 'run', side_effect=imports):
                report = audit.audit(root, app, None, 'windows', 'deployed')
                self.assertEqual(report['status'], 'failed')
                self.assertEqual(report['unresolved_imports'], 1)
                (root/'msvcp140.dll').write_bytes(b'MZlocal')
                report = audit.audit(root, app, None, 'windows', 'deployed')
                self.assertEqual(report['status'], 'passed')
                self.assertTrue(all(e['scope'] == 'artifact' for e in report['binary_imports']))

    def test_runtime_copy_rejects_tampering_before_writing(self):
        with tempfile.TemporaryDirectory() as temp:
            base = Path(temp); runtime = base/'runtime'; runtime.mkdir(); root = base/'product'; root.mkdir()
            config = json.loads((deployment.ROOT/'scripts/windows-runtime.json').read_text())
            for name in list(config['files'])+['Microsoft-VC-runtime-license.rtf']:
                (runtime/name).write_bytes(b'tampered')
            with self.assertRaisesRegex(ValueError, 'hash mismatch'):
                deployment.copy_runtime(runtime, root)
            self.assertEqual(list(root.iterdir()), [])


if __name__ == '__main__':
    unittest.main()
