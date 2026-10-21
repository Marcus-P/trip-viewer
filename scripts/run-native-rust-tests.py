#!/usr/bin/env python3
"""Keep native test failures visible in GitHub check annotations as well as logs."""
import collections
import pathlib
import re
import shutil
import subprocess
import sys

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
    # Check annotations are truncated by GitHub. Remove successful test lines
    # and emit small parts so assertion details are retained in the Checks UI.
    details = "".join(line for line in tail if not
                      (line.startswith("test ") and line.rstrip().endswith("... ok")))[-12000:]
    for index, start in enumerate(range(0, len(details), 1000), 1):
        part = details[start:start + 1000]
        escaped = part.replace("%", "%25").replace("\r", "%0D").replace("\n", "%0A")
        print("::error title=Native Rust failure (" + host + ") part " + str(index) + "::" + escaped, flush=True)
sys.exit(status)
