# scripts/test-native-codec-sources.py

"""Reject untrusted source archives and verify offline pinned preparation."""
import importlib.util
import hashlib
import io
from pathlib import Path
import tarfile
import tempfile
import unittest
from unittest import mock

SPEC = importlib.util.spec_from_file_location('codec_sources', Path(__file__).with_name('prepare-native-codecs.py'))
SOURCES = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(SOURCES)
INTEROP_SPEC = importlib.util.spec_from_file_location(
    'codec_interop', Path(__file__).with_name('prepare-native-interop.py'))
INTEROP = importlib.util.module_from_spec(INTEROP_SPEC)
INTEROP_SPEC.loader.exec_module(INTEROP)


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
        self.assertEqual(set(items), {'unrar'})
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


class ReferenceBootstrapTests(unittest.TestCase):
    def temporary_output(self):
        build = INTEROP.ROOT / 'build'
        build.mkdir(exist_ok=True)
        return tempfile.TemporaryDirectory(prefix='reference-bootstrap-test-', dir=build)

    def test_windows_uses_only_pinned_bootstrap_and_exact_member(self):
        data = b'MZowned reference fixture'
        with self.temporary_output() as temporary:
            output = Path(temporary)
            with mock.patch.object(INTEROP.platform, 'system', return_value='Windows'), \
                 mock.patch.object(INTEROP, 'verified_download', side_effect=lambda root, name, digest: root / name) as download, \
                 mock.patch.object(INTEROP, 'WINDOWS_EXECUTABLE_SHA256', hashlib.sha256(data).hexdigest()), \
                 mock.patch.object(INTEROP.subprocess, 'check_output', return_value=data) as execute, \
                 mock.patch.object(INTEROP.tarfile, 'open') as system_tar:
                executable = INTEROP.prepare(output)
            self.assertEqual(executable.name, '7za.exe')
            self.assertEqual(executable.read_bytes(), data)
            archive, checksum, member = INTEROP.TOOLS['Windows']
            self.assertEqual(download.call_args_list, [
                mock.call(output, archive, checksum), mock.call(output, *INTEROP.WINDOWS_BOOTSTRAP)])
            execute.assert_called_once_with(
                [str(output / '7zr.exe'), 'x', '-so', str(output / archive), member], timeout=60)
            system_tar.assert_not_called()

    def test_corrupt_cached_bootstrap_is_not_executed_or_published(self):
        archive, bootstrap = b'verified archive', b'verified bootstrap'
        with self.temporary_output() as temporary:
            output = Path(temporary)
            (output / 'reference.7z').write_bytes(archive)
            (output / '7zr.exe').write_bytes(b'tampered executable')
            (output / '7za.exe').write_bytes(b'previous reference')
            with mock.patch.object(INTEROP.platform, 'system', return_value='Windows'), \
                 mock.patch.dict(INTEROP.TOOLS, {'Windows': ('reference.7z', hashlib.sha256(archive).hexdigest(), 'x64/7za.exe')}), \
                 mock.patch.object(INTEROP, 'WINDOWS_BOOTSTRAP', ('7zr.exe', hashlib.sha256(bootstrap).hexdigest())), \
                 mock.patch.object(INTEROP.subprocess, 'check_output') as execute, \
                 mock.patch.object(INTEROP.urllib.request, 'urlopen') as network:
                with self.assertRaisesRegex(ValueError, 'cache checksum'):
                    INTEROP.prepare(output)
            execute.assert_not_called()
            network.assert_not_called()
            self.assertEqual((output / '7za.exe').read_bytes(), b'previous reference')

    def test_wrong_extracted_member_never_overwrites_reference(self):
        with self.temporary_output() as temporary:
            output = Path(temporary)
            (output / '7za.exe').write_bytes(b'previous reference')
            with mock.patch.object(INTEROP.platform, 'system', return_value='Windows'), \
                 mock.patch.object(INTEROP, 'verified_download', side_effect=lambda root, name, digest: root / name), \
                 mock.patch.object(INTEROP.subprocess, 'check_output', return_value=b'wrong member'):
                with self.assertRaisesRegex(ValueError, 'executable checksum'):
                    INTEROP.prepare(output)
            self.assertEqual((output / '7za.exe').read_bytes(), b'previous reference')

    def test_bad_download_and_oversized_cache_fail_before_use(self):
        with self.temporary_output() as temporary:
            output = Path(temporary)
            with mock.patch.object(INTEROP.urllib.request, 'urlopen', return_value=io.BytesIO(b'wrong bytes')) as network:
                with self.assertRaisesRegex(ValueError, 'download checksum'):
                    INTEROP.verified_download(output, '7zr.exe', '0' * 64)
            network.assert_called_once_with(INTEROP.RELEASE_URL + '7zr.exe', timeout=60)
            self.assertFalse((output / '7zr.exe').exists())
            (output / '7zr.exe').write_bytes(b'12345')
            with mock.patch.object(INTEROP, 'MAX_DOWNLOAD', 4):
                with self.assertRaisesRegex(ValueError, 'cache checksum'):
                    INTEROP.verified_download(output, '7zr.exe', hashlib.sha256(b'12345').hexdigest())

    def test_unix_reference_still_reads_only_verified_tar_member(self):
        for system in ['Linux', 'Darwin']:
            with self.subTest(system=system), self.temporary_output() as temporary:
                output = Path(temporary)
                data = b'owned Unix reference'
                archive = output / 'reference.tar'
                with tarfile.open(archive, 'w') as container:
                    entry = tarfile.TarInfo('7zz')
                    entry.size = len(data)
                    container.addfile(entry, io.BytesIO(data))
                checksum = hashlib.sha256(archive.read_bytes()).hexdigest()
                with mock.patch.object(INTEROP.platform, 'system', return_value=system), \
                     mock.patch.dict(INTEROP.TOOLS, {system: (archive.name, checksum, '7zz')}), \
                     mock.patch.object(INTEROP.subprocess, 'check_output') as execute:
                    executable = INTEROP.prepare(output)
                self.assertEqual(executable.name, '7zz')
                self.assertEqual(executable.read_bytes(), data)
                execute.assert_not_called()


if __name__ == '__main__':
    unittest.main()
