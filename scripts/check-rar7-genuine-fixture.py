"""Fetch pinned build-only RAR7 data and run its complete genuine algorithm1 release decode."""
import argparse
import hashlib
import io
import json
import os
from pathlib import Path
import platform
import re
import subprocess
import sys
import time
import unittest
import urllib.request
import uuid

ROOT = Path(__file__).resolve().parents[1]
COMMIT = 'c831d1ed9eccf63083f2d4304149019d43f48c06'
REPOSITORY = 'https://github.com/junrar/junrar'
BASE = f'https://raw.githubusercontent.com/junrar/junrar/{COMMIT}/'
RESPONSE_LIMIT = 256 * 1024
SIZE = 4563402752
DICTIONARY = 6442450944
ORIGINAL_SHA = '3c404d434bf9533da6f4b3f1b813094c81fe71c39e878b9d376e8eb54bb84b2c'
ORIGINAL_CRC = '8e9b8bf4'
ARCHIVE_SHA = 'd915d38bd81ae0eeb8a72f0a6b31fb0474a53c5bb85982752e3b12569421416d'
FILES = (
    dict(file='rar7-md6g.rar', path='src/test/resources/com/github/junrar/rar7/rar7-md6g.rar',
         bytes=183863, sha256=ARCHIVE_SHA, git_blob_sha1='7f2616faba756cb3efb458ed51af67385d706379'),
    dict(file='LICENSE', path='LICENSE', bytes=1861,
         sha256='9d90c3649d4eb064ce2fff366a7421107fbbc3c65d8eede45df5dec95e4e016c',
         git_blob_sha1='ec35e09a8fbbf8e8acb3eeba7794b960abcc4e4f'),
)
NOTICE = ('Public archive testdata and the unmodified upstream top-level UnRAR notice are retained '
          'only under build/CI. No separate fixture-license exception or redistribution approval is '
          'inferred. No third-party decoder source, README, author executable or original plaintext '
          'is downloaded, vendored or shipped by this check.')
RESULT = re.compile(
    r'\bRAR7_GENUINE_RESULT\t(\d+)\t([0-9a-f]{64})\t([0-9a-f]{8})\t(\d+)\t(\d+)'
    r'\t(\d+)\t(\d+)\t(\d+)\t(\d+)\t(\d+)\t([0-9a-f]{64})'
    r'\tall_zero\tsize_crc_verified$', re.MULTILINE)


def validate(data, item):
    if len(data) != item['bytes'] or len(data) > RESPONSE_LIMIT:
        raise ValueError('pinned fixture response length mismatch')
    if hashlib.sha256(data).hexdigest() != item['sha256']:
        raise ValueError('pinned fixture SHA256 mismatch')
    blob = b'blob ' + str(len(data)).encode('ascii') + b'\0' + data
    if hashlib.sha1(blob).hexdigest() != item['git_blob_sha1']:
        raise ValueError('pinned fixture Git blob mismatch')


def bounded_read(response, item):
    declared = response.headers.get('Content-Length')
    if declared is not None and int(declared) != item['bytes']:
        raise ValueError('pinned response Content-Length mismatch')
    data = bytearray()
    while True:
        chunk = response.read(min(65536, RESPONSE_LIMIT + 1 - len(data)))
        if not chunk:
            break
        data.extend(chunk)
        if len(data) > RESPONSE_LIMIT or len(data) > item['bytes']:
            raise ValueError('pinned fixture response exceeds byte cap')
    result = bytes(data)
    validate(result, item)
    return result


def output_root(path):
    resolved = path.resolve()
    if not resolved.is_relative_to((ROOT / 'build').resolve()) or resolved == (ROOT / 'build').resolve():
        raise ValueError('output must stay below workspace build/')
    return resolved


def fixed_file(path, data):
    if path.is_symlink():
        raise ValueError('build fixture destination must not be a symlink')
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_name(path.name + '.' + uuid.uuid4().hex + '.tmp')
    try:
        with temporary.open('xb') as stream:
            stream.write(data)
        os.replace(temporary, path)
    finally:
        temporary.unlink(missing_ok=True)


