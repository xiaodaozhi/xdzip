#!/usr/bin/env python3
"""Generate built-in format declarations and entry candidates from one specification."""
import argparse
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / 'specs/engineering/archive-formats.json'
FIELDS = {
    'id', 'rust_variant', 'label', 'container', 'filters', 'worker_id', 'read_methods',
    'write_methods', 'rewrite', 'read_encryption', 'write_encryption', 'read_volumes',
    'write_volumes', 'creation_suffixes', 'shell_mime_types', 'extensions', 'check_kind', 'creation_levels',
}
LISTS = ('filters', 'read_methods', 'write_methods', 'read_encryption', 'write_encryption',
         'read_volumes', 'write_volumes', 'creation_suffixes', 'shell_mime_types')


def format_group(item):
    """Return the stable UI family for a format without adding product logic."""
    if item['container'] == 'text-envelope':
        return 'text-envelopes'
    if item['container'] == 'nsis':
        return 'installer-metadata'
    if item['container'] in ('single-file', 'encrypted-single-file'):
        return 'compressed-streams'
    if item['container'] == 'tar' and item['filters']:
        return 'compressed-tar'
    return 'archives'


def ui_groups(formats, *, shell_only=False):
    groups = []
    by_id = {}
    for item in formats:
        group_id = format_group(item)
        group = by_id.setdefault(group_id, {'id': group_id, 'labels': [], 'suffixes': []})
        if item['label'] not in group['labels']:
            group['labels'].append(item['label'])
        group['suffixes'].extend(extension['suffix'] for extension in item['extensions']
                                 if not shell_only or extension['shell'])
        if group not in groups:
            groups.append(group)
    return groups


def require(condition, message):
    if not condition:
        raise ValueError(message)


def validate(spec):
    require(set(spec) == {'schema_version', 'creation_methods', 'creation_wrappers', 'formats'} and
            type(spec['schema_version']) is int and spec['schema_version'] == 1,
            'Unsupported format specification')
    methods = spec['creation_methods']
    require(isinstance(methods, dict) and 0 < len(methods) <= 64 and
            all(isinstance(name, str) and re.fullmatch('[a-z0-9][a-z0-9-]{0,47}', name) and
                type(value) is int and 0 <= value <= 65535 for name, value in methods.items()),
            'Invalid creation method identifiers')
    require(isinstance(spec['formats'], list) and 0 < len(spec['formats']) <= 128,
            'Expected 1..128 formats')
    identities, variants, workers, suffixes, mime_types = set(), set(), set(), set(), set()
    for item in spec['formats']:
        require(isinstance(item, dict) and set(item) == FIELDS, 'Invalid format fields')
        for key in ('id', 'container'):
            require(isinstance(item[key], str) and re.fullmatch('[a-z0-9][a-z0-9-]{0,47}', item[key]),
                    'Invalid ' + key)
        require(item['id'] not in identities, 'Duplicate format ID')
        identities.add(item['id'])
        require(isinstance(item['rust_variant'], str) and
                re.fullmatch('[A-Z][A-Za-z0-9]{0,47}', item['rust_variant']) and
                item['rust_variant'] not in variants, 'Invalid or duplicate Rust variant')
        variants.add(item['rust_variant'])
        require(isinstance(item['label'], str) and re.fullmatch('[A-Za-z0-9 ._-]{1,48}', item['label']),
                'Invalid format label')
        require(type(item['rewrite']) is bool, 'Invalid rewrite capability')
        require(isinstance(item['check_kind'], str) and item['check_kind'] in
                {'supported_content', 'structure_and_readability', 'format_declared_checks'}, 'Invalid check kind')
        worker = item['worker_id']
        require(worker is None or (type(worker) is int and 0 < worker < 256 and worker not in workers),
                'Invalid or duplicate worker ID')
        if worker is not None:
            workers.add(worker)
        for key in LISTS:
            values = item[key]
            require(isinstance(values, list) and len(values) <= 64 and
                    all(isinstance(v, str) and re.fullmatch('[a-z0-9][a-z0-9/._+-]{0,95}|\\.[a-z0-9.]{1,31}', v)
                        for v in values) and len(values) == len(set(values)), 'Invalid ' + key)
        require(item['read_methods'], 'Missing read methods')
        require(set(item['write_methods']) <= set(item['read_methods']), 'Writer must have a reader')
        require(set(item['write_methods']) <= methods.keys(), 'Missing creation method identifier')
        method_ids = [methods[name] for name in item['write_methods']]
        require(len(method_ids) == len(set(method_ids)), 'Ambiguous creation method identifier')
        levels = item['creation_levels']
        require(isinstance(levels, list) and len(levels) <= 16 and
                all(type(level) is int and 0 <= level <= 255 for level in levels) and
                len(levels) == len(set(levels)) and bool(levels) == bool(item['write_methods']),
                'Invalid creation levels')
        require(set(item['write_encryption']) <= set(item['read_encryption']), 'Unverifiable encryption')
        require(set(item['write_volumes']) <= set(item['read_volumes']), 'Unverifiable volumes')
        require(bool(item['creation_suffixes']) == bool(item['write_methods']), 'Inconsistent creation')
        require(not item['rewrite'] or item['write_methods'], 'Rewrite needs write capability')
        require(item['write_methods'] or not (item['write_encryption'] or item['write_volumes']),
                'Read-only format declares write options')
        require(len(item['filters']) <= 3, 'Too many compression layers')
        require(isinstance(item['extensions'], list) and 0 < len(item['extensions']) <= 32,
                'Missing or excessive extensions')
        own = set()
        for extension in item['extensions']:
            require(isinstance(extension, dict) and set(extension) ==
                    {'suffix', 'drop', 'shell', 'windows_icon', 'macos_icon'}, 'Invalid extension fields')
            suffix = extension['suffix']
            require(isinstance(suffix, str) and re.fullmatch('\\.[a-z0-9]+(?:\\.[a-z0-9]+)*', suffix)
                    and len(suffix) <= 32 and suffix not in suffixes, 'Invalid or duplicate suffix')
            suffixes.add(suffix)
            own.add(suffix)
            require(type(extension['drop']) is bool and type(extension['shell']) is bool,
                    'Invalid entry-point flags')
            for platform in ('windows', 'macos'):
                icon = extension[platform + '_icon']
                require(icon is None or (isinstance(icon, str) and
                        re.fullmatch('[a-z0-9][a-z0-9-]{0,31}', icon)), 'Invalid association icon')
            require(extension['windows_icon'] is None or suffix.count('.') == 1,
                    'Windows association needs a final extension')
        require(set(item['creation_suffixes']) <= own, 'Creation suffix is not registered')
        for mime in item['shell_mime_types']:
            require(re.fullmatch('[a-z0-9.+-]+/[a-z0-9.+-]+', mime) and mime not in mime_types,
                    'Invalid or duplicate MIME type')
            mime_types.add(mime)
    wrappers = spec['creation_wrappers']
    require(isinstance(wrappers, list) and len(wrappers) == 1, 'Expected the owned SFX wrapper')
    wrapper = wrappers[0]
    require(wrapper == {'id': 'zip-sfx', 'format': 'zip', 'label': 'ZIP SFX',
                        'suffix': '.sfx.exe', 'platforms': ['windows']}, 'Invalid SFX wrapper')
    owner = next(item for item in spec['formats'] if item['id'] == wrapper['format'])
    require(wrapper['suffix'] in owner['creation_suffixes'], 'Unregistered wrapper suffix')
    return spec['formats']


