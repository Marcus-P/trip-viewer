#!/usr/bin/env python3
"""Read-only Novatek time diagnostics. Python 3.9+, standard library only.
No network, database writes, paths, filenames, hashes, coordinates, absolute
recording timestamps or device metadata in the JSON report. Counts and relative
timing remain technical recording characteristics: review before sharing.
Prepared with Codex for @Marcus-P. Decoder layout and temporal rules follow
chrisl8/trip-viewer PR21/091b392 under the repository MIT license; this independent Python
implementation does not run or certify the application's Rust pipeline.
"""
import argparse
import datetime as dt
import json
import math
from pathlib import Path
import struct
import time


class DiagnosticError(Exception):
    pass


def read_at(f, pos, length):
    f.seek(pos)
    value = f.read(length)
    if len(value) != length:
        raise DiagnosticError('truncated_input')
    return value


def boxes(f, start, end):
    while start + 8 <= end:
        size, kind = struct.unpack('>I4s', read_at(f, start, 8))
        header = 8
        if size == 1:
            size = struct.unpack('>Q', read_at(f, start + 8, 8))[0]
            header = 16
        elif size == 0:
            size = end - start
        if size < header or start + size > end:
            raise DiagnosticError('invalid_box_bounds')
        yield kind, start + header, start + size
        start += size


def video_duration(f, children):
    for kind, start, end in children:
        if kind != b'trak':
            continue
        for k, a, b in boxes(f, start, end):
            if k != b'mdia':
                continue
            media = list(boxes(f, a, b))
            handlers = [(x,y) for t,x,y in media if t == b'hdlr']
            if not handlers or handlers[0][1] - handlers[0][0] < 12:
                continue
            if read_at(f, handlers[0][0] + 8, 4) != b'vide':
                continue
            for t,x,y in media:
                if t != b'mdhd':
                    continue
                version = read_at(f, x, 1)[0]
                offset, fmt = (12, '>II') if version == 0 else (20, '>IQ')
                if version not in (0,1) or x + offset + struct.calcsize(fmt) > y:
                    raise DiagnosticError('unsupported_media_duration')
                scale, duration = struct.unpack(fmt, read_at(f, x + offset, struct.calcsize(fmt)))
                if not scale or duration == (2**(32 if version == 0 else 64)-1):
                    raise DiagnosticError('invalid_media_duration')
                return duration / scale
    raise DiagnosticError('no_video_duration')


def record_time(f, offset, size, deadline=float("inf")):
    if size < 12:
        return False, None
    head = read_at(f, offset, 12)
    if struct.unpack('>I', head[:4])[0] != size or head[4:] != b'freeGPS ':
        return False, None
    end = size - 12
    while end:
        if time.monotonic() > deadline:
            raise DiagnosticError('analysis_time_limit')
        start = max(0, end - 4096)
        buf = read_at(f, offset + 12 + start, end - start)
        # Match the application's last A/N-or-S/E-or-W marker rule.
        for marker in range(len(buf)-3, 23, -1):
            if buf[marker:marker+1] == b'A' and buf[marker+1] in b'NS' and buf[marker+2] in b'EW':
                pos = marker - 24
                if pos + 44 > len(buf):
                    return False, None
                hour, minute, second, year, month, day = struct.unpack_from('<6I', buf, pos)
                try:
                    stamp = dt.datetime(year+2000, month, day, hour, minute, second)
                    # Never serialized; only differences/counts leave memory.
                    return True, stamp.toordinal()*86400 + hour*3600 + minute*60 + second
                except (ValueError, OverflowError):
                    return True, None
        if start == 0:
            break
        end = start + 43
    return False, None


