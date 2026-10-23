#!/usr/bin/env python3
"""Keep native test failures visible in GitHub check annotations as well as logs."""
import collections
import pathlib
import re
import shutil
import subprocess
import sys
from native_diagnostics import report

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

root = pathlib.Path(__file__).resolve().parent.parent
rustc = shutil.which("rustc")
cargo = shutil.which("cargo")
if not rustc or not cargo:
    raise RuntimeError("Rust toolchain is unavailable")
info = subprocess.check_output([rustc, "-vV"], text=True, encoding="utf-8")
host = re.search(r"^host: (\S+)$", info, re.M)[1]
command = [cargo, "test", "--quiet", "--manifest-path", "src-tauri/Cargo.toml", "--locked",
           "--release", "--target", host]
process = subprocess.Popen(command, cwd=root, stdout=subprocess.PIPE,
                           stderr=subprocess.STDOUT, text=True, encoding="utf-8", errors="replace")
tail = collections.deque(maxlen=100)
for line in process.stdout:
    print(line, end="", flush=True)
    tail.append(line)
status = process.wait()
if status:
    report("Native Rust failure (" + host + ")", "".join(tail))
sys.exit(status)
