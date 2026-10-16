#!/usr/bin/env python3
"""Reject false SDK/source matches, unknown patches and leaking build paths."""
import hashlib
import io
import json
from pathlib import Path
import tarfile
import tempfile
import unittest
from unittest.mock import patch
import qt_sdk_correspondence as audit


class CorrespondenceTests(unittest.TestCase):
    def fixture(self, root):
        archive = root/'owned-src.tar.xz'
        with tarfile.open(archive, 'w:xz') as output:
            for name, content in {'.tag': b'a'*40, 'cmake/helper.cmake': b'original\n'}.items():
                info = tarfile.TarInfo('owned-src/'+name)
                info.size = len(content)
                output.addfile(info, io.BytesIO(content))
        declaration = root/'source.spdx'
        declaration.write_text('FileName: ./cmake/helper.cmake\nFileChecksum: SHA1: '+
                               hashlib.sha1(b'original\n').hexdigest()+'\n')
        item = {'module': 'owned', 'file': archive.name, 'sha256': audit.digest(archive),
                'source_commit': 'a'*40}
        return archive, declaration, item

    def test_source_match_and_incomplete_coverage(self):
        with tempfile.TemporaryDirectory() as temp:
            archive, declaration, item = self.fixture(Path(temp))
            result = audit.compare_sources(archive, item, declaration)
            self.assertEqual(result['matched_files'], 1)
            self.assertEqual(result['archive_only_files'], 1)
            self.assertEqual(result['status'], 'matching-declared-files')
            result = audit.compare_sources(archive, item)
            self.assertEqual(result['status'], 'incomplete')
            self.assertIsNone(result['sdk_source_spdx_sha256'])

    def test_difference_and_missing_are_evidence_not_success(self):
        with tempfile.TemporaryDirectory() as temp:
            archive, declaration, item = self.fixture(Path(temp))
            declaration.write_text('FileName: ./cmake/helper.cmake\nFileChecksum: SHA1: '+ 'b'*40+
                                   '\nFileName: ./missing\nFileChecksum: SHA1: '+'c'*40+'\n')
            result = audit.compare_sources(archive, item, declaration)
            self.assertEqual(result['status'], 'incomplete')
            self.assertEqual(len(result['different_files']), 1)
            self.assertEqual(result['sdk_only_files'], ['missing'])

    def test_archive_hash_and_commit_fail_closed(self):
        with tempfile.TemporaryDirectory() as temp:
            archive, declaration, item = self.fixture(Path(temp))
            item['source_commit'] = 'b'*40
            with self.assertRaisesRegex(ValueError, 'commit'):
                audit.compare_sources(archive, item, declaration)
            item['sha256'] = '0'*64
            with self.assertRaisesRegex(ValueError, 'checksum'):
                audit.compare_sources(archive, item)

    def test_source_declarations_reject_escape_and_duplicates(self):
        with tempfile.TemporaryDirectory() as temp:
            p = Path(temp)/'source.spdx'
            for name in ['../escape', '/private/file', 'C:/file', '..\\file']:
                p.write_text('FileName: '+name+'\nFileChecksum: SHA1: '+'a'*40+'\n')
                with self.assertRaisesRegex(ValueError, 'Unsafe'):
                    audit.source_checksums(p)
            p.write_text(('FileName: ./same\nFileChecksum: SHA1: '+'a'*40+'\n')*2)
            with self.assertRaisesRegex(ValueError, 'duplicate'):
                audit.source_checksums(p)

    def test_patch_replay_exact_bytes_and_unknown_sdk_rejected(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            archive, _, _ = self.fixture(root)
            folder = root/'scripts/qt-patches'
            folder.mkdir(parents=True)
            p = folder/'owned.patch'
            p.write_text('--- a/cmake/helper.cmake\n+++ b/cmake/helper.cmake\n@@ -1 +1 @@\n-original\n+modified\n', newline='\n')
            item = {'patch': 'scripts/qt-patches/owned.patch', 'patch_sha256': audit.digest(p),
                    'source_path': 'cmake/helper.cmake', 'sdk_path': 'lib/helper.cmake',
                    'id': 'owned', 'scope': 'fixture',
                    'before_sha256': hashlib.sha256(b'original\n').hexdigest(),
                    'after_sha256': hashlib.sha256(b'modified\n').hexdigest()}
            sdk = root/'sdk'; (sdk/'lib').mkdir(parents=True)
            target = sdk/'lib/helper.cmake'
            with patch.object(audit, 'ROOT', root):
                for content, expected in [('original\n', 'upstream-unmodified'), ('modified\n', 'known-local-patch')]:
                    target.write_bytes(content.encode('utf-8'))
                    self.assertEqual(audit.patch_status(sdk, item, archive)['state'], expected)
                target.write_text('unknown\n')
                with self.assertRaisesRegex(ValueError, 'Unrecognized'):
                    audit.patch_status(sdk, item, archive)
                p.write_text(p.read_text().replace('+modified', '+other'))
                with self.assertRaisesRegex(ValueError, 'checksum'):
                    audit.patch_status(sdk, item, archive)

    def test_redaction_keeps_flags_and_no_private_paths(self):
        result = audit.sanitize('-DFOO=/Users/builder/secret -DBAR=C:\\host\\secret -DFEATURE=ON /usr/local/tool')
        self.assertNotIn('secret', result)
        self.assertIn('-DFEATURE=ON', result)

    def test_repository_patch_and_archive_pins_are_valid(self):
        config = json.loads((audit.ROOT/'scripts/corresponding-sources.json').read_text())
        for item in config['archives']:
            self.assertRegex(item['source_commit'], r'^[a-f0-9]{40}$')
        for item in config['sdk_patches']:
            self.assertEqual(audit.digest(audit.ROOT/item['patch']), item['patch_sha256'])


if __name__ == '__main__':
    unittest.main()
