#!/usr/bin/env python3
"""Compare an independently extracted Inno app payload to a portable ZIP."""
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import zipfile

BLOCKED = {'d3dcompiler_47.dll', 'dxcompiler.dll', 'dxil.dll'}


def digest(stream):
    result = hashlib.sha256()
    for block in iter(lambda: stream.read(1024 * 1024), b''):
        result.update(block)
    return result.hexdigest()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--installer-app', type=Path, required=True)
    parser.add_argument('--portable', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    root = args.installer_app.resolve()
    if not root.is_dir() or args.output.exists():
        raise ValueError('Expected an extracted app and a new output path')
    installed = {}
    for path in root.rglob('*'):
        if path.is_symlink():
            raise ValueError('Link in extracted installer payload')
        if not path.is_file():
            continue
        name = path.relative_to(root).as_posix().casefold()
        if name in installed or path.name.casefold() in BLOCKED:
            raise ValueError('Duplicate or excluded installer member')
        with path.open('rb') as stream:
            installed[name] = digest(stream)
    portable = {}
    with zipfile.ZipFile(args.portable) as archive:
        if archive.testzip() is not None:
            raise ValueError('Portable archive CRC mismatch')
        for member in archive.infolist():
            if member.is_dir():
                continue
            path = PurePosixPath(member.filename)
            name = member.filename.casefold()
            if (not path.parts or path.is_absolute() or '..' in path.parts or
                    '\\' in member.filename or name in portable or path.name.casefold() in BLOCKED):
                raise ValueError('Unsafe, duplicate or excluded portable member')
            with archive.open(member) as stream:
                portable[name] = digest(stream)
    missing = sorted(set(portable) - set(installed))
    extra = sorted(set(installed) - set(portable))
    different = sorted(name for name in portable.keys() & installed.keys()
                       if portable[name] != installed[name])
    manifest = root/'install-manifest.txt'
    if extra != ['install-manifest.txt'] or not manifest.is_file() or manifest.stat().st_size > 2 * 1024 * 1024:
        raise ValueError('Unexpected installer-only members or missing install manifest')
    lines = manifest.read_text(encoding='utf-8-sig').splitlines()
    if not lines or lines[0] != 'xdzip-install-files-v1':
        raise ValueError('Unexpected install manifest header')
    recorded = {}
    for line in lines[1:]:
        sha, sep, raw = line.partition('\t')
        name = raw.replace('\\', '/').casefold()
        if not sep or not re.fullmatch('[0-9a-f]{64}', sha) or not name or name in recorded:
            raise ValueError('Invalid install manifest entry')
        recorded[name] = sha
    if recorded != portable:
        raise ValueError('Install manifest differs from portable payload')
    if not {'xdzip.exe', 'qt6core.dll', 'xdzip-update-helper.exe'} <= set(portable):
        raise ValueError('Missing required product files')
    report = {'portable_files': len(portable), 'installer_app_files': len(installed),
              'missing': missing, 'extra': extra, 'different': different,
              'excluded_sdk_graphics_found': False,
              'scope': 'static extracted payload comparison, not installer or update execution'}
    args.output.write_text(json.dumps(report, indent=2)+'\n', encoding='utf-8')
    print(json.dumps(report, indent=2))
    if missing or different:
        raise SystemExit(1)


if __name__ == '__main__':
    main()
