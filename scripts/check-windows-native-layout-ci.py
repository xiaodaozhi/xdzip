# scripts/check-windows-native-layout-ci.py

"""Collect hosted native client layouts at three process DPR overrides.

This is automated virtual-desktop evidence, not physical DPI, dual-monitor,
reference-machine performance, or human visual acceptance.
"""
import argparse
from datetime import datetime, timezone
import importlib.util
import json
import math
import os
from pathlib import Path
import re
import struct
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('layout_ci_owned_processes', ROOT / 'scripts/check-windows-native-ci.py')
common = importlib.util.module_from_spec(spec)
spec.loader.exec_module(common)
visual_spec = importlib.util.spec_from_file_location('layout_ci_visual_contracts', ROOT / 'scripts/check-windows-native-visual-ci.py')
visual = importlib.util.module_from_spec(visual_spec)
visual_spec.loader.exec_module(visual)
METHODS = ('nativeUiLayoutAcceptance', 'fixedFileActionsAndColumnResize',
           'tabContextClosesStableWindowLocalRanges', 'aboutUpdateStateLayoutAndReadOnlyActions',
           'directoryContextExpansionReasons')
LANGUAGES, THEMES, SCALES, WIDTHS = ('zh-CN', 'en-US'), ('light', 'dark'), ('1', '1.25', '1.5'), (832, 1024, 1440)
SCENES = {f'{lang}-{theme}-{scale}-{width}' for lang in LANGUAGES for theme in THEMES
          for scale in SCALES for width in WIDTHS}


def require(condition, code):
    if not condition:
        raise ValueError(code)


def read(file):
    return json.loads(file.read_text(encoding='utf-8-sig'))


def png(file):
    with file.open('rb') as stream:
        header = stream.read(24)
    require(len(header) == 24 and header[:8] == b'\x89PNG\r\n\x1a\n' and header[12:16] == b'IHDR', 'invalid_png')
    size = struct.unpack('>II', header[16:24])
    require(all(value > 0 for value in size), 'empty_png')
    return size


def number(value):
    return type(value) in (int, float) and math.isfinite(value)


def check_log(text):
    require(re.search(r'Totals:\s+12 passed, 0 failed, 0 skipped', text)
            and not re.search(r'FAIL!|QFATAL|SKIP', text), 'incomplete_or_failed_qtest_matrix')
    rows = [(method, '') for method in METHODS[:-1]] + [('directoryContextExpansionReasons', language + '-' + key)
            for language in LANGUAGES for key in ('mouse', 'menu', 'shift-f10')]
    for method, row in rows:
        require(re.search(r'PASS\s*:\s*BrowserTest::' + re.escape(method + '(' + row + ')'), text), 'required_qtest_row_missing')
    require('Creating QRhi with backend D3D11' in text, 'actual_d3d11_log_required')


def check_rows(raw, source, target):
    require(raw.get('revision') == source[:12] and raw.get('dirty') is False
            and raw.get('configuration') == 'Release' and raw.get('platform') == 'windows'
            and raw.get('qt') == '6.8.2' and raw.get('requested_backend') in ('', None)
            and raw.get('qt_scale_factor_override') == target, 'layout_build_or_environment_mismatch')
    rows = raw.get('measurements', [])
    require(len(rows) == 36 and {row.get('scene') for row in rows} == SCENES, 'exact_36_layout_scenes_required')
    layouts, clamped = [], []
    for row in rows:
        scene = row['scene']
        language, theme, scale, width = scene.rsplit('-', 3)
        requested = int(width)
        require(row.get('language') == language and row.get('dark') is (theme == 'dark')
                and row.get('font_scale') == float(scale) and number(row.get('dpr'))
                and abs(row['dpr'] - float(target)) < .01, 'actual_qpa_dpr_or_scene_mismatch')
        require(number(row.get('width')) and 832 <= row['width'] <= requested
                and number(row.get('height')) and row['height'] > 0, 'invalid_actual_window_size')
        actions, columns = row.get('actions', []), row.get('columns', [])
        require(len(actions) in (3, 4) and all(action.get('width') == 28 for action in actions)
                and all(number(action.get('gap')) and abs(action['gap'] - 6.4) < .5 for action in actions[1:]),
                'fixed_action_geometry_regressed')
        require(len(columns) == 5 and {column.get('column') for column in columns} == set(range(5))
                and all(number(column.get('width')) and column['width'] > 0 and column.get('font', 0) > 0
                        and number(column.get('arrow_gap')) and abs(column['arrow_gap'] - 6) < .5 for column in columns),
                'five_columns_or_arrow_gap_regressed')
        if requested == 1024:
            require(.95 <= row.get('menu_table_font_ratio', 0) <= 1.10 and row.get('separator_height') == 1,
                    'menu_body_scale_or_separator_regressed')
        item = dict(scene=scene, requested_width=requested, actual_width=row['width'],
                    requested_height=544 if requested == 832 else 700, actual_height=row['height'], actual_dpr=row['dpr'])
        item['requested_size_realized'] = item['requested_width'] == item['actual_width'] and item['requested_height'] == item['actual_height']
        layouts.append(item)
        if not item['requested_size_realized']:
            clamped.append(dict(item, reason='hosted_virtual_desktop_size_limit', accepted_as_requested_size=False))
    return layouts, clamped


