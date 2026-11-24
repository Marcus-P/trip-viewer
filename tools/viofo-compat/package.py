#!/usr/bin/env python3
"""Package the Linux comparison worker with checksums and license notices."""
from pathlib import Path
import hashlib
import json
import shutil
import subprocess
import tarfile

root = Path(__file__).resolve().parent
out = root / 'package'
name = 'VIOFO-Compare-Ubuntu-24.04-x64-20261124'
bundle = out / name
bundle.mkdir(parents=True, exist_ok=False)
for source in ['run-compare.py', 'README.md', 'LICENSE', 'PROVENANCE.json']:
    shutil.copyfile(root / source, bundle / source)
shutil.copyfile(root / 'target/release/viofo-compare', bundle / 'viofo-compare')
(bundle / 'viofo-compare').chmod(0o755)
metadata = json.loads(subprocess.check_output(['cargo', 'metadata', '--locked', '--format-version', '1'], cwd=root))
notices = []
for pkg in sorted(metadata['packages'], key=lambda p: p['name']):
    if pkg['name'] == 'viofo-compare':
        continue
    notices.append(f"{pkg['name']} {pkg['version']} — {pkg.get('license', 'unspecified')} — {pkg.get('repository') or ''}")
    for item in Path(pkg['manifest_path']).parent.iterdir():
        if item.is_file() and item.name.upper().startswith(('LICENSE', 'COPYING', 'NOTICE', 'COPYRIGHT')):
            dest = bundle / 'dependency-licenses' / (pkg['name'] + '-' + pkg['version'])
            dest.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(item, dest / item.name)
(bundle / 'THIRD_PARTY_NOTICES.txt').write_text('VIOFO comparison tool dependency notices\n\n' + '\n'.join(notices) + '\n')
checks = [hashlib.sha256(p.read_bytes()).hexdigest() + '  ' + p.relative_to(bundle).as_posix()
          for p in sorted(bundle.rglob('*')) if p.is_file()]
(bundle / 'SHA256SUMS.txt').write_text('\n'.join(checks) + '\n')
archive = out / (name + '.tar.gz')
with tarfile.open(archive, 'w:gz') as tar:
    tar.add(bundle, arcname=name)
(out / 'SHA256SUMS-viofo-compare.txt').write_text(hashlib.sha256(archive.read_bytes()).hexdigest() + '  ' + archive.name + '\n')
