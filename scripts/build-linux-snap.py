#!/usr/bin/env python3
"""Pack a trusted XDZip Linux runtime without invoking a build provider or the Store."""

import argparse
import hashlib
import json
import os
import platform
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
import urllib.request

from deployment_audit import ROOT, digest, linux_imports, native, within
from qt_image_security import identity as image_identity
from qt_network_security import identity as network_identity


PACKAGING = ROOT / 'apps/desktop/packaging/linux'
LIBRARIES = Path('usr/lib/x86_64-linux-gnu')
CORE24_SHA256 = '819186f53e7e277ff4eb92c64bb44af99e8a8b67ba168dd6e40e1eb006cf9f5b'


def command(*args, **kwargs):
    return subprocess.check_output(args, text=True, **kwargs).strip()


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + '\n', encoding='utf-8')


def fetch_deb(cache, pin):
    destination = cache / pin['url'].rsplit('/', 1)[-1]
    if destination.is_file():
        if digest(destination) != pin['sha256']:
            raise ValueError(f'Cached Ubuntu package differs from its lock: {pin["package"]}')
        return destination
    with tempfile.NamedTemporaryFile(dir=cache, suffix='.download', delete=False) as stream:
        partial = Path(stream.name)
        try:
            with urllib.request.urlopen(pin['url'], timeout=60) as response:
                shutil.copyfileobj(response, stream)
            stream.close()
            if digest(partial) != pin['sha256']:
                raise ValueError(f'Downloaded Ubuntu package differs from its lock: {pin["package"]}')
            os.replace(partial, destination)
        finally:
            partial.unlink(missing_ok=True)
    return destination


def stage_ubuntu_packages(stage, scratch, cache):
    lock = json.loads((PACKAGING / 'snap-system-packages.json').read_bytes())
    cache.mkdir(parents=True, exist_ok=True)
    source = scratch / 'ubuntu-packages'
    for pin in lock['packages']:
        archive = fetch_deb(cache, pin)
        fields = dict(line.split(': ', 1) for line in command(
            'dpkg-deb', '-f', archive, 'Package', 'Version', 'Architecture').splitlines())
        if any(fields[key.capitalize()] != pin[key] for key in ('package', 'version', 'architecture')):
            raise ValueError(f'Ubuntu package metadata differs from its lock: {pin["package"]}')
        command('dpkg-deb', '-x', archive, source)

    # Ubuntu multi-binary packages may share a copyright file by symlink.
    records = []
    for pin in lock['packages']:
        notice = Path('usr/share/doc') / pin['package'] / 'copyright'
        files = []
        for relative in [*map(Path, pin['paths']), notice]:
            origin = source / relative
            if not within(origin, source) or not origin.exists():
                raise ValueError(f'Missing or escaping Ubuntu package path: {relative}')
            if origin.is_dir():
                links = [link for link in origin.rglob('*') if link.is_symlink()]
                if any(not within(link, origin) or not link.exists() for link in links):
                    raise ValueError(f'Escaping or broken symlink in Ubuntu package: {relative}')
                shutil.copytree(origin, stage / relative)
            else:
                destination = stage / relative
                destination.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(origin, destination)
            deployed = stage / relative
            selected = sorted(deployed.rglob('*')) if deployed.is_dir() else [deployed]
            files.extend({'path': path.relative_to(stage).as_posix(), 'sha256': digest(path)}
                         for path in selected if path.is_file())
        records.append({**pin, 'notice': notice.as_posix(), 'files': files})
    return records


def verify_qt(runtime, qt, grade):
    images = image_identity(qt, required=grade == 'stable')
    network = network_identity(qt)
    deployed = json.loads((runtime / 'share/deployment.json').read_bytes())
    if not deployed['experimental_qt_wayland']:
        raise ValueError('Build the Linux runtime with --experimental-wayland')
    if digest(runtime / 'bin/xdzip') != deployed['application_transform']['artifact_sha256']:
        raise ValueError('The desktop binary differs from its deployment record')
    expected = {network['network_sha256']}
    if images:
        expected.update(images['module_sha256'].values())
    for entry in deployed['sdk_files']:
        path, source = runtime / entry['path'], qt / entry['sdk_path']
        if (not within(path, runtime) or not within(source, qt) or
                digest(path) != entry['artifact_sha256'] or digest(source) != entry['sdk_sha256']):
            raise ValueError(f'Runtime differs from the attested Qt SDK: {entry["path"]}')
        expected.discard(entry['sdk_sha256'])
    if expected:
        raise ValueError('A security-patched Qt module is missing from the runtime inventory')
    return images, network


