#!/usr/bin/env python3
"""Owned-fixture tests: installer provenance, default preservation and bounded cleanup."""
import importlib.util
import configparser
import copy
import json
from pathlib import Path
import tempfile
import unittest
from deployment_audit import digest

ROOT=Path(__file__).resolve().parents[1]
def module(name):
    spec=importlib.util.spec_from_file_location(name,ROOT/'scripts'/f'{name}.py');result=importlib.util.module_from_spec(spec);spec.loader.exec_module(result);return result
linux=module('build-linux-installer');windows=module('build-windows-installer');macos=module('build-macos-dmg')


class InstallerTests(unittest.TestCase):
    def payload(self,base,platform):
        root=base/'payload';root.mkdir()
        app=root/('xdzip.exe' if platform=='windows' else 'bin/xdzip');app.parent.mkdir(exist_ok=True);app.write_bytes(b'MZowned' if platform=='windows' else b'\x7fELFowned')
        (root/'share').mkdir();(root/'share/deployment.json').write_text(json.dumps({'system_packages':['libc6:amd64','libxcb1:amd64']}))
        (root/'xdzip').write_text('#!/bin/sh\n');(root/'xdzip').chmod(0o755)
        audit={'status':'passed','platform':platform,'mode':'deployed','identity':{'version':'0.1.0-dev.1','build':1,'revision':'a'*12,'dirty':False},
               'files':[{'path':p.relative_to(root).as_posix(),'sha256':digest(p)} for p in root.rglob('*') if p.is_file()]}
        return root,audit

    def test_linux_has_dependencies_literal_argument_and_no_default_takeover(self):
        with tempfile.TemporaryDirectory() as temp:
            base=Path(temp);root,audit=self.payload(base,'linux');stage=base/'stage';r=linux.package_tree(root,audit,stage)
            self.assertIn('~dev.1+git',r['version'])
            self.assertIn('libc6 (>= 2.39)',(stage/'DEBIAN/control').read_text(encoding='utf-8'))
            self.assertIn('-- %f',(stage/'usr/share/applications/xdzip.desktop').read_text(encoding='utf-8'))
            self.assertTrue((stage/'usr/bin/xdzip').read_text(encoding='utf-8').startswith('#!/bin/sh\nexec "/opt/xdzip/versions/'))
            self.assertIn('installing',(stage/'DEBIAN/preinst').read_text(encoding='utf-8'))
            self.assertNotIn('xdg-mime default',(stage/'DEBIAN/postinst').read_text(encoding='utf-8'))
            self.assertNotIn('rm -rf',(stage/'DEBIAN/postrm').read_text(encoding='utf-8'))

    def test_changed_audit_and_sdk_only_build_are_rejected(self):
        with tempfile.TemporaryDirectory() as temp:
            base=Path(temp);root,audit=self.payload(base,'linux');(root/'xdzip').write_text('changed')
            with self.assertRaisesRegex(ValueError,'changed'):linux.package_tree(root,audit,base/'bad')
            audit['mode']='sdk'
            with self.assertRaisesRegex(ValueError,'deployed'):linux.package_tree(root,audit,base/'bad')

    def test_failure_injection_never_enters_product_package(self):
        with tempfile.TemporaryDirectory() as temp:
            base=Path(temp);root,audit=self.payload(base,'linux')
            with self.assertRaisesRegex(ValueError,'QA variant 2'):
                linux.package_tree(root,audit,base/'bad',failure='postinst-once')
            linux.package_tree(root,audit,base/'product')
            self.assertNotIn('qa-postinst-failed',(base/'product/DEBIAN/postinst').read_text())
            result=linux.package_tree(root,audit,base/'fixture',qa='2',failure='postinst-once')
            self.assertTrue(result['qa_fixture'])
            self.assertIn('exit 85',(base/'fixture/DEBIAN/postinst').read_text())

    def test_windows_is_per_user_quoted_and_tracks_only_owned_keys(self):
        with tempfile.TemporaryDirectory() as temp:
            base=Path(temp);root,audit=self.payload(base,'windows')
            text,r=windows.render(root,audit,base/'setup.exe',base/'Chinese.isl')
            self.assertIn('PrivilegesRequired=lowest',text)
            self.assertIn('AppMutex=Local\\xdzip.desktop.running',text)
            self.assertIn(' -- ""%1""',text)
            self.assertNotIn('HKLM',text);self.assertNotIn('UserChoice',text);self.assertNotIn('[UninstallDelete]',text)
            for line in text.splitlines():
                if '\\OpenWithProgids' in line:
                    self.assertIn('uninsdeletevalue',line);self.assertIn('ValueType: string',line)
            self.assertFalse(r['qa_fixture'])

    def test_windows_flat_entrypoints_and_legacy_fixture(self):
        with tempfile.TemporaryDirectory() as temp:
            base=Path(temp);root,audit=self.payload(base,'windows')
            text,record=windows.render(root,audit,base/'setup.exe',base/'Chinese.isl')
            self.assertEqual(record['layout'],'flat')
            self.assertEqual(record['executable_relative_path'],'xdzip.exe')
            self.assertIn('DestDir: "{app}"; Flags: ignoreversion recursesubdirs',text)
            self.assertIn('Filename: "{app}\\xdzip.exe"',text)
            self.assertIn('UninstallDisplayIcon={app}\\xdzip.exe',text)
            self.assertNotIn('{app}\\versions\\',text)
            self.assertIn('install-manifest.txt',text)
            self.assertLess(text.index('DestName: "xdzip-package-files.txt"'),text.index('recursesubdirs createallsubdirs'))
            self.assertIn('CurStep <> ssDone',text)
            self.assertIn('FILE_ATTRIBUTE_REPARSE_POINT',text)
            legacy,record=windows.render(root,audit,base/'old.exe',base/'Chinese.isl',legacy=True)
            self.assertTrue(record['qa_fixture'])
            self.assertEqual(record['layout'],'legacy-qa')
            self.assertIn('{app}\\versions\\',legacy)
            self.assertNotIn('function PrepareToInstall',legacy)
            self.assertNotIn('DestName: "install-manifest.txt"',legacy)

    def test_windows_manifest_is_exact_bounded_and_rejects_path_aliases(self):
        with tempfile.TemporaryDirectory() as temp:
            root,audit=self.payload(Path(temp),'windows')
            entries=windows.payload_manifest(audit).splitlines()
            self.assertEqual(entries[0],'xdzip-install-files-v1')
            actual={line.split('\t')[1].replace('\\','/'):line.split('\t')[0] for line in entries[1:]}
            self.assertEqual(actual,{p['path']:p['sha256'] for p in audit['files']})
            for path in ['../keep.txt','C:/keep.txt','/keep.txt','share//keep.txt',
                         'share/./keep.txt','share/keep.txt:stream','share/NUL.txt',
                         'share/COM1','share/keep.','share/keep ',
                         'versions/owned.txt','unins000.dat','install-manifest.txt']:
                with self.subTest(path=path):
                    bad=copy.deepcopy(audit);bad['files'][0]['path']=path
                    with self.assertRaisesRegex(ValueError,'Unsafe'):
                        windows.payload_manifest(bad)
            duplicate=copy.deepcopy(audit);duplicate['files'].append(dict(duplicate['files'][0],path=duplicate['files'][0]['path'].upper()))
            with self.assertRaisesRegex(ValueError,'Unsafe'):windows.payload_manifest(duplicate)

    def test_qa_versions_are_distinct_and_dirty_payload_is_not_delivered(self):
        with tempfile.TemporaryDirectory() as temp:
            base=Path(temp);root,audit=self.payload(base,'windows')
            _,one=windows.render(root,audit,base/'one.exe',base/'Chinese.isl','1')
            _,two=windows.render(root,audit,base/'two.exe',base/'Chinese.isl','2')
            self.assertNotEqual(one['runtime_id'],two['runtime_id']);self.assertTrue(one['qa_fixture'])
            audit['identity']['dirty']=True
            with self.assertRaisesRegex(ValueError,'clean'):windows.render(root,audit,base/'bad.exe',base/'Chinese.isl')

    def test_unrecorded_file_cannot_enter_installer(self):
        with tempfile.TemporaryDirectory() as temp:
            base=Path(temp);root,audit=self.payload(base,'windows');(root/'unexpected.dll').write_bytes(b'MZunexpected')
            with self.assertRaisesRegex(ValueError,'unrecorded'):windows.render(root,audit,base/'bad.exe',base/'Chinese.isl')

    def test_macos_native_translations_use_bundled_path(self):
        with tempfile.TemporaryDirectory() as directory:
            app=Path(directory)/'xdzip.app'
            resource=app/'Contents/Resources'
            resource.mkdir(parents=True)
            path=resource/'qt.conf'
            path.write_text('[Paths]\nPlugins = PlugIns\nQmlImports = Resources/qml\n[Other]\nkeep = owned\n')
            macos.configure_translation_path(app)
            config=configparser.ConfigParser(interpolation=None)
            config.read(path)
            self.assertEqual(config['Paths']['Translations'],'Resources/translations')
            self.assertEqual(config['Paths']['Plugins'],'PlugIns')
            self.assertEqual(config['Paths']['QmlImports'],'Resources/qml')
            self.assertEqual(config['Other']['keep'],'owned')
            first=path.read_bytes()
            macos.configure_translation_path(app)
            self.assertEqual(path.read_bytes(),first)

    def test_macos_alternate_documents_preserve_system_localizations(self):
        text=(ROOT/'apps/desktop/packaging/macos/Info.plist.in').read_text(encoding='utf-8')
        self.assertIn('<string>Alternate</string>',text);self.assertIn('xdzip-zip.icns',text)
        self.assertIn('<string>public.zip-archive</string>',text);self.assertIn('<string>org.gnu.gnu-zip-tar-archive</string>',text)
        self.assertIn('<string>zh-Hans</string>',text)

if __name__=='__main__':unittest.main()
