"""Identify and validate the one native library reused by update CI."""
import hashlib
import json
import os
from pathlib import Path
import platform
import struct
import subprocess
import sys
import zipfile


MAX_LIBRARY_BYTES = 128 * 1024 * 1024
BUNDLE_NAME = 'qt-network.zip'


def command(*args):
    return subprocess.check_output(args, text=True, timeout=30).strip()


def native_arch():
    machine = platform.machine().lower()
    arch = {'amd64': 'x86_64', 'x86_64': 'x86_64', 'aarch64': 'arm64', 'arm64': 'arm64'}.get(machine)
    if not arch or (sys.platform != 'darwin' and arch != 'x86_64'):
        raise ValueError('Unsupported native Qt Network architecture')
    if sys.platform == 'darwin':
        from macos_arch import macos_arch
        translated = subprocess.run(['sysctl', '-n', 'sysctl.proc_translated'], capture_output=True, text=True, timeout=10)
        if arch != macos_arch() or translated.stdout.strip() == '1':
            raise ValueError('Qt Network reuse requires the selected native Mac architecture')
    return arch


def inputs(prefix):
    from qt_network_security import configuration, core_features, digest, features, library, MACOS_NETWORK_TOOLS
    config, source, recipe = configuration()
    arch = native_arch()
    compiler = []
    if sys.platform == 'win32':
        import shutil
        for name in ('cl', 'link', 'nmake'):
            path = shutil.which(name)
            if not path:
                raise ValueError('MSVC toolchain is not active')
            compiler.append([name, digest(path)])
        for name in ('VCToolsVersion', 'WindowsSDKVersion'):
            compiler.append([name, os.environ[name].strip('\\/')])
    elif sys.platform == 'darwin':
        compiler = [command('xcodebuild', '-version'), command('clang', '--version'),
                    command('xcrun', '--sdk', 'macosx', '--show-sdk-build-version')]
    else:
        compiler = [command('c++', '--version'), command('ld', '--version'),
                    sorted(command('dpkg-query', '-W', '-f=${binary:Package}\t${Version}\n').splitlines())]
    return {'os': sys.platform, 'arch': arch, 'runner': os.environ.get('ImageOS', ''),
            'toolchain': compiler, 'cmake': command('cmake', '--version').splitlines()[0],
            'ninja': command('ninja', '--version'), 'variant': config['variant'], 'recipe_sha256': recipe,
            'source': {'url': source['url'], 'sha256': source['sha256']}, 'patches': config['patches'],
            'core_sha256': digest(library(prefix, 'Core')), 'upstream_network_sha256': digest(library(prefix, 'Network')),
            'core_features': core_features(prefix), 'network_features': features(prefix),
            'upstream_tools': {path: digest(prefix/path) for path in MACOS_NETWORK_TOOLS} if sys.platform == 'darwin' else {}}


def encode(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':')).encode()


def key(identity):
    return 'qt-network-v1-' + identity['os'] + '-' + identity['arch'] + '-' + hashlib.sha256(encode(identity)).hexdigest()


def validate_architecture(binary, arch):
    if sys.platform == 'darwin':
        if command('lipo', '-archs', str(binary)).split() != [arch]:
            raise ValueError('Qt Network Mach-O architecture differs')
        return
    with binary.open('rb') as stream:
        header = stream.read(64)
        if sys.platform == 'win32' and header.startswith(b'MZ') and len(header) == 64:
            offset, = struct.unpack_from('<I', header, 60)
            stream.seek(offset)
            valid = stream.read(6) == b'PE\0\0\x64\x86'
        else:
            valid = sys.platform == 'linux' and header.startswith(b'\x7fELF\x02\x01') and header[18:20] == b'\x3e\x00'
    if arch != 'x86_64' or not valid:
        raise ValueError('Qt Network binary architecture differs')


def export_bundle(output, identity, binary, abi, core_imports):
    from qt_network_security import digest
    validate_architecture(binary, identity['arch'])
    size = binary.stat().st_size
    if not 0 < size <= MAX_LIBRARY_BYTES:
        raise ValueError('Qt Network library exceeds the reuse budget')
    manifest = {'schema_version': 1, 'key': key(identity), 'identity': identity, 'abi': abi,
                'core_imports': core_imports, 'bytes': size, 'sha256': digest(binary)}
    output.parent.mkdir(parents=True, exist_ok=True)
    temp = output.with_suffix('.tmp')
    try:
        with zipfile.ZipFile(temp, 'w', compression=zipfile.ZIP_DEFLATED, compresslevel=3) as archive:
            archive.writestr('manifest.json', encode(manifest))
            archive.write(binary, 'network.bin')
        os.replace(temp, output)
    finally:
        temp.unlink(missing_ok=True)


def restore_bundle(bundle, identity, binary):
    from qt_network_security import digest
    with zipfile.ZipFile(bundle) as archive:
        if sorted(archive.namelist()) != ['manifest.json', 'network.bin']:
            raise ValueError('Unexpected Qt Network archive inventory')
        meta = archive.getinfo('manifest.json')
        payload = archive.getinfo('network.bin')
        if meta.file_size > 1024 * 1024 or not 0 < payload.file_size <= MAX_LIBRARY_BYTES:
            raise ValueError('Qt Network archive exceeds the reuse budget')
        manifest = json.loads(archive.read(meta))
        if (manifest['schema_version'] != 1 or manifest['key'] != key(identity)
                or manifest['identity'] != identity or manifest['bytes'] != payload.file_size):
            raise ValueError('Qt Network archive does not match this runner and SDK')
        with archive.open(payload) as source, binary.open('wb') as target:
            while block := source.read(1024 * 1024):
                target.write(block)
    if digest(binary) != manifest['sha256']:
        raise ValueError('Qt Network library digest differs')
    validate_architecture(binary, identity['arch'])
    return manifest
