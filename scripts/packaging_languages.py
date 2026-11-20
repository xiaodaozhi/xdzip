"""Render project-owned localized metadata for desktop packages."""
import argparse
from html import escape
import json
from pathlib import Path
import plistlib
import re

from ui_languages import ROOT, load_languages, macos_language

PACKAGING = ROOT / 'apps/desktop/packaging'
CATALOG = ROOT / 'resources/packaging-i18n.json'


def packaging_languages():
    identifiers = [row['id'] for row in load_languages()['languages']]
    return ('en-US', *(locale for locale in identifiers if locale != 'en-US'))


def packaging_catalogs():
    messages = json.loads(CATALOG.read_text(encoding='utf-8'))
    baseline = messages['en-US']
    if set(messages) != set(packaging_languages()):
        raise ValueError('Packaging language IDs differ from the shared registry')
    for locale, catalog in messages.items():
        if catalog.keys() != baseline.keys():
            raise ValueError(f'Packaging messages missing for {locale}')
        for key, text in catalog.items():
            if (not isinstance(text, str) or not text.strip() or any(ord(c) < 32 for c in text)
                    or re.findall(r'%\d+', text) != re.findall(r'%\d+', baseline[key])):
                raise ValueError(f'Invalid packaging message {locale}/{key}')
    return messages


def linux_resources():
    sources = PACKAGING / 'linux'
    catalogs = packaging_catalogs()
    localized = {locale.replace('-', '_'): catalog for locale, catalog in catalogs.items()
                 if locale != 'en-US'}
    # The system falls back from a regional POSIX locale to the bare language.
    localized.update({locale.split('-')[0]: catalog for locale, catalog in catalogs.items()
                      if locale not in ('en-US', 'zh-TW')})
    localized.update({locale: catalogs['zh-TW'] for locale in ('zh_HK', 'zh_MO')})
    desktop = (sources / 'xdzip.desktop').read_text(encoding='utf-8')
    # Linux chooses these values using the desktop locale, not the Qt preference.
    comments = [f"Comment={catalogs['en-US']['DesktopComment']}"]
    comments.extend(f"Comment[{locale}]={catalog['DesktopComment']}"
                    for locale, catalog in localized.items())
    desktop, matches = re.subn(r'^Comment=[^\n]+\nComment\[zh_CN\]=[^\n]+$',
                               lambda _: '\n'.join(comments), desktop, flags=re.M)
    if matches != 1:
        raise ValueError('Missing or ambiguous desktop comments')

    def xml_labels(source, element, key):
        labels = [f"    <{element}>{escape(catalogs['en-US'][key])}</{element}>"]
        labels.extend(f'    <{element} xml:lang="{locale}">{escape(catalog[key])}</{element}>'
                      for locale, catalog in localized.items())
        pattern = (rf'^    <{element}>.*</{element}>' '\n'
                   rf'    <{element} xml:lang="zh_CN">.*</{element}>$')
        rendered, matches = re.subn(pattern, lambda _: '\n'.join(labels), source, flags=re.M)
        if matches != 1:
            raise ValueError(f'Missing or ambiguous XML labels for {key}')
        return rendered

    policy = (sources / 'org.xdzip.update.policy').read_text(encoding='utf-8')
    policy = xml_labels(policy, 'description', 'UpdateDescription')
    policy = xml_labels(policy, 'message', 'UpdateMessage')
    mime = xml_labels((sources / 'xdzip-mime.xml').read_text(encoding='utf-8'), 'comment', 'Udf')
    appstream = (sources / 'xdzip-local.yml').read_text(encoding='utf-8')
    for header, key in [('Summary', 'AppStreamSummary'), ('Description', 'AppStreamDescription')]:
        entries = []
        for name, catalog in {'C': catalogs['en-US'], **localized}.items():
            text = catalog[key]
            if header == 'Description':
                text = f'<p>{escape(text)}</p>'
            entries.append(f'  {name}: {json.dumps(text, ensure_ascii=False)}')
        pattern = rf'^{header}:\n(?:  [^\n]+\n)+'
        appstream, matches = re.subn(pattern, lambda _: header + ':\n' + '\n'.join(entries) + '\n',
                                     appstream, flags=re.M)
        if matches != 1:
            raise ValueError(f'Missing or ambiguous AppStream section {header}')
    return {'xdzip.desktop': desktop, 'org.xdzip.update.policy': policy, 'xdzip-mime.xml': mime,
            'xdzip-local.yml': appstream}


def write_resource(path, data):
    if not path.is_file() or path.read_bytes() != data:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)


def macos_resources(output):
    source = plistlib.loads((PACKAGING / 'macos/Info.plist.in').read_bytes())
    names = {item['CFBundleTypeName'] for item in source['CFBundleDocumentTypes']}
    names.update(item['UTTypeDescription'] for item in source.get('UTImportedTypeDeclarations', [])
                 if 'UTTypeDescription' in item)

    def document_name(label, messages):
        if label.endswith(' ZIP-derived archive'):
            return messages['ZipDerived'].replace('%1', label.split()[0])
        if label.endswith(' compressed stream'):
            return messages['Stream'].replace('%1', label.split()[0])
        if label.endswith(' file'):
            return messages['File'].replace('%1', label[:-5])
        keys = {'Compressed TAR archive': 'CompressedTar', 'Microsoft Installer package': 'Msi',
                'ISO image': 'Iso', 'UDF image': 'Udf', 'UDF disc image': 'Udf',
                'Microsoft Cabinet archive': 'Cabinet'}
        if label in keys:
            return messages[keys[label]]
        if label.endswith(' archive'):
            return messages['Archive'].replace('%1', label[:-8])
        raise ValueError(f'Unregistered macOS document type label {label}')

    for locale, catalog in packaging_catalogs().items():
        directory = output / 'bundle' / (macos_language(locale) + '.lproj')
        values = {name: document_name(name, catalog) for name in sorted(names)}
        if locale == 'en-US':
            values = {name: name for name in sorted(names)}
        write_resource(directory / 'InfoPlist.strings', plistlib.dumps(values))

        product_catalog = ROOT / 'resources/i18n' / f'{locale}.json'
        if product_catalog.is_file():
            service = output / 'service' / (macos_language(locale) + '.lproj')
            messages = json.loads(product_catalog.read_text(encoding='utf-8'))['messages']
            write_resource(service / 'ServicesMenu.strings',
                           plistlib.dumps({'Open with xdzip': messages['shell.title']}))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--platform', choices=['linux', 'macos'], required=True)
    parser.add_argument('--output', type=Path, required=True)
    arguments = parser.parse_args()
    if arguments.platform == 'macos':
        macos_resources(arguments.output)
        print(';'.join(macos_language(locale) for locale in packaging_languages()))
    else:
        for filename, content in linux_resources().items():
            write_resource(arguments.output / filename, content.encode('utf-8'))
