# scripts/check-update-interop.py

"""Independently verify locally generated update fixtures with Python readers."""
import argparse
from pathlib import Path
import tarfile
import zipfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--input', type=Path, required=True)
args = parser.parse_args()
expected = {'资料/原文件.txt': b'original bytes', '资料/新文件.txt': b'new bytes'}
with zipfile.ZipFile(args.input/'updated.zip') as archive:
    assert archive.testzip() is None
    assert {name: archive.read(name) for name in archive.namelist() if not name.endswith('/')} == expected
    assert '资料/空目录/' in archive.namelist()
for extension in ('tar', 'tar.gz'):
    with tarfile.open(args.input/('updated.'+extension)) as archive:
        assert {item.name: archive.extractfile(item).read() for item in archive if item.isfile()} == expected
        assert archive.getmember('资料/空目录').isdir()
print('ZIP, TAR and TAR.GZ updates: independent Python readers verified original and added bytes plus empty directories')
