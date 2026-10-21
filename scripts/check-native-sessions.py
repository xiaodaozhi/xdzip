# scripts/check-native-sessions.py

"""Native window/session regression on Windows, Cocoa and X11 with a real WM."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
CASES = ['desktopActivationPreservesWindowState', 'coldOpenSessionsWaitForNativeLaunch',
         'modalOpenDefersNewWindowUntilUnblocked',
         'globalSettingsKeepDraftAndAboutCallerAcrossWindows',
         'settingsOpenModeSelectionPersists',
         'sessionTabReorderingPreservesControllers',
         'closingWindowClearsTabsBeforeTrayRestore',
         'sessionLimitKeepsBackgroundTaskEntryReachable', 'independentSessionsAndOpenModes',
         'singleInstanceStructuredForwarding', 'singleInstanceConcurrentStartupAndCrashRecovery',
         'backgroundTasksKeepSessionIdentityAndSerialAdmission',
         'closedSessionRetainsPasswordPromptUntilTaskIsOpened',
         'globalWaitForExitDoesNotCloseACompletedPageEarly']


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--tests', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--inside-x11', action='store_true', help=argparse.SUPPRESS)
    args = parser.parse_args()
    output = args.output.resolve()
    if not output.is_relative_to(ROOT/'build') or output.suffix != '.json':
        raise ValueError('Native evidence belongs under build/')
    output.parent.mkdir(parents=True, exist_ok=True)
    log = output.with_suffix('.log')
    def sanitized(text):
        return text.replace(str(ROOT), '<workspace>').replace(str(Path.home()), '<home>')
    env = dict(os.environ)
    env['QT_QPA_PLATFORM'] = {'win32': 'windows', 'darwin': 'cocoa', 'linux': 'xcb'}[sys.platform]
    if sys.platform == 'linux' and not args.inside_x11:
        subprocess.run(['xvfb-run', '-a', sys.executable, str(Path(__file__).resolve()),
                        '--tests', str(args.tests.resolve()), '--output', str(output), '--inside-x11'],
                       env=env, check=True, timeout=180)
        return
    window_manager = None
    try:
        if sys.platform == 'linux':
            window_manager = subprocess.Popen(['openbox', '--sm-disable'], env=env,
                stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            deadline = time.monotonic() + 10
            while time.monotonic() < deadline:
                if window_manager.poll() is not None: raise ValueError('X11 window manager exited')
                ready = subprocess.run(['xprop', '-root', '_NET_SUPPORTING_WM_CHECK'], env=env,
                    capture_output=True, text=True, timeout=3)
                if ready.returncode == 0 and 'window id #' in ready.stdout: break
                time.sleep(.05)
            else: raise ValueError('X11 window manager did not become ready')
        command = [str(args.tests.resolve()), *CASES, '-o', str(log)+',txt']
        timed_out = False
        try:
            result = subprocess.run(command, env=env, timeout=150, check=False, capture_output=True, text=True)
        except subprocess.TimeoutExpired as error:
            timed_out = True
            def decoded(value):
                return value.decode('utf-8', errors='replace') if isinstance(value, bytes) else value or ''
            result = subprocess.CompletedProcess(command, 124, decoded(error.stdout), decoded(error.stderr))
        output.with_suffix('.console.log').write_text(sanitized(result.stdout+result.stderr), encoding='utf-8')
        report = dict(status='passed' if result.returncode == 0 else 'failed', platform=sys.platform,
                      native_plugin=env['QT_QPA_PLATFORM'], window_manager='Openbox' if window_manager else 'system',
                      cases=CASES, timed_out=timed_out,
                      limits='Native window/session automation. Not manual visual, GPU, IME or screen-reader acceptance.')
        output.write_text(json.dumps(report, indent=2)+'\n', encoding='utf-8')
        if result.returncode: raise ValueError('Native session regression failed; inspect its log')
    finally:
        if log.is_file():
            log.write_text(sanitized(log.read_text(encoding='utf-8', errors='replace')), encoding='utf-8')
        if window_manager:
            window_manager.terminate()
            try: window_manager.wait(timeout=10)
            except subprocess.TimeoutExpired:
                window_manager.kill()
                window_manager.wait(timeout=5)


if __name__ == '__main__':
    try: main()
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        raise SystemExit('Native session validation failed: '+type(error).__name__) from None
