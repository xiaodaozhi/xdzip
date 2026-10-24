# scripts/check-move-interop.py

"""Verify archive move fixtures independently with standard Python readers."""
import argparse
from pathlib import Path
import tarfile
import zipfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--input', type=Path, required=True)
args = parser.parse_args()
expected = {'资料/目标/原文件.txt': b'original bytes', '资料/目标/嵌套/子目录/a.txt': b'nested data'}
with zipfile.ZipFile(args.input / 'moved.zip') as archive:
    assert archive.testzip() is None
    assert {name: archive.read(name) for name in archive.namelist() if not name.endswith('/')} == expected
    assert '资料/目标/空目录/' in archive.namelist()
    assert '资料/原文件.txt' not in archive.namelist()
for extension in ('tar', 'tar.gz'):
    with tarfile.open(args.input / ('moved.' + extension)) as archive:
        assert {item.name: archive.extractfile(item).read() for item in archive if item.isfile()} == expected
        assert archive.getmember('资料/目标/空目录').isdir()
        assert '资料/原文件.txt' not in archive.getnames()
print('ZIP, TAR and TAR.GZ moves: independent readers verified moved contents and empty directories')
