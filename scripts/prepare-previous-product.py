#!/usr/bin/env python3
"""CI-only: rebuild a pinned full historical product for real upgrade/rollback tests."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
from deployment_audit import digest, within

ROOT = Path(__file__).resolve().parents[1]
BASELINE = '4cf0f8ca44bac3c0deaadbdaf5594504c5a6afa6'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--qt-prefix', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--notices', type=Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    if os.environ.get('CI') != 'true' or sys.platform not in ('win32', 'linux'):
        parser.error('Historical installer validation runs only on disposable Windows/Linux CI hosts')
    if not within(output, ROOT/'build') or output.exists():
        parser.error('Use a new build/ output directory')
    output.mkdir(parents=True)
    log = (output/'previous-product.log').open('w', encoding='utf-8')
    def run(command, env=None):
        subprocess.run([str(x) for x in command], cwd=ROOT, env=env, stdout=log, stderr=log, check=True)
    source = output/'source'
    build = output/'compiled'
    qt = args.qt_prefix.resolve()
    # A detached worktree retains the original verified CLI revision. No branch
    # or repository source is rewritten; build outputs stay outside that tree.
    run(['git', 'fetch', '--no-tags', '--depth=1', 'origin', BASELINE])
    run(['git', 'worktree', 'add', '--detach', source, BASELINE])
    run(['cmake', '-S', source, '-B', build, '-G', 'Ninja', '-DCMAKE_BUILD_TYPE=Release',
         '-DXDZIP_BUILD_DESKTOP_PREVIEW=ON', '-DXDZIP_BUILD_QT_PROBE=OFF', '-DBUILD_TESTING=OFF',
         '-DCMAKE_PREFIX_PATH='+str(qt)])
    run(['cmake', '--build', build, '--target', 'xdzip-browser', '--parallel', '3'])
    runtime = output/'runtime'
    windows = sys.platform == 'win32'
    if windows:
        runtime.mkdir()
        shutil.copy2(build/'apps/desktop/xdzip-browser.exe', runtime/'xdzip.exe')
        shutil.copy2(source/'design/assets/brand/xdzip-zip.ico', runtime/'xdzip-zip.ico')
        run([qt/'bin/windeployqt.exe', '--release', '--qmldir', source/'apps/desktop/ui',
             '--qmlimport', build/'qml', runtime/'xdzip.exe'])
        shutil.copytree(args.notices, runtime/'share/notices')
        executable = runtime/'xdzip.exe'
    else:
        # Use the historical QML import set, not the current UI's import graph.
        run([sys.executable, source/'scripts/deploy_linux.py', '--app', build/'apps/desktop/xdzip-browser',
             '--qt-prefix', qt, '--notices', args.notices.resolve(), '--output', source/'build/runtime'])
        runtime = source/'build/runtime'
        executable = runtime/'bin/xdzip'
    audit = output/'deployment-audit.json'
    deployed_environment = dict(os.environ)
    for key in ['LD_LIBRARY_PATH', 'QT_PLUGIN_PATH', 'QML_IMPORT_PATH', 'QML2_IMPORT_PATH']:
        deployed_environment.pop(key, None)
    run([sys.executable, ROOT/'scripts/deployment_audit.py', '--root', runtime,
         '--app', executable, '--qt-prefix', qt, '--mode', 'deployed', '--output', audit], env=deployed_environment)
    record = json.loads(audit.read_text(encoding='utf-8'))
    if record['identity']['revision'] != BASELINE[:12] or record['identity']['dirty']:
        raise ValueError('Historical full product identity is not the pinned clean revision')
    report = output/'installer.json'
    if windows:
        package = output/'previous.exe'
        run([sys.executable, ROOT/'scripts/build-windows-installer.py', '--root', runtime, '--audit', audit,
             '--iscc', ROOT/'build/installer-toolchain/compiler/ISCC.exe',
             '--chinese', ROOT/'build/installer-toolchain/ChineseSimplified.isl', '--output', package, '--report', report])
    else:
        package = output/'previous.deb'
        run([sys.executable, ROOT/'scripts/build-linux-installer.py', '--root', runtime, '--audit', audit,
             '--output', package, '--report', report])
    provenance = {'revision': BASELINE, 'executable_sha256': digest(executable), 'installer_sha256': digest(package),
                  'identity': record['identity'], 'scope': 'Full historical source rebuilt with current pinned SDK, QA only; not an original signed release artifact.'}
    (output/'previous-product.json').write_text(json.dumps(provenance, indent=2)+'\n')
    log.close()
    print('Pinned full historical product rebuilt and packaged for CI upgrade/rollback')


if __name__ == '__main__':
    main()
