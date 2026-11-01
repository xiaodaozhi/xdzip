#!/usr/bin/env python3
"""Test bounded ad-hoc signature hypotheses on copies of official Qt SDK files."""
import hashlib
import io
import json
from pathlib import Path, PurePosixPath
import shutil
import subprocess
import sys
import tarfile
import tempfile
import urllib.request

sdk, output = Path(sys.argv[1]).resolve(), Path(sys.argv[2]).resolve()
build = (Path.cwd()/'build').resolve()
if sys.platform != 'darwin' or not sdk.is_dir() or output.exists() or not output.is_relative_to(build):
    raise ValueError('Require macOS, an installed Qt SDK and a new output file')
asset = ('https://github.com/xiaodaozhi/xdzip/releases/download/v0.1.0-alpha.8/'
         'xdzip-0.1.0-alpha.8-macos-materials.tar.gz')
with urllib.request.urlopen(asset, timeout=60) as response:
    data = response.read(5*1024*1024+1)
if len(data) > 5*1024*1024 or hashlib.sha256(data).hexdigest() != \
        '1f790160bb227549d35bfeecfefb41f45208aa9179ab53e1cfd5ed9f88507fe4':
    raise ValueError('Published material archive identity mismatch')
with tarfile.open(fileobj=io.BytesIO(data), mode='r:gz') as archive:
    members = [member for member in archive.getmembers() if member.name == 'qt-binary-differences.json']
    if len(members) != 1 or not members[0].isfile() or members[0].size > 1024*1024:
        raise ValueError('Unexpected published Qt analysis member')
    report = json.load(archive.extractfile(members[0]))
if report['identity']['revision'] != 'e1cd6a3e125d' or report['platform'] != 'macos':
    raise ValueError('Unexpected published release identity')
unique = {}
for row in report['files']:
    path = PurePosixPath(row['sdk_path'])
    if path.is_absolute() or '..' in path.parts:
        raise ValueError('Unbounded SDK path')
    sha1 = {entry['expected'] for entry in row['results'] if entry['algorithm'].upper() == 'SHA1'}
    if len(sha1) != 1:
        raise ValueError('Unexpected SDK checksum set')
    pair = (row['sdk_sha256'], next(iter(sha1)))
    if str(path) in unique and unique[str(path)] != pair:
        raise ValueError('Conflicting repeated SDK checksums')
    unique[str(path)] = pair
if len(report['files']) != 110 or len(unique) != 61:
    raise ValueError('Unexpected published difference count')
rows = []
with tempfile.TemporaryDirectory(prefix='qt-adhoc-probe-', dir=output.parent) as temporary:
    temporary = Path(temporary)
    for relative, (expected_sdk, expected_spdx) in sorted(unique.items()):
        original = sdk.joinpath(*PurePosixPath(relative).parts).resolve(strict=True)
        if not original.is_relative_to(sdk) or not original.is_file():
            raise ValueError('SDK image outside the fixed SDK')
        with original.open('rb') as stream:
            if hashlib.file_digest(stream, 'sha256').hexdigest() != expected_sdk:
                raise ValueError('SDK image does not match the published SDK input')
        outcomes = []
        for mode in ('adhoc-replace', 'remove-then-adhoc', 'adhoc-hardened'):
            candidate = temporary/original.name
            shutil.copyfile(original, candidate)
            if mode == 'remove-then-adhoc':
                removed = subprocess.run(['codesign', '--remove-signature', str(candidate)],
                                         capture_output=True, timeout=60)
                if removed.returncode:
                    outcomes.append({'mode': mode, 'status': 'remove-failed'})
                    candidate.unlink()
                    continue
            command = ['codesign', '--force', '--sign', '-', '--timestamp=none']
            if mode == 'adhoc-hardened':
                command += ['--options', 'runtime']
            signed = subprocess.run(command+[str(candidate)], capture_output=True, timeout=60)
            if signed.returncode:
                outcomes.append({'mode': mode, 'status': 'sign-failed'})
            else:
                with candidate.open('rb') as stream:
                    candidate_sha1 = hashlib.file_digest(stream, 'sha1').hexdigest()
                outcomes.append({'mode': mode, 'status': 'exact-match' if candidate_sha1 == expected_spdx
                                 else 'not-matched', 'candidate_sha1': candidate_sha1})
            candidate.unlink()
        rows.append({'sdk_path': relative, 'sdk_sha256': expected_sdk, 'spdx_sha1': expected_spdx,
                     'outcomes': outcomes})
result = {'published_revision': report['identity']['revision'], 'original_records': 110,
          'unique_paths': len(rows), 'files': rows,
          'limits': ['The original publisher signing and build history is not assumed.',
                     'Only exact SHA-1 matches establish the tested byte reconstruction.',
                     'Ad-hoc signatures are applied only to disposable SDK image copies, not an application package.',
                     'A non-match does not rule out other original signatures or transformations.']}
output.write_text(json.dumps(result, indent=2)+'\n', encoding='utf-8')
matches = sum(any(outcome['status'] == 'exact-match' for outcome in row['outcomes']) for row in rows)
print('Qt SDK images tested:', len(rows), 'exact matches:', matches)
