"""Run real RAR worker/runtime and adversarial protocol checks with fixed data."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import platform
import re
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]


def file_digest(path):
    digest = hashlib.sha256()
    with path.open('rb') as stream:
        while block := stream.read(1024 * 1024):
            digest.update(block)
    return digest.hexdigest()


def bridge_inputs(worker):
    """Use only the built worker's fixed CMake Rust output layout."""
    rust = worker.parent / 'rust'
    include = rust / 'cxxbridge'
    headers = [include / 'xdzip-codec-bridge/src/lib.rs.h', include / 'rust/cxx.h']
    if not all(path.is_file() for path in headers):
        raise ValueError('built worker CXX headers are missing')
    archives = [path for name in ['xdzip_codec_bridge.lib', 'libxdzip_codec_bridge.a']
                for path in rust.glob('*/release/' + name)]
    if len(archives) != 1 or not archives[0].is_file():
        raise ValueError('built worker must have exactly one release CXX archive')
    return include, archives[0], headers


def module(name, file):
    spec = importlib.util.spec_from_file_location(name, ROOT / 'scripts' / file)
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--worker', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--archives', type=Path)
    parser.add_argument('--offline', action='store_true')
    parser.add_argument('--target-dir', type=Path)
    parser.add_argument('--runtime-filter', default='native_backend_tests',
                        help='explicitly narrower local scope; CI uses the complete default batch')
    parser.add_argument('--large-digest', action='store_true',
                        help='also hash the complete 2 GiB stream through the real worker')
    args = parser.parse_args()
    prepare = module('rar_fixture_preparer', 'prepare-rar-rust-fixtures.py')
    output = prepare.build_output(args.output)
    output.mkdir(parents=True, exist_ok=True)
    worker = args.worker.resolve(strict=True)
    report = dict(schema_version=1, status='running', platform=sys.platform,
                  machine=platform.machine(), worker_sha256=hashlib.sha256(worker.read_bytes()).hexdigest(),
                  runtime_filter=args.runtime_filter, checks=[],
                  scope='Real worker/Native/runtime plus hostile fake-worker protocol. No author executable or large plaintext is created.',
                  limits=['Desktop/manual acceptance remains separate.',
                          'The component large fixture wrapper runs separately; this batch checks it once through runtime.'])
    environment = dict(os.environ, PYTHONUTF8='1', CARGO_INCREMENTAL='0')
    if args.target_dir:
        environment['CARGO_TARGET_DIR'] = str(args.target_dir.resolve())

    def save():
        (output / 'report.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')

    def run(label, command, env=environment, timeout=300, require_tests=False):
        started = time.monotonic()
        completed = subprocess.run(command, cwd=ROOT, env=env, capture_output=True,
                                   text=True, encoding='utf-8', errors='replace', timeout=timeout)
        log = completed.stdout + '\n' + completed.stderr
        for value, replacement in [(str(ROOT), '<workspace>'), (str(Path.home()), '<home>')]:
            log = log.replace(value, replacement).replace(value.replace('\\', '/'), replacement)
        (output / (label + '.log')).write_text(log, encoding='utf-8')
        if completed.returncode:
            raise RuntimeError(label + ' failed; see build log')
        passed = sum(map(int, re.findall(r'test result: ok\. (\d+) passed;', log)))
        if require_tests and not passed:
            raise ValueError(label + ' ran no tests')
        report['checks'].append(dict(check=label, status='passed', tests_passed=passed,
                                     elapsed_seconds=time.monotonic() - started))
        save()

    def test(label, package, selection, env=environment):
        run(label, ['cargo', 'test', '-p', package, '--locked', selection, '--',
                    '--ignored', '--nocapture', '--test-threads=1'], env, timeout=600, require_tests=True)

    save()
    try:
        fixed, sources = prepare.prepare(output / 'inputs', args.archives, args.offline)
        environment.update(fixed)
        # This is the public upstream fixture's documented mathematical test key.
        # It is passed only in the child environment, never argv or report/logs.
        environment['XDZIP_RAR_FIXTURE_PASSWORD'] = 'password'
        environment['XDZIP_NATIVE_CODEC_WORKER'] = str(worker)
        environment['XDZIP_NATIVE_CODEC'] = str(worker)
        report['fixture_sources'] = sources
        include, library, headers = bridge_inputs(worker)
        report['os_probe_inputs'] = dict(
            archive=dict(relative_to_worker=str(library.relative_to(worker.parent)),
                         sha256=file_digest(library)),
            headers=[dict(relative_to_include=str(path.relative_to(include)), sha256=file_digest(path))
                     for path in headers])
        stub_build = output / 'protocol-worker'
        run('protocol-configure', ['cmake', '-S', 'crates/xdzip-codec-native/tests', '-B',
                                  str(stub_build), '-G', 'Ninja', '-DCMAKE_BUILD_TYPE=Release',
                                  '-DXDZIP_CODEC_BRIDGE_INCLUDE=' + str(include),
                                  '-DXDZIP_CODEC_BRIDGE_ARCHIVE=' + str(library)])
        run('protocol-build', ['cmake', '--build', str(stub_build), '--target',
                              'xdzip-auxiliary-protocol-worker', 'xdzip-native-codepage-probe',
                              '--parallel', '2'])
        stub = stub_build / ('xdzip-auxiliary-protocol-worker.exe' if os.name == 'nt' else 'xdzip-auxiliary-protocol-worker')
        probe = stub_build / ('xdzip-native-codepage-probe.exe' if os.name == 'nt' else 'xdzip-native-codepage-probe')
        environment['XDZIP_NATIVE_CODEPAGE_PROBE'] = str(probe)
        report['os_probe_sha256'] = file_digest(probe)
        fake = dict(environment, XDZIP_NATIVE_CODEC_WORKER=str(stub), XDZIP_AUXILIARY_PROTOCOL_FIXTURE='1')
        test('auxiliary-protocol', 'xdzip-codec-native', 'auxiliary_protocol', fake)
        test('unknown-protocol', 'xdzip-codec-native', 'unknown_size_protocol', fake)
        test('os-codepage-native', 'xdzip-codec-native', 'native_os_codepage_services')
        for label, selection in [('ppm-native', 'rust_rar_ppm_auxiliary'),
                                 ('unknown-native', 'rust_rar_unknown_metadata'),
                                 ('legacy20-native', 'rust_rar20_solid_audio')]:
            test(label, 'xdzip-codec-native', selection)
        # Preserve the original complete native_backend_tests batch, now with all
        # required data. A narrower local run is explicitly recorded above.
        test('runtime-native-batch', 'xdzip-runtime', args.runtime_filter)
        native = module('rar_native_helper', 'check-native-codecs.py')
        tiny = ROOT / 'crates/xdzip-codec-rar/tests/fixtures/filter-author/rar29-e8e9-auto.rar'
        archive = tiny.read_bytes()
        if hashlib.sha256(archive).hexdigest() != 'df693c44b99e9eeebd261f1cb55760047efe5add584c22f050bfb58258ef842a':
            raise ValueError('tiny E8E9 archive identity')
        _, actual = native.run_worker(worker, [archive], format_id=4, selected=0, digest_only=True)
        if (actual['bytes'], actual['sha256']) != (2048, '6de2e3e6e612b27a70138e34c512a2799c16f709a663583d5d50c36fe3f884f3'):
            raise ValueError('tiny E8E9 complete original mismatch')
        report['tiny_e8e9'] = dict(status='passed', archive_sha256=hashlib.sha256(archive).hexdigest(), **actual)
        if args.large_digest:
            large = Path(fixed['XDZIP_RAR26_FIXTURE']).read_bytes()
            _, actual = native.run_worker(worker, [large], format_id=4, selected=0, digest_only=True)
            if (actual['bytes'], actual['sha256']) != (2147483649, 'd1ea9e8a1dd77c140253106db1be9331430622ee141e57a1ccdf31bab1d5a13a'):
                raise ValueError('large26 complete original mismatch')
            report['large26_full_digest'] = dict(status='passed', **actual)
        report['status'] = 'passed'
        save()
    except Exception as error:
        report.update(status='failed', error=type(error).__name__ + ': ' + str(error))
        save()
        raise RuntimeError('RAR interop failed; see build report') from None
    print(json.dumps(dict(status='passed', checks=len(report['checks']),
                          tests_passed=sum(item['tests_passed'] for item in report['checks']),
                          worker_sha256=report['worker_sha256'])))


if __name__ == '__main__':
    main()
