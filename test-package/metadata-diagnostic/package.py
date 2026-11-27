"""Build the small source-only diagnostic ZIP; never include owner results."""
import hashlib
import json
from pathlib import Path
import zipfile

ROOT = Path(__file__).resolve().parent
COMMON = ROOT.parent / 'parking-diagnostic'
FILES = {
    **{name: ROOT / name for name in ('capture.py', 'run-windows.ps1', 'README.md')},
    **{name: COMMON / name for name in ('diagnose.py', 'parking_analysis.py', 'time_core.py', 'LICENSE.txt')},
}
contents = {name: path.read_bytes() for name, path in FILES.items()}
contents['SHA256SUMS.json'] = (json.dumps({name: hashlib.sha256(value).hexdigest()
    for name, value in contents.items()}, indent=2) + '\n').encode()
target = ROOT.parent / 'viofo-metadata-diagnostic-v1.zip'
with zipfile.ZipFile(target, 'w', compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
    for name, value in sorted(contents.items()):
        info = zipfile.ZipInfo(name, date_time=(2020, 1, 1, 0, 0, 0))
        info.compress_type = zipfile.ZIP_DEFLATED
        info.external_attr = 0o100644 << 16
        archive.writestr(info, value)
print(target.name, target.stat().st_size, hashlib.sha256(target.read_bytes()).hexdigest())
