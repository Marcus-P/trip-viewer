#!/usr/bin/env python3
"""Read-only comparison; decoder workers have a timeout and Linux memory cap."""
import argparse
from collections import defaultdict
import json
import math
import os
from pathlib import Path
import subprocess
import sys
import time


def child_limits():
    if sys.platform.startswith('linux'):
        import resource
        resource.setrlimit(resource.RLIMIT_AS, (512 * 1024**2, 512 * 1024**2))
        resource.setrlimit(resource.RLIMIT_CORE, (0, 0))


def worker(binary, mode, path, timeout=15):
    start = time.monotonic()
    try:
        process = subprocess.run([str(binary), mode, str(path)], stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, timeout=timeout,
            preexec_fn=child_limits if sys.platform.startswith('linux') else None)
        if process.returncode:
            return {'status': 'error', 'exit_code': process.returncode}
        value = json.loads(process.stdout)
        value['status'] = 'ok'
        value['elapsed_seconds'] = round(time.monotonic() - start, 3)
        return value
    except subprocess.TimeoutExpired:
        return {'status': 'timeout', 'timeout_seconds': timeout}
    except (OSError, ValueError) as exc:
        return {'status': 'error', 'error_type': type(exc).__name__}


def distance_km(a, b):
    lat1, lat2 = map(math.radians, [a['lat'], b['lat']])
    dlat = lat2 - lat1
    dlon = math.radians(b['lon'] - a['lon'])
    term = math.sin(dlat / 2)**2 + math.cos(lat1) * math.cos(lat2) * math.sin(dlon / 2)**2
    return 12742 * math.asin(math.sqrt(max(0, min(1, term))))


def summary(result):
    if result['status'] != 'ok':
        return result
    points = result.get('points', [])
    valid = [p for p in points if p.get('fixOk') and all(
        isinstance(p.get(k), (int, float)) and math.isfinite(p[k]) for k in ['lat', 'lon', 'tOffsetS'])
        and abs(p['lat']) <= 90 and abs(p['lon']) <= 180]
    steps = [distance_km(a, b) for a, b in zip(valid, valid[1:])]
    times = [p['tOffsetS'] for p in valid]
    return {'status': 'ok', 'point_count': len(points), 'valid_point_count': len(valid),
        'first_offset_s': times[0] if times else None,
        'last_offset_s': times[-1] if times else None,
        'non_increasing_time_steps': sum(b <= a for a, b in zip(times, times[1:])),
        'negative_offsets': sum(t < 0 for t in times),
        'max_adjacent_distance_km': round(max(steps, default=0), 6),
        'elapsed_seconds': result.get('elapsed_seconds')}


def compare_points(a, b):
    if a['status'] != 'ok' or b['status'] != 'ok':
        return {'status': 'not_comparable'}
    # Exact-time matches only. Different first-fix alignment can reduce matches.
    left = {p['tOffsetS']: p for p in a.get('points', []) if p.get('fixOk')}
    distances = [distance_km(left[p['tOffsetS']], p) * 1000 for p in b.get('points', [])
        if p.get('fixOk') and p['tOffsetS'] in left and all(
            isinstance(v, (int, float)) and math.isfinite(v)
            for v in [p['lat'], p['lon'], left[p['tOffsetS']]['lat'], left[p['tOffsetS']]['lon']])]
    return {'status': 'compared', 'matching_time_offsets': len(distances),
        'max_position_difference_m_at_matching_offset': round(max(distances), 3) if distances else None,
        'note': 'Different validation/alignment can change point counts and offsets; differences require review.'}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('inputs', nargs='+', type=Path, help='MP4 files or folder (read only)')
    parser.add_argument('--binary', type=Path, default=Path(__file__).with_name('viofo-compare' + ('.exe' if os.name == 'nt' else '')))
    parser.add_argument('--limit', type=int, default=12, help='Maximum files decoded (default 12)')
    parser.add_argument('--report', type=Path, default=Path('viofo-compare-report.json'))
    args = parser.parse_args()
    if args.limit < 1 or not args.binary.is_file():
        parser.error('Need a built worker binary and a positive file limit.')
    if args.report.exists():
        parser.error('Report already exists; choose a new --report filename.')
    paths = set()
    for entry in args.inputs:
        if not entry.exists():
            parser.error(f'Input not found: {entry}')
        candidates = entry.rglob('*') if entry.is_dir() else [entry]
        for path in candidates:
            if path.is_file() and not path.is_symlink() and path.suffix.lower() == '.mp4':
                paths.add(path.resolve())
    if not paths:
        parser.error('No MP4 files found.')
    selected = sorted(paths)[:args.limit]
    report = {'schema': 1, 'original_commit': '919abfd44d6009e2cc9d218709bdcfa06265e8b1',
        'our_pr21_commit': 'eb0a5a3f22bd2f89581c7ec3e54c461e78c4c6a5',
        'scope': 'Per-file naming and GPS decoder comparison only; no archive writes, playback or cross-file GPS boundary pass.',
        'discovered_files': len(paths), 'tested_files': len(selected), 'files': []}
    groups = {'original': defaultdict(list), 'ours': defaultdict(list)}
    for index, path in enumerate(selected, 1):
        print(f'[{index}/{len(selected)}] {path.name}', flush=True)
        names = worker(args.binary.resolve(), 'names', path)
        original = worker(args.binary.resolve(), 'original', path)
        ours = worker(args.binary.resolve(), 'ours', path)
        row = {'file': path.name, 'bytes': path.stat().st_size, 'naming': names,
            'original': summary(original), 'ours': summary(ours), 'comparison': compare_points(original, ours)}
        report['files'].append(row)
        if names['status'] == 'ok':
            for implementation in groups:
                item = names.get(implementation)
                if item:
                    groups[implementation][item['group']].append({'file_index': index, 'channel': item['channel']})
        print(f"  @alinnastac: {row['original'].get('point_count', row['original']['status'])}; PR21: {row['ours'].get('point_count', row['ours']['status'])}", flush=True)
    report['groups'] = {key: list(value.values()) for key, value in groups.items()}
    # Exclusive create: never overwrite media, an archive, or an existing report.
    with args.report.open('x', encoding='utf-8') as out:
        json.dump(report, out, indent=2, allow_nan=False)
        out.write('\n')
    print(f'Report saved: {args.report}. Nothing was uploaded. Review before sharing (contains clip basenames/times, no route coordinates).')


if __name__ == '__main__':
    main()
