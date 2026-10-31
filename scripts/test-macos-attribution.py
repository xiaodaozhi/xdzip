# scripts/test-macos-attribution.py

"""Regression boundaries for signed/thinned Qt component attribution."""
import hashlib
import json
import os
from pathlib import Path
import struct
import tempfile
import unittest
from unittest.mock import patch

from macos_attribution import compare_images, deployment_record, image_identity, sdk_source_for_path
from runtime_attribution import attribute


def image(content=b'owned code', dependency=b'@rpath/QtCore.framework/Versions/A/QtCore', uuid=b'u'*16):
    dependency += b'\0'
    length = (24 + len(dependency) + 7) // 8 * 8
    commands_size = 152 + 24 + length
    offset = 32 + commands_size
    segment = struct.pack('<II16sQQQQIIII', 0x19, 152, b'__TEXT', 0, 4096, 0, offset + len(content), 7, 5, 1, 0)
    section = struct.pack('<16s16sQQIIIIIIII', b'__text', b'__TEXT', 4096, len(content), offset, 0, 0, 0, 0, 0, 0, 0)
    load = struct.pack('<IIIIII', 0xc, length, 24, 0, 1, 1) + dependency
    load += b'\0' * (length - len(load))
    header = struct.pack('<8I', 0xfeedfacf, 0x100000c, 0, 6, 3, commands_size, 0, 0)
    return header + segment + section + struct.pack('<II16s', 0x1b, 24, uuid) + load + content