def prepare(output, cache=None, offline=False):
    output = output_root(output)
    output.mkdir(parents=True, exist_ok=True)
    inputs = output / 'inputs'
    if inputs.is_symlink():
        raise ValueError('build fixture input directory must not be a symlink')
    records = []
    for item in FILES:
        candidates = [inputs / item['file']]
        if cache is not None:
            candidates.append(cache / item['file'])
        existing = next((path for path in candidates if path.is_file()), None)
        downloaded = 0
        if existing is not None:
            if existing.stat().st_size != item['bytes']:
                raise ValueError('cached fixture length mismatch')
            with existing.open('rb') as stream:
                data = stream.read(RESPONSE_LIMIT + 1)
            validate(data, item)
        else:
            if offline:
                raise ValueError('offline pinned fixture or notice is absent')
            request = urllib.request.Request(BASE + item['path'], headers={'User-Agent': 'xdzip-fixture-check'})
            with urllib.request.urlopen(request, timeout=30) as response:
                if response.status != 200 or response.geturl() != BASE + item['path']:
                    raise ValueError('unexpected pinned fixture response or redirect')
                data = bounded_read(response, item)
            downloaded = len(data)
        fixed_file(inputs / item['file'], data)
        records.append(dict(item, url=BASE + item['path'], downloaded_body_bytes=downloaded))
    provenance = dict(schema_version=1, repository=REPOSITORY, commit=COMMIT,
                      files=records, notice=NOTICE, network_per_response_limit=RESPONSE_LIMIT,
                      downloaded_body_bytes=sum(item['downloaded_body_bytes'] for item in records),
                      original=dict(bytes=SIZE, formula='Every byte is zero.', sha256=ORIGINAL_SHA,
                                    crc32=ORIGINAL_CRC, meaning='Independent known-input oracle, not an observed decode.'),
                      no_large_plaintext_created=True)
    fixed_file(output / 'sources.json', (json.dumps(provenance, indent=2) + '\n').encode('utf-8'))
    return inputs / FILES[0]['file'], provenance


def checked_result(log):
    matches = RESULT.findall(log)
    if len(matches) != 1 or 'test result: ok. 1 passed; 0 failed;' not in log:
        raise ValueError('exactly one successful complete release decode result required')
    count, original_sha, crc, work, auxiliary, chunk, credit, logical, allocated, raw, archive_sha = matches[0]
    if (int(count), original_sha, crc, int(work), int(credit), int(logical), archive_sha) != (
            SIZE, ORIGINAL_SHA, ORIGINAL_CRC, SIZE, 0, DICTIONARY, ARCHIVE_SHA):
        raise ValueError('complete original/count/SHA/CRC/work/dictionary identity mismatch')
    auxiliary, allocated, raw, chunk = int(auxiliary), int(allocated), int(raw), int(chunk)
    if not (0 < auxiliary <= 20 * 1024**3 - SIZE and 0 < chunk <= 8192
            and 0 < allocated <= 512 * 1024**2 and 0 <= raw <= allocated):
        raise ValueError('auxiliary/chunk/paged memory admission mismatch')
    return dict(decoded_bytes=SIZE, original_sha256=original_sha, original_crc32=crc,
                output_work_bytes=SIZE, auxiliary_work=auxiliary, maximum_output_chunk=chunk,
                remaining_prepaid_credit=0, full_zero_byte_comparison=True, full_sha_verified=True,
                full_crc_verified=True, final_container_integrity_verified=True,
                dictionary_bytes=int(logical), paged_peak_allocated_with_allowance=allocated,
                paged_peak_raw_capacity=raw, paged_memory_limit=512 * 1024**2,
                memory_semantics='Instance retained capacities are monotonic and include its conservative allocator allowance; not process RSS.')


def sanitized(text):
    for value, replacement in ((str(ROOT), '<workspace>'), (str(Path.home()), '<home>')):
        text = text.replace(value, replacement).replace(value.replace('\\', '/'), replacement)
    return text


def single_test_result(log, expression):
    matches = re.findall(expression, log, re.MULTILINE)
    if len(matches) != 1 or 'test result: ok. 1 passed; 0 failed;' not in log:
        raise ValueError('exactly one successful integration result required')
    return tuple(map(int, matches[0]))


def checked_native_result(log):
    values = single_test_result(
        log, r'RAR7_NATIVE_RESULT\t(\d+)\t(\d+)\t(\d+)\t(\d+)\t(\d+)\t(\d+)'
        r'\tall_zero\tsize_crc_verified$')
    count, work, auxiliary, credit, chunk, memory = values
    if (count, work, credit, memory) != (SIZE, SIZE, 0, 1024**3):
        raise ValueError('Native size/work/credit/memory mismatch')
    if not (0 < auxiliary <= 20 * 1024**3 and 0 < chunk <= 8192):
        raise ValueError('Native auxiliary/chunk mismatch')
    return dict(decoded_bytes=count, output_work_bytes=work, auxiliary_work=auxiliary,
                remaining_prepaid_credit=credit, maximum_output_chunk=chunk,
                enforced_worker_memory_limit_bytes=memory,
                full_zero_byte_comparison=True, final_container_integrity_verified=True,
                no_large_plaintext_created=True)


