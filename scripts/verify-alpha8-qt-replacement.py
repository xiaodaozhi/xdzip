#!/usr/bin/env python3
"""Test a modified source-built QtCore in isolated alpha.8 portable packages."""
import argparse
import json
import os
from pathlib import Path, PurePosixPath
import shutil
import stat
import subprocess
import sys
import tarfile
import urllib.request
import zipfile

from deployment_audit import ROOT, digest, within

REVISION = 'e1cd6a3e125d5fcabc85a9f4edb106d013709c89'
PROOF = 'xdzip-qt-6.8.2-source-proof-v1'
PACKAGES = {
    'windows': {'name': 'xdzip-0.1.0-alpha.8-windows-x86_64.zip',
                'sha256': '4a808747b47f039c16d6967d9770381007861b7217856f58860aaa4b3ba2ec27',
                'app': 'xdzip.exe', 'core': 'Qt6Core.dll', 'source_core': 'bin/Qt6Core.dll',
                'platform_plugin': 'platforms/qwindows.dll'},
    'linux': {'name': 'xdzip-0.1.0-alpha.8-linux-x86_64.tar.gz',
              'sha256': '7ba07fc05e2e5013561132629bb7dfe7bc0a84e7f5043eadf5b98bf30d4f9318',
              'app': 'xdzip-linux-x86_64/xdzip', 'core': 'xdzip-linux-x86_64/lib/libQt6Core.so.6',
              'source_core': 'lib/libQt6Core.so.6.8.2',
              'platform_plugin': 'xdzip-linux-x86_64/plugins/platforms/libqoffscreen.so'},
}


def check_member_names(names):
    seen = set()
    for name in names:
        path = PurePosixPath(name)
        if not name or path.is_absolute() or '..' in path.parts or '\\' in name or ':' in name:
            raise ValueError('Unsafe archive member')
        normalized = path.as_posix()
        if normalized in seen:
            raise ValueError('Duplicate archive member')
        seen.add(normalized)


def unpack(archive_path, target, platform):
    """Only a separately SHA-256 checked official archive may reach this step."""
    if platform == 'windows':
        with zipfile.ZipFile(archive_path) as archive:
            members = archive.infolist()
            check_member_names([item.filename for item in members])
            if any(stat.S_ISLNK(item.external_attr >> 16) for item in members):
                raise ValueError('Unexpected portable ZIP symlink')
            archive.extractall(target)
    else:
        with tarfile.open(archive_path, 'r:gz') as archive:
            members = archive.getmembers()
            check_member_names([item.name for item in members])
            archive.extractall(target, members=members, filter='data')


def acquire(path, item):
    """Fetch the immutable public asset, enforcing size and content identity."""
    url = 'https://github.com/xiaodaozhi/xdzip/releases/download/v0.1.0-alpha.8/'+item['name']
    count = 0
    with urllib.request.urlopen(url, timeout=90) as source, path.open('xb') as target:
        while block := source.read(1024*1024):
            count += len(block)
            if count > 128*1024*1024:
                raise ValueError('Public package exceeds expected size limit')
            target.write(block)
    if digest(path) != item['sha256']:
        raise ValueError('Public alpha.8 package checksum mismatch')


def events_from(result):
    events = []
    for line in result.stdout.splitlines():
        if line.startswith('XDZIP_PROBE '):
            events.append(json.loads(line.removeprefix('XDZIP_PROBE ')))
    names = [event['event'] for event in events]
    if len(names) != len(set(names)):
        raise ValueError('Duplicate product probe event')
    return {event['event']: event for event in events}


