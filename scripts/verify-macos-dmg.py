#!/usr/bin/env python3
"""Mount read-only, relocate and run a local developer DMG without SDK search paths."""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import plistlib
import shutil
import subprocess
from macos_launch_services import temporary_bundle_path, transient_registrations
from deployment_audit import audit, digest, within

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('macdmg', ROOT/'scripts/build-macos-dmg.py')
macdmg = importlib.util.module_from_spec(spec)
spec.loader.exec_module(macdmg)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--dmg', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    if not within(output, ROOT/'build') or output.exists():
        parser.error('Use a new verification directory under build/')
    app = output/'中文 space/xdzip.app'
    with temporary_bundle_path(app), transient_registrations(output):
        output.mkdir(parents=True)
        mounted = plistlib.loads(subprocess.check_output(
            ['hdiutil', 'attach', '-readonly', '-nobrowse', '-noautoopen', '-plist', str(args.dmg.resolve())]))
        mount = next(Path(e['mount-point']) for e in mounted['system-entities'] if 'mount-point' in e)
        try:
            if not (mount/'Applications').is_symlink() or os.readlink(mount/'Applications') != '/Applications':
                raise ValueError('Drag-to-Applications entry missing')
            shutil.copytree(mount/'xdzip.app', app, symlinks=True)
        finally:
            subprocess.run(['hdiutil', 'detach', str(mount)], check=True, capture_output=True)
        binary = macdmg.validate_bundle(app)
        report = audit(app, binary, None, 'macos', 'deployed')
        if report['status'] != 'passed':
            raise ValueError('Relocated DMG still depends on external SDK libraries')
        env = dict(os.environ, PATH='/usr/bin:/bin:/usr/sbin:/sbin', QT_QPA_PLATFORM='cocoa',
                   QSG_RHI_BACKEND='metal', XDZIP_SETTINGS_PATH=str(output/'isolated/settings.ini'))
        for key in ['QT_PLUGIN_PATH', 'QML_IMPORT_PATH', 'QML2_IMPORT_PATH', 'DYLD_LIBRARY_PATH',
                    'DYLD_FRAMEWORK_PATH', 'QT_QUICK_BACKEND', 'QT_QPA_PLATFORM_PLUGIN_PATH']:
            env.pop(key, None)
        def smoke(label):
            result = subprocess.run([str(binary), '--smoke-test'], env=env, capture_output=True, text=True, timeout=30)
            # Framework diagnostics can contain SDK/workspace paths; keep only local
            # artifact-relative paths in evidence, never user profile names.
            log = result.stdout+result.stderr
            (output/(label+'.log')).write_text(log.replace(str(ROOT), '<workspace>'), encoding='utf-8')
            if result.returncode:
                raise ValueError(label+' failed; see local verification log')
        smoke('relocated-cocoa-metal')
        core = app/'Contents/Frameworks/QtCore.framework/Versions/A/QtCore'
        before = digest(core)
        # Change a library load identity in an isolated copy, then ad-hoc re-sign.
        # This verifies replaceability of a modified compatible image, not a rebuild
        # of Qt from source or arbitrary ABI compatibility.
        macdmg.checked(['install_name_tool', '-id', '@rpath/QtCore-replacement-proof', core])
        macdmg.sign_local(app)
        modified = digest(core)
        if modified == before:
            raise ValueError('Replacement proof did not modify the framework')
        smoke('modified-qtcore-cocoa-metal')
    result = {'status': 'passed', 'dmg_sha256': digest(args.dmg), 'platform': 'macos-arm64',
              'cases': ['read-only-mount', 'applications-link', 'unicode-space-relocation',
                        'sdk-independent-native-imports', 'cocoa-metal-smoke', 'modified-qtcore-ad-hoc-smoke'],
              'replacement': {'original_sha256': before, 'modified_sha256': modified, 'kind': 'Mach-O load-identity modification'},
              'developer_id_signed': False, 'notarized': False,
              'limits': ['Current host has SDK installed; clean search paths and recursive import audit, not a separate clean machine.',
                         'No Finder drag/drop, default selection, Gatekeeper trust or source-rebuilt Qt validation claimed.']}
    (output/'verification.json').write_text(json.dumps(result, indent=2)+'\n')
    print('DMG read-only mount, relocation, native startup and modified QtCore replacement passed')


if __name__ == '__main__':
    main()
