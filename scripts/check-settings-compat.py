#!/usr/bin/env python3
"""Compile actual previous/current preferences sources, then round-trip an owned profile."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import urllib.request
from deployment_audit import digest, within

ROOT = Path(__file__).resolve().parents[1]
BASELINE = '4cf0f8ca44bac3c0deaadbdaf5594504c5a6afa6'
SOURCES = {
    'UiPreferences.cpp': 'b1f8a14a82d4a29ecf23a35ebf92a7f7f2d2c8279a6c46c1d617ec6107578a0b',
    'UiPreferences.h': '445035a7d5b6aa4751ba533100616c3db3267269487663be512e2b77984a0972',
    'UiText.h': '637ba60110ab539103d807b158a76cfa79a73ba07bc3d40d8cb1b2fdc96208d3',
}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--qt-prefix', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    if not within(output, ROOT/'build') or output.exists():
        parser.error('Use a new build/ output directory')
    previous = output/'previous'
    previous.mkdir(parents=True)
    for name, expected in SOURCES.items():
        result = subprocess.run(['git', 'show', BASELINE+':apps/desktop/'+name], cwd=ROOT, capture_output=True)
        data = result.stdout if result.returncode == 0 else urllib.request.urlopen(
            'https://raw.githubusercontent.com/xiaodaozhi/xdzip/'+BASELINE+'/apps/desktop/'+name, timeout=30).read()
        if hashlib.sha256(data).hexdigest() != expected:
            raise ValueError('Previous revision source checksum mismatch')
        (previous/name).write_bytes(data)
    build = output/'compiled'
    log = output/'compile.log'
    with log.open('w') as stream:
        for command in [
            ['cmake', '-S', str(ROOT/'apps/desktop/tests/preferences-compat'), '-B', str(build), '-G', 'Ninja',
             '-DCMAKE_BUILD_TYPE=Release', '-DCMAKE_PREFIX_PATH='+str(args.qt_prefix.resolve()),
             '-Dprevious_SOURCE='+previous.as_posix(), '-Dcurrent_SOURCE='+(ROOT/'apps/desktop').as_posix()],
            ['cmake', '--build', str(build), '--parallel', '3'],
        ]:
            try:
                subprocess.run(command, stdout=stream, stderr=stream, check=True)
            except subprocess.CalledProcessError:
                stream.flush()
                detail = '\n'.join(log.read_text().splitlines()[-30:])
                print(detail.replace(str(ROOT), '<workspace>').replace(str(args.qt_prefix.resolve()), '<qt-sdk>'))
                raise SystemExit('Settings compatibility compilation failed; see compile.log')
    profile = output/'settings.ini'
    environment = dict(os.environ, XDZIP_SETTINGS_PATH=str(profile), QT_QPA_PLATFORM='offscreen')
    suffix = '.exe' if os.name == 'nt' else ''
    def invoke(revision, action=None):
        command = [str(build/('preferences-'+revision+suffix))] + ([action] if action else [])
        return json.loads(subprocess.check_output(command, env=environment, text=True))
    dark = invoke('previous', '--write-dark')
    if not dark['saved'] or invoke('current') != dict(dark, saved=False):
        raise ValueError('Previous profile was not restored by current code')
    light = invoke('current', '--write-light')
    if not light['saved'] or invoke('previous') != dict(light, saved=False):
        raise ValueError('Same-schema profile cannot be read after rollback')
    if not invoke('previous', '--write-dark')['saved'] or invoke('current')['theme'] != 'dark':
        raise ValueError('Round-trip profile could not be saved after rollback')
    future = b'[General]\nschemaVersion=2\n[future]\nkeep=owned-canary\n'
    profile.write_bytes(future)
    if invoke('current', '--write-light')['saved'] or profile.read_bytes() != future:
        raise ValueError('Future profile schema was downgraded')
    report = {'status': 'passed', 'previous_revision': BASELINE,
              'current_revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
              'executables': {r: digest(build/('preferences-'+r+suffix)) for r in ['previous', 'current']},
              'cases': ['previous-save-current-read', 'current-save-previous-read', 'rollback-save-current-read', 'future-schema-write-refused'],
              'limits': 'Two real source revisions of the settings component, same Qt SDK and schema 1. Not two complete historical installers or a schema-2 migration.'}
    (output/'settings-compat.json').write_text(json.dumps(report, indent=2)+'\n')
    print('Previous/current settings binaries passed profile round-trip and future-schema refusal')


if __name__ == '__main__':
    main()
