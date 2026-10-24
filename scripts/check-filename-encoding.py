# scripts/check-filename-encoding.py

"""Verify legacy-name extraction through the worker service and rewritten names independently."""
import argparse
import gzip
import io
import json
from pathlib import Path
import struct
import subprocess
import tarfile
import tempfile
import unicodedata
import zipfile
import zlib

ROOT = Path(__file__).resolve().parents[1]
CONTENT = b'owned legacy payload\n'
SAMPLES = {
    'GBK-overlap': ('gbk', ['目录/', '目录/中文.txt']),
    'GBK': ('gbk', ['中文资料/', '中文资料/中文文件与压缩测试.txt', '中文资料/跨平台解压使用说明.txt']),
    'GB18030': ('gb18030', ['中文资料/', '中文资料/中文文件与压缩测试.txt', '中文资料/跨平台解压使用说明.txt', '中文资料/𠮷野家😀.txt']),
    'Big5': ('big5', ['繁體中文資料/', '繁體中文資料/檔案說明與測試.txt', '繁體中文資料/壓縮文件和相片.txt']),
    'Shift_JIS': ('cp932', ['日本語資料/', '日本語資料/ソフトウェア説明書.txt', '日本語資料/ひらがなカタカナ.txt']),
    'EUC-KR': ('cp949', ['한글자료/', '한글자료/압축파일설명서.txt', '한글자료/사진과문서입니다.txt']),
    'windows-1251': ('cp1251', ['Русские документы/', 'Русские документы/Описание архива.txt', 'Русские документы/Тестирование файлов.txt']),
    'CP437': ('cp437', ['café.txt']),
    'UTF8-unmarked': ('utf-8', ['多语言资料/', '多语言资料/日本語と中文.txt']),
    'UnicodePath': ('ascii', ['资料/', '资料/原文件.txt']),
}


class RawNameInfo(zipfile.ZipInfo):
    # Isolated test writer intentionally bypasses Python's automatic UTF-8 filename flag.
    def _encodeFilenameFlags(self):
        return self.raw, self.flag_bits & ~0x800


def zipped(label, encoding, names):
    stream = io.BytesIO()
    with zipfile.ZipFile(stream, 'w', compression=zipfile.ZIP_DEFLATED) as archive:
        for index, name in enumerate(names):
            item = RawNameInfo(name)
            item.raw = name.encode(encoding) if label != 'UnicodePath' else (b'legacy/' if index == 0 else b'legacy/original.txt')
            item.compress_type = zipfile.ZIP_DEFLATED
            if label == 'UnicodePath':
                encoded = name.encode('utf-8')
                item.extra = struct.pack('<HHBI', 0x7075, len(encoded) + 5, 1, zlib.crc32(item.raw)) + encoded
            archive.writestr(item, b'' if name.endswith('/') else CONTENT)
    return stream.getvalue()


def tarred(encoding, names):
    stream = io.BytesIO()
    with tarfile.open(fileobj=stream, mode='w', format=tarfile.GNU_FORMAT, encoding=encoding, errors='strict') as archive:
        for name in names:
            item = tarfile.TarInfo(name)
            item.type = tarfile.DIRTYPE if name.endswith('/') else tarfile.REGTYPE
            item.size = 0 if item.isdir() else len(CONTENT)
            archive.addfile(item, None if item.isdir() else io.BytesIO(CONTENT))
    return stream.getvalue()


