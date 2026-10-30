# scripts/test-release-protocol.py

import copy
from contextlib import redirect_stdout
import importlib.util
import io
import json
import os
from pathlib import Path
import shutil
import struct
import subprocess
import sys
import tempfile
import tarfile
import unittest
from unittest.mock import patch
from release_protocol import TARGETS, create, select, validate
from release_evidence import compare_macho_sections, macho_sections, verify_bindings

SPEC = importlib.util.spec_from_file_location('publish_release', Path(__file__).with_name('publish-release.py'))
PUBLISH = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PUBLISH)
PREPARE_SPEC = importlib.util.spec_from_file_location('prepare_release', Path(__file__).with_name('prepare-release.py'))
PREPARE = importlib.util.module_from_spec(PREPARE_SPEC)
PREPARE_SPEC.loader.exec_module(PREPARE)


class MacReleaseEvidenceTests(unittest.TestCase):
    @staticmethod
    def image(code=b'owned code', signature=b'first signature', zero_bytes=16,
              uuid=b'0123456789abcdef', import_version=0x10000):
        commands = []
        for segment, section, length, offset, flags in [
            (b'__TEXT', b'__text', len(code), 512, 0),
            (b'__DATA', b'__bss', zero_bytes, 0, 1),
        ]:
            command = struct.pack('<II16sQQQQiiII', 0x19, 152, segment, 0x100000000,
                                  4096, 0, len(code), 7, 5, 1, 0)
            command += struct.pack('<16s16sQQIIIIIIII', section, segment, 0x100000200,
                                   length, offset, 0, 0, 0, flags, 0, 0, 0)
            commands.append(command)
        commands.append(struct.pack('<II16s', 0x1B, 24, uuid))
        name = b'@rpath/QtCore.framework/Versions/A/QtCore\0'
        import_size = (24 + len(name) + 7) & ~7
        commands.append((struct.pack('<6I', 0xC, import_size, 24, 0, import_version, 0x10000) + name).ljust(import_size, b'\0'))
        commands.append(struct.pack('<4I', 0x1D, 16, 512 + len(code), len(signature)))
        table = b''.join(commands)
        header = struct.pack('<8I', 0xFEEDFACF, 0x0100000C, 0, 2, len(commands), len(table), 0, 0)
        return (header + table).ljust(512, b'\0') + code + signature

    def test_signature_metadata_changes_preserve_exact_section_relation(self):
        before, after = self.image(), self.image(signature=b'replacement ad-hoc signature')
        proof = compare_macho_sections(before, after)
        self.assertNotEqual(proof['compiled_sha256'], proof['deployed_sha256'])
        self.assertEqual(proof['status'], 'exact-code-data-section-match')
        self.assertEqual(len(proof['sections']), 2)

    def test_changed_code_and_zero_fill_size_are_rejected(self):
        for changed in [self.image(code=b'changed code'), self.image(zero_bytes=17)]:
            with self.subTest(changed=changed[-20:]):
                with self.assertRaisesRegex(ValueError, 'changed compiled'):
                    compare_macho_sections(self.image(), changed)

    def test_changed_uuid_and_import_versions_are_rejected(self):
        for changed in [self.image(uuid=b'fedcba9876543210'), self.image(import_version=0x20000)]:
            with self.subTest(changed=changed[-20:]):
                with self.assertRaises(ValueError):
                    compare_macho_sections(self.image(), changed)

    def test_truncated_commands_and_section_data_are_rejected(self):
        data = self.image()
        truncated_command = bytearray(data)
        struct.pack_into('<I', truncated_command, 20, len(data) + 1)
        escaped_section = bytearray(data)
        struct.pack_into('<Q', escaped_section, 32 + 72 + 40, len(data) + 1)
        bad_size = bytearray(data)
        struct.pack_into('<I', bad_size, 36, 7)
        for bad in [data[:30], truncated_command, escaped_section, bad_size]:
            with self.subTest(length=len(bad)):
                with self.assertRaises(ValueError):
                    macho_sections(bad)

    def test_architecture_and_duplicate_sections_are_rejected(self):
        foreign = bytearray(self.image())
        struct.pack_into('<I', foreign, 4, 0x01000007)
        with self.assertRaisesRegex(ValueError, 'arm64'):
            macho_sections(foreign)
        duplicate = bytearray(self.image())
        duplicate[32 + 152:32 + 304] = duplicate[32:32 + 152]
        with self.assertRaisesRegex(ValueError, 'Duplicate'):
            macho_sections(duplicate)

    def bindings(self):
        revision = 'a' * 40
        identity = {'revision': revision[:12], 'dirty': False}
        match = compare_macho_sections(self.image(), self.image(signature=b'new signature'))
        audit = {'status': 'passed', 'mode': 'deployed', 'platform': 'macos', 'identity': identity,
                 'app': 'Contents/MacOS/xdzip',
                 'files': [{'path': 'Contents/MacOS/xdzip', 'sha256': match['deployed_sha256']}]}
        resource = {'identity': dict(identity), 'app_sha256': match['deployed_sha256']}
        link = {'app_sha256': match['compiled_sha256']}
        sdk = {'revision': revision, 'source_dirty': False}
        return [revision, audit, resource, link, sdk, match]

    def test_distinct_compiled_and_deployed_images_bind_to_their_own_proof(self):
        verify_bindings(*self.bindings())

    def test_mixed_resource_link_and_sdk_proofs_are_rejected(self):
        for index, field, value in [(2, 'app_sha256', '0' * 64),
                                    (3, 'app_sha256', '0' * 64),
                                    (4, 'revision', 'b' * 40),
                                    (4, 'source_dirty', True),
                                    (5, 'deployed_sha256', '0' * 64),
                                    (5, 'comparison', 'unverified'),
                                    (5, 'fingerprint', None)]:
            values = self.bindings()
            values[index][field] = value
            with self.subTest(proof=index, field=field):
                with self.assertRaises(ValueError):
                    verify_bindings(*values)
        values = self.bindings()
        values[2]['identity']['dirty'] = True
        with self.assertRaises(ValueError):
            verify_bindings(*values)