def check_group(folder, source, target):
    check_log((folder / 'qt.log').read_text(encoding='utf-8-sig'))
    images = folder / 'images'
    layouts, clamped = check_rows(read(images / 'measurements.json'), source, target)
    originals = sorted(images.rglob('*.png'))
    require(originals, 'original_layout_images_missing')
    for file in originals:
        data, pixels = read(file.with_suffix('.json')), png(file)
        require(data.get('platform') == 'windows' and number(data.get('dpr'))
                and abs(data['dpr'] - float(target)) < .01 and pixels == (data.get('imageWidth'), data.get('imageHeight')),
                'paired_image_dpr_or_geometry_mismatch')
        require(all(number(data.get(key)) and data[key] > 0 and abs(pixel - data[key] * data['dpr']) <= 1
                    for pixel, key in zip(pixels, ('width', 'height'))), 'original_pixel_logical_size_mismatch')
    for item in layouts:
        data = read(images / (item['scene'] + '-main.json'))
        require((images / (item['scene'] + '-main.png')).is_file()
                and data.get('width') == item['actual_width'] and data.get('height') == item['actual_height'],
                'main_layout_actual_size_mismatch')
    reasons = []
    for language in LANGUAGES:
        for theme in THEMES:
            for scale in SCALES:
                for key in ('mouse', 'menu', 'shift-f10'):
                    stem = f'directory-reason-{language}-{theme}-{scale}-{key}'
                    data = read(images / (stem + '.json'))
                    require((images / (stem + '.png')).is_file(), 'root_reason_image_missing')
                    visual.check_directory_command_capture(data, language)
                    reasons.append(stem)
    return dict(actual_layout_count=len(layouts), actual_dpr=float(target), layouts=layouts, clamped=clamped,
                actual_widths=sorted({row['actual_width'] for row in layouts}), directory_reason_captures=len(reasons),
                accessible_reason_only_captures=len(reasons),
                all_generated_pngs_paired_and_measured=len(originals),
                full_requested_size_matrix_realized=not clamped)


def inventory(folder):
    return [dict(path=file.relative_to(ROOT).as_posix(), bytes=file.stat().st_size, sha256=common.digest(file))
            for file in sorted(folder.rglob('*')) if file.is_file() and 'private-temp' not in file.relative_to(folder).parts]


