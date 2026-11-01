#!/usr/bin/env python3
"""Bounded package, marker and extraction checks using owned fixtures."""
import importlib.util
from pathlib import Path
import stat
import subprocess
import sys
import tempfile
import unittest
import zipfile

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('replacement', ROOT/'scripts/verify-alpha8-qt-replacement.py')
replacement = importlib.util.module_from_spec(spec); spec.loader.exec_module(replacement)


class ReplacementTests(unittest.TestCase):
    def test_archive_names_reject_escape_and_duplicates(self):
        replacement.check_member_names(['dir/', 'dir/file.txt'])
        for names in [['../file'], ['/absolute'], ['C:/drive'], ['dir\\file'], ['dir/a', 'dir/a']]:
            with self.subTest(names=names), self.assertRaises(ValueError):
                replacement.check_member_names(names)

    def test_zip_extraction_rejects_symlink_and_escape(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary); target = root/'output'; target.mkdir()
            archive = root/'owned.zip'
            with zipfile.ZipFile(archive, 'w') as data:
                data.writestr('../escape', b'owned')
            with self.assertRaisesRegex(ValueError, 'Unsafe'):
                replacement.unpack(archive, target, 'windows')
            self.assertFalse((root/'escape').exists())
            with zipfile.ZipFile(archive, 'w') as data:
                link = zipfile.ZipInfo('link'); link.create_system = 3
                link.external_attr = (stat.S_IFLNK | 0o777) << 16
                data.writestr(link, '../outside')
            with self.assertRaisesRegex(ValueError, 'symlink'):
                replacement.unpack(archive, target, 'windows')

    def test_probe_event_identity_and_marker_are_required(self):
        identity = {'event': 'first_frame', 'version': '0.1.0-alpha.8', 'build': 21,
                    'revision': replacement.REVISION[:12], 'dirty': False}
        events = {name: {'event': name} for name in ['search_frame', 'sort_frame', 'browse_stable',
                  'quit_requested', 'cancel_requested', 'cancel_settled', 'extract_finished']}
        events['first_frame'] = identity
        events['qt_source_proof'] = {'event': 'qt_source_proof', 'value': replacement.PROOF}
        events['open_ready'] = {'event': 'open_ready', 'records': 100000}
        events['cancel_requested']['pending'] = True
        events['cancel_settled']['cancelled'] = True
        replacement.validate_success(events, 100000, 'browse')
        for field, value in [('version', '0.1.0-alpha.7'), ('revision', ''), ('dirty', True)]:
            with self.subTest(field=field):
                modified = {**events, 'first_frame': {**identity, field: value}}
                with self.assertRaisesRegex(ValueError, 'identity'):
                    replacement.validate_success(modified, 100000, 'browse')
        modified = {**events, 'qt_source_proof': {'value': ''}}
        with self.assertRaisesRegex(ValueError, 'proof missing'):
            replacement.validate_success(modified, 100000, 'browse')
        events['cancel_requested']['pending'] = False
        with self.assertRaisesRegex(ValueError, 'cancellation'):
            replacement.validate_success(events, 100000, 'browse')

    def test_duplicate_probe_events_are_rejected(self):
        line = 'XDZIP_PROBE {"event":"first_frame"}\n'
        with self.assertRaisesRegex(ValueError, 'Duplicate'):
            replacement.events_from(subprocess.CompletedProcess([], 0, stdout=line+line, stderr=''))

    def test_owned_extraction_checks_every_byte_and_file(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary); fixture = root/'fixture.zip'; target = root/'output'; target.mkdir()
            subprocess.run([sys.executable, str(ROOT/'scripts/generate-m3-fixture.py'), '--output', str(fixture)],
                           check=True, capture_output=True)
            with zipfile.ZipFile(fixture) as archive:
                archive.extractall(target)
            self.assertEqual(replacement.verify_output(fixture, target), 64)
            changed = next(path for path in target.rglob('*') if path.is_file())
            with changed.open('r+b') as stream:
                stream.write(b'changed')
            with self.assertRaisesRegex(ValueError, 'bytes differ'):
                replacement.verify_output(fixture, target)


if __name__ == '__main__':
    unittest.main()