def verify_native(stage, base):
    runtime = stage / 'runtime'
    loader = base / LIBRARIES / 'ld-linux-x86-64.so.2'
    libraries = ':'.join(map(str, (runtime / 'lib', stage / LIBRARIES, base / LIBRARIES)))
    environment = {'LANG': 'C'}
    load = (loader, '--inhibit-cache', '--library-path', libraries)
    resolved = {}
    for path in sorted(stage.rglob('*')):
        if path.is_symlink() and not within(path, stage):
            raise ValueError(f'Escaping symlink in the Snap: {path.relative_to(stage)}')
        if not path.is_file() or not native(path, 'linux'):
            continue
        with path.open('rb') as stream:
            elf = stream.read(20)
        if elf[:6] != b'\x7fELF\x02\x01' or int.from_bytes(elf[18:20], 'little') != 62:
            raise ValueError(f'Unexpected ELF architecture: {path.relative_to(stage)}')
        result = subprocess.run([*load, '--list', path], env=environment,
                                capture_output=True, text=True, check=False)
        if result.returncode:
            message = result.stderr.replace(str(stage) + '/', '').replace(str(base) + '/', '')
            raise ValueError(f'core24 could not load {path.relative_to(stage)}: {message.strip()}')
        imports = {}
        for name, library in linux_imports(result.stdout):
            if library and within(library, stage):
                imports[name] = 'snap/' + library.resolve().relative_to(stage).as_posix()
            elif library and within(library, base):
                imports[name] = 'core24/' + library.resolve().relative_to(base).as_posix()
            else:
                raise ValueError(f'{path.relative_to(stage)} requires {name} outside the Snap and core24')
        resolved[path.relative_to(stage).as_posix()] = imports
    app = json.loads(command(*load, runtime / 'bin/xdzip', '--version-json', env=environment))
    return app, resolved


def save_candidate(packed, outfile, report, receipt, grade):
    values = {**receipt, 'filename': outfile.name, 'sha256': digest(packed),
              'bytes': packed.stat().st_size, 'grade': grade, 'confinement': 'strict'}
    with outfile.open('xb') as destination:
        try:
            with packed.open('rb') as source:
                shutil.copyfileobj(source, destination)
            destination.flush()
            if digest(outfile) != values['sha256']:
                raise OSError('Copied Snap differs from the verified output')
            with report.open('x', encoding='utf-8') as stream:
                try:
                    json.dump(values, stream, indent=2, ensure_ascii=False)
                    stream.write('\n')
                    stream.flush()
                except OSError:
                    report.unlink()
                    raise
        except OSError:
            outfile.unlink()
            raise


