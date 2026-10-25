"""Check independent CAB inputs through the Rust codec and an explicit host worker.

Only owned build/ inputs/reports and private temporary extraction roots are used.
The optional preserved official worker is a separate reference. A Rust host run
requires the dedicated build's explicit XDZIP_RUST_CAB_TEST_ROUTE=ON evidence.
"""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import struct
import subprocess
import tempfile
import unicodedata

ROOT = Path(__file__).resolve().parents[1]
MAX_INPUT = 8 * 1024 * 1024
MAX_OUTPUT = 16 * 1024 * 1024
# Debug symbols are part of tool identity, not CAB fixture input allowance.
MAX_BINARY = 128 * 1024 * 1024


def module(path):
    spec = importlib.util.spec_from_file_location(path.stem.replace('-', '_'), path)
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


def require(condition, message):
    if not condition:
        raise AssertionError(message)


def invoke(command, env=None):
    return subprocess.run([str(value) for value in command], capture_output=True, timeout=90, env=env)


def identity(path, limit=MAX_INPUT):
    digest = hashlib.sha256()
    size = 0
    with Path(path).open('rb') as source:
        for block in iter(lambda: source.read(65536), b''):
            size += len(block)
            require(size <= limit, 'input size limit')
            digest.update(block)
    return size, digest.hexdigest()


def normalize(name):
    return unicodedata.normalize('NFC', name.replace('\\', '/'))


def prepare(output):
    legacy = output / 'legacy'
    legacy.mkdir(parents=True, exist_ok=True)
    manifest = json.loads((ROOT / 'tests/fixtures/cab-iso/sources.json').read_text(encoding='utf-8'))
    for item in manifest['fixtures']:
        name = item['file']
        if not name.endswith('.cab'):
            continue
        require(Path(name).name == name and '/' not in name and '\\' not in name, 'legacy CAB name')
        source = ROOT / 'tests/fixtures/cab-iso' / name
        size, sha = identity(source)
        require(sha == item['sha256'], 'legacy CAB identity')
        shutil.copyfile(source, legacy / name)
    cases = []

    def add(name, expected, paths=None, **details):
        cases.append(dict(name=name, paths=[str(path) for path in (paths or [legacy / name])],
                          expected=expected, source='fixed_independent_legacy_CAB', **details))

    add('named.cab', {'hello.txt': b'xdzip read-only archive fixture\n',
                      'nested/\u4e2d\u6587.txt': 'Unicode payload: \u4e2d\u6587\n'.encode(), 'empty.txt': b''})
    add('normal_2files_1folder.cab', {
        'hello.c': b'#include <stdio.h>\r\n\r\nvoid main(void)\r\n{\r\n    printf("Hello, world!\\n");\r\n}\r\n',
        'welcome.c': b'#include <stdio.h>\r\n\r\nvoid main(void)\r\n{\r\n    printf("Welcome!\\n");\r\n}\r\n\r\n'})
    add('mszip_lzx_qtm.cab', {
        'mszip.txt': b'If you can read this, the MSZIP decompressor is working!\n',
        'lzx.txt': b'-' * 65 + b'\nIf you can read this, the LZX decompressor is working!\n' + b'-' * 65 + b'\n',
        'qtm.txt': b'If you can read this, the Quantum decompressor is working!\n'})
    joined = b''.join(f'This is the data from cabinet part {index}.\n'.encode() for index in range(1, 6))
    add('legacy-five-volume', {'test1.txt': joined[:76], 'test2.txt': joined[76:114], 'test3.txt': joined[114:]},
        paths=[legacy / f'cabd_multi_basic_pt{index}.cab' for index in range(1, 6)], physical_fixtures=5)
    makecab = module(ROOT / 'tests/fixtures/cab/generate-makecab.py').prepare(output / 'makecab')
    for case in makecab:
        case['paths'] = [case.pop('path')]
        cases.append(case)
    quantum = output / 'quantum'
    generated = invoke([os.sys.executable, ROOT / 'tests/fixtures/cab/generate-quantum.py', '--output', quantum])
    require(generated.returncode == 0, 'Quantum mathematical fixture generation failed')
    manifest = json.loads((quantum / 'cases.json').read_text(encoding='utf-8'))
    require(len(manifest) == 15, 'Quantum case count')
    for item in manifest:
        base = quantum / item['name']
        path = base / 'input.cab'
        require(0 <= item['bytes'] <= MAX_INPUT and (base / 'known.bin').stat().st_size <= MAX_INPUT,
                'Quantum known-byte limit')
        expected = (base / 'known.bin').read_bytes()
        stored, sha = identity(path)
        require(sha == item['cab_sha256'] and len(expected) == item['bytes']
                and hashlib.sha256(expected).hexdigest() == item['plain_sha256'], 'Quantum fixed identity')
        cases.append(dict(name='quantum-' + item['name'], paths=[str(path)], expected={'known.bin': expected},
                          source='self_authored_mathematical_Quantum_encoder', compression='Quantum',
                          window_bits=item['window_bits'], frames=item['frames'],
                          fixture_allow_high_ratio=len(expected) > 1024 * 1024 and len(expected) > stored * 1000,
                          source_limit='Not authored by an independent third-party Quantum compressor; decoded by the preserved official worker.'))
    require(sum(sum(map(len, case['expected'].values())) for case in cases) <= MAX_OUTPUT,
            'matrix known-output limit')
    return cases


