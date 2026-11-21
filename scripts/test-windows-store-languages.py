#!/usr/bin/env python3
"""Check that archived partial-language captures remain valid without inventing views."""
import copy
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
from zipfile import ZipFile

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('store_package', ROOT / 'scripts/package-windows-store-materials.py')
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
refresh_spec = importlib.util.spec_from_file_location('store_refresh', ROOT / 'scripts/refresh-windows-store-screenshots.py')
refresh = importlib.util.module_from_spec(refresh_spec)
refresh_spec.loader.exec_module(refresh)


class StoreLanguages(unittest.TestCase):
    def setUp(self):
        (ROOT / 'build').mkdir(exist_ok=True)
        self.temp = tempfile.TemporaryDirectory(prefix='test-store-languages-', dir=ROOT / 'build')
        self.addCleanup(self.temp.cleanup)
        self.base = Path(self.temp.name)
        self.source = self.base / 'source'
        self.marketing = module.read_json(ROOT / module.MARKETING)
        self.images = []
        for language in ('en-US', 'zh-CN', 'ja-JP'):
            for scene in self.marketing['locales'][language]['screenshot_captions']:
                native = Path('design/assets/marketing/windows-store-native') / language / (scene + '.png')
                cutout = module.store_base(language) / 'images/screenshots' / (scene + '.png')
                data = f'fixture {language} {scene}'.encode()
                for file in (native, cutout):
                    target = self.source / file
                    target.parent.mkdir(parents=True, exist_ok=True)
                    target.write_bytes(data)
                self.images.append({'locale': language, 'scene': scene, 'native_file': native.as_posix(),
                                    'file': cutout.as_posix(), 'cutout_file': cutout.as_posix(),
                                    'source_sha256': module.digest(data), 'cutout_sha256': module.digest(data)})
        self.write_source()

    def write_source(self):
        baseline = self.marketing['baseline']
        receipt = {'schema_version': 3, 'window_mode': 'default', 'baseline_revision': baseline['revision'],
                   'package_sha256': baseline['package_sha256'], 'runtime_exe_sha256': baseline['runtime_exe_sha256'],
                   'images': self.images}
        module.write_json(self.source / module.MARKETING, self.marketing)
        module.write_json(self.source / module.CAPTURES, receipt)

    def test_old_bilingual_and_new_language_capture_sets(self):
        for languages in ({'en-US', 'zh-CN'}, {'en-US', 'zh-CN', 'ja-JP'}):
            original = self.images
            self.images = [item for item in original if item['locale'] in languages]
            self.write_source()
            result, files = module.stage_capture_source(self.source, self.base / ('stage-' + str(len(languages))))
            self.assertEqual(len(result['locales']), 10)
            self.assertEqual(len(files), 2 + 8 * len(languages))
            self.images = original

    def test_ten_language_capture_uses_distinct_creation_directories(self):
        package = self.base / 'fixture.zip'
        with ZipFile(package, 'w') as archive:
            archive.writestr('xdzip.exe', b'fixture')
        commands = []
        with patch.object(refresh.subprocess, 'run', side_effect=lambda command, **kwargs: commands.append(command)):
            refresh.capture_languages(package, {'runtime_exe_sha256': module.digest(b'fixture'), 'revision': 'a' * 40},
                                      self.base / 'capture', self.base / 'config', refresh.LANGUAGES)
        self.assertEqual(len(commands), 10)
        paths = [command[command.index('-CreationDemoDirectory') + 1] for command in commands]
        self.assertEqual(len(paths), len(set(paths)))

    def test_incomplete_duplicate_and_tampered_capture_rejected(self):
        original = copy.deepcopy(self.images)
        for images in (original[:-1], original + [original[0]]):
            self.images = images
            self.write_source()
            with self.assertRaisesRegex(ValueError, 'every scene'):
                module.stage_capture_source(self.source, self.base / 'bad')
        self.images = original
        self.write_source()
        (self.source / original[0]['native_file']).write_bytes(b'tampered')
        with self.assertRaisesRegex(ValueError, 'digest'):
            module.stage_capture_source(self.source, self.base / 'tampered')


if __name__ == '__main__':
    unittest.main()
