#!/usr/bin/env python3
"""Regressions for document discovery, bilingual navigation and local links."""
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('check_docs', ROOT / 'scripts/check-docs.py')
check_docs = importlib.util.module_from_spec(spec)
spec.loader.exec_module(check_docs)
BILINGUAL = {'en-US': 'English', 'zh-CN': '简体中文'}


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


class DocumentTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name).resolve()

    def write(self, name, contents):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(contents, encoding='utf-8')
        return path

    def root_pairs(self):
        files = [
            self.write('README.md', '<h1>XDZip</h1>\n<p>Introduction</p>\n'
                       '<p align="center">English | <a href="README.zh-CN.md">简体中文</a></p>\n'),
            self.write('README.zh-CN.md', '<h1>XDZip</h1>\n<p>介绍</p>\n'
                       '<p align="center"><a href="README.md">English</a> | 简体中文</p>\n'),
        ]
        for topic in ('CHANGELOG', 'CODE_OF_CONDUCT', 'CONTRIBUTING', 'GOVERNANCE', 'PRIVACY', 'SECURITY'):
            files.append(self.write(f'{topic}.md', f'# Title\n\nEnglish | [简体中文]({topic}.zh-CN.md)\n'))
            files.append(self.write(f'{topic}.zh-CN.md', f'# 标题\n\n[English]({topic}.md) | 简体中文\n'))
        return files

    def test_hifi_catalog_checks_artwork_digest_and_optional_source(self):
        image = self.write('design/hifi/screens/test.png', 'artwork')
        source = self.write('design/hifi/reference/test.svg', 'source')
        catalog = self.root / 'design/hifi/catalog.json'
        record = {'screens': [{'file': 'screens/test.png', 'source': 'reference/test.svg',
                               'sha256': hashlib.sha256(image.read_bytes()).hexdigest()}]}
        catalog.write_text(json.dumps(record), encoding='utf-8')
        self.assertEqual(check_docs.hifi_catalog(self.root), [])
        source.unlink()
        image.write_text('changed', encoding='utf-8')
        failures = check_docs.hifi_catalog(self.root)
        self.assertTrue(any('source missing' in failure for failure in failures))
        self.assertTrue(any('digest mismatch' in failure for failure in failures))

    def test_language_selectors_are_reciprocal_in_both_trees(self):
        files = self.root_pairs()
        for tree in ('docs', 'developers'):
            files.append(self.write(f'{tree}/guide/README.md',
                                    '# Guide\n\nEnglish | [简体中文](../zh-CN/guide/README.md)\n'))
            files.append(self.write(f'{tree}/zh-CN/guide/README.md',
                                    '# 指南\n\n[English](../../guide/README.md) | 简体中文\n'))

        self.assertEqual(check_docs.localized_documents(self.root, files, BILINGUAL), (9, []))
        self.write('developers/zh-CN/guide/README.md',
                   '# 指南\n\n[English](../../guide/README.md) | [简体中文](README.md)\n')
        pairs, failures = check_docs.localized_documents(self.root, files, BILINGUAL)
        self.assertEqual(pairs, 9)
        self.assertEqual(len(failures), 1)
        self.assertIn('developers/zh-CN/guide/README.md', failures[0])
        pairs, failures = check_docs.localized_documents(self.root, files[:-1], BILINGUAL)
        self.assertEqual(pairs, 8)
        self.assertIn('missing language counterpart', failures[0])

    def test_root_requires_both_versions_and_recognizes_new_localized_pages(self):
        files = self.root_pairs()
        self.assertEqual(check_docs.localized_documents(self.root, files, BILINGUAL), (7, []))
        files = [path for path in files if not path.name.startswith('SECURITY.')]
        files.append(self.write('FAQ.zh-CN.md', '# 说明\n\n[English](FAQ.md) | 简体中文\n'))
        pairs, failures = check_docs.localized_documents(self.root, files, BILINGUAL)
        self.assertEqual(pairs, 6)
        self.assertEqual(failures, [
            'FAQ.md: missing language counterpart',
            'SECURITY.md, SECURITY.zh-CN.md: missing language counterpart',
        ])

    def test_root_rejects_linked_current_language_and_wrong_counterpart(self):
        files = self.root_pairs()
        invalid = [
            ('README.md', '<h1>XDZip</h1>\n<p><a href="README.md">English</a> | '
             '<a href="README.zh-CN.md">简体中文</a></p>\n', 'README.zh-CN.md'),
            ('README.zh-CN.md', '<h1>XDZip</h1>\n<p><a href="PRIVACY.md">English</a> | 简体中文</p>\n',
             'README.md'),
            ('CONTRIBUTING.md', '# Title\n\nBody text.\n\nEnglish | [简体中文](CONTRIBUTING.zh-CN.md)\n',
             'CONTRIBUTING.zh-CN.md'),
            ('GOVERNANCE.md', '# Title\n\nEnglish | [简体中文](GOVERNANCE.zh-CN.md#标题)\n',
             'GOVERNANCE.zh-CN.md'),
        ]
        for name, content, expected in invalid:
            with self.subTest(path=name):
                self.write(name, content)
                pairs, failures = check_docs.localized_documents(self.root, files, BILINGUAL)
                self.assertEqual(pairs, 7)
                self.assertEqual(len(failures), 1)
                self.assertTrue(failures[0].startswith(f'{name}:'))
                self.assertIn(expected, failures[0])
                self.root_pairs()

    def test_reader_docs_require_all_languages_but_developers_only_two(self):
        languages = {**BILINGUAL, 'fr-FR': 'Français'}
        files = self.root_pairs()
        files.extend([
            self.write('README.md', '<h1>XDZip</h1>\n<p>English | <a href="README.zh-CN.md">简体中文</a>'
                       ' | <a href="README.fr-FR.md">Français</a></p>\n'),
            self.write('README.zh-CN.md', '<h1>XDZip</h1>\n<p><a href="README.md">English</a> | 简体中文'
                       ' | <a href="README.fr-FR.md">Français</a></p>\n'),
            self.write('README.fr-FR.md', '<h1>XDZip</h1>\n<p><a href="README.md">English</a> | '
                       '<a href="README.zh-CN.md">简体中文</a> | Français</p>\n'),
            self.write('docs/guide.md', '# Guide\n\nEnglish | [简体中文](zh-CN/guide.md) | '
                       '[Français](fr-FR/guide.md)\n'),
            self.write('docs/zh-CN/guide.md', '# 指南\n\n[English](../guide.md) | 简体中文 | '
                       '[Français](../fr-FR/guide.md)\n'),
            self.write('developers/guide.md', '# Guide\n\nEnglish | [简体中文](zh-CN/guide.md)\n'),
            self.write('developers/zh-CN/guide.md', '# 指南\n\n[English](../guide.md) | 简体中文\n'),
        ])
        count, failures = check_docs.localized_documents(self.root, files, languages)
        self.assertEqual(count, 8)
        self.assertEqual(failures, ['docs/fr-FR/guide.md: missing language counterpart'])
        files.append(self.write('docs/fr-FR/guide.md', '# Guide\n\n[English](../guide.md) | '
                                '[简体中文](../zh-CN/guide.md) | Français\n'))
        self.assertEqual(check_docs.localized_documents(self.root, files, languages), (9, []))

    def test_reader_links_follow_their_language_and_use_english_developers_fallback(self):
        self.write('docs/guide.md', '# Guide\n')
        target = self.write('docs/fr-FR/guide.md', '# Guide\n\n## Démarrage\n')
        self.write('developers/qa.md', '# Verification\n\n## Layout\n')
        self.write('developers/zh-CN/qa.md', '# 验证\n\n## 布局\n')
        index = self.write('docs/fr-FR/index.md', '# XDZip\n\n'
                           '[Guide](guide.md#démarrage)\n'
                           '[Check](../../developers/qa.md#layout)\n'
                           '[Wrong language](../guide.md)\n'
                           '[Wrong developers](https://github.com/xiaodaozhi/xdzip/blob/main/developers/zh-CN/qa.md#布局)\n'
                           '[Wrong section](../../developers/qa.md#布局)\n')
        links, fragments, failures = check_docs.markdown_links(self.root, [index, target])
        self.assertEqual((links, fragments), (5, 4))
        self.assertEqual(len(failures), 3)
        self.assertIn('expected docs/fr-FR/guide.md', failures[0])
        self.assertIn('expected developers/qa.md', failures[1])
        self.assertIn('missing Markdown anchor', failures[2])

    def test_markdown_anchor_styles_and_local_targets(self):
        topic = self.write('docs/归档 名称 (FAQ).md', """# **格式** 与 _安全_：\x60file_name\x60 ###

## Repeat
## Repeat-1
## Repeat

Setext [guide](https://example.invalid) &amp; tips
------------------------------------------------

<a id="固定入口"></a>
<!--
~~~~
-->
<a name='legacy-anchor'></a>

[Same page](#repeat-2)
[Repository root](/README.md#index)
""")
        index = self.write('README.md', """# Index

[Unicode](<docs/归档 名称 (FAQ).md#%E6%A0%BC%E5%BC%8F-%E4%B8%8E-%E5%AE%89%E5%85%A8file_name>)
[Duplicate suffix](docs/归档%20名称%20(FAQ).md#repeat-2 "Heading link")
[Second title](docs/归档%20名称%20%28FAQ%29.md#repeat-1)
[Escaped destination](docs/归档%20名称%20\\(FAQ\\).md#repeat)
<a href="docs/归档 名称 (FAQ).md#固定入口">HTML link</a>
[Setext title][guide]
[guide]: <docs/归档 名称 (FAQ).md#setext-guide--tips>
[Legacy](https://github.com/xiaodaozhi/xdzip/blob/main/docs/归档%20名称%20(FAQ).md#legacy-anchor)
[GitHub line selector](https://github.com/xiaodaozhi/xdzip/blob/HEAD/docs/归档%20名称%20(FAQ).md#L1-L2)
[External](https://example.invalid/README.md#missing)
[Other branch](https://github.com/xiaodaozhi/xdzip/blob/old/docs/removed.md#missing)
""")
        self.assertEqual(check_docs.markdown_links(self.root, [index, topic]), (10, 9, []))

    def test_missing_targets_and_anchors_report_source_and_line(self):
        topic = self.write('docs/page.md', '''# Present

~~~~md
## Only an example
~~~
<a id="fenced-anchor"></a>
~~~~~

<!-- <a id="commented-anchor"></a> -->

The examples \x60<a id="inline-anchor"></a>\x60 and \x60<!--\x60 are literal.
''')
        index = self.write('README.md', '''# Index

[Same page](#gone)
[Missing file](docs/removed.md#section)
[Fenced heading](docs/page.md#only-an-example)
[Fenced anchor][anchor]
[anchor]: docs/page.md#fenced-anchor
<a href="docs/page.md#commented-anchor">Commented anchor</a>
[Inline HTML](docs/page.md#inline-anchor)
[Published path](https://github.com/xiaodaozhi/xdzip/blob/refs/heads/main/docs/page.md#stale)
''')
        links, fragments, failures = check_docs.markdown_links(self.root, [index, topic])
        self.assertEqual((links, fragments), (7, 6))
        self.assertEqual(failures, [
            'README.md:3: missing Markdown anchor #gone',
            'README.md:4: missing or external local target docs/removed.md#section',
            'README.md:5: missing Markdown anchor docs/page.md#only-an-example',
            'README.md:7: missing Markdown anchor docs/page.md#fenced-anchor',
            'README.md:8: missing Markdown anchor docs/page.md#commented-anchor',
            'README.md:9: missing Markdown anchor docs/page.md#inline-anchor',
            'README.md:10: missing Markdown anchor '
            'https://github.com/xiaodaozhi/xdzip/blob/refs/heads/main/docs/page.md#stale',
        ])

    def test_example_links_and_non_markdown_fragments_are_ignored(self):
        index = self.write('README.md', """# Index

\x60\x60\x60\x60md
[Example](missing.md#section)
\x60\x60\x60
<a href="other.md#section">Another example</a>
<!--
\x60\x60\x60\x60

An inline example \x60[Sample](missing.md#section)\x60.
<!-- [Comment](missing.md#section) -->
[Generated HTML](demo.html#generated-elsewhere)
""")
        self.write('demo.html', '<p>Demo</p>')
        self.assertEqual(check_docs.markdown_links(self.root, [index]), (1, 0, []))


if __name__ == '__main__':
    unittest.main()
