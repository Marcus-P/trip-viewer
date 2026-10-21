import { test } from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { execFileSync, spawnSync } from "node:child_process";

function fixture(run) {
  const root = mkdtempSync(join(tmpdir(), "trip-version-policy-"));
  try {
    mkdirSync(join(root, "scripts"));
    mkdirSync(join(root, "src-tauri"));
    writeFileSync(join(root, "scripts/check-version.mjs"),
      readFileSync(new URL("./check-version.mjs", import.meta.url)));
    const write = (path, text) => writeFileSync(join(root, path), text);
    const version = (value) => {
      write("package.json", JSON.stringify({ version: value }));
      write("package-lock.json", JSON.stringify({ version: value, packages: { "": { version: value } } }));
      write("src-tauri/Cargo.toml", '[package]\nversion = "' + value + '"\n[dependencies]\n');
      write("src-tauri/Cargo.lock", '[[package]]\nname = "tripviewer"\nversion = "' + value + '"\n');
      write("src-tauri/tauri.conf.json", JSON.stringify({ version: "../package.json" }));
      write("RELEASE_VERSION", value + "\n");
    };
    version("0.6.0");
    write("RELEASE_NOTES.md", "# Trip Viewer 0.6.0\n\nReleased changes\n");
    execFileSync("git", ["init", "-q"], { cwd: root });
    execFileSync("git", ["add", "."], { cwd: root });
    execFileSync("git", ["-c", "user.name=Policy test", "-c", "user.email=test@example.invalid",
      "commit", "-qm", "Baseline"], { cwd: root });
    const base = execFileSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" }).trim();
    const check = () => spawnSync(process.execPath, ["scripts/check-version.mjs"], {
      cwd: root, encoding: "utf8", env: { ...process.env, VERSION_BASE_SHA: base },
    });
    run({ write, version, check });
  } finally { rmSync(root, { recursive: true, force: true }); }
}

test("one patch increment with matching release notes is accepted", () => fixture(({ version, write, check }) => {
  version("0.6.1");
  write("RELEASE_NOTES.md", "# Trip Viewer 0.6.1\n\nDocumentation correction\n");
  assert.equal(check().status, 0);
}));

test("a version bump without its release notes is rejected", () => fixture(({ version, check }) => {
  version("0.6.1");
  const result = check();
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /RELEASE_NOTES/);
}));

test("a docs-only merge still needs a version increase", () => fixture(({ write, check }) => {
  write("README.md", "Corrected documentation\n");
  write("RELEASE_NOTES.md", "# Trip Viewer 0.6.0\n\nCorrected docs\n");
  const result = check();
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /version must increase/);
}));

test("skipped version numbers and unsynchronized metadata are rejected", () => fixture(({ version, write, check }) => {
  version("0.6.2");
  write("RELEASE_NOTES.md", "# Trip Viewer 0.6.2\n\nChanges\n");
  assert.notEqual(check().status, 0);
  version("0.6.1");
  write("RELEASE_NOTES.md", "# Trip Viewer 0.6.1\n\nChanges\n");
  write("RELEASE_VERSION", "0.7.0\n");
  assert.match(check().stderr, /differs/);
}));
