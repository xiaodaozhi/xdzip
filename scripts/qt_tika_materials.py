#!/usr/bin/env python3
"""Verify and deliver the pinned Tika MIME input and relevant notices."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile

from deployment_audit import digest
from qt_psl_source import git_blob

ROOT = Path(__file__).resolve().parents[1]
ASF_NOTICE = (b'Apache Tika\nCopyright 2021 The Apache Software Foundation\n\n'
              b'This product includes software developed at\n'
              b'The Apache Software Foundation (http://www.apache.org/).')
PRONOM_NOTICE = (
    b'Tika-mimetypes.xml includes mimetype definitions that were adapted from the PRONOM Technical Registry\n'
    b'by The National Archives (http://www.nationalarchives.gov.uk/PRONOM/Default.aspx). PRONOM is published\n'
    b'under the Open Government License 3.0 (http://www.nationalarchives.gov.uk/doc/open-government-licence/version/3/)'
)


def inputs(root):
    config = json.loads((root/'scripts/qt-tika-materials.json').read_bytes())
    directory = root/'third_party/qt-tika-mimetypes'
    data = {}
    for name, sha, blob in [
        ('tika-mimetypes.xml.orig', 'source_sha256', 'source_git_blob'),
        ('TIKA-NOTICE.txt', 'notice_sha256', 'notice_git_blob'),
        ('OGL-UK-3.0.txt', 'ogl_sha256', 'ogl_git_blob'),
    ]:
        path = directory/name
        if not path.is_file() or path.is_symlink():
            raise ValueError('Qt Tika input missing or linked: '+name)
        content = path.read_bytes()
        if hashlib.sha256(content).hexdigest() != config[sha] or git_blob(content) != config[blob]:
            raise ValueError('Qt Tika input checksum mismatch: '+name)
        data[name] = content
    if ASF_NOTICE not in data['TIKA-NOTICE.txt'] or PRONOM_NOTICE not in data['TIKA-NOTICE.txt']:
        raise ValueError('Qt Tika relevant upstream notice missing')
    readme = directory/'README.md'
    if not readme.is_file() or readme.is_symlink():
        raise ValueError('Qt Tika source explanation missing or linked')
    return config, data, readme.read_bytes()


def collect_runtime(output, root=None):
    """Copy original notices and the relevant attribution without overwrites."""
    root = ROOT if root is None else Path(root)
    output = Path(output)
    config, data, _ = inputs(root)
    apache = output/'license-texts/Apache-2.0.txt'
    if not output.is_dir() or output.is_symlink() or not apache.is_file() or apache.is_symlink() or \
            digest(apache) != config['apache_license_sha256']:
        raise ValueError('Qt Tika Apache license or output mismatch')
    notice = (
        '# Qt Core Apache Tika MIME data\n\n'
        'Qt Core includes adapted Apache Tika MIME definitions. The following entries are '
        'reproduced from the pinned upstream NOTICE:\n\n'
    ).encode('utf-8') + ASF_NOTICE + b'\n\n' + PRONOM_NOTICE + (
        '\n\nContains public sector information licensed under the Open Government Licence v3.0.\n\n'
        'The Apache-2.0 license is in `../license-texts/Apache-2.0.txt`; the OGL-UK-3.0 text '
        'is in `OGL-UK-3.0.txt`. `TIKA-NOTICE.txt` is the unmodified upstream NOTICE for '
        'the broader Tika distribution, not a list of dependencies shipped with xdzip.\n\n'
        'Qt Core 包含改编自 Apache Tika 的 MIME 定义。以上保留该数据相关的上游署名；'
        '完整上游 NOTICE 涉及更大范围的 Tika 发行，不表示其他依赖均随 xdzip 提供。\n\n'
        f"Tika source revision: `{config['tika_commit']}`\n\n"
        f"Source: {config['tika_repository']}/blob/{config['tika_commit']}/{config['tika_path']}\n"
    ).encode('utf-8')
    destination = output/'qt-tika-mimetypes'
    if destination.is_symlink() or (destination.exists() and not destination.is_dir()):
        raise ValueError('Qt Tika runtime destination unavailable')
    files = {'ATTRIBUTION.md': notice, 'TIKA-NOTICE.txt': data['TIKA-NOTICE.txt'],
             'OGL-UK-3.0.txt': data['OGL-UK-3.0.txt']}
    for name, expected in files.items():
        path = destination/name
        if path.is_symlink() or (path.exists() and (not path.is_file() or path.read_bytes() != expected)):
            raise ValueError('Existing Qt Tika runtime destination differs')
    destination.mkdir(exist_ok=True)
    for name, content in files.items():
        (destination/name).write_bytes(content)
    return {'attribution': 'qt-tika-mimetypes/ATTRIBUTION.md',
            'upstream_notice': 'qt-tika-mimetypes/TIKA-NOTICE.txt',
            'ogl_license': 'qt-tika-mimetypes/OGL-UK-3.0.txt',
            'apache_license': 'license-texts/Apache-2.0.txt',
            'tika_commit': config['tika_commit']}


def collect(output, qt_archive, root=None):
    """Regenerate the fixed Qt source before admitting the original Tika input."""
    root = ROOT if root is None else Path(root)
    output, qt_archive = Path(output), Path(qt_archive)
    config, data, readme = inputs(root)
    qt_config = json.loads((root/'scripts/corresponding-sources.json').read_bytes())
    qtbase = [entry for entry in qt_config['archives'] if entry['module'] == 'qtbase']
    if len(qtbase) != 1 or qtbase[0]['file'] != config['qt_archive'] or \
            qtbase[0]['sha256'] != config['qt_archive_sha256'] or \
            qt_archive.name != config['qt_archive'] or qt_archive.is_symlink() or \
            digest(qt_archive) != config['qt_archive_sha256']:
        raise ValueError('Pinned Qt Tika archive identity mismatch')
    destination = output/'qt-tika-mimetypes'
    if not output.is_dir() or output.is_symlink() or destination.exists() or destination.is_symlink():
        raise ValueError('Qt Tika source destination unavailable')
    prefix = config['qt_archive'].removesuffix('.tar.xz')+'/src/corelib/mimetypes/3rdparty/'
    files = {}
    with tarfile.open(qt_archive, 'r:xz') as archive:
        for name in ['process_tika_mimetypes.py', 'tika-mimetypes.xml', 'qt_attribution.json']:
            member = archive.getmember(prefix+name)
            if not member.isfile() or member.size > 8*1024*1024:
                raise ValueError('Invalid Qt Tika archive member')
            files[name] = archive.extractfile(member).read()
    if hashlib.sha256(files['process_tika_mimetypes.py']).hexdigest() != config['generator_sha256'] or \
            hashlib.sha256(files['tika-mimetypes.xml']).hexdigest() != config['qt_generated_sha256']:
        raise ValueError('Qt Tika archive member checksum mismatch')
    declarations = json.loads(files['qt_attribution.json'])
    selected = [entry for entry in declarations if entry.get('Id') == 'tika-mimetypes']
    if len(selected) != 1 or selected[0].get('Version') != config['tika_commit'] or \
            selected[0].get('LicenseId') != 'Apache-2.0':
        raise ValueError('Qt Tika archive attribution mismatch')
    with tempfile.TemporaryDirectory(prefix='qt-tika-check-', dir=output) as temporary:
        temporary = Path(temporary)
        (temporary/'tika-mimetypes.xml.orig').write_bytes(data['tika-mimetypes.xml.orig'])
        generator = temporary/'process_tika_mimetypes.py'
        generator.write_bytes(files['process_tika_mimetypes.py'])
        environment = dict(os.environ, PYTHONUTF8='1')
        subprocess.run([sys.executable, str(generator)], cwd=temporary, env=environment,
                       check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=180)
        generated = (temporary/'tika-mimetypes.xml').read_bytes()
    expected = files['tika-mimetypes.xml']
    if generated == expected:
        mode = 'byte-identical'
    elif os.name == 'nt' and generated == expected.replace(b'\n', b'\r\n'):
        mode = 'windows-native-line-endings'
    else:
        raise ValueError('Qt Tika regenerated data differs')
    destination.mkdir()
    for name, content in data.items():
        (destination/name).write_bytes(content)
    (destination/'README.md').write_bytes(readme)
    (destination/'source.json').write_bytes((root/'scripts/qt-tika-materials.json').read_bytes())
    return {'kind': 'qt-tika-mimetypes-source', 'file': 'qt-tika-mimetypes/tika-mimetypes.xml.orig',
            'tika_commit': config['tika_commit'], 'source_sha256': config['source_sha256'],
            'qt_archive_sha256': config['qt_archive_sha256'], 'regeneration': mode}
