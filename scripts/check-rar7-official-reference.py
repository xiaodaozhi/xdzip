"""Linux x86_64 CI-only official UnRAR7.23 full-stream oracle; no plaintext files."""
import argparse
import gzip
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path, PurePosixPath
import platform
import selectors
import signal
import stat
import subprocess
import sys
import tarfile
import tempfile
import time
from types import SimpleNamespace
import unittest
from unittest import mock
import urllib.request
import zlib

ROOT = Path(__file__).resolve().parents[1]
PACKAGE_URL = 'https://www.rarlab.com/rar/rarlinux-x64-723.tar.gz'
PACKAGE = dict(file='rarlinux-x64-723.tar.gz', bytes=746868,
               sha256='759b4b6aa0d9f77131882162951193f3a0e54bf60e1d8dc4255aa308accab588')
MEMBERS = {
    'rar/unrar': dict(file='unrar.bin', bytes=441632,
                      sha256='926d3a00775ed96afccfdef69c3781334b71ccdb733931edf30f4866e4f08410'),
    'rar/license.txt': dict(file='license.txt', bytes=6753,
                            sha256='0420a0c2cd464466c13ddac80c2efd083163ca82c193fa59df068d8386c3f141'),
}
DOWNLOAD_LIMIT = 1024 * 1024
TAR_LIMIT = 8 * 1024 * 1024
CHUNK = 65536
STDERR_LIMIT = 1024 * 1024
ADDRESS_LIMIT = 8 * 1024**3
PRODUCT_MEMORY_LIMIT = 1024**3
MINIMUM_AVAILABLE = 9 * 1024**3
CPU_SECONDS = 600
WALL_SECONDS = 600
NOTICE = ('Official proprietary RAR EULA is retained verbatim beside the selected UnRAR reference '
          'component. This is a separate build/CI oracle, never a product dependency or fallback. '
          'No open-source, fixture-redistribution or release-license approval is inferred. '
          'Only the exact regular unrar and license members are selected; no author executable, '
          'decoder source, installer or upstream README is opened or executed.')


