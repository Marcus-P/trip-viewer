#!/usr/bin/env python3
"""Read package metadata and payload without installing into the developer's OS."""
import json
import pathlib
import subprocess
import sys
import tarfile

root = pathlib.Path(__file__).resolve().parent.parent
package = pathlib.Path(sys.argv[1]).resolve()
version = json.loads((root / "package.json").read_text())["version"]
def field(name):
    return subprocess.check_output(["dpkg-deb", "--field", str(package), name], text=True).strip()
assert field("Package") == "trip-viewer", "Unexpected Debian package identity"
assert field("Version") == version, "Package and application versions differ"
assert field("Architecture") == "amd64", "Only Ubuntu x86_64 is published"
depends = field("Depends")
for name in ["gstreamer1.0-libav", "gstreamer1.0-plugins-good",
             "gstreamer1.0-plugins-bad", "gstreamer1.0-plugins-ugly"]:
    assert name in depends, "Missing playback dependency: " + name
required = {"usr/bin/tripviewer"}
for name in ["LICENSE", "THIRD_PARTY_NOTICES.md", "NODE_RUNTIME_LICENSES.txt",
             "RUST_DEPENDENCY_LICENSES.txt", "NODE_RUNTIME_LICENSE_TEXTS.tar.gz",
             "RUST_DEPENDENCY_LICENSE_TEXTS.tar.gz"]:
    required.add("usr/share/doc/trip-viewer/" + name)
desktop = False
icon = False
process = subprocess.Popen(["dpkg-deb", "--fsys-tarfile", str(package)], stdout=subprocess.PIPE)
try:
    with tarfile.open(fileobj=process.stdout, mode="r|") as archive:
        for member in archive:
            name = member.name.removeprefix("./")
            if name in required:
                assert member.isfile() and member.size > 0, "Empty payload: " + name
                required.remove(name)
            if name.startswith("usr/share/applications/") and name.endswith(".desktop"):
                assert name == "usr/share/applications/com.tripviewer.app.desktop", "Desktop/app identity mismatch"
                entry = archive.extractfile(member).read().decode()
                assert "Exec=tripviewer" in entry or "Exec=/usr/bin/tripviewer" in entry
                assert "Icon=" in entry and "Trip Viewer" in entry
                assert "StartupWMClass=com.tripviewer.app" in entry
                desktop = True
            if name.startswith("usr/share/icons/") and name.endswith(".png"):
                icon = True
finally:
    process.stdout.close()
    if process.wait() != 0:
        raise RuntimeError("Cannot read Debian package payload")
assert not required, "Missing package files: " + ", ".join(sorted(required))
assert desktop and icon, "Desktop entry or original icon missing"
print("Validated Ubuntu installer " + package.name + " (" + version + "), playback dependencies and notices")
