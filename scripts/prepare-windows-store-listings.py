#!/usr/bin/env python3
"""Prepare and verify Partner Center MSIX listings from an exported CSV."""

import argparse
import csv
import hashlib
import json
import os
import re
import struct
import xml.etree.ElementTree as ET
from pathlib import Path
from urllib.error import HTTPError, URLError
from urllib.parse import urlencode, urlsplit
from urllib.request import HTTPRedirectHandler, Request, build_opener
from zipfile import ZipFile


ROOT = Path(__file__).resolve().parents[1]
MARKETING = ROOT / 'specs/marketing/windows-store.json'
LOCALES = ROOT / 'specs/marketing/windows-store-locales.json'
SCENES = ('01-welcome', '02-create', '03-preview', '04-workspace')
SCREENSHOTS = tuple(f'DesktopScreenshot{i}' for i in range(1, 5))
CAPTIONS = tuple(f'DesktopScreenshotCaption{i}' for i in range(1, 5))
TEXT_FIELDS = {
    'Description': ('description', 10000), 'ReleaseNotes': ('release_notes', 1500),
    'ShortTitle': ('short_title', 50), 'ShortDescription': ('short_description', 1000),
    'DevStudio': ('developer', 255), 'CopyrightTrademarkInformation': ('copyright', 200),
}
LOCALE_CODE = re.compile(r'[a-z]{2,3}(?:-[a-z0-9]{2,8})+')


def read_json(path):
    return json.loads(path.read_text(encoding='utf-8-sig'))


def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def package_info(path):
    with ZipFile(path) as package:
        manifest = ET.fromstring(package.read('AppxManifest.xml'))
    return {
        'version': manifest.find('./{*}Identity').attrib['Version'], 'sha256': digest(path),
        'languages': sorted(r.attrib['Language'].lower() for r in
                            manifest.findall('./{*}Resources/{*}Resource') if 'Language' in r.attrib),
    }


def locale_map(sources, copy, package_languages):
    if set(sources) != set(copy) or {key.lower() for key in sources} != set(package_languages):
        raise ValueError('MSIX languages, marketing copy and locale map sources must agree')
    result = {}
    for source, targets in sources.items():
        if source.lower() not in targets:
            raise ValueError(f'Missing base locale for {source}')
        for target in targets:
            family_matches = target.split('-')[0] == source.split('-')[0]
            if not LOCALE_CODE.fullmatch(target) or not family_matches or target in result:
                raise ValueError(f'Invalid or duplicate locale mapping: {target}')
            if source == 'en-US' and target != 'en-us':
                raise ValueError('Only en-US may be used for English listings')
            if source in ('zh-CN', 'zh-TW'):
                traditional = target in ('zh-tw', 'zh-hk', 'zh-mo') or target.startswith('zh-hant-')
                simplified = target in ('zh-cn', 'zh-sg') or target.startswith('zh-hans-')
                if not (traditional if source == 'zh-TW' else simplified):
                    raise ValueError(f'Chinese script does not match {source}: {target}')
            result[target] = source
    return result


def read_csv(path):
    with path.open(encoding='utf-8-sig', newline='') as stream:
        header, *rows = csv.reader(stream)
    rows = [row for row in rows if any(row)]
    if header[:2] != ['Field', 'ID'] or header[3:4] != ['default']:
        raise ValueError('Use an exported Microsoft Store MSIX listings CSV')
    locales = [cell.lower() for cell in header[4:]]
    if len(set(locales)) != len(locales) or any(not LOCALE_CODE.fullmatch(code) for code in locales):
        raise ValueError('CSV contains duplicate or invalid locale columns')
    keys = [row[0] for row in rows if row and row[0]]
    ids = [row[1] for row in rows if len(row) > 1 and row[1]]
    if (len(keys) != len(rows) or len(set(keys)) != len(keys)
            or len(ids) != len(rows) or len(set(ids)) != len(ids)
            or any(len(row) != len(header) for row in rows)):
        raise ValueError('CSV contains duplicate fields or inconsistent row widths')
    return header, rows, {code: i for i, code in enumerate(locales, 4)}


def check_text(name, value, limit):
    if not isinstance(value, str) or len(value.encode('utf-16-le')) // 2 > limit:
        raise ValueError(f'Invalid text or length for {name} (maximum {limit})')
    return value


