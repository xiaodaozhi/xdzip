#!/usr/bin/env python3
"""Validate workflow graphs and retain desktop CI without browser-driven web tests."""
import importlib.util
from pathlib import Path
import re
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

    def test_default_workflow_gates_match_evidence_producers(self):
        ordinary = {name for name, value in self.workflows.items()
                    if 'pull_request' in value.get('on', {})}
        self.assertEqual(ordinary, {'docs.yml', 'foundation.yml', 'qt-components.yml'})
        self.assertEqual(set(self.plan.SCOPES), {'docs', 'foundation', 'qt'})
        self.assertEqual(set(self.evidence.JOBS), set(self.plan.SCOPES))
        for filename, scope in [('docs.yml', 'docs'), ('foundation.yml', 'foundation'),
                                ('qt-components.yml', 'qt')]:
            with self.subTest(scope=scope):
                workflow = self.workflows[filename]
                self.assertIn('push', workflow['on'])
                self.assertIn('workflow_dispatch', workflow['on'])
                gate = workflow['jobs'][scope + '-gate']
                self.assertEqual(set(gate['needs']), set(self.evidence.JOBS[scope]))
                self.assertEqual(gate.get('if'), 'always()')
                runs = '\n'.join(step.get('run', '') for step in gate['steps'])
                self.assertIn('ci-evidence.py gate --scope ' + scope, runs)

    def test_workflows_do_not_start_web_browser_tests(self):
        browser_drivers = re.compile(r'playwright|chromium|webdriver|selenium', re.I)
        web_scripts = re.compile(
            r'scripts/(?:check-html-[\w-]+|check-tab-strip|check-integrity-progress|'
            r'check-extract-fidelity|check-directory-tree-delivery|export-hifi-titlebars|'
            r'export-welcome-design|render-design-assets)\.py', re.I)
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

    def test_docs_use_isolated_pinned_rust_before_rust_tests(self):
        job = self.workflows['docs.yml']['jobs']['docs']
        self.assertNotIn('RUSTUP_HOME', job.get('env', {}))
        steps = job['steps']
        prepare = [i for i, step in enumerate(steps)
                   if step.get('name') == 'Prepare isolated pinned Rust toolchain']
        self.assertEqual(len(prepare), 1)
        step = steps[prepare[0]]
        self.assertEqual(step['env']['RUSTUP_HOME'], '${{ runner.temp }}/xdzip-docs-rustup')
        self.assertIn('>> "$GITHUB_ENV"', step['run'])
        self.assertNotIn('continue-on-error', step)
        version = tomllib.loads((ROOT / 'rust-toolchain.toml').read_text(encoding='utf-8'))['toolchain']['channel']
        for required in [f'rustup toolchain install {version} --profile minimal --component rustfmt --component clippy',
                         f'toolchains/{version}-*/bin/rustc', 'rustup which rustc', '$RUSTUP_HOME',
                         'cargo clippy --version', 'rustfmt --version']:
            self.assertIn(required, step['run'])
        consumers = [i for i, item in enumerate(steps)
                     if 'scripts/test-production-update-signing.py' in item.get('run', '')]
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
