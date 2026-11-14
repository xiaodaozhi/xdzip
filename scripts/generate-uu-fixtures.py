#!/usr/bin/env python3
"""Build a pinned independent stream encoder and regenerate owned UU/XXE inputs."""
import argparse
import base64
import binascii
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
DEST = ROOT / 'tests/fixtures/uu'
COMMIT = '7640bc56aa5016cdc9c139eb1ab3ec874e47c744'
SOURCES = 'uulib uucheck uunconc uuutil uuencode uuscan uustring fptools crc32'.split()
PAYLOAD = b'Owned UU/XXE payload.\n' * 17 + bytes(range(256))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--reference', type=Path, required=True)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    ref = args.reference.resolve()
    if not ref.is_relative_to(ROOT / 'build/toolchain'):
        parser.error('Keep reference source under build/toolchain/')
    def git(*arguments):
        return subprocess.check_output(['git', '-C', str(ref), *arguments], text=True).strip()
    if git('rev-parse', 'HEAD') != COMMIT or git('diff', '--name-only', 'HEAD', '--', 'uulib'):
        raise ValueError('Unexpected or modified independent reference source')
    report = ROOT / 'build/review/format-expansion'
    report.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='uu-reference-', dir=report) as temporary:
        work = Path(temporary)
        for name in ['config.h', 'reference.c']:
            shutil.copyfile(DEST / name, work / name)
        executable = work / ('reference.exe' if os.name == 'nt' else 'reference')
        sources = [str(ref / 'uulib' / (name + '.c')) for name in SOURCES]
        if os.name == 'nt':
            command = ['cl', '/nologo', '/O2', '/W0', '/DHAVE_CONFIG_H', '/I' + str(work),
                       '/I' + str(ref / 'uulib'), '/Fe' + str(executable), str(work / 'reference.c'), *sources]
        else:
            command = ['cc', '-O2', '-DHAVE_CONFIG_H', '-I' + str(work), '-I' + str(ref / 'uulib'),
                       str(work / 'reference.c'), *sources, '-o', str(executable)]
        build = subprocess.run(command, cwd=work, capture_output=True, check=True)
        (report / 'm7-uu-reference-build.log').write_bytes(build.stdout + build.stderr)
        inputs = {}
        for encoding, extension in [(1, 'uue'), (3, 'xxe')]:
            for label, payload in [('ordinary', PAYLOAD), ('empty', b''), ('tails', bytes(range(46)))]:
                source, output = work / 'owned.bin', work / 'owned.txt'
                source.write_bytes(payload)
                subprocess.run([str(executable), 'encode', str(source), str(output), str(encoding),
                                'hello.bin'], check=True)
                encoded = output.read_bytes().replace(b'\r\n', b'\n')
                if encoding == 1:
                    lines = encoded.splitlines()
                    decoded = b''.join(binascii.a2b_uu(line) for line in lines[1:lines.index(b'end')])
                    if decoded != payload:
                        raise ValueError('Independent Python UU decoding mismatch')
                inputs[label + '.' + extension] = encoded
        inputs['spaces.uu'] = inputs['ordinary.uue'].replace(b'`', b' ').replace(b'\n', b'\r\n')
        encoded = base64.b64encode(PAYLOAD)
        if base64.b64decode(encoded, validate=True) != PAYLOAD:
            raise ValueError('Independent Base64 mismatch')
        base64_member = b'begin-base64 644 data.bin\n' + b'\n'.join(
            encoded[at:at + 76] for at in range(0, len(encoded), 76)) + b'\n====\n'
        inputs['base64.uue'] = base64_member
        inputs['multi.uue'] = (b'Owned transport preamble\n\n' + inputs['ordinary.uue'] +
                               b'between members\n' + inputs['tails.xxe'] + base64_member + b'Owned trailer\n')
        digests = {}
        for name, data in inputs.items():
            target = DEST / name
            if args.check:
                if target.read_bytes() != data:
                    raise ValueError('Stale owned input: ' + name)
            else:
                target.write_bytes(data)
            digests[name] = hashlib.sha256(data).hexdigest()
        (report / 'm7-uu-reference.json').write_text(json.dumps(
            {'reference_commit': COMMIT, 'inputs': digests}, indent=2) + '\n', encoding='utf-8')
    print(f'{len(inputs)} owned UU/XXE inputs independently verified')


if __name__ == '__main__':
    main()
