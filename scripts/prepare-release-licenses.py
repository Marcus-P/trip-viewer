#!/usr/bin/env python3
"""Prepare the same complete notice material for GitHub assets and Debian bundles."""
import json
import pathlib
import re
import shutil
import subprocess
import tarfile
import tempfile

root = pathlib.Path(__file__).resolve().parent.parent
out = root / "release-assets"
out.mkdir(exist_ok=True)
version = json.loads((root / "package.json").read_text())["version"]
lock = json.loads((root / "package-lock.json").read_text())
metadata = json.loads(subprocess.check_output(
    ["cargo", "metadata", "--locked", "--format-version", "1",
     "--manifest-path", str(root / "src-tauri/Cargo.toml")], cwd=root))

def inventory(name, heading, rows):
    (out / name).write_text(heading + "\nGenerated for Trip Viewer " + version +
                           "\n\n" + "\n".join("\t".join(row) for row in sorted(rows)) + "\n")

def normalized(info):
    info.uid = info.gid = 0
    info.uname = info.gname = ""
    return info

def copy_notices(source, dest):
    found = False
    for p in sorted(source.iterdir()):
        if p.is_file() and re.match(r"^(license|licence|copying|notice)([.\-]|$)", p.name, re.I):
            dest.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(p, dest / p.name)
            found = True
    return found

with tempfile.TemporaryDirectory(prefix="trip-viewer-licenses-") as work:
    temp = pathlib.Path(work)
    node_dir = temp / "node"
    node_dir.mkdir()
    node_rows = []
    found = set()
    for path, package in lock["packages"].items():
        if not path.startswith("node_modules/") or package.get("dev"):
            continue
        name = path.removeprefix("node_modules/")
        rev = package.get("version", "?")
        node_rows.append((name, rev, package.get("license") or "UNKNOWN"))
        source = root / path
        if source.is_dir() and copy_notices(source, node_dir / (name + "@" + rev).replace("/", "__")):
            found.add(name)
    for required in ["react-leaflet", "@react-leaflet/core"]:
        if required not in found:
            raise RuntimeError("Required runtime license text missing: " + required)
    inventory("NODE_RUNTIME_LICENSES.txt", "Node runtime dependency license inventory", node_rows)

    rust_dir = temp / "rust"
    rust_dir.mkdir()
    rust_rows = []
    for package in metadata["packages"]:
        if package["name"] == "tripviewer":
            continue
        name, rev = package["name"], package["version"]
        rust_rows.append((name, rev, package.get("license") or "UNKNOWN",
                          package.get("repository") or package.get("source") or ""))
        copy_notices(pathlib.Path(package["manifest_path"]).parent,
                     rust_dir / (name + "@" + rev).replace("/", "__"))
    inventory("RUST_DEPENDENCY_LICENSES.txt", "Rust dependency license inventory", rust_rows)
    for name, directory in [("NODE_RUNTIME_LICENSE_TEXTS.tar.gz", node_dir),
                            ("RUST_DEPENDENCY_LICENSE_TEXTS.tar.gz", rust_dir)]:
        with tarfile.open(out / name, "w:gz") as archive:
            archive.add(directory, arcname=".", filter=normalized)
for name in ["LICENSE", "THIRD_PARTY_NOTICES.md"]:
    shutil.copyfile(root / name, out / name)
print("Prepared application and dependency notices in release-assets")
