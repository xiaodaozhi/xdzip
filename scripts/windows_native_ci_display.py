# scripts/windows_native_ci_display.py

"""Temporary enumerated display mode for one disposable Windows Actions job.

No DPI or registry settings are written. The wrapper must call --restore in its
finally block. Importing this module never queries or changes the local display.
"""
import argparse
import ctypes
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import uuid

ROOT = Path(__file__).resolve().parents[1]
HELPER = Path(__file__).resolve()
MINIMUM = (1280, 1024)
PUBLIC_MODE_BYTES, MAX_PRIVATE_BYTES = 220, 65535
MODE_KEYS = ('width', 'height', 'bits_per_pixel', 'frequency', 'orientation',
             'position_x', 'position_y', 'display_flags')


class DisplayBlocked(ValueError):
    pass


def require(value, code):
    if not value:
        raise DisplayBlocked(code)


class DisplayFields(ctypes.Structure):
    _fields_ = [('x', ctypes.c_int32), ('y', ctypes.c_int32),
                ('orientation', ctypes.c_uint32), ('fixed_output', ctypes.c_uint32)]


class DeviceMode(ctypes.Structure):
    # Fixed-width Win32 ABI, including two-byte WCHAR, is also mock-testable on
    # other hosts. The display branch of DEVMODEW's 16-byte union is sufficient.
    _fields_ = [('device_name', ctypes.c_uint16 * 32), ('spec_version', ctypes.c_uint16),
                ('driver_version', ctypes.c_uint16), ('size', ctypes.c_uint16),
                ('driver_extra', ctypes.c_uint16), ('fields', ctypes.c_uint32),
                ('display', DisplayFields), ('color', ctypes.c_int16), ('duplex', ctypes.c_int16),
                ('y_resolution', ctypes.c_int16), ('tt_option', ctypes.c_int16),
                ('collate', ctypes.c_int16), ('form_name', ctypes.c_uint16 * 32),
                ('log_pixels', ctypes.c_uint16), ('bits_per_pixel', ctypes.c_uint32),
                ('width', ctypes.c_uint32), ('height', ctypes.c_uint32),
                ('display_flags', ctypes.c_uint32), ('frequency', ctypes.c_uint32),
                ('icm_method', ctypes.c_uint32), ('icm_intent', ctypes.c_uint32),
                ('media_type', ctypes.c_uint32), ('dither_type', ctypes.c_uint32),
                ('reserved1', ctypes.c_uint32), ('reserved2', ctypes.c_uint32),
                ('panning_width', ctypes.c_uint32), ('panning_height', ctypes.c_uint32)]


def structural_mode(raw):
    require(ctypes.sizeof(DeviceMode) == PUBLIC_MODE_BYTES
            and PUBLIC_MODE_BYTES <= len(raw) <= PUBLIC_MODE_BYTES + MAX_PRIVATE_BYTES,
            'invalid_devmode_size')
    mode = DeviceMode.from_buffer_copy(raw[:PUBLIC_MODE_BYTES])
    require(mode.size == PUBLIC_MODE_BYTES and 0 <= mode.driver_extra <= MAX_PRIVATE_BYTES
            and len(raw) == PUBLIC_MODE_BYTES + mode.driver_extra, 'invalid_devmode_private_data_size')
    return mode


def parse_mode(raw):
    mode = structural_mode(raw)
    require(mode.width > 0 and mode.height > 0 and mode.bits_per_pixel == 32,
            'invalid_display_dimensions_or_depth')
    require(not mode.fields & 0x00020000, 'dpi_field_must_not_be_set')  # DM_LOGPIXELS
    return mode


def safe_modes(modes):
    result = []
    for raw in modes:
        mode = structural_mode(raw)  # A corrupt ABI/private-driver payload is never skipped.
        if mode.width > 0 and mode.height > 0 and mode.bits_per_pixel == 32 and not mode.fields & 0x00020000:
            result.append(raw)
    return result


def public_mode(raw):
    mode = parse_mode(raw)
    return dict(width=int(mode.width), height=int(mode.height), bits_per_pixel=int(mode.bits_per_pixel),
                frequency=int(mode.frequency), orientation=int(mode.display.orientation),
                position_x=int(mode.display.x), position_y=int(mode.display.y), display_flags=int(mode.display_flags))


