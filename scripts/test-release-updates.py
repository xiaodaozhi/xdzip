# scripts/test-release-updates.py

"""Exercise exact-byte publisher signatures and single-candidate signed feeds."""

import base64
import copy
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch
import xml.etree.ElementTree as ET

from release_protocol import TARGETS, compare_versions, create, release_tag_eligible, semantic_version, sha256
from release_updates import ROOT, SPARKLE, appcast_bytes, build_signing_tool, prepare_appcast, run_tool, sign_manifest, verify_release_updates


class UpdatePublishingTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tool = ROOT/'target/debug'/('xdzip-update-tool.exe' if os.name == 'nt' else 'xdzip-update-tool')
        subprocess.run(['cargo', 'build', '--locked', '--quiet', '-p', 'xdzip-update', '--bin', 'xdzip-update-tool'],
                       cwd=ROOT, check=True)

    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.directory = Path(self.temporary.name)
        self.version = dict(display_version='0.1.0-alpha.7', channel='alpha', build=20)
        for suffix, _ in TARGETS.values():
            (self.directory/('xdzip-0.1.0-alpha.7-'+suffix)).write_bytes(b'owned native package fixture')
        self.manifest = create(self.directory, self.version, 'xiaodaozhi/xdzip', 'a'*40)

    def metadata(self):
        asset = next(a for a in self.manifest['assets'] if a['package_type'] == 'app.zip')
        return dict(schema_version=1, version=self.manifest['version'], build=self.manifest['build'],
                    filename=asset['filename'], bytes=asset['bytes'], sha256=asset['sha256'],
                    ed_signature=base64.b64encode(bytes(64)).decode())

    def test_shared_python_rust_selection_vectors(self):
        cases = json.loads((ROOT/'tests/fixtures/update/selection-cases.json').read_bytes())
        for left, right, expected in cases['comparisons']:
            self.assertEqual(compare_versions(left, right), expected)
        for tag, current, channel, expected in cases['eligibility']:
            self.assertEqual(release_tag_eligible(tag, current, channel), expected)

    def test_build_subprocess_never_receives_signing_or_publishing_authority(self):
        build_signing_tool.cache_clear()
        self.addCleanup(build_signing_tool.cache_clear)
        private_names = (
            'XDZIP_UPDATE_SIGNING_KEY_HEX', 'XDZIP_SPARKLE_SIGNING_KEY_HEX',
            'GH_TOKEN', 'GITHUB_TOKEN', 'GH_ENTERPRISE_TOKEN', 'GITHUB_ENTERPRISE_TOKEN',
            'GIT_ASKPASS', 'SSH_ASKPASS', 'SSH_ASKPASS_REQUIRE', 'SSH_AUTH_SOCK', 'SSH_AGENT_PID',
            'GIT_SSH', 'GIT_SSH_COMMAND', 'GIT_CONFIG', 'GIT_CONFIG_PARAMETERS',
            'GIT_CONFIG_COUNT', 'GIT_CONFIG_KEY_0', 'GIT_CONFIG_VALUE_0',
            'CARGO_REGISTRY_TOKEN', 'CARGO_REGISTRIES_EXAMPLE_TOKEN',
        )
        private_environment = dict.fromkeys(private_names, 'fixture-only')
        build_environment = {'CARGO_HOME': 'fixture-cargo-home', 'RUSTUP_HOME': 'fixture-rustup-home',
                             'CARGO_TARGET_DIR': 'fixture-target', 'RUSTFLAGS': '--cfg fixture',
                             'CC': 'fixture-cc', 'CXX': 'fixture-cxx'}
        with patch.dict(os.environ, private_environment | build_environment), \
             patch('release_updates.subprocess.run', return_value=subprocess.CompletedProcess([], 0)) as run:
            parent_environment = os.environ.copy()
            build_signing_tool()
            self.assertEqual(dict(os.environ), parent_environment)
        environment = run.call_args.kwargs['env']
        for name in private_names:
            with self.subTest(name=name):
                self.assertNotIn(name, environment)
        for name, value in build_environment.items():
            self.assertEqual(environment[name], value)
        self.assertEqual(environment.get('PATH'), parent_environment.get('PATH'))

    def test_candidate_content_and_metadata_mutations(self):
        content = appcast_bytes(self.manifest, self.metadata())
        root = ET.fromstring(content)
        items = root.findall('./channel/item')
        self.assertEqual(len(items), 1)
        self.assertEqual(items[0].find(f'{{{SPARKLE}}}version').text, '20')
        self.assertEqual(items[0].find(f'{{{SPARKLE}}}channel').text, 'alpha')
        self.assertIsNone(items[0].find(f'{{{SPARKLE}}}releaseNotesLink'))
        for field, value in [('build', 19), ('version', '0.1.0-alpha.6'), ('bytes', 100),
                             ('filename', '../archive.zip'), ('ed_signature', 'invalid')]:
            metadata = self.metadata()
            metadata[field] = value
            with self.subTest(field=field), self.assertRaises(ValueError):
                appcast_bytes(self.manifest, metadata)
        self.assertLess(semantic_version('0.1.0-dev.99', allow_development=True), semantic_version('0.1.0-alpha.1'))

    def test_unconfigured_validation_is_explicit_and_publication_fails(self):
        with patch.dict(os.environ, {'XDZIP_UPDATE_SIGNING_KEY_HEX': '', 'XDZIP_SPARKLE_SIGNING_KEY_HEX': ''}):
            self.assertFalse(prepare_appcast(self.directory, self.manifest, tool=self.tool))
            self.assertFalse(sign_manifest(self.directory, key_id=None, tool=self.tool))
            with self.assertRaises(ValueError):
                prepare_appcast(self.directory, self.manifest, required=True, tool=self.tool)
            with self.assertRaises(ValueError):
                sign_manifest(self.directory, key_id=None, required=True, tool=self.tool)
            with self.assertRaises(ValueError):
                verify_release_updates(self.directory, self.manifest, tool=self.tool)

    def test_independent_keys_sign_archives_feeds_and_exact_manifest_bytes(self):
        # Deterministic disposable test keys are never written to production resources.
        environment = {'XDZIP_UPDATE_SIGNING_KEY_HEX': '2a'*32, 'XDZIP_SPARKLE_SIGNING_KEY_HEX': '2b'*32}
        keyring = self.directory/'trust.json'
        sparkle_key = self.directory/'sparkle-key.json'
        archive = next(a for a in self.manifest['assets'] if a['package_type'] == 'app.zip')
        with patch.dict(os.environ, environment):
            run_tool(['public-key', '--key-id', 'fixture-only', '--output', str(keyring)], self.tool)
            run_tool(['sparkle-public-key', '--output', str(sparkle_key)], self.tool)
            run_tool(['sign-archive', '--archive', str(self.directory/archive['filename']),
                      '--version', self.manifest['version'], '--build', str(self.manifest['build']),
                      '--public-key', str(sparkle_key), '--output', str(self.directory/'macos-update.json')], self.tool)
            self.assertTrue(prepare_appcast(self.directory, self.manifest, required=True, tool=self.tool, public_key=sparkle_key))
            manifest_file = self.directory/'release.json'
            manifest_file.write_text(json.dumps(self.manifest)+'\n', encoding='utf-8')
            self.assertTrue(sign_manifest(self.directory, key_id='fixture-only', required=True, tool=self.tool, keyring=keyring))
            verify_release_updates(self.directory, self.manifest, tool=self.tool, keyring=keyring, public_key=sparkle_key)
            run_tool(['verify', '--manifest', str(manifest_file), '--signature', str(self.directory/'release.json.sig'),
                      '--keyring', str(keyring)], self.tool)
            signed_feed = self.directory/'appcast.xml'
            self.assertIn(b'<!-- sparkle-signatures:', signed_feed.read_bytes())
            original_feed = signed_feed.read_bytes()
            signed_feed.write_bytes(original_feed.replace(b'xdzip updates', b'other updates'))
            with self.assertRaises(ValueError):
                run_tool(['verify-appcast', '--appcast', str(signed_feed), '--public-key', str(sparkle_key)], self.tool)
            signed_feed.write_bytes(original_feed)
            manifest_file.write_bytes(manifest_file.read_bytes()+b' ')
            with self.assertRaises(ValueError):
                run_tool(['verify', '--manifest', str(manifest_file), '--signature', str(self.directory/'release.json.sig'),
                          '--keyring', str(keyring)], self.tool)
            (self.directory/archive['filename']).write_bytes(b'mutated native package')
            with self.assertRaises(ValueError):
                prepare_appcast(self.directory, self.manifest, required=True, tool=self.tool, public_key=sparkle_key)


if __name__ == '__main__':
    unittest.main()
