#!/usr/bin/env python3
"""Source cache integrity and portable bundle boundary tests, using owned fixtures."""
import importlib.util
import io
import json
from contextlib import nullcontext, redirect_stdout
from pathlib import Path
import plistlib
import subprocess
import sys
import tarfile
import tempfile
import unittest
from unittest.mock import patch
from deployment_audit import digest
from desktop_product import built_product
import macos_launch_services as ls

ROOT = Path(__file__).resolve().parents[1]


def module(name):
    spec = importlib.util.spec_from_file_location(name, ROOT/'scripts'/(name+'.py'))
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


source = module('prepare-source-delivery')
mac = module('build-macos-dmg')
verify_dmg = module('verify-macos-dmg')
replace_qt = module('verify-qt-replacement')


class DeliveryTests(unittest.TestCase):
    def test_current_product_and_historical_rebuild_names(self):
        for platform, name, legacy in [('linux', 'xdzip', 'xdzip-browser'),
                                       ('win32', 'xdzip.exe', 'xdzip-browser.exe')]:
            with self.subTest(platform=platform), tempfile.TemporaryDirectory() as temp:
                build = Path(temp)
                desktop = build/'apps/desktop'
                desktop.mkdir(parents=True)
                old = desktop/legacy
                current = desktop/name
                old.write_bytes(b'owned historical executable')
                self.assertEqual(built_product(build, platform=platform, allow_legacy_name=True), old)
                with self.assertRaisesRegex(ValueError, 'exactly one'):
                    built_product(build, platform=platform)
                current.write_bytes(b'owned current executable')
                self.assertEqual(built_product(build, platform=platform), current)
                with self.assertRaisesRegex(ValueError, 'exactly one'):
                    built_product(build, platform=platform, allow_legacy_name=True)
                old.unlink()
                self.assertEqual(built_product(build, platform=platform, allow_legacy_name=True), current)
                current.unlink()
                with self.assertRaisesRegex(ValueError, 'exactly one'):
                    built_product(build, platform=platform, allow_legacy_name=True)

    def test_macos_product_remains_inside_its_bundle(self):
        with tempfile.TemporaryDirectory() as temp:
            build = Path(temp)
            app = build/'apps/desktop/xdzip.app/Contents/MacOS/xdzip'
            app.parent.mkdir(parents=True)
            app.write_bytes(b'owned bundle executable')
            for historical in [False, True]:
                self.assertEqual(built_product(build, platform='darwin', allow_legacy_name=historical), app)

    def archive(self, root, member='owned-src-1/file.txt'):
        path = root/'owned-src-1.tar.xz'
        with tarfile.open(path, 'w:xz') as archive:
            info = tarfile.TarInfo(member)
            info.size = 4
            archive.addfile(info, io.BytesIO(b'own!'))
        return {'file': path.name, 'sha256': digest(path), 'url': 'https://invalid.test/no-network'}

    def test_verified_cache_is_offline_and_tampered_cache_fails(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            item = self.archive(root)
            self.assertEqual(source.archive_source(item, root).name, item['file'])
            (root/item['file']).write_bytes(b'corrupted')
            with self.assertRaisesRegex(ValueError, 'checksum'):
                source.archive_source(item, root)

    def test_source_paths_are_inspected_without_extracting(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            for member in ['../escape', 'owned-src-1/../escape', '/absolute', 'wrong-src/file']:
                item = self.archive(root, member)
                with self.assertRaisesRegex(ValueError, 'layout'):
                    source.archive_source(item, root)
            self.assertFalse((root/'escape').exists())

    def test_bundle_links_cannot_escape_the_application(self):
        with tempfile.TemporaryDirectory() as temp:
            app = Path(temp)/'xdzip.app'
            (app/'Contents/MacOS').mkdir(parents=True)
            (app/'Contents/MacOS/xdzip').write_bytes(b'owned')
            (app/'Contents/Info.plist').write_bytes(plistlib.dumps(
                {'CFBundleIdentifier': 'org.xdzip.desktop', 'CFBundleExecutable': 'xdzip'}))
            (app/'Contents/escape').symlink_to(Path(temp))
            with patch.object(mac, 'checked', return_value='arm64'):
                with self.assertRaisesRegex(ValueError, 'escapes'):
                    mac.validate_bundle(app)


class DisposableBundleTests(unittest.TestCase):
    """Simulated native tools, actual owned copies, hashes and disposal."""
    def bundle(self, root):
        app = root/'xdzip.app'
        binary = app/'Contents/MacOS/xdzip'
        core = app/'Contents/Frameworks/QtCore.framework/Versions/A/QtCore'
        for path, data in [(binary, b'owned executable'), (core, b'original core')]:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
        (app/'Contents/Info.plist').write_bytes(plistlib.dumps({
            'CFBundleIdentifier': ls.BUNDLE_ID, 'CFBundleExecutable': 'xdzip',
            'CFBundleDocumentTypes': [{'LSHandlerRank': 'Alternate', 'CFBundleTypeRole': 'Viewer'}]}))
        return app, binary, core

    def test_dmg_builder_preserves_image_and_evidence_but_removes_stage_on_success_or_failure(self):
        for failed in [False, True]:
            with self.subTest(failed=failed), tempfile.TemporaryDirectory() as temp:
                root = Path(temp).resolve();source, binary, core = self.bundle(root/'input')
                output = root/'build/package.dmg';qt = root/'qt';revision = 'a'*12
                identity = 'xdzip (development, '+revision+')'
                protected = {path: digest(path) for path in [binary, core, source/'Contents/Info.plist']}
                staged = output.parent/'package-stage/xdzip.app'
                phases = []
                optimization = {'before_bytes': 150, 'after_bytes': 80, 'bytes_saved': 70}
                bundle_validation = {'status': 'passed', 'architectures': ['arm64'], 'control_style': 'Basic'}
                def checked(command, **_):
                    if str(command[0]) == 'lipo':return 'arm64'
                    if '--version' in command:return identity
                    if 'rev-parse' in command:return revision
                    if len(command)>1 and Path(str(command[1])).name == 'collect-notices.py':
                        notices = Path(command[2]);notices.mkdir(parents=True, exist_ok=True)
                        (notices/'build-inventory.json').write_text(json.dumps({'display_version':'0.1.0','version_source':{'build':1}}))
                        (notices/'source-build.cdx.json').write_text('{}')
                    if len(command)>1 and Path(str(command[1])).name == 'runtime_attribution.py':
                        self.assertTrue(staged.is_dir())
                        self.assertTrue((output.parent/'deployment-audit.json').is_file())
                        Path(command[command.index('--output')+1]).write_text('{"owned":"attribution"}')
                    if str(command[0]) == 'hdiutil':
                        self.assertTrue(staged.is_dir())
                        if command[1] == 'create':output.write_bytes(b'owned image')
                        elif failed:raise ValueError('owned verify failure')
                    return ''
                def optimize(app):
                    self.assertEqual(app, staged)
                    phases.append('optimize')
                    return optimization
                def sign(app, *_):
                    self.assertEqual(app, staged)
                    phases.append('sign')
                def validate(app):
                    self.assertEqual(app, staged)
                    phases.append('validate')
                    return bundle_validation
                with patch.object(ls,'ROOT',root), patch.object(mac,'ROOT',root), \
                     patch.object(mac.sys,'platform','darwin'), patch.object(mac,'checked',side_effect=checked), \
                     patch.object(mac,'transient_registrations',side_effect=lambda _:nullcontext()), \
                     patch.object(mac,'optimize_bundle',side_effect=optimize), \
                     patch.object(mac,'sdk_from_build',return_value=qt), \
                     patch.object(mac,'embed_sparkle',return_value=staged/'Contents/Frameworks/Sparkle.framework'), \
                     patch.object(mac,'signing_metadata',return_value=[]), \
                     patch.object(mac,'attribute_sparkle'), \
                     patch.object(mac,'validate_optimized_bundle',side_effect=validate), \
                     patch.object(mac,'sign_local',side_effect=sign), patch.object(mac,'native',return_value=False), \
                     patch.object(mac,'audit',return_value={'status':'passed'}), \
                     patch.object(mac,'deployment_record',return_value={'sdk_files':[]}), \
                     patch.object(mac,'artifact_bom',return_value={'owned':'sbom'}):
                    if failed:
                        with self.assertRaisesRegex(ValueError,'owned verify failure'):mac.build(source,qt,output)
                    else:
                        report = mac.build(source,qt,output)
                        self.assertEqual(report['sha256'],digest(output))
                        self.assertEqual(report['bundle_optimization'], optimization)
                        self.assertEqual(report['bundle'], bundle_validation)
                self.assertEqual(phases, ['optimize', 'sign', 'validate'])
                self.assertFalse(staged.exists())
                self.assertTrue(output.is_file())
                self.assertTrue(output.with_suffix('.cdx.json').is_file())
                self.assertEqual(output.with_suffix('.json').exists(),not failed)
                self.assertEqual(protected,{path:digest(path) for path in protected})

    def test_dmg_optimization_failure_stops_before_signing_and_discards_stage(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp).resolve();source, binary, core = self.bundle(root/'input')
            output = root/'build/package.dmg';qt = root/'qt';revision = 'a'*12
            protected = {path:digest(path) for path in [binary,core,source/'Contents/Info.plist']}
            def checked(command, **_):
                if str(command[0]) == 'lipo':return 'arm64'
                if '--version' in command:return 'xdzip (development, '+revision+')'
                if 'rev-parse' in command:return revision
                return ''
            with patch.object(ls,'ROOT',root), patch.object(mac,'ROOT',root), \
                 patch.object(mac.sys,'platform','darwin'), patch.object(mac,'checked',side_effect=checked), \
                 patch.object(mac,'transient_registrations',side_effect=lambda _:nullcontext()), \
                 patch.object(mac,'sdk_from_build',return_value=qt), \
                 patch.object(mac,'embed_sparkle',return_value=root/'Sparkle.framework'), \
                 patch.object(mac,'signing_metadata',return_value=[]), \
                 patch.object(mac,'optimize_bundle',side_effect=ValueError('invalid deployment')), \
                 patch.object(mac,'sign_local') as sign, patch.object(mac,'audit') as audit:
                with self.assertRaisesRegex(ValueError,'invalid deployment'):mac.build(source,qt,output)
                sign.assert_not_called();audit.assert_not_called()
            self.assertFalse((output.parent/'package-stage/xdzip.app').exists())
            self.assertFalse(output.exists())
            self.assertFalse(output.with_suffix('.json').exists())
            self.assertEqual(protected,{path:digest(path) for path in protected})

    def test_dmg_verifier_freezes_hashes_and_removes_copy_on_success_or_failure(self):
        for failed in [False, True]:
            with self.subTest(failed=failed), tempfile.TemporaryDirectory() as temp:
                root = Path(temp).resolve();mount = root/'mounted'
                source, binary, core = self.bundle(mount)
                (mount/'Applications').symlink_to('/Applications')
                dmg = root/'input.dmg';dmg.write_bytes(b'owned image')
                output = root/'build/verification';copy = output/'中文 space/xdzip.app'
                protected = {path:digest(path) for path in [binary,core,dmg]}
                modified = core.read_bytes()+b' modified'
                def checked(command, **_):
                    if str(command[0]) == 'lipo':return 'arm64'
                    if str(command[0]) == 'install_name_tool':Path(command[-1]).write_bytes(modified)
                    return ''
                def run(command, **_):
                    if '--smoke-test' in command:
                        self.assertTrue(copy.exists())
                        return subprocess.CompletedProcess(command,1 if failed else 0,'owned output','')
                    return subprocess.CompletedProcess(command,0,'','')
                arguments = ['verify-macos-dmg','--dmg',str(dmg),'--output',str(output)]
                mounted = plistlib.dumps({'system-entities':[{'mount-point':str(mount)}]})
                with patch.object(ls,'ROOT',root), patch.object(verify_dmg,'ROOT',root), \
                     patch.object(verify_dmg,'transient_registrations',side_effect=lambda _:nullcontext()), \
                     patch.object(verify_dmg,'audit',return_value={'status':'passed'}), \
                     patch.object(verify_dmg,'validate_optimized_bundle',return_value={'status':'passed'}), \
                     patch.object(verify_dmg,'verify_signatures',return_value={'status':'passed'}), \
                     patch.object(verify_dmg,'product_identity',return_value=('xdzip owned fixture',{})), \
                     patch.object(verify_dmg,'verify_extraction',return_value={'status':'passed'}), \
                     patch.object(verify_dmg.macdmg,'checked',side_effect=checked), \
                     patch.object(verify_dmg.macdmg,'sign_local'), \
                     patch.object(verify_dmg.subprocess,'check_output',return_value=mounted), \
                     patch.object(verify_dmg.subprocess,'run',side_effect=run), patch.object(sys,'argv',arguments), \
                     redirect_stdout(io.StringIO()):
                    if failed:
                        with self.assertRaisesRegex(ValueError,'relocated-cocoa-metal failed'):verify_dmg.main()
                    else:verify_dmg.main()
                self.assertFalse(copy.exists())
                self.assertTrue((output/'relocated-cocoa-metal.log').is_file())
                self.assertEqual(protected,{path:digest(path) for path in protected})
                self.assertEqual((output/'verification.json').exists(),not failed)
                if not failed:
                    expected = root/'modified';expected.write_bytes(modified)
                    result = json.loads((output/'verification.json').read_text())
                    self.assertEqual(result['replacement']['original_sha256'],digest(core))
                    self.assertEqual(result['replacement']['modified_sha256'],digest(expected))

    def test_qt_replacement_freezes_hashes_and_removes_copy_on_success_or_failure(self):
        for failed in [False, True]:
            with self.subTest(failed=failed), tempfile.TemporaryDirectory() as temp:
                root = Path(temp).resolve();source, binary, core = self.bundle(root/'input')
                qt = root/'source-qt';new_core = qt/'qt-install/lib/QtCore.framework/Versions/A/QtCore'
                new_core.parent.mkdir(parents=True);new_core.write_bytes(b'source built core')
                record = qt/'qt-source-build.json'
                record.write_text(json.dumps({'revision':'a'*40,'qtcore_sha256':digest(new_core)}))
                output = root/'build/replacement';copy = output/'中文 relocated/xdzip.app'
                protected = {path:digest(path) for path in [binary,core,new_core,record]}
                def checked(command, **_):
                    if str(command[0]) == 'lipo':return 'arm64'
                    if '--version' in command:return 'xdzip ('+'a'*12+')'
                    return ''
                def sign(app):
                    path = app/'Contents/Frameworks/QtCore.framework/Versions/A/QtCore'
                    path.write_bytes(path.read_bytes()+b' signed')
                def run(command, **_):
                    target = Path(command[command.index('--output')+1])
                    if Path(command[1]).name.startswith('generate-'):target.write_bytes(b'owned fixture')
                    else:
                        self.assertTrue(copy.exists())
                        if failed and target.stem == 'extract':raise subprocess.CalledProcessError(1,command)
                        target.write_text(json.dumps({'runs':[{'events':[{'event':'qt_source_proof',
                            'value':'xdzip-qt-6.8.2-source-proof-v1'}],'extraction':{'independent_byte_crc_check':True}}]}))
                    return subprocess.CompletedProcess(command,0)
                arguments = ['verify-qt-replacement','--app',str(source),'--qt-build',str(qt),
                             '--metrics',str(root/'metrics'),'--output',str(output)]
                with patch.object(ls,'ROOT',root), patch.object(replace_qt,'ROOT',root), \
                     patch.object(replace_qt.sys,'platform','darwin'), \
                     patch.object(replace_qt,'transient_registrations',side_effect=lambda _:nullcontext()), \
                     patch.object(replace_qt,'module',return_value=mac), \
                     patch.object(mac,'checked',side_effect=checked), patch.object(mac,'sign_local',side_effect=sign), \
                     patch.object(replace_qt,'audit',return_value={'status':'passed'}), \
                     patch.object(replace_qt.subprocess,'run',side_effect=run), patch.object(sys,'argv',arguments), \
                     redirect_stdout(io.StringIO()):
                    if failed:
                        with self.assertRaises(subprocess.CalledProcessError):replace_qt.main()
                    else:replace_qt.main()
                self.assertFalse(copy.exists())
                self.assertTrue((output/'replacement.log').is_file())
                self.assertTrue((output/'metadata.json').is_file())
                self.assertEqual(protected,{path:digest(path) for path in protected})
                self.assertEqual((output/'qt-source-replacement.json').exists(),not failed)
                if not failed:
                    expected = root/'signed-core';expected.write_bytes(new_core.read_bytes()+b' signed')
                    result = json.loads((output/'qt-source-replacement.json').read_text())
                    self.assertEqual(result['original_qtcore_sha256'],digest(core))
                    self.assertEqual(result['replaced_signed_qtcore_sha256'],digest(expected))
                    self.assertEqual(result['app_sha256'],digest(binary))


if __name__ == '__main__':
    unittest.main()
