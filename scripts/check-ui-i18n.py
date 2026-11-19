#!/usr/bin/env python3
"""Validate product UI catalogs, placeholders and source coverage (stdlib only)."""
from collections import Counter
import json
from pathlib import Path
import re
from ui_terminology import inspect as inspect_terminology
from ui_languages import PLURAL_CATEGORIES, load_languages

ROOT = Path(__file__).resolve().parents[1]
NAME_FILTER = re.compile(r'\((\*[^()]*)\)\s*$')


def file_dialog_patterns(message):
    patterns = [NAME_FILTER.search(part) for part in message.split(';;')]
    return tuple(pattern[1] for pattern in patterns) if all(patterns) else ()


def main():
    registry = load_languages()
    languages = {language['id']: language for language in registry['languages']}
    files = {path.stem: path for path in (ROOT/'resources/i18n').glob('*.json')}
    if missing := languages.keys() - files.keys():
        raise SystemExit('Missing UI catalogs: ' + ', '.join(sorted(missing)))
    if unexpected := files.keys() - languages.keys():
        raise SystemExit('Unregistered UI catalogs: ' + ', '.join(sorted(unexpected)))
    catalogs = {locale: json.loads(path.read_text(encoding='utf-8')) for locale, path in files.items()}
    errors = []
    source = catalogs['zh-CN']['messages']
    filters = {key: patterns for key, value in source.items() if isinstance(value, str)
               and (patterns := file_dialog_patterns(value))}
    errors.extend(inspect_terminology(source))
    for locale, catalog in catalogs.items():
        if source.keys() != catalog['messages'].keys():
            errors.append(f'{locale}: catalog IDs differ')
        if catalog['locale'] != locale or catalog['schemaVersion'] != 1:
            errors.append(f'{locale}: invalid metadata')
        if catalog.get('name') != languages[locale]['name']:
            errors.append(f'{locale}: language name differs from registry')
        for key, value in catalog['messages'].items():
            texts = list(value.values()) if isinstance(value, dict) else [value]
            if not texts or any(not isinstance(text, str) or not text for text in texts):
                errors.append(f'{locale}/{key}: empty text')
                continue
            # The historical product still reads one/other from the Simplified Chinese catalog.
            categories = {'one', 'other'} if locale == 'zh-CN' else PLURAL_CATEGORIES[languages[locale]['plural']]
            if isinstance(value, dict) and set(value) != categories:
                errors.append(f'{locale}/{key}: incorrect plural forms')
            if locale == 'en-US' and key != 'language.zhCN' and any(re.search(r'[\u3400-\u9fff]', text) for text in texts):
                errors.append(f'{locale}/{key}: untranslated CJK text')
            expected = source.get(key, '')
            if isinstance(value, dict) != isinstance(expected, dict):
                errors.append(f'{locale}/{key}: plural shape differs')
            if key in filters and (not isinstance(value, str) or file_dialog_patterns(value) != filters[key]):
                errors.append(f'{locale}/{key}: file dialog patterns differ')
            expected = list(expected.values()) if isinstance(expected, dict) else [expected]
            placeholders = Counter(re.findall(r'%(?:L?\d+|L?n)', expected[0]))
            if any(Counter(re.findall(r'%(?:L?\d+|L?n)', text)) != placeholders for text in texts):
                errors.append(f'{locale}/{key}: mismatched placeholders')
    used = set()
    paths = list((ROOT/'apps/desktop/ui').glob('*.qml')) + [ROOT/'apps/desktop'/name for name in ['BrowserController.cpp','BrowserTransfers.cpp','UiPreferences.cpp','DesktopIntegration.cpp','ApplicationSessions.cpp','FileAssociations.cpp','MacFileAssociations.mm','browser_main.cpp','ShellBatchController.cpp','ShellIntegration.cpp']]
    paths += [ROOT/'apps/desktop/shell'/name for name in ['MacFinderSync.mm', 'MacFinderRelay.mm']]
    paths.append(ROOT/'crates/xdzip-cli/src/sfx/windows.rs')
    for path in paths:
        text = path.read_text(encoding='utf-8')
        for line in text.splitlines():
            if line.lstrip().startswith('//'): continue
            if re.search(r'"(?:[^"\\]|\\.)*[\u3400-\u9fff](?:[^"\\]|\\.)*"', line):
                errors.append(f'{path.name}: static CJK text remains in source')
        for match in re.finditer(r'(?:\b(?:qsTrId|qtTrId|systemDialogText|sfx_text)\(\s*|\bxdzip_shell::text\([^,]+,\s*)"([^"]+)"', text):
            key = match.group(1)
            used.add(key)
            if key not in source: errors.append(f'{path.name}: missing {key}')
            elif isinstance(source[key], dict) and not text[match.end():].lstrip().startswith(','):
                errors.append(f'{path.name}/{key}: plural call lacks quantity')
    if set(source)-used: errors.append('Unused message IDs: '+', '.join(sorted(set(source)-used)))
    if errors: raise SystemExit('\n'.join(errors))
    print(f'UI i18n: {len(source)} IDs, {len(catalogs)} complete catalogs, placeholders and source coverage passed')

if __name__ == '__main__': main()
