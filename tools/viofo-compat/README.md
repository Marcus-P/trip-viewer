# Focused GPS investigation added

The newer focus download and `diagnose-gps.py` replay the 0.13.0 source-selection
and cross-file checks. See [FOCUS_README.md](FOCUS_README.md) for scope and commands.
The per-file comparison documented below remains available and unchanged.

# Read-only VIOFO comparison test

This independent command-line tool compares @alinnastac's PR #12 decoder and
filename parser at `919abfd44d6009e2cc9d218709bdcfa06265e8b1` with the isolated
VIOFO contribution in PR #21 at `eb0a5a3f22bd2f89581c7ec3e54c461e78c4c6a5`.
The application sources from @alinnastac remain unchanged. The PR #21 source
copies are byte-identical; SHA-256 checksums are in PROVENANCE.json. The small
headless harness provides shared model/error definitions and calls the same
MP4 duration probe. No GUI, archive database or app installation is involved.

## Run on Ubuntu 24.04 x64

Download and extract the Linux comparison bundle from the test release. In the
extracted directory:

```sh
sha256sum -c SHA256SUMS.txt
python3 run-compare.py "$HOME/VIOFO-TripViewer-Test" --report "$HOME/Downloads/viofo-compare-report.json"
```

Replace the input folder with a folder containing a small selection of original
MP4s. Up to 12 MP4s are decoded in path order. For a more precise test pass
individual files instead of a folder. Include front/interior/rear, an older
counter-free example and a parking example if available. Adjust `--limit` to
process more files. The report must not already exist; use another report name
for a second run. The binary can also be invoked on Windows after compiling
it there; this particular prebuilt bundle is for Ubuntu 24.04 x64 only.

Each worker is stopped after 15 seconds. Linux workers also have a 512 MiB
address-space cap. These limits matter because the original decoder still
hangs on an empty GPS descriptor and allocates from unbounded descriptor sizes.
Errors/timeouts are reported instead of treated as empty successful results.

The report contains basenames (including recording timestamps), byte sizes,
recognition/grouping results and aggregate GPS diagnostics. It contains no full
paths or route coordinates. No report or media is uploaded. Review the report
before sharing it. Original files are only opened for reading; no archives,
sidecars, application settings or existing reports are modified.

This is a per-file decoder/naming test. It does not test playback, live camera
hardware, application segment assembly, cross-file boundary validation or the
GPU pipeline. The isolated PR #21 decoder is the baseline, not the entire
0.13.0 application. Differences in first-fix alignment or rejected GPS records
can legitimately change point counts and offsets; a report needs interpretation.
It does not certify compatibility with all firmware or camera models.

## Build it yourself

With stable Rust/Cargo installed, clone this test branch, then:

```sh
cd tools/viofo-compat
./build.sh
python3 run-compare.py /path/to/samples --report report.json
```

No Node.js, GTK, FFmpeg or system-wide app installation is required for the tool.
For Windows use `cargo build --locked --release` in this directory, copy
`target/release/viofo-compare.exe` next to `run-compare.py`, and use `python`.
The tool has its own locked dependencies and does not alter application versions.

Reproduce the automated checks:

```sh
cargo test --locked
cargo build --locked
python3 test-comparison.py target/debug/viofo-compare
```

## Findings at the pinned commits (2026-11-24)

- Original application Rust suite: 309 passed, 1 ignored; frontend 19 passed;
  frontend production build passed.
- Headless included parser/decoder tests: 63 passed.
- Nine subprocess comparison probes passed. They establish these differences,
  rather than claiming that all probes are compatibility successes:
  - The real A129 sample *record* already published in @alinnastac's unit test
    decodes to the same expected position through both repackaged MP4 fixtures.
    This is not a test on the complete original A129 video.
  - Numbered front/interior/rear names are recognized by both.
  - Counter-free names such as `2025_0706_201953_F.MP4` are rejected by the
    original branch and accepted by PR #21.
  - Minute-level grouping in the original branch combines separate same-channel
    recordings within a minute. It handles a one-second shift within the same
    minute, but fails across a minute boundary. PR #21 exact-time grouping fails
    both one-second-shift cases; neither implementation fully solves the skew.
  - Normal/parking filename group keys differ in both implementations. The
    reported mixing in application segment assembly is not reproduced by this
    filename-only test and remains unresolved.
  - A synthetic in-range longitude outlier produces an over-2,000-km adjacent
    jump in the original decoder; PR #21 rejects the bad middle position.
  - An empty GPS descriptor hangs in the original branch and is safely rejected
    by PR #21. The worker watchdog bounds the regression probe.
- A generated five-second H.264 MP4 with five well-formed GPS records passed
  the complete runner with equal positions in both decoders. Input SHA-256
  remained unchanged; no archive was created; existing-report overwrite was
  refused.

Original A229 Pro media is not available in this cloud workspace. A report from
owner-supplied files is still needed before claiming actual A229 compatibility.
Do not install this upstream-based app over the productive 0.13.0 installation
or open its archive with an older schema. This command-line test avoids that
risk without changing the Tauri application identifier.

Source authorship and MIT notices are retained. The comparison harness was
prepared with Codex by OpenAI. See LICENSE and the package's dependency notices.
No message to @alinnastac or Chris is sent by this tool.
