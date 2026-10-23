# scripts/check-windows-native-failures-ci.py

"""Diagnose two hosted Windows native failures without relaxing QTest results."""
import argparse
from datetime import datetime, timezone
import importlib.util
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import sys
import tempfile
import zipfile

ROOT = Path(__file__).resolve().parents[1]


def load(name, filename):
    specification = importlib.util.spec_from_file_location(name, ROOT / 'scripts' / filename)
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


common = load('focused_native_common', 'check-windows-native-ci.py')
visual = load('focused_native_visual', 'check-windows-native-visual-ci.py')
LAYOUT_METHODS = ('nativeUiLayoutAcceptance', 'fixedFileActionsAndColumnResize',
                  'tabContextClosesStableWindowLocalRanges', 'aboutUpdateStateLayoutAndReadOnlyActions',
                  'directoryContextExpansionReasons:zh-CN-menu')


def require(condition, code):
    if not condition:
        raise ValueError(code)


def now():
    return datetime.now(timezone.utc).isoformat()


def read(file):
    return json.loads(file.read_text(encoding='utf-8-sig'))


def write(file, value):
    file.write_text(json.dumps(value, indent=2) + '\n', encoding='utf-8')


def relative(file):
    return file.relative_to(ROOT).as_posix()


def inventory(folder):
    return [dict(path=relative(file), bytes=file.stat().st_size, sha256=common.digest(file))
            for file in sorted(folder.rglob('*')) if file.is_file()
            and 'private-temp' not in file.relative_to(folder).parts]


def stage_configuration():
    gallery = [entry for entry in visual.CASES if entry[0] == 'file_gallery']
    require(len(gallery) == 1 and gallery[0][1] == 'nativeFileCategoryVisualAudit',
            'gallery_stage_definition_changed')
    return (dict(role='gallery', methods=(gallery[0][1],), audit_variable=gallery[0][2], timeout_seconds=90),
            dict(role='layout-dpr-1', methods=LAYOUT_METHODS,
                 audit_variable='XDZIP_NATIVE_LAYOUT_AUDIT', timeout_seconds=240))


def initial_report(source, binary):
    return dict(schema_version=1, status='running', kind='focused_failure_diagnosis',
                run_kind='focused_failure_diagnosis', full_source_sha=source, binary_sha256=binary,
                started_at_utc=now(), full45=False, full108=False, real_user_click=False,
                full_menu_45_executed=False, full_three_dpr_layout_matrix_executed=False,
                manual_native_acceptance=False, manual_visual_accepted=False,
                fixed_reference_machine=False, reference_performance_accepted=False,
                win11_accepted=False, physical_dpr_transitions_accepted=False,
                physical_dual_monitor_accepted=False, final_acceptance=False, release_authorized=False,
                runner={key: os.environ.get(key) for key in
                        ('GITHUB_RUN_ID', 'GITHUB_RUN_ATTEMPT', 'GITHUB_JOB', 'ImageOS', 'ImageVersion')},
                limits=['Two independent owned native test processes, no retries or expected-failure conversion.',
                        'Gallery runs its entire method; layout runs only DPR 1 and one directory menu row.',
                        'Hosted diagnostics do not approve manual visuals, reference performance, Windows 11 or physical monitors.'],
                stages=[])


def protected_temp(output):
    expected = output.parent / 'private-temp'
    require(expected.is_dir() and not expected.is_symlink()
            and not expected.stat().st_file_attributes & stat.FILE_ATTRIBUTE_REPARSE_POINT
            and all(os.environ.get(key) and Path(os.environ[key]).resolve() == expected.resolve()
                    for key in ('TEMP', 'TMP')), 'wrapper_protected_temp_required')
    # Query only the wrapper-owned directory. No ACL or local display is changed.
    answer = subprocess.check_output(['pwsh', '-NoProfile', '-NonInteractive', '-Command',
        "$ErrorActionPreference='Stop'; [bool](Get-Acl -LiteralPath $env:TEMP).AreAccessRulesProtected"],
        stderr=subprocess.PIPE, timeout=10).decode('utf-8-sig').strip()
    require(answer == 'True', 'wrapper_temp_inheritance_not_protected')
    return dict(path=relative(expected), same_temp_and_tmp=True, inheritance_protected=True,
                wrapper_owned_directory=True)


