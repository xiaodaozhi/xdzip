"""Run Rust lint, dependency policy and targeted archive-name regressions."""

import os
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
STRICT_CLIPPY = ['cargo', 'clippy', '--workspace', '--all-targets', '--locked', '--', '-D', 'warnings']
ARCHIVE_NAME_TESTS = [
    'cargo', 'test', '--locked', '-p', 'xdzip-runtime', '--lib', '--',
    'extraction::', 'creation::tests::native_archive_names_',
    'creation::tests::duplicate_archive_names_', 'creation::tests::iso_creation_refuses_',
    'creation::tests::creation_requires_the_checked_directory_identity', 'source_cab::tests::',
    'updating::tests::', 'legacy_names_tests::',
]
RUST_INPUTS = [
    '*.rs', 'Cargo.toml', 'Cargo.lock', ':(glob)**/Cargo.toml', ':(glob)**/Cargo.lock',
    'rust-toolchain.toml', '.cargo', 'crates', 'third_party/brotli-rfc7932/dictionary.bin',
    'tools/native-codec', 'cmake/RustBuild.cmake', 'cmake/RustStaticLibrary.cmake',
    'scripts/build-rust-bridge.py', 'scripts/rust-bridge-inputs.py',
    'VERSION.json', 'resources/update-trust.json', 'resources/ui-languages.json',
    'resources/i18n', 'scripts/build_identity.py',
    '.github/workflows/docs.yml', 'scripts/check-rust-clippy.py',
    'scripts/check-rust-codec-policy.py', 'scripts/rust-codec-policy.json', 'tests/fixtures/extract',
]


def should_run(root, event, before):
    def git(*args, quiet=False):
        output = subprocess.DEVNULL if quiet else None
        return subprocess.run(['git', *args], cwd=root, stdout=output, stderr=output).returncode

    if event == 'pull_request' and git('rev-parse', '--verify', 'HEAD^2', quiet=True) == 0:
        # The checked-out test merge has the actual target tree as its first parent.
        base = 'HEAD^1'
    elif event == 'push' and re.fullmatch('[0-9a-f]{40}', before) and before != '0' * 40:
        base = before
        if git('cat-file', '-e', f'{base}^{{commit}}', quiet=True) != 0:
            if git('fetch', '--no-tags', '--depth=1', 'origin', base) != 0:
                return True
    else:
        return True
    # A failed comparison also requires Clippy.
    return git('diff', '--quiet', base, 'HEAD', '--', *RUST_INPUTS) != 0


def verify_archive_names():
    build = ROOT / 'build/ci-archive-name-worker'
    configure = [
        'cmake', '-S', 'tools/native-codec', '-B', str(build), '-G', 'Ninja',
        '-DCMAKE_BUILD_TYPE=Debug', '-DBUILD_TESTING=OFF',
    ]
    if sys.platform == 'darwin':
        configure.append('-DCMAKE_OSX_DEPLOYMENT_TARGET=14.0')
    result = subprocess.run(configure, cwd=ROOT)
    if result.returncode:
        return result.returncode
    result = subprocess.run(
        ['cmake', '--build', str(build), '--target', 'xdzip-native-codec', '--parallel', '2'],
        cwd=ROOT,
    )
    if result.returncode:
        return result.returncode
    worker = build / ('xdzip-native-codec.exe' if os.name == 'nt' else 'xdzip-native-codec')
    return subprocess.run(
        ARCHIVE_NAME_TESTS, cwd=ROOT, env=dict(os.environ, XDZIP_NATIVE_CODEC_WORKER=str(worker)),
    ).returncode


def main():
    if should_run(ROOT, os.environ.get('XDZIP_EVENT_NAME', ''), os.environ.get('XDZIP_PUSH_BEFORE', '')):
        result = subprocess.run(STRICT_CLIPPY, cwd=ROOT)
        if result.returncode:
            return result.returncode
        result = subprocess.run([sys.executable, 'scripts/check-rust-codec-policy.py'], cwd=ROOT)
        if result.returncode:
            return result.returncode
        return verify_archive_names()
    print('No Rust inputs changed')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
