#!/usr/bin/env python3
"""Verify the actual ISO creation entry with an independent libarchive tar reader."""
import argparse
import csv
from collections import Counter
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess


def run(args, *, success=True):
    result = subprocess.run([str(arg) for arg in args], capture_output=True, timeout=90)
    if success and result.returncode:
        raise AssertionError(result.stdout.decode('utf-8', 'replace') + result.stderr.decode('utf-8', 'replace'))
    return result


def files(root):
    return {path.relative_to(root).as_posix(): hashlib.sha256(path.read_bytes()).hexdigest()
            for path in root.rglob('*') if path.is_file()}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cli', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--tar', default=shutil.which('tar'))
    args = parser.parse_args()
    output = args.output.resolve()
    root = Path(__file__).resolve().parents[1]
    assert output.is_relative_to(root / 'build') and output != root / 'build'
    assert not output.exists(), 'Use a fresh evidence directory'
    assert args.tar, 'Independent libarchive tar is required'
    output.mkdir(parents=True, mode=0o700)
    if os.name == 'nt':
        sid = list(csv.reader(run(['whoami', '/user', '/fo', 'csv', '/nh']).stdout.decode().splitlines()))[0][1]
        run(['icacls', output, '/inheritance:r', '/grant:r', '*' + sid + ':(OI)(CI)F'])
    source = output / 'dataset'
    (source / '目录' / 'empty').mkdir(parents=True)
    (source / 'zero').write_bytes(b'')
    (source / '目录' / '中文.txt').write_bytes('中文与 Joliet\n'.encode() * 300)
    (source / ('中' * 64)).write_bytes(bytes(range(256)) * 17)
    boundary = source / ('长' * 64)
    boundary.mkdir()
    (boundary / ('界' * 46)).write_bytes(b'240-byte encoded Joliet path')
    (source / 'large.bin').write_bytes(bytes(range(251)) * 9000)
    image = output / 'created.iso'
    result = run([args.cli.resolve(), 'create', image, source, '--json'])
    response = json.loads(result.stdout)
    assert response['status'] == 'succeeded', response
    expected = files(source)
    for primary in (False, True):
        target = output / ('primary-extracted' if primary else 'joliet-extracted')
        target.mkdir()
        options = ['--options', 'iso9660:joliet=off'] if primary else []
        run([args.tar, *options, '-xf', image, '-C', target])
        actual = files(target)
        if primary:
            assert Counter(actual.values()) == Counter(expected.values()), actual
            assert all(name.isascii() for name in actual)
        else:
            assert actual == {'dataset/' + name: digest for name, digest in expected.items()}, actual
            assert (target / 'dataset' / '目录' / 'empty').is_dir()
    before = hashlib.sha256(image.read_bytes()).hexdigest()
    refused = run([args.cli.resolve(), 'create', image, source, '--json', '--conflict', 'skip'], success=False)
    assert refused.returncode != 0
    assert hashlib.sha256(image.read_bytes()).hexdigest() == before
    bad = output / 'bad'
    bad.mkdir()
    (bad / 'emoji-😀.txt').write_bytes(b'must not be renamed')
    refused = run([args.cli.resolve(), 'create', output / 'rejected.iso', bad, '--json'], success=False)
    assert refused.returncode != 0 and not (output / 'rejected.iso').exists()
    (bad / 'emoji-😀.txt').unlink()
    too_long = bad / ('长' * 64)
    too_long.mkdir()
    (too_long / ('界' * 51)).write_bytes(b'242-byte encoded Joliet path')
    refused = run([args.cli.resolve(), 'create', output / 'rejected-path.iso', bad, '--json'], success=False)
    assert refused.returncode != 0 and not (output / 'rejected-path.iso').exists()
    receipt = {'status': 'passed', 'cli_sha256': hashlib.sha256(args.cli.read_bytes()).hexdigest(),
               'image_sha256': before, 'independent_reader': run([args.tar, '--version']).stdout.decode().strip(),
               'checks': ['real-cli-default-options', 'joliet-exact-names-and-bytes', 'primary-alias-bytes',
                          '64-character-name', 'multi-sector-files', 'empty-directory', 'overwrite-protection',
                          'unsupported-name-no-publication', '240-byte-path-and-overlimit-no-publication']}
    (output / 'receipt.json').write_text(json.dumps(receipt, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print('ISO creation: 9 checks passed using an independent reader')


if __name__ == '__main__':
    main()
