"""Associate selected retained CLDR web assets with third-party notices."""
import hashlib
import json
import io
from pathlib import Path
import re
import stat
import tarfile
import zipfile

ROOT = Path(__file__).resolve().parents[1]
DIRECTORY = 'cldr43-webapp-notices'
ARCHIVE_ROOT = 'cldr-37d524354422f46731db74f3bd6b3f46516c3ced'
FONTS_DIRECTORY = 'tools/cldr-apps/src/main/webapp/fonts'
BOOTSTRAP_SCRIPT = 'tools/cldr-apps/src/main/webapp/js/bootstrap.min.js'
AUTOSIZE_SCRIPT = 'tools/cldr-apps/src/main/webapp/js/autosize.min.js'
EMOJI_DIRECTORY = 'tools/cldr-apps/src/main/webapp/images/emoji'
NOTO_EMOJI_COMMIT = '934a5706f1f3dd2605c4d9b5d9162fd7f89d8702'
NOTO_EMOJI_2021_COMMIT = 'de7f127a26f7044e5647490af9e3b187d9532703'
REPLACEMENT_PATH = EMOJI_DIRECTORY + '/emoji_1f699.png'
REPLACEMENT_FILE = 'Emoji-alternatives.zip'
REPLACEMENT_UPSTREAM = 'png/72/emoji_u1f699.png'
TWEMOJI_COMMIT = 'cb4fb472e7dc2a83c30391a67d8e5f6574306584'
REPLACEMENT_UPSTREAMS = {
    'noto': {'commit': NOTO_EMOJI_COMMIT,
             'archive_sha256': '3946753f8b148713877eb68af1d52d21ec3e6051ec8a1d2d6eed9674bb683e4f'},
    'twemoji': {'commit': TWEMOJI_COMMIT,
                'archive_sha256': '942ca5f79cf4ae178b93e7b6966f7a5a17c2176c14935b7074c00c7b8a9ec856'},
}
MANUAL_EMOJI_MAPPINGS = {
    'emoji_1f470_200d_2640 2.png': 'assets/72x72/1f470-200d-2640-fe0f.png',
    'emoji_1f62e_200d_1f4a8.png .png': 'assets/72x72/1f62e-200d-1f4a8.png',
    'emoji_1f635_200d_1f4ab.png.png': 'assets/72x72/1f635-200d-1f4ab.png',
    'emoji_1f636_200d_f32b.png': 'assets/72x72/1f636-200d-1f32b-fe0f.png',
}
FONT_FILES = frozenset(('LICENSE-Noto.txt', 'NotoSerifNyiakengPuachueHmong-Bold.otf',
                        'NotoSerifNyiakengPuachueHmong-Regular.ttf',
                        *(f'glyphicons-halflings-regular.{extension}'
                          for extension in ('eot', 'svg', 'ttf', 'woff'))))
NOTICE_FILES = frozenset(('Autosize-MIT.txt', 'Bootstrap-MIT.txt', 'Glyphicons-MIT.txt',
                          'Noto-Emoji-Apache-2.0.txt', 'Noto-OFL-1.1.txt', 'Twemoji-CC-BY-4.0.txt'))


def corresponding_emoji(name, record):
    """Compare code points, apart from emoji presentation selectors and reviewed irregular names."""
    if name in MANUAL_EMOJI_MAPPINGS:
        return record['source'] == 'twemoji' and record['upstream_path'] == MANUAL_EMOJI_MAPPINGS[name]
    original = re.fullmatch(r'emoji_([0-9a-f]+(?:_[0-9a-f]+)*)\.png', name)
    if not original:
        return False
    points = tuple(int(part, 16) for part in original.group(1).split('_') if part != 'fe0f')
    path = record['upstream_path'].rsplit('/', 1)[-1].removesuffix('.png')
    parts = path.removeprefix('emoji_u').split('_') if record['source'] == 'noto' else path.split('-')
    upstream = tuple(int(part, 16) for part in parts if part != 'fe0f')
    flag = (len(points) == 2 and all(0x1f1e6 <= part <= 0x1f1ff for part in points)) or (
        len(points) > 1 and points[0] == 0x1f3f4 and 0xe0061 <= points[1] <= 0xe007a)
    return points == upstream and not (record['source'] == 'noto' and flag)


