#!/usr/bin/env python3
"""Locate GPS jumps in one recording and replay the 0.13.0 neighboring-file checks."""
import argparse
from collections import Counter, defaultdict
from datetime import datetime, timedelta
import importlib.util
from itertools import combinations
import json
import math
import os
from pathlib import Path
import tempfile

spec = importlib.util.spec_from_file_location('comparison', Path(__file__).with_name('run-compare.py'))
comparison = importlib.util.module_from_spec(spec)
spec.loader.exec_module(comparison)


def utc_time(text):
    return datetime.fromisoformat(text.rstrip('Z')) if text else None


def camera_time(file, offset):
    try:
        prefix = '_'.join(file.split('_')[:3])
        start = datetime.strptime(prefix, '%Y_%m%d_%H%M%S')
        return (start + timedelta(seconds=offset)).isoformat(timespec='milliseconds')
    except (ValueError, TypeError, OverflowError):
        return None


def usable(item):
    p = item['point']
    return p.get('fixOk') and all(isinstance(p.get(k), (int, float)) and math.isfinite(p[k])
        for k in ['lat', 'lon', 'tOffsetS']) and abs(p['lat']) <= 90 and abs(p['lon']) <= 180


def point_info(item, index, file, coordinates):
    point = item['point']
    value = {'decoded_point_number': index + 1, 'video_offset_s': point['tOffsetS'],
        'camera_time_estimate': camera_time(file, point['tOffsetS']),
        'gps_utc': item.get('utc'), 'fix_ok': point['fixOk']}
    if coordinates:
        value['latitude'] = point['lat']
        value['longitude'] = point['lon']
    return value


def describe(timed, file, coordinates=False):
    points = [(i, item) for i, item in enumerate(timed) if usable(item)]
    jumps = []
    for (ai, a), (bi, b) in zip(points, points[1:]):
        gap = b['point']['tOffsetS'] - a['point']['tOffsetS']
        metres = comparison.distance_km(a['point'], b['point']) * 1000
        jump = {'distance_m': round(metres, 3), 'video_offset_gap_s': gap,
            'from': point_info(a, ai, file, coordinates),
            'to': point_info(b, bi, file, coordinates), '_from_index': ai, '_to_index': bi}
        if a.get('utc') and b.get('utc'):
            jump['gps_utc_gap_s'] = (utc_time(b['utc']) - utc_time(a['utc'])).total_seconds()
        jumps.append(jump)
    jumps.sort(key=lambda j: j['distance_m'], reverse=True)
    top = jumps[:3]
    context = []
    if top:
        first, last = top[0]['_from_index'], top[0]['_to_index']
        context = [point_info(timed[i], i, file, coordinates)
            for i in range(max(0, first - 2), min(len(timed), last + 3))]
    for jump in top:
        jump.pop('_from_index')
        jump.pop('_to_index')
    return {'point_count': len(timed), 'valid_point_count': len(points),
        'largest_adjacent_distance_m': top[0]['distance_m'] if top else None,
        'largest_steps': top, 'context_around_largest_step': context}


def removed(before, after, file, coordinates):
    counts = Counter(json.dumps(p, sort_keys=True) for p in after)
    result = []
    for i, item in enumerate(before):
        key = json.dumps(item, sort_keys=True)
        if counts[key]:
            counts[key] -= 1
        else:
            result.append(point_info(item, i, file, coordinates))
    return result


def compare_channels(candidates):
    result = []
    for a, b in combinations(candidates, 2):
        left, right = a['timed'], b['timed']
        by_time = {p['utc']: p for p in left if usable(p)}
        distances = [comparison.distance_km(by_time[p['utc']]['point'], p['point']) * 1000
            for p in right if usable(p) and p['utc'] in by_time]
        result.append({'files': [a['file'], b['file']],
            'identical_validated_decoded_series': left == right if left and right else None,
            'matching_gps_utc_records': len(distances),
            'max_position_difference_at_matching_utc_m': round(max(distances), 3) if distances else None})
    return result


