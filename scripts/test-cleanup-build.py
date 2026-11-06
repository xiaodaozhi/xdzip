#!/usr/bin/env python3
"""Regression tests for the build cleanup safety boundaries."""
import importlib.util
import os
from pathlib import Path
import tempfile
import time
import unittest
from unittest.mock import patch


SCRIPT = Path(__file__).with_name("cleanup-build.py")
SPEC = importlib.util.spec_from_file_location("cleanup_build", SCRIPT)
cleanup = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(cleanup)


class CleanupBuildTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.build = self.root / "build"
        self.build.mkdir()
        self.patches = patch.multiple(cleanup, ROOT=self.root, BUILD=self.build)
        self.patches.start()
        self.addCleanup(self.patches.stop)

    def test_explicit_target_stays_below_build_and_rejects_protected_directory(self):
        cycle = self.build / "issue230-cycle"
        cycle.mkdir()
        self.assertEqual(cleanup.explicit_candidates([cycle], set()), [cycle])
        with self.assertRaisesRegex(ValueError, "protected"):
            cleanup.validate_target(self.build / "desktop")
        with self.assertRaisesRegex(ValueError, "below build"):
            cleanup.validate_target(self.root / "outside")

    def test_stale_scan_skips_active_worktrees_and_protected_directories(self):
        old = self.build / "issue230-old"
        active = self.build / "issue230-active"
        protected = self.build / "desktop"
        for directory in (old, active, protected):
            directory.mkdir()
            (directory / "marker").write_text("temporary")
            timestamp = time.time() - 90 * 86400
            os.utime(directory / "marker", (timestamp, timestamp))
            os.utime(directory, (timestamp, timestamp))
        candidates = cleanup.stale_candidates(30, {active.resolve()})
        self.assertEqual([path for path, _ in candidates], [old])

    def test_main_worktree_does_not_protect_build_children(self):
        cycle = self.build / "issue230-cycle"
        active = cycle / "nested-worktree"
        cycle.mkdir()
        active.mkdir()

        self.assertEqual(cleanup.explicit_candidates([cycle], {self.root.resolve()}), [cycle])
        self.assertEqual(cleanup.explicit_candidates([cycle], {active.resolve()}), [])
        self.assertEqual(cleanup.explicit_candidates([active], {active.resolve()}), [])

    def test_remove_deletes_only_selected_target(self):
        cycle = self.build / "issue230-cycle"
        cycle.mkdir()
        (cycle / "output.log").write_text("evidence")
        cleanup.remove(cycle)
        self.assertFalse(cycle.exists())


if __name__ == "__main__":
    unittest.main()
