"""Author self-owned sparable UDF test input with upstream tools in disposable Docker.

Only repository build/ output is admitted. No block device or image mount is used.
The authoring tools are independent test references, never product dependencies.
"""
import argparse
import gzip
import hashlib
import json
from pathlib import Path
import subprocess
import uuid

ROOT = Path(__file__).resolve().parents[1]

AUTHOR = r'''#!/bin/sh
set -eu
export DEBIAN_FRONTEND=noninteractive
apt-get update > /work/tool-install.log 2>&1
apt-get install -y --no-install-recommends udftools=2.3-1build2 udfclient=0.8.11-2 >> /work/tool-install.log 2>&1
dpkg-query -W udftools udfclient > /work/tool-versions.txt
sha256sum /usr/sbin/mkudffs /usr/bin/udfclient > /work/tool-sha256.txt
cp /usr/share/doc/udftools/copyright /work/udftools-copyright.txt
cp /usr/share/doc/udfclient/copyright /work/udfclient-copyright.txt
mkudffs --new-file --blocksize=2048 --media-type=dvdrw --udfrev=2.01 --label=XDZIP_SPARABLE_SEEDED --sparspace=16 /work/sparable-seeded.iso 65536 > /work/format.log 2>&1
udfinfo --blocksize=2048 /work/sparable-seeded.iso > /work/volume-before.txt 2>&1
set_id=$(sed -n 's/^fullvsid=//p' /work/volume-before.txt)
cd /
printf 'cd "%s:XDZIP_SPARABLE_SEEDED:XDZIP_SPARABLE_SEEDED:LinuxUDF"\nput /work/input/hello.txt hello.txt\nmkdir nested\ncd nested\nput /work/input/data.bin data.bin\ncd ..\nput /work/input/empty.txt empty.txt\nsync\nquit\n' "$set_id" > /work/write-commands.txt
timeout 30 udfclient -b 2048 -W /work/sparable-seeded.iso < /work/write-commands.txt > /work/write.log 2>&1
printf 'cd "%s:XDZIP_SPARABLE_SEEDED:XDZIP_SPARABLE_SEEDED:LinuxUDF"\nlcd /work/readback\nget hello.txt hello.txt\nget empty.txt empty.txt\ncd nested\nget data.bin data.bin\nquit\n' "$set_id" > /work/read-commands.txt
timeout 30 udfclient -b 2048 /work/sparable-seeded.iso < /work/read-commands.txt > /work/readback.log 2>&1
udfinfo --blocksize=2048 /work/sparable-seeded.iso > /work/volume-after.txt 2>&1
'''


def generate(output):
    output = Path(output).resolve()
    if not output.is_relative_to(ROOT / 'build') or output == ROOT / 'build':
        raise ValueError('Generated image artifacts belong below repository build/')
    if output.exists() and (not output.is_dir() or any(output.iterdir())):
        raise ValueError('Output must not exist or must be an empty directory')
    output.mkdir(parents=True, exist_ok=True)
    (output / 'input').mkdir()
    (output / 'readback').mkdir()
    payloads = {'hello.txt': b'independent Microsoft IMAPI fixture\n',
                'empty.txt': b'', 'data.bin': bytes(range(256)) * 32}
    for name, data in payloads.items():
        (output / 'input' / name).write_bytes(data)
    (output / 'author.sh').write_text(AUTHOR, encoding='utf-8', newline='\n')
    container = 'xdzip-optical-fixture-' + uuid.uuid4().hex
    try:
        subprocess.run(['docker', 'run', '--rm', '--name', container, '--mount',
                        'type=bind,source=' + str(output) + ',target=/work',
                        'ubuntu:24.04', 'sh', '/work/author.sh'], check=True, timeout=240)
    finally:
        # A CLI timeout does not stop its container. Clean up only the random
        # container created by this invocation; retain all diagnostic files.
        subprocess.run(['docker', 'rm', '-f', container], capture_output=True, timeout=30)
    readback = {}
    for name, expected in payloads.items():
        actual = (output / 'readback' / name).read_bytes()
        if actual != expected:
            raise ValueError('Independent readback mismatch: ' + name)
        readback[name] = {'bytes': len(actual), 'sha256': hashlib.sha256(actual).hexdigest()}
    source = output / 'sparable-seeded.iso'
    digest = hashlib.sha256()
    packed = output / 'sparable-seeded.iso.gz'
    with source.open('rb') as input_file, packed.open('wb') as output_file:
        with gzip.GzipFile(filename='', mode='wb', fileobj=output_file, mtime=0) as compressor:
            while chunk := input_file.read(1024 * 1024):
                digest.update(chunk)
                compressor.write(chunk)
    report = {'decoded_bytes': source.stat().st_size, 'decoded_sha256': digest.hexdigest(),
              'compressed_bytes': packed.stat().st_size,
              'sha256': hashlib.sha256(packed.read_bytes()).hexdigest(), 'readback': readback}
    (output / 'generation.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', required=True, type=Path)
    generate(parser.parse_args().output)
