#!/usr/bin/env python3
"""Verify and collect the editable Public Suffix List used by pinned Qt."""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import tempfile

from deployment_audit import digest

ROOT = Path(__file__).resolve().parents[1]


def git_blob(data):
    return hashlib.sha1(b'blob ' + str(len(data)).encode('ascii') + b'\0' + data).hexdigest()


def collect_runtime_source(output, root=None):
    """Provide the preferred editable form beside the Qt runtime notice."""
    root = ROOT if root is None else root
    output = Path(output)
    config = json.loads((root/'scripts/qt-psl-source.json').read_bytes())
    inputs = root/'third_party/qt-public-suffix-list'
    source, license_file = inputs/'public_suffix_list.dat', inputs/'PSL-LICENSE.txt'
    if any(not path.is_file() or path.is_symlink() for path in [source, license_file]):
        raise ValueError('Qt Public Suffix List input missing or linked')
    data, license_text = source.read_bytes(), license_file.read_bytes()
    installed_license = output/'license-texts/MPL-2.0.txt'
    if hashlib.sha256(data).hexdigest() != config['source_sha256'] or \
            git_blob(data) != config['upstream_git_blob'] or \
            hashlib.sha256(license_text).hexdigest() != config['license_sha256'] or \
            not installed_license.is_file() or installed_license.is_symlink() or \
            installed_license.read_bytes() != license_text:
        raise ValueError('Qt Public Suffix List runtime source or license mismatch')
    destination = output/'public_suffix_list.dat'
    notice = output/'public-suffix-list-SOURCE.md'
    url = config['upstream_repository']+'/blob/'+config['upstream_commit']+'/public_suffix_list.dat'
    text = ('# Qt Network Public Suffix List source\n\n'
            'Qt Network includes generated data from the Public Suffix List under MPL-2.0. '
            'The preferred editable source form is the accompanying `public_suffix_list.dat`; '
            'the full license is in `license-texts/MPL-2.0.txt`.\n\n'
            'Qt Network 内嵌的 Public Suffix List 数据采用 MPL-2.0。可编辑源码为同目录的 '
            '`public_suffix_list.dat`，完整许可见 `license-texts/MPL-2.0.txt`。\n\n'
            f"Upstream revision: `{config['upstream_commit']}`\n\n"
            f"Source SHA-256: `{config['source_sha256']}`\n\n"
            f'Fixed upstream source: {url}\n')
    notice_data = text.encode('utf-8')
    for path, expected in [(destination, data), (notice, notice_data)]:
        if path.is_symlink() or (path.exists() and (not path.is_file() or path.read_bytes() != expected)):
            raise ValueError('Existing Qt Public Suffix List runtime destination differs')
    destination.write_bytes(data)
    notice.write_bytes(notice_data)
    return {'file': destination.name, 'sha256': config['source_sha256'],
            'license': 'MPL-2.0', 'notice': notice.name, 'source_url': url}


def collect(output, qt_archive, root=None):
    """Rebuild both Qt-generated forms before admitting the fixed source input."""
    root = ROOT if root is None else root
    output, qt_archive = Path(output), Path(qt_archive)
    config_path = root/'scripts/qt-psl-source.json'
    config_bytes = config_path.read_bytes()
    config = json.loads(config_bytes)
    qt_config = json.loads((root/'scripts/corresponding-sources.json').read_text(encoding='utf-8'))
    qtbase = [entry for entry in qt_config['archives'] if entry['module'] == 'qtbase']
    if len(qtbase) != 1 or qtbase[0]['file'] != config['qt_archive'] or \
            qtbase[0]['sha256'] != config['qt_archive_sha256'] or \
            qt_archive.name != config['qt_archive'] or digest(qt_archive) != config['qt_archive_sha256']:
        raise ValueError('Pinned Qt archive identity mismatch')
    inputs = root/'third_party/qt-public-suffix-list'
    names = ['public_suffix_list.dat', 'PSL-LICENSE.txt', 'README.md']
    if any(not (inputs/name).is_file() or (inputs/name).is_symlink() for name in names):
        raise ValueError('Qt Public Suffix List input missing or linked')
    data = (inputs/names[0]).read_bytes()
    license_text = (inputs/names[1]).read_bytes()
    if hashlib.sha256(data).hexdigest() != config['source_sha256'] or \
            git_blob(data) != config['upstream_git_blob'] or \
            hashlib.sha256(license_text).hexdigest() != config['license_sha256']:
        raise ValueError('Qt Public Suffix List input checksum mismatch')
    if not output.is_dir() or output.is_symlink() or (output/'qt-public-suffix-list').exists() or \
            (output/'qt-public-suffix-list').is_symlink():
        raise ValueError('Qt Public Suffix List destination unavailable')

    prefix = config['qt_archive'].removesuffix('.tar.xz')+'/'
    paths = {
        'generator': 'src/3rdparty/libpsl/src/psl-make-dafsa',
        'license': 'src/3rdparty/libpsl/PSL-LICENSE.txt',
        'attribution': 'src/3rdparty/libpsl/qt_attribution.json',
        'cpp': 'src/3rdparty/libpsl/psl_data.cpp',
        'binary': 'tests/auto/network/access/qnetworkcookiejar/testdata/publicsuffix/public_suffix_list.dafsa',
    }
    files = {}
    with tarfile.open(qt_archive, 'r:xz') as archive:
        for kind, path in paths.items():
            member = archive.getmember(prefix+path)
            if not member.isfile() or member.size > 8*1024*1024:
                raise ValueError('Invalid Qt Public Suffix List archive member')
            files[kind] = archive.extractfile(member).read()
    for kind, key in [('generator', 'generator_sha256'), ('cpp', 'generated_cpp_sha256'),
                      ('binary', 'generated_binary_sha256')]:
        if hashlib.sha256(files[kind]).hexdigest() != config[key]:
            raise ValueError('Qt Public Suffix List archive member checksum mismatch')
    if files['license'] != license_text or \
            config['upstream_commit'].encode('ascii') not in files['attribution'] or \
            b'"LicenseId": "MPL-2.0"' not in files['attribution']:
        raise ValueError('Qt Public Suffix List upstream attribution mismatch')

    with tempfile.TemporaryDirectory(prefix='qt-psl-check-', dir=output) as temporary:
        temporary = Path(temporary)
        generator = temporary/'psl-make-dafsa.py'
        generator.write_bytes(files['generator'])
        source = temporary/'public_suffix_list.dat'
        source.write_bytes(data)
        for kind, option in [('cpp', 'cxx'), ('binary', 'binary')]:
            generated = temporary/(kind+'.out')
            subprocess.run([sys.executable, str(generator), '--output-format='+option,
                            str(source), str(generated)], check=True,
                           stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=180)
            if generated.read_bytes() != files[kind]:
                raise ValueError('Qt Public Suffix List regenerated '+kind+' differs')

    destination = output/'qt-public-suffix-list'
    destination.mkdir()
    (destination/names[0]).write_bytes(data)
    (destination/names[1]).write_bytes(license_text)
    shutil.copyfile(inputs/names[2], destination/names[2])
    (destination/'source.json').write_bytes(config_bytes)
    return {'kind': 'qt-public-suffix-list-source',
            'file': 'qt-public-suffix-list/public_suffix_list.dat',
            'upstream_commit': config['upstream_commit'],
            'source_sha256': config['source_sha256'],
            'qt_archive_sha256': config['qt_archive_sha256'],
            'regenerated': ['psl_data.cpp', 'public_suffix_list.dafsa']}
