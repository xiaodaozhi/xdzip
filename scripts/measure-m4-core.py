#!/usr/bin/env python3
"""Create real ZIPs and independently verify bytes/CRC/ZIP64 with Python zipfile."""
import argparse
import hashlib
import json
import platform
from pathlib import Path
import re
import subprocess
import tempfile
import time
import zipfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--binary', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
binary = args.binary.resolve()
native_temp = subprocess.check_output([str(binary), '--native-temp-base'], text=True).strip()
reports = []
with tempfile.TemporaryDirectory(prefix='xdzip-m4-measure-', dir=native_temp) as temporary:
    owned = Path(temporary)
    source, target = owned / '资料', owned / 'output'
    source.mkdir()
    target.mkdir(mode=0o700 if platform.system() != 'Windows' else 0o777)
    (source / '空目录').mkdir()
    contents = {'中文.txt': ('中文 ZIP 内容\n' * 1024).encode(), 'empty.bin': b'',
                'payload.bin': bytes(range(256)) * (32 * 1024)}
    for name, data in contents.items():
        (source / name).write_bytes(data)

    def run(root, name, method, level):
        command = [str(binary), str(root), str(target), name, str(method), str(level)]
        if platform.system() == 'Darwin':
            command = ['/usr/bin/time', '-l'] + command
        process = subprocess.run(command, capture_output=True, text=True, timeout=180)
        if process.returncode:
            raise SystemExit(process.stderr)
        report = json.loads(process.stdout)
        report.update(method=method, level=level)
        if platform.system() == 'Darwin':
            match = re.search(r'(\d+)\s+maximum resident set size', process.stderr)
            report['peak_rss_kib'] = int(match.group(1)) / 1024 if match else None
        assert sorted(p.name for p in target.iterdir()) == sorted([r['archive'] for r in reports] + [name])
        report['archive'] = name
        return report

    for method, level in [(0, 6), (8, 1), (8, 6), (8, 9)]:
        name = f'created-{method}-{level}.zip'
        report = run(source, name, method, level)
        start = time.perf_counter()
        with zipfile.ZipFile(target / name) as archive:
            assert archive.testzip() is None
            assert set(archive.namelist()) == {'资料/', '资料/空目录/'} | {'资料/' + n for n in contents}
            for filename, data in contents.items():
                info = archive.getinfo('资料/' + filename)
                assert info.compress_type == method
                assert info.flag_bits & 0x800  # UTF-8 name marker.
                assert hashlib.sha256(archive.read(info)).digest() == hashlib.sha256(data).digest()
            assert archive.getinfo('资料/空目录/').is_dir()
        report.update(independent_byte_crc_check=True, independent_ms=(time.perf_counter()-start)*1000)
        reports.append(report)
    # More than 65,535 central-directory records exercises ZIP64 entry-count
    # publication without allocating multi-gigabyte buffers or fixtures.
    many = owned / 'many'
    many.mkdir()
    for index in range(66_000):
        (many / f'f-{index:05}.txt').touch()
    report = run(many, 'zip64-count.zip', 0, 6)
    with zipfile.ZipFile(target / 'zip64-count.zip') as archive:
        assert len(archive.infolist()) == 66_001
        assert archive.testzip() is None
        assert archive.getinfo('many/f-65999.txt').file_size == 0
    report.update(independent_byte_crc_check=True, zip64_entry_count=66_001)
    reports.append(report)

result = dict(os=platform.system(), architecture=platform.machine(), profile='release',
              cases=reports, passed=len(reports),
              limits=['One owned-fixture run per case; filesystem and warm-cache effects apply',
                      'Verification/commit time includes final source checks and fsync/publication',
                      'ZIP64 large-byte offsets are not exercised by the count fixture',
                      'Unavailable native RSS is recorded as null'])
args.output.parent.mkdir(parents=True, exist_ok=True)
args.output.write_text(json.dumps(result, ensure_ascii=False, indent=2)+'\n', encoding='utf-8')
print(f"ZIP creation interoperability: {result['passed']} passed")
