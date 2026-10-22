#!/usr/bin/env python3
"""Windows deployment policy regressions using owned fixtures, no native execution."""
import json
import hashlib
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import deploy_windows as deployment
import deployment_audit as audit


class WindowsDeploymentTests(unittest.TestCase):
    def test_graphics_use_explicit_redist_with_notices_and_never_path(self):
        with tempfile.TemporaryDirectory() as temp:
            base = Path(temp); qt = base/'qt'; sdk = base/'sdk'; root = base/'product'
            root.mkdir()
            for name in deployment.FALLBACKS:
                path = qt/'bin'/name if name == 'opengl32sw.dll' else sdk/'Redist/D3D/x64'/name
                path.parent.mkdir(parents=True, exist_ok=True); path.write_bytes(b'owned SDK '+name.encode())
            license_file = sdk/'Licenses/10.0.1.0/sdk_license.rtf'
            license_file.parent.mkdir(parents=True); license_file.write_bytes(b'owned license')
            config = {'mesa_binary': {'sha256': hashlib.sha256((qt/'bin/opengl32sw.dll').read_bytes()).hexdigest()}}
            with patch.object(deployment, 'configuration', return_value=config):
                result = deployment.copy_graphics(qt, sdk, '10.0.1.0', root)
            self.assertEqual(len(result['files']), 4)
            self.assertTrue((root/'share/notices/Windows-SDK-graphics/sdk_license.rtf').is_file())
            self.assertNotIn(str(base), json.dumps(result))
            (sdk/'Redist/D3D/x64/dxil.dll').unlink()
            with self.assertRaisesRegex(ValueError, 'Missing bounded'):
                deployment.copy_graphics(qt, sdk, '10.0.1.0', root)
            with self.assertRaisesRegex(ValueError, 'Explicit Windows SDK'):
                deployment.copy_graphics(qt, sdk, '', root)

    def test_unreviewed_mesa_is_rejected_before_copy(self):
        with tempfile.TemporaryDirectory() as temp:
            base = Path(temp); sdk = base/'sdk'; qt = base/'qt'; root = base/'product'; root.mkdir()
            for name in deployment.FALLBACKS:
                path = qt/'bin'/name if name == 'opengl32sw.dll' else sdk/'Redist/D3D/x64'/name
                path.parent.mkdir(parents=True, exist_ok=True); path.write_bytes(b'unknown build')
            (sdk/'license.rtf').write_bytes(b'owned license')
            with self.assertRaisesRegex(ValueError, 'Unreviewed Mesa'):
                deployment.copy_graphics(qt, sdk, '10.0.1.0', root)
            self.assertEqual(list(root.iterdir()), [])

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
