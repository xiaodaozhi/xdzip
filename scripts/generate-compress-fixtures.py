#!/usr/bin/env python3
"""Regenerate owned Unix compress inputs and verify them with two independent readers."""
import argparse
import hashlib
import importlib
import json
import lzma
from pathlib import Path
import subprocess
import sys
import tempfile
import zipfile

ROOT = Path(__file__).resolve().parents[1]
DEST = ROOT / 'tests/fixtures/compress'
WHEEL = 'ncompress-1.0.2-cp312-abi3-win_amd64.whl'
WHEEL_SHA256 = '672f7166abf634df43a592ec2d1053c991057a1f7020d55288fa03ab60ebae98'
SEVENZIP_SHA256 = {'7z.exe':'dc0493429fe28136fcb177a0e77c9875cbb3b0bff5f2f8eeba4df302e98cef76',
                   '7z.dll':'500db9eb17f3c21b729ba41dd02ecfe1823da05028c7e4ba129084cc74cc364e'}


def pattern(count):
    state = 0x6d2b79f5
    output = bytearray()
    for _ in range(count):
        state ^= (state << 13) & 0xffffffff
        state ^= state >> 17
        state ^= (state << 5) & 0xffffffff
        output.append(state >> 24)
    return bytes(output)


def pack(codes, bits=9, block=True):
    """Pack explicit wire codes; width follows decoder dictionary occupancy."""
    output = bytearray([0x1f, 0x9d, bits | (128 if block else 0)])
    width, next_code, previous, group, used = 9, 257 if block else 256, False, 0, 0
    def flush(full=False):
        nonlocal group, used
        output.extend(group.to_bytes(width if full else (used + 7) // 8, 'little'))
        group, used = 0, 0
    for code in codes:
        if next_code == 1 << width and width < bits:
            if used:
                flush(True)
            width += 1
        group |= code << used
        used += width
        if used == width * 8:
            flush(True)
        if block and code == 256:
            if used:
                flush(True)
            width, next_code, previous = 9, 257, False
        else:
            if previous and next_code < 1 << bits:
                next_code += 1
            previous = True
    if used:
        flush()
    return bytes(output)


def encode(data, bits, block):
    """Owned textbook LZW producer, independently checked before a fixture is accepted."""
    dictionary = {bytes([value]):value for value in range(256)}
    next_code = 257 if block else 256
    previous, codes = b'', []
    for byte in data:
        symbol = bytes([byte])
        candidate = previous + symbol
        if candidate in dictionary:
            previous = candidate
        else:
            codes.append(dictionary[previous])
            if next_code < 1 << bits:
                dictionary[candidate] = next_code
                next_code += 1
            previous = symbol
    if previous:
        codes.append(dictionary[previous])
    return pack(codes, bits, block)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--reference', type=Path, required=True)
    parser.add_argument('--sevenzip', type=Path, required=True)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    ref, sevenzip = args.reference.resolve(), args.sevenzip.resolve()
    for folder in [ref, sevenzip]:
        if not folder.is_relative_to(ROOT / 'build/toolchain'):
            parser.error('Keep reference tools under build/toolchain/')
    wheel = ref / WHEEL
    if hashlib.sha256(wheel.read_bytes()).hexdigest() != WHEEL_SHA256:
        raise ValueError('Independent ncompress wheel identity mismatch')
    with zipfile.ZipFile(wheel) as archive:
        for entry in archive.infolist():
            if entry.is_dir() or entry.filename.endswith('.dist-info/RECORD'):
                continue
            target = ref / 'python' / entry.filename
            if target.is_symlink() or target.read_bytes() != archive.read(entry):
                raise ValueError('Installed reference differs from pinned wheel: ' + entry.filename)
    for name, digest in SEVENZIP_SHA256.items():
        if hashlib.sha256((sevenzip / 'tools' / name).read_bytes()).hexdigest() != digest:
            raise ValueError('Independent 7-Zip identity mismatch')
    sys.path.insert(0, str(ref / 'python'))
    reference = importlib.import_module('ncompress')
    phrase = b'abracadabra abracadabra\n' * 6
    cases = {}
    for name, payload in [('empty.Z', b''), ('phrase.Z', phrase), ('random.Z', pattern(90000)),
                          ('natural-clear.Z', phrase * 4000 + pattern(250000)),
                          ('zeros.Z', bytes(4096)), ('empty-tar.Z', bytes(1024)),
                          ('files.taz', lzma.decompress((ROOT / 'tests/fixtures/streams/files.txz').read_bytes()))]:
        cases[name] = reference.compress(payload), payload
    cases['nonblock.Z'] = pack([65, 66, 256, 258], block=False), b'ABABABA'
    cases['clear.Z'] = pack([65, 66, 257, 256, 67, 68, 257]), b'ABABCDCD'
    for bits in range(9, 17):
        payload = pattern((2 << bits) + 2048)
        cases[f'width-{bits}.Z'] = encode(payload, bits, True), payload
    payload = pattern(5000)
    cases['nonblock-width.Z'] = encode(payload, 12, False), payload
    report = ROOT / 'build/review/format-expansion'
    report.mkdir(parents=True, exist_ok=True)
    DEST.mkdir(exist_ok=True)
    digests = {}
    with tempfile.TemporaryDirectory(prefix='compress-reference-', dir=report) as temporary:
        source = Path(temporary) / 'owned.Z'
        for name, (encoded, payload) in cases.items():
            if reference.decompress(encoded) != payload:
                raise ValueError('Independent ncompress mismatch: ' + name)
            source.write_bytes(encoded)
            run = subprocess.run([str(sevenzip / 'tools/7z.exe'), 'e', '-so', '-bd', '-y', '-tZ', str(source)],
                                 capture_output=True, timeout=30)
            nine_bit_limitation = name == 'width-9.Z' and run.returncode == 2 and run.stdout == payload[:256]
            if run.returncode != 0 and not nine_bit_limitation:
                raise ValueError(f'{name}: independent 7-Zip exit {run.returncode}: {run.stderr.decode(errors="replace")[:250]}')
            if run.stdout != payload and not nine_bit_limitation:
                raise ValueError('Independent 7-Zip mismatch: ' + name)
            target = DEST / name
            if args.check:
                if target.read_bytes() != encoded:
                    raise ValueError('Stale owned compress input: ' + name)
            else:
                target.write_bytes(encoded)
            digests[name] = {'sha256':hashlib.sha256(encoded).hexdigest(), 'packed':len(encoded), 'decoded':len(payload),
                             'ncompress':True, 'sevenzip':not nine_bit_limitation, 'sevenzip_exit':run.returncode}
    (report / 'm7-z-reference.json').write_text(json.dumps(digests, indent=2) + '\n', encoding='utf-8')
    print(f'{len(cases)} owned compress inputs verified by ncompress; {len(cases)-1} also verified by 7-Zip; saturated 9-bit limitation recorded')


if __name__ == '__main__':
    main()