class MaterialsTests(unittest.TestCase):
    def test_referenced_license_texts_are_delivered_and_tampering_is_rejected(self):
        with tempfile.TemporaryDirectory() as temp:
            source = Path(temp)/'source'; (source/'notices').mkdir(parents=True)
            text = source/'runtime-notices/license-texts/owned.txt'
            text.parent.mkdir(parents=True); text.write_bytes(b'owned license')
            report = {'license_materials_base': 'runtime-notices',
                      'license_materials': {'owned': {'file': 'license-texts/owned.txt', 'sha256': PREPARE.sha256(text)}}}
            (source/'runtime-attribution.json').write_text(json.dumps(report))
            output = Path(temp)/'materials.tar.gz'
            PREPARE.materials_archive(source, output)
            with tarfile.open(output) as archive:
                self.assertEqual(archive.extractfile('runtime-notices/license-texts/owned.txt').read(), b'owned license')
            text.write_bytes(b'changed')
            with self.assertRaisesRegex(ValueError, 'missing or changed'):
                PREPARE.materials_archive(source, output)
            report['license_materials_base'] = '../outside'
            (source/'runtime-attribution.json').write_text(json.dumps(report))
            with self.assertRaisesRegex(ValueError, 'Unbounded'):
                PREPARE.materials_archive(source, output)


class ReleaseProtocolTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.directory = Path(self.temp.name)
        for suffix, _ in TARGETS.values():
            (self.directory/('xdzip-0.1.0-alpha.1-'+suffix)).write_bytes(b'owned package fixture')
        self.manifest = create(self.directory, dict(display_version='0.1.0-alpha.1', channel='alpha', build=14), 'owner/repo', 'a'*40)

    def test_channel_and_semantic_version_selection(self):
        args = ('windows', 'x86_64', 'zip')
        self.assertIsNotNone(select(self.manifest, '0.0.9', ['alpha'], *args))
        self.assertIsNone(select(self.manifest, '0.1.0-alpha.1', ['alpha'], *args))
        self.assertIsNone(select(self.manifest, '0.1.0', ['alpha'], *args))
        self.assertIsNone(select(self.manifest, '0.0.9', ['stable'], *args))
        self.assertIsNone(select(self.manifest, '0.0.9', ['alpha'], 'windows', 'arm64', 'zip'))

    def test_missing_package_and_mutated_hash_fail_closed(self):
        invalid = copy.deepcopy(self.manifest)
        invalid['assets'].pop()
        with self.assertRaises(ValueError): validate(invalid)
        invalid = copy.deepcopy(self.manifest)
        invalid['assets'][0]['url'] = invalid['assets'][0]['url'].replace('/v0.1.0-alpha.1/', '/latest/')
        with self.assertRaises(ValueError): validate(invalid)
        asset = self.directory/self.manifest['assets'][0]['filename']
        asset.write_bytes(b'tampered')
        with self.assertRaises(ValueError): validate(self.manifest, self.directory)


class ReleaseNotesTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.version = dict(display_version='0.1.0-alpha.7', channel='alpha', build=20)
        self.notes = self.root/'reviewed.md'
        self.body = '# xdzip 0.1.0-alpha.7\n\nWindows 人工验收通过，macOS 未执行。\n\nWindows accepted; macOS not performed.\n'
        self.notes.write_text('<!-- xdzip-release {"version":"0.1.0-alpha.7","build":20} -->\n\n'+self.body,
                              encoding='utf-8')
        self.commands = []

    def invoke(self, *arguments, state=None, source=None, use_override=True):
        def command(args, **kwargs):
            self.commands.append(args)
            if args[:3] == ['gh', 'release', 'view']:
                return subprocess.CompletedProcess(args, 1 if state is None else 0,
                    '' if state is None else json.dumps(state))
            if args[:3] == ['gh', 'release', 'download']:
                shutil.copytree(source, Path(args[args.index('--dir')+1]), dirs_exist_ok=True)
            return subprocess.CompletedProcess(args, 0, '')
        with patch.object(PUBLISH, 'ROOT', self.root), patch.object(PUBLISH.runpy, 'run_path',
                return_value={'version_info': lambda: self.version}), patch.object(PUBLISH.subprocess, 'run', side_effect=command), \
                patch.object(sys, 'argv', ['publish-release.py', *(['--notes-file', str(self.notes)] if use_override else []), *arguments]), \
                patch('release_updates.verify_release_updates'), \
                redirect_stdout(io.StringIO()):
            PUBLISH.main()

    def test_offline_preflight_and_default_version_file_preserve_reviewed_acceptance(self):
        path = self.root/'release-notes/0.1.0-alpha.7.md'
        path.parent.mkdir()
        path.write_bytes(self.notes.read_bytes())
        self.invoke('--check-notes', use_override=False)
        self.assertEqual(self.commands, [])
        self.assertEqual(PUBLISH.load_notes(path, self.version), self.body)
        self.assertNotIn('首个', self.body)

    def test_new_version_without_notes_cannot_reuse_previous_acceptance(self):
        with self.assertRaises(FileNotFoundError): self.invoke('--check-notes', use_override=False)
        self.assertEqual(self.commands, [])

    def test_gh_captures_multilingual_output_without_changing_characters(self):
        original_run = subprocess.run
        message = '已验收，仍待审查'
        def utf8_output(args, **kwargs):
            code = 'import sys;sys.stdout.buffer.write('+repr(message.encode('utf-8'))+')'
            return original_run([sys.executable, '-c', code], **kwargs)
        with patch.object(PUBLISH.subprocess, 'run', side_effect=utf8_output):
            self.assertEqual(PUBLISH.gh('test-output', capture=True), message)

    def test_mismatched_version_build_and_unreviewed_notes_fail_before_github(self):
        valid = self.notes.read_text(encoding='utf-8')
        for invalid in [valid.replace('"build":20', '"build":19'), valid.replace('alpha.7', 'alpha.6'),
                        valid.replace('"build":20', '"build":true'), '# xdzip 0.1.0-alpha.7\n\nUnversioned notes']:
            with self.subTest(invalid=invalid):
                self.notes.write_text(invalid, encoding='utf-8')
                with self.assertRaises(ValueError): self.invoke('--check-notes')
                self.assertEqual(self.commands, [])

    def test_initial_draft_uses_reviewed_body_and_rerun_keeps_it(self):
        arguments = ['--tag', 'v0.1.0-alpha.7', '--repository', 'owner/repo', '--draft-only']
        self.invoke(*arguments)
        create_command = next(c for c in self.commands if c[:3] == ['gh', 'release', 'create'])
        snapshot = Path(create_command[create_command.index('--notes-file')+1])
        self.assertEqual(snapshot.read_text(encoding='utf-8'), self.body)
        self.commands.clear()
        self.invoke(*arguments, state=dict(isDraft=True, assets=[], body=self.body.replace('\n', '\r\n')))
        self.assertEqual(len(self.commands), 1)

    def test_changed_draft_and_public_release_are_never_silently_overwritten(self):
        for state in [dict(isDraft=True, assets=[], body='Manually reviewed replacement'),
                      dict(isDraft=False, assets=[], body=self.body)]:
            with self.subTest(state=state):
                self.commands.clear()
                with self.assertRaises(ValueError):
                    self.invoke('--tag', 'v0.1.0-alpha.7', '--repository', 'owner/repo', '--draft-only', state=state)
                self.assertEqual(len(self.commands), 1)

    def test_final_publication_uses_same_notes_and_verifies_both_downloads(self):
        source = self.root/'packages'
        source.mkdir()
        for suffix, _ in TARGETS.values():
            (source/('xdzip-0.1.0-alpha.7-'+suffix)).write_bytes(b'owned native package')
        manifest = create(source, self.version, 'owner/repo', 'a'*40)
        (source/'release.json').write_text(json.dumps(manifest), encoding='utf-8')
        (source/'SHA256SUMS').write_text(''.join(PUBLISH.sha256(p)+'  '+p.name+'\n' for p in sorted(source.iterdir())))
        self.invoke('--tag', 'v0.1.0-alpha.7', '--repository', 'owner/repo', '--directory', str(source),
                    state=dict(isDraft=True, assets=[], body=self.body), source=source)
        edits = [c for c in self.commands if c[:3] == ['gh', 'release', 'edit']]
        self.assertEqual(len(edits), 1)
        self.assertEqual(Path(edits[0][edits[0].index('--notes-file')+1]).read_text(encoding='utf-8'), self.body)
        self.assertEqual(sum(c[:3] == ['gh', 'release', 'download'] for c in self.commands), 2)