def emoji_replacements(config=None, bundle=None):
    """Load pinned, individually hashed images without extracting archive paths."""
    source = ROOT / 'third_party' / DIRECTORY
    config_path = source / 'sources.json'
    bundle_path = source / REPLACEMENT_FILE
    if config is None or bundle is None:
        if any(path.is_symlink() or not path.is_file() for path in (config_path, bundle_path)):
            raise ValueError('CLDR replacement input must be a regular file')
        config, bundle = json.loads(config_path.read_bytes()), bundle_path.read_bytes()
    item = config.get('emoji_replacements', {})
    if (not isinstance(item, dict) or item.get('file') != REPLACEMENT_FILE or
            item.get('upstreams') != REPLACEMENT_UPSTREAMS or not isinstance(item.get('files'), dict) or
            not item['files'] or len(item['files']) > 5000 or len(bundle) > 10 * 1024 * 1024 or
            hashlib.sha256(bundle).hexdigest() != item.get('sha256')):
        raise ValueError('Unexpected CLDR emoji replacement configuration')
    result = {}
    try:
        with zipfile.ZipFile(io.BytesIO(bundle)) as archive:
            images = archive.infolist()
            if (len(images) != len(item['files']) or len({image.filename for image in images}) != len(images) or
                    {image.filename for image in images} != set(item['files']) or
                    sum(image.file_size for image in images) > 64 * 1024 * 1024):
                raise ValueError('Unexpected CLDR replacement bundle contents')
            for image in images:
                name, record = image.filename, item['files'][image.filename]
                if (not name.startswith('emoji_') or not name.endswith('.png') or len(name) > 200 or
                        any(part in name for part in ('/', '\\', '..', ':', '\0')) or
                        not isinstance(record, dict) or set(record) != {'original_sha256', 'sha256', 'source', 'upstream_path'} or
                        any(not isinstance(record.get(key), str) or not re.fullmatch(r'[0-9a-f]{64}', record[key])
                            for key in ('original_sha256', 'sha256')) or record['original_sha256'] == record['sha256'] or
                        record.get('source') not in REPLACEMENT_UPSTREAMS or not isinstance(record.get('upstream_path'), str) or
                        not re.fullmatch(r'png/72/emoji_u[0-9a-f_]+\.png' if record['source'] == 'noto' else
                                         r'assets/72x72/[0-9a-f-]+\.png', record['upstream_path']) or
                        not corresponding_emoji(name, record) or
                        not stat.S_ISREG(image.external_attr >> 16) or image.file_size > 1024 * 1024):
                    raise ValueError('Unexpected CLDR replacement image metadata')
                data = archive.read(image)
                if not data.startswith(b'\x89PNG\r\n\x1a\n') or hashlib.sha256(data).hexdigest() != record['sha256']:
                    raise ValueError('CLDR emoji replacement checksum mismatch')
                result[name] = {**record, 'data': data}
    except (zipfile.BadZipFile, OSError, RuntimeError) as error:
        raise ValueError('Invalid CLDR replacement bundle') from error
    return result


