"""Owned deterministic optical vectors, generated only below build/.

ISO records follow ECMA-119 directory records and volume descriptors. El Torito
uses a no-emulation entry and a validation-entry 16-bit checksum. UDF mutations
use ECMA-167 descriptor tags, File Entry short_ad fields, and CRC-16 polynomial
0x1021. These are independent test writers, never a product parsing dependency.
"""
import hashlib
import json
from pathlib import Path
import struct

BLOCK = 2048
BOOT = b'Owned El Torito data, never executed.\n'.ljust(BLOCK, b'\0')
FIRST = bytes(range(256)) * 8
SECOND = b'owned second ISO extent\n'
FAR = b'owned payload beyond four GiB\n'


def both16(value):
    return struct.pack('<H', value) + struct.pack('>H', value)


def both32(value):
    return struct.pack('<I', value) + struct.pack('>I', value)


def record(name, extent, length, flags=0):
    name = name if isinstance(name, bytes) else name.encode('ascii')
    size = 33 + len(name) + (len(name) % 2 == 0)
    return bytes([size, 0]) + both32(extent) + both32(length) + bytes([126, 1, 2, 3, 4, 5, 0, flags, 0, 0]) + both16(1) + bytes([len(name)]) + name + bytes(size - 33 - len(name))


def iso_image(files, blocks=30, boot=False):
    """Small root-only ECMA-119 image; duplicate names may form multi-extents."""
    image = bytearray(BLOCK * blocks)
    pvd = bytearray(BLOCK)
    pvd[:7] = b'\x01CD001\x01'
    pvd[8:40] = b'XDZIP OWNED TEST'.ljust(32, b' ')
    pvd[40:72] = b'OPTICAL INTEROP'.ljust(32, b' ')
    pvd[80:88] = both32(blocks)
    pvd[120:124] = both16(1)
    pvd[124:128] = both16(1)
    pvd[128:132] = both16(BLOCK)
    pvd[132:140] = both32(10)
    pvd[140:144] = struct.pack('<I', 20)
    pvd[148:152] = struct.pack('>I', 21)
    pvd[156:190] = record(b'\0', 22, BLOCK, 2)
    pvd[881] = 1
    image[16 * BLOCK:17 * BLOCK] = pvd
    end = 17
    if boot:
        descriptor = bytearray(BLOCK)
        descriptor[:7] = b'\x00CD001\x01'
        descriptor[7:39] = b'EL TORITO SPECIFICATION'.ljust(32, b'\0')
        descriptor[71:75] = struct.pack('<I', 23)
        image[17 * BLOCK:18 * BLOCK] = descriptor
        end = 18
    image[end * BLOCK:end * BLOCK + 7] = b'\xffCD001\x01'
    image[20 * BLOCK:20 * BLOCK + 10] = b'\x01\x00' + struct.pack('<I', 22) + struct.pack('<H', 1) + b'\0\0'
    image[21 * BLOCK:21 * BLOCK + 10] = b'\x01\x00' + struct.pack('>I', 22) + struct.pack('>H', 1) + b'\0\0'
    directory = record(b'\0', 22, BLOCK, 2) + record(b'\1', 22, BLOCK, 2)
    for name, extent, payload, flags in files:
        directory += record(name, extent, len(payload), flags)
        at = extent * BLOCK
        if at < len(image):
            image[at:at + len(payload)] = payload
    if len(directory) > BLOCK:
        raise ValueError('owned root directory overflow')
    image[22 * BLOCK:22 * BLOCK + len(directory)] = directory
    return bytes(image)


def crc16(data):
    result = 0
    for byte in data:
        result ^= byte << 8
        for _ in range(8):
            result = ((result << 1) ^ (0x1021 if result & 0x8000 else 0)) & 0xffff
    return result


def retag(image, at):
    size = struct.unpack_from('<H', image, at + 10)[0]
    struct.pack_into('<H', image, at + 8, crc16(image[at + 16:at + 16 + size]))
    image[at + 4] = 0
    image[at + 4] = sum(image[at:at + 16]) & 0xff


