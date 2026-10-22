# scripts/sparkle_bundle.py

"""Pinned Sparkle deployment, signing policy, and evidence for the Mac provider."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import plistlib
import shutil
import tarfile
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
PREFIX = 'Contents/Frameworks/Sparkle.framework/'
INSTALL_ID = 'org.xdzip.desktop.release.v1'


def configuration():
    return json.loads((ROOT/'scripts/sparkle-dependency.json').read_text(encoding='utf-8'))


def checksum(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def sdk_from_build(app):
    cache = app.parents[2]/'CMakeCache.txt'
    for line in cache.read_text(encoding='utf-8').splitlines():
        if line.startswith('XDZIP_SPARKLE_SDK:PATH='):
            sdk = Path(line.split('=', 1)[1]).resolve()
            validate_sdk(sdk)
            return sdk
    raise ValueError('Mac build has no pinned Sparkle SDK receipt')


def validate_sdk(sdk):
    spec = configuration()
    if (checksum(sdk/'LICENSE') != spec['license_sha256'] or
            not (sdk/'Sparkle.framework/Versions/B/Headers/SPUUpdater.h').is_file()):
        raise ValueError('Sparkle SDK license or framework identity differs')
    info = plistlib.loads((sdk/'Sparkle.framework/Versions/B/Resources/Info.plist').read_bytes())
    if info.get('CFBundleShortVersionString') != spec['version']:
        raise ValueError('Sparkle SDK version differs')


def write_notice(sdk, output):
    validate_sdk(sdk)
    spec = configuration()
    target = output/('Sparkle-'+spec['version'])
    target.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(sdk/'LICENSE', target/'LICENSE')
    (target/'provenance.json').write_text(json.dumps(spec, indent=2)+'\n', encoding='utf-8')
    return [target/'LICENSE', target/'provenance.json']


def collect_sources(output, cache):
    """Retain both original archives; the source tarball is not a binary rebuild claim."""
    spec = configuration()
    result = []
    cache.mkdir(parents=True, exist_ok=True)
    for kind in ('source', 'sdk'):
        item = spec[kind]
        archive = cache/item['file']
        if not archive.exists():
            partial = archive.with_suffix(archive.suffix+'.partial')
            try:
                with urllib.request.urlopen(item['url'], timeout=90) as source, partial.open('wb') as target:
                    count = 0
                    while data := source.read(1024*1024):
                        count += len(data)
                        if count > 128*1024*1024:
                            raise ValueError('Sparkle material exceeds size limit')
                        target.write(data)
                if checksum(partial) != item['sha256']:
                    raise ValueError('Sparkle material checksum differs')
                partial.replace(archive)
            finally:
                partial.unlink(missing_ok=True)
        if checksum(archive) != item['sha256']:
            raise ValueError('Cached Sparkle material checksum differs')
        with tarfile.open(archive, 'r:*') as source:
            members = source.getmembers()
            if not members or any(Path(m.name).is_absolute() or '..' in Path(m.name).parts for m in members):
                raise ValueError('Sparkle material has unbounded members')
        shutil.copyfile(archive, output/item['file'])
        result.append(dict(item, module='sparkle', version=spec['version'], kind=kind))
    (output/'sparkle-dependency.json').write_text(json.dumps(spec, indent=2)+'\n', encoding='utf-8')
    return result


def sign_staged_archive(directory, version, build, *, tool=None):
    """Sign only the final immutable asset name, after native package verification."""
    if not os.environ.get('XDZIP_SPARKLE_SIGNING_KEY_HEX'):
        return False  # Validation-only candidates may remain explicitly unsigned.
    from release_updates import run_tool
    archive = directory/f'xdzip-{version}-macos-arm64.app.zip'
    if archive.is_symlink() or not archive.is_file():
        raise ValueError('Final macOS archive is missing')
    output = directory/'macos-update.json'
    key = ROOT/'resources/sparkle-update-key.json'
    run_tool(['sign-archive', '--archive', str(archive), '--public-key', str(key),
              '--version', version, '--build', str(build), '--output', str(output)], tool)
    metadata = json.loads(output.read_bytes())
    run_tool(['verify-archive', '--archive', str(archive), '--public-key', str(key),
              '--signature', metadata['ed_signature']], tool)
    return True


def embed(sdk, app, *, installable):
    """Only mutate the caller's disposable deployment, never the source or SDK."""
    validate_sdk(sdk)
    destination = app/'Contents/Frameworks/Sparkle.framework'
    if destination.is_symlink() or not destination.resolve().is_relative_to(app.resolve()):
        raise ValueError('Sparkle deployment escapes staged application')
    if destination.exists():
        shutil.rmtree(destination)
    shutil.copytree(sdk/'Sparkle.framework', destination, symlinks=True)
    # The product is not sandboxed. These opt-in services are not used; keeping
    # them would add unnecessary signed executables and permission requirements.
    services = destination/'Versions/B/XPCServices'
    if services.is_symlink() or not services.resolve().is_relative_to(destination.resolve()):
        raise ValueError('Sparkle service path escapes framework')
    shutil.rmtree(services)
    alias = destination/'XPCServices'
    if alias.is_symlink():
        alias.unlink()
    info_path = app/'Contents/Info.plist'
    info = plistlib.loads(info_path.read_bytes())
    info['XdzipUpdateInstallable'] = bool(installable and info.get('SUPublicEDKey'))
    info['XdzipUpdateInstallId'] = INSTALL_ID
    for key in ('SUEnableInstallerLauncherService', 'SUEnableDownloaderService'):
        info[key] = False
    info_path.write_bytes(plistlib.dumps(info, sort_keys=False))
    return destination


