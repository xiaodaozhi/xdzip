"""Build pinned PNG and TIFF updates into their two Qt 6.8.2 modules."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import sys
from macos_arch import macos_arch
import tarfile
import tempfile
import urllib.request

from qt_network_security import ROOT, core_features, digest, library, run, symbol_names
from qt_sdk_correspondence import replay_patch
from windows_ninja_safety import utf8_console


MANIFEST = ROOT/'scripts/qt-image-security.json'
QT_INPUTS = ROOT/'scripts/corresponding-sources.json'
APPLE_PATCHES = {
    'src/corelib/thread/qyieldcpu.h': 'clang-arm-acle.patch',
    'cmake/FindWrapOpenGL.cmake': 'macos-no-agl.patch',
}
RECEIPT = 'xdzip-image-security.json'
LINUX_ABI_CMAKE = ROOT/'tools/image-preview/retain-linux-abi.cmake'
LINUX_ABI_SOURCE = ROOT/'tools/qt-network-security/linux-abi-retain.cpp'
WINDOWS_VERSION_CMAKE = ROOT/'tools/image-preview/retain-windows-versions.cmake'


def configuration():
    config = json.loads(MANIFEST.read_bytes())
    qt = json.loads(QT_INPUTS.read_bytes())
    if (config['schema_version'] != 1 or qt['qt_version'] != config['qt_version'] or
            set(config['components']) != {'libpng', 'libtiff'}):
        raise ValueError('Unexpected Qt image security manifest')
    sources = {item['module']: item for item in qt['archives']}
    sources = {key: sources[key] for key in ('qtbase', 'qtimageformats')}
    return config, sources


def recipe_digest():
    files = [MANIFEST, QT_INPUTS, Path(__file__), ROOT/'scripts/qt_network_security.py', ROOT/'scripts/macos_arch.py']
    files.extend(ROOT/'scripts/qt-patches'/patch for patch in APPLE_PATCHES.values())
    if sys.platform == 'linux':
        files.extend((LINUX_ABI_CMAKE, LINUX_ABI_SOURCE))
    if os.name == 'nt':
        files.append(WINDOWS_VERSION_CMAKE)
    return hashlib.sha256(''.join(digest(path) for path in files).encode()).hexdigest()


def plugin(prefix):
    return prefix/'plugins/imageformats'/(
        'libqtiff.dylib' if sys.platform == 'darwin' else 'qtiff.dll' if os.name == 'nt' else 'libqtiff.so')


def image_modules(prefix):
    return {'libpng': library(prefix, 'Gui'), 'libtiff': plugin(prefix)}


def identity(prefix, *, required=True):
    receipt = prefix/RECEIPT
    if not receipt.is_file() or receipt.is_symlink():
        if required:
            raise ValueError('Prepare the patched Qt image modules before using this SDK')
        return None
    config, qt = configuration()
    record = json.loads(receipt.read_bytes())
    if (not isinstance(record, dict) or record.get('variant') != config['variant'] or
            record.get('recipe_sha256') != recipe_digest() or
            record.get('components') != config['components'] or
            record.get('qt_source_sha256') != {key: item['sha256'] for key, item in qt.items()} or
            record.get('core_sha256') != digest(library(prefix, 'Core')) or
            record.get('module_sha256') != {key: digest(path) for key, path in image_modules(prefix).items()}):
        raise ValueError('Qt image security receipt differs from its inputs or SDK binaries')
    for key, pin in record['components'].items():
        if digest(prefix/'licenses'/f'xdzip-{key}-LICENSE.txt') != pin['license_sha256']:
            raise ValueError('Qt image security license differs from its pinned text')
    return record


def unpack(item, cache, work):
    cache.mkdir(parents=True, exist_ok=True)
    archive = cache/item['file']
    if not archive.is_file():
        with tempfile.NamedTemporaryFile(dir=cache, suffix='.download', delete=False) as stream:
            partial = Path(stream.name)
            try:
                with urllib.request.urlopen(item['url'], timeout=60) as reply:
                    shutil.copyfileobj(reply, stream)
                stream.close()
                if digest(partial) != item['sha256']:
                    raise ValueError('Downloaded Qt image source checksum differs')
                os.replace(partial, archive)
            finally:
                stream.close()
                partial.unlink(missing_ok=True)
    if digest(archive) != item['sha256']:
        raise ValueError('Cached Qt image source checksum differs')
    with tarfile.open(archive) as source:
        source.extractall(work, filter='data')
    return work/item['file'].removesuffix('.xz').removesuffix('.gz').removesuffix('.tar')


def overlay(source, target, keep=()):
    for path in source.iterdir():
        if path.is_file() and path.suffix in ('.c', '.h') and path.name not in keep:
            shutil.copyfile(path, target/path.name)


def flags_from(features, names):
    return ['-DFEATURE_'+key+'='+('ON' if features[key] == '1' else 'OFF')
            for key in names if key in features]


def gui_features(header):
    return dict(re.findall(r'^\s*#\s*define\s+QT_FEATURE_(\w+)\s+(-?1)\s*$', header.read_text(), re.M))


def build(prefix, work, parallel):
    config, qt = configuration()
    sources = {key: unpack(item, ROOT/'build/source-cache', work) for key, item in qt.items()}
    codecs = {key: unpack(item, ROOT/'build/toolchain/qt-install-tool', work)
              for key, item in config['components'].items()}
    for key, path in codecs.items():
        pin = config['components'][key]
        if digest(path/pin['license_file']) != pin['license_sha256']:
            raise ValueError('Qt image dependency license differs from the pinned input')
    base = sources['qtbase']
    png_src = base/'src/3rdparty/libpng'
    overlay(codecs['libpng'], png_src)
    shutil.copyfile(codecs['libpng']/'scripts/pnglibconf.h.prebuilt', png_src/'pnglibconf.h')
    formats = sources['qtimageformats']
    overlay(codecs['libtiff']/'libtiff', formats/'src/3rdparty/libtiff/libtiff',
            keep=('tif_config.h', 'tiffconf.h'))

    common = ['-DCMAKE_BUILD_TYPE=Release', '-DQT_BUILD_TESTS=OFF', '-DQT_BUILD_EXAMPLES=OFF',
              '-DBUILD_WITH_PCH=OFF', '-DWARNINGS_ARE_ERRORS=OFF', '-DCMAKE_POLICY_VERSION_MINIMUM=3.16']
    if os.name == 'nt':
        common += ['-DCMAKE_PROJECT_INCLUDE:FILEPATH='+str(WINDOWS_VERSION_CMAKE)]
    if sys.platform == 'darwin':
        for relative, patch in APPLE_PATCHES.items():
            path = base/relative
            path.write_bytes(replay_patch(path.read_bytes(), (ROOT/'scripts/qt-patches'/patch).read_bytes()))
        common += ['-DCMAKE_OSX_ARCHITECTURES='+macos_arch(), '-DCMAKE_OSX_DEPLOYMENT_TARGET=14.0',
                   '-DQT_FORCE_WARN_APPLE_SDK_AND_XCODE_CHECK=ON']

    qt_version = config['qt_version']
    include = (prefix/'lib/QtGui.framework/Versions/A/Headers' if sys.platform == 'darwin'
               else prefix/'include/QtGui')
    private = f'{qt_version}/QtGui/private/qtgui-config_p.h'
    gui = gui_features(include/'qtgui-config.h') | gui_features(include/private)
    core = core_features(prefix)
    gui_flags = ['-DFEATURE_gui=ON', '-DFEATURE_openssl=OFF', '-DFEATURE_network=ON',
                 '-DFEATURE_system_png=OFF', '-DFEATURE_system_jpeg=OFF']
    gui_flags += ['-DFEATURE_'+feature+'=OFF' for feature in ('widgets', 'sql', 'xml', 'testlib', 'printsupport')]
    gui_flags += ['-DFEATURE_dbus='+('ON' if gui.get('accessibility_atspi_bridge') == '1' else 'OFF')]
    gui_flags += flags_from(core, ('system_zlib', 'system_pcre2', 'icu', 'glib', 'zstd', 'reduce_exports',
                                   'reduce_relocations', 'use_bfd_linker', 'use_gold_linker', 'use_lld_linker'))
    gui_flags += flags_from(gui, ('system_harfbuzz', 'system_freetype', 'fontconfig', 'xcb', 'metal', 'opengl',
                                  'egl', 'vulkan', 'tuiotouch'))
    if gui.get('vulkan') == '1':
        headers = unpack(config['vulkan_headers'], ROOT/'build/toolchain/qt-install-tool', work)
        gui_flags += ['-DVulkan_INCLUDE_DIR:PATH='+str(headers/'include')]
    if sys.platform == 'linux':
        gui_flags += ['-DCMAKE_SHARED_LINKER_FLAGS=-Wl,--undefined=__bss_start -Wl,--undefined=_edata -Wl,--undefined=_end',
                      '-DCMAKE_PROJECT_INCLUDE:FILEPATH='+str(LINUX_ABI_CMAKE),
                      '-DXDZIP_QT_GUI_ABI_SOURCE:FILEPATH='+str(LINUX_ABI_SOURCE)]
        gui_flags += flags_from(gui, ('xcb_sm',))
    gui_build, tiff_build = work/'gui-build', work/'tiff-build'
    run(['cmake', '-S', base, '-B', gui_build, '-G', 'Ninja', *common, *gui_flags], work/'gui-configure.log')
    generated = gui_features(gui_build/'src/gui/qtgui-config.h') | gui_features(gui_build/'src/gui/qtgui-config_p.h')
    if generated != gui:
        changed = sorted(key for key in generated.keys() | gui.keys() if generated.get(key) != gui.get(key))
        raise ValueError('Rebuilt QtGui configuration differs: '+', '.join(
            f'{key} ({gui.get(key)} to {generated.get(key)})' for key in changed))
    run(['cmake', '--build', gui_build, '--target', 'Gui', '--parallel', parallel], work/'gui-build.log')
    run(['cmake', '-S', formats, '-B', tiff_build, '-G', 'Ninja', *common, '-DCMAKE_PREFIX_PATH='+str(prefix),
         '-DFEATURE_system_tiff=OFF', '-DFEATURE_system_webp=OFF', '-DFEATURE_jasper=OFF', '-DFEATURE_mng=OFF'],
        work/'tiff-configure.log')
    run(['cmake', '--build', tiff_build, '--target', 'QTiffPlugin', '--parallel', parallel], work/'tiff-build.log')

    targets = {'libpng': library(gui_build, 'Gui'), 'libtiff': plugin(tiff_build)}
    if sys.platform == 'linux':
        for key, original in image_modules(prefix).items():
            # Build-tree paths expire before the SDK or a deployed worker can load the modules.
            rpath = run(['patchelf', '--print-rpath', original], work/(key+'-old-rpath.log')).strip()
            options = ['--set-rpath', rpath] if rpath else ['--remove-rpath']
            run(['patchelf', *options, targets[key]], work/(key+'-set-rpath.log'))
    for key, caption in (('libpng', 'libpng version '), ('libtiff', 'LIBTIFF, Version ')):
        if (caption+config['components'][key]['version']).encode() not in targets[key].read_bytes():
            raise ValueError('Rebuilt module lacks its pinned library version: '+key)
    before = symbol_names(library(prefix, 'Gui'))
    if missing := before-symbol_names(targets['libpng']):
        (work/'gui-abi.log').write_text('\n'.join(sorted(missing))+'\n', encoding='utf-8')
        raise ValueError('Rebuilt QtGui lacks upstream exports: '+', '.join(sorted(missing)[:30]))
    new_core = symbol_names(library(gui_build, 'Core'))
    gui_imports = symbol_names(targets['libpng'], undefined=True)
    if (gui_imports & new_core)-symbol_names(library(prefix, 'Core')):
        raise ValueError('Rebuilt QtGui imports symbols unavailable in SDK QtCore')
    return targets, codecs


def prepare(prefix, work, parallel):
    config, qt = configuration()
    work.mkdir(parents=True, exist_ok=True)
    lock = work/'prepare.lock'
    os.close(os.open(lock, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600))
    try:
        try:
            previous = identity(prefix, required=False)
        except (ValueError, FileNotFoundError):
            previous = None
        if previous:
            return previous
        paths = image_modules(prefix)
        if any(not path.is_file() or path.is_symlink() or not path.resolve().is_relative_to(prefix)
               for path in paths.values()):
            raise ValueError('SDK image modules must be regular files inside the pinned prefix')
        with tempfile.TemporaryDirectory(prefix='native-', dir=work) as temporary:
            directory = Path(temporary)
            try:
                replacements, sources = build(prefix, directory, parallel)
            finally:
                for log in directory.glob('*.log'):
                    shutil.copy2(log, work/log.name)
            outputs = {'libpng': prefix/'licenses/xdzip-libpng-LICENSE.txt',
                       'libtiff': prefix/'licenses/xdzip-libtiff-LICENSE.txt'}
            receipt = prefix/RECEIPT
            originals = {path: path.read_bytes() if path.is_file() else None
                         for path in [receipt, *outputs.values()]}
            backups = {}
            for key, path in paths.items():
                backups[key] = directory/(key+'.original')
                shutil.copy2(path, backups[key])
            framework = paths['libpng'].parents[2]
            signature = framework/'Versions/A/_CodeSignature/CodeResources'
            if sys.platform == 'darwin':
                originals[signature] = signature.read_bytes() if signature.is_file() else None
            originals_sha = {key: digest(path) for key, path in paths.items()}
            try:
                receipt.unlink(missing_ok=True)
                for key, path in paths.items():
                    staged = path.with_name(path.name+'.xdzip-new')
                    try:
                        shutil.copy2(replacements[key], staged)
                        os.replace(staged, path)
                    finally:
                        staged.unlink(missing_ok=True)
                    outputs[key].parent.mkdir(exist_ok=True)
                    shutil.copyfile(sources[key]/config['components'][key]['license_file'], outputs[key])
                if sys.platform == 'darwin':
                    for name, path in (('gui', framework), ('tiff', paths['libtiff'])):
                        run(['codesign', '--force', '--sign', '-', '--timestamp=none', path], work/(name+'-seal.log'))
                        run(['codesign', '--verify', '--strict', path], work/(name+'-seal-check.log'))
                record = {'variant': config['variant'], 'recipe_sha256': recipe_digest(),
                          'core_sha256': digest(library(prefix, 'Core')),
                          'components': config['components'],
                          'qt_source_sha256': {key: item['sha256'] for key, item in qt.items()},
                          'module_sha256': {key: digest(path) for key, path in paths.items()},
                          'original_module_sha256': originals_sha}
                receipt.write_text(json.dumps(record, indent=2)+'\n', encoding='utf-8')
                return identity(prefix)
            except BaseException:
                for key, path in paths.items():
                    shutil.copy2(backups[key], path)
                for path, before in originals.items():
                    if before is None:
                        path.unlink(missing_ok=True)
                    else:
                        path.write_bytes(before)
                raise
    finally:
        lock.unlink()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--qt-prefix', type=Path, required=True)
    parser.add_argument('--work-dir', type=Path, required=True)
    parser.add_argument('--parallel', type=int, default=6)
    args = parser.parse_args()
    prefix, work = args.qt_prefix.resolve(), args.work_dir.resolve()
    if not work.is_relative_to(ROOT/'build') or work == ROOT/'build' or not 1 <= args.parallel <= 12:
        parser.error('Use an owned build/ directory and 1–12 jobs')
    with utf8_console():
        record = prepare(prefix, work, args.parallel)
    print(json.dumps({'status': 'abi-checked', **record}, sort_keys=True))


if __name__ == '__main__':
    main()
