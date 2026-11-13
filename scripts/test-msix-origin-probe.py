#!/usr/bin/env python3
"""Regression checks for the read-only staged-package observation."""
import importlib.util
from pathlib import Path
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('msix_origin_probe', ROOT / 'scripts/msix-origin-probe.py')
probe = importlib.util.module_from_spec(spec)
spec.loader.exec_module(probe)


class OriginProbeTests(unittest.TestCase):
    def test_observation_never_classifies_store_signature_as_install_channel(self):
        report = probe.observe('xdzip.Test', 'copied-sideload',
            package_query=lambda _: {'full_name': 'example_full_name', 'signature': 'Store'},
            origin_query=lambda _: (0, 3))
        self.assertEqual(report['signature_kind'], 'Store')
        self.assertEqual(report['staged_origin_name'], 'Store')
        self.assertFalse(report['scenario_label_is_evidence'])
        self.assertNotIn('install_channel', report)
        self.assertNotIn('example_full_name', str(report))

    def test_api_error_has_no_fabricated_origin(self):
        report = probe.observe('xdzip.Test', 'unspecified',
            package_query=lambda _: {'full_name': 'example', 'signature': 'Developer'},
            origin_query=lambda _: (87, None))
        self.assertEqual(report['staged_origin_api_status'], 87)
        self.assertIsNone(report['staged_origin_value'])
        self.assertIsNone(report['staged_origin_name'])

    def test_unknown_origin_stays_unrecognized(self):
        report = probe.observe('xdzip.Test', 'other-user',
            package_query=lambda _: {'full_name': 'example', 'signature': 'Store'},
            origin_query=lambda _: (0, 987))
        self.assertEqual(report['staged_origin_name'], 'Unrecognized')

    def test_invalid_package_name_never_starts_powershell(self):
        with patch.object(probe.subprocess, 'run') as run:
            with self.assertRaises(ValueError):
                probe.installed_package('bad*name')
            run.assert_not_called()

    def test_missing_or_ambiguous_package_is_rejected(self):
        for data in ('[]', '[{},{}]', '[{"full_name":"","signature":"Store"}]'):
            with self.subTest(data=data), patch.object(probe.subprocess, 'run') as run:
                run.return_value.returncode = 0
                run.return_value.stdout = data
                with self.assertRaisesRegex(ValueError, 'exactly one'):
                    probe.installed_package('xdzip.Test')


if __name__ == '__main__':
    unittest.main()
