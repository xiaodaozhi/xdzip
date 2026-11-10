# scripts/prepare-sparkle.py

"""Prepare the pinned upstream SDK with the source-built xdzip framework library."""
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
import sparkle_build

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


def prepare_upstream(output, local_archive):
    output = output.resolve()
    marker = output/'xdzip-sdk.json'
    if marker.is_file():
        expected = json.loads(marker.read_text())
        if expected.get('sha256') == SHA256 and (output/'Sparkle.framework/Versions/B/Headers/SPUUpdater.h').is_file():
            validate_sdk(output, require_proxy=False)
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


def material(kind, local_archive=None):
    spec = DEPENDENCY[kind]
    path = Path(local_archive) if local_archive else ROOT/'build/source-cache'/spec['file']
    if not path.exists() and local_archive is None:
        path.parent.mkdir(parents=True, exist_ok=True)
        partial = path.with_suffix(path.suffix + '.partial')
        try:
            request = urllib.request.Request(spec['url'], headers={'User-Agent': 'xdzip-build/1'})
            with urllib.request.urlopen(request, timeout=90) as source, partial.open('wb') as target:
                total = 0
                while data := source.read(1024*1024):
                    total += len(data)
                    if total > 128*1024*1024:
                        raise ValueError('Sparkle material exceeds size limit')
                    target.write(data)
            if sparkle_build.digest(partial) != spec['sha256']:
                raise ValueError('Sparkle material checksum differs')
            partial.replace(path)
        finally:
            partial.unlink(missing_ok=True)
    if path.is_symlink() or not path.is_file() or path.stat().st_size > 128*1024*1024:
        raise ValueError('Invalid Sparkle material')
    if sparkle_build.digest(path) != spec['sha256']:
        raise ValueError('Sparkle material checksum differs')
    return path


def prepare(output, local_archive=None, source_archive=None):
    output = output.resolve()
    if output.exists():
        validate_sdk(output)
        return output
    output.parent.mkdir(parents=True, exist_ok=True)
    sdk_archive = material('sdk', local_archive)
    source_archive = material('source', source_archive)
    with tempfile.TemporaryDirectory(prefix='sparkle-build-', dir=output.parent) as temporary:
        stage = Path(temporary)
        sdk = prepare_upstream(stage/'sdk', sdk_archive)
        extracted = stage/'source'
        extracted.mkdir()
        with tarfile.open(source_archive) as source:
            for member in source.getmembers():
                path = PurePosixPath(member.name)
                if path.is_absolute() or '..' in path.parts or not (member.isfile() or member.isdir()):
                    raise ValueError('Unexpected Sparkle source archive member')
            # The full source archive is pinned, contains no links, and has
            # bounded paths. Never execute downloaded build scripts.
            source.extractall(extracted)
        roots = list(extracted.iterdir())
        if len(roots) != 1 or not roots[0].is_dir():
            raise ValueError('Unexpected Sparkle source root')
        record = sparkle_build.build(roots[0], sdk, stage/'objects', DEPENDENCY,
                                     output.parent/(output.name+'-build.log'))
        marker = json.loads((sdk/'xdzip-sdk.json').read_bytes())
        marker['framework_build'] = record
        (sdk/'xdzip-sdk.json').write_text(json.dumps(marker, indent=2)+'\n')
        validate_sdk(sdk)
        shutil.move(str(sdk), output)
    return output


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path)
    parser.add_argument('--public-key', action='store_true')
    parser.add_argument('--archive', type=Path, help='Use the pinned SDK archive for offline preparation')
    parser.add_argument('--source-archive', type=Path, help='Use the pinned source archive for offline preparation')
    args = parser.parse_args()
    if args.public_key:
        print(public_key())
    elif args.output:
        try:
            prepare(args.output, args.archive, args.source_archive)
            print('Pinned xdzip Sparkle framework ready')
        except (ValueError, OSError) as error:
            raise SystemExit(str(error).replace(str(ROOT), '<workspace>').replace(str(Path.home()), '<user>'))
    else:
        parser.error('--output or --public-key is required')
