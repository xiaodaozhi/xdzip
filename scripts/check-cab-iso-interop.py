# scripts/check-cab-iso-interop.py

"""Decode fixed independent CAB/ISO/UDF inputs through the real native worker."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def module(filename):
    spec = importlib.util.spec_from_file_location(filename.replace('-', '_'), ROOT / 'scripts' / filename)
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


NATIVE = module('check-native-codecs.py')
PREPARE = module('prepare-cab-iso-fixtures.py')
HELLO = b'xdzip read-only archive fixture\n'
NESTED = 'Unicode payload: \u4e2d\u6587\n'.encode('utf-8')


def verify(worker, parts, format_id, expected, names=None, special=()):
    kwargs = dict(format_id=format_id)
    if format_id == 8:
        kwargs['volume_names'] = names or [b'fixture.cab']
    rows, _ = NATIVE.run_worker(worker, parts, **kwargs)
    actual = {}
    flagged = set()
    for row in rows:
        name = row['name'].replace('\\', '/')
        if row['flags'] & 1:
            continue
        if row['flags'] & 12:
            flagged.add(name)
            try:
                NATIVE.run_worker(worker, parts, selected=row['index'], **kwargs)
            except NATIVE.WorkerError:
                continue
            raise AssertionError('Special record decoded as ordinary file: ' + name)
        _, data = NATIVE.run_worker(worker, parts, selected=row['index'], **kwargs)
        assert len(data) == row['size'], (name, 'decoded size')
        actual[name] = data
    assert actual == expected, (format_id, list(actual), list(expected))
    assert flagged == set(special), ('special records', flagged, special)
    try:
        NATIVE.run_worker(worker, parts, creation={'new.txt': HELLO}, **kwargs)
    except NATIVE.WorkerError as error:
        assert str(error) == 'native_error_3', ('read-only creation', str(error))
    else:
        raise AssertionError('Read-only format accepted creation')
    return dict(files=len(actual), special=len(flagged), decoded_bytes=sum(map(len, actual.values())))


def check(worker, output):
    fixtures = PREPARE.prepare(output / 'fixtures')
    results = []
    def one(filename, format_id, expected, special=()):
        details = verify(worker, [(fixtures / filename).read_bytes()], format_id, expected, special=special)
        results.append(dict(case=filename, status='passed', **details))
    one('named.cab', 8, {'hello.txt': HELLO, 'nested/\u4e2d\u6587.txt': NESTED, 'empty.txt': b''})
    methods = {
        'mszip.txt': b'If you can read this, the MSZIP decompressor is working!\n',
        'lzx.txt': b'-' * 65 + b'\nIf you can read this, the LZX decompressor is working!\n' + b'-' * 65 + b'\n',
        'qtm.txt': b'If you can read this, the Quantum decompressor is working!\n',
    }
    for name, digest in [('mszip.txt', '940cba86658fbceb582faecd2b5975d1'),
                         ('lzx.txt', '703474293b614e7110b3eb8ac2762b53'),
                         ('qtm.txt', '98fcfa4962a0f169a3c7fdbcb445cf17')]:
        assert hashlib.md5(methods[name]).hexdigest() == digest, 'upstream plaintext identity'
    one('mszip_lzx_qtm.cab', 8, methods)
    one('normal_2files_1folder.cab', 8, {
        'hello.c': b'#include <stdio.h>\r\n\r\nvoid main(void)\r\n{\r\n    printf("Hello, world!\\n");\r\n}\r\n',
        'welcome.c': b'#include <stdio.h>\r\n\r\nvoid main(void)\r\n{\r\n    printf("Welcome!\\n");\r\n}\r\n\r\n',
    })
    names = [f'cabd_multi_basic_pt{i}.cab'.encode() for i in range(1, 6)]
    parts = [(fixtures / name.decode()).read_bytes() for name in names]
    joined = b''.join(f'This is the data from cabinet part {i}.\n'.encode() for i in range(1, 6))
    details = verify(worker, parts, 8, {'test1.txt': joined[:76], 'test2.txt': joined[76:114],
                                      'test3.txt': joined[114:]}, names=names)
    results.append(dict(case='cab-cross-volume-files', status='passed', **details))
    logical_names = [f'owned-part{i}.cab'.encode() for i in range(1, 5)]
    logical_parts = [(fixtures / name.decode()).read_bytes() for name in logical_names]
    value, logical_payload = 0x713A91D5, bytearray()
    for _ in range(196608):
        value = (value * 1664525 + 1013904223) & 0xFFFFFFFF
        logical_payload.append(value >> 24)
    details = verify(worker, logical_parts, 8, {'payload.bin': bytes(logical_payload)}, names=logical_names)
    results.append(dict(case='cab-logical-previous-origin', status='passed', **details))
    wrong_origin = logical_parts[3].replace(b'owned-part1.cab', b'owned-part2.cab')
    assert wrong_origin != logical_parts[3]
    try:
        NATIVE.run_worker(worker, [*logical_parts[:3], wrong_origin], format_id=8,
                          volume_names=logical_names)
    except NATIVE.WorkerError:
        results.append(dict(case='cab-unrelated-previous-origin', status='passed'))
    else:
        raise AssertionError('Unrelated earlier cabinet accepted as a continued file origin')
    wrong_set = bytearray(parts[2])
    wrong_set[32:34] = b'\x01\x00'
    for label, invalid_parts, invalid_names in [
        ('missing-volume', parts[:-1], names[:-1]),
        ('wrong-set', [*parts[:2], bytes(wrong_set), *parts[3:]], names),
        ('unauthorized-name', parts, [*names[:2], b'other.cab', *names[3:]]),
    ]:
        try:
            NATIVE.run_worker(worker, invalid_parts, format_id=8, volume_names=invalid_names)
        except NATIVE.WorkerError:
            results.append(dict(case='cab-' + label, status='passed'))
        else:
            raise AssertionError('Incomplete CAB accepted: ' + label)
    one('iso9660.iso', 9, {'HELLO.TXT': HELLO, 'NESTED/DATA.TXT': NESTED, 'ZERO.TXT': b''})
    unicode_files = {'hello.txt': HELLO, 'nested/data.txt': NESTED, 'zero.txt': b'', '\u4e2d\u6587.txt': NESTED}
    one('joliet.iso', 9, unicode_files)
    one('rock-ridge.iso', 9, unicode_files, special=('link',))
    for filename in ('udf.iso', 'hybrid.iso'):
        one(filename, 10, {**unicode_files, 'udf-only.txt': HELLO}, special=('udf-link',))
    one('udf-regular.iso', 10, {**unicode_files, 'udf-only.txt': HELLO})
    for filename, format_id in [('named.cab', 8), ('iso9660.iso', 9), ('udf.iso', 10)]:
        data = (fixtures / filename).read_bytes()
        kwargs = dict(format_id=format_id)
        if format_id == 8:
            kwargs['volume_names'] = [b'fixture.cab']
        try:
            NATIVE.run_worker(worker, [data[:len(data) // 2]], **kwargs)
        except NATIVE.WorkerError:
            results.append(dict(case=filename + '-truncated', status='passed'))
        else:
            raise AssertionError('Truncated input accepted: ' + filename)
    output.mkdir(parents=True, exist_ok=True)
    (output / 'cab-iso-interop.json').write_text(json.dumps(dict(status='passed', cases=results), indent=2) + '\n')
    print(f'CAB/ISO/UDF independent fixture checks passed: {len(results)} cases')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--worker', type=Path, required=True)
    parser.add_argument('--output', type=Path, default=ROOT / 'build/review/cab-iso-interop')
    args = parser.parse_args()
    check(args.worker.resolve(), args.output.resolve())
