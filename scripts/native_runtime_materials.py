# scripts/native_runtime_materials.py

"""Collect hash-pinned upstream notices without selecting licenses or erasing gaps."""
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import shutil
import tarfile
import urllib.request

from deployment_audit import digest

ROOT = Path(__file__).resolve().parents[1]


def configuration():
    return json.loads((ROOT/'scripts/native-runtime-sources.json').read_text())


def fetch(item, cache):
    name = item['file']
    if PurePosixPath(name).name != name or '\\' in name or ':' in name:
        raise ValueError('Unbounded native material filename')
    cache.mkdir(parents=True, exist_ok=True)
    path = cache/name
    if path.is_symlink():
        raise ValueError('Native material cache must not be a symlink')
    if not path.exists():
        temporary = path.with_suffix(path.suffix+'.partial')
        try:
            with urllib.request.urlopen(item['url'], timeout=60) as source, temporary.open('wb') as output:
                shutil.copyfileobj(source, output)
            if digest(temporary) != item['sha256']:
                raise ValueError('Native material checksum mismatch')
            temporary.replace(path)
        finally:
            temporary.unlink(missing_ok=True)
    if digest(path) != item['sha256']:
        raise ValueError('Cached native material checksum mismatch')
    return path


def archive_notices(path, item):
    prefix = item['file'].removesuffix('.tar.xz') + '/'
    records, comments = {}, {}
    with tarfile.open(path, 'r:xz') as archive:
        for member in archive.getmembers():
            relative = PurePosixPath(member.name)
            if relative.is_absolute() or '..' in relative.parts or '\\' in member.name or not (member.name == prefix[:-1] or member.name.startswith(prefix)):
                raise ValueError('Unbounded native source member')
            name = member.name.removeprefix(prefix)
            if name in item['notices']:
                if not member.isfile() or member.size > 1024*1024:
                    raise ValueError('Invalid upstream notice member')
                records[name] = archive.extractfile(member).read()
            # Preserve original per-file notices as a conservative source-wide
            # reference. Unused drivers/targets are not asserted to be linked.
            if member.isfile() and relative.suffix.lower() in {'.c', '.h', '.cpp', '.hpp', '.cc'} and member.size <= 2*1024*1024:
                raw = archive.extractfile(member).read()
                try:
                    data = raw.decode('utf-8')
                except UnicodeDecodeError:
                    data = raw.decode('latin-1')
                for block in re.findall(r'/\*[\s\S]*?\*/|(?://[^\n]*(?:\n|$))+', data):
                    if re.search(r'copyright|permission|licen[cs]e|warranty', block, re.I):
                        key = hashlib.sha256(block.encode()).hexdigest()
                        comments.setdefault(key, {'source': name, 'text': block})
    if set(records) != set(item['notices']):
        raise ValueError('Missing upstream notice')
    records['SOURCE-NOTICES.txt'] = (
        'Source-wide reference notices, including unused components. This is not a linked-code license selection.\n\n' +
        '\n\n'.join(row['source']+'\n'+row['text'] for _, row in sorted(comments.items())) + '\n').encode('utf-8')
    return records


def prepare(output, platform, cache=None):
    config = configuration()
    cache = cache or ROOT/'build/native-source-cache'
    inventory = []
    if platform == 'windows':
        for item in config['archives']:
            source = fetch(item, cache)
            directory = output/(item['component']+'-'+item['version'])
            for name, data in archive_notices(source, item).items():
                target = directory/name
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(data)
                inventory.append({'file': target.relative_to(output).as_posix(),
                                  'sha256': hashlib.sha256(data).hexdigest()})
    elif platform == 'linux':
        item = config['icu']
        target = output/'ICU-73.2'/item['file']
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(fetch(item, cache), target)
        inventory.append({'file': target.relative_to(output).as_posix(), 'sha256': digest(target)})
    output.mkdir(parents=True, exist_ok=True)
    report = {'schema_version': 1, 'platform': platform, 'upstream': config, 'files': inventory,
              'limits': ['Pinned upstream notices and source references do not prove full supplier build equivalence or approve redistribution.']}
    (output/'native-runtime-materials.json').write_text(json.dumps(report, indent=2)+'\n')
    return report