def text_fields(copy, release_notes):
    data = {name: check_text(name, copy[key], limit) for name, (key, limit) in TEXT_FIELDS.items()}
    data['ReleaseNotes'] = check_text('ReleaseNotes', release_notes, 1500)
    if not data['Description'].strip():
        raise ValueError('Description is required')
    for field, values, slots, limit in (('Feature', copy['features'], 20, 200),
                                        ('SearchTerm', copy['keywords'], 7, 40)):
        if len(values) > slots:
            raise ValueError(f'Too many entries for {field}')
        for index in range(slots):
            name = f'{field}{index + 1}'
            data[name] = check_text(name, values[index] if index < len(values) else '', limit)
    return data


def material_evidence(root, sources):
    manifest = read_json(root / 'manifest.json')
    hashes = {item['file']: item['sha256'] for item in manifest['files']}
    result = {}
    for source in sorted(sources):
        result[source] = []
        for scene in SCENES:
            relative = f'{source}/promotional/{scene}-1920x1080.png'
            path = root / relative
            with path.open('rb') as stream:
                header = stream.read(24)
            sha = digest(path)
            if (header[:16] != b'\x89PNG\r\n\x1a\n\0\0\0\rIHDR'
                    or struct.unpack('>II', header[16:24]) != (1920, 1080) or hashes.get(relative) != sha):
                raise ValueError(f'Artwork does not match the export manifest: {relative}')
            result[source].append({'scene': scene, 'path': relative, 'sha256': sha})
    return result


def screenshot_url(value):
    parsed = urlsplit(value)
    if parsed.scheme != 'https' or not parsed.hostname or parsed.username or parsed.password:
        raise ValueError('Missing HTTPS screenshot URL in the Partner Center export')
    return value


def generate(baseline, mapping, copy, *, only=(), notes=None, reuse=False, images_only=False):
    header, rows, columns = baseline
    fields = {row[0]: row for row in rows}
    missing = sorted(set(mapping) - set(columns))
    unexpected = sorted(set(columns) - set(mapping))
    if missing or unexpected:
        raise ValueError(f'Locale inventory differs from the map. Missing: {missing}. Unexpected: {unexpected}.')
    targets = [code.lower() for code in only] if only else list(columns)
    if len(targets) != len(set(targets)) or set(targets) - set(columns):
        raise ValueError('Select unique locales in the baseline export')
    if notes is not None and set(notes) != set(copy):
        raise ValueError('Release notes must cover each source language')
    required = set(SCREENSHOTS + CAPTIONS)
    if not images_only:
        required.update(TEXT_FIELDS)
        required.update(f'Feature{i}' for i in range(1, 21))
        required.update(f'SearchTerm{i}' for i in range(1, 8))
    if required - fields.keys():
        raise ValueError(f'Export is missing fields: {sorted(required - fields.keys())}')
    desired = {}
    changed = {}
    for code in targets:
        index = columns[code]
        source = mapping[code]
        material = copy[source]
        values = {} if images_only else text_fields(material, (notes or {}).get(source, material['release_notes']))
        values.update({name: check_text(name, material['screenshot_captions'][scene], 200)
                       for name, scene in zip(CAPTIONS, SCENES)})
        source_index = columns[source.lower()] if reuse else index
        for name in SCREENSHOTS:
            values[name] = screenshot_url(fields[name][source_index])
        if any(not values[name].strip() for name in CAPTIONS):
            raise ValueError(f'Missing screenshot caption for {source}')
        if any(row[index] for row in rows if re.fullmatch(r'DesktopScreenshot(?:[5-9]|[1-9][0-9])', row[0])):
            raise ValueError(f'Expected four desktop screenshots for {code}')
        desired[code] = values
        changed[code] = [name for name, value in values.items() if fields[name][index] != value]
    # Preserve every row, the default column and the existing values outside the owned fields.
    upload_rows = [[*header[:4], *(header[columns[code]] for code in targets)]]
    upload_rows.extend([*row[:4], *(desired[code].get(row[0], row[columns[code]]) for code in targets)]
                       for row in rows)
    return upload_rows, desired, changed


