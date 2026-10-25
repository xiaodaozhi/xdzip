"""Generate independent VAT test images with a pinned NetBSD makefs Linux port.

Only fresh repository build/ output is admitted. The optional authoring tools
run in disposable Docker, without devices or image mounts; CI uses fixed inputs.
"""
import argparse
import gzip
import hashlib
import json
from pathlib import Path, PurePosixPath
import shutil
import subprocess
import tarfile
import urllib.request
import uuid

ROOT = Path(__file__).resolve().parents[1]
COMMIT = 'dc7e09b66b5900763299add245d6bb327882292e'
SOURCE_URL = 'https://codeload.github.com/john-tornblom/makefs_termux/tar.gz/' + COMMIT
SOURCE_SHA256 = '63d244e91ea6150dd0228f63eea09e7d49d43ea56a09ee9e5baa8a263902b78f'
PAYLOADS = {'hello.txt': b'independent Microsoft IMAPI fixture\n',
            'empty.txt': b'', 'nested/data.bin': bytes(range(256)) * 32}

AUTHOR = r'''#!/bin/sh
set -eu
export DEBIAN_FRONTEND=noninteractive
apt-get update > /work/tool-install.log 2>&1
apt-get install -y --no-install-recommends build-essential pkg-config libbsd-dev udftools=2.3-1build2 udfclient=0.8.11-2 >> /work/tool-install.log 2>&1
dpkg-query -W gcc make libbsd-dev pkg-config udftools udfclient > /work/tool-versions.txt
cat /etc/os-release > /work/tool-os.txt
cp /usr/share/doc/udftools/copyright /work/udftools-copyright.txt
cp /usr/share/doc/udfclient/copyright /work/udfclient-copyright.txt
cd /work/reference-source
make prog > /work/build-tool.log 2>&1
sha256sum makefs /usr/bin/udfclient /usr/bin/udfinfo > /work/tool-sha256.txt
for revision in 150 201; do
    timeout 30 ./makefs -t udf -S 2048 -s 8m -o "disctype=cdr,minver=0x$revision,maxver=0x$revision,loglabel=XDZIP_VAT$revision" "/work/vat$revision.udf" /work/input > "/work/vat$revision-create.log" 2>&1
    udfinfo --blocksize=2048 "/work/vat$revision.udf" > "/work/vat$revision-info.txt" 2>&1
    set_id=$(sed -n 's/^fullvsid=//p' "/work/vat$revision-info.txt")
    volume_id=$(sed -n 's/^vid=//p' "/work/vat$revision-info.txt")
    logical_id=$(sed -n 's/^lvid=//p' "/work/vat$revision-info.txt")
    file_set_id=$(sed -n 's/^fsid=//p' "/work/vat$revision-info.txt")
    test -n "$set_id" && test -n "$volume_id" && test -n "$logical_id" && test -n "$file_set_id"
    printf 'cd "%s:%s:%s:%s"\nlcd /work/readback-vat%s\nget hello.txt hello.txt\nget empty.txt empty.txt\ncd nested\nget data.bin data.bin\nquit\n' "$set_id" "$volume_id" "$logical_id" "$file_set_id" "$revision" > "/work/vat$revision-read-commands.txt"
    # udfclient resolves absolute local names from its process directory.
    cd /
    timeout 30 udfclient -b 2048 "/work/vat$revision.udf" < "/work/vat$revision-read-commands.txt" > "/work/vat$revision-readback.log" 2>&1
    cd /work/reference-source
done
'''


def download_source(output):
    archive = output / 'makefs-source.tar.gz'
    digest = hashlib.sha256()
    total = 0
    with urllib.request.urlopen(SOURCE_URL, timeout=30) as response, archive.open('wb') as target:
        while chunk := response.read(65536):
            total += len(chunk)
            if total > 1024 * 1024:
                raise ValueError('Reference source archive exceeds size limit')
            digest.update(chunk)
            target.write(chunk)
    if digest.hexdigest() != SOURCE_SHA256:
        raise ValueError('Reference source archive SHA256 mismatch')
    destination = output / 'reference-source'
    destination.mkdir()
    with tarfile.open(archive, 'r:gz') as source:
        members = source.getmembers()
        if len(members) > 256 or sum(item.size for item in members) > 4 * 1024 * 1024:
            raise ValueError('Reference source extraction exceeds limits')
        for item in members:
            path = PurePosixPath(item.name)
            if (path.is_absolute() or '..' in path.parts or
                    not path.parts or path.parts[0] != 'makefs_termux-' + COMMIT or
                    not (item.isdir() or item.isfile())):
                raise ValueError('Unexpected reference source archive member')
            target = destination.joinpath(*path.parts[1:])
            if item.isdir():
                target.mkdir(parents=True, exist_ok=True)
            else:
                target.parent.mkdir(parents=True, exist_ok=True)
                with source.extractfile(item) as content, target.open('wb') as extracted:
                    shutil.copyfileobj(content, extracted)


def generate(output):
    output = Path(output).resolve()
    if not output.is_relative_to((ROOT / 'build').resolve()) or output == (ROOT / 'build').resolve():
        raise ValueError('Generated image artifacts belong below repository build/')
    if output.exists() and (not output.is_dir() or any(output.iterdir())):
        raise ValueError('Output must not exist or must be an empty directory')
    output.mkdir(parents=True, exist_ok=True)
    for name, data in PAYLOADS.items():
        target = output / 'input' / name
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(data)
    for revision in ('150', '201'):
        (output / ('readback-vat' + revision)).mkdir()
    download_source(output)
    (output / 'author.sh').write_text(AUTHOR, encoding='utf-8', newline='\n')
    container = 'xdzip-makefs-fixture-' + uuid.uuid4().hex
    try:
        subprocess.run(['docker', 'run', '--rm', '--name', container, '--mount',
                        'type=bind,source=' + str(output) + ',target=/work',
                        'ubuntu:24.04', 'sh', '/work/author.sh'], check=True, timeout=300)
    finally:
        # A timed-out Docker CLI can leave its container running. Remove only
        # this invocation's dedicated container, and keep diagnostic artifacts.
        subprocess.run(['docker', 'rm', '-f', container], capture_output=True, timeout=30)
    fixtures = []
    for revision in ('150', '201'):
        readback = {}
        for name, expected in PAYLOADS.items():
            local = Path(name).name
            actual = (output / ('readback-vat' + revision) / local).read_bytes()
            if actual != expected:
                raise ValueError('Independent readback mismatch: VAT' + revision + '/' + name)
            readback[name] = {'bytes': len(actual), 'sha256': hashlib.sha256(actual).hexdigest()}
        source = output / ('vat' + revision + '.udf')
        if source.stat().st_size > 8 * 1024 * 1024:
            raise ValueError('Generated image exceeds size limit')
        packed = output / (source.name + '.gz')
        with source.open('rb') as raw, packed.open('wb') as target:
            with gzip.GzipFile(filename='', mode='wb', fileobj=target, mtime=0) as compressor:
                shutil.copyfileobj(raw, compressor)
        fixtures.append({'file': packed.name, 'udf_revision': revision,
                         'decoded_bytes': source.stat().st_size,
                         'decoded_sha256': hashlib.sha256(source.read_bytes()).hexdigest(),
                         'compressed_bytes': packed.stat().st_size,
                         'sha256': hashlib.sha256(packed.read_bytes()).hexdigest(),
                         'readback': readback})
    report = {'source_commit': COMMIT, 'source_archive_sha256': SOURCE_SHA256,
              'fixtures': fixtures}
    (output / 'generation.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', required=True, type=Path)
    generate(parser.parse_args().output)
