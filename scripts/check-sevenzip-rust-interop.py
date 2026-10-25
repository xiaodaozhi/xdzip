"""Check owned all-method 7z fixtures through the actual worker and host service."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
FIXTURES = ROOT / 'crates/xdzip-codec-7z/tests/fixtures'


def module(name, filename):
    spec = importlib.util.spec_from_file_location(name, ROOT / 'scripts' / filename)
    value = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(value)
    return value


def require(condition, message):
    if not condition:
        raise ValueError(message)


def known_payloads():
    return {'empty.txt': b'', 'hello.txt': b'independent 7z preparation\n',
            'payload.bin': bytes(range(256)) * 32
            + b'\xe8\x01\0\0\0\xe9\xf0\xff\xff\xff' * 128
            + b'\0\0\0\xeb' * 128 + b'end'}


def worker_case(native, worker, data, password, name, expected):
    rows, _ = native.run_worker(worker, [data], password=password)
    require(len(rows) == len(expected), name + ': physical record count')
    actual = {}
    for row in rows:
        require(row['flags'] & 13 == 0 and row['raw_name'] == row['name'].encode('utf-8'), name + ': ordinary UTF-8 record')
        require(row['name'] not in actual, name + ': duplicate fixture record')
        _, actual[row['name']] = native.run_worker(worker, [data], password=password, selected=row['index'])
    require(actual == expected, name + ': worker full-byte mismatch')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--worker', type=Path, required=True)
    parser.add_argument('--worker-build-cache', type=Path, required=True)
    parser.add_argument('--extract-binary', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    require(output.is_relative_to(ROOT / 'build') and output != ROOT / 'build', 'output below build/')
    output.parent.mkdir(parents=True, exist_ok=True)
    worker, probe = args.worker.resolve(), args.extract_binary.resolve()
    manifest = json.loads((FIXTURES / 'sources.json').read_text(encoding='utf-8'))
    expected = known_payloads()
    require(len(manifest['fixtures']) == 24, 'all 24 profiles required')
    for name, data in expected.items():
        require(manifest['known_payloads'][name] == dict(bytes=len(data), sha256=hashlib.sha256(data).hexdigest()),
                'independent input identity')
    native = module('sevenzip_native_transport', 'check-native-codecs.py')
    cab = module('sevenzip_worker_identity', 'check-cab-rust-interop.py')
    # Both formats share one fixed worker. Reuse its complete source-closure check.
    identity = cab.worker_identity(worker, 'rust-cab', args.worker_build_cache.resolve())
    env = os.environ.copy()
    env.pop('XDZIP_NATIVE_CODEC_WORKER', None)
    env['XDZIP_NATIVE_CODEC'] = str(worker)
    base = subprocess.run([str(probe), '--native-temp-base'], env=env, capture_output=True, check=True, timeout=30)
    report = dict(passed=False, product_worker_identity=identity, host_probe_sha256=hashlib.sha256(probe.read_bytes()).hexdigest(),
                  fixture_manifest_sha256=hashlib.sha256((FIXTURES / 'sources.json').read_bytes()).hexdigest(),
                  cases=[], limits=['Owned small independent-author fixtures, not exhaustive algorithm conformance.',
                                    'Direct codec coverage runs as an ordinary Rust test; manual desktop acceptance is separate.'])
    try:
        retry_root = FIXTURES / 'password'
        retry = json.loads((retry_root / 'sources.json').read_text(encoding='utf-8'))
        retry_data = (retry_root / 'copy-aes.7z').read_bytes()
        require(retry['fixture'] == dict(file='copy-aes.7z', bytes=len(retry_data), sha256=hashlib.sha256(retry_data).hexdigest()),
                'retry fixture identity')
        retry_expected = {'first.bin': bytes((17 * index + index // 251) % 256 for index in range(16391)),
                          'nested/second.bin': (b'solid-metadata-known\0' * 4682)[:98321], 'zero.bin': b''}
        for name, data in retry_expected.items():
            require(retry['known_input'][name]['bytes'] == len(data)
                    and retry['known_input'][name]['sha256'] == hashlib.sha256(data).hexdigest(), 'retry original input identity')
        worker_case(native, worker, retry_data, retry['synthetic_password'], 'password-retry-Copy-AES', retry_expected)
        report['password_retry_fixture'] = dict(worker_full_bytes=True, independent_original_input=True, files=3,
                                                bytes=sum(map(len, retry_expected.values())),
                                                manifest_sha256=hashlib.sha256((retry_root / 'sources.json').read_bytes()).hexdigest())
        with tempfile.TemporaryDirectory(prefix='xdzip-sevenzip-interop-', dir=Path(base.stdout.decode().strip())) as temporary:
            owned = Path(temporary)
            for item in manifest['fixtures']:
                name = item['file']
                require(Path(name).name == name and name.endswith('.7z'), 'fixture path')
                data = (FIXTURES / name).read_bytes()
                require(len(data) == item['bytes'] and hashlib.sha256(data).hexdigest() == item['sha256'], 'fixture identity')
                password = item['synthetic_password']
                worker_case(native, worker, data, password, name, expected)
                folder = owned / name.removesuffix('.7z')
                folder.mkdir()
                source, target = folder / name, folder / 'output'
                shutil.copyfile(FIXTURES / name, source)
                target.mkdir()
                command = [str(probe), str(source), str(target), '--generic']
                if password is not None:
                    command.append('--fixture-password-stdin')
                result = subprocess.run(command, input=password.encode() if password is not None else b'',
                                        env=env, capture_output=True, timeout=90)
                require(result.returncode == 0, name + ': host service failed: ' + result.stderr.decode('utf-8', 'replace')[:220])
                require(not any(path.is_symlink() for path in target.rglob('*')), name + ': unexpected output link')
                actual = {path.relative_to(target).as_posix(): path.read_bytes() for path in target.rglob('*') if path.is_file()}
                require(actual == expected and source.read_bytes() == data, name + ': published bytes or source identity')
                summary = json.loads(result.stdout)
                require(summary['files'] == 3 and summary['bytes'] == sum(map(len, expected.values())), name + ': host counters')
                header_encrypted = name.startswith('header-encrypted')
                require(summary['open_password_retries'] == int(header_encrypted), name + ': metadata password retry')
                report['cases'].append(dict(case=name, passed=True, worker_full_bytes=True, host_service_full_bytes=True,
                                            independent_original_input=True, open_password_retries=summary['open_password_retries'],
                                            files=3, bytes=summary['bytes']))
        report['passed'] = True
    finally:
        output.write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print('Passed 24 7z all-method worker and host-service cases')


if __name__ == '__main__':
    main()
