#!/usr/bin/env python3
"""Capture and prepare both languages of Microsoft Store imagery on Windows."""

import argparse
from datetime import datetime
import hashlib
import json
from pathlib import Path
import re
import subprocess
import tempfile
from zipfile import ZipFile


ROOT = Path(__file__).resolve().parents[1]
CAPTURE = Path('specs/capture/windows-store.json')
MARKETING = Path('specs/marketing/windows-store.json')
CAPTURES = Path('specs/marketing/windows-store-captures.json')
ASSETS = Path('specs/marketing/windows-store-assets.json')
LANGUAGES = ('en-US', 'zh-CN')
LISTINGS = (Path('developers/market/windows/listing.md'), Path('developers/zh-CN/market/windows/listing.md'))


def read_json(path):
    return json.loads(path.read_text(encoding='utf-8-sig'))


def write_json(path, data):
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + '\n', encoding='utf-8', newline='\n')


def digest(stream):
    return hashlib.file_digest(stream, 'sha256').hexdigest()


def verified_package(directory):
    package = directory / 'xdzip-windows-x64.msix'
    manifest = read_json(directory / 'windows-msix.json')
    build = read_json(directory / 'package-msix.json')
    verify = read_json(directory / 'verify.json')
    with package.open('rb') as stream:
        package_sha = digest(stream)
    artifact = {'file': package.name, 'sha256': package_sha, 'bytes': package.stat().st_size}
    source = build['source']
    if (manifest['status'] != 'passed' or manifest['channel'] != 'store'
            or manifest['package'] != package.name or manifest['sha256'] != package_sha
            or not re.fullmatch(r'1\.0\.[1-9][0-9]*\.0', manifest['version'])
            or not re.fullmatch(r'[0-9a-f]{40}', source['revision']) or source['dirty'] is not False
            or build['status'] != 'passed' or verify['status'] != 'passed'
            or verify['stage'] != 'verify' or verify['source'] != source
            or not all(artifact in entry['artifacts'] for entry in (build, verify))):
        raise ValueError('The MSIX, manifest and package verification receipts do not match')
    with ZipFile(package) as archive, archive.open('xdzip.exe') as executable:
        executable_sha = digest(executable)
    return package, {
        'revision': source['revision'], 'package_version': manifest['version'],
        'runtime_exe_sha256': executable_sha, 'package_sha256': package_sha,
    }


def capture_pair(package, baseline, output, configuration):
    runtime = output / 'runtime'
    with ZipFile(package) as archive:
        archive.extractall(runtime)
    executable = runtime / 'xdzip.exe'
    with executable.open('rb') as stream:
        if digest(stream) != baseline['runtime_exe_sha256']:
            raise ValueError('Extracted xdzip executable differs from the verified package')
    directories = [output / language for language in LANGUAGES]
    # Keep the example destination short enough for the default creation window.
    with tempfile.TemporaryDirectory(prefix='', dir=ROOT / 'build') as demo:
        for language, directory, folder in zip(LANGUAGES, directories, ('Pics', '图片')):
            subprocess.run([
                'powershell.exe', '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File',
                str(ROOT / 'scripts/capture-windows-store.ps1'), '-Language', language,
                '-Configuration', str(configuration), '-Executable', str(executable),
                '-ExpectedSha256', baseline['runtime_exe_sha256'], '-SourceRevision', baseline['revision'],
                '-OutputDirectory', str(directory), '-CreationDemoDirectory', str(Path(demo) / folder),
            ], cwd=ROOT, check=True)
    return directories


def store_canvas(directories, expected_dpi, canvas, trim):
    images = []
    for directory, language in zip(directories, LANGUAGES):
        receipt = read_json(directory / 'capture-session.json')
        if receipt.get('schema_version') != 3 or receipt.get('complete') is not True or receipt['language'] != language:
            raise ValueError(f'{language} capture session is incomplete or has the wrong language')
        images.extend(receipt['images'])
    if not images or any(image['dpi'] != expected_dpi for image in images):
        actual = sorted({image['dpi'] for image in images})
        raise ValueError(f'Capture DPI is {actual}; expected {expected_dpi}. Capture at the selected Windows display scale.')
    width = max(image['width'] for image in images)
    height = max(image['height'] for image in images) - trim
    for candidate in (canvas, [2560, 1440], [3840, 2160]):
        if width <= candidate[0] and height <= candidate[1]:
            return candidate
    raise ValueError(f'Native window {width} × {height} exceeds the Store canvas')


