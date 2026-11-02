#!/usr/bin/env python3
"""Reproduce alpha.8 Qt SDK differences from pinned published reference bytes."""
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import struct
import tarfile
import zipfile

from deployment_audit import ROOT, digest, within

MATERIAL_SHA256 = '1f790160bb227549d35bfeecfefb41f45208aa9179ab53e1cfd5ed9f88507fe4'
WHEEL_SHA256 = 'abe8446367413b7b56626352e05adc7b15a180956ac9d90e4ee6419c16eab23d'
ARCHES = {0x1000007: 'x86_64', 0x100000c: 'arm64'}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def relative_path(value):
    require(isinstance(value, str) and value and '\\' not in value and ':' not in value and
            not any(ord(char) < 32 for char in value), 'Invalid SDK-relative path')
    path = PurePosixPath(value)
    require(not path.is_absolute() and '..' not in path.parts and len(path.parts) >= 2 and
            path.parts[0] in {'lib', 'qml', 'plugins'}, 'Invalid SDK-relative path')
    return path


def fat(data):
    require(len(data) >= 48, 'Truncated universal image')
    magic, count = struct.unpack_from('>II', data)
    require(magic == 0xcafebabe and count == 2, 'Expected two-architecture universal image')
    entries = {}
    ranges = []
    for i in range(count):
        cpu, subtype, offset, size, alignment = struct.unpack_from('>IIIII', data, 8 + i * 20)
        require(cpu in ARCHES and cpu not in entries and alignment <= 20 and offset >= 48 and
                offset % (1 << alignment) == 0 and offset + size <= len(data), 'Invalid universal slice')
        entries[cpu] = {'subtype': subtype, 'offset': offset, 'alignment': alignment,
                        'image': data[offset:offset + size]}
        ranges.append((offset, offset + size))
    require(set(entries) == set(ARCHES) and
            (ranges[0][1] <= ranges[1][0] or ranges[1][1] <= ranges[0][0]), 'Overlapping universal slices')
    return entries


def signature_slots(data, start, size):
    require(size >= 20 and start + size <= len(data), 'Invalid code signature bounds')
    magic, total, count = struct.unpack_from('>III', data, start)
    require(magic == 0xfade0cc0 and total <= size and count <= 32 and
            12 + count * 8 <= total, 'Invalid code signature container')
    slots = {}
    for i in range(count):
        slot, offset = struct.unpack_from('>II', data, start + 12 + i * 8)
        require(slot not in slots and offset + 8 <= total, 'Invalid code signature slot')
        magic, length = struct.unpack_from('>II', data, start + offset)
        require(length >= 8 and offset + length <= total, 'Invalid code signature component')
        slots[slot] = (magic, start + offset, length)
    require(0 in slots and slots[0][0] == 0xfade0c02 and slots[0][2] >= 16,
            'Missing code directory')
    require(0x10000 not in slots or slots[0x10000][0] == 0xfade0b01, 'Invalid CMS signature slot')
    flags = struct.unpack_from('>I', data, slots[0][1] + 12)[0]
    return {'flags': flags, 'cms': 0x10000 in slots}


def image(data):
    require(len(data) >= 32 and data[:4] == b'\xcf\xfa\xed\xfe', 'Invalid 64-bit Mach-O image')
    count, length = struct.unpack_from('<II', data, 16)
    require(count <= 4096 and 32 + length <= len(data), 'Invalid load-command bounds')
    commands, position, signature = [], 32, None
    for _ in range(count):
        require(position + 8 <= 32 + length, 'Truncated load command')
        kind, size = struct.unpack_from('<II', data, position)
        require(size >= 8 and size % 8 == 0 and position + size <= 32 + length, 'Invalid load command')
        block = data[position:position + size]
        if kind == 0x1d:
            require(signature is None and size == 16, 'Invalid signature command')
            offset, sigsize = struct.unpack_from('<II', block, 8)
            details = signature_slots(data, offset, sigsize)
            signature = {'offset': offset, 'size': sigsize, **details}
        commands.append((kind, block))
        position += size
    require(position == 32 + length, 'Invalid load-command size')
    return {'commands': commands, 'end': position, 'signature': signature}


def reconstruct_slice(original, signed, arch):
    old, new = image(original), image(signed)
    require(new['signature'] is not None and new['signature']['cms'], 'SDK CMS signature missing')
    if arch == 'arm64':
        require(old['signature'] is not None and not old['signature']['cms'] and
                old['signature']['flags'] & 0x20002 == 0x20002, 'Expected linker ad-hoc signature')
    else:
        require(old['signature'] is None, 'Expected unsigned x86_64 reference')
    old_commands = [(kind, block) for kind, block in old['commands'] if kind != 0x1d]
    new_commands = [(kind, block) for kind, block in new['commands'] if kind != 0x1d]
    require(len(old_commands) == len(new_commands), 'Unexpected non-signature command count')
    rebuilt = []
    for (old_kind, old_block), (new_kind, new_block) in zip(old_commands, new_commands):
        require(old_kind == new_kind, 'Unexpected non-signature command')
        block = bytearray(old_block)
        if old_kind == 0x19:
            require(len(block) >= 72, 'Truncated segment command')
            if block[8:24].rstrip(b'\0') == b'__LINKEDIT':
                block[32:40] = new_block[32:40]
                block[48:56] = new_block[48:56]
        require(block == new_block, 'Unexpected non-signature command change')
        rebuilt.append(bytes(block))
    remaining = iter(rebuilt)
    commands = b''.join(block if kind == 0x1d else next(remaining)
                        for kind, block in new['commands'])
    require(next(remaining, None) is None and len(commands) == new['end'] - 32,
            'Invalid reconstructed commands')
    header = bytearray(original[:32])
    header[16:24] = signed[16:24]
    require(header == signed[:32], 'Unexpected Mach-O header change')
    old_end = old['signature']['offset'] if old['signature'] else len(original)
    new_start, new_size = new['signature']['offset'], new['signature']['size']
    require(old_end <= new_start and old['end'] <= new['end'] <= old_end and
            not any(original[old['end']:new['end']]), 'Unexpected content before signature')
    result = bytearray(original[:old_end])
    result.extend(bytes(new_start - old_end))
    result[:32] = header
    result[32:new['end']] = commands
    result.extend(signed[new_start:new_start + new_size])
    require(len(result) <= len(signed) and not any(signed[len(result):]), 'Unexpected signature trailer')
    result.extend(bytes(len(signed) - len(result)))
    require(result == signed, 'Non-signature bytes differ')
    return bytes(result), {'reference_signature': 'linker-ad-hoc' if arch == 'arm64' else 'absent',
                           'extra_zero_bytes': new_start - old_end}


