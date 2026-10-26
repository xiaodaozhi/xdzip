"""Prepare fixed RAR data and original mathematical Store headers under build/."""
import argparse
import binascii
import hashlib
import json
from pathlib import Path
import struct
import urllib.parse
import urllib.request
import zlib

ROOT = Path(__file__).resolve().parents[1]
MAX_DOWNLOAD = 512 * 1024
MAX_ARCHIVE = 2 * 1024 * 1024
PUBLIC_COMPATIBILITY = [
    dict(file='junrar-unicode.rar', bytes=236,
         sha256='b35d9c57cab65a2f15827cb072aed971a99902d332f8c0ccf1ebcac69ab951e2',
         source='src/test/resources/com/github/junrar/unicode.rar'),
    dict(file='junrar-rar3-nonascii-password.rar', bytes=148,
         sha256='53d19b0aec508bd49926162b45a4fd7ecaa1b9d8c13060bc4dbe1dbb953896be',
         source='src/test/resources/com/github/junrar/password/rar3-nonascii-password.rar'),
    dict(file='junrar-LICENSE.txt', bytes=1861,
         sha256='9d90c3649d4eb064ce2fff366a7421107fbbc3c65d8eede45df5dec95e4e016c', source='LICENSE'),
    dict(file='junrar-rar3-nonascii-recipe.txt', bytes=2799,
         sha256='04b90e26040b9472e927487e6265e09c4170333f1b6d48edf021d253e32f1d57',
         source='src/test/resources/com/github/junrar/password/generate_nonascii_password_fixture.py'),
]
for item in PUBLIC_COMPATIBILITY:
    item.update(
        url='https://raw.githubusercontent.com/junrar/junrar/'
            '57091f9ccd43661cf8f12c389917cc24950df707/' + item['source'],
        encoding='raw',
        notice='Pinned Junrar public data and original UnRAR terms. Build/CI-only, no inferred relicensing. '
               'The recipe is only an official-author reproduction docstring, not decoder source. '
               'Unicode data does not cover uncorrected high-bit narrow-byte runs.')


def digest(data):
    return hashlib.sha256(data).hexdigest()


def build_output(path):
    path = Path(path).resolve()
    base = (ROOT / 'build').resolve()
    if path == base or not path.is_relative_to(base):
        raise ValueError('RAR fixture output must be a dedicated build directory')
    return path


def fixed_file(path, data):
    if path.is_symlink():
        raise ValueError('fixture target is a link')
    if path.exists():
        if not path.is_file() or path.stat().st_size != len(data) or path.read_bytes() != data:
            raise ValueError('cached fixture differs from its pinned identity')
    else:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)


def decode_uu(data):
    if len(data) > MAX_DOWNLOAD:
        raise ValueError('encoded fixture exceeds limit')
    lines = data.splitlines()
    starts = [i for i, line in enumerate(lines) if line.startswith(b'begin ')]
    if len(starts) != 1:
        raise ValueError('invalid UU fixture begin')
    result = bytearray()
    for line in lines[starts[0] + 1:]:
        if line == b'end':
            return bytes(result)
        result.extend(binascii.a2b_uu(line))
        if len(result) > MAX_ARCHIVE:
            raise ValueError('decoded fixture exceeds limit')
    raise ValueError('missing UU fixture end')