def verify_output(fixture, target):
    expected = set()
    with zipfile.ZipFile(fixture) as archive:
        check_member_names([item.filename for item in archive.infolist()])
        if sum(item.file_size for item in archive.infolist() if not item.is_dir()) != 32*1024*1024:
            raise ValueError('Owned extraction fixture size differs')
        for item in archive.infolist():
            if item.is_dir():
                continue
            destination = target/item.filename
            if not within(destination, target) or destination.is_symlink() or not destination.is_file():
                raise ValueError('Extracted file missing or unsafe')
            with archive.open(item) as original, destination.open('rb') as extracted:
                while True:
                    block = original.read(1024*1024)
                    if extracted.read(len(block) if block else 1) != block:
                        raise ValueError('Extracted bytes differ from owned fixture')
                    if not block:
                        break
            expected.add(item.filename)
    actual = {path.relative_to(target).as_posix() for path in target.rglob('*') if path.is_file()}
    if actual != expected or len(actual) != 64:
        raise ValueError('Unexpected extracted file set')
    return len(actual)


def run_probe(app, fixture, environment, log, expected_code=0):
    try:
        result = subprocess.run([str(app), '--desktop-probe', str(fixture)], env=environment,
                                capture_output=True, text=True, encoding='utf-8', errors='replace', timeout=220)
    except subprocess.TimeoutExpired as error:
        with log.open('a', encoding='utf-8') as output:
            for stream in [error.stdout, error.stderr]:
                if stream:
                    output.write(stream.decode('utf-8', errors='replace') if isinstance(stream, bytes) else stream)
        raise ValueError('Released product probe timed out; inspect replacement log') from error
    with log.open('a', encoding='utf-8') as output:
        output.write(result.stdout+'\n'+result.stderr+'\n')
    if result.returncode != expected_code:
        raise ValueError('Released product probe exited unexpectedly; inspect replacement log')
    return events_from(result)