class NativeDisplay:
    # FFI is used only after the hosted-Windows guard. Enumeration, the current
    # process session and input desktop are read-only. Apply uses one exact
    # enumerated DEVMODE with flags=0, after CDS_TEST; failures stop without a
    # registry/DPI write. Restoration accepts only this job's saved state.
    def __init__(self):
        require(sys.platform == 'win32', 'windows_only')
        self.user = ctypes.WinDLL('user32', use_last_error=True)
        self.kernel = ctypes.WinDLL('kernel32', use_last_error=True)
        self.query_headers = []
        self.user.EnumDisplaySettingsW.argtypes = [ctypes.c_wchar_p, ctypes.c_uint32, ctypes.POINTER(DeviceMode)]
        self.user.EnumDisplaySettingsW.restype = ctypes.c_int32
        self.user.ChangeDisplaySettingsExW.argtypes = [ctypes.c_wchar_p, ctypes.POINTER(DeviceMode),
                                                       ctypes.c_void_p, ctypes.c_uint32, ctypes.c_void_p]
        self.user.ChangeDisplaySettingsExW.restype = ctypes.c_int32

    def conditions(self):
        self.kernel.ProcessIdToSessionId.argtypes = [ctypes.c_uint32, ctypes.POINTER(ctypes.c_uint32)]
        self.kernel.ProcessIdToSessionId.restype = ctypes.c_int32
        session = ctypes.c_uint32()
        require(self.kernel.ProcessIdToSessionId(os.getpid(), ctypes.byref(session)), 'session_query_failed')
        self.user.OpenInputDesktop.argtypes = [ctypes.c_uint32, ctypes.c_int32, ctypes.c_uint32]
        self.user.OpenInputDesktop.restype = ctypes.c_void_p
        self.user.CloseDesktop.argtypes = [ctypes.c_void_p]
        self.user.CloseDesktop.restype = ctypes.c_int32
        self.user.GetSystemMetrics.argtypes = [ctypes.c_int]
        self.user.GetSystemMetrics.restype = ctypes.c_int
        desktop = self.user.OpenInputDesktop(0, False, 1)  # DESKTOP_READOBJECTS, never switch desktops.
        if desktop:
            require(self.user.CloseDesktop(desktop), 'desktop_handle_close_failed')
        return dict(session_id=int(session.value), input_desktop_accessible=bool(desktop),
                    monitor_count=int(self.user.GetSystemMetrics(80)))

    def _enum(self, index):
        # EnumDisplaySettingsW takes the available private-data capacity in
        # dmDriverExtra and returns a complete DEVMODE plus driver data. Keep
        # that opaque tail intact; it is required by the matching driver on CDS.
        capacity = PUBLIC_MODE_BYTES + MAX_PRIVATE_BYTES
        buffer = ctypes.create_string_buffer(capacity)
        mode = DeviceMode.from_buffer(buffer)
        mode.size, mode.driver_extra = PUBLIC_MODE_BYTES, MAX_PRIVATE_BYTES
        success = bool(self.user.EnumDisplaySettingsW(None, index, ctypes.cast(buffer, ctypes.POINTER(DeviceMode))))
        self.query_headers.append(dict(enum_index=index, capacity_bytes=capacity,
            input_dm_size=PUBLIC_MODE_BYTES, input_dm_driver_extra=MAX_PRIVATE_BYTES, success=success,
            returned_dm_size=int(mode.size) if success else None,
            returned_dm_driver_extra=int(mode.driver_extra) if success else None))
        if not success:
            return None
        require(mode.size == PUBLIC_MODE_BYTES and mode.driver_extra <= MAX_PRIVATE_BYTES,
                'invalid_reported_devmode_header')
        actual_bytes = PUBLIC_MODE_BYTES + mode.driver_extra
        require(actual_bytes <= capacity, 'reported_private_mode_exceeds_buffer')
        raw = buffer.raw[:actual_bytes]
        structural_mode(raw)
        return raw

    def current_raw(self):
        result = self._enum(0xffffffff)  # ENUM_CURRENT_SETTINGS, never registry defaults.
        require(result is not None, 'current_mode_query_failed')
        parse_mode(result)
        return result

    def enumerated_raw(self):
        modes = []
        for index in range(4096):
            raw = self._enum(index)
            if raw is None:
                return safe_modes(modes)
            modes.append(raw)
        raise DisplayBlocked('display_mode_enumeration_limit')

    def test(self, raw):
        return self._change(raw, 2)  # CDS_TEST

    def apply(self, raw):
        return self._change(raw, 0)

    def _change(self, raw, flags):
        parse_mode(raw)
        # This stable local buffer includes every private byte. Neither the
        # pointer nor its storage outlives the synchronous Win32 API call.
        buffer = ctypes.create_string_buffer(raw, len(raw))
        return int(self.user.ChangeDisplaySettingsExW(None, ctypes.cast(buffer, ctypes.POINTER(DeviceMode)),
                                                      None, flags, None))


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def _hosted(env):
    require(sys.platform == 'win32' and env.get('GITHUB_ACTIONS') == 'true'
            and env.get('RUNNER_ENVIRONMENT') == 'github-hosted' and env.get('RUNNER_OS') == 'Windows',
            'windows_hosted_actions_only')


