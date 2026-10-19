#!/usr/bin/env python3
"""Validate source and optionally built macOS system-localization metadata."""
import plistlib
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
paths = [ROOT / 'apps/desktop/packaging/macos/Info.plist.in'] + [Path(arg) for arg in sys.argv[1:]]
for path in paths:
    info = plistlib.loads(path.read_bytes())
    assert info.get('CFBundleAllowMixedLocalizations') is True, 'System frameworks must follow OS preferences'
    assert {'zh-Hans', 'en'} <= set(info.get('CFBundleLocalizations', [])), 'Bundle languages missing'
    assert 'AppleLanguages' not in info, 'Do not force the system language'
print(f'macOS system-localization metadata: {len(paths)} files passed')
