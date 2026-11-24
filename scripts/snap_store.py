#!/usr/bin/env python3
"""Publish and verify XDZip's exact Snap Store revision from GitHub Actions."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import time
from urllib.request import Request, urlopen


def read_candidate(directory, source_revision):
    packages = list(directory.glob('*.snap'))
    if len(packages) != 1:
        raise ValueError('Expected one Snap in the build artifact')
    package = packages[0]
    receipt = json.loads(package.with_suffix('.json').read_bytes())
    version = receipt['version_json']['version']
    if (not re.fullmatch(r'[0-9][0-9A-Za-z.+~-]{0,31}', version) or
            package.name != f'xdzip_{version}_amd64.snap' or receipt['filename'] != package.name or
            receipt['source_revision'] != source_revision or receipt['version_json']['dirty'] is not False or
            receipt['base'] != 'core24' or receipt['confinement'] != 'strict' or receipt['grade'] != 'stable'):
        raise ValueError('Snap receipt does not match the selected source and package identity')
    sha256 = hashlib.sha256()
    sha3 = hashlib.sha3_384()
    with package.open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            sha256.update(block)
            sha3.update(block)
    if receipt['sha256'] != sha256.hexdigest() or receipt['bytes'] != package.stat().st_size:
        raise ValueError('Snap bytes do not match the build receipt')
    return package, version, sha256.hexdigest(), sha3.hexdigest()


def channel_entry(risk):
    request = Request('https://api.snapcraft.io/v2/snaps/info/xdzip', headers={
        'Snap-Device-Series': '16', 'Snap-Device-Architecture': 'amd64',
    })
    with urlopen(request, timeout=15) as response:
        entries = json.load(response)['channel-map']
    for entry in entries:
        channel = entry['channel']
        if (channel['track'] == 'latest' and channel['risk'] == risk and
                channel['architecture'] == 'amd64' and not channel.get('branch')):
            return entry
    return None


def matches(entry, sha3, revision=None):
    return (entry is not None and entry['download']['sha3-384'] == sha3 and
            type(entry['revision']) is int and entry['revision'] > 0 and
            (revision is None or entry['revision'] == revision))


def wait_for_channel(risk, sha3, revision=None):
    for attempt in range(12):
        if attempt:
            time.sleep(5)
        try:
            entry = channel_entry(risk)
        except OSError:
            continue
        if matches(entry, sha3, revision):
            return entry
    raise ValueError(f'Store did not report the expected Snap on latest/{risk}')


def summary(message):
    print(message)
    with Path(os.environ['GITHUB_STEP_SUMMARY']).open('a', encoding='utf-8') as stream:
        stream.write(message + '\n')


def publish_beta(directory, source_revision):
    package, version, sha256, sha3 = read_candidate(directory, source_revision)
    entry = channel_entry('beta')
    if not matches(entry, sha3):
        subprocess.run(['snapcraft', 'upload', str(package), '--release=latest/beta'], check=True)
        entry = wait_for_channel('beta', sha3)
    if entry['version'] != version:
        raise ValueError('The Store version differs from the candidate receipt')
    revision = entry['revision']
    with Path(os.environ['GITHUB_OUTPUT']).open('a', encoding='utf-8') as stream:
        stream.write(f'revision={revision}\nsha3={sha3}\n')
    summary(f'### XDZip {version} amd64 on `latest/beta`\n\n'
            f'- Store revision: **{revision}**\n- Source: `{source_revision}`\n- SHA-256: `{sha256}`\n\n'
            'Install or refresh `xdzip` from `latest/beta` in VMware and confirm the revision with '
            '`snap list xdzip`. Approve the `snap-stable` deployment after acceptance.')


def publish_stable(revision, sha3):
    if revision < 1 or not re.fullmatch(r'[0-9a-f]{96}', sha3):
        raise ValueError('Expected a Store revision and its SHA3-384 from the beta job')
    if not matches(channel_entry('beta'), sha3, revision):
        raise ValueError('The beta channel changed while this revision awaited acceptance')
    if not matches(channel_entry('stable'), sha3, revision):
        subprocess.run(['snapcraft', 'release', 'xdzip', str(revision), 'latest/stable'], check=True)
        wait_for_channel('stable', sha3, revision)
    summary(f'### XDZip amd64 revision {revision} verified on `latest/stable`\n')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='command', required=True)
    beta = commands.add_parser('beta')
    beta.add_argument('directory', type=Path)
    beta.add_argument('source_revision')
    stable = commands.add_parser('stable')
    stable.add_argument('revision', type=int)
    stable.add_argument('sha3')
    args = parser.parse_args()
    if not os.environ.get('SNAPCRAFT_STORE_CREDENTIALS'):
        parser.error(f'Configure SNAPCRAFT_STORE_CREDENTIALS in the snap-{args.command} environment')
    try:
        if args.command == 'beta':
            publish_beta(args.directory, args.source_revision)
        else:
            publish_stable(args.revision, args.sha3)
    except (KeyError, OSError, ValueError, subprocess.CalledProcessError) as error:
        raise SystemExit(str(error)) from None


if __name__ == '__main__':
    main()