def _api(api, env):
    if api is None:
        _hosted(env)  # Fail before loading the Win32 adapter on an unapproved host.
        return NativeDisplay()
    return api  # Dependency injection for pure mocks, not a CLI option.


def _binding(source, env, api):
    _hosted(env)
    require(re.fullmatch('[0-9a-f]{40}', source), 'invalid_source')
    github = dict(run_id=env.get('GITHUB_RUN_ID'), run_attempt=env.get('GITHUB_RUN_ATTEMPT'), job=env.get('GITHUB_JOB'))
    require(re.fullmatch('[1-9][0-9]*', github['run_id'] or '')
            and re.fullmatch('[1-9][0-9]*', github['run_attempt'] or '')
            and re.fullmatch('[A-Za-z0-9_-]+', github['job'] or ''), 'invalid_actions_identity')
    run_git = lambda *args: subprocess.check_output(['git', *args], cwd=ROOT, timeout=10, stderr=subprocess.PIPE).decode().strip()
    require(run_git('rev-parse', 'HEAD') == source and not run_git('status', '--porcelain'), 'clean_source_required')
    name = HELPER.relative_to(ROOT).as_posix()
    require(run_git('rev-parse', source + ':' + name) == run_git('hash-object', '--path', name, name),
            'helper_not_bound_to_source')
    conditions = api.conditions()
    require(conditions['session_id'] > 0 and conditions['input_desktop_accessible'] is True
            and conditions['monitor_count'] == 1, 'interactive_single_display_session_required')
    return dict(full_source_sha=source, github=github, helper_sha256=digest(HELPER.read_bytes()),
                session_id=conditions['session_id'])


def _folder(path):
    folder = Path(path).resolve()
    require(folder.is_relative_to(ROOT / 'build') and folder != ROOT / 'build', 'owned_build_directory_required')
    return folder


def select_mode(modes, before):
    current = public_mode(before)
    eligible = [raw for raw in safe_modes(modes) if public_mode(raw)['width'] >= MINIMUM[0]
                and public_mode(raw)['height'] >= MINIMUM[1]
                and public_mode(raw)['orientation'] == current['orientation']
                and not public_mode(raw)['display_flags'] & 2]  # No interlaced mode.
    require(eligible, 'no_supported_mode_at_least_1280x1024')
    rank = lambda raw: 0 if (public_mode(raw)['width'], public_mode(raw)['height']) == (3840, 2160) else (
        1 if (public_mode(raw)['width'], public_mode(raw)['height']) == (1920, 1080) else 2)
    return min(eligible, key=lambda raw: (rank(raw),
                public_mode(raw)['width'] * public_mode(raw)['height'],
                abs(public_mode(raw)['frequency'] - current['frequency'])))


def _write(file, value):
    file.write_text(json.dumps(value, indent=2) + '\n', encoding='utf-8')


