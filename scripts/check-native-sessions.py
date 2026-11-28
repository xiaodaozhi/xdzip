# scripts/check-native-sessions.py

"""Native window/session regression on Windows, Cocoa and X11 with a real WM."""
import argparse
import json
import os
from pathlib import Path
import platform
import subprocess
import sys
import time
from native_window_manager import OpenboxSession

ROOT = Path(__file__).resolve().parents[1]
CASES = ['windowHoverEndsBeforeRestore', 'windowHoverSensorsEndBeforeRestore',
          'nativeTrayMenuSurvivesReenable',
         'trayCaptionHoverEndsBeforeRestore',
         'desktopActivationPreservesWindowState', 'coldOpenSessionsWaitForNativeLaunch',
         'modalOpenDefersNewWindowUntilUnblocked',
         'globalSettingsKeepDraftAndAboutCallerAcrossWindows',
         'settingsOpenModeSelectionPersists',
         'sessionTabReorderingPreservesControllers',
         'closingWindowClearsTabsBeforeTrayRestore',
         'sessionLimitKeepsBackgroundTaskEntryReachable', 'independentSessionsAndOpenModes',
         'singleInstanceStructuredForwarding', 'singleInstanceConcurrentStartupAndCrashRecovery',
         'backgroundTasksKeepSessionIdentityAndSerialAdmission',
         'closedSessionRetainsPasswordPromptUntilTaskIsOpened',
         'globalQuitReturnsToPasswordTaskThenCancelsAndJoinsEverySession',
         'directoryContextExpansionReasons',
         'globalWaitForExitDoesNotCloseACompletedPageEarly']
PLATFORM_CASES = {'linux': ['linuxMainFrameOutline'], 'win32': [], 'darwin': []}
COCOA_CONFLICT_CASES = ['moveConflictsUseFileActionsAndAccessibleApplyAll', 'nativeQuitMenuWithoutTask',
                        'nativeTransitionObserverFollowsSurfaceRecreation']
COCOA_RESTORE_CASES = ['cocoaMinimizeDuringFullscreenRestore']
COCOA_ENTRY_CASES = ['cocoaMinimizeAtFullscreenEntry']


def sanitized(text):
    return text.replace(str(ROOT), '<workspace>').replace(str(Path.home()), '<home>')


def run_cases(executable, cases, log, env, timeout):
    log.unlink(missing_ok=True)
    start = time.monotonic()
    command = [str(executable.resolve()), *cases, '-o', str(log)+',txt']
    timed_out = False
    try:
        result = subprocess.run(command, env=env, timeout=timeout, check=False, capture_output=True, text=True)
    except subprocess.TimeoutExpired as error:
        timed_out = True

        def decoded(value):
            return value.decode('utf-8', errors='replace') if isinstance(value, bytes) else value or ''

        result = subprocess.CompletedProcess(command, 124, decoded(error.stdout), decoded(error.stderr))
    console = sanitized(result.stdout+result.stderr)
    log.write_text(sanitized(log.read_text(encoding='utf-8', errors='replace')) if log.is_file() else console,
                   encoding='utf-8')
    log.with_suffix('.console.log').write_text(console, encoding='utf-8')
    return {'cases': cases, 'log': log.name, 'exit_code': result.returncode, 'timed_out': timed_out,
            'elapsed_ms': round((time.monotonic()-start)*1000)}


