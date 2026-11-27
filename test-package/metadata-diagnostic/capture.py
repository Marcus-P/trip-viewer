#!/usr/bin/env python3
"""Read-only metadata capture, prepared with Codex for @Marcus-P (MIT).

Raw metadata and unknown MP4 blocks stay in PRIVATE-DO-NOT-SHARE. The
separate report contains a field inventory, never unreviewed source values.
"""
import argparse
from collections import Counter
import json
import os
from pathlib import Path
import re
import signal
import struct
import subprocess
import time

from diagnose import discover, PROFILES
from parking_analysis import filename_info

MAX_OUTPUT = 128 * 1024 * 1024
MAX_BLOCK = 32 * 1024 * 1024
MAX_BLOCK_TOTAL = 128 * 1024 * 1024
TIMEOUT = 180
KNOWN_BOXES = set('ftyp moov mdat free skip wide uuid meta udta moof mfra sidx'.split())


def write_json(path, value):
    with path.open('x', encoding='utf-8') as stream:
        json.dump(value, stream, indent=2, ensure_ascii=True, allow_nan=False)
        stream.write('\n')


def stop_child(process):
    if process.poll() is not None:
        return
    if os.name == 'nt':
        # ExifTool's Windows launcher starts its own Perl child. Kill only this
        # invocation's process tree, never another ExifTool or media process.
        subprocess.run(['taskkill', '/PID', str(process.pid), '/T', '/F'],
                       stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
                       check=False, timeout=10)
    else:
        os.killpg(process.pid, signal.SIGKILL)
    process.wait(timeout=10)


def run_tool(command, output, error, timeout=TIMEOUT):
    status = 'complete'
    process = None
    with output.open('xb') as stdout, error.open('xb') as stderr:
        try:
            process = subprocess.Popen(command, stdin=subprocess.DEVNULL,
                                       stdout=stdout, stderr=stderr,
                                       start_new_session=os.name != 'nt')
            deadline = time.monotonic() + timeout
            while process.poll() is None:
                if time.monotonic() > deadline:
                    status = 'time_limit'
                    break
                if output.stat().st_size + error.stat().st_size > MAX_OUTPUT:
                    status = 'output_limit'
                    break
                time.sleep(.2)
            if status != 'complete':
                stop_child(process)
            elif process.returncode:
                status = 'tool_error'
        except OSError:
            status = 'tool_unavailable'
        finally:
            if process is not None and process.poll() is None:
                stop_child(process)
    if output.stat().st_size + error.stat().st_size > MAX_OUTPUT:
        status = 'output_limit'
    return {'status': status}


def load_json(path):
    try:
        with path.open(encoding='utf-8-sig') as stream:
            return json.load(stream)
    except (OSError, ValueError, UnicodeError):
        return None


def raw_blocks(path, destination):
    """Preserve top-level non-media boxes verbatim, including unknown types.

    Saving moov/meta/udta also retains nested unrecognized bytes. Do not copy
    video/audio payloads or claim to understand unknown structures within mdat.
    """
    index, total, status, counts = [], 0, 'complete', Counter()
    try:
        with path.open('rb') as source:
            size = path.stat().st_size
            position = 0
            for number in range(100000):
                if position == size:
                    break
                source.seek(position)
                header = source.read(8)
                if len(header) != 8:
                    status = 'invalid_container'
                    break
                length, kind = struct.unpack('>I4s', header)
                header_length = 8
                if length == 1:
                    extended = source.read(8)
                    if len(extended) != 8:
                        status = 'invalid_container'
                        break
                    length = struct.unpack('>Q', extended)[0]
                    header_length = 16
                elif length == 0:
                    length = size - position
                if length < header_length or position + length > size:
                    status = 'invalid_container'
                    break
                name = kind.decode('latin1')
                counts[name if name in KNOWN_BOXES else 'unrecognized_type'] += 1
                item = {'offset': position, 'length': length,
                        'type_hex': kind.hex(), 'status': 'media_payload_not_copied'}
                if kind != b'mdat':
                    if length > MAX_BLOCK or total + length > MAX_BLOCK_TOTAL:
                        item['status'] = 'block_size_limit'
                        status = 'partial'
                    else:
                        target = destination / ('block-%06d.bin' % number)
                        with target.open('xb') as output:
                            source.seek(position)
                            remaining = length
                            while remaining:
                                block = source.read(min(remaining, 1024 * 1024))
                                if not block:
                                    raise OSError('short read')
                                output.write(block)
                                remaining -= len(block)
                        total += length
                        item.update(status='saved_verbatim', saved_as=target.name)
                index.append(item)
                position += length
            else:
                status = 'box_count_limit'
    except (OSError, struct.error):
        status = 'input_read_error'
    write_json(destination / 'container-index-PRIVATE.json', index)
    return {'status': status, 'top_level_box_counts': dict(counts),
            'saved_blocks': sum(i['status'] == 'saved_verbatim' for i in index),
            'limited_blocks': sum(i['status'] == 'block_size_limit' for i in index)}


