# scripts/check-windows-native-ci.py

"""Hosted Windows menu diagnostics, never real-click/reference-machine approval."""
import argparse
import ctypes
from ctypes import wintypes
from datetime import datetime, timezone
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time
from types import SimpleNamespace

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('hosted_native_menus', ROOT / 'scripts/check-native-menus.py')
menus = importlib.util.module_from_spec(spec)
spec.loader.exec_module(menus)


class Blocked(RuntimeError):
    pass


class Failed(RuntimeError):
    pass


def digest(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def sanitized(text):
    # Hosted Linux/macOS checkouts can live inside the home directory. Replace
    # the more specific root first so identity labels are stable on every host.
    for path, label in sorted(((Path.home(), '<home>'), (ROOT, '<repo>')),
                              key=lambda item: len(str(item[0])), reverse=True):
        for value in (str(path), str(path).replace('\\', '/')):
            text = re.sub(re.escape(value), lambda _: label, text, flags=re.I)
    return text


def source_snapshot(tests, source, expected_binary):
    head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, stderr=subprocess.PIPE, timeout=10).decode().strip()
    clean = not subprocess.check_output(['git', 'status', '--porcelain'], cwd=ROOT, stderr=subprocess.PIPE, timeout=10).strip()
    version = tests.parent / 'generated/BuildVersion.h'
    text = version.read_text(encoding='utf-8')
    actual = digest(tests)
    if (head != source or not clean or actual != expected_binary
            or '#define XDZIP_REVISION "' + source[:12] + '"' not in text
            or '#define XDZIP_DIRTY false' not in text or '#define XDZIP_CONFIGURATION "Release"' not in text):
        raise Blocked('source_binary_or_release_metadata_mismatch')
    return dict(full_source_sha=head, clean=clean, binary_sha256=actual, build_version_sha256=digest(version))


def native_environment(output):
    env = dict(os.environ)
    for key in ('QT_QUICK_BACKEND', 'QT_SCALE_FACTOR', 'QT_SCREEN_SCALE_FACTORS', 'QT_DEVICE_PIXEL_RATIO',
                'QT_FONT_DPI', 'QT_ENABLE_HIGHDPI_SCALING', 'QT_SCALE_FACTOR_ROUNDING_POLICY',
                'GH_TOKEN', 'GITHUB_TOKEN', 'GH_ENTERPRISE_TOKEN', 'GITHUB_ENTERPRISE_TOKEN'):
        env.pop(key, None)
    env.update(QT_QPA_PLATFORM='windows', QSG_RHI_BACKEND='d3d11',
               XDZIP_SETTINGS_PATH=str(output / 'settings.ini'))
    return env


def display_state(source):
    """Use the controlled helper's source/run-bound receipt and live readback."""
    path = os.environ.get('XDZIP_NATIVE_CI_DISPLAY_RECEIPT')
    if not path:
        return dict(virtual_display_mode_changed=False, os_display_settings_changed=False,
                    managed_display_receipt=False)
    specification = importlib.util.spec_from_file_location('native_ci_display', ROOT / 'scripts/windows_native_ci_display.py')
    helper = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(helper)
    try:
        receipt = helper.validate_active_receipt(Path(path), source, os.environ)
        before, active = receipt['before'], receipt['active']
        fields = ('width', 'height', 'bits_per_pixel', 'frequency', 'orientation',
                  'position_x', 'position_y', 'display_flags')
        if any(type(mode.get(field)) is not int for mode in (before, active) for field in fields):
            raise ValueError('invalid_display_mode_fields')
        changed = any(before[field] != active[field] for field in fields)
        return dict(virtual_display_mode_changed=changed, os_display_settings_changed=changed,
                    managed_display_receipt=True, display_mode_receipt=receipt)
    except (ValueError, KeyError, OSError, TypeError) as error:
        raise Blocked('display_receipt_or_readback_rejected') from error