def public_data(item, destination, archives=None, offline=False):
    cached = destination / item['file']
    candidates = [cached] + ([Path(archives) / item.get('cache_subdirectory', '') / item['file']] if archives else [])
    for path in candidates:
        if path.exists():
            if path.is_symlink() or not path.is_file() or path.stat().st_size != item['bytes']:
                raise ValueError('cached public fixture size/type mismatch')
            data = path.read_bytes()
            if digest(data) != item['sha256']:
                raise ValueError('cached public fixture digest mismatch')
            fixed_file(cached, data)
            return data, 0
    if offline:
        raise ValueError('pinned public fixture is unavailable offline')
    parsed = urllib.parse.urlparse(item['url'])
    if parsed.scheme != 'https' or parsed.hostname != 'raw.githubusercontent.com' or parsed.query or parsed.username:
        raise ValueError('public fixture URL is outside the fixed HTTPS host')
    request = urllib.request.Request(item['url'], headers={'User-Agent': 'xdzip-fixed-fixture-check'})
    data = bytearray()
    with urllib.request.urlopen(request, timeout=30) as response:
        final = urllib.parse.urlparse(response.geturl())
        if final.scheme != 'https' or final.hostname != parsed.hostname:
            raise ValueError('public fixture redirect left its fixed host')
        length = response.headers.get('Content-Length')
        if length is not None and int(length) > MAX_DOWNLOAD:
            raise ValueError('public fixture response exceeds limit')
        while block := response.read(65536):
            data.extend(block)
            if len(data) > MAX_DOWNLOAD:
                raise ValueError('public fixture response exceeds limit')
    encoded = bytes(data)
    if item.get('encoded_sha256') and digest(encoded) != item['encoded_sha256']:
        raise ValueError('public encoded fixture digest mismatch')
    data = decode_uu(encoded) if item['encoding'] == 'uu' else encoded
    if len(data) != item['bytes'] or digest(data) != item['sha256']:
        raise ValueError('public decoded fixture identity mismatch')
    fixed_file(cached, data)
    return data, len(encoded)


def ppm_subset(data):
    """Retain existing main/file/end and unchanged 33-byte compressed payload."""
    main = 7 + struct.unpack_from('<H', data, 12)[0]
    at, selected, ending = main, [], None
    while at < len(data):
        _, kind, flags, size = struct.unpack_from('<HBHH', data, at)
        packed = struct.unpack_from('<I', data, at + 7)[0] if flags & 0x8000 else 0
        if size < 7 or at + size + packed > len(data):
            raise ValueError('fixed PPM fixture record bounds')
        if kind == 0x74:
            count = struct.unpack_from('<H', data, at + 26)[0]
            name = data[at + 32:at + 32 + count]
            if name.replace(b'\\', b'/') == b'testdir/test.txt':
                selected.append(data[at:at + size + packed])
        if kind == 0x7b:
            ending = data[at:at + size]
            break
        at += size + packed
    if len(selected) != 1 or ending is None:
        raise ValueError('fixed PPM subset identity')
    result = data[:main] + selected[0] + ending
    if digest(result) != 'b2fa4809fea6191300542234f71066a5ae109b2c4e37f2780ade14d77934e5a3':
        raise ValueError('PPM subset digest mismatch')
    return result


def vint(value):
    out = bytearray()
    while True:
        byte = value & 127
        value >>= 7
        out.append(byte | (128 if value else 0))
        if not value:
            return bytes(out)


def header(kind, flags, fields, data=b''):
    body = vint(kind) + vint(flags)
    if flags & 2:
        body += vint(len(data))
    body += fields
    checked = vint(len(body)) + body
    return struct.pack('<I', zlib.crc32(checked)) + checked + data


def mathematical_store(unknown):
    # Original test metadata only; no compressed RAR stream is produced.
    name = b'unknown.bin' if unknown else b'empty.bin'
    fields = vint(4 | (8 if unknown else 0)) + vint(0) + vint(0x20)
    fields += struct.pack('<I', 0) + vint(0) + vint(0) + vint(len(name)) + name
    return (b'Rar!\x1a\x07\x01\x00' + header(1, 0, vint(0))
            + header(2, 2, fields) + header(5, 0, vint(0)))