def service_cases(probe):
    records = []
    base = subprocess.check_output([str(probe), '--native-temp-base'], text=True, encoding='utf-8', timeout=15).strip()
    with tempfile.TemporaryDirectory(prefix='xdzip-legacy-interop-', dir=base) as owned:
        root = Path(owned)
        for label, (encoding, names) in SAMPLES.items():
            formats = ['zip'] if label in ('CP437', 'UnicodePath') else ['zip', 'tar', 'tar.gz']
            for extension in formats:
                data = zipped(label, encoding, names) if extension == 'zip' else tarred(encoding, names)
                if extension == 'tar.gz':
                    data = gzip.compress(data, mtime=0)
                path = root / ('owned.' + extension)
                path.write_bytes(data)
                target = root / (label + '-' + extension)
                target.mkdir()
                result = subprocess.run([str(probe), str(path), str(target), '--generic'], capture_output=True, timeout=45)
                assert result.returncode == 0, (label, extension, 'worker extraction failed', result.returncode)
                # Native filesystems may expose canonically decomposed spellings. Compare
                # equivalent filesystem names here; archive metadata below remains byte-exact.
                canonical = lambda value: unicodedata.normalize('NFC', value)
                actual_files = [p for p in target.rglob('*') if p.is_file()]
                actual = {canonical(p.relative_to(target).as_posix()): p.read_bytes() for p in actual_files}
                expected = {canonical(name): CONTENT for name in names if not name.endswith('/')}
                assert len(actual_files) == len(expected)
                assert actual == expected, (label, extension, list(actual))
                directories = {canonical(p.relative_to(target).as_posix()) for p in target.rglob('*') if p.is_dir()}
                assert directories == {canonical(name.rstrip('/')) for name in names if name.endswith('/')}, (label, extension)
                assert path.read_bytes() == data
                summary = json.loads(result.stdout)
                assert summary['files'] == len(expected) and summary['bytes'] == sum(map(len, expected.values()))
                records.append({'encoding': label, 'format': extension, 'files': len(expected), 'passed': True})
    return records


def rewritten_cases(directory):
    records = []
    for label in SAMPLES:
        formats = ['zip'] if label in ('CP437', 'UnicodePath') else ['zip', 'tar', 'tar.gz']
        for extension in formats:
            case = directory / label / extension
            expected = (case / 'names.txt').read_text(encoding='utf-8').splitlines()
            path = case / ('rewritten.' + extension)
            if extension == 'zip':
                with zipfile.ZipFile(path) as archive:
                    assert archive.testzip() is None
                    rows = archive.infolist()
                    names = [item.filename for item in rows]
                    assert all(item.flag_bits & 0x800 for item in rows if not item.filename.isascii())
                    contents = {item.filename: archive.read(item) for item in rows if not item.is_dir()}
            else:
                with tarfile.open(fileobj=io.BytesIO(gzip.decompress(path.read_bytes())) if extension == 'tar.gz' else io.BytesIO(path.read_bytes()), encoding='utf-8', errors='strict') as archive:
                    rows = archive.getmembers()
                    names = [item.name + ('/' if item.isdir() and not item.name.endswith('/') else '') for item in rows]
                    assert all(item.pax_headers.get('path', '').rstrip('/') == item.name.rstrip('/') for item in rows if not item.name.isascii())
                    contents = {item.name: archive.extractfile(item).read() for item in rows if item.isfile()}
            assert names == expected, (label, extension, names, expected)
            assert all(content == (b'new owned input' if name.endswith('added.txt') else CONTENT) for name, content in contents.items())
            records.append({'encoding': label, 'format': extension, 'files': len(contents), 'passed': True})
    return records


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--probe', type=Path)
    parser.add_argument('--input', type=Path)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    assert args.probe or args.input
    output = args.output.resolve()
    assert any(output.is_relative_to(ROOT / folder) for folder in ('build', 'dist'))
    service = service_cases(args.probe.resolve()) if args.probe else []
    rewritten = rewritten_cases(args.input) if args.input else []
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps({'passed': True, 'service': service, 'rewritten': rewritten,
        'limits': ['Statistical detection cannot uniquely recover every short or mixed-encoding archive.',
                   'This is worker and format interoperability evidence, not native desktop acceptance.']}, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(f'Filename encoding: {len(service)} independent ZIP/TAR/TAR.GZ worker cases, {len(rewritten)} rewritten archives passed')


if __name__ == '__main__':
    main()
