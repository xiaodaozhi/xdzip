#!/usr/bin/env python3
"""Build the separately installed Ubuntu Files extension for the XDZip Snap."""

import argparse
import hashlib
import json
from pathlib import Path
import platform
import re
import shlex
import shutil
import subprocess
import sys
import tempfile


ROOT = Path(__file__).resolve().parents[1]
PACKAGE = 'xdzip-nautilus-snap'
EXTENSION = Path('usr/lib/x86_64-linux-gnu/nautilus/extensions-4/libxdzip-nautilus.so')


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def command(*args, **kwargs):
    return subprocess.check_output(args, text=True, **kwargs).strip()


def build(destination, work):
    revision = command('git', '-C', ROOT, 'rev-parse', 'HEAD')
    if command('git', '-C', ROOT, 'status', '--porcelain'):
        raise ValueError('Build the Files extension from a clean checkout')
    version = json.loads((ROOT / 'VERSION.json').read_bytes())['version'].replace('-', '~', 1)
    if not re.fullmatch(r'[0-9][0-9A-Za-z.+~]*', version):
        raise ValueError('Unsupported Debian version')
    version += f'+git{revision[:12]}'
    filename = f'{PACKAGE}_{version}_amd64.deb'
    output = destination / filename
    receipt = output.with_suffix('.json')
    if output.exists() or receipt.exists():
        raise FileExistsError('The Files extension output already exists')

    stage = work / 'package'
    extension = stage / EXTENSION
    extension.parent.mkdir(parents=True)
    subprocess.run([sys.executable, '-B', ROOT / 'scripts/ui_languages.py',
                    '--cpp', work / 'UiLanguageData.h'], check=True)
    flags = shlex.split(command('pkg-config', '--cflags', '--libs', 'libnautilus-extension-4',
                                'gio-2.0', 'gmodule-2.0'))
    subprocess.run(['c++', '-std=c++17', '-O2', '-Wall', '-Wextra', '-Werror', '-fPIC',
                    '-shared', '-pthread', '-Wl,--no-undefined', '-DXDZIP_NAUTILUS_SNAP', '-I', work,
                    ROOT / 'apps/desktop/shell/NautilusMenu.cpp', '-o', extension, *flags],
                   check=True)
    extension.chmod(0o644)

    # Ask Ubuntu's package tools for the exact direct ABI dependencies.
    debian = work / 'debian'
    debian.mkdir()
    (debian / 'control').write_text(f'Source: {PACKAGE}\nSection: utils\nPriority: optional\n'
                                    'Maintainer: xiaodaozhi <noreply@xdzip.invalid>\n\n'
                                    f'Package: {PACKAGE}\nArchitecture: amd64\n'
                                    'Description: XDZip Snap Files integration\n', encoding='utf-8')
    prefix = 'shlibs:Depends='
    dependency = command('dpkg-shlibdeps', '-O', '-e', extension, cwd=work)
    if not dependency.startswith(prefix) or '\n' in dependency:
        raise ValueError('Could not identify the Nautilus module dependencies')

    documentation = stage / 'usr/share/doc' / PACKAGE
    documentation.mkdir(parents=True)
    shutil.copyfile(ROOT / 'COPYRIGHT', documentation / 'copyright')
    shutil.copyfile(ROOT / 'LICENSE', documentation / 'LICENSE')
    shutil.copyfile(ROOT / 'third_party/qt/LGPL-3.0.txt', documentation / 'LGPL-3.0.txt')
    shutil.copyfile(ROOT / 'third_party/qt/GPL-3.0.txt', documentation / 'GPL-3.0.txt')
    control = stage / 'DEBIAN'
    control.mkdir()
    # The two packages own the same module path and must never show duplicate menus.
    (control / 'control').write_text(
        f'Package: {PACKAGE}\nVersion: {version}\nArchitecture: amd64\n'
        'Maintainer: xiaodaozhi <noreply@xdzip.invalid>\nSection: utils\nPriority: optional\n'
        f'Depends: snapd, {dependency.removeprefix(prefix)}\nConflicts: xdzip\n'
        'Description: Ubuntu Files context menu for the XDZip Snap\n'
        ' Provides archive actions when a compatible XDZip Snap is installed.\n', encoding='utf-8')
    packed = work / filename
    subprocess.run(['dpkg-deb', '--root-owner-group', '--build', stage, packed], check=True)
    readback = work / 'readback'
    subprocess.run(['dpkg-deb', '-x', packed, readback], check=True)
    if digest(extension) != digest(readback / EXTENSION):
        raise ValueError('Nautilus module differs after packaging')

    destination.mkdir(parents=True, exist_ok=True)
    with output.open('xb') as stream:
        with packed.open('rb') as source:
            shutil.copyfileobj(source, stream)
    receipt.write_text(json.dumps({'filename': filename, 'sha256': digest(output),
                                  'source_revision': revision, 'extension_sha256': digest(extension),
                                  'shell_protocol': 1}, indent=2) + '\n', encoding='utf-8')
    print(output)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    if platform.system() != 'Linux' or platform.machine() != 'x86_64':
        parser.error('Build the extension on Ubuntu 24.04 amd64')
    try:
        with tempfile.TemporaryDirectory(prefix='xdzip-snap-nautilus-') as directory:
            build(args.output.resolve(), Path(directory))
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        raise SystemExit(str(error)) from error


if __name__ == '__main__':
    main()
