"""Preserve bounded RAR mapping provenance, without importing an official engine."""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DIRECTORY = 'rar-wire-mappings'


def collect(output):
    source = ROOT / 'third_party' / DIRECTORY
    names = ('README.md', 'sources.json', 'UnRAR-7.3.1-license.txt')
    data = {}
    for name in names:
        path = source / name
        if path.is_symlink() or not path.is_file():
            raise ValueError('RAR mapping notice input must be a regular file')
        data[name] = path.read_bytes()
    config = json.loads(data['sources.json'])
    notice = config['original_notice']
    if (config['schema_version'] != 1 or config['kind'] != 'limited-reference-provenance' or
            config['official_engine_in_product'] is not False or notice['file'] != names[2] or
            len(data[names[2]]) != notice['bytes'] or
            hashlib.sha256(data[names[2]]).hexdigest() != notice['sha256']):
        raise ValueError('RAR mapping original notice checksum or scope mismatch')
    destination = Path(output) / DIRECTORY
    if destination.is_symlink():
        raise ValueError('RAR mapping notice destination must not be a symlink')
    for name in names:
        path = destination / name
        if path.is_symlink() or (path.exists() and (not path.is_file() or path.read_bytes() != data[name])):
            raise ValueError('Existing RAR mapping notice differs from pinned input')
    destination.mkdir(parents=True, exist_ok=True)
    for name in names:
        (destination / name).write_bytes(data[name])
    record = {key: config[key] for key in ('name', 'kind', 'source_release', 'source_license',
                                         'source_archive', 'official_engine_in_product', 'licensing_status')}
    record.update(file=DIRECTORY + '/' + notice['file'], bytes=notice['bytes'], sha256=notice['sha256'])
    return [destination / name for name in names], record