def stage_environment(stage, folder, fixture, fixture_hash):
    env = common.native_environment(folder)
    for key in list(env):
        if key.startswith('XDZIP_') and ('AUDIT' in key or key.startswith('XDZIP_FILE_ICON_')):
            env.pop(key)
    images = folder / 'images'
    env.update(QT_QPA_PLATFORM='windows', QSG_RHI_BACKEND='d3d11', QSG_INFO='1', QT_SCALE_FACTOR='1',
               XDZIP_SETTINGS_PATH=str(folder / 'settings.ini'))
    env[stage['audit_variable']] = str(images)
    if stage['role'] == 'gallery':
        env.update(XDZIP_FILE_ICON_FIXTURE=str(fixture), XDZIP_FILE_ICON_FIXTURE_SHA256=fixture_hash)
    else:
        env.update(XDZIP_ABOUT_UPDATE_AUDIT=str(images / 'about'),
                   XDZIP_DIRECTORY_CONTEXT_DIAGNOSTICS=str(folder / 'failure-images'))
    return env


def check_log(text, methods):
    require(not re.search(r'FAIL!|QFATAL|SKIP', text)
            and re.search(r'Totals:\s+' + str(len(methods) + 2) + r' passed, 0 failed, 0 skipped', text),
            'qtest_failed_skipped_or_incomplete')
    for method in methods:
        function, _, row = method.partition(':')
        require(re.search(r'PASS\s*:\s*BrowserTest::' + re.escape(function + '(' + row + ')'), text),
                'requested_qtest_row_missing')
    require('Creating QRhi with backend D3D11' in text, 'actual_d3d11_log_required')


def execute_stage(stage, tests, output, source, binary, fixture, fixture_hash):
    folder = output / stage['role']
    (folder / 'images' / 'about').mkdir(parents=True)
    env = stage_environment(stage, folder, fixture, fixture_hash)
    log = folder / 'qt.log'
    command = [str(tests), *stage['methods'], '-v1', '-o', str(log) + ',txt']
    entry = dict(role=stage['role'], methods=list(stage['methods']), requested_dpr=1,
                 timeout_seconds=stage['timeout_seconds'], started_at_utc=now(), status='failed',
                 argv=[common.sanitized(value) for value in command])
    try:
        completed, receipt = common.run_owned(command, env, folder, tests, source, binary,
                                             timeout=stage['timeout_seconds'])
        entry.update(execution=receipt, exit_code=completed.returncode, timed_out=receipt['timed_out'])
        (folder / 'stdout.log').write_bytes(completed.stdout)
        (folder / 'stderr.log').write_bytes(completed.stderr)
        require(completed.returncode == 0 and receipt['timed_out'] is False, 'qtest_failed_or_timed_out')
        check_log(log.read_text(encoding='utf-8-sig'), stage['methods'])
        entry['status'] = 'passed'
    except (ValueError, KeyError, OSError, TypeError, common.Blocked, subprocess.SubprocessError) as error:
        entry['reason'] = str(error) if type(error) in (ValueError, common.Blocked) else type(error).__name__
        entry['ownership_or_identity_guard_rejected'] = isinstance(error, common.Blocked)
        if (folder / 'process-receipt.json').is_file():
            entry['execution'] = read(folder / 'process-receipt.json')
            entry.update(exit_code=entry['execution'].get('exit_code'),
                         timed_out=entry['execution'].get('timed_out'))
    entry['completed_at_utc'] = now()
    entry['files'] = inventory(folder)
    entry['png_json_inventory'] = [item for item in entry['files']
                                   if Path(item['path']).suffix.lower() in ('.png', '.json')]
    return entry


