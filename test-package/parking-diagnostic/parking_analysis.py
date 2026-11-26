"""Read-only recording observations. No positions or absolute times are exported.

Prepared with Codex for @Marcus-P; uses the repository's MIT-licensed VIOFO
layout and the separately identified Python temporal-rule reproduction.
"""
from collections import Counter
import datetime as dt
import math
from pathlib import Path
import re
import statistics
import struct
import time

import time_core as core

MAX_TABLE = 16 * 1024 * 1024
MAX_RECORDS = 100_000
MAX_SAMPLES = 1_000_000
NAME = re.compile(r'^(\d{4})_(\d{4})_(\d{6})_(\d*)(P?)([FIR]?)$', re.I)


def check_deadline(deadline):
    if time.monotonic() > deadline:
        raise core.DiagnosticError('analysis_time_limit')


def clean_error(error):
    if isinstance(error, core.DiagnosticError):
        return str(error)
    if isinstance(error, OSError):
        return core.input_error_reason(error)
    return 'unsupported_or_malformed_structure'


def read_body(f, bounds):
    a, b = bounds
    if b - a > MAX_TABLE:
        raise core.DiagnosticError('metadata_table_limit')
    return core.read_at(f, a, b - a)


def children(f, bounds):
    return {k: (a, b) for k, a, b in core.boxes(f, *bounds)}


def timing_header(data):
    if not data or data[0] not in (0, 1):
        raise core.DiagnosticError('unsupported_time_header')
    position, form = (12, '>II') if data[0] == 0 else (20, '>IQ')
    scale, duration = struct.unpack_from(form, data, position)
    if not scale or duration == (2 ** (32 if data[0] == 0 else 64) - 1):
        raise core.DiagnosticError('invalid_time_header')
    return scale, duration / scale


def quantiles(values):
    if not values:
        return {'count': 0}
    ordered = sorted(values)
    return {
        'count': len(values), 'minimum': round(ordered[0], 6),
        'median': round(statistics.median(ordered), 6),
        'p90': round(ordered[round((len(ordered) - 1) * .9)], 6),
        'maximum': round(ordered[-1], 6),
    }


def time_stats(values):
    valid = [v for v in values if v is not None]
    differences = [b - a for a, b in zip(values, values[1:]) if a is not None and b is not None]
    common = Counter(differences)
    return {
        'records': len(values), 'valid': len(valid), 'missing': len(values) - len(valid),
        'elapsed_span_seconds': max(valid) - min(valid) if valid else None,
        'last_minus_first_seconds': valid[-1] - valid[0] if valid else None,
        'backward_steps': sum(d < 0 for d in differences),
        'equal_steps': sum(d == 0 for d in differences),
        'adjacent_delta_seconds': quantiles(differences),
        'most_common_delta_seconds': [
            {'delta': d, 'count': n} for d, n in common.most_common(8)
        ],
    }


def filename_info(path):
    match = NAME.fullmatch(path.stem)
    if not match:
        return {'recognized': False}, None
    year, day, clock, counter, parking, channel = match.groups()
    try:
        stamp = dt.datetime.strptime(year + day + clock, '%Y%m%d%H%M%S')
    except ValueError:
        return {'recognized': False}, None
    return {
        'recognized': True, 'parking_marker': bool(parking),
        'channel': channel.upper() or 'not_encoded',
        'sequence_present': bool(counter),
    }, stamp.toordinal() * 86400 + stamp.hour * 3600 + stamp.minute * 60 + stamp.second


def packet(buf, start):
    if start < 0 or start + 44 > len(buf):
        return None
    hour, minute, second, year, month, day = struct.unpack_from('<6I', buf, start)
    timestamp = None
    try:
        stamp = dt.datetime(year + 2000, month, day, hour, minute, second)
        timestamp = stamp.toordinal() * 86400 + hour * 3600 + minute * 60 + second
    except (ValueError, OverflowError):
        pass
    raw_lat, raw_lon, speed, heading = struct.unpack_from('<4f', buf, start + 28)

    def coordinate(raw, hemisphere, maximum):
        if not math.isfinite(raw) or raw < 0:
            return None
        degrees = math.floor(raw / 100)
        minutes = raw - degrees * 100
        if minutes < 0 or minutes >= 60 or degrees + minutes / 60 > maximum:
            return None
        value = degrees + minutes / 60
        return -value if hemisphere in (ord('S'), ord('W')) else value

    latitude = coordinate(raw_lat, buf[start + 25], 90)
    longitude = coordinate(raw_lon, buf[start + 26], 180)
    return {
        'utc': timestamp, 'active': buf[start + 24] == ord('A'),
        'position': (latitude, longitude) if latitude is not None and longitude is not None else None,
        'speed': speed * 0.5144444444444445 if math.isfinite(speed) and speed >= 0 else None,
        'heading_valid': math.isfinite(heading),
    }


