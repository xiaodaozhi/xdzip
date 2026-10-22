# scripts/test-native-window-manager.py

"""Startup synchronization and owned-process cleanup, without a Qt dependency."""
from pathlib import Path
import shlex
import subprocess
import tempfile
import unittest
from unittest import mock

import native_window_manager as manager


class WindowManagerTests(unittest.TestCase):
    def setUp(self):
        (manager.ROOT/'build').mkdir(exist_ok=True)
        self.work = tempfile.TemporaryDirectory(dir=manager.ROOT/'build')
        self.addCleanup(self.work.cleanup)
        self.directory = Path(self.work.name)
        self.process = mock.Mock()
        self.process.poll.return_value = None
        self.spawn = mock.patch.object(manager.subprocess, 'Popen', return_value=self.process)
        self.popen = self.spawn.start()
        self.addCleanup(self.spawn.stop)

    def marker(self):
        args = self.popen.call_args.args[0]
        self.assertEqual(args[:3], ['openbox', '--sm-disable', '--startup'])
        return Path(shlex.split(args[3])[-1])

    def test_metadata_does_not_replace_startup_acknowledgement(self):
        sleeps = []
        def startup(_):
            sleeps.append(1)
            if len(sleeps) == 2:
                manager.signal_ready(self.marker())
        with mock.patch.object(manager.time, 'sleep', side_effect=startup):
            with manager.OpenboxSession({}, self.directory) as session:
                self.assertEqual(len(sleeps), 2)
                self.assertIsInstance(session.ready_ms, int)
                self.assertEqual(self.marker().read_text(), manager.READY)
        self.process.terminate.assert_called_once()
        self.assertFalse(self.marker().parent.exists())

    def test_incomplete_marker_cannot_release_tests(self):
        sleeps = []
        def startup(_):
            sleeps.append(1)
            self.marker().write_text('partial' if len(sleeps) == 1 else manager.READY)
        with mock.patch.object(manager.time, 'sleep', side_effect=startup):
            with manager.OpenboxSession({}, self.directory):
                self.assertEqual(len(sleeps), 2)

    def test_missing_callback_fails_and_cleans_up(self):
        with self.assertRaisesRegex(ValueError, 'callback timed out'):
            manager.OpenboxSession({}, self.directory, timeout=0).start()
        self.process.terminate.assert_called_once()
        self.assertEqual(list(self.directory.iterdir()), [])

    def test_manager_exit_before_callback_fails(self):
        self.process.poll.return_value = 9
        with self.assertRaisesRegex(ValueError, 'exited before startup'):
            manager.OpenboxSession({}, self.directory).start()
        self.process.terminate.assert_not_called()
        self.assertEqual(list(self.directory.iterdir()), [])

    def test_spawn_error_cleans_private_directory(self):
        self.popen.side_effect = OSError('missing executable')
        with self.assertRaises(OSError):
            manager.OpenboxSession({}, self.directory).start()
        self.assertEqual(list(self.directory.iterdir()), [])

    def test_cleanup_is_bounded_and_idempotent(self):
        self.process.wait.side_effect = [subprocess.TimeoutExpired('openbox', 10), 0]
        session = manager.OpenboxSession({}, self.directory)
        session.process = self.process
        session.close()
        session.close()
        self.process.terminate.assert_called_once()
        self.process.kill.assert_called_once()
        self.assertEqual(self.process.wait.call_args_list, [mock.call(timeout=10), mock.call(timeout=5)])

    def test_each_session_has_a_fresh_marker(self):
        paths = []
        def startup(_):
            paths.append(self.marker())
            manager.signal_ready(paths[-1])
        with mock.patch.object(manager.time, 'sleep', side_effect=startup):
            for _ in range(2):
                with manager.OpenboxSession({}, self.directory):
                    pass
        self.assertNotEqual(paths[0], paths[1])

    def test_startup_arguments_preserve_spaces_and_unicode(self):
        def startup(_):
            manager.signal_ready(self.marker())
        with mock.patch.object(manager.sys, 'executable', '/tools/Python 中文/bin/python'), \
                mock.patch.object(manager.time, 'sleep', side_effect=startup):
            with manager.OpenboxSession({}, self.directory):
                command = shlex.split(self.popen.call_args.args[0][3])
                self.assertEqual(command[0], '/tools/Python 中文/bin/python')
                self.assertEqual(command[1], str(Path(manager.__file__).resolve()))

    def test_callback_is_exclusive_and_build_scoped(self):
        marker = self.directory/'ready'
        manager.signal_ready(marker)
        with self.assertRaises(FileExistsError):
            manager.signal_ready(marker)
        with self.assertRaisesRegex(ValueError, 'under build'):
            manager.signal_ready(manager.ROOT/'outside-ready')


if __name__ == '__main__':
    unittest.main()
