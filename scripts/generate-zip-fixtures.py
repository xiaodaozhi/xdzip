#!/usr/bin/env python3
"""Create tiny deterministic ZIP samples, owned by xdzip; no third-party archives."""
import io
from pathlib import Path
import struct
import warnings
import zipfile

ROOT = Path(__file__).resolve().parents[1] / 'tests/fixtures/zip'
ROOT.mkdir(parents=True, exist_ok=True)

def archive(records):
    output = io.BytesIO()
    with warnings.catch_warnings():
        warnings.simplefilter('ignore', UserWarning)
        with zipfile.ZipFile(output, 'w') as writer:
            for name, content, method in records:
                item = zipfile.ZipInfo(name, (2026, 1, 1, 0, 0, 0))
                item.compress_type = method
                writer.writestr(item, content)
    return output.getvalue()

normal = archive([('旅行/', b'', 0), ('旅行/说明.txt', '北境旅行'.encode(), 8),
                  ('notes.txt', b'first', 0), ('notes.txt', b'second', 0),
                  ('empty.txt', b'', 0)])
(ROOT / 'normal.zip').write_bytes(normal)
(ROOT / 'empty.zip').write_bytes(archive([]))
(ROOT / 'unsafe-names.zip').write_bytes(archive([('../escape.txt', b'x', 0),
    ('/absolute.txt', b'x', 0), ('C:\\drive.txt', b'x', 0), ('rtl\u202efile.txt', b'x', 0)]))
(ROOT / 'truncated.zip').write_bytes(normal[:-10])
wrong_count = bytearray(normal)
struct.pack_into('<HH', wrong_count, len(normal) - 14, 6, 6)
(ROOT / 'wrong-count.zip').write_bytes(wrong_count)
(ROOT / 'pages.zip').write_bytes(archive([(f'folder/entry-{i:03}.txt', b'x', 0) for i in range(270)]))
