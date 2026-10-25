# scripts/prepare-cab-iso-fixtures.py

"""Verify and expand fixed, small CAB/disc-image test inputs without dependencies."""
import argparse
import gzip
import hashlib
import io
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BASE = ROOT / 'tests/fixtures/cab-iso'
MAX_IMAGE = 8 * 1024**2


def prepare(output):
    output = Path(output).resolve()
    if not output.is_relative_to(ROOT / 'build') or output == ROOT / 'build':
        raise ValueError('Fixture outputs belong below build/')
    manifest = json.loads((BASE / 'sources.json').read_text(encoding='utf-8'))
    output.mkdir(parents=True, exist_ok=True)
    for record in manifest['fixtures']:
        name = record['file']
        if Path(name).name != name or '/' in name or '\\' in name:
            raise ValueError('Unexpected fixture name')
        stored = (BASE / name).read_bytes()
        if len(stored) > MAX_IMAGE or hashlib.sha256(stored).hexdigest() != record['sha256']:
            raise ValueError('Fixture identity mismatch: ' + name)
        if name.endswith('.gz'):
            with gzip.GzipFile(fileobj=io.BytesIO(stored)) as source:
                data = source.read(MAX_IMAGE + 1)
            if len(data) > MAX_IMAGE or len(data) != record['decoded_bytes']:
                raise ValueError('Fixture size mismatch: ' + name)
            if hashlib.sha256(data).hexdigest() != record['decoded_sha256']:
                raise ValueError('Decoded fixture identity mismatch: ' + name)
            name = name[:-3]
        else:
            data = stored
        (output / name).write_bytes(data)
    return output


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    prepare(args.output)
    print('Prepared verified CAB and disc-image test fixtures')
