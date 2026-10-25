"""Generate owned Quantum CAB test streams under build/, without deleting files.

This mathematical test encoder has been independently decoded by the pinned
official worker. It is not a third-party compressor or a product dependency.
"""
import argparse
from pathlib import Path
import hashlib
import json
import struct

ROOT = Path(__file__).resolve().parents[3]
BASE = None


class Model:
    def __init__(self, first, count):
        self.rows = [[s, 1] for s in range(first, first + count)]
        self.turns = 4

    def interval(self, symbol):
        if sum(f for _, f in self.rows) > 3800:
            self.turns -= 1
            if self.turns == 0:
                self.turns = 50
                for row in self.rows:
                    row[1] = (row[1] + 1) // 2
                for i in range(len(self.rows)):
                    for j in range(i + 1, len(self.rows)):
                        if self.rows[i][1] < self.rows[j][1]:
                            self.rows[i], self.rows[j] = self.rows[j], self.rows[i]
            else:
                old, new = 0, 0
                for row in reversed(self.rows):
                    old += row[1]
                    cumulative = max(old // 2, new + 1)
                    row[1] = cumulative - new
                    new = cumulative
        total = sum(f for _, f in self.rows)
        upper = total
        for row in self.rows:
            lower = upper - row[1]
            if row[0] == symbol:
                row[1] += 8
                return lower, upper, total
            upper = lower
        raise ValueError('symbol')


class Arithmetic:
    def __init__(self):
        self.low, self.high = 0, 65535
        self.follow = 0
        self.bits = []
        self.steps = 0
        self.direct = []

    def put(self, bit):
        self.bits.extend([bit] + [1 - bit] * self.follow)
        self.follow = 0

    def symbol(self, model, symbol):
        lower, upper, total = model.interval(symbol)
        width = self.high - self.low + 1
        origin = self.low
        self.low = origin + width * lower // total
        self.high = origin + width * upper // total - 1
        while True:
            if self.high < 32768:
                translation = 0
                self.put(0)
            elif self.low >= 32768:
                translation = 32768
                self.put(1)
            elif self.low >= 16384 and self.high < 49152:
                translation = 16384
                self.follow += 1
            else:
                break
            self.low = (self.low - translation) * 2
            self.high = (self.high - translation) * 2 + 1
            self.steps += 1

    def extra(self, value, count):
        if count:
            self.direct.append((16 + self.steps, [(value >> bit) & 1 for bit in reversed(range(count))]))

    def finish(self):
        self.follow += 1
        self.put(0 if self.low < 16384 else 1)
        self.bits.extend([0] * 16)
        assert len(self.bits) == self.steps + 18
        # Direct bits bypass arithmetic state, after its 16-bit lookahead.
        result = []
        at = 0
        for position, bits in self.direct:
            result.extend(self.bits[at:position])
            result.extend(bits)
            at = position
        result.extend(self.bits[at:])
        result.extend([0] * (-len(result) % 8))
        return bytes(sum(result[i + j] << (7-j) for j in range(8)) for i in range(0, len(result), 8))


class Folder:
    def __init__(self, bits):
        self.bits = bits
        self.models = [Model(0, 7)] + [Model(g * 64, 64) for g in range(4)]
        self.distance = [Model(0, min(2 * bits, limit)) for limit in [24, 36, 42]]
        self.length = Model(5, 27)
        self.plain = bytearray()
        self.frames = []

    def encode(self, tokens):
        a = Arithmetic()
        start = len(self.plain)
        for token in tokens:
            if token[0] == 'literal':
                byte = token[1]
                a.symbol(self.models[0], byte // 64)
                a.symbol(self.models[1 + byte // 64], byte)
                self.plain.append(byte)
                continue
            _, length, distance = token
            selector = 4 if length == 3 else 5 if length == 4 else 6
            a.symbol(self.models[0], selector)
            if selector == 6:
                for v in range(5, 32):
                    if v <= 10:
                        base, extra = v, 0
                    else:
                        q, r = divmod(v - 7, 4)
                        base, extra = (4 + r) * 2**q + 3, q if q < 6 else 0
                    if base <= length < base + 2**extra:
                        a.symbol(self.length, v)
                        length_extra = (length - base, extra)
                        a.extra(*length_extra)
                        break
                else:
                    raise ValueError('length')
            for slot in range(min(2 * self.bits, [24, 36, 42][selector - 4])):
                q = slot // 2 - 1 if slot >= 4 else 0
                base = (2 + slot % 2) * 2**q + 1 if slot >= 4 else slot + 1
                if base <= distance < base + 2**q:
                    a.symbol(self.distance[selector - 4], slot)
                    distance_extra = (distance - base, q)
                    break
            else:
                raise ValueError('distance')
            a.extra(*distance_extra)
            assert 0 < distance <= min(2**self.bits, len(self.plain))
            for _ in range(length):
                self.plain.append(self.plain[-distance])
        packed = a.finish()
        self.frames.append((packed, bytes(self.plain[start:])))

    def save(self, name, level=2):
        directory = BASE / name
        directory.mkdir(exist_ok=True)
        for i, (packed, plain) in enumerate(self.frames):
            (directory / f'{i:03}.frame').write_bytes(packed)
            (directory / f'{i:03}.plain').write_bytes(plain)
        record = struct.pack('<IIHHHH', len(self.plain), 0, 0, 0x5821, 0, 0x20) + b'known.bin\0'
        offset = 44 + len(record)
        payload = b''.join(struct.pack('<IHH', 0, len(p), len(d)) + p for p, d in self.frames)
        header = b'MSCF' + struct.pack('<IIIII', 0, offset + len(payload), 0, 44, 0)
        header += struct.pack('<BBHHHHH', 3, 1, 1, 1, 0, 0x5844, 0)
        cab = header + struct.pack('<IHH', offset, len(self.frames), 2 | level << 4 | self.bits << 8) + record + payload
        (directory / 'input.cab').write_bytes(cab)
        (directory / 'known.bin').write_bytes(self.plain)
        return dict(name=name,window_bits=self.bits,level=level,frames=len(self.frames),
                    bytes=len(self.plain),cab_sha256=hashlib.sha256(cab).hexdigest(),
                    plain_sha256=hashlib.sha256(self.plain).hexdigest())


def main():
    global BASE
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, default=ROOT/'build/quantum-fixtures')
    args = parser.parse_args()
    build = (ROOT/'build').resolve()
    BASE = args.output.resolve()
    if not build.is_relative_to(ROOT) or not BASE.is_relative_to(build) or BASE == build:
        parser.error('output must be a subdirectory of this workspace build/')
    BASE.mkdir(parents=True, exist_ok=True)
    cases = []
    # Full literal alphabet exercises every literal model and rescale ordering.
    f = Folder(18)
    for frame in range(6):
        f.encode([('literal', (i * 73 + frame * 19) % 256) for i in range(32768)])
    cases.append(f.save('literal-rescale'))
    for bits in range(10, 22):
        f = Folder(bits)
        target = max(65536, 2**bits + 1024)
        while len(f.plain) < target:
            count = min(32768, target - len(f.plain))
            tokens = []
            length = 0
            if not f.plain:
                tokens.extend(('literal', byte) for byte in b'ABCD')
                length = 4
            while count - length >= 3:
                n = min(259, count - length)
                distance = min(2**bits, len(f.plain) + length)
                # Distance alphabet for short matches is intentionally smaller.
                if n in (3, 4):
                    distance = min(distance, 4)
                tokens.append(('match', n, distance))
                length += n
            tokens.extend(('literal', 0x51) for _ in range(count - length))
            f.encode(tokens)
        cases.append(f.save(f'window-{bits}', 1 + (bits - 10) % 7))
    f = Folder(10)
    for index in range(2):
        tokens = [('literal', ord('Q'))] if index == 0 else []
        count = 32768 - len(tokens)
        while count >= 3:
            n = min(259, count)
            tokens.append(('match', n, 1 if index == 0 else 1024))
            count -= n
        tokens.extend(('literal', ord('Q')) for _ in range(count))
        f.encode(tokens)
    cases.append(f.save('small-history'))
    f = Folder(21)
    for index in range(64):
        tokens = [('literal', ord('Q'))] if index == 0 else []
        count = 32768 - len(tokens)
        while count >= 3:
            n = min(259, count)
            tokens.append(('match', n, 1))
            count -= n
        tokens.extend(('literal', ord('Q')) for _ in range(count))
        f.encode(tokens)
    edges = [('match', n, 1) for n in range(3, 260)]
    for n, slots in [(3,24), (4,36), (5,42)]:
        for slot in range(slots):
            q = slot // 2 - 1 if slot >= 4 else 0
            low = (2 + slot % 2) * 2**q + 1 if slot >= 4 else slot + 1
            edges.extend([('match',n,low),('match',n,low+2**q-1)])
    tokens, count = [], 0
    for token in edges:
        if count + token[1] > 32768:
            tokens.extend(('literal', ord('Q')) for _ in range(32768-count))
            f.encode(tokens)
            tokens, count = [], 0
        tokens.append(token)
        count += token[1]
    if tokens:
        f.encode(tokens)
    cases.append(f.save('token-boundaries'))
    manifest = json.loads(Path(__file__).with_name('quantum-sources.json').read_text())
    expected = [{key:value for key,value in case.items()
                 if key not in ('rust','independent_official','exact_bytes')}
                for case in manifest['cases']]
    if cases != expected:
        raise ValueError('generated source identity differs from fixed manifest')
    (BASE / 'cases.json').write_text(json.dumps(cases, indent=2)+'\n')
    print(json.dumps(cases))


if __name__ == '__main__':
    main()
