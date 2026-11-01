#!/usr/bin/env python3
"""Build a modified Qt runtime from pinned source, isolated from the installed SDK.

Supports explicit resumable owned build directories. No system install or release
signing. SDK equivalence is deliberately not inferred from a successful build.
"""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tarfile
from deployment_audit import ROOT, digest, within
from qt_sdk_correspondence import replay_patch, sanitize
from qt_source_identity import source_identity

COMMON = ['-DCMAKE_BUILD_TYPE=Release', '-DQT_BUILD_TESTS=OFF', '-DQT_BUILD_EXAMPLES=OFF',
          '-DQT_BUILD_EXAMPLES_BY_DEFAULT=OFF', '-DBUILD_WITH_PCH=OFF', '-DWARNINGS_ARE_ERRORS=OFF']
BASE = ['-DFEATURE_system_'+f+'=OFF' for f in ['zlib', 'pcre2', 'doubleconversion', 'freetype', 'harfbuzz', 'jpeg', 'png', 'sqlite']]
BASE += ['-DFEATURE_'+f+'=OFF' for f in ['zstd', 'brotli', 'dbus', 'icu', 'glib', 'sql_mysql', 'sql_odbc', 'sql_psql', 'sql_mimer']]
TOOLS = ['-DFEATURE_'+f+'=OFF' for f in ['assistant', 'designer', 'distancefieldgenerator', 'kmap2qmap', 'pixeltool', 'qdbus', 'qdoc', 'qtattributionsscanner', 'qtplugininfo', 'clang', 'clangcpp']]
TOOLS += ['-DCMAKE_DISABLE_FIND_PACKAGE_WrapLibClang=ON']
ORDER = ['qtbase', 'qtshadertools', 'qtsvg', 'qtdeclarative', 'qttools', 'qttranslations']


def qtbase_flags(platform):
    # Linux has no Schannel or SecureTransport fallback. Disabling OpenSSL
    # there disables QSslSocket, which the actual product's update code uses.
    openssl = 'ON' if platform.startswith('linux') else 'OFF'
    flags = BASE + ['-DFEATURE_ssl=ON', '-DFEATURE_openssl='+openssl]
    if platform.startswith('linux'):
        flags += ['-DFEATURE_openssl_runtime=ON']
    return flags


def require_ssl_configuration(install):
    header = install/'include/QtNetwork/qtnetwork-config.h'
    if not header.is_file() or header.is_symlink():
        raise ValueError('Source-built Qt Network SSL configuration missing')
    content = header.read_text(encoding='utf-8')
    features = re.findall(r'^\s*#\s*define\s+QT_FEATURE_ssl\s+(-?\d+)\s*$', content, re.MULTILINE)
    if features != ['1'] or re.search(r'^\s*#\s*define\s+QT_NO_SSL\b', content, re.MULTILINE):
        raise ValueError('Source-built Qt Network must have SSL enabled')
    return digest(header)


def load_delivery():
    spec = importlib.util.spec_from_file_location('delivery', ROOT/'scripts/prepare-source-delivery.py')
    module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
    return module


def source_inputs(config, cache, destination):
    delivery = load_delivery()
    items = {i['module']: i for i in config['archives']+config['build_tools']}
    changes = {}
    for patch in config.get('source_build_patches', []) + config.get('sdk_patches', []):
        # AGL consumer helper is also used by a source build on macOS.
        if patch.get('id') == 'macos-no-agl' and sys.platform != 'darwin':
            continue
        file = ROOT/patch['patch']
        if digest(file) != patch['patch_sha256']:
            raise ValueError('Source patch hash mismatch')
        changes[(patch['module'], patch['source_path'])] = file
    for name in ORDER:
        item = items[name]
        archive = delivery.archive_source(item, cache)
        directory = destination/item['file'].removesuffix('.tar.xz')
        if not directory.exists():
            with tarfile.open(archive) as source:
                source.extractall(destination, filter='data')
        # Verify every regular input, allowing exactly the recorded patches.
        # No arbitrary modified/resumed source tree can silently enter evidence.
        with tarfile.open(archive) as source:
            prefix = item['file'].removesuffix('.tar.xz')+'/'
            known=set()
            for member in source:
                if not member.isdir():known.add(member.name.removeprefix(prefix))
                if not member.isfile():
                    continue
                relative = member.name.removeprefix(prefix)
                target = directory/relative
                if not within(target, directory) or not target.is_file() or target.is_symlink():
                    raise ValueError('Source member missing or outside extraction')
                before = source.extractfile(member).read()
                patch = changes.get((name, relative))
                expected = replay_patch(before, patch.read_bytes()) if patch else before
                current = target.read_bytes()
                if current == before and patch:
                    target.write_bytes(expected)
                elif current != expected:
                    raise ValueError('Unexpected modified source input: '+name+'/'+relative)
        actual={f.relative_to(directory).as_posix() for f in directory.rglob('*') if f.is_file() or f.is_symlink()}
        if actual-known:
            raise ValueError('Unexpected extra source input')
    return items