def source_command(binary, paths, guards=False):
    command = [binary]
    for path in paths:
        command.extend(['--volume', path])
    if guards:
        command.append('--guards')
    return command


def codec_case(binary, case):
    result = invoke(source_command(binary, case['paths'], True))
    require(result.returncode == 0, case['name'] + ': Rust scan/decode/guards failed: ' + result.stderr.decode('utf-8', 'replace')[:220])
    report = json.loads(result.stdout)
    require(report['passed'] and report['source_kind'] == 'physical_files', case['name'] + ': source adapter')
    require(report['volume_lengths'] == [identity(path)[0] for path in case['paths']], case['name'] + ': source volume boundaries')
    actual = {}
    skipped = 0
    for row in report['rows']:
        require(row['format'] == 'CAB' and not row['link'] and not row['warning'], case['name'] + ': unsupported record')
        if row['directory']:
            continue
        name = normalize(bytes.fromhex(row['name_hex']).decode('utf-8'))
        data = bytes.fromhex(row['hex'])
        require(name not in actual and len(data) == row['size'], case['name'] + ': physical file identity/size')
        actual[name] = data
        skipped += row['actual_skipped_bytes']
    require(actual == case['expected'], case['name'] + ': Rust known names/bytes mismatch')
    require(report['maximum_read_offset'] <= report['source_length'], case['name'] + ': source read bounds')
    require(all(guard['passed'] for guard in report['guards']), case['name'] + ': guard failure')
    return dict(case=case['name'], passed=True, files=len(actual), source=case['source'],
                source_kind=report['source_kind'], source_bytes=report['source_length'],
                actual_input_bytes=report['bytes_read'], actual_skipped_bytes=skipped,
                guards=report['guards'], **{key: case[key] for key in ('compression', 'window_bits', 'frames', 'source_limit') if key in case})


def worker_case(worker, case, label):
    native = module(ROOT / 'scripts/check-native-codecs.py')
    parts = [Path(path).read_bytes() for path in case['paths']]
    names = [Path(path).name.encode() for path in case['paths']]
    rows, _ = native.run_worker(worker, parts, format_id=8, volume_names=names)
    actual = {}
    for row in rows:
        require(row['flags'] & 12 == 0, case['name'] + ': worker special record')
        if row['flags'] & 1:
            continue
        _, data = native.run_worker(worker, parts, format_id=8, volume_names=names, selected=row['index'])
        actual[normalize(row['name'])] = data
    require(actual == case['expected'], case['name'] + ': ' + label + ' names/bytes mismatch')
    return dict(case=case['name'], passed=True, files=len(actual), backend=label)


