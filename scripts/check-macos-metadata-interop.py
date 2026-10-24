# scripts/check-macos-metadata-interop.py

"""Verify filtered creation and addition with independent standard-library readers."""
import argparse
from pathlib import Path
import tarfile
import zipfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--input', type=Path, required=True)
args = parser.parse_args()
expected = {
    'created': {'输入/文件.txt': ('中文与 ZIP 互操作\n' * 1000).encode(), '输入/empty.bin': b''},
    'updated': {'原文件.txt': b'', '.DS_Store': b'', '__MACOSX/._original': b'', 'add/新文件.txt': b'new bytes'},
}
for operation in ('created', 'updated'):
    directories = {'输入', '输入/空目录'} if operation == 'created' else {'add', 'add/空目录'}
    for extension in ('zip', 'tar', 'tar.gz'):
        path = args.input / f'{operation}.{extension}'
        if extension == 'zip':
            with zipfile.ZipFile(path) as archive:
                assert archive.testzip() is None
                files = {item.filename: archive.read(item) for item in archive.infolist() if not item.is_dir()}
                actual_directories = {item.filename.rstrip('/') for item in archive.infolist() if item.is_dir()}
                assert all(item.flag_bits & 0x800 for item in archive.infolist() if not item.filename.isascii())
        else:
            with tarfile.open(path) as archive:
                files = {item.name: archive.extractfile(item).read() for item in archive if item.isfile()}
                actual_directories = {item.name.rstrip('/') for item in archive if item.isdir()}
                assert all(item.pax_headers.get('path', '').rstrip('/') == item.name.rstrip('/') for item in archive if not item.name.isascii())
        assert files == expected[operation], (operation, extension, files.keys())
        assert actual_directories == directories, (operation, extension, actual_directories)
print('macOS metadata: six real ZIP/TAR/TAR.GZ archives verified, filtered inputs, preserved existing records, empty directories and UTF-8 names')
