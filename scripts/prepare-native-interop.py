# scripts/prepare-native-interop.py

"""Prepare a pinned official reference tool for tests only, never product deployment."""
import argparse
import hashlib
from pathlib import Path
import platform
import subprocess
import tarfile
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
RELEASE_URL = 'https://github.com/ip7z/7zip/releases/download/26.05/'
MAX_DOWNLOAD = 16 * 1024**2
WINDOWS_BOOTSTRAP = ('7zr.exe', '9bf582b894dfa665f852d7e9112b02290a1c303c2d95a21fcdc41ffcb03143b7')
WINDOWS_EXECUTABLE_SHA256 = '7ddf0a3bc451e166679296be630d6f0573eb386159c04184761c43020c484291'
TOOLS = {
    'Windows': ('7z2605-extra.7z', 'f27a8f103c84785026edcba1f67faf1357225df9d19ddb0b7760d55947141bc7', 'x64/7za.exe'),
    'Linux': ('7z2605-linux-x64.tar.xz', '22cf82fcc6eb7474f1d5bb4f02c0f8ebd573a718089d6833dc30f8350a3b232e', '7zz'),
    'Darwin': ('7z2605-mac.tar.xz', '12ef6830317743cfd3f0ab1724bde43fcc270d8b6d8564f8c12c2a2b7f993570', '7zz'),
}


def verified_download(output, filename, checksum):
    archive = output / filename
    if not archive.is_file():
        with urllib.request.urlopen(RELEASE_URL + filename, timeout=60) as response:
            data = response.read(MAX_DOWNLOAD + 1)
        if len(data) > MAX_DOWNLOAD or hashlib.sha256(data).hexdigest() != checksum:
            raise ValueError('Reference tool download checksum mismatch')
        archive.write_bytes(data)
    with archive.open('rb') as source:
        data = source.read(MAX_DOWNLOAD + 1)
    if len(data) > MAX_DOWNLOAD or hashlib.sha256(data).hexdigest() != checksum:
        raise ValueError('Reference tool cache checksum mismatch')
    return archive


def prepare(output):
    output = output.resolve()
    if not output.is_relative_to(ROOT / 'build') or output == ROOT / 'build':
        raise ValueError('Reference tools belong below build/')
    output.mkdir(parents=True, exist_ok=True)
    system = platform.system()
    filename, checksum, member = TOOLS[system]
    archive = verified_download(output, filename, checksum)
    if system == 'Windows':
        # Windows tar may omit LZMA. Bootstrap with independently pinned official
        # 7zr bytes; never invoke an installer, ambient tool or product backend.
        bootstrap = verified_download(output, *WINDOWS_BOOTSTRAP)
        data = subprocess.check_output(
            [str(bootstrap), 'x', '-so', str(archive), member], timeout=60)
        if hashlib.sha256(data).hexdigest() != WINDOWS_EXECUTABLE_SHA256:
            raise ValueError('Reference executable checksum mismatch')
    else:
        with tarfile.open(archive) as source:
            item = source.getmember(member)
            if not item.isfile() or item.size > MAX_DOWNLOAD:
                raise ValueError('Invalid reference executable')
            data = source.extractfile(item).read()
    executable = output / ('7za.exe' if system == 'Windows' else '7zz')
    executable.write_bytes(data)
    if system != 'Windows':
        executable.chmod(0o755)
    return executable


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, default=ROOT / 'build/native-codec/reference')
    args = parser.parse_args()
    prepare(args.output)
    print('Prepared official 7-Zip reference executable for interoperability tests')
