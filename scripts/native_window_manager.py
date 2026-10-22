# scripts/native_window_manager.py

"""Wait for Openbox's completed startup, without creating a Qt warm-up window."""
from pathlib import Path
import shlex
import subprocess
import sys
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
READY = 'xdzip-openbox-ready\n'


def stop_window_manager(process):
    """Only stop the Popen child owned by this session, never a manager found by name."""
    if process is not None and process.poll() is None:
        process.terminate()
        try:
            process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=5)


def signal_ready(path):
    path = Path(path).resolve()
    if not path.is_relative_to(ROOT/'build'):
        raise ValueError('Window-manager evidence belongs under build/')
    # The parent creates a fresh private directory. Exclusive creation never replaces evidence.
    with path.open('x', encoding='ascii') as output:
        output.write(READY)


class OpenboxSession:
    def __init__(self, env, output_dir, timeout=10):
        self.env = env
        self.output_dir = Path(output_dir)
        self.timeout = timeout
        self.process = None
        self.directory = None
        self.ready_ms = None

    def start(self):
        if self.process is not None:
            raise ValueError('Window manager is already started')
        self.directory = tempfile.TemporaryDirectory(prefix='wm-ready-', dir=self.output_dir)
        marker = Path(self.directory.name)/'ready'
        command = shlex.join([sys.executable, str(Path(__file__).resolve()), str(marker)])
        started = time.monotonic()
        try:
            self.process = subprocess.Popen(['openbox', '--sm-disable', '--startup', command],
                                            env=self.env, stdout=subprocess.DEVNULL,
                                            stderr=subprocess.DEVNULL)
            # Openbox publishes EWMH identity before theme/configuration initialization.
            # Its startup callback runs after that initialization and event registration.
            while time.monotonic() - started < self.timeout:
                if self.process.poll() is not None:
                    raise ValueError('X11 window manager exited before startup completed')
                if marker.is_file() and marker.read_text(encoding='ascii') == READY:
                    self.ready_ms = round((time.monotonic()-started)*1000)
                    return self
                time.sleep(.02)
            raise ValueError('X11 window manager startup callback timed out')
        except BaseException:
            self.close()
            raise

    def close(self):
        process, self.process = self.process, None
        try:
            stop_window_manager(process)
        finally:
            if self.directory is not None:
                self.directory.cleanup()
                self.directory = None

    def __enter__(self):
        return self.start()

    def __exit__(self, *_):
        self.close()


if __name__ == '__main__':
    if len(sys.argv) != 2:
        raise SystemExit('Expected the private Openbox startup marker')
    signal_ready(sys.argv[1])