def filename_alignment_summary(first_utc, filename_time, duration):
    """Reproduce candidate count, never publish inferred timezone or dates."""
    if first_utc is None or filename_time is None or duration is None:
        return {'status': 'unavailable'}
    candidates = []
    for minutes in range(-12 * 60, 14 * 60 + 1, 15):
        delta = first_utc + minutes * 60 - filename_time
        if -2 <= delta <= duration + 2:
            candidates.append(max(0, delta))
    return {
        'status': 'complete', 'candidate_count': len(candidates),
        'unique_under_current_rule': len(candidates) == 1,
        'possible_delay_seconds': quantiles(candidates),
        'scope': 'current_filename_clock_hypothesis_not_verified_mapping',
    }


def scan_block(f, offset, size, deadline):
    """Fixed-size reverse windows; retain original last-active-marker behavior."""
    head = core.read_at(f, offset, 12)
    if struct.unpack('>I', head[:4])[0] != size or head[4:] != b'freeGPS ':
        return None, [], [], False
    end = size - 12
    found = {}
    last_active = None
    last_active_seen = False
    partial_active = False
    while end > 0:
        check_deadline(deadline)
        start = max(0, end - 4096)
        buf = core.read_at(f, offset + 12 + start, end - start)
        for marker in range(len(buf) - 3, 23, -1):
            if buf[marker] not in b'AV' or buf[marker + 1] not in b'NS' or buf[marker + 2] not in b'EW':
                continue
            absolute = offset + 12 + start + marker - 24
            if absolute in found:
                continue
            value = packet(buf, marker - 24)
            if value is None:
                # Match the current reader: a trailing active marker whose
                # packet is truncated is not silently replaced by an earlier one.
                if buf[marker] == ord('A') and not last_active_seen:
                    last_active_seen = True
                    partial_active = True
                continue
            value['byte_position'] = absolute  # internal only
            found[absolute] = value
            if value['active'] and not last_active_seen:
                last_active = value
                last_active_seen = True
            if len(found) > MAX_RECORDS:
                raise core.DiagnosticError('record_limit')
        if start == 0:
            break
        end = start + 43
    ordered = [found[k] for k in sorted(found)]
    active = [v for v in ordered if v['active']]
    return last_active, active, ordered, partial_active


def distance(a, b):
    # Coordinates stay in memory. Never return the center or individual points.
    lat1, lon1, lat2, lon2 = map(math.radians, (*a, *b))
    term = math.sin((lat2 - lat1) / 2) ** 2 + math.cos(lat1) * math.cos(lat2) * math.sin((lon2 - lon1) / 2) ** 2
    return 6371008.8 * 2 * math.asin(math.sqrt(min(1, max(0, term))))


def spatial_summary(records):
    positions = [r['position'] for r in records if r['position'] is not None]
    speeds = [r['speed'] for r in records if r['speed'] is not None]
    result = {
        'scope': 'descriptive_only_not_application_spatial_validator_or_location_truth',
        'positions_valid': len(positions), 'positions_invalid': len(records) - len(positions),
        'exact_duplicate_positions': len(positions) - len(set(positions)),
        'speed_mps': quantiles(speeds),
    }
    if positions:
        center = (statistics.median(p[0] for p in positions), statistics.median(p[1] for p in positions))
        distances = [distance(center, position) for position in positions]
        result['distance_from_coordinate_median_metres'] = quantiles(distances)
        result['counts_within_radius_metres'] = {
            str(radius): sum(d <= radius for d in distances) for radius in (10, 25, 50, 100, 250)
        }
    return result


