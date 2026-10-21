#!/usr/bin/env python3
"""Keep the Debian launcher aligned with the existing GTK/application identity."""
import hashlib
import pathlib
import re
import subprocess
import sys
import tempfile

package = pathlib.Path(sys.argv[1]).resolve()
with tempfile.TemporaryDirectory(prefix=".trip-viewer-deb-", dir=package.parent) as work:
    root = pathlib.Path(work)
    subprocess.run(["dpkg-deb", "--raw-extract", str(package), str(root)], check=True)
    applications = root / "usr/share/applications"
    entries = list(applications.glob("*.desktop"))
    if len(entries) != 1:
        raise RuntimeError("Expected one Trip Viewer desktop entry")
    source = entries[0]
    target = applications / "com.tripviewer.app.desktop"
    content = source.read_text()
    if "StartupWMClass=" in content:
        content = re.sub(r"^StartupWMClass=.*$", "StartupWMClass=com.tripviewer.app", content, flags=re.M)
    else:
        content += "\nStartupWMClass=com.tripviewer.app\n"
    if source != target:
        source.unlink()
    target.write_text(content)
    target.chmod(0o644)
    rows = []
    for file in sorted(root.rglob("*")):
        if not file.is_file() or file.is_symlink() or "DEBIAN" in file.relative_to(root).parts:
            continue
        rows.append(hashlib.md5(file.read_bytes()).hexdigest() + "  " + file.relative_to(root).as_posix())
    (root / "DEBIAN/md5sums").write_text("\n".join(rows) + "\n")
    result = root.parent / (root.name + ".deb")
    try:
        subprocess.run(["dpkg-deb", "--build", "--root-owner-group", str(root), str(result)], check=True)
        result.replace(package)
    finally:
        result.unlink(missing_ok=True)
print("Prepared stable GTK desktop identity in " + package.name)
