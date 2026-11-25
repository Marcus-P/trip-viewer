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

if __name__=='__main__':
    unittest.main()