def checked_runtime_result(log):
    values = single_test_result(
        log, r'RAR7_RUNTIME_RESULT\t(\d+)\t(\d+)\t(\d+)\t(\d+)\t(\d+)\t(\d+)\tno_destination$')
    files, verified, decoded, read, written, committed = values
    if (files, verified, decoded, written, committed) != (1, SIZE, SIZE, 0, 0) or read == 0:
        raise ValueError('runtime full-check counters mismatch')
    return dict(verified_files=files, verified_bytes=verified, decoded_bytes=decoded,
                read_bytes=read, written_bytes=written, committed_bytes=committed,
                no_destination=True, no_large_plaintext_created=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path)
    parser.add_argument('--cache', type=Path, help='optional exact-data cache; no archive or notice identity overrides')
    parser.add_argument('--offline', action='store_true')
    parser.add_argument('--prepare-only', action='store_true', help='prepare evidence only; does not claim a decode passed')
    parser.add_argument('--self-test', action='store_true')
    parser.add_argument('--worker', type=Path, help='also execute the real Native and runtime full-stream checks')
    parser.add_argument('--integration-only', action='store_true',
                        help='run only worker/runtime checks; does not claim a component SHA check')
    args = parser.parse_args()
    if args.self_test:
        unittest.main(argv=[sys.argv[0]], exit=False).result.wasSuccessful() or sys.exit(1)
        return
    if args.output is None:
        parser.error('--output is required')
    if args.integration_only and (not args.worker or args.prepare_only):
        parser.error('--integration-only requires --worker and cannot prepare only')
    output = output_root(args.output)
    output.mkdir(parents=True, exist_ok=True)
    report = dict(schema_version=1, status='running', platform=sys.platform, machine=platform.machine(),
                  case='public-official723-genuine-algorithm1-6g', archive_sha256=ARCHIVE_SHA,
                  archive_bytes=183863, original_bytes=SIZE, original_sha256=ORIGINAL_SHA,
                  original_crc32=ORIGINAL_CRC, dictionary_bytes=DICTIONARY,
                  declared_algorithm=1, effective_algorithm=1, compatibility_bit=False, compression_method=1,
                  notice=NOTICE, no_large_plaintext_created=True,
                  scope=dict(component=not args.integration_only, native_and_runtime=bool(args.worker)),
                  checks=[], explicit_tests_passed=0,
                  limits=['No official decoder is executed by this component wrapper; the published creator oracle is separate.',
                          'Component checks hash and CRC every byte; Native compares every byte to the zero recipe, while runtime checks final container integrity and counters.',
                          'Desktop/manual acceptance remains separate.',
                          'One actual1 stream does not establish use of every extended distance symbol or encrypted/solid/split profiles.',
                          'The archive and upstream notice remain build/CI-only; no redistribution or legal clearance is inferred.'])

    def save():
        fixed_file(output / 'report.json', (json.dumps(report, indent=2) + '\n').encode('utf-8'))

    save()
    try:
        fixture, provenance = prepare(output, args.cache, args.offline)
        report['public_source'] = provenance
        if args.prepare_only:
            report.update(status='prepared', explicit_tests_passed=0)
            save()
            print(json.dumps(dict(status='prepared', explicit_tests_passed=0)))
            return
        environment = dict(os.environ, CARGO_INCREMENTAL='0', XDZIP_RAR7_GENUINE_FIXTURE=str(fixture))
        if args.worker:
            worker = args.worker.resolve(strict=True)
            environment.update(XDZIP_NATIVE_CODEC=str(worker), XDZIP_NATIVE_CODEC_WORKER=str(worker))
            report['worker_sha256'] = hashlib.sha256(worker.read_bytes()).hexdigest()
        plan = []
        if not args.integration_only:
            plan.append(('component', 'xdzip-codec-rar',
                         'rar7_genuine_tests::public_official_723_algorithm1_full_stream', checked_result))
        if args.worker:
            plan.extend([
                ('native', 'xdzip-codec-native',
                 'rust_rar7_genuine_large_dictionary_full_stream_stays_bounded', checked_native_result),
                ('runtime', 'xdzip-runtime',
                 'rar7_genuine_tests::full_check_keeps_memory_and_u64_counters', checked_runtime_result),
            ])
        for label, package, selection, validate_result in plan:
            command = ['cargo', 'test', '-p', package, '--locked', '--release', selection,
                       '--', '--ignored', '--nocapture', '--test-threads=1']
            started = time.monotonic()
            completed = subprocess.run(command, cwd=ROOT, env=environment, capture_output=True,
                                       text=True, encoding='utf-8', errors='replace', timeout=600)
            log = sanitized(completed.stdout + '\n' + completed.stderr)
            fixed_file(output / (label + '.log'), log.encode('utf-8'))
            if completed.returncode:
                raise RuntimeError(label + ' release test failed; see local build log')
            observed = validate_result(log)
            report['checks'].append(dict(check=label, status='passed', tests_passed=1,
                                         elapsed_seconds=time.monotonic() - started, **observed))
            report['explicit_tests_passed'] += 1
            save()
        report['status'] = 'passed'
        save()
    except Exception as error:
        report.update(status='failed', error=sanitized(type(error).__name__ + ': ' + str(error)))
        save()
        raise RuntimeError('genuine algorithm1 validation failed; see build report') from None
    print(json.dumps(dict(status='passed', explicit_tests_passed=report['explicit_tests_passed'],
                          original_bytes=SIZE, scope=report['scope'])))


