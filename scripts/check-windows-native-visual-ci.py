# scripts/check-windows-native-visual-ci.py

"""Collect automated Windows-hosted native visuals, without manual approval.

The Release test build and Qt SDK bin directory must already be available. Each
case keeps its existing Qt/native guards and runs in a separate owned process.
"""
import argparse
from datetime import datetime, timezone
import importlib.util
import json
import math
import os
from pathlib import Path
import platform
import re
import struct
import subprocess
import sys
import tempfile
import zipfile

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('visual_ci_owned_processes', ROOT / 'scripts/check-windows-native-ci.py')
common = importlib.util.module_from_spec(spec)
spec.loader.exec_module(common)
CASES = (
    ('directory_reason', 'directoryContextExpansionReasons', 'XDZIP_NATIVE_LAYOUT_AUDIT'),
    ('initial_about', 'versionAboutWindow', 'XDZIP_ABOUT_AUDIT'),
    ('file_gallery', 'nativeFileCategoryVisualAudit', 'XDZIP_FILE_ICON_AUDIT'),
    ('windows_frame', 'windowsCaptionControlsAndWindowState', 'XDZIP_WINDOWS_AUDIT'),
)
FILES = ('file-pdf.pdf', 'file-document.doc', 'file-spreadsheet.xls',
         'file-presentation.ppt', 'file-markdown.md', 'image-file.jpg', 'file-code.js',
         'file-audio.mp3', 'file-video.mp4', 'file-archive.zip', 'file-installer.exe',
         'file-font.ttf', 'file-disk.iso', 'unknown.xyz')
SCALES = ('1', '1.25', '1.5')


def now():
    return datetime.now(timezone.utc).isoformat()


def require(condition, code):
    if not condition:
        raise ValueError(code)


def digest(file):
    return common.digest(file)


def relative(file):
    return file.relative_to(ROOT).as_posix()


def matches(value, source, binary):
    return value.get('full_source_sha') == source and value.get('clean') is True and value.get('binary_sha256') == binary


def inventory(folder):
    return [dict(path=relative(file), bytes=file.stat().st_size, sha256=digest(file))
            for file in sorted(folder.rglob('*')) if file.is_file()
            and 'private-temp' not in file.relative_to(folder).parts]


def make_fixture(file):
    with zipfile.ZipFile(file, 'x', compression=zipfile.ZIP_STORED) as archive:
        for name in (*FILES, 'folder/'):
            info = zipfile.ZipInfo(name, (2020, 1, 1, 0, 0, 0))
            info.create_system = 3
            info.external_attr = (0o40755 if name.endswith('/') else 0o100644) << 16
            archive.writestr(info, b'')
    require(file.stat().st_size <= 4096, 'fixture_too_large')


def read(file):
    return json.loads(file.read_text(encoding='utf-8-sig'))


def png(file):
    with file.open('rb') as stream:
        header = stream.read(24)
    require(len(header) == 24 and header[:8] == b'\x89PNG\r\n\x1a\n'
            and header[12:16] == b'IHDR', 'invalid_png')
    width, height = struct.unpack('>II', header[16:24])
    require(width > 0 and height > 0, 'empty_png')
    return [width, height]


def paired(images, stem):
    geometry = read(images / (stem + '.json'))
    size = png(images / (stem + '.png'))
    require(geometry.get('platform') == 'windows' and geometry.get('dpr') == 1,
            'actual_windows_dpr1_required')
    require(size == [geometry.get('imageWidth'), geometry.get('imageHeight')]
            and all(abs(size[i] - geometry[key]) <= 1 for i, key in enumerate(('width', 'height'))),
            'png_geometry_size_mismatch')
    return geometry


def check_log(text, method):
    totals = re.search(r'Totals:\s+(\d+) passed, 0 failed, 0 skipped', text)
    expected = [f'{language}-{key}' for language in ('zh-CN', 'en-US')
                for key in ('mouse', 'menu', 'shift-f10')] if method == 'directoryContextExpansionReasons' else ['']
    require(totals is not None and int(totals.group(1)) == len(expected) + 2
            and not re.search(r'FAIL!|QFATAL|SKIP', text), 'qtest_not_complete_pass')
    for row in expected:
        require(re.search(r'PASS\s*:\s*BrowserTest::' + re.escape(method + '(' + row + ')'), text),
                'required_qtest_row_missing')
    require('Creating QRhi with backend D3D11' in text, 'actual_d3d11_log_required')


