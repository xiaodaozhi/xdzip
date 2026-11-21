#!/usr/bin/env python3
"""Package a checked, platform-specific marketing export without modifying its contents."""
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import zipfile

ROOT = Path(__file__).resolve().parents[1]
PLATFORMS = ('windows', 'macos', 'linux')


def digest(data):
    return hashlib.sha256(data).hexdigest()


def export_files(directory, platform):
    directory = directory.resolve(strict=True)
    if not directory.is_dir():
        raise ValueError('Input must be a generated export directory')
    sums_path = directory / 'SHA256SUMS.txt'
    manifest_path = directory / 'manifest.json'
    if sums_path.is_symlink() or manifest_path.is_symlink():
        raise ValueError('Export metadata cannot be symlinks')
    sums = sums_path.read_bytes()
    expected = {}
    for line in sums.decode('utf-8').splitlines():
        match = re.fullmatch(r'([0-9a-f]{64})  (.+)', line)
        if not match:
            raise ValueError('Invalid SHA256SUMS.txt entry')
        sha, name = match.groups()
        relative = PurePosixPath(name)
        if (relative.is_absolute() or '\\' in name or relative.as_posix() != name
                or any(part in ('', '.', '..') for part in relative.parts)
                or name in expected or name == 'SHA256SUMS.txt'):
            raise ValueError('Invalid or duplicate export path')
        expected[name] = sha
    if 'manifest.json' not in expected:
        raise ValueError('Manifest must be covered by SHA256SUMS.txt')
    files = {}
    for file in directory.rglob('*'):
        if file.is_symlink():
            raise ValueError('Export cannot contain symlinks')
        if file.is_file():
            name = file.relative_to(directory).as_posix()
            if name != 'SHA256SUMS.txt':
                files[name] = file
        elif not file.is_dir():
            raise ValueError('Export contains an unsupported file type')
    if set(files) != set(expected):
        raise ValueError('Export files do not match SHA256SUMS.txt')
    data = {}
    for name, file in sorted(files.items()):
        content = file.read_bytes()
        if digest(content) != expected[name]:
            raise ValueError(f'Export digest differs: {name}')
        if name.lower().endswith('.png') and (len(content) < 24 or content[:8] != b'\x89PNG\r\n\x1a\n'
                                               or content[12:16] != b'IHDR'):
            raise ValueError(f'Invalid PNG header: {name}')
        data[name] = content
    data['SHA256SUMS.txt'] = sums
    manifest = json.loads(data['manifest.json'])
    # Older verified Windows capture exports carry an MSIX identity, not a platform field.
    source_platform = manifest.get('platform')
    if source_platform is None and 'msix_version' in manifest and 'baseline' in manifest:
        source_platform = 'windows'
    if manifest.get('schema_version') != 1 or source_platform != platform:
        raise ValueError('Export manifest has the wrong platform or schema')
    assets = manifest.get('assets', manifest.get('files'))
    if not isinstance(assets, list):
        raise ValueError('Export manifest has no file inventory')
    listed = {}
    for entry in assets:
        name = entry.get('file')
        if not isinstance(name, str) or name in listed or name not in data:
            raise ValueError('Invalid manifest file entry')
        if entry.get('sha256') != digest(data[name]) or entry.get('bytes') != len(data[name]):
            raise ValueError(f'Manifest file differs: {name}')
        listed[name] = entry
    if set(listed) != set(expected) - {'manifest.json'}:
        raise ValueError('Manifest and checksums describe different files')
    if not any(name.lower().endswith('.png') for name in listed):
        raise ValueError('Export has no PNG artwork')
    captured = manifest.get('captured_languages', [])
    if not isinstance(captured, list):
        raise ValueError('Invalid captured languages')
    if manifest.get('status') == 'backgrounds-only' and captured:
        raise ValueError('Background-only export cannot claim captured languages')
    return data, manifest, bool(captured)


def package(directory, platform, destination):
    if platform not in PLATFORMS:
        raise ValueError('Unsupported platform')
    data, manifest, captured = export_files(directory, platform)
    destination = destination.resolve()
    source = directory.resolve(strict=True)
    if destination == source or destination.is_relative_to(source):
        raise ValueError('Package output must not be inside its export source')
    if destination.is_relative_to(ROOT) and not any(destination == base or destination.is_relative_to(base)
            for base in (ROOT / 'build', ROOT / 'dist')):
        raise ValueError('Package output must be under build/ or dist/, or outside the repository')
    destination.mkdir(parents=True, exist_ok=True)
    identity = digest(data['manifest.json'])[:16]
    kind = 'captures' if captured else 'backgrounds'
    name = f'xdzip-{platform}-marketing-{kind}-{identity}.zip'
    target = destination / name
    # Exclusive creation prevents accidental replacement; remove only our own incomplete output.
    with target.open('xb') as stream:
        try:
            with zipfile.ZipFile(stream, 'w', compression=zipfile.ZIP_DEFLATED, compresslevel=6) as archive:
                for name, content in sorted(data.items()):
                    info = zipfile.ZipInfo(name, date_time=(1980, 1, 1, 0, 0, 0))
                    info.compress_type = zipfile.ZIP_DEFLATED
                    info.external_attr = 0o100644 << 16
                    archive.writestr(info, content)
            stream.flush()
        except BaseException:
            target.unlink(missing_ok=True)
            raise
    try:
        with zipfile.ZipFile(target) as archive:
            if sorted(archive.namelist()) != sorted(data) or archive.testzip() is not None:
                raise ValueError('Package inventory or CRC verification failed')
            for name, content in data.items():
                if digest(archive.read(name)) != digest(content):
                    raise ValueError(f'Package digest differs: {name}')
    except BaseException:
        target.unlink(missing_ok=True)
        raise
    package_sha = digest(target.read_bytes())
    print(json.dumps({'platform': platform, 'file': target.name, 'bytes': target.stat().st_size,
                      'sha256': package_sha, 'captured_languages': manifest.get('captured_languages', []),
                      'status': kind}, ensure_ascii=False))
    return target


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--platform', choices=PLATFORMS, required=True)
    parser.add_argument('--input', type=Path, required=True, help='completed marketing export directory')
    parser.add_argument('--output-dir', type=Path, default=ROOT / 'build/marketing-packages')
    args = parser.parse_args()
    package(args.input, args.platform, args.output_dir)


if __name__ == '__main__':
    try:
        main()
    except (OSError, ValueError, UnicodeError, json.JSONDecodeError, zipfile.BadZipFile) as error:
        raise SystemExit(str(error)) from error
