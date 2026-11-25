# VIOFO time-only diagnostic

This is an optional standalone check for the temporal-ambiguity messages in
PR #21. It does not change Trip Viewer, its database, the validator or recordings.
Python 3.9 or newer is required; no third-party Python packages, FFmpeg or Rust
build is needed. If Python is not available, there is no need to install it
just for this check: report that limitation instead.

Download `viofo-time-diagnostic.py` from the same revision as this guide and
save it in a working folder. Open a terminal in that folder. Select one normal
clip that produced the warnings and, if convenient, one parking clip. Two clips
are enough; do not scan the whole archive. Each input argument is a quoted path
to an existing MP4. Dragging a file into the terminal may help enter its path.

Linux/macOS (with Python already installed):

```sh
python3 viofo-time-diagnostic.py --output viofo-time-report.json "/path/to/normal.MP4" "/path/to/parking.MP4"
```

Windows PowerShell (with Python and its launcher already installed):

```powershell
py -3 .\viofo-time-diagnostic.py --output viofo-time-report.json "D:\Clips\normal.MP4" "D:\Clips\parking.MP4"
```

Replace the two example input paths with your chosen files; omit the second
argument if only one clip is convenient. Use an output filename that does not
exist yet. The tool refuses to overwrite an existing report or input.

The script creates the report locally and uploads nothing. It contains ordinal
IDs such as `file-001`, counts, video duration and relative GPS elapsed spans.
No filenames, paths, hashes, coordinates, absolute recording timestamps,
computer/user details or raw GPS records are written to the report. Input order
is the only local correspondence to those IDs. Timing statistics remain
characteristics of recordings, so inspect the JSON before choosing to share it;
this is data minimization, not a guarantee of mathematical anonymity.

## Interpretation and limits

- `missing` counts decoded records without a valid date/time.
- `contradictory` counts isolated time contradictions identified by the rule.
- `ambiguous` counts unresolved ordering/span conflicts.
- The current application message combines `missing` and `ambiguous` as
  “temporal ambiguous”; this report separates them.
- `temporal_with_duration_bound` and `temporal_without_duration_bound` reproduce
  the temporal rules with and without the measured media-duration constraint.
  Their difference is diagnostic evidence, not permission to weaken validation.
- Duration comes from the first video track's MP4 media header (`mdhd`). The
  script does not apply an edit list or prove synchronization against video
  frames; duration discrepancies need further examination.
- This independent Python implementation follows the parser layout and temporal
  rules at PR21/091b392. It does not run the Rust app or spatial validator, assess
  trip/channel grouping, or explain the earlier 15-point comparison by itself.
- A malformed/unsupported file, 100,000-descriptor limit, or 60-second per-file
  analysis limit produces `incomplete`; no partial result is reported as complete.

If convenient, share only the reviewed JSON on the existing PR discussion. No
footage, screenshots, database, full application logs or coordinates are needed.
The optional parking test does not require the fork owner to possess parking
footage. A complete result with no GPS box is not claimed for unsupported files.