def browser_runs(executable, cases, log, env):
    if sys.platform != 'darwin' or platform.machine() != 'x86_64':
        return [run_cases(executable, cases, log, env, 150)]
    # Keep expensive password/quit scenarios and the bilingual directory
    # matrix on independent deadlines without dropping or reordering cases.
    sessions = cases.index('globalSettingsKeepDraftAndAboutCallerAcrossWindows')
    password_quit = cases.index('globalQuitReturnsToPasswordTaskThenCancelsAndJoinsEverySession')
    directory = cases.index('directoryContextExpansionReasons')
    runs = [run_cases(executable, group, log.with_name(f'{log.stem}-{name}.log'), env, 150)
            for name, group in (('window', cases[:sessions]),
                                ('sessions', cases[sessions:password_quit]),
                                ('password-quit', cases[password_quit:directory]),
                                ('directory', cases[directory:]))]
    for suffix in ('.log', '.console.log'):
        combined = '\n'.join(log.with_name(run['log']).with_suffix(suffix).read_text(encoding='utf-8') for run in runs)
        log.with_suffix(suffix).write_text(combined, encoding='utf-8')
    return runs


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--tests', type=Path, required=True)
    parser.add_argument('--shell-tests', type=Path)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--macos-conflicts', action='store_true', help='Run the release Cocoa conflict and fullscreen cases')
    parser.add_argument('--inside-x11', action='store_true', help=argparse.SUPPRESS)
    args = parser.parse_args()
    if args.macos_conflicts and sys.platform != 'darwin':
        parser.error('The Cocoa conflict cases require macOS')
    output = args.output.resolve()
    if not output.is_relative_to(ROOT/'build') or output.suffix != '.json':
        raise ValueError('Native evidence belongs under build/')
    output.parent.mkdir(parents=True, exist_ok=True)
    log = output.with_suffix('.log')
    env = dict(os.environ)
    env['QT_QPA_PLATFORM'] = {'win32': 'windows', 'darwin': 'cocoa', 'linux': 'xcb'}[sys.platform]
    if sys.platform == 'linux' and not args.inside_x11:
        subprocess.run(['xvfb-run', '-a', sys.executable, str(Path(__file__).resolve()),
                        '--tests', str(args.tests.resolve()), '--output', str(output), '--inside-x11',
                        *(['--shell-tests', str(args.shell_tests.resolve())] if args.shell_tests else [])],
                       env=env, check=True, timeout=180)
        return
    window_manager = None
    try:
        if sys.platform == 'linux':
            window_manager = OpenboxSession(env, output.parent).start()
        cases = CASES + PLATFORM_CASES[sys.platform]
        runs = browser_runs(args.tests, cases, log, env)
        failed = any(run['exit_code'] for run in runs)
        report = dict(platform=sys.platform,
                      native_plugin=env['QT_QPA_PLATFORM'], window_manager='Openbox' if window_manager else 'system',
                      window_manager_ready_ms=window_manager.ready_ms if window_manager else None,
                      cases=cases, browser_runs=runs, timed_out=any(run['timed_out'] for run in runs),
                      limits='Native window/session automation. Not manual visual, GPU, IME or screen-reader acceptance.')
        if args.shell_tests and not failed:
            shell_log = output.with_name('shell-native.log')
            shell_env = dict(env, XDZIP_SHELL_CAPTURE_DIR=str(output.parent/'shell-native-scenes'))
            shell_cases = ['shellLayoutMatrix', 'shellSettingsAccessibleActionsUpdateTheRealModel',
                           'hostHasNoMainWindowAndDeduplicates', 'independentPasswordRetry',
                           'independentTaskLifecycle', 'trayLossDoesNotReopenClosedResults',
                           'ordinaryTaskTrayLossKeepsPromptAndHiddenMain',
                           'trayLossRestoresOnlyActiveTaskWithoutOtherEntry']
            shell_result = run_cases(args.shell_tests, shell_cases, shell_log, shell_env, 90)
            report['shell_cases'] = shell_cases
            report['shell_status'] = 'failed' if shell_result['exit_code'] else 'passed'
            report['shell_run'] = shell_result
            report['timed_out'] |= shell_result['timed_out']
            failed |= bool(shell_result['exit_code'])
        if args.macos_conflicts:
            # Each fullscreen sequence owns its Cocoa process and native surfaces.
            # The entry sequence may destroy its surface and must stay last.
            conflicts = [run_cases(args.tests, group, log.with_name(f'{log.stem}-macos-{name}.log'), env, 90)
                         for name, group in (('conflict', COCOA_CONFLICT_CASES), ('fullscreen-restore', COCOA_RESTORE_CASES),
                                             ('fullscreen-entry', COCOA_ENTRY_CASES))]
            report['cocoa_runs'] = conflicts
            report['timed_out'] |= any(run['timed_out'] for run in conflicts)
            failed |= any(run['exit_code'] for run in conflicts)
        report['status'] = 'failed' if failed else 'passed'
        output.write_text(json.dumps(report, indent=2)+'\n', encoding='utf-8')
        if failed:
            raise ValueError('Native session regression failed; inspect its logs')
    finally:
        if window_manager:
            window_manager.close()


if __name__ == '__main__':
    try: main()
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        raise SystemExit('Native session validation failed: '+type(error).__name__) from None
