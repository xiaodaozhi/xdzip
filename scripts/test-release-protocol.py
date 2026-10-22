# scripts/test-release-protocol.py

import copy
from contextlib import redirect_stdout
import importlib.util
import io
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import tarfile
import unittest
from unittest.mock import patch
from release_protocol import TARGETS, create, select, validate

SPEC = importlib.util.spec_from_file_location('publish_release', Path(__file__).with_name('publish-release.py'))
PUBLISH = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PUBLISH)
PREPARE_SPEC = importlib.util.spec_from_file_location('prepare_release', Path(__file__).with_name('prepare-release.py'))
PREPARE = importlib.util.module_from_spec(PREPARE_SPEC)
PREPARE_SPEC.loader.exec_module(PREPARE)


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


if __name__ == '__main__': unittest.main()