def validate_success(events, records, operation):
    required = {'first_frame', 'qt_source_proof', 'open_ready', 'search_frame', 'sort_frame',
                'browse_stable', 'quit_requested'}
    required |= {'extract_finished'} if operation == 'extract' else {'cancel_requested', 'cancel_settled'}
    if not required <= events.keys() or events['qt_source_proof'].get('value') != PROOF or \
            events['open_ready'].get('records') != records:
        raise ValueError('Modified Qt load or product interaction proof missing')
    identity = events['first_frame']
    revision = identity.get('revision', '')
    if identity.get('version') != '0.1.0-alpha.8' or identity.get('build') != 21 or \
            len(revision) < 12 or not REVISION.startswith(revision) or identity.get('dirty') is not False:
        raise ValueError('Released product probe identity differs')
    if operation == 'extract' and events['extract_finished'].get('status') != 'succeeded':
        raise ValueError('Released product extraction failed')
    if operation == 'browse' and (not events['cancel_requested'].get('pending') or
                                  not events['cancel_settled'].get('cancelled')):
        raise ValueError('Released product cancellation did not settle')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--qt-build', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--archive', type=Path, help='Existing exact public package, for offline verification')
    args = parser.parse_args()
    platform = 'windows' if os.name == 'nt' else 'linux' if sys.platform.startswith('linux') else None
    output = args.output.resolve()
    if platform is None or output.exists() or not within(output, ROOT/'build'):
        parser.error('Use a new build/ directory on Windows or Linux')
    item = PACKAGES[platform]
    qt_build = args.qt_build.resolve()
    source_record = json.loads((qt_build/'qt-source-build.json').read_text(encoding='utf-8'))
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    core_source = qt_build/'qt-install'/item['source_core']
    if source_record['revision'] != revision or source_record['source_dirty'] or \
            source_record['status'] != 'source-build-passed-not-sdk-equivalence' or \
            not core_source.is_file() or digest(core_source) != source_record['qtcore_sha256']:
        raise ValueError('Source-built QtCore identity differs')
    output.mkdir(parents=True)
    if args.archive:
        archive = args.archive.resolve()
        if not archive.is_file() or digest(archive) != item['sha256']:
            raise ValueError('Public alpha.8 package checksum mismatch')
    else:
        archive = output/item['name']
        acquire(archive, item)
    stage = output/'portable'; stage.mkdir()
    unpack(archive, stage, platform)
    app, core, plugin = stage/item['app'], stage/item['core'], stage/item['platform_plugin']
    if any(not within(path, stage) or not path.is_file() or path.is_symlink() for path in [app, core, plugin]):
        raise ValueError('Expected released application, QtCore or platform plugin missing')
    app_sha256, old_core_sha256 = digest(app), digest(core)
    env = os.environ.copy()
    for key in ['QT_PLUGIN_PATH', 'QML_IMPORT_PATH', 'QML2_IMPORT_PATH', 'QT_QPA_PLATFORM_PLUGIN_PATH',
                'LD_LIBRARY_PATH', 'DYLD_LIBRARY_PATH', 'DYLD_FRAMEWORK_PATH', 'XDZIP_PROBE_OPERATION',
                'XDZIP_PROBE_EXTRACT_TARGET']:
        env.pop(key, None)
    # The shipped Windows package contains qwindows, not qoffscreen.
    env.update(QT_QPA_PLATFORM='windows' if platform == 'windows' else 'offscreen', QT_QUICK_BACKEND='software',
               XDZIP_SETTINGS_PATH=str(output/'isolated/settings.ini'), XDZIP_EXPECT_QT_SOURCE_PROOF=PROOF,
               XDZIP_PROBE_IDLE_MS='100')
    version = json.loads(subprocess.check_output([str(app), '--version-json'], env=env, text=True))
    original_revision = version.get('revision', '')
    if version.get('version') != '0.1.0-alpha.8' or version.get('build') != 21 or \
            len(original_revision) < 12 or not REVISION.startswith(original_revision) or version.get('dirty') is not False:
        raise ValueError('Original alpha.8 package identity differs')
    fixture = output/'metadata.zip'
    subprocess.run([sys.executable, str(ROOT/'scripts/generate-m2-fixture.py'), '--output', str(fixture)], check=True)
    log = output/'replacement.log'
    negative = run_probe(app, fixture, env, log, expected_code=10)
    if 'qt_source_proof' not in negative or negative['qt_source_proof'].get('value') == PROOF:
        raise ValueError('Unmodified QtCore unexpectedly passed source marker')
    shutil.copyfile(core_source, core)
    if digest(core) != source_record['qtcore_sha256'] or digest(core) == old_core_sha256 or digest(app) != app_sha256:
        raise ValueError('QtCore replacement or application identity differs')
    browse = run_probe(app, fixture, env, log)
    validate_success(browse, 100000, 'browse')
    extraction_fixture = output/'payload.zip'
    subprocess.run([sys.executable, str(ROOT/'scripts/generate-m3-fixture.py'), '--output', str(extraction_fixture)], check=True)
    target = output/'extracted'; target.mkdir(mode=0o700)
    env.update(XDZIP_PROBE_OPERATION='extract', XDZIP_PROBE_EXTRACT_TARGET=str(target))
    extraction = run_probe(app, extraction_fixture, env, log)
    validate_success(extraction, 64, 'extract')
    checked_files = verify_output(extraction_fixture, target)
    report = {'status': 'passed', 'platform': platform, 'release_revision': REVISION,
              'release_archive_sha256': item['sha256'], 'source_revision': revision,
              'source_build_sha256': digest(qt_build/'qt-source-build.json'), 'app_sha256': app_sha256,
              'original_qtcore_sha256': old_core_sha256, 'modified_qtcore_sha256': digest(core),
              'unmodified_marker_rejected': True, 'modified_marker_loaded': True,
              'browse_records': 100000, 'extracted_files_verified': checked_files,
              'limits': ['Isolated released portable package; no original installed files or release assets changed.',
                         'One source-built compatible QtCore, not arbitrary ABI changes or full SDK build equivalence.',
                         'Automated Windows native-platform or Linux offscreen interaction, not manual desktop or legal approval.']}
    (output/'alpha8-qt-replacement.json').write_text(json.dumps(report, indent=2)+'\n', encoding='utf-8')
    print('Released alpha.8 loaded modified QtCore; browsing, cancellation and extraction passed')


if __name__ == '__main__':
    main()
