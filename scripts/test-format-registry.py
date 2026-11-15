#!/usr/bin/env python3
"""Regression checks for format admission, existing entry points and stale generation."""
from contextlib import redirect_stdout
from copy import deepcopy
import importlib.util
import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location('format_registry', ROOT / 'scripts/generate-format-registry.py')
REGISTRY = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(REGISTRY)


class RegistryTests(unittest.TestCase):
    def setUp(self):
        self.source = json.loads(REGISTRY.SOURCE.read_text(encoding='utf-8'))

    def reject(self, change):
        change(self.source)
        with self.assertRaises(ValueError):
            REGISTRY.render(self.source)

    def test_current_entry_candidates_and_protocol_ids(self):
        formats = REGISTRY.validate(self.source)
        entries = [entry for item in formats for entry in item['extensions']]
        self.assertEqual({entry['suffix'] for entry in entries if entry['drop']},
                         {'.zip', '.zipx', '.xpi', '.apk', '.ipa', '.cbz', '.tar', '.tar.gz', '.tgz', '.gz', '.7z', '.rar', '.cab', '.iso', '.udf', '.bz2', '.bz', '.tar.bz2', '.tar.bz', '.tbz', '.tbz2', '.lzma', '.tar.lzma', '.tlz', '.lz', '.tar.lz', '.xz', '.tar.xz', '.txz', '.lz4', '.tar.lz4', '.zst', '.tar.zst', '.tzst', '.deb', '.asar', '.uu', '.uue', '.xxe', '.z', '.tar.z', '.taz', '.aes', '.br', '.tar.br', '.lha', '.lzh', '.pma'})
        self.assertEqual({entry['suffix'] for entry in entries if entry['shell']},
                         {'.zip', '.zipx', '.xpi', '.apk', '.ipa', '.cbz', '.tar', '.tar.gz', '.tgz', '.gz', '.7z', '.rar', '.cab', '.iso', '.udf', '.bz2', '.bz', '.tar.bz2', '.tar.bz', '.tbz', '.tbz2', '.lzma', '.tar.lzma', '.tlz', '.lz', '.tar.lz', '.xz', '.tar.xz', '.txz', '.lz4', '.tar.lz4', '.zst', '.tar.zst', '.tzst', '.deb', '.asar', '.uu', '.uue', '.xxe', '.z', '.tar.z', '.taz', '.aes', '.br', '.tar.br', '.lha', '.lzh', '.pma'})
        self.assertEqual({item['id']: item['worker_id'] for item in formats if item['worker_id']},
                         {'7z': 7, 'rar': 4, 'cab': 8, 'iso': 9, 'udf': 10})
        self.assertEqual({suffix for item in formats for suffix in item['creation_suffixes']},
                         {'.zip', '.tar', '.tar.gz', '.tgz', '.gz', '.7z', '.xz', '.tar.xz', '.txz', '.lz4', '.tar.lz4', '.zst', '.tar.zst', '.tzst', '.lzh', '.lha'})

    def test_duplicate_id(self):
        self.reject(lambda source: source['formats'][1].update(id='zip'))

    def test_ui_choices_preserve_stable_ids_and_independent_gzip_capabilities(self):
        output = REGISTRY.render(self.source)['apps/desktop/ui/ArchiveFormats.js']
        choices = json.loads(output.split('var creationFormats = ', 1)[1].split(';\n', 1)[0])
        by_id = {item['id']: item for item in choices}
        self.assertEqual(set(by_id), {'zip', 'tar', 'tar-gzip', '7z', 'gzip', 'xz', 'tar-xz', 'lz4', 'tar-lz4', 'zstd', 'tar-zstd', 'lzh'})
        self.assertEqual(by_id['lzh']['suffix'], '.lzh')
        self.assertEqual(by_id['lzh']['methods'], ['lh5'])
        self.assertEqual(by_id['lzh']['methodIds'], [259])
        self.assertEqual(by_id['lzh']['levels'], [1, 6, 9])
        self.assertEqual(by_id['lzh']['encryption'], [])
        self.assertEqual(by_id['lzh']['volumes'], [])
        self.assertEqual(by_id['tar-gzip']['suffix'], '.tar.gz')
        self.assertEqual(by_id['gzip']['suffix'], '.gz')
        self.assertEqual(by_id['gzip']['methods'], ['deflate'])
        self.assertEqual(by_id['gzip']['encryption'], [])
        self.assertEqual(by_id['gzip']['volumes'], [])

    def test_linux_mime_declarations_match_registered_candidates(self):
        declared = next(line.removeprefix('MimeType=') for line in
                        (ROOT / 'apps/desktop/packaging/linux/xdzip.desktop').read_text().splitlines()
                        if line.startswith('MimeType=')).strip(';').split(';')
        expected = [mime for item in self.source['formats'] for mime in item['shell_mime_types']]
        self.assertEqual(set(declared), set(expected))
        self.assertEqual(len(declared), len(set(declared)))

    def test_boolean_is_not_schema_version(self):
        self.reject(lambda source: source.update(schema_version=True))

    def test_creation_identifiers_and_levels_are_shared_and_scoped(self):
        self.assertEqual(self.source['creation_methods']['lz4'], 256)
        by_id = {item['id']: item for item in self.source['formats']}
        self.assertEqual(by_id['lz4']['creation_levels'], [1, 6, 9])
        self.assertEqual(by_id['gzip']['creation_levels'], [0, 1, 6, 9])
        for change in [
            lambda source: source['creation_methods'].update(lz4=True),
            lambda source: source['creation_methods'].update(lz4=65536),
            lambda source: source['creation_methods'].update(deflate=0),
            lambda source: source['creation_methods'].pop('lz4'),
            lambda source: source['formats'][0].update(creation_levels=[True]),
            lambda source: source['formats'][0].update(creation_levels=[6, 6]),
            lambda source: source['formats'][0].update(creation_levels=[]),
            lambda source: source['formats'][-1].update(creation_levels=[6]),
        ]:
            candidate = deepcopy(self.source)
            change(candidate)
            with self.assertRaises(ValueError):
                REGISTRY.render(candidate)

    def test_duplicate_variant(self):
        self.reject(lambda source: source['formats'][1].update(rust_variant='Zip'))

    def test_worker_identity_collision(self):
        self.reject(lambda source: source['formats'][4].update(worker_id=7))

    def test_boolean_is_not_worker_id(self):
        self.reject(lambda source: source['formats'][4].update(worker_id=True))

    def test_rejects_creation_without_verifier(self):
        self.reject(lambda source: source['formats'][0].update(write_methods=['unknown']))

    def test_rejects_unreadable_encryption(self):
        self.reject(lambda source: source['formats'][0].update(write_encryption=['unknown']))

    def test_rejects_unreadable_volumes(self):
        self.reject(lambda source: source['formats'][0].update(write_volumes=['unknown']))

    def test_check_strength_distinguishes_formats_without_content_checksums(self):
        by_id = {item['id']: item for item in self.source['formats']}
        for identity in ['tar', 'lzma', 'tar-lzma', 'iso', 'udf']:
            self.assertEqual(by_id[identity]['check_kind'], 'structure_and_readability')
        for identity in ['gzip', 'bzip2', 'tar-bzip2', 'lzip', 'tar-lzip']:
            self.assertEqual(by_id[identity]['check_kind'], 'supported_content')
        self.reject(lambda source: source['formats'][0].update(check_kind='unknown'))

    def test_shared_macos_artwork_is_inventoried_once(self):
        output = REGISTRY.render(self.source)['scripts/archive_formats_generated.py']
        self.assertIn("'archive'", output)
        import ast
        icons = ast.literal_eval(output.split('MACOS_ICONS = ', 1)[1].split('\n', 1)[0])
        self.assertEqual(icons.count('archive'), 1)

    def test_rejects_undeclared_creation_suffix(self):
        self.reject(lambda source: source['formats'][0].update(creation_suffixes=['.exe']))

    def test_rejects_ambiguous_suffix(self):
        self.reject(lambda source: source['formats'][1]['extensions'][0].update(suffix='.zip'))

    def test_rejects_code_injection_and_paths(self):
        for suffix in ('.zip/../../target', '.zip";', '.ZIP', 'zip', '.'):
            source = deepcopy(self.source)
            source['formats'][0]['extensions'][0]['suffix'] = suffix
            with self.assertRaises(ValueError):
                REGISTRY.render(source)
        self.reject(lambda source: source['formats'][0].update(rust_variant='Zip; panic!()'))

    def test_rejects_excessive_layers(self):
        self.reject(lambda source: source['formats'][2].update(filters=['gzip', 'xz', 'bzip2', 'zstd']))

    def test_check_detects_missing_and_changed_consumers(self):
        outputs = REGISTRY.render(self.source)
        parent = ROOT / 'build/test-tmp'
        parent.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(prefix='format-registry-', dir=parent) as temporary:
            root = Path(temporary)
            with patch.object(REGISTRY, 'ROOT', root), patch('sys.argv', ['generator', '--check']):
                with self.assertRaisesRegex(SystemExit, 'Regenerate format declarations'):
                    REGISTRY.main()
                for name, content in outputs.items():
                    target = root / name
                    target.parent.mkdir(parents=True, exist_ok=True)
                    target.write_text(content, encoding='utf-8', newline='\n')
                with redirect_stdout(io.StringIO()):
                    REGISTRY.main()
                modified = {name: (root / name).stat().st_mtime_ns for name in outputs}
                with patch('sys.argv', ['generator']), redirect_stdout(io.StringIO()):
                    REGISTRY.main()
                self.assertEqual(modified, {name: (root / name).stat().st_mtime_ns for name in outputs})
                name = 'apps/desktop/ArchiveFormatsGenerated.h'
                (root / name).write_text(outputs[name] + '// stale\n', encoding='utf-8')
                with self.assertRaisesRegex(SystemExit, name):
                    REGISTRY.main()


if __name__ == '__main__':
    unittest.main()
