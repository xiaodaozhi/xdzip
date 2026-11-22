#!/usr/bin/env python3
"""Check version contract boundaries without touching repository metadata."""
import json
from pathlib import Path
import runpy
import shutil
import subprocess
from tempfile import TemporaryDirectory
import unittest

from build_identity import source_identity

ROOT = Path(__file__).resolve().parents[1]
class VersionTests(unittest.TestCase):
    def evaluate(self, value):
        module = runpy.run_path(str(ROOT / 'scripts/sync-version.py'))
        with TemporaryDirectory() as temp:
            (Path(temp) / 'VERSION.json').write_text(json.dumps(value))
            module['version_info'].__globals__['ROOT'] = Path(temp)
            return module['version_info']()
    @unittest.skipUnless(shutil.which('cmake'), 'CMake is unavailable')
    def test_archive_without_git_has_unknown_provenance(self):
        with TemporaryDirectory() as temp:
            subprocess.run(['cmake', '-DXDZIP_GIT=OFF', f'-DXDZIP_SOURCE_DIR={ROOT}',
                f'-DXDZIP_OUTPUT_DIR={temp}', '-DXDZIP_DISPLAY_VERSION=1.2.3-dev.2',
                '-DXDZIP_BUILD_NUMBER=2', '-DXDZIP_CHANNEL=development', '-DXDZIP_CONFIGURATION=Debug',
                '-P', str(ROOT/'cmake/BuildVersion.cmake')], check=True, capture_output=True)
            header=(Path(temp)/'BuildVersion.h').read_text()
            self.assertIn('#define XDZIP_REVISION "unknown"', header)
            self.assertIn('#define XDZIP_DIRTY false', header)
    @unittest.skipUnless(shutil.which('cmake') and shutil.which('git'), 'CMake/Git are unavailable')
    def test_source_archive_does_not_inherit_parent_repository(self):
        (ROOT/'build').mkdir(exist_ok=True)
        with TemporaryDirectory(dir=ROOT/'build') as temp:
            source=Path(temp)/'source';(source/'cmake').mkdir(parents=True)
            shutil.copyfile(ROOT/'cmake/BuildVersion.h.in',source/'cmake/BuildVersion.h.in')
            output=Path(temp)/'output'
            subprocess.run(['cmake', f'-DXDZIP_SOURCE_DIR={source}', f'-DXDZIP_OUTPUT_DIR={output}',
                '-DXDZIP_DISPLAY_VERSION=1.2.3-dev.2', '-DXDZIP_BUILD_NUMBER=2',
                '-DXDZIP_CHANNEL=development', '-DXDZIP_CONFIGURATION=Debug',
                '-P',str(ROOT/'cmake/BuildVersion.cmake')],check=True,capture_output=True)
            self.assertIn('#define XDZIP_REVISION "unknown"',(output/'BuildVersion.h').read_text())
    @unittest.skipUnless(shutil.which('git'), 'Git is unavailable')
    def test_source_identity_tracks_untracked_unicode_paths_and_commit_changes(self):
        with TemporaryDirectory() as temp:
            root = Path(temp)

            def git(*args):
                return subprocess.check_output(['git', *args], cwd=root, text=True).strip()

            def commit(title):
                git('add', '.')
                git('-c', 'user.name=Build identity test', '-c', 'user.email=ci@example.invalid',
                    'commit', '-qm', title)

            git('init', '-q')
            path = root / '版本 示例.txt'
            path.write_text('baseline', encoding='utf-8')
            commit('baseline')
            baseline = source_identity(root)
            self.assertEqual(baseline, {'revision': git('rev-parse', '--short=12', 'HEAD'), 'dirty': False})
            note = root / '未跟踪 文件.txt'
            note.write_text('new', encoding='utf-8')
            self.assertEqual(source_identity(root), {**baseline, 'dirty': True})
            note.unlink()
            self.assertEqual(source_identity(root), baseline)
            path.write_text('changed', encoding='utf-8')
            commit('changed')
            self.assertNotEqual(source_identity(root)['revision'], baseline['revision'])
            self.assertFalse(source_identity(root)['dirty'])
    def test_dependency_versions_are_preserved(self):
        module = runpy.run_path(str(ROOT / 'scripts/sync-version.py'))
        source = '[package]\nname = "example"\nversion = "0.1.0"\n[dependencies.other]\nversion = "2.3.4"\n'
        result = module['set_version'](source, 'package', 'version.workspace = true')
        self.assertIn('version.workspace = true', result)
        self.assertIn('[dependencies.other]\nversion = "2.3.4"', result)
    def test_channel_mapping(self):
        for channel, suffix in [('development','-dev.2'),('alpha','-alpha.2'),('beta','-beta.2'),('rc','-rc.2'),('stable','')]:
            with self.subTest(channel=channel):
                self.assertEqual(self.evaluate(dict(version='1.2.3',channel=channel,build=2))['display_version'],'1.2.3'+suffix)
    def test_prerelease_sequence_is_independent(self):
        self.assertEqual(self.evaluate(dict(version='0.1.0',channel='alpha',build=14,prerelease_number=1))['display_version'], '0.1.0-alpha.1')
        for value in [0, True, '1', 65536]:
            with self.assertRaises(ValueError):
                self.evaluate(dict(version='0.1.0',channel='alpha',build=14,prerelease_number=value))
    def test_reject_invalid_os_and_semver_values(self):
        for key,value in [('version','01.2.3'),('version','1.2'),('version','65536.0.0'),('version','1.2.3-beta'),('build',0),('build',65536),('build',True),('build',1.5),('channel','release')]:
            with self.subTest(key=key,value=value):
                data=dict(version='1.2.3',channel='development',build=1);data[key]=value
                with self.assertRaises(ValueError): self.evaluate(data)
if __name__ == '__main__':unittest.main()
