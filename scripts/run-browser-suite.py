#!/usr/bin/env python3
"""Run every Qt browser test within one CTest deadline.

Offscreen runs use isolated processes; native or explicitly shared settings runs
remain serial. Test functions are discovered from Qt Test on every invocation.
"""
from __future__ import annotations

import argparse
import collections
import datetime
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time
import uuid

# Relative scheduling hints only. Unknown/new functions are always included.
WEIGHTS = {
    'treeMoveRejectsEverySelectedFolderAndDescendant': 59,
    'closingWindowClearsTabsBeforeTrayRestore': 68,
    'readonlyFormatsUseSharedBrowserAndTasks': 61,
    'sessionTabReorderingPreservesControllers': 42,
    'sessionLimitKeepsBackgroundTaskEntryReachable': 21,
    'taskMenuScrollsAboveItsFixedClearButton': 22,
    'internalMoveDrag': 30,
    'sevenZipCreationLayout': 11,
    'rightClickOpenAndDropAddition': 23,
    'creationPreservesSelectedDestination': 14,
    'nativeDragSuppressesSourceNameTooltips': 11,
    'aboutUpdateStateLayoutAndReadOnlyActions': 15,
    'readonlyFormatLabelsAndIntegrityNoteLayout': 11,
    'settingsOpenModeSelectionPersists': 8,
    'operationHeadersUseSharedGeometry': 11,
    'taskMenuLabelsMatchOpenedTaskKinds': 16,
    'directoryContextExpansionReasons': 17,
    'taskMenuProgressPreservesHoverAndFocus': 7,
    'm2LargeArchiveLatestQueryAndSourceChange': 16,
    'settingsWindowPersistsAndRollsBack': 5,
    'globalQuitReturnsToPasswordTaskThenCancelsAndJoinsEverySession': 5,
}
FUNCTION = re.compile(r'^[A-Za-z_]\w*\(\)$')
ENTERED = re.compile(r'^INFO\s+: BrowserTest::(\w+)\(\) entering\s*$', re.MULTILINE)


class WindowsJob:
    """Contain test descendants in a Windows job with kill-on-close enabled.

    Supported Windows versions allow nested jobs. If setup or assignment fails,
    fail the suite rather than leave a timed-out worker running unattended.
    """

    def __init__(self):
        self.handle = None
        if os.name != 'nt':
            return
        import ctypes
        from ctypes import wintypes

        class BasicLimits(ctypes.Structure):
            _fields_ = [('process_time', ctypes.c_longlong), ('job_time', ctypes.c_longlong),
                        ('flags', wintypes.DWORD), ('minimum_working_set', ctypes.c_size_t),
                        ('maximum_working_set', ctypes.c_size_t), ('active_process_limit', wintypes.DWORD),
                        ('affinity', ctypes.c_size_t), ('priority_class', wintypes.DWORD),
                        ('scheduling_class', wintypes.DWORD)]

        class IoCounters(ctypes.Structure):
            _fields_ = [(name, ctypes.c_ulonglong) for name in (
                'reads', 'writes', 'other', 'read_bytes', 'write_bytes', 'other_bytes')]

        class ExtendedLimits(ctypes.Structure):
            _fields_ = [('basic', BasicLimits), ('io', IoCounters),
                        ('process_memory', ctypes.c_size_t), ('job_memory', ctypes.c_size_t),
                        ('peak_process_memory', ctypes.c_size_t), ('peak_job_memory', ctypes.c_size_t)]

        self.kernel = ctypes.WinDLL('kernel32', use_last_error=True)
        self.kernel.CreateJobObjectW.argtypes = [ctypes.c_void_p, wintypes.LPCWSTR]
        self.kernel.CreateJobObjectW.restype = wintypes.HANDLE
        self.kernel.SetInformationJobObject.argtypes = [wintypes.HANDLE, ctypes.c_int,
                                                        ctypes.c_void_p, wintypes.DWORD]
        self.kernel.SetInformationJobObject.restype = wintypes.BOOL
        self.kernel.AssignProcessToJobObject.argtypes = [wintypes.HANDLE, wintypes.HANDLE]
        self.kernel.AssignProcessToJobObject.restype = wintypes.BOOL
        self.kernel.CloseHandle.argtypes = [wintypes.HANDLE]
        self.kernel.CloseHandle.restype = wintypes.BOOL
        self.handle = self.kernel.CreateJobObjectW(None, None)
        if not self.handle:
            raise ctypes.WinError(ctypes.get_last_error())
        limits = ExtendedLimits()
        limits.basic.flags = 0x2000  # JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
        if not self.kernel.SetInformationJobObject(self.handle, 9, ctypes.byref(limits),
                                                   ctypes.sizeof(limits)):
            error = ctypes.get_last_error()
            self.close()
            raise ctypes.WinError(error)

    def assign(self, process):
        if self.handle is None:
            return
        import ctypes
        if not self.kernel.AssignProcessToJobObject(self.handle, int(process._handle)):
            raise ctypes.WinError(ctypes.get_last_error())

    def close(self):
        if self.handle is not None:
            self.kernel.CloseHandle(self.handle)
            self.handle = None


