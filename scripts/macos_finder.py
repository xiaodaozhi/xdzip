"""Finder Sync's sandbox signature must survive all bundle signing passes."""
from pathlib import Path
import plistlib

ENTITLEMENTS = {
    'com.apple.security.app-sandbox': True,
    'com.apple.security.temporary-exception.files.home-relative-path.read-only':
        ['/Library/Application Support/xdzip/finder-sync.json'],
}


def sign_finder(app: Path, checked):
    info = plistlib.loads((app/'Contents/Info.plist').read_bytes())
    if 'XdzipFinderSyncProtocol' not in info:
        return  # Historical owned fixtures and older application bundles.
    if info['XdzipFinderSyncProtocol'] != 1:
        raise ValueError('Unsupported Finder Sync handoff protocol')
    extension = app/'Contents/PlugIns/xdzip-finder-sync.appex'
    relay = app/'Contents/Helpers/xdzip-shell-relay.app'
    for bundle, identity, executable in (
            (extension, 'org.xdzip.finder-sync', 'xdzip-finder-sync'),
            (relay, 'org.xdzip.shell-relay', 'xdzip-shell-relay')):
        if bundle.is_symlink():
            raise ValueError('Finder bundle must not be a symbolic link')
        metadata = plistlib.loads((bundle/'Contents/Info.plist').read_bytes())
        if metadata.get('CFBundleIdentifier') != identity or metadata.get('CFBundleExecutable') != executable:
            raise ValueError('Unexpected Finder bundle identity')
        binary = bundle/'Contents/MacOS'/executable
        if not binary.is_file() or binary.is_symlink():
            raise ValueError('Finder executable missing')
    extension_info = plistlib.loads((extension/'Contents/Info.plist').read_bytes())
    if extension_info.get('NSExtension', {}).get('NSExtensionPointIdentifier') != 'com.apple.FinderSync':
        raise ValueError('Unexpected Finder extension point')
    entitlements = extension/'Contents/Resources/Finder.entitlements'
    if entitlements.is_symlink() or plistlib.loads(entitlements.read_bytes()) != ENTITLEMENTS:
        raise ValueError('Unexpected Finder sandbox entitlements')
    checked(['codesign', '--force', '--sign', '-', '--timestamp=none',
             '--entitlements', entitlements, extension])
    checked(['codesign', '--force', '--sign', '-', '--timestamp=none', relay])
    checked(['codesign', '--verify', '--deep', '--strict', extension])
    checked(['codesign', '--verify', '--deep', '--strict', relay])
