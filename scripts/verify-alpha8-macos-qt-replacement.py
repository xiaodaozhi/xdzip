#!/usr/bin/env python3
"""Locally test a source-built QtCore in an isolated published alpha.8 Mac app."""
import argparse
import importlib.util
import json
import os
from pathlib import Path, PurePosixPath
import posixpath
import shutil
import stat
import subprocess
import sys
import zipfile

from deployment_audit import ROOT, audit, digest, mac_rpaths, within
from macos_attribution import compare_images
from macos_launch_services import temporary_bundle_path, transient_registrations

PACKAGE = {'name': 'xdzip-0.1.0-alpha.8-macos-arm64.app.zip',
           'sha256': 'edc205103256f84dfbeef504b7d1d9dfb327f6bc8c975540dc07ed769c0a420f'}


def module(name):
    spec = importlib.util.spec_from_file_location(name, ROOT/'scripts'/(name+'.py'))
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


replacement = module('verify-alpha8-qt-replacement')


def require_local_macos(platform, ci):
    if platform != 'darwin' or ci:
        raise ValueError('This app-copy signing probe is restricted to a local Mac, never CI')


def inspect_archive(path):
    """Preflight every file and internal symlink before invoking ditto."""
    with zipfile.ZipFile(path) as archive:
        entries = archive.infolist()
        if not entries or len(entries) > 10000 or sum(item.file_size for item in entries) > 512*1024*1024:
            raise ValueError('Unexpected Mac package size or entry count')
        replacement.check_member_names([entry.filename for entry in entries])
        links = []
        names = {entry.filename.rstrip('/') for entry in entries}
        for entry in entries:
            relative = PurePosixPath(entry.filename)
            if not relative.parts or relative.parts[0] != 'xdzip.app':
                raise ValueError('Mac package has a path outside the application')
            mode = entry.external_attr >> 16
            kind = stat.S_IFMT(mode)
            if kind not in {0, stat.S_IFREG, stat.S_IFDIR, stat.S_IFLNK}:
                raise ValueError('Unsupported Mac package entry type')
            if stat.S_ISLNK(mode):
                if entry.file_size > 4096:
                    raise ValueError('Mac package link is too large')
                target = archive.read(entry).decode('utf-8')
                if not target or '\\' in target or '\0' in target or target.startswith('/'):
                    raise ValueError('Unsafe Mac package link')
                resolved = posixpath.normpath(posixpath.join(posixpath.dirname(entry.filename), target))
                if not resolved.startswith('xdzip.app/'):
                    raise ValueError('Mac package link escapes the application')
                links.append(entry.filename.rstrip('/'))
        if any(any(name.startswith(link+'/') for name in names) for link in links):
            raise ValueError('Mac package contains entries beneath a symlink')
        return {'entries': len(entries), 'internal_links': len(links)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--qt-build', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--archive', type=Path, help='Existing exact public app.zip for offline use')
    args = parser.parse_args()
    require_local_macos(sys.platform, os.environ.get('CI'))
    output, qt_build = args.output.resolve(), args.qt_build.resolve()
    if output.exists() or not within(output, ROOT/'build'):
        parser.error('Use a new local build/ output directory')
    source = json.loads((qt_build/'qt-source-build.json').read_text(encoding='utf-8'))
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    core_source = qt_build/'qt-install/lib/QtCore.framework/Versions/A/QtCore'
    if source['revision'] != revision or source['source_dirty'] or \
            source['status'] != 'source-build-passed-not-sdk-equivalence' or \
            not core_source.is_file() or core_source.is_symlink() or digest(core_source) != source['qtcore_sha256']:
        raise ValueError('Source-built QtCore identity differs')
    output.mkdir(parents=True)
    if args.archive:
        archive = args.archive.resolve()
        if not archive.is_file() or digest(archive) != PACKAGE['sha256']:
            raise ValueError('Public alpha.8 Mac package checksum mismatch')
    else:
        archive = output/PACKAGE['name']
        replacement.acquire(archive, PACKAGE)
    archive_check = inspect_archive(archive)
    stage = output/'portable'; stage.mkdir()
    app = stage/'xdzip.app'
    mac = module('build-macos-dmg')
    with temporary_bundle_path(app), transient_registrations(output):
        subprocess.run(['ditto', '-x', '-k', str(archive), str(stage)], check=True,
                       capture_output=True, timeout=120)
        binary = mac.validate_bundle(app)
        core = app/'Contents/Frameworks/QtCore.framework/Versions/A/QtCore'
        plugin = app/'Contents/PlugIns/platforms/libqcocoa.dylib'
        if any(not within(path, app) or not path.is_file() or path.is_symlink()
               for path in [binary, core, plugin]):
            raise ValueError('Expected released Mac application, QtCore or Cocoa plugin missing')
        app_original = binary.read_bytes()
        app_original_sha = digest(binary)
        core_original_sha = digest(core)
        environment = os.environ.copy()
        for key in ['QT_PLUGIN_PATH', 'QML_IMPORT_PATH', 'QML2_IMPORT_PATH', 'QT_QPA_PLATFORM_PLUGIN_PATH',
                    'QT_QUICK_BACKEND', 'LD_LIBRARY_PATH', 'DYLD_LIBRARY_PATH', 'DYLD_FRAMEWORK_PATH',
                    'XDZIP_PROBE_OPERATION', 'XDZIP_PROBE_EXTRACT_TARGET']:
            environment.pop(key, None)
        environment.update(PATH='/usr/bin:/bin:/usr/sbin:/sbin', QT_QPA_PLATFORM='cocoa',
                           QSG_RHI_BACKEND='metal', XDZIP_SETTINGS_PATH=str(output/'isolated/settings.ini'),
                           XDZIP_EXPECT_QT_SOURCE_PROOF=replacement.PROOF, XDZIP_PROBE_IDLE_MS='100')
        version = json.loads(subprocess.check_output([str(binary), '--version-json'], env=environment, text=True))
        original_revision = version.get('revision', '')
        if version.get('version') != '0.1.0-alpha.8' or version.get('build') != 21 or \
                len(original_revision) < 12 or not replacement.REVISION.startswith(original_revision) or \
                version.get('dirty') is not False:
            raise ValueError('Original alpha.8 Mac package identity differs')
        fixture = output/'metadata.zip'
        subprocess.run([sys.executable, str(ROOT/'scripts/generate-m2-fixture.py'), '--output', str(fixture)], check=True)
        log = output/'replacement.log'
        negative = replacement.run_probe(binary, fixture, environment, log, expected_code=10)
        if 'qt_source_proof' not in negative or negative['qt_source_proof'].get('value') == replacement.PROOF:
            raise ValueError('Unmodified Mac QtCore unexpectedly passed source marker')
        shutil.copyfile(core_source, core)
        for value in mac_rpaths(mac.checked(['otool', '-l', core])):
            if value.startswith('/') and not value.startswith(('/usr/lib/', '/System/Library/')):
                mac.checked(['install_name_tool', '-delete_rpath', value, core])
        mac.sign_local(app)
        fingerprint = compare_images(binary.read_bytes(), app_original)
        if digest(core) == core_original_sha or audit(app, binary, None, 'macos', 'deployed')['status'] != 'passed':
            raise ValueError('Modified Mac QtCore unchanged or depends on the build SDK')
        browse = replacement.run_probe(binary, fixture, environment, log)
        replacement.validate_success(browse, 100000, 'browse')
        payload = output/'payload.zip'
        subprocess.run([sys.executable, str(ROOT/'scripts/generate-m3-fixture.py'), '--output', str(payload)], check=True)
        target = output/'extracted'; target.mkdir(mode=0o700)
        environment.update(XDZIP_PROBE_OPERATION='extract', XDZIP_PROBE_EXTRACT_TARGET=str(target))
        extraction = replacement.run_probe(binary, payload, environment, log)
        replacement.validate_success(extraction, 64, 'extract')
        files = replacement.verify_output(payload, target)
        result = {'status': 'passed', 'platform': 'macos', 'release_revision': replacement.REVISION,
                  'release_archive_sha256': PACKAGE['sha256'], 'source_revision': revision,
                  'source_build_sha256': digest(qt_build/'qt-source-build.json'),
                  'source_qtcore_sha256': source['qtcore_sha256'], 'original_qtcore_sha256': core_original_sha,
                  'signed_qtcore_sha256': digest(core), 'original_app_sha256': app_original_sha,
                  'resigned_app_sha256': digest(binary), 'app_sections_uuid_imports_match': fingerprint,
                  'unmodified_marker_rejected': True, 'modified_marker_loaded': True,
                  'archive_preflight': archive_check, 'browse_records': 100000, 'extracted_files_verified': files,
                  'developer_id_signed': False, 'notarized': False,
                  'limits': ['Local isolated original app copy, ad-hoc signed for integrity; original package unchanged.',
                             'One compatible source-built QtCore, not arbitrary ABI or full SDK binary equivalence.',
                             'Controlled native interaction, not manual desktop or legal approval.']}
    (output/'alpha8-macos-qt-replacement.json').write_text(json.dumps(result, indent=2)+'\n', encoding='utf-8')
    print('Local alpha.8 app loaded modified QtCore; browsing, cancellation and extraction passed')


if __name__ == '__main__':
    main()