def table_summary(f, bounds, deadline):
    tracks = []
    movie = children(f, bounds)
    movie_scale = None
    overview = {'movie_header': {'status': 'unavailable'}, 'fragmented_movie': b'mvex' in movie}
    if b'mvhd' in movie:
        try:
            movie_scale, duration = timing_header(read_body(f, movie[b'mvhd']))
            overview['movie_header'] = {'status': 'complete', 'timescale': movie_scale, 'duration_seconds': round(duration, 6)}
        except (core.DiagnosticError, OSError, struct.error) as error:
            overview['movie_header']['reason'] = clean_error(error)
    for kind, start, end in core.boxes(f, *bounds):
        if kind != b'trak':
            continue
        track = {'track_id': f'track-{len(tracks) + 1:03}', 'status': 'partial'}
        tracks.append(track)
        try:
            check_deadline(deadline)
            parts = children(f, (start, end))
            media = children(f, parts[b'mdia'])
            handler = read_body(f, media[b'hdlr'])[8:12]
            track['handler'] = handler.decode() if handler in (b'vide', b'soun', b'meta', b'gps ', b'text', b'sbtl') else 'other'
            scale, duration = timing_header(read_body(f, media[b'mdhd']))
            track.update(timescale=scale, media_duration_seconds=round(duration, 6))
            if b'tkhd' in parts and movie_scale:
                data = read_body(f, parts[b'tkhd'])
                if data[0] in (0, 1):
                    value = struct.unpack_from('>I' if data[0] == 0 else '>Q', data, 20 if data[0] == 0 else 28)[0]
                    track['movie_track_duration_seconds'] = round(value / movie_scale, 6)
            if b'edts' in parts:
                edits = children(f, parts[b'edts'])
                if b'elst' in edits:
                    data = read_body(f, edits[b'elst'])
                    version, count = data[0], struct.unpack_from('>I', data, 4)[0]
                    if version not in (0, 1) or count > 10000:
                        raise core.DiagnosticError('unsupported_edit_list')
                    form = '>Iihh' if version == 0 else '>Qqhh'
                    width = struct.calcsize(form)
                    if 8 + count * width > len(data):
                        raise core.DiagnosticError('truncated_edit_list')
                    entries = [struct.unpack_from(form, data, 8 + i * width) for i in range(count)]
                    track['edit_list'] = {
                        'entries': count, 'empty_edits': sum(e[1] == -1 for e in entries),
                        'rates': sorted(set(round(e[2] + e[3] / 65536, 6) for e in entries)),
                        'movie_duration_seconds': round(sum(e[0] for e in entries) / movie_scale, 6) if movie_scale else None,
                        'media_offsets_seconds': quantiles([e[1] / scale for e in entries if e[1] != -1]),
                        'applied_to_gps_mapping': False,
                    }
            minimum = children(f, media[b'minf'])
            tables = children(f, minimum[b'stbl'])
            track['tables'] = [k.decode() for k in (b'stsz', b'stz2', b'stts', b'ctts', b'stsc', b'stco', b'co64', b'stss') if k in tables]
            if b'stsz' in tables:
                data = read_body(f, tables[b'stsz'])
                track['sample_count'] = struct.unpack_from('>I', data, 8)[0]
            if b'stts' in tables:
                data = read_body(f, tables[b'stts'])
                count = struct.unpack_from('>I', data, 4)[0]
                if count > MAX_SAMPLES or 8 + count * 8 > len(data):
                    raise core.DiagnosticError('invalid_time_to_sample_table')
                runs = [struct.unpack_from('>II', data, 8 + i * 8) for i in range(count)]
                samples = sum(n for n, _ in runs)
                ticks = sum(n * delta for n, delta in runs)
                track['sample_timing'] = {
                    'runs': count, 'samples': samples,
                    'zero_delta_samples': sum(n for n, d in runs if d == 0),
                    'delta_seconds': quantiles([delta / scale for _, delta in runs]),
                    'sum_duration_seconds': round(ticks / scale, 6),
                    'mean_playback_fps': round(samples * scale / ticks, 6) if ticks else None,
                    'is_capture_rate': False,
                }
            if b'ctts' in tables:
                data = read_body(f, tables[b'ctts'])
                track['composition_table_version'] = data[0]
            if b'stsd' in tables:
                data = read_body(f, tables[b'stsd'])
                if len(data) >= 16:
                    codec = data[12:16]
                    track['codec'] = codec.decode() if codec in (b'avc1', b'avc3', b'hvc1', b'hev1', b'mp4a', b'mp4v') else 'other'
            track['status'] = 'complete'
        except (core.DiagnosticError, OSError, KeyError, struct.error) as error:
            track['reason'] = clean_error(error)
    overview['tracks'] = tracks
    overview['audio_track_present'] = any(t.get('handler') == 'soun' for t in tracks)
    return overview


