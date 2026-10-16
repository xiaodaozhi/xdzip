#!/usr/bin/env python3
"""Boundary tests for Linux runtime planning and isolated product event evidence."""
import json
import tarfile
import sys
from pathlib import Path
import tempfile
import unittest
from deploy_linux import qml_roots, relative_rpath, runtime_origin
from verify_linux_runtime import check_events
from package_linux import archive_payload


class LinuxDeploymentTests(unittest.TestCase):
    def test_only_sdk_modules_and_explicit_static_or_optional_modules(self):
        with tempfile.TemporaryDirectory() as temp:
            sdk=Path(temp)
            (sdk/'qml/QtQuick/Controls').mkdir(parents=True);(sdk/'qml/QtQml').mkdir()
            entries=[{'name':'QtQuick','type':'module','path':str(sdk/'qml/QtQuick')},
                     {'name':'QtQuick.Controls','type':'module','path':str(sdk/'qml/QtQuick/Controls')},
                     {'name':'QtQml','type':'module','path':str(sdk/'qml/QtQml')},
                     {'name':'Xdzip.Settings','type':'module'},
                     {'name':'QtQuick.Controls.macOS','type':'module'}]
            self.assertEqual(qml_roots(entries,sdk),(['QtQml','QtQuick'],['QtQuick.Controls.macOS']))
            entries.append({'name':'Unknown.Required','type':'module'})
            with self.assertRaisesRegex(ValueError,'Unresolved QML'):qml_roots(entries,sdk)
            entries[-1]={'name':'Unknown.Required','type':'module','path':str(sdk/'outside')}
            with self.assertRaisesRegex(ValueError,'escapes'):qml_roots(entries,sdk)

    def test_copied_icu_sibling_is_an_internal_dependency(self):
        with tempfile.TemporaryDirectory() as temp:
            base=Path(temp);sdk=base/'sdk';payload=base/'app'
            (sdk/'lib').mkdir(parents=True);(payload/'lib').mkdir(parents=True)
            copied=payload/'lib/libicudata.so.73';copied.touch()
            self.assertEqual(runtime_origin(copied,sdk,payload),'artifact')
            self.assertEqual(runtime_origin(sdk/'lib/libicudata.so.73',sdk,payload),'sdk')
            with self.assertRaisesRegex(ValueError,'Unexpected external'):
                runtime_origin(base/'unknown/libicudata.so.73',sdk,payload)

    def test_nested_plugin_origin_paths_remain_inside_relocatable_root(self):
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp)
            for relative in ['bin/xdzip','lib/libQt6Core.so.6','qml/QtQuick/Layouts/libplugin.so','plugins/platforms/libqxcb.so']:
                binary=root/relative;binary.parent.mkdir(parents=True,exist_ok=True);binary.touch()
                rpath=relative_rpath(binary,root)
                self.assertTrue(rpath.startswith('$ORIGIN/'))
                target=binary.parent/rpath.removeprefix('$ORIGIN/')
                self.assertEqual(target.resolve(),(root/'lib').resolve())
            with self.assertRaisesRegex(ValueError,'outside'):relative_rpath(root.parent/'foreign',root)

    def test_runtime_evidence_requires_real_open_and_interrupted_cancel(self):
        events=[{'event':'first_frame','runtime_available':True}, {'event':'open_ready','records':100000},
                {'event':'cancel_requested','pending':True}, {'event':'cancel_settled','cancelled':True},
                {'event':'quit_requested'}]
        def output():return '\n'.join('XDZIP_PROBE '+json.dumps(e) for e in events)
        self.assertEqual(check_events(output()),events)
        events[3]['cancelled']=False
        with self.assertRaisesRegex(ValueError,'cancellation'):check_events(output())
        events[3]['cancelled']=True;events[1]['records']=1
        with self.assertRaisesRegex(ValueError,'owned fixture'):check_events(output())
        events.pop()
        with self.assertRaisesRegex(ValueError,'finish all stages'):check_events(output())

    @unittest.skipIf(sys.platform=='win32','POSIX mode bits require a Unix filesystem')
    def test_archive_preserves_entrypoint_and_has_stable_metadata(self):
        with tempfile.TemporaryDirectory() as temp:
            base=Path(temp);root=base/'payload';(root/'bin').mkdir(parents=True)
            (root/'bin/xdzip').write_bytes(b'owned fixture');(root/'bin/xdzip').chmod(0o755)
            (root/'notice').write_text('license')
            first=archive_payload(root,base/'one.tar.gz');second=archive_payload(root,base/'two.tar.gz')
            self.assertEqual(first['sha256'],second['sha256'])
            with tarfile.open(base/'one.tar.gz') as archive:
                app=archive.getmember('xdzip-linux-x86_64/bin/xdzip')
                self.assertEqual(app.mode,0o755);self.assertEqual(app.uid,0);self.assertEqual(app.mtime,0)
            with self.assertRaisesRegex(ValueError,'outside'):archive_payload(root,root/'bad.tar.gz')

if __name__=='__main__':unittest.main()
