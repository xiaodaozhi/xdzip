#!/usr/bin/env python3
"""Build and verify the explicitly invoked, non-product SDK trial."""
import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import struct
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def digest(path):
    result = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(65536), b''):
            result.update(block)
    return result.hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=['test', 'example'])
    parser.add_argument('--output-dir', default='build/review/sdk-trial')
    args = parser.parse_args()
    output = (ROOT / args.output_dir).resolve()
    if not output.is_relative_to(ROOT / 'build'):
        parser.error('Output must be inside build/')
    output.mkdir(parents=True, exist_ok=True)
    env = os.environ.copy()
    env['RUSTUP_AUTO_INSTALL'] = '0'
    suffix = '.exe' if sys.platform == 'win32' else ''
    binary = ROOT / f'target/debug/xdzip-sdk-reference{suffix}'
    host = ROOT / f'target/debug/xdzip-plugin-host{suffix}'
    commands = []

    def run(name, command):
        result = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, text=True)
        log = (result.stdout + result.stderr).replace(str(ROOT), '<workspace>').replace(str(Path.home()), '<home>')
        (output / f'{name}.log').write_text(log, encoding='utf-8')
        commands.append({'check': name, 'exit_code': result.returncode})
        return result.returncode

    code = run('build', ['cargo', 'build', '--locked', '-p', 'xdzip-plugin-host', '--bins'])
    if not code and args.command == 'test':
        env['XDZIP_SDK_REFERENCE'] = str(binary)
        env['XDZIP_SDK_PYTHON'] = sys.executable
        code = run('conformance', ['cargo', 'test', '--locked', '-p', 'xdzip-plugin-protocol', '-p', 'xdzip-plugin-host', '--lib', '--', '--include-ignored'])
    if not code and args.command == 'example':
        package = output / 'owned-package'
        package.mkdir(exist_ok=True)
        shutil.copy2(binary, package / f'peer{suffix}')
        shutil.copy2(ROOT / 'LICENSE', package / 'LICENSE')
        manifest = {'manifest_version': 1, 'plugin_id': 'org.xdzip.sdk.reference', 'version': '0.1.0', 'author': 'xiaodaozhi',
                    'protocol_majors': [1], 'targets': [f'{platform.system().lower().replace("darwin", "macos")}-{platform.machine().lower().replace("arm64", "aarch64").replace("amd64", "x86_64")}'],
                    'entrypoint': f'peer{suffix}', 'sha256': digest(binary), 'formats': ['xdp1'],
                    'permissions': {'broker_source_read': True, 'broker_output_write': True, 'network': False},
                    'resources': {'memory_bytes': 536870912, 'threads': 1}, 'license_files': ['LICENSE'], 'source_url': 'https://github.com/xiaodaozhi/xdzip'}
        (package / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n', encoding='utf-8')
        name, content = 'hello.txt'.encode(), b'owned SDK example\n'
        sample = b'XDP1' + struct.pack('<IHI', 1, len(name), len(content)) + name + content
        (output / 'owned.xdp').write_bytes(sample)
    revision = subprocess.run(['git', 'rev-parse', 'HEAD'], cwd=ROOT, capture_output=True, text=True, check=True).stdout.strip()
    dirty = bool(subprocess.run(['git', 'status', '--porcelain'], cwd=ROOT, capture_output=True, text=True, check=True).stdout)
    gate = subprocess.run([str(host), 'gate'], cwd=ROOT, capture_output=True, text=True).stdout.strip() if host.exists() else 'unbuilt'
    report = {'schema': 1, 'utc': datetime.now(timezone.utc).isoformat(), 'revision': revision, 'dirty': dirty,
              'system': platform.system(), 'os_release': platform.release(), 'architecture': platform.machine(), 'compiled_policy': gate,
              'checks': commands, 'passed': code == 0, 'native_peer_sha256': digest(binary) if binary.exists() else None,
              'host_sha256': digest(host) if host.exists() else None, 'python': platform.python_version()}
    (output / 'result.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(report))
    raise SystemExit(code)


if __name__ == '__main__':
    main()
