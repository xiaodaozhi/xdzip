#!/usr/bin/env python3
"""Publish the Snap from the verified GitHub Release directly to latest/stable."""

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


def stable_entry():
    request = Request('https://api.snapcraft.io/v2/snaps/info/xdzip', headers={
        'Snap-Device-Series': '16', 'Snap-Device-Architecture': 'amd64',
    })
    with urlopen(request, timeout=15) as response:
        entries = json.load(response)['channel-map']
    for entry in entries:
        channel = entry['channel']
        if (channel['track'] == 'latest' and channel['risk'] == 'stable' and
                channel['architecture'] == 'amd64' and not channel.get('branch')):
            return entry
    return None


def matches(entry, sha3, version):
    return (entry is not None and entry['download']['sha3-384'] == sha3 and
            type(entry['revision']) is int and entry['revision'] > 0 and entry['version'] == version)


def wait_for_stable(sha3, version):
    for attempt in range(12):
        if attempt:
            time.sleep(5)
        try:
            entry = stable_entry()
        except OSError:
            continue
        if matches(entry, sha3, version):
            return entry
    raise ValueError('Store did not report the expected Snap on latest/stable')


def summary(message):
    print(message)
    with Path(os.environ['GITHUB_STEP_SUMMARY']).open('a', encoding='utf-8') as stream:
        stream.write(message + '\n')


def publish_stable(directory, source_revision, tag):
    package, version, sha256, sha3 = read_candidate(directory, source_revision)
    if not re.fullmatch(r'v[0-9]+\.[0-9]+\.[0-9]+', tag) or tag != f'v{version}':
        raise ValueError('The Snap receipt does not match the stable release tag')
    entry = stable_entry()
    if not matches(entry, sha3, version):
        subprocess.run(['snapcraft', 'upload', str(package), '--release=latest/stable'], check=True)
        entry = wait_for_stable(sha3, version)
    revision = entry['revision']
    summary(f'### XDZip {version} amd64 on `latest/stable`\n\n'
            f'- Store revision: **{revision}**\n- Source: `{source_revision}`\n- SHA-256: `{sha256}`\n')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('directory', type=Path)
    parser.add_argument('source_revision')
    parser.add_argument('tag')
    args = parser.parse_args()
    if not os.environ.get('SNAPCRAFT_STORE_CREDENTIALS'):
        parser.error('Configure SNAPCRAFT_STORE_CREDENTIALS in the release-signing environment')
    try:
        publish_stable(args.directory, args.source_revision, args.tag)
    except (KeyError, OSError, ValueError, subprocess.CalledProcessError) as error:
        raise SystemExit(str(error)) from None


if __name__ == '__main__':
    main()
