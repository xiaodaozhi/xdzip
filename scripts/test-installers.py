#!/usr/bin/env python3
"""Owned-fixture tests: installer provenance, default preservation and bounded cleanup."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from deployment_audit import digest

ROOT=Path(__file__).resolve().parents[1]
def module(name):
    spec=importlib.util.spec_from_file_location(name,ROOT/'scripts'/f'{name}.py');result=importlib.util.module_from_spec(spec);spec.loader.exec_module(result);return result
linux=module('build-linux-installer');windows=module('build-windows-installer')


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

    def test_macos_alternate_documents_preserve_system_localizations(self):
        text=(ROOT/'apps/desktop/packaging/macos/Info.plist.in').read_text(encoding='utf-8')
        self.assertIn('<string>Alternate</string>',text);self.assertIn('xdzip-zip.icns',text)
        self.assertIn('<string>public.zip-archive</string>',text);self.assertIn('<string>org.gnu.gnu-zip-tar-archive</string>',text)
        self.assertIn('<string>zh-Hans</string>',text)

if __name__=='__main__':unittest.main()
