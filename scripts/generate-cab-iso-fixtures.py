# scripts/generate-cab-iso-fixtures.py

"""Regenerate owned CAB/ISO/UDF inputs with pycdlib 1.14.0 (test tooling only)."""
import argparse
import gzip
import hashlib
import io
import importlib.metadata
import json
from pathlib import Path
import struct

ROOT = Path(__file__).resolve().parents[1]
HELLO = b'xdzip read-only archive fixture\n'
NESTED = 'Unicode payload: \u4e2d\u6587\n'.encode('utf-8')


def cabinet():
    """One UTF-8, uncompressed folder, written independently of the product."""
    items = [('hello.txt', HELLO), ('nested\\\u4e2d\u6587.txt', NESTED), ('empty.txt', b'')]
    records, content = bytearray(), bytearray()
    for name, data in items:
        records += struct.pack('<IIHHHH', len(data), len(content), 0, 0x5821, 0, 0xA0)
        records += name.encode('utf-8') + b'\0'
        content += data
    data_offset = 36 + 8 + len(records)
    block = struct.pack('<IHH', 0, len(content), len(content)) + content
    header = b'MSCF' + struct.pack('<IIIII', 0, data_offset + len(block), 0, 44, 0)
    header += struct.pack('<BBHHHHH', 3, 1, 1, len(items), 0, 0x5844, 0)
    return header + struct.pack('<IHH', data_offset, 1, 0) + records + block


def image(kind):
    import pycdlib
    iso = pycdlib.PyCdlib()
    options = dict(interchange_level=3, vol_ident='XDZIP_FIXTURE')
    if kind == 'joliet':
        options['joliet'] = 3
    elif kind == 'rock-ridge':
        options['rock_ridge'] = '1.09'
    elif kind in ('udf', 'udf-regular', 'hybrid'):
        options.update(udf='2.60', joliet=3)
    iso.new(**options)
    joliet, rr, udf = 'joliet' in options, 'rock_ridge' in options, 'udf' in options
    def paths(iso_path, normal_path):
        values = dict(iso_path=iso_path)
        if joliet:
            values['joliet_path'] = normal_path
        if rr:
            values['rr_name'] = normal_path.rsplit('/', 1)[-1]
        if udf:
            values['udf_path'] = normal_path
        return values
    iso.add_directory(**paths('/NESTED', '/nested'))
    iso.add_directory(**paths('/EMPTY', '/empty'))
    files = [('/HELLO.TXT;1', '/hello.txt', HELLO),
             ('/NESTED/DATA.TXT;1', '/nested/data.txt', NESTED),
             ('/ZERO.TXT;1', '/zero.txt', b'')]
    if joliet or rr:
        files.append(('/UNICODE.TXT;1', '/\u4e2d\u6587.txt', NESTED))
    for original, normal, content in files:
        iso.add_fp(io.BytesIO(content), len(content), **paths(original, normal))
    if rr:
        iso.add_symlink(symlink_path='/LINK.;1', rr_symlink_name='link', rr_path='hello.txt')
    if udf:
        # Deliberately different namespaces prove that routing selects UDF.
        iso.add_fp(io.BytesIO(HELLO), len(HELLO), udf_path='/udf-only.txt')
        if kind != 'udf-regular':
            iso.add_symlink(udf_symlink_path='/udf-link', udf_target='/hello.txt')
    output = io.BytesIO()
    iso.write_fp(output)
    iso.close()
    result = bytearray(output.getvalue())
    if kind == 'udf':
        # Leave UDF VRS and anchors intact, remove the ISO namespace descriptors.
        for sector in range(16, 32):
            at = sector * 2048
            if result[at + 1:at + 6] == b'CD001':
                result[at:at + 2048] = b'\0' * 2048
    return bytes(result)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, default=ROOT / 'tests/fixtures/cab-iso')
    args = parser.parse_args()
    if importlib.metadata.version('pycdlib') != '1.14.0':
        raise ValueError('Fixture regeneration requires pycdlib 1.14.0')
    args.output.mkdir(parents=True, exist_ok=True)
    manifest_path = args.output / 'sources.json'
    manifest = json.loads(manifest_path.read_text()) if manifest_path.exists() else {'fixtures': []}
    manifest['fixtures'] = [item for item in manifest['fixtures'] if not item.get('generator')]
    for name in ('named.cab', 'iso9660', 'joliet', 'rock-ridge', 'udf', 'udf-regular', 'hybrid'):
        data = cabinet() if name.endswith('.cab') else image(name)
        filename = name if name.endswith('.cab') else name + '.iso.gz'
        stored = data if name.endswith('.cab') else gzip.compress(data, mtime=0)
        (args.output / filename).write_bytes(stored)
        manifest['fixtures'].append(dict(file=filename, sha256=hashlib.sha256(stored).hexdigest(),
                                         decoded_sha256=hashlib.sha256(data).hexdigest(),
                                         decoded_bytes=len(data), license='Apache-2.0',
                                         generator='scripts/generate-cab-iso-fixtures.py',
                                         tool='Python struct' if name.endswith('.cab') else 'pycdlib 1.14.0'))
    manifest_path.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print('Generated owned CAB, ISO9660, Joliet, Rock Ridge, UDF and hybrid test inputs')


if __name__ == '__main__':
    main()
