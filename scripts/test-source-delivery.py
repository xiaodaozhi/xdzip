#!/usr/bin/env python3
"""Source cache integrity and portable bundle boundary tests, using owned fixtures."""
import importlib.util
import hashlib
import io
import json
import os
from contextlib import nullcontext, redirect_stdout
from pathlib import Path
import plistlib
import runpy
import re
import subprocess
import sys
import tarfile
import tempfile
import unittest
import zipfile
from unittest.mock import patch
from deployment_audit import digest, within
from desktop_product import built_product
import macos_launch_services as ls
import rar_mapping_materials
import vendor_notice_materials
import qt_psl_source
import cldr_webapp_notices

ROOT = Path(__file__).resolve().parents[1]


def module(name):
    spec = importlib.util.spec_from_file_location(name, ROOT/'scripts'/(name+'.py'))
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


source = module('prepare-source-delivery')
mac = module('build-macos-dmg')
verify_dmg = module('verify-macos-dmg')
replace_qt = module('verify-qt-replacement')
derive_locale_codes = module('derive-qt-locale-code-input')
convert_iso_codes = module('convert-iso-codes-for-qt')
convert_iana_loc = module('convert-iana-loc-for-qt')
verify_emoji_alternatives = module('verify-cldr-emoji-alternatives')


def iana_loc_fixture():
    iana = ('File-Date: 2026-09-17\n%%\nType: language\nSubtag: de\nDescription: owned\n'
            '%%\nType: language\nSubtag: en\n%%\nType: language\nSubtag: xyz\n'
            '%%\nType: language\nSubtag: zz\n%%\nType: language\nSubtag: qaa..qtz\n'
            '%%\nType: region\nSubtag: ZZ\n')
    loc = ('ger|deu|de|owned|owned\neng||en|owned|owned\n'
           'qaa-qtz||||\nyyy|||owned|owned\n')
    return iana, loc


def psl_fixture(root, wrong_generated=None):
    """Owned miniature archive exercises provenance and both regeneration paths."""
    inputs = root/'third_party/qt-public-suffix-list'
    inputs.mkdir(parents=True)
    (root/'scripts').mkdir()
    data, license_text = b'owned.example\n', b'owned license fixture\n'
    generator = (b'import pathlib,sys\n'
                 b'data=pathlib.Path(sys.argv[-2]).read_bytes()\n'
                 b'prefix=b"BIN:" if "binary" in sys.argv[1] else b"CPP:"\n'
                 b'pathlib.Path(sys.argv[-1]).write_bytes(prefix+data)\n')
    cpp, binary = b'CPP:'+data, b'BIN:'+data
    if wrong_generated == 'cpp':
        cpp = b'incorrect C++ data'
    if wrong_generated == 'binary':
        binary = b'incorrect binary data'
    for name, value in [('public_suffix_list.dat', data), ('PSL-LICENSE.txt', license_text),
                        ('README.md', b'owned provenance fixture')]:
        (inputs/name).write_bytes(value)
    commit = 'a'*40
    attribution = b'"Version": "'+commit.encode()+b'", "LicenseId": "MPL-2.0"'
    archive_name = 'qtbase-everywhere-src-fixture.tar.xz'
    prefix = archive_name.removesuffix('.tar.xz')+'/'
    members = {'src/3rdparty/libpsl/src/psl-make-dafsa': generator,
               'src/3rdparty/libpsl/PSL-LICENSE.txt': license_text,
               'src/3rdparty/libpsl/qt_attribution.json': attribution,
               'src/3rdparty/libpsl/psl_data.cpp': cpp,
               'tests/auto/network/access/qnetworkcookiejar/testdata/publicsuffix/public_suffix_list.dafsa': binary}
    archive = root/archive_name
    with tarfile.open(archive, 'w:xz') as output:
        for name, content in members.items():
            info = tarfile.TarInfo(prefix+name); info.size = len(content)
            output.addfile(info, io.BytesIO(content))
    sha = lambda value: hashlib.sha256(value).hexdigest()
    config = {'qt_archive': archive_name, 'qt_archive_sha256': digest(archive),
              'upstream_repository': 'https://github.com/publicsuffix/list',
              'upstream_commit': commit, 'upstream_git_blob': qt_psl_source.git_blob(data),
              'source_sha256': sha(data), 'license_sha256': sha(license_text),
              'generator_sha256': sha(generator), 'generated_cpp_sha256': sha(cpp),
              'generated_binary_sha256': sha(binary)}
    (root/'scripts/qt-psl-source.json').write_text(json.dumps(config), encoding='utf-8')
    (root/'scripts/corresponding-sources.json').write_text(json.dumps({'archives': [
        {'module': 'qtbase', 'file': archive_name, 'sha256': digest(archive)}]}), encoding='utf-8')
    destination = root/'delivery 中文 with spaces'; destination.mkdir()
    return archive, destination


