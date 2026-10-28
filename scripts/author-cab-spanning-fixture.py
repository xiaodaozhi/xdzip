# scripts/author-cab-spanning-fixture.py

"""Author a small independent, project-owned spanning CAB fixture on Windows."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def payload():
    value = 0x713A91D5
    result = bytearray()
    for _ in range(196608):
        value = (value * 1664525 + 1013904223) & 0xFFFFFFFF
        result.append(value >> 24)
    return bytes(result)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    build = (ROOT / 'build').resolve()
    if not output.is_relative_to(build) or output == build:
        raise ValueError('Author output belongs below build/')
    if output.exists() and (not output.is_dir() or any(output.iterdir())):
        raise ValueError('Author output must be empty')
    if any(character in str(output) for character in '\r\n"'):
        raise ValueError('Output path cannot be represented in a directive')
    output.mkdir(parents=True, exist_ok=True)
    data = payload()
    (output / 'payload.bin').write_bytes(data)
    lines = [
        '.OPTION EXPLICIT',
        '.Set Cabinet=on',
        '.Set Compress=on',
        '.Set CompressionType=MSZIP',
        '.Set MaxDiskSize=65536',
        '.Set MaxCabinetSize=65536',
        '.Set CabinetNameTemplate=owned-part*.cab',
        f'.Set DiskDirectoryTemplate="{output}"',
        f'.Set InfFileName="{output / "author.inf"}"',
        f'.Set RptFileName="{output / "author.rpt"}"',
        f'"{output / "payload.bin"}" "payload.bin"',
    ]
    directive = output / 'author.ddf'
    directive.write_text('\n'.join(lines) + '\n', encoding='utf-8')
    tool = Path(os.environ['SystemRoot']) / 'System32/makecab.exe'
    result = subprocess.run([str(tool), '/F', str(directive)], cwd=output,
                            capture_output=True, timeout=30)
    (output / 'author.log').write_bytes(result.stdout + result.stderr)
    if result.returncode:
        raise ValueError('MakeCAB failed; see author.log')
    parts = sorted(output.glob('owned-part*.cab'))
    if len(parts) != 4:
        raise ValueError('Expected four cabinets, got ' + str(len(parts)))
    digest = lambda value: hashlib.sha256(value).hexdigest()
    manifest = {
        'tool_sha256': digest(tool.read_bytes()),
        'payload_sha256': digest(data),
        'volumes': [{'file': part.name, 'bytes': part.stat().st_size,
                     'sha256': digest(part.read_bytes())} for part in parts],
    }
    (output / 'author-manifest.json').write_text(
        json.dumps(manifest, indent=2) + '\n', encoding='utf-8')
    print('Authored four cabinets containing one 196608-byte owned file')


if __name__ == '__main__':
    main()
