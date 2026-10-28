#!/usr/bin/env python3
"""Prepare bounded archive inputs for isolated native association checks."""
import argparse
import gzip
import io
from pathlib import Path
import shutil
import tarfile

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--zip', required=True, type=Path)
    args = parser.parse_args()
    output = args.output.resolve()
    if not output.is_relative_to(ROOT / 'build') or output == ROOT / 'build' or output.exists():
        parser.error('Use a new child directory under build/')
    output.mkdir(parents=True)
    shutil.copyfile(args.zip, output / 'sample.zip')
    content = b'xdzip association fixture\n'
    for name, mode in [('sample.tar', 'w'), ('sample.tar.gz', 'w:gz')]:
        with tarfile.open(output / name, mode) as archive:
            item = tarfile.TarInfo('hello.txt')
            item.size = len(content)
            archive.addfile(item, io.BytesIO(content))
    shutil.copyfile(output / 'sample.tar.gz', output / 'sample.tgz')
    for source, name in [
        ('crates/xdzip-codec-7z/tests/fixtures/copy.7z', 'sample.7z'),
        ('tests/fixtures/native-archives/rar7-store-small.rar', 'sample.rar'),
        ('tests/fixtures/cab-iso/named.cab', 'sample.cab'),
    ]:
        shutil.copyfile(ROOT / source, output / name)
    for source, name in [('iso9660.iso.gz', 'sample.iso'), ('udf-regular.iso.gz', 'sample.udf')]:
        with gzip.open(ROOT / 'tests/fixtures/cab-iso' / source, 'rb') as archive:
            data = archive.read(64 * 1024 * 1024 + 1)
        if not data or len(data) > 64 * 1024 * 1024:
            raise ValueError('Optical fixture exceeds its bounded output')
        (output / name).write_bytes(data)
    print('Prepared ZIP, TAR, compressed TAR, 7z, RAR, CAB, ISO and UDF inputs')


if __name__ == '__main__':
    main()
