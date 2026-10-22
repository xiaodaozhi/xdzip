# scripts/prepare-sparkle.py

"""Prepare the pinned official Sparkle SDK outside the source tree."""
import argparse
import base64
import hashlib
import json
from pathlib import Path, PurePosixPath
import shutil
import tarfile
import tempfile
import urllib.request
from sparkle_bundle import configuration, validate_sdk

ROOT = Path(__file__).resolve().parents[1]
DEPENDENCY = configuration()
VERSION = DEPENDENCY['version']
URL = DEPENDENCY['sdk']['url']
SHA256 = DEPENDENCY['sdk']['sha256']


def public_key():
    value = json.loads((ROOT/'resources/sparkle-update-key.json').read_text(encoding='utf-8'))
    key = value.get('public_key', '')
    if value.get('schema_version') != 1 or (key and (len(key) != 64 or any(c not in '0123456789abcdef' for c in key))):
        raise ValueError('Invalid Sparkle update public key')
    return base64.b64encode(bytes.fromhex(key)).decode('ascii') if key else ''


def prepare(output, local_archive=None):
    output = output.resolve()
    marker = output/'xdzip-sdk.json'
    if marker.is_file():
        expected = json.loads(marker.read_text())
        if expected.get('sha256') == SHA256 and (output/'Sparkle.framework/Versions/B/Headers/SPUUpdater.h').is_file():
            validate_sdk(output)
            return output
        raise ValueError('Existing Sparkle SDK identity does not match')
    if output.exists():
        raise ValueError('Refusing to replace an unrecognized SDK directory')
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='sparkle-stage-', dir=output.parent) as temporary:
        stage = Path(temporary)
        archive = stage/'sdk.tar.xz'
        if local_archive is not None:
            if local_archive.is_symlink() or local_archive.stat().st_size > 64*1024*1024:
                raise ValueError('Invalid local Sparkle archive')
            shutil.copyfile(local_archive, archive)
        else:
            request = urllib.request.Request(URL, headers={'User-Agent': 'xdzip-build/1'})
            with urllib.request.urlopen(request, timeout=90) as response, archive.open('wb') as target:
                total = 0
                while chunk := response.read(1024 * 1024):
                    total += len(chunk)
                    if total > 64 * 1024 * 1024:
                        raise ValueError('Sparkle SDK download exceeds limit')
                    target.write(chunk)
        if hashlib.sha256(archive.read_bytes()).hexdigest() != SHA256:
            raise ValueError('Sparkle SDK checksum mismatch')
        extracted = stage/'extracted'
        extracted.mkdir()
        with tarfile.open(archive) as source:
            for member in source.getmembers():
                name = PurePosixPath(member.name)
                if name.is_absolute() or '..' in name.parts or not (member.isfile() or member.isdir() or member.issym()):
                    raise ValueError('Unsafe SDK archive member')
                if member.issym():
                    link = (extracted/member.name).parent/member.linkname
                    if not link.resolve().is_relative_to(extracted.resolve()):
                        raise ValueError('SDK link escapes extraction root')
            # The archive's pinned digest is verified before extraction. Members
            # and symbolic link destinations are independently contained above.
            source.extractall(extracted)
        if not (extracted/'Sparkle.framework/Versions/B/Headers/SPUUpdater.h').is_file():
            raise ValueError('Sparkle framework missing')
        marker_data = dict(schema_version=1, version=VERSION, url=URL, sha256=SHA256)
        (extracted/'xdzip-sdk.json').write_text(json.dumps(marker_data, indent=2)+'\n')
        shutil.move(str(extracted), output)
    return output


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path)
    parser.add_argument('--public-key', action='store_true')
    parser.add_argument('--archive', type=Path, help='Use the pinned SDK archive for offline preparation')
    args = parser.parse_args()
    if args.public_key:
        print(public_key())
    elif args.output:
        print(prepare(args.output, args.archive))
    else:
        parser.error('--output or --public-key is required')
