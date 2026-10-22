# scripts/check-x11-startup.py

"""Reproduce the early-EWMH startup race and verify cold Qt windows after startup acknowledgement."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import time

from native_window_manager import OpenboxSession, stop_window_manager

ROOT = Path(__file__).resolve().parents[1]
FIRST = 'BrowserTest::windowHoverEndsBeforeRestore(WindowCaptionButton-hide)'


def sanitized(value):
    return value.replace(str(ROOT), '<workspace>').replace(str(Path.home()), '<home>')


def run_case(args):
    output = args.output.resolve()
    env = dict(os.environ, QT_QPA_PLATFORM='xcb', QT_QUICK_BACKEND='software')
    # Inject only into our WM child. The Qt process never inherits LD_PRELOAD.
    wm_env = dict(env, LD_PRELOAD=str(args.injector.resolve()),
                  XDZIP_TEST_WM_DELAY_MS=str(args.delay_ms))
    session = process = None
    started = time.monotonic()
    try:
        if args.mode == 'identity':
            process = subprocess.Popen(['openbox', '--sm-disable'], env=wm_env,
                                       stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            while time.monotonic()-started < 10:
                if process.poll() is not None:
                    raise ValueError('Negative-control WM exited')
                property_ = subprocess.run(['xprop', '-root', '_NET_SUPPORTING_WM_CHECK'],
                    env=env, capture_output=True, text=True, timeout=2)
                if property_.returncode == 0 and 'window id #' in property_.stdout:
                    break
                time.sleep(.02)
            else:
                raise ValueError('Negative-control EWMH identity timed out')
        else:
            session = OpenboxSession(wm_env, output.parent).start()
        setup_ms = round((time.monotonic()-started)*1000)
        test_started = time.monotonic()
        log = output.with_suffix('.log')
        result = subprocess.run([str(args.tests.resolve()), 'windowHoverEndsBeforeRestore',
                                 '-o', str(log)+',txt'], env=env, capture_output=True,
                                text=True, timeout=25)
        elapsed_ms = round((time.monotonic()-test_started)*1000)
        text = sanitized(log.read_text(encoding='utf-8'))
        log.write_text(text, encoding='utf-8')
        output.with_suffix('.console.log').write_text(
            sanitized(result.stdout+result.stderr), encoding='utf-8')
        exposed = 'PASS   : '+FIRST in text
        expected_failure = args.mode == 'identity'
        if expected_failure:
            passed = (result.returncode != 0 and 'FAIL!  : '+FIRST in text and
                      'Map State: IsUnMapped' in text)
        else:
            passed = result.returncode == 0 and exposed
        report = dict(status='passed' if passed else 'failed', mode=args.mode,
                      delay_ms=args.delay_ms, expected_failure=expected_failure,
                      qt_exit_code=result.returncode, first_window_exposed=exposed,
                      window_manager_ready_ms=session.ready_ms if session else None,
                      setup_ms=setup_ms, qt_duration_ms=elapsed_ms)
        output.write_text(json.dumps(report, indent=2)+'\n', encoding='utf-8')
        if not passed:
            raise ValueError('X11 startup regression did not match its expected outcome')
    finally:
        if session:
            session.close()
        stop_window_manager(process)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--tests', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--repeat', type=int, default=3)
    parser.add_argument('--inside-x11', action='store_true', help=argparse.SUPPRESS)
    parser.add_argument('--mode', choices=['identity', 'callback'], default='callback', help=argparse.SUPPRESS)
    parser.add_argument('--delay-ms', type=int, default=0, help=argparse.SUPPRESS)
    parser.add_argument('--injector', type=Path, help=argparse.SUPPRESS)
    args = parser.parse_args()
    if sys.platform != 'linux':
        raise ValueError('X11 startup regression requires Linux')
    output = args.output.resolve()
    if not output.is_relative_to(ROOT/'build') or output.suffix != '.json':
        raise ValueError('X11 startup evidence belongs under build/')
    if not 1 <= args.repeat <= 20 or not 0 <= args.delay_ms <= 8000:
        raise ValueError('Startup repetitions/delay are outside the bounded test range')
    output.parent.mkdir(parents=True, exist_ok=True)
    if args.inside_x11:
        if not args.injector or not args.injector.resolve().is_relative_to(ROOT/'build'):
            raise ValueError('Startup injector must belong under build/')
        run_case(args)
        return
    cases_dir = output.parent/output.stem
    cases_dir.mkdir(exist_ok=True)
    injector = cases_dir/'delayed-openbox.so'
    compile_ = subprocess.run(['cc', '-std=c11', '-Wall', '-Wextra', '-Werror', '-shared', '-fPIC',
        str(ROOT/'tests/fixtures/native/delayed_openbox.c'), '-o', str(injector), '-ldl', '-lX11'],
        capture_output=True, text=True)
    (cases_dir/'compile.log').write_text(sanitized(compile_.stdout+compile_.stderr), encoding='utf-8')
    compile_.check_returncode()
    cases = [('normal-'+str(i), 'callback', 0) for i in range(args.repeat)]
    cases += [('delayed-'+str(i), 'callback', 1500) for i in range(args.repeat)]
    cases += [('slow-start', 'callback', 6500), ('legacy-negative', 'identity', 6500)]
    reports = []
    aggregate = dict(status='running', cases=reports,
                     limits='Real X11/Openbox and a fresh Qt process for every case; no Qt warm-up or assertion retries.')
    try:
        for name, mode, delay in cases:
            case_output = cases_dir/(name+'.json')
            case_output.unlink(missing_ok=True)
            result = subprocess.run(['xvfb-run', '-a', sys.executable, str(Path(__file__).resolve()),
                '--inside-x11', '--mode', mode, '--delay-ms', str(delay), '--injector', str(injector),
                '--tests', str(args.tests.resolve()), '--output', str(case_output)], check=False, timeout=60)
            if case_output.is_file():
                reports.append(dict(name=name, **json.loads(case_output.read_text(encoding='utf-8'))))
            if result.returncode:
                raise ValueError('Cold X11 startup case failed: '+name)
            print('X11 cold startup verified: '+name, flush=True)
        aggregate['status'] = 'passed'
    finally:
        if aggregate['status'] != 'passed':
            aggregate['status'] = 'failed'
        output.write_text(json.dumps(aggregate, indent=2)+'\n', encoding='utf-8')


if __name__ == '__main__':
    try:
        main()
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        raise SystemExit('X11 startup regression failed: '+sanitized(str(error))) from None