class MacAttributionTests(unittest.TestCase):
    def plugin_fixture(self, base):
        sdk = base/'sdk'; root = base/'app'
        sdk_relative = 'qml/QtQuick/Effects/libeffectsplugin.dylib'
        sdk_file = sdk/sdk_relative; sdk_file.parent.mkdir(parents=True); sdk_file.write_bytes(image())
        target_relative = 'Contents/PlugIns/quick/libeffectsplugin.dylib'
        target = root/target_relative; target.parent.mkdir(parents=True)
        target.write_bytes(image() + b'owned signature')
        alias_relative = 'Contents/Resources/' + sdk_relative
        alias = root/alias_relative; alias.parent.mkdir(parents=True); alias.write_bytes(target.read_bytes())
        link_target = os.path.relpath(target.resolve(), alias.parent.resolve()).replace('\\', '/')
        digest = hashlib.sha256(target.read_bytes()).hexdigest()
        audit = {'status': 'passed', 'platform': 'macos', 'mode': 'deployed',
                 'identity': {'version': '1', 'revision': 'a'*12, 'build': 1, 'dirty': False},
                 'files': [{'path': target_relative, 'sha256': digest},
                           {'path': alias_relative, 'sha256': digest, 'symlink_target': link_target}],
                 'external_files': [], 'binary_imports': [], 'sdk_independent_imports': True}
        (sdk/'sbom').mkdir()
        document = {'spdxVersion': 'SPDX-2.3', 'documentNamespace': 'https://qt.io/test',
                    'packages': [{'SPDXID': 'repo', 'name': 'qtbase', 'versionInfo': '6.8.2'},
                                 {'SPDXID': 'plugin', 'name': 'Effects', 'versionInfo': '1', 'licenseConcluded': 'MIT'}],
                    'files': [{'SPDXID': 'file', 'fileName': './'+sdk_relative,
                               'checksums': [{'algorithm': 'SHA1', 'checksumValue': hashlib.sha1(sdk_file.read_bytes()).hexdigest()}]}],
                    'relationships': [{'relationshipType': 'CONTAINS', 'spdxElementId': 'plugin', 'relatedSpdxElement': 'file'}]}
        (sdk/'sbom/qtbase-6.8.2.spdx.json').write_text(json.dumps(document))
        return sdk, root, audit, alias, target, sdk_relative

    def test_relocation_and_signature_tail_do_not_change_section_identity(self):
        source = image()
        target = image(dependency=b'@executable_path/../Frameworks/QtCore.framework/Versions/A/QtCore')
        self.assertEqual(compare_images(source, target + b'owned-signature'), compare_images(source, source))
        universal = struct.pack('>II5I', 0xcafebabe, 1, 0x100000c, 0, 28, len(source), 0) + source
        self.assertEqual(compare_images(target, universal), compare_images(target, source))

    def test_changed_code_uuid_import_or_bounds_are_rejected(self):
        for changed in [image(b'changed code'), image(uuid=b'v'*16), image(dependency=b'/usr/lib/other.dylib')]:
            with self.assertRaisesRegex(ValueError, 'differ'):
                compare_images(image(), changed)
        for changed in [image()[:40], image()[:-2], b'not Mach-O']:
            with self.assertRaises(ValueError):
                image_identity(changed)

    def test_record_binds_sdk_and_artifact_hashes(self):
        with tempfile.TemporaryDirectory() as temp:
            base = Path(temp); sdk = base/'sdk'; root = base/'app'
            original = sdk/'lib/QtCore.framework/Versions/A/QtCore'
            original.parent.mkdir(parents=True); original.write_bytes(image())
            relative = 'Contents/Frameworks/QtCore.framework/Versions/A/QtCore'
            target = root/relative; target.parent.mkdir(parents=True)
            target.write_bytes(image() + b'owned signature')
            audit = {'files': [{'path': relative, 'sha256': hashlib.sha256(target.read_bytes()).hexdigest()}]}
            result = deployment_record(root, sdk, audit)
            self.assertEqual(result['sdk_files'][0]['sdk_path'], 'lib/QtCore.framework/Versions/A/QtCore')
            self.assertEqual(result['sdk_files'][0]['comparison'], 'mach-o-sections-uuid-and-imports')
            target.write_bytes(image(b'tampered'))
            with self.assertRaisesRegex(ValueError, 'changed before'):
                deployment_record(root, sdk, audit)

    def test_unique_qml_sdk_source_and_ambiguous_or_missing_candidates(self):
        with tempfile.TemporaryDirectory() as temp:
            sdk, root, audit, alias, target, source = self.plugin_fixture(Path(temp))
            relative = audit['files'][0]['path']
            self.assertEqual(sdk_source_for_path(sdk, relative, {relative: {source}}), source)
            self.assertIsNone(sdk_source_for_path(sdk, relative, {relative: {'qml/missing/plugin.dylib'}}))
            other = 'qml/Other/libeffectsplugin.dylib'
            (sdk/other).parent.mkdir(parents=True); (sdk/other).write_bytes(image())
            with self.assertRaisesRegex(ValueError, 'Ambiguous'):
                sdk_source_for_path(sdk, relative, {relative: {source, other}})

    def test_qml_link_provenance_round_trip_and_tampering(self):
        with tempfile.TemporaryDirectory() as temp:
            sdk, root, audit, alias, target, source = self.plugin_fixture(Path(temp))
            links = {alias: target}
            original_resolve, original_is_symlink = Path.resolve, Path.is_symlink

            def resolve(path, *args, **kwargs):
                return original_resolve(links.get(path, path), *args, **kwargs)

            def is_symlink(path):
                return path in links or original_is_symlink(path)

            with patch.object(Path, 'resolve', resolve), patch.object(Path, 'is_symlink', is_symlink):
                record = deployment_record(root, sdk, audit)
                target_record = next(row for row in record['sdk_files'] if row['path'] == audit['files'][0]['path'])
                self.assertEqual(target_record['sdk_path'], source)
                self.assertEqual(target_record['comparison'], 'mach-o-sections-uuid-and-imports')
                report = attribute(audit, root, sdk, record)
                linked = next(row for row in report['files'] if row['path'] == 'artifact/' + audit['files'][0]['path'])
                self.assertEqual(linked['evidence'], 'mach-o-sections-uuid-and-imports')
                self.assertFalse(any(g['kind'] == 'unverified-rewrite' for g in report['gaps']))

                alternative = 'qml/Other/libeffectsplugin.dylib'
                (sdk/alternative).parent.mkdir(parents=True); (sdk/alternative).write_bytes(image())
                target_record['sdk_path'] = alternative
                with self.assertRaisesRegex(ValueError, 'Unexpected macOS SDK ownership'):
                    attribute(audit, root, sdk, record)
                target_record['sdk_path'] = source
                target.write_bytes(image(b'tampered'))
                with self.assertRaisesRegex(ValueError, 'changed after'):
                    attribute(audit, root, sdk, record)
                target.write_bytes(image() + b'owned signature')
                audit['files'][1]['symlink_target'] = 'wrong-target'
                with self.assertRaisesRegex(ValueError, 'link changed'):
                    attribute(audit, root, sdk, record)
                audit['files'][1]['symlink_target'] = os.path.relpath(target.resolve(), alias.parent.resolve()).replace('\\', '/')
                audit['files'].append(dict(audit['files'][1]))
                with self.assertRaisesRegex(ValueError, 'Duplicate audited QML link'):
                    deployment_record(root, sdk, audit)
                audit['files'].pop()
                (sdk/source).unlink()
                missing = deployment_record(root, sdk, audit)
                self.assertFalse(any(r['path'] == audit['files'][0]['path'] for r in missing['sdk_files']))
                links.clear()
                with self.assertRaisesRegex(ValueError, 'no longer a symlink'):
                    deployment_record(root, sdk, audit)

    def test_real_qml_symlink_is_checked_when_supported(self):
        with tempfile.TemporaryDirectory() as temp:
            sdk, root, audit, alias, target, source = self.plugin_fixture(Path(temp))
            alias.unlink()
            try:
                alias.symlink_to(os.path.relpath(target, alias.parent))
            except OSError as error:
                if os.name != 'nt' or getattr(error, 'winerror', None) != 1314:
                    raise
                self.skipTest(f'Creating a native symlink is unavailable: {error.__class__.__name__}')
            record = deployment_record(root, sdk, audit)
            self.assertEqual(next(r for r in record['sdk_files'] if r['path'] == audit['files'][0]['path'])['sdk_path'], source)
            report = attribute(audit, root, sdk, record)
            self.assertFalse(any(g['kind'] == 'unverified-rewrite' for g in report['gaps']))
            root_alias = Path(temp)/'app-alias'
            root_alias.symlink_to('app', target_is_directory=True)
            via_alias = deployment_record(root_alias, sdk, audit)
            self.assertEqual(next(r for r in via_alias['sdk_files'] if r['path'] == audit['files'][0]['path'])['sdk_path'], source)
            self.assertFalse(any(g['kind'] == 'unverified-rewrite' for g in attribute(audit, root_alias, sdk, via_alias)['gaps']))
            alias.unlink()
            outside = Path(temp)/'outside'; outside.write_bytes(target.read_bytes())
            alias.symlink_to(os.path.relpath(outside, alias.parent))
            with self.assertRaisesRegex(ValueError, 'escapes root'):
                deployment_record(root, sdk, audit)


if __name__ == '__main__':
    unittest.main()