def run(command, log):
    with log.open('a', encoding='utf-8') as output:
        result = subprocess.run([str(x) for x in command], stdout=output, stderr=output)
    if result.returncode:
        raise ValueError('Source build failed; inspect '+log.name)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--cache', type=Path, default=ROOT/'build/source-cache')
    parser.add_argument('--resume', action='store_true')
    parser.add_argument('--apple-sdk', type=Path)
    parser.add_argument('--parallel', type=int, default=3)
    args = parser.parse_args(); output = args.output.resolve()
    if not within(output, ROOT/'build') or (output.exists() and not args.resume) or not 1 <= args.parallel <= 12:
        parser.error('Use a new build/ directory or explicitly --resume; parallel 1–12')
    if sys.platform=='darwin' and not args.apple_sdk:
        args.apple_sdk=Path(subprocess.check_output(['xcrun','--show-sdk-path'],text=True).strip()).resolve()
    output.mkdir(parents=True, exist_ok=True); args.cache.mkdir(parents=True, exist_ok=True)
    if args.apple_sdk and not (args.apple_sdk/'SDKSettings.json').is_file():
        parser.error('Selected Apple SDK settings missing')
    config = json.loads((ROOT/'scripts/corresponding-sources.json').read_text())
    sources = output/'qt-src'; sources.mkdir(exist_ok=True)
    items = source_inputs(config, args.cache.resolve(), sources)
    install = output/'qt-install'; records = []
    native_flags = ['-DCMAKE_OSX_ARCHITECTURES=arm64', '-DCMAKE_OSX_DEPLOYMENT_TARGET=14.0',
                    '-DQT_FORCE_WARN_APPLE_SDK_AND_XCODE_CHECK=ON'] if sys.platform == 'darwin' else []
    if sys.platform == 'darwin' and args.apple_sdk:
        native_flags += ['-DCMAKE_OSX_SYSROOT='+str(args.apple_sdk.resolve())]
    commands = []
    for name in ORDER:
        source = sources/items[name]['file'].removesuffix('.tar.xz'); build = output/'qt-build'/name
        flags = COMMON+native_flags+[f'-DCMAKE_INSTALL_PREFIX={install}']
        if name == 'qtbase':
            flags += qtbase_flags(sys.platform)
        else:
            flags += [f'-DCMAKE_PREFIX_PATH={install}']
            if name == 'qttools': flags += TOOLS
        configure = ['cmake', '-S', source, '-B', build, '-G', 'Ninja']+flags
        log = output/(name+'-build.log')
        run(configure, log)
        run(['cmake', '--build', build, '--parallel', str(args.parallel)], log)
        run(['cmake', '--install', build], log)
        if name == 'qtbase':
            ssl_config_sha256 = require_ssl_configuration(install)
        commands.append({'module': name, 'configure_flags': [sanitize(x.replace(str(install), '<qt-install>')) for x in flags]})
        identity = source_identity(items[name])
        records.append({'module': name, **identity,
                        'archive_sha256': items[name]['sha256'],
                        'cmake_cache_sha256': digest(build/'CMakeCache.txt')})
        print('Source-built and installed '+name, flush=True)
    # Actual exported proof is part of the new QtCore machine code, not metadata.
    core = install/('lib/QtCore.framework/Versions/A/QtCore' if sys.platform == 'darwin' else
                    'bin/Qt6Core.dll' if os.name == 'nt' else 'lib/libQt6Core.so.6.8.2')
    if not core.is_file(): raise ValueError('Source-built QtCore missing')
    configuration = []
    for file in sorted(install.rglob('*config*.h')):
        if file.is_file(): configuration.append({'file':file.relative_to(install).as_posix(), 'sha256':digest(file)})
    report = {'schema_version':1, 'status':'source-build-passed-not-sdk-equivalence',
              'revision':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
              'source_dirty':bool(subprocess.check_output(['git','status','--porcelain'],cwd=ROOT).strip()),
              'apple_sdk':({'name':args.apple_sdk.name,'settings_sha256':digest(args.apple_sdk/'SDKSettings.json')} if args.apple_sdk else None),
              'source_config_sha256':digest(ROOT/'scripts/corresponding-sources.json'),
              'qt_version':config['qt_version'], 'modules':records, 'configuration':commands,
              'installed_headers':configuration, 'qtcore_sha256':digest(core),
              'qt_network_ssl_config_sha256':ssl_config_sha256,
              'patches':[{'file':p['patch'], 'sha256':p['patch_sha256']} for p in config['source_build_patches']+config['sdk_patches']],
              'toolchain':{tool:sanitize(subprocess.run(command,text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT).stdout.splitlines()[0])
                           for tool,command in [('cmake',['cmake','--version']),('ninja',['ninja','--version']),('compiler',['clang++','--version'] if sys.platform=='darwin' else ['cl'] if os.name=='nt' else ['c++','--version'])]},
              'limits':['Native modified compatible build, not byte-for-byte reproduction of the prebuilt SDK.',
                        'Current toolchain/system SDK differs from Qt upstream release builders; no supported-toolchain or license approval claim.']}
    (output/'qt-source-build.json').write_text(json.dumps(report,indent=2)+'\n')
    print('All required Qt runtime and build-tool modules built from pinned sources')


if __name__ == '__main__': main()
