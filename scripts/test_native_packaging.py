"""Small offline checks for the Windows tool bootstrap and failure annotations."""
import contextlib
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch
import zipfile


def load(name, filename):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(filename))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


nsis = load("windows_nsis", "prepare-windows-nsis.py")
diagnostics = load("native_diagnostics", "native_diagnostics.py")


class NativePackagingTests(unittest.TestCase):
    def test_download_retries_transport_failure_then_verifies(self):
        with tempfile.TemporaryDirectory() as folder:
            data = b"verified tool"
            asset = ("owner/repo", "tag", "tool.zip", hashlib.sha256(data).hexdigest())

            def download(*args, **kwargs):
                (Path(folder) / "tool.zip").write_bytes(data)

            with patch.object(nsis.subprocess, "run") as run, \
                 patch.object(nsis.time, "sleep") as sleep:
                def attempts(*args, **kwargs):
                    if run.call_count == 1:
                        raise subprocess.CalledProcessError(1, "gh")
                    download()
                run.side_effect = attempts
                self.assertEqual(nsis.download_asset(asset, Path(folder)).read_bytes(), data)
                self.assertEqual(run.call_count, 2)
                sleep.assert_called_once_with(5)
                self.assertEqual(run.call_args.kwargs["timeout"], 120)

    def test_transport_retries_are_bounded(self):
        with tempfile.TemporaryDirectory() as folder, \
             patch.object(nsis.subprocess, "run", side_effect=subprocess.TimeoutExpired("gh", 120)) as run, \
             patch.object(nsis.time, "sleep"):
            with self.assertRaises(subprocess.TimeoutExpired):
                nsis.download_asset(nsis.ASSETS[0], Path(folder))
            self.assertEqual(run.call_count, 3)

    def test_hash_mismatch_is_rejected_without_retry(self):
        with tempfile.TemporaryDirectory() as folder:
            def download(*args, **kwargs):
                (Path(folder) / "bad.zip").write_bytes(b"bad")
            with patch.object(nsis.subprocess, "run", side_effect=download) as run:
                with self.assertRaisesRegex(RuntimeError, "SHA-256 mismatch"):
                    nsis.download_asset(("owner/repo", "tag", "bad.zip", "0" * 64), Path(folder))
                self.assertEqual(run.call_count, 1)
                self.assertFalse((Path(folder) / "bad.zip").exists())

    def fixture(self, root, unsafe=False, incomplete=False):
        archive = root / "nsis-3.11.zip"
        with zipfile.ZipFile(archive, "w") as package:
            for relative in nsis.REQUIRED:
                if "additional" not in relative and not incomplete:
                    package.writestr("nsis-3.11/" + relative, b"tool")
            if unsafe:
                package.writestr("nsis-3.11/../../outside", b"bad")
        plugin = root / "nsis_tauri_utils.dll"
        plugin.write_bytes(b"plugin")
        return archive, plugin

    def prepare_fixture(self, root, **kwargs):
        assets = self.fixture(root, **kwargs)
        with patch.object(nsis.json, "loads", return_value={"version": nsis.CLI_VERSION}), \
             patch.object(nsis.Path, "read_text", return_value="{}"), \
             patch.object(nsis, "download_asset", side_effect=assets):
            nsis.prepare(root / "tools")

    def test_complete_tools_and_plugin_are_installed_in_tauri_directory(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            self.prepare_fixture(root)
            for relative in nsis.REQUIRED:
                self.assertTrue((root / "tools/NSIS" / relative).is_file(), relative)

    def test_unsafe_archive_is_rejected_before_tool_installation(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            with self.assertRaisesRegex(RuntimeError, "Unexpected path"):
                self.prepare_fixture(root, unsafe=True)
            self.assertFalse((root / "tools").exists())
            self.assertFalse((root / "outside").exists())

    def test_incomplete_tools_are_rejected(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            with self.assertRaisesRegex(RuntimeError, "missing"):
                self.prepare_fixture(root, incomplete=True)
            self.assertFalse((root / "tools").exists())

    def test_cli_upgrade_requires_pin_review_before_network_request(self):
        with patch.object(nsis.Path, "read_text", return_value=json.dumps({"version": "9.0.0"})), \
             patch.object(nsis, "download_asset") as download:
            with self.assertRaisesRegex(RuntimeError, "Tauri CLI changed"):
                nsis.prepare(Path("unused"))
            download.assert_not_called()

    def test_real_failure_remains_in_first_ten_annotations(self):
        output = io.StringIO()
        with contextlib.redirect_stdout(output):
            diagnostics.report("Build", "Compiling dependency\n" * 2000 + "failed: HTTP 504 100%\n")
        annotations = output.getvalue().splitlines()
        self.assertLessEqual(len(annotations), 10)
        self.assertIn("failed: HTTP 504 100%25%0A", annotations[-1])


if __name__ == "__main__":
    unittest.main()