def check_directory_command_capture(geometry, language):
    reason = 'Root is always expanded' if language == 'en-US' else '根目录始终展开'
    title = 'Collapse subfolders' if language == 'en-US' else '收起子目录'
    nodes = geometry.get('nodes', [])
    commands = [node for node in nodes if node.get('name') == 'directoryExpansionCommand'
                and node.get('visible') is True]
    require(len(commands) == 1, 'visible_directory_expansion_command_required')
    command = commands[0]
    require(command.get('enabled') is False and command.get('text') == title
            and command.get('disabledReason') == reason
            and command.get('accessibleDescription') == reason,
            'disabled_command_accessible_reason_mismatch')
    labels = [node for node in nodes if node.get('name') == 'directoryExpansionCommandLabel'
              and node.get('visible') is True]
    require(len(labels) == 1 and labels[0].get('text') == title
            and all(type(labels[0].get(key)) in (int, float) and math.isfinite(labels[0][key])
                    for key in ('width', 'height', 'contentWidth'))
            and labels[0]['width'] > 0 and labels[0]['height'] > 0
            and 0 <= labels[0]['contentWidth'] <= labels[0]['width'] + 1,
            'visible_directory_command_label_missing_or_overflowing')
    reasons = {node['disabledReason'] for node in nodes if node.get('visible') is True
               and isinstance(node.get('disabledReason'), str)
               and node['disabledReason']}
    require(not any(node.get('visible') is True and isinstance(node.get('text'), str)
                    and any(value in node['text'] for value in reasons) for node in nodes),
            'disabled_reason_must_not_be_visible_text')
    require(not any(node.get('visible') is True and 'tooltip' in node.get('class', '').lower()
                    for node in nodes), 'visible_menu_tooltip_forbidden')


def check_captures(role, images, source, fixture_hash):
    scenes = {language + '-' + theme + '-' + scale for language in ('zh-CN', 'en-US')
              for theme in ('light', 'dark') for scale in SCALES}
    if role == 'directory_reason':
        expected = {'directory-reason-' + scene + '-' + key for scene in scenes
                    for key in ('mouse', 'menu', 'shift-f10')}
    elif role == 'initial_about':
        expected = scenes
    elif role == 'file_gallery':
        catalog = read(images / 'file-icon-catalog.json')
        require(catalog.get('schema_version') == 2 and catalog.get('revision') == source[:12]
                and catalog.get('dirty') is False and catalog.get('configuration') == 'Release'
                and catalog.get('platform') == 'windows' and catalog.get('fixture_sha256') == fixture_hash,
                'gallery_identity_mismatch')
        captures = catalog.get('captures', [])
        require(len(captures) == 24 and {row.get('scene') for row in captures}
                == {scene + '-' + view for scene in scenes for view in ('list', 'grid')},
                'gallery_scene_matrix_incomplete')
        expected = set()
        for capture in captures:
            require(capture.get('dpr') == 1 and capture.get('graphics_api') == 4
                    and capture.get('covered_files') == 15 and 1 <= len(capture.get('frames', [])) <= 2,
                    'gallery_capture_invalid')
            covered = set()
            for index, frame in enumerate(capture['frames']):
                stem = capture['scene'] + '-part' + str(index)
                require(frame.get('part') == index and frame.get('image') == stem + '.png'
                        and frame.get('geometry') == stem + '.json'
                        and frame.get('hover_paint_changed') is True and frame.get('selected_count') == 1,
                        'gallery_frame_invalid')
                require(frame.get('selected_paint_checked') is (index == 0)
                        and frame.get('selected_paint_changed') is (index == 0), 'gallery_selected_paint_invalid')
                icons = frame.get('icons', [])
                require(icons and all(icon.get('inside_viewport') is True for icon in icons), 'gallery_icons_outside_view')
                covered.update(icon['file'] for icon in icons)
                expected.add(stem)
            require(covered == set(FILES) | {'folder'}, 'gallery_file_union_incomplete')
    else:
        frame = read(images / 'windows-native-frame.json')
        require(frame.get('attached') is True and frame.get('captionStyle') is True
                and frame.get('resizeStyle') is True and frame.get('composition') is True
                and frame.get('nonClientRendering') is True and frame.get('topInsetPixels') == 1,
                'native_dwm_frame_unavailable')
        require(png(images / 'windows-dwm-frame.png')[0] > 0, 'missing_owned_backdrop_frame')
        expected = {'windows-caption'}
    actual = {file.stem for file in images.glob('*.png')}
    require(actual == expected | ({'windows-dwm-frame'} if role == 'windows_frame' else set()),
            'capture_png_set_mismatch')
    for stem in expected:
        geometry = paired(images, stem)
        if role == 'directory_reason':
            language = 'en-US' if stem.startswith('directory-reason-en-US-') else 'zh-CN'
            check_directory_command_capture(geometry, language)
    return dict(scenes=24 if role == 'file_gallery' else len(expected), client_frames=len(expected),
                os_frame_images=1 if role == 'windows_frame' else 0, actual_dpr=1,
                accessible_reason_only_captures=len(expected) if role == 'directory_reason' else 0)


