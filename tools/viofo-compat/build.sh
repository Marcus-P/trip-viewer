#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "$0")"
cargo build --locked --release --target-dir "$PWD/target"
cp target/release/viofo-compare ./viofo-compare
printf '%s\n' 'Built ./viofo-compare. Run: python3 run-compare.py /path/to/sample-folder --report report.json'