def desktop_diagnostics():
    """Read only the current process/session/desktop; never enumerate/capture windows."""
    result = dict(available=False, desktop_pixels_captured=False, foreign_window_identifiers_recorded=False)
    if sys.platform != 'win32':
        return result
    try:
        user = ctypes.WinDLL('user32', use_last_error=True)
        kernel = ctypes.WinDLL('kernel32', use_last_error=True)
        user.SetThreadDpiAwarenessContext.argtypes = [ctypes.c_void_p]
        user.SetThreadDpiAwarenessContext.restype = ctypes.c_void_p
        previous = user.SetThreadDpiAwarenessContext(ctypes.c_void_p(-4))
        try:
            user.OpenInputDesktop.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
            user.OpenInputDesktop.restype = wintypes.HANDLE
            user.CloseDesktop.argtypes = [wintypes.HANDLE]
            user.CloseDesktop.restype = wintypes.BOOL
            user.GetSystemMetrics.argtypes = [ctypes.c_int]
            user.GetSystemMetrics.restype = ctypes.c_int
            desktop = user.OpenInputDesktop(0, False, 1)  # DESKTOP_READOBJECTS, never switch desktops.
            result.update(available=True, input_desktop_accessible=bool(desktop),
                          observer_thread_per_monitor_v2=bool(previous),
                          primary_screen_width=user.GetSystemMetrics(0), primary_screen_height=user.GetSystemMetrics(1),
                          screen_count=user.GetSystemMetrics(80), remote_session=bool(user.GetSystemMetrics(0x1000)))
            if desktop:
                user.CloseDesktop(desktop)
            kernel.ProcessIdToSessionId.argtypes = [wintypes.DWORD, ctypes.POINTER(wintypes.DWORD)]
            kernel.ProcessIdToSessionId.restype = wintypes.BOOL
            session = wintypes.DWORD()
            if kernel.ProcessIdToSessionId(os.getpid(), ctypes.byref(session)):
                result.update(own_session_id=session.value, session_zero=session.value == 0)
            dwm = ctypes.WinDLL('dwmapi')
            dwm.DwmIsCompositionEnabled.argtypes = [ctypes.POINTER(wintypes.BOOL)]
            dwm.DwmIsCompositionEnabled.restype = ctypes.c_long
            enabled = wintypes.BOOL()
            result['dwm_composition_enabled'] = bool(enabled.value) if dwm.DwmIsCompositionEnabled(ctypes.byref(enabled)) == 0 else None
        finally:
            if previous:
                user.SetThreadDpiAwarenessContext(previous)
    except (OSError, AttributeError, ValueError):
        result['diagnostic_error'] = 'desktop_query_unavailable'
    return result


def owned_image_matches(process, tests):
    # The only inspected handle comes directly from our Popen. A failed query
    # rejects the sample; no foreign process/window is opened or activated.
    kernel = ctypes.WinDLL('kernel32', use_last_error=True)
    kernel.QueryFullProcessImageNameW.argtypes = [wintypes.HANDLE, wintypes.DWORD, wintypes.LPWSTR, ctypes.POINTER(wintypes.DWORD)]
    kernel.QueryFullProcessImageNameW.restype = wintypes.BOOL
    size = wintypes.DWORD(32768)
    image = ctypes.create_unicode_buffer(size.value)
    return bool(kernel.QueryFullProcessImageNameW(wintypes.HANDLE(int(process._handle)), 0, image, ctypes.byref(size))) and Path(image.value).resolve() == tests.resolve()


