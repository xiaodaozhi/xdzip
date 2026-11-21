#!/usr/bin/env python3
"""Record operator-provided native screenshots and the selected xdzip build identity."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import struct
import subprocess
import sys
import tempfile
import zlib
from datetime import datetime, timezone

ROOT = Path(__file__).resolve().parents[1]
LANGUAGES = tuple(item['id'] for item in json.loads((ROOT / 'resources/ui-languages.json').read_text(encoding='utf-8'))['languages'])
SCENES = ('01-browse', '02-create', '03-preview', '04-workspace')


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def png_dimensions(data):
    if len(data) < 33 or data[:8] != b'\x89PNG\r\n\x1a\n' or data[12:16] != b'IHDR':
        raise ValueError('Screenshot must be a PNG with an IHDR')
    width, height = struct.unpack('>II', data[16:24])
    if not width or not height or width * height > 40_000_000:
        raise ValueError('Screenshot dimensions are invalid or too large')
    cursor, seen_data, seen_end = 8, False, False
    while cursor + 12 <= len(data):
        length = struct.unpack('>I', data[cursor:cursor + 4])[0]
        end = cursor + 12 + length
        if end > len(data):
            raise ValueError('Truncated PNG chunk')
        kind = data[cursor + 4:cursor + 8]
        expected = struct.unpack('>I', data[end - 4:end])[0]
        if zlib.crc32(data[cursor + 4:end - 4]) != expected:
            raise ValueError('PNG chunk checksum differs')
        if kind == b'IDAT':
            seen_data = True
        if kind == b'IEND':
            seen_end = length == 0 and end == len(data)
            break
        cursor = end
    if not seen_data or not seen_end:
        raise ValueError('PNG is missing image data or its end marker')
    return width, height


def safe_output(value, target):
    stamp = datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
    output = (value or ROOT / 'build' / f'{target}-store-captures' / stamp).resolve()
    if output == ROOT or (output.is_relative_to(ROOT) and not any(
            output != base and output.is_relative_to(base) for base in (ROOT / 'build', ROOT / 'dist'))):
        raise ValueError('Output must be below build/ or dist/, or outside the repository')
    if output.exists() or output.is_symlink():
        raise ValueError('Output already exists')
    return output


def capture(args):
    languages = getattr(args, 'languages', None) or LANGUAGES
    if len(set(languages)) != len(languages) or any(language not in LANGUAGES for language in languages):
        raise ValueError('Unknown or duplicate language')
    host = {'darwin': 'macos', 'linux': 'linux'}.get(sys.platform)
    if host != args.platform:
        raise ValueError('Capture must be recorded on its native platform')
    executable = args.executable.resolve(strict=True)
    if not executable.is_file() or not re.fullmatch(r'[a-f0-9]{64}', args.expected_sha256):
        raise ValueError('An executable and its expected lowercase SHA-256 are required')
    actual_hash = sha256(executable.read_bytes())
    if actual_hash != args.expected_sha256:
        raise ValueError('Executable digest differs')
    result = subprocess.run([str(executable), '--version-json'], capture_output=True, text=True,
                            check=True, timeout=15)
    identity = json.loads(result.stdout)
    if (identity.get('product') != 'xdzip' or not isinstance(identity.get('version'), str)
            or type(identity.get('build')) is not int or type(identity.get('dirty')) is not bool
            or not re.fullmatch(r'[a-f0-9]{12}', identity.get('revision', ''))):
        raise ValueError('Executable did not provide a complete build identity')
    if args.source_revision and (not re.fullmatch(r'[a-f0-9]{40}', args.source_revision)
                                 or not args.source_revision.startswith(identity['revision'])):
        raise ValueError('Source revision differs from the executable identity')
    source = args.from_directory.resolve(strict=True) if args.from_directory else None
    output = safe_output(args.output, args.platform)
    if source and (output == source or output.is_relative_to(source)):
        raise ValueError('Output must not be inside the input directory')
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='.platform-capture-', dir=output.parent) as temporary:
        stage = Path(temporary) / 'capture'
        stage.mkdir(mode=0o700)
        records = []
        for language in languages:
            for scene in SCENES:
                relative = Path(language) / (scene + '.png')
                destination = stage / relative
                destination.parent.mkdir(exist_ok=True)
                if source:
                    item = source / relative
                    if item.is_symlink() or not item.resolve(strict=True).is_relative_to(source):
                        raise ValueError('Screenshot input escapes its source directory')
                    if item.stat().st_size > 50_000_000:
                        raise ValueError('Screenshot input is too large')
                    data = item.read_bytes()
                else:
                    print(f'{language} / {scene}: prepare the corresponding xdzip window, then press Enter.')
                    input()
                    if args.platform == 'macos':
                        command = ['screencapture', '-i', '-w', '-x', '-o', str(destination)]
                        print('Select the xdzip window in the native capture interface.')
                    else:
                        command = ['gnome-screenshot', '--window', f'--delay={args.delay}', f'--file={destination}']
                        print(f'Activate the xdzip window within {args.delay} seconds.')
                    if shutil.which(command[0]) is None:
                        raise ValueError(f'Required capture tool is unavailable: {command[0]}')
                    subprocess.run(command, check=True)
                    if destination.stat().st_size > 50_000_000:
                        raise ValueError('Screenshot input is too large')
                    data = destination.read_bytes()
                    destination.unlink()
                width, height = png_dimensions(data)
                destination.write_bytes(data)
                records.append({'language': language, 'scene': scene, 'file': relative.as_posix(),
                                'width': width, 'height': height, 'sha256': sha256(data)})
        if sha256(executable.read_bytes()) != actual_hash:
            raise ValueError('Executable changed while recording the screenshots')
        session = {'schema_version': 1, 'platform': args.platform, 'complete': True,
                   'provenance': 'operator-supplied',
                   'capture_method': 'existing-png' if source else 'interactive-window',
                   'limitation': 'The operator selects and reviews the application window and scene; the executable identity is not proof of the screenshot subject.',
                   'identity': {'version': identity['version'], 'build': identity['build'],
                                'revision': identity['revision'], 'source_revision': args.source_revision,
                                'dirty': identity['dirty'], 'executable_sha256': actual_hash,
                                'host_os': platform.system(), 'host_architecture': platform.machine()},
                   'images': records}
        (stage / 'capture-session.json').write_text(json.dumps(session, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
        stage.rename(output)
    label = output.relative_to(ROOT).as_posix() if output.is_relative_to(ROOT) else 'operator-specified external directory'
    print(f'Recorded {len(records)} screenshots and build identity: {label}')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--platform', choices=('macos', 'linux'), required=True)
    parser.add_argument('--executable', type=Path, required=True)
    parser.add_argument('--expected-sha256', required=True)
    parser.add_argument('--source-revision', help='optional full commit matching the executable identity')
    source = parser.add_mutually_exclusive_group(required=True)
    source.add_argument('--from-directory', type=Path, help='existing PNGs organized by language')
    source.add_argument('--interactive', action='store_true', help='select actual windows for the eight scenes')
    parser.add_argument('--languages', nargs='+', choices=LANGUAGES, help='capture selected languages; default is all product languages')
    parser.add_argument('--delay', type=int, default=5, help='Linux seconds to activate the application window')
    parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    if not 2 <= args.delay <= 60:
        parser.error('--delay must be between 2 and 60 seconds')
    capture(args)


if __name__ == '__main__':
    try:
        main()
    except (OSError, ValueError, subprocess.SubprocessError, json.JSONDecodeError) as error:
        raise SystemExit(str(error)) from error
