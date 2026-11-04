#!/usr/bin/env python3
"""Upgrade the public alpha.8 installer to a candidate on disposable Windows CI."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import winreg

BLOCKED = {'d3dcompiler_47.dll', 'dxcompiler.dll', 'dxil.dll'}
OLD_HASH = '246e0107c4d1e0c2c64f296042bfcafc4e8f92768c76b50cc9fc80c065a10600'
NEW_REVISION = '8b277ac41856'


def digest(path):
    value = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            value.update(block)
    return value.hexdigest()


def invoke(args, **kwargs):
    return subprocess.run([str(x) for x in args], check=True, timeout=240, **kwargs)


def identity(executable):
    return json.loads(invoke([executable, '--version-json'], capture_output=True, text=True).stdout)


def main():
    public_probe_only = sys.argv[1:] == ['--public-update-probe-only']
    baseline_probe_only = sys.argv[1:] == ['--baseline-update-probe-only']
    probe_only = public_probe_only or baseline_probe_only
    if sys.argv[1:] and not probe_only:
        raise RuntimeError('Unknown arguments')
    if (sys.platform != 'win32' or os.environ.get('CI') != 'true' or
            os.environ.get('GITHUB_ACTIONS') != 'true' or
            os.environ.get('RUNNER_ENVIRONMENT') != 'github-hosted'):
        raise RuntimeError('Disposable GitHub-hosted Windows runner required')
    for key in (r'Software\xdzip\Install', r'Software\Classes\xdzip.Archive'):
        try:
            with winreg.OpenKey(winreg.HKEY_CURRENT_USER, key):
                raise RuntimeError('Existing xdzip install or association; refusing to continue')
        except FileNotFoundError:
            pass
    base = Path(os.environ['RUNNER_TEMP']).resolve()
    root = base/'issue216-original-upgrade'
    packages = base/'issue216-packages'
    old = packages/'alpha8-setup.exe'
    new = packages/'xdzip-0.1.0-alpha.9-windows-x86_64-setup.exe'
    if (root.exists() or not old.is_file() or digest(old) != OLD_HASH or
            (not probe_only and not new.is_file())):
        raise RuntimeError('Unexpected input or occupied owned directory')
    output = Path('build/issue216-upgrade-review')
    output.mkdir(parents=True, exist_ok=False)
    result = {'status': 'failed', 'scope': ('original installed client live discovery; no installation'
              if probe_only else 'actual installers, direct upgrade; not client-mediated update'),
              'old_installer_sha256': digest(old)}
    if not probe_only:
        result['candidate_installer_sha256'] = digest(new)
    try:
        def install(path):
            invoke([path, '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART', '/DIR='+str(root)])
        install(old)
        original = identity(root/'xdzip.exe')
        if original['version'] != '0.1.0-alpha.8' or original['build'] != 21:
            raise RuntimeError('Unexpected original installed identity')
        probe = base/'issue216-original-update-probe.json'
        settings = base/'issue216-original-update-settings.ini'
        if probe.exists() or settings.exists():
            raise RuntimeError('Unexpected occupied probe output')
        environment = os.environ.copy()
        environment['XDZIP_SETTINGS_PATH'] = str(settings)
        invoke([root/'xdzip.exe', '--update-probe', probe], env=environment,
               capture_output=True, text=True, creationflags=subprocess.CREATE_NO_WINDOW)
        live = json.loads(probe.read_text(encoding='utf-8'))
        expected_state = 'available' if public_probe_only else 'up_to_date'
        if (live.get('version') != '0.1.0-alpha.8' or live.get('build') != 21 or
                live.get('state') != expected_state or live.get('error') or
                not live.get('ssl_supported') or
                (public_probe_only and (live.get('candidate') != '0.1.0-alpha.9' or
                                        live.get('trusted') is not True))):
            raise RuntimeError('Unexpected original installed update probe')
        if probe_only:
            result.update(status='passed', original_installed_update_probe=live)
            return
        old_files = {p.name.casefold() for p in root.rglob('*') if p.is_file()}
        if not BLOCKED <= old_files:
            raise RuntimeError('Expected original SDK DLLs absent')
        sentinel = root/'issue216-owned-user-file.txt'
        sentinel.write_bytes(b'owned upgrade preservation fixture\n')
        install(new)
        current = identity(root/'xdzip.exe')
        if (current['version'] != '0.1.0-alpha.9' or current['build'] != 22 or
                current['revision'] != NEW_REVISION or current['dirty']):
            raise RuntimeError('Unexpected candidate installed identity')
        remaining = sorted({p.name.casefold() for p in root.rglob('*') if p.is_file()} & BLOCKED)
        if remaining or sentinel.read_bytes() != b'owned upgrade preservation fixture\n':
            raise RuntimeError('Old SDK DLL or user-file preservation check failed')
        result.update(status='passed', original={'version': original['version'], 'build': original['build']},
                      candidate={'version': current['version'], 'build': current['build'],
                                 'revision': current['revision'], 'dirty': current['dirty']},
                      original_installed_update_probe=live,
                      original_sdk_dlls_present=True, candidate_sdk_dlls_remaining=remaining,
                      owned_user_file_preserved=True)
    finally:
        (output/'result.json').write_text(json.dumps(result, indent=2)+'\n', encoding='utf-8')
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
