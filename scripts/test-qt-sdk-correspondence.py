#!/usr/bin/env python3
"""Reject false SDK/source matches, unknown patches and leaking build paths."""
import base64
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import tarfile
import tempfile
import unittest
from unittest.mock import patch
import qt_sdk_correspondence as audit
import qt_source_identity as identity

source_spec = importlib.util.spec_from_file_location('source_delivery', audit.ROOT/'scripts/prepare-source-delivery.py')
source = importlib.util.module_from_spec(source_spec)
source_spec.loader.exec_module(source)


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
            self.assertEqual(result['archive_commit'], item['source_commit'])
            self.assertEqual(result['archive_marker'], item['source_commit'])
            self.assertEqual(result['archive_marker_type'], 'commit')
            self.assertFalse(result['commit_tree_verified'])
            self.assertEqual(result['archive_only_files'], 1)
            self.assertEqual(result['status'], 'matching-declared-files')
            declaration.write_text('FileName: ./cmake/helper.cmake\nFileChecksum: SHA1: '+hashlib.sha1(b'original\r\n').hexdigest()+'\n')
            crlf = audit.compare_sources(archive, item, declaration)
            self.assertEqual(crlf['exact_crlf_transformation_files'],1)
            self.assertEqual(crlf['unexplained_different_files'],[])
            self.assertEqual(len(crlf['different_files']),1)
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

    def tree_fixture(self, root):
        archive, declaration, item = self.fixture(root)
        marker = 'a'*40
        payload = (f'tree {marker}\nparent ' + 'b'*40 + '\n\nOwned fixture\n').encode('ascii')
        commit = hashlib.sha1(b'commit '+str(len(payload)).encode('ascii')+b'\0'+payload).hexdigest()
        proof = root/'scripts/qt-source-identity/owned.commit.base64'
        proof.parent.mkdir(parents=True)
        proof.write_bytes(base64.b64encode(payload)+b'\n')
        item.update(source_commit=commit, archive_marker=marker, archive_marker_type='tree',
                    commit_object='scripts/qt-source-identity/owned.commit.base64')
        return archive, declaration, item, proof, payload

    def test_tree_marker_binds_commit_across_both_collectors(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            archive, declaration, item, proof, payload = self.tree_fixture(root)
            with patch.object(identity, 'ROOT', root):
                result = audit.compare_sources(archive, item, declaration)
                self.assertEqual(result['archive_commit'], item['source_commit'])
                self.assertEqual(result['archive_marker'], item['archive_marker'])
                self.assertEqual(result['archive_marker_type'], 'tree')
                self.assertTrue(result['commit_tree_verified'])
                self.assertNotEqual(result['archive_commit'], result['archive_marker'])
                self.assertEqual(source.archive_source(item, root), archive)
                output = root/'delivery'; output.mkdir()
                source.collect_source_identity_proofs(output, [item])
                self.assertEqual((output/item['commit_object']).read_bytes(), proof.read_bytes())
                with self.assertRaisesRegex(ValueError, 'Duplicate'):
                    source.collect_source_identity_proofs(output, [item])

                for field, value in [('source_commit', 'b'*40), ('archive_marker', 'c'*40),
                                     ('archive_marker_type', 'commit'), ('archive_marker_type', 'unknown')]:
                    changed = dict(item, **{field: value})
                    with self.subTest(field=field, value=value), self.assertRaises(ValueError):
                        audit.compare_sources(archive, changed, declaration)
                    with self.assertRaises(ValueError):
                        source.archive_source(changed, root)
                proof.write_bytes(base64.b64encode(payload+b'altered'))
                with self.assertRaisesRegex(ValueError, 'proof mismatch'):
                    audit.compare_sources(archive, item, declaration)
                with self.assertRaisesRegex(ValueError, 'proof mismatch'):
                    source.archive_source(item, root)
                proof.write_bytes(b'not valid base64!')
                with self.assertRaisesRegex(ValueError, 'Invalid Git commit object'):
                    audit.compare_sources(archive, item, declaration)
                proof.write_bytes(b'A'*16385)
                with self.assertRaisesRegex(ValueError, 'too large'):
                    audit.compare_sources(archive, item, declaration)
                proof.write_bytes(base64.b64encode(payload)+b'\n')
                with tarfile.open(archive, 'w:xz') as changed_archive:
                    for name, content in {'.tag': b'b'*40, 'cmake/helper.cmake': b'original\n'}.items():
                        member = tarfile.TarInfo('owned-src/'+name)
                        member.size = len(content)
                        changed_archive.addfile(member, io.BytesIO(content))
                item['sha256'] = audit.digest(archive)
                with self.assertRaisesRegex(ValueError, 'marker mismatch'):
                    audit.compare_sources(archive, item, declaration)
                with self.assertRaisesRegex(ValueError, 'marker mismatch'):
                    source.archive_source(item, root)

    def test_commit_object_path_rejects_escape(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            _, _, item, _, _ = self.tree_fixture(root)
            for name in ['../escape', '/absolute', 'scripts/qt-source-identity/../../escape',
                         'scripts/qt-source-identity-other/file', 'C:/outside', 'scripts\\outside']:
                with self.subTest(name=name), self.assertRaises(ValueError):
                    identity.source_identity(dict(item, commit_object=name), root)

    def test_commit_object_parent_symlink_cannot_escape(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            outside = root/'outside'; outside.mkdir()
            scripts = root/'scripts'; scripts.mkdir()
            try:
                (scripts/'qt-source-identity').symlink_to(outside, target_is_directory=True)
            except OSError as error:
                if getattr(error, 'winerror', None) != 1314:
                    raise
                self.skipTest('Native symlink creation needs Windows privilege')
            (outside/'proof.base64').write_bytes(b'YQ==')
            item = {'source_commit': 'a'*40, 'archive_marker': 'b'*40,
                    'archive_marker_type': 'tree', 'commit_object': 'scripts/qt-source-identity/proof.base64'}
            with self.assertRaisesRegex(ValueError, 'escapes'):
                identity.source_identity(item, root)

    def test_commit_object_delivery_symlink_cannot_escape(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            _, _, item, _, _ = self.tree_fixture(root)
            output = root/'delivery'; output.mkdir()
            outside = root/'outside'; outside.mkdir()
            try:
                (output/'scripts').symlink_to(outside, target_is_directory=True)
            except OSError as error:
                if getattr(error, 'winerror', None) != 1314:
                    raise
                self.skipTest('Native symlink creation needs Windows privilege')
            with patch.object(identity, 'ROOT', root), self.assertRaisesRegex(ValueError, 'escapes'):
                source.collect_source_identity_proofs(output, [item])
            self.assertEqual(list(outside.iterdir()), [])

    def test_source_declarations_reject_escape_and_duplicates(self):
        with tempfile.TemporaryDirectory() as temp:
            p = Path(temp)/'source.spdx'
            for name in ['../escape', '/private/file', 'C:/file', '..\\file']:
                p.write_text('FileName: '+name+'\nFileChecksum: SHA1: '+'a'*40+'\n')
                with self.assertRaisesRegex(ValueError, 'Unsafe'):
                    audit.source_checksums(p)
            p.write_text('FileName: .\\cmake\\helper.cmake\nFileChecksum: SHA1: '+'a'*40+'\n')
            self.assertEqual(audit.source_checksums(p), {'cmake/helper.cmake': 'a'*40})
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
                target.write_bytes(b'original\r\n')
                result = audit.patch_status(sdk, item, archive)
                self.assertEqual(result['state'], 'upstream-unmodified')
                self.assertEqual(result['line_endings'], 'CRLF-normalized-for-comparison')
                self.assertNotEqual(result['sdk_sha256'], result['lf_sha256'])
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
            result = identity.source_identity(item)
            if item['module'] == 'qtdeclarative':
                self.assertEqual(result['source_commit'], '75534f3e7fff24ed7ccb364e2ed9950a73da879f')
                self.assertEqual(result['archive_marker'], '0a8139c574fe150aba92d5f2484a8f3fdb774977')
                self.assertTrue(result['commit_tree_verified'])
        for item in config['sdk_patches']:
            self.assertEqual(audit.digest(audit.ROOT/item['patch']), item['patch_sha256'])


if __name__ == '__main__':
    unittest.main()
