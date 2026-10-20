#!/usr/bin/env python3
"""Run native Qt regressions in a separate deployed tree with no SDK search paths."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
from deployment_audit import ROOT, digest, within


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, required=True)
    parser.add_argument('--qt-prefix', type=Path, required=True)
    parser.add_argument('--tests', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--full', action='store_true')
    args = parser.parse_args()
    output = args.output.resolve()
    if output.exists() or not within(output, ROOT/'build') or within(output, args.root):
        parser.error('Use a new build/ test directory outside the product')
    root = output/'runtime'
    shutil.copytree(args.root, root)
    shutil.copyfile(args.tests, root/'xdzip-browser-tests.exe')
    shutil.copyfile(args.qt_prefix/'bin/Qt6Test.dll', root/'Qt6Test.dll')
    shutil.copyfile(args.qt_prefix/'plugins/platforms/qoffscreen.dll', root/'platforms/qoffscreen.dll')
    # The harness is separate from the package; test-only DLLs never enter it.
    env = os.environ.copy()
    system = Path(os.environ['SystemRoot'])
    env['PATH'] = os.pathsep.join(map(str, [system/'System32', system,
        system/'System32/WindowsPowerShell/v1.0']))
    env.update(QT_PLUGIN_PATH=str(root), QT_QPA_PLATFORM_PLUGIN_PATH=str(root/'platforms'),
        QML_IMPORT_PATH=str(root/'qml'), QML2_IMPORT_PATH=str(root/'qml'),
        QT_QPA_PLATFORM='offscreen', QT_QUICK_BACKEND='software')
    selected = [] if args.full else ['qtRuntimeTranslations', 'settingsWindowPersistsAndRollsBack']
    subprocess.run([str(root/'xdzip-browser-tests.exe'), *selected, '-o',
                    str(output/'browser-tests.log')+',txt'], cwd=output, env=env, check=True)
    env['QT_QPA_PLATFORM'] = 'windows'
    subprocess.run([str(root/'xdzip-browser-tests.exe'), 'windowsCaptionControlsAndWindowState',
                    '-o', str(output/'native-window-tests.log')+',txt'], cwd=output, env=env, check=True)
    for language in ('zh-CN', 'en-US'):
        env['XDZIP_LANGUAGE'] = language
        subprocess.run([str(root/'xdzip.exe'), '--smoke-test'], cwd=output, env=env, check=True)
    (output/'result.json').write_text(json.dumps({'status': 'passed', 'sdk_search_paths': False,
        'full_browser_suite': args.full, 'product_sha256': digest(args.root/'xdzip.exe'),
        'limits': ['Host Windows still supplies OS APIs; this is not a clean-machine installation test.']}, indent=2)+'\n')
    shutil.rmtree(root)  # Fresh owned harness only; preserve logs outside it.
    print('Deployed translations, settings restart, native caption and bilingual startup passed without SDK paths')


if __name__ == '__main__':
    main()