def boundaries(segments, key, coordinates):
    result = []
    for left, right in zip(segments, segments[1:]):
        a, b = left[key], right[key]
        if not a['timed'] or not b['timed']:
            result.append({'from_file': a['file'], 'to_file': b['file'], 'status': 'missing_gps'})
            continue
        pa, pb = a['timed'][-1], b['timed'][0]
        result.append({'from_file': a['file'], 'to_file': b['file'], 'status': 'available',
            'gps_utc_gap_s': (utc_time(pb['utc']) - utc_time(pa['utc'])).total_seconds(),
            'distance_m': round(comparison.distance_km(pa['point'], pb['point']) * 1000, 3),
            'from': point_info(pa, len(a['timed']) - 1, a['file'], coordinates),
            'to': point_info(pb, 0, b['file'], coordinates)})
    return result


def choose_groups(items, anchor):
    anchors = [item for item in items if item['file'] == anchor]
    if len(anchors) != 1:
        raise ValueError(f'Expected exactly one anchor file, found {len(anchors)}. Use a narrower input folder.')
    anchor = anchors[0]
    parent = Path(anchor['path']).parent
    mode = anchor['group'].rsplit('_', 1)[-1]
    groups = defaultdict(list)
    for item in items:
        if Path(item['path']).parent == parent and item['group'].rsplit('_', 1)[-1] == mode:
            groups[item['group']].append(item)
    ordered = sorted(groups.values(), key=lambda group: group[0]['start'])
    index = next(i for i, group in enumerate(ordered) if any(p['path'] == anchor['path'] for p in group))
    if index == 0 or index + 1 == len(ordered):
        raise ValueError('Need the preceding and following recording in the same directory and recording mode.')
    chosen = ordered[index - 1:index + 2]
    rank = {'Front': 0, 'Interior': 1, 'Rear': 2}
    for group in chosen:
        if len({p['channel'] for p in group}) != len(group):
            raise ValueError('Ambiguous duplicate channel in a recording group; use a narrower input folder.')
        group.sort(key=lambda p: rank[p['channel']])
    return chosen


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('folder', type=Path)
    parser.add_argument('--anchor', required=True, help='Exact MP4 basename to investigate')
    parser.add_argument('--report', type=Path, required=True)
    parser.add_argument('--include-coordinates', action='store_true', help='Opt in to exact route coordinates in the saved report')
    parser.add_argument('--binary', type=Path, default=Path(__file__).with_name('viofo-compare' + ('.exe' if os.name == 'nt' else '')))
    args = parser.parse_args()
    if not args.folder.is_dir() or not args.binary.is_file():
        parser.error('Input folder or worker binary not found.')
    if args.report.exists():
        parser.error('Report already exists; choose another report filename.')
    if not args.report.parent.is_dir():
        parser.error('Report output folder does not exist.')
    paths = sorted(str(p.resolve()) for p in args.folder.rglob('*')
        if p.is_file() and not p.is_symlink() and p.suffix.lower() == '.mp4')
    if not paths:
        parser.error('No MP4 files found.')
    binary = args.binary.resolve()
    with tempfile.TemporaryDirectory(prefix='viofo-focus-') as temporary:
        manifest = Path(temporary) / 'manifest.json'
        manifest.write_text(json.dumps(paths))
        catalog = comparison.worker(binary, 'catalog', manifest)
        if catalog['status'] != 'ok':
            parser.error('Filename catalog failed: ' + str(catalog))
        try:
            chosen = choose_groups(catalog['items'], args.anchor)
        except ValueError as error:
            parser.error(str(error))
        manifest.write_text(json.dumps({'groups': [[p['path'] for p in group] for group in chosen]}))
        print('Reading these three recording groups (originals stay unchanged):', flush=True)
        for i, group in enumerate(chosen):
            print(('PREVIOUS', 'FOCUS', 'NEXT')[i] + ': ' + ', '.join(p['file'] for p in group), flush=True)
        pipeline = comparison.worker(binary, 'focus', manifest, timeout=60)
    if pipeline['status'] != 'ok':
        parser.error('Application-rule replay failed: ' + str(pipeline))
    report = {'schema': 2, 'diagnostic': 'VIOFO focused GPS boundary investigation',
        'anchor': args.anchor, 'coordinates_included': args.include_coordinates,
        'app_pipeline_commit': 'b7d5fd643e39ba48d916b7ceedc69f06f7fefadd',
        'original_decoder_commit': '919abfd44d6009e2cc9d218709bdcfa06265e8b1',
        'scope': pipeline['scope'],
        'time_notes': 'gps_utc comes from embedded validated records. camera_time_estimate is filename start plus decoded offset, not an independent reading of the video overlay. Point numbers refer to decoded output, not MP4 byte offsets.',
        'groups': []}
    for i, (group, segment) in enumerate(zip(chosen, pipeline['segments'])):
        print('Analyzing ' + ('previous', 'focus', 'next')[i] + ' recording...', flush=True)
        candidates = []
        for source, candidate in zip(group, segment['candidates']):
            raw = comparison.worker(binary, 'original', Path(source['path']))
            original = {'status': raw['status']}
            if raw['status'] == 'ok':
                original.update(describe([{'point':p} for p in raw['points']], candidate['file'], args.include_coordinates))
            else:
                original.update({k:v for k,v in raw.items() if k in ['exit_code','timeout_seconds','error_type']})
            candidates.append({'file':candidate['file'],'duration_s':candidate['duration_s'],
                'app_decode_ok':candidate['decode_ok'],'app_selection_quality':candidate['quality'],
                'original_per_file':original,
                'app_validated_per_file':describe(candidate['timed'],candidate['file'],args.include_coordinates)})
        before, after = segment['selected_before_boundary'], segment['selected_after_boundary']
        trimmed = [p for p in after['timed'] if 0 <= p['point']['tOffsetS'] <= segment['duration_s']]
        report['groups'].append({'role':('previous','focus','next')[i],
            'duration_s':segment['duration_s'],'candidates':candidates,
            'channel_comparison':compare_channels(segment['candidates']),
            'selected_file':before['file'],
            'before_boundary':describe(before['timed'],before['file'],args.include_coordinates),
            'after_boundary_and_duration_trim':describe(trimmed,after['file'],args.include_coordinates),
            'points_removed_by_boundary':removed(before['timed'],after['timed'],before['file'],args.include_coordinates),
            'points_removed_by_duration_trim':removed(after['timed'],trimmed,after['file'],args.include_coordinates)})
    report['boundaries_before'] = boundaries(pipeline['segments'],'selected_before_boundary',args.include_coordinates)
    report['boundaries_after'] = boundaries(pipeline['segments'],'selected_after_boundary',args.include_coordinates)
    report['stitched_summary'] = comparison.summary({'status':'ok','points':pipeline['stitched']})
    descriptor = os.open(args.report, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(descriptor, 'w', encoding='utf-8') as out:
        json.dump(report,out,indent=2,allow_nan=False)
        out.write('\n')
    focus = report['groups'][1]
    print('Focus source: ' + focus['selected_file'])
    print('Largest focus distance before boundary: ' + str(focus['before_boundary']['largest_adjacent_distance_m']) + ' m')
    print('Largest focus distance after boundary/trim: ' + str(focus['after_boundary_and_duration_trim']['largest_adjacent_distance_m']) + ' m')
    print('Points removed at focus boundary: ' + str(len(focus['points_removed_by_boundary'])))
    print('Report: ' + str(args.report))
    print('No upload. Exact coordinates included: ' + str(args.include_coordinates))


if __name__ == '__main__':
    main()
