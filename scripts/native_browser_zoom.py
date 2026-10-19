"""Drive real Chromium page zoom with native keys on a dedicated virtual display.

DOM key injection and CSS zoom do not exercise the browser's page zoom. A private
Xvfb display also prevents verification keys from reaching another desktop app.
"""
from contextlib import contextmanager
import ctypes
import ctypes.util
import os
import select
import shutil
import subprocess
import tempfile


@contextmanager
def isolated_display():
    executable = os.environ.get('XDZIP_XVFB') or shutil.which('Xvfb')
    if not executable:
        raise RuntimeError('Actual browser zoom verification requires Xvfb')
    with tempfile.TemporaryFile() as log:
        process = subprocess.Popen([executable, '-displayfd', '1', '-screen', '0',
                                    '2400x1600x24', '-ac', '-nolisten', 'tcp'],
                                   stdout=subprocess.PIPE, stderr=log)
        try:
            if not select.select([process.stdout], [], [], 10)[0]:
                raise RuntimeError('Private Xvfb display did not start')
            number = process.stdout.readline().decode().strip()
            if not number.isdigit():
                log.seek(0)
                raise RuntimeError('Xvfb failed: ' + log.read().decode(errors='replace')[-1000:])
            yield ':' + number
        finally:
            process.terminate()
            process.wait(timeout=10)


class NativeZoom:
    def __init__(self, display):
        self.x = ctypes.CDLL(ctypes.util.find_library('X11'))
        self.xt = ctypes.CDLL(ctypes.util.find_library('Xtst'))
        self.x.XOpenDisplay.argtypes = [ctypes.c_char_p]
        self.x.XOpenDisplay.restype = ctypes.c_void_p
        self.x.XCloseDisplay.argtypes = [ctypes.c_void_p]
        self.x.XStringToKeysym.argtypes = [ctypes.c_char_p]
        self.x.XStringToKeysym.restype = ctypes.c_ulong
        self.x.XKeysymToKeycode.argtypes = [ctypes.c_void_p, ctypes.c_ulong]
        self.x.XKeysymToKeycode.restype = ctypes.c_uint
        self.x.XFlush.argtypes = [ctypes.c_void_p]
        self.xt.XTestFakeKeyEvent.argtypes = [ctypes.c_void_p, ctypes.c_uint, ctypes.c_int, ctypes.c_ulong]
        self.display = self.x.XOpenDisplay(display.encode())
        if not self.display:
            raise RuntimeError('Cannot connect to the private verification display')

    def chord(self, key):
        control = self.x.XKeysymToKeycode(self.display, self.x.XStringToKeysym(b'Control_L'))
        target = self.x.XKeysymToKeycode(self.display, self.x.XStringToKeysym(key.encode()))
        for code, pressed in [(control, 1), (target, 1), (target, 0), (control, 0)]:
            self.xt.XTestFakeKeyEvent(self.display, code, pressed, 0)
        self.x.XFlush(self.display)

    def set_zoom(self, page, native_dpr, zoom):
        page.bring_to_front()
        self.chord('0')
        page.wait_for_function('dpr => Math.abs(devicePixelRatio - dpr) < .001', arg=native_dpr)
        if zoom == 1.25:
            for factor in [1.1, 1.25]:
                self.chord('equal')
                page.wait_for_function('dpr => Math.abs(devicePixelRatio - dpr) < .001', arg=native_dpr * factor)
        elif zoom != 1:
            raise ValueError('This verification covers actual page zoom 100% and 125%')
        # A desktop page zoom changes DPR/layout viewport, not the pinch-zoom viewport.
        if abs(page.evaluate('visualViewport.scale') - 1) > .001:
            raise RuntimeError('Unexpected pinch zoom; desktop page zoom evidence is invalid')

    def close(self):
        self.x.XCloseDisplay(self.display)
