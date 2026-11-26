#!/usr/bin/env python3
"""One-run, read-only VIOFO parking observations; Python 3.9+, no dependencies.
Prepared with Codex for @Marcus-P. See README.md for source and privacy scope.
"""
import argparse
import json
import os
from pathlib import Path
import sys

from parking_analysis import analyse_file, clean_error, compare_files

PROFILES = {
    '2ch-lowbitrate': {
        'source': 'owner_confirmed_settings_not_inferred_from_media',
        'model': 'A229 Pro', 'main_firmware': 'V1.5_260813',
        'channels': 2, 'parking_mode': 'low_bitrate',
        'parking_timer_minutes': 30, 'entry_delay_seconds': 0,
        'parking_gps_enabled': True, 'parking_hdr_enabled': True,
    },
    '3ch-time2fps': {
        'source': 'owner_confirmed_settings_not_inferred_from_media',
        'model': 'A229 Pro', 'main_firmware': 'V1.5_260813',
        'channels': 3, 'parking_mode': 'time_lapse', 'capture_fps_setting': 2,
        'parking_timer_minutes': 30, 'entry_delay_seconds': 90,
        'parking_gps_enabled': True, 'parking_hdr_enabled': True,
    },
    'unknown': {'source': 'settings_not_provided'},
}


def discover(root):
    files = []
    summary = {'status': 'complete', 'ignored_non_video_files': 0,
               'skipped_links': 0, 'directory_errors': []}
    if not root.exists():
        return files, {'status': 'incomplete', 'reason': 'input_file_not_found'}
    if root.is_file():
        return [root], summary
    pending = [(root, 0)]
    visited = 0
    while pending:
        directory, depth = pending.pop()
        if depth > 32 or visited > 100_000 or len(files) > 2000:
            summary.update(status='partial', reason='discovery_limit')
            break
        try:
            with os.scandir(directory) as entries:
                for entry in entries:
                    visited += 1
                    if entry.is_symlink():
                        summary['skipped_links'] += 1
                    elif entry.is_dir(follow_symlinks=False):
                        metadata = entry.stat(follow_symlinks=False)
                        if getattr(metadata, 'st_file_attributes', 0) & 0x400:
                            summary['skipped_links'] += 1
                        else:
                            pending.append((Path(entry.path), depth + 1))
                    elif entry.is_file(follow_symlinks=False):
                        if Path(entry.name).suffix.lower() in ('.mp4', '.mov', '.ts'):
                            if len(files) >= 2000:
                                summary.update(status='partial', reason='discovery_limit')
                                pending.clear()
                                break
                            files.append(Path(entry.path))
                        else:
                            summary['ignored_non_video_files'] += 1
                    if visited > 100_000 or len(files) > 2000:
                        summary.update(status='partial', reason='discovery_limit')
                        break
        except OSError as error:
            summary['status'] = 'partial'
            summary['directory_errors'].append(clean_error(error))
    return sorted(files, key=lambda path: str(path).casefold()), summary


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--set', action='append', nargs=2, metavar=('PROFILE', 'FOLDER'), required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    if any(profile not in PROFILES for profile, _ in args.set):
        parser.error('Unknown profile. Choose 2ch-lowbitrate, 3ch-time2fps or unknown.')
    if args.output.exists():
        parser.exit(2, 'Output already exists. Choose a new report filename.\n')
    # Probe the destination before reading media. Exclusive creation prevents
    # accidentally opening an original recording or prior report for writing.
    try:
        stream = args.output.open('x', encoding='utf-8')
    except OSError:
        parser.exit(2, 'Cannot create the report. Choose a writable, new output path.\n')
    report = {
        'format': 'viofo-parking-observations-v3',
        'scope': 'independent_read_only_observations_not_application_or_hardware_certification',
        'privacy': 'no_paths_filenames_coordinates_absolute_times_hashes_or_serials',
        'status': 'complete', 'sets': [],
        'limits': {'per_file_seconds': 120, 'records_per_file': 100000,
                   'files_per_set': 2000, 'timed_samples_per_track': 1000000},
    }
    current = None
    exit_code = 0
    try:
        for group_number, (profile, folder) in enumerate(args.set, 1):
            current = {'set_id': f'set-{group_number:02}',
                       'declared_settings': PROFILES[profile], 'files': []}
            report['sets'].append(current)
            try:
                paths, discovery = discover(Path(folder))
            except OSError as error:
                paths, discovery = [], {'status': 'incomplete', 'reason': clean_error(error)}
            current['discovery'] = discovery
            current['discovery']['video_inputs_found'] = len(paths)
            private = []
            for number, path in enumerate(paths, 1):
                file_id = f'file-{number:03}'
                print(f"{current['set_id']} {file_id} ({number}/{len(paths)}): reading", flush=True)
                if path.suffix.lower() == '.ts':
                    result, internal = {'status': 'unsupported', 'reason': 'mpeg_transport_stream_not_parsed'}, None
                else:
                    try:
                        result, internal = analyse_file(path)
                    except Exception:
                        result, internal = {'status': 'partial', 'error': {
                            'stage': 'unexpected_diagnostic_failure',
                            'reason': 'unexpected_error_without_private_exception_details'}}, None
                result['file_id'] = file_id
                current['files'].append(result)
                if internal is not None:
                    private.append((file_id, internal))
                print(f"{current['set_id']} {file_id}: {result['status']}", flush=True)
            current['cross_channel_observations'] = compare_files(private)
            if discovery['status'] != 'complete' or not paths or any(f['status'] != 'complete' for f in current['files']):
                report['status'] = 'partial'
    except KeyboardInterrupt:
        report.update(status='partial', stopped_reason='user_interrupted')
        exit_code = 130
    except Exception:
        report.update(status='partial', stopped_reason='unexpected_diagnostic_failure')
        exit_code = 1
    finally:
        with stream:
            json.dump(report, stream, indent=2, allow_nan=False)
            stream.write('\n')
    print('Local report written. Nothing uploaded. Review the JSON before sharing.')
    return exit_code


if __name__ == '__main__':
    sys.exit(main())
