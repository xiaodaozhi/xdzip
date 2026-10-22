#!/usr/bin/env python3
"""Native Apple Silicon DMG and optional release app ZIP; ad-hoc integrity only."""
import argparse
import configparser
import json
import os
from pathlib import Path
import plistlib
import shutil
import subprocess
import sys
from deployment_audit import audit, digest, mac_rpaths, native, within
from sbom import artifact_bom
from macos_launch_services import temporary_bundle_path, transient_registrations
from macos_bundle import optimize_bundle, validate_optimized_bundle
from macos_attribution import deployment_record
from sparkle_bundle import (attribute as attribute_sparkle, embed as embed_sparkle,
                            sdk_from_build, signing_metadata, sign_helpers)

ROOT = Path(__file__).resolve().parents[1]


def checked(command, **kwargs):
    result = subprocess.run([str(x) for x in command], capture_output=True, text=True, **kwargs)
    if result.returncode:
        raise ValueError(Path(str(command[0])).name + ' failed; inspect local build log')
    return result.stdout


def validate_bundle(app, *, require_handler=False):
    info = plistlib.loads((app/'Contents/Info.plist').read_bytes())
    if info.get('CFBundleIdentifier') != 'org.xdzip.desktop':
        raise ValueError('Unexpected application identity')
    if require_handler:
        documents = info.get('CFBundleDocumentTypes')
        if (not isinstance(documents, list) or not documents or
                any(not isinstance(item, dict) or item.get('LSHandlerRank') != 'Alternate'
                    or item.get('CFBundleTypeRole') != 'Viewer' for item in documents)):
            raise ValueError('DMG requires an Alternate document handler; rebuild with XDZIP_MACOS_HANDLER_RANK=Alternate')
    binary = app/'Contents/MacOS'/info['CFBundleExecutable']
    if 'arm64' not in checked(['lipo', '-archs', binary]).split():
        raise ValueError('Native arm64 executable required')
    for path in app.rglob('*'):
        if path.is_symlink() and not within(path, app):
            raise ValueError('Bundle symlink escapes application')
    return binary


def sign_local(app, sparkle_metadata=None, work=None):
    # ARM64 modified Mach-O images require a valid code directory. Ad-hoc signing
    # is local integrity only, never a Developer ID or Gatekeeper trust claim.
    sparkle = app/'Contents/Frameworks/Sparkle.framework'
    images = {p.resolve() for p in app.rglob('*') if p.is_file() and native(p, 'macos')
              and not p.resolve().is_relative_to(sparkle.resolve())}
    for path in sorted(images):
        checked(['codesign', '--force', '--sign', '-', '--timestamp=none', path])
    for framework in sorted((app/'Contents/Frameworks').glob('*.framework')):
        if framework == sparkle:
            continue
        checked(['codesign', '--force', '--sign', '-', '--timestamp=none', framework])
    if sparkle.exists():
        if sparkle_metadata is None:
            # Qt replacement probes do not change Sparkle. Preserve its nested
            # signatures rather than applying a generic signing recipe to them.
            checked(['codesign', '--verify', '--deep', '--strict', sparkle])
        else:
            if work is None:
                raise ValueError('Sparkle helper signing needs a private work directory')
            sign_helpers(sparkle, sparkle_metadata, checked, work)
    checked(['codesign', '--force', '--sign', '-', '--timestamp=none', app])
    checked(['codesign', '--verify', '--deep', '--strict', app])


def configure_translation_path(app):
    # Qt's native edit roles and widget dialogs load the system-locale qtbase
    # catalog through QLibraryInfo, independently of the application's catalog.
    path = app/'Contents/Resources/qt.conf'
    paths = configparser.ConfigParser(interpolation=None)
    paths.optionxform = str
    paths.read(path, encoding='utf-8')
    if not paths.has_section('Paths'):
        paths.add_section('Paths')
    paths.set('Paths', 'Translations', 'Resources/translations')
    with path.open('w', encoding='utf-8') as stream:
        paths.write(stream)


def export_release_app(app, output):
    """Archive the signed app while its disposable staging copy still exists."""
    archive = output.with_suffix('.app.zip')
    notices = output.parent/'notices'
    if archive.exists() or notices.exists():
        raise ValueError('Use a fresh release output directory')
    checked(['ditto', '-c', '-k', '--sequesterRsrc', '--keepParent', app, archive])
    shutil.copytree(app/'Contents/Resources/notices', notices)


