"""Read the shared desktop and documentation language registry."""
import argparse
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
REGISTRY = ROOT / 'resources/ui-languages.json'
PLURAL_CATEGORIES = {
    'other': {'other'},
    'one': {'one', 'other'},
    'oneMillion': {'one', 'many', 'other'},
    'zeroOneMillion': {'one', 'many', 'other'},
    'russian': {'one', 'few', 'many'},
}


def load_languages(path=REGISTRY):
    registry = json.loads(path.read_text(encoding='utf-8'))
    languages = registry['languages']
    identifiers = [language['id'] for language in languages]
    if len(identifiers) != len(set(identifiers)) or registry['default'] not in identifiers:
        raise ValueError('Language IDs must be unique and include the default')
    if any(not re.fullmatch(r'[a-z]{2}-[A-Z]{2}', identifier) for identifier in identifiers):
        raise ValueError('Invalid language ID')
    legacy = {language['id'] for language in languages if language['legacy'] == language['id']}
    if legacy != {'en-US', 'zh-CN'} or registry['default'] != 'en-US':
        raise ValueError('The legacy languages and default must match existing settings')
    names = [language.get('documentName', language['name']) for language in languages]
    if len(names) != len(set(names)) or any(not name or any(c in name for c in '|[]<>\n') for name in names):
        raise ValueError('Invalid documentation language names')
    for language in languages:
        if language['legacy'] not in legacy or language['plural'] not in PLURAL_CATEGORIES:
            raise ValueError(f'Invalid legacy fallback or plural rule for {language["id"]}')
        if not re.fullmatch(r'[a-z]+', language['font']):
            raise ValueError(f'Invalid font group for {language["id"]}')
    return registry


def document_languages(registry=None):
    registry = load_languages() if registry is None else registry
    names = {item['id']: item.get('documentName', item['name']) for item in registry['languages']}
    default = registry['default']
    return {default: names.pop(default), **names}


def macos_language(locale):
    return {'zh-CN': 'zh-Hans', 'zh-TW': 'zh-Hant', 'pt-BR': 'pt-BR'}.get(locale, locale[:2])


def write_cpp_registry(output, registry):
    def utf8_literal(value):
        encoded = value.encode('utf-8')
        if encoded.isascii():
            return json.dumps(value)
        return '"' + ''.join(f'\\x{byte:02x}' for byte in encoded) + '"'

    entries = []
    for language in registry['languages']:
        values = ', '.join([*(utf8_literal(language[key]) for key in ('id', 'name', 'legacy', 'font')),
                            utf8_literal(macos_language(language['id']))])
        entries.append(f'    {{{values}, PluralRule::{language["plural"]}}},')
    rules = ', '.join(PLURAL_CATEGORIES)
    text = '\n'.join([
        '#pragma once',
        '#include <string_view>',
        'namespace xdzip_i18n {',
        f'enum class PluralRule {{ {rules} }};',
        'struct Language {',
        '    std::string_view id;',
        '    std::string_view name;',
        '    std::string_view legacy;',
        '    std::string_view font;',
        '    std::string_view macos;',
        '    PluralRule plural;',
        '};',
        f'inline constexpr std::string_view defaultLanguage = "{registry["default"]}";',
        'inline constexpr Language languages[] = {',
        *entries,
        '};',
        '}',
        '',
    ])
    if not output.is_file() or output.read_text(encoding='utf-8') != text:
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(text, encoding='utf-8')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cpp', type=Path, required=True)
    arguments = parser.parse_args()
    write_cpp_registry(arguments.cpp, load_languages())
