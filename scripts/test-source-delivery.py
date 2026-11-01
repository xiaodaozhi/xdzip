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
from unittest.mock import patch
from deployment_audit import digest, within
from desktop_product import built_product
import macos_launch_services as ls
import rar_mapping_materials
import vendor_notice_materials

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


class DeliveryTests(unittest.TestCase):
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

    def test_native_source_delivery_retains_icu_without_software_opengl_archives(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary); cache = root/'cache'; output = root/'delivery'
            cache.mkdir(); output.mkdir(); (root/'scripts').mkdir()
            data = b'Owned ICU license fixture'
            item = {'file': 'ICU.txt', 'sha256': hashlib.sha256(data).hexdigest(),
                    'url': 'https://invalid.test/not-used'}
            (cache/'ICU.txt').write_bytes(data)
            (root/'scripts/native-runtime-sources.json').write_text(json.dumps({'icu': item}))
            with patch.object(source, 'ROOT', root), patch.object(source, 'configuration', return_value={'icu': item}):
                material = source.collect_native_sources(output, cache)
            self.assertEqual(material, item)
            self.assertEqual((output/'ICU.txt').read_bytes(), data)
            self.assertEqual({path.name for path in output.iterdir()}, {'ICU.txt', 'native-runtime-sources.json'})

    def test_rar_mapping_notice_bytes_match_in_notices_and_separate_delivery(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            results = []
            for output in [root/'notices', root/'delivery']:
                paths, record = source.collect_rar_mapping_materials(output)
                self.assertEqual(len(paths), 3)
                self.assertEqual(record['source_release'], 'UnRAR 7.30 beta1')
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
            for name in ['README.md', 'sources.json', 'UnRAR-7.3.1-license.txt']:
                path = 'rar-wire-mappings/'+name
                self.assertEqual(hashes[path], hashlib.sha256((output/path).read_bytes()).hexdigest())
            self.assertEqual(inventory['native_dependencies'], [])
            self.assertEqual(len(inventory['limited_reference_materials']), 1)
            self.assertEqual(inventory['limited_reference_materials'][0]['source_license'], 'LicenseRef-UnRAR')
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