def run_owned(command, env, directory, tests, source, expected_binary, timeout=35):
    before = source_snapshot(tests, source, expected_binary)
    display_before = display_state(source)
    if Path(command[0]).resolve() != tests.resolve():
        raise Blocked('unexpected_child_executable')
    receipt = dict(source_before=before, display_before=display_before,
                   started_at_utc=datetime.now(timezone.utc).isoformat(),
                   argv=[sanitized(str(value)) for value in command], timed_out=False)
    start = time.monotonic()
    process = subprocess.Popen(command, cwd=ROOT, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    receipt['owned_child_pid'] = process.pid
    stdout, stderr = b'', b''
    try:
        try:
            receipt['binary_path_verified'] = owned_image_matches(process, tests)
        except OSError:
            receipt['binary_path_verified'] = False
        if not receipt['binary_path_verified']:
            raise Blocked('owned_child_image_query_failed')
        try:
            stdout, stderr = process.communicate(timeout=timeout)
        except subprocess.TimeoutExpired:
            receipt['timed_out'] = True
            process.kill()
            stdout, stderr = process.communicate(timeout=5)
            process.wait(timeout=5)
    finally:
        if process.poll() is None:
            process.kill()
            stdout, stderr = process.communicate(timeout=5)
            process.wait(timeout=5)
        receipt.update(exit_code=process.returncode, seconds=time.monotonic() - start,
                       finished_at_utc=datetime.now(timezone.utc).isoformat())
        try:
            receipt['source_after'] = source_snapshot(tests, source, expected_binary)
            receipt['display_after'] = display_state(source)
            receipt['identity_unchanged'] = receipt['source_after'] == before and receipt['display_after'] == display_before
        except (Blocked, OSError, subprocess.SubprocessError):
            receipt['identity_unchanged'] = False
        (directory / 'process-receipt.json').write_text(json.dumps(receipt, indent=2) + '\n', encoding='utf-8')
        (directory / 'console.log').write_text(sanitized((stdout + stderr).decode('utf-8', errors='replace')), encoding='utf-8')
        for file in directory.glob('*.log'):
            file.write_text(sanitized(file.read_text(encoding='utf-8', errors='replace')), encoding='utf-8')
    if not receipt['identity_unchanged']:
        raise Blocked('source_or_binary_changed_during_sample')
    return subprocess.CompletedProcess(command, process.returncode,
        sanitized(stdout.decode('utf-8', errors='replace')).encode('utf-8'),
        sanitized(stderr.decode('utf-8', errors='replace')).encode('utf-8')), receipt


def guard_sample(directory, case, result, receipt, source, contract):
    if receipt['timed_out']:
        raise Blocked('native_process_timeout')
    diagnostic = directory / 'capture-diagnostic.json'
    if (directory / 'occlusion-rejected.json').exists() or not diagnostic.is_file():
        raise Blocked('owned_capture_guard_unavailable')
    value = json.loads(diagnostic.read_text(encoding='utf-8-sig'))
    if (value.get('captured_owned_pixels_accepted') is not True or value.get('native_capture_failed') is not False
            or value.get('native_capture_error') != 0 or value.get('occlusion_rejections') != 0
            or value.get('first_frame_superseded') is not False):
        raise Blocked('owned_capture_or_first_frame_guard_rejected')
    first = value.get(contract['reference_generation_field'])
    if (not menus.number(first) or first < 1 or first != int(first)
            or any(value.get(field) != first for field in contract['required_generation_fields'])):
        raise Blocked('first_generation_guard_rejected')
    record_path = directory / (case + '.json')
    # A real visual/timing assertion may fail after valid capture. Preserve its
    # original exit/result and let the unchanged automatic summary reject it;
    # hosted diagnostics still collect later samples. Invalid ownership/identity
    # or first-generation guards above always stop immediately.
    if not record_path.is_file():
        return None
    record = json.loads(record_path.read_text(encoding='utf-8-sig'))
    if (record.get('menu') != case or record.get('revision') != source[:12]
            or any(record.get(key) != expected for key, expected in contract['windows'].items())
            or any(record.get(field) != value.get(field) for field in
                   [contract['reference_generation_field'], *contract['required_generation_fields']])):
        raise Blocked('native_record_identity_or_generation_mismatch')
    return record


def automatic_stage(name, cases, repetitions, output, tests, source, binary, contract, receipts):
    """Reuse the unmodified automatic main with a module-local child adapter."""
    original_subprocess, original_argv = menus.subprocess, sys.argv

    def run(command, **options):
        directory = Path(options['env']['XDZIP_MENU_FRAME_OUTPUT'])
        env = dict(options['env'], XDZIP_SETTINGS_PATH=str(directory / 'settings.ini'))
        try:
            result, receipt = run_owned(command, env, directory, tests, source, binary, options['timeout'])
        except Blocked:
            receipt_path = directory / 'process-receipt.json'
            if receipt_path.is_file():
                receipts.append({**json.loads(receipt_path.read_text(encoding='utf-8')), 'stage': name})
            raise
        receipt['stage'] = name
        receipts.append(receipt)
        case = command[1].split(':', 1)[1]
        guard_sample(directory, case, result, receipt, source, contract)
        return result

    # Replace only this imported module's dependency, never subprocess.run globally.
    menus.subprocess = SimpleNamespace(run=run, TimeoutExpired=subprocess.TimeoutExpired)
    sys.argv = [str(ROOT / 'scripts/check-native-menus.py'), '--tests', str(tests), '--output', str(output),
                '--backend', 'native', '--source', source, '--cases', *cases, '--repetitions', str(repetitions)]
    old_environment = dict(os.environ)
    prepared_environment = native_environment(output)
    try:
        os.environ.clear()
        os.environ.update(prepared_environment)
        try:
            menus.main()
        except SystemExit as error:
            if error.code not in (0, 1) or not (output / 'review.json').is_file():
                raise Failed('automatic_menu_summary_unavailable') from None
    finally:
        os.environ.clear()
        os.environ.update(old_environment)
        menus.subprocess, sys.argv = original_subprocess, original_argv
    return json.loads((output / 'review.json').read_text(encoding='utf-8'))


def execute(tests, output, source, binary):
    report = dict(schema_version=1, run_kind='hosted_ci_diagnostic', real_user_click=False,
                  started_by_real_foreground_mouse_click=False, fixed_reference_machine=False,
                  reference_performance_approved=False, physical_dual_monitor_tested=False,
                  manual_native_acceptance=False, full_source_sha=source, binary_sha256=binary,
                  capture_complete=False, strict_menu_contract_passed=False,
                  virtual_display_mode_changed=None, os_display_settings_changed=None,
                  started_at_utc=datetime.now(timezone.utc).isoformat(), process_receipts=[],
                  limits=['Shared hosted runner diagnostics are not fixed-machine performance, real-click or physical multi-monitor approval.'])
    try:
        if (sys.platform != 'win32' or os.environ.get('GITHUB_ACTIONS') != 'true'
                or os.environ.get('RUNNER_ENVIRONMENT') != 'github-hosted' or os.environ.get('RUNNER_OS') != 'Windows'):
            raise Blocked('windows_github_hosted_runner_required')
        report['source_before'] = source_snapshot(tests, source, binary)
        display = display_state(source)
        report.update(display)
        report['desktop'] = desktop_diagnostics()
        contract = menus.load_contract()
        report['contract_sha256'] = digest(menus.CONTRACT_PATH)
        report['pilot'] = automatic_stage('pilot', ['toolbar-more'], 1, output / 'pilot', tests, source, binary, contract, report['process_receipts'])
        report['matrix'] = automatic_stage('matrix', contract['cases'], contract['repetitions'], output / 'matrix', tests, source, binary, contract, report['process_receipts'])
        report['source_after'] = source_snapshot(tests, source, binary)
        if display_state(source) != display:
            raise Blocked('display_mode_or_receipt_changed_during_run')
        if report['source_before'] != report['source_after']:
            raise Blocked('source_or_binary_changed_during_run')
        report['capture_complete'] = len([item for item in report['process_receipts'] if item['stage'] == 'matrix']) == len(contract['cases']) * contract['repetitions']
        report['strict_menu_contract_passed'] = report['pilot']['status'] == 'passed' and report['matrix']['status'] == 'passed'
        report['status'] = 'diagnostic_passed' if report['strict_menu_contract_passed'] else 'diagnostic_failed'
        code = 0 if report['strict_menu_contract_passed'] else 1
    except Blocked as error:
        report.update(status='blocked', reason=str(error))
        code = 2
    except (Failed, OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        report.update(status='failed', reason=str(error) if isinstance(error, Failed) else type(error).__name__)
        code = 1
    report['finished_at_utc'] = datetime.now(timezone.utc).isoformat()
    report['files'] = [dict(path=file.relative_to(output).as_posix(), bytes=file.stat().st_size, sha256=digest(file))
                       for file in sorted(output.rglob('*')) if file.is_file() and 'private-temp' not in file.relative_to(output).parts]
    (output / 'review.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    return code, report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--tests', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--source', required=True)
    parser.add_argument('--binary-sha256', required=True)
    args = parser.parse_args()
    tests, output = args.tests.resolve(), args.output.resolve()
    if (not tests.is_relative_to(ROOT / 'build') or not tests.is_file() or not output.is_relative_to(ROOT / 'build')
            or (output.exists() and (not output.is_dir() or any(item.name != 'private-temp' or not item.is_dir() or item.is_symlink() for item in output.iterdir())))
            or not re.fullmatch('[0-9a-f]{40}', args.source)
            or not re.fullmatch('[0-9a-f]{64}', args.binary_sha256)):
        parser.error('Use an owned test executable, fresh build output and exact source/binary hashes')
    output.mkdir(parents=True, exist_ok=True)
    code, report = execute(tests, output, args.source, args.binary_sha256)
    print(json.dumps({key: report.get(key) for key in ('status', 'reason', 'run_kind', 'real_user_click')}))
    raise SystemExit(code)


if __name__ == '__main__':
    main()
