#!/usr/bin/env python3
"""Observe native ImageIO decoding independently of Qt and product resource limits."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import struct
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def qt_probe(directory, install):
    from desktop import Workflow, parser as desktop_parser
    workflow = Workflow(desktop_parser().parse_args(['prepare', '--install-qt']))
    if install:
        workflow.prepare_tools_and_qt()
    qt = workflow.qt
    frameworks = qt/'lib'
    flags = ['xcrun', 'clang++', '-std=c++17', '-O2', '-arch', 'arm64', '-mmacosx-version-min=14.0',
             '-Wall', '-Wextra', '-Werror', '-I', str(ROOT/'apps/desktop'), '-F', str(frameworks),
             '-Wl,-rpath,'+str(frameworks)]
    for name in ('QtCore', 'QtGui', 'QtSvg'):
        flags += ['-isystem', str(frameworks/(name+'.framework')/'Headers'), '-framework', name]
    binaries = {}
    for name, source in [('worker', ROOT/'tools/image-preview/main.cpp'),
                         ('control', ROOT/'apps/desktop/tests/macos_image_limits_probe.cpp')]:
        binary = directory/name
        result = subprocess.run([*flags, str(source), '-o', str(binary)], capture_output=True, text=True)
        log = (result.stdout+result.stderr).replace(str(ROOT), '<workspace>').replace(str(Path.home()), '<home>')
        (directory/(name+'-build.log')).write_text(log)
        if result.returncode:
            raise ValueError('Native image probe compilation failed: '+name)
        binaries[name] = binary
    env = dict(os.environ, QT_QPA_PLATFORM='offscreen', QT_PLUGIN_PATH=str(qt/'plugins'))
    fixtures = [ROOT/'tests/fixtures/preview'/n for n in ('owned-wide.heic', 'owned-phone.heic')]
    result = subprocess.run([str(binaries['control']), *map(str, fixtures)], capture_output=True,
                            timeout=20, env=env, check=True)
    control = json.loads(result.stdout)
    records = []
    for source in fixtures:
        payload = source.read_bytes()
        request = b'XIP1'+struct.pack('<II', 12, len(payload))+payload
        profiles = {}
        for profile in ('product', 'all', 'as', 'fsize', 'nofile', 'cpu'):
            command = [str(binaries['worker'])] if profile == 'product' else [str(binaries['control']), 'worker-'+profile]
            result = subprocess.run(command, input=request, capture_output=True, timeout=10, env=env)
            valid = result.returncode == 0 and result.stdout.startswith(b'XIR1') and len(result.stdout) >= 16
            dimensions = list(struct.unpack('<III', result.stdout[4:16])) if valid else None
            if dimensions:
                width, height, length = dimensions
                valid = 0 < width <= 512 and 0 < height <= 512 and length == width*height*4 and len(result.stdout) == length+16
            diagnostic = result.stderr.decode('utf-8', errors='replace').replace(str(ROOT), '<workspace>')
            profiles[profile] = {'exit_code': result.returncode, 'decoded': valid,
                                 'dimensions': dimensions, 'diagnostic': diagnostic[:4096]}
        records.append({'fixture': source.name, 'profiles': profiles})
    return {'qt': '6.8.2', 'unrestricted_qt_reader': control, 'worker_limits': records,
            'scope': 'Standalone actual worker and one-limit-at-a-time diagnostic; no desktop application build.'}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--qt-probe', action='store_true')
    parser.add_argument('--install-qt', action='store_true')
    args = parser.parse_args()
    output = args.output.resolve()
    if platform.system() != 'Darwin' or not output.is_relative_to(ROOT/'build'):
        parser.error('Use native macOS and keep observations under build/')
    if args.install_qt and not args.qt_probe:
        parser.error('--install-qt requires --qt-probe')
    output.parent.mkdir(parents=True, exist_ok=True)
    env = {k: v for k, v in os.environ.items()
           if not k.startswith(('QT_', 'QML_', 'DYLD_', 'XDZIP_'))}
    records = []
    with tempfile.TemporaryDirectory(prefix='imageio-', dir=output.parent) as temp:
        for name in ('owned-wide.heic', 'owned-phone.heic'):
            source = ROOT/'tests/fixtures/preview'/name
            target = Path(temp)/(source.stem+'.png')
            result = subprocess.run(['/usr/bin/sips', '--resampleHeightWidthMax', '512',
                                     '-s', 'format', 'png', str(source), '--out', str(target)],
                                    capture_output=True, text=True, timeout=20, env=env)
            dimensions = None
            if target.is_file():
                header = target.read_bytes()[:24]
                if len(header) == 24 and header.startswith(b'\x89PNG\r\n\x1a\n'):
                    dimensions = list(struct.unpack('>II', header[16:24]))
            available = (result.returncode == 0 and dimensions is not None
                         and all(0 < n <= 512 for n in dimensions))
            diagnostic = (result.stdout+'\n'+result.stderr)
            for private, label in ((str(ROOT), '<workspace>'), (temp, '<temporary>'),
                                   (str(Path.home()), '<home>')):
                diagnostic = diagnostic.replace(private, label)
            records.append({'fixture': name, 'sha256': hashlib.sha256(source.read_bytes()).hexdigest(),
                            'system_decode_available': available, 'sips_exit_code': result.returncode,
                            'png_dimensions': dimensions, 'diagnostic': diagnostic.strip()[:4096]})
    record = {'schema': 1, 'status': 'observed', 'macos': platform.mac_ver()[0], 'inputs': records,
              'scope': 'Owned native sips/ImageIO control without Qt or product limits; not a product test pass.'}
    if args.qt_probe:
        directory = output.parent/'qt-image-probe'
        directory.mkdir(exist_ok=True)
        record['qt_probe'] = qt_probe(directory, args.install_qt)
    output.write_text(json.dumps(record, indent=2)+'\n')
    print(json.dumps(record))


if __name__ == '__main__':
    main()