def run(args):
    require(sys.platform == 'win32' and os.environ.get('GITHUB_ACTIONS') == 'true'
            and os.environ.get('RUNNER_ENVIRONMENT') == 'github-hosted'
            and os.environ.get('RUNNER_OS') == 'Windows', 'windows_github_hosted_only')
    require(re.fullmatch('[0-9a-f]{40}', args.source) and re.fullmatch('[0-9a-f]{64}', args.binary_sha256),
            'invalid_source_or_binary_hash')
    tests, output = args.tests.resolve(), args.output.resolve()
    require(tests.is_file() and tests.is_relative_to(ROOT / 'build') and output.is_relative_to(ROOT / 'build')
            and output != ROOT / 'build' and not output.exists(), 'fresh_owned_build_paths_required')
    output.mkdir(parents=True)
    report = initial_report(args.source, args.binary_sha256)
    result_file = output / 'result.json'
    try:
        report['protected_temp'] = protected_temp(output)
        report['source_before'] = common.source_snapshot(tests, args.source, args.binary_sha256)
        display = common.display_state(args.source)
        require(display.get('managed_display_receipt') is True, 'managed_display_receipt_required')
        report.update(display_before=display, desktop=common.desktop_diagnostics())
        (output / 'browser-test-source.cpp').write_bytes((ROOT / 'apps/desktop/tests/browser_test.cpp').read_bytes())
        (output / 'source.patch').write_bytes(subprocess.check_output(['git', 'diff'], cwd=ROOT, timeout=10))
        fixture = output / 'file-icon-coverage.zip'
        visual.make_fixture(fixture)
        fixture_hash = common.digest(fixture)
        report['fixture'] = dict(path=relative(fixture), bytes=fixture.stat().st_size, sha256=fixture_hash,
                                type_count=15, entries_have_empty_payload=True)
        for stage in stage_configuration():
            report['stages'].append(execute_stage(stage, tests, output, args.source, args.binary_sha256,
                                                  fixture, fixture_hash))
            write(result_file, report)
            print(json.dumps({key: report['stages'][-1].get(key) for key in
                              ('role', 'status', 'exit_code', 'timed_out', 'reason')}), flush=True)
            # A genuine QTest failure does not suppress the independent stage;
            # identity or managed-display drift always stops further launches.
            require(report['stages'][-1].get('ownership_or_identity_guard_rejected') is not True,
                    'owned_process_guard_rejected')
            require(common.source_snapshot(tests, args.source, args.binary_sha256) == report['source_before']
                    and common.display_state(args.source) == display, 'source_binary_or_display_changed')
        report['source_after'] = common.source_snapshot(tests, args.source, args.binary_sha256)
        report['display_after'] = common.display_state(args.source)
        require(report['source_after'] == report['source_before'] and report['display_after'] == display,
                'source_binary_or_display_changed')
        report['status'] = 'passed' if len(report['stages']) == 2 and all(
            stage['status'] == 'passed' for stage in report['stages']) else 'failed'
    except (ValueError, KeyError, OSError, TypeError, common.Blocked, subprocess.SubprocessError) as error:
        report.update(status='failed', reason=str(error) if type(error) in (ValueError, common.Blocked)
                      else type(error).__name__)
    report['completed_at_utc'] = now()
    report['files'] = [row for row in inventory(output) if row['path'] != relative(result_file)]
    write(result_file, report)
    return 0 if report['status'] == 'passed' else 1


def self_test():
    """Small structural regressions, without children, native calls or a display."""
    stages = stage_configuration()
    assert stages[0]['methods'] == ('nativeFileCategoryVisualAudit',) and stages[0]['timeout_seconds'] == 90
    assert stages[1]['methods'] == LAYOUT_METHODS and stages[1]['timeout_seconds'] == 240
    report = initial_report('a' * 40, 'b' * 64)
    assert report['full_source_sha'] == 'a' * 40 and report['binary_sha256'] == 'b' * 64
    assert report['kind'] == 'focused_failure_diagnosis' and all(report[key] is False for key in
        ('full45', 'full108', 'manual_native_acceptance', 'reference_performance_accepted', 'win11_accepted',
         'physical_dual_monitor_accepted', 'release_authorized', 'final_acceptance'))
    methods = stages[1]['methods']
    log = '\n'.join('PASS : BrowserTest::' + method.replace(':', '(') + (')' if ':' in method else '()')
                    for method in methods) + '\nCreating QRhi with backend D3D11\nTotals: 7 passed, 0 failed, 0 skipped\n'
    check_log(log, methods)
    rejected = 0
    for text in (log.replace('0 failed', '1 failed'), log + 'SKIP',
                 log.replace('zh-CN-menu', 'en-US-menu'), log.replace('D3D11', 'OpenGL')):
        try:
            check_log(text, methods)
        except ValueError:
            rejected += 1
        else:
            raise AssertionError('incomplete_or_failed_stage_accepted')
    (ROOT / 'build').mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='focused-native-selftest-', dir=ROOT / 'build') as temporary:
        folder = Path(temporary)
        fixture = folder / 'fixture.zip'
        visual.make_fixture(fixture)
        with zipfile.ZipFile(fixture) as archive:
            assert len(archive.infolist()) == 15 and set(archive.namelist()) == set(visual.FILES) | {'folder/'}
            assert all(entry.file_size == 0 for entry in archive.infolist())
        env = stage_environment(stages[1], folder, fixture, common.digest(fixture))
        assert env['QT_SCALE_FACTOR'] == '1' and env['QSG_RHI_BACKEND'] == 'd3d11'
        assert env['XDZIP_SETTINGS_PATH'] == str(folder / 'settings.ini')
        assert env['XDZIP_ABOUT_UPDATE_AUDIT'] == str(folder / 'images' / 'about')
    print(json.dumps(dict(status='self_test_passed', checks=5 + rejected, synthetic_data_only=True,
                          native_processes_started=0, full45=False, full108=False)))
    return 0


def main():
    if sys.argv[1:] == ['--self-test']:
        return self_test()
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--tests', type=Path, required=True)
    parser.add_argument('--source', required=True)
    parser.add_argument('--binary-sha256', required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    try:
        return run(args)
    except (ValueError, OSError, common.Blocked, subprocess.SubprocessError) as error:
        print(json.dumps(dict(status='failed_preflight', error_type=type(error).__name__)))
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
