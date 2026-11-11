#!/usr/bin/env python3
"""Generate an owned ZIP64 in bounded memory and probe the actual runtime index."""
import argparse
import hashlib
import json
from pathlib import Path
import struct
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def generate(path, count):
    def name(index):
        # Include CJK, repeated names and nested paths without changing record identity.
        return (f'目录/item-{index:06}.txt' if index % 100 == 0 else
                'duplicate.txt' if index % 100 == 1 else
                f'long-{"界" * 64}-item-{index:06}.txt' if index % 100 == 2 else
                f'item-{index:06}.txt').encode()
    with path.open('xb') as stream:
        for index in range(count):
            text = name(index)
            stream.write(struct.pack('<I5H3I2H', 0x04034B50, 20, 2048, 0, 0, 0, 0, 0, 0, len(text), 0))
            stream.write(text)
        central_at = stream.tell()
        local_at = 0
        for index in range(count):
            text = name(index)
            stream.write(struct.pack('<I6H3I5H2I', 0x02014B50, 20, 20, 2048, 0, 0, 0,
                                     0, 0, 0, len(text), 0, 0, 0, 0, 0, local_at))
            stream.write(text)
            local_at += 30 + len(text)
        end_at = stream.tell()
        stream.write(struct.pack('<IQ2H2I4Q', 0x06064B50, 44, 45, 45, 0, 0,
                                 count, count, end_at - central_at, central_at))
        stream.write(struct.pack('<IIQI', 0x07064B50, 0, end_at, 1))
        stream.write(struct.pack('<I4H2IH', 0x06054B50, 0, 0, 65535, 65535, 0xFFFFFFFF, 0xFFFFFFFF, 0))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output-dir', default='build/issue265-index')
    parser.add_argument('--count', type=int, default=1_000_000)
    parser.add_argument('--probe', default='target/release/xdzip-m2-probe')
    args = parser.parse_args()
    output = (ROOT / args.output_dir).resolve()
    if not output.is_relative_to(ROOT / 'build') or not 1 <= args.count <= 1_000_001:
        parser.error('Use a dedicated build/ output and at most 1,000,001 owned records')
    output.mkdir(parents=True, exist_ok=True)
    path = output / f'owned-{args.count}.zip'
    if path.exists():
        parser.error('Fixture already exists; choose a fresh output directory')
    generate(path, args.count)
    digest = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            digest.update(block)
    probe = (ROOT / args.probe).resolve()
    if not probe.is_file():
        parser.error('Build the actual m2-probe before running')
    result = subprocess.run([str(probe), str(path)], cwd=ROOT, capture_output=True, text=True)
    record = {'fixture': str(path.relative_to(ROOT)), 'records': args.count, 'bytes': path.stat().st_size,
              'sha256': digest.hexdigest(), 'exit_code': result.returncode}
    if result.returncode == 0:
        record['runtime'] = json.loads(result.stdout)
    else:
        record['error'] = result.stderr[-2048:].replace(str(ROOT), '<workspace>')
    (output / 'result.json').write_text(json.dumps(record, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps(record, ensure_ascii=False))
    if result.returncode:
        raise SystemExit(result.returncode)


if __name__ == '__main__':
    main()