def compare(baseline, intended, actual):
    _, baseline_rows, baseline_columns = baseline
    _, intended_rows, intended_columns = intended
    _, actual_rows, actual_columns = actual
    issues = []
    if set(intended_columns) - set(baseline_columns):
        raise ValueError('The import must be prepared from this baseline')
    for code in sorted(set(baseline_columns) ^ set(actual_columns)):
        issues.append({'locale': code, 'problem': 'missing or unexpected locale'})
    expected_rows = {row[0]: row for row in intended_rows}
    actual_rows = {row[0]: row for row in actual_rows}
    if set(expected_rows) != {row[0] for row in baseline_rows}:
        raise ValueError('The import must preserve all baseline field rows')
    for field in sorted(actual_rows.keys() - expected_rows.keys()):
        issues.append({'field': field, 'problem': 'unexpected field'})
    for row in baseline_rows:
        field = row[0]
        actual_row = actual_rows.get(field)
        if actual_row is None or actual_row[1] != row[1]:
            issues.append({'field': field, 'problem': 'missing field or changed ID'})
            continue
        if actual_row[3] != row[3]:
            issues.append({'field': field, 'problem': 'changed default'})
        for code in baseline_columns.keys() & actual_columns.keys():
            expected = (expected_rows[field][intended_columns[code]] if code in intended_columns
                        else row[baseline_columns[code]])
            actual_value = actual_row[actual_columns[code]]
            if expected.replace('\r\n', '\n') != actual_value.replace('\r\n', '\n'):
                issues.append({'locale': code, 'field': field, 'problem': 'value differs'})
    return issues


def api_report(export, submission, expected_id):
    if str(submission['id']) != expected_id:
        raise ValueError('API response does not match the requested submission ID')
    _, rows, columns = export
    fields = {row[0]: row for row in rows}
    listings = {code.lower(): entry['baseListing'] for code, entry in submission['listings'].items()}
    issues = [{'locale': c, 'problem': 'missing from API'} for c in sorted(columns.keys() - listings.keys())]
    issues.extend({'locale': c, 'problem': 'unexpected API locale'} for c in sorted(listings.keys() - columns.keys()))
    properties = {'Title': 'title', 'Description': 'description', 'ReleaseNotes': 'releaseNotes',
                  'ShortTitle': 'shortTitle', 'ShortDescription': 'shortDescription',
                  'DevStudio': 'devStudio', 'CopyrightTrademarkInformation': 'copyrightAndTrademarkInfo'}
    for code in sorted(columns.keys() & listings.keys()):
        index = columns[code]
        listing = listings[code]
        screenshots = [i for i in listing['images'] if i['imageType'] == 'Screenshot']
        expected = [(fields[key][index] or fields[key][3]) for key in CAPTIONS]
        actual = [(image.get('description') or '') for image in screenshots]
        if expected != actual:
            issues.append({'locale': code, 'problem': 'desktop screenshot count, order or caption differs'})
        values = {field: listing.get(key) or '' for field, key in properties.items()}
        for slot, prefix, key in ((20, 'Feature', 'features'), (7, 'SearchTerm', 'keywords')):
            entries = listing.get(key) or []
            values.update({f'{prefix}{i+1}': entries[i] if i < len(entries) else '' for i in range(slot)})
        for field, value in values.items():
            expected_value = fields[field][index] or fields[field][3]
            if expected_value.replace('\r\n', '\n') != value.replace('\r\n', '\n'):
                issues.append({'locale': code, 'field': field, 'problem': 'API value differs'})
    return {'api_locales': len(listings), 'csv_locales': len(columns), 'differences': len(issues),
            'sample': issues[:30], 'package_versions': [p.get('version') for p in submission['applicationPackages']],
            'run_full_trust_declared': any('runFullTrust' in (p.get('capabilities') or [])
                                           for p in submission['applicationPackages']),
            'certification_notes_present': bool(submission.get('notesForCertification'))}


class NoRedirect(HTTPRedirectHandler):
    def redirect_request(self, *args, **kwargs):
        return None


