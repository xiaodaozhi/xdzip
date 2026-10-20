"""Locate the product artifact, keeping historical fixture names explicit."""
from pathlib import Path
import sys


def platform_executable_name(platform: str = sys.platform) -> str:
    return 'xdzip.exe' if platform == 'win32' else 'xdzip'


def built_product(build: Path, *, platform: str = sys.platform,
                  allow_legacy_name: bool = False) -> Path:
    desktop = build/'apps/desktop'
    if platform == 'darwin':
        candidates = [desktop/'xdzip.app/Contents/MacOS/xdzip']
    else:
        suffix = '.exe' if platform == 'win32' else ''
        candidates = [desktop/platform_executable_name(platform)]
        if allow_legacy_name:
            # Only pinned historical product rebuilds may use the pre-rename
            # artifact. Refuse an ambiguous reused directory rather than pick
            # a potentially stale executable for upgrade/rollback evidence.
            candidates.append(desktop/('xdzip-browser'+suffix))
    present = [path for path in candidates if path.is_file()]
    if len(present) != 1:
        raise ValueError('Expected exactly one built xdzip product artifact')
    return present[0]
