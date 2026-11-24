#!/usr/bin/env python3
"""Build, verify and package the Qt-independent CLI using the shared native worker."""
from __future__ import annotations
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
from macos_arch import macos_arch
import shutil
import stat
import subprocess
import tarfile
import zipfile

from build_identity import source_identity

ROOT = Path(__file__).resolve().parents[1]


def digest(path):
    sha = hashlib.sha256()
    with Path(path).open('rb') as source:
        for block in iter(lambda: source.read(65536), b''):
            sha.update(block)
    return sha.hexdigest()


def copy_program_if_changed(source, target):
    source_info = source.stat()
    try:
        target_info = target.stat()
    except FileNotFoundError:
        shutil.copy2(source, target)
        return
    if os.path.samestat(source_info, target_info):
        raise shutil.SameFileError(f'{source} and {target} are the same file')
    if (not stat.S_ISREG(target_info.st_mode) or source_info.st_size != target_info.st_size
            or stat.S_IMODE(source_info.st_mode) != stat.S_IMODE(target_info.st_mode)):
        shutil.copy2(source, target)
        return
    with source.open('rb') as original, target.open('rb') as existing:
        for block in iter(lambda: original.read(65536), b''):
            if block != existing.read(len(block)):
                break
        else:
            return
    shutil.copy2(source, target)


def write_text_if_changed(path, text):
    contents = text.replace('\n', os.linesep).encode('utf-8')
    if not path.exists() or path.read_bytes() != contents:
        path.write_bytes(contents)


def generated(path):
    path = Path(path).resolve()
    if not path.is_relative_to(ROOT/'build') or path == ROOT/'build':
        raise ValueError('CLI outputs must use a dedicated directory under build/')
    return path


def run(command, *, capture=False, env=None):
    result = subprocess.run([str(part) for part in command], cwd=ROOT, env=env,
                            capture_output=capture, text=True)
    if result.returncode:
        raise ValueError(Path(command[0]).name+' failed with exit '+str(result.returncode))
    return result.stdout if capture else None


def exe(name):
    return name + ('.exe' if os.name == 'nt' else '')


def build(args):
    directory = generated(args.build_dir)
    environment = dict(os.environ, CARGO_TARGET_DIR=str(directory/'target'),
                       XDZIP_BUILD_SOURCE=json.dumps(source_identity(ROOT)),
                       XDZIP_CLI_CONFIGURATION=args.profile)
    if platform.system() == 'Darwin':
        environment['MACOSX_DEPLOYMENT_TARGET'] = '14.0'
    command = ['cargo', 'build', '--locked', '-p', 'xdzip-cli']
    cargo_profile = 'dev' if args.profile == 'debug' else args.profile
    configuration = {'debug': 'Debug', 'release': 'Release', 'relwithdebinfo': 'RelWithDebInfo'}[args.profile]
    command.extend(['--profile', cargo_profile])
    run(command, env=environment)
    if os.name == 'nt':
        for name in ['xdzip-launcher', 'xdzip-sfx']:
            launcher = ['cargo','rustc','--locked','-p','xdzip-cli','--bin',name]
            launcher.extend(['--profile', cargo_profile])
            # Launcher and independently distributed SFX cannot require the app-local CRT.
            run([*launcher,'--','-C','target-feature=+crt-static'],env=environment)
    worker = args.codec
    if worker is None:
        codec = directory/'codec'
        configure = ['cmake', '-S', ROOT/'tools/native-codec', '-B', codec,
                     '-G', 'Ninja', '-DCMAKE_BUILD_TYPE='+configuration, '-DBUILD_TESTING=OFF']
        if platform.system() == 'Darwin':
            configure += ['-DCMAKE_OSX_ARCHITECTURES='+macos_arch(), '-DCMAKE_OSX_DEPLOYMENT_TARGET=14.0']
        run(configure, env=environment)
        run(['cmake', '--build', codec, '--target', 'xdzip-native-codec', '--parallel', '2'], env=environment)
        worker = codec/exe('xdzip-native-codec')
    output = directory/'bin'
    output.mkdir(parents=True, exist_ok=True)
    for name in ['xdzip', 'xdzip-launcher'] + (['xdzip-sfx'] if os.name == 'nt' else []):
        copy_program_if_changed(directory/'target'/args.profile/exe(name), output/exe(name))
    copy_program_if_changed(Path(worker), output/exe('xdzip-native-codec'))
    identity = json.loads(run([output/exe('xdzip'), '--version-json'], capture=True, env=environment))
    write_text_if_changed(directory/'identity.json', json.dumps(identity, indent=2)+'\n')
    print('CLI build completed: '+identity['version'])