def clock_links(f, movie_children, blocks, deadline):
    references = {}
    stamps = {}

    def add(key, kind, stamp):
        references.setdefault(key, set()).add(kind)
        if stamp is not None:
            stamps.setdefault(key, set()).add(stamp)

    for offset, size, selected, candidates in blocks:
        stamp = selected['utc'] if selected else None
        for shift, kind in ((0, 'whole_gps_box'), (8, 'free_box_body'), (12, 'gps_payload')):
            if size > shift:
                add((offset + shift, size - shift), kind, stamp)
        for value in candidates:
            add((value['byte_position'], 44), 'decoded_record_44_bytes', value['utc'])
    observed = {}

    def matched(track, key, pts):
        values = stamps.get(key, set())
        if len(values) == 1:
            observed.setdefault(track, []).append((pts, next(iter(values))))

    tracks = core.container_time_links(f, movie_children, references, deadline, matched)
    for track in tracks:
        track['scope'] = 'exact_byte_range_reference_not_content_synchronization_proof'
        pairs = observed.get(track.get('track_id'), [])
        # Only relative clock relationships leave memory; no raw PTS/UTC pairs.
        if pairs:
            ratios = [(b[1] - a[1]) / (b[0] - a[0]) for a, b in zip(pairs, pairs[1:]) if b[0] > a[0]]
            track['clock_observations'] = {
                'pairs': len(pairs),
                'gps_span_seconds': max(t for _, t in pairs) - min(t for _, t in pairs),
                'gps_seconds_per_media_second': quantiles(ratios),
                'constant_rate_assumed': False,
                'edit_list_applied': False,
            }
    return tracks