def reconstruct(original, signed):
    old, new = fat(original), fat(signed)
    result = bytearray(len(signed))
    struct.pack_into('>II', result, 0, 0xcafebabe, 2)
    details = {}
    for index in range(2):
        cpu = struct.unpack_from('>I', signed, 8 + index * 20)[0]
        source, target = old[cpu], new[cpu]
        require(source['subtype'] == target['subtype'], 'Unexpected architecture subtype')
        rebuilt, detail = reconstruct_slice(source['image'], target['image'], ARCHES[cpu])
        struct.pack_into('>IIIII', result, 8 + index * 20, cpu, source['subtype'], target['offset'],
                         len(rebuilt), target['alignment'])
        result[target['offset']:target['offset'] + len(rebuilt)] = rebuilt
        details[ARCHES[cpu]] = detail
    require(result == signed, 'Unexpected universal-container change')
    return details


def release_records(materials):
    require(digest(materials) == MATERIAL_SHA256, 'Release materials digest mismatch')
    with tarfile.open(materials, 'r:gz') as archive:
        found = [item for item in archive.getmembers() if item.name == 'qt-binary-differences.json']
        require(len(found) == 1 and found[0].isfile() and found[0].size <= 2 * 1024 * 1024,
                'Invalid release difference report')
        records = json.load(archive.extractfile(found[0]))['files']
    require(len(records) == 110, 'Unexpected release difference count')
    grouped = {}
    for item in records:
        path = relative_path(item['sdk_path']).as_posix()
        require(len(item['results']) == 1 and item['results'][0]['algorithm'] == 'SHA1',
                'Unexpected release checksum')
        pair = (item['sdk_sha256'], item['results'][0]['expected'])
        require(path not in grouped or grouped[path] == pair, 'Conflicting release records')
        grouped[path] = pair
    require(len(grouped) == 61, 'Unexpected distinct SDK file count')
    return grouped


def verify(materials, wheel, sdk):
    records = release_records(materials)
    require(digest(wheel) == WHEEL_SHA256, 'Reference wheel digest mismatch')
    require(sdk.is_dir(), 'SDK directory missing')
    results = []
    with zipfile.ZipFile(wheel) as archive:
        names = archive.namelist()
        require(len(names) == len(set(names)), 'Duplicate reference wheel member')
        for name, (expected_sdk, expected_sha1) in sorted(records.items()):
            relative = relative_path(name)
            file = sdk.joinpath(*relative.parts)
            require(within(file, sdk) and file.is_file() and digest(file) == expected_sdk,
                    'SDK file missing or digest mismatch')
            member = 'PySide6/Qt/' + relative.as_posix()
            require(member in names and archive.getinfo(member).file_size <= 100 * 1024 * 1024,
                    'Reference wheel member missing or too large')
            reference = archive.read(member)
            require(hashlib.sha1(reference).hexdigest() == expected_sha1, 'Reference SHA-1 mismatch')
            details = reconstruct(reference, file.read_bytes())
            results.append({'sdk_path': name, 'reference_sha1': expected_sha1,
                            'reference_sha256': hashlib.sha256(reference).hexdigest(),
                            'sdk_sha256': expected_sdk, 'architectures': details})
    return {'schema_version': 1, 'status': 'exact-signature-layout-reconstruction',
            'release_revision': 'e1cd6a3e125d5fcabc85a9f4edb106d013709c89',
            'materials_sha256': MATERIAL_SHA256, 'reference_wheel_sha256': WHEEL_SHA256,
            'release_difference_records': 110, 'distinct_sdk_files': len(results), 'files': results,
            'limits': ['The SDK signature bytes are copied from the already distributed SDK.',
                       'This tests exact byte relationships, not the historical signing actor or command.',
                       'This is not a license approval, publisher trust verification or source rebuild.']}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--materials', required=True, type=Path)
    parser.add_argument('--reference-wheel', required=True, type=Path)
    parser.add_argument('--qt-prefix', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    require(within(args.output, ROOT / 'build') and args.output.parent.is_dir() and not args.output.exists(),
            'Use a new output file inside build/')
    result = verify(args.materials, args.reference_wheel, args.qt_prefix)
    with args.output.open('x', encoding='utf-8') as stream:
        json.dump(result, stream, indent=2)
        stream.write('\n')
    print('Reconstructed 61 Qt SDK files exactly from signature-related differences')


if __name__ == '__main__':
    try:
        main()
    except (OSError, tarfile.TarError, zipfile.BadZipFile, KeyError, TypeError, struct.error):
        raise SystemExit('Invalid or unreadable evidence input') from None
    except ValueError as error:
        raise SystemExit(str(error)) from None