def run(args):
    tests, output = args.tests.resolve(), args.output.resolve()
    require(sys.platform == 'win32' and os.environ.get('GITHUB_ACTIONS') == 'true'
            and os.environ.get('RUNNER_ENVIRONMENT') == 'github-hosted'
            and os.environ.get('RUNNER_OS') == 'Windows', 'windows_github_hosted_only')
    require(re.fullmatch('[0-9a-f]{40}', args.source) and re.fullmatch('[0-9a-f]{64}', args.binary_sha256), 'invalid_source_or_binary_hash')
    require(tests.is_file() and tests.is_relative_to(ROOT / 'build') and output.is_relative_to(ROOT / 'build'),
            'build_output_and_owned_test_required')
    # The CI wrapper may create only its protected TEMP directory beforehand.
    require(not output.exists() or (output.is_dir() and all(file.name == 'private-temp'
            and file.is_dir() for file in output.iterdir())), 'fresh_evidence_directory_required')
    output.mkdir(parents=True, exist_ok=True)
    require(all(os.environ.get(key) and Path(os.environ[key]).is_dir() for key in ('TEMP', 'TMP')),
            'wrapper_protected_temp_required')
    report = dict(schema_version=1, status='running', full_source_sha=args.source, binary_sha256=args.binary_sha256,
                  started_at_utc=now(), mode='automated_github_hosted_native_visual',
                  real_user_click=False, manual_visual_accepted=False, fixed_reference_performance_accepted=False,
                  physical_dual_monitor_accepted=False, win11_accepted=False,
                  virtual_display_mode_changed=None, os_display_settings_changed=None,
                  platform=dict(system=platform.system(), release=platform.release(), version=platform.version()),
                  runner={key: os.environ.get(key) for key in ('GITHUB_RUN_ID', 'GITHUB_RUN_ATTEMPT', 'GITHUB_JOB', 'ImageOS', 'ImageVersion')},
                  limits=['Automated native client/frame capture only; actual images still need independent inspection.',
                          'Gallery scene and selection setup use APIs; hover paint uses the real system pointer.',
                          'No menu timing, physical monitor, installer, association, or manual acceptance.'],
                  cases=[])
    result_file = output / 'result.json'
    try:
        initial = common.source_snapshot(tests, args.source, args.binary_sha256)
        display = common.display_state(args.source)
        report.update(display)
        (output / 'browser-test-source.cpp').write_bytes((ROOT / 'apps/desktop/tests/browser_test.cpp').read_bytes())
        (output / 'source.patch').write_bytes(subprocess.check_output(['git', 'diff'], cwd=ROOT, timeout=10))
        fixture = output / 'file-icon-coverage.zip'
        make_fixture(fixture)
        fixture_hash = digest(fixture)
        report.update(initial=initial, desktop=common.desktop_diagnostics(), build_version_sha256=initial['build_version_sha256'],
                      fixture=dict(path=relative(fixture), bytes=fixture.stat().st_size, sha256=fixture_hash))
        for role, method, variable in CASES:
            folder = output / role
            images = folder / 'images'
            images.mkdir(parents=True)
            env = common.native_environment(folder)
            for key in list(env):
                if key.startswith('XDZIP_') and ('AUDIT' in key or key.startswith('XDZIP_FILE_ICON_')):
                    env.pop(key)
            for key in ('QT_QUICK_BACKEND', 'QT_SCREEN_SCALE_FACTORS', 'QT_DEVICE_PIXEL_RATIO', 'QT_FONT_DPI'):
                env.pop(key, None)
            env.update(QT_QPA_PLATFORM='windows', QSG_RHI_BACKEND='d3d11', QSG_INFO='1', QT_SCALE_FACTOR='1',
                       XDZIP_SETTINGS_PATH=str(folder / 'settings.ini'))
            env[variable] = str(images)
            if role == 'directory_reason':
                env['XDZIP_DIRECTORY_CONTEXT_DIAGNOSTICS'] = str(folder / 'failure-images')
            if role == 'file_gallery':
                env.update(XDZIP_FILE_ICON_FIXTURE=str(fixture), XDZIP_FILE_ICON_FIXTURE_SHA256=fixture_hash)
            entry = dict(role=role, function=method, started_at_utc=now(), timeout_seconds=90,
                         argv=[relative(tests), method, '-v1', '-o', relative(folder / 'qt.log') + ',txt'])
            log = folder / 'qt.log'
            try:
                _, execution = common.run_owned([str(tests), method, '-v1', '-o', str(log) + ',txt'],
                                               env, folder, tests, args.source, args.binary_sha256, timeout=90)
                entry.update(execution=execution, pid=execution['owned_child_pid'], exit_code=execution['exit_code'],
                             timed_out=execution['timed_out'], seconds=execution['seconds'])
                require(entry['exit_code'] == 0 and not entry['timed_out'], 'native_case_failed_or_timed_out')
                check_log(log.read_text(encoding='utf-8'), method)
                entry['captures'] = check_captures(role, images, args.source, fixture_hash)
                entry['status'] = 'passed'
            except (ValueError, KeyError, OSError, TypeError, common.Blocked, subprocess.SubprocessError) as error:
                entry.update(status='failed', failure_code=str(error) if type(error) in (ValueError, common.Blocked) else type(error).__name__)
                if (folder / 'process-receipt.json').is_file():
                    entry['execution'] = read(folder / 'process-receipt.json')
            entry['completed_at_utc'] = now()
            entry['files'] = inventory(folder)
            report['cases'].append(entry)
            result_file.write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
            print(json.dumps({key: entry.get(key) for key in ('role', 'status', 'pid', 'exit_code', 'timed_out')}), flush=True)
            common.source_snapshot(tests, args.source, args.binary_sha256)
        require(common.display_state(args.source) == display, 'display_mode_or_receipt_changed_during_run')
        report.update(final=common.source_snapshot(tests, args.source, args.binary_sha256), completed_at_utc=now())
        report['status'] = 'passed' if len(report['cases']) == len(CASES) and all(row['status'] == 'passed' for row in report['cases']) and matches(report['final'], args.source, args.binary_sha256) else 'failed'
    except (ValueError, KeyError, OSError, common.Blocked, subprocess.SubprocessError) as error:
        report.update(status='failed', failure_code=str(error) if type(error) in (ValueError, common.Blocked) else type(error).__name__, completed_at_utc=now())
    report['files'] = [row for row in inventory(output) if row['path'] != relative(result_file)]
    result_file.write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    return 0 if report['status'] == 'passed' else 1