def service_case(binary, case, owned, env, opening=0):
    label = case['name'] + ('-open-part' + str(opening + 1) if len(case['paths']) > 1 else '')
    base = owned / label
    base.mkdir()
    copies = []
    for index, source in enumerate(case['paths']):
        target = base / Path(source).name
        shutil.copyfile(source, target)
        copies.append(target)
    before = [identity(path) for path in copies]
    target = base / 'output'
    target.mkdir()
    command = [binary, copies[opening], target, '--generic']
    allow_high_ratio = case.get('fixture_allow_high_ratio', False)
    if allow_high_ratio:
        require(case['source'] == 'self_authored_mathematical_Quantum_encoder', 'high-ratio fixture scope')
        command.append('--fixture-allow-high-ratio')
    result = invoke(command, env)
    require([identity(path) for path in copies] == before, label + ': host changed authorized source')
    require(result.returncode == 0, label + ': service extraction failed: ' + result.stderr.decode('utf-8', 'replace')[:220])
    actual = {normalize(path.relative_to(target).as_posix()): path.read_bytes()
              for path in target.rglob('*') if path.is_file()}
    require(not any(path.is_symlink() for path in target.rglob('*')), label + ': unexpected output link')
    require(actual == case['expected'], label + ': host names/bytes mismatch')
    summary = json.loads(result.stdout)
    require(summary['files'] == len(actual) and summary['bytes'] == sum(map(len, actual.values())), label + ': host counters')
    return dict(case=label, passed=True, files=summary['files'], bytes=summary['bytes'], directories=summary['directories'],
                fixture_allow_high_ratio=allow_high_ratio)


