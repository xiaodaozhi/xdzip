#!/usr/bin/env python3
"""Verify delivered hashes, then rebuild the actual app offline with an existing Qt SDK."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
from macos_launch_services import transient_registrations
from deployment_audit import digest, within

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--delivery', type=Path, required=True)
    parser.add_argument('--qt-prefix', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    delivery = args.delivery.resolve()
    output = args.output.resolve()
    if not within(output, ROOT/'build') or output.exists():
        parser.error('Use a new rebuild directory under build/')
    report = json.loads((delivery/'source-delivery.json').read_text())
    for item in report['files']:
        path = delivery/item['file']
        if not within(path, delivery) or not path.is_file() or digest(path) != item['sha256']:
            raise ValueError('Delivered source hash mismatch')
    with transient_registrations(output):
        source = output/'source'
        source.mkdir(parents=True)
        for name in ['xdzip-source.tar', 'rust-vendor.tar.gz']:
            with tarfile.open(delivery/name) as archive:
                archive.extractall(source, filter='data')
        (source/'.cargo').mkdir(exist_ok=True)
        shutil.copy2(delivery/'cargo-config.toml', source/'.cargo/config.toml')
        environment = dict(os.environ, CARGO_NET_OFFLINE='true', CARGO_HOME=str(output/'empty-cargo-home'),
                           XDZIP_SETTINGS_PATH=str(output/'isolated/settings.ini'))
        build = output/'compiled'
        qt = args.qt_prefix.resolve()
        configure = ['cmake', '-S', str(source), '-B', str(build), '-G', 'Ninja',
                     '-DCMAKE_BUILD_TYPE=Release', '-DXDZIP_BUILD_DESKTOP_PREVIEW=ON',
                     '-DBUILD_TESTING=OFF', '-DXDZIP_BUILD_QT_PROBE=OFF', '-DCMAKE_PREFIX_PATH='+str(qt)]
        if sys.platform == 'darwin':
            configure += ['-DCMAKE_OSX_ARCHITECTURES=arm64', '-DCMAKE_OSX_DEPLOYMENT_TARGET=14.0']
        with (output/'rebuild.log').open('w') as log:
            for command in [configure, ['cmake', '--build', str(build), '--target', 'xdzip-browser', '--parallel', '3']]:
                subprocess.run(command, cwd=source, env=environment, stdout=log, stderr=log, check=True)
            app = build/'apps/desktop'/('xdzip.app/Contents/MacOS/xdzip' if sys.platform == 'darwin' else
                                       'xdzip-browser.exe' if os.name == 'nt' else 'xdzip-browser')
            identity = subprocess.check_output([str(app), '--version'], env=environment, text=True).strip()
            environment.update(QT_QPA_PLATFORM='cocoa' if sys.platform == 'darwin' else 'offscreen',
                               QT_PLUGIN_PATH=str(qt/'plugins'), QML_IMPORT_PATH=str(qt/'qml'))
            if sys.platform == 'darwin':
                environment['QSG_RHI_BACKEND'] = 'metal'
                environment.pop('QT_QUICK_BACKEND', None)
            else:
                environment['QT_QUICK_BACKEND'] = 'software'
            subprocess.run([str(app), '--smoke-test'], cwd=output, env=environment,
                           stdout=log, stderr=log, timeout=30, check=True)
    result = {'status': 'passed', 'revision': report['revision'], 'source_manifest_sha256': digest(delivery/'source-delivery.json'),
              'rebuilt_executable_sha256': digest(app), 'rebuilt_identity': identity,
              'cargo_offline': True, 'empty_cargo_home': True,
              'limits': ['Actual app rebuilt from delivered project and Rust vendor with the installed Qt SDK, not Qt rebuilt from module source.',
                         'Exported source has unknown Git identity; provenance is bound by the delivery manifest.',
                         'Not a bit-for-bit reproducibility, clean-machine or compliance approval claim.']}
    (output/'source-rebuild.json').write_text(json.dumps(result, indent=2)+'\n')
    print('Delivered project and vendored Rust rebuilt offline with an empty Cargo cache; actual app smoke passed')


if __name__ == '__main__':
    main()
