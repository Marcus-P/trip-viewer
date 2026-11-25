import datetime as dt
import importlib.util
import io
import json
from pathlib import Path
import struct
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('diagnostic', Path(__file__).with_name('viofo-time-diagnostic.py'))
d = importlib.util.module_from_spec(spec)
spec.loader.exec_module(d)

def box(kind,body):
    return struct.pack('>I4s',len(body)+8,kind)+body

def fixture(count=180,step=1,duration=180,invalid=False):
    packets=[]
    for i in range(count):
        stamp=dt.datetime(2020,1,1)+dt.timedelta(seconds=i*step)
        packet=bytearray(44)
        struct.pack_into('<6I',packet,0,stamp.hour,stamp.minute,stamp.second,20,1,0 if invalid else 1)
        packet[24:27]=b'ANE'
        packets.append(box(b'free',b'GPS '+packet))
    mdhd=box(b'mdhd',bytes(12)+struct.pack('>II',1000,duration*1000))
    track=box(b'trak',box(b'mdia',mdhd+box(b'hdlr',bytes(8)+b'vide')))
    placeholder=box(b'moov',track+box(b'gps ',bytes(8+8*count)))
    offset=len(placeholder)
    descriptors=b''
    for packet in packets:
        descriptors+=struct.pack('>II',offset,len(packet))
        offset+=len(packet)
    return box(b'moov',track+box(b'gps ',bytes(8)+descriptors))+b''.join(packets)

def timed_fixture(co64=False, signed_composition=False, body_only=False, edits=False):
    def metadata(offsets):
        size = 48 if body_only else 56
        sizes = box(b'stsz', bytes(4) + struct.pack('>II', size, 3))
        chunks = box(b'stsc', bytes(4) + struct.pack('>4I', 1, 1, 1, 1))
        times = box(b'stts', bytes(4) + struct.pack('>3I', 1, 3, 1000))
        fmt = '>3Q' if co64 else '>3I'
        positions = box(b'co64' if co64 else b'stco', bytes(4) + struct.pack('>I', 3) + struct.pack(fmt, *offsets))
        composition = b''
        if signed_composition:
            composition = box(b'ctts', bytes([1, 0, 0, 0]) + struct.pack('>I', 3) + b''.join(struct.pack('>Ii', 1, v) for v in [-1000, 0, 0]))
        mdhd = box(b'mdhd', bytes(12) + struct.pack('>II', 1000, 3000))
        media = box(b'mdia', mdhd + box(b'hdlr', bytes(8) + b'meta') + box(b'minf', box(b'stbl', sizes + chunks + times + positions + composition)))
        edit = box(b'edts', box(b'elst', bytes(4) + struct.pack('>IIiHH', 1, 3000, 0, 1, 0))) if edits else b''
        return box(b'trak', edit + media)
    original = fixture(count=3, duration=3)
    movie_end = struct.unpack_from('>I', original)[0]
    chunks = list(d.boxes(io.BytesIO(original), 8, movie_end))
    video = original[chunks[0][1]-8:chunks[0][2]]
    gps = box(b'gps ', bytes(8+24))
    movie = box(b'moov', video + metadata([0]*3) + gps)
    offsets = [len(movie) + i*56 for i in range(3)]
    gps = box(b'gps ', bytes(8) + b''.join(struct.pack('>II', offset, 56) for offset in offsets))
    actual = [offset+8 if body_only else offset for offset in offsets]
    return box(b'moov', video + metadata(actual) + gps) + original[movie_end:]

class Checks(unittest.TestCase):
    def analyse(self,data):
        with tempfile.TemporaryDirectory() as root:
            path=Path(root)/'PRIVATE-person-date-device.mp4'
            path.write_bytes(data)
            result=d.analyse(path)
            self.assertEqual(path.read_bytes(),data)
            self.assertNotIn('PRIVATE',json.dumps(result))
            self.assertNotIn('2020',json.dumps(result))
            return result
    def test_regular_clip_and_privacy(self):
        r=self.analyse(fixture())
        self.assertEqual(r['decoded_records'],180)
        self.assertEqual(r['temporal_with_duration_bound']['consistent'],180)
    def test_duration_bound_reproduces_110(self):
        r=self.analyse(fixture(count=290))
        self.assertEqual(r['temporal_with_duration_bound']['ambiguous'],110)
        self.assertEqual(r['temporal_without_duration_bound']['ambiguous'],0)
    def test_invalid_dates_are_distinct(self):
        r=self.analyse(fixture(count=10,invalid=True))
        self.assertEqual(r['temporal_with_duration_bound']['missing'],10)
        self.assertEqual(r['temporal_with_duration_bound']['ambiguous'],0)
    def test_backward_times(self):
        r=d.classify([10,11,0,1],None,float('inf'))
        self.assertGreater(r['ambiguous']+r['contradictory'],0)
    def test_padded_record_across_window(self):
        packet=fixture(count=1)[-44:]
        payload=b'X'*5000+packet+b'X'*4074
        body=box(b'free',b'GPS '+payload)
        found,stamp=d.record_time(io.BytesIO(body),0,len(body))
        self.assertTrue(found)
        self.assertIsNotNone(stamp)
    def test_box_bounds_rejected(self):
        with self.assertRaises(d.DiagnosticError):
            list(d.boxes(io.BytesIO(struct.pack('>I4s',100,b'moov')),0,8))
    def test_analysis_limit_is_explicit(self):
        with self.assertRaises(d.DiagnosticError):
            d.classify([1,2],3,0)

    def test_exact_timed_metadata_reference(self):
        r = self.analyse(timed_fixture())
        track = r['container_time_links'][1]
        self.assertEqual(track['status'], 'complete')
        self.assertEqual(track['exact_gps_sample_matches'], 3)
        self.assertEqual(track['matched_media_pts_span_seconds'], 2)

    def test_64bit_offsets_and_signed_composition(self):
        track = self.analyse(timed_fixture(co64=True, signed_composition=True))['container_time_links'][1]
        self.assertEqual(track['status'], 'complete')
        self.assertEqual(track['exact_gps_sample_matches'], 3)
        self.assertEqual(track['matched_media_pts_span_seconds'], 3)

    def test_partial_block_not_claimed_as_exact_link(self):
        track = self.analyse(timed_fixture(body_only=True))['container_time_links'][1]
        self.assertEqual(track['status'], 'complete')
        self.assertEqual(track['exact_gps_sample_matches'], 0)

    def test_edit_list_flag_no_false_mapping_claim(self):
        track = self.analyse(timed_fixture(edits=True))['container_time_links'][1]
        self.assertTrue(track['edit_list_present'])
        self.assertEqual(track['matched_media_pts_span_seconds'], 2)

    def test_missing_tables_explicitly_unavailable(self):
        track = self.analyse(fixture(count=3))['container_time_links'][0]
        self.assertEqual(track['status'], 'unavailable')
        self.assertNotIn('exact_gps_sample_matches', track)

if __name__=='__main__':
    unittest.main()