def classify(values, limit, deadline):
    states = ['missing' if v is None else 'consistent' for v in values]
    def active():
        if time.monotonic() > deadline:
            raise DiagnosticError('analysis_time_limit')
        return [i for i,s in enumerate(states) if s == 'consistent']
    while True:
        indices = active()
        bad = set()
        for a,b,c in zip(indices, indices[1:], indices[2:]):
            if values[a] <= values[c] and not values[a] <= values[b] <= values[c]:
                bad.add(b)
        if not bad:
            break
        for i in bad:
            states[i] = 'contradictory'
    while True:
        indices = active()
        bad = set()
        for a,b in zip(indices, indices[1:]):
            if values[a] > values[b] or (limit is not None and abs(values[a]-values[b]) > limit):
                bad.update((a,b))
        if len(indices)>1 and limit is not None and abs(values[indices[0]]-values[indices[-1]]) > limit:
            bad.update((indices[0], indices[-1]))
        if not bad:
            break
        for i in bad:
            states[i] = 'ambiguous'
    return {name: states.count(name) for name in ('missing','contradictory','ambiguous','consistent')}


def analyse(path):
    deadline = time.monotonic() + 60
    with path.open('rb') as f:
        f.seek(0, 2)
        length = f.tell()
        moov = next(((a,b) for k,a,b in boxes(f,0,length) if k == b'moov'), None)
        if moov is None:
            raise DiagnosticError('no_movie_box')
        children = list(boxes(f,*moov))
        duration = video_duration(f, children)
        gps = next(((a,b) for k,a,b in children if k == b'gps '), None)
        if gps is None:
            raise DiagnosticError('no_gps_descriptor_box')
        values = []
        descriptors = skipped = undecodable = 0
        for pos in range(gps[0]+8, gps[1]-7, 8):
            if time.monotonic() > deadline:
                raise DiagnosticError('analysis_time_limit')
            descriptors += 1
            if descriptors > 100000:
                raise DiagnosticError('descriptor_limit')
            offset, size = struct.unpack('>II', read_at(f,pos,8))
            if not offset or size < 12 or offset+size > length:
                skipped += 1
                continue
            decoded, stamp = record_time(f, offset, size, deadline)
            if decoded:
                values.append(stamp)
            else:
                undecodable += 1
    valid = [x for x in values if x is not None]
    adjacent = [(a,b) for a,b in zip(values,values[1:]) if a is not None and b is not None]
    limit = math.ceil(duration)
    return {
        'status': 'complete',
        'video_media_duration_seconds': round(duration,3),
        'duration_source': 'first_video_track_mdhd',
        'descriptors': descriptors, 'invalid_descriptors': skipped,
        'undecodable_records': undecodable, 'decoded_records': len(values),
        'valid_timestamp_records': len(valid),
        'gps_elapsed_span_seconds': max(valid)-min(valid) if valid else None,
        'adjacent_backwards_transitions': sum(b<a for a,b in adjacent),
        'adjacent_equal_timestamps': sum(b==a for a,b in adjacent),
        'adjacent_gaps_exceeding_video_duration': sum(abs(b-a)>limit for a,b in adjacent),
        'temporal_with_duration_bound': classify(values,limit,deadline),
        'temporal_without_duration_bound': classify(values,None,deadline),
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('files', nargs='+', type=Path)
    parser.add_argument('--output', type=Path, default=Path('viofo-time-report.json'))
    args = parser.parse_args()
    results = []
    for i,path in enumerate(args.files,1):
        try:
            result = analyse(path)
        except DiagnosticError as e:
            result = {'status':'incomplete','reason':str(e)}
        except OSError:
            result = {'status':'incomplete','reason':'input_read_error'}
        except Exception:
            result = {'status':'incomplete','reason':'unsupported_or_malformed_input'}
        result['file_id'] = f'file-{i:03}'
        results.append(result)
        print(result['file_id'] + ': ' + result['status'])
    report = {'format':'viofo-time-diagnostic-v1',
              'scope':'time fields only; independent Python reproduction, no spatial validation',
              'files':results}
    # Exclusive create avoids overwriting any input or pre-existing report.
    try:
        with args.output.open('x',encoding='utf-8') as f:
            json.dump(report,f,indent=2,allow_nan=False)
            f.write('\n')
    except OSError:
        parser.exit(2, 'Report not written: choose a new output file in a writable folder.\n')
    print('Report created locally. Review the JSON before sharing. Nothing was uploaded.')


if __name__ == '__main__':
    main()