def build(source, qt, output, release_distribution=False):
    if sys.platform != 'darwin':
        raise ValueError('DMG requires native macOS')
    if not within(output, ROOT/'build') or output.exists() or output.suffix != '.dmg':
        raise ValueError('Use a new .dmg under build/')
    binary = validate_bundle(source, require_handler=True)
    identity = checked([binary, '--version']).strip()
    if 'dirty' in identity or 'unknown' in identity:
        raise ValueError('DMG requires a clean verifiable product build')
    revision = checked(['git', '-C', ROOT, 'rev-parse', '--short=12', 'HEAD']).strip()
    if checked(['git', '-C', ROOT, 'status', '--porcelain']).strip() or not identity.endswith(', '+revision+')'):
        raise ValueError('DMG source and notice materials must match the clean executable revision')
    stage = output.parent/(output.stem+'-stage')
    if stage.exists():
        raise ValueError('Refusing an existing staging directory')
    stage.mkdir(parents=True)
    sparkle_sdk = sdk_from_build(source)
    app = stage/'xdzip.app'
    with temporary_bundle_path(app), transient_registrations(stage):
        shutil.copytree(source, app, symlinks=True)
        checked([qt/'bin/macdeployqt', app, '-always-overwrite', '-no-strip',
                 '-qmldir='+str(ROOT/'apps/desktop/ui'),
                 '-qmlimport='+str(source.parents[2]/'qml'), '-verbose=1'])
        sparkle = embed_sparkle(sparkle_sdk, app, installable=release_distribution)
        sparkle_metadata = signing_metadata(sparkle, checked)
        # macdeployqt copies every SQL driver when QML's optional LocalStorage pulls
        # QtSql in. Only SQLite is self-contained here; server/ODBC drivers are not
        # product capabilities and require SDK-external client libraries.
        omitted_sql = []
        for driver in (app/'Contents/PlugIns/sqldrivers').glob('*.dylib'):
            if driver.name != 'libqsqlite.dylib':
                omitted_sql.append(driver.name)
                driver.unlink()
        notices = app/'Contents/Resources/notices'
        notices.mkdir(parents=True, exist_ok=True)
        checked([sys.executable, ROOT/'scripts/collect-notices.py', notices,
                 '--sparkle-sdk', sparkle_sdk])
        for name in ['LICENSE', 'NOTICE']:
            if (ROOT/name).exists():
                shutil.copy2(ROOT/name, notices/name)
        translations = app/'Contents/Resources/translations'
        translations.mkdir(exist_ok=True)
        for path in (qt/'translations').glob('qtbase_*.qm'):
            shutil.copy2(path, translations/path.name)
        checked([sys.executable, ROOT/'scripts/prepare-runtime-notices.py', '--qt-prefix', qt,
                 '--output', notices/'Qt-runtime'])
        configure_translation_path(app)
        # Deployment copies universal libraries and optional Controls styles from
        # the SDK. Trim only this disposable copy, before hashes and signatures.
        optimization = optimize_bundle(app)
        # Absolute development RPATHs must not survive into a portable application.
        for path in {p.resolve() for p in app.rglob('*') if p.is_file() and native(p, 'macos')}:
            rpaths = mac_rpaths(checked(['otool', '-l', path]))
            for value in rpaths:
                if value.startswith('/') and not value.startswith(('/usr/lib/', '/System/Library/')):
                    checked(['install_name_tool', '-delete_rpath', value, path])
        sign_local(app, sparkle_metadata, stage)
        for path in stage.glob('sparkle-helper-*.entitlements'):
            path.unlink()
        final_bundle = validate_optimized_bundle(app)
        binary = validate_bundle(app)
        report = audit(app, binary, None, 'macos', 'deployed')
        if report['status'] != 'passed':
            raise ValueError('DMG application still has external dependencies')
        if checked([binary, '--version']).strip() != identity:
            raise ValueError('Deployment changed the product identity')
        inventory = json.loads((notices/'build-inventory.json').read_text(encoding='utf-8'))
        report['identity'] = {'version': inventory['display_version'], 'build': inventory['version_source']['build'],
                              'revision': revision, 'dirty': False}
        report['bundle_optimization'] = optimization
        report['bundle'] = final_bundle
        # Bind the final signed/thinned images, while this exact application and
        # its SDK inputs still exist. SDK-only CI records cannot fill this role.
        audit_path = output.parent/'deployment-audit.json'
        audit_path.write_text(json.dumps(report, indent=2)+'\n')
        deployment_path = output.parent/'macos-deployment.json'
        deployment_path.write_text(json.dumps(deployment_record(app, qt, report), indent=2)+'\n')
        attribution_path = output.parent/'runtime-attribution.json'
        checked([sys.executable, ROOT/'scripts/runtime_attribution.py', '--audit', audit_path,
                 '--root', app, '--qt-prefix', qt, '--deployment', deployment_path,
                 '--output', attribution_path])
        attribution = json.loads(attribution_path.read_text())
        attribute_sparkle(attribution, app, sparkle_sdk, report)
        attribution_path.write_text(json.dumps(attribution, indent=2)+'\n')
        source_bom = json.loads((notices/'source-build.cdx.json').read_text(encoding='utf-8'))
        file_bom = artifact_bom(report, source_bom, attribution)
        output.with_suffix('.cdx.json').write_text(json.dumps(file_bom, indent=2)+'\n')
        (stage/'Applications').symlink_to('/Applications')
        (stage/'安装说明.txt').write_text(
            '将 xdzip.app 拖入 Applications 后打开。替换旧版本前请退出 xdzip。\n'
            '本镜像为开发构建，仅 ad-hoc 本地完整性签名，未经 Developer ID 签名或 Apple 公证。\n'
            '如系统阻止运行，请按系统安全设置确认来源，不要关闭系统安全保护。\n'
            '设置与最近记录保存在用户配置目录，移除应用不会自动删除。\n', encoding='utf-8')
        checked(['hdiutil', 'create', '-volname', 'xdzip', '-srcfolder', stage,
                 '-format', 'UDZO', '-fs', 'HFS+', output])
        checked(['hdiutil', 'verify', output])
        if release_distribution:
            export_release_app(app, output)
        report.update(identity_text=identity, file=output.name, sha256=digest(output),
                      dmg_bytes=output.stat().st_size,
                      signing='ad-hoc', developer_id_signed=False, notarized=False,
                      app_tree='xdzip.app', local_only=not release_distribution, unused_sql_drivers_omitted=sorted(omitted_sql))
    output.with_suffix('.json').write_text(json.dumps(report, indent=2)+'\n')
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--app', type=Path, required=True)
    parser.add_argument('--qt-prefix', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--release-distribution', action='store_true')
    args = parser.parse_args()
    result = build(args.app.resolve(), args.qt_prefix.resolve(), args.output.resolve(), args.release_distribution)
    print('Local developer DMG verified, SDK-independent native imports, ad-hoc only: '+result['file'])


if __name__ == '__main__':
    main()
