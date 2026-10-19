#!/usr/bin/env python3
"""Owned, deterministic nine-entry ZIP for UI review; no synthetic runtime model."""
import io
from pathlib import Path
from zipfile import ZipFile, ZipInfo, ZIP_STORED

ROOT = Path(__file__).resolve().parents[1]
# A valid blank PDF with a page tree, used only for the file-type icon in this fixture.
pdf = bytearray(b'%PDF-1.4\n')
objects = [b'<< /Type /Catalog /Pages 2 0 R >>',
           b'<< /Type /Pages /Kids [3 0 R] /Count 1 >>',
           b'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] >>']
offsets = [0]
for index, content in enumerate(objects, 1):
    offsets.append(len(pdf))
    pdf.extend(f'{index} 0 obj\n'.encode() + content + b'\nendobj\n')
xref = len(pdf)
pdf.extend(b'xref\n0 4\n0000000000 65535 f \n')
for offset in offsets[1:]:
    pdf.extend(f'{offset:010d} 00000 n \n'.encode())
pdf.extend(f'trailer\n<< /Size 4 /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n'.encode())
records = [('01 照片/', b''), ('02 文档/', b''), ('03 地图/', b''),
           ('冰川湖.png', (ROOT / 'design/assets/demo/photo-lake-thumbnail.png').read_bytes()),
           ('山间日出.png', (ROOT / 'design/assets/demo/photo-sunrise-thumbnail.png').read_bytes()),
           ('行程手册.pdf', bytes(pdf)), ('旅行清单.txt', '北境旅行\n'.encode()),
           ('路线规划.gpx', b'<?xml version="1.0"?><gpx version="1.1" creator="xdzip"/>'),
           ('README.md', '# UI review fixture\n'.encode())]
result = io.BytesIO()
with ZipFile(result, 'w') as archive:
    for name, data in records:
        entry = ZipInfo(name, (2026, 10, 1, 0, 0, 0))
        entry.compress_type = ZIP_STORED
        archive.writestr(entry, data)
path = ROOT / 'tests/fixtures/zip/visual-main.zip'
path.write_bytes(result.getvalue())
print(f'{path.name}: {len(records)} real entries, {path.stat().st_size} bytes')
