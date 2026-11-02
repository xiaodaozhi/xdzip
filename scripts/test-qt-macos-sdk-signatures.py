#!/usr/bin/env python3
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import struct
import tarfile
import tempfile
import unittest
from unittest.mock import patch
import zipfile

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('sdk_signature_verifier',
    ROOT / 'scripts/verify-alpha8-macos-sdk-signatures.py')
verifier = importlib.util.module_from_spec(spec)
spec.loader.exec_module(verifier)


def fake_signature(flags, cms):
    directory = struct.pack('>IIII', 0xfade0c02, 16, 0x20400, flags)
    parts = [(0, directory)]
    if cms:
        parts.append((0x10000, struct.pack('>II', 0xfade0b01, 8)))
    offset = 12 + 8 * len(parts)
    entries, payload = [], bytearray()
    for slot, value in parts:
        entries.append(struct.pack('>II', slot, offset + len(payload)))
        payload.extend(value)
    return struct.pack('>III', 0xfade0cc0, offset + len(payload), len(parts)) + b''.join(entries) + payload


def slice_pair(cpu):
    arm = cpu == 0x100000c
    old_signature = fake_signature(0x20002, False) if arm else b''
    new_signature = fake_signature(0x10000, True)
    old_end = 273
    new_offset = old_end if arm else 281

    def segment(file_size):
        return struct.pack('<II16sQQQQiiII', 0x19, 72, b'__LINKEDIT', 0x1000, file_size,
                           128, file_size, 1, 1, 0, 0)

    def build(signature, sig_start, old):
        count = 1 + bool(signature)
        commands = [segment(145 if old else 153 + len(new_signature))]
        if signature:
            commands.append(struct.pack('<IIII', 0x1d, 16, sig_start, len(signature)))
        header = struct.pack('<8I', 0xfeedfacf, cpu, 0, 6, count, sum(map(len, commands)), 0, 0)
        prefix = bytearray(header + b''.join(commands))
        prefix.extend(bytes(256 - len(prefix)))
        prefix.extend(b'owned-payload-123')
        if not old and not arm:
            prefix.extend(bytes(8))
        if len(prefix) != sig_start:
            raise ValueError('fixture length mismatch')
        return bytes(prefix) + signature

    return build(old_signature, old_end, True), build(new_signature, new_offset, False)


def make_fat(slices, alignment):
    data = bytearray(48)
    struct.pack_into('>II', data, 0, 0xcafebabe, 2)
    for index, (cpu, blob) in enumerate(slices):
        offset = (len(data) + (1 << alignment) - 1) // (1 << alignment) * (1 << alignment)
        data.extend(bytes(offset - len(data)))
        struct.pack_into('>IIIII', data, 8 + index * 20, cpu, 0, offset, len(blob), alignment)
        data.extend(blob)
    return bytes(data)


def fixture():
    x86_old, x86_new = slice_pair(0x1000007)
    arm_old, arm_new = slice_pair(0x100000c)
    return (make_fat([(0x1000007, x86_old), (0x100000c, arm_old)], 8),
            make_fat([(0x1000007, x86_new), (0x100000c, arm_new)], 10))


class SignatureTransitionTests(unittest.TestCase):
    def test_signature_transition_and_content_tamper(self):
        original, signed = fixture()
        details = verifier.reconstruct(original, signed)
        self.assertEqual(details['x86_64'], {'reference_signature': 'absent', 'extra_zero_bytes': 8})
        self.assertEqual(details['arm64'], {'reference_signature': 'linker-ad-hoc', 'extra_zero_bytes': 0})
        changed = bytearray(original)
        x86 = struct.unpack_from('>IIIII', changed, 8)[2]
        changed[x86 + 260] ^= 1
        with self.assertRaisesRegex(ValueError, 'Non-signature bytes differ'):
            verifier.reconstruct(changed, signed)
        missing_cms = bytearray(signed)
        arm = verifier.fat(signed)[0x100000c]
        signature = verifier.image(arm['image'])['signature']
        struct.pack_into('>I', missing_cms, arm['offset'] + signature['offset'] + 20, 0x10001)
        with self.assertRaisesRegex(ValueError, 'SDK CMS signature missing'):
            verifier.reconstruct(original, missing_cms)

    def test_rejects_invalid_paths_and_overlapping_slices(self):
        for value in ('../private/file', 'lib/../../private', 'C:/private', 'lib\\private', 'other/file'):
            with self.subTest(value=value), self.assertRaises(ValueError):
                verifier.relative_path(value)
        original, _ = fixture()
        invalid = bytearray(original)
        first_offset = struct.unpack_from('>I', invalid, 16)[0]
        struct.pack_into('>I', invalid, 36, first_offset)
        with self.assertRaisesRegex(ValueError, 'Overlapping'):
            verifier.fat(invalid)

    def test_61_files_and_duplicate_release_records(self):
        original, signed = fixture()
        original_sha1 = hashlib.sha1(original).hexdigest()
        signed_sha256 = hashlib.sha256(signed).hexdigest()
        temp_root = ROOT / 'build/qt-sdk-signature-tests'
        temp_root.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=temp_root) as temporary:
            root = Path(temporary)
            sdk = root / 'sdk'
            (sdk / 'lib').mkdir(parents=True)
            wheel = root / 'reference.whl'
            records = []
            with zipfile.ZipFile(wheel, 'w') as archive:
                for index in range(61):
                    path = f'lib/QtOwned{index}'
                    (sdk / path).write_bytes(signed)
                    archive.writestr('PySide6/Qt/' + path, original)
                    records.append({'sdk_path': path, 'sdk_sha256': signed_sha256,
                        'results': [{'algorithm': 'SHA1', 'expected': original_sha1}]})
            records.extend(records[:49])
            contents = json.dumps({'files': records}).encode()
            materials = root / 'materials.tar.gz'
            with tarfile.open(materials, 'w:gz') as archive:
                info = tarfile.TarInfo('qt-binary-differences.json')
                info.size = len(contents)
                archive.addfile(info, io.BytesIO(contents))
            with patch.object(verifier, 'MATERIAL_SHA256', verifier.digest(materials)), \
                 patch.object(verifier, 'WHEEL_SHA256', verifier.digest(wheel)):
                report = verifier.verify(materials, wheel, sdk)
                self.assertEqual(report['release_difference_records'], 110)
                self.assertEqual(report['distinct_sdk_files'], 61)
                wheel_bytes = wheel.read_bytes()
                wheel.write_bytes(wheel_bytes + b'changed')
                with self.assertRaisesRegex(ValueError, 'Reference wheel digest mismatch'):
                    verifier.verify(materials, wheel, sdk)
                wheel.write_bytes(wheel_bytes)
                (sdk / 'lib/QtOwned0').write_bytes(signed + b'changed')
                with self.assertRaisesRegex(ValueError, 'SDK file missing or digest mismatch'):
                    verifier.verify(materials, wheel, sdk)


if __name__ == '__main__':
    unittest.main()