class AdmissionTests(unittest.TestCase):
    def test_integration_markers_require_exact_success_and_real_counts(self):
        native = (f'test actual ... RAR7_NATIVE_RESULT\t{SIZE}\t{SIZE}\t583440\t0\t8192\t{1024**3}'
                  '\tall_zero\tsize_crc_verified\n'
                  'test result: ok. 1 passed; 0 failed;\n')
        self.assertEqual(checked_native_result(native)['decoded_bytes'], SIZE)
        runtime = (f'test actual ... RAR7_RUNTIME_RESULT\t1\t{SIZE}\t{SIZE}\t400000\t0\t0'
                   '\tno_destination\ntest result: ok. 1 passed; 0 failed;\n')
        self.assertTrue(checked_runtime_result(runtime)['no_destination'])
        for validator, log in ((checked_native_result, native), (checked_runtime_result, runtime)):
            for invalid in (log + log, log.replace('1 passed', '0 passed'),
                            log.replace(str(SIZE), str(SIZE - 1), 1)):
                with self.subTest(validator=validator.__name__), self.assertRaises(ValueError):
                    validator(invalid)
        with self.assertRaises(ValueError):
            checked_native_result(native.replace(str(1024**3), str(8 * 1024**3)))
        with self.assertRaises(ValueError):
            checked_runtime_result(runtime.replace('\t0\t0\tno_destination', '\t1\t0\tno_destination'))

    def test_exact_sha_and_git_blob_are_independent(self):
        data = b'fixed original data'
        item = dict(bytes=len(data), sha256=hashlib.sha256(data).hexdigest(),
                    git_blob_sha1=hashlib.sha1(b'blob 19\0' + data).hexdigest())
        validate(data, item)
        for key in ('sha256', 'git_blob_sha1'):
            with self.subTest(key=key), self.assertRaises(ValueError):
                validate(data, dict(item, **{key: '0' * len(item[key])}))

    def test_streamed_byte_cap_and_content_length(self):
        response = io.BytesIO(b'x' * (RESPONSE_LIMIT + 1))
        response.headers = {}
        with self.assertRaises(ValueError):
            bounded_read(response, dict(bytes=183863))
        response = io.BytesIO(b'not read')
        response.headers = {'Content-Length': str(RESPONSE_LIMIT + 1)}
        with self.assertRaises(ValueError):
            bounded_read(response, dict(bytes=183863))
        self.assertEqual(response.tell(), 0)

    def test_output_outside_build_is_rejected(self):
        with self.assertRaises(ValueError):
            output_root(ROOT / 'crates/fixture-evidence')

    def test_complete_decode_marker_and_memory_credit_bounds(self):
        fields = [str(SIZE), ORIGINAL_SHA, ORIGINAL_CRC, str(SIZE), '393216', '8192', '0',
                  str(DICTIONARY), str(17 * 1024**2), '0', ARCHIVE_SHA]
        suffix = '\tall_zero\tsize_crc_verified\ntest result: ok. 1 passed; 0 failed;\n'
        log = 'RAR7_GENUINE_RESULT\t' + '\t'.join(fields) + suffix
        self.assertTrue(checked_result(log)['full_sha_verified'])
        self.assertTrue(checked_result('test case ... ' + log)['full_sha_verified'])
        with self.assertRaises(ValueError):
            checked_result(log + log)
        for index, invalid in ((0, str(SIZE - 1)), (6, '1'), (8, str(512 * 1024**2 + 1)),
                               (4, str(20 * 1024**3)), (5, '65536')):
            changed = list(fields)
            changed[index] = invalid
            with self.subTest(index=index), self.assertRaises(ValueError):
                checked_result('RAR7_GENUINE_RESULT\t' + '\t'.join(changed) + suffix)


if __name__ == '__main__':
    main()