def negative_cases(binary, extract_binary, cases, output, owned, env):
    named = next(case for case in cases if case['name'] == 'named.cab')
    original = Path(named['paths'][0]).read_bytes()
    block = struct.unpack_from('<I', original, 36)[0]
    bad_checksum = bytearray(original)
    struct.pack_into('<I', bad_checksum, block, 0xdeadbeef)
    bad_method = bytearray(original)
    struct.pack_into('<H', bad_method, 42, 15)
    bad_range = bytearray(original)
    file_offset = struct.unpack_from('<I', original, 16)[0]
    struct.pack_into('<I', bad_range, file_offset, 0xffffffff)
    mutations = [('truncated', original[:len(original) // 2], ('invalid_archive',)),
                 ('checksum', bad_checksum, ('integrity_error',)),
                 ('unknown-method', bad_method, ('unsupported_feature',)),
                 ('file-range', bad_range, ('invalid_archive',))]
    results = []
    for label, data, errors in mutations:
        path = output / ('bad-' + label + '.cab')
        path.write_bytes(data)
        result = invoke(source_command(binary, [path]))
        require(result.returncode != 0 and any(code.encode() in result.stderr for code in errors), label + ': Rust bad-input classification')
        if extract_binary:
            base = owned / ('bad-' + label)
            base.mkdir()
            source = base / path.name
            source.write_bytes(data)
            target = base / 'output'
            target.mkdir()
            host = invoke([extract_binary, source, target, '--generic'], env)
            require(host.returncode != 0 and any(code.encode() in host.stderr for code in errors), label + ': host bad-input classification')
            require(not list(target.iterdir()) and source.read_bytes() == data, label + ': failed host task published or changed input')
        results.append(dict(case=label, passed=True, codec_checked=True, host_service_checked=bool(extract_binary)))
    split = next(case for case in cases if case['name'] == 'legacy-five-volume')
    data = [Path(path).read_bytes() for path in split['paths']]
    wrong_set = bytearray(data[2]); wrong_set[32:34] = b'\x01\x00'
    for label, parts, names in [('missing-volume', data[:-1], [Path(path).name for path in split['paths'][:-1]]),
                                ('wrong-set', [*data[:2], wrong_set, *data[3:]], [Path(path).name for path in split['paths']]),
                                ('unauthorized-name', data, [Path(path).name if index != 2 else 'other.cab' for index, path in enumerate(split['paths'])])]:
        base = output / label
        base.mkdir(exist_ok=True)
        paths = []
        for name, part in zip(names, parts):
            path = base / name; path.write_bytes(part); paths.append(path)
        result = invoke(source_command(binary, paths))
        require(result.returncode != 0 and any(code in result.stderr for code in (b'invalid_archive', b'missing_volume')), label + ': incomplete set admitted')
        if extract_binary:
            target = owned / (label + '-output'); target.mkdir()
            # Copy to the private service-owned root so only self-owned data is authorized.
            group = owned / label; group.mkdir()
            for path in paths: shutil.copyfile(path, group / path.name)
            host = invoke([extract_binary, group / paths[0].name, target, '--generic'], env)
            require(host.returncode != 0 and not list(target.iterdir()), label + ': host published incomplete set')
        results.append(dict(case=label, passed=True, codec_checked=True, host_service_checked=bool(extract_binary)))
    if extract_binary:
        high_ratio = next(case for case in cases if case.get('fixture_allow_high_ratio', False))
        base = owned / 'default-high-ratio'; base.mkdir()
        source = base / Path(high_ratio['paths'][0]).name
        shutil.copyfile(high_ratio['paths'][0], source)
        before = identity(source)
        target = base / 'output'; target.mkdir()
        host = invoke([extract_binary, source, target, '--generic'], env)
        require(host.returncode != 0 and b'high_ratio' in host.stderr, 'default high-ratio policy was bypassed')
        require(not list(target.iterdir()) and identity(source) == before,
                'default high-ratio failure published or changed source')
        results.append(dict(case='default-high-ratio', fixture=high_ratio['name'], passed=True,
                            codec_checked=False, host_service_checked=True,
                            fixture_allow_high_ratio=False, expected_error='high_ratio'))
    return results


def worker_identity(worker, kind, cache):
    info = dict(sha256=identity(worker, MAX_BINARY)[1], filename=worker.name, backend=kind)
    if kind == 'rust-cab-test-route':
        require(cache is not None and cache.is_file(), 'Rust worker requires its --worker-build-cache')
        text = cache.read_text(encoding='utf-8')
        require('XDZIP_RUST_CAB_TEST_ROUTE:BOOL=ON' in text.splitlines(), 'worker test-route flag is not ON')
        # Require the cache and executable to belong to the same build tree.
        require(worker.is_relative_to(cache.parent), 'worker/cache build-tree mismatch')
        info['test_route'] = 'XDZIP_RUST_CAB_TEST_ROUTE:BOOL=ON'
        info['build_cache_sha256'] = hashlib.sha256(cache.read_bytes()).hexdigest()
    return info


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, help='xdzip-cab-probe, direct Rust only')
    parser.add_argument('--extract-binary', type=Path, help='actual xdzip-m3-probe service')
    parser.add_argument('--worker', type=Path, help='explicit tested host worker, never inferred from PATH')
    parser.add_argument('--worker-kind', choices=['rust-cab-test-route', 'official-cab-baseline'])
    parser.add_argument('--worker-build-cache', type=Path)
    parser.add_argument('--reference-worker', type=Path, help='optional preserved official CAB worker')
    parser.add_argument('--output', type=Path, default=ROOT / 'build/review/cab-rust-interop')
    parser.add_argument('--prepare-only', action='store_true')
    args = parser.parse_args()
    output = args.output.resolve()
    require(output.is_relative_to((ROOT / 'build').resolve()) and output != (ROOT / 'build').resolve(), 'outputs belong below build/')
    output.mkdir(parents=True, exist_ok=True)
    cases = prepare(output)
    if args.prepare_only:
        print('Prepared %d owned CAB cases; no codec or service checks were run' % len(cases))
        return
    require(args.binary is not None, '--binary is required')
    binary = args.binary.resolve()
    worker = args.worker.resolve() if args.worker else None
    extract = args.extract_binary.resolve() if args.extract_binary else None
    reference = args.reference_worker.resolve() if args.reference_worker else None
    report = dict(passed=False, codec=[], host_worker=[], host_service=[], reference=[], negative=[],
                  probe_sha256=identity(binary, MAX_BINARY)[1],
                  native_transport_sha256=identity(ROOT / 'scripts/check-native-codecs.py')[1],
                  fixture_manifests={},
                  limits=['Desktop/Qt acceptance is separate.',
                          'Per-case service records are real m3 extraction, not a claim that all host guards ran per case.',
                          'Codec guards check small budgets, source-I/O, cancellation and read-only APIs. Host cancellation/source-change/readonly task regressions are recorded separately by the runtime test owner.',
                          'Quantum mathematical encoder samples are independent-reader validated; they are not third-party-compressor samples.'])
    for name in ('cab-iso/sources.json', 'cab/lzx-sources.json', 'cab/mszip-sources.json', 'cab/quantum-sources.json'):
        report['fixture_manifests'][name] = hashlib.sha256((ROOT / 'tests/fixtures' / name).read_bytes()).hexdigest()
    env = os.environ.copy()
    if worker:
        require(args.worker_kind is not None, '--worker requires --worker-kind')
        report['host_worker_identity'] = worker_identity(worker, args.worker_kind, args.worker_build_cache.resolve() if args.worker_build_cache else None)
        # NativeCodec gives this legacy override priority. The explicit CLI
        # worker must be the actual service backend whose identity we report.
        report['inherited_worker_override_cleared'] = 'XDZIP_NATIVE_CODEC_WORKER' in env
        env.pop('XDZIP_NATIVE_CODEC_WORKER', None)
        env['XDZIP_NATIVE_CODEC'] = str(worker)
    if reference:
        report['reference_worker_sha256'] = identity(reference, MAX_BINARY)[1]
        require(not worker or report['host_worker_identity']['sha256'] != report['reference_worker_sha256'], 'reference and host worker must differ')
    if extract:
        require(worker is not None, 'service extraction requires explicit --worker')
        report['m3_probe_sha256'] = identity(extract, MAX_BINARY)[1]
        base = invoke([extract, '--native-temp-base'], env)
        require(base.returncode == 0, 'host temporary base query failed')
        native_base = Path(base.stdout.decode().strip())
    else:
        native_base = output / 'scratch'; native_base.mkdir(exist_ok=True)
        report['limits'].append('Actual host service extraction was not run.')
    try:
        with tempfile.TemporaryDirectory(prefix='xdzip-cab-interop-', dir=native_base) as temporary:
            owned = Path(temporary)
            for case in cases:
                report['codec'].append(codec_case(binary, case))
                if worker:
                    report['host_worker'].append(worker_case(worker, case, args.worker_kind))
                if reference:
                    report['reference'].append(worker_case(reference, case, 'preserved_official_CAB_worker'))
                if extract:
                    for opening in range(len(case['paths'])):
                        report['host_service'].append(service_case(extract, case, owned, env, opening))
            report['negative'] = negative_cases(binary, extract, cases, output, owned, env)
        report['passed'] = True
        report['rust_host_service_checked'] = bool(extract and args.worker_kind == 'rust-cab-test-route')
        print('CAB interoperability passed: %d Rust codec, %d host worker, %d real service, %d official reference, %d negative cases'
              % tuple(len(report[key]) for key in ('codec', 'host_worker', 'host_service', 'reference', 'negative')))
    except Exception as error:
        report['failure'] = str(error) if isinstance(error, AssertionError) else type(error).__name__
        raise
    finally:
        (output / 'cab-rust-interop.json').write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')


if __name__ == '__main__':
    main()
