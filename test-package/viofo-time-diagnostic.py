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


def container_time_links(f, children, references, deadline):
    """Check exact whole-GPS-block references in standard timed MP4 samples.

    Byte proximity to video samples is not accepted as a timing link. Output
    contains counts and elapsed spans only. Edit-list application and metadata
    formats referencing only part of a GPS block require separate analysis.
    """
    maximum = 1_000_000
    f.seek(0, 2)
    file_length = f.tell()

    def body(start, end):
        if end - start > 16 * 1024 * 1024:
            raise DiagnosticError('timing_table_limit')
        return read_at(f, start, end - start)

    def table(start, end, width):
        data = body(start, end)
        if len(data) < 8:
            raise DiagnosticError('invalid_timing_table')
        count = struct.unpack_from('>I', data, 4)[0]
        if count > maximum or 8 + count * width > len(data):
            raise DiagnosticError('invalid_timing_table')
        return data, count

    def expand(data, count, signed=False):
        out = []
        for i in range(count):
            if time.monotonic() > deadline:
                raise DiagnosticError('analysis_time_limit')
            n, value = struct.unpack_from('>Ii' if signed else '>II', data, 8 + i * 8)
            if len(out) + n > maximum:
                raise DiagnosticError('sample_limit')
            out.extend([value] * n)
        return out

    result = []
    for kind, start, end in children:
        if kind != b'trak':
            continue
        info = {
            'track_id': f'track-{len(result) + 1:03}',
            'status': 'unavailable',
        }
        result.append(info)
        try:
            track = {k: (a, b) for k, a, b in boxes(f, start, end)}
            edits = track.get(b'edts')
            info['edit_list_present'] = bool(
                edits and any(k == b'elst' for k, _, _ in boxes(f, *edits))
            )
            media = {k: (a, b) for k, a, b in boxes(f, *track[b'mdia'])}
            handler = body(*media[b'hdlr'])[8:12]
            known = (b'vide', b'soun', b'meta', b'gps ', b'text', b'sbtl')
            info['handler'] = handler.decode('ascii') if handler in known else 'other'
            mdhd = body(*media[b'mdhd'])
            if not mdhd or mdhd[0] not in (0, 1):
                raise DiagnosticError('unsupported_media_duration')
            scale = struct.unpack_from('>I', mdhd, 12 if mdhd[0] == 0 else 20)[0]
            if not scale:
                raise DiagnosticError('invalid_timescale')
            minf = {k: (a, b) for k, a, b in boxes(f, *media[b'minf'])}
            tables = {k: (a, b) for k, a, b in boxes(f, *minf[b'stbl'])}
            sizes_data = body(*tables[b'stsz'])
            fixed, count = struct.unpack_from('>II', sizes_data, 4)
            if count > maximum or (not fixed and len(sizes_data) < 12 + 4 * count):
                raise DiagnosticError('sample_limit')
            sizes = [fixed] * count if fixed else list(
                struct.unpack_from(f'>{count}I', sizes_data, 12)
            )
            offsets_kind = b'stco' if b'stco' in tables else b'co64'
            width = 4 if offsets_kind == b'stco' else 8
            data, n = table(*tables[offsets_kind], width)
            offsets = struct.unpack_from(f'>{n}{"I" if width == 4 else "Q"}', data, 8)
            data, n = table(*tables[b'stsc'], 12)
            chunks = [struct.unpack_from('>III', data, 8 + i * 12) for i in range(n)]
            if not chunks or chunks[0][0] != 1 or any(
                b[0] <= a[0] for a, b in zip(chunks, chunks[1:])
            ):
                raise DiagnosticError('invalid_sample_chunks')
            if any(not per_chunk or not description for _, per_chunk, description in chunks):
                raise DiagnosticError('invalid_sample_chunks')
            if chunks[-1][0] > len(offsets):
                raise DiagnosticError('sample_chunk_count_mismatch')
            data, n = table(*tables[b'stts'], 8)
            durations = expand(data, n)
            composition = [0] * count
            if b'ctts' in tables:
                data, n = table(*tables[b'ctts'], 8)
                if data[0] not in (0, 1):
                    raise DiagnosticError('unsupported_composition_times')
                composition = expand(data, n, signed=data[0] == 1)
            if len(durations) != count or len(composition) != count:
                raise DiagnosticError('sample_time_count_mismatch')
            sample = run = dts = 0
            matched = set()
            pts = []
            for chunk_index, offset in enumerate(offsets, 1):
                if time.monotonic() > deadline:
                    raise DiagnosticError('analysis_time_limit')
                while run + 1 < len(chunks) and chunks[run + 1][0] <= chunk_index:
                    run += 1
                per_chunk = chunks[run][1]
                if sample + per_chunk > count:
                    raise DiagnosticError('sample_chunk_count_mismatch')
                for _ in range(per_chunk):
                    if offset + sizes[sample] > file_length:
                        raise DiagnosticError('sample_outside_file')
                    key = (offset, sizes[sample])
                    if key in references:
                        matched.add(key)
                        pts.append((dts + composition[sample]) / scale)
                    offset += sizes[sample]
                    dts += durations[sample]
                    sample += 1
            if sample != count:
                raise DiagnosticError('sample_chunk_count_mismatch')
            info.update(
                status='complete', sample_count=count,
                exact_gps_sample_matches=len(matched),
                matched_media_pts_span_seconds=round(max(pts) - min(pts), 6) if pts else None,
                matched_media_pts_backwards=sum(b < a for a, b in zip(pts, pts[1:])),
            )
        except DiagnosticError as error:
            info['reason'] = str(error)
        except (KeyError, struct.error):
            info['reason'] = 'missing_or_unsupported_sample_tables'
    return result


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
        references = set()
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
            references.add((offset, size))
            decoded, stamp = record_time(f, offset, size, deadline)
            if decoded:
                values.append(stamp)
            else:
                undecodable += 1
        timing_links = container_time_links(f, children, references, deadline)
    valid = [x for x in values if x is not None]
    adjacent = [(a,b) for a,b in zip(values,values[1:]) if a is not None and b is not None]
    limit = math.ceil(duration)
    return {
        'status': 'complete',
        'video_media_duration_seconds': round(duration,3),
        'duration_source': 'first_video_track_mdhd',
        'container_time_links': timing_links,
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
    report = {'format':'viofo-time-diagnostic-v2',
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
