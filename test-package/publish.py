"""Publish the already-checked test package; never rebuild or touch stable assets."""

import json
from pathlib import Path
import subprocess
import tarfile

repo = "Marcus-P/trip-viewer"
release_id = "458179709"
tag = "pr21-test-17f6f97"
root = Path("test-package")
archive = root / "out/TripViewer-PR21-Test-17f6f97-Ubuntu-x86_64.tar.gz"
with tarfile.open(archive) as bundle:
    info = json.load(bundle.extractfile("TripViewer-PR21-Test-17f6f97/BUILD-INFO.json"))
assert info["archive_schema_version"] == 13
assert info["bundled_gl_factories_checked"] is True
assert info["updater_endpoints"] == []


def api(*args):
    return json.loads(subprocess.check_output(["gh", "api", *args], text=True))


release = api(f"repos/{repo}/releases/{release_id}")
assert release["draft"], "Refuse to modify an already-published test release"
payload = {
    "tag_name": tag,
    "target_commitish": info["packaging_commit"],
    "name": "PR21 isolated Ubuntu playback test — 17f6f97",
    "draft": True,
    "prerelease": True,
    "make_latest": "false",
    "body": (root / "release-notes.md").read_text(),
}
request_file = root / "out/release-request.json"
request_file.write_text(json.dumps(payload))
release = api(
    "--method",
    "PATCH",
    f"repos/{repo}/releases/{release_id}",
    "--input",
    str(request_file),
)
assert release["tag_name"] == tag and release["draft"]
assert release["target_commitish"] == info["packaging_commit"]
subprocess.run(
    [
        "gh",
        "release",
        "upload",
        tag,
        "--repo",
        repo,
        str(archive),
        str(root / "out/TEST-PACKAGE-SHA256SUMS.txt"),
        str(root / "tauri.test.conf.json"),
        str(root / "PR21-description.md"),
        str(root / "PR21-A129-test-request.md"),
    ],
    check=True,
)
payload["draft"] = False
request_file.write_text(json.dumps(payload))
release = api(
    "--method",
    "PATCH",
    f"repos/{repo}/releases/{release_id}",
    "--input",
    str(request_file),
)
assert release["tag_name"] == tag and not release["draft"] and release["prerelease"]
assert api(f"repos/{repo}/releases/latest")["tag_name"] == "v0.13.0"
print(release["html_url"])