def run(args):
    require(sys.platform == 'win32' and os.environ.get('GITHUB_ACTIONS') == 'true'
            and os.environ.get('RUNNER_ENVIRONMENT') == 'github-hosted' and os.environ.get('RUNNER_OS') == 'Windows',
            'windows_github_hosted_only')
    require(re.fullmatch('[0-9a-f]{40}', args.source) and re.fullmatch('[0-9a-f]{64}', args.binary_sha256), 'invalid_source_or_binary_hash')
    tests, output = args.tests.resolve(), args.output.resolve()
    require(tests.is_file() and tests.is_relative_to(ROOT / 'build') and output.is_relative_to(ROOT / 'build')
            and output != ROOT / 'build', 'owned_build_paths_required')
    require(not output.exists() or (output.is_dir() and all(file.name == 'private-temp' and file.is_dir()
            for file in output.iterdir())), 'fresh_evidence_directory_required')
    require(all(os.environ.get(key) and Path(os.environ[key]).is_dir() for key in ('TEMP', 'TMP')), 'protected_parent_temp_required')
    output.mkdir(parents=True, exist_ok=True)
    report = dict(schema_version=1, status='running', full_source_sha=args.source, binary_sha256=args.binary_sha256,
                  run_kind='hosted_automated_native_layout_collection', started_at_utc=datetime.now(timezone.utc).isoformat(),
                  methods=list(METHODS), actual_layout_count=0, full_requested_size_matrix_realized=False,
                  real_user_click=False, physical_dpr_transitions_accepted=False, physical_dual_monitor_accepted=False,
                  reference_performance_accepted=False, manual_visual_accepted=False, win11_accepted=False,
                  scope='Native QQuickWindow client captures with process DPR overrides; no physical screen/DPI or OS-frame approval.', groups=[])
    result_file = output / 'result.json'
    try:
        report['source_before'] = common.source_snapshot(tests, args.source, args.binary_sha256)
        display = common.display_state(args.source)
        report.update(display=display, virtual_display_mode_changed=display['virtual_display_mode_changed'],
                      os_display_settings_changed=display['os_display_settings_changed'])
        (output / 'browser-test-source.cpp').write_bytes((ROOT / 'apps/desktop/tests/browser_test.cpp').read_bytes())
        for label in ('1', '1.25', '2'):
            folder = output / ('dpr-' + label)
            images = folder / 'images'
            (images / 'about').mkdir(parents=True)
            env = common.native_environment(folder)
            for key in list(env):
                if key.startswith('XDZIP_') and 'AUDIT' in key:
                    env.pop(key)
            env.update(QSG_INFO='1', QT_SCALE_FACTOR=label, XDZIP_NATIVE_LAYOUT_AUDIT=str(images),
                       XDZIP_ABOUT_UPDATE_AUDIT=str(images / 'about'))
            entry = dict(target_dpr=label, process_scale_override=label, timeout_seconds=240, status='failed')
            try:
                completed, receipt = common.run_owned([str(tests), *METHODS, '-v1', '-o', str(folder / 'qt.log') + ',txt'],
                    env, folder, tests, args.source, args.binary_sha256, timeout=240)
                entry['execution'] = receipt
                require(completed.returncode == 0 and receipt['timed_out'] is False, 'matrix_child_failed_or_timed_out')
                entry['collection'] = check_group(folder, args.source, label)
                entry['status'] = 'passed'
            except (ValueError, KeyError, OSError, TypeError, common.Blocked, subprocess.SubprocessError) as error:
                entry['reason'] = str(error) if type(error) in (ValueError, common.Blocked) else type(error).__name__
                if (folder / 'process-receipt.json').is_file():
                    entry['execution'] = read(folder / 'process-receipt.json')
            entry['files'] = inventory(folder)
            report['groups'].append(entry)
            result_file.write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
            print(json.dumps(dict(dpr=label, status=entry['status'])), flush=True)
            common.source_snapshot(tests, args.source, args.binary_sha256)
        report['source_after'] = common.source_snapshot(tests, args.source, args.binary_sha256)
        require(report['source_before'] == report['source_after'] and common.display_state(args.source) == display,
                'source_or_display_changed_during_matrix')
        good = len(report['groups']) == 3 and all(row['status'] == 'passed' for row in report['groups'])
        report['actual_layout_count'] = sum(row.get('collection', {}).get('actual_layout_count', 0) for row in report['groups'])
        report['clamped'] = [dict(row, target_dpr=group['target_dpr']) for group in report['groups']
                             for row in group.get('collection', {}).get('clamped', [])]
        report['full_requested_size_matrix_realized'] = good and not report['clamped']
        report['status'] = 'passed' if good and report['actual_layout_count'] == 108 else 'failed'
    except (ValueError, KeyError, OSError, common.Blocked, subprocess.SubprocessError) as error:
        report.update(status='failed', reason=str(error) if type(error) in (ValueError, common.Blocked) else type(error).__name__)
    report['completed_at_utc'] = datetime.now(timezone.utc).isoformat()
    report['files'] = [row for row in inventory(output) if row['path'] != result_file.relative_to(ROOT).as_posix()]
    result_file.write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    return 0 if report['status'] == 'passed' else 1


