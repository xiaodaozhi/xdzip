#!/usr/bin/env python3
"""Focused checks for the shared native screenshot recording contract."""
import importlib.util
import contextlib
import io
import json
from pathlib import Path
import struct
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch
import zlib

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('capture_platform', ROOT / 'scripts/capture-platform-store-assets.py')
capture = importlib.util.module_from_spec(spec)
spec.loader.exec_module(capture)


def chunk(kind, data):
    return struct.pack('>I', len(data)) + kind + data + struct.pack('>I', zlib.crc32(kind + data))


def png():
    return (b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', 2, 2, 8, 2, 0, 0, 0))
            + chunk(b'IDAT', zlib.compress(b'\0' + b'\xff\0\0' * 2 + b'\0' + b'\0\xff\0' * 2))
            + chunk(b'IEND', b''))


class CaptureTests(unittest.TestCase):
    def setUp(self):
        (ROOT / 'build').mkdir(exist_ok=True)
        self.temp = tempfile.TemporaryDirectory(prefix='test-platform-capture-', dir=ROOT / 'build')
        self.addCleanup(self.temp.cleanup)
        self.base = Path(self.temp.name)
        self.exe = self.base / 'xdzip-test'
        self.exe.write_bytes(b'test executable identity')
        self.source = self.base / 'input'
        for language in capture.LANGUAGES:
            (self.source / language).mkdir(parents=True)
            for scene in capture.SCENES:
                (self.source / language / (scene + '.png')).write_bytes(png())
        self.args = SimpleNamespace(platform='macos', executable=self.exe,
                                    expected_sha256=capture.sha256(self.exe.read_bytes()),
                                    source_revision='a' * 40, from_directory=self.source,
                                    output=self.base / 'recorded', delay=5)
        self.identity = {'product': 'xdzip', 'version': '0.1', 'build': 1,
                         'dirty': False, 'revision': 'a' * 12}

    def record(self, host='darwin'):
        with contextlib.redirect_stdout(io.StringIO()), patch.object(capture.sys, 'platform', host), patch.object(capture.subprocess, 'run',
                return_value=SimpleNamespace(stdout=json.dumps(self.identity))):
            capture.capture(self.args)

    def test_platform_specs_match_capture_contract(self):
        for target in ('macos', 'linux'):
            config = json.loads((ROOT / 'specs/marketing' / (target + '-store.json')).read_text())
            self.assertEqual(config['platform'], target)
            self.assertEqual(config['scenes'], list(capture.SCENES))
            self.assertEqual(set(config['locales']), set(capture.LANGUAGES))
            self.assertEqual(config['visual_source'], 'specs/marketing/windows-store.json')
            for language in capture.LANGUAGES:
                locale = config['locales'][language]
                self.assertEqual([item['id'] for item in locale['promotions']], list(capture.SCENES))
                self.assertEqual(set(locale['screenshot_captions']), set(capture.SCENES))
                self.assertTrue(locale['description'])
                self.assertEqual(len(locale['features']), 10)

    def test_both_platforms_preserve_inputs_and_identity(self):
        for platform, host in [('macos', 'darwin'), ('linux', 'linux')]:
            self.args.platform = platform
            self.args.output = self.base / platform
            self.record(host)
            receipt = json.loads((self.args.output / 'capture-session.json').read_text())
            self.assertEqual(receipt['platform'], platform)
            self.assertEqual(receipt['provenance'], 'operator-supplied')
            self.assertEqual(receipt['identity']['source_revision'], 'a' * 40)
            self.assertEqual(len(receipt['images']), 4 * len(capture.LANGUAGES))
            for image in receipt['images']:
                data = (self.args.output / image['file']).read_bytes()
                self.assertEqual(data, (self.source / image['file']).read_bytes())
                self.assertEqual(image['sha256'], capture.sha256(data))
            self.assertNotIn(str(self.base), json.dumps(receipt))

    def test_interactive_native_commands_and_receipt(self):
        for target, host in [('macos', 'darwin'), ('linux', 'linux')]:
            self.args.platform = target
            self.args.from_directory = None
            self.args.output = self.base / ('interactive-' + target)
            commands = []

            def run(command, **kwargs):
                if command[-1] == '--version-json':
                    return SimpleNamespace(stdout=json.dumps(self.identity))
                commands.append(command)
                destination = command[-1].removeprefix('--file=')
                Path(destination).write_bytes(png())
                return SimpleNamespace()

            with contextlib.redirect_stdout(io.StringIO()), patch.object(capture.sys, 'platform', host), patch.object(capture.subprocess, 'run', side_effect=run), \
                    patch.object(capture.shutil, 'which', return_value='/available'), patch('builtins.input', return_value=''):
                capture.capture(self.args)
            self.assertEqual(len(commands), 4 * len(capture.LANGUAGES))
            self.assertEqual(commands[0][0], 'screencapture' if target == 'macos' else 'gnome-screenshot')
            if target == 'macos':
                self.assertIn('-o', commands[0])
            receipt = json.loads((self.args.output / 'capture-session.json').read_text())
            self.assertEqual(receipt['capture_method'], 'interactive-window')

    def test_selected_languages_are_complete(self):
        self.args.languages = ['ja-JP']
        self.record()
        receipt = json.loads((self.args.output / 'capture-session.json').read_text())
        self.assertEqual(len(receipt['images']), 4)
        self.assertTrue(all(image['language'] == 'ja-JP' for image in receipt['images']))

    def test_corruption_and_missing_files_do_not_leave_output(self):
        item = self.source / 'zh-CN/04-workspace.png'
        item.write_bytes(png()[:-1] + b'X')
        with self.assertRaises(ValueError):
            self.record()
        self.assertFalse(self.args.output.exists())
        item.unlink()
        with self.assertRaises(FileNotFoundError):
            self.record()
        self.assertFalse(self.args.output.exists())

    def test_identity_and_platform_rejected(self):
        self.args.expected_sha256 = '0' * 64
        with self.assertRaisesRegex(ValueError, 'digest'):
            self.record()
        self.args.expected_sha256 = capture.sha256(self.exe.read_bytes())
        self.args.source_revision = 'b' * 40
        with self.assertRaisesRegex(ValueError, 'revision'):
            self.record()
        with self.assertRaisesRegex(ValueError, 'native platform'):
            self.record('linux')

    def test_existing_output_is_not_overwritten(self):
        self.args.output.mkdir()
        (self.args.output / 'keep').write_text('keep')
        with self.assertRaisesRegex(ValueError, 'already exists'):
            self.record()
        self.assertEqual((self.args.output / 'keep').read_text(), 'keep')


if __name__ == '__main__':
    unittest.main()