@unittest.skipUnless(shutil.which('pwsh'), 'PowerShell 7 is required for the native Release entry contract')
class NativeReleaseInputsTests(unittest.TestCase):
    fixture_names = (
        'XDZIP_RAR_PPM_FIXTURE', 'XDZIP_RAR_LARGE_PPM_FIXTURE', 'XDZIP_RAR_LEGACY_FIXTURE',
        'XDZIP_RAR_UNKNOWN_FIXTURE', 'XDZIP_RAR_KNOWN_EMPTY_FIXTURE',
        'XDZIP_RAR290_FIXTURE_ROOT', 'XDZIP_RAR26_FIXTURE',
    )

    def run_release(self, case):
        root = Path(__file__).resolve().parents[1]
        temporary = tempfile.TemporaryDirectory(dir=root/'build', prefix='release-input-test-')
        self.addCleanup(temporary.cleanup)
        directory = Path(temporary.name).resolve()
        (directory/'scripts').mkdir()
        shutil.copyfile(root/'scripts/ci-release.ps1', directory/'scripts/ci-release.ps1')
        (directory/'scripts/ci-desktop.ps1').write_text('$global:LASTEXITCODE = 0\n', encoding='utf-8')
        capture = directory/'capture.py'
        capture.write_text('''import json, os
from pathlib import Path
import sys
root = Path.cwd()
names = json.loads(os.environ['XDZIP_RELEASE_TEST_NAMES'])
args = sys.argv[1:]
stage = 'after' if args == ['--after'] else (args[1] if args[0] == 'scripts/desktop.py' else args[0])
with (root/'calls.jsonl').open('a', encoding='utf-8') as stream:
    stream.write(json.dumps(dict(stage=stage, args=args,
        environment={name: os.environ.get(name) for name in names}))+'\\n')
if stage == 'scripts/prepare-rar-rust-fixtures.py':
    if os.environ['XDZIP_RELEASE_TEST_CASE'] == 'prepare-failure':
        raise SystemExit(17)
    output = root/args[args.index('--output')+1]
    output.mkdir(parents=True)
    environment = {}
    for index, name in enumerate(names[:-1]):
        path = output/(str(index)+'.rar')
        path.write_bytes(b'fixed fixture')
        environment[name] = str(path.relative_to(root))
    (output/'sources.json').write_text(json.dumps(dict(status='prepared', environment=environment)))
elif stage == 'test' and os.environ['XDZIP_RELEASE_TEST_CASE'] == 'test-failure':
    raise SystemExit(23)
''', encoding='utf-8')

        def literal(value):
            return "'"+str(value).replace("'", "''")+"'"

        harness = directory/'harness.ps1'
        harness.write_text('''$ErrorActionPreference = 'Stop'
# The commands are fixtures; exercise the real Mac orchestration on every host.
Set-Variable IsMacOS -Value $true -Force -Scope Global
Set-Variable IsWindows -Value $false -Force -Scope Global
Set-Variable IsLinux -Value $false -Force -Scope Global
$nativePython = '''+literal(sys.executable)+'''
$capture = '''+literal(capture)+'''
function python { & $nativePython $capture @args }
$failed = $false
try { & ./scripts/ci-release.ps1 } catch { $failed = $true }
& $nativePython $capture --after
if ($failed) { exit 1 }
''', encoding='utf-8')
        environment = dict(os.environ)
        names = [*self.fixture_names, 'XDZIP_RAR_FIXTURE_PASSWORD']
        for name in [*names, 'XDZIP_UPDATE_SIGNING_KEY_HEX', 'XDZIP_SPARKLE_SIGNING_KEY_HEX']:
            environment.pop(name, None)
        original = {self.fixture_names[0]: 'original fixture value', 'XDZIP_RAR_FIXTURE_PASSWORD': 'original test value'}
        environment.update(original, XDZIP_RELEASE_TEST_NAMES=json.dumps(names), XDZIP_RELEASE_TEST_CASE=case)
        result = subprocess.run([shutil.which('pwsh'), '-NoProfile', '-File', str(harness)],
                                cwd=directory, env=environment, capture_output=True, text=True,
                                encoding='utf-8', errors='replace', timeout=60)
        calls = [json.loads(line) for line in (directory/'calls.jsonl').read_text(encoding='utf-8').splitlines()]
        after = calls[-1]
        self.assertEqual(after['stage'], 'after')
        self.assertEqual(after['environment'], {name: original.get(name) for name in names})
        return result, calls[:-1], directory

    def test_mac_release_tests_receive_all_verified_inputs_and_keep_full_rust_scope(self):
        result, calls, directory = self.run_release('success')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual([call['stage'] for call in calls], [
            'build', 'scripts/prepare-rar-rust-fixtures.py', 'test', 'package', 'verify', 'scripts/prepare-release.py'])
        test = calls[2]
        sources = json.loads((directory/'build/release-rar-fixtures/sources.json').read_text())
        for name, value in sources['environment'].items():
            self.assertEqual(Path(test['environment'][name]), directory/value)
        self.assertEqual(test['environment']['XDZIP_RAR_FIXTURE_PASSWORD'], 'password')
        self.assertNotIn('--skip-rust', test['args'])
        self.assertIn('--release-distribution', test['args'])
        self.assertEqual(calls[3]['environment'], calls[-1]['environment'])

    def test_fixture_preparation_failure_prevents_tests_and_packages(self):
        result, calls, _ = self.run_release('prepare-failure')
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual([call['stage'] for call in calls], ['build', 'scripts/prepare-rar-rust-fixtures.py'])

    def test_rust_test_failure_restores_environment_and_prevents_packages(self):
        result, calls, _ = self.run_release('test-failure')
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual([call['stage'] for call in calls], ['build', 'scripts/prepare-rar-rust-fixtures.py', 'test'])


if __name__ == '__main__': unittest.main()