def prepare(output, source, env=None, api=None):
    env = os.environ if env is None else env
    api = _api(api, env)
    binding = _binding(source, env, api)
    folder = _folder(output)
    require(not folder.exists(), 'fresh_display_output_required')
    folder.mkdir(parents=True)
    record = dict(schema_version=1, status='blocked', **binding, creator_pid=os.getpid(),
                  owner_token=uuid.uuid4().hex, created_at_utc=datetime.now(timezone.utc).isoformat(),
                  applied=False, apply_attempted=False, restored=False, registry_written=False, dpi_changed=False,
                  test_flags=2, apply_flags=0, minimum_size=list(MINIMUM), physical_dual_monitor_accepted=False)
    try:
        before = api.current_raw()
        (folder / 'before-devmode.bin').write_bytes(before)
        record.update(before=public_mode(before), before_devmode_sha256=digest(before))
        selected = select_mode(api.enumerated_raw(), before)
        (folder / 'requested-devmode.bin').write_bytes(selected)
        record.update(requested=public_mode(selected), requested_devmode_sha256=digest(selected),
                      cds_test_return_code=api.test(selected))
        require(record['cds_test_return_code'] == 0, 'enumerated_mode_cds_test_failed')
        record['apply_attempted'] = True
        # Persist the saved original state before the sole dynamic apply call.
        _write(folder / 'receipt.json', record)
        record['apply_return_code'] = api.apply(selected)
        active = api.current_raw()
        (folder / 'active-devmode.bin').write_bytes(active)
        record.update(active=public_mode(active), active_devmode_sha256=digest(active),
                      mode_changed=public_mode(active) != public_mode(before))
        require(record['apply_return_code'] == 0 and public_mode(active) == public_mode(selected),
                'applied_mode_readback_mismatch')
        record.update(status='active', applied=True)
    except (DisplayBlocked, OSError) as error:
        record['reason'] = str(error) if isinstance(error, DisplayBlocked) else type(error).__name__
    record['query_headers'] = getattr(api, 'query_headers', [])
    _write(folder / 'receipt.json', record)
    return record


def _saved(path, source, env, api):
    file = Path(path)
    require(file.name == 'receipt.json' and not file.is_symlink(), 'fixed_receipt_name_required')
    folder = _folder(file.parent)
    file = folder / 'receipt.json'
    record = json.loads(file.read_text(encoding='utf-8'))
    binding = _binding(source, env, api)
    require(record.get('schema_version') == 1 and all(record.get(key) == value for key, value in binding.items()),
            'display_receipt_identity_mismatch')
    require(type(record.get('creator_pid')) is int and record['creator_pid'] > 0
            and re.fullmatch('[0-9a-f]{32}', record.get('owner_token', '')),
            'invalid_receipt_owner')
    age = (datetime.now(timezone.utc) - datetime.fromisoformat(record['created_at_utc'])).total_seconds()
    require(0 <= age <= 10800 and record.get('registry_written') is False and record.get('dpi_changed') is False
            and record.get('test_flags') == 2 and record.get('apply_flags') == 0, 'invalid_or_expired_display_receipt')
    before = (folder / 'before-devmode.bin').read_bytes()
    require(digest(before) == record.get('before_devmode_sha256') and public_mode(before) == record.get('before'),
            'saved_original_mode_mismatch')
    for key in ('requested', 'active'):
        if key in record:
            raw = (folder / (key + '-devmode.bin')).read_bytes()
            require(digest(raw) == record.get(key + '_devmode_sha256') and public_mode(raw) == record[key],
                    'saved_' + key + '_mode_mismatch')
    return file, record, before


def read_current_mode():
    return public_mode(_api(None, os.environ).current_raw())


def validate_active_receipt(path, source, env=None, api=None):
    env = os.environ if env is None else env
    api = _api(api, env)
    file, record, _ = _saved(path, source, env, api)
    require(not (file.parent / 'restore.json').exists() and record.get('status') == 'active'
            and record.get('applied') is True and record.get('restored') is False
            and record.get('apply_return_code') == record.get('cds_test_return_code') == 0,
            'display_receipt_not_active')
    active = (file.parent / 'active-devmode.bin').read_bytes()
    require(digest(active) == record.get('active_devmode_sha256') and public_mode(active) == record.get('active')
            and public_mode(api.current_raw()) == record['active']
            and record['active']['width'] >= MINIMUM[0] and record['active']['height'] >= MINIMUM[1],
            'active_display_mode_changed_or_insufficient')
    return dict(record, receipt_path=file.relative_to(ROOT).as_posix(), receipt_sha256=digest(file.read_bytes()))