def render(spec):
    formats = validate(spec)
    format_groups = ui_groups(formats)
    association_groups = [group for group in ui_groups(formats, shell_only=True) if group['suffixes']]
    rust = ['// Generated by scripts/generate-format-registry.py; edit the specification.',
            'use super::FormatDescriptor;', 'use crate::Format;', '',
            '#[rustfmt::skip]', 'pub(super) static CREATION_METHODS: &[(&str, u16)] = &[']
    rust.extend(f'    ({json.dumps(name)}, {value}),' for name, value in spec['creation_methods'].items())
    rust.extend(['];', '', 'pub(super) const SFX_SUFFIX: &str = ' + json.dumps(spec['creation_wrappers'][0]['suffix']) + ';', '', '#[rustfmt::skip]', 'pub(super) static DESCRIPTORS: &[FormatDescriptor] = &['])
    for item in formats:
        rust.extend(['    FormatDescriptor {', f'        format: Format::{item["rust_variant"]},'])
        for key in ('id', 'label', 'container'):
            rust.append(f'        {key}: {json.dumps(item[key])},')
        for key in ('filters', 'read_methods', 'write_methods', 'read_encryption', 'write_encryption',
                    'read_volumes', 'write_volumes', 'creation_suffixes'):
            # One item per line stays stable under rustfmt for both short and long lists.
            if item[key]:
                rust.append(f'        {key}: &[')
                rust.extend(f'            {json.dumps(value)},' for value in item[key])
                rust.append('        ],')
            else:
                rust.append(f'        {key}: &[],')
        rust.append(f'        rewrite: {str(item["rewrite"]).lower()},')
        rust.append('        creation_levels: &[' + ', '.join(str(level) for level in item['creation_levels']) + '],')
        rust.append(f'        check_kind: {json.dumps(item["check_kind"])},')
        rust.append('        extensions: &[')
        rust.extend(f'            {json.dumps(extension["suffix"])},' for extension in item['extensions'])
        rust.append('        ],')
        worker = 'None' if item['worker_id'] is None else f'Some({item["worker_id"]})'
        rust.extend([f'        worker_id: {worker},', '    },'])
    rust.extend(['];', '', 'pub(super) fn descriptor(format: Format) -> &\'static FormatDescriptor {',
                 '    match format {'])
    rust.extend(f'        Format::{item["rust_variant"]} => &DESCRIPTORS[{index}],'
                for index, item in enumerate(formats))
    rust.extend(['    }', '}', ''])
    # rustfmt skips generated tables so regeneration needs no installed formatter.
    cpp = ['// Generated by scripts/generate-format-registry.py; edit the specification.',
           '#pragma once', '#include <array>', '#include <string_view>', '// clang-format off',
           'namespace xdzip_formats {']
    extensions = [extension for item in formats for extension in item['extensions']]
    for name, values in (
            ('DropSuffixes', [e['suffix'] for e in extensions if e['drop']]),
            ('ShellSuffixes', [e['suffix'] for e in extensions if e['shell']]),
            ('ShellMimeTypes', [m for item in formats for m in item['shell_mime_types']]),
            ('ReadOnlyLabels', [item['label'] for item in formats if not item['write_methods']])):
        cpp.append(f'inline constexpr std::array<std::string_view, {len(values)}> {name} = {{{{')
        cpp.extend(f'    {json.dumps(value)},' for value in values)
        cpp.append('}};')
    cpp.append('inline constexpr std::array<std::string_view, ' + str(sum(item['container'] == 'tar' and bool(item['filters']) for item in formats)) + '> UnknownPackedLabels = {{')
    cpp.extend('    ' + json.dumps(item['label']) + ',' for item in formats if item['container'] == 'tar' and item['filters'])
    cpp.append('}};')
    cpp.extend(['} // namespace xdzip_formats', '// clang-format on', ''])
    windows = {e['suffix'][1:]: e['windows_icon'] for e in extensions if e['windows_icon'] is not None}
    macos = tuple(dict.fromkeys(e['macos_icon'] for e in extensions if e['macos_icon'] is not None))
    python = ['# Generated by scripts/generate-format-registry.py; edit the specification.',
              'WINDOWS_ICONS = {']
    python.extend(f'    {key!r}: {value!r},' for key, value in windows.items())
    python.extend(['}', 'MACOS_ICONS = ' + repr(macos),
                   'ASSOCIATION_FIXTURES = ' + repr(tuple('sample' + e['suffix'] for e in extensions if e['shell'])),
                   'SHELL_MIME_TYPES = ' + repr(tuple(m for item in formats for m in item['shell_mime_types'])), ''])
    choices = [dict(id=item['id'], label=item['label'], suffix=item['creation_suffixes'][0],
                    container=item['container'], methods=item['write_methods'],
                    methodIds=[spec['creation_methods'][name] for name in item['write_methods']],
                    levels=item['creation_levels'], encryption=item['write_encryption'],
                    volumes=item['write_volumes']) for item in formats if item['creation_suffixes']]
    for wrapper in spec['creation_wrappers']:
        choice = next(item for item in choices if item['id'] == wrapper['format']).copy()
        choice.update(id=wrapper['id'], label=wrapper['label'], suffix=wrapper['suffix'],
                      encryption=[], volumes=[], platforms=wrapper['platforms'])
        choices.append(choice)
    javascript = ('// Generated by scripts/generate-format-registry.py; edit the specification.\n'
                  '.pragma library\nvar creationFormats = ' + json.dumps(choices, indent=4) + ';\n'
                  'var associationSuffixes = ' + json.dumps([e['suffix'] for e in extensions if e['shell']]) + ';\n'
                  'var formatLabels = ' + json.dumps([item['label'] for item in formats]) + ';\n'
                  'var formatGroups = ' + json.dumps(format_groups, indent=4) + ';\n'
                  'var associationGroups = ' + json.dumps(association_groups, indent=4) + ';\n'
                  'var structureOnlyFormats = ' + json.dumps([item['label'] for item in formats if item['check_kind'] == 'structure_and_readability']) + ';\n'
                  'var discImageFormats = ' + json.dumps([item['label'] for item in formats if item['container'] in {'iso', 'udf'}]) + ';\n'
                  'var declaredCheckFormats = ' + json.dumps([item['label'] for item in formats if item['check_kind'] == 'format_declared_checks']) + ';\n')
    return {
        'crates/xdzip-codec-api/src/registry/generated.rs': '\n'.join(rust),
        'apps/desktop/ArchiveFormatsGenerated.h': '\n'.join(cpp),
        'scripts/archive_formats_generated.py': '\n'.join(python),
        'apps/desktop/ui/ArchiveFormats.js': javascript,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    try:
        outputs = render(json.loads(SOURCE.read_text(encoding='utf-8')))
    except (ValueError, TypeError, KeyError) as error:
        raise SystemExit('Invalid format registry: ' + str(error)) from error
    stale = []
    for name, content in outputs.items():
        path = ROOT / name
        if args.check:
            if not path.is_file() or path.read_text(encoding='utf-8') != content:
                stale.append(name)
        else:
            if not path.is_file() or path.read_text(encoding='utf-8') != content:
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(content, encoding='utf-8', newline='\n')
    if stale:
        raise SystemExit('Regenerate format declarations: ' + ', '.join(stale))
    print('Format registry generated sources are current' if args.check else 'Generated format registry')


if __name__ == '__main__':
    main()
