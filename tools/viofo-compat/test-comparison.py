#!/usr/bin/env python3
"""Synthetic regression probes against the unchanged pinned source files."""
import importlib.util
import json
from pathlib import Path
import struct
import sys
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('runner', Path(__file__).with_name('run-compare.py'))
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)
BINARY = Path(sys.argv.pop(1)).resolve()


def box(kind, data):
    return struct.pack('>I', 8 + len(data)) + kind + data


def packet(second=0, lon=0.0):
    raw_lon = int(abs(lon)) * 100 + (abs(lon) % 1) * 60
    return struct.pack('<6I', 0, 0, second, 26, 8, 31) + b'ANE\0' + struct.pack('<4f', 0.0, raw_lon, 11.12 / 0.514444, 90.0)


def fixture(records):
    frees = [box(b'free', b'GPS ' + bytes(16) + record + bytes(32)) for record in records]
    offset = 8 + 16 + 8 * len(records)
    descriptors = []
    for free in frees:
        descriptors.append(struct.pack('>II', offset, len(free)))
        offset += len(free)
    return box(b'moov', box(b'gps ', bytes(8) + b''.join(descriptors))) + b''.join(frees)


class ComparisonTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)

    def decode(self, data, mode):
        path = Path(self.temp.name) / '2026_0831_000000_001F.MP4'
        path.write_bytes(data)
        return runner.worker(BINARY, mode, path, timeout=0.5)

    def names(self, name):
        return runner.worker(BINARY, 'names', Path(name))

    def test_sample_record_from_original_a129_unit_test_decodes_in_both(self):
        record = bytes.fromhex('0000000023000000070000001a000000080000000e000000414e450035149d4530bc3c4433339a4214ee1a43')
        for mode in ['original', 'ours-raw']:
            output = self.decode(fixture([record]), mode)
            self.assertEqual(output['status'], 'ok')
            self.assertEqual(len(output['points']), 1)
            self.assertAlmostEqual(output['points'][0]['lat'], 50.442098, places=6)
            self.assertAlmostEqual(output['points'][0]['lon'], 7.915674, places=6)

    def test_legacy_counter_free_names_only_our_parser_accepts(self):
        for channel in ['F', 'I', 'R']:
            result = self.names(f'2025_0706_201953_{channel}.MP4')
            self.assertIsNone(result['original'])
            self.assertEqual(result['ours']['kind'], 'viofo')

    def test_numbered_three_channels_work_in_both(self):
        for channel, serial in [('F', 646), ('I', 647), ('R', 648)]:
            result = self.names(f'2026_0831_065229_{serial:06}{channel}.MP4')
            self.assertEqual(result['original']['kind'], 'viofo')
            self.assertEqual(result['ours']['kind'], 'viofo')

    def test_minute_grouping_collides_for_distinct_recordings(self):
        a = self.names('2026_0831_065201_001F.MP4')
        b = self.names('2026_0831_065259_002F.MP4')
        self.assertEqual(a['original']['group'], b['original']['group'])
        self.assertNotEqual(a['ours']['group'], b['ours']['group'])

    def test_one_second_shift_within_minute_only_original_groups(self):
        a = self.names('2026_0831_065229_001F.MP4')
        b = self.names('2026_0831_065230_002R.MP4')
        self.assertEqual(a['original']['group'], b['original']['group'])
        self.assertNotEqual(a['ours']['group'], b['ours']['group'])

    def test_one_second_shift_at_minute_boundary_neither_groups(self):
        a = self.names('2026_0831_065259_001F.MP4')
        b = self.names('2026_0831_065300_002R.MP4')
        for mode in ['original', 'ours']:
            self.assertNotEqual(a[mode]['group'], b[mode]['group'])

    def test_parking_group_keys_are_separate_in_both(self):
        a = self.names('2026_0831_065229_001F.MP4')
        b = self.names('2026_0831_065229_002PF.MP4')
        for mode in ['original', 'ours']:
            self.assertNotEqual(a[mode]['group'], b[mode]['group'])

    def test_in_range_continent_jump_is_retained_only_by_original(self):
        data = fixture([packet(i, 20 if i == 2 else i * 0.0001) for i in range(5)])
        original = self.decode(data, 'original')
        ours = self.decode(data, 'ours-raw')
        self.assertEqual(len(original['points']), 5)
        self.assertEqual(len(ours['points']), 4)
        self.assertGreater(runner.summary(original)['max_adjacent_distance_km'], 2000)
        self.assertLess(runner.summary(ours)['max_adjacent_distance_km'], 1)

    def test_empty_gps_descriptor_original_times_out_ours_returns(self):
        data = box(b'moov', box(b'gps ', bytes(8)))
        self.assertEqual(self.decode(data, 'original')['status'], 'timeout')
        self.assertEqual(self.decode(data, 'ours-raw')['points'], [])


if __name__ == '__main__':
    unittest.main()
