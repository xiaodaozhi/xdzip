#!/usr/bin/env python3
"""Test Tika provenance and delivery boundaries with owned fixtures."""
import hashlib
import io
import json
from pathlib import Path
import tarfile
import tempfile
import unittest

from deployment_audit import digest
import qt_tika_materials as tika


def fixture(root, wrong_result=False):
    inputs = root/'third_party/qt-tika-mimetypes'
    inputs.mkdir(parents=True)
    (root/'scripts').mkdir()
    source = b'owned MIME input\n'
    notice = tika.ASF_NOTICE+b'\n\n'+tika.PRONOM_NOTICE+b'\n\nOther upstream components.\n'
    ogl = b'owned OGL fixture\n'
    apache = b'owned Apache fixture\n'
    generator = (b'from pathlib import Path\n'
                 b'source=Path("tika-mimetypes.xml.orig").read_text(encoding="utf-8")\n'
                 b'Path("tika-mimetypes.xml").write_text("converted:"+source, encoding="utf-8")\n')
    result = b'wrong data\n' if wrong_result else b'converted:'+source
    values = {'tika-mimetypes.xml.orig': source, 'TIKA-NOTICE.txt': notice,
              'OGL-UK-3.0.txt': ogl, 'README.md': b'owned explanation\n'}
    for name, content in values.items():
        (inputs/name).write_bytes(content)
    commit = 'a'*40
    attribution = json.dumps([{'Id': 'tika-mimetypes', 'Version': commit,
                               'LicenseId': 'Apache-2.0'}]).encode()
    archive_name = 'qtbase-everywhere-src-fixture.tar.xz'
    prefix = archive_name.removesuffix('.tar.xz')+'/src/corelib/mimetypes/3rdparty/'
    archive = root/archive_name
    with tarfile.open(archive, 'w:xz') as output:
        for name, content in {'process_tika_mimetypes.py': generator, 'tika-mimetypes.xml': result,
                              'qt_attribution.json': attribution}.items():
            info = tarfile.TarInfo(prefix+name)
            info.size = len(content)
            output.addfile(info, io.BytesIO(content))
    sha = lambda content: hashlib.sha256(content).hexdigest()
    config = {'qt_archive': archive_name, 'qt_archive_sha256': digest(archive),
              'tika_repository': 'https://example.invalid/tika', 'tika_commit': commit,
              'tika_path': 'tika-mimetypes.xml', 'source_git_blob': tika.git_blob(source),
              'source_sha256': sha(source), 'notice_git_blob': tika.git_blob(notice),
              'notice_sha256': sha(notice), 'ogl_git_blob': tika.git_blob(ogl),
              'ogl_sha256': sha(ogl), 'generator_sha256': sha(generator),
              'qt_generated_sha256': sha(result), 'apache_license_sha256': sha(apache)}
    (root/'scripts/qt-tika-materials.json').write_text(json.dumps(config), encoding='utf-8')
    (root/'scripts/corresponding-sources.json').write_text(json.dumps({'archives': [
        {'module': 'qtbase', 'file': archive_name, 'sha256': digest(archive)}]}), encoding='utf-8')
    destination = root/'delivery 中文 with spaces'
    destination.mkdir()
    (destination/'license-texts').mkdir()
    (destination/'license-texts/Apache-2.0.txt').write_bytes(apache)
    return archive, destination


class TikaMaterialsTests(unittest.TestCase):
    def test_source_delivery_regenerates_and_copies_originals(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            archive, output = fixture(root)
            record = tika.collect(output, archive, root)
            self.assertIn(record['regeneration'], ['byte-identical', 'windows-native-line-endings'])
            self.assertEqual((output/record['file']).read_bytes(), b'owned MIME input\n')
            self.assertEqual({p.name for p in (output/'qt-tika-mimetypes').iterdir()},
                             {'tika-mimetypes.xml.orig', 'TIKA-NOTICE.txt', 'OGL-UK-3.0.txt',
                              'README.md', 'source.json'})

    def test_runtime_notices_are_scoped_and_repeatable(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            _, output = fixture(root)
            record = tika.collect_runtime(output, root)
            attribution = (output/record['attribution']).read_bytes()
            self.assertIn(tika.ASF_NOTICE, attribution)
            self.assertIn(tika.PRONOM_NOTICE, attribution)
            self.assertIn(b'not a list of dependencies shipped with xdzip', attribution)
            self.assertEqual((output/record['upstream_notice']).read_bytes(),
                             (root/'third_party/qt-tika-mimetypes/TIKA-NOTICE.txt').read_bytes())
            self.assertEqual(tika.collect_runtime(output, root), record)
            (output/record['attribution']).write_bytes(b'keep existing')
            with self.assertRaisesRegex(ValueError, 'destination differs'):
                tika.collect_runtime(output, root)
            self.assertEqual((output/record['attribution']).read_bytes(), b'keep existing')

    def test_modified_inputs_or_apache_license_are_rejected(self):
        for name in ['tika-mimetypes.xml.orig', 'TIKA-NOTICE.txt', 'OGL-UK-3.0.txt', 'Apache-2.0.txt']:
            with self.subTest(name=name), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                _, output = fixture(root)
                path = (output/'license-texts'/name if name == 'Apache-2.0.txt'
                        else root/'third_party/qt-tika-mimetypes'/name)
                path.write_bytes(b'altered')
                with self.assertRaisesRegex(ValueError, 'mismatch'):
                    tika.collect_runtime(output, root)
                self.assertFalse((output/'qt-tika-mimetypes').exists())

    def test_archive_and_unreproducible_result_are_rejected(self):
        for changed in ['archive', 'result']:
            with self.subTest(changed=changed), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                archive, output = fixture(root, wrong_result=changed == 'result')
                if changed == 'archive':
                    archive.write_bytes(b'altered')
                with self.assertRaisesRegex(ValueError, 'mismatch|regenerated data differs'):
                    tika.collect(output, archive, root)
                self.assertFalse((output/'qt-tika-mimetypes').exists())

    def test_existing_source_destination_is_not_overwritten(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            archive, output = fixture(root)
            destination = output/'qt-tika-mimetypes'
            destination.mkdir()
            (destination/'owned').write_bytes(b'keep')
            with self.assertRaisesRegex(ValueError, 'destination unavailable'):
                tika.collect(output, archive, root)
            self.assertEqual((destination/'owned').read_bytes(), b'keep')


if __name__ == '__main__':
    unittest.main()