def pack(args, scratch):
    if digest(args.core24) != CORE24_SHA256:
        raise ValueError('core24 does not match the pinned amd64 base used for verification')
    revision = command('git', '-C', ROOT, 'rev-parse', 'HEAD')
    inventory = json.loads((args.runtime / 'share/notices/build-inventory.json').read_bytes())
    if inventory.get('revision') != revision or inventory.get('dirty') is not False:
        raise ValueError('Deploy a clean runtime from the same source revision as this recipe')
    image, network = verify_qt(args.runtime, args.qt_prefix, args.grade)
    stage, base = scratch / 'payload', scratch / 'core24'
    shutil.copytree(args.runtime, stage / 'runtime', symlinks=True)
    command('unsquashfs', '-q', '-processors', '2', '-no-progress', '-no-xattrs',
            '-d', base, args.core24, LIBRARIES)
    packages = stage_ubuntu_packages(stage, scratch, args.package_cache)

    app, imports = verify_native(stage, base)
    if (app['dirty'] is not False or app['revision'] != revision[:12] or
            not re.fullmatch(r'[0-9][0-9A-Za-z.+~-]{0,31}', app['version'])):
        raise ValueError('The desktop executable does not match the declared build')
    version = app['version'] if args.grade == 'stable' else f'{app["version"]}.dev.{revision[:12]}'

    gui = stage / 'meta/gui'
    gui.mkdir(parents=True)
    directory = stage / 'bin'
    directory.mkdir()
    launch = directory / 'launch'
    launch.write_bytes((PACKAGING / 'snap-launch.sh').read_bytes().replace(b'\r\n', b'\n'))
    launch.chmod(0o755)
    (stage / 'meta/xdzip-shell-protocol').write_bytes(b'1\n')
    desktop = (PACKAGING / 'xdzip.desktop').read_text(encoding='utf-8')
    if desktop.count('\nIcon=xdzip\n') != 1:
        raise ValueError('The desktop entry no longer has its expected icon field')
    desktop = desktop.replace('\nIcon=xdzip\n', '\nIcon=${SNAP}/meta/gui/icon.png\n')
    (gui / 'xdzip.desktop').write_text(desktop, encoding='utf-8')
    # The desktop-file validator expects snapd's expanded, absolute icon path.
    validation_entry = scratch / 'xdzip.desktop'
    validation_entry.write_text(desktop.replace('${SNAP}', str(stage)), encoding='utf-8')
    command('desktop-file-validate', validation_entry)
    shutil.copyfile(ROOT / 'design/assets/brand/xdzip-256.png', gui / 'icon.png')
    shutil.copyfile(PACKAGING / 'snap-fonts.conf', stage / 'meta/fonts.conf')
    template = (PACKAGING / 'snap.yaml.in').read_text(encoding='utf-8')
    (stage / 'meta/snap.yaml').write_text(template.replace('@VERSION@', version).replace('@GRADE@', args.grade),
                                         encoding='utf-8')
    receipt = {'source_revision': revision, 'version_json': app, 'base': 'core24',
               'base_sha256': CORE24_SHA256, 'qt_image_security': image, 'qt_network_security': network,
               'ubuntu_packages': packages, 'elf_imports': imports,
               'launcher_sha256': digest(launch), 'application_sha256': digest(stage / 'runtime/bin/xdzip'),
               'cli_sha256': digest(stage / 'runtime/cli/xdzip')}
    write_json(stage / 'meta/xdzip-build.json', receipt)
    command('snap', 'pack', '--check-skeleton', stage)
    outfile = args.output / f'xdzip_{version}_amd64.snap'
    report = outfile.with_suffix('.json')
    if outfile.exists() or report.exists():
        raise FileExistsError('The candidate output already exists')
    args.output.mkdir(parents=True, exist_ok=True)
    command('snap', 'pack', stage, scratch)
    packed = scratch / outfile.name
    for path in ('meta/snap.yaml', 'meta/gui/xdzip.desktop', 'meta/xdzip-shell-protocol',
                 'bin/launch', 'runtime/bin/xdzip', 'runtime/cli/xdzip'):
        payload = subprocess.check_output(['unsquashfs', '-cat', packed, path])
        if hashlib.sha256(payload).hexdigest() != digest(stage / path):
            raise ValueError(f'Packed file differs from the staged input: {path}')
    save_candidate(packed, outfile, report, receipt, args.grade)
    return outfile


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('runtime', 'qt-prefix', 'core24', 'package-cache', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--grade', choices=('devel', 'stable'), default='devel')
    args = parser.parse_args()
    if platform.system() != 'Linux' or platform.machine() != 'x86_64':
        parser.error('Build and verify the Snap on native amd64 Linux')
    for name in ('runtime', 'qt_prefix', 'core24', 'package_cache', 'output'):
        setattr(args, name, getattr(args, name).resolve())
    try:
        with tempfile.TemporaryDirectory(prefix='xdzip-snap-') as directory:
            print(pack(args, Path(directory)))
    except (KeyError, ValueError, OSError, subprocess.CalledProcessError) as error:
        raise SystemExit(str(error)) from error


if __name__ == '__main__':
    main()
