"""Targeted capture/privacy checks; no original media or application build."""
import importlib.util
import json
import os
from pathlib import Path
import shutil
import struct
import subprocess
import sys
import tempfile
import unittest

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / 'parking-diagnostic'))
spec = importlib.util.spec_from_file_location('capture', HERE / 'capture.py')
capture = importlib.util.module_from_spec(spec)
spec.loader.exec_module(capture)


class CaptureTests(unittest.TestCase):
    def test_custom_names_values_and_known_private_values_stay_private(self):
        inventory = capture.Inventory({'GPSLatitude', 'RecordingMode'})
        raw = [{'Track-PRIVATE_MARKER:GPSLatitude': 41.123456,
                'XMP:PRIVATE_MARKER': 'private-location-value',
                'XMP:RecordingMode': 'TimeLapse'}]
        inventory.add('exiftool', raw, 'set-01', 'file-01')
        inventory.add('exiftool', [{'XMP:RecordingMode': 'LowBitrate'}], 'set-02', 'file-02')
        public, private = inventory.reports()
        text = json.dumps(public)
        for secret in ['PRIVATE_MARKER', '41.123456', 'private-location-value', 'TimeLapse', 'LowBitrate']:
            self.assertNotIn(secret, text)
        self.assertIn('GPSLatitude', text)
        self.assertIn('PRIVATE_MARKER', json.dumps(private))
        self.assertTrue(public[2]['constant_within_each_set_and_different_between_sets'])

    def test_unknown_blocks_are_saved_verbatim_without_leaking_types(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            block = struct.pack('>I4s', 8 + len(b'PRIVATE_MARKER'), b'abcd') + b'PRIVATE_MARKER'
            media = struct.pack('>I4s', 12, b'mdat') + b'data'
            source = root / 'input.mp4'
            source.write_bytes(block + media)
            output = root / 'raw'
            output.mkdir()
            result = capture.raw_blocks(source, output)
            self.assertEqual((output / 'block-000000.bin').read_bytes(), block)
            self.assertEqual(source.read_bytes(), block + media)
            self.assertEqual(result['saved_blocks'], 1)
            self.assertNotIn('abcd', json.dumps(result))
            self.assertFalse((output / 'block-000001.bin').exists())

    def test_limit_is_reported_not_silently_ignored(self):
        previous = capture.MAX_BLOCK
        try:
            capture.MAX_BLOCK = 8
            with tempfile.TemporaryDirectory() as temp:
                root = Path(temp)
                source = root / 'input.mp4'
                source.write_bytes(struct.pack('>I4s', 9, b'uuid') + b'x')
                result = capture.raw_blocks(source, root)
                self.assertEqual(result['status'], 'partial')
                self.assertEqual(result['limited_blocks'], 1)
        finally:
            capture.MAX_BLOCK = previous

    def test_tool_timeout_is_reported(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            result = capture.run_tool([sys.executable, '-c', 'import time;time.sleep(20)'],
                                      root / 'out', root / 'error', timeout=.05)
            self.assertEqual(result['status'], 'time_limit')

    @unittest.skipUnless(os.environ.get('EXIFTOOL_TEST') and shutil.which('ffprobe') and shutil.which('ffmpeg'),
                         'Set EXIFTOOL_TEST for the real-tool integration check.')
    def test_real_tools_keep_custom_values_local_and_refuse_overwrite(self):
        with tempfile.TemporaryDirectory(prefix='PRIVATE_MARKER-') as temp:
            root = Path(temp)
            input_dir = root / 'recordings'
            input_dir.mkdir()
            original = input_dir / 'PRIVATE_MARKER.mp4'
            subprocess.run(['ffmpeg', '-v', 'error', '-f', 'lavfi', '-i',
                            'color=size=32x32:rate=2:duration=1', '-c:v', 'libx264',
                            '-metadata', 'ParkingMode=TimeLapse', '-metadata',
                            'PRIVATE_MARKER=PRIVATE_MARKER_VALUE', '-movflags',
                            'use_metadata_tags', str(original)], check=True)
            before = original.read_bytes()
            # Corrupt sibling must not abort capture of the valid input.
            (input_dir / 'broken.mp4').write_bytes(b'not an MP4')
            output = root / 'results'
            command = [sys.executable, str(HERE / 'capture.py'), '--set', 'unknown',
                       str(input_dir), '--exiftool', os.environ['EXIFTOOL_TEST'],
                       '--ffprobe', shutil.which('ffprobe'), '--output-dir', str(output)]
            environment = dict(os.environ, PYTHONPATH=str(HERE.parent / 'parking-diagnostic'))
            result = subprocess.run(command, env=environment, capture_output=True, text=True)
            self.assertEqual(result.returncode, 1, result.stderr)
            report = (output / 'SHARE-metadata-report.json').read_text()
            self.assertEqual(json.loads(report)['status'], 'partial')
            self.assertNotIn('PRIVATE_MARKER', report + result.stdout + result.stderr)
            self.assertNotIn(str(root), report + result.stdout + result.stderr)
            raw_files = list((output / 'PRIVATE-DO-NOT-SHARE').glob('*/exiftool-PRIVATE.json'))
            self.assertTrue(any('PRIVATE_MARKER_VALUE' in path.read_text() for path in raw_files))
            self.assertTrue(any('TimeLapse' in path.read_text() for path in raw_files))
            self.assertEqual(original.read_bytes(), before)
            retry = subprocess.run(command, env=environment, capture_output=True, text=True)
            self.assertEqual(retry.returncode, 2)
            self.assertEqual((output / 'SHARE-metadata-report.json').read_text(), report)


if __name__ == '__main__':
    unittest.main()
