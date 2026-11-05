#!/usr/bin/env python3
"""Compare an owned CI installation to the immutable published portable package."""
from pathlib import Path, PurePosixPath
import hashlib
import json
import os
import sys
import urllib.request
import zipfile

if os.environ.get('GITHUB_ACTIONS') != 'true' or os.environ.get('RUNNER_OS') != 'Windows':
    raise ValueError('Disposable GitHub-hosted Windows runner required')
root, records = (Path(arg).resolve() for arg in sys.argv[1:])
runner = Path(os.environ['RUNNER_TEMP']).resolve()
if not root.is_relative_to(runner) or root == runner or not root.is_dir():
    raise ValueError('Installation is not in the owned runner directory')
name = 'xdzip-0.1.0-alpha.9-windows-x86_64.zip'
url = 'https://github.com/xiaodaozhi/xdzip/releases/download/v0.1.0-alpha.9/' + name
with urllib.request.urlopen(url, timeout=90) as response:
    data = response.read(128*1024*1024+1)
digest = hashlib.sha256(data).hexdigest()
if len(data) > 128*1024*1024 or digest != '932fd4b08f083c460d73cdbb226ff40b0f4792a3172a8a4b8a106f322df17bfa':
    raise ValueError('Published archive identity differs')
import io
count = 0
with zipfile.ZipFile(io.BytesIO(data)) as archive:
    seen = set()
    for item in archive.infolist():
        name = PurePosixPath(item.filename)
        if item.is_dir():
            continue
        if not item.filename or name.is_absolute() or '..' in name.parts or '\\' in item.filename or ':' in item.filename or item.filename in seen:
            raise ValueError('Unsafe or duplicate public archive member')
        seen.add(item.filename)
        path = root.joinpath(*name.parts)
        if not path.resolve().is_relative_to(root) or not path.is_file() or path.is_symlink():
            raise ValueError('Installed member missing or unsafe')
        if hashlib.sha256(path.read_bytes()).digest() != hashlib.sha256(archive.read(item)).digest():
            raise ValueError('Installed payload member differs: ' + item.filename)
        count += 1
if count == 0:
    raise ValueError('Unexpected published payload member count')
blocked = {'d3dcompiler_47.dll', 'dxcompiler.dll', 'dxil.dll'}
if any(path.name.casefold() in blocked for path in root.rglob('*') if path.is_file()):
    raise ValueError('Retired SDK graphics compiler remains in installation')
report = {'status': 'passed', 'public_archive_sha256': digest, 'matched_payload_files': count,
          'limits': ['Published portable payload compared to an isolated updated installation.',
                     'Extra owned user file and installer metadata were separately checked.']}
(records/'payload.json').write_text(json.dumps(report, indent=2)+'\n', encoding='utf-8')
print(f'All {count} published payload files match')
