#!/usr/bin/env python3
"""Exercise automatic glyph updates, offline integrity, concurrency and consumers."""
import copy
from functools import partial
import importlib.util
import io
import json
from pathlib import Path
import subprocess
import sys
import tempfile
from threading import Thread
import unittest
import urllib.request
import xml.etree.ElementTree as ET

from fontTools.ttLib import TTFont
import ui_fonts as fonts

ROOT = fonts.ROOT


class FontTests(unittest.TestCase):
    def setUp(self):
        parent = ROOT / 'build/test-ui-fonts'
        parent.mkdir(parents=True, exist_ok=True)
        self.temporary = tempfile.TemporaryDirectory(dir=parent)
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        spec = copy.deepcopy(fonts.configuration())
        spec['inputs'] = ['resources/i18n/*.json', 'apps/desktop/ui/**/*.qml', 'apps/desktop/**/*.cpp']
        for name, data in [(fonts.SPEC, json.dumps(spec).encode()),
                           (Path(spec['license']['path']), (ROOT / spec['license']['path']).read_bytes())]:
            target = self.root / name
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
        self.output = self.root / 'build/generated/fonts'
        self.cache = self.root / 'build/toolchain/fonts'
        self.label('current.json', '确定')

    def label(self, name, text):
        target = self.root / 'resources/i18n' / name
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(json.dumps({'messages': {'example': text}}, ensure_ascii=False), encoding='utf-8')
        return target

    def prepare(self):
        return fonts.prepare(self.cache, source=fonts.source_path(), root=self.root)

    def build(self, **options):
        return fonts.generate(self.output, cache=self.cache, root=self.root, **options)

    def test_new_text_and_added_deleted_files_update_coverage_without_manual_subset(self):
        self.prepare()
        initial = self.build()
        path = self.output / initial['native']['path']
        before = path.stat().st_mtime_ns
        self.label('current.json', '确定确定')
        self.assertEqual(self.build(), initial)
        self.assertEqual(path.stat().st_mtime_ns, before)
        extra = self.label('new-language.json', '龘')
        expanded = self.build()
        self.assertNotEqual(expanded['input_key'], initial['input_key'])
        self.assertIn(ord('龘'), TTFont(path).getBestCmap())
        extra.unlink()
        self.assertEqual(self.build(), initial)

    def test_recursive_qml_cpp_and_unicode_escapes_are_collected(self):
        qml = self.root / 'apps/desktop/ui/nested/Label.qml'
        qml.parent.mkdir(parents=True)
        qml.write_text('Text { text: qsTr("\\u9F98") }', encoding='utf-8')
        cpp = self.root / 'apps/desktop/nested/Label.cpp'
        cpp.parent.mkdir(parents=True)
        cpp.write_text('tr("麤"); QStringLiteral("靐");', encoding='utf-8')
        self.assertTrue(set(map(ord, '龘麤靐')) <= fonts.current_characters(self.root))
        self.assertEqual(fonts.decoded_characters(r'\ud840\udc00'), {0x20000})

    def test_fresh_generations_are_identical_and_ttf_is_same_source(self):
        self.prepare()
        first = self.build()
        other = self.root / 'build/second-fonts'
        second = fonts.generate(other, cache=self.cache, root=self.root)
        self.assertEqual(first, second)
        for kind in ('subset', 'native'):
            self.assertEqual((self.output / first[kind]['path']).read_bytes(),
                             (other / second[kind]['path']).read_bytes())
        font = TTFont(self.output / first['subset']['path'], recalcTimestamp=False)
        font.flavor = None
        data = io.BytesIO()
        font.save(data)
        self.assertEqual(data.getvalue(), (self.output / first['native']['path']).read_bytes())

    def test_corrupt_generated_bytes_fail_check_and_are_repaired_by_build(self):
        self.prepare()
        initial = self.build()
        (self.output / initial['native']['path']).write_bytes(b'corrupt')
        with self.assertRaises(ValueError):
            self.build(check=True)
        self.assertEqual(self.build(), initial)

    def test_offline_cache_missing_corrupt_and_wrong_import_fail(self):
        with self.assertRaisesRegex(ValueError, 'missing'):
            fonts.prepare(self.cache, offline=True, root=self.root)
        wrong = self.root / 'wrong.ttf'
        wrong.write_bytes(b'not the pinned upstream font')
        with self.assertRaisesRegex(ValueError, 'SHA-256'):
            fonts.prepare(self.cache, source=wrong, root=self.root)
        source = self.prepare()
        self.build()
        source.write_bytes(b'corrupt')
        with self.assertRaisesRegex(ValueError, 'SHA-256'):
            self.build()

    def test_stale_manifest_is_rejected_by_consumers_without_regeneration(self):
        self.prepare()
        self.build()
        self.label('current.json', '新增字')
        with self.assertRaisesRegex(ValueError, 'stale'):
            fonts.load_manifest(self.output, root=self.root)

    def test_parallel_processes_publish_one_complete_generation(self):
        self.prepare()
        code = ('import sys;sys.path.insert(0,sys.argv[1]);import ui_fonts as f;'
                'from pathlib import Path;f.generate(Path(sys.argv[2]),cache=Path(sys.argv[3]),root=Path(sys.argv[4]))')
        command = [sys.executable, '-c', code, str(ROOT / 'scripts'), str(self.output), str(self.cache), str(self.root)]
        processes = [subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE) for _ in range(2)]
        try:
            for process in processes:
                stdout, stderr = process.communicate(timeout=90)
                self.assertEqual(process.returncode, 0, (stdout + stderr).decode(errors='replace'))
        finally:
            for process in processes:
                if process.poll() is None:
                    process.kill()
                    process.wait()
        fonts.load_manifest(self.output, root=self.root)

    def test_source_output_locations_are_rejected(self):
        with self.assertRaisesRegex(ValueError, 'build/'):
            fonts.generate(self.root / 'design/assets/fonts', root=self.root)

    def test_preview_urls_serve_generated_fonts_and_resource_audit_binds_bytes(self):
        from html_review_server import ReviewHTTPServer, ReviewHTTPRequestHandler
        fonts.generate()
        server = ReviewHTTPServer(('127.0.0.1', 0), partial(ReviewHTTPRequestHandler, directory=str(ROOT)))
        thread = Thread(target=server.serve_forever, daemon=True)
        thread.start()
        try:
            for suffix in ('.woff2', '.ttf'):
                name = fonts.FONT_NAME + suffix
                with urllib.request.urlopen(f'http://127.0.0.1:{server.server_port}/design/assets/fonts/{name}?test=1', timeout=10) as response:
                    self.assertEqual(response.read(), (fonts.DEFAULT_OUTPUT / name).read_bytes())
        finally:
            server.shutdown()
            server.server_close()
            thread.join(timeout=5)
        spec = importlib.util.spec_from_file_location('font_resources', ROOT / 'scripts/check-runtime-resources.py')
        resources = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(resources)
        qrc = ROOT / 'apps/desktop/ui.qrc'
        files = []
        for group in ET.parse(qrc).getroot().findall('qresource'):
            for file in group.findall('file'):
                files.append({'resource': ':' + group.get('prefix', '') + '/' + file.get('alias', file.text),
                              'sha256': fonts.digest(qrc.parent / file.text)})
        manifest = fonts.load_manifest(fonts.DEFAULT_OUTPUT)
        files.append({'resource': ':/xdzip/fonts/' + manifest['native']['path'], 'sha256': manifest['native']['sha256']})
        report = {'runs': [{'events': [{'event': 'resource_manifest', 'files': files}]}]}
        self.assertEqual(len(resources.check(report, fonts.DEFAULT_OUTPUT)), len(files))
        files[-1]['sha256'] = '0' * 64
        with self.assertRaisesRegex(ValueError, 'bytes'):
            resources.check(report, fonts.DEFAULT_OUTPUT)


if __name__ == '__main__':
    unittest.main()
