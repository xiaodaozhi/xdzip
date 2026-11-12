#!/usr/bin/env python3
"""Deploy the Basic-only Windows runtime without app-local graphics compiler DLLs."""
import argparse
import json
from pathlib import Path
import shutil
import subprocess
from deployment_audit import ROOT, digest, run, windows_imports, within
from installer_payload import historical_qa_identity, historical_qa_revision

STYLES = ('FluentWinUI3', 'Fusion', 'Imagine', 'Material', 'Universal', 'Windows')
MODULES = tuple('qml/QtQuick/Controls/'+name for name in STYLES) + (
    'qml/QtQuick/NativeStyle', 'qml/QtQuick/Effects', 'qmltooling')
LIBRARIES = tuple('Qt6QuickControls2'+name+'.dll' for name in (
    'FluentWinUI3StyleImpl', 'Fusion', 'FusionStyleImpl', 'Imagine', 'ImagineStyleImpl',
    'Material', 'MaterialStyleImpl', 'Universal', 'UniversalStyleImpl', 'WindowsStyleImpl')) + (
    'Qt6QuickEffects.dll', 'vc_redist.x64.exe', 'opengl32sw.dll')
EXCLUDED_GRAPHICS = ('dxcompiler.dll', 'dxil.dll', 'd3dcompiler_47.dll')


def standalone_update_helper(path):
    imports=windows_imports(run('dumpbin','/dependents',str(path)))
    system={'kernel32.dll','kernelbase.dll','advapi32.dll','ntdll.dll','bcrypt.dll',
            'bcryptprimitives.dll','userenv.dll','ws2_32.dll','secur32.dll','shell32.dll',
            'user32.dll','ole32.dll'}
    if not imports or any(name.lower() not in system and
                          not name.lower().startswith(('api-ms-win-','ext-ms-win-')) for name in imports):
        raise ValueError('Update helper must use static CRT and Windows system imports only')
    return imports


def update_helper_deployment(root, historical_revision=None):
    helper = root/'xdzip-update-helper.exe'
    if historical_revision is not None:
        historical_qa_revision(historical_revision)
        if helper.exists() or helper.is_symlink():
            raise ValueError('Pinned historical source must not be supplied a newer update helper')
        cli = run(str((root/'xdzip.exe').resolve()), '--version').strip()
        expected = f'xdzip 0.1.0-dev.1 (build 1, {historical_revision[:12]})'
        if cli != expected:
            raise ValueError('Historical helper omission requires the pinned clean product identity')
        record = historical_qa_identity(historical_revision, {
            'version': '0.1.0-dev.1', 'build': 1,
            'revision': historical_revision[:12], 'dirty': False})
        return {'update_helper_imports': None, 'historical_product_qa': record}
    if not helper.is_file() or helper.is_symlink():
        raise ValueError('Windows deployment requires the standalone update helper')
    return {'update_helper_imports': standalone_update_helper(helper)}


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


def require_no_software_opengl(root):
    if any(path.name.lower() == 'opengl32sw.dll' for path in root.rglob('*')):
        raise ValueError('Software OpenGL library must not be present in the deployment')


def require_no_sdk_graphics(root):
    excluded = set(EXCLUDED_GRAPHICS)
    if any(path.name.lower() in excluded for path in root.rglob('*')):
        raise ValueError('SDK graphics compiler libraries must not be present in the deployment')
    if (root/'share/notices/Windows-SDK-graphics').exists():
        raise ValueError('Stale SDK graphics notices must not be present in the deployment')


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


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, required=True)
    parser.add_argument('--qt-prefix', type=Path, required=True)
    parser.add_argument('--qml-dir', type=Path, required=True)
    parser.add_argument('--qml-import', type=Path, required=True)
    parser.add_argument('--runtime', type=Path, required=True)
    parser.add_argument('--qa-historical-revision', help='CI-only fixed historical source without an update helper')
    args = parser.parse_args()
    if args.qa_historical_revision is not None:
        try:
            historical_qa_revision(args.qa_historical_revision)
        except ValueError as error:
            parser.error(str(error))
    root = args.root.resolve()
    if not within(root, ROOT/'build') or root == ROOT/'build' or not (root/'xdzip.exe').is_file():
        parser.error('Use an application staging directory under build/')
    binaries = [str(root/'xdzip.exe')]
    if not args.qa_historical_revision:
        preview = root/'xdzip-image-preview.exe'
        if not preview.is_file() or preview.is_symlink():
            raise ValueError('Expected isolated image preview worker beside application')
        binaries.append(str(preview))
    subprocess.run([str(args.qt_prefix/'bin/windeployqt.exe'), '--release', '--no-compiler-runtime',
                    '--no-opengl-sw',
                    '--no-system-d3d-compiler', '--no-system-dxc-compiler',
                    '--skip-plugin-types', 'qmltooling', '--qmldir', str(args.qml_dir),
                    '--qmlimport', str(args.qml_import), *binaries], check=True)
    removed = prune(root)
    if not args.qa_historical_revision:
        offscreen = args.qt_prefix/'plugins/platforms/qoffscreen.dll'
        if not offscreen.is_file() or offscreen.is_symlink():
            raise ValueError('Image preview requires the SDK offscreen platform plugin')
        shutil.copy2(offscreen, root/'platforms/qoffscreen.dll')
    require_no_sdk_graphics(root)
    copy_runtime(args.runtime, root)
    require_no_software_opengl(root)
    required = ('styles/qmodernwindowsstyle.dll', 'platforms/qwindows.dll',
                           'qml/QtQuick/Controls/Basic/qmldir', 'translations/qt_zh_CN.qm')
    if any(not (root/path).is_file() for path in required):
        raise ValueError('Required Basic/native/translation runtime missing')
    helper_record = update_helper_deployment(root, args.qa_historical_revision)
    (root/'share/deployment.json').write_text(json.dumps({
        'schema_version': 1, 'quick_style': 'Basic', 'removed': removed,
        'graphics_fallbacks': [], 'vc_runtime': 'app-local',
        'vc_runtime_config_sha256': digest(ROOT/'scripts/windows-runtime.json'),
        **helper_record}, indent=2)+'\n')
    print(f'Windows runtime deployed, {len(removed)} redundant files removed; no SDK graphics compilers')


if __name__ == '__main__':
    main()
