#!/usr/bin/env python3
"""Verify the actual private image worker, including its installed plugin closure."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import struct
import subprocess
import zlib

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--worker', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    if not output.is_relative_to(ROOT/'build'):
        parser.error('Keep verification records under build/')
    env = os.environ.copy()
    env['QT_QPA_PLATFORM'] = 'offscreen'
    worker = args.worker.resolve()
    result = subprocess.run([str(worker), '--formats'], env=env, capture_output=True, timeout=7, check=True)
    formats = json.loads(result.stdout)
    expected = ['png', 'jpeg', 'gif', 'bmp', 'ico', 'webp', 'tiff', 'tga', 'icns', 'svg']
    if platform.system() == 'Darwin':
        expected.append('heic')
    if formats != expected:
        raise ValueError('Required image plugins are unavailable: ' + ', '.join(set(expected)-set(formats)))

    def chunk(kind, data):
        return struct.pack('>I', len(data)) + kind + data + struct.pack('>I', zlib.crc32(kind+data))
    png = (b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', 1, 1, 8, 6, 0, 0, 0))
           + chunk(b'IDAT', zlib.compress(b'\0\x12\x34\x56\xff')) + chunk(b'IEND', b''))
    def decode(kind, source, pixel):
        request = b'XIP1' + struct.pack('<II', kind, len(source)) + source
        result = subprocess.run([str(worker)], input=request, env=env, capture_output=True, timeout=7, check=True)
        assert result.stdout[:4] == b'XIR1' and len(result.stdout) >= 16
        width, height, length = struct.unpack('<III', result.stdout[4:16])
        assert 0 < width <= 512 and 0 < height <= 512 and length == width*height*4
        assert result.stdout[16:] == pixel * (width*height)
        return [width, height]

    # A one-pixel, single-strip grayscale TIFF. It uses the packaged TIFF decoder.
    identity = [(256, 4, 1), (257, 4, 1), (258, 3, 8), (259, 3, 1), (262, 3, 1),
                (273, 4, 134), (277, 3, 1), (278, 4, 1), (279, 4, 1), (284, 3, 1)]
    tiff = (b'II*\0\x08\0\0\0' + struct.pack('<H', len(identity)) +
            b''.join(struct.pack('<HHII', tag, field_type, 1, value) for tag, field_type, value in identity) +
            b'\0\0\0\0\x40')
    record = {'schema': 1, 'status': 'passed', 'formats': formats,
              'native_png_dimensions': decode(2, png, b'\x12\x34\x56\xff'),
              'native_tiff_dimensions': decode(8, tiff, b'\x40\x40\x40\xff'),
              'worker_sha256': hashlib.sha256(worker.read_bytes()).hexdigest(),
              'scope': 'Actual executable/plugin availability and owned PNG and TIFF.'}
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(record, indent=2)+'\n')
    print(json.dumps(record))


if __name__ == '__main__':
    main()