def prepare(output, archives=None, offline=False):
    output = build_output(output)
    output.mkdir(parents=True, exist_ok=True)
    catalogue = json.loads((ROOT / 'scripts/rar-rust-fixtures.json').read_text(encoding='utf-8'))
    public, records = {}, []
    for item in catalogue['public'] + PUBLIC_COMPATIBILITY:
        data, downloaded = public_data(item, output / 'public', archives, offline)
        public[item['file']] = data
        records.append(dict(file=item['file'], url=item['url'], sha256=digest(data),
                            bytes=len(data), downloaded_body_bytes=downloaded, notice=item['notice']))
    ppm = output / 'single-20b-ppm.rar'
    fixed_file(ppm, ppm_subset(public['test_read_format_rar_compress_best.rar']))
    unknown, empty = output / 'unknown-size.rar', output / 'known-empty.rar'
    fixed_file(unknown, mathematical_store(True))
    fixed_file(empty, mathematical_store(False))
    owned = ROOT / 'crates/xdzip-codec-rar/tests/fixtures'
    manifest = json.loads((owned / 'legacy20-owned/sources.json').read_text(encoding='utf-8'))
    owned20 = output / 'owned20'
    for item in manifest['fixtures']:
        if Path(item['file']).name != item['file']:
            raise ValueError('owned fixture basename')
        data = (owned / 'legacy20-owned' / item['file']).read_bytes()
        if len(data) != item['bytes'] or digest(data) != item['sha256']:
            raise ValueError('owned20 archive identity mismatch')
        fixed_file(owned20 / item['file'], data)
    first = bytes((31 * i + 7 * (i >> 5) + 13) & 255 for i in range(65537))
    inputs = {'first.bin': first, 'second.bin': first[173:] + first[:173] + first[:32780],
              'third.bin': bytes((i * i + 5 * i + (i >> 3)) & 255 for i in range(33333))}
    for name, data in inputs.items():
        fixed_file(owned20 / name, data)
    item = json.loads((owned / 'legacy26-owned/sources.json').read_text(encoding='utf-8'))['archive']
    large = owned / 'legacy26-owned' / item['file']
    if large.stat().st_size > MAX_ARCHIVE or large.stat().st_size != item['bytes'] or digest(large.read_bytes()) != item['sha256']:
        raise ValueError('owned26 archive identity mismatch')
    environment = dict(XDZIP_RAR_PPM_FIXTURE=str(ppm),
                       XDZIP_RAR_LARGE_PPM_FIXTURE=str(output / 'public/test_read_format_rar_ppmd_lzss_conversion.rar'),
                       XDZIP_RAR_LEGACY_FIXTURE=str(output / 'public/rar20-password-compressed.rar'),
                       XDZIP_RAR_UNKNOWN_FIXTURE=str(unknown), XDZIP_RAR_KNOWN_EMPTY_FIXTURE=str(empty),
                       XDZIP_RAR290_FIXTURE_ROOT=str(owned20), XDZIP_RAR26_FIXTURE=str(large),
                       XDZIP_RAR_PUBLIC_UNICODE_FIXTURE=str(output / 'public/junrar-unicode.rar'),
                       XDZIP_RAR_PUBLIC_PASSWORD_FIXTURE=str(output / 'public/junrar-rar3-nonascii-password.rar'),
                       XDZIP_RAR_PUBLIC_PASSWORD_RECIPE=str(output / 'public/junrar-rar3-nonascii-recipe.txt'))
    report = dict(schema_version=1, status='prepared', public=records,
                  network_per_response_limit=MAX_DOWNLOAD, no_large_plaintext_created=True,
                  ppm_subset_sha256=digest(ppm.read_bytes()),
                  mathematical_store=dict(unknown_sha256=digest(unknown.read_bytes()), empty_sha256=digest(empty.read_bytes())),
                  environment={key: str(Path(value).relative_to(ROOT)) for key, value in environment.items()})
    (output / 'sources.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    return environment, report


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--archives', type=Path, help='optional pinned decoded-data cache')
    parser.add_argument('--offline', action='store_true')
    args = parser.parse_args()
    _, report = prepare(args.output, args.archives, args.offline)
    print(json.dumps(dict(status=report['status'], public_fixtures=len(report['public']))))
