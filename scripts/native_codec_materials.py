# scripts/native_codec_materials.py

"""Attribution/source delivery for declared product codecs, currently empty.

Pinned independent test references and historical deliveries retain their own
notices; they do not enter this current product-source manifest.
"""
import hashlib
import json
from pathlib import Path
import shutil

ROOT = Path(__file__).resolve().parents[1]


def configuration():
    return json.loads((ROOT / 'scripts/native-codec-sources.json').read_text(encoding='utf-8'))['sources']


def notices(output):
    output = Path(output)
    files, dependencies = [], []
    for item in configuration():
        source = ROOT / 'build/native-codec/sources' / item['id'] / item['source_subdirectory']
        destination = output / (item['name'] + '-' + item['version'])
        destination.mkdir(parents=True, exist_ok=True)
        for relative in item['license_files']:
            target = destination / Path(relative).name
            shutil.copyfile(source / relative, target)
            files.append(target)
        (destination / 'source.json').write_text(json.dumps(item, indent=2) + '\n', encoding='utf-8')
        files.append(destination / 'source.json')
        dependencies.append(dict(name=item['name'], version=item['version'], license=item['license'],
                                 sdk=dict(url=item['url'], sha256=item['sha256']),
                                 source=dict(url=item['url'], sha256=item['sha256'])))
    return files, dependencies


def corresponding_sources(output):
    output = Path(output) / 'native-codecs'
    output.mkdir(parents=True, exist_ok=True)
    records = []
    for item in configuration():
        archive = ROOT / 'build/native-codec/downloads' / item['archive']
        if hashlib.sha256(archive.read_bytes()).hexdigest() != item['sha256']:
            raise ValueError('Native codec corresponding-source hash mismatch')
        shutil.copyfile(archive, output / item['archive'])
        records.append(dict(file='native-codecs/' + item['archive'], url=item['url'], sha256=item['sha256']))
    shutil.copyfile(ROOT / 'scripts/native-codec-sources.json', output / 'sources.json')
    return records
