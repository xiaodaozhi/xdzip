#!/usr/bin/env python3
"""Check document discovery against tracked, new and generated files."""
import importlib.util
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('check_docs', ROOT / 'scripts/check-docs.py')
check_docs = importlib.util.module_from_spec(spec)
spec.loader.exec_module(check_docs)


class DocumentDiscoveryTests(unittest.TestCase):
    def test_tracked_and_new_documents_exclude_generated_and_ignored_trees(self):
        parent = ROOT / 'build/test-check-docs'
        parent.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=parent) as temporary:
            root = Path(temporary)
            subprocess.run(['git', 'init', '-q'], cwd=root, check=True)
            (root / '.gitignore').write_text('/build/\n/ignored.md\n', encoding='utf-8')
            (root / 'tracked.md').write_text('tracked', encoding='utf-8')
            (root / 'deleted.md').write_text('deleted', encoding='utf-8')
            subprocess.run(['git', 'add', '.gitignore', 'tracked.md', 'deleted.md'], cwd=root, check=True)
            (root / 'deleted.md').unlink()
            (root / 'new.md').write_text('new', encoding='utf-8')
            (root / 'ignored.md').write_text('ignored', encoding='utf-8')
            (root / 'build').mkdir()
            (root / 'build/generated.md').write_text('generated', encoding='utf-8')
            subprocess.run(['git', 'add', '-f', 'build/generated.md'], cwd=root, check=True)
            found = {path.relative_to(root).as_posix() for path in check_docs.markdown_documents(root)}
            self.assertEqual(found, {'tracked.md', 'new.md'})

    def test_language_selectors_are_reciprocal_in_both_trees(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            files = []
            for tree in ('docs', 'developers'):
                for name, contents in (
                    ('guide/README.md', '# Guide\n\nEnglish | [简体中文](../zh-CN/guide/README.md)\n'),
                    ('zh-CN/guide/README.md', '# 指南\n\n[English](../../guide/README.md) | 简体中文\n'),
                ):
                    path = root / tree / name
                    path.parent.mkdir(parents=True)
                    path.write_text(contents, encoding='utf-8')
                    files.append(path)

            self.assertEqual(check_docs.bilingual_documents(root, files), (2, []))
            files[-1].write_text('# 指南\n\n[English](../../guide/README.md) | [简体中文](README.md)\n',
                                 encoding='utf-8')
            pairs, failures = check_docs.bilingual_documents(root, files)
            self.assertEqual(pairs, 2)
            self.assertEqual(len(failures), 1)
            self.assertIn('developers/zh-CN/guide/README.md', failures[0])
            pairs, failures = check_docs.bilingual_documents(root, files[:-1])
            self.assertEqual(pairs, 1)
            self.assertIn('missing language counterpart', failures[0])


if __name__ == '__main__':
    unittest.main()