def bundle(args):
    directory = generated(args.build_dir)
    output = generated(args.output)
    if output == (directory/'bin').resolve():
        raise ValueError('Desktop CLI output must differ from the build bin directory')
    context = output/'xdzip-cli-context.json'
    if platform.system() == 'Darwin':
        if output.name != 'cli' or output.parent.name != 'Helpers' or output.parent.parent.name != 'Contents':
            raise ValueError('Mac desktop CLI requires Contents/Helpers/cli')
        context = generated(output.parent.parent/'Resources')/'xdzip-cli-context.json'
    output.mkdir(parents=True, exist_ok=True)
    worker_name = 'xdzip-native-codec'+('.exe' if platform.system()=='Windows' else '')
    for path in (directory/'bin').iterdir():
        if path.is_file() and path.name!=worker_name:
            copy_program_if_changed(path, output/path.name)
    duplicate=output/worker_name
    if duplicate.is_symlink() or duplicate.is_dir():
        raise ValueError('Unexpected desktop CLI worker path')
    duplicate.unlink(missing_ok=True)
    relative = {'Darwin':'../../MacOS/xdzip', 'Windows':'../xdzip.exe', 'Linux':'../xdzip'}[platform.system()]
    worker_relative='../../MacOS/'+worker_name if platform.system()=='Darwin' else '../'+worker_name
    context.parent.mkdir(parents=True, exist_ok=True)
    write_text_if_changed(context, json.dumps(dict(schema_version=1, kind='desktop', desktop_relative=relative,
                                                  worker_relative=worker_relative))+'\n')
    if platform.system() == 'Darwin':
        (output/'xdzip-cli-context.json').unlink(missing_ok=True)


def notices(output):
    """Collect actual CLI/worker normal dependency notices, without Qt or Sparkle."""
    output.mkdir(parents=True, exist_ok=True)
    host = next(line.removeprefix('host: ') for line in run(['rustc', '-vV'], capture=True).splitlines() if line.startswith('host: '))
    metadata = json.loads(run(['cargo', 'metadata', '--locked', '--offline', '--format-version', '1', '--filter-platform', host], capture=True))
    packages = {p['id']:p for p in metadata['packages']}
    nodes = {n['id']:n for n in metadata['resolve']['nodes']}
    selected = set()
    pending = [p['id'] for p in packages.values() if p['name'] in ['xdzip-cli', 'xdzip-codec-bridge'] and p['source'] is None]
    while pending:
        key = pending.pop()
        if key in selected or any('proc-macro' in t['kind'] for t in packages[key]['targets']):
            continue
        selected.add(key)
        pending.extend(d['pkg'] for d in nodes[key]['deps'] if any(kind['kind'] is None for kind in d['dep_kinds']))
    inventory = []
    for key in sorted(selected):
        package = packages[key]
        if package['source'] is None:
            continue
        source = Path(package['manifest_path']).parent
        files = [p for p in source.iterdir() if p.is_file() and p.name.upper().startswith(('LICENSE', 'LICENCE', 'COPYING', 'COPYRIGHT', 'NOTICE'))]
        if not files and (package['name'], package['version']) == ('r-efi', '6.0.0'):
            files = [source/'AUTHORS']
        if not files:
            raise ValueError('Missing upstream notices: '+package['name'])
        target = output/(package['name']+'-'+package['version'])
        target.mkdir(exist_ok=True)
        for file in files:
            shutil.copy2(file, target/file.name)
        inventory.append({key:package[key] for key in ('name','version','license','repository')})
    for name in ['LICENSE', 'COPYRIGHT']:
        shutil.copy2(ROOT/name, output/name)
    from rar_mapping_materials import collect
    collect(output)
    from brotli_data_materials import collect as collect_brotli_data
    collect_brotli_data(output)
    (output/'rust-dependencies.json').write_text(json.dumps(inventory, indent=2)+'\n')


