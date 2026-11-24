#!/usr/bin/env python3
"""Focused report invariants and real-MP4 read-only integration checks."""
import hashlib
import importlib.util
import json
from pathlib import Path
import struct
import subprocess
import sys
import tempfile
import unittest

root = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('focus', root / 'diagnose-gps.py')
focus = importlib.util.module_from_spec(spec)
spec.loader.exec_module(focus)
BINARY = Path(sys.argv.pop(1)).resolve()
BASE = Path(sys.argv.pop(1)).read_bytes()


def point(t, lon=0, utc=None):
    return {'utc': utc, 'point': {'tOffsetS':t,'lat':0,'lon':lon,'fixOk':True,
        'speedMps':11.12,'headingDeg':90,'altitudeM':0}}


def box(kind, data):
    return struct.pack('>I', 8 + len(data)) + kind + data


def packet(second, lon):
    raw_lon = int(abs(lon)) * 100 + (abs(lon) % 1) * 60
    return struct.pack('<6I',0,0,second,26,8,31) + b'ANE\0' + struct.pack('<4f',0,raw_lon,11.12/0.514444,90)


def media(records):
    pos=0
    while pos<len(BASE):
        size,kind=struct.unpack('>I4s',BASE[pos:pos+8])
        if kind==b'moov': break
        pos+=size
    assert pos+size==len(BASE)
    frees=[box(b'free',b'GPS '+bytes(16)+record+bytes(32)) for record in records]
    gps_size=16+8*len(frees)
    offset=len(BASE)+gps_size
    pairs=[]
    for free in frees:
        pairs.append(struct.pack('>II',offset,len(free)));offset+=len(free)
    gps=box(b'gps ',bytes(8)+b''.join(pairs))
    return BASE[:pos]+box(b'moov',BASE[pos+8:]+gps)+b''.join(frees)


class FocusTests(unittest.TestCase):
    def test_exact_position_defaults_private(self):
        output=focus.describe([point(0,0),point(1,20)],'2026_0831_065829_F.MP4')
        self.assertGreater(output['largest_adjacent_distance_m'],2000000)
        self.assertEqual(output['largest_steps'][0]['to']['camera_time_estimate'],'2026-08-31T06:58:30.000')
        self.assertNotIn('latitude',json.dumps(output))
        self.assertNotIn('longitude',json.dumps(output))

    def test_coordinates_require_opt_in(self):
        output=focus.describe([point(0,0),point(1,20)],'2026_0831_065829_F.MP4',True)
        self.assertEqual(output['largest_steps'][0]['to']['longitude'],20)

    def test_compare_channels_compares_full_series(self):
        a=[point(0,0,'2026-08-31T00:00:00'),point(1,0.0001,'2026-08-31T00:00:01')]
        b=[point(0,0,'2026-08-31T00:00:00'),point(1,0.0002,'2026-08-31T00:00:01')]
        result=focus.compare_channels([{'file':'F','timed':a},{'file':'R','timed':b}])[0]
        self.assertFalse(result['identical_validated_decoded_series'])
        self.assertGreater(result['max_position_difference_at_matching_utc_m'],10)

    def test_empty_series_is_not_reported_as_proven_equal(self):
        result=focus.compare_channels([{'file':'F','timed':[]},{'file':'R','timed':[]}])[0]
        self.assertIsNone(result['identical_validated_decoded_series'])

    def test_ambiguous_anchor_is_rejected(self):
        with self.assertRaises(ValueError):
            focus.choose_groups([{'file':'same'},{'file':'same'}],'same')

    def test_missing_neighbor_is_rejected(self):
        with self.assertRaises(ValueError):
            focus.choose_groups([{'file':'same','path':'/media/same','group':'vf:0_N','start':'0','channel':'Front'}],'same')

    def test_removed_points_preserve_duplicate_counts(self):
        a=point(0,0)
        self.assertEqual(len(focus.removed([a,a],[a],'2026_0831_000000_F.MP4',False)),1)

    def test_end_to_end_removes_boundary_outlier_preserves_pause_and_inputs(self):
        with tempfile.TemporaryDirectory() as td:
            folder=Path(td)/'media';folder.mkdir()
            serial=0;files=[]
            for start in [0,5,25]:
                for channel in ['F','I','R']:
                    serial+=1
                    name=f'2026_0831_0000{start:02}_{serial:06}{channel}.MP4'
                    records=[packet(i,20 if start==5 and i==5 else i*0.0001) for i in range(start,start+5)]
                    path=folder/name;path.write_bytes(media(records));files.append(path)
            hashes={p:hashlib.sha256(p.read_bytes()).hexdigest() for p in files}
            report=Path(td)/'result.json'
            command=[sys.executable,str(root/'diagnose-gps.py'),str(folder),
                '--anchor','2026_0831_000005_000004F.MP4','--binary',str(BINARY),'--report',str(report)]
            subprocess.run(command,check=True,stdout=subprocess.DEVNULL)
            result=json.loads(report.read_text());middle=result['groups'][1]
            if sys.platform.startswith('linux'):
                self.assertEqual(report.stat().st_mode & 0o777,0o600)
            self.assertEqual(middle['selected_file'],'2026_0831_000005_000004F.MP4')
            self.assertGreater(middle['before_boundary']['largest_adjacent_distance_m'],2000000)
            self.assertLess(middle['after_boundary_and_duration_trim']['largest_adjacent_distance_m'],50)
            self.assertEqual(len(middle['points_removed_by_boundary']),1)
            self.assertEqual(middle['points_removed_by_boundary'][0]['video_offset_s'],0)
            self.assertEqual(result['boundaries_after'][1]['gps_utc_gap_s'],16)
            self.assertNotIn('latitude',report.read_text())
            self.assertNotIn('longitude',report.read_text())
            self.assertNotIn(str(folder),report.read_text())
            self.assertTrue(all(c['identical_validated_decoded_series'] for c in middle['channel_comparison']))
            self.assertEqual(len(list(folder.iterdir())),9)
            self.assertEqual(hashes,{p:hashlib.sha256(p.read_bytes()).hexdigest() for p in files})
            retry=subprocess.run(command,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
            self.assertNotEqual(retry.returncode,0)
            # Opt-in emits coordinates, under a separate never-overwritten report.
            opt=Path(td)/'coordinates.json'
            command[-1]=str(opt);command.append('--include-coordinates')
            subprocess.run(command,check=True,stdout=subprocess.DEVNULL)
            self.assertIn('latitude',opt.read_text())
            self.assertIn('longitude',opt.read_text())


if __name__=='__main__':
    unittest.main()
