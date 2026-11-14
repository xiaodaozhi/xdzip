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
                         {'.zip', '.xpi', '.apk', '.ipa', '.cbz', '.tar', '.tar.gz', '.tgz', '.gz', '.7z', '.rar', '.cab', '.iso', '.udf'})
        self.assertEqual({entry['suffix'] for entry in entries if entry['shell']},
                         {'.zip', '.xpi', '.apk', '.ipa', '.cbz', '.tar', '.tar.gz', '.tgz', '.gz', '.7z', '.rar', '.cab', '.iso', '.udf'})
        self.assertEqual({item['id']: item['worker_id'] for item in formats if item['worker_id']},
                         {'7z': 7, 'rar': 4, 'cab': 8, 'iso': 9, 'udf': 10})
        self.assertEqual({suffix for item in formats for suffix in item['creation_suffixes']},
                         {'.zip', '.tar', '.tar.gz', '.tgz', '.gz', '.7z'})

    def test_duplicate_id(self):
        self.reject(lambda source: source['formats'][1].update(id='zip'))

    def test_ui_choices_preserve_stable_ids_and_independent_gzip_capabilities(self):
        output = REGISTRY.render(self.source)['apps/desktop/ui/ArchiveFormats.js']
        choices = json.loads(output.split('var creationFormats = ', 1)[1].split(';\n', 1)[0])
        by_id = {item['id']: item for item in choices}
        self.assertEqual(set(by_id), {'zip', 'tar', 'tar-gzip', '7z', 'gzip'})
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
