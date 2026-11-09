#!/usr/bin/env python3
"""Prefetch Tauri's pinned Windows tools through the GitHub Release API.

Keep these pins/required files aligned with tauri-cli 2.10.1's
crates/tauri-bundler/src/bundle/windows/nsis/mod.rs. No persistent CI cache.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import time
import zipfile

CLI_VERSION = "2.10.1"
ASSETS = (
    ("tauri-apps/binary-releases", "nsis-3.11", "nsis-3.11.zip",
     "c7d27f780ddb6cffb4730138cd1591e841f4b7edb155856901cdf5f214394fa1"),
    ("tauri-apps/nsis-tauri-utils", "nsis_tauri_utils-v0.5.3", "nsis_tauri_utils.dll",
     "5ba143b5db4a87d32d6e7802e033330aae56cbceabe0d1e3ba41948385ad4709"),
)
REQUIRED = (
    "makensis.exe", "Bin/makensis.exe", "Stubs/lzma-x86-unicode",
    "Stubs/lzma_solid-x86-unicode",
    "Plugins/x86-unicode/additional/nsis_tauri_utils.dll",
    "Include/MUI2.nsh", "Include/FileFunc.nsh", "Include/x64.nsh",
    "Include/nsDialogs.nsh", "Include/WinMessages.nsh", "Include/Win/COM.nsh",
    "Include/Win/Propkey.nsh", "Include/Win/RestartManager.nsh",
)


def download_asset(asset, directory):
    repo, tag, name, expected = asset
    destination = directory / name
    for attempt in range(1, 4):
        destination.unlink(missing_ok=True)
        print(f"Downloading {repo}/{name} (attempt {attempt}/3)", flush=True)
        try:
            subprocess.run(
                ["gh", "release", "download", tag, "--repo", repo,
                 "--pattern", name, "--dir", str(directory)],
                check=True, timeout=120,
            )
        except (subprocess.CalledProcessError, subprocess.TimeoutExpired):
            if attempt == 3:
                raise
            time.sleep(5 * attempt)
            continue
        if hashlib.sha256(destination.read_bytes()).hexdigest() != expected:
            destination.unlink()
            raise RuntimeError(f"SHA-256 mismatch for {name}; refusing this tool")
        return destination


def prepare(tools_dir):
    cli = Path("node_modules/@tauri-apps/cli/package.json")
    if json.loads(cli.read_text(encoding="utf-8"))["version"] != CLI_VERSION:
        raise RuntimeError("Tauri CLI changed; review the pinned NSIS tools first")
    # Download and verify everything before touching Tauri's tool directory.
    with tempfile.TemporaryDirectory(prefix="tripviewer-nsis-") as scratch:
        scratch = Path(scratch)
        archive, plugin = [download_asset(asset, scratch) for asset in ASSETS]
        extracted = scratch / "extracted"
        with zipfile.ZipFile(archive) as package:
            # Only the expected archive subtree may be extracted.
            for member in package.infolist():
                parts = member.filename.replace("\\", "/").split("/")
                if parts[0] != "nsis-3.11" or ".." in parts or ":" in member.filename:
                    raise RuntimeError("Unexpected path in NSIS archive")
            package.extractall(extracted)
        source = extracted / "nsis-3.11"
        plugin_dir = source / "Plugins/x86-unicode/additional"
        plugin_dir.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(plugin, plugin_dir / plugin.name)
        for relative in REQUIRED:
            if not (source / relative).is_file():
                raise RuntimeError(f"NSIS tool is missing: {relative}")
        tools_dir.mkdir(parents=True, exist_ok=True)
        target = tools_dir / "NSIS"
        if target.exists():
            shutil.rmtree(target)
        shutil.copytree(source, target)
        print("Verified NSIS tools ready before compilation", flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tools-dir", type=Path)
    args = parser.parse_args()
    # dirs::cache_dir() on Windows is LOCALAPPDATA; Tauri appends tauri/NSIS.
    tools_dir = args.tools_dir or Path(os.environ["LOCALAPPDATA"]) / "tauri"
    prepare(tools_dir)
