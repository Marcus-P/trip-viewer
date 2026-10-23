#!/usr/bin/env python3
"""Keep one small, identified installer per platform, never a build directory."""
import argparse
import hashlib
import json
import os
import pathlib
import shutil

from native_diagnostics import annotate_exceptions

annotate_exceptions("Native package preparation")

parser = argparse.ArgumentParser()
parser.add_argument("platform", choices=["Windows-x64", "macOS-universal"])
parser.add_argument("--out", required=True, type=pathlib.Path)
parser.add_argument("--test", action="store_true")
args = parser.parse_args()
root = pathlib.Path(__file__).resolve().parent.parent
version = json.loads((root / "package.json").read_text())["version"]
settings = {
    "Windows-x64": ("x86_64-pc-windows-msvc", "nsis", "*.exe", "Windows_x64_setup.exe"),
    "macOS-universal": ("universal-apple-darwin", "dmg", "*.dmg", "macOS_universal_experimental.dmg"),
}
target, bundle, pattern, suffix = settings[args.platform]
sources = list((root / "src-tauri/target" / target / "release/bundle" / bundle).glob(pattern))
if len(sources) != 1 or sources[0].stat().st_size <= 0:
    raise RuntimeError("Expected exactly one nonempty " + args.platform + " installer")
sha = os.environ.get("GITHUB_SHA", "local")
marker = "_test-" + sha[:7] if args.test else ""
name = "Trip.Viewer_" + version + marker + "_" + suffix
args.out.mkdir(parents=True, exist_ok=True)
dest = args.out / name
shutil.copyfile(sources[0], dest)
digest = hashlib.file_digest(dest.open("rb"), "sha256").hexdigest()
(args.out / ("SHA256SUMS-" + args.platform + ".txt")).write_text(digest + "  " + name + "\n", encoding="utf-8")
if args.test:
    (args.out / "BUILD_INFO.txt").write_text(
        "Trip Viewer " + args.platform + " acceptance installer\nVersion: " + version +
        "\nCommit: " + sha + "\nmacOS: experimental, no real-device acceptance yet.\n", encoding="utf-8")
print(name)
