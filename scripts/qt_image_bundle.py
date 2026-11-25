"""Exchange the verified QtGui and TIFF plugin between native probe runners."""
import hashlib
import json
import os
import shutil
import subprocess
import sys
from time import perf_counter
import zipfile

import qt_image_security as image
from qt_network_bundle import MAX_LIBRARY_BYTES, encode, inputs as sdk_inputs, validate_architecture
from qt_network_security import ROOT, digest, run


MEMBERS = {'libpng.module': MAX_LIBRARY_BYTES, 'libtiff.module': MAX_LIBRARY_BYTES,
           'libpng.license': 64 * 1024, 'libtiff.license': 64 * 1024}


def inputs(prefix, sdk=None):
    if (prefix/image.RECEIPT).exists():
        raise ValueError('The probe requires a fresh SDK with stock QtGui and TIFF binaries')
    config, sources = image.configuration()
    sdk = sdk or sdk_inputs(prefix)
    return {**{key: sdk[key] for key in ('os', 'arch', 'runner', 'toolchain', 'cmake', 'ninja',
                                        'core_sha256', 'core_features')},
            'recipe_sha256': image.recipe_digest(),
            'sdk_images': {key: digest(path) for key, path in image.image_modules(prefix).items()},
            'gui_features': image.sdk_gui_features(prefix, config['qt_version']),
            'components': config['components'],
            'qt_sources': {key: pin['sha256'] for key, pin in sources.items()}}


def key(inputs):
    return f'qt-image-v1-{inputs["os"]}-{inputs["arch"]}-' + hashlib.sha256(encode(inputs)).hexdigest()


def pack(bundle, prefix, inputs):
    image.identity(prefix)
    files = {f'{key}.{suffix}': path for suffix, paths in (
        ('module', image.image_modules(prefix)), ('license', image.licenses(prefix))) for key, path in paths.items()}
    inventory = {name: {'bytes': path.stat().st_size, 'sha256': digest(path)} for name, path in files.items()}
    if any(not 0 < entry['bytes'] <= MEMBERS[name] for name, entry in inventory.items()):
        raise ValueError('Qt image module archive exceeds the reuse budget')
    with zipfile.ZipFile(bundle, 'x', compression=zipfile.ZIP_DEFLATED, compresslevel=3) as archive:
        archive.writestr('manifest.json', encode({'schema': 1, 'key': key(inputs), 'inputs': inputs,
                                                  'members': inventory}))
        for name, path in files.items():
            archive.write(path, name)


def unpack(bundle, directory, inputs):
    with zipfile.ZipFile(bundle) as archive:
        names = archive.namelist()
        if len(names) != len(MEMBERS)+1 or set(names) != {*MEMBERS, 'manifest.json'}:
            raise ValueError('Unexpected Qt image module archive inventory')
        meta = archive.getinfo('manifest.json')
        if not 0 < meta.file_size <= 1024 * 1024:
            raise ValueError('Qt image module manifest exceeds the reuse budget')
        manifest = json.loads(archive.read(meta))
        if manifest['schema'] != 1 or manifest['key'] != key(inputs) or manifest['inputs'] != inputs:
            raise ValueError('Qt image module archive does not match this runner and SDK')
        if set(manifest['members']) != MEMBERS.keys():
            raise ValueError('Unexpected Qt image module manifest inventory')
        for name, maximum in MEMBERS.items():
            entry = archive.getinfo(name)
            expected = manifest['members'][name]
            if not 0 < entry.file_size <= maximum or entry.file_size != expected['bytes']:
                raise ValueError('Unexpected Qt image module member size: '+name)
            path = directory/name
            with archive.open(entry) as source, path.open('xb') as output:
                shutil.copyfileobj(source, output)
            if digest(path) != expected['sha256']:
                raise ValueError('Qt image module member digest differs: '+name)


def restore(bundle, prefix, directory, inputs):
    unpack(bundle, directory, inputs)
    modules, licenses = {}, {}
    for key, original in image.image_modules(prefix).items():
        modules[key], licenses[key] = directory/f'{key}.module', directory/f'{key}.license'
        shutil.copymode(original, modules[key])
        validate_architecture(modules[key], inputs['arch'])
        if digest(licenses[key]) != inputs['components'][key]['license_sha256']:
            raise ValueError('Qt image module license differs from the pinned text: '+key)
    image.check_modules(prefix, modules, {'components': inputs['components']}, directory)
    return modules, licenses


def check_decode(prefix, work):
    build = work/'worker'
    flags = []
    if sys.platform == 'darwin':
        from macos_arch import macos_arch
        flags = ['-DCMAKE_OSX_ARCHITECTURES='+macos_arch(), '-DCMAKE_OSX_DEPLOYMENT_TARGET=14.0',
                 '-DCMAKE_BUILD_RPATH='+str(prefix/'lib')]
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
