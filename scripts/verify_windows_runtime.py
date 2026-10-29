#!/usr/bin/env python3
"""Run native Qt regressions in a separate deployed tree with no SDK search paths."""
import argparse
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
from deployment_audit import ROOT, digest, within
from deploy_windows import require_no_software_opengl


def renderer_environment(environment, root, output, software):
    env = environment.copy()
    for key in list(env):
        if key.upper().startswith(('QT_', 'QSG_', 'QML_', 'XDZIP_')):
            env.pop(key, None)
    env.update(QT_PLUGIN_PATH=str(root), QT_QPA_PLATFORM_PLUGIN_PATH=str(root/'platforms'),
               QML_IMPORT_PATH=str(root/'qml'), QML2_IMPORT_PATH=str(root/'qml'),
               QT_QPA_PLATFORM='windows', QSG_INFO='1', QT_FORCE_STDERR_LOGGING='1',
               QT_LOGGING_RULES='qt.rhi.general.debug=true;qt.scenegraph.general.debug=true',
               QT_MESSAGE_PATTERN='%{category}: %{message}',
               XDZIP_SETTINGS_PATH=str(output/('warp-settings.ini' if software else 'default-settings.ini')))
    if software:
        env['QSG_RHI_PREFER_SOFTWARE_RENDERER'] = '1'
    return env


def renderer_observation(output, software):
    # Qt 6.8 logs the DXGI adapter immediately before marking it as selected.
    # The SOFTWARE bit is 0x2; Microsoft vendor 0x1414/device 0x8c identifies
    # the Basic Render Driver. Keep paths and adapter descriptions out of the receipt.
    if 'Creating QRhi with backend D3D11' not in output or 'Created QRhi' not in output:
        raise ValueError('D3D11 scene graph initialization was not observed')
    pattern = re.compile(r'\bAdapter \d+: .*?\(vendor 0x([0-9a-f]+) device 0x([0-9a-f]+) flags 0x([0-9a-f]+)\)', re.I)
    candidate = None
    selected = []
    for line in output.splitlines():
        match = pattern.search(line)
        if match:
            candidate = tuple(int(value, 16) for value in match.groups())
        elif 'using this adapter' in line and candidate is not None:
            selected.append(candidate)
            candidate = None
    if not selected:
        raise ValueError('D3D11 selected adapter was not observed')
    vendor, device, flags = selected[-1]
    if software and (not flags & 2 or vendor != 0x1414 or device != 0x8c):
        raise ValueError('Microsoft WARP software adapter was not observed')
    return {'backend': 'd3d11', 'software_device': bool(flags & 2),
            'vendor_id': f'{vendor:04x}', 'device_id': f'{device:04x}',
            'scenegraph_initialized': True}


def verify_renderer(root, output, environment, software):
    env = renderer_environment(environment, root, output, software)
    result = subprocess.run([str(root/'xdzip.exe'), '--smoke-test'], cwd=output, env=env,
                            capture_output=True, text=True, errors='replace', timeout=60)
    if result.returncode != 0:
        raise ValueError('Windows graphics startup failed')
    return renderer_observation(result.stdout+'\n'+result.stderr, software)


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
    require_no_software_opengl(args.root)
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
        QT_QPA_PLATFORM='offscreen', QT_QUICK_BACKEND='software',
        XDZIP_SETTINGS_PATH=str(output/'settings.ini'))
    selected = [] if args.full else ['qtRuntimeTranslations', 'settingsWindowPersistsAndRollsBack']
    subprocess.run([str(root/'xdzip-browser-tests.exe'), *selected, '-o',
                    str(output/'browser-tests.log')+',txt'], cwd=output, env=env, check=True)
    env['QT_QPA_PLATFORM'] = 'windows'
    subprocess.run([str(root/'xdzip-browser-tests.exe'), 'windowsCaptionControlsAndWindowState',
                    '-o', str(output/'native-window-tests.log')+',txt'], cwd=output, env=env, check=True)
    for language in ('zh-CN', 'en-US'):
        env['XDZIP_LANGUAGE'] = language
        subprocess.run([str(root/'xdzip.exe'), '--smoke-test'], cwd=output, env=env, check=True)
    rendering = {'default': verify_renderer(root, output, env, False),
                 'warp': verify_renderer(root, output, env, True)}
    (output/'result.json').write_text(json.dumps({'status': 'passed', 'sdk_search_paths': False,
        'full_browser_suite': args.full, 'product_sha256': digest(args.root/'xdzip.exe'),
        'rendering': rendering, 'software_opengl_present': False,
        'limits': ['Host Windows still supplies OS APIs; this is not a clean-machine installation test.',
                   'WARP was requested explicitly; automatic fallback and visual acceptance were not tested.']}, indent=2)+'\n')
    shutil.rmtree(root)  # Fresh owned harness only; preserve logs outside it.
    print('Deployed translation, settings, native caption, bilingual startup and D3D11/WARP checks passed')


if __name__ == '__main__':
    main()
