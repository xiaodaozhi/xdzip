#!/usr/bin/env python3
"""Owned-fixture tests: installer provenance, default preservation and bounded cleanup."""
import importlib.util
import configparser
import copy
import json
import os
from pathlib import Path
import plistlib
import re
import shlex
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch
from deployment_audit import digest
from installer_payload import HISTORICAL_PRODUCT_REVISION
from association_types import WINDOWS_ICONS, MACOS_ICONS, windows_progid

ROOT=Path(__file__).resolve().parents[1]
def module(name):
    spec=importlib.util.spec_from_file_location(name,ROOT/'scripts'/f'{name}.py');result=importlib.util.module_from_spec(spec);spec.loader.exec_module(result);return result
linux=module('build-linux-installer');windows=module('build-windows-installer');macos=module('build-macos-dmg')


class InstallerTests(unittest.TestCase):
    def test_windows_runtime_agreement_rejects_changed_supplier_text_and_notice(self):
        with tempfile.TemporaryDirectory() as temp:
            base=Path(temp); source=base/'apps/desktop/packaging/windows/Windows-runtime-NOTICE.txt'
            source.parent.mkdir(parents=True); source.write_bytes('中文说明\nEnglish notice\n'.encode('utf-8'))
            root=base/'payload'; license_file=root/'share/notices/Microsoft-VC-runtime/license.rtf'
            license_file.parent.mkdir(parents=True); license_file.write_bytes(b'{\\rtf1 owned agreement}')
            notice=license_file.parent.parent/source.name; notice.write_bytes(b'\xef\xbb\xbf'+source.read_bytes())
            config=base/'scripts/windows-runtime.json'; config.parent.mkdir()
            config.write_text(json.dumps({'license_sha256':digest(license_file)}))
            with patch.object(windows,'ROOT',base):
                record=windows.runtime_agreement(root)
                self.assertEqual(record['sha256'],digest(license_file))
                self.assertTrue(record['installer_acceptance_required'])
                license_file.write_bytes(b'{\\rtf1 modified agreement}')
                with self.assertRaisesRegex(ValueError,'original pinned'):
                    windows.runtime_agreement(root)
                license_file.write_bytes(b'{\\rtf1 owned agreement}')
                notice.write_bytes(source.read_bytes())
                with self.assertRaisesRegex(ValueError,'complete bilingual'):
                    windows.runtime_agreement(root)
                notice.unlink()
                with self.assertRaisesRegex(ValueError,'complete bilingual'):
                    windows.runtime_agreement(root)

    def test_legacy_debian_version_flag_is_ci_only(self):
        with tempfile.TemporaryDirectory() as temp:
            base=Path(temp)
            result=subprocess.run([sys.executable,str(ROOT/'scripts/build-linux-installer.py'),
                '--root',str(base/'payload'),'--audit',str(base/'audit.json'),
                '--output',str(base/'old.deb'),'--report',str(base/'old.json'),'--qa-legacy-version'],
                env=dict(os.environ,CI=''),capture_output=True,text=True)
            self.assertNotEqual(result.returncode,0)
            self.assertIn('restricted to disposable CI',result.stderr)
            self.assertFalse((base/'old.deb').exists())

    def test_historical_helper_cli_rejects_unpinned_or_non_ci_requests_before_native_tools(self):
        with tempfile.TemporaryDirectory() as temp:
            base=Path(temp)
            commands=[
                [str(ROOT/'scripts/deploy_windows.py'),'--root',str(base/'payload'),
                 '--qt-prefix',str(base/'qt'),'--qml-dir',str(base/'qml'),
                 '--qml-import',str(base/'imports'),'--runtime',str(base/'crt')],
                [str(ROOT/'scripts/build-linux-installer.py'),'--root',str(base/'payload'),
                 '--audit',str(base/'audit.json'),'--output',str(base/'old.deb'),'--report',str(base/'old.json')]]
            for command in commands:
                for ci,pin,expected in [('',HISTORICAL_PRODUCT_REVISION,'disposable CI'),
                                        ('true','a'*40,'fixed full revision'),
                                        ('true',HISTORICAL_PRODUCT_REVISION[:12],'fixed full revision')]:
                    with self.subTest(script=Path(command[0]).name,ci=ci,pin=pin):
                        result=subprocess.run([sys.executable,*command,'--qa-historical-revision',pin],
                            env=dict(os.environ,CI=ci),capture_output=True,text=True)
                        self.assertNotEqual(result.returncode,0)
                        self.assertIn(expected,result.stderr)
            for options in [[],['--qa-legacy-version','--qa-variant','1']]:
                result=subprocess.run([sys.executable,*commands[1],'--qa-historical-revision',HISTORICAL_PRODUCT_REVISION,*options],
                    env=dict(os.environ,CI='true'),capture_output=True,text=True)
                self.assertNotEqual(result.returncode,0)
                self.assertIn('only the pinned legacy version fixture',result.stderr)
            self.assertEqual(list(base.iterdir()),[])

    def test_debian_epoch_preserves_upgrade_from_historical_development(self):
        previous=dict(version='0.1.0-dev.9',revision='a'*12)
        current=dict(version='0.1.0-alpha.1',revision='b'*12)
        old=linux.debian_version(previous,legacy=True)
        new=linux.debian_version(current)
        self.assertEqual(old,'0.1.0~dev.9+git'+'a'*12)
        self.assertEqual(new,'1:0.1.0~alpha.1+git'+'b'*12)
        if shutil.which('dpkg'):
            subprocess.run(['dpkg','--compare-versions',new,'gt',old],check=True)
            versions=['0.1.0-alpha.2','0.1.0-beta.1','0.1.0-rc.1','0.1.0','0.1.1-alpha.1']
            for version in versions:
                following=linux.debian_version(dict(version=version,revision='c'*12))
                subprocess.run(['dpkg','--compare-versions',following,'gt',new],check=True)
                new=following
    def test_linux_candidates_with_same_filename_have_distinct_staging(self):
        first=ROOT/'build/candidate-a/xdzip-ubuntu24.04-amd64.deb'
        second=ROOT/'build/candidate-b/xdzip-ubuntu24.04-amd64.deb'
        self.assertNotEqual(linux.staging_directory(first),linux.staging_directory(second))
        self.assertEqual(linux.staging_directory(first),linux.staging_directory(first.parent/'.'/first.name))
        self.assertTrue(linux.staging_directory(first).is_relative_to(ROOT/'build/installer-staging'))

    def payload(self,base,platform):
        root=base/'payload';root.mkdir()
        app=root/('xdzip.exe' if platform=='windows' else 'bin/xdzip');app.parent.mkdir(exist_ok=True);app.write_bytes(b'MZowned' if platform=='windows' else b'\x7fELFowned')
        helper=root/('xdzip-update-helper.exe' if platform=='windows' else 'bin/xdzip-update-helper')
        helper.write_bytes(b'MZowned helper' if platform=='windows' else b'\x7fELFowned helper')
        if platform == 'windows':
            (root/'xdzip-association.exe').write_bytes(b'MZowned association launcher')
            for icon_id in WINDOWS_ICONS.values():
                shutil.copy2(ROOT/'design/assets/file-associations'/f'xdzip-{icon_id}.ico', root)
        (root/'share').mkdir();(root/'share/deployment.json').write_text(json.dumps({'system_packages':['libc6:amd64','libxcb1:amd64']}))
        (root/'xdzip').write_text('#!/bin/sh\n');(root/'xdzip').chmod(0o755)
        audit={'status':'passed','platform':platform,'mode':'deployed','identity':{'version':'0.1.0-dev.1','build':1,'revision':'a'*12,'dirty':False},
               'files':[{'path':p.relative_to(root).as_posix(),'sha256':digest(p)} for p in root.rglob('*') if p.is_file()]}
        return root,audit

    def test_linux_missing_helper_is_only_valid_for_the_exact_historical_fixture(self):
        with tempfile.TemporaryDirectory() as temp:
            base=Path(temp);root,audit=self.payload(base,'linux');pin=HISTORICAL_PRODUCT_REVISION
            (root/'bin/xdzip-update-helper').unlink()
            audit['files']=[item for item in audit['files'] if item['path']!='bin/xdzip-update-helper']
            audit['identity']['revision']=pin[:12]
            stage=base/'stage'
            with patch.dict(os.environ,{'CI':'true'}):
                for options in [{},{'legacy':True},{'qa':'1'},{'qa':'2'}]:
                    with self.subTest(options=options):
                        with self.assertRaisesRegex(ValueError,'requires the audited standalone'):
                            linux.package_tree(root,audit,stage,**options)
                        self.assertFalse(stage.exists())
                for options in [{'historical_revision':pin},
                                {'legacy':True,'historical_revision':pin,'qa':'1'},
                                {'legacy':True,'historical_revision':pin[:12]},
                                {'legacy':True,'historical_revision':'b'*40}]:
                    with self.subTest(options=options):
                        with self.assertRaisesRegex(ValueError,'Historical helper omission'):
                            linux.package_tree(root,audit,stage,**options)
                        self.assertFalse(stage.exists())
                for field,value in [('revision','b'*12),('version','0.1.0-alpha.1'),('build',2),('dirty',True)]:
                    invalid=copy.deepcopy(audit);invalid['identity'][field]=value
                    with self.subTest(field=field):
                        with self.assertRaisesRegex(ValueError,'pinned clean|clean build'):
                            linux.package_tree(root,invalid,stage,legacy=True,historical_revision=pin)
                        self.assertFalse(stage.exists())
                result=linux.package_tree(root,audit,stage,legacy=True,historical_revision=pin)
                self.assertTrue(result['qa_fixture'])
                self.assertFalse(result['version'].startswith('1:'))
                self.assertEqual(result['historical_product_qa']['revision'],pin)
                self.assertIs(result['historical_product_qa']['update_helper_in_historical_source'],False)
                self.assertFalse((stage/'usr/libexec/xdzip-update-helper').exists())
                self.assertFalse((stage/'usr/share/polkit-1/actions/org.xdzip.update.policy').exists())
                self.assertFalse(any((stage/'opt/xdzip/versions').glob('*/bin/xdzip-update-helper')))
            with patch.dict(os.environ,{'CI':'false'}):
                with self.assertRaisesRegex(ValueError,'disposable CI'):
                    linux.package_tree(root,audit,base/'not-ci',legacy=True,historical_revision=pin)
                self.assertFalse((base/'not-ci').exists())
            (root/'bin/xdzip-update-helper').write_bytes(b'new helper added to old source')
            with patch.dict(os.environ,{'CI':'true'}):
                with self.assertRaisesRegex(ValueError,'must not be supplied a newer'):
                    linux.package_tree(root,audit,base/'injected',legacy=True,historical_revision=pin)
                self.assertFalse((base/'injected').exists())

    def test_linux_has_dependencies_literal_argument_and_no_default_takeover(self):
        with tempfile.TemporaryDirectory() as temp:
            base=Path(temp);root,audit=self.payload(base,'linux');stage=base/'stage';r=linux.package_tree(root,audit,stage)
            control=(stage/'DEBIAN/control').read_text()
            self.assertIn('libssl3t64',control);self.assertIn('ca-certificates',control)
            self.assertIn('~dev.1+git',r['version'])
            self.assertIn('libc6 (>= 2.39)',(stage/'DEBIAN/control').read_text(encoding='utf-8'))
            source_entry=configparser.ConfigParser(interpolation=None)
            source_entry.read(ROOT/'apps/desktop/packaging/linux/xdzip.desktop',encoding='utf-8')
            self.assertEqual(shlex.split(source_entry['Desktop Entry']['Exec']),['xdzip','--','%f'])
            self.assertTrue({'application/x-7z-compressed','application/vnd.rar',
                             'application/vnd.ms-cab-compressed','application/x-iso9660-image',
                             'application/x-udf'} <=
                            set(source_entry['Desktop Entry']['MimeType'].split(';')))
            self.assertNotIn('application/octet-stream', source_entry['Desktop Entry']['MimeType'])
            mime=(stage/'usr/share/mime/packages/xdzip-mime.xml').read_text(encoding='utf-8')
            self.assertIn('type="application/x-udf"',mime)
            self.assertIn('pattern="*.udf"',mime)
            self.assertNotIn('*.img',mime)
            installed_entry=configparser.ConfigParser(interpolation=None)
            installed_entry.read(stage/'usr/share/applications/xdzip.desktop',encoding='utf-8')
            self.assertEqual(shlex.split(installed_entry['Desktop Entry']['Exec']),['/usr/bin/xdzip','--','%f'])
            self.assertTrue((stage/'usr/bin/xdzip').read_text(encoding='utf-8').startswith('#!/bin/sh\nexec "/opt/xdzip/versions/'))
            self.assertIn('installing',(stage/'DEBIAN/preinst').read_text(encoding='utf-8'))
            self.assertNotIn('xdg-mime default',(stage/'DEBIAN/postinst').read_text(encoding='utf-8'))
            self.assertNotIn('rm -rf',(stage/'DEBIAN/postrm').read_text(encoding='utf-8'))
            self.assertEqual((stage/'usr/libexec/xdzip-update-helper').read_bytes(),(root/'bin/xdzip-update-helper').read_bytes())
            marker=json.loads(next((stage/'opt/xdzip/versions').glob('*/share/xdzip-install.json')).read_text())
            self.assertEqual((marker['product'],marker['package_type']),('xdzip','deb'))
            policy=(stage/'usr/share/polkit-1/actions/org.xdzip.update.policy').read_text(encoding='utf-8')
            self.assertIn('<allow_active>auth_admin</allow_active>',policy)
            self.assertNotIn('auth_admin_keep',policy)
            self.assertIn('/usr/libexec/xdzip-update-helper',policy)
            self.assertIn('pkexec',(stage/'DEBIAN/control').read_text())

    def test_linux_installer_rejects_stale_or_ambiguous_desktop_commands(self):
        desktop=(ROOT/'apps/desktop/packaging/linux/xdzip.desktop').read_text(encoding='utf-8')
        for invalid in ['Exec=xdzip-browser -- %f','Exec=xdzip %f','Exec=xdzip -- %f\nExec=xdzip -- %f']:
            with self.subTest(command=invalid):
                with self.assertRaisesRegex(ValueError,'Desktop entry'):
                    linux.installed_desktop_entry(desktop.replace('Exec=xdzip -- %f',invalid))

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
            self.assertIn('LicenseFile='+str(root.resolve())+'\\share\\notices\\Microsoft-VC-runtime\\license.rtf',text)
            self.assertIn('InfoBeforeFile='+str(root.resolve())+'\\share\\notices\\Windows-runtime-NOTICE.txt',text)
            self.assertIn('english.LicenseLabel3=',text)
            self.assertIn('chinesesimplified.LicenseLabel3=',text)
            self.assertIn(' -- ""%1""',text)
            self.assertNotIn('HKLM',text);self.assertNotIn('UserChoice',text);self.assertNotIn('[UninstallDelete]',text)
            self.assertIn('Software\\Classes\\Applications\\xdzip-association.exe',text)
            self.assertIn('ValueName: "FriendlyAppName"; ValueData: "xdzip"',text)
            self.assertIn('Subkey: "Software\\Classes\\Applications\\xdzip.exe"; ValueType: string; ValueName: "NoOpenWith"',text)
            self.assertIn('Subkey: "Software\\Classes\\xdzip.Archive"; ValueType: string; ValueName: "NoOpenWith"',text)
            for extension, icon in WINDOWS_ICONS.items():
                progid = windows_progid(extension)
                self.assertEqual(progid, 'xdzip.Archive.'+extension.upper())
                self.assertIn('Software\\Classes\\.'+extension+'\\OpenWithProgids',text)
                self.assertIn('ValueName: ".'+extension+'"; ValueData: "'+progid+'"',text)
                self.assertIn('Software\\Classes\\'+progid+'\\DefaultIcon',text)
                self.assertIn('xdzip-'+icon+'.ico',text)
                self.assertIn('Software\\Classes\\'+progid+'\\shell\\open\\command',text)
                self.assertIn('AfterInstall: PreventSilentDefault(\''+progid+'\')',text)
                self.assertIn('Subkey: "Software\\Classes\\Applications\\xdzip-association.exe\\SupportedTypes"; ValueType: string; ValueName: ".'+extension+'"',text)
                self.assertIn('Subkey: "Software\\Classes\\.'+extension+'\\OpenWithProgids"; ValueType: none; ValueName: "xdzip.Archive"; Flags: deletevalue',text)
            self.assertNotIn('Software\\Classes\\.001\\',text)
            self.assertNotIn('Software\\Classes\\.img\\',text)
            for line in text.splitlines():
                if '\\OpenWithProgids' in line:
                    if 'Flags: deletevalue' in line:
                        self.assertIn('ValueType: none; ValueName: "xdzip.Archive"',line)
                    else:
                        self.assertIn('uninsdeletevalue',line);self.assertIn('ValueType: string',line)
            self.assertFalse(r['qa_fixture'])

    def test_windows_flat_entrypoints_and_legacy_fixture(self):
        with tempfile.TemporaryDirectory() as temp:
            base=Path(temp);root,audit=self.payload(base,'windows')
            text,record=windows.render(root,audit,base/'setup.exe',base/'Chinese.isl')
            self.assertEqual(record['layout'],'flat')
            self.assertEqual(record['executable_relative_path'],'xdzip.exe')
            resource=(ROOT/'apps/desktop/packaging/windows/app.rc.in').read_text(encoding='utf-8')
            original=re.findall(r'VALUE\s+"OriginalFilename",\s*"([^"\\]+)\\0"',resource)
            self.assertEqual(original,['@XDZIP_RESOURCE_FILENAME@'])
            self.assertIn('DestDir: "{app}"; Flags: ignoreversion recursesubdirs',text)
            self.assertIn('Filename: "{app}\\xdzip.exe"',text)
            self.assertIn('UninstallDisplayIcon={app}\\xdzip.exe',text)
            self.assertNotIn('{app}\\versions\\',text)
            self.assertIn('install-manifest.txt',text)
            self.assertIn('ValueName: "Location"; ValueData: "{app}"',text)
            self.assertIn('ValueName: "InstallId"',text)
            self.assertIn('SetupMutex=Local\\xdzip.desktop.installing',text)
            self.assertIn('AcquireUpdateGate()',text)
            self.assertLess(text.index('DestName: "xdzip-package-files.txt"'),text.index('recursesubdirs createallsubdirs'))
            self.assertIn('CurStep <> ssDone',text)
            self.assertIn('FILE_ATTRIBUTE_REPARSE_POINT',text)
            legacy,record=windows.render(root,audit,base/'old.exe',base/'Chinese.isl',legacy=True)
            self.assertTrue(record['qa_fixture'])
            self.assertEqual(record['layout'],'legacy-qa')
            self.assertIn('{app}\\versions\\',legacy)
            self.assertNotIn('function PrepareToInstall',legacy)
            self.assertNotIn('DestName: "install-manifest.txt"',legacy)
            self.assertNotIn('NoOpenWith',legacy)
            self.assertNotIn('PreventSilentDefault',legacy)
            self.assertNotIn('xdzip.Archive.ZIP',legacy)

    def test_windows_rejects_missing_or_changed_format_icon(self):
        with tempfile.TemporaryDirectory() as temp:
            base=Path(temp); root,audit=self.payload(base,'windows')
            icon=root/'xdzip-rar.ico'
            icon.unlink()
            audit['files']=[item for item in audit['files'] if item['path'] != icon.name]
            with self.assertRaisesRegex(ValueError, 'lacks the verified xdzip-rar.ico'):
                windows.render(root,audit,base/'setup.exe',base/'Chinese.isl')
            icon.write_bytes(b'wrong icon')
            audit['files'].append({'path':icon.name,'sha256':digest(icon)})
            with self.assertRaisesRegex(ValueError, 'lacks the verified xdzip-rar.ico'):
                windows.render(root,audit,base/'setup.exe',base/'Chinese.isl')

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

    @unittest.skipUnless(shutil.which('cmake'), 'CMake unavailable')
    def test_macos_generated_document_ranks_and_system_localizations(self):
        # Run the actual product's configuration before Qt discovery so this
        # metadata contract also runs on hosts without an installed Qt SDK.
        configure=(ROOT/'apps/desktop/CMakeLists.txt').read_text(encoding='utf-8').split('find_package(Qt6',1)[0]
        with tempfile.TemporaryDirectory() as directory:
            base=Path(directory);script=base/'render.cmake'
            template=(ROOT/'apps/desktop/packaging/macos/Info.plist.in').as_posix()
            script.write_text(configure+'\nconfigure_file("'+template+'" "${OUTPUT}")\n')
            for requested in [None,'None','Alternate','Owner','Default','none',' Alternate','Alternate;None','']:
                with self.subTest(rank=requested):
                    output=base/'Info.plist'
                    if output.exists():output.unlink()
                    command=['cmake','-DOUTPUT='+str(output)]
                    if requested is not None:command+=['-DXDZIP_MACOS_HANDLER_RANK='+requested]
                    result=subprocess.run(command+['-P',str(script)],capture_output=True,text=True)
                    if requested not in [None,'None','Alternate']:
                        self.assertNotEqual(result.returncode,0)
                        self.assertIn('must be None or Alternate',result.stderr)
                        self.assertFalse(output.exists())
                        continue
                    self.assertEqual(result.returncode,0,result.stderr)
                    info=plistlib.loads(output.read_bytes())
                    documents=info['CFBundleDocumentTypes']
                    self.assertEqual(len(documents),len(MACOS_ICONS))
                    expected={'public.zip-archive':'zip','public.tar-archive':'tar',
                        'org.gnu.gnu-zip-tar-archive':'tar-gz','org.gnu.gnu-zip-archive':'gz',
                        'org.7-zip.7-zip-archive':'7z','com.rarlab.rar-archive':'rar',
                        'com.microsoft.cab-archive':'cab','public.iso-image':'iso','org.xdzip.udf-image':'udf'}
                    observed={}
                    extensions=set()
                    for document in documents:
                        self.assertEqual(document['LSHandlerRank'],requested or 'None')
                        self.assertEqual(document['CFBundleTypeRole'],'Viewer')
                        self.assertEqual(len(document['LSItemContentTypes']),1)
                        uti=document['LSItemContentTypes'][0]
                        self.assertNotIn(uti,observed)
                        observed[uti]=document['CFBundleTypeIconFile']
                        extensions.update(document['CFBundleTypeExtensions'])
                    self.assertEqual(observed,{uti:'xdzip-'+icon+'.icns' for uti,icon in expected.items()})
                    self.assertEqual(extensions,set(WINDOWS_ICONS))
                    imported={item['UTTypeIdentifier']:item for item in info['UTImportedTypeDeclarations']}
                    self.assertEqual(imported['org.xdzip.udf-image']['UTTypeTagSpecification']['public.filename-extension'],['udf'])
                    self.assertEqual(info['CFBundleAllowMixedLocalizations'],True)
                    self.assertTrue({'zh-Hans','en'}<=set(info['CFBundleLocalizations']))

    def test_macos_dmg_rejects_nonhandler_builds_without_modifying_them(self):
        with tempfile.TemporaryDirectory() as directory:
            app=Path(directory)/'xdzip.app';(app/'Contents/MacOS').mkdir(parents=True)
            (app/'Contents/MacOS/xdzip').write_bytes(b'owned')
            path=app/'Contents/Info.plist'
            base={'CFBundleIdentifier':'org.xdzip.desktop','CFBundleExecutable':'xdzip'}
            accepted={'LSHandlerRank':'Alternate','CFBundleTypeRole':'Viewer'}
            for documents in [None,[],[{'LSHandlerRank':'None','CFBundleTypeRole':'Viewer'}],
                              [accepted,{'LSHandlerRank':'None','CFBundleTypeRole':'Viewer'}],
                              [{'LSHandlerRank':'Alternate','CFBundleTypeRole':'None'}],['Alternate']]:
                with self.subTest(documents=documents):
                    info=dict(base)
                    if documents is not None:info['CFBundleDocumentTypes']=documents
                    original=plistlib.dumps(info);path.write_bytes(original)
                    with patch.object(macos,'checked') as checked:
                        with self.assertRaisesRegex(ValueError,'XDZIP_MACOS_HANDLER_RANK=Alternate'):
                            macos.validate_bundle(app,require_handler=True)
                        checked.assert_not_called()
                    self.assertEqual(path.read_bytes(),original)
            path.write_bytes(plistlib.dumps(dict(base,CFBundleDocumentTypes=[accepted])))
            with patch.object(macos,'checked',return_value='arm64'):
                self.assertEqual(macos.validate_bundle(app,require_handler=True),app/'Contents/MacOS/xdzip')

if __name__=='__main__':unittest.main()
