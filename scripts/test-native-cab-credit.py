"""Exercise the actual Python worker transport parser with bounded wire frames."""
import importlib.util
import io
from pathlib import Path
import struct
import unittest
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location('native_codec_check', Path(__file__).with_name('check-native-codecs.py'))
NATIVE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(NATIVE)


def frame(kind, value=b''):
    return struct.pack('<I', 1 + len(value)) + bytes([kind]) + value


def work(count):
    return frame(17, struct.pack('<Q', count))


def row(size, index=0):
    name = b'known.bin'
    return frame(13, struct.pack('<QBQQIII', index, 0, size, 0, 0, 0, len(name)) + name)


class Pipe(io.BytesIO):
    def close(self):
        self.sent = self.getvalue()
        super().close()


class Process:
    def __init__(self, frames):
        self.stdin = Pipe()
        self.stdout = io.BytesIO(b''.join(frames))
        self.finished = False
        self.killed = False

    def wait(self, timeout):
        self.finished = True
        return 0

    def poll(self):
        return 0 if self.finished else None

    def kill(self):
        self.killed = self.finished = True

    def acks(self):
        stream = io.BytesIO(self.stdin.sent)
        kinds = []
        while length := stream.read(4):
            kinds.append(stream.read(struct.unpack('<I', length)[0])[0])
        return kinds.count(21)


class CreditWireTests(unittest.TestCase):
    def run_payload(self, frames, size=3, format_id=8, selected=0, creation=None):
        processes = []
        if format_id == 8 and selected is not None and creation is None:
            processes.append(Process([row(size), frame(15)]))
        payload = Process(frames)
        processes.append(payload)
        with patch.object(NATIVE.subprocess, 'Popen', side_effect=processes), patch.object(NATIVE.threading, 'Timer'):
            result = NATIVE.run_worker('self-owned-wire-test', [b'input'], format_id=format_id,
                                       selected=selected, creation=creation)
        return result, payload

    def rejected(self, expected, frames, **options):
        with self.assertRaisesRegex(NATIVE.WorkerError, '^' + expected + '$'):
            self.run_payload(frames, **options)

    def test_paid_output_and_ordinary_output_keep_separate_admission(self):
        (rows, data), process = self.run_payload([work(2), frame(12, b'a'), frame(23, b'bc'), frame(15)])
        self.assertEqual((rows, data), ([], b'abc'))
        self.assertEqual(process.acks(), 3)
        self.assertFalse(process.killed)

    def test_work_credit_accumulates_only_within_the_same_session(self):
        (_, data), _ = self.run_payload([work(1), work(2), frame(23, b'abc'), frame(15)])
        self.assertEqual(data, b'abc')
        self.rejected('precharged_credit', [frame(23, b'abc')])

    def test_credit_is_spent_once(self):
        self.rejected('precharged_credit', [work(2), frame(23, b'ab'), frame(23, b'c')])

    def test_prepaid_output_needs_credit_and_cannot_exceed_it(self):
        self.rejected('precharged_credit', [frame(23, b'x')])
        self.rejected('precharged_credit', [work(2), frame(23, b'abc')])

    def test_non_cab_scan_and_creation_cannot_emit_prepaid_output(self):
        for options in ({'format_id': 7}, {'format_id': 4}, {'format_id': 9}, {'format_id': 10},
                        {'selected': None}, {'selected': None, 'creation': {}}):
            with self.subTest(options=options):
                self.rejected('unexpected_precharged_data', [work(3), frame(23, b'abc')], **options)

    def test_metadata_work_cannot_fund_a_later_payload_session(self):
        scan = Process([work(3), row(3), frame(15)])
        payload = Process([frame(23, b'abc')])
        with patch.object(NATIVE.subprocess, 'Popen', side_effect=[scan, payload]), patch.object(NATIVE.threading, 'Timer'):
            with self.assertRaisesRegex(NATIVE.WorkerError, '^precharged_credit$'):
                NATIVE.run_worker('self-owned-wire-test', [b'input'], selected=0, format_id=8)
        self.assertEqual(payload.acks(), 0)
        self.assertTrue(payload.killed)

    def test_invalid_work_lengths_and_bounds_are_not_credit(self):
        for value in (b'', b'1234567', b'123456789'):
            with self.subTest(length=len(value)):
                self.rejected('work_length', [frame(17, value)])
        self.rejected('work_limit', [work(NATIVE.CHUNK + 1)])
        self.rejected('work_limit', [work(NATIVE.CHUNK)] * 1025)

    def test_prepaid_chunk_is_nonempty_and_bounded(self):
        self.rejected('precharged_empty', [work(1), frame(23)])
        self.rejected('output_limit', [work(NATIVE.CHUNK), work(1), frame(23, b'x' * (NATIVE.CHUNK + 1))],
                      size=NATIVE.CHUNK + 1)

    def test_both_output_paths_obey_the_selected_metadata_size(self):
        self.rejected('entry_output_bounds', [work(4), frame(23, b'abcd')])
        self.rejected('entry_output_bounds', [frame(12, b'abcd')])
        self.rejected('entry_output_bounds', [work(3), frame(23, b'ab'), frame(15)])
        self.rejected('entry_output_bounds', [frame(15)])

    def test_missing_or_duplicate_selected_metadata_is_not_output_authority(self):
        for rows in ([], [row(3, index=1)], [row(3), row(3)]):
            with self.subTest(rows=len(rows)):
                scan = Process([*rows, frame(15)])
                with patch.object(NATIVE.subprocess, 'Popen', return_value=scan), patch.object(NATIVE.threading, 'Timer'):
                    with self.assertRaisesRegex(NATIVE.WorkerError, '^selected_metadata$'):
                        NATIVE.run_worker('self-owned-wire-test', [b'input'], selected=0, format_id=8)

    def test_ordinary_non_cab_data_remains_compatible_without_credit(self):
        (_, data), _ = self.run_payload([frame(12, b'abc'), frame(15)], format_id=7)
        self.assertEqual(data, b'abc')


if __name__ == '__main__':
    unittest.main()
