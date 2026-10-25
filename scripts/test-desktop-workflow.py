#!/usr/bin/env python3
"""Cross-platform orchestration contracts, without installing or deploying tools."""
import contextlib
import io
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import desktop


class WorkflowTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve()
        self.addCleanup(patch.stopall)
        # Tests simulating a local Mac must not inherit the host runner's CI
        # policy. The explicit CI guard cases below set CI=true themselves.
        patch.dict(os.environ, {'CI': ''}).start()
        patch.object(desktop, 'ROOT', self.root).start()
        self.qt = self.root/'sdk'
        for name in ['qmake', 'qmake.exe']:
            path = self.qt/'bin'/name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(b'pinned qmake fixture')
        self.args = desktop.parser().parse_args([
            'build', '--qt-prefix', str(self.qt),
            '--build-dir', str(self.root/'build/native'),
            '--output-dir', str(self.root/'build/package'),
            '--tools-dir', str(self.root/'tools'),
        ])
        self.source = {'revision': 'a'*40, 'dirty': False, 'fingerprint': 'b'*64}

    def workflow(self, system='macos'):
        return desktop.Workflow(self.args, system=system)

    def receipt(self, workflow):
        workflow.build.mkdir(parents=True, exist_ok=True)
        workflow.app.parent.mkdir(parents=True, exist_ok=True)
        workflow.app.write_bytes(b'product')
        (workflow.build/'CMakeCache.txt').write_bytes(b'compiler + qt + settings')
        receipt = {'status': 'passed', 'source': self.source, 'platform': workflow.system,
                   'configuration': 'Release', 'qt_sdk': workflow.sdk_identity(),
                   'product_sha256': desktop.digest(workflow.app),
                   'cmake_cache_sha256': desktop.digest(workflow.build/'CMakeCache.txt'),
                   'macos_handler_rank': 'Alternate'}
        (workflow.build/'desktop-build.json').write_text(json.dumps(receipt))
        (workflow.build/'desktop-test.json').write_text(json.dumps({
            'status': 'passed', 'source': self.source,
            'product_sha256': receipt['product_sha256']}))
        return receipt

    def test_no_ci_environment_is_required(self):
        with patch.dict(os.environ, {}, clear=True):
            workflow = self.workflow('linux')
        self.assertEqual(workflow.args.configuration, 'Release')
        self.assertEqual(workflow.build, self.root/'build/native')
        self.assertEqual(workflow.runtime, self.root/'build/package/linux')
        self.assertNotIn('RUNNER_TEMP', workflow.env)

    def test_explicit_qt_prefix_wins_over_environment(self):
        with patch.dict(os.environ, {'XDZIP_QT_PREFIX': '/other-sdk'}):
            self.assertEqual(self.workflow().qt, self.qt)

    def test_outputs_cannot_target_source_or_parent_directories(self):
        for directory in [self.root, self.root/'docs', self.root/'build', self.root/'dist/package', self.root.parent/'escape']:
            with self.subTest(directory=directory), self.assertRaises(ValueError):
                desktop.generated_path(directory)
        self.assertEqual(desktop.generated_path(self.root/'build/owned'), self.root/'build/owned')

    def test_test_requires_shared_build_and_rejects_mutated_inputs(self):
        workflow = self.workflow()
        with self.assertRaisesRegex(ValueError, 'shared build'):
            workflow.build_receipt()
        self.receipt(workflow)
        with patch.object(workflow, 'source', return_value=self.source):
            workflow.build_receipt()
            workflow.app.write_bytes(b'changed product')
            with self.assertRaisesRegex(ValueError, 'Product differs'):
                workflow.build_receipt()
        self.receipt(workflow)
        with patch.object(workflow, 'source', return_value={**self.source, 'dirty': True}):
            with self.assertRaisesRegex(ValueError, 'Source differs'):
                workflow.build_receipt()
        with patch.object(workflow, 'source', return_value=self.source):
            (workflow.build/'CMakeCache.txt').write_bytes(b'new compiler')
            with self.assertRaisesRegex(ValueError, 'configuration changed'):
                workflow.build_receipt()

    def test_build_receipt_rejects_different_sdk_or_configuration(self):
        workflow = self.workflow('linux')
        self.receipt(workflow)
        with patch.object(workflow, 'source', return_value=self.source):
            (self.qt/'bin/qmake').write_bytes(b'different qmake')
            with self.assertRaisesRegex(ValueError, 'Qt SDK differs'):
                workflow.build_receipt()
            self.receipt(workflow)
            workflow.args.configuration = 'Debug'
            with self.assertRaisesRegex(ValueError, 'configuration or native platform'):
                workflow.build_receipt()

    def test_cached_cmake_sdk_cannot_silently_use_a_different_prefix(self):
        workflow = self.workflow('linux')
        workflow.build.mkdir(parents=True)
        cache = workflow.build/'CMakeCache.txt'
        cache.write_text('Qt6_DIR:PATH='+str(self.qt/'lib/cmake/Qt6')+'\n')
        workflow.check_cached_sdk(configured=True)
        cache.write_text('Qt6_DIR:PATH='+str(self.root/'other-sdk/lib/cmake/Qt6')+'\n')
        with self.assertRaisesRegex(ValueError, 'different Qt SDK'):
            workflow.check_cached_sdk()
        cache.write_text('CMAKE_BUILD_TYPE:STRING=Release\n')
        with self.assertRaisesRegex(ValueError, 'does not declare'):
            workflow.check_cached_sdk(configured=True)

    def test_macos_ci_blocks_packaging_before_any_tool_or_copy(self):
        workflow = self.workflow()
        with patch.dict(os.environ, {'CI': 'true'}), patch.object(workflow, 'doctor') as doctor:
            with self.assertRaisesRegex(ValueError, 'must not deploy'):
                workflow.package()
            doctor.assert_not_called()
        with patch.dict(os.environ, {'CI': 'true'}), patch.object(workflow, 'doctor') as doctor:
            with self.assertRaisesRegex(ValueError, 'does not validate'):
                workflow.verify()
            doctor.assert_not_called()

    def test_package_rejects_dirty_source_before_licensing_or_deployment(self):
        workflow = self.workflow('windows')
        with patch.object(workflow, 'source', return_value={**self.source, 'dirty': True}), \
                patch.object(workflow, 'script') as script:
            with self.assertRaisesRegex(ValueError, 'clean source'):
                workflow.package()
            script.assert_not_called()
        self.assertFalse(workflow.runtime.exists())

    def test_windows_debug_package_fails_before_deployment(self):
        workflow = self.workflow('windows')
        workflow.args.configuration = 'Debug'
        with patch.object(workflow, 'script') as script:
            with self.assertRaisesRegex(ValueError, 'Debug Qt deployment'):
                workflow.package_preflight()
            script.assert_not_called()

    def test_missing_environment_blocks_packaging_before_any_copy(self):
        workflow = self.workflow('linux')
        self.receipt(workflow)
        failed = {'status': 'failed', 'checks': [{'name': 'tool.cargo', 'status': 'failed', 'hint': 'Restore Rust PATH'}]}
        with patch.object(workflow, 'source', return_value=self.source), \
                patch.object(desktop, 'check_environment', return_value=failed), \
                patch.object(workflow, 'script') as script, contextlib.redirect_stdout(io.StringIO()):
            with self.assertRaisesRegex(ValueError, 'preflight failed'):
                workflow.package()
            script.assert_not_called()
        self.assertFalse(workflow.runtime.exists())

    def test_macos_package_requires_explicit_alternate_build(self):
        workflow = self.workflow()
        receipt = self.receipt(workflow)
        receipt['macos_handler_rank'] = 'None'
        (workflow.build/'desktop-build.json').write_text(json.dumps(receipt))
        with patch.object(workflow, 'source', return_value=self.source), \
                patch.object(workflow, 'doctor'), patch.object(workflow, 'script') as script:
            with self.assertRaisesRegex(ValueError, 'Alternate'):
                workflow.package()
            script.assert_not_called()

    def test_package_requires_tests_from_the_same_build(self):
        workflow = self.workflow('linux')
        self.receipt(workflow)
        (workflow.build/'desktop-test.json').unlink()
        with patch.object(workflow, 'source', return_value=self.source), patch.object(workflow, 'doctor') as doctor:
            with self.assertRaisesRegex(ValueError, 'Run test'):
                workflow.package_preflight()
            doctor.assert_not_called()

    def test_deployment_audit_cannot_resolve_runtime_from_sdk_environment(self):
        workflow = self.workflow('linux')
        workflow.env.update(QT_PLUGIN_PATH=str(self.qt/'plugins'), QML_IMPORT_PATH=str(self.qt/'qml'))
        with patch.object(workflow, 'script') as script:
            workflow.audit_runtime()
        for call in script.call_args_list:
            env = call.kwargs['env']
            for name in ['LD_LIBRARY_PATH', 'QT_PLUGIN_PATH', 'QML_IMPORT_PATH']:
                self.assertNotIn(name, env)
            self.assertNotIn(str(self.qt/'bin'), env['PATH'].split(os.pathsep))

    def test_windows_notice_merge_preserves_deployer_runtime_materials(self):
        workflow = self.workflow('windows')
        self.receipt(workflow)
        (workflow.app.parent/'xdzip-update-helper.exe').write_bytes(b'updater fixture')
        (workflow.app.parent/'xdzip-native-codec.exe').write_bytes(b'codec fixture')
        notices = workflow.output/'notices'
        notices.mkdir(parents=True)
        (notices/'source-license.txt').write_text('source')
        icon = self.root/'design/assets/brand/xdzip-zip.ico'
        icon.parent.mkdir(parents=True)
        icon.write_bytes(b'fixture icon')

        def deploy(name, *args):
            self.assertEqual(name, 'deploy_windows.py')
            crt = workflow.runtime/'share/notices/Microsoft-VC-runtime'
            crt.mkdir(parents=True)
            (crt/'LICENSE.txt').write_text('CRT notice')
            sdk = crt.parent/'Windows-SDK-graphics'
            sdk.mkdir()
            (sdk/'sdk_license.rtf').write_bytes(b'original SDK terms')
            (sdk/'provenance.json').write_bytes(b'{"source":"SDK REDIST"}')
            (crt.parent/'Windows-runtime-NOTICE.txt').write_bytes(b'\xef\xbb\xbfbilingual notice')

        def audit():
            (workflow.output/'deployment-audit.json').write_text('{}')

        with patch.object(workflow, 'notices', return_value=notices), patch.object(workflow, 'script', side_effect=deploy), \
                patch.object(workflow, 'audit_runtime', side_effect=audit), patch.object(workflow, 'source', return_value=self.source):
            workflow.runtime_package()
        self.assertEqual((workflow.runtime/'share/notices/source-license.txt').read_text(), 'source')
        self.assertEqual((workflow.runtime/'xdzip-update-helper.exe').read_bytes(), b'updater fixture')
        self.assertEqual((workflow.runtime/'xdzip-native-codec.exe').read_bytes(), b'codec fixture')
        self.assertEqual((workflow.runtime/'share/notices/Microsoft-VC-runtime/LICENSE.txt').read_text(), 'CRT notice')
        for relative in ['Microsoft-VC-runtime/LICENSE.txt', 'Windows-SDK-graphics/sdk_license.rtf',
                         'Windows-SDK-graphics/provenance.json', 'Windows-runtime-NOTICE.txt']:
            self.assertEqual((notices/relative).read_bytes(), (workflow.runtime/'share/notices'/relative).read_bytes())

    def test_test_is_offscreen_and_never_implicitly_builds_or_installs(self):
        workflow = self.workflow('linux')
        self.receipt(workflow)
        with patch.object(workflow, 'source', return_value=self.source), \
                patch.object(workflow, 'doctor'), patch.object(workflow, 'run') as run:
            result = workflow.test()
        commands = [call.args[0] for call in run.call_args_list]
        self.assertIn(['cargo', 'test', '--workspace', '--locked'], commands)
        self.assertTrue(any(command[0] == 'ctest' for command in commands))
        self.assertTrue(all('--no-tests=error' in command for command in commands if command[0] == 'ctest'))
        self.assertFalse(any('--build' in command or 'install-qt' in command for command in commands))
        self.assertTrue(all(call.kwargs['env']['QT_QPA_PLATFORM'] == 'offscreen' for call in run.call_args_list))
        self.assertNotIn('QT_QPA_PLATFORM', workflow.env)
        self.assertIn('separate', result['rendering'])

    def test_subscripts_use_the_selected_python_and_same_environment(self):
        workflow = self.workflow('windows')
        workflow.tools_python.parent.mkdir(parents=True)
        workflow.tools_python.write_bytes(b'python fixture')
        workflow = self.workflow('windows')
        with patch.object(desktop.subprocess, 'run', return_value=subprocess.CompletedProcess([], 0, 'ok')) as run, \
                contextlib.redirect_stdout(io.StringIO()):
            workflow.script('collect-notices.py', workflow.output/'notices')
        command = run.call_args.args[0]
        self.assertEqual(command[0], str(workflow.tools_python))
        self.assertEqual(command[1], str(self.root/'scripts/collect-notices.py'))
        self.assertIs(run.call_args.kwargs['env'], workflow.env)

    def test_verify_rejects_modified_or_escaped_artifacts(self):
        workflow = self.workflow()
        workflow.output.mkdir(parents=True)
        artifact = workflow.output/'package.dmg'
        artifact.write_bytes(b'owned package')
        items = workflow.hashes([artifact])
        workflow.verify_hashes(items)
        artifact.write_bytes(b'changed')
        with self.assertRaisesRegex(ValueError, 'changed or is missing'):
            workflow.verify_hashes(items)
        with self.assertRaisesRegex(ValueError, 'changed or is missing'):
            workflow.verify_hashes([{'file': '../outside', 'sha256': 'a'*64}])

    def test_verify_does_not_require_current_checkout_to_match_package(self):
        workflow = self.workflow()
        workflow.output.mkdir(parents=True)
        dmg = workflow.output/'package.dmg'
        dmg.write_bytes(b'old verified package')
        report = {'status': 'passed', 'platform': 'macos', 'artifacts': workflow.hashes([dmg])}
        (workflow.output/'package.json').write_text(json.dumps(report))
        with patch.object(workflow, 'doctor'), patch.object(workflow, 'script') as script, \
                patch.object(workflow, 'source', side_effect=AssertionError('Verify must bind package, not checkout')):
            result = workflow.verify()
        self.assertEqual(result['deployment'], 'passed')
        self.assertEqual(script.call_args_list[0].args[0], 'verify-macos-dmg.py')

    def test_installer_does_not_implicitly_redeploy_runtime(self):
        workflow = self.workflow('linux')
        with patch.object(workflow, 'script') as script:
            with self.assertRaisesRegex(ValueError, 'kind runtime'):
                workflow.installer_package()
            script.assert_not_called()

    def test_repeated_windows_verification_preserves_previous_harness(self):
        workflow = self.workflow('windows')
        workflow.args.kind = 'runtime'
        workflow.output.mkdir(parents=True)
        for required in (workflow.tests, workflow.qt/'bin/Qt6Test.dll',
                         workflow.qt/'plugins/platforms/qoffscreen.dll'):
            required.parent.mkdir(parents=True, exist_ok=True)
            required.write_bytes(b'fixture')
        previous = workflow.output/'windows-runtime-test'
        previous.mkdir()
        evidence = previous/'result.json'
        evidence.write_text('previous evidence')
        with patch.object(workflow, 'doctor'), patch.object(workflow, 'runtime_receipt', return_value={'source': self.source}), \
                patch.object(workflow, 'script') as script:
            workflow.verify()
            workflow.verify()
        calls = [call.args for call in script.call_args_list if call.args[0] == 'verify_windows_runtime.py']
        directories = [args[args.index('--output')+1] for args in calls]
        self.assertEqual(len(set(directories)), 2)
        self.assertTrue(all(path.parent == workflow.output and not path.exists() for path in directories))
        self.assertEqual(evidence.read_text(), 'previous evidence')

    def test_all_verification_rejects_receipt_from_another_platform(self):
        workflow = self.workflow('linux')
        workflow.output.mkdir(parents=True)
        fixture = workflow.build/'apps/desktop/m2-flat.zip'
        fixture.parent.mkdir(parents=True)
        fixture.write_bytes(b'owned zip fixture')
        (workflow.output/'package-installer.json').write_text(json.dumps({
            'status': 'passed', 'source': self.source, 'platform': 'windows', 'artifacts': []}))
        with patch.object(workflow, 'doctor'), patch.object(workflow, 'runtime_receipt', return_value={'source': self.source}), \
                patch.object(workflow, 'script') as script:
            with self.assertRaisesRegex(ValueError, 'identities differ'):
                workflow.verify()
            script.assert_not_called()

    def test_source_fingerprint_includes_changed_untracked_bytes(self):
        subprocess.run(['git', 'init', '-q', self.root], check=True)
        subprocess.run(['git', '-C', self.root, '-c', 'user.name=Test', '-c', 'user.email=test@example.invalid',
                        'commit', '-q', '--allow-empty', '-m', 'fixture'], check=True)
        source = self.root/'new-source.txt'
        source.write_text('first')
        workflow = self.workflow('linux')
        first = workflow.source()
        source.write_text('other')
        second = workflow.source()
        self.assertTrue(first['dirty'])
        self.assertEqual(first['revision'], second['revision'])
        self.assertNotEqual(first['fingerprint'], second['fingerprint'])


if __name__ == '__main__':
    unittest.main()
