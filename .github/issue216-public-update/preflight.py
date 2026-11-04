#!/usr/bin/env python3
"""Check published update payload and unchanged license before the UI automation."""
import hashlib
import io
import json
import os
from pathlib import Path
import urllib.request
import zipfile

if (os.environ.get('GITHUB_ACTIONS') != 'true' or os.environ.get('RUNNER_OS') != 'Windows'
        or os.environ.get('RUNNER_ENVIRONMENT') != 'github-hosted'):
    raise RuntimeError('Disposable hosted Windows runner required')

name = 'xdzip-0.1.0-alpha.9-windows-x86_64.zip'
url = 'https://github.com/xiaodaozhi/xdzip/releases/download/v0.1.0-alpha.9/' + name
with urllib.request.urlopen(url, timeout=120) as response:
    data = response.read(128 * 1024 * 1024 + 1)
archive_hash = hashlib.sha256(data).hexdigest()
if len(data) > 128 * 1024 * 1024 or archive_hash != '__PUBLIC_PORTABLE_SHA256__':
    raise RuntimeError('Published portable package identity differs')

blocked = {'d3dcompiler_47.dll', 'dxcompiler.dll', 'dxil.dll'}
with zipfile.ZipFile(io.BytesIO(data)) as archive:
    names = archive.namelist()
    if len(names) != len({name.casefold() for name in names}):
        raise RuntimeError('Duplicate package entry')
    if any(name.replace('\\', '/').rsplit('/', 1)[-1].casefold() in blocked for name in names):
        raise RuntimeError('SDK graphics compiler still present')
    app = archive.read('xdzip.exe')
    license_text = archive.read('share/notices/Microsoft-VC-runtime/license.rtf')
    license_hash = hashlib.sha256(license_text).hexdigest()
    if license_hash != '8099dc3cf9502c335da829e5c755948a12e3e6de490eb492a99deb673d883d8b':
        raise RuntimeError('Runtime license differs from original alpha.8')
    app_hash = hashlib.sha256(app).hexdigest()
    if app_hash != '__PUBLIC_APP_SHA256__':
        raise RuntimeError('Published application identity differs')

records = Path('build/public-update-records')
records.mkdir(parents=True, exist_ok=True)
(records/'preflight.json').write_text(json.dumps({
    'status': 'passed', 'portable_sha256': archive_hash, 'app_sha256': app_hash,
    'runtime_license_sha256': license_hash, 'sdk_graphics_files_absent': True,
}, indent=2)+'\n', encoding='utf-8')
print('Published payload and unchanged runtime license verified')
