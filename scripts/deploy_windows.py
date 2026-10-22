#!/usr/bin/env python3
"""Deploy the Basic-only Windows runtime, retaining native widgets and graphics fallbacks."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
from deployment_audit import ROOT, digest, within
from native_runtime_materials import configuration

STYLES = ('FluentWinUI3', 'Fusion', 'Imagine', 'Material', 'Universal', 'Windows')
MODULES = tuple('qml/QtQuick/Controls/'+name for name in STYLES) + (
    'qml/QtQuick/NativeStyle', 'qml/QtQuick/Effects', 'qmltooling')
LIBRARIES = tuple('Qt6QuickControls2'+name+'.dll' for name in (
    'FluentWinUI3StyleImpl', 'Fusion', 'FusionStyleImpl', 'Imagine', 'ImagineStyleImpl',
    'Material', 'MaterialStyleImpl', 'Universal', 'UniversalStyleImpl', 'WindowsStyleImpl')) + (
    'Qt6QuickEffects.dll', 'vc_redist.x64.exe')
FALLBACKS = ('opengl32sw.dll', 'dxcompiler.dll', 'dxil.dll', 'd3dcompiler_47.dll')


def prune(root):
    # This deletes only known generated deployment files. Check the complete tree
    # first, rejecting links/junction escapes before any destructive operation.
    paths = list(root.rglob('*'))
    if any(p.is_symlink() or not within(p, root) or
           (hasattr(p, 'is_junction') and p.is_junction()) for p in paths):
        raise ValueError('Deployment tree must not contain links or reparse escapes')
    removed = []
    targets = [root/p for p in MODULES+LIBRARIES] + list(root.rglob('*.qmltypes'))
    for path in targets:
        if not path.exists():
            continue
        files = list(path.rglob('*')) if path.is_dir() else [path]
        removed.extend({'path': p.relative_to(root).as_posix(), 'bytes': p.stat().st_size}
                       for p in files if p.is_file())
        if path.is_dir():
            shutil.rmtree(path)
        else:
            path.unlink()
    return removed


def copy_runtime(runtime, root):
    config = json.loads((ROOT/'scripts/windows-runtime.json').read_text())
    materials = dict(config['files'])
    materials['Microsoft-VC-runtime-license.rtf'] = config['license_sha256']
    # Validate the entire input before copying anything; never harvest System32.
    for name, expected in materials.items():
        if not within(runtime/name, runtime) or digest(runtime/name) != expected:
            raise ValueError('App-local runtime material hash mismatch')
    notices = root/'share/notices/Microsoft-VC-runtime'; notices.mkdir(parents=True, exist_ok=True)
    for name in config['files']:
        shutil.copyfile(runtime/name, root/name)
    shutil.copyfile(runtime/'Microsoft-VC-runtime-license.rtf', notices/'license.rtf')
    (notices/'provenance.json').write_text(json.dumps(config, indent=2)+'\n')
    # Inno's text page needs a BOM to display the same Chinese/English notice
    # independently of the installer machine's ANSI code page.
    notice = ROOT/'apps/desktop/packaging/windows/Windows-runtime-NOTICE.txt'
    (notices.parent/notice.name).write_bytes(b'\xef\xbb\xbf'+notice.read_bytes())


def copy_graphics(qt, sdk, sdk_version, root):
    # Do not let windeployqt discover these through PATH/System32. Only the
    # selected Windows SDK's redistributable directory and the fixed Qt SDK
    # are admitted. This records supplier inputs, not licensee eligibility.
    if not sdk_version or '/' in sdk_version or '\\' in sdk_version:
        raise ValueError('Explicit Windows SDK version required for graphics provenance')
    redist = sdk/'Redist/D3D/x64'
    license_file = sdk/'Licenses'/sdk_version/'sdk_license.rtf'
    if not license_file.is_file():
        license_file = sdk/'license.rtf'
    third_party = license_file.with_name('sdk_third_party_notices.rtf')
    inputs = {name: redist/name for name in FALLBACKS if name != 'opengl32sw.dll'}
    inputs['opengl32sw.dll'] = qt/'bin/opengl32sw.dll'
    for path in [*inputs.values(), license_file]:
        base = qt if path == inputs['opengl32sw.dll'] else sdk
        if not within(path, base) or not path.is_file() or path.is_symlink():
            raise ValueError('Missing bounded SDK graphics redistributable/license')
    if digest(inputs['opengl32sw.dll']) != configuration()['mesa_binary']['sha256']:
        raise ValueError('Unreviewed Mesa software renderer; update source and notices together')
    notices = root/'share/notices/Windows-SDK-graphics'
    notices.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(license_file, notices/'sdk_license.rtf')
    if third_party.is_file() and within(third_party, sdk):
        shutil.copyfile(third_party, notices/third_party.name)
    files = []
    for name, path in inputs.items():
        shutil.copyfile(path, root/name)
        base = qt if name == 'opengl32sw.dll' else sdk
        files.append({'file': name, 'sha256': digest(path),
                      'supplier': 'Qt SDK Mesa build' if name == 'opengl32sw.dll' else 'Microsoft Windows SDK REDIST',
                      'source_relative': path.relative_to(base).as_posix()})
    result = {'schema_version': 1, 'windows_sdk_version': sdk_version, 'files': files,
              'license': {'file': 'sdk_license.rtf', 'sha256': digest(license_file)},
              'redistribution_list': 'https://go.microsoft.com/fwlink/?LinkId=524842',
              'limits': ['SDK file provenance does not certify licensee eligibility or Mesa/LLVM notice completeness.']}
    (notices/'provenance.json').write_text(json.dumps(result, indent=2)+'\n')
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, required=True)
    parser.add_argument('--qt-prefix', type=Path, required=True)
    parser.add_argument('--qml-dir', type=Path, required=True)
    parser.add_argument('--qml-import', type=Path, required=True)
    parser.add_argument('--runtime', type=Path, required=True)
    args = parser.parse_args()
    root = args.root.resolve()
    if not within(root, ROOT/'build') or root == ROOT/'build' or not (root/'xdzip.exe').is_file():
        parser.error('Use an application staging directory under build/')
    subprocess.run([str(args.qt_prefix/'bin/windeployqt.exe'), '--release', '--no-compiler-runtime',
                    '--no-system-d3d-compiler', '--no-system-dxc-compiler',
                    '--skip-plugin-types', 'qmltooling', '--qmldir', str(args.qml_dir),
                    '--qmlimport', str(args.qml_import), str(root/'xdzip.exe')], check=True)
    removed = prune(root)
    copy_runtime(args.runtime, root)
    sdk = Path(os.environ.get('WindowsSdkDir', ''))
    sdk_version = os.environ.get('WindowsSDKVersion', '').strip('\\/')
    copy_graphics(args.qt_prefix.resolve(), sdk.resolve(), sdk_version, root)
    required = FALLBACKS + ('styles/qmodernwindowsstyle.dll', 'platforms/qwindows.dll',
                           'qml/QtQuick/Controls/Basic/qmldir', 'translations/qt_zh_CN.qm')
    if any(not (root/path).is_file() for path in required):
        raise ValueError('Required Basic/native/translation/graphics runtime missing')
    (root/'share/deployment.json').write_text(json.dumps({
        'schema_version': 1, 'quick_style': 'Basic', 'removed': removed,
        'graphics_fallbacks': list(FALLBACKS), 'vc_runtime': 'app-local',
        'vc_runtime_config_sha256': digest(ROOT/'scripts/windows-runtime.json')}, indent=2)+'\n')
    print(f'Windows runtime deployed, {len(removed)} redundant files removed; graphics retained')


if __name__ == '__main__':
    main()
