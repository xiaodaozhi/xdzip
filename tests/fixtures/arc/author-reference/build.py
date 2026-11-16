#!/usr/bin/env python3
"""Build the owned test adapter against byte-verified external ARC sources."""
import argparse
import hashlib
import os
from pathlib import Path
import subprocess
import zipfile

ROOT = Path(__file__).resolve().parents[4]
OWN = Path(__file__).resolve().parent
FILES = ['arclzw.c', 'arcusq.c', 'arcsq.c', 'arcunp.c', 'arcpack.c', 'arccode.c',
         'arcdata.c', 'arc.h', 'arcs.h', 'proto.h']
REVISION = '70f0b93516d0ee222db5cf1241aad0db1b721c0d'
SOURCE_SHA = 'd8722a28ed9612f7712efed3836f6fac3253643b7b54f06d15ba759cae849985'
EXPECTED_SHA = 'd5d2e5b47154da2766589e8f8007eb0bc8ae320cfae69d2bb6b226c1023171b4'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cc', type=Path, required=True)
    parser.add_argument('--reference-root', type=Path,
                        default=ROOT / 'build/toolchain/references/arc-5.21q')
    args = parser.parse_args()
    base, cc = args.reference_root.resolve(), args.cc.resolve()
    assert base.is_relative_to(ROOT / 'build') and cc.is_file()
    source = base / 'source'
    package = base / 'source.zip'
    assert hashlib.sha256(package.read_bytes()).hexdigest() == SOURCE_SHA
    with zipfile.ZipFile(package) as archive:
        for name in FILES:
            assert (source / name).read_bytes() == archive.read(f'arc-{REVISION}/{name}'), name
    temporary = ROOT / 'build/test-tmp/arc-reference'
    temporary.mkdir(exist_ok=True)
    environment = os.environ.copy()
    environment['TEMP'] = environment['TMP'] = str(temporary)
    environment['PATH'] = str(cc.parent) + os.pathsep + environment['PATH']
    binary = base / 'reference.exe'
    command = [str(cc), '-std=gnu89', '-fcommon', '-O2', '-DSYSV=1',
               '-Wl,--no-insert-timestamp', '-include', str(OWN / 'compat.h'),
               '-I' + str(base), '-I' + str(source), str(OWN / 'reference.c')]
    command += [str(source / name) for name in FILES if name.endswith('.c')]
    command += ['-o', str(binary)]
    done = subprocess.run(command, env=environment, capture_output=True, timeout=60)
    details = (done.stdout + done.stderr).decode(errors='replace')
    details = details.replace(os.environ.get('USERPROFILE', '<absent>'), '<user>')
    (ROOT / 'build/review/format-expansion/m8-arc-owned-reference-rebuild.log').write_text(
        details, encoding='utf-8')
    assert done.returncode == 0, f'Reference compilation failed: {done.returncode}'
    identity = hashlib.sha256(binary.read_bytes()).hexdigest()
    assert identity == EXPECTED_SHA, identity
    print('Verified unchanged external ARC sources and rebuilt exact reference', identity)


if __name__ == '__main__':
    main()
