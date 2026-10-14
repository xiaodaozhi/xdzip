#!/usr/bin/env python3
"""Decode AES and numbered split output with an independent 7-Zip executable.

Only owned synthetic fixtures and a public fixture password are used. Product
creation never invokes subprocesses or places a real password in arguments.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--reader', default=shutil.which('7zz') or shutil.which('7z'))
    parser.add_argument('--output', type=Path, default=ROOT/'build/review/advanced-creation.json')
    args = parser.parse_args()
    if not args.reader:
        raise SystemExit('Independent 7-Zip is required; install outside the repository')
    binary = str(args.binary.resolve())
    cases = []
    with tempfile.TemporaryDirectory(prefix='xdzip-aes-split-') as owned:
        root = Path(owned)
        source = root/'input'
        source.mkdir()
        (source/'empty.txt').write_bytes(b'')
        (source/'空目录').mkdir()
        (source/'中文.txt').write_bytes(('AES and split ZIP，UTF-8\n'*5000).encode())
        (source/'random.bin').write_bytes(hashlib.shake_256(b'public synthetic fixture').digest(200_000))
        password = 'public fixture 密码'
        for method in [0, 8]:
            for split in [False, True]:
                for encrypt in [False, True]:
                    out = root/f'out-{method}-{split}-{encrypt}'
                    out.mkdir()
                    env = {k:v for k,v in os.environ.items() if k != 'XDZIP_TEST_PASSWORD'}
                    if encrypt: env['XDZIP_TEST_PASSWORD'] = password
                    result = subprocess.run([binary,str(source),str(out),'fixture.zip',str(method),'6',str(65536 if split else 0)],env=env,capture_output=True,text=True)
                    if result.returncode:
                        raise RuntimeError('Owned creation fixture failed: '+result.stderr[-2000:])
                    archive = out/('fixture.zip.001' if split else 'fixture.zip')
                    target = root/f'decoded-{method}-{split}-{encrypt}'
                    tool = subprocess.run([args.reader,'x',str(archive),'-o'+str(target),'-y','-p'+password],capture_output=True)
                    if tool.returncode: raise RuntimeError('Independent decoder failed: '+tool.stdout.decode(errors='replace')[-2000:])
                    for path in source.rglob('*'):
                        decoded = target/'input'/path.relative_to(source)
                        assert decoded.is_dir() if path.is_dir() else decoded.read_bytes() == path.read_bytes(), 'Independent payload mismatch'
                    assert not any(p.name.startswith('.xdzip-work-') for p in out.iterdir())
                    parts = sorted(out.glob('fixture.zip.*')) if split else [archive]
                    if split:
                        assert len(parts) > 1
                        assert all(0 < p.stat().st_size <= 65536 for p in parts)
                    if encrypt:
                        wrong = subprocess.run([args.reader,'t',str(archive),'-pwrong-public-password'],capture_output=True)
                        assert wrong.returncode != 0, 'Wrong password unexpectedly accepted'
                    cases.append(dict(method=method,split=split,encrypted=encrypt,parts=len(parts),passed=True))
    args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_text(json.dumps(dict(cases=cases,independent_reader=Path(args.reader).name),indent=2)+'\n')
    print(f'Independent ZIP creation: {len(cases)} cases passed')


if __name__ == '__main__': main()
