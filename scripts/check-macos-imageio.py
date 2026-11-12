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
        sources = [str(source)]
        if name == 'control':
            sources.append(str(ROOT/'apps/desktop/ImagePreviewProcess.cpp'))
        result = subprocess.run([*flags, *sources, '-o', str(binary)], capture_output=True, text=True)
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
    result = subprocess.run([str(binaries['control']), 'parent', str(binaries['worker']), *map(str, fixtures)],
                            capture_output=True, timeout=20, env=env, check=True)
    parent = json.loads(result.stdout)
    result = subprocess.run([str(binaries['control']), 'file-boundary'], capture_output=True,
                            timeout=10, env=env, check=True)
    boundary = json.loads(result.stdout)
    parent_env = {k: v for k, v in env.items() if k in ('SystemRoot', 'WINDIR', 'PATH', 'LD_LIBRARY_PATH',
                                                     'DYLD_LIBRARY_PATH', 'DYLD_FRAMEWORK_PATH',
                                                     'QT_PLUGIN_PATH', 'QT_QPA_PLATFORM')}
    records = []
    for source in fixtures:
        payload = source.read_bytes()
        request = b'XIP1'+struct.pack('<II', 12, len(payload))+payload
        profiles = {}
        for profile in ('product', 'all', 'as', 'fsize', 'fsize64k', 'fsize1m', 'fsize16m',
                        'fsize64m', 'fsize128m', 'fsize512m', 'nofile', 'cpu', 'parent-environment',
                        'parent-with-home', 'parent-with-tmpdir'):
            inherited = profile == 'product' or profile.startswith('parent-')
            command = [str(binaries['worker'])] if inherited else [str(binaries['control']), 'worker-'+profile]
            child_env = parent_env.copy() if profile.startswith('parent-') else env
            extra = {'parent-with-home': 'HOME', 'parent-with-tmpdir': 'TMPDIR'}.get(profile)
            if extra and extra in env:
                child_env[extra] = env[extra]
            result = subprocess.run(command, input=request, capture_output=True, timeout=10, env=child_env)
            valid = result.returncode == 0 and result.stdout.startswith(b'XIR1') and len(result.stdout) >= 16
            dimensions = list(struct.unpack('<III', result.stdout[4:16])) if valid else None
            if dimensions:
                width, height, length = dimensions
                valid = 0 < width <= 512 and 0 < height <= 512 and length == width*height*4 and len(result.stdout) == length+16
            diagnostic = result.stderr.decode('utf-8', errors='replace').replace(str(ROOT), '<workspace>')
            pixel = None
            if valid:
                at = 16 + ((height//2)*width + width//2)*4
                pixel = list(result.stdout[at:at+4])
            profiles[profile] = {'exit_code': result.returncode, 'decoded': valid,
                                 'dimensions': dimensions, 'pixel': pixel, 'diagnostic': diagnostic[:4096]}
        records.append({'fixture': source.name, 'profiles': profiles})
    return {'qt': '6.8.2', 'unrestricted_qt_reader': control, 'parent_renderer': parent,
            'runtime_file_boundary': boundary, 'worker_limits': records,
            'scope': 'Standalone actual worker and one-limit-at-a-time diagnostic; no desktop application build.'}


def verified_worker(probe):
    boundary = probe['runtime_file_boundary']
    if not (0 < boundary['limit'] <= 16*1024*1024 and boundary['at_limit_allowed']
            and boundary['over_limit_refused']):
        return False
    def owned_pixel(pixel):
        return pixel is not None and len(pixel) == 4 and pixel[3] == 255 and all(
            abs(a-b) <= 8 for a, b in zip(pixel[:3], (52, 146, 189)))
    expected = ((512, 288), (512, 384))
    if len(probe['parent_renderer']) != 2 or len(probe['worker_limits']) != 2:
        return False
    for parent, worker, size in zip(probe['parent_renderer'], probe['worker_limits'], expected):
        if not (parent['decoded'] and (parent['width'], parent['height']) == size
                and owned_pixel(parent.get('pixel'))):
            return False
        for name in ('product', 'parent-environment'):
            result = worker['profiles'][name]
            if not (result['decoded'] and tuple(result['dimensions'][:2]) == size
                    and owned_pixel(result.get('pixel'))):
                return False
    return True


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--qt-probe', action='store_true')
    parser.add_argument('--install-qt', action='store_true')
    parser.add_argument('--verify-worker', action='store_true')
    args = parser.parse_args()
    output = args.output.resolve()
    if platform.system() != 'Darwin' or not output.is_relative_to(ROOT/'build'):
        parser.error('Use native macOS and keep observations under build/')
    if (args.install_qt or args.verify_worker) and not args.qt_probe:
        parser.error('--install-qt/--verify-worker requires --qt-probe')
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
        if args.verify_worker:
            record['worker_regression_status'] = 'passed' if verified_worker(record['qt_probe']) else 'failed'
    output.write_text(json.dumps(record, indent=2)+'\n')
    print(json.dumps(record))
    if args.verify_worker and record['worker_regression_status'] != 'passed':
        raise SystemExit('Actual image-worker or file-budget regression failed')


if __name__ == '__main__':
    main()
