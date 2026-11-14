"""Preserve RFC 7932 normative-data notices without an external Brotli runtime."""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DIRECTORY = 'brotli-rfc7932'
DICTIONARY_SHA = '20e42eb1b511c21806d4d227d07e5dd06877d8ce7b3a817f378f313653f35c70'
LICENSE_SHA = 'bf488ed6ce9501e73723785c431fdf86d0b11de9153f4df49fb645735b715af6'
RFC_SHA = '394abd1879e016f4eac89512c2acaf92f7b2d67feb8ab58b6b118c213be07146'


def collect(output, *, root=ROOT):
    source = Path(root)/'third_party'/DIRECTORY
    data = {}
    for name in ['README.md', 'LICENSE.txt', 'sources.json', 'dictionary.bin']:
        path = source/name
        if path.is_symlink() or not path.is_file():
            raise ValueError('Brotli normative data must be a regular pinned input')
        data[name] = path.read_bytes()
    config = json.loads(data['sources.json'])
    if (config['schema_version'] != 1 or config['kind'] != 'normative-format-data' or
            config['license'] != 'BSD-3-Clause' or config['reference_implementation_in_product'] is not False or
            config['version'] != 'RFC7932' or config['source_sha256'] != RFC_SHA or
            config['dictionary_sha256'] != DICTIONARY_SHA or config['source_url'] != 'https://www.rfc-editor.org/rfc/rfc7932.txt' or
            len(data['dictionary.bin']) != 122784 or hashlib.sha256(data['dictionary.bin']).hexdigest() != DICTIONARY_SHA or
            hashlib.sha256(data['LICENSE.txt']).hexdigest() != LICENSE_SHA):
        raise ValueError('Brotli normative-data identity or license mismatch')
    tables = Path(root)/'crates/xdzip-compression/src/brotli/data.rs'
    if tables.is_symlink() or not tables.is_file() or hashlib.sha256(tables.read_bytes()).hexdigest() != config['tables_sha256']:
        raise ValueError('Brotli generated wire tables differ from pinned material')
    destination = Path(output)/DIRECTORY
    if destination.is_symlink():
        raise ValueError('Brotli notice destination must not be a symlink')
    names = ['README.md', 'LICENSE.txt', 'sources.json']
    for name in names:
        path = destination/name
        if path.is_symlink() or path.exists() and (not path.is_file() or path.read_bytes() != data[name]):
            raise ValueError('Existing Brotli notice differs from its pinned input')
    destination.mkdir(parents=True, exist_ok=True)
    for name in names:
        (destination/name).write_bytes(data[name])
    return [destination/name for name in names], config