def analyse_file(path):
    deadline = time.monotonic() + 120
    name, filename_stamp = filename_info(path)
    output = {
        'status': 'partial', 'filename_observation': name,
        'position_policy': 'observed_only_no_stationary_or_time_lapse_assumption',
        'content_limits': ['burned_in_text_not_read', 'video_frames_not_decoded',
                           'codec_sei_not_decoded', 'parking_submode_not_inferred'],
    }
    internal = {'filename_time': filename_stamp, 'channel': name.get('channel'), 'utc': []}
    stage = 'open_input'
    try:
        with path.open('rb') as f:
            f.seek(0, 2)
            length = f.tell()
            stage = 'top_level_boxes'
            top = list(core.boxes(f, 0, length))
            movie = next(((a, b) for k, a, b in top if k == b'moov'), None)
            output['container'] = {
                'movie_box_present': movie is not None,
                'fragment_boxes_present': any(k == b'moof' for k, _, _ in top),
            }
            if movie is None:
                raise core.DiagnosticError('no_movie_box')
            stage = 'track_metadata'
            metadata = table_summary(f, movie, deadline)
            output['media'] = metadata
            movie_children = list(core.boxes(f, *movie))
            duration = None
            try:
                duration = core.video_duration(f, movie_children)
                output['video_media_duration_seconds'] = round(duration, 6)
            except (core.DiagnosticError, OSError, struct.error) as error:
                output['video_duration_error'] = clean_error(error)
            gps = next(((a, b) for k, a, b in movie_children if k == b'gps '), None)
            if gps is None:
                output.update(status='complete', gps={'status': 'absent', 'interpretation': 'no_gps_descriptor_list_not_proof_gps_setting_off'})
                return output, internal
            stage = 'gps_descriptors'
            pairs = []
            invalid = 0
            for position in range(gps[0] + 8, gps[1] - 7, 8):
                check_deadline(deadline)
                if len(pairs) + invalid >= MAX_RECORDS:
                    raise core.DiagnosticError('descriptor_limit')
                offset, size = struct.unpack('>II', core.read_at(f, position, 8))
                if offset == 0 or size < 12 or offset + size > length:
                    invalid += 1
                    continue
                pairs.append((offset, size))
            gps_output = {
                'status': 'partial', 'valid_descriptors': len(pairs),
                'invalid_descriptors': invalid,
                'duplicate_descriptor_references': len(pairs) - len(set(pairs)),
                'descriptor_offset_backward_steps': sum(b[0] < a[0] for a, b in zip(pairs, pairs[1:])),
            }
            output['gps'] = gps_output
            chosen = []
            all_records = {}
            blocks = []
            incomplete_markers = 0
            per_block = []
            failures = Counter()
            for offset, size in pairs:
                stage = 'gps_record_blocks'
                check_deadline(deadline)
                try:
                    selected, active, candidates, partial = scan_block(f, offset, size, deadline)
                except (core.DiagnosticError, OSError, struct.error) as error:
                    failures[clean_error(error)] += 1
                    continue
                incomplete_markers += int(partial)
                per_block.append(len(candidates))
                if selected is not None:
                    chosen.append(selected)
                for value in candidates:
                    all_records[value['byte_position']] = value
                if len(all_records) > MAX_RECORDS:
                    raise core.DiagnosticError('record_limit')
                blocks.append((offset, size, selected, candidates))
            ordered = [all_records[k] for k in sorted(all_records)]
            selected_times = [v['utc'] for v in chosen]
            physical_active = [v for v in ordered if v['active']]
            gps_output.update({
                'status': 'partial' if failures else 'complete',
                'block_errors': dict(failures),
                'truncated_last_active_markers': incomplete_markers,
                'candidate_records_per_block': quantiles(per_block),
                'active_record_candidates': len(physical_active),
                'inactive_record_candidates': sum(not v['active'] for v in ordered),
                'selected_record_times': time_stats(selected_times),
                'physical_order_all_active_times': time_stats([v['utc'] for v in physical_active]),
                'selected_position_statistics': spatial_summary(chosen),
                'all_active_position_statistics': spatial_summary(physical_active),
                'selection_rule': 'last_active_marker_per_descriptor_as_current_reader',
                'fix_status': {
                    'inactive_with_valid_time': sum(not v['active'] and v['utc'] is not None for v in ordered),
                    'candidates_before_first_active': next((i for i, v in enumerate(ordered) if v['active']), None),
                    'first_active_after_first_dated_candidate_seconds': (
                        next(v['utc'] for v in ordered if v['active'] and v['utc'] is not None)
                        - next(v['utc'] for v in ordered if v['utc'] is not None)
                        if any(v['active'] and v['utc'] is not None for v in ordered) else None),
                    'delay_clock': 'gps_wall_time_not_video_time',
                },
                'candidate_warning': 'marker_matches_are_not_independent_proof_of_record_authenticity',
            })
            internal['utc'] = [t for t in selected_times if t is not None]
            stage = 'temporal_rule_comparison'
            try:
                comparisons = {'without_media_duration_bound': core.classify(selected_times, None, deadline)}
                if duration is not None:
                    counts, states, evidence = core.classify(selected_times, math.ceil(duration), deadline, details=True)
                    comparisons['with_media_duration_bound'] = counts
                    gps_output['duration_bound_evidence_endpoints'] = evidence
                    filtered_times = [t for t, state in zip(selected_times, states) if state == 'consistent']
                    gps_output['filename_alignment_before_temporal_filter'] = filename_alignment_summary(
                        next((t for t in selected_times if t is not None), None), filename_stamp, duration)
                    gps_output['filename_alignment_after_temporal_filter'] = filename_alignment_summary(
                        filtered_times[0] if filtered_times else None, filename_stamp, duration)
                    gps_output['validity_vs_mapping_warning'] = (
                        'removing_duration_bound_does_not_establish_media_time_mapping'
                    )
                    valid = internal['utc']
                    gps_output['utc_from_first_valid_exceeds_video_duration'] = (
                        sum(t - valid[0] > duration for t in valid) if valid else 0
                    )
                gps_output['temporal_rules'] = comparisons
            except core.DiagnosticError as error:
                gps_output['temporal_comparison_error'] = clean_error(error)
            stage = 'explicit_sample_reference_check'
            try:
                output['container_time_links'] = clock_links(f, movie_children, blocks, deadline)
            except (core.DiagnosticError, OSError, struct.error) as error:
                output['container_time_links'] = [{'status': 'unavailable', 'stage': stage, 'reason': clean_error(error)}]
            output['status'] = 'partial' if failures else 'complete'
            return output, internal
    except (core.DiagnosticError, OSError, KeyError, struct.error) as error:
        output['error'] = {'stage': stage, 'reason': clean_error(error)}
        return output, internal


def compare_files(items):
    """Candidate overlap only, never inferred recording identity or grouping."""
    result = []
    evaluated = 0
    for i, (file_a, a) in enumerate(items):
        for file_b, b in items[i + 1:]:
            if not a['utc'] or not b['utc'] or a.get('channel') == b.get('channel'):
                continue
            evaluated += 1
            if evaluated > 20_000:
                return {'status': 'partial', 'reason': 'pair_limit', 'pairs': result}
            begin, end = max(min(a['utc']), min(b['utc'])), min(max(a['utc']), max(b['utc']))
            if end < begin:
                continue
            pair = {
                'file_a': file_a, 'file_b': file_b,
                'gps_wall_time_overlap_seconds': end - begin,
                'first_gps_time_difference_seconds': b['utc'][0] - a['utc'][0],
                'interpretation': 'overlapping_gps_clock_ranges_not_proven_simultaneous_content',
            }
            if a['filename_time'] is not None and b['filename_time'] is not None:
                pair['filename_start_difference_seconds'] = b['filename_time'] - a['filename_time']
            result.append(pair)
    return {'status': 'complete', 'pairs': result}
