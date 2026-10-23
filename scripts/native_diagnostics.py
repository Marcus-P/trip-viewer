#!/usr/bin/env python3
"""Make native failures readable in check annotations without large log uploads."""
import pathlib
import re
import sys
import traceback


def report(title, details):
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    details = re.sub(r"\x1b\[[0-?]*[ -/]*[@-~]", "", details)
    details = "".join(line for line in details.splitlines(keepends=True) if not
                      (line.startswith("test ") and line.rstrip().endswith("... ok")))[-12000:]
    for index, start in enumerate(range(0, len(details), 1000), 1):
        part = details[start:start + 1000]
        escaped = part.replace("%", "%25").replace("\r", "%0D").replace("\n", "%0A")
        print("::error title=" + title + " part " + str(index) + "::" + escaped, flush=True)


def annotate_exceptions(title):
    original = sys.excepthook

    def hook(kind, value, trace):
        report(title, "".join(traceback.format_exception(kind, value, trace)))
        original(kind, value, trace)
    sys.excepthook = hook


if __name__ == "__main__":
    report(sys.argv[2], pathlib.Path(sys.argv[1]).read_text(encoding="utf-8", errors="replace"))