def sparse_udf(original):
    image = bytearray(original)
    selected = None
    for at in range(0, len(image), BLOCK):
        if (struct.unpack_from('<H', image, at)[0] == 261
                and image[at + 27] == 5
                and struct.unpack_from('<Q', image, at + 56)[0] == 32
                and struct.unpack_from('<I', image, at + 168)[0] == 0
                and struct.unpack_from('<I', image, at + 172)[0] == 8):
            selected = at
            break
    if selected is None:
        raise ValueError('expected owned UDF File Entry not found')
    # short_ad type 2 is unallocated and unrecorded; its logical bytes are zero.
    struct.pack_into('<Q', image, selected + 64, 0)
    struct.pack_into('<II', image, selected + 176, (2 << 30) | 32, 0)
    retag(image, selected)
    return bytes(image)


def generate(output, prepared):
    output = Path(output).resolve()
    root = Path(__file__).resolve().parents[3]
    if not output.is_relative_to(root / 'build'):
        raise ValueError('owned optical vectors belong below build/')
    output.mkdir(parents=True, exist_ok=True)
    cases = []

    def store(name, data, expected, format='iso'):
        path = output / name
        path.write_bytes(data)
        cases.append(dict(name=name, path=str(path), format=format,
                          expected={key: value.hex() for key, value in expected.items()},
                          sha256=hashlib.sha256(data).hexdigest()))
        return path

    store('multi-extent.iso', iso_image([
        ('MULTI.BIN;1', 24, FIRST, 0x80), ('MULTI.BIN;1', 26, SECOND, 0)]),
        {'MULTI.BIN': FIRST + SECOND})
    legacy_names = {
        'gbk': ['中文文件与压缩测试.txt', '跨平台解压使用说明.txt', '目录.txt', '目录1.txt'],
        'cp932': ['ソフトウェア説明書.txt', 'ひらがなカタカナ.txt'],
    }
    for encoding, names in legacy_names.items():
        files = [(name.encode(encoding) + b';1', 24 + index, FAR, 0)
                 for index, name in enumerate(names)]
        store('legacy-' + encoding + '.iso', iso_image(files), {name: FAR for name in names})
        cases[-1]['expected_raw_names'] = {name: name.encode(encoding).hex() for name in names}
        cases[-1]['legacy_encoding'] = encoding
        if encoding == 'gbk':
            for name in names[-2:]:
                if name.encode(encoding).decode('utf-8') == name:
                    raise ValueError('legacy overlap must have a different UTF-8 interpretation')
    catalog = bytearray(BLOCK)
    catalog[0] = 1
    catalog[4:28] = b'XDZIP OWNED BOOT FIXTURE'.ljust(24, b' ')[:24]
    catalog[30:32] = b'\x55\xaa'
    checksum = (-sum(struct.unpack('<16H', catalog[:32]))) & 0xffff
    struct.pack_into('<H', catalog, 28, checksum)
    catalog[32] = 0x88
    struct.pack_into('<H', catalog, 38, 4)
    struct.pack_into('<I', catalog, 40, 24)
    store('eltorito.iso', iso_image([
        ('BOOT.CAT;1', 23, bytes(catalog), 0), ('BOOT.IMG;1', 24, BOOT, 0)], boot=True),
        {'BOOT.CAT': bytes(catalog), 'BOOT.IMG': BOOT, '[BOOT]/Boot-NoEmul.img': BOOT})

    far_lba = (1 << 32) // BLOCK + 1
    header = bytearray(iso_image([('FAR.TXT;1', far_lba, FAR, 0)]))
    header[16 * BLOCK + 80:16 * BLOCK + 88] = both32(far_lba + 1)
    (output / 'far-header.bin').write_bytes(header)
    (output / 'far-payload.bin').write_bytes(FAR)
    mapping = {'length': (far_lba + 1) * BLOCK, 'segments': [
        {'offset': 0, 'file': 'far-header.bin'},
        {'offset': far_lba * BLOCK, 'file': 'far-payload.bin'}]}
    path = output / 'far-source.json'
    path.write_text(json.dumps(mapping, indent=2) + '\n', encoding='utf-8')
    cases.append(dict(name='iso-virtual-beyond-4gib', path=str(path), format='iso', virtual=True,
                      expected={'FAR.TXT': FAR.hex()}))

    udf_regular = (Path(prepared) / 'udf-regular.iso').read_bytes()
    store('udf-sparse.iso', sparse_udf(udf_regular), {}, format='udf')
    return cases