class Processes:
    def __init__(self):
        self.job = WindowsJob()
        self.items = []

    def start(self, *args, **kwargs):
        process = subprocess.Popen(*args, **kwargs)
        self.items.append(process)
        self.job.assign(process)
        return process

    def close(self):
        # Closing the job also terminates surviving descendants on Windows.
        self.job.close()
        for process in self.items:
            if process.poll() is None:
                try:
                    process.terminate()
                except ProcessLookupError:
                    pass
        for process in self.items:
            try:
                process.wait(timeout=3)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()


def discover(command, env, cwd):
    discovery_env = env.copy()
    discovery_env.pop('XDZIP_BROWSER_TIMING_DIR', None)
    processes = Processes()
    try:
        process = processes.start([*command, '-functions'], env=discovery_env, cwd=cwd,
                                  stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        stdout, stderr = process.communicate()
        if process.returncode:
            raise RuntimeError('Qt Test discovery failed: ' + stderr.decode('utf-8', 'replace').strip())
    finally:
        processes.close()
    lines = [line.strip() for line in stdout.decode('utf-8', 'replace').splitlines() if line.strip()]
    if not lines or any(not FUNCTION.fullmatch(line) for line in lines):
        raise RuntimeError('Qt Test returned an invalid or empty function list')
    names = [line[:-2] for line in lines]
    if len(names) != len(set(names)):
        raise RuntimeError('Qt Test returned duplicate functions')
    return names


def worker_count():
    available = os.cpu_count() or 2
    if hasattr(os, 'sched_getaffinity'):
        try:
            available = len(os.sched_getaffinity(0))
        except OSError:
            pass
    # Leave a core for Qt and codec helper processes on small CI runners.
    return min(4, max(2, available - 1))


def partition(names):
    groups = [[] for _ in range(min(worker_count(), len(names)))]
    weights = [0] * len(groups)
    for name in sorted(names, key=lambda item: -WEIGHTS.get(item, 2)):
        index = min(range(len(groups)), key=weights.__getitem__)
        groups[index].append(name)
        weights[index] += WEIGHTS.get(name, 2)
    order = {name: index for index, name in enumerate(names)}
    for group in groups:
        group.sort(key=order.__getitem__)
    return groups


def write_manifest(path, data):
    if path is None:
        return
    temporary = path.with_suffix('.tmp')
    temporary.write_text(json.dumps(data, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    temporary.replace(path)


def run_suite(command, output, env=None, cwd=None):
    env = os.environ.copy() if env is None else env.copy()
    output = Path(output).resolve()
    cwd = str(cwd or output.parent)
    output.parent.mkdir(parents=True, exist_ok=True)
    # Native windows and caller-owned settings cannot safely be shared by shards.
    offscreen = env.get('QT_QPA_PLATFORM', '').split(':', 1)[0].lower() == 'offscreen'
    auditing = any(name.startswith('XDZIP_') and '_AUDIT' in name and value
                   for name, value in env.items())
    parallel = offscreen and not env.get('XDZIP_SETTINGS_PATH') and not auditing
    processes = Processes()
    consoles = []
    try:
        if not parallel:
            process = processes.start([*command, '-v1', '-o', str(output) + ',txt'], env=env, cwd=cwd)
            return process.wait()
        names = discover(command, env, cwd)
        groups = partition(names)
        suite_id = str(uuid.uuid4())
        timing_dir = Path(env['XDZIP_BROWSER_TIMING_DIR']).resolve() if env.get('XDZIP_BROWSER_TIMING_DIR') else None
        manifest_path = timing_dir / ('browser-suite-' + suite_id + '.json') if timing_dir else None
        if timing_dir:
            timing_dir.mkdir(parents=True, exist_ok=True)
        manifest = {'schema_version': 1, 'suite_id': suite_id, 'status': 'running',
                    'started_at': datetime.datetime.now(datetime.timezone.utc).isoformat(),
                    'functions': len(names), 'shards': []}
        output.write_text('Qt browser suite running in isolated offscreen processes.\n', encoding='utf-8')
        logs = []
        start = time.monotonic()
        for index, group in enumerate(groups):
            log = output.with_name(output.stem + f'-shard-{index}.log')
            # A failed or silent launch must never reuse a previous run's results.
            log.write_text('', encoding='utf-8')
            console = output.with_name(output.stem + f'-shard-{index}.console.log').open('wb')
            consoles.append(console)
            logs.append(log)
            child_env = env.copy()
            if timing_dir:
                shard_timing = timing_dir / f'browser-suite-{suite_id}-shard-{index}'
                shard_timing.mkdir(parents=True, exist_ok=True)
                child_env['XDZIP_BROWSER_TIMING_DIR'] = str(shard_timing)
            manifest['shards'].append({'index': index, 'functions': group,
                                       'timing_directory': shard_timing.name if timing_dir else None,
                                       'status': 'running'})
            write_manifest(manifest_path, manifest)
            processes.start([*command, *group, '-v1', '-o', str(log) + ',txt'], env=child_env,
                            cwd=cwd, stdout=console, stderr=subprocess.STDOUT)
        failed = False
        for index, (process, group, log) in enumerate(zip(processes.items, groups, logs)):
            result = process.wait()
            consoles[index].close()
            content = log.read_text(encoding='utf-8', errors='replace') if log.exists() else ''
            counts = collections.Counter(ENTERED.findall(content))
            missing = [name for name in group if counts[name] != 1]
            extra = sorted(set(counts) - set(group) - {'initTestCase', 'cleanupTestCase'})
            valid = result == 0 and not missing and not extra
            failed |= not valid
            manifest['shards'][index].update(status='passed' if valid else 'failed',
                                              exit_code=result, missing_or_duplicate=missing, extra=extra)
            write_manifest(manifest_path, manifest)
            if not valid:
                print(f'Browser shard {index} failed: exit={result}, '
                      f'missing_or_duplicate={missing}, extra={extra}', file=sys.stderr)
        with output.open('w', encoding='utf-8', newline='\n') as combined:
            for index, log in enumerate(logs):
                combined.write(f'\n--- Browser shard {index} ---\n')
                if log.exists():
                    combined.write(log.read_text(encoding='utf-8', errors='replace'))
        manifest.update(status='failed' if failed else 'passed',
                        elapsed_ms=round((time.monotonic() - start) * 1000))
        write_manifest(manifest_path, manifest)
        print(f'Browser suite: {len(names)} functions, {len(groups)} shards, '
              f'{manifest["elapsed_ms"] / 1000:.2f}s, {manifest["status"]}')
        return 1 if failed else 0
    finally:
        processes.close()
        for console in consoles:
            if not console.closed:
                console.close()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--executable', required=True)
    parser.add_argument('--output', required=True)
    args = parser.parse_args()
    try:
        return run_suite([str(Path(args.executable).resolve())], args.output)
    except (OSError, RuntimeError, ValueError) as error:
        print(f'Browser suite runner: {error}', file=sys.stderr)
        return 1


if __name__ == '__main__':
    sys.exit(main())