def replace_file(destination, content):
    with tempfile.TemporaryDirectory(dir=destination.parent, prefix='.store-') as temporary:
        path = Path(temporary) / destination.name
        path.write_bytes(content)
        path.replace(destination)


def apply_preview(preview, originals):
    changes = {}
    for file, original in originals.items():
        content = (preview / file).read_bytes()
        if content != original:
            changes[file] = content
    applied = []
    try:
        for file, content in changes.items():
            destination = ROOT / file
            if destination.read_bytes() != originals[file]:
                raise ValueError(f'Working file changed during capture: {file}')
            replace_file(destination, content)
            applied.append(file)
    except (OSError, ValueError):
        for file in reversed(applied):
            destination = ROOT / file
            if destination.read_bytes() == changes[file]:
                replace_file(destination, originals[file])
        raise
    return len(changes)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--package-dir', type=Path, required=True, help='verified store-msix-package directory')
    parser.add_argument('--output', type=Path, default=Path('build/screenshots/windows-store') / datetime.now().strftime('%Y%m%d-%H%M%S'))
    parser.add_argument('--dpi', type=int, default=144, help='expected Windows window DPI, 144 for 150%% scaling')
    parser.add_argument('--from-captures', nargs=2, metavar=('EN', 'ZH'), type=Path, help='reuse two completed local sessions')
    parser.add_argument('--apply', action='store_true', help='replace repository assets after the preview passes validation')
    args = parser.parse_args()
    if args.dpi <= 0:
        parser.error('--dpi must be positive')

    subprocess.run(['node', '-e', "require('./scripts/windows-store-sharp.cjs')"], cwd=ROOT, check=True)
    package, baseline = verified_package(args.package_dir.resolve())
    output = args.output.resolve()
    build_directory = ROOT / 'build'
    if output == build_directory or not output.is_relative_to(build_directory):
        parser.error('--output must be a new directory below build/')
    output.mkdir(parents=True, exist_ok=False)
    preview = output / 'staging'
    files = {Path(asset['file']) for asset in read_json(ROOT / ASSETS)['assets']}
    files.update((CAPTURE, MARKETING, CAPTURES, ASSETS, *LISTINGS))
    originals = {file: (ROOT / file).read_bytes() for file in sorted(files)}
    for file in (*originals, Path('VERSION.json'), Path('design/assets/brand/xdzip.svg')):
        destination = preview / file
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes(originals[file] if file in originals else (ROOT / file).read_bytes())

    config = read_json(preview / CAPTURE)
    config['baseline'].update({key: baseline[key] for key in config['baseline']})
    write_json(preview / CAPTURE, config)
    directories = args.from_captures or capture_pair(package, baseline, output, preview / CAPTURE)
    spec = read_json(preview / MARKETING)
    spec['baseline'].update(baseline)
    artwork = spec['screenshot_artwork']
    artwork['store_canvas'] = store_canvas(directories, args.dpi, artwork['store_canvas'], artwork['cleanup']['top_trim_pixels'])
    write_json(preview / MARKETING, spec)
    for script, arguments in (
        ('import-windows-store-captures.cjs', [str(path.resolve()) for path in directories]),
        ('build-windows-store-assets.cjs', []),
    ):
        subprocess.run(['node', str(ROOT / 'scripts' / script), *arguments, '--root', str(preview)], cwd=ROOT, check=True)
    preview = preview.rename(output / 'materials')
    print(f'Generated materials: {preview}')
    if args.apply:
        print(f'Updated {apply_preview(preview, originals)} repository files.')
    else:
        print('Repository files are unchanged. Review the preview first.')


if __name__ == '__main__':
    try:
        main()
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError) as error:
        raise SystemExit(str(error)) from error