def self_test():
    """Synthetic data contracts only; no child, display query, or GUI."""
    source, rows = 'a' * 40, []
    for scene in sorted(SCENES):
        language, theme, scale, width = scene.rsplit('-', 3)
        requested = int(width)
        rows.append(dict(scene=scene, language=language, dark=theme == 'dark', font_scale=float(scale),
            width=min(requested, 944), height=544 if requested == 832 else 700, dpr=2,
            actions=[dict(width=28, gap=0), dict(width=28, gap=6.4), dict(width=28, gap=6.4)],
            columns=[dict(column=i, width=100, font=12, arrow_gap=6) for i in range(5)],
            menu_table_font_ratio=1, separator_height=1))
    raw = dict(revision=source[:12], dirty=False, configuration='Release', platform='windows', qt='6.8.2',
               requested_backend='', qt_scale_factor_override='2', measurements=rows)
    layouts, clamps = check_rows(raw, source, '2')
    assert len(layouts) == 36 and len(clamps) == 24 and all(not row['accepted_as_requested_size'] for row in clamps)
    checks = 1
    for change in ('dpr', 'source', 'missing', 'gap'):
        bad = json.loads(json.dumps(raw))
        if change == 'dpr': bad['measurements'][0]['dpr'] = 1
        elif change == 'source': bad['revision'] = 'b' * 12
        elif change == 'missing': bad['measurements'].pop()
        else: bad['measurements'][0]['actions'][1]['gap'] = 40
        try:
            check_rows(bad, source, '2')
        except ValueError:
            checks += 1
        else:
            raise AssertionError('invalid_matrix_accepted')
    valid_log = '\n'.join('PASS : BrowserTest::' + method + '()' for method in METHODS[:-1])
    valid_log += '\n' + '\n'.join('PASS : BrowserTest::directoryContextExpansionReasons(' + lang + '-' + key + ')'
        for lang in LANGUAGES for key in ('mouse', 'menu', 'shift-f10'))
    valid_log += '\nCreating QRhi with backend D3D11\nTotals: 12 passed, 0 failed, 0 skipped\n'
    check_log(valid_log)
    checks += 1
    for text in (valid_log + 'SKIP', valid_log.replace('0 failed', '1 failed'), valid_log.replace('D3D11', 'OpenGL')):
        try:
            check_log(text)
        except ValueError:
            checks += 1
        else:
            raise AssertionError('invalid_qtest_log_accepted')
    geometry = dict(nodes=[dict(name='directoryExpansionCommand', visible=True, enabled=False,
                               text='收起子目录', disabledReason='根目录始终展开',
                               accessibleDescription='根目录始终展开'),
                           dict(name='directoryExpansionCommandLabel', visible=True, text='收起子目录',
                                width=180, height=24, contentWidth=120)])
    visual.check_directory_command_capture(geometry, 'zh-CN')
    checks += 1
    geometry['nodes'].append(dict(visible=True, text='根目录始终展开'))
    try:
        visual.check_directory_command_capture(geometry, 'zh-CN')
    except ValueError:
        checks += 1
    else:
        raise AssertionError('visible_disabled_reason_accepted')
    return dict(status='self_test_passed', checks=checks, synthetic_data_only=True, native_processes_started=0)


def main():
    if sys.argv[1:] == ['--self-test']:
        print(json.dumps(self_test()))
        return 0
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--tests', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--source', required=True)
    parser.add_argument('--binary-sha256', required=True)
    args = parser.parse_args()
    try:
        return run(args)
    except (ValueError, OSError, common.Blocked, subprocess.SubprocessError) as error:
        print(json.dumps(dict(status='failed_preflight', error_type=type(error).__name__)))
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
