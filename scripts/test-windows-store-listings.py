#!/usr/bin/env python3
"""Exercise the Store import and export contract with synthetic listing data."""

import csv
import runpy
import tempfile
import unittest
from pathlib import Path


store = runpy.run_path(str(Path(__file__).with_name('prepare-windows-store-listings.py')))


class StoreListingsTests(unittest.TestCase):
    def setUp(self):
        self.sources = store['read_json'](store['LOCALES'])['sources']
        self.copy = store['read_json'](store['MARKETING'])['locales']
        self.mapping = store['locale_map'](self.sources, self.copy, [s.lower() for s in self.sources])
        self.screenshots = store['SCREENSHOTS']
        self.captions = store['CAPTIONS']
        fields = ('Title', 'Poster', *store['text_fields'](self.copy['en-US'], ''),
                  *self.screenshots, *self.captions)
        self.locales = sorted(self.mapping)
        header = ['Field', 'ID', 'Type', 'default', *self.locales]
        rows = []
        for i, field in enumerate(fields):
            values = [f'https://assets.example.invalid/{code}/{field}' if field in self.screenshots
                      else f'{code} {field}' for code in self.locales]
            rows.append([field, str(i), 'Text', f'default {field}', *values])
        self.baseline = self.table([header, *rows])

    def table(self, records):
        header, *rows = records
        return header, rows, {code: i for i, code in enumerate(header[4:], 4)}

    def test_known_catalog_and_inventory_drift(self):
        self.assertEqual(len(self.mapping), 337)
        self.assertEqual(self.mapping['zh-hans-cn'], 'zh-CN')
        self.assertEqual(self.mapping['zh-hant-hk'], 'zh-TW')
        self.assertEqual(self.mapping['ru-am'], 'ru-RU')
        self.assertNotIn('en-gb', self.mapping)
        with self.assertRaisesRegex(ValueError, 'ru-am'):
            store['generate'](self.baseline, {c: s for c, s in self.mapping.items() if c != 'ru-am'}, self.copy)

    def test_full_import_and_pilot_preserve_all_other_values(self):
        full, _, _ = store['generate'](self.baseline, self.mapping, self.copy)
        self.assertEqual(len(full[0]), 341)
        pilot, desired, _ = store['generate'](self.baseline, self.mapping, self.copy, only=['zh-Hant-HK'],
                                              reuse=True, images_only=True)
        self.assertEqual(set(desired['zh-hant-hk']), set(self.screenshots + self.captions))
        self.assertIn('/zh-tw/', desired['zh-hant-hk'][self.screenshots[0]])
        result = self.table([self.baseline[0], *(row.copy() for row in self.baseline[1])])
        _, rows, columns = result
        for actual, expected in zip(rows, pilot[1:]):
            self.assertEqual(actual[:4], expected[:4])
            actual[columns['zh-hant-hk']] = expected[-1]
        self.assertEqual(store['compare'](self.baseline, self.table(pilot), result), [])
        by_field = {row[0]: row for row in rows}
        by_field['Feature1'][columns['zh-hant-hk']] = ''
        by_field['SearchTerm1'][columns['en-us']] = ''
        by_field[self.captions[0]][columns['zh-hant-hk']] = 'old caption'
        issues = store['compare'](self.baseline, self.table(pilot), result)
        self.assertEqual({(i['locale'], i['field']) for i in issues},
                         {('zh-hant-hk', 'Feature1'), ('en-us', 'SearchTerm1'), ('zh-hant-hk', self.captions[0])})

    def test_export_with_blank_trailing_rows(self):
        header, rows, _ = self.baseline
        with tempfile.TemporaryDirectory() as path:
            export = Path(path) / 'listing.csv'
            with export.open('w', encoding='utf-8-sig', newline='') as stream:
                csv.writer(stream).writerows([header, *rows, [''] * len(header)])
            self.assertEqual(store['read_csv'](export), self.baseline)

    def test_api_reports_missing_locale_and_mismatched_caption(self):
        _, rows, _ = self.baseline
        export = self.table([['Field', 'ID', 'Type', 'default', 'en-us', 'fr-aw'],
                             *([*r[:4], '', ''] for r in rows)])
        listing = {'baseListing': {'images': [{'imageType': 'Screenshot', 'description': 'wrong'}]}}
        data = {'id': '12', 'listings': {'en-us': listing}, 'applicationPackages': [{'version': '1.0.32.0',
                                                                                   'capabilities': ['runFullTrust']}]}
        report = store['api_report'](export, data, '12')
        self.assertEqual(report['api_locales'], 1)
        self.assertTrue(report['run_full_trust_declared'])
        self.assertIn({'locale': 'fr-aw', 'problem': 'missing from API'}, report['sample'])
        self.assertIn({'locale': 'en-us', 'problem': 'desktop screenshot count, order or caption differs'},
                      report['sample'])


if __name__ == '__main__':
    unittest.main()
