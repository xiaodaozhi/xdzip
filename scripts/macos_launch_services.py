"""Own only xdzip registrations created by this checkout's build/test copies."""
from contextlib import contextmanager
from pathlib import Path
import plistlib
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
BUNDLE_ID = 'org.xdzip.desktop'
LSREGISTER = Path('/System/Library/Frameworks/CoreServices.framework/Frameworks/'
                  'LaunchServices.framework/Support/lsregister')


def registered_apps():
    data = subprocess.check_output([str(LSREGISTER), '-dump'], text=True)
    apps = set()
    for record in re.split(r'\n-{10,}\n', data):
        identity = re.search(r'^identifier:\s*(.*?)\s*$', record, re.M)
        path = re.search(r'^path:\s*(.*?)\s+\(0x[0-9a-fA-F]+\)\s*$', record, re.M)
        if identity and identity[1] == BUNDLE_ID and path:
            app = Path(path[1])
            if app.is_absolute() and app.suffix == '.app':
                apps.add(app)
    return apps


def build_copy(app, build=None):
    build = (build or ROOT/'build').resolve()
    app = Path(app)
    resolved = app.resolve()
    if app.is_symlink() or not resolved.is_relative_to(build) or resolved == build:
        return False
    if app.exists():
        try:
            info = plistlib.loads((app/'Contents/Info.plist').read_bytes())
        except (OSError, ValueError, plistlib.InvalidFileException):
            return False
        return info.get('CFBundleIdentifier') == BUNDLE_ID
    # Stale paths are eligible only when they came from the exact-id registry query.
    return app.suffix == '.app'


def unregister(app):
    result = subprocess.run([str(LSREGISTER), '-u', str(app)], capture_output=True, text=True)
    if result.returncode:
        raise RuntimeError('Could not unregister owned test application')


@contextmanager
def transient_registrations(directory):
    """Remove new test registrations even on failure, preserving prior registration."""
    if sys.platform != 'darwin':
        yield
        return
    directory = Path(directory).resolve()
    if not directory.is_relative_to(ROOT/'build'):
        raise ValueError('Temporary application registration must stay under build/')
    before = registered_apps()
    try:
        yield
    finally:
        for app in registered_apps() - before:
            if app.resolve().is_relative_to(directory) and build_copy(app):
                unregister(app)
        remaining = [app for app in registered_apps() - before
                     if app.resolve().is_relative_to(directory) and build_copy(app)]
        if remaining:
            raise RuntimeError('Temporary application registration cleanup did not converge')


def cleanup(keep, apply=False):
    keep = Path(keep).resolve()
    info = plistlib.loads((keep/'Contents/Info.plist').read_bytes())
    if info.get('CFBundleIdentifier') != BUNDLE_ID:
        raise ValueError('Kept application must be an existing xdzip bundle')
    before = registered_apps()
    candidates = sorted(app for app in before if app.resolve() != keep and build_copy(app))
    if apply:
        for app in candidates:
            unregister(app)
        # Repair the selected application's own record, without setting any default handler.
        result = subprocess.run([str(LSREGISTER), '-f', str(keep)],
                                capture_output=True, text=True)
        if result.returncode:
            raise RuntimeError('Could not register selected primary application')
    after = registered_apps() if apply else before
    remaining = [app for app in after if app.resolve() != keep and build_copy(app)]
    if apply and (remaining or keep not in {p.resolve() for p in after}):
        raise RuntimeError('LaunchServices did not converge to the requested build-copy set')
    return {'mode': 'applied' if apply else 'preview', 'candidate_count': len(candidates),
            'remaining_build_duplicates': len(remaining), 'kept_registered': keep in after,
            'outside_build_untouched': len([p for p in before if not build_copy(p)]),
            'application_files_deleted': False, 'default_handler_changed': False}
