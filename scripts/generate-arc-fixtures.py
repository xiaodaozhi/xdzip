#!/usr/bin/env python3
"""Generate owned ARC wire inputs and verify complete bytes with two references."""
from __future__ import annotations
import argparse
import hashlib
import heapq
import json
import os
from pathlib import Path
import struct
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
REFERENCE_SHA = 'd5d2e5b47154da2766589e8f8007eb0bc8ae320cfae69d2bb6b226c1023171b4'
NOMARCH_SHA = '1382786fc49ed66dffc649a7418f020a121bbeea913ecb56c4d6d3421e397706'
INPUT_SHA = '6e4cda347196139b25849237493d0c392dcad72a2cda1deca321941e55a6e547'


def crc16(data):
    crc = 0
    for value in data:
        crc ^= value
        for _ in range(8):
            crc = (crc >> 1) ^ (0xa001 if crc & 1 else 0)
    return crc


def plain(kind):
    if kind.startswith('clear-'):
        return bytes(range(65, 65 + int(kind[6:]))) + b'AFTERRESET' * 12
    if kind == 'empty':
        return b''
    if kind == 'phrase':
        return b'abracadabra abracadabra\n' * 128
    if kind == 'allbytes':
        return bytes(range(256)) * 256
    if kind == 'runs':
        return b''.join(bytes([i]) * (3 + i % 253) for i in range(256))
    if kind == 'escaped':
        return bytes([0x90]) * 257 + b'A' * 512 + b'\x90B\x90\x90C' * 128
    if kind == 'random':
        state = 0x12345678
        result = bytearray()
        for _ in range(65536):
            state ^= state << 13 & 0xffffffff
            state ^= state >> 17
            state ^= state << 5 & 0xffffffff
            result.append(state & 255)
        return bytes(result)
    raise ValueError(kind)


def rle(data):
    out = bytearray()
    at = 0
    while at < len(data):
        value = data[at]
        end = at + 1
        while end < len(data) and data[end] == value and end - at < 255:
            end += 1
        count = end - at
        if value == 0x90:
            out += b'\x90\0' * count
        elif count >= 3:
            out += bytes([value, 0x90, count])
        else:
            out += bytes([value]) * count
        at = end
    return bytes(out)


def squeeze(data):
    counts = [0] * 257
    for value in data:
        counts[value] += 1
    counts[256] = 1
    heap = [(count, value, value) for value, count in enumerate(counts) if count]
    heapq.heapify(heap)
    serial = 257
    while len(heap) > 1:
        a = heapq.heappop(heap)
        b = heapq.heappop(heap)
        heapq.heappush(heap, (a[0] + b[0], serial, (a[2], b[2])))
        serial += 1
    tree = heap[0][2]
    if isinstance(tree, int):
        tree = (tree, tree)
    nodes, paths = [], {}

    def visit(node, bits):
        if isinstance(node, int):
            paths[node] = bits
            return -node - 1
        index = len(nodes)
        nodes.append(None)
        left = visit(node[0], bits + (0,))
        right = visit(node[1], bits + (1,))
        nodes[index] = (left, right)
        return index

    visit(tree, ())
    bits = [bit for value in [*data, 256] for bit in paths[value]]
    packed = bytearray(struct.pack('<H', len(nodes)))
    for node in nodes:
        packed += struct.pack('<hh', *node)
    for at in range(0, len(bits), 8):
        packed.append(sum(bit << shift for shift, bit in enumerate(bits[at:at + 8])))
    return bytes(packed)


class FixedDictionary:
    def __init__(self, improved):
        self.improved = improved
        self.entries = [None] * 4096
        self.links = [None] * 4096
        self.strings = {}
        for value in range(256):
            self.add(0xffff, value, bytes([value]))

    def add(self, prefix, suffix, text):
        key = (prefix + suffix) & 0xffff
        at = (key * 15073 & 4095) if self.improved else (((key | 0x800) ** 2 >> 6) & 4095)
        if self.entries[at] is not None:
            while self.links[at] is not None:
                at = self.links[at]
            last = at
            at = (at + 101) & 4095
            while self.entries[at] is not None:
                at = (at + 1) & 4095
            self.links[last] = at
        self.entries[at] = text
        self.strings[text] = at


