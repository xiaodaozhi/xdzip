#!/usr/bin/env python3
"""Generate owned ASAR inputs through a digest-verified independent author tool."""
import argparse
import base64
import hashlib
import json
from pathlib import Path
import subprocess
import tarfile
import tempfile

ROOT = Path(__file__).resolve().parents[1]
DESTINATION = ROOT / 'tests/fixtures/asar'
INTEGRITY = '2nwyO3OgmLgkYQe6TzB8FmyrBKGgmXzSO51ozoG2BpHOLPTPdQpBt/zD0MdPkhQr4UqVgw3u09y4uYxgZKhmtg=='


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--reference', type=Path, required=True)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    reference = args.reference.resolve()
    if not reference.is_relative_to(ROOT / 'build/toolchain'):
        parser.error('Keep the reference under build/toolchain/')
    archive = reference / 'electron-asar-4.3.2.tgz'
    if base64.b64encode(hashlib.sha512(archive.read_bytes()).digest()).decode() != INTEGRITY:
        raise ValueError('ASAR reference package integrity mismatch')
    package = reference / 'node_modules/@electron/asar'
    with tarfile.open(archive) as source:
        for entry in source.getmembers():
            if not entry.isfile():
                continue
            relative = Path(entry.name).relative_to('package')
            if '..' in relative.parts or (package / relative).is_symlink():
                raise ValueError('Unsafe reference package path')
            if (package / relative).read_bytes() != source.extractfile(entry).read():
                raise ValueError('Installed reference differs from the pinned package')
    if json.loads((package / 'package.json').read_text())['version'] != '4.3.2':
        raise ValueError('Unexpected ASAR reference version')
    scratch = ROOT / 'build/review/format-expansion'
    scratch.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='asar-reference-', dir=scratch) as temporary:
        result = subprocess.run(['node', str(DESTINATION / 'reference.mjs'), str(reference), temporary],
                                capture_output=True, text=True, encoding='utf-8', check=True)
        report = json.loads(result.stdout)
        inputs = sorted(Path(temporary).glob('*.asar'))
        if len(inputs) != report['cases'] or len(inputs) != 7:
            raise ValueError('Incomplete independent ASAR inputs')
        for source in inputs:
            target = DESTINATION / source.name
            if args.check:
                if not target.is_file() or source.read_bytes() != target.read_bytes():
                    raise ValueError('Stale owned ASAR input: ' + source.name)
            else:
                target.write_bytes(source.read_bytes())
        (scratch / 'm7-asar-reference.json').write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print('7 owned ASAR inputs independently verified')


if __name__ == '__main__':
    main()
