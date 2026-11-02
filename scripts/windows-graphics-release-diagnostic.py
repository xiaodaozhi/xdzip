import argparse
import ctypes
from ctypes import wintypes
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time

root = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser()
parser.add_argument('--backend', choices=['d3d11', 'd3d12'], required=True)
args = parser.parse_args()
task = root/'build'/('issue135-graphics-'+args.backend)
if task.exists():
    raise ValueError('Refusing to replace prior evidence')
task.mkdir()
sys.path.insert(0, str(root/'scripts'))
spec = importlib.util.spec_from_file_location('verify_alpha8', root/'scripts/verify-alpha8-qt-replacement.py')
release = importlib.util.module_from_spec(spec)
spec.loader.exec_module(release)
package = root/'build/release-input/xdzip-0.1.0-alpha.8-windows-x86_64.zip'
assert release.digest(package) == release.PACKAGES['windows']['sha256']
portable = task/'portable'
portable.mkdir()
release.unpack(package, portable, 'windows')
assert (portable/'xdzip.exe').is_file()

class ModuleEntry(ctypes.Structure):
    _fields_ = [('dwSize', wintypes.DWORD), ('th32ModuleID', wintypes.DWORD),
                ('th32ProcessID', wintypes.DWORD), ('GlblcntUsage', wintypes.DWORD),
                ('ProccntUsage', wintypes.DWORD), ('modBaseAddr', ctypes.POINTER(ctypes.c_byte)),
                ('modBaseSize', wintypes.DWORD), ('hModule', wintypes.HMODULE),
                ('szModule', wintypes.WCHAR*256), ('szExePath', wintypes.WCHAR*260)]

kernel = ctypes.WinDLL('kernel32', use_last_error=True)
kernel.CreateToolhelp32Snapshot.argtypes = [wintypes.DWORD, wintypes.DWORD]
kernel.CreateToolhelp32Snapshot.restype = wintypes.HANDLE
kernel.Module32FirstW.argtypes = [wintypes.HANDLE, ctypes.POINTER(ModuleEntry)]
kernel.Module32NextW.argtypes = [wintypes.HANDLE, ctypes.POINTER(ModuleEntry)]
kernel.CloseHandle.argtypes = [wintypes.HANDLE]
targets = {'d3dcompiler_47.dll', 'dxcompiler.dll', 'dxil.dll'}
def modules(pid):
    handle = kernel.CreateToolhelp32Snapshot(0x00000008 | 0x00000010, pid)
    if handle == ctypes.c_void_p(-1).value:
        return None
    result = {}
    try:
        entry = ModuleEntry()
        entry.dwSize = ctypes.sizeof(entry)
        success = kernel.Module32FirstW(handle, ctypes.byref(entry))
        if not success:
            return None
        while success:
            name, path = entry.szModule.lower(), Path(entry.szExePath)
            if name in targets:
                origin = 'system32' if path.parent == Path(os.environ['SystemRoot'])/'System32' else \
                         'portable' if path.parent == portable else 'other'
                result[name] = origin
            success = kernel.Module32NextW(handle, ctypes.byref(entry))
    finally:
        kernel.CloseHandle(handle)
    return result

def inventory():
    return {p.relative_to(portable).as_posix(): release.digest(p)
            for p in portable.rglob('*') if p.is_file()}

initial = inventory()
assert len(initial) == 480 and targets <= set(initial)
fixture, payload = task/'owned-browse.zip', task/'owned-payload.zip'
for script, target in [('generate-m2-fixture.py', fixture), ('generate-m3-fixture.py', payload)]:
    subprocess.run([sys.executable, str(root/'scripts'/script), '--output', str(target)], check=True)

