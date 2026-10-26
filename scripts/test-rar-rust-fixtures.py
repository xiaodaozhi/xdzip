"""Offline bounds/identity tests for the fixed RAR CI data preparer."""
import binascii
import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('rar_prepare', ROOT / 'scripts/prepare-rar-rust-fixtures.py')
prepare = importlib.util.module_from_spec(spec)
spec.loader.exec_module(prepare)
interop_spec = importlib.util.spec_from_file_location('rar_interop', ROOT / 'scripts/check-rar-rust-interop.py')
interop = importlib.util.module_from_spec(interop_spec)
interop_spec.loader.exec_module(interop)


class FixedRarData(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        (ROOT / 'build').mkdir(exist_ok=True)

    def test_output_is_dedicated_build_directory(self):
        for path in [ROOT, ROOT / 'build', ROOT / 'build/../scripts']:
            with self.assertRaises(ValueError):
                prepare.build_output(path)
        self.assertEqual(prepare.build_output(ROOT / 'build/rar-test'), ROOT / 'build/rar-test')

    def test_fixed_cache_never_replaces_changed_data(self):
        with tempfile.TemporaryDirectory(dir=ROOT / 'build') as directory:
            path = Path(directory) / 'fixed.rar'
            prepare.fixed_file(path, b'fixed')
            prepare.fixed_file(path, b'fixed')
            with self.assertRaises(ValueError):
                prepare.fixed_file(path, b'other')
            self.assertEqual(path.read_bytes(), b'fixed')

    def test_uu_is_bounded_and_has_exact_envelope(self):
        self.assertEqual(prepare.decode_uu(b'begin 644 fixture\n' + binascii.b2a_uu(b'fixed') + b'`\nend\n'), b'fixed')
        for bad in [b'begin 644 fixture\n', b'garbage', b'x' * (prepare.MAX_DOWNLOAD + 1),
                    b'begin 644 a\nbegin 644 b\nend\n']:
            with self.assertRaises(ValueError):
                prepare.decode_uu(bad)

    def test_public_cache_checks_sha_and_never_uses_network_offline(self):
        with tempfile.TemporaryDirectory(dir=ROOT / 'build') as directory:
            path = Path(directory)
            item = dict(file='fixture.rar', bytes=5, sha256=hashlib.sha256(b'fixed').hexdigest())
            with patch.object(prepare.urllib.request, 'urlopen', side_effect=AssertionError('unexpected network')):
                with self.assertRaises(ValueError):
                    prepare.public_data(item, path, offline=True)
                (path / 'fixture.rar').write_bytes(b'fixed')
                self.assertEqual(prepare.public_data(item, path, offline=True), (b'fixed', 0))
                (path / 'fixture.rar').write_bytes(b'other')
                with self.assertRaises(ValueError):
                    prepare.public_data(item, path, offline=True)

    def test_unknown_metadata_and_empty_store_are_distinct(self):
        unknown, empty = prepare.mathematical_store(True), prepare.mathematical_store(False)
        self.assertNotEqual(unknown, empty)
        self.assertLess(len(unknown), 100)
        self.assertLess(len(empty), 100)
        self.assertTrue(unknown.startswith(b'Rar!\x1a\x07\x01\x00'))

    def test_catalogue_is_fixed_small_https_data(self):
        catalogue = json.loads((ROOT / 'scripts/rar-rust-fixtures.json').read_text(encoding='utf-8'))
        self.assertEqual(len(catalogue['public']), 3)
        for item in catalogue['public']:
            self.assertEqual(Path(item['file']).name, item['file'])
            self.assertLessEqual(item['bytes'], prepare.MAX_ARCHIVE)
            self.assertRegex(item['sha256'], r'^[0-9a-f]{64}$')
            self.assertTrue(item['url'].startswith('https://raw.githubusercontent.com/'))
            self.assertTrue(item['notice'])
            if 'encoded_sha256' in item:
                self.assertRegex(item['encoded_sha256'], r'^[0-9a-f]{64}$')

    def test_public_unicode_password_material_is_small_pinned_and_never_serializes_key(self):
        items = prepare.PUBLIC_COMPATIBILITY
        self.assertEqual(len(items), 4)
        self.assertEqual(sum(item['bytes'] for item in items if item['file'].endswith('.rar')), 384)
        for item in items:
            self.assertEqual(Path(item['file']).name, item['file'])
            self.assertLessEqual(item['bytes'], 4096)
            self.assertRegex(item['sha256'], r'^[0-9a-f]{64}$')
            self.assertIn('/57091f9ccd43661cf8f12c389917cc24950df707/', item['url'])
            self.assertEqual(item['encoding'], 'raw')
            self.assertIn('UnRAR terms', item['notice'])
            self.assertNotIn('password', item)
        self.assertTrue(any(item['source'] == 'LICENSE' for item in items))

    def test_os_probe_uses_only_fixed_headers_and_a_unique_release_archive(self):
        with tempfile.TemporaryDirectory(dir=ROOT / 'build') as directory:
            worker = Path(directory) / 'worker'
            include = worker.parent / 'rust/cxxbridge'
            for name in ['xdzip-codec-bridge/src/lib.rs.h', 'rust/cxx.h']:
                path = include / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(b'fixed-header')
            with self.assertRaises(ValueError):
                interop.bridge_inputs(worker)
            library = worker.parent / 'rust/native/release/xdzip_codec_bridge.lib'
            library.parent.mkdir(parents=True)
            library.write_bytes(b'fixed-archive')
            self.assertEqual(interop.bridge_inputs(worker)[:2], (include, library))
            other = worker.parent / 'rust/other/release/libxdzip_codec_bridge.a'
            other.parent.mkdir(parents=True)
            other.write_bytes(b'other-archive')
            with self.assertRaises(ValueError):
                interop.bridge_inputs(worker)
            other.unlink()
            (include / 'rust/cxx.h').unlink()
            with self.assertRaises(ValueError):
                interop.bridge_inputs(worker)


if __name__ == '__main__':
    unittest.main()
