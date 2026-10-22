# scripts/test-macos-attribution.py

"""Regression boundaries for signed/thinned Qt component attribution."""
import hashlib
from pathlib import Path
import struct
import tempfile
import unittest

from macos_attribution import compare_images, deployment_record, image_identity


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


if __name__ == '__main__':
    unittest.main()
