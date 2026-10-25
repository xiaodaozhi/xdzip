"""Prepare fixed Microsoft-authored CAB fixtures, or author replacements on Windows.

Normal preparation needs only Python and fixed repository inputs. --author uses
the Windows makecab test reference, with only project-owned generated contents.
All outputs belong below repository build/. The author output must be empty.
"""
import argparse
import gzip
import hashlib
import io
import json
import os
from pathlib import Path
import platform
import struct
import subprocess
import zlib

ROOT = Path(__file__).resolve().parents[3]
BASE = Path(__file__).resolve().parent
MAX_CAB = 8 * 1024 * 1024
MAX_GZIP = 512 * 1024


def digest(data):
    return hashlib.sha256(data).hexdigest()


def payloads(recipe):
    if recipe in ('pattern-multifile', 'mszip-history-multifile'):
        data = bytes(range(256)) * 768 + b'CAB LZX cross frame and file boundary.\n'
    elif recipe == 'aligned-multifile':
        seed = b''.join(hashlib.sha256(struct.pack('<I', index)).digest() for index in range(512))
        data = b''.join(seed[hashlib.sha256(struct.pack('<I', index)).digest()[0] * 64:][:64]
                        for index in range(32768))
    else:
        raise ValueError('Unknown LZX payload recipe')
    return {'first.bin': data[:65551], 'second.bin': data[65551:], 'empty.bin': b''}


def output_path(value, empty=False):
    output = Path(value).resolve()
    build = (ROOT / 'build').resolve()
    if not output.is_relative_to(build) or output == build:
        raise ValueError('Outputs belong below repository build/')
    if empty and output.exists() and (not output.is_dir() or any(output.iterdir())):
        raise ValueError('Author output must not exist or must be empty')
    output.mkdir(parents=True, exist_ok=True)
    return output


def cabinet_info(data):
    if (data[:4] != b'MSCF' or len(data) < 44 or struct.unpack_from('<H', data, 30)[0] != 0 or
            struct.unpack_from('<H', data, 26)[0] != 1):
        raise ValueError('Expected ordinary single-folder authored CAB')
    start, count, method = struct.unpack_from('<IHH', data, 36)
    return {'frames': count, 'window_bits': method >> 8 & 31,
            'method': method & 15, 'data_offset': start}


def mszip_history(data, info, expected):
    at = info['data_offset']
    history = b''
    required = 0
    output = bytearray()
    for _ in range(info['frames']):
        packed, unpacked = struct.unpack_from('<HH', data, at + 4)
        frame = data[at + 8:at + 8 + packed]
        if frame[:2] != b'CK':
            raise ValueError('Authored MSZIP frame marker')
        decoder = zlib.decompressobj(-15, zdict=history)
        decoded = decoder.decompress(frame[2:]) + decoder.flush()
        if len(decoded) != unpacked or not decoder.eof or decoder.unused_data or decoder.unconsumed_tail:
            raise ValueError('Independent zlib MSZIP frame boundary mismatch')
        if history:
            try:
                zlib.decompress(frame[2:], -15)
            except zlib.error:
                required += 1
        history = (history + decoded)[-32768:]
        output.extend(decoded)
        at += 8 + packed
    if output != expected:
        raise ValueError('Independent zlib MSZIP known-payload mismatch')
    if not required:
        raise ValueError('MSZIP input does not require cross-frame history')
    return required


