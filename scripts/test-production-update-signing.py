"""Exercise bounded production-signing admission with isolated test public roots."""

import base64
from contextlib import redirect_stdout
import copy
import importlib.util
import io
import json
import os
from pathlib import Path
import plistlib
import shutil
import subprocess
import tempfile
import unittest
from unittest.mock import patch
import zipfile

from release_protocol import TARGETS


ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location('production_fixture_signer', ROOT / 'scripts/sign-production-update-fixtures.py')
SIGN = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(SIGN)
TEST_SECRETS = {'XDZIP_UPDATE_SIGNING_KEY_HEX': '2a' * 32,
                'XDZIP_SPARKLE_SIGNING_KEY_HEX': '2b' * 32,
                'XDZIP_UPDATE_KEY_ID': 'xdzip-release-1'}


def dump(path, value):
    path.write_text(json.dumps(value, sort_keys=True) + '\n', encoding='utf-8')


class ProductionSigningTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tool = ROOT / 'target/debug' / ('xdzip-update-tool.exe' if os.name == 'nt' else 'xdzip-update-tool')
        environment = os.environ.copy()
        for name in SIGN.PRIVATE:
            environment.pop(name, None)
        subprocess.run(['cargo', 'build', '--locked', '--quiet', '-p', 'xdzip-update',
                        '--bin', 'xdzip-update-tool'], cwd=ROOT, env=environment, check=True)

    def setUp(self):
        # Generated evidence remains under build/. Public roots are a disposable
        # copy; neither production resources nor a production secret are touched.
        (ROOT / 'build').mkdir(exist_ok=True)
        self.temporary = tempfile.TemporaryDirectory(prefix='production-signing-contract-', dir=ROOT / 'build')
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        (self.root / 'resources').mkdir()
        (self.root / 'build').mkdir()
        self.environment = patch.dict(os.environ, TEST_SECRETS)
        self.environment.start()
        self.addCleanup(self.environment.stop)
        for command, name, secret in (
                ('public-key', 'update-trust.json', SIGN.PRIVATE[0]),
                ('sparkle-public-key', 'sparkle-update-key.json', SIGN.PRIVATE[1])):
            arguments = [str(self.tool), command, '--output', str(self.root / 'resources' / name)]
            if command == 'public-key':
                arguments += ['--key-id', SIGN.KEY_ID]
            subprocess.run(arguments, env=SIGN.clean_environment(secret), check=True, capture_output=True)
        root_patch = patch.object(SIGN, 'ROOT', self.root)
        root_patch.start()
        self.addCleanup(root_patch.stop)
        self.source_patch = patch.object(SIGN, 'source_commit', return_value='c' * 40)
        self.source_patch.start()
        self.addCleanup(self.source_patch.stop)

    def prepare(self, platform='windows'):
        directory = self.root / 'build' / platform
        directory.mkdir()
        (directory / 'resources').mkdir()
        (directory / 'signing').mkdir()
        for name in SIGN.PUBLIC_FILES:
            shutil.copy2(self.root / 'resources' / name, directory / 'resources' / name)
        # Windows CreateProcess accepts a native PE with this extensionless name
        # as well, so the Mac archive contract is exercised on every test host.
        tool = directory / 'signing' / ('xdzip-update-tool.exe' if platform == 'windows' else 'xdzip-update-tool')
        shutil.copy2(self.tool, tool)
        products = {}
        for label, commit in (('old', 'a' * 40), ('new', 'b' * 40)):
            products[label] = dict(commit=commit, identity=dict(version='0.2.0-alpha.' + ('1' if label == 'old' else '2'),
                                    build=200 if label == 'old' else 201, revision=commit[:12], dirty=False))
        if platform != 'macos':
            products['failure'] = copy.deepcopy(products['new'])
        paths = [tool, *(directory / 'resources' / name for name in SIGN.PUBLIC_FILES)]
        if platform == 'macos':
            app_name = 'xdzip-owned-update-更新-' + 'd' * 32 + '.app'
            public = json.loads((self.root / 'resources/sparkle-update-key.json').read_bytes())['public_key']
            for label, value in products.items():
                app = directory / label / app_name
                (app / 'Contents/MacOS').mkdir(parents=True)
                plist = app / 'Contents/Info.plist'
                plist.write_bytes(plistlib.dumps(dict(SUPublicEDKey=base64.b64encode(bytes.fromhex(public)).decode(),
                    CFBundleIdentifier='org.xdzip.desktop', CFBundleShortVersionString='0.2.0',
                    CFBundleVersion=str(value['identity']['build']), XdzipUpdateInstallable=True)))
                binary = app / 'Contents/MacOS/xdzip'
                binary.write_bytes(('isolated fixture ' + label).encode())
                value.update(app=app.relative_to(directory).as_posix(), SUPublicEDKey=base64.b64encode(bytes.fromhex(public)).decode(),
                             binary_sha256=SIGN.digest(binary), plist_sha256=SIGN.digest(plist))
            (directory / 'owned').mkdir()
            package = directory / 'owned/xdzip-0.2.0-alpha.2-macos-arm64.app.zip'
            with zipfile.ZipFile(package, 'w') as archive:
                app = directory / products['new']['app']
                for path in app.rglob('*'):
                    if path.is_file():
                        archive.write(path, app_name + '/' + path.relative_to(app).as_posix())
            self.manifest(package, platform)
            paths += [package, package.parent / 'release.json']
        else:
            for label in ('new', 'failure'):
                folder = directory / label
                folder.mkdir()
                suffix = 'windows-x86_64-setup.exe' if platform == 'windows' else 'ubuntu24.04-x86_64.deb'
                package = folder / ('xdzip-0.2.0-alpha.2-' + suffix)
                package.write_bytes(('isolated fixture ' + label).encode())
                self.manifest(package, platform)
                paths += [package, folder / 'release.json']
        record = dict(schema_version=1, platform=platform, source_commit='c' * 40,
                      tool=dict(path=tool.relative_to(directory).as_posix(), sha256=SIGN.digest(tool)),
                      artifacts={path.relative_to(directory).as_posix(): dict(bytes=path.stat().st_size, sha256=SIGN.digest(path))
                                 for path in paths}, products=products)
        dump(directory / SIGN.PREPARED, record)
        return directory, record

    def manifest(self, package, platform):
        value = dict(schema_version=1, product='xdzip', version=SIGN.VERSION, channel='alpha', build=SIGN.BUILD,
                     tag='v' + SIGN.VERSION, commit='b' * 40, repository='xiaodaozhi/xdzip', assets=[])
        for (system, architecture, kind), (suffix, minimum) in TARGETS.items():
            selected = system == platform and kind == {'windows': 'exe', 'linux': 'deb', 'macos': 'app.zip'}[platform]
            name = 'xdzip-' + SIGN.VERSION + '-' + suffix
            value['assets'].append(dict(filename=name, platform=system, architecture=architecture, package_type=kind,
                minimum_system=minimum, bytes=package.stat().st_size if selected else 1,
                sha256=SIGN.digest(package) if selected else SIGN.hashlib.sha256(b'x').hexdigest(),
                url='https://github.com/xiaodaozhi/xdzip/releases/download/v' + SIGN.VERSION + '/' + name))
        if platform == 'macos':
            value['macos_appcast_url'] = 'https://github.com/xiaodaozhi/xdzip/releases/download/v' + SIGN.VERSION + '/appcast.xml'
        dump(package.parent / 'release.json', value)

    def rewrite_record(self, directory, record):
        dump(directory / SIGN.PREPARED, record)

    def refresh_artifact(self, directory, record, name):
        path = directory / name
        record['artifacts'][name] = dict(bytes=path.stat().st_size, sha256=SIGN.digest(path))
        self.rewrite_record(directory, record)

    def test_actual_independent_keys_and_two_manifests_sign_and_reverify(self):
        directory, _ = self.prepare()
        report = SIGN.sign_fixture(directory)
        self.assertEqual(report['status'], 'passed')
        self.assertEqual(len(report['signatures']), 2)
        for folder in ('new', 'failure'):
            envelope = json.loads((directory / folder / 'release.json.sig').read_bytes())
            self.assertEqual(envelope['key_id'], SIGN.KEY_ID)
        # Exact-byte mutations fail independently, even after successful signing.
        manifest = directory / 'new/release.json'
        manifest.write_bytes(manifest.read_bytes() + b' ')
        tool = SIGN.Tool(directory / report_tool_path(), report['signer_sha256'])
        with self.assertRaisesRegex(SIGN.AdmissionError, 'compiled_signing_tool_failed'):
            tool.run(['verify', '--manifest', manifest, '--signature', directory / 'new/release.json.sig',
                      '--keyring', directory / 'resources/update-trust.json'])

    def test_actual_mac_archive_feed_and_manifest_are_independently_signed(self):
        directory, _ = self.prepare('macos')
        report = SIGN.sign_fixture(directory)
        self.assertEqual(len(report['signatures']), 3)
        self.assertIn(b'<!-- sparkle-signatures:', (directory / 'owned/appcast.xml').read_bytes())
        self.assertEqual(report['public_key_match']['general_public_key_matched'], True)
        self.assertEqual(report['public_key_match']['sparkle_public_key_matched'], True)

    def test_actual_linux_prepared_packages_sign_and_reverify(self):
        directory, _ = self.prepare('linux')
        report = SIGN.sign_fixture(directory)
        self.assertEqual(report['platform'], 'linux')
        self.assertEqual(len(report['signatures']), 2)
        self.assertTrue(all(value['signature_verified'] for value in report['signatures']))

    def test_check_only_derives_public_values_without_signing_payload(self):
        tool = self.root / 'build' / self.tool.name
        shutil.copy2(self.tool, tool)
        output = self.root / 'build/keys'
        report = SIGN.check_only(tool, output)
        self.assertFalse(report['signing_performed'])
        self.assertEqual(sorted(path.name for path in output.iterdir()), ['production-key-match.json'])

    def test_wrong_private_key_or_missing_or_wrong_key_id_prevents_all_signatures(self):
        for name, value in ((SIGN.PRIVATE[0], '2c' * 32), (SIGN.PRIVATE[1], '2c' * 32),
                            (SIGN.PRIVATE[0], ''), ('XDZIP_UPDATE_KEY_ID', 'other-key')):
            with self.subTest(name=name, value_is_empty=not value):
                directory, _ = self.prepare()
                with patch.dict(os.environ, {name: value}), self.assertRaises(SIGN.AdmissionError):
                    SIGN.sign_fixture(directory)
                self.assertFalse(list(directory.rglob('*.sig')))
                shutil.rmtree(directory)

    def test_all_subprocesses_receive_only_required_key_and_no_publishing_authority(self):
        directory, _ = self.prepare()
        original = subprocess.run
        seen = []
        def observe(arguments, **kwargs):
            seen.append((arguments[1], kwargs['env'].copy()))
            return original(arguments, **kwargs)
        with patch.dict(os.environ, {'GITHUB_TOKEN': 'must-not-inherit', 'LD_PRELOAD': 'must-not-inherit',
                                    'CARGO_REGISTRY_TOKEN': 'must-not-inherit'}), \
                patch.object(SIGN.subprocess, 'run', side_effect=observe):
            SIGN.sign_fixture(directory)
        for command, environment in seen:
            self.assertNotIn('GITHUB_TOKEN', environment)
            self.assertNotIn('LD_PRELOAD', environment)
            self.assertNotIn('CARGO_REGISTRY_TOKEN', environment)
            keys = set(environment) & set(SIGN.PRIVATE)
            self.assertEqual(keys, {SIGN.PRIVATE[0]} if command in ('public-key', 'sign') else
                             {SIGN.PRIVATE[1]} if command == 'sparkle-public-key' else set())
            self.assertNotIn(command, ('cargo', 'cmake', 'cc', 'cl'))

    def test_changed_package_manifest_signer_or_public_resource_rejected_before_private_use(self):
        for name in ('new/xdzip-0.2.0-alpha.2-windows-x86_64-setup.exe', 'new/release.json',
                     'signing/xdzip-update-tool.exe', 'resources/update-trust.json'):
            with self.subTest(name=name):
                directory, _ = self.prepare()
                path = directory / name
                path.write_bytes(path.read_bytes() + b' ')
                with patch.object(SIGN.Tool, 'run') as run, self.assertRaises(SIGN.AdmissionError):
                    SIGN.sign_fixture(directory)
                run.assert_not_called()
                shutil.rmtree(directory)

    def test_refreshed_manifest_cannot_sign_unrelated_version_commit_or_asset(self):
        for field, replacement in (('version', '9.0.0'), ('commit', 'd' * 40), ('repository', 'example/other')):
            with self.subTest(field=field):
                directory, record = self.prepare()
                path = directory / 'new/release.json'
                value = json.loads(path.read_bytes())
                value[field] = replacement
                dump(path, value)
                self.refresh_artifact(directory, record, 'new/release.json')
                with patch.object(SIGN.Tool, 'run') as run, self.assertRaises((SIGN.AdmissionError, ValueError)):
                    SIGN.sign_fixture(directory)
                run.assert_not_called()
                shutil.rmtree(directory)

    def test_traversal_duplicate_json_and_existing_signature_rejected(self):
        directory, record = self.prepare()
        record['artifacts']['../outside'] = dict(bytes=1, sha256='0' * 64)
        self.rewrite_record(directory, record)
        with self.assertRaises(SIGN.AdmissionError):
            SIGN.sign_fixture(directory)
        path = directory / SIGN.PREPARED
        path.write_text('{"schema_version":1,"schema_version":1}', encoding='utf-8')
        with self.assertRaisesRegex(SIGN.AdmissionError, 'duplicate_json_key'):
            SIGN.sign_fixture(directory)
        shutil.rmtree(directory)
        directory, _ = self.prepare()
        (directory / 'new/release.json.sig').write_bytes(b'existing')
        with patch.object(SIGN.Tool, 'run') as run, self.assertRaisesRegex(SIGN.AdmissionError, 'signing_output_exists'):
            SIGN.sign_fixture(directory)
        run.assert_not_called()

    def test_source_and_actual_embedded_mac_key_are_bound(self):
        directory, record = self.prepare('macos')
        record['source_commit'] = 'e' * 40
        self.rewrite_record(directory, record)
        with self.assertRaisesRegex(SIGN.AdmissionError, 'prepared_source_mismatch'):
            SIGN.sign_fixture(directory)
        record['source_commit'] = 'c' * 40
        plist = directory / record['products']['new']['app'] / 'Contents/Info.plist'
        value = plistlib.loads(plist.read_bytes())
        value['SUPublicEDKey'] = base64.b64encode(bytes(32)).decode()
        plist.write_bytes(plistlib.dumps(value))
        record['products']['new']['plist_sha256'] = SIGN.digest(plist)
        self.rewrite_record(directory, record)
        with patch.object(SIGN.Tool, 'run') as run, self.assertRaisesRegex(SIGN.AdmissionError, 'mac_embedded_sparkle_key_mismatch'):
            SIGN.sign_fixture(directory)
        run.assert_not_called()

    def test_mac_archive_identity_cannot_be_replaced_even_with_refreshed_outer_hash(self):
        directory, record = self.prepare('macos')
        package_name = 'owned/xdzip-0.2.0-alpha.2-macos-arm64.app.zip'
        package = directory / package_name
        app = directory / record['products']['new']['app']
        with zipfile.ZipFile(package, 'w') as archive:
            for path in app.rglob('*'):
                if path.is_file():
                    value = b'changed executable' if path.name == 'xdzip' else path.read_bytes()
                    archive.writestr(app.name + '/' + path.relative_to(app).as_posix(), value)
        manifest = directory / 'owned/release.json'
        value = json.loads(manifest.read_bytes())
        selected = next(a for a in value['assets'] if a['package_type'] == 'app.zip')
        selected.update(bytes=package.stat().st_size, sha256=SIGN.digest(package))
        dump(manifest, value)
        self.refresh_artifact(directory, record, package_name)
        self.refresh_artifact(directory, record, 'owned/release.json')
        with patch.object(SIGN.Tool, 'run') as run, self.assertRaisesRegex(SIGN.AdmissionError, 'mac_archive_product_mismatch'):
            SIGN.sign_fixture(directory)
        run.assert_not_called()

    def test_hardlinked_prepared_inputs_are_rejected_before_key_access(self):
        directory, _ = self.prepare()
        path = directory / 'new/release.json'
        os.link(path, directory / 'manifest-hardlink')
        with patch.object(SIGN.Tool, 'run') as run, self.assertRaisesRegex(SIGN.AdmissionError, 'hardlinked_input_rejected'):
            SIGN.sign_fixture(directory)
        run.assert_not_called()

    def test_prepared_record_changes_during_signing_fail_completion(self):
        directory, _ = self.prepare()
        original = SIGN.Tool.run
        def mutate(tool, arguments, secret=None):
            original(tool, arguments, secret)
            if arguments[0] == 'sign':
                path = directory / SIGN.PREPARED
                path.write_bytes(path.read_bytes() + b' ')
        with patch.object(SIGN.Tool, 'run', new=mutate), \
                self.assertRaisesRegex(SIGN.AdmissionError, 'prepared_record_changed'):
            SIGN.sign_fixture(directory)
        self.assertFalse((directory / 'production-signing.json').exists())

    def test_failing_tool_output_and_os_errors_never_escape_final_json(self):
        directory, _ = self.prepare()
        secret = 'unprintable-private-value'
        result = subprocess.CompletedProcess([], 1, stdout=secret.encode(), stderr=secret.encode())
        output = io.StringIO()
        with patch.object(SIGN.subprocess, 'run', return_value=result), redirect_stdout(output):
            self.assertEqual(SIGN.main(['--directory', str(directory)]), 1)
        self.assertNotIn(secret, output.getvalue())
        self.assertEqual(json.loads(output.getvalue())['error'], 'compiled_signing_tool_failed')
        retained = directory / 'production-signing-result.json'
        self.assertEqual(json.loads(retained.read_bytes()), json.loads(output.getvalue()))
        self.assertNotIn(secret, retained.read_text(encoding='utf-8'))
        original_failure = retained.read_bytes()
        output = io.StringIO()
        with patch.object(SIGN.Tool, 'run', side_effect=OSError(secret)), redirect_stdout(output):
            self.assertEqual(SIGN.main(['--directory', str(directory)]), 1)
        self.assertNotIn(secret, output.getvalue())
        self.assertEqual(json.loads(output.getvalue())['error'], 'production_signing_failed')
        self.assertEqual(retained.read_bytes(), original_failure)

    def test_key_match_failure_retains_public_json_and_does_not_overwrite(self):
        tool = self.root / 'build' / self.tool.name
        shutil.copy2(self.tool, tool)
        output = self.root / 'build/keys'
        stdout = io.StringIO()
        with patch.dict(os.environ, {SIGN.PRIVATE[0]: ''}), redirect_stdout(stdout):
            self.assertEqual(SIGN.main(['--check-only', '--tool', str(tool), '--output', str(output)]), 1)
        retained = output / 'production-key-match.json'
        self.assertEqual(json.loads(retained.read_bytes()), json.loads(stdout.getvalue()))
        self.assertEqual(json.loads(retained.read_bytes())['error'], 'signing_key_missing_or_invalid')
        original = retained.read_bytes()
        with redirect_stdout(io.StringIO()):
            self.assertEqual(SIGN.main(['--check-only', '--tool', str(tool), '--output', str(output)]), 1)
        self.assertEqual(retained.read_bytes(), original)

    def test_failure_record_never_calls_tools_or_git_and_rejects_unsafe_paths(self):
        directory, _ = self.prepare()
        with patch.object(SIGN, 'sign_fixture', side_effect=OSError('private-value-and-user-path')), \
                patch.object(SIGN, 'source_commit', side_effect=AssertionError('must not invoke git')), \
                patch.object(SIGN.Tool, 'run', side_effect=AssertionError('must not invoke tool')), \
                redirect_stdout(io.StringIO()):
            self.assertEqual(SIGN.main(['--directory', str(directory)]), 1)
        retained = directory / 'production-signing-result.json'
        self.assertEqual(json.loads(retained.read_bytes()),
                         {'schema_version': 1, 'status': 'failed', 'error': 'production_signing_failed'})
        unsafe = self.root / 'resources'
        with redirect_stdout(io.StringIO()):
            self.assertEqual(SIGN.main(['--directory', str(unsafe)]), 1)
        self.assertFalse((unsafe / 'production-signing-result.json').exists())

    def test_failure_evidence_write_error_keeps_nonzero_and_suppresses_os_details(self):
        directory, _ = self.prepare()
        output = io.StringIO()
        original_open = Path.open
        def refuse_failure_record(path, *args, **kwargs):
            if path.name == 'production-signing-result.json':
                raise OSError('private-value-and-user-path')
            return original_open(path, *args, **kwargs)
        with patch.object(SIGN, 'sign_fixture', side_effect=SIGN.AdmissionError('compiled_signing_tool_failed')), \
                patch.object(Path, 'open', new=refuse_failure_record), redirect_stdout(output):
            self.assertEqual(SIGN.main(['--directory', str(directory)]), 1)
        self.assertEqual(json.loads(output.getvalue()),
                         {'schema_version': 1, 'status': 'failed', 'error': 'compiled_signing_tool_failed'})
        self.assertNotIn('private-value', output.getvalue())


def report_tool_path():
    return 'signing/xdzip-update-tool.exe'


if __name__ == '__main__':
    unittest.main()