def fixture_module():
    spec = importlib.util.spec_from_file_location('rar7_genuine_fixture', ROOT / 'scripts/check-rar7-genuine-fixture.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def identity(data, expected):
    if len(data) != expected['bytes'] or hashlib.sha256(data).hexdigest() != expected['sha256']:
        raise ValueError('fixed reference data identity mismatch')


def download_body(response, expected):
    length = response.headers.get('Content-Length')
    if length is not None and int(length) != expected['bytes']:
        raise ValueError('fixed package Content-Length mismatch')
    data = bytearray()
    while True:
        chunk = response.read(min(CHUNK, DOWNLOAD_LIMIT + 1 - len(data)))
        if not chunk:
            break
        data.extend(chunk)
        if len(data) > DOWNLOAD_LIMIT or len(data) > expected['bytes']:
            raise ValueError('fixed package response exceeds byte cap')
    data = bytes(data)
    identity(data, expected)
    return data


class BoundedGzip:
    def __init__(self, source, limit):
        self.source, self.limit, self.bytes = source, limit, 0

    def read(self, size=-1):
        remaining = self.limit - self.bytes
        count = remaining + 1 if size < 0 else min(size, remaining + 1)
        data = self.source.read(count)
        self.bytes += len(data)
        if self.bytes > self.limit:
            raise ValueError('reference tar expansion exceeds byte cap')
        return data


def selected_members(package, members=MEMBERS):
    selected = {}
    with gzip.GzipFile(fileobj=io.BytesIO(package)) as compressed:
        bounded = BoundedGzip(compressed, TAR_LIMIT)
        with tarfile.open(fileobj=bounded, mode='r|') as archive:
            for number, member in enumerate(archive):
                if number >= 128:
                    raise ValueError('reference tar member count exceeds cap')
                name = PurePosixPath(member.name)
                if name.is_absolute() or '..' in name.parts or '\x00' in member.name or '\\' in member.name:
                    raise ValueError('unsafe reference tar member name')
                if member.name not in members:
                    continue
                if member.name in selected:
                    raise ValueError('duplicate selected reference member')
                expected = members[member.name]
                if (member.type not in (tarfile.REGTYPE, tarfile.AREGTYPE) or member.issparse()
                        or member.size != expected['bytes'] or member.size > DOWNLOAD_LIMIT):
                    raise ValueError('selected reference member must be exact regular data')
                stream = archive.extractfile(member)
                if stream is None:
                    raise ValueError('missing selected regular data')
                with stream:
                    data = stream.read(expected['bytes'] + 1)
                identity(data, expected)
                selected[member.name] = data
        # A capped drain also checks the gzip trailer, without interpreting any
        # ignored member's contents or writing package paths to the filesystem.
        while bounded.read(CHUNK):
            pass
    if selected.keys() != members.keys():
        raise ValueError('required reference binary or notice is absent')
    return selected


def check_elf(data):
    if (len(data) < 20 or data[:6] != b'\x7fELF\x02\x01'
            or int.from_bytes(data[18:20], 'little') != 62):
        raise ValueError('reference must be Linux ELF64 little-endian x86_64')


def prepare_tool(output, fixture, cache=None, offline=False):
    output.mkdir(parents=True, exist_ok=True)
    if output.is_symlink():
        raise ValueError('reference tool cache must not be a symlink')
    candidates = [output / PACKAGE['file']]
    if cache is not None:
        candidates.append(cache / PACKAGE['file'])
    existing = next((path for path in candidates if path.is_file()), None)
    downloaded = 0
    if existing is not None:
        if existing.stat().st_size != PACKAGE['bytes']:
            raise ValueError('cached official package length mismatch')
        with existing.open('rb') as stream:
            package = stream.read(DOWNLOAD_LIMIT + 1)
        identity(package, PACKAGE)
    else:
        if offline:
            raise ValueError('offline official package is absent')
        request = urllib.request.Request(PACKAGE_URL, headers={'User-Agent': 'xdzip-reference-check'})
        with urllib.request.urlopen(request, timeout=30) as response:
            if response.status != 200 or response.geturl() != PACKAGE_URL:
                raise ValueError('unexpected official package response or redirect')
            package = download_body(response, PACKAGE)
        downloaded = len(package)
    selected = selected_members(package)
    check_elf(selected['rar/unrar'])
    fixture.fixed_file(output / PACKAGE['file'], package)
    for name, data in selected.items():
        fixture.fixed_file(output / MEMBERS[name]['file'], data)
    provenance = dict(url=PACKAGE_URL, package=PACKAGE,
                      members=[dict(path=name, **item) for name, item in MEMBERS.items()],
                      elf=dict(class_bits=64, endian='little', machine='x86_64'),
                      downloaded_body_bytes=downloaded, response_byte_cap=DOWNLOAD_LIMIT,
                      tar_expansion_byte_cap=TAR_LIMIT, notice=NOTICE)
    return selected, provenance


def require_linux(system=None, machine=None):
    system, machine = system or platform.system(), machine or platform.machine()
    if system != 'Linux' or machine.lower() not in ('x86_64', 'amd64'):
        raise ValueError('reference execution requires Linux x86_64')


def available_memory():
    for line in Path('/proc/meminfo').read_text(encoding='ascii').splitlines():
        fields = line.split()
        if fields[0] == 'MemAvailable:' and len(fields) == 3 and fields[2] == 'kB':
            return int(fields[1]) * 1024
    raise ValueError('Linux available-memory admission could not be established')


def apply_limits(resource):
    for kind, value in ((resource.RLIMIT_AS, ADDRESS_LIMIT),
                        (resource.RLIMIT_CPU, CPU_SECONDS), (resource.RLIMIT_CORE, 0)):
        resource.setrlimit(kind, (value, value))
        if resource.getrlimit(kind) != (value, value):
            raise ValueError('reference child resource guard was not applied')


def child_guard():
    import resource
    os.umask(0o077)
    apply_limits(resource)


class StreamVerifier:
    def __init__(self, count, sha256, crc32):
        if count <= 0:
            raise ValueError('reference expected output must not be empty')
        self.expected, self.expected_sha, self.expected_crc = count, sha256, crc32
        self.count, self.crc, self.maximum_chunk = 0, 0, 0
        self.hash = hashlib.sha256()

    def feed(self, data):
        if not data or len(data) > CHUNK or self.count + len(data) > self.expected:
            raise ValueError('reference output exceeds exact byte/chunk quota')
        if data.count(0) != len(data):
            raise ValueError('reference output differs from known zero original')
        self.count += len(data)
        self.maximum_chunk = max(self.maximum_chunk, len(data))
        self.hash.update(data)
        self.crc = zlib.crc32(data, self.crc)

    def finish(self, exit_code):
        if exit_code != 0 or self.count != self.expected:
            raise ValueError('reference did not complete exact output with exit zero')
        digest, crc = self.hash.hexdigest(), f'{self.crc:08x}'
        if digest != self.expected_sha or crc != self.expected_crc:
            raise ValueError('reference complete SHA or CRC mismatch')
        return dict(decoded_bytes=self.count, original_sha256=digest, original_crc32=crc,
                    full_zero_byte_comparison=True, full_sha_verified=True, full_crc_verified=True,
                    maximum_stdout_chunk=self.maximum_chunk, exit_code=exit_code)


class ReferenceCancelled(RuntimeError):
    pass


def private_execution_directory(output, selected, archive, fixture):
    directory = Path(tempfile.mkdtemp(prefix='reference-private-', dir=output))
    os.chmod(directory, 0o700)
    status = directory.stat()
    if directory.is_symlink() or stat.S_IMODE(status.st_mode) != 0o700 or status.st_uid != os.getuid():
        raise ValueError('reference execution directory must be owner-only mode0700')
    for name, data, mode in (('unrar', selected['rar/unrar'], 0o700),
                             ('license.txt', selected['rar/license.txt'], 0o600),
                             ('rar7-md6g.rar', archive, 0o600)):
        fixture.fixed_file(directory / name, data)
        os.chmod(directory / name, mode)
        if stat.S_IMODE((directory / name).stat().st_mode) != mode:
            raise ValueError('reference private file mode mismatch')
    return directory


def kill_and_reap(child):
    if child is None:
        return
    try:
        os.killpg(child.pid, signal.SIGKILL)
    except ProcessLookupError:
        pass
    try:
        child.wait(timeout=5)
    except subprocess.TimeoutExpired:
        child.kill()
        child.wait(timeout=5)


def run_reference(directory, fixture):
    require_linux()
    free = available_memory()
    if free < MINIMUM_AVAILABLE:
        raise ValueError('reference-only runner has insufficient available memory')
    verifier = StreamVerifier(fixture.SIZE, fixture.ORIGINAL_SHA, fixture.ORIGINAL_CRC)
    stderr_hash, stderr_bytes = hashlib.sha256(), 0
    started = time.monotonic()
    deadline = started + WALL_SECONDS
    child, previous_handlers = None, {}
    selector = selectors.DefaultSelector()

    def cancelled(_signal, _frame):
        raise ReferenceCancelled('reference cancelled')

    try:
        for kind in (signal.SIGINT, signal.SIGTERM):
            previous_handlers[kind] = signal.signal(kind, cancelled)
        # One synchronous single-threaded fork. The child verifies hard/soft
        # limits before exec; no shell, inherited credentials or output path.
        child = subprocess.Popen(
            [str(directory / 'unrar'), 'p', '-cfg-', '-idq', '-md6g',
             str(directory / 'rar7-md6g.rar'), 'zeros.bin'], cwd=directory,
            stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
            env={'PATH': '/usr/bin:/bin', 'HOME': str(directory), 'LANG': 'C', 'LC_ALL': 'C'},
            bufsize=0, shell=False, close_fds=True, start_new_session=True, preexec_fn=child_guard)
        selector.register(child.stdout, selectors.EVENT_READ, 'stdout')
        selector.register(child.stderr, selectors.EVENT_READ, 'stderr')
        while selector.get_map() or child.poll() is None:
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                raise TimeoutError('reference wall deadline exceeded')
            for key, _events in selector.select(min(remaining, 0.2)):
                data = os.read(key.fd, CHUNK)
                if not data:
                    selector.unregister(key.fileobj)
                    key.fileobj.close()
                elif key.data == 'stdout':
                    verifier.feed(data)
                else:
                    stderr_bytes += len(data)
                    if stderr_bytes > STDERR_LIMIT:
                        raise ValueError('reference stderr traffic exceeds cap')
                    stderr_hash.update(data)
        observed = verifier.finish(child.wait(timeout=1))
        import resource
        return dict(observed, elapsed_seconds=time.monotonic() - started,
                    stderr_bytes=stderr_bytes, stderr_sha256=stderr_hash.hexdigest(),
                    stderr_traffic_limit=STDERR_LIMIT, raw_stderr_retained=False,
                    child_max_rss_bytes=resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss * 1024,
                    available_memory_before=free,
                    applied_limits=dict(address_space_bytes=ADDRESS_LIMIT, cpu_seconds=CPU_SECONDS,
                                        wall_seconds=WALL_SECONDS, core_bytes=0,
                                        hard_and_soft_verified_before_exec=True),
                    private_directory_mode='0700')
    finally:
        # On every error/cancellation, kill the isolated process group and reap
        # this exact Popen child. Successful completion has already reaped it.
        if child is not None and child.poll() is None:
            kill_and_reap(child)
        if child is not None:
            for pipe in (child.stdout, child.stderr):
                if pipe is not None:
                    pipe.close()
        selector.close()
        for kind, handler in previous_handlers.items():
            signal.signal(kind, handler)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path)
    parser.add_argument('--tool-cache', type=Path, help='optional exact official package cache')
    parser.add_argument('--fixture-cache', type=Path, help='optional exact public fixture and notice cache')
    parser.add_argument('--offline', action='store_true')
    parser.add_argument('--prepare-only', action='store_true', help='portable preparation only; never executes the ELF')
    parser.add_argument('--self-test', action='store_true')
    args = parser.parse_args()
    if args.self_test:
        unittest.main(argv=[sys.argv[0]], exit=False).result.wasSuccessful() or sys.exit(1)
        return
    if args.output is None:
        parser.error('--output is required')
    fixture = fixture_module()
    output = fixture.output_root(args.output)
    output.mkdir(parents=True, exist_ok=True)
    report = dict(schema_version=1, status='running', platform=sys.platform, machine=platform.machine(),
                  case='official-linux-unrar723-genuine-algorithm1-full-reference',
                  role='reference-only, never product validation or a runtime fallback',
                  product_memory_limit_bytes=PRODUCT_MEMORY_LIMIT,
                  requested_reference_limits=dict(address_space_bytes=ADDRESS_LIMIT,
                                                   cpu_seconds=CPU_SECONDS, wall_seconds=WALL_SECONDS),
                  expected=dict(original_bytes=fixture.SIZE, original_sha256=fixture.ORIGINAL_SHA,
                                original_crc32=fixture.ORIGINAL_CRC, fixture_sha256=fixture.ARCHIVE_SHA),
                  command=['unrar', 'p', '-cfg-', '-idq', '-md6g', '<pinned-fixture>', 'zeros.bin'],
                  notice=NOTICE, no_plaintext_file_created=True, explicit_reference_passed=0,
                  limits=['An8GiB official oracle is distinct from the product1GiB memory acceptance.',
                          'Complete output SHA/CRC/zero comparison requires the actual official child to exit0.',
                          'This one nonsolid unencrypted stream establishes no other historical or extended-distance combination.',
                          'The proprietary reference executable and original notices remain build/CI-only.'])

    def save():
        fixture.fixed_file(output / 'report.json', (json.dumps(report, indent=2) + '\n').encode('utf-8'))

    save()
    try:
        if not args.prepare_only:
            require_linux()
        archive_path, public = fixture.prepare(output / 'fixture', args.fixture_cache, args.offline)
        selected, official = prepare_tool(output / 'tool', fixture, args.tool_cache, args.offline)
        report.update(public_source=public, official_source=official)
        if args.prepare_only:
            report.update(status='prepared', explicit_reference_passed=0)
        else:
            with archive_path.open('rb') as stream:
                archive = stream.read(fixture.RESPONSE_LIMIT + 1)
            fixture.validate(archive, fixture.FILES[0])
            directory = private_execution_directory(output, selected, archive, fixture)
            report.update(run_reference(directory, fixture), status='passed', explicit_reference_passed=1)
        save()
    except Exception as error:
        report.update(status='failed', error=fixture.sanitized(type(error).__name__ + ': ' + str(error)))
        save()
        raise SystemExit('official reference failed; see build report') from None
    print(json.dumps(dict(status=report['status'], explicit_reference_passed=report['explicit_reference_passed'])))


