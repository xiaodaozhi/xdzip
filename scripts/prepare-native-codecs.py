# scripts/prepare-native-codecs.py

"""Verify and prepare the pinned official codec sources, never user archives."""
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import shutil
import tarfile
import tempfile
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / 'scripts/native-codec-sources.json'
MAX_ARCHIVE = 64 * 1024 * 1024
MAX_EXPANDED = 256 * 1024 * 1024


def digest(path):
    checksum = hashlib.sha256()
    with Path(path).open('rb') as source:
        for block in iter(lambda: source.read(1024 * 1024), b''):
            checksum.update(block)
    return checksum.hexdigest()


def sources():
    return json.loads(MANIFEST.read_text(encoding='utf-8'))['sources']


def validate_members(archive):
    members = archive.getmembers()
    total = 0
    names = set()
    if len(members) > 20000:
        raise ValueError('Codec source archive has too many members')
    for member in members:
        name = PurePosixPath(member.name)
        if (name.is_absolute() or '..' in name.parts or '\\' in member.name
                or ':' in member.name or not (member.isfile() or member.isdir())):
            raise ValueError('Unsafe codec source archive member')
        canonical = name.as_posix().casefold()
        if canonical in names:
            raise ValueError('Duplicate codec source archive member')
        names.add(canonical)
        total += member.size
        if total > MAX_EXPANDED:
            raise ValueError('Codec source archive expands beyond limit')
    return members


def verify_tree(destination, archive):
    """A cache marker alone cannot establish correspondence with locked sources."""
    if destination.is_symlink():
        raise ValueError('Codec source directory is a symbolic link')
    expected = {'xdzip-source.json'}
    with tarfile.open(archive) as source:
        for member in validate_members(source):
            target = destination / member.name
            if target.is_symlink() or not target.resolve().is_relative_to(destination.resolve()):
                raise ValueError('Codec source tree escapes its root')
            if member.isdir():
                if not target.is_dir():
                    raise ValueError('Codec source directory is missing')
                continue
            expected.add(PurePosixPath(member.name).as_posix())
            if not target.is_file() or target.stat().st_size != member.size:
                raise ValueError('Prepared codec source size mismatch')
            checksum = hashlib.sha256()
            with source.extractfile(member) as stream:
                for block in iter(lambda: stream.read(1024 * 1024), b''):
                    checksum.update(block)
            if digest(target) != checksum.hexdigest():
                raise ValueError('Prepared codec source content differs from pinned archive')
    actual = {path.relative_to(destination).as_posix() for path in destination.rglob('*') if path.is_file()}
    if actual != expected:
        raise ValueError('Prepared codec source has unexpected files')


def prepare(output, downloads, offline=False, archives=None):
    output, downloads = Path(output).resolve(), Path(downloads).resolve()
    # Source preparation can only populate generated build/cache directories.
    for path in (output, downloads):
        if not any(path.is_relative_to(ROOT / prefix) and path != ROOT / prefix
                   for prefix in ('build', '.cache')):
            raise ValueError('Codec outputs must be below build/ or .cache/')
        path.mkdir(parents=True, exist_ok=True)
    roots = {}
    for item in sources():
        archive = downloads / item['archive']
        if not archive.exists() and archives is not None:
            supplied = Path(archives) / item['archive']
            if supplied.is_symlink() or supplied.stat().st_size > MAX_ARCHIVE or digest(supplied) != item['sha256']:
                raise ValueError('Supplied codec source checksum mismatch')
            shutil.copyfile(supplied, archive)
        if not archive.exists():
            if offline:
                raise ValueError('Pinned codec source archive is missing')
            request = urllib.request.Request(item['url'], headers={'User-Agent': 'xdzip-build/1'})
            with urllib.request.urlopen(request, timeout=60) as response, tempfile.NamedTemporaryFile(
                    dir=downloads, delete=False, suffix='.download') as target:
                temporary = Path(target.name)
                try:
                    total = 0
                    while block := response.read(1024 * 1024):
                        total += len(block)
                        if total > MAX_ARCHIVE:
                            raise ValueError('Codec source download exceeds limit')
                        target.write(block)
                except BaseException:
                    target.close()
                    temporary.unlink(missing_ok=True)
                    raise
            if digest(temporary) != item['sha256']:
                temporary.unlink()
                raise ValueError('Downloaded codec source checksum mismatch')
            temporary.replace(archive)
        if archive.is_symlink() or archive.stat().st_size > MAX_ARCHIVE or digest(archive) != item['sha256']:
            raise ValueError('Pinned codec source checksum mismatch')
        destination = output / item['id']
        marker = destination / 'xdzip-source.json'
        if destination.exists():
            if not marker.is_file() or json.loads(marker.read_text()) != item:
                raise ValueError('Unrecognized codec source directory; use a fresh output directory')
        else:
            with tempfile.TemporaryDirectory(prefix='codec-source-', dir=output) as temporary:
                extracted = Path(temporary) / 'source'
                extracted.mkdir()
                with tarfile.open(archive) as source:
                    members = validate_members(source)
                    for member in members:
                        target = extracted / member.name
                        if member.isdir():
                            target.mkdir(parents=True, exist_ok=True)
                        else:
                            target.parent.mkdir(parents=True, exist_ok=True)
                            with source.extractfile(member) as origin, target.open('xb') as destination_file:
                                shutil.copyfileobj(origin, destination_file)
                (extracted / 'xdzip-source.json').write_text(json.dumps(item, indent=2) + '\n', encoding='utf-8')
                shutil.move(str(extracted), destination)
        verify_tree(destination, archive)
        root = destination / item['source_subdirectory']
        for name in item['license_files']:
            if not (root / name).is_file():
                raise ValueError('Codec license material is missing')
        roots[item['id']] = str(root)
    return roots


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, default=ROOT / 'build/native-codec/sources')
    parser.add_argument('--downloads', type=Path, default=ROOT / 'build/native-codec/downloads')
    parser.add_argument('--offline', action='store_true')
    parser.add_argument('--archives', type=Path, help='Read pinned archives from a corresponding-source delivery')
    args = parser.parse_args()
    result = prepare(args.output, args.downloads, args.offline, args.archives)
    print('Prepared pinned official native codec sources')