def main():
    if sys.argv[1:] == ['--self-test']:
        return self_test()
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--tests', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--source', required=True)
    parser.add_argument('--binary-sha256', required=True)
    args = parser.parse_args()
    try:
        return run(args)
    except (ValueError, OSError, subprocess.SubprocessError) as error:
        print(json.dumps(dict(status='failed_preflight', error_type=type(error).__name__)))
        return 1


def self_test():
    """Synthetic admission/fixture tests only, without any native child or GUI."""
    count = 0

    def rejected(action):
        nonlocal count
        try:
            action()
        except (ValueError, KeyError):
            count += 1
            return
        raise AssertionError('invalid_capture_was_accepted')

    method = 'versionAboutWindow'
    log = ('PASS : BrowserTest::versionAboutWindow()\n'
           'QDEBUG : Creating QRhi with backend D3D11\nTotals: 3 passed, 0 failed, 0 skipped\n')
    check_log(log, method)
    count += 1
    for text in (log.replace('0 failed', '1 failed'), log + 'SKIP',
                 log.replace('D3D11', 'OpenGL'), log.replace(method, 'otherMethod')):
        rejected(lambda: check_log(text, method))
    (ROOT / 'build').mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='visual-driver-selftest-', dir=ROOT / 'build') as temporary:
        folder = Path(temporary)
        fixture = folder / 'fixture.zip'
        make_fixture(fixture)
        with zipfile.ZipFile(fixture) as archive:
            require(set(archive.namelist()) == set(FILES) | {'folder/'}
                    and all(item.file_size == 0 for item in archive.infolist()), 'synthetic_fixture_mismatch')
        count += 1
        source = 'a' * 40
        images = folder / 'images'
        images.mkdir()
        captures = []
        for language in ('zh-CN', 'en-US'):
            for theme in ('light', 'dark'):
                for scale in SCALES:
                    for view in ('list', 'grid'):
                        scene = '-'.join((language, theme, scale, view))
                        stem = scene + '-part0'
                        (images / (stem + '.png')).write_bytes(b'\x89PNG\r\n\x1a\n' + b'\0\0\0\rIHDR' + struct.pack('>II', 1024, 900))
                        (images / (stem + '.json')).write_text(json.dumps(dict(platform='windows', dpr=1,
                            width=1024, height=900, imageWidth=1024, imageHeight=900)), encoding='utf-8')
                        captures.append(dict(scene=scene, dpr=1, graphics_api=4, covered_files=15, frames=[dict(
                            part=0, image=stem + '.png', geometry=stem + '.json', hover_paint_changed=True,
                            selected_count=1, selected_paint_checked=True, selected_paint_changed=True,
                            icons=[dict(file=name, inside_viewport=True) for name in (*FILES, 'folder')])]))
        catalog = dict(schema_version=2, revision=source[:12], dirty=False, configuration='Release',
                       platform='windows', fixture_sha256=digest(fixture), captures=captures)
        file = images / 'file-icon-catalog.json'
        file.write_text(json.dumps(catalog), encoding='utf-8')
        check_captures('file_gallery', images, source, digest(fixture))
        count += 1
        rejected(lambda: check_captures('file_gallery', images, 'b' * 40, digest(fixture)))
        captures[0]['frames'][0]['icons'].pop()
        file.write_text(json.dumps(catalog), encoding='utf-8')
        rejected(lambda: check_captures('file_gallery', images, source, digest(fixture)))
        empty = images / 'empty.png'
        empty.write_bytes(b'')
        rejected(lambda: png(empty))
        command = dict(name='directoryExpansionCommand', visible=True, enabled=False,
                       text='Collapse subfolders', disabledReason='Root is always expanded',
                       accessibleDescription='Root is always expanded')
        label = dict(name='directoryExpansionCommandLabel', visible=True, text=command['text'],
                     width=180, height=24, contentWidth=140)
        geometry = dict(nodes=[command, label])
        check_directory_command_capture(geometry, 'en-US')
        count += 1
        geometry['nodes'].append(dict(visible=True, text=command['disabledReason']))
        rejected(lambda: check_directory_command_capture(geometry, 'en-US'))
        geometry['nodes'].pop()
        command['accessibleDescription'] = ''
        rejected(lambda: check_directory_command_capture(geometry, 'en-US'))
        command['accessibleDescription'] = command['disabledReason']
        geometry['nodes'].append(dict(visible=True, **{'class': 'ToolTip_QMLTYPE'}))
        rejected(lambda: check_directory_command_capture(geometry, 'en-US'))
    print(json.dumps(dict(status='self_test_passed', checks=count, synthetic_data_only=True,
                          native_processes_started=0, native_visual_accepted=False)))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