def collect(output, subset):
    source = ROOT / 'third_party' / DIRECTORY
    names = ('README.md', 'sources.json', *sorted(NOTICE_FILES), REPLACEMENT_FILE)
    inputs = {}
    for name in names:
        path = source / name
        if path.is_symlink() or not path.is_file():
            raise ValueError('CLDR web font notice input must be a regular file')
        inputs[name] = path.read_bytes()
    config = json.loads(inputs['sources.json'])
    replacements = emoji_replacements(config, inputs[REPLACEMENT_FILE])
    emoji = config.get('noto_emoji', {})
    if not isinstance(emoji, dict):
        raise ValueError('Unexpected CLDR web notice configuration')
    emoji_files = emoji.get('files', {})
    emoji_2021_files = emoji.get('files_2021', {})
    if not isinstance(emoji_files, dict) or not isinstance(emoji_2021_files, dict):
        raise ValueError('Unexpected CLDR web notice configuration')
    if (config.get('schema_version') != 1 or config.get('kind') != DIRECTORY or
            config.get('archive_root') != ARCHIVE_ROOT or config.get('fonts_directory') != FONTS_DIRECTORY or
            config.get('bootstrap_script', {}).get('file') != BOOTSTRAP_SCRIPT or
            config.get('autosize_script', {}).get('file') != AUTOSIZE_SCRIPT or
            emoji.get('directory') != EMOJI_DIRECTORY or emoji.get('upstream_commit') != NOTO_EMOJI_COMMIT or
            emoji.get('upstream_commit_2021') != NOTO_EMOJI_2021_COMMIT or
            not emoji_files or not emoji_2021_files or set(emoji_files).intersection(emoji_2021_files) or
            any(not re.fullmatch(r'emoji_[0-9a-f_]+\.png', name) for name in (*emoji_files, *emoji_2021_files)) or
            set(config.get('fonts', {})) != FONT_FILES or set(config.get('notices', {})) != NOTICE_FILES or
            any(not re.fullmatch(r'[0-9a-f]{64}', value)
                for value in (*config['fonts'].values(), *config['notices'].values(),
                              *emoji_files.values(), *emoji_2021_files.values(),
                              config['bootstrap_script'].get('sha256', ''),
                              config['autosize_script'].get('sha256', '')))):
        raise ValueError('Unexpected CLDR web font notice configuration')
    emoji_files = {**emoji_files, **emoji_2021_files}
    if set(emoji_files).intersection(replacements):
        raise ValueError('CLDR matched and substituted emoji must be disjoint')
    for name, expected in config['notices'].items():
        if hashlib.sha256(inputs[name]).hexdigest() != expected:
            raise ValueError('CLDR web font notice checksum mismatch')

    prefix = ARCHIVE_ROOT + '/' + FONTS_DIRECTORY + '/'
    with tarfile.open(subset, 'r:gz') as archive:
        members = [member for member in archive if member.name.startswith(prefix)]
        if (len(members) != len(FONT_FILES) or any(not member.isfile() for member in members) or
                {member.name.removeprefix(prefix) for member in members} != FONT_FILES):
            raise ValueError('Unexpected CLDR web font files')
        for member in members:
            if member.size > 1024 * 1024:
                raise ValueError('Oversized CLDR web font file')
            if hashlib.sha256(archive.extractfile(member).read()).hexdigest() != config['fonts'][member.name.removeprefix(prefix)]:
                raise ValueError('CLDR web font source checksum mismatch')
        for key, path in (('bootstrap_script', BOOTSTRAP_SCRIPT), ('autosize_script', AUTOSIZE_SCRIPT)):
            scripts = [member for member in archive if member.name == ARCHIVE_ROOT+'/'+path]
            if (len(scripts) != 1 or not scripts[0].isfile() or scripts[0].size > 1024 * 1024 or
                    hashlib.sha256(archive.extractfile(scripts[0]).read()).hexdigest() != config[key]['sha256']):
                raise ValueError('CLDR web script checksum mismatch')
        emoji_prefix = ARCHIVE_ROOT + '/' + EMOJI_DIRECTORY + '/'
        images = [member for member in archive if member.name.startswith(emoji_prefix)]
        expected_images = set(emoji_files) | set(replacements)
        if (len(images) != len(expected_images) or any(not member.isfile() for member in images) or
                {member.name.removeprefix(emoji_prefix) for member in images} != expected_images):
            raise ValueError('Unexpected CLDR emoji image set')
        for member in images:
            name = member.name.removeprefix(emoji_prefix)
            expected = emoji_files[name] if name in emoji_files else replacements[name]['sha256']
            if member.size > 1024 * 1024 or hashlib.sha256(archive.extractfile(member).read()).hexdigest() != expected:
                raise ValueError('CLDR emoji image checksum mismatch')

    destination = Path(output) / DIRECTORY
    if destination.exists() or destination.is_symlink():
        raise ValueError('CLDR web font notice destination already exists')
    destination.mkdir(parents=True)
    for name in names:
        (destination / name).write_bytes(inputs[name])
    return {'kind': DIRECTORY, 'file': DIRECTORY + '/sources.json', 'font_files': len(FONT_FILES) - 1,
            'bootstrap_script': True, 'autosize_script': True,
            'noto_emoji_files': len(emoji_files),
            'substituted_emoji_files': len(replacements),
            'scope': 'CLDR 43.1 webapp fonts, Bootstrap/Autosize scripts and listed emoji images only'}
