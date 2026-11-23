#!/usr/bin/env python3
"""Exercise Snap desktop backend selection with real Unix-domain sockets."""
import json
import os
from pathlib import Path
import shutil
import socket
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
LAUNCH = ROOT / 'apps/desktop/packaging/linux/snap-launch.sh'


@unittest.skipUnless(shutil.which('sh'), 'requires a POSIX shell')
class SnapKeyboardTests(unittest.TestCase):
    def test_launcher_sets_libxkbcommon_config_root(self):
        with tempfile.TemporaryDirectory(prefix='xdzip-snap-keyboard-') as directory:
            root = Path(directory)
            snap = root / 'snap with spaces'
            app = snap / 'runtime/bin/xdzip'
            app.parent.mkdir(parents=True)
            app.write_bytes(b'#!/bin/sh\nprintf "%s\\n" "${XKB_CONFIG_ROOT-unset}"\n')
            app.chmod(0o755)
            environment = dict(os.environ, SNAP=snap.name, QT_QPA_PLATFORM='wayland',
                               WAYLAND_DISPLAY='', XKB_CONFIG_ROOT='/nonexistent')
            result = subprocess.run(['sh', LAUNCH.as_posix()], cwd=root, env=environment,
                                    text=True, capture_output=True, check=True, timeout=10)
            self.assertEqual(result.stdout.strip(), f'{snap.name}/usr/share/X11/xkb')


@unittest.skipUnless(os.name == 'posix' and hasattr(socket, 'AF_UNIX'), 'requires Unix sockets')
class SnapLaunchTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='xdzip-snap-launch-')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.runtime = self.root / 'run' / 'snap.xdzip'
        self.runtime.mkdir(parents=True)
        self.snap = self.root / 'snap'
        app = self.snap / 'runtime/bin/xdzip'
        app.parent.mkdir(parents=True)
        app.write_text(f'''#!{sys.executable}
import json, os, socket
platform = os.environ['QT_QPA_PLATFORM']
if platform == 'wayland':
    display = os.environ['WAYLAND_DISPLAY']
    address = display if os.path.isabs(display) else os.path.join(os.environ['XDG_RUNTIME_DIR'], display)
    client = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    client.connect(address)
    client.close()
print(json.dumps({{'platform': platform, 'args': __import__('sys').argv[1:]}}))
''')
        app.chmod(0o755)

    def listener(self, name='wayland-0'):
        server = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.addCleanup(server.close)
        path = self.runtime.parent / name
        server.bind(str(path))
        server.listen(5)
        return path

    def launch(self, display=None, platform=None):
        environment = dict(os.environ, SNAP=str(self.snap), XDG_RUNTIME_DIR=str(self.runtime))
        environment.pop('WAYLAND_DISPLAY', None)
        environment.pop('QT_QPA_PLATFORM', None)
        if display is not None:
            environment['WAYLAND_DISPLAY'] = str(display)
        if platform is not None:
            environment['QT_QPA_PLATFORM'] = platform
        result = subprocess.run(['sh', str(LAUNCH), 'archive with spaces.zip'], env=environment,
                                text=True, capture_output=True, check=True, timeout=10)
        return json.loads(result.stdout)

    def test_private_runtime_connects_to_parent_socket_and_reuses_link(self):
        self.listener()
        expected = {'platform': 'wayland', 'args': ['archive with spaces.zip']}
        self.assertEqual(self.launch('wayland-0'), expected)
        link = self.runtime / 'wayland-0'
        self.assertTrue(link.is_symlink())
        self.assertEqual(link.resolve(), (self.runtime.parent / 'wayland-0').resolve())
        self.assertEqual(self.launch('wayland-0'), expected)

    def test_missing_or_blocked_socket_falls_back_without_replacing_files(self):
        self.assertEqual(self.launch('wayland-0')['platform'], 'xcb')
        self.listener()
        blocked = self.runtime / 'wayland-0'
        blocked.write_text('keep')
        self.assertEqual(self.launch('wayland-0')['platform'], 'xcb')
        self.assertEqual(blocked.read_text(), 'keep')

    def test_absolute_socket_and_explicit_platform(self):
        path = self.listener()
        self.assertEqual(self.launch(path)['platform'], 'wayland')
        self.assertEqual(self.launch('wayland-0', 'offscreen')['platform'], 'offscreen')
        self.assertEqual(self.launch()['platform'], 'xcb')
        self.assertEqual(self.launch('../wayland-0')['platform'], 'xcb')


if __name__ == '__main__':
    unittest.main()
