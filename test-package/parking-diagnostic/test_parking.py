import contextlib
import importlib.util
import io
import json
from pathlib import Path
import struct
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import diagnose
import parking_analysis as analysis
import time_core

# Reuse the already verified MP4 timing and Novatek layout fixtures.
spec = importlib.util.spec_from_file_location('previous_tests', HERE.parent / 'test_viofo_time_diagnostic.py')
previous = importlib.util.module_from_spec(spec)
spec.loader.exec_module(previous)


class ParkingTests(unittest.TestCase):
    def analyse(self, data, name='2020_0101_000000_001PF.MP4'):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / name
            path.write_bytes(data)
            result, private = analysis.analyse_file(path)
            self.assertEqual(path.read_bytes(), data)
            serialized = json.dumps(result, allow_nan=False)
            self.assertNotIn(directory, serialized)
            self.assertNotIn(name, serialized)
            self.assertNotIn('2020', serialized)
            self.assertNotIn('byte_position', serialized)
            self.assertNotIn('latitude', serialized)
            self.assertNotIn('longitude', serialized)
            self.assertNotIn('utc', serialized.replace('utc_from_first_valid_exceeds_video_duration', ''))
            return result, private

    def test_regular_and_compressed_clocks(self):
        regular, _ = self.analyse(previous.fixture())
        self.assertEqual(regular['gps']['temporal_rules']['with_media_duration_bound']['ambiguous'], 0)
        parking, _ = self.analyse(previous.fixture(count=119, step=15, duration=120))
        self.assertEqual(parking['gps']['temporal_rules']['with_media_duration_bound']['ambiguous'], 110)
        self.assertEqual(parking['gps']['temporal_rules']['without_media_duration_bound']['consistent'], 119)
        self.assertGreater(parking['gps']['utc_from_first_valid_exceeds_video_duration'], 0)

    def test_all_candidates_are_counted_but_selection_stays_last(self):
        one = previous.fixture(count=1, duration=3)
        packet = one[-44:]
        free = previous.box(b'free', b'GPS ' + packet + packet)
        mdhd = previous.box(b'mdhd', bytes(12) + struct.pack('>II', 1000, 3000))
        video = previous.box(b'trak', previous.box(b'mdia', mdhd + previous.box(b'hdlr', bytes(8) + b'vide')))
        placeholder = previous.box(b'moov', video + previous.box(b'gps ', bytes(16)))
        data = previous.box(b'moov', video + previous.box(b'gps ', bytes(8) + struct.pack('>II', len(placeholder), len(free)))) + free
        result, _ = self.analyse(data)
        self.assertEqual(result['gps']['active_record_candidates'], 2)
        self.assertEqual(result['gps']['selected_record_times']['records'], 1)
        self.assertEqual(result['gps']['candidate_records_per_block']['maximum'], 2)

    def test_inactive_fix_records_are_not_confused_with_missing_time(self):
        data = bytearray(previous.fixture(count=1))
        data[-20] = ord('V')
        result, _ = self.analyse(bytes(data))
        self.assertEqual(result['gps']['inactive_record_candidates'], 1)
        self.assertEqual(result['gps']['selected_record_times']['records'], 0)

    def test_missing_gps_is_explicit_and_valid_input(self):
        data = previous.box(b'moov', previous.box(b'trak', previous.box(b'mdia',
            previous.box(b'mdhd', bytes(12) + struct.pack('>II', 1000, 3000)) +
            previous.box(b'hdlr', bytes(8) + b'vide'))))
        result, _ = self.analyse(data)
        self.assertEqual(result['gps']['status'], 'absent')
        self.assertEqual(result['status'], 'complete')

    def test_time_reference_and_edit_list_are_observations(self):
        result, _ = self.analyse(previous.timed_fixture(co64=True, signed_composition=True, edits=True))
        track = result['container_time_links'][1]
        self.assertEqual(track['exact_gps_sample_matches'], 3)
        self.assertIn('whole_gps_box', track['reference_kinds'])
        self.assertFalse(track['clock_observations']['edit_list_applied'])
        self.assertIn('edit_list', result['media']['tracks'][1])
        self.assertEqual(result['media']['tracks'][1]['sample_timing']['mean_playback_fps'], 1)

    def test_filename_is_not_published_or_assumed_to_mean_timelapse(self):
        result, _ = self.analyse(previous.fixture(count=1), '2020_0101_000000_001PF.MP4')
        self.assertTrue(result['filename_observation']['parking_marker'])
        self.assertNotIn('capture_fps', result['filename_observation'])
        self.assertIn('no_stationary_or_time_lapse_assumption', result['position_policy'])

    def test_inaccessible_input_has_stage_and_safe_error(self):
        with patch.object(Path, 'open', side_effect=PermissionError(13, 'PRIVATE', '/PRIVATE')):
            result, _ = analysis.analyse_file(Path('/PRIVATE/test.MP4'))
        self.assertEqual(result['error'], {'stage': 'open_input', 'reason': 'input_permission_denied'})
        self.assertNotIn('PRIVATE', json.dumps(result))

    def test_spatial_summary_does_not_leak_position_or_choose_truth(self):
        values = [{'position': (53.123456, 10.234567), 'speed': 0.0}] * 8
        values += [{'position': (-30.123456, 100.234567), 'speed': 0.0}]
        result = analysis.spatial_summary(values)
        text = json.dumps(result)
        self.assertNotIn('53.123456', text)
        self.assertNotIn('10.234567', text)
        self.assertEqual(result['exact_duplicate_positions'], 7)
        self.assertIn('not_application_spatial_validator', result['scope'])

    def test_cross_file_comparison_only_relative_and_not_identity(self):
        data = [
            ('file-001', {'utc': [1000, 1010], 'filename_time': 5000, 'channel': 'F'}),
            ('file-002', {'utc': [1001, 1011], 'filename_time': 5005, 'channel': 'R'}),
        ]
        result = analysis.compare_files(data)
        self.assertEqual(result['pairs'][0]['filename_start_difference_seconds'], 5)
        self.assertEqual(result['pairs'][0]['gps_wall_time_overlap_seconds'], 9)
        self.assertIn('not_proven', result['pairs'][0]['interpretation'])
        self.assertNotIn('5000', json.dumps(result))

    def test_bad_file_does_not_hide_good_file_in_same_run(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            media = root / 'PRIVATE-input'
            media.mkdir()
            good = media / '2020_0101_000000_001PF.MP4'
            good.write_bytes(previous.fixture(count=3))
            (media / 'bad.mp4').write_bytes(b'not a video')
            (media / 'unsupported.ts').write_bytes(b'transport stream')
            output = root / 'report.json'
            command = [sys.executable, str(HERE / 'diagnose.py'), '--set', 'unknown', str(media), '--output', str(output)]
            run = subprocess.run(command, capture_output=True, text=True, check=True)
            report = json.loads(output.read_text())
            statuses = [f['status'] for f in report['sets'][0]['files']]
            self.assertIn('complete', statuses)
            self.assertIn('partial', statuses)
            self.assertIn('unsupported', statuses)
            self.assertEqual(report['status'], 'partial')
            self.assertNotIn(str(root), output.read_text() + run.stdout + run.stderr)
            self.assertNotIn('PRIVATE', output.read_text() + run.stdout + run.stderr)
            self.assertEqual(good.read_bytes(), previous.fixture(count=3))
            before = output.read_bytes()
            retry = subprocess.run(command, capture_output=True, text=True)
            self.assertEqual(retry.returncode, 2)
            self.assertEqual(output.read_bytes(), before)

    def test_missing_group_does_not_stop_other_group(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            good = root / 'clip.MP4'
            good.write_bytes(previous.fixture(count=1))
            output = root / 'report.json'
            run = subprocess.run([sys.executable, str(HERE / 'diagnose.py'),
                '--set', '2ch-lowbitrate', str(root / 'missing'),
                '--set', '3ch-time2fps', str(good), '--output', str(output)],
                capture_output=True, text=True, check=True)
            report = json.loads(output.read_text())
            self.assertEqual(len(report['sets']), 2)
            self.assertEqual(report['sets'][0]['discovery']['reason'], 'input_file_not_found')
            self.assertEqual(report['sets'][1]['files'][0]['status'], 'complete')
            self.assertNotIn(str(root), run.stdout + run.stderr + output.read_text())


    def test_alignment_rule_has_only_relative_results(self):
        result = analysis.filename_alignment_summary(1000000, 1000000, 120)
        self.assertTrue(result['unique_under_current_rule'])
        self.assertEqual(result['possible_delay_seconds']['minimum'], 0)
        self.assertNotIn('1000000', json.dumps(result))
        self.assertNotIn('timezone', json.dumps(result))

    def test_duration_evidence_identifies_span_rule(self):
        result, _ = self.analyse(previous.fixture(count=119, step=15, duration=120))
        self.assertEqual(result['gps']['duration_bound_evidence_endpoints']['total_span'], 110)
        self.assertEqual(result['gps']['duration_bound_evidence_endpoints']['backwards_edge'], 0)

    def test_padded_blocks_and_window_boundary_keep_records(self):
        packet = previous.fixture(count=1)[-44:]
        body = previous.box(b'free', b'GPS ' + b'X' * 5000 + packet + b'X' * 4074)
        selected, active, candidates, partial = analysis.scan_block(io.BytesIO(body), 0, len(body), float('inf'))
        self.assertIsNotNone(selected)
        self.assertEqual(len(active), 1)
        self.assertEqual(len(candidates), 1)
        self.assertFalse(partial)

if __name__ == '__main__':
    unittest.main()
