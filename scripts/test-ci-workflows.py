#!/usr/bin/env python3
"""Validate lightweight CI and the separately dispatched desktop workflows."""
import importlib.util
from pathlib import Path
import re
import subprocess
import tempfile
import tomllib
import unittest

import yaml

ROOT = Path(__file__).resolve().parents[1]


class UniqueLoader(yaml.BaseLoader):
    def construct_mapping(self, node, deep=False):
        keys = [self.construct_object(key, deep=deep) for key, _ in node.value]
        if len(keys) != len(set(keys)):
            raise ValueError('duplicate-yaml-key')
        return super().construct_mapping(node, deep=deep)


def load(path):
    with path.open(encoding='utf-8') as source:
        return yaml.load(source, Loader=UniqueLoader)


def import_script(name, filename):
    spec = importlib.util.spec_from_file_location(name, ROOT / 'scripts' / filename)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def validate_job_env_contexts(job):
    for value in job.get('env', {}).values():
        if re.search(r'\$\{\{[^}]*\brunner(?:\.|\[|\b)', str(value)):
            raise ValueError('runner-context-unavailable-in-job-env')


def validate_graph(jobs):
    graph = {}
    for name, job in jobs.items():
        needs = job.get('needs', [])
        if isinstance(needs, str):
            needs = [needs]
        if not isinstance(needs, list) or any(item not in jobs for item in needs):
            raise ValueError('missing-job-dependency')
        graph[name] = needs
    visiting, visited = set(), set()

    def visit(name):
        if name in visiting:
            raise ValueError('cyclic-job-dependency')
        if name in visited:
            return
        visiting.add(name)
        for dependency in graph[name]:
            visit(dependency)
        visiting.remove(name)
        visited.add(name)

    for name in graph:
        visit(name)
    return graph


class WorkflowTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        directory = ROOT / '.github/workflows'
        files = sorted([*directory.glob('*.yml'), *directory.glob('*.yaml')])
        cls.workflows = {p.name: load(p) for p in files}
        cls.evidence = import_script('workflow_ci_evidence', 'ci-evidence.py')
        cls.plan = import_script('workflow_ci_plan', 'ci-plan.py')

    def test_all_workflows_and_cache_action_have_valid_yaml_and_job_graphs(self):
        self.assertTrue(self.workflows)
        self.assertIsInstance(load(ROOT / '.github/actions/cache-inputs/action.yml'), dict)
        for name, workflow in self.workflows.items():
            with self.subTest(workflow=name):
                self.assertIsInstance(workflow, dict)
                self.assertIn('on', workflow)
                self.assertTrue(workflow.get('jobs'))
                validate_graph(workflow['jobs'])
                for job in workflow['jobs'].values():
                    validate_job_env_contexts(job)
                    for step in job.get('steps', []):
                        target = step.get('uses', '')
                        if target.startswith('./'):
                            directory = ROOT / target[2:]
                            self.assertTrue((directory / 'action.yml').is_file() or
                                            (directory / 'action.yaml').is_file() or
                                            (directory / 'Dockerfile').is_file(), target)
                        for script in re.findall(r'(?<![\w/.-])(?:scripts|design/assets/fonts)/[\w./-]+\.(?:py|ps1|sh)',
                                                 step.get('run', '')):
                            self.assertTrue((ROOT / script).is_file(), script)

    def test_graph_rejects_missing_jobs_cycles_and_duplicate_yaml_keys(self):
        with self.assertRaisesRegex(ValueError, 'missing-job-dependency'):
            validate_graph({'a': {'needs': 'missing'}})
        with self.assertRaisesRegex(ValueError, 'cyclic-job-dependency'):
            validate_graph({'a': {'needs': 'b'}, 'b': {'needs': 'a'}})
        with self.assertRaisesRegex(ValueError, 'duplicate-yaml-key'):
            yaml.load('jobs:\n  a: {}\n  a: {}\n', Loader=UniqueLoader)
        with self.assertRaisesRegex(ValueError, 'runner-context-unavailable-in-job-env'):
            validate_job_env_contexts({'env': {'X': '${{ runner.temp }}/test'}})
        validate_job_env_contexts({'steps': [{'env': {'X': '${{ runner.temp }}/test'}}]})

    def test_only_lightweight_workflow_runs_on_pull_requests_and_main(self):
        ordinary = {name for name, value in self.workflows.items()
                    if 'pull_request' in value.get('on', {})}
        self.assertEqual(ordinary, {'docs.yml'})
        intel = self.workflows['macos-intel.yml']
        self.assertEqual(set(intel['on']), {'workflow_dispatch'})
        self.assertEqual(intel['jobs']['native-intel']['runs-on'], 'macos-15-intel')
        evidence = next(step for step in intel['jobs']['native-intel']['steps']
                        if step.get('name') == 'Retain validation records only')
        self.assertIn('browser-tests-shard-*.log', evidence['with']['path'])
        push_workflows = {name for name, value in self.workflows.items() if 'push' in value.get('on', {})}
        self.assertEqual(push_workflows, {'docs.yml'})
        workflow = self.workflows['docs.yml']
        self.assertEqual(workflow['name'], 'CI')
        self.assertEqual(set(workflow['on']), {'push', 'pull_request', 'workflow_dispatch'})
        self.assertEqual(workflow['on']['push']['branches'], ['main'])
        self.assertNotIn('tags', workflow['on']['push'])
        self.assertEqual(set(workflow['jobs']), {'checks'})
        runs = '\n'.join(step.get('run', '') for step in workflow['jobs']['checks']['steps'])
        for required in ['cargo fmt --all -- --check', 'scripts/check-native-format.py',
                         'scripts/check-docs.py', 'scripts/check-ui-i18n.py',
                         'scripts/check-window-contracts.py', 'scripts/check-operation-components.py',
                         'scripts/test-ci-workflows.py']:
            self.assertIn(required, runs)
        for heavy in ['cargo test', 'cmake --build', 'ctest ', 'ci-desktop.ps1',
                      'check-foundation.ps1', 'ci-evidence.py', 'ci-plan.py']:
            self.assertNotIn(heavy, runs)

    def test_native_format_checks_share_the_tracked_source_inventory(self):
        foundation = self.workflows['foundation.yml']['jobs']['cpp-format']['steps']
        self.assertTrue(any('scripts/check-native-format.py' in step.get('run', '') for step in foundation))
        native_format = import_script('workflow_native_format', 'check-native-format.py')
        sources = native_format.native_sources(ROOT)
        self.assertIn('apps/desktop/shell/NativeMessages.h', sources)
        self.assertIn('apps/desktop/tests/sparkle_proxy_probe.m', sources)
        self.assertIn('apps/desktop/MacTitlebar.mm', sources)
        self.assertNotIn('apps/desktop/ArchiveFormatsGenerated.h', sources)
        self.assertNotIn('apps/desktop/tests/preferences-compat/previous/UiPreferences.cpp', sources)

    def test_strict_clippy_checks_the_whole_push_or_pull_request_tree_difference(self):
        steps = self.workflows['docs.yml']['jobs']['checks']['steps']
        step, = (step for step in steps if 'scripts/check-rust-clippy.py' in step.get('run', ''))
        self.assertEqual(step['env'], {'XDZIP_EVENT_NAME': '${{ github.event_name }}',
                                       'XDZIP_PUSH_BEFORE': '${{ github.event.before }}'})
        clippy = import_script('workflow_rust_clippy', 'check-rust-clippy.py')
        release_steps = self.workflows['release.yml']['jobs']['contracts']['steps']
        self.assertTrue(any(' '.join(clippy.STRICT_CLIPPY) in step.get('run', '')
                            for step in release_steps))
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)

            def git(*args):
                return subprocess.check_output(['git', *args], cwd=root, text=True).strip()

            def commit(path):
                (root / path).write_text(path, encoding='utf-8')
                git('add', path)
                git('-c', 'user.name=XDZip CI test', '-c', 'user.email=ci@example.invalid',
                    'commit', '-qm', path)
                return git('rev-parse', 'HEAD')

            git('init', '-qb', 'main')
            base = commit('README.md')
            rust = commit('lib.rs')
            docs = commit('notes.md')
            self.assertTrue(clippy.should_run(root, 'push', base))  # Rust changed before the last commit.
            self.assertFalse(clippy.should_run(root, 'push', rust))
            self.assertTrue(clippy.should_run(root, 'push', '0' * 40))
            self.assertTrue(clippy.should_run(root, 'workflow_dispatch', docs))
            with tempfile.TemporaryDirectory() as clone_dir:
                clone = Path(clone_dir).resolve()
                git('clone', '-q', '--depth=2', root.resolve().as_uri(), str(clone))
                self.assertTrue(clippy.should_run(clone, 'push', base))  # Fetch the missing base.
            git('branch', 'contribution')
            git('checkout', '-qb', 'target', base)
            commit('maintainers.md')
            git('-c', 'user.name=XDZip CI test', '-c', 'user.email=ci@example.invalid',
                'merge', '--no-ff', '--no-edit', 'contribution')
            self.assertTrue(clippy.should_run(root, 'pull_request', ''))
            for rust_input in ('VERSION.json', 'resources/update-trust.json',
                               'resources/ui-languages.json', 'resources/i18n/en-US.json',
                               'scripts/check-rust-codec-policy.py', 'scripts/rust-codec-policy.json'):
                (root / rust_input).parent.mkdir(parents=True, exist_ok=True)
                before = git('rev-parse', 'HEAD')
                commit(rust_input)
                self.assertTrue(clippy.should_run(root, 'push', before), rust_input)


    def test_full_workflows_are_manually_dispatched_and_keep_their_gates(self):
        for filename, scope in [('foundation.yml', 'foundation'), ('qt-components.yml', 'qt')]:
            with self.subTest(scope=scope):
                workflow = self.workflows[filename]
                self.assertEqual(set(workflow['on']), {'workflow_dispatch'})
                gate = workflow['jobs'][scope + '-gate']
                self.assertEqual(set(gate['needs']), set(self.evidence.JOBS[scope]))
                self.assertEqual(gate.get('if'), 'always() && !inputs.sdk_only && !inputs.image_only' if scope == 'foundation' else 'always()')
                if scope == 'foundation':
                    self.assertEqual(workflow['on']['workflow_dispatch']['inputs']['sdk_only']['default'], 'false')
                    self.assertEqual(workflow['on']['workflow_dispatch']['inputs']['image_only']['default'], 'false')
                    self.assertEqual(workflow['jobs']['sdk-preflight']['uses'], './.github/workflows/sdk-trial.yml')
                    self.assertEqual(workflow['jobs']['sdk-preflight']['if'], 'inputs.sdk_only && !inputs.image_only')
                    image = workflow['jobs']['image-preflight']
                    self.assertEqual(image['if'], 'inputs.image_only')
                    self.assertEqual(image['runs-on'], 'macos-15')
                    self.assertIn('check-macos-imageio.py', '\n'.join(s.get('run', '') for s in image['steps']))
                    for name in self.evidence.JOBS[scope]:
                        self.assertIn('!inputs.sdk_only', workflow['jobs'][name]['if'])
                        self.assertIn('!inputs.image_only', workflow['jobs'][name]['if'])
                runs = '\n'.join(step.get('run', '') for step in gate['steps'])
                self.assertIn('ci-evidence.py gate --scope ' + scope, runs)
                job = workflow['jobs']['bridge' if scope == 'foundation' else 'native-qt']
                save = [step for step in job['steps'] if step.get('name') == 'Save trusted dependency inputs']
                self.assertEqual(len(save), 1)
                self.assertIn("github.event_name == 'workflow_dispatch'", save[0]['if'])
                self.assertIn("github.ref == 'refs/heads/main'", save[0]['if'])

    def test_release_uses_one_manual_dispatch_and_its_own_native_build(self):
        workflow = self.workflows['release.yml']
        self.assertEqual(set(workflow['on']), {'workflow_dispatch'})
        self.assertIn('tag', workflow['on']['workflow_dispatch']['inputs'])
        self.assertNotIn('validate_only', workflow['on']['workflow_dispatch']['inputs'])
        native = workflow['jobs']['native']
        preflight = workflow['jobs']['macos-signing-preflight']
        self.assertEqual(set(preflight['strategy']['matrix']['os']), {'macos-15', 'macos-15-intel'})
        self.assertIn('scripts/test-cli-bundle.py', '\n'.join(step.get('run', '') for step in preflight['steps']))
        self.assertIn('macos-signing-preflight', native['needs'])
        self.assertIn("needs.macos-signing-preflight.result == 'success'", native['if'])
        self.assertEqual(set(native['strategy']['matrix']['os']), {'windows-2022', 'macos-15', 'macos-15-intel', 'ubuntu-24.04'})
        self.assertIn('scripts/ci-release.ps1 -Stage Build', '\n'.join(step.get('run', '') for step in native['steps']))
        jobs = workflow['jobs']
        verify = jobs['verify']
        self.assertEqual(verify['strategy']['matrix'], native['strategy']['matrix'])
        for job in (native, verify):
            self.assertIn('macos-15-intel', job['env']['XDZIP_MACOS_INTEL_CI'])
        intel_update = self.workflows['update-regression.yml']['jobs']['macos_intel']
        self.assertEqual(intel_update['runs-on'], 'macos-15-intel')
        self.assertIn('check-macos-product-update.py', '\n'.join(step.get('run', '') for step in intel_update['steps']))
        font_step = next(i for i, step in enumerate(intel_update['steps'])
                         if step.get('name') == 'Prepare and check product font tools')
        dependency_step = next(i for i, step in enumerate(intel_update['steps'])
                               if step.get('name') == 'Prepare pinned native Intel dependencies')
        self.assertLess(font_step, dependency_step)
        self.assertIn('design/assets/fonts/requirements.txt', intel_update['steps'][font_step]['run'])
        self.assertIn('from ui_fonts import tools; tools()', intel_update['steps'][font_step]['run'])
        preflight = next(i for i, step in enumerate(intel_update['steps'])
                         if step.get('name') == 'Check update fixture contracts before native dependencies')
        self.assertLess(preflight, dependency_step)
        self.assertIn('scripts/check-system-proxy-fixture.py', intel_update['steps'][preflight]['run'])
        production = self.workflows['production-update-acceptance.yml']
        platforms = production['on']['workflow_dispatch']['inputs']['platform']
        self.assertEqual(platforms['default'], 'all')
        self.assertEqual(set(platforms['options']), {'all', 'windows-linux', 'windows', 'linux', 'macos', 'macos-intel'})
        self.assertIn("inputs.platform == 'windows-linux'", production['jobs']['windows']['if'])
        self.assertIn("inputs.platform == 'windows-linux'", production['jobs']['linux']['if'])
        self.assertNotIn('windows-linux', production['jobs']['macos']['if'])
        self.assertNotIn('windows-linux', production['jobs']['macos_intel']['if'])
        dependencies = production['on']['workflow_dispatch']['inputs']['dependencies']
        self.assertEqual(dependencies['default'], 'auto')
        self.assertEqual(dependencies['options'], ['auto', 'cold'])
        windows_production = production['jobs']['windows']
        preparation = next(step['run'] for step in windows_production['steps']
                           if step.get('name') == 'Prepare native tools and Qt without signing authority')
        self.assertLess(preparation.index('Enter-VsDevShell'), preparation.index('aqt install-qt'))
        for name in ('perl', 'nmake', 'cmake', 'ninja'):
            self.assertIn("'" + name + "'", preparation)
        for name in ('windows', 'macos', 'macos_intel', 'linux'):
            job = production['jobs'][name]
            self.assertEqual(job['permissions'], {'contents': 'read', 'actions': 'read'})
            prepare_network = next(i for i, step in enumerate(job['steps']) if 'Qt Network without signing authority' in step.get('name', ''))
            sign = next(i for i, step in enumerate(job['steps']) if 'sign-production-update-fixtures.py' in step.get('run', ''))
            self.assertLess(prepare_network, sign)
            self.assertEqual(job['steps'][prepare_network]['env']['GITHUB_TOKEN'], '${{ github.token }}')
            self.assertNotIn('secrets.', repr(job['steps'][prepare_network]))
            self.assertNotIn('--publish', repr(job))
        self.assertNotIn('actions', production['permissions'])
        image = (ROOT/'tests/fixtures/native/update-qt-build.Dockerfile').read_text(encoding='utf-8')
        self.assertIn('libnautilus-extension-dev', image)
        intel_production = production['jobs']['macos_intel']
        self.assertEqual(production['jobs']['keys']['if'], "${{ github.ref == 'refs/heads/main' }}")
        self.assertEqual(intel_production['runs-on'], 'macos-15-intel')
        self.assertEqual(intel_production['needs'], 'keys')
        self.assertEqual(intel_production['environment'], 'release-signing')
        steps = intel_production['steps']
        signers = [step for step in steps if 'sign-production-update-fixtures.py' in step.get('run', '')]
        self.assertEqual(len(signers), 1)
        self.assertIn('secrets.XDZIP_UPDATE_SIGNING_KEY_HEX', repr(signers[0]['env']))
        for step in steps:
            if step is not signers[0]:
                self.assertNotIn('secrets.', repr(step))
        self.assertIn('evidence-macos-intel/*', steps[-1]['with']['path'])
        sign_steps = '\n'.join(step.get('run', '') for step in workflow['jobs']['sign']['steps'])
        self.assertIn('sign-macos-update.py --directory build/packages --architecture x86_64', sign_steps)
        self.assertIn('--require-update-signature --include-intel', sign_steps)
        self.assertEqual(verify['needs'], ['resolve', 'native'])
        self.assertIn('scripts/ci-release.ps1 -Stage Verify', '\n'.join(step.get('run', '') for step in verify['steps']))
        self.assertEqual(jobs['sign']['needs'], ['resolve', 'verify', 'contracts'])
        self.assertEqual(jobs['publish']['needs'], ['resolve', 'sign'])
        self.assertEqual(jobs['verify-public']['needs'], ['resolve', 'publish'])
        for name in ['draft', 'sign', 'publish', 'verify-public']:
            self.assertIn('!inputs.candidate', jobs[name]['if'])
        for name in ['resolve', 'native', 'verify', 'contracts', 'candidate-assembly', 'candidate-download']:
            self.assertNotIn('environment', jobs[name])
            self.assertNotIn('permissions', jobs[name])
            self.assertNotIn('secrets.', repr(jobs[name]))
        self.assertEqual(jobs['sign']['environment'], 'release-signing')
        self.assertEqual(jobs['publish']['environment'], 'release-signing')
        stability = self.workflows['browser-stability.yml']
        self.assertEqual(set(stability['on']), {'workflow_dispatch'})
        self.assertIn('-BrowserStability', '\n'.join(step.get('run', '') for step in stability['jobs']['repeat']['steps']))

    def test_store_msix_is_manual_and_exports_verified_package_only(self):
        workflow = self.workflows['windows-store-msix.yml']
        self.assertEqual(set(workflow['on']), {'workflow_dispatch'})
        inputs = workflow['on']['workflow_dispatch']['inputs']
        self.assertEqual(inputs['source_revision']['required'], 'true')
        self.assertEqual(workflow['permissions'], {'contents': 'read'})
        self.assertEqual(set(workflow['jobs']), {'package'})
        job = workflow['jobs']['package']
        self.assertNotIn('permissions', job)
        self.assertNotIn('environment', job)
        steps = job['steps']
        actions = {step['uses'].split('@')[0] for step in steps if 'uses' in step}
        self.assertEqual(actions, {'actions/checkout', 'actions/upload-artifact'})
        build = next(step for step in steps if step.get('id') == 'msix')
        self.assertEqual({key: value for key, value in build['env'].items() if key != 'SOURCE_REVISION'}, {
            key: '${{ vars.' + key + ' }}' for key in (
                'XDZIP_STORE_MSIX_NAME', 'XDZIP_STORE_MSIX_PUBLISHER', 'XDZIP_STORE_MSIX_PUBLISHER_DISPLAY_NAME')
        })
        archive, diagnostics = steps[-2:]
        self.assertTrue(archive['uses'].startswith('actions/upload-artifact@'))
        self.assertNotIn('if', archive)
        self.assertEqual(set(archive['with']['path'].splitlines()), {
            'build/store-msix-package/xdzip-windows-x64.msix',
            'build/store-msix-package/windows-msix.json',
            'build/store-msix-package/package-msix.json',
            'build/store-msix-package/verify.json',
            'build/store-msix-package/verify-runtime.json',
            'build/desktop/desktop-build.json',
            'build/desktop/desktop-test.json',
        })
        self.assertEqual(diagnostics['if'], 'failure()')
        self.assertTrue(diagnostics['uses'].startswith('actions/upload-artifact@'))
        for path in diagnostics['with']['path'].splitlines():
            self.assertIn(Path(path).suffix, {'.log', '.json'})

    def test_stable_snap_uses_the_same_native_build_and_protected_release(self):
        self.assertNotIn('snap-store.yml', self.workflows)
        jobs = self.workflows['release.yml']['jobs']
        self.assertIn('"$CANDIDATE" == false', jobs['resolve']['steps'][-1]['run'])
        self.assertIn('^v[0-9]+\\.[0-9]+\\.[0-9]+$', jobs['resolve']['steps'][-1]['run'])
        native = jobs['native']['steps']
        build = next(i for i, step in enumerate(native) if 'ci-release.ps1 -Stage Build' in step.get('run', ''))
        self.assertIn('ci-release-snap.sh prepare', native[build - 1]['run'])
        self.assertIn('ci-release-snap.sh pack', native[build + 1]['run'])
        self.assertEqual(native[build + 2]['with']['name'], 'release-snap-amd64')
        publish = jobs['publish']
        self.assertEqual(publish['environment'], 'release-signing')
        upload = publish['steps'][-1]
        self.assertEqual(upload['if'], "needs.resolve.outputs.snap_stable == 'true'")
        self.assertEqual(upload['env']['SOURCE_REVISION'], '${{ needs.resolve.outputs.revision }}')
        self.assertIn('release-policy/scripts/snap_store.py', upload['run'])
        self.assertIn('--verify-public', publish['steps'][-2]['run'])

    def test_workflows_do_not_start_web_browser_tests(self):
        browser_drivers = re.compile(r'playwright|chromium|webdriver|selenium', re.I)
        web_scripts = re.compile(
            r'scripts/(?:render-design-assets)\.py', re.I)
        for name, workflow in self.workflows.items():
            for job in workflow.get('jobs', {}).values():
                for step in job.get('steps', []):
                    run = step.get('run', '')
                    with self.subTest(workflow=name, step=step.get('name', 'unnamed')):
                        self.assertNotRegex(run, browser_drivers)
                        self.assertNotRegex(run, web_scripts)
                        if 'check-add-dialog.py' in run:
                            self.assertNotIn('--html', run)
                            self.assertNotIn('--native-zoom', run)
        for name in ['ci-desktop.ps1', 'check-foundation.ps1', 'ci-release.ps1']:
            content = (ROOT / 'scripts' / name).read_text(encoding='utf-8')
            with self.subTest(script=name):
                self.assertNotRegex(content, browser_drivers)
                self.assertNotRegex(content, web_scripts)
                for line in content.splitlines():
                    if 'check-add-dialog.py' in line:
                        self.assertIn('--qt-input', line)
                        self.assertNotIn('--html', line)
                        self.assertNotIn('--native-zoom', line)

    def test_lightweight_ci_uses_isolated_pinned_rust_for_formatting(self):
        job = self.workflows['docs.yml']['jobs']['checks']
        self.assertNotIn('RUSTUP_HOME', job.get('env', {}))
        steps = job['steps']
        prepare = [i for i, step in enumerate(steps)
                   if step.get('name') == 'Prepare isolated pinned Rust toolchain']
        self.assertEqual(len(prepare), 1)
        step = steps[prepare[0]]
        self.assertEqual(step['env']['RUSTUP_HOME'], '${{ runner.temp }}/xdzip-ci-rustup')
        self.assertIn('>> "$GITHUB_ENV"', step['run'])
        self.assertNotIn('continue-on-error', step)
        version = tomllib.loads((ROOT / 'rust-toolchain.toml').read_text(encoding='utf-8'))['toolchain']['channel']
        for required in [f'rustup toolchain install {version} --profile minimal --component rustfmt --component clippy',
                         f'toolchains/{version}-*/bin/rustc', 'rustup which rustc', '$RUSTUP_HOME']:
            self.assertIn(required, step['run'])
        consumers = [i for i, item in enumerate(steps)
                     if 'cargo fmt --all -- --check' in item.get('run', '')]
        self.assertEqual(len(consumers), 1)
        self.assertLess(prepare[0], consumers[0])

    def test_qt_source_uses_isolated_complete_rust_toolchain(self):
        job = self.workflows['qt-source.yml']['jobs']['source-runtime']
        self.assertEqual(job['if'], "github.ref == 'refs/heads/main'")
        steps = job['steps']
        isolation = [i for i, step in enumerate(steps)
                     if step.get('name') == 'Isolate Rust toolchain for source validation']
        build = [i for i, step in enumerate(steps)
                 if 'scripts/ci-qt-source.ps1' in step.get('run', '')]
        self.assertEqual(len(isolation), 1)
        self.assertEqual(len(build), 1)
        self.assertLess(isolation[0], build[0])
        self.assertNotIn('if', steps[isolation[0]])
        self.assertNotIn('continue-on-error', steps[isolation[0]])
        for expected in ['$env:RUNNER_TEMP', 'xdzip-qt-source-rustup',
                         'RUSTUP_HOME=', '$env:GITHUB_ENV']:
            self.assertIn(expected, steps[isolation[0]]['run'])
        version = tomllib.loads((ROOT / 'rust-toolchain.toml').read_text(encoding='utf-8'))['toolchain']['channel']
        script = (ROOT / 'scripts/ci-qt-source.ps1').read_text(encoding='utf-8')
        self.assertIn(f"'install','{version}','--profile','minimal','--component','rust-src',"
                      "'--component','rustfmt','--component','clippy'", script)

    def test_qt_desktop_and_native_coverage_remains(self):
        source = (ROOT / 'scripts/ci-desktop.ps1').read_text(encoding='utf-8')
        for required in ['qt_zip_browser', 'until-fail:10', 'qt_task_components',
                         'check-native-sessions.py', 'check-add-dialog.py', '--qt-input']:
            with self.subTest(check=required):
                self.assertIn(required, source)
        self.assertIn('native-qt', self.workflows['qt-components.yml']['jobs'])
        self.assertEqual(self.evidence.JOBS['qt'], ['qt-plan', 'native-qt'])
        self.assertIn('browser_evidence', (ROOT / 'scripts/ci-evidence.py').read_text(encoding='utf-8'))
        self.assertNotIn('prepare-private-test.py', source)

    def test_macos_attribution_links_run_on_native_macos(self):
        steps = self.workflows['qt-components.yml']['jobs']['native-qt']['steps']
        matching = [step for step in steps if 'scripts/test-macos-attribution.py' in step.get('run', '')]
        self.assertEqual(len(matching), 1)
        self.assertEqual(matching[0].get('if'), "runner.os == 'macOS'")
        self.assertNotIn('continue-on-error', matching[0])

    def test_portable_vendor_config_runs_on_all_native_platforms(self):
        steps = self.workflows['qt-components.yml']['jobs']['native-qt']['steps']
        matching = [step for step in steps if 'DeliveryTests.test_vendor_config_real_cargo' in step.get('run', '')]
        self.assertEqual(len(matching), 1)
        self.assertNotIn('if', matching[0])
        self.assertNotIn('continue-on-error', matching[0])
        self.assertEqual(matching[0]['env']['CARGO_NET_OFFLINE'], 'true')
        self.assertEqual(matching[0]['env']['XDZIP_VENDOR_INTEGRATION'], '1')


if __name__ == '__main__':
    unittest.main()
