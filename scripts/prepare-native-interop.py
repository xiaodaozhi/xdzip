# scripts/prepare-native-interop.py

"""Prepare a pinned official reference tool for tests only, never product deployment."""
import argparse
import hashlib
import os
from pathlib import Path
import platform
import subprocess
import tarfile
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
TOOLS = {
    'Windows': ('7z2605-extra.7z', 'f27a8f103c84785026edcba1f67faf1357225df9d19ddb0b7760d55947141bc7', 'x64/7za.exe'),
    'Linux': ('7z2605-linux-x64.tar.xz', '22cf82fcc6eb7474f1d5bb4f02c0f8ebd573a718089d6833dc30f8350a3b232e', '7zz'),
    'Darwin': ('7z2605-mac.tar.xz', '12ef6830317743cfd3f0ab1724bde43fcc270d8b6d8564f8c12c2a2b7f993570', '7zz'),
}


def prepare(output):
    output = output.resolve()
    if not output.is_relative_to(ROOT / 'build') or output == ROOT / 'build':
        raise ValueError('Reference tools belong below build/')
    output.mkdir(parents=True, exist_ok=True)
    filename, checksum, member = TOOLS[platform.system()]
    archive = output / filename
    if not archive.is_file():
        with urllib.request.urlopen('https://github.com/ip7z/7zip/releases/download/26.05/' + filename, timeout=60) as response:
            data = response.read(16 * 1024**2 + 1)
        if len(data) > 16 * 1024**2 or hashlib.sha256(data).hexdigest() != checksum:
            raise ValueError('Reference tool download checksum mismatch')
        archive.write_bytes(data)
    if hashlib.sha256(archive.read_bytes()).hexdigest() != checksum:
        raise ValueError('Reference tool cache checksum mismatch')
    if platform.system() == 'Windows':
        # Only one known member of a verified upstream archive reaches disk.
        data = subprocess.check_output(['tar', '-xOf', str(archive), member])
    else:
        with tarfile.open(archive) as source:
            item = source.getmember(member)
            if not item.isfile() or item.size > 16 * 1024**2:
                raise ValueError('Invalid reference executable')
            data = source.extractfile(item).read()
    executable = output / ('7za.exe' if os.name == 'nt' else '7zz')
    executable.write_bytes(data)
    if os.name != 'nt':
        executable.chmod(0o755)
    return executable


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, default=ROOT / 'build/native-codec/reference')
    args = parser.parse_args()
    prepare(args.output)
    print('Prepared official 7-Zip reference executable for interoperability tests')
