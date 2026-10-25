# scripts/check-native-codecs.py

"""Exercise the actual native worker against independent tools and fixed fixtures."""
import argparse
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import threading

ROOT = Path(__file__).resolve().parents[1]
CHUNK = 64 * 1024


class WorkerError(Exception):
    pass


def string(value):
    data = value.encode('utf-8')
    return struct.pack('<I', len(data)) + data


def exact(stream, size):
    value = bytearray()
    while len(value) < size:
        block = stream.read(size - len(value))
        if not block:
            raise WorkerError('worker_eof')
        value.extend(block)
    return bytes(value)


def run_worker(worker, parts, *, password=None, selected=None, format_id=7, creation=None, solid=False, encrypt_headers=False):
    source = b''.join(parts)
    request = bytes([2 if creation is not None else 0 if selected is None else 1, format_id])
    request += struct.pack('<QIIQ', 1024**3, 100000, 4096, len(source))
    request += bytes([password is not None]) + string(password or '') + struct.pack('<QI', selected or 0, len(parts))
    request += b''.join(struct.pack('<Q', len(part)) for part in parts)
    if creation is not None:
        request += bytes([6, solid, encrypt_headers, 0]) + struct.pack('<I', len(creation))
    child = subprocess.Popen([str(worker)], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
    timer = threading.Timer(45, child.kill)
    timer.start()
    rows, payload, reads = [], bytearray(), 0
    inputs = list((creation or {}).items())
    positions = [0] * len(inputs)

    def send(kind, value=b''):
        child.stdin.write(struct.pack('<I', len(value) + 1) + bytes([kind]) + value)
        child.stdin.flush()

    try:
        send(1, request)
        for name, content in inputs:
            send(13, bytes([0]) + struct.pack('<QqQ', len(content), 1700000000, 2**64 - 1) + string(name))
        while True:
            size = struct.unpack('<I', exact(child.stdout, 4))[0]
            if not 1 <= size <= 1024**2:
                raise WorkerError('frame_limit')
            frame = exact(child.stdout, size)
            kind, value = frame[0], frame[1:]
            if kind == 10:
                offset, count = struct.unpack('<QI', value)
                if count > CHUNK or offset > len(source):
                    raise WorkerError('source_bounds')
                reads += count
                if reads > 64 * 1024**2:
                    raise WorkerError('read_limit')
                send(20, source[offset:offset + count])
            elif kind == 13:
                index, flags, unpacked, packed, modified, method, length = struct.unpack('<QBQQIII', value[:37])
                if len(value) != 37 + length:
                    raise WorkerError('metadata_length')
                rows.append(dict(index=index, flags=flags, size=unpacked, packed=packed,
                                 method=method, name=value[37:].decode('utf-8')))
                send(21)
            elif kind == 12:
                if len(value) > CHUNK or len(payload) + len(value) > 32 * 1024**2:
                    raise WorkerError('output_limit')
                payload.extend(value)
                send(21)
            elif kind == 11:
                index, count = struct.unpack('<II', value)
                if index >= len(inputs) or count > CHUNK:
                    raise WorkerError('input_bounds')
                content = inputs[index][1]
                block = content[positions[index]:positions[index] + count]
                positions[index] += len(block)
                send(20, block)
            elif kind == 19:
                offset = struct.unpack('<Q', value[:8])[0]
                block = value[8:]
                end = offset + len(block)
                if len(block) > CHUNK or end > 32 * 1024**2:
                    raise WorkerError('write_at_limit')
                if end > len(payload):
                    payload.extend(b'\0' * (end - len(payload)))
                payload[offset:end] = block
                send(21)
            elif kind in (16, 17):
                send(21)
            elif kind == 14:
                raise WorkerError('native_error_' + str(value[0]))
            elif kind == 15:
                break
            else:
                raise WorkerError('unexpected_message_' + str(kind))
        child.stdin.close()
        if child.wait(timeout=5) != 0:
            raise WorkerError('worker_exit')
        return rows, bytes(payload)
    finally:
        timer.cancel()
        if child.poll() is None:
            child.kill()
        child.wait(timeout=5)
        child.stdout.close()
        if not child.stdin.closed:
            child.stdin.close()


def rar_fixtures(worker):
    base = ROOT / 'tests/fixtures/native-archives'
    manifest = json.loads((base / 'sources.json').read_text())
    for item in manifest['fixtures']:
        if hashlib.sha256((base / item['file']).read_bytes()).hexdigest() != item['sha256']:
            raise WorkerError('fixture_identity')
    results = []
    for filename in ('test_read_format_rar4_encrypted_filenames.rar',
                     'test_read_format_rar5_encrypted_filenames.rar',
                     'test_read_format_rar5_solid_encrypted.rar',
                     'test_read_format_rar5_solid_encrypted_filenames.rar'):
        parts = [(base / filename).read_bytes()]
        rows, _ = run_worker(worker, parts, password='password', format_id=4)
        assert len(rows) == 4, (filename, 'record_count')
        for row in rows:
            _, decoded = run_worker(worker, parts, password='password', selected=row['index'], format_id=4)
            assert decoded == ('This is from ' + row['name']).encode(), (filename, row['name'], 'bytes')
        results.append(dict(case=filename, files=len(rows), status='passed'))
    for filename in ('test_read_format_rar_compress_normal.rar', 'test_read_format_rar5_compressed.rar',
                     'test_read_format_rar5_multiple_files_solid.rar'):
        parts = [(base / filename).read_bytes()]
        rows, _ = run_worker(worker, parts, format_id=4)
        total = 0
        for row in rows:
            if not row['flags'] & 5:
                _, decoded = run_worker(worker, parts, selected=row['index'], format_id=4)
                assert len(decoded) == row['size'], (filename, 'decoded_size')
                total += len(decoded)
        results.append(dict(case=filename, files=len(rows), decoded_bytes=total, status='passed'))
    parts = [(base / f'test_read_format_rar5_multiarchive_solid.part{index:02}.rar').read_bytes() for index in range(1, 5)]
    rows, _ = run_worker(worker, parts, format_id=4)
    for row in rows:
        if not row['flags'] & 5:
            _, decoded = run_worker(worker, parts, selected=row['index'], format_id=4)
            assert len(decoded) == row['size'], 'split_decoded_size'
    results.append(dict(case='rar5_solid_multivolume', files=len(rows), status='passed'))
    parts = [(base / 'rar7-store-small.rar').read_bytes()]
    rows, _ = run_worker(worker, parts, format_id=4)
    assert len(rows) == 1 and rows[0]['method'] == 70, 'rar7_declared_algorithm'
    _, decoded = run_worker(worker, parts, selected=0, format_id=4)
    assert decoded == b'RAR newest algorithm verification sample', 'rar7_store_bytes'
    results.append(dict(case='rar7_store', files=1, status='passed'))
    try:
        run_worker(worker, [(base / 'rar7-dictionary-limit.rar').read_bytes()], selected=0, format_id=4)
    except WorkerError as error:
        assert str(error) == 'native_error_4', str(error)
    else:
        raise WorkerError('rar7_dictionary_limit_not_enforced')
    results.append(dict(case='rar7_dictionary_admission', status='passed'))
    return results


def seven_interop(worker, reference, output):
    inputs = output / 'input'
    inputs.mkdir()
    expected = {'alpha.txt': b'official interoperability\n' * 1000, 'empty': b'',
                'unicode.txt': '\u4e2d\u6587 filename\n'.encode('utf-8') * 100}
    for name, content in expected.items():
        (inputs / name).write_bytes(content)
    results = []
    for solid in (False, True):
        for encrypted in (False, True):
            for headers in ((False, True) if encrypted else (False,)):
                label = f'solid-{int(solid)}-encrypted-{int(encrypted)}-headers-{int(headers)}'
                archive = output / (label + '.7z')
                command = [str(reference), 'a', '-t7z', '-m0=lzma2', '-mx=5', '-ms=' + ('on' if solid else 'off')]
                if encrypted:
                    command += ['-ptest-only', '-mhe=' + ('on' if headers else 'off')]
                command += [str(archive), *expected]
                subprocess.run(command, cwd=inputs, check=True, stdout=subprocess.DEVNULL, timeout=45)
                parts = [archive.read_bytes()]
                rows, _ = run_worker(worker, parts, password='test-only' if encrypted else None)
                assert {row['name'] for row in rows} == set(expected), (label, 'names')
                for row in rows:
                    _, decoded = run_worker(worker, parts, password='test-only' if encrypted else None, selected=row['index'])
                    assert decoded == expected[row['name']], (label, row['name'], 'bytes')
                results.append(dict(case=label, files=len(rows), status='passed'))
                _, encoded = run_worker(worker, [], password='test-only' if encrypted else None,
                                        creation=expected, solid=solid, encrypt_headers=headers)
                native_archive = output / ('native-' + label + '.7z')
                native_archive.write_bytes(encoded)
                for name, content in expected.items():
                    command = [str(reference), 'e', '-so', '-y', str(native_archive), name]
                    if encrypted:
                        command.append('-ptest-only')
                    decoded = subprocess.check_output(command, timeout=45)
                    assert decoded == content, (label, name, 'reference_bytes')
                results.append(dict(case='native-' + label, files=len(expected), status='passed'))
    return results


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--worker', type=Path, required=True)
    parser.add_argument('--sevenzip', type=Path)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    if not output.is_relative_to(ROOT / 'build') or output == ROOT / 'build':
        raise ValueError('Interop output must be a dedicated build/ directory')
    output.mkdir(parents=True, exist_ok=False)
    results = rar_fixtures(args.worker.resolve())
    if args.sevenzip:
        results += seven_interop(args.worker.resolve(), args.sevenzip.resolve(), output)
    (output / 'native-interop.json').write_text(json.dumps(dict(status='passed', cases=results), indent=2) + '\n')
    print(f'Official native interoperability passed: {len(results)} cases')