def fetch_submission(product, submission):
    if not re.fullmatch(r'[A-Z0-9]{12}', product) or not re.fullmatch(r'[0-9]+', submission):
        raise ValueError('Use the Store product ID and numeric submission ID')
    tenant = os.environ.get('PARTNER_CENTER_TENANT_ID', '')
    client = os.environ.get('PARTNER_CENTER_CLIENT_ID', '')
    secret = os.environ.get('PARTNER_CENTER_CLIENT_SECRET', '')
    guid = r'[0-9a-fA-F]{8}(?:-[0-9a-fA-F]{4}){3}-[0-9a-fA-F]{12}'
    if not re.fullmatch(guid, tenant) or not re.fullmatch(guid, client) or not secret:
        raise ValueError('Set PARTNER_CENTER_TENANT_ID, PARTNER_CENTER_CLIENT_ID and PARTNER_CENTER_CLIENT_SECRET')
    host = 'https://manage.devcenter.microsoft.com'
    token_request = Request(f'https://login.microsoftonline.com/{tenant}/oauth2/token', data=urlencode({
        'grant_type': 'client_credentials', 'client_id': client, 'client_secret': secret, 'resource': host,
    }).encode('utf-8'))
    opener = build_opener(NoRedirect())
    with opener.open(token_request, timeout=30) as response:
        token = json.load(response)['access_token']
    endpoint = f'{host}/v1.0/my/applications/{product}/submissions/{submission}'
    with opener.open(Request(endpoint, headers={'Authorization': f'Bearer {token}'}), timeout=30) as response:
        return json.load(response)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='command', required=True)
    prepare = sub.add_parser('prepare')
    prepare.add_argument('--baseline', required=True, type=Path)
    prepare.add_argument('--package', required=True, type=Path)
    prepare.add_argument('--output', required=True, type=Path, help='New directory for the CSV and report')
    prepare.add_argument('--only', nargs='+', default=())
    prepare.add_argument('--release-notes', type=Path, help='Version-specific notes keyed by source language')
    prepare.add_argument('--materials', type=Path, help='Optional verified material export for the artwork inventory')
    prepare.add_argument('--reuse-source-urls', action='store_true')
    prepare.add_argument('--images-only', action='store_true', help='Change screenshot URLs and captions only')
    verify = sub.add_parser('verify')
    for name in ('baseline', 'intended', 'actual'):
        verify.add_argument(f'--{name}', required=True, type=Path)
    probe = sub.add_parser('api-report', help='Compare a read-only MSIX submission response with a Store export')
    probe.add_argument('--export', required=True, type=Path)
    source = probe.add_mutually_exclusive_group(required=True)
    source.add_argument('--submission-json', type=Path)
    source.add_argument('--product-id', help='Read the specified submission from the official MSIX API')
    probe.add_argument('--submission-id', required=True)
    args = parser.parse_args()
    try:
        if args.command == 'api-report':
            export = read_csv(args.export)
            submission = (read_json(args.submission_json) if args.submission_json
                          else fetch_submission(args.product_id, args.submission_id))
            report = api_report(export, submission, args.submission_id)
            print(json.dumps(report, indent=2))
            return bool(report['differences'])
        if args.command == 'verify':
            issues = compare(*(read_csv(file) for file in (args.baseline, args.intended, args.actual)))
            print(json.dumps({'differences': len(issues), 'sample': issues[:30]}, indent=2))
            return bool(issues)
        data = read_json(MARKETING)['locales']
        package = package_info(args.package)
        sources = read_json(LOCALES)
        if sources['schema_version'] != 1:
            raise ValueError('Unknown locale map version')
        mapping = locale_map(sources['sources'], data, package['languages'])
        rows, desired, changed = generate(read_csv(args.baseline), mapping, data, only=args.only,
                                          notes=read_json(args.release_notes) if args.release_notes else None,
                                          reuse=args.reuse_source_urls, images_only=args.images_only)
        artwork = material_evidence(args.materials, {mapping[code] for code in desired}) if args.materials else {}
        args.output.mkdir(parents=True, exist_ok=False)
        with (args.output / 'listing.csv').open('w', encoding='utf-8-sig', newline='') as stream:
            csv.writer(stream).writerows(rows)
        report = {'schema_version': 1, 'package': package, 'baseline_sha256': digest(args.baseline),
                  'listing_sha256': digest(args.output / 'listing.csv'), 'artwork': artwork,
                  'locales': {code: {'source': mapping[code], 'changed_fields': changed[code]}
                              for code in desired}}
        (args.output / 'plan.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
        (args.output / 'listings.json').write_text(json.dumps(desired, ensure_ascii=False, indent=2) + '\n',
                                                  encoding='utf-8')
        print(json.dumps({'locales': len(desired), 'changed_locales': sum(map(bool, changed.values())),
                          'fields': len(rows) - 1, 'artwork_files_verified': sum(map(len, artwork.values()))}))
    except HTTPError as error:
        parser.exit(1, f'Microsoft API returned HTTP {error.code}\n')
    except URLError:
        parser.exit(1, 'Could not reach the Microsoft API\n')
    except (ValueError, KeyError, OSError, ET.ParseError) as error:
        parser.exit(1, f'{error}\n')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
