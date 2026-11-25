#!/usr/bin/env python3
"""Measure preparation and verify actual PNG and TIFF decoding against the selected SDK."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
from time import perf_counter

import qt_image_bundle as bundle
import qt_image_security as image
from qt_network_security import ROOT, run
from windows_ninja_safety import utf8_console


def check_decode(prefix, work):
    build = work/'worker'
    flags = ['-DCMAKE_OSX_ARCHITECTURES=x86_64', '-DCMAKE_OSX_DEPLOYMENT_TARGET=14.0',
             '-DCMAKE_BUILD_RPATH='+str(prefix/'lib')] if sys.platform == 'darwin' else []
    run(['cmake', '-S', ROOT/'tools/image-preview', '-B', build, '-G', 'Ninja',
         '-DCMAKE_BUILD_TYPE=Release', '-DQt6_DIR='+str(prefix/'lib/cmake/Qt6'), *flags],
        work/'worker-configure.log')
    run(['cmake', '--build', build, '--target', 'xdzip-image-preview'], work/'worker-build.log')
    worker = build/'apps/desktop'/('xdzip-image-preview.exe' if os.name == 'nt' else 'xdzip-image-preview')
    (worker.parent/'qt.conf').write_text(f'[Paths]\nPrefix = {prefix.as_posix()}\n', encoding='utf-8')
    start = perf_counter()
    subprocess.run([sys.executable, ROOT/'scripts/check-image-preview.py', '--worker', worker,
                    '--output', work/'decode.json', '--qt-prefix', prefix], check=True)
    return round(perf_counter()-start, 3)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--mode', required=True, choices=('cold', 'hit'))
    parser.add_argument('--qt-prefix', type=Path, required=True)
    parser.add_argument('--work-dir', type=Path, required=True)
    parser.add_argument('--bundle', type=Path, required=True)
    args = parser.parse_args()
    prefix, work, archive = (path.resolve() for path in (args.qt_prefix, args.work_dir, args.bundle))
    if any(path == ROOT/'build' or not path.is_relative_to(ROOT/'build') for path in (work, archive)):
        parser.error('Use owned paths under build/')
    work.mkdir(parents=True, exist_ok=True)

    def restore(sdk, directory, _parallel):
        return bundle.restore(archive, sdk, directory, inputs)

    start = perf_counter()
    with utf8_console():
        inputs = bundle.inputs(prefix)
        build_modules = image.build if args.mode == 'cold' else restore
        receipt = image.prepare(prefix, work, 6, build_modules=build_modules)
    preparation_seconds = round(perf_counter()-start, 3)
    for path in image.image_modules(prefix).values():
        bundle.validate_architecture(path, inputs['arch'])
    with utf8_console():
        decode_seconds = check_decode(prefix, work)
    if args.mode == 'cold':
        archive.parent.mkdir(parents=True, exist_ok=True)
        bundle.pack(archive, prefix, inputs)
    record = {'schema': 1, 'mode': args.mode, 'key': bundle.key(inputs), 'architecture': inputs['arch'],
              'revision': os.environ.get('GITHUB_SHA', ''), 'runner_image': os.environ.get('ImageVersion', ''),
              'image_prepare_seconds': preparation_seconds, 'decode_check_seconds': decode_seconds,
              'archive_bytes': archive.stat().st_size, 'installed_module_sha256': receipt['module_sha256']}
    (work/'image-timing.json').write_text(json.dumps(record, indent=2)+'\n', encoding='utf-8')
    print(json.dumps(record), flush=True)
    if summary := os.environ.get('GITHUB_STEP_SUMMARY'):
        with Path(summary).open('a', encoding='utf-8') as stream:
            print(f"Image modules ({args.mode}, {inputs['os']}): **{preparation_seconds:.3f} s**. "
                  f'PNG and TIFF pixel checks passed in **{decode_seconds:.3f} s**.', file=stream)


if __name__ == '__main__':
    main()