def author(value):
    output = output_path(value, empty=True)
    if any(char in str(output) for char in '\r\n"'):
        raise ValueError('Output cannot be represented in a makecab directive')
    tool = Path(os.environ['SystemRoot']) / 'System32/makecab.exe'
    version = subprocess.run(['powershell.exe', '-NoProfile', '-Command',
                              '(Get-Item -LiteralPath "$env:SystemRoot\\System32\\makecab.exe").VersionInfo.FileVersion'],
                             capture_output=True, text=True, check=True, timeout=15).stdout.strip()
    groups = {'LZX': [], 'MSZIP': []}
    jobs = [(f'lzx-window{bits}', bits, 'pattern-multifile', 'LZX') for bits in range(15, 22)]
    jobs += [('lzx-aligned21', 21, 'aligned-multifile', 'LZX'),
             ('mszip-history', 15, 'mszip-history-multifile', 'MSZIP')]
    for label, bits, recipe, compression in jobs:
        case = output / label
        case.mkdir()
        known = payloads(recipe)
        for name, data in known.items():
            (case / name).write_bytes(data)
        lines = ['.OPTION EXPLICIT', '.Set Cabinet=ON', '.Set Compress=ON',
                 '.Set CompressionType=' + compression,
                 '.Set MaxDiskSize=0', '.Set FolderSizeThreshold=104857600',
                 '.Set CabinetNameTemplate=' + label + '.cab',
                 '.Set DiskDirectoryTemplate="' + str(case) + '"',
                 '.Set InfFileName="' + str(case / 'author.inf') + '"',
                 '.Set RptFileName="' + str(case / 'author.rpt') + '"']
        if compression == 'LZX':
            lines.append('.Set CompressionMemory=' + str(bits))
        lines += ['"' + str(case / name) + '" "' + name + '"' for name in known]
        ddf = case / 'author.ddf'
        ddf.write_text('\n'.join(lines) + '\n', encoding='utf-8')
        result = subprocess.run([str(tool), '/F', str(ddf)], capture_output=True, timeout=60)
        (case / 'author.log').write_bytes(result.stdout + result.stderr)
        if result.returncode:
            raise ValueError('Microsoft makecab failed: ' + label)
        data = (case / (label + '.cab')).read_bytes()
        info = cabinet_info(data)
        if (info['method'] != (3 if compression == 'LZX' else 1) or info['frames'] < 2 or
                (compression == 'LZX' and info['window_bits'] != bits)):
            raise ValueError('Authored LZX method/window/frame mismatch')
        packed = io.BytesIO()
        with gzip.GzipFile(fileobj=packed, mode='wb', filename='', mtime=0) as compressor:
            compressor.write(data)
        stored = packed.getvalue()
        if len(data) > MAX_CAB or len(stored) > MAX_GZIP:
            raise ValueError('Authored fixture exceeds fixed input limit')
        name = label + '.cab.gz'
        (output / name).write_bytes(stored)
        record = {'file': name, 'case': label, 'recipe': recipe, 'compression': compression,
                        'compressed_bytes': len(stored), 'sha256': digest(stored),
                        'decoded_bytes': len(data), 'decoded_sha256': digest(data),
                        'window_bits': bits, 'frames': info['frames'],
                        'payloads': [{'name': key, 'bytes': len(value), 'sha256': digest(value)}
                                     for key, value in known.items()]}
        if compression == 'MSZIP':
            record['history_required_frames'] = mszip_history(data, info, b''.join(known.values()))
            record['independent_algorithm_check'] = {'tool': 'Python zlib', 'version': zlib.ZLIB_RUNTIME_VERSION,
                                                     'scope': 'Every frame terminates exactly and the full known input matches; later frames require history.'}
        groups[compression].append(record)
    manifest = {'schema_version': 1, 'generator': 'tests/fixtures/cab/generate-makecab.py',
                'source_kind': 'independent_Microsoft_makecab', 'content_license': 'Apache-2.0',
                'content': 'Project-owned deterministic files only; no user files or Microsoft executable code.',
                'tool': {'name': 'Microsoft Windows makecab.exe', 'file_version': version,
                         'sha256': digest(tool.read_bytes()), 'os': platform.platform(),
                         'official_documentation': 'https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/makecab'},
                'recipes': {'pattern-multifile': 'bytes0..255 repeated768 times, then ASCII CAB LZX cross frame and file boundary. plus LF; split at65551 into first.bin and second.bin, then empty.bin.',
                            'mszip-history-multifile': 'Same deterministic bytes and65551 split as pattern-multifile; independent zlib verifies that later frames cannot decode without preceding32KiB history.',
                            'aligned-multifile': 'seed=SHA256(LE32(i)) concatenated for i=0..511 (16KiB); for i=0..32767 select seed chunk of64B using SHA256(LE32(i))[0]; split concatenation at65551, then empty.bin.'},
                'validation_scope': 'Known ordinary-file bytes and exercised LZX frame/window/block behavior; complete CAB host-service validation is performed separately.',
                'fixtures': []}
    for compression, records in groups.items():
        manifest['fixtures'] = records
        manifest['compression'] = compression
        (output / (compression.lower() + '-sources.json')).write_text(json.dumps(manifest, indent=2) + '\n', encoding='utf-8')
    records = [item for group in groups.values() for item in group]
    print(json.dumps({'fixtures': len(records), 'compressed_bytes': sum(x['compressed_bytes'] for x in records)}, indent=2))


def prepare(value):
    output = output_path(value)
    cases = []
    for filename, count in [('lzx-sources.json', 8), ('mszip-sources.json', 1)]:
        manifest = json.loads((BASE / filename).read_text(encoding='utf-8'))
        if manifest['schema_version'] != 1 or len(manifest['fixtures']) != count:
            raise ValueError('Unsupported makecab fixture manifest')
        cases.extend(prepare_group(output, manifest))
    return cases


def prepare_group(output, manifest):
    cases = []
    for item in manifest['fixtures']:
        name = item['file']
        if Path(name).name != name or '/' in name or '\\' in name or not name.endswith('.cab.gz'):
            raise ValueError('Unexpected fixed LZX fixture name')
        archive = BASE / name
        if not 0 < archive.stat().st_size <= MAX_GZIP:
            raise ValueError('Fixed compressed LZX size limit')
        stored = archive.read_bytes()
        if len(stored) != item['compressed_bytes'] or digest(stored) != item['sha256']:
            raise ValueError('Fixed compressed LZX identity mismatch')
        with gzip.GzipFile(fileobj=io.BytesIO(stored)) as source:
            data = source.read(MAX_CAB + 1)
        if len(data) > MAX_CAB or len(data) != item['decoded_bytes'] or digest(data) != item['decoded_sha256']:
            raise ValueError('Fixed LZX identity mismatch')
        info = cabinet_info(data)
        if (info['method'] != (3 if item['compression'] == 'LZX' else 1) or info['frames'] != item['frames'] or
                (item['compression'] == 'LZX' and info['window_bits'] != item['window_bits'])):
            raise ValueError('Fixed LZX declared structure mismatch')
        known = payloads(item['recipe'])
        identity = [{'name': key, 'bytes': len(value), 'sha256': digest(value)} for key, value in known.items()]
        if identity != item['payloads']:
            raise ValueError('Fixed LZX known-payload identity mismatch')
        target = output / name[:-3]
        temporary = target.with_suffix('.cab.part')
        with temporary.open('xb') as stream:
            stream.write(data)
        temporary.replace(target)
        cases.append({'name': item['case'], 'path': str(target), 'expected': known,
                      'compression': item['compression'], 'window_bits': item['window_bits'], 'frames': item['frames'],
                      'source': manifest['source_kind'], 'fixture_tools': manifest['tool']})
    return cases


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--author', action='store_true')
    args = parser.parse_args()
    if args.author:
        author(args.output)
    else:
        print(json.dumps([{key: value for key, value in case.items() if key != 'expected'}
                          for case in prepare(args.output)], indent=2))
