# scripts/test-native-codec-sources.py

"""Reject untrusted source archives and verify offline pinned preparation."""
import importlib.util
import io
from pathlib import Path
import tarfile
import tempfile
import unittest

SPEC = importlib.util.spec_from_file_location('codec_sources', Path(__file__).with_name('prepare-native-codecs.py'))
SOURCES = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(SOURCES)


class SourceAdmissionTests(unittest.TestCase):
    def archive(self, *records):
        buffer = io.BytesIO()
        with tarfile.open(fileobj=buffer, mode='w') as archive:
            for name, kind, size in records:
                member = tarfile.TarInfo(name)
                member.type, member.size = kind, size
                member.linkname = '../../outside' if kind in (tarfile.SYMTYPE, tarfile.LNKTYPE) else ''
                archive.addfile(member, io.BytesIO(b'x' * size) if size else None)
        buffer.seek(0)
        return tarfile.open(fileobj=buffer)

    def test_rejects_escape_absolute_windows_and_links(self):
        for name, kind in [('../outside', tarfile.REGTYPE), ('/outside', tarfile.REGTYPE),
                           ('C:/outside', tarfile.REGTYPE), ('dir\\outside', tarfile.REGTYPE),
                           ('link', tarfile.SYMTYPE), ('hard', tarfile.LNKTYPE), ('fifo', tarfile.FIFOTYPE)]:
            with self.subTest(name=name, kind=kind), self.archive((name, kind, 0)) as archive:
                with self.assertRaises(ValueError):
                    SOURCES.validate_members(archive)

    def test_case_collisions_rejected_on_all_hosts(self):
        with self.archive(('Code.cpp', tarfile.REGTYPE, 1), ('code.cpp', tarfile.REGTYPE, 1)) as archive:
            with self.assertRaisesRegex(ValueError, 'Duplicate'):
                SOURCES.validate_members(archive)

    def test_expansion_budget_checked_before_extraction(self):
        with self.archive(('source.cpp', tarfile.REGTYPE, 4)) as archive:
            previous = SOURCES.MAX_EXPANDED
            SOURCES.MAX_EXPANDED = 3
            try:
                with self.assertRaisesRegex(ValueError, 'expands'):
                    SOURCES.validate_members(archive)
            finally:
                SOURCES.MAX_EXPANDED = previous

    def test_official_source_identity_and_format_boundaries(self):
        items = {item['id']: item for item in SOURCES.sources()}
        self.assertEqual(set(items), {'7zip', 'unrar'})
        self.assertEqual(items['7zip']['formats'], ['7z'])
        self.assertEqual(items['unrar']['formats'], ['rar'])
        self.assertFalse(items['unrar']['creation'])
        for item in items.values():
            self.assertEqual(len(item['sha256']), 64)
            self.assertTrue(item['url'].startswith('https://'))

    def test_marker_does_not_hide_prepared_source_modifications(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / 'source'
            source.mkdir()
            (source / 'xdzip-source.json').write_text('{}')
            (source / 'engine.cpp').write_bytes(b'original')
            archive = root / 'source.tar'
            with tarfile.open(archive, 'w') as output:
                output.add(source / 'engine.cpp', arcname='engine.cpp')
            SOURCES.verify_tree(source, archive)
            (source / 'engine.cpp').write_bytes(b'modified')
            with self.assertRaisesRegex(ValueError, 'differs'):
                SOURCES.verify_tree(source, archive)


if __name__ == '__main__':
    unittest.main()
