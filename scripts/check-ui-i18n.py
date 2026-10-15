#!/usr/bin/env python3
"""Validate product UI catalogs, placeholders and source coverage (stdlib only)."""
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]

def main():
    catalogs = {locale: json.loads((ROOT/'resources/i18n'/f'{locale}.json').read_text(encoding='utf-8')) for locale in ['zh-CN','en-US']}
    errors = []
    source = catalogs['zh-CN']['messages']
    target = catalogs['en-US']['messages']
    if source.keys() != target.keys():
        errors.append('Catalog IDs differ')
    for locale, catalog in catalogs.items():
        if catalog['locale'] != locale or catalog['schemaVersion'] != 1:
            errors.append(f'{locale}: invalid metadata')
        for key, value in catalog['messages'].items():
            texts = list(value.values()) if isinstance(value, dict) else [value]
            if not texts or any(not isinstance(text, str) or not text for text in texts):
                errors.append(f'{locale}/{key}: empty text')
            if isinstance(value, dict) and set(value) != {'one','other'}:
                errors.append(f'{locale}/{key}: plural forms missing')
            if locale == 'en-US' and key != 'language.zhCN' and any(re.search(r'[\u3400-\u9fff]', text) for text in texts):
                errors.append(f'{locale}/{key}: untranslated CJK text')
            expected = source.get(key, '')
            if isinstance(value, dict) != isinstance(expected, dict):
                errors.append(f'{locale}/{key}: plural shape differs')
            expected = list(expected.values()) if isinstance(expected, dict) else [expected]
            placeholders = set(re.findall(r'%(?:L?\d+|n)', expected[0]))
            if any(set(re.findall(r'%(?:L?\d+|n)', text)) != placeholders for text in texts):
                errors.append(f'{locale}/{key}: mismatched placeholders')
    used = set()
    paths = list((ROOT/'apps/desktop/ui').glob('*.qml')) + [ROOT/'apps/desktop/BrowserController.cpp',ROOT/'apps/desktop/UiPreferences.cpp']
    for path in paths:
        text = path.read_text(encoding='utf-8')
        for line in text.splitlines():
            if line.lstrip().startswith('//'): continue
            if re.search(r'"(?:[^"\\]|\\.)*[\u3400-\u9fff](?:[^"\\]|\\.)*"', line):
                errors.append(f'{path.name}: static CJK text remains in source')
        for match in re.finditer(r'(?:qsTrId|qtTrId)\("([^"]+)"', text):
            key = match.group(1)
            used.add(key)
            if key not in source: errors.append(f'{path.name}: missing {key}')
            elif isinstance(source[key], dict) and not text[match.end():].lstrip().startswith(','):
                errors.append(f'{path.name}/{key}: plural call lacks quantity')
    if set(source)-used: errors.append('Unused message IDs: '+', '.join(sorted(set(source)-used)))
    if errors: raise SystemExit('\n'.join(errors))
    print(f'UI i18n: {len(source)} IDs, two complete catalogs, placeholders and source coverage passed')

if __name__ == '__main__': main()
