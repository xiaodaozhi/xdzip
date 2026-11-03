#!/usr/bin/env python3
"""Verify the original and replacement images against independently obtained archives."""
import argparse
import hashlib
import json
from pathlib import Path
import tarfile

from cldr_webapp_notices import (ARCHIVE_ROOT, DIRECTORY, EMOJI_DIRECTORY, REPLACEMENT_UPSTREAMS,
                                ROOT, emoji_replacements)


def digest(path):
    value = hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b''):
            value.update(chunk)
    return value.hexdigest()


def verify_archive(path, expected_archive, expected_files):
    """Check a fixed archive and listed regular members without extracting paths."""
    if digest(path) != expected_archive:
        raise ValueError('Upstream archive checksum mismatch')
    seen = set()
    with tarfile.open(path, 'r:gz') as archive:
        for member in archive:
            if member.name not in expected_files:
                continue
            if member.name in seen or not member.isfile() or member.size > 1024 * 1024:
                raise ValueError('Unexpected or duplicated upstream member')
            data = archive.extractfile(member).read()
            if hashlib.sha256(data).hexdigest() != expected_files[member.name]:
                raise ValueError('Upstream member checksum mismatch')
            seen.add(member.name)
    if seen != set(expected_files):
        raise ValueError('Missing upstream members: '+str(len(set(expected_files)-seen)))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cldr', type=Path, required=True)
    parser.add_argument('--noto', type=Path, required=True)
    parser.add_argument('--twemoji', type=Path, required=True)
    args = parser.parse_args()
    replacements = emoji_replacements()
    manifest = json.loads((ROOT/'scripts/native-runtime-sources.json').read_bytes())
    cldr, = (item for item in manifest['icu_data_sources'] if item['module'] == 'cldr-data-source')
    if cldr['archive_root'] != ARCHIVE_ROOT:
        raise ValueError('Unexpected CLDR archive identity')
    original = {ARCHIVE_ROOT+'/'+EMOJI_DIRECTORY+'/'+name: item['original_sha256']
                for name, item in replacements.items()}
    sources = {}
    for source in REPLACEMENT_UPSTREAMS:
        commit = REPLACEMENT_UPSTREAMS[source]['commit']
        prefix = ('googlefonts-noto-emoji-'+commit[:7] if source == 'noto' else 'twemoji-'+commit)+'/'
        records = [item for item in replacements.values() if item['source'] == source]
        members = {prefix+item['upstream_path']: item['sha256'] for item in records}
        if len(members) != len(records):
            raise ValueError('Replacement upstream paths must be unique')
        sources[source] = members
    notice = ROOT/'third_party'/DIRECTORY
    sources['noto']['googlefonts-noto-emoji-'+REPLACEMENT_UPSTREAMS['noto']['commit'][:7]+'/LICENSE'] = digest(notice/'Noto-Emoji-Apache-2.0.txt')
    sources['twemoji']['twemoji-'+REPLACEMENT_UPSTREAMS['twemoji']['commit']+'/LICENSE-GRAPHICS'] = digest(notice/'Twemoji-CC-BY-4.0.txt')
    sources['twemoji']['twemoji-'+REPLACEMENT_UPSTREAMS['twemoji']['commit']+'/README.md'] = '819f64b0e6fcfe15b7979e9aec42d1e2801dda35ead50c14566f4051e0dc37ef'
    verify_archive(args.cldr, cldr['sha256'], original)
    for source in REPLACEMENT_UPSTREAMS:
        verify_archive(getattr(args, source), REPLACEMENT_UPSTREAMS[source]['archive_sha256'], sources[source])
    print(json.dumps({'original_images_verified': len(original),
                      'noto_images_verified': len(sources['noto'])-1,
                      'twemoji_images_verified': len(sources['twemoji'])-2,
                      'limits': 'Source byte comparison; does not establish the original images provenance.'}, indent=2))


if __name__ == '__main__':
    main()
