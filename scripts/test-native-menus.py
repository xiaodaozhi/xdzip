# scripts/test-native-menus.py

"""Check first-frame evidence admission without Qt, user input or visible windows."""
import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest import mock

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('native_menus', ROOT / 'scripts/check-native-menus.py')
menus = importlib.util.module_from_spec(spec)
spec.loader.exec_module(menus)
SOURCE = '1' * 40


class MenuEvidenceTests(unittest.TestCase):
    def setUp(self):
        self.contract = menus.load_contract()
        self.records = []
        for repetition in range(1, self.contract['repetitions'] + 1):
            for case in self.contract['cases']:
                self.records.append({'menu': case, 'repetition': repetition,
                    'trigger_to_complete_frame_ms': 25, 'trigger_to_capture_complete_ms': 35,
                    'shadow_same_frame': True, 'first_visible_menu_sync_generation': 2,
                    **{field: 2 for field in self.contract['required_generation_fields']},
                    **self.contract['windows'], 'revision': SOURCE[:12]})

    def summary(self, records=None):
        return menus.summarize(self.records if records is None else records, [], self.contract,
                               source=SOURCE, strict_windows=True)

    def test_complete_matrix_passes_and_generation_or_stage_forgery_fails(self):
        self.assertEqual(len(self.records), 45)
        self.assertEqual(self.summary()['status'], 'passed')
        invalid = {field: 3 for field in self.contract['required_generation_fields']}
        invalid.update(capture_stage='GUI queued QScreen readback', shadow_same_frame=False,
                       first_visible_menu_sync_generation=0, revision='wrong',
                       trigger_to_complete_frame_ms=float('nan'))
        for field, value in invalid.items():
            with self.subTest(field=field):
                records = copy.deepcopy(self.records)
                records[0][field] = value
                self.assertEqual(self.summary(records)['status'], 'failed')

    def test_partial_and_duplicate_matrices_cannot_be_relabelled_complete(self):
        self.assertEqual(self.summary(self.records[:26])['status'], 'failed')
        records = copy.deepcopy(self.records)
        records[-1] = records[0]
        self.assertEqual(self.summary(records)['status'], 'failed')

    def test_nearest_rank_timing_limits_remain_strict(self):
        self.records[-1]['trigger_to_complete_frame_ms'] = 200
        self.assertEqual(self.summary()['status'], 'passed')
        self.records[-1]['trigger_to_complete_frame_ms'] = 200.01
        self.assertEqual(self.summary()['status'], 'failed')
        for record in self.records[-3:]:
            record['trigger_to_complete_frame_ms'] = 100.01
        self.assertEqual(self.summary()['status'], 'failed')

    def test_existing_evidence_binds_native_records_source_binary_and_owned_launches(self):
        (ROOT / 'build').mkdir(exist_ok=True)
        with tempfile.TemporaryDirectory(prefix='menu-contract-', dir=ROOT / 'build') as temporary:
            directory = Path(temporary)
            binary = directory / 'owned-not-executed.exe'
            binary.write_bytes(b'owned test identity, never executable')
            digest = hashlib.sha256(binary.read_bytes()).hexdigest()
            launcher = {'status': 'passed', 'full_source_sha': SOURCE, 'binary_sha256': digest,
                'started_by_real_foreground_mouse_click': True, 'fresh_process_per_sample': True,
                'menu_warmup': False, 'valid_samples': 45, 'records': self.records, 'failures': [],
                'owned_child_launches': [{'sample': index + 1, 'menu': record['menu'],
                    'owned_child_pid': 4000 + index,
                    'binary_sha256': digest, 'binary_path_verified': True,
                    'child_os_foreground_seen': True, 'exit_code': 0}
                    for index, record in enumerate(self.records)]}
            for record in self.records:
                case, repetition = record['menu'], record['repetition']
                sample = directory / f'{repetition:02}-{case}'
                sample.mkdir()
                native = {key: value for key, value in record.items() if key != 'repetition'}
                (sample / (case + '.json')).write_text(json.dumps(native), encoding='utf-8')
            report = directory / 'review.json'
            report.write_text(json.dumps(launcher), encoding='utf-8')
            before = report.read_bytes()
            with mock.patch.object(menus.subprocess, 'run', side_effect=AssertionError('Qt must not run')):
                def validate(source=SOURCE, expected=digest):
                    return menus.validate_existing(directory, binary, source, expected, self.contract)
                self.assertEqual(validate()['status'], 'passed')
                self.assertEqual(validate(source='2' * 40)['status'], 'failed')
                self.assertEqual(validate(expected='0' * 64)['status'], 'failed')
                for invalid_pid in (None, True, '4000', 0, -1):
                    launcher['owned_child_launches'][0]['owned_child_pid'] = invalid_pid
                    report.write_text(json.dumps(launcher), encoding='utf-8')
                    self.assertEqual(validate()['status'], 'failed')
                launcher['owned_child_launches'][0]['owned_child_pid'] = 4000
                launcher['owned_child_launches'][-1]['owned_child_pid'] = 4000
                report.write_text(json.dumps(launcher), encoding='utf-8')
                self.assertEqual(validate()['status'], 'passed')
                launcher['owned_child_launches'][0]['child_os_foreground_seen'] = False
                report.write_text(json.dumps(launcher), encoding='utf-8')
                self.assertEqual(validate()['status'], 'failed')
                launcher['owned_child_launches'][0]['child_os_foreground_seen'] = True
                launcher['records'][0]['capture_end_sync_generation'] = 3
                report.write_text(json.dumps(launcher), encoding='utf-8')
                self.assertEqual(validate()['status'], 'failed')
                report.write_bytes(before)
                self.assertEqual(validate()['status'], 'passed')
                self.assertEqual(report.read_bytes(), before)


if __name__ == '__main__':
    unittest.main()