def restore(output, source, env=None, api=None):
    env = os.environ if env is None else env
    api = _api(api, env)
    folder = _folder(output)
    file, record, before = _saved(folder / 'receipt.json', source, env, api)
    require(not (folder / 'restore.json').exists(), 'restore_already_recorded')
    current = public_mode(api.current_raw())
    known = [record['active']] if 'active' in record else [record['before']]
    if 'active' not in record and record.get('apply_attempted') is True and 'requested' in record:
        known.append(record['requested'])
    require(current in known, 'display_state_changed_outside_owned_run')
    result = dict(schema_version=1, **_binding(source, env, api), owner_token=record['owner_token'],
                  original_receipt_sha256=digest(file.read_bytes()), restored=False,
                  registry_written=False, dpi_changed=False, restore_flags=0)
    if record.get('apply_attempted') is True:
        require(any(public_mode(raw) == public_mode(before) for raw in safe_modes(api.enumerated_raw())), 'original_mode_no_longer_supported')
        result['restore_test_return_code'] = api.test(before)
        require(result['restore_test_return_code'] == 0, 'original_mode_restore_test_failed')
        result['restore_return_code'] = api.apply(before)
        require(result['restore_return_code'] == 0, 'original_mode_restore_failed')
    require(public_mode(api.current_raw()) == public_mode(before), 'original_mode_restore_readback_mismatch')
    result.update(status='restored', restored=True, restored_mode=public_mode(before),
                  query_headers=getattr(api, 'query_headers', []),
                  completed_at_utc=datetime.now(timezone.utc).isoformat())
    # The original receipt/binaries remain immutable for before/after audit.
    _write(folder / 'restore.json', result)
    return result


