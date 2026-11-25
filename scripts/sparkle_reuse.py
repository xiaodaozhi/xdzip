"""Validate an owned, native build of the pinned Sparkle SDK before reusing it."""
import hashlib
import json
from pathlib import Path, PurePosixPath
import subprocess
import tarfile
import tempfile

from qt_network_bundle import encode, validate_architecture
from qt_network_security import ROOT, digest
import sparkle_build
from sparkle_bundle import configuration, validate_sdk


def inputs(native):
    return {**sparkle_build.inputs(configuration()), 'toolchain': native['toolchain'],
            'prepare_sha256': digest(ROOT/'scripts/prepare-sparkle.py'),
            'dependency_sha256': digest(ROOT/'scripts/sparkle-dependency.json')}


def key(expected):
    return 'sparkle-sdk-v1-' + hashlib.sha256(encode(expected)).hexdigest()


def verify(sdk, expected):
    validate_sdk(sdk)
    framework = sdk/'Sparkle.framework'
    validate_architecture(framework/'Versions/B/Sparkle', expected['architecture'])
    subprocess.run(['codesign', '--verify', '--deep', '--strict', framework], check=True, capture_output=True)


def pack(output, sdk, expected):
    verify(sdk, expected)
    with tarfile.open(output, 'x:gz', compresslevel=3) as archive:
        archive.add(sdk, arcname='sdk')


def restore(bundle, sdk, expected):
    sdk.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='sparkle-reuse-', dir=sdk.parent) as temporary:
        with tarfile.open(bundle) as archive:
            members = archive.getmembers()
            names = set()
            if not 0 < len(members) <= 5000 or sum(member.size for member in members) > 256*1024*1024:
                raise ValueError('Sparkle SDK archive exceeds the reuse budget')
            for member in members:
                path = PurePosixPath(member.name)
                if (not path.parts or path.parts[0] != 'sdk' or '..' in path.parts or member.name in names
                        or not (member.isfile() or member.isdir() or member.issym())):
                    raise ValueError('Unexpected Sparkle SDK archive inventory')
                names.add(member.name)
            archive.extractall(temporary, members=members, filter='data')
        extracted = Path(temporary)/'sdk'
        verify(extracted, expected)
        extracted.rename(sdk)