def package(args):
    if platform.system() == 'Darwin' and macos_arch() != 'arm64':
        raise ValueError('Intel CI builds are not eligible for packaging')
    directory = generated(args.build_dir)
    output = generated(args.output)
    output.mkdir(parents=True, exist_ok=True)
    identity = json.loads((directory/'identity.json').read_text())
    actual = json.loads(run([directory/'bin'/exe('xdzip'), '--version-json'], capture=True))
    if actual != identity:
        raise ValueError('CLI identity changed after build')
    verification = directory/'verification/report.json'
    verified = json.loads(verification.read_text())
    if (verified.get('status') != 'passed' or verified.get('identity') != identity
            or verified.get('executable_sha256') != digest(directory/'bin'/exe('xdzip'))
            or verified.get('worker_sha256') != digest(directory/'bin'/exe('xdzip-native-codec'))):
        raise ValueError('Run CLI test against these exact binaries before packaging')
    payload = output/'payload'
    if payload.exists():
        raise ValueError('Use a fresh CLI package output')
    shutil.copytree(directory/'bin', payload/'bin')
    if os.name == 'nt':
        runtime = args.runtime or ROOT/'build/windows-runtime/runtime'
        config = json.loads((ROOT/'scripts/windows-runtime.json').read_text())
        for name, expected in config['files'].items():
            if digest(runtime/name) != expected:
                raise ValueError('Prepare the pinned app-local VC++ runtime first')
            shutil.copy2(runtime/name, payload/'bin'/name)
        if digest(runtime/'Microsoft-VC-runtime-license.rtf') != config['license_sha256']:
            raise ValueError('Missing original VC++ runtime terms')
        (payload/'notices').mkdir()
        shutil.copy2(runtime/'Microsoft-VC-runtime-license.rtf', payload/'notices')
    notices(payload/'notices')
    files = []
    for path in sorted(payload.rglob('*')):
        if path.is_symlink():
            raise ValueError('CLI payload cannot contain links')
        if path.is_file():
            files.append(dict(path=path.relative_to(payload).as_posix(), sha256=digest(path), executable=path.name in [exe(n) for n in ['xdzip','xdzip-launcher','xdzip-native-codec','xdzip-sfx']]))
    (payload/'cli-payload.json').write_text(json.dumps(dict(schema_version=1,product='xdzip',identity=identity,files=files), indent=2)+'\n')
    target = {'Darwin':'macos-arm64.tar.gz','Windows':'windows-x86_64.zip','Linux':'linux-x86_64.tar.gz'}[platform.system()]
    archive = output/f'xdzip-cli-{identity["version"]}-{target}'
    if target.endswith('.zip'):
        with zipfile.ZipFile(archive, 'w', zipfile.ZIP_DEFLATED, strict_timestamps=False) as writer:
            for path in sorted(payload.rglob('*')):
                if path.is_file():
                    writer.write(path, path.relative_to(payload))
    else:
        with tarfile.open(archive, 'w:gz', format=tarfile.PAX_FORMAT) as writer:
            for path in sorted(payload.rglob('*')):
                writer.add(path, arcname=path.relative_to(payload), recursive=False)
    receipt = dict(schema_version=1,platform=target.split('-')[0],identity=identity,file=archive.name,bytes=archive.stat().st_size,sha256=digest(archive),verification_sha256=digest(verification),source=dict(commit=run(['git','rev-parse','HEAD'],capture=True).strip(),dirty=bool(run(['git','status','--porcelain','--untracked-files=normal'],capture=True))))
    (output/'cli-package.json').write_text(json.dumps(receipt, indent=2)+'\n')
    print('CLI package prepared: '+archive.name)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('stage', choices=['build','test','bundle','package'])
    parser.add_argument('--build-dir', type=Path, default=ROOT/'build/cli')
    parser.add_argument('--profile', choices=['debug', 'release', 'relwithdebinfo'], default='release')
    parser.add_argument('--codec', type=Path)
    parser.add_argument('--output', type=Path)
    parser.add_argument('--runtime', type=Path)
    parser.add_argument('--install-fixture', action='store_true')
    args = parser.parse_args()
    if args.stage == 'build':
        build(args)
    elif args.stage == 'test':
        run(['python3' if os.name != 'nt' else 'python', ROOT/'scripts/check-cli.py', '--binary', generated(args.build_dir)/'bin'/exe('xdzip'), '--worker', generated(args.build_dir)/'bin'/exe('xdzip-native-codec'), '--output', args.output or generated(args.build_dir)/'verification', *(['--install-fixture'] if args.install_fixture else [])])
    elif args.output is None:
        parser.error('--output is required for bundle/package')
    elif args.stage == 'bundle':
        bundle(args)
    else:
        package(args)


if __name__ == '__main__':
    main()
