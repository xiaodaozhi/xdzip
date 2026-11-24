"""Transfer native release packages and explicit verification inputs within one Actions run."""
import argparse
import json
import os
from pathlib import Path, PurePosixPath
import platform
import re
import shutil
import subprocess
import sys
import tarfile

from release_protocol import sha256

ROOT = Path(__file__).resolve().parents[1]
BUILD = ROOT / 'build'
REVIEW = BUILD / 'review'
HANDOFF = BUILD / 'release-handoff'
ARCHIVE = 'native-packages.tar.gz'
RECEIPT = 'receipt.json'
PLATFORMS = {'Windows': 'windows', 'Linux': 'linux', 'Darwin': 'macos'}
HANDOFF_OS = {'windows': 'Windows', 'linux': 'Linux', 'macos': 'macOS', 'macos-intel': 'macOS-X64'}
ARCHITECTURES = {'windows': 'X64', 'linux': 'X64', 'macos': 'ARM64', 'macos-intel': 'X64'}
QT_FILES = ('bin/Qt6Test.dll', 'bin/Qt6QuickTest.dll', 'plugins/platforms/qoffscreen.dll')


def package_directory(system):
    return BUILD / 'release-package' if system in ('macos', 'macos-intel') else REVIEW


def identity(system):
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    if (
        os.environ.get('GITHUB_ACTIONS') != 'true'
        or not re.fullmatch(r'[a-f0-9]{40}', revision)
        or not os.environ.get('GITHUB_RUN_ID', '').isdigit()
        or subprocess.check_output(['git', 'status', '--porcelain'], cwd=ROOT, text=True).strip()
    ):
        raise ValueError('Release handoffs require a clean Actions checkout and run identity')
    return {'schema_version': 1, 'repository': os.environ['GITHUB_REPOSITORY'],
            'run_id': os.environ['GITHUB_RUN_ID'], 'revision': revision, 'platform': system,
            'architecture': ARCHITECTURES[system]}


def write_receipt(directory, receipt):
    (directory / RECEIPT).write_text(json.dumps(receipt, indent=2) + '\n', encoding='utf-8')


def read_receipt(directory, system):
    receipt = json.loads((directory / RECEIPT).read_bytes())
    if any(receipt.get(key) != value for key, value in identity(system).items()):
        raise ValueError('Release artifact belongs to a different run, commit, or platform')
    return receipt


def paths_for(system):
    packages = package_directory(system)
    paths = [packages / 'notices', packages / 'runtime-notices', REVIEW / 'sbom-schemas']
    paths.extend(sorted(packages.glob('*.json')))
    for pattern in ('*.deb', '*.dmg', '*.tar.gz', '*.exe', '*.zip'):
        paths.extend(sorted(packages.glob(pattern)))
    if system not in ('macos', 'macos-intel'):
        paths += [REVIEW / system, BUILD / 'desktop/apps/desktop/m2-flat.zip',
                  BUILD / f'installer-tests/{system}-packages', BUILD / 'installer-tests/association-fixtures']
    if system == 'windows':
        paths += [BUILD / 'desktop/apps/desktop/xdzip-browser-tests.exe',
                  BUILD / 'desktop/apps/desktop/xdzip-windows-shell-test.exe',
                  BUILD / 'release-verify-qt']
    return paths


def pack(system):
    if os.environ.get('RUNNER_ARCH') != ARCHITECTURES[system]:
        raise ValueError('Release build requires the expected native architecture')
    HANDOFF.mkdir()
    if system == 'windows':
        qt = Path(os.environ['RUNNER_TEMP']) / 'Qt/6.8.2/msvc2022_64'
        for relative in QT_FILES:
            output = BUILD / 'release-verify-qt' / relative
            output.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(qt / relative, output)
    paths = paths_for(system)
    packages = package_directory(system)
    if not (packages / 'package.json').is_file() or not (packages / 'cli-package.json').is_file():
        raise ValueError('Release build did not produce desktop and CLI package receipts')
    archive = HANDOFF / ARCHIVE
    with tarfile.open(archive, 'w:gz', compresslevel=3) as writer:
        for path in paths:
            writer.add(path, arcname=path.relative_to(ROOT).as_posix())
    write_receipt(HANDOFF, {**identity(system), 'archive_sha256': sha256(archive)})


def unpack(system):
    if os.environ.get('RUNNER_ARCH') != ARCHITECTURES[system]:
        raise ValueError('Release verification requires the expected native architecture')
    receipt = read_receipt(HANDOFF, system)
    archive = HANDOFF / ARCHIVE
    if sha256(archive) != receipt.get('archive_sha256'):
        raise ValueError('Native package handoff checksum mismatch')
    allowed = ('build/review/', 'build/release-package/', 'build/desktop/apps/desktop/', 'build/installer-tests/',
               'build/release-verify-qt/')
    names = set()
    with tarfile.open(archive, 'r:gz') as reader:
        for member in reader.getmembers():
            name = PurePosixPath(member.name)
            target = ROOT / name
            if (not (member.isfile() or member.isdir())
                    or not (str(name) + '/').startswith(allowed) or '..' in name.parts
                    or '\\' in member.name or ':' in member.name
                    or not target.resolve().is_relative_to(BUILD)
                    or target.exists() or target.is_symlink()
                    or name.as_posix().casefold() in names):
                raise ValueError('Invalid or occupied native handoff path')
            names.add(name.as_posix().casefold())
        reader.extractall(ROOT, filter='data')
    print('Verified native handoff SHA-256 ' + receipt['archive_sha256'])


def inventory(directory):
    files = {}
    for path in sorted(directory.iterdir()):
        if path.name == RECEIPT:
            continue
        if path.is_symlink() or not path.is_file():
            raise ValueError('Verified release assets must be regular files')
        files[path.name] = sha256(path)
    if not files:
        raise ValueError('Empty verified release asset directory')
    return files


def stage(system):
    handoff = read_receipt(HANDOFF, system)
    output = BUILD / 'release-assets'
    output.mkdir()
    subprocess.run([sys.executable, 'scripts/prepare-release.py',
                    '--packages', str(package_directory(system)), '--output', str(output), '--platform', system],
                   cwd=ROOT, check=True)
    write_receipt(output, {**identity(system), 'archive_sha256': handoff['archive_sha256'],
                           'verified': inventory(output)})


def collect():
    packages = BUILD / 'packages'
    packages.mkdir()
    for system, label in HANDOFF_OS.items():
        directory = BUILD / 'downloaded' / f'release-verified-{label}'
        receipt = read_receipt(directory, system)
        files = inventory(directory)
        if not re.fullmatch(r'[a-f0-9]{64}', receipt.get('archive_sha256', '')) or files != receipt.get('verified'):
            raise ValueError('Verified platform asset checksum mismatch')
        for name in files:
            if (packages / name).exists():
                raise ValueError('Duplicate verified platform asset')
            shutil.copyfile(directory / name, packages / name)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('operation', choices=('pack', 'unpack', 'stage', 'collect'))
    operation = parser.parse_args().operation
    if operation == 'collect':
        collect()
    else:
        system = PLATFORMS[platform.system()]
        if system == 'macos' and os.environ.get('XDZIP_MACOS_INTEL_CI') == '1':
            system = 'macos-intel'
        {'pack': pack, 'unpack': unpack, 'stage': stage}[operation](system)


if __name__ == '__main__':
    main()