class ReferenceTests(unittest.TestCase):
    @staticmethod
    def expected(data):
        return dict(bytes=len(data), sha256=hashlib.sha256(data).hexdigest())

    @staticmethod
    def package(entries):
        result = io.BytesIO()
        with tarfile.open(fileobj=result, mode='w:gz') as archive:
            for name, data, kind in entries:
                member = tarfile.TarInfo(name)
                member.type, member.size = kind, len(data)
                archive.addfile(member, io.BytesIO(data) if kind == tarfile.REGTYPE else None)
        return result.getvalue()

    def test_download_requires_exact_size_hash_and_bounded_body(self):
        data = b'pinned small package'
        response = io.BytesIO(data)
        response.headers = {'Content-Length': str(len(data))}
        self.assertEqual(download_body(response, self.expected(data)), data)
        with self.assertRaises(ValueError):
            identity(data, dict(self.expected(data), sha256='0' * 64))
        response = io.BytesIO(data)
        response.headers = {'Content-Length': str(DOWNLOAD_LIMIT + 1)}
        with self.assertRaises(ValueError):
            download_body(response, self.expected(data))
        self.assertEqual(response.tell(), 0)
        response = io.BytesIO(b'x' * (DOWNLOAD_LIMIT + 1))
        response.headers = {}
        with self.assertRaises(ValueError):
            download_body(response, dict(bytes=DOWNLOAD_LIMIT))

    def test_only_exact_regular_members_are_selected(self):
        members = {'rar/unrar': self.expected(b'binary'), 'rar/license.txt': self.expected(b'notice')}
        entries = [('rar/unrar', b'binary', tarfile.REGTYPE), ('rar/license.txt', b'notice', tarfile.REGTYPE)]
        self.assertEqual(selected_members(self.package(entries), members),
                         {'rar/unrar': b'binary', 'rar/license.txt': b'notice'})
        for changed in (entries[:1], entries + entries[:1],
                        [('rar/unrar', b'', tarfile.SYMTYPE)] + entries[1:],
                        [('../rar/unrar', b'binary', tarfile.REGTYPE)] + entries[1:]):
            with self.subTest(entries=changed), self.assertRaises(ValueError):
                selected_members(self.package(changed), members)
        with self.assertRaises(ValueError):
            selected_members(self.package(entries), dict(members, **{'rar/unrar': self.expected(b'wrong!')}))

    def test_tar_expansion_cap_and_elf_architecture(self):
        source = BoundedGzip(io.BytesIO(b'x' * 65), 64)
        with self.assertRaises(ValueError):
            source.read(65)
        binary = bytearray(20)
        binary[:6], binary[18:20] = b'\x7fELF\x02\x01', (62).to_bytes(2, 'little')
        check_elf(binary)
        binary[18:20] = (183).to_bytes(2, 'little')
        with self.assertRaises(ValueError):
            check_elf(binary)

    def test_platform_and_hard_soft_guards(self):
        require_linux('Linux', 'x86_64')
        for system, machine in (('Windows', 'AMD64'), ('Darwin', 'arm64'), ('Linux', 'aarch64')):
            with self.subTest(system=system, machine=machine), self.assertRaises(ValueError):
                require_linux(system, machine)

        class Limits:
            RLIMIT_AS, RLIMIT_CPU, RLIMIT_CORE = 'address', 'cpu', 'core'

            def __init__(self):
                self.values = {}

            def setrlimit(self, kind, value):
                self.values[kind] = value

            def getrlimit(self, kind):
                return self.values[kind]

        limits = Limits()
        apply_limits(limits)
        self.assertEqual(limits.values, dict(address=(ADDRESS_LIMIT, ADDRESS_LIMIT),
                                            cpu=(600, 600), core=(0, 0)))
        limits.getrlimit = lambda _kind: (0, 0)
        with self.assertRaises(ValueError):
            apply_limits(limits)

    def test_no_output_exit_or_hash_cannot_false_pass(self):
        data = bytes(19)
        def verifier():
            return StreamVerifier(len(data), hashlib.sha256(data).hexdigest(), f'{zlib.crc32(data):08x}')
        with self.assertRaises(ValueError):
            verifier().finish(0)
        checked = verifier()
        checked.feed(data[:7])
        checked.feed(data[7:])
        self.assertTrue(checked.finish(0)['full_sha_verified'])
        for exit_code in (None, 1, -9):
            with self.subTest(exit_code=exit_code), self.assertRaises(ValueError):
                checked.finish(exit_code)
        with self.assertRaises(ValueError):
            StreamVerifier(0, '', '')
        checked.expected_sha = '0' * 64
        with self.assertRaises(ValueError):
            checked.finish(0)

    def test_stream_quota_nonzero_and_chunk_are_rejected(self):
        for data in (b'\x01', bytes(20), bytes(CHUNK + 1)):
            checked = StreamVerifier(19, '', '')
            with self.subTest(length=len(data)), self.assertRaises(ValueError):
                checked.feed(data)

    def test_timeout_cancellation_and_stderr_cap_kill_and_reap_only_the_child(self):
        class Pipe:
            closed = False

            def close(self):
                self.closed = True

        class Child:
            pid, returncode, waits = 424242, None, 0

            def __init__(self):
                self.stdout, self.stderr = Pipe(), Pipe()

            def poll(self):
                return self.returncode

            def wait(self, timeout):
                self.waits += 1
                self.returncode = -9
                return self.returncode

        class Selector:
            closed = False

            def __init__(self, child, mode):
                self.child, self.mode = child, mode

            def register(self, _pipe, _events, _name):
                pass

            def get_map(self):
                return {1: True}

            def select(self, _timeout):
                if self.mode == 'cancel':
                    raise ReferenceCancelled('test cancellation')
                return [(SimpleNamespace(fd=2, data='stderr', fileobj=self.child.stderr), 1)]

            def close(self):
                self.closed = True

        oracle = SimpleNamespace(SIZE=19, ORIGINAL_SHA='', ORIGINAL_CRC='')
        for mode, expected in (('timeout', TimeoutError), ('cancel', ReferenceCancelled), ('stderr', ValueError)):
            child, selector = Child(), None
            selector = Selector(child, mode)
            clock = [0, WALL_SECONDS + 1] if mode == 'timeout' else None
            with self.subTest(mode=mode), \
                    mock.patch(__name__ + '.require_linux'), \
                    mock.patch(__name__ + '.available_memory', return_value=MINIMUM_AVAILABLE), \
                    mock.patch.object(subprocess, 'Popen', return_value=child) as popen, \
                    mock.patch.object(selectors, 'DefaultSelector', return_value=selector), \
                    mock.patch.object(signal, 'signal', return_value=signal.SIG_DFL), \
                    mock.patch.object(signal, 'SIGKILL', 9, create=True), \
                    mock.patch.object(os, 'killpg', create=True) as kill, \
                    mock.patch.object(os, 'read', return_value=bytes(CHUNK)), \
                    mock.patch.object(time, 'monotonic', side_effect=clock, return_value=0):
                with self.assertRaises(expected):
                    run_reference(Path('/unused-test-only'), oracle)
                kill.assert_called_once_with(child.pid, 9)
                self.assertEqual(child.waits, 1)
                self.assertTrue(child.stdout.closed and child.stderr.closed and selector.closed)
                self.assertIs(popen.call_args.kwargs['preexec_fn'], child_guard)
                self.assertFalse(popen.call_args.kwargs['shell'])
                self.assertTrue(popen.call_args.kwargs['start_new_session'])


if __name__ == '__main__':
    main()
