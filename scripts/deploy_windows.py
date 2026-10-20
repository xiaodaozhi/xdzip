#!/usr/bin/env python3
"""Deploy the Basic-only Windows runtime, retaining native widgets and graphics fallbacks."""
import argparse
import json
from pathlib import Path
import shutil
import subprocess
from deployment_audit import ROOT, digest, within

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
                    '--skip-plugin-types', 'qmltooling', '--qmldir', str(args.qml_dir),
                    '--qmlimport', str(args.qml_import), str(root/'xdzip.exe')], check=True)
    removed = prune(root)
    copy_runtime(args.runtime, root)
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
