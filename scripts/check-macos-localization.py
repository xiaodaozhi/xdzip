#!/usr/bin/env python3
"""Validate source and optionally built macOS system-localization metadata."""
import json
import plistlib
import sys
from pathlib import Path
from ui_languages import ROOT, load_languages, macos_language


def read_info(path):
    info = plistlib.loads(path.read_bytes())
    assert info.get('CFBundleAllowMixedLocalizations') is True, 'System frameworks must follow OS preferences'
    assert 'AppleLanguages' not in info, 'Do not force the system language'
    return info


def verify_bundle(path, localizations):
    info = read_info(path)
    assert set(info['CFBundleLocalizations']) == set(localizations), 'Bundle languages differ from registry'
    names = {item['CFBundleTypeName'] for item in info['CFBundleDocumentTypes']}
    names.update(item['UTTypeDescription'] for item in info.get('UTImportedTypeDeclarations', [])
                 if 'UTTypeDescription' in item)
    services = path.parent / 'Helpers/xdzip.service/Contents/Resources'
    for language, locale in localizations.items():
        folder = f'{language}.lproj'
        types = plistlib.loads((path.parent / 'Resources' / folder / 'InfoPlist.strings').read_bytes())
        assert types.keys() == names and all(types.values()), f'Incomplete document names for {locale}'
        menu = plistlib.loads((services / folder / 'ServicesMenu.strings').read_bytes())
        catalog = ROOT / 'resources/i18n' / f'{locale}.json'
        label = json.loads(catalog.read_text(encoding='utf-8'))['messages']['shell.title']
        assert menu == {'Open with xdzip': label}, f'Incorrect Finder Service name for {locale}'


if __name__ == '__main__':
    template = read_info(ROOT / 'apps/desktop/packaging/macos/Info.plist.in')
    assert {'zh-Hans', 'en'} <= set(template.get('CFBundleLocalizations', [])), 'Template languages missing'
    localizations = {macos_language(row['id']): row['id'] for row in load_languages()['languages']}
    for path in sys.argv[1:]:
        verify_bundle(Path(path), localizations)
    print(f'macOS system-localization metadata: {len(sys.argv)} files passed')
