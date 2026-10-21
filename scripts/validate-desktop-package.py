#!/usr/bin/env python3
"""Verify native package payload, identity, architecture and a basic app launch.

A running empty-window app is not a recording/codec/GPS functional certification.
"""
import argparse
import hashlib
import json
import os
import pathlib
import plistlib
import struct
import subprocess
import sys
import tempfile
import time

from native_diagnostics import annotate_exceptions

annotate_exceptions("Native package validation")

parser = argparse.ArgumentParser()
parser.add_argument("platform", choices=["Windows-x64", "macOS-universal"])
parser.add_argument("--installed", type=pathlib.Path)
args = parser.parse_args()
root = pathlib.Path(__file__).resolve().parent.parent
version = json.loads((root / "package.json").read_text())["version"]
notices = ["LICENSE", "THIRD_PARTY_NOTICES.md", "NODE_RUNTIME_LICENSES.txt",
           "RUST_DEPENDENCY_LICENSES.txt", "NODE_RUNTIME_LICENSE_TEXTS.tar.gz",
           "RUST_DEPENDENCY_LICENSE_TEXTS.tar.gz"]


def check_notices(directory):
    for name in notices:
        assert (directory / name).read_bytes() == (root / "release-assets" / name).read_bytes(), name


def checked(command):
    result = subprocess.run(command, capture_output=True, text=True, encoding="utf-8", errors="replace")
    if result.returncode:
        raise RuntimeError("Command failed: " + repr(command) + "\n" + result.stdout + result.stderr)


def smoke(binary):
    with tempfile.TemporaryFile() as log:
        process = subprocess.Popen([str(binary)], stdout=log, stderr=log)
        try:
            deadline = time.monotonic() + 12
            while time.monotonic() < deadline:
                if process.poll() is not None:
                    log.seek(0)
                    raise RuntimeError("App exited during basic launch: " +
                                       log.read().decode("utf-8", errors="replace")[-6000:])
                time.sleep(0.5)
        finally:
            if process.poll() is None:
                process.terminate()
                try:
                    process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait()


if args.platform == "Windows-x64":
    assert sys.platform == "win32" and args.installed
    binary = args.installed / "tripviewer.exe"
    with binary.open("rb") as file:
        assert file.read(2) == b"MZ"
        file.seek(0x3C)
        header = struct.unpack("<I", file.read(4))[0]
        file.seek(header)
        assert file.read(4) == b"PE\0\0"
        assert struct.unpack("<H", file.read(2))[0] == 0x8664, "Expected x64 PE"
        file.seek(header + 24 + 68)
        assert struct.unpack("<H", file.read(2))[0] == 2, "Expected GUI subsystem"
    check_notices(args.installed / "licenses")
    shortcut = pathlib.Path(os.environ["APPDATA"]) / "Microsoft/Windows/Start Menu/Programs/Trip Viewer.lnk"
    assert shortcut.is_file(), "Missing Start Menu shortcut"
    smoke(binary)
else:
    assert sys.platform == "darwin"
    bundle = root / "src-tauri/target/universal-apple-darwin/release/bundle"
    app = bundle / "macos/Trip Viewer.app"
    info = plistlib.loads((app / "Contents/Info.plist").read_bytes())
    assert info["CFBundleIdentifier"] == "com.tripviewer.app"
    assert info["CFBundleShortVersionString"] == version
    binary = app / "Contents/MacOS" / info["CFBundleExecutable"]
    checked(["lipo", "-verify_arch", "arm64", "x86_64", str(binary)])
    checked(["codesign", "--verify", "--deep", "--strict", str(app)])
    check_notices(app / "Contents/Resources/licenses")
    assert list((app / "Contents/Resources").glob("*.icns")), "Missing original icon"
    images = list((bundle / "dmg").glob("*.dmg"))
    assert len(images) == 1
    with tempfile.TemporaryDirectory(prefix="trip-viewer-dmg-") as mount:
        checked(["hdiutil", "attach", str(images[0]), "-readonly", "-nobrowse", "-mountpoint", mount])
        try:
            mounted = pathlib.Path(mount) / "Trip Viewer.app"
            other = mounted / "Contents/MacOS" / info["CFBundleExecutable"]
            assert other.read_bytes() == binary.read_bytes(), "DMG payload differs"
            assert (pathlib.Path(mount) / "Applications").is_symlink()
            check_notices(mounted / "Contents/Resources/licenses")
        finally:
            checked(["hdiutil", "detach", mount])
    smoke(binary)
print(args.platform + ": package identity, architecture, notices and basic launch pass")
