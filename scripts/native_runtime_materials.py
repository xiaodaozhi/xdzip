# scripts/native_runtime_materials.py

"""Collect hash-pinned upstream notices without selecting licenses or erasing gaps."""
import json
from pathlib import Path, PurePosixPath
import shutil
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


def prepare(output, platform, cache=None):
    config = configuration()
    cache = cache or ROOT/'build/native-source-cache'
    inventory = []
    if platform == 'windows' and any((output/name).exists() for name in ('Mesa-11.2.2', 'LLVM-3.6.2')):
        raise ValueError('Obsolete software OpenGL notices found; use a clean output directory')
    if platform == 'linux':
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


def installer_notices(tools, output):
    config = json.loads((ROOT/'scripts/installer-toolchain.json').read_text())['inno_setup']
    items = [i for i in config['materials'] if i['file'] in {'InnoSetup-license.txt', 'InnoSetup-PascalScript-license.md'}]
    if len(items) != 2 or any(digest(tools/i['file']) != i['sha256'] for i in items):
        raise ValueError('Installer notice hash mismatch')
    output.mkdir(parents=True, exist_ok=True)
    for item in items:
        shutil.copyfile(tools/item['file'], output/item['file'])
    # Pascal Script's license explicitly requires an acknowledgement in the
    # program's About box OR documentation, not just a generic license label.
    (output/'Installer-CREDITS.txt').write_text(
        'Windows 安装程序使用 Inno Setup 6.7.3，https://jrsoftware.org/。\n'
        '安装程序使用 RemObjects Pascal Script 制作，https://www.remobjects.com/ps。\n'
        '完整许可见 InnoSetup-license.txt 和 InnoSetup-PascalScript-license.md。\n\n'
        'The Windows installer is made using Inno Setup 6.7.3 (https://jrsoftware.org/)\n'
        'and RemObjects Pascal Script (https://www.remobjects.com/ps).\n'
        'Original licenses accompany this acknowledgement.\n', encoding='utf-8')