def probe(label, fixture, operation, prefer_software=False):
    case = task/label
    case.mkdir()
    for directory in ['profile', 'local', 'temp']:
        (case/directory).mkdir()
    env = os.environ.copy()
    for key in ['QT_PLUGIN_PATH', 'QML_IMPORT_PATH', 'QML2_IMPORT_PATH', 'QT_QPA_PLATFORM_PLUGIN_PATH',
                'QT_QUICK_BACKEND', 'XDZIP_EXPECT_QT_SOURCE_PROOF', 'XDZIP_PROBE_OPERATION',
                'XDZIP_PROBE_EXTRACT_TARGET', 'QSG_RHI_PREFER_SOFTWARE_RENDERER']:
        env.pop(key, None)
    system = Path(os.environ['SystemRoot'])
    env.update(QT_QPA_PLATFORM='windows', QSG_RHI_BACKEND=args.backend, QSG_INFO='1',
               QT_FORCE_STDERR_LOGGING='1', QT_LOGGING_RULES='qt.rhi.general.debug=true;qt.scenegraph.general.debug=true',
               QML_DISABLE_DISK_CACHE='1', XDZIP_PROBE_IDLE_MS='100',
               XDZIP_SETTINGS_PATH=str(case/'settings.ini'), APPDATA=str(case/'profile'),
               LOCALAPPDATA=str(case/'local'), TEMP=str(case/'temp'), TMP=str(case/'temp'),
               PATH=os.pathsep.join([str(system/'System32'), str(system)]))
    if prefer_software:
        env['QSG_RHI_PREFER_SOFTWARE_RENDERER'] = '1'
    if operation == 'extract':
        target = case/'extracted'
        target.mkdir()
        sid = subprocess.check_output(['powershell.exe', '-NoProfile', '-Command',
            '[System.Security.Principal.WindowsIdentity]::GetCurrent().User.Value'], text=True).strip()
        assert re.fullmatch(r'S-1-(?:\d+-)+\d+', sid)
        subprocess.run(['icacls.exe', str(target), '/inheritance:r', '/grant:r',
                        '*'+sid+':(OI)(CI)F'], stdout=subprocess.DEVNULL, check=True)
        env.update(XDZIP_PROBE_OPERATION='extract', XDZIP_PROBE_EXTRACT_TARGET=str(target))
    captured = {}
    successful_samples = 0
    log = case/'probe.log'
    with log.open('w', encoding='utf-8') as stream:
        process = subprocess.Popen([str(portable/'xdzip.exe'), '--desktop-probe', str(fixture)],
                                   env=env, stdout=stream, stderr=subprocess.STDOUT)
        deadline = time.monotonic()+220
        while process.poll() is None and time.monotonic() < deadline:
            current = modules(process.pid)
            if current is not None:
                successful_samples += 1
                captured.update(current)
            time.sleep(.25)
        if process.poll() is None:
            process.kill()
            process.wait()
            raise TimeoutError(label)
    text = log.read_text(encoding='utf-8', errors='replace')
    selected = re.search(r"Adapter \d+: '[^']*' \(vendor 0x[0-9A-Fa-f]+ device 0x[0-9A-Fa-f]+ flags 0x([0-9A-Fa-f]+)\)[^\n]*\n[^\n]*using this adapter", text)
    selected_flags = int(selected.group(1), 16) if selected else None
    if prefer_software:
        assert selected_flags is not None and selected_flags & 2, 'Software adapter selection was not observed'
    events = {}
    for line in text.splitlines():
        if line.startswith('XDZIP_PROBE '):
            item = json.loads(line.removeprefix('XDZIP_PROBE '))
            assert item['event'] not in events
            events[item['event']] = item
    assert process.returncode == 0, (label, process.returncode)
    frame = events['first_frame']
    assert frame['backend'] == args.backend and frame['version'] == '0.1.0-alpha.8' and frame['build'] == 21
    assert frame['revision'].startswith('e1cd6a3') and frame['dirty'] is False and frame['runtime_available']
    assert 'browse_stable' in events and 'quit_requested' in events and successful_samples > 0
    if operation == 'extract':
        assert events['extract_finished']['status'] == 'succeeded'
        verified = release.verify_output(fixture, target)
    else:
        assert events['open_ready']['records'] == 100000
        assert events['cancel_requested']['pending'] and events['cancel_settled']['cancelled']
        verified = None
    return {'returncode': process.returncode, 'first_frame': frame, 'adapter_flags': selected_flags,
            'geometry_warnings': text.count('QWindowsWindow::setGeometry'),
            'software_adapter_requested': prefer_software, 'module_origins': captured,
            'module_samples': successful_samples, 'extracted_files_verified': verified}

system = Path(os.environ['SystemRoot'])/'System32'
system_dlls = {name: release.digest(system/name) if (system/name).is_file() else None for name in targets}
report = {'archive_sha256': release.digest(package), 'initial_files': len(initial),
          'backend': args.backend, 'system_dll_sha256_or_absent': system_dlls, 'runs': {}}
for operation in ['browse', 'extract']:
    for mode, software in [('default', False), ('software', True)]:
        label = 'original-'+mode+'-'+operation
        report['runs'][label] = probe(label, fixture if operation == 'browse' else payload, operation, software)
assert inventory() == initial
for name in targets:
    path = portable/name
    assert path.parent == portable and path.is_file() and not path.is_symlink()
    path.unlink()
remaining = inventory()
assert remaining == {name: value for name, value in initial.items() if name not in targets}
for operation in ['browse', 'extract']:
    for mode, software in [('default', False), ('software', True)]:
        label = 'without-sdk-graphics-'+mode+'-'+operation
        report['runs'][label] = probe(label, fixture if operation == 'browse' else payload, operation, software)
assert inventory() == remaining
report['removed_from_copy'] = sorted(targets)
report['remaining_files'] = len(remaining)
report['limits'] = ['One Windows runner and one graphics API only; not a clean consumer system or coverage of alternate drivers.',
                    'Sampling loaded modules can miss short-lived loads; absence from samples is not proof of no use.',
                    'The released asset and installed program were not changed; no decision on SDK terms follows from this experiment.']
(task/'review.json').write_text(json.dumps(report, indent=2)+'\n', encoding='utf-8')
print(json.dumps(report, indent=2))
