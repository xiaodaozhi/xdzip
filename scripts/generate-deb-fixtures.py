#!/usr/bin/env python3
"""Create owned DEB envelopes, verified by an independent common-ar reader."""
import argparse
import bz2
import compression.zstd as zstd
import gzip
import hashlib
import io
import lzma
from pathlib import Path
import subprocess
import tarfile
import tempfile

ROOT = Path(__file__).resolve().parents[1]
DESTINATION = ROOT / 'tests/fixtures/deb'
CONTROL = (b'Package: xdzip-owned-fixture\nVersion: 1.0\nArchitecture: all\n'
           b'Maintainer: Owned test <owned@example.invalid>\n'
           b'Description: Owned container fixture, never installed\n')
HELLO = b'abracadabra abracadabra\n' * 6


def tar_bytes(control, duplicate=False):
    output = io.BytesIO()
    contents = [('.', None)]
    if control:
        contents += [('control', CONTROL), ('postinst', b'#!/bin/sh\necho owned-test-only\n')]
    else:
        contents += [('usr/share/xdzip', None), ('usr/share/xdzip/hello.txt', HELLO),
                     ('usr/share/xdzip/资料.txt', b'owned UTF-8 content\n'),
                     ('usr/share/xdzip/empty.bin', b'')]
        if duplicate:
            contents.append(('usr/share/xdzip/hello.txt', b'second physical record\n'))
    with tarfile.open(fileobj=output, mode='w', format=tarfile.USTAR_FORMAT) as archive:
        for name, content in contents:
            item = tarfile.TarInfo(name)
            item.mtime = 1700000000
            item.uid = item.gid = 0
            item.mode = 0o755 if content is None or name == 'postinst' else 0o644
            if content is None:
                item.type = tarfile.DIRTYPE
                archive.addfile(item)
            else:
                item.size = len(content)
                archive.addfile(item, io.BytesIO(content))
    return output.getvalue()


def compressed(data, kind):
    if kind == 'none':
        return data
    if kind == 'gz':
        return gzip.compress(data, mtime=0)
    if kind == 'xz':
        return lzma.compress(data, format=lzma.FORMAT_XZ, check=lzma.CHECK_CRC64)
    if kind == 'zst':
        return zstd.compress(data, options={zstd.CompressionParameter.checksum_flag: 1})
    if kind == 'bz2':
        return bz2.compress(data)
    if kind == 'lzma':
        return lzma.compress(data, format=lzma.FORMAT_ALONE)
    raise ValueError(kind)


def decoded(data, kind):
    return {'none': lambda data: data, 'gz': gzip.decompress, 'xz': lzma.decompress,
            'zst': zstd.decompress, 'bz2': bz2.decompress, 'lzma': lzma.decompress}[kind](data)


def member(name, data, slash):
    name = name.encode('ascii') + (b'/' if slash else b'')
    assert 0 < len(name) <= 16
    header = name.ljust(16, b' ') + b'1700000000'.ljust(12, b' ')
    header += b'0'.ljust(6, b' ') * 2 + b'100644'.ljust(8, b' ')
    header += str(len(data)).encode('ascii').ljust(10, b' ') + b'`\n'
    assert len(header) == 60
    return header + data + (b'\n' if len(data) % 2 else b'')


def package(control_kind, data_kind, *, duplicate=False, extensions=False, slash=False):
    control = tar_bytes(True)
    data = tar_bytes(False, duplicate)
    control_name = 'control.tar' + ('' if control_kind == 'none' else '.' + control_kind)
    data_name = 'data.tar' + ('' if data_kind == 'none' else '.' + data_kind)
    members = [('debian-binary', b'2.7\nowned-extension-line\n' if extensions else b'2.0\n')]
    if extensions:
        members.append(('_buildid', b'odd'))
    members.append((control_name, compressed(control, control_kind)))
    if extensions:
        members.append(('_between', b'x'))
    members.append((data_name, compressed(data, data_kind)))
    if extensions:
        members.append(('after-data', b'owned additional member\n'))
    assert decoded(members[2 if extensions else 1][1], control_kind) == control
    data_index = 4 if extensions else 2
    assert decoded(members[data_index][1], data_kind) == data
    return b'!<arch>\n' + b''.join(member(name, payload, slash) for name, payload in members), members


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--ar-reference', type=Path, required=True)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    reference = args.ar_reference.resolve()
    expected = 'a3ac62fd4cdf619a7c53f71ca3fe5fb42f72020975670ff29afe3603f6416a8e'
    if hashlib.sha256(reference.read_bytes()).hexdigest() != expected:
        parser.error('Use the fixed bsdtar 3.8.9 test reference')
    if zstd.zstd_version_info != (1, 5, 7):
        parser.error('Use the fixed independent Zstandard 1.5.7 binding')
    version = subprocess.check_output([str(reference), '--version'], text=True)
    assert version.startswith('bsdtar 3.8.9 - libarchive 3.8.9 ')
    fixtures = {}
    for control_kind in ['none', 'gz', 'xz', 'zst']:
        for data_kind in ['none', 'gz', 'xz', 'zst', 'bz2', 'lzma']:
            fixtures[f'control-{control_kind}-data-{data_kind}.deb'] = package(control_kind, data_kind)
    fixtures['extensions.deb'] = package('zst', 'xz', extensions=True, slash=True)
    fixtures['duplicates.deb'] = package('gz', 'zst', duplicate=True)
    (ROOT / 'build/test-tmp').mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='deb-reference-', dir=ROOT / 'build/test-tmp') as directory:
        archive = Path(directory) / 'owned.deb'
        for name, (data, members) in fixtures.items():
            archive.write_bytes(data)
            listed = subprocess.check_output([str(reference), '-tf', str(archive)]).decode('ascii').splitlines()
            assert listed == [name for name, _ in members], (name, listed)
            for member_name, payload in members:
                extracted = subprocess.check_output([str(reference), '-xOf', str(archive), member_name])
                assert extracted == payload, (name, member_name)
            path = DESTINATION / name
            if args.check:
                if not path.is_file() or path.read_bytes() != data:
                    raise SystemExit('Stale owned fixture: ' + name)
            else:
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(data)
    print(f'{len(fixtures)} owned DEB envelopes ' + ('verified' if args.check else 'generated'))


if __name__ == '__main__':
    main()