def fixed_lzw(data, improved):
    if not data:
        return b''
    table = FixedDictionary(improved)
    current = bytes([data[0]])
    codes = []
    for value in data[1:]:
        extended = current + bytes([value])
        if extended in table.strings:
            current = extended
        else:
            code = table.strings[current]
            codes.append(code)
            if len(table.strings) < 4096:
                table.add(code, value, extended)
            current = bytes([value])
    codes.append(table.strings[current])
    out, bitbox, available = bytearray(), 0, 0
    for code in codes:
        bitbox = (bitbox << 12) | code
        available += 12
        while available >= 8:
            available -= 8
            out.append((bitbox >> available) & 255)
    if available:
        out.append((bitbox << (8 - available)) & 255)
    return bytes(out)


def dynamic_tokens(tokens, method):
    """Author complete 8-code groups independently of either native encoder."""
    width, maximum, next_code, previous = 9, 12 if method == 8 else 13, 257, False
    output = bytearray(b'\x0c' if method == 8 else b'')
    pending = []

    def flush(full):
        bits = [((code >> shift) & 1) for code in pending for shift in range(width)]
        bits += [0] * ((width * 8 if full else (len(bits) + 7) // 8 * 8) - len(bits))
        output.extend(sum(bit << shift for shift, bit in enumerate(bits[at:at + 8]))
                      for at in range(0, len(bits), 8))
        pending.clear()

    for code in tokens:
        if next_code == 1 << width and width < maximum:
            if pending:
                flush(True)
            width += 1
        pending.append(code)
        if len(pending) == 8:
            flush(True)
        if code == 256:
            if pending:
                flush(True)
            width, next_code, previous = 9, 257, False
        elif previous:
            next_code = min(next_code + 1, 1 << maximum)
        else:
            previous = True
    if pending:
        flush(False)
    return bytes(output)


def frame(method, packed, decoded):
    header = b'\x1a' + bytes([method]) + b'HELLO.BIN\0\0\0\0'
    header += struct.pack('<IHHH', len(packed), 0x576e, 0xb1aa, crc16(decoded))
    if method != 1:
        header += struct.pack('<I', len(decoded))
    return header + packed + b'\x1a\0'


def run(args, **kwargs):
    done = subprocess.run([str(x) for x in args], input=b'', capture_output=True,
                          timeout=30, creationflags=0x08000000 if os.name == 'nt' else 0, **kwargs)
    if done.returncode:
        raise RuntimeError(f'Reference failed {done.returncode}: {done.stdout!r} {done.stderr!r}')
    return done.stdout


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--reference', type=Path, required=True)
    parser.add_argument('--nomarch', type=Path, required=True)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    reference, nomarch = args.reference.resolve(), args.nomarch.resolve()
    assert hashlib.sha256(reference.read_bytes()).hexdigest() == REFERENCE_SHA
    assert hashlib.sha256(nomarch.read_bytes()).hexdigest() == NOMARCH_SHA
    base = ROOT / 'build/review/format-expansion'
    work = Path(tempfile.mkdtemp(prefix='arc-fixtures-', dir=base)).resolve()
    destination = ROOT / 'tests/fixtures/arc'
    if not args.check:
        destination.mkdir(exist_ok=True)
    cases, hashes = [], []
    kinds = ['empty', 'phrase', 'allbytes', 'runs', 'escaped', 'random']
    kinds += [f'clear-{count}' for count in range(1, 9)]
    for kind in kinds:
        data = plain(kind)
        source = work / 'INPUT.BIN'
        source.write_bytes(data)
        for requested in range(1, 10):
            if kind.startswith('clear-') and requested not in [8, 9]:
                continue
            if requested == 7 and not data:
                continue
            if kind.startswith('clear-'):
                count = int(kind[6:])
                packed = dynamic_tokens([*data[:count], 256, *data[count:]], requested)
                method, origin = requested, 'wire-clear'
            elif requested in [8, 9]:
                raw = work / 'PACKED.RAW'
                force = kind == 'random'
                meta = json.loads(run([reference, 'lzw' if force else 'pack', source, raw, requested]))
                method = meta['method']
                packed = raw.read_bytes()[:meta['packed']]
                assert meta['decoded'] == len(data) and meta['crc'] == crc16(data)
                origin = 'native-forced-lzw' if force else 'native'
            else:
                method, origin = requested, 'wire'
                packed = data if method <= 2 else rle(data)
                if method == 4:
                    packed = squeeze(packed)
                if method in [5, 6, 7]:
                    packed = fixed_lzw(data if method == 5 else rle(data), method == 7)
            name = f'm{requested}-{kind}.arc'
            archive = frame(method, packed, data)
            raw = work / 'VERIFY.RAW'
            raw.write_bytes(packed)
            native_out = work / 'NATIVE.BIN'
            # The old native crunch reader cannot represent an empty first code.
            if method not in [5, 6, 7] or data:
                run([reference, 'unpack', raw, native_out, method, len(packed), len(data), crc16(data)])
                assert native_out.read_bytes() == data, name
            own = work / name
            own.write_bytes(archive)
            target = work / (name + '-nomarch')
            target.mkdir()
            if method != 7:
                run([nomarch, '-U', own], cwd=target)
                output = target / 'HELLO.BIN'
                assert output.read_bytes() == data, name
            if args.check:
                assert (destination / name).read_bytes() == archive, name
            else:
                (destination / name).write_bytes(archive)
            sha = hashlib.sha256(archive).hexdigest()
            hashes.append(f'{name} {sha}\n')
            cases.append(dict(name=name, kind=kind, requested=requested, method=method,
                              origin=origin, packed=len(packed), decoded=len(data),
                              crc=crc16(data), sha256=sha,
                              native_verified=method not in [5, 6, 7] or bool(data),
                              nomarch_verified=method != 7))
    report = dict(source_revision='70f0b93516d0ee222db5cf1241aad0db1b721c0d',
                  native_sha256=REFERENCE_SHA, nomarch_sha256=NOMARCH_SHA,
                  input_set_sha256=hashlib.sha256(''.join(hashes).encode()).hexdigest(), cases=cases)
    assert report['input_set_sha256'] == INPUT_SHA
    (base / 'm8-arc-reference.json').write_text(json.dumps(report, indent=2) + '\n')
    fields = ['name: &\'static str', 'kind: &\'static str', 'requested: u8', 'method: u8',
              'packed: usize', 'decoded: usize', 'crc: u16']
    rust = '// Generated by scripts/generate-arc-fixtures.py; owned wire/reference inputs.\n'
    rust += '#[derive(Clone, Copy)]\npub(crate) struct Case {\n'
    rust += ''.join(f'    pub {field},\n' for field in fields) + '}\n'
    rust += f'#[rustfmt::skip]\npub(crate) const CASES: [Case; {len(cases)}] = [\n'
    for case in cases:
        rust += '    Case { ' + ', '.join(f'{key}: {json.dumps(case[key])}'
                    for key in ['name', 'kind', 'requested', 'method', 'packed', 'decoded', 'crc']) + ' },\n'
    rust += '];\n'
    generated = ROOT / 'crates/xdzip-compression/src/arc/fixture_cases.rs'
    if args.check:
        assert generated.read_bytes() == rust.encode()
    else:
        generated.parent.mkdir(exist_ok=True)
        generated.write_text(rust, encoding='utf-8', newline='\n')
    print(f"Verified {len(cases)} owned ARC inputs, set SHA-256 {report['input_set_sha256']}")


if __name__ == '__main__':
    main()