class DeliveryTests(unittest.TestCase):
    def test_convert_iana_loc_for_qt_preserves_codes_and_reports_omissions(self):
        iana, loc = iana_loc_fixture()
        table, report = convert_iana_loc.convert(iana, loc)
        self.assertEqual(table, b'Id\tPart2b\tPart2t\tPart1\tUnused\n'
                         b'deu\tger\tdeu\tde\t\neng\teng\teng\ten\t\nxyz\t\t\t\t\n')
        self.assertEqual(report['iana_file_date'], '2026-09-17')
        self.assertEqual(report['iana_two_letter_without_loc'], ['zz'])
        self.assertEqual(report['loc_unmatched'], ['yyy'])
        self.assertFalse(report['historical_input_verified'])
        self.assertFalse(report['complete_corresponding_source_verified'])
        self.assertEqual(convert_iana_loc.convert(iana.replace('\n', '\r\n'),
                                                  '\ufeff'+loc)[0], table)

    def test_convert_iana_loc_for_qt_rejects_malformed_and_conflicting_sources(self):
        iana, loc = iana_loc_fixture()
        invalid_iana = ('bad header\n'+iana, iana.replace('2026-09-17', '2026-99-99'),
                        iana+'%%\nType: language\nSubtag: de\n',
                        iana+'%%\nType: language\nSubtag: EN\n',
                        iana+'%%\nType: language\n',
                        iana+'%%\nType: language\nType: region\nSubtag: aa\n',
                        iana+'%%\nType: language\nSubtag: qaa..qtz\n')
        for fixture in invalid_iana:
            with self.subTest(iana=fixture), self.assertRaises(ValueError):
                convert_iana_loc.convert(fixture, loc)
        invalid_loc = ('', loc+'bad\n', loc+'eng||en|duplicate|duplicate\n',
                       loc+'aaa||en|duplicate|duplicate\n',
                       loc+'xyz||aa|inconsistent|inconsistent\n',
                       loc+'qaa-qtz||||\n', loc.replace('ger|deu|de', 'GER|deu|de'))
        for fixture in invalid_loc:
            with self.subTest(loc=fixture), self.assertRaises(ValueError):
                convert_iana_loc.convert(iana, fixture)

    def test_convert_iana_loc_cli_stays_in_build_and_never_overwrites(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            iana, loc = iana_loc_fixture()
            first, second = root/'iana.txt', root/'loc.txt'
            first.write_text(iana, encoding='utf-8')
            second.write_text(loc, encoding='utf-8')
            output = root/'build/codes.tsv'
            argv = ['convert', '--iana', str(first), '--loc', str(second), '--output', str(output)]
            with patch.object(convert_iana_loc, 'ROOT', root), patch.object(sys, 'argv', argv), \
                    redirect_stdout(io.StringIO()) as out:
                convert_iana_loc.main()
                saved = output.read_bytes()
                self.assertEqual(json.loads(out.getvalue())['output_sha256'],
                                 hashlib.sha256(saved).hexdigest())
                with self.assertRaises(FileExistsError):
                    convert_iana_loc.main()
                self.assertEqual(output.read_bytes(), saved)
            argv[-1] = str(root/'outside.tsv')
            with patch.object(convert_iana_loc, 'ROOT', root), patch.object(sys, 'argv', argv), \
                    patch('sys.stderr', new_callable=io.StringIO):
                with self.assertRaises(SystemExit):
                    convert_iana_loc.main()
            self.assertFalse((root/'outside.tsv').exists())

    def test_convert_iso_codes_for_qt_handles_bibliographic_and_missing_part2(self):
        part2 = json.dumps({'639-2': [{'alpha_3': 'eng'},
                                     {'alpha_3': 'deu', 'bibliographic': 'ger'},
                                     {'alpha_3': 'qaa-qtz'}]})
        part3 = json.dumps({'639-3': [{'alpha_3': 'deu', 'alpha_2': 'de'},
                                     {'alpha_3': 'eng', 'alpha_2': 'en'},
                                     {'alpha_3': 'xyz'}]})
        table, count2, count3 = convert_iso_codes.convert(part2, part3)
        self.assertEqual((count2, count3), (3, 3))
        self.assertEqual(table, b'Id\tPart2b\tPart2t\tPart1\tUnused\n'
                         b'deu\tger\tdeu\tde\t\neng\teng\teng\ten\t\nxyz\t\t\t\t\n')

    def test_convert_iso_codes_for_qt_rejects_ambiguous_or_invalid_inputs(self):
        part2 = {'639-2': [{'alpha_3': 'eng'}]}
        for part3 in ({'639-3': []}, {'639-3': [{'alpha_3': 'ENG'}]},
                      {'639-3': [{'alpha_3': 'eng'}, {'alpha_3': 'eng'}]},
                      {'639-3': [{'alpha_3': 'eng', 'alpha_2': 'en'},
                                 {'alpha_3': 'xyz', 'alpha_2': 'en'}]}):
            with self.subTest(part3=part3), self.assertRaises(ValueError):
                convert_iso_codes.convert(json.dumps(part2), json.dumps(part3))
        with self.assertRaises(ValueError):
            convert_iso_codes.convert('{', '{}')
        with self.assertRaises(ValueError):
            convert_iso_codes.convert(json.dumps({'639-2': [{'alpha_3': 'eng'},
                                                             {'alpha_3': 'eng'}]}),
                                      json.dumps({'639-3': [{'alpha_3': 'eng'}]}))
        with self.assertRaises(ValueError):
            convert_iso_codes.convert(json.dumps({'639-2': [{'alpha_3': 'qaa-qtz'},
                                                             {'alpha_3': 'qaa-qtz'}]}),
                                      json.dumps({'639-3': [{'alpha_3': 'eng'}]}))

    def test_convert_iso_codes_cli_stays_in_build_and_never_overwrites(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            part2, part3 = root/'2.json', root/'3.json'
            part2.write_text('{"639-2":[{"alpha_3":"eng"}]}', encoding='utf-8')
            part3.write_text('{"639-3":[{"alpha_3":"eng","alpha_2":"en"}]}', encoding='utf-8')
            output = root/'build/codes.tsv'
            argv = ['convert', '--iso-639-2', str(part2), '--iso-639-3', str(part3),
                    '--output', str(output)]
            with patch.object(convert_iso_codes, 'ROOT', root), patch.object(sys, 'argv', argv), \
                    redirect_stdout(io.StringIO()):
                convert_iso_codes.main()
                expected = output.read_bytes()
                with self.assertRaises(FileExistsError):
                    convert_iso_codes.main()
                self.assertEqual(output.read_bytes(), expected)
            argv[-1] = str(root/'outside.tsv')
            with patch.object(convert_iso_codes, 'ROOT', root), patch.object(sys, 'argv', argv), \
                    patch('sys.stderr', new_callable=io.StringIO):
                with self.assertRaises(SystemExit):
                    convert_iso_codes.main()
            self.assertFalse((root/'outside.tsv').exists())

    def test_derive_qt_locale_codes_preserves_fields_and_deduplicates_lookup(self):
        fixture = '''constexpr std::array<LanguageCodeEntry, 3> languageCodeList {
    LanguageCodeEntry {{}, {'u', 'n', 'd'}, {'u', 'n', 'd'}, {'u', 'n', 'd'}}, // AnyLanguage
    LanguageCodeEntry {{}, {'u', 'n', 'd'}, {'u', 'n', 'd'}, {'u', 'n', 'd'}}, // C
    LanguageCodeEntry {{'e', 'n'}, {'e', 'n', 'g'}, {'e', 'n', 'g'}, {'e', 'n', 'g'}}, // English
};
'''
        data, entries, unique = derive_locale_codes.derived_table(fixture)
        self.assertEqual((entries, unique), (3, 2))
        self.assertEqual(data, b'Id\tPart2b\tPart2t\tPart1\tUnused\n'
                         b'eng\teng\teng\ten\t\nund\tund\tund\t\t\n')
        self.assertEqual(derive_locale_codes.derived_table(fixture.replace('\n', '\r\n')),
                         (data, entries, unique))

    def test_derive_qt_locale_codes_adds_only_explicit_registered_mappings(self):
        qt = '''constexpr std::array<LanguageCodeEntry, 2> languageCodeList {
    LanguageCodeEntry {{}, {'u', 'n', 'd'}, {'u', 'n', 'd'}, {'u', 'n', 'd'}}, // C
    LanguageCodeEntry {{'e', 'n'}, {'e', 'n', 'g'}, {'e', 'n', 'g'}, {'e', 'n', 'g'}}, // English
};
'''
        iana, _ = iana_loc_fixture()
        table, entries, unique, file_date = derive_locale_codes.extend_table(qt, iana, ['xyz,xyz,xyz,'])
        self.assertEqual((entries, unique, file_date), (2, 2, '2026-09-17'))
        self.assertEqual(table, b'Id\tPart2b\tPart2t\tPart1\tUnused\n'
                         b'eng\teng\teng\ten\t\nund\tund\tund\t\t\nxyz\txyz\txyz\t\t\n')
        without_part2, _, _, _ = derive_locale_codes.extend_table(qt, iana, ['xyz,,,'])
        self.assertIn(b'xyz\t\t\t\t\n', without_part2)
        with_two_letter, _, _, _ = derive_locale_codes.extend_table(qt, iana, ['deu,ger,deu,de'])
        self.assertIn(b'deu\tger\tdeu\tde\t\n', with_two_letter)
        self.assertNotIn(b'zzz\t', table)

    def test_derive_qt_locale_codes_rejects_implicit_or_conflicting_mappings(self):
        qt = '''constexpr std::array<LanguageCodeEntry, 1> languageCodeList {
    LanguageCodeEntry {{'e', 'n'}, {'e', 'n', 'g'}, {'e', 'n', 'g'}, {'e', 'n', 'g'}}, // English
};
'''
        iana, _ = iana_loc_fixture()
        invalid = [[], ['xyz'], ['xyz,xyz,,'], ['xyz,xyz,abc,'], ['XYZ,xyz,xyz,'],
                   ['qqq,,,'], ['eng,eng,eng,en'], ['xyz,xyz,xyz,en'],
                   ['xyz,eng,xyz,'], ['xyz,ger,xyz,', 'deu,ger,deu,de'],
                   ['xyz,,,', 'xyz,,,']]
        for additions in invalid:
            with self.subTest(additions=additions), self.assertRaises(ValueError):
                derive_locale_codes.extend_table(qt, iana, additions)
        with self.assertRaises(ValueError):
            derive_locale_codes.extend_table(qt, iana + '%%\nType: language\nSubtag: xyz\n', ['xyz,,,'])

    def test_derive_qt_locale_cli_requires_paired_registry_and_explicit_mapping(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root/'qtbase/src/corelib/text/qlocale_data_p.h'
            source.parent.mkdir(parents=True)
            source.write_text('''constexpr std::array<LanguageCodeEntry, 1> languageCodeList {
    LanguageCodeEntry {{'e', 'n'}, {'e', 'n', 'g'}, {'e', 'n', 'g'}, {'e', 'n', 'g'}}, // English
};
''', encoding='utf-8')
            registry = root/'iana.txt'
            registry.write_text(iana_loc_fixture()[0], encoding='utf-8')
            output = root/'build/codes.tsv'
            argv = ['derive', '--qtbase', str(root/'qtbase'), '--output', str(output),
                    '--iana', str(registry), '--add-language', 'xyz,xyz,xyz,']
            with patch.object(derive_locale_codes, 'ROOT', root), patch.object(sys, 'argv', argv), \
                    redirect_stdout(io.StringIO()) as out:
                derive_locale_codes.main()
                report = json.loads(out.getvalue())
                self.assertFalse(report['iso_field_mapping_verified'])
                self.assertEqual(report['explicit_additions'], 1)
                before = output.read_bytes()
                with self.assertRaises(FileExistsError):
                    derive_locale_codes.main()
                self.assertEqual(output.read_bytes(), before)
            with patch.object(derive_locale_codes, 'ROOT', root), \
                    patch.object(sys, 'argv', argv[:-2]), patch('sys.stderr', new_callable=io.StringIO):
                with self.assertRaises(SystemExit):
                    derive_locale_codes.main()

    def test_derive_qt_locale_codes_rejects_missing_malformed_and_conflicting_rows(self):
        entry = "    LanguageCodeEntry {{}, {'u', 'n', 'd'}, {'u', 'n', 'd'}, {'u', 'n', 'd'}}, // C\n"
        prefix = 'constexpr std::array<LanguageCodeEntry, 2> languageCodeList {\n'
        conflict = "    LanguageCodeEntry {{}, {'u', 'n', 'd'}, {'a', 'a', 'a'}, {'a', 'a', 'a'}}, // other\n"
        for fixture in ('no table', prefix + entry + '};\n',
                        prefix + entry + entry.replace("'d'", "'D'", 1) + '};\n',
                        prefix + entry + conflict + '};\n'):
            with self.subTest(fixture=fixture), self.assertRaises(ValueError):
                derive_locale_codes.derived_table(fixture)

    def test_derive_qt_locale_cli_stays_in_build_and_never_overwrites(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root/'qtbase/src/corelib/text/qlocale_data_p.h'
            source.parent.mkdir(parents=True)
            source.write_text("constexpr std::array<LanguageCodeEntry, 1> languageCodeList {\n"
                              "    LanguageCodeEntry {{}, {'u', 'n', 'd'}, {'u', 'n', 'd'}, {'u', 'n', 'd'}}, // C\n"
                              "};\n", encoding='utf-8')
            output = root/'build/codes.tsv'
            with patch.object(derive_locale_codes, 'ROOT', root), \
                    patch.object(sys, 'argv', ['derive', '--qtbase', str(root/'qtbase'), '--output', str(output)]), \
                    redirect_stdout(io.StringIO()):
                derive_locale_codes.main()
                expected = output.read_bytes()
                with self.assertRaises(FileExistsError):
                    derive_locale_codes.main()
                self.assertEqual(output.read_bytes(), expected)
            with patch.object(derive_locale_codes, 'ROOT', root), \
                    patch.object(sys, 'argv', ['derive', '--qtbase', str(root/'qtbase'), '--output', str(root/'outside.tsv')]), \
                    patch('sys.stderr', new_callable=io.StringIO):
                with self.assertRaises(SystemExit):
                    derive_locale_codes.main()
            self.assertFalse((root/'outside.tsv').exists())

    def test_qt_psl_runtime_notice_delivers_source_and_exact_license(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary); _, output = psl_fixture(root)
            (output/'license-texts').mkdir()
            license_data = (root/'third_party/qt-public-suffix-list/PSL-LICENSE.txt').read_bytes()
            (output/'license-texts/MPL-2.0.txt').write_bytes(license_data)
            record = qt_psl_source.collect_runtime_source(output, root)
            self.assertEqual((output/record['file']).read_bytes(), b'owned.example\n')
            notice = (output/record['notice']).read_text(encoding='utf-8')
            self.assertIn(record['source_url'], notice)
            self.assertIn('license-texts/MPL-2.0.txt', notice)
            self.assertIn(record['sha256'], notice)
            self.assertEqual(qt_psl_source.collect_runtime_source(output, root), record)
            (output/record['notice']).write_bytes(b'changed existing notice')
            with self.assertRaisesRegex(ValueError, 'destination differs'):
                qt_psl_source.collect_runtime_source(output, root)
            self.assertEqual((output/record['notice']).read_bytes(), b'changed existing notice')

    def test_qt_psl_runtime_notice_rejects_changed_source_or_license(self):
        for changed in ['source', 'bundled_license', 'input_license']:
            with self.subTest(changed=changed), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary); _, output = psl_fixture(root)
                (output/'license-texts').mkdir()
                license_path = root/'third_party/qt-public-suffix-list/PSL-LICENSE.txt'
                (output/'license-texts/MPL-2.0.txt').write_bytes(license_path.read_bytes())
                target = {'source': root/'third_party/qt-public-suffix-list/public_suffix_list.dat',
                          'bundled_license': output/'license-texts/MPL-2.0.txt',
                          'input_license': license_path}[changed]
                target.write_bytes(b'altered')
                with self.assertRaisesRegex(ValueError, 'mismatch'):
                    qt_psl_source.collect_runtime_source(output, root)
                self.assertFalse((output/'public_suffix_list.dat').exists())
                self.assertFalse((output/'public-suffix-list-SOURCE.md').exists())

    def test_qt_psl_collects_only_after_regenerating_both_forms(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary); archive, output = psl_fixture(root)
            record = qt_psl_source.collect(output, archive, root)
            self.assertEqual(record['regenerated'], ['psl_data.cpp', 'public_suffix_list.dafsa'])
            self.assertEqual((output/record['file']).read_bytes(), b'owned.example\n')
            self.assertEqual({path.name for path in (output/'qt-public-suffix-list').iterdir()},
                             {'public_suffix_list.dat', 'PSL-LICENSE.txt', 'README.md', 'source.json'})

    def test_qt_psl_rejects_modified_inputs_and_archive_before_delivery(self):
        for altered in ['list', 'license', 'archive', 'git_blob']:
            with self.subTest(altered=altered), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary); archive, output = psl_fixture(root)
                if altered in ['list', 'license']:
                    name = 'public_suffix_list.dat' if altered == 'list' else 'PSL-LICENSE.txt'
                    (root/'third_party/qt-public-suffix-list'/name).write_bytes(b'altered')
                elif altered == 'archive':
                    archive.write_bytes(b'altered')
                else:
                    path = root/'scripts/qt-psl-source.json'
                    config = json.loads(path.read_text(encoding='utf-8'))
                    config['upstream_git_blob'] = 'b'*40
                    path.write_text(json.dumps(config), encoding='utf-8')
                with self.assertRaisesRegex(ValueError, 'mismatch'):
                    qt_psl_source.collect(output, archive, root)
                self.assertFalse((output/'qt-public-suffix-list').exists())

    def test_qt_psl_rejects_unreproducible_qt_generated_data(self):
        for wrong in ['cpp', 'binary']:
            with self.subTest(wrong=wrong), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary); archive, output = psl_fixture(root, wrong)
                with self.assertRaisesRegex(ValueError, 'regenerated .* differs'):
                    qt_psl_source.collect(output, archive, root)
                self.assertFalse((output/'qt-public-suffix-list').exists())

    def test_qt_psl_never_overwrites_existing_delivery(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary); archive, output = psl_fixture(root)
            original = output/'qt-public-suffix-list'; original.mkdir()
            (original/'owned').write_bytes(b'keep')
            with self.assertRaisesRegex(ValueError, 'destination unavailable'):
                qt_psl_source.collect(output, archive, root)
            self.assertEqual((original/'owned').read_bytes(), b'keep')

    @unittest.skipUnless(os.environ.get('XDZIP_VENDOR_INTEGRATION') == '1', 'Explicit real Cargo vendor check only')
    def test_vendor_config_real_cargo(self):
        base = ROOT/'build'
        base.mkdir(exist_ok=True)
        with tempfile.TemporaryDirectory(dir=base, prefix='vendor-test-') as temporary:
            self.assertTrue(within(Path(temporary), base))
            output = Path(temporary)/'交付 with spaces'
            output.mkdir()
            self.assertTrue(within(output, base))
            source.write_vendor_config(output)
            config = (output/'cargo-config.toml').read_text(encoding='utf-8')
            self.assertIn('directory = "vendor"', config)
            self.assertNotIn(str(output), config)
            self.assertTrue(any(path.is_dir() for path in (output/'vendor').iterdir()))
            before = {path.relative_to(output/'vendor').as_posix(): digest(path)
                      for path in (output/'vendor').rglob('*') if path.is_file()}
            record = vendor_notice_materials.collect(output/'vendor', output/'rust-vendor-notices')
            self.assertEqual({item['name'] for item in record['packages']},
                             {'r-efi', 'wasip3', 'wasm-encoder', 'wasm-metadata', 'wasmparser',
                              'wit-component', 'wit-parser'})
            self.assertEqual(before, {path.relative_to(output/'vendor').as_posix(): digest(path)
                                     for path in (output/'vendor').rglob('*') if path.is_file()})
            self.assertTrue((output/'rust-vendor-notices/index.json').is_file())

    def test_vendor_config_uses_relative_output_even_with_spaces(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)/'workspace 中文 with spaces'; root.mkdir()
            output = root/'delivery 路径 with spaces'; output.mkdir()
            config = ('[source.crates-io]\nreplace-with = "vendored-sources"\n\n'
                      '[source.vendored-sources]\ndirectory = "vendor"\n')
            completed = subprocess.CompletedProcess([], 0, stdout=config, stderr='')
            with patch.object(source, 'ROOT', root), patch.object(source.subprocess, 'run', return_value=completed) as run:
                source.write_vendor_config(output)
            args, kwargs = run.call_args
            self.assertEqual(args[0], ['cargo', 'vendor', '--locked', '--versioned-dirs',
                                       '--manifest-path', str(root/'Cargo.toml'), 'vendor'])
            self.assertEqual(kwargs['cwd'], output)
            self.assertEqual((output/'cargo-config.toml').read_text(encoding='utf-8'), config)
            self.assertNotIn(str(root), config)

    def test_vendor_config_rejects_absolute_ambiguous_or_missing_paths(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            template = ('[source.crates-io]\nreplace-with = "vendored-sources"\n\n'
                        '[source.vendored-sources]\ndirectory = "vendor"\n')
            invalid = [template.replace('"vendor"', '"C:/host/build/vendor"'),
                       template.replace('"vendor"', '"C:\\\\host\\\\build\\\\vendor"'),
                       template.replace('"vendor"', '"/host/build/vendor"'),
                       template.replace('"vendor"', '"../vendor"'),
                       template.replace('directory = "vendor"\n', ''),
                       template + 'directory = "vendor"\n',
                       template + '[source.other]\nreplace-with = "vendored-sources"\n']
            for index, value in enumerate(invalid):
                output = root/str(index); output.mkdir()
                result = subprocess.CompletedProcess([], 0, stdout=value, stderr='')
                with self.subTest(index=index), patch.object(source.subprocess, 'run', return_value=result):
                    with self.assertRaisesRegex(ValueError, 'non-relocatable'):
                        source.write_vendor_config(output)
                self.assertFalse((output/'cargo-config.toml').exists())

    def test_native_source_delivery_retains_verified_icu_source_and_license(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary); cache = root/'cache'; output = root/'delivery'
            cache.mkdir(); output.mkdir(); (root/'scripts').mkdir()
            data = b'Owned ICU license fixture'
            item = {'file': 'ICU.txt', 'version': '73.2', 'sha256': hashlib.sha256(data).hexdigest(),
                    'url': 'https://invalid.test/not-used'}
            (cache/'ICU.txt').write_bytes(data)
            archive = cache/'icu-fixture.tgz'
            self.write_icu_fixture(archive, data)
            source_item = {'file': archive.name, 'version': '73.2',
                           'sha256': digest(archive), 'url': 'https://invalid.test/not-used'}
            config = {'icu': item, 'icu_source': source_item}
            (root/'scripts/native-runtime-sources.json').write_text(json.dumps(config))
            with patch.object(source, 'ROOT', root), patch.object(source, 'configuration', return_value=config):
                material = source.collect_native_sources(output, cache)
            self.assertEqual(material, [item, source_item])
            self.assertEqual((output/'ICU.txt').read_bytes(), data)
            self.assertEqual((output/archive.name).read_bytes(), archive.read_bytes())
            self.assertEqual({path.name for path in output.iterdir()},
                             {'ICU.txt', archive.name, 'native-runtime-sources.json'})

    @staticmethod
    def write_icu_fixture(path, license_text, version='73.2', extra=None, include_version=True):
        files = {'icu/LICENSE': license_text}
        if include_version:
            files['icu/source/common/unicode/uvernum.h'] = f'/* © ICU fixture */\n#define U_ICU_VERSION "{version}"\n'.encode()
        files.update(extra or {})
        with tarfile.open(path, 'w:gz') as archive:
            for name, contents in files.items():
                member = tarfile.TarInfo(name)
                member.size = len(contents)
                archive.addfile(member, io.BytesIO(contents))

    def test_native_source_delivery_rejects_invalid_icu_archive(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary); cache = root/'cache'; output = root/'delivery'
            cache.mkdir(); output.mkdir(); (root/'scripts').mkdir()
            license_text = b'Owned ICU license fixture'
            license_item = {'file': 'ICU.txt', 'version': '73.2',
                            'sha256': hashlib.sha256(license_text).hexdigest(),
                            'url': (root/'missing-license').as_uri()}
            (cache/'ICU.txt').write_bytes(license_text)
            archive = cache/'icu-fixture.tgz'
            cases = [
                ('wrong license', {'license_text': b'other license'}),
                ('wrong version', {'license_text': license_text, 'version': '74.1'}),
                ('missing version', {'license_text': license_text, 'include_version': False}),
                ('escaping member', {'license_text': license_text, 'extra': {'icu/../escape': b'x'}}),
            ]
            for name, options in cases:
                with self.subTest(name=name):
                    self.write_icu_fixture(archive, **options)
                    source_item = {'file': archive.name, 'version': '73.2',
                                   'sha256': digest(archive), 'url': (root/'missing-source').as_uri()}
                    config = {'icu': license_item, 'icu_source': source_item}
                    (root/'scripts/native-runtime-sources.json').write_text(json.dumps(config))
                    with patch.object(source, 'ROOT', root), patch.object(source, 'configuration', return_value=config):
                        with self.assertRaises(ValueError):
                            source.collect_native_sources(output, cache)
                    self.assertEqual(list(output.iterdir()), [])
            self.write_icu_fixture(archive, license_text)
            source_item['sha256'] = '0'*64
            with patch.object(source, 'ROOT', root), patch.object(source, 'configuration', return_value=config):
                with self.assertRaisesRegex(ValueError, 'checksum mismatch'):
                    source.collect_native_sources(output, cache)
            archive.unlink()
            with patch.object(source, 'ROOT', root), patch.object(source, 'configuration', return_value=config):
                with self.assertRaises(OSError):
                    source.collect_native_sources(output, cache)

    @staticmethod
    def data_source_fixture(cache, module, root_name, data, extra=None):
        path = cache/(root_name+'.tar.gz')
        members = {root_name+'/data.txt': data}
        members.update(extra or {})
        with tarfile.open(path, 'w:gz') as archive:
            for name, contents in members.items():
                member = tarfile.TarInfo(name)
                member.size = len(contents)
                archive.addfile(member, io.BytesIO(contents))
        return {'module': module, 'file': path.name, 'url': path.as_uri(),
                'sha256': digest(path), 'archive_root': root_name,
                'anchors': {'data.txt': hashlib.sha256(data).hexdigest()}}

    @staticmethod
    def cldr_data_fixture(cache, extra=None, with_standard=True, with_screenshots=True):
        members = {'cldr-owned/'+name: b'owned ISO fixture' for name in source.ISO639_TABLES}
        if with_screenshots:
            members.update({'cldr-owned/'+name: b'owned screenshot fixture' for name in source.CLDR_DOC_SCREENSHOTS})
        if with_standard:
            members.update({'cldr-owned/'+name: b'owned standard fixture' for name in source.CLDR_STANDARD_PARTS})
            members['cldr-owned/docs/ldml/images/owned.png'] = b'owned standard image'
        members['cldr-owned/'+cldr_webapp_notices.REPLACEMENT_PATH] = b'owned original image'
        members.update(extra or {})
        return DeliveryTests.data_source_fixture(cache, 'cldr-data-source', 'cldr-owned',
                                                  b'CLDR data', members)

    @staticmethod
    def cldr_replacement_fixture():
        data = b'owned replacement image'
        return {'emoji_1f699.png': {'original_sha256': hashlib.sha256(b'owned original image').hexdigest(),
                'sha256': hashlib.sha256(data).hexdigest(), 'data': data, 'source': 'noto',
                'upstream_path': cldr_webapp_notices.REPLACEMENT_UPSTREAM}}

    @staticmethod
    def qt_cldr_fixture(root, extra=None, qt_marker=b'Common Locale Data Repository v46'):
        cache = root/'cache'; cache.mkdir()
        output = root/'delivery'; output.mkdir()
        (root/'scripts').mkdir()
        qtbase = root/'qtbase-everywhere-src-fixture.tar.xz'
        prefix = qtbase.name.removesuffix('.tar.xz')
        with tarfile.open(qtbase, 'w:xz') as archive:
            for relative in ('src/corelib/text/qlocale_data_p.h', 'src/corelib/time/qtimezoneprivate_data_p.h'):
                member = tarfile.TarInfo(prefix+'/'+relative); member.size = len(qt_marker)
                archive.addfile(member, io.BytesIO(qt_marker))
        commit = 'a'*40
        archive_root = 'cldr-'+commit
        anchors = {'LICENSE': b'owned license', 'common/main/root.xml': b'root',
                   'common/main/en.xml': b'english', 'common/supplemental/windowsZones.xml': b'zones',
                   'common/bcp47/timezone.xml': b'timezones'}
        members = {archive_root+'/'+name: data for name, data in anchors.items()}
        members[archive_root+'/tools/iso-639-3.tab'] = b'owned restricted fixture'
        members.update(extra or {})
        upstream = cache/'cldr-fixture.tar.gz'
        with tarfile.open(upstream, 'w:gz') as archive:
            for name, data in members.items():
                member = tarfile.TarInfo(name); member.size = len(data)
                archive.addfile(member, io.BytesIO(data))
        item = {'version': '46', 'source_commit': commit, 'common_tree': 'b'*40,
                'archive_root': archive_root, 'file': upstream.name, 'url': upstream.as_uri(),
                'sha256': digest(upstream),
                'anchors': {name: hashlib.sha256(data).hexdigest() for name, data in anchors.items()}}
        config = {'schema_version': 1, 'qt_version': '6.8.2', 'qtbase_archive': qtbase.name,
                  'qtbase_archive_sha256': digest(qtbase), 'cldr': item}
        (root/'scripts/qt-cldr-sources.json').write_text(json.dumps(config), encoding='utf-8')
        return qtbase, output, cache, config

    def test_qt_cldr_delivery_retains_common_and_license_but_not_iso_table(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary); qtbase, output, cache, config = self.qt_cldr_fixture(root)
            with patch.object(source, 'ROOT', root):
                record = source.collect_qt_cldr_source(output, qtbase, cache)
            self.assertEqual(record['upstream_archive_sha256'], config['cldr']['sha256'])
            self.assertFalse(record['iso_639_3_table_included'])
            self.assertEqual(record['identified_language_identifier_files'], [])
            self.assertFalse(record['identifier_file_list_exhaustive'])
            self.assertEqual(record['scope'], 'common and LICENSE only')
            self.assertEqual(record['sha256'], digest(output/record['file']))
            with tarfile.open(output/record['file'], 'r:gz') as archive:
                self.assertEqual(set(archive.getnames()),
                                 {config['cldr']['archive_root']+'/'+name for name in config['cldr']['anchors']})
                self.assertEqual(archive.extractfile(config['cldr']['archive_root']+'/common/main/en.xml').read(),
                                 b'english')
            self.assertFalse((output/config['cldr']['file']).exists())
            self.assertEqual(json.loads((output/'qt-cldr-sources.json').read_text()), config)

    def test_qt_cldr_manifest_identifies_retained_language_data(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            prefix = 'cldr-'+'a'*40+'/'
            extra = {prefix+name: b'owned language fixture' for name in source.CLDR_IDENTIFIER_FILES}
            extra[prefix+source.CLDR_LANGUAGE_REGISTRY] = b'outside common fixture'
            qtbase, output, cache, config = self.qt_cldr_fixture(root, extra)
            with patch.object(source, 'ROOT', root):
                record = source.collect_qt_cldr_source(output, qtbase, cache)
            self.assertEqual(record['identified_language_identifier_files'], list(source.CLDR_IDENTIFIER_FILES))
            self.assertFalse(record['identifier_file_list_exhaustive'])
            with tarfile.open(output/record['file'], 'r:gz') as archive:
                for name in source.CLDR_IDENTIFIER_FILES:
                    self.assertEqual(archive.extractfile(prefix+name).read(), b'owned language fixture')
                self.assertNotIn(prefix+source.CLDR_LANGUAGE_REGISTRY, archive.getnames())

    def test_qt_cldr_delivery_rejects_wrong_or_unsafe_inputs(self):
        for case in ('qt version', 'qt hash', 'source hash', 'anchor', 'escaping member', 'iso in common'):
            with self.subTest(case=case), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                extra = ({'cldr-'+'a'*40+'/../escape': b'x'} if case == 'escaping member' else
                         {'cldr-'+'a'*40+'/common/iso-639-3.tab': b'x'} if case == 'iso in common' else None)
                qtbase, output, cache, config = self.qt_cldr_fixture(
                    root, extra, b'Common Locale Data Repository v45' if case == 'qt version' else
                    b'Common Locale Data Repository v46')
                if case == 'qt hash': config['qtbase_archive_sha256'] = '0'*64
                if case == 'source hash': config['cldr']['sha256'] = '0'*64
                if case == 'anchor': config['cldr']['anchors']['common/main/en.xml'] = '0'*64
                (root/'scripts/qt-cldr-sources.json').write_text(json.dumps(config), encoding='utf-8')
                with patch.object(source, 'ROOT', root), self.assertRaises(ValueError):
                    source.collect_qt_cldr_source(output, qtbase, cache)
                self.assertEqual(list(output.iterdir()), [])

    def test_editable_icu_and_cldr_data_sources_are_collected(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary); cache = root/'cache'; output = root/'delivery'
            cache.mkdir(); output.mkdir()
            items = [self.data_source_fixture(cache, 'icu-data-source', 'icu-owned', b'ICU data'),
                     self.cldr_data_fixture(cache),
                     self.data_source_fixture(cache, 'icu-cldr-bootstrap-source', 'bootstrap-owned',
                                              b'ICU4J bootstrap')]
            with patch.object(source, 'configuration', return_value={'icu_data_sources': items}), \
                 patch.object(source, 'cldr_emoji_replacements', return_value=self.cldr_replacement_fixture()):
                records = source.collect_icu_data_sources(output, cache)
            self.assertEqual(records[0], items[0])
            self.assertEqual((output/items[0]['file']).read_bytes(), (cache/items[0]['file']).read_bytes())
            self.assertEqual(records[1]['upstream_archive_sha256'], items[1]['sha256'])
            self.assertEqual(records[1]['upstream_url'], items[1]['url'])
            self.assertEqual(records[2], items[2])
            self.assertEqual((output/items[2]['file']).read_bytes(), (cache/items[2]['file']).read_bytes())
            self.assertNotIn('url', records[1])
            self.assertFalse(records[1]['complete_upstream_tree'])
            self.assertEqual(set(records[1]['omitted_files']),
                             source.ISO639_TABLES | source.CLDR_STANDARD_PARTS | source.CLDR_DOC_SCREENSHOTS |
                             {'docs/ldml/images/owned.png'})
            self.assertTrue(records[1]['file'].endswith('-without-iso6393-and-uts35.tar.gz'))
            self.assertEqual(records[1]['substituted_files'][0]['cldr_file'], cldr_webapp_notices.REPLACEMENT_PATH)
            self.assertEqual(records[1]['identified_language_identifier_files'], [])
            self.assertFalse(records[1]['identifier_file_list_exhaustive'])
            with tarfile.open(output/records[1]['file'], 'r:gz') as archive:
                self.assertEqual(set(archive.getnames()), {'cldr-owned/data.txt',
                                 'cldr-owned/'+cldr_webapp_notices.REPLACEMENT_PATH})
                self.assertEqual(archive.extractfile('cldr-owned/data.txt').read(), b'CLDR data')
                self.assertEqual(archive.extractfile('cldr-owned/'+cldr_webapp_notices.REPLACEMENT_PATH).read(),
                                 b'owned replacement image')
            second = root/'second-delivery'; second.mkdir()
            with patch.object(source, 'configuration', return_value={'icu_data_sources': items}), \
                 patch.object(source, 'cldr_emoji_replacements', return_value=self.cldr_replacement_fixture()):
                second_records = source.collect_icu_data_sources(second, cache)
            self.assertEqual(records[1]['sha256'], second_records[1]['sha256'])

    def test_cldr_subset_requires_the_pinned_technical_standard_parts(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary); cache = root/'cache'; output = root/'delivery'
            cache.mkdir(); output.mkdir()
            items = [self.data_source_fixture(cache, 'icu-data-source', 'icu-owned', b'ICU data'),
                     self.cldr_data_fixture(cache, with_standard=False),
                     self.data_source_fixture(cache, 'icu-cldr-bootstrap-source', 'bootstrap-owned', b'bootstrap')]
            with patch.object(source, 'configuration', return_value={'icu_data_sources': items}), \
                 patch.object(source, 'cldr_emoji_replacements', return_value=self.cldr_replacement_fixture()), \
                 self.assertRaisesRegex(ValueError, 'Missing CLDR technical standard parts'):
                source.collect_icu_data_sources(output, cache)
            self.assertEqual(list(output.iterdir()), [])

    def test_cldr_subset_preserves_paths_next_to_the_standard_directory(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary); cache = root/'cache'; output = root/'delivery'
            cache.mkdir(); output.mkdir()
            adjacent = 'docs/ldml-other/owned.txt'
            items = [self.data_source_fixture(cache, 'icu-data-source', 'icu-owned', b'ICU data'),
                     self.cldr_data_fixture(cache, {'cldr-owned/'+adjacent: b'keep'}),
                     self.data_source_fixture(cache, 'icu-cldr-bootstrap-source', 'bootstrap-owned', b'bootstrap')]
            with patch.object(source, 'configuration', return_value={'icu_data_sources': items}), \
                 patch.object(source, 'cldr_emoji_replacements', return_value=self.cldr_replacement_fixture()):
                record = source.collect_icu_data_sources(output, cache)[1]
            with tarfile.open(output/record['file'], 'r:gz') as archive:
                self.assertEqual(archive.extractfile('cldr-owned/'+adjacent).read(), b'keep')
                self.assertFalse(any(name.startswith('cldr-owned/docs/ldml/') for name in archive.getnames()))
            self.assertNotIn(adjacent, record['omitted_files'])

    def test_cldr_subset_requires_expected_documentation_screenshots(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary); cache = root/'cache'; output = root/'delivery'
            cache.mkdir(); output.mkdir()
            items = [self.data_source_fixture(cache, 'icu-data-source', 'icu-owned', b'ICU data'),
                     self.cldr_data_fixture(cache, with_screenshots=False),
                     self.data_source_fixture(cache, 'icu-cldr-bootstrap-source', 'bootstrap-owned', b'bootstrap')]
            with patch.object(source, 'configuration', return_value={'icu_data_sources': items}), \
                 patch.object(source, 'cldr_emoji_replacements', return_value=self.cldr_replacement_fixture()), \
                 self.assertRaisesRegex(ValueError, 'Missing expected CLDR documentation screenshots'):
                source.collect_icu_data_sources(output, cache)
            self.assertEqual(list(output.iterdir()), [])

    def test_icu_cldr_manifest_identifies_retained_language_data_and_registry(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary); cache = root/'cache'; output = root/'delivery'
            cache.mkdir(); output.mkdir()
            retained = (*source.CLDR_IDENTIFIER_FILES, source.CLDR_LANGUAGE_REGISTRY)
            extra = {'cldr-owned/'+name: b'owned language fixture' for name in retained}
            items = [self.data_source_fixture(cache, 'icu-data-source', 'icu-owned', b'ICU data'),
                     self.cldr_data_fixture(cache, extra),
                     self.data_source_fixture(cache, 'icu-cldr-bootstrap-source', 'bootstrap-owned',
                                              b'ICU4J bootstrap')]
            with patch.object(source, 'configuration', return_value={'icu_data_sources': items}), \
                 patch.object(source, 'cldr_emoji_replacements', return_value=self.cldr_replacement_fixture()):
                record = source.collect_icu_data_sources(output, cache)[1]
            self.assertEqual(record['identified_language_identifier_files'], list(retained))
            self.assertFalse(record['identifier_file_list_exhaustive'])
            with tarfile.open(output/record['file'], 'r:gz') as archive:
                for name in retained:
                    self.assertEqual(archive.extractfile('cldr-owned/'+name).read(), b'owned language fixture')
                for name in source.ISO639_TABLES:
                    self.assertNotIn('cldr-owned/'+name, archive.getnames())

    @staticmethod
    def cldr_web_fonts_fixture(root, case=None):
        inputs = root/'third_party'/cldr_webapp_notices.DIRECTORY
        inputs.mkdir(parents=True)
        fonts = {name: ('owned font fixture '+name).encode() for name in cldr_webapp_notices.FONT_FILES}
        notices = {name: ('owned notice fixture '+name).encode() for name in cldr_webapp_notices.NOTICE_FILES}
        bootstrap = b'owned Bootstrap fixture'
        autosize = b'owned Autosize fixture'
        ibm_sources = {name: ('owned IBM web source '+name).encode()
                       for name in cldr_webapp_notices.IBM_WEB_SOURCES}
        cldr_licenses = {name: ('owned CLDR license fixture '+name).encode()
                         for name in cldr_webapp_notices.CLDR_LICENSE_SOURCES}
        web_images = {name: ('owned static image '+name).encode()
                      for name in cldr_webapp_notices.CLDR_WEB_IMAGE_FILES}
        emoji = {'emoji_1f600.png': b'owned emoji fixture', 'emoji_1f31a.png': b'another emoji fixture'}
        emoji_2021 = {'emoji_1f601.png': b'older emoji fixture'}
        replacement = b'\x89PNG\r\n\x1a\nowned replacement fixture'
        bundle_buffer = io.BytesIO()
        with zipfile.ZipFile(bundle_buffer, 'w') as bundle:
            member = zipfile.ZipInfo('emoji_1f699.png')
            member.external_attr = 0o100644 << 16
            bundle.writestr(member, replacement)
        bundle = bundle_buffer.getvalue()
        config = {'schema_version': 1, 'kind': cldr_webapp_notices.DIRECTORY,
                  'archive_root': cldr_webapp_notices.ARCHIVE_ROOT,
                  'fonts_directory': cldr_webapp_notices.FONTS_DIRECTORY,
                  'bootstrap_script': {'file': cldr_webapp_notices.BOOTSTRAP_SCRIPT,
                                       'sha256': hashlib.sha256(bootstrap).hexdigest()},
                  'autosize_script': {'file': cldr_webapp_notices.AUTOSIZE_SCRIPT,
                                      'sha256': hashlib.sha256(autosize).hexdigest()},
                  'ibm_web_sources': {name: hashlib.sha256(data).hexdigest() for name, data in ibm_sources.items()},
                  'cldr_license_sources': {name: hashlib.sha256(data).hexdigest() for name, data in cldr_licenses.items()},
                  'cldr_web_images': {'directory': cldr_webapp_notices.WEBAPP_DIRECTORY,
                                      'upstream_commit': cldr_webapp_notices.CLDR_WEB_IMAGES_COMMIT,
                                      'license_notice': 'CLDR-2026-LICENSE.txt',
                                      'files': {name: {'sha256': hashlib.sha256(data).hexdigest(),
                                                       'git_blob': hashlib.sha1(b'blob '+str(len(data)).encode()+b'\0'+data).hexdigest()}
                                                for name, data in web_images.items()}},
                  'noto_emoji': {'directory': cldr_webapp_notices.EMOJI_DIRECTORY,
                                 'upstream_commit': cldr_webapp_notices.NOTO_EMOJI_COMMIT,
                                 'files': {name: hashlib.sha256(data).hexdigest() for name, data in emoji.items()},
                                 'upstream_commit_2021': cldr_webapp_notices.NOTO_EMOJI_2021_COMMIT,
                                 'files_2021': {name: hashlib.sha256(data).hexdigest() for name, data in emoji_2021.items()}},
                  'emoji_replacements': {'file': cldr_webapp_notices.REPLACEMENT_FILE,
                                         'sha256': hashlib.sha256(bundle).hexdigest(),
                                         'upstreams': cldr_webapp_notices.REPLACEMENT_UPSTREAMS,
                                         'files': {'emoji_1f699.png': {
                                             'original_sha256': hashlib.sha256(b'owned original fixture').hexdigest(),
                                             'sha256': hashlib.sha256(replacement).hexdigest(), 'source': 'noto',
                                             'upstream_path': cldr_webapp_notices.REPLACEMENT_UPSTREAM}}},
                  'fonts': {name: hashlib.sha256(data).hexdigest() for name, data in fonts.items()},
                  'notices': {name: hashlib.sha256(data).hexdigest() for name, data in notices.items()}}
        if case == 'emoji-2021-overlap':
            config['noto_emoji']['files_2021']['emoji_1f600.png'] = hashlib.sha256(emoji['emoji_1f600.png']).hexdigest()
        if case == 'emoji-2021-commit':
            config['noto_emoji']['upstream_commit_2021'] = '0'*40
        if case == 'replacement-upstream':
            config['emoji_replacements']['upstreams'] = {'noto': {'commit': '0'*40}}
        if case == 'web-image-upstream':
            config['cldr_web_images']['upstream_commit'] = '0'*40
        if case == 'web-image-blob':
            config['cldr_web_images']['files']['warn.png']['git_blob'] = '0'*40
        (inputs/'sources.json').write_text(json.dumps(config), encoding='utf-8')
        (inputs/'README.md').write_text('owned fixture', encoding='utf-8')
        (inputs/cldr_webapp_notices.REPLACEMENT_FILE).write_bytes(
            b'tampered' if case == 'replacement-asset' else bundle)
        for name, data in notices.items():
            (inputs/name).write_bytes(b'tampered' if case == 'notice' or
                                     (case == 'autosize-notice' and name == 'Autosize-MIT.txt') or
                                     (case == 'web-image-notice' and name == 'CLDR-2026-LICENSE.txt') or
                                     (case == 'emoji-notice' and name == 'Noto-Emoji-Apache-2.0.txt') else data)
        archive = root/'subset.tar.gz'
        prefix = cldr_webapp_notices.ARCHIVE_ROOT+'/'+cldr_webapp_notices.FONTS_DIRECTORY+'/'
        if case == 'extra':
            fonts['unreviewed.ttf'] = b'owned extra font'
        if case == 'missing':
            fonts.pop('glyphicons-halflings-regular.ttf')
        if case == 'font':
            fonts['glyphicons-halflings-regular.ttf'] = b'tampered'
        with tarfile.open(archive, 'w:gz') as tf:
            for name, data in web_images.items():
                if case == 'web-image-missing' and name == 'warn.png':
                    continue
                payload = b'tampered' if case == 'web-image-tamper' and name == 'warn.png' else data
                member = tarfile.TarInfo(cldr_webapp_notices.ARCHIVE_ROOT+'/'+cldr_webapp_notices.WEBAPP_DIRECTORY+'/'+name)
                if case == 'web-image-symlink' and name == 'warn.png':
                    member.type = tarfile.SYMTYPE; member.linkname = 'elsewhere'
                    tf.addfile(member)
                else:
                    member.size = len(payload)
                    tf.addfile(member, io.BytesIO(payload))
                    if case == 'web-image-duplicate' and name == 'warn.png':
                        tf.addfile(member, io.BytesIO(payload))
            if case == 'web-image-extra':
                payload = b'owned extra image'
                member = tarfile.TarInfo(cldr_webapp_notices.ARCHIVE_ROOT+'/'+cldr_webapp_notices.WEBAPP_DIRECTORY+'/extra.png')
                member.size = len(payload)
                tf.addfile(member, io.BytesIO(payload))
            for name, data in {**ibm_sources, **cldr_licenses}.items():
                if ((case == 'ibm-missing' and name.endswith('/browse_results.jsp')) or
                        (case == 'cldr-license-missing' and name == 'unicode-license.txt')):
                    continue
                payload = b'tampered' if ((case == 'ibm-tamper' and name.endswith('/browse_results.jsp')) or
                                          (case == 'cldr-readme-tamper' and name == 'README.md')) else data
                member = tarfile.TarInfo(cldr_webapp_notices.ARCHIVE_ROOT+'/'+name)
                if case == 'ibm-symlink' and name.endswith('/browse_results.jsp'):
                    member.type = tarfile.SYMTYPE; member.linkname = 'elsewhere'
                    tf.addfile(member)
                else:
                    member.size = len(payload)
                    tf.addfile(member, io.BytesIO(payload))
                    if case == 'ibm-duplicate' and name.endswith('/browse_results.jsp'):
                        tf.addfile(member, io.BytesIO(payload))
            for name, data in fonts.items():
                member = tarfile.TarInfo(prefix+name)
                if case == 'symlink' and name == 'glyphicons-halflings-regular.ttf':
                    member.type = tarfile.SYMTYPE; member.linkname = 'elsewhere'
                    tf.addfile(member)
                else:
                    member.size = len(data)
                    tf.addfile(member, io.BytesIO(data))
            if case != 'bootstrap-missing':
                payload = b'tampered' if case == 'bootstrap' else bootstrap
                member = tarfile.TarInfo(cldr_webapp_notices.ARCHIVE_ROOT+'/'+cldr_webapp_notices.BOOTSTRAP_SCRIPT)
                member.size = len(payload)
                tf.addfile(member, io.BytesIO(payload))
            if case != 'autosize-missing':
                payload = b'tampered' if case == 'autosize' else autosize
                member = tarfile.TarInfo(cldr_webapp_notices.ARCHIVE_ROOT+'/'+cldr_webapp_notices.AUTOSIZE_SCRIPT)
                if case == 'autosize-symlink':
                    member.type = tarfile.SYMTYPE; member.linkname = 'elsewhere'
                    tf.addfile(member)
                else:
                    member.size = len(payload)
                    tf.addfile(member, io.BytesIO(payload))
                if case == 'autosize-duplicate':
                    tf.addfile(member, io.BytesIO(payload))
            emoji_prefix = cldr_webapp_notices.ARCHIVE_ROOT+'/'+cldr_webapp_notices.EMOJI_DIRECTORY+'/'
            for name, data in (*emoji.items(), *emoji_2021.items()):
                if ((case == 'emoji-missing' and name == 'emoji_1f600.png') or
                        (case == 'emoji-2021-missing' and name == 'emoji_1f601.png')):
                    continue
                member = tarfile.TarInfo(emoji_prefix+name)
                if case == 'emoji-symlink' and name == 'emoji_1f600.png':
                    member.type = tarfile.SYMTYPE; member.linkname = 'elsewhere'
                    tf.addfile(member)
                else:
                    payload = b'tampered' if ((case == 'emoji-tamper' and name == 'emoji_1f600.png') or
                        (case == 'emoji-2021-tamper' and name == 'emoji_1f601.png')) else data
                    member.size = len(payload)
                    tf.addfile(member, io.BytesIO(payload))
                    if case == 'emoji-duplicate' and name == 'emoji_1f600.png':
                        tf.addfile(member, io.BytesIO(payload))
            if case == 'emoji-extra':
                member = tarfile.TarInfo(emoji_prefix+'emoji_123.png')
                member.size = len(b'unreviewed emoji')
                tf.addfile(member, io.BytesIO(b'unreviewed emoji'))
            if case != 'replacement-missing':
                payload = b'tampered' if case == 'replacement-output' else replacement
                member = tarfile.TarInfo(cldr_webapp_notices.ARCHIVE_ROOT+'/'+cldr_webapp_notices.REPLACEMENT_PATH)
                if case == 'replacement-symlink':
                    member.type = tarfile.SYMTYPE; member.linkname = 'elsewhere'
                    tf.addfile(member)
                else:
                    member.size = len(payload)
                    tf.addfile(member, io.BytesIO(payload))
                    if case == 'replacement-duplicate': tf.addfile(member, io.BytesIO(payload))
        return archive

    def test_cldr_web_font_notices_are_bound_to_subset_and_license_bytes(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary); archive = self.cldr_web_fonts_fixture(root)
            with patch.object(cldr_webapp_notices, 'ROOT', root):
                record = cldr_webapp_notices.collect(root/'delivery', archive)
            self.assertEqual(record['font_files'], 6)
            self.assertTrue(record['bootstrap_script'])
            self.assertTrue(record['autosize_script'])
            self.assertEqual(record['ibm_marked_sources'], 3)
            self.assertEqual(record['cldr_web_image_files'], 27)
            self.assertEqual(record['noto_emoji_files'], 3)
            self.assertEqual(record['substituted_emoji_files'], 1)
            destination = root/'delivery'/cldr_webapp_notices.DIRECTORY
            for name in ('README.md', 'sources.json', *cldr_webapp_notices.NOTICE_FILES,
                         cldr_webapp_notices.REPLACEMENT_FILE):
                self.assertEqual((destination/name).read_bytes(),
                                 (root/'third_party'/cldr_webapp_notices.DIRECTORY/name).read_bytes())

    def test_cldr_web_font_notices_reject_changed_or_unreviewed_inputs(self):
        for case in ('font', 'missing', 'extra', 'notice', 'symlink', 'bootstrap', 'bootstrap-missing',
                     'autosize', 'autosize-missing', 'autosize-symlink', 'autosize-duplicate', 'autosize-notice',
                     'ibm-missing', 'ibm-tamper', 'ibm-symlink', 'ibm-duplicate',
                     'web-image-missing', 'web-image-tamper', 'web-image-symlink', 'web-image-duplicate',
                     'web-image-extra', 'web-image-upstream', 'web-image-blob', 'web-image-notice',
                     'cldr-license-missing', 'cldr-readme-tamper',
                     'emoji-missing', 'emoji-tamper', 'emoji-symlink', 'emoji-duplicate', 'emoji-notice', 'emoji-extra',
                     'emoji-2021-missing', 'emoji-2021-tamper', 'emoji-2021-overlap', 'emoji-2021-commit',
                     'replacement-asset', 'replacement-output', 'replacement-missing', 'replacement-symlink',
                     'replacement-duplicate', 'replacement-upstream'):
            with self.subTest(case=case), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary); archive = self.cldr_web_fonts_fixture(root, case)
                with patch.object(cldr_webapp_notices, 'ROOT', root), self.assertRaises(ValueError):
                    cldr_webapp_notices.collect(root/'delivery', archive)
                self.assertFalse((root/'delivery'/cldr_webapp_notices.DIRECTORY).exists())

    def test_editable_data_sources_reject_missing_or_tampered_inputs(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary); cache = root/'cache'; output = root/'delivery'
            cache.mkdir(); output.mkdir()
            cases = ['missing module', 'wrong archive hash', 'wrong anchor', 'missing anchor',
                     'escaping member', 'unexpected ISO table', 'bootstrap hash', 'bootstrap anchor',
                     'bootstrap ISO table', 'replacement original', 'replacement missing']
            for case in cases:
                with self.subTest(case=case):
                    items = [self.data_source_fixture(cache, 'icu-data-source', 'icu-owned', b'ICU data'),
                             self.cldr_data_fixture(cache,
                                 {'cldr-owned/../escape': b'x'} if case == 'escaping member' else
                                 {'cldr-owned/unexpected/iso-639-3-extra.tab': b'x'}
                                 if case == 'unexpected ISO table' else None),
                             self.data_source_fixture(cache, 'icu-cldr-bootstrap-source', 'bootstrap-owned',
                                                      b'ICU4J bootstrap',
                                                      {'bootstrap-owned/iso-639-3.tab': b'x'}
                                                      if case == 'bootstrap ISO table' else None)]
                    if case == 'missing module': items.pop()
                    if case == 'wrong archive hash': items[1]['sha256'] = '0'*64
                    if case == 'wrong anchor': items[1]['anchors']['data.txt'] = '0'*64
                    if case == 'missing anchor': items[1]['anchors'] = {'absent.txt': '0'*64}
                    if case == 'bootstrap hash': items[2]['sha256'] = '0'*64
                    if case == 'bootstrap anchor': items[2]['anchors']['data.txt'] = '0'*64
                    replacement = self.cldr_replacement_fixture()
                    if case == 'replacement original': replacement['emoji_1f699.png']['original_sha256'] = '0'*64
                    if case == 'replacement missing': replacement['emoji_absent.png'] = replacement.pop('emoji_1f699.png')
                    with patch.object(source, 'configuration', return_value={'icu_data_sources': items}), \
                         patch.object(source, 'cldr_emoji_replacements', return_value=replacement):
                        with self.assertRaises(ValueError):
                            source.collect_icu_data_sources(output, cache)
                    self.assertEqual(list(output.iterdir()), [])

    def test_source_subset_replacement_rejects_missing_duplicate_and_nonfiles_without_partial_output(self):
        for case in ('missing', 'duplicate', 'symlink'):
            with self.subTest(case=case), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                original = root/'input.tar.gz'
                target = root/'output.tar.gz'
                with tarfile.open(original, 'w:gz') as archive:
                    member = tarfile.TarInfo('owned/image.png')
                    if case == 'symlink':
                        member.type = tarfile.SYMTYPE
                        member.linkname = 'elsewhere'
                        archive.addfile(member)
                    else:
                        member.size = 3
                        archive.addfile(member, io.BytesIO(b'old'))
                        if case == 'duplicate':
                            archive.addfile(member, io.BytesIO(b'old'))
                name = 'owned/missing.png' if case == 'missing' else 'owned/image.png'
                with self.assertRaises(ValueError):
                    source.filter_source_tar(original, target, lambda path: True, replacements={name: b'new'})
                self.assertFalse(target.exists())
                self.assertFalse(target.with_suffix(target.suffix+'.partial').exists())

    def test_emoji_replacement_bundle_checks_members_and_paths(self):
        data = b'\x89PNG\r\n\x1a\nowned image'
        for case in ('valid', 'duplicate', 'symlink', 'path', 'missing', 'wrong-hash', 'not-png',
                     'same-hash', 'upstream-path', 'upstream-version', 'wrong-image', 'noto-flag'):
            with self.subTest(case=case):
                name = ('../emoji_1f699.png' if case == 'path' else
                        'emoji_1f1e6_1f1e8.png' if case == 'noto-flag' else 'emoji_1f699.png')
                payload = b'not an image' if case == 'not-png' else data
                buffer = io.BytesIO()
                with zipfile.ZipFile(buffer, 'w') as archive:
                    member = zipfile.ZipInfo(name)
                    member.external_attr = (0o120777 if case == 'symlink' else 0o100644) << 16
                    archive.writestr(member, payload)
                    if case == 'duplicate':
                        with self.assertWarns(UserWarning):
                            archive.writestr(member, payload)
                bundle = buffer.getvalue()
                sha = hashlib.sha256(payload).hexdigest()
                record = {'original_sha256': sha if case == 'same-hash' else '0'*64,
                          'sha256': 'f'*64 if case == 'wrong-hash' else sha,
                          'source': 'noto', 'upstream_path': '../elsewhere' if case == 'upstream-path' else
                          'png/72/emoji_u1f600.png' if case == 'wrong-image' else
                          'png/72/emoji_u1f1e6_1f1e8.png' if case == 'noto-flag' else
                          cldr_webapp_notices.REPLACEMENT_UPSTREAM}
                listed = 'emoji_missing.png' if case == 'missing' else name
                config = {'emoji_replacements': {'file': cldr_webapp_notices.REPLACEMENT_FILE,
                          'sha256': hashlib.sha256(bundle).hexdigest(),
                          'upstreams': {} if case == 'upstream-version' else cldr_webapp_notices.REPLACEMENT_UPSTREAMS,
                          'files': {listed: record}}}
                if case == 'valid':
                    self.assertEqual(cldr_webapp_notices.emoji_replacements(config, bundle)[name]['data'], data)
                else:
                    with self.assertRaises(ValueError):
                        cldr_webapp_notices.emoji_replacements(config, bundle)

    def test_emoji_upstream_archive_verifier_checks_hashes_and_ambiguity(self):
        for case in ('valid', 'archive-hash', 'member-hash', 'missing', 'duplicate', 'symlink'):
            with self.subTest(case=case), tempfile.TemporaryDirectory() as temporary:
                archive_path = Path(temporary)/'owned.tar.gz'
                with tarfile.open(archive_path, 'w:gz') as archive:
                    member = tarfile.TarInfo('owned/image.png')
                    if case == 'symlink':
                        member.type = tarfile.SYMTYPE
                        member.linkname = 'outside'
                        archive.addfile(member)
                    else:
                        member.size = 3
                        archive.addfile(member, io.BytesIO(b'img'))
                        if case == 'duplicate': archive.addfile(member, io.BytesIO(b'img'))
                name = 'owned/missing.png' if case == 'missing' else 'owned/image.png'
                expected = {name: '0'*64 if case == 'member-hash' else hashlib.sha256(b'img').hexdigest()}
                archive_hash = '0'*64 if case == 'archive-hash' else verify_emoji_alternatives.digest(archive_path)
                if case == 'valid':
                    verify_emoji_alternatives.verify_archive(archive_path, archive_hash, expected)
                else:
                    with self.assertRaises(ValueError):
                        verify_emoji_alternatives.verify_archive(archive_path, archive_hash, expected)

    def test_rar_mapping_notice_bytes_match_in_notices_and_separate_delivery(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            results = []
            for output in [root/'notices', root/'delivery']:
                paths, record = source.collect_rar_mapping_materials(output)
                self.assertEqual(len(paths), 3)
                self.assertEqual(record['source_release'], 'UnRAR 7.30 beta1')
                self.assertEqual(record['source_license'], 'UnRAR')
                self.assertEqual(record['source_archive']['file'], 'unrarsrc-7.3.1.tar.gz')
                self.assertFalse(record['official_engine_in_product'])
                self.assertEqual(record['kind'], 'limited-reference-provenance')
                original = output/record['file']
                self.assertEqual(original.stat().st_size, 1976)
                self.assertEqual(digest(original), '6ecc1687808b7d66b24f874755abfed7464d9751ed0001cd4e8e5d9bf397ff8a')
                results.append({path.name: path.read_bytes() for path in paths})
            self.assertEqual(results[0], results[1])

    def test_rar_mapping_changed_original_notice_fails_before_output(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            inputs = root/'third_party/rar-wire-mappings'
            inputs.mkdir(parents=True)
            for original in (ROOT/'third_party/rar-wire-mappings').iterdir():
                (inputs/original.name).write_bytes(original.read_bytes())
            (inputs/'UnRAR-7.3.1-license.txt').write_bytes(b'altered reference notice')
            with patch.object(rar_mapping_materials, 'ROOT', root):
                with self.assertRaisesRegex(ValueError, 'checksum or scope'):
                    rar_mapping_materials.collect(root/'output')
            self.assertFalse((root/'output').exists())

    def test_rar_mapping_rejects_changed_license_identifier(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            inputs = root/'third_party/rar-wire-mappings'
            inputs.mkdir(parents=True)
            for original in (ROOT/'third_party/rar-wire-mappings').iterdir():
                (inputs/original.name).write_bytes(original.read_bytes())
            config = json.loads((inputs/'sources.json').read_text(encoding='utf-8'))
            config['source_license'] = 'Apache-2.0'
            (inputs/'sources.json').write_text(json.dumps(config), encoding='utf-8')
            with patch.object(rar_mapping_materials, 'ROOT', root):
                with self.assertRaisesRegex(ValueError, 'checksum or scope'):
                    rar_mapping_materials.collect(root/'output')
            self.assertFalse((root/'output').exists())

    def test_rar_mapping_missing_or_changed_source_comment_fails_before_output(self):
        notice_dir = ROOT/'third_party/rar-wire-mappings'
        for target_name in rar_mapping_materials.SOURCE_FILES:
            for replacement in ('', '// RAR archives without limitations free of charge, but can be'):
                with self.subTest(target=target_name, replacement=replacement), tempfile.TemporaryDirectory() as temporary:
                    root = Path(temporary)
                    inputs = root/'third_party/rar-wire-mappings'
                    inputs.mkdir(parents=True)
                    for original in notice_dir.iterdir():
                        (inputs/original.name).write_bytes(original.read_bytes())
                    for name in rar_mapping_materials.SOURCE_FILES:
                        target = root/name
                        target.parent.mkdir(parents=True, exist_ok=True)
                        text = (ROOT/name).read_text(encoding='utf-8')
                        if name == target_name:
                            original_line = '// RAR archives without limitations free of charge, but cannot be'
                            self.assertEqual(text.count(original_line), 1)
                            text = text.replace(original_line, replacement, 1)
                        target.write_text(text, encoding='utf-8')
                    with patch.object(rar_mapping_materials, 'ROOT', root):
                        with self.assertRaisesRegex(ValueError, 'source notice paragraph missing or changed'):
                            rar_mapping_materials.collect(root/'output')
                    self.assertFalse((root/'output').exists())

    def test_rar_mapping_existing_candidate_notice_is_never_rewritten(self):
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary)
            paths, _ = rar_mapping_materials.collect(output)
            notice = next(path for path in paths if path.name.endswith('.txt'))
            notice.write_bytes(b'earlier candidate notice')
            with self.assertRaisesRegex(ValueError, 'Existing RAR mapping notice differs'):
                rar_mapping_materials.collect(output)
            self.assertEqual(notice.read_bytes(), b'earlier candidate notice')

    def test_notice_inventory_hashes_bounded_rar_materials_without_declaring_an_engine(self):
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary)/'notices'
            def checked(command, **_):
                if command[:2] == ['rustc', '-vV']:
                    return 'host: x86_64-unknown-linux-gnu\n'
                if command[:2] == ['cargo', 'metadata']:
                    return json.dumps({'packages': [], 'resolve': {'nodes': []}})
                if command[:3] == ['git', 'rev-parse', 'HEAD']:
                    return 'a'*40
                if command[:2] == ['git', 'status']:
                    return ''
                self.fail('Unexpected subprocess in owned notice fixture')
            with patch('native_runtime_materials.prepare', return_value={'files': []}), \
                 patch('native_codec_materials.notices', return_value=([], [])), \
                 patch('runtime_rust.runtime_candidates', return_value={'root': 'owned', 'packages': []}), \
                 patch.object(subprocess, 'check_output', side_effect=checked), \
                 patch.object(sys, 'platform', 'linux'), \
                 patch.object(sys, 'argv', ['collect-notices.py', str(output)]), redirect_stdout(io.StringIO()):
                runpy.run_path(str(ROOT/'scripts/collect-notices.py'), run_name='__main__')
            inventory = json.loads((output/'build-inventory.json').read_text())
            hashes = {item['file']: item['sha256'] for item in inventory['notice_hashes']}
            copyright_notice = output/'xdzip-COPYRIGHT.txt'
            self.assertEqual(copyright_notice.read_bytes(), (ROOT/'COPYRIGHT').read_bytes())
            self.assertEqual(hashes['xdzip-COPYRIGHT.txt'], hashlib.sha256(copyright_notice.read_bytes()).hexdigest())
            for name in ['README.md', 'sources.json', 'UnRAR-7.3.1-license.txt']:
                path = 'rar-wire-mappings/'+name
                self.assertEqual(hashes[path], hashlib.sha256((output/path).read_bytes()).hexdigest())
            self.assertEqual(inventory['native_dependencies'], [])
            self.assertEqual(len(inventory['limited_reference_materials']), 1)
            self.assertEqual(inventory['limited_reference_materials'][0]['source_license'], 'UnRAR')
            self.assertFalse(inventory['limited_reference_materials'][0]['official_engine_in_product'])

    def test_current_product_and_historical_rebuild_names(self):
        for platform, name, legacy in [('linux', 'xdzip', 'xdzip-browser'),
                                       ('win32', 'xdzip.exe', 'xdzip-browser.exe')]:
            with self.subTest(platform=platform), tempfile.TemporaryDirectory() as temp:
                build = Path(temp)
                desktop = build/'apps/desktop'
                desktop.mkdir(parents=True)
                old = desktop/legacy
                current = desktop/name
                old.write_bytes(b'owned historical executable')
                self.assertEqual(built_product(build, platform=platform, allow_legacy_name=True), old)
                with self.assertRaisesRegex(ValueError, 'exactly one'):
                    built_product(build, platform=platform)
                current.write_bytes(b'owned current executable')
                self.assertEqual(built_product(build, platform=platform), current)
                with self.assertRaisesRegex(ValueError, 'exactly one'):
                    built_product(build, platform=platform, allow_legacy_name=True)
                old.unlink()
                self.assertEqual(built_product(build, platform=platform, allow_legacy_name=True), current)
                current.unlink()
                with self.assertRaisesRegex(ValueError, 'exactly one'):
                    built_product(build, platform=platform, allow_legacy_name=True)

    def test_macos_product_remains_inside_its_bundle(self):
        with tempfile.TemporaryDirectory() as temp:
            build = Path(temp)
            app = build/'apps/desktop/xdzip.app/Contents/MacOS/xdzip'
            app.parent.mkdir(parents=True)
            app.write_bytes(b'owned bundle executable')
            for historical in [False, True]:
                self.assertEqual(built_product(build, platform='darwin', allow_legacy_name=historical), app)

    def archive(self, root, member='owned-src-1/file.txt'):
        path = root/'owned-src-1.tar.xz'
        with tarfile.open(path, 'w:xz') as archive:
            info = tarfile.TarInfo(member)
            info.size = 4
            archive.addfile(info, io.BytesIO(b'own!'))
        return {'file': path.name, 'sha256': digest(path), 'url': 'https://invalid.test/no-network'}

    def test_verified_cache_is_offline_and_tampered_cache_fails(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            item = self.archive(root)
            self.assertEqual(source.archive_source(item, root).name, item['file'])
            (root/item['file']).write_bytes(b'corrupted')
            with self.assertRaisesRegex(ValueError, 'checksum'):
                source.archive_source(item, root)

    def test_source_paths_are_inspected_without_extracting(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            for member in ['../escape', 'owned-src-1/../escape', '/absolute', 'wrong-src/file']:
                item = self.archive(root, member)
                with self.assertRaisesRegex(ValueError, 'layout'):
                    source.archive_source(item, root)
            self.assertFalse((root/'escape').exists())

    def test_bundle_links_cannot_escape_the_application(self):
        with tempfile.TemporaryDirectory() as temp:
            app = Path(temp)/'xdzip.app'
            (app/'Contents/MacOS').mkdir(parents=True)
            (app/'Contents/MacOS/xdzip').write_bytes(b'owned')
            (app/'Contents/Info.plist').write_bytes(plistlib.dumps(
                {'CFBundleIdentifier': 'org.xdzip.desktop', 'CFBundleExecutable': 'xdzip'}))
            (app/'Contents/escape').symlink_to(Path(temp))
            with patch.object(mac, 'checked', return_value='arm64'):
                with self.assertRaisesRegex(ValueError, 'escapes'):
                    mac.validate_bundle(app)


class DocumentedRuntimeTestInputs(unittest.TestCase):
    """Execute the delivered/documented Python handoff with owned test inputs."""
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(dir=ROOT/'build', prefix='documented-test-inputs-')
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name).resolve()
        self.worker = self.root/'build/custom/xdzip-native-codec'
        self.worker.parent.mkdir(parents=True)
        self.worker.write_bytes(b'owned worker placeholder; never executed')
        self.fixture_environment = {}
        for name in ['XDZIP_RAR_PPM_FIXTURE', 'XDZIP_RAR_LEGACY_FIXTURE',
                     'XDZIP_RAR_LARGE_PPM_FIXTURE', 'XDZIP_RAR_UNKNOWN_FIXTURE', 'XDZIP_RAR26_FIXTURE']:
            relative = 'build/native-test-inputs/'+name+'.rar'
            path = self.root/relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(b'owned fixture placeholder')
            self.fixture_environment[name] = relative
        self.report = self.root/'build/native-test-inputs/sources.json'
        self.report.write_text(json.dumps({'environment': self.fixture_environment}), encoding='utf-8')

    def code(self, document):
        matches = re.findall(r'python3 -c "\n(.*?)\n" build/', document, flags=re.DOTALL)
        self.assertEqual(len(matches), 1)
        return compile(matches[0], '<documented runtime test handoff>', 'exec')

    def invoke(self, document, arguments=(), failure=None):
        with patch.object(Path, 'cwd', return_value=self.root), \
             patch.object(sys, 'argv', ['-c', str(self.worker), *arguments]), \
             patch.dict(os.environ, {'XDZIP_RAR_FIXTURE_PASSWORD': 'inherited test value', 'XDZIP_CAB_BACKEND': 'inherited backend'}), \
             patch.object(source.urllib.request, 'urlopen', side_effect=AssertionError('test handoff must not download')), \
             patch.object(subprocess, 'run', side_effect=failure,
                          return_value=subprocess.CompletedProcess([], 0)) as run:
            before = dict(os.environ)
            if failure:
                with self.assertRaises(subprocess.CalledProcessError):
                    exec(self.code(document), {})
            else:
                exec(self.code(document), {})
            self.assertEqual(dict(os.environ), before)
            calls = run.call_args_list
        for call in calls:
            environment = call.kwargs['env']
            self.assertTrue(call.kwargs['check'])
            self.assertEqual(environment['XDZIP_NATIVE_CODEC'], str(self.worker))
            self.assertEqual(environment['XDZIP_NATIVE_CODEC_WORKER'], str(self.worker))
            self.assertEqual(environment['XDZIP_CAB_BACKEND'], 'rust')
            self.assertEqual(environment['XDZIP_RAR_FIXTURE_PASSWORD'], 'password')
            for name, relative in self.fixture_environment.items():
                self.assertEqual(Path(environment[name]), self.root/relative)
        return calls

    def test_generated_rebuild_handoff_runs_both_full_offline_rust_commands(self):
        document = source.rebuild_instructions()
        calls = self.invoke(document)
        self.assertEqual([call.args[0] for call in calls], [
            ['cargo', 'test', '--workspace', '--locked', '--offline'],
            ['cargo', 'test', '-p', 'xdzip-runtime', '--locked', '--offline', 'native_backend_tests', '--', '--ignored']])
        self.assertIn('--archives /path/to/fixed-rar-cache --offline', document)
        self.assertIn('当前交付不包含', document)
        self.assertIn('tests/fixtures/cab-iso/', document)
        self.assertIn('tests/fixtures/native-archives/', document)
        self.assertIn('scripts/convert-iso-codes-for-qt.py', document)
        self.assertIn('scripts/convert-iana-loc-for-qt.py', document)
        self.assertIn("--add-language 'cho,cho,cho,'", document)
        self.assertIn('工具不从 IANA 推断 ISO 映射', document)
        self.assertIn('IANA presence does not establish ISO mappings', document)
        self.assertIn('完整输入会生成完整代码集', document)
        self.assertIn('iso-codes_4.16.0-1_all.deb', document)
        self.assertIn('cafca0a3a1f07b44fbba53ab7a1f1d9755e6b80cc041173afe2b52e53bed97a7', document)
        self.assertIn('docs/engineering/source-delivery.md', document)
        self.assertIn('CLDR 43.1 子集另含较早的语言注册表', document)
        self.assertIn('CLDR 43.1 subset separately retains an earlier registry', document)

    def test_generated_rebuild_stops_when_workspace_tests_fail(self):
        failure = subprocess.CalledProcessError(17, ['cargo', 'test'])
        calls = self.invoke(source.rebuild_instructions(), failure=failure)
        self.assertEqual(len(calls), 1)

    def test_developer_handoff_forwards_selected_sdk_and_build_without_skipping_rust(self):
        document = (ROOT/'docs/engineering/developer-build.md').read_text(encoding='utf-8')
        arguments = ['--qt-prefix', str(self.root/'owned-sdk'), '--build-dir', 'build/custom']
        calls = self.invoke(document, arguments)
        self.assertEqual(len(calls), 1)
        self.assertEqual(calls[0].args[0], [sys.executable, 'scripts/desktop.py', 'test', *arguments])
        self.assertNotIn('--skip-rust', calls[0].args[0])

    def test_generated_handoff_requires_explicitly_prepared_inputs_before_any_test(self):
        self.report.unlink()
        with patch.object(Path, 'cwd', return_value=self.root), \
             patch.object(sys, 'argv', ['-c', str(self.worker)]), \
             patch.object(subprocess, 'run') as run, \
             patch.object(source.urllib.request, 'urlopen') as download:
            with self.assertRaises(FileNotFoundError):
                exec(self.code(source.rebuild_instructions()), {})
            run.assert_not_called()
            download.assert_not_called()


class VendorNoticeMaterialTests(unittest.TestCase):
    def fixture(self, root):
        vendor = root/'vendor'
        package = vendor/'demo-1.2.3'
        package.mkdir(parents=True)
        cargo = b'[package]\nname = "demo"\nversion = "1.2.3"\nlicense = "MIT OR Apache-2.0"\n'
        (package/'Cargo.toml').write_bytes(cargo)
        revision = 'a'*40
        (package/'.cargo_vcs_info.json').write_text(json.dumps({'git': {'sha1': revision},
                                                               'path_in_vcs': 'crates/demo'}))
        upstream = root/'third_party/rust-vendor-notices/fixture/AUTHORS'
        upstream.parent.mkdir(parents=True)
        data = b'Owned test notice, not an actual license.'
        upstream.write_bytes(data)
        blob = hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest()
        item = {'name': 'demo', 'version': '1.2.3', 'vendor_directory': 'demo-1.2.3',
                'license_expression': 'MIT OR Apache-2.0',
                'manifest_sha256': hashlib.sha256(cargo).hexdigest(),
                'upstream_repository': 'https://github.com/example/demo',
                'upstream_revision': revision, 'upstream_package_path': 'crates/demo',
                'notices': [{'name': 'AUTHORS',
                             'repo_file': 'third_party/rust-vendor-notices/fixture/AUTHORS',
                             'sha256': hashlib.sha256(data).hexdigest(),
                             'upstream_git_blob': blob,
                             'upstream_url': f'https://github.com/example/demo/blob/{revision}/AUTHORS'}]}
        config = root/'sources.json'
        config.write_text(json.dumps({'schema_version': 1, 'packages': [item]}), encoding='utf-8')
        return vendor, package, upstream, config, item, data

    def test_supplement_indexes_upstream_without_changing_vendor(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            vendor, package, _, config, item, data = self.fixture(root)
            before = {p.name: p.read_bytes() for p in package.iterdir()}
            output = root/'supplement'
            record = vendor_notice_materials.collect(vendor, output, root=root, config_path=config)
            self.assertEqual({p.name: p.read_bytes() for p in package.iterdir()}, before)
            self.assertEqual((output/'demo-1.2.3/AUTHORS').read_bytes(), data)
            self.assertEqual(record['packages'][0]['license_expression'], item['license_expression'])
            self.assertEqual(record['packages'][0]['notices'][0]['upstream_url'], item['notices'][0]['upstream_url'])
            self.assertEqual(json.loads((output/'index.json').read_text(encoding='utf-8')), record)

    def test_unresolved_or_tampered_material_fails_before_output(self):
        for kind in ('unknown', 'manifest', 'metadata', 'revision', 'notice', 'path', 'url', 'vendor-notice'):
            with self.subTest(kind=kind), tempfile.TemporaryDirectory() as temp:
                root = Path(temp)
                vendor, package, upstream, config, item, _ = self.fixture(root)
                if kind == 'unknown':
                    unknown = vendor/'unidentified-1'; unknown.mkdir()
                    (unknown/'Cargo.toml').write_text('[package]')
                elif kind == 'manifest':
                    (package/'Cargo.toml').write_text('changed')
                elif kind == 'metadata':
                    item['license_expression'] = 'MIT'
                elif kind == 'revision':
                    (package/'.cargo_vcs_info.json').write_text(json.dumps({'git': {'sha1': 'b'*40},
                                                                           'path_in_vcs': 'crates/demo'}))
                elif kind == 'notice':
                    upstream.write_bytes(b'tampered')
                elif kind == 'path':
                    item['notices'][0]['repo_file'] = '../outside'
                elif kind == 'url':
                    item['notices'][0]['upstream_url'] = 'https://example.invalid/notice'
                elif kind == 'vendor-notice':
                    (package/'AUTHORS').write_bytes(b'different')
                config.write_text(json.dumps({'schema_version': 1, 'packages': [item]}), encoding='utf-8')
                output = root/'supplement'
                with self.assertRaises(ValueError):
                    vendor_notice_materials.collect(vendor, output, root=root, config_path=config)
                self.assertFalse(output.exists())

    def test_nested_license_source_code_is_not_a_notice(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            vendor, package, _, config, _, _ = self.fixture(root)
            code = package/'src/oci_annotations/licenses.rs'
            code.parent.mkdir(parents=True)
            code.write_text('pub struct Licenses;')
            (package/'licenses.rs').write_text('pub struct Licenses;')
            record = vendor_notice_materials.collect(vendor, root/'supplement', root=root, config_path=config)
            self.assertEqual([item['name'] for item in record['packages']], ['demo'])


class DisposableBundleTests(unittest.TestCase):
    """Simulated native tools, actual owned copies, hashes and disposal."""
    def bundle(self, root):
        app = root/'xdzip.app'
        binary = app/'Contents/MacOS/xdzip'
        core = app/'Contents/Frameworks/QtCore.framework/Versions/A/QtCore'
        for path, data in [(binary, b'owned executable'), (core, b'original core')]:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
        (app/'Contents/Info.plist').write_bytes(plistlib.dumps({
            'CFBundleIdentifier': ls.BUNDLE_ID, 'CFBundleExecutable': 'xdzip',
            'CFBundleDocumentTypes': [{'LSHandlerRank': 'Alternate', 'CFBundleTypeRole': 'Viewer'}]}))
        return app, binary, core

    def test_dmg_builder_preserves_image_and_evidence_but_removes_stage_on_success_or_failure(self):
        for failed in [False, True]:
            with self.subTest(failed=failed), tempfile.TemporaryDirectory() as temp:
                root = Path(temp).resolve();source, binary, core = self.bundle(root/'input')
                output = root/'build/package.dmg';qt = root/'qt';revision = 'a'*12
                identity = 'xdzip (development, '+revision+')'
                protected = {path: digest(path) for path in [binary, core, source/'Contents/Info.plist']}
                staged = output.parent/'package-stage/xdzip.app'
                phases = []
                optimization = {'before_bytes': 150, 'after_bytes': 80, 'bytes_saved': 70}
                bundle_validation = {'status': 'passed', 'architectures': ['arm64'], 'control_style': 'Basic'}
                def checked(command, **_):
                    if str(command[0]) == 'lipo':return 'arm64'
                    if '--version' in command:return identity
                    if 'rev-parse' in command:return revision
                    if len(command)>1 and Path(str(command[1])).name == 'collect-notices.py':
                        notices = Path(command[2]);notices.mkdir(parents=True, exist_ok=True)
                        (notices/'build-inventory.json').write_text(json.dumps({'display_version':'0.1.0','version_source':{'build':1}}))
                        (notices/'source-build.cdx.json').write_text('{}')
                    if len(command)>1 and Path(str(command[1])).name == 'runtime_attribution.py':
                        self.assertTrue(staged.is_dir())
                        self.assertTrue((output.parent/'deployment-audit.json').is_file())
                        Path(command[command.index('--output')+1]).write_text('{"owned":"attribution"}')
                    if str(command[0]) == 'hdiutil':
                        self.assertTrue(staged.is_dir())
                        if command[1] == 'create':output.write_bytes(b'owned image')
                        elif failed:raise ValueError('owned verify failure')
                    return ''
                def optimize(app):
                    self.assertEqual(app, staged)
                    phases.append('optimize')
                    return optimization
                def sign(app, *_):
                    self.assertEqual(app, staged)
                    phases.append('sign')
                def validate(app):
                    self.assertEqual(app, staged)
                    phases.append('validate')
                    return bundle_validation
                with patch.object(ls,'ROOT',root), patch.object(mac,'ROOT',root), \
                     patch.object(mac.sys,'platform','darwin'), patch.object(mac,'checked',side_effect=checked), \
                     patch.object(mac,'transient_registrations',side_effect=lambda _:nullcontext()), \
                     patch.object(mac,'optimize_bundle',side_effect=optimize), \
                     patch.object(mac,'sdk_from_build',return_value=qt), \
                     patch.object(mac,'embed_sparkle',return_value=staged/'Contents/Frameworks/Sparkle.framework'), \
                     patch.object(mac,'signing_metadata',return_value=[]), \
                     patch.object(mac,'attribute_sparkle'), \
                     patch.object(mac,'validate_optimized_bundle',side_effect=validate), \
                     patch.object(mac,'sign_local',side_effect=sign), patch.object(mac,'native',return_value=False), \
                     patch.object(mac,'audit',return_value={'status':'passed'}), \
                     patch.object(mac,'deployment_record',return_value={'sdk_files':[]}), \
                     patch.object(mac,'artifact_bom',return_value={'owned':'sbom'}):
                    if failed:
                        with self.assertRaisesRegex(ValueError,'owned verify failure'):mac.build(source,qt,output)
                    else:
                        report = mac.build(source,qt,output)
                        self.assertEqual(report['sha256'],digest(output))
                        self.assertEqual(report['bundle_optimization'], optimization)
                        self.assertEqual(report['bundle'], bundle_validation)
                self.assertEqual(phases, ['optimize', 'sign', 'validate'])
                self.assertFalse(staged.exists())
                self.assertTrue(output.is_file())
                self.assertTrue(output.with_suffix('.cdx.json').is_file())
                self.assertEqual(output.with_suffix('.json').exists(),not failed)
                self.assertEqual(protected,{path:digest(path) for path in protected})

    def test_dmg_optimization_failure_stops_before_signing_and_discards_stage(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp).resolve();source, binary, core = self.bundle(root/'input')
            output = root/'build/package.dmg';qt = root/'qt';revision = 'a'*12
            protected = {path:digest(path) for path in [binary,core,source/'Contents/Info.plist']}
            def checked(command, **_):
                if str(command[0]) == 'lipo':return 'arm64'
                if '--version' in command:return 'xdzip (development, '+revision+')'
                if 'rev-parse' in command:return revision
                return ''
            with patch.object(ls,'ROOT',root), patch.object(mac,'ROOT',root), \
                 patch.object(mac.sys,'platform','darwin'), patch.object(mac,'checked',side_effect=checked), \
                 patch.object(mac,'transient_registrations',side_effect=lambda _:nullcontext()), \
                 patch.object(mac,'sdk_from_build',return_value=qt), \
                 patch.object(mac,'embed_sparkle',return_value=root/'Sparkle.framework'), \
                 patch.object(mac,'signing_metadata',return_value=[]), \
                 patch.object(mac,'optimize_bundle',side_effect=ValueError('invalid deployment')), \
                 patch.object(mac,'sign_local') as sign, patch.object(mac,'audit') as audit:
                with self.assertRaisesRegex(ValueError,'invalid deployment'):mac.build(source,qt,output)
                sign.assert_not_called();audit.assert_not_called()
            self.assertFalse((output.parent/'package-stage/xdzip.app').exists())
            self.assertFalse(output.exists())
            self.assertFalse(output.with_suffix('.json').exists())
            self.assertEqual(protected,{path:digest(path) for path in protected})

    def test_dmg_verifier_freezes_hashes_and_removes_copy_on_success_or_failure(self):
        for failed in [False, True]:
            with self.subTest(failed=failed), tempfile.TemporaryDirectory() as temp:
                root = Path(temp).resolve();mount = root/'mounted'
                source, binary, core = self.bundle(mount)
                (mount/'Applications').symlink_to('/Applications')
                dmg = root/'input.dmg';dmg.write_bytes(b'owned image')
                output = root/'build/verification';copy = output/'中文 space/xdzip.app'
                protected = {path:digest(path) for path in [binary,core,dmg]}
                modified = core.read_bytes()+b' modified'
                def checked(command, **_):
                    if str(command[0]) == 'lipo':return 'arm64'
                    if str(command[0]) == 'install_name_tool':Path(command[-1]).write_bytes(modified)
                    return ''
                def run(command, **_):
                    if '--smoke-test' in command:
                        self.assertTrue(copy.exists())
                        return subprocess.CompletedProcess(command,1 if failed else 0,'owned output','')
                    return subprocess.CompletedProcess(command,0,'','')
                arguments = ['verify-macos-dmg','--dmg',str(dmg),'--output',str(output)]
                mounted = plistlib.dumps({'system-entities':[{'mount-point':str(mount)}]})
                with patch.object(ls,'ROOT',root), patch.object(verify_dmg,'ROOT',root), \
                     patch.object(verify_dmg,'transient_registrations',side_effect=lambda _:nullcontext()), \
                     patch.object(verify_dmg,'audit',return_value={'status':'passed'}), \
                     patch.object(verify_dmg,'validate_optimized_bundle',return_value={'status':'passed'}), \
                     patch.object(verify_dmg,'verify_signatures',return_value={'status':'passed'}), \
                     patch.object(verify_dmg,'product_identity',return_value=('xdzip owned fixture',{})), \
                     patch.object(verify_dmg,'verify_extraction',return_value={'status':'passed'}), \
                     patch.object(verify_dmg.macdmg,'checked',side_effect=checked), \
                     patch.object(verify_dmg.macdmg,'sign_local'), \
                     patch.object(verify_dmg.subprocess,'check_output',return_value=mounted), \
                     patch.object(verify_dmg.subprocess,'run',side_effect=run), patch.object(sys,'argv',arguments), \
                     redirect_stdout(io.StringIO()):
                    if failed:
                        with self.assertRaisesRegex(ValueError,'relocated-cocoa-metal failed'):verify_dmg.main()
                    else:verify_dmg.main()
                self.assertFalse(copy.exists())
                self.assertTrue((output/'relocated-cocoa-metal.log').is_file())
                self.assertEqual(protected,{path:digest(path) for path in protected})
                self.assertEqual((output/'verification.json').exists(),not failed)
                if not failed:
                    expected = root/'modified';expected.write_bytes(modified)
                    result = json.loads((output/'verification.json').read_text())
                    self.assertEqual(result['replacement']['original_sha256'],digest(core))
                    self.assertEqual(result['replacement']['modified_sha256'],digest(expected))

    def test_qt_replacement_freezes_hashes_and_removes_copy_on_success_or_failure(self):
        for failed in [False, True]:
            with self.subTest(failed=failed), tempfile.TemporaryDirectory() as temp:
                root = Path(temp).resolve();source, binary, core = self.bundle(root/'input')
                qt = root/'source-qt';new_core = qt/'qt-install/lib/QtCore.framework/Versions/A/QtCore'
                new_core.parent.mkdir(parents=True);new_core.write_bytes(b'source built core')
                record = qt/'qt-source-build.json'
                record.write_text(json.dumps({'revision':'a'*40,'qtcore_sha256':digest(new_core)}))
                output = root/'build/replacement';copy = output/'中文 relocated/xdzip.app'
                protected = {path:digest(path) for path in [binary,core,new_core,record]}
                def checked(command, **_):
                    if str(command[0]) == 'lipo':return 'arm64'
                    if '--version' in command:return 'xdzip ('+'a'*12+')'
                    return ''
                def sign(app):
                    path = app/'Contents/Frameworks/QtCore.framework/Versions/A/QtCore'
                    path.write_bytes(path.read_bytes()+b' signed')
                def run(command, **_):
                    target = Path(command[command.index('--output')+1])
                    if Path(command[1]).name.startswith('generate-'):target.write_bytes(b'owned fixture')
                    else:
                        self.assertTrue(copy.exists())
                        if failed and target.stem == 'extract':raise subprocess.CalledProcessError(1,command)
                        target.write_text(json.dumps({'runs':[{'events':[{'event':'qt_source_proof',
                            'value':'xdzip-qt-6.8.2-source-proof-v1'}],'extraction':{'independent_byte_crc_check':True}}]}))
                    return subprocess.CompletedProcess(command,0)
                arguments = ['verify-qt-replacement','--app',str(source),'--qt-build',str(qt),
                             '--metrics',str(root/'metrics'),'--output',str(output)]
                with patch.object(ls,'ROOT',root), patch.object(replace_qt,'ROOT',root), \
                     patch.object(replace_qt.sys,'platform','darwin'), \
                     patch.object(replace_qt,'transient_registrations',side_effect=lambda _:nullcontext()), \
                     patch.object(replace_qt,'module',return_value=mac), \
                     patch.object(mac,'checked',side_effect=checked), patch.object(mac,'sign_local',side_effect=sign), \
                     patch.object(replace_qt,'audit',return_value={'status':'passed'}), \
                     patch.object(replace_qt.subprocess,'run',side_effect=run), patch.object(sys,'argv',arguments), \
                     redirect_stdout(io.StringIO()):
                    if failed:
                        with self.assertRaises(subprocess.CalledProcessError):replace_qt.main()
                    else:replace_qt.main()
                self.assertFalse(copy.exists())
                self.assertTrue((output/'replacement.log').is_file())
                self.assertTrue((output/'metadata.json').is_file())
                self.assertEqual(protected,{path:digest(path) for path in protected})
                self.assertEqual((output/'qt-source-replacement.json').exists(),not failed)
                if not failed:
                    expected = root/'signed-core';expected.write_bytes(new_core.read_bytes()+b' signed')
                    result = json.loads((output/'qt-source-replacement.json').read_text())
                    self.assertEqual(result['original_qtcore_sha256'],digest(core))
                    self.assertEqual(result['replaced_signed_qtcore_sha256'],digest(expected))
                    self.assertEqual(result['app_sha256'],digest(binary))


if __name__ == '__main__':
    unittest.main()