def signing_metadata(framework, checked):
    """Capture helper entitlements before lipo or relocation invalidates signatures."""
    targets = [framework/'Versions/B/Autoupdate', framework/'Versions/B/Updater.app']
    result = []
    for target in targets:
        if not target.exists() or target.is_symlink():
            raise ValueError('Sparkle installer helper missing')
        data = checked(['codesign', '-d', '--entitlements', ':-', '--xml', target]).strip()
        entitlements = plistlib.loads(data.encode()) if data else {}
        # Official release helpers must never grant debugger attachment.
        if entitlements.get('com.apple.security.get-task-allow'):
            raise ValueError('Sparkle release helper permits debugger attachment')
        result.append((target, entitlements))
    return result


def sign_helpers(framework, metadata, checked, work):
    """Preserve per-helper entitlements and Hardened Runtime; sign inside out."""
    for index, (target, entitlements) in enumerate(metadata):
        args = ['codesign', '--force', '--sign', '-', '--timestamp=none', '--options', 'runtime']
        if entitlements:
            path = work/f'sparkle-helper-{index}.entitlements'
            path.write_bytes(plistlib.dumps(entitlements))
            args += ['--entitlements', path]
        checked(args + [target])
    checked(['codesign', '--force', '--sign', '-', '--timestamp=none', '--options', 'runtime', framework])
    checked(['codesign', '--verify', '--deep', '--strict', framework])


def attribute(report, app, sdk, audit):
    """Supplement Qt attribution with exact Sparkle files and arm64 code evidence."""
    from deployment_audit import native
    from macos_attribution import compare_images
    validate_sdk(sdk)
    spec = configuration()
    owner = 'sparkle:Sparkle@'+spec['version']
    records, owned = [], set()
    for item in audit['files']:
        relative = item['path']
        if not relative.startswith(PREFIX):
            continue
        path = app/relative
        original = sdk/'Sparkle.framework'/relative.removeprefix(PREFIX)
        if (not original.resolve().is_relative_to(sdk.resolve()) or not original.is_file() or
                checksum(path) != item['sha256']):
            raise ValueError('Sparkle runtime source is missing or stale')
        if native(path, 'macos'):
            fingerprint = compare_images(path.read_bytes(), original.read_bytes())
            method = 'mach-o-sections-uuid-and-imports'
        elif checksum(original) == item['sha256']:
            fingerprint, method = None, 'sdk-byte-identity'
        elif '/_CodeSignature/' in relative:
            continue  # Recreated seals are generated package metadata.
        else:
            raise ValueError('Unexplained Sparkle resource change')
        records.append({'path': 'artifact/'+relative, 'sha256': item['sha256'], 'owners': [owner],
                        'evidence': method, 'sdk_path': original.relative_to(sdk).as_posix(),
                        'sdk_sha256': checksum(original), 'fingerprint': fingerprint,
                        'upstream_checksums': []})
        owned.add(relative)
    if not records or not any(r['evidence'].startswith('mach-o') for r in records):
        raise ValueError('No native Sparkle deployment evidence')
    report['files'] += records
    report['gaps'] = [g for g in report['gaps'] if not (
        g.get('kind') == 'native-outside-qt-attribution' and g.get('file') in owned)]
    report['components'].append({'ref': owner, 'name': 'Sparkle', 'version': spec['version'],
        'repository': 'sparkle', 'license_declared': spec['license'],
        'license_concluded_by_upstream': spec['license'], 'copyright': 'See bundled Sparkle LICENSE',
        'source': 'https://github.com/sparkle-project/Sparkle/tree/'+spec['version']})
    report['dependencies'].append({'ref': owner, 'dependsOn': []})
    report['sparkle'] = dict(spec, files=len(records),
        limitation='SDK-to-arm64 section comparison, not a claim that upstream binaries were rebuilt.')
    return report