def self_test():
    """Pure mocked mode/receipt regressions; no Win32 adapter is constructed."""
    checks = 0

    def check(value):
        nonlocal checks
        assert value
        checks += 1

    def rejected(action):
        try:
            action()
        except DisplayBlocked:
            check(True)
            return
        raise AssertionError('unsafe_display_state_was_accepted')

    def mode(width, height, bits=32, fields=0x5c0000):
        value = DeviceMode()
        value.size = 220
        value.fields = fields
        value.bits_per_pixel = bits
        value.width, value.height, value.frequency = width, height, 60
        return bytes(value)

    before, fallback, preferred = mode(1024, 768), mode(1280, 1024), mode(1920, 1080)
    tail = b'\x00\xf1owned-private-mode\x00\x7f'

    def with_private(raw):
        header = DeviceMode.from_buffer_copy(raw)
        header.driver_extra = len(tail)
        return bytes(header) + tail

    private_before, private_preferred = with_private(before), with_private(preferred)
    check(public_mode(private_before) == public_mode(before))
    rejected(lambda: parse_mode(private_before[:-1]))
    rejected(lambda: parse_mode(private_before + b'\0' * (PUBLIC_MODE_BYTES + MAX_PRIVATE_BYTES)))
    maximum = DeviceMode.from_buffer_copy(before)
    maximum.driver_extra = MAX_PRIVATE_BYTES
    check(public_mode(bytes(maximum) + b'\0' * MAX_PRIVATE_BYTES) == public_mode(before))
    mixed = [mode(1920, 1080, 16), mode(1920, 1080, fields=0x5e0000), before, fallback, preferred]
    check(ctypes.sizeof(DeviceMode) == 220 and DeviceMode.width.offset == 172)
    check(select_mode(mixed, before) == preferred)
    four_k = mode(3840, 2160)
    check(select_mode([*mixed, four_k], before) == four_k)
    check(select_mode([before, fallback], before) == fallback)
    rejected(lambda: select_mode([before], before))
    rejected(lambda: safe_modes([b'bad']))
    rejected(lambda: _hosted({}))

    class Fake:
        def __init__(self, modes=None, test_code=0, fail_readback=False):
            self.raw, self.modes = before, mixed if modes is None else modes
            self.test_code, self.fail_readback = test_code, fail_readback
            self.applies, self.readback_error = 0, False

        def current_raw(self):
            if self.readback_error:
                self.readback_error = False
                raise OSError('mock_readback_failure')
            return self.raw

        def enumerated_raw(self):
            return self.modes

        def test(self, raw):
            return self.test_code

        def apply(self, raw):
            self.applies += 1
            self.raw = raw
            self.readback_error = self.fail_readback and self.applies == 1
            return 0

    original_binding = globals()['_binding']
    source = 'a' * 40

    def binding(value, env, api):
        return dict(full_source_sha=value, github=dict(run_id=env.get('GITHUB_RUN_ID', '1'),
                    run_attempt='1', job='pure_mock'), helper_sha256=digest(HELPER.read_bytes()), session_id=1)

    (ROOT / 'build').mkdir(exist_ok=True)
    try:
        globals()['_binding'] = binding
        with tempfile.TemporaryDirectory(prefix='display-helper-selftest-', dir=ROOT / 'build') as temporary:
            base = Path(temporary)
            unsupported = Fake(modes=[before])
            check(prepare(base / 'unsupported', source, {}, unsupported)['status'] == 'blocked' and unsupported.applies == 0)
            invalid = Fake(test_code=-2)
            check(prepare(base / 'test-failed', source, {}, invalid)['status'] == 'blocked' and invalid.applies == 0)
            api, folder = Fake(), base / 'valid'
            check(prepare(folder, source, {}, api)['status'] == 'active' and api.applies == 1)
            file = folder / 'receipt.json'
            stable = validate_active_receipt(file, source, {}, api)
            check(stable == validate_active_receipt(file, source, {}, api))
            rejected(lambda: validate_active_receipt(file, 'b' * 40, {}, api))
            rejected(lambda: validate_active_receipt(file, source, {'GITHUB_RUN_ID': '2'}, api))
            api.raw = fallback
            rejected(lambda: validate_active_receipt(file, source, {}, api))
            rejected(lambda: restore(folder, source, {}, api))
            api.raw = preferred
            saved = (folder / 'before-devmode.bin').read_bytes()
            (folder / 'before-devmode.bin').write_bytes(preferred)
            rejected(lambda: restore(folder, source, {}, api))
            (folder / 'before-devmode.bin').write_bytes(saved)
            original_receipt = file.read_bytes()
            restored = restore(folder, source, {}, api)
            check(restored['restored'] and api.raw == before and api.applies == 2 and file.read_bytes() == original_receipt)
            rejected(lambda: validate_active_receipt(file, source, {}, api))
            rejected(lambda: restore(folder, source, {}, api))
            failed_readback = Fake(fail_readback=True)
            failed_folder = base / 'readback-failed'
            check(prepare(failed_folder, source, {}, failed_readback)['status'] == 'blocked' and failed_readback.raw == preferred)
            check(restore(failed_folder, source, {}, failed_readback)['restored'] and failed_readback.raw == before)
            extra_api = Fake(modes=[private_before, private_preferred])
            extra_api.raw = private_before
            extra_folder = base / 'private-tail'
            check(prepare(extra_folder, source, {}, extra_api)['status'] == 'active'
                  and (extra_folder / 'before-devmode.bin').read_bytes() == private_before
                  and (extra_folder / 'requested-devmode.bin').read_bytes() == private_preferred
                  and (extra_folder / 'active-devmode.bin').read_bytes() == private_preferred)
            check(restore(extra_folder, source, {}, extra_api)['restored'] and extra_api.raw == private_before)
            rejected(lambda: _folder(ROOT / 'scripts'))
    finally:
        globals()['_binding'] = original_binding
    flags, transmitted = [], []
    adapter = object.__new__(NativeDisplay)

    class NoWin32:
        def ChangeDisplaySettingsExW(self, device, value, window, flag, parameter):
            flags.append(flag)
            transmitted.append(ctypes.string_at(value, value.contents.size + value.contents.driver_extra))
            return 0

        def EnumDisplaySettingsW(self, device, index, pointer):
            check(pointer.contents.size == PUBLIC_MODE_BYTES and pointer.contents.driver_extra == MAX_PRIVATE_BYTES)
            ctypes.memmove(pointer, private_preferred, len(private_preferred))
            return 1

    adapter.user = NoWin32()
    adapter.test(private_preferred)
    adapter.apply(private_preferred)
    check(flags == [2, 0] and transmitted == [private_preferred, private_preferred])
    adapter.query_headers = []
    check(adapter._enum(0) == private_preferred)
    check(adapter.query_headers[0]['returned_dm_size'] == PUBLIC_MODE_BYTES
          and adapter.query_headers[0]['returned_dm_driver_extra'] == len(tail)
          and adapter.query_headers[0]['capacity_bytes'] == PUBLIC_MODE_BYTES + MAX_PRIVATE_BYTES)
    return dict(status='self_test_passed', checks=checks, synthetic_data_only=True, native_display_calls=0)


def main():
    if sys.argv[1:] == ['--self-test']:
        print(json.dumps(self_test()))
        return 0
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument('--prepare', action='store_true')
    mode.add_argument('--restore', action='store_true')
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--source', required=True)
    args = parser.parse_args()
    try:
        result = (prepare if args.prepare else restore)(args.output, args.source)
        print(json.dumps(result))
        return 0 if result['status'] in ('active', 'restored') else 2
    except (DisplayBlocked, OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        print(json.dumps(dict(status='blocked', reason=str(error) if isinstance(error, DisplayBlocked)
                             else type(error).__name__, saved_receipt_review_required=True)))
        return 2


if __name__ == '__main__':
    raise SystemExit(main())
