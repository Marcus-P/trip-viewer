#!/usr/bin/env bash
set -euo pipefail
bundle_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
test_root="${TRIPVIEWER_PR21_HOME:-$HOME/.local/share/tripviewer-pr21-test/5dbdaf9}"
export XDG_DATA_HOME="$test_root/data"
export XDG_CONFIG_HOME="$test_root/config"
export XDG_CACHE_HOME="$test_root/cache"
export XDG_STATE_HOME="$test_root/state"
export APPIMAGE_EXTRACT_AND_RUN=1
test_archive="${TRIPVIEWER_PR21_ARCHIVE:-$HOME/Downloads/TripViewer-PR21-Testarchiv-5dbdaf9}"
mkdir -p "$XDG_DATA_HOME/com.tripviewer.pr21test" "$XDG_CONFIG_HOME" "$XDG_CACHE_HOME" "$XDG_STATE_HOME" "$test_archive/Videos/RO"
# Only this isolated application's settings are changed. Preserve its other keys.
python3 - "$XDG_DATA_HOME/com.tripviewer.pr21test/settings.json" "$test_archive" <<'PYSETTINGS'
import json, os, sys
from pathlib import Path
p=Path(sys.argv[1])
s=json.loads(p.read_text()) if p.exists() else {}
s.update(schema_version=1, last_archive=str(Path(sys.argv[2]).resolve()), recent_archives=[])
tmp=p.with_suffix('.tmp')
tmp.write_text(json.dumps(s, indent=2)+'\n')
os.replace(tmp,p)
PYSETTINGS
printf 'PR21-Testarchiv: %s\nLokales Startlog: %s/last-run.log\n' "$test_archive" "$test_root"
exec "$bundle_dir/TripViewer-PR21-Test.AppImage" > "$test_root/last-run.log" 2>&1
