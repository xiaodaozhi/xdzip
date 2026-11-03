"""Associate selected retained CLDR web assets with third-party notices."""
import hashlib
import json
from pathlib import Path
import re
import tarfile

ROOT = Path(__file__).resolve().parents[1]
DIRECTORY = 'cldr43-webapp-notices'
ARCHIVE_ROOT = 'cldr-37d524354422f46731db74f3bd6b3f46516c3ced'
FONTS_DIRECTORY = 'tools/cldr-apps/src/main/webapp/fonts'
BOOTSTRAP_SCRIPT = 'tools/cldr-apps/src/main/webapp/js/bootstrap.min.js'
AUTOSIZE_SCRIPT = 'tools/cldr-apps/src/main/webapp/js/autosize.min.js'
FONT_FILES = frozenset(('LICENSE-Noto.txt', 'NotoSerifNyiakengPuachueHmong-Bold.otf',
                        'NotoSerifNyiakengPuachueHmong-Regular.ttf',
                        *(f'glyphicons-halflings-regular.{extension}'
                          for extension in ('eot', 'svg', 'ttf', 'woff'))))
NOTICE_FILES = frozenset(('Autosize-MIT.txt', 'Bootstrap-MIT.txt', 'Glyphicons-MIT.txt', 'Noto-OFL-1.1.txt'))


def collect(output, subset):
    source = ROOT / 'third_party' / DIRECTORY
    names = ('README.md', 'sources.json', *sorted(NOTICE_FILES))
    inputs = {}
    for name in names:
        path = source / name
        if path.is_symlink() or not path.is_file():
            raise ValueError('CLDR web font notice input must be a regular file')
        inputs[name] = path.read_bytes()
    config = json.loads(inputs['sources.json'])
    if (config.get('schema_version') != 1 or config.get('kind') != DIRECTORY or
            config.get('archive_root') != ARCHIVE_ROOT or config.get('fonts_directory') != FONTS_DIRECTORY or
            config.get('bootstrap_script', {}).get('file') != BOOTSTRAP_SCRIPT or
            config.get('autosize_script', {}).get('file') != AUTOSIZE_SCRIPT or
            set(config.get('fonts', {})) != FONT_FILES or set(config.get('notices', {})) != NOTICE_FILES or
            any(not re.fullmatch(r'[0-9a-f]{64}', value)
                for value in (*config['fonts'].values(), *config['notices'].values(),
                              config['bootstrap_script'].get('sha256', ''),
                              config['autosize_script'].get('sha256', '')))):
        raise ValueError('Unexpected CLDR web font notice configuration')
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

    destination = Path(output) / DIRECTORY
    if destination.exists() or destination.is_symlink():
        raise ValueError('CLDR web font notice destination already exists')
    destination.mkdir(parents=True)
    for name in names:
        (destination / name).write_bytes(inputs[name])
    return {'kind': DIRECTORY, 'file': DIRECTORY + '/sources.json', 'font_files': len(FONT_FILES) - 1,
            'bootstrap_script': True, 'autosize_script': True,
            'scope': 'CLDR 43.1 webapp fonts and Bootstrap/Autosize scripts only'}