def fields(value, prefix=()):
    if isinstance(value, dict):
        for key, child in value.items():
            yield from fields(child, prefix + (str(key),))
    elif isinstance(value, list):
        for child in value:
            yield from fields(child, prefix + ('[]',))
    else:
        yield prefix, value


class Inventory:
    def __init__(self, standard_tags):
        self.standard_tags = standard_tags
        self.rows = {}

    def add(self, tool, data, set_id, file_id):
        for path, value in fields(data):
            key = (tool, path)
            if key not in self.rows:
                self.rows[key] = {'id': 'field-%05d' % (len(self.rows) + 1),
                                  'types': Counter(), 'sets': {}}
            row = self.rows[key]
            row['types'][type(value).__name__] += 1
            entry = row['sets'].setdefault(set_id, {'files': set(), 'values': set(),
                                                   'occurrences': 0})
            entry['files'].add(file_id)
            # Values are used only in memory for equality; neither fingerprints
            # nor raw strings/numbers are emitted to the sharing report.
            entry['values'].add(json.dumps(value, sort_keys=True, ensure_ascii=True))
            entry['occurrences'] += 1

    def reports(self):
        public, private = [], []
        for (tool, path), row in self.rows.items():
            final = path[-1].split(':')[-1] if path else ''
            item = {'field_id': row['id'], 'tool': tool,
                    'value_types': dict(row['types']), 'sets': {}}
            # Group labels and custom keys can themselves contain private text.
            # Only names from ExifTool's own tag dictionary are made public.
            if tool == 'exiftool' and final in self.standard_tags:
                item['standard_tag_name'] = final
            else:
                item['name'] = 'see_private_field_index'
            item['possible_mode_or_rate_field'] = bool(re.search(
                r'park|timelapse|recordingmode|capturemode|capture.*rate', final, re.I))
            sets = row['sets']
            for set_id, entry in sets.items():
                item['sets'][set_id] = {
                    'files_with_field': len(entry['files']),
                    'occurrences': entry['occurrences'],
                    'distinct_values': len(entry['values']),
                }
            item['constant_within_each_set_and_different_between_sets'] = (
                len(sets) > 1 and all(len(s['values']) == 1 for s in sets.values())
                and len({next(iter(s['values'])) for s in sets.values()}) > 1)
            public.append(item)
            private.append({'field_id': row['id'], 'tool': tool,
                            'raw_json_path': list(path)})
        return public, private


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--set', action='append', nargs=2, required=True)
    parser.add_argument('--exiftool', type=Path, required=True)
    parser.add_argument('--ffprobe', type=Path, required=True)
    parser.add_argument('--output-dir', type=Path, required=True)
    args = parser.parse_args()
    if any(profile not in PROFILES for profile, _ in args.set):
        parser.error('Unknown recording profile.')
    if not args.exiftool.is_file() or not args.ffprobe.is_file():
        parser.error('Both metadata tools must exist before starting.')
    try:
        args.output_dir.mkdir(mode=0o700, parents=False, exist_ok=False)
    except OSError:
        parser.exit(2, 'Choose a new, writable output directory.\n')
    private = args.output_dir / 'PRIVATE-DO-NOT-SHARE'
    private.mkdir(mode=0o700)
    config = private / 'empty-exiftool-config.txt'
    config.write_text('1;\n', encoding='ascii')
    exif = [str(args.exiftool.resolve()), '-config', str(config.resolve())]
    dictionary = private / 'tool-standard-tags.txt'
    dictionary_status = run_tool(exif + ['-list', '-s'], dictionary,
                                 private / 'dictionary-errors.txt')
    standard_tags = set()
    if dictionary_status['status'] == 'complete':
        standard_tags.update(dictionary.read_text(encoding='utf-8', errors='replace').split())
    inventory = Inventory(standard_tags)
    report = {'format': 'viofo-metadata-inventory-v1', 'status': 'complete',
              'privacy': 'source_values_names_paths_times_positions_and_hashes_omitted',
              'scope': 'stored_metadata_visible_to_tools_and_bounded_raw_MP4_boxes',
              'limits': {'seconds_per_tool_per_file': TIMEOUT,
                         'output_bytes_per_tool': MAX_OUTPUT,
                         'raw_bytes_per_box': MAX_BLOCK,
                         'raw_box_bytes_per_file': MAX_BLOCK_TOTAL},
              'not_claimed': ['all_vendor_semantics_decoded', 'all_mdat_bytes_extracted',
                              'mode_inferred_from_test_set', 'universal_compatibility'],
              'sets': []}
    mapping = []
    current = None
    try:
        for set_number, (profile, root) in enumerate(args.set, 1):
            set_id = 'set-%02d' % set_number
            paths, discovery = discover(Path(root))
            group = {'id': set_id, 'declared_profile': PROFILES[profile],
                     'discovery': discovery, 'files': []}
            report['sets'].append(group)
            if discovery.get('status') != 'complete' or not paths:
                report['status'] = 'partial'
            for file_number, path in enumerate(paths, 1):
                file_id = '%s-file-%04d' % (set_id, file_number)
                print('Reading %s (%d/%d)' % (file_id, file_number, len(paths)), flush=True)
                destination = private / file_id
                destination.mkdir()
                mapping.append({'id': file_id, 'source_path': str(path.resolve())})
                current = {'id': file_id, 'filename_pattern': filename_info(path)[0],
                           'status': 'running'}
                group['files'].append(current)
                current['raw_container'] = raw_blocks(path, destination)
                commands = {
                    'exiftool': exif + ['-json', '-a', '-u', '-U', '-ee3', '-G4:1',
                                      '-struct', '-b', '-api', 'LargeFileSupport=1',
                                      str(path.resolve())],
                    'ffprobe': [str(args.ffprobe.resolve()), '-v', 'error',
                                '-show_error', '-show_format', '-show_streams',
                                '-show_chapters', '-show_programs', '-show_private_data',
                                '-show_data', '-of', 'json', '-i', str(path.resolve())],
                }
                for tool, command in commands.items():
                    raw = destination / (tool + '-PRIVATE.json')
                    result = run_tool(command, raw, destination / (tool + '-errors-PRIVATE.txt'))
                    data = load_json(raw)
                    result['json_readable'] = data is not None
                    current[tool] = result
                    if data is not None:
                        inventory.add(tool, data, set_id, file_id)
                current['status'] = ('complete' if all(current[k]['status'] == 'complete'
                    for k in ('raw_container', 'exiftool', 'ffprobe'))
                    and all(current[k]['json_readable'] for k in commands) else 'partial')
                if current['status'] != 'complete':
                    report['status'] = 'partial'
                current = None
    except KeyboardInterrupt:
        report['status'] = 'interrupted'
        if current is not None:
            current['status'] = 'interrupted'
    except Exception:
        # Exception text and traceback could include private metadata or paths.
        report['status'] = 'partial'
        if current is not None:
            current['status'] = 'collection_error'
    finally:
        report['fields'], field_mapping = inventory.reports()
        write_json(private / 'input-index-PRIVATE.json', mapping)
        write_json(private / 'field-index-PRIVATE.json', field_mapping)
        write_json(args.output_dir / 'SHARE-metadata-report.json', report)
    print('Finished: %s. Review SHARE-metadata-report.json; keep PRIVATE local.' % report['status'])
    return 0 if report['status'] == 'complete' else 1


if __name__ == '__main__':
    raise SystemExit(main())
