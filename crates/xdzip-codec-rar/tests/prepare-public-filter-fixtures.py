"""Prepare fixed public RAR ARM input and independently published original data.

Binary inputs remain under build/ and are never executed. The independent
original is GPL-3.0-or-later data; its exact upstream notice is preserved.
No RAR decompressor or compressor is implemented by this preparer.
"""
from pathlib import Path
import argparse
import binascii
import hashlib
import json
import urllib.request

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = Path(__file__).resolve().parent / 'fixtures/filter-author/arm-sources.json'

def fetch(url, cap):
    with urllib.request.urlopen(url, timeout=30) as response:
        value = response.read(cap + 1)
        if len(value) > cap:
            raise ValueError('download resource policy')
        return value

def sha(value):
    return hashlib.sha256(value).hexdigest()

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    directory = args.output.resolve()
    if not directory.is_relative_to((ROOT / 'build').resolve()):
        raise ValueError('output must be under workspace build/')
    directory.mkdir(parents=True, exist_ok=True)
    if any(directory.iterdir()):
        raise ValueError('preserve previous input/evidence; use an empty directory')
    metadata = json.loads(MANIFEST.read_text(encoding='utf-8-sig'))
    archive_meta = metadata['archive']
    text = fetch(archive_meta['url'], 128 * 1024)
    if sha(text) != archive_meta['uu_sha256']:
        raise ValueError('fixed encoded archive identity')
    lines = iter(text.splitlines())
    for line in lines:
        if line.startswith(b'begin '):
            break
    else:
        raise ValueError('uu header')
    archive = bytearray()
    for line in lines:
        if line == b'end':
            break
        archive.extend(binascii.a2b_uu(line))
        if len(archive) > 64 * 1024:
            raise ValueError('decoded archive resource policy')
    else:
        raise ValueError('uu terminator')
    if sha(archive) != archive_meta['sha256'] or len(archive) != archive_meta['bytes']:
        raise ValueError('fixed archive identity')
    original_meta = metadata['independent_original']
    original = fetch(original_meta['url'], 128 * 1024)
    notice = fetch(original_meta['license_url'], 16 * 1024)
    if sha(original) != original_meta['sha256'] or len(original) != original_meta['bytes']:
        raise ValueError('independent original identity')
    if sha(notice) != original_meta['license_sha256']:
        raise ValueError('fixed original license identity')
    (directory / 'arm.rar').write_bytes(archive)
    (directory / 'arm.original').write_bytes(original)
    (directory / 'arm.original.license').write_bytes(notice)
    (directory / 'sources.json').write_text(json.dumps(metadata, indent=2) + '\n', encoding='utf-8')
    (directory / 'cases.tsv').write_text('\t'.join([
        'upstream-arm-independent-original', '50', 'arm.rar', archive_meta['sha256'],
        'arm.original', original_meta['sha256'], '3']) + '\n', encoding='utf-8')

if __name__ == '__main__':
    main()
