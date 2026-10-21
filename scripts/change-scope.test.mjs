import { test } from "node:test";
import assert from "node:assert/strict";
import { classifyChanges } from "./change-scope.mjs";

test("documentation requires no runtime tests, audit or bundle", () => {
  const s = classifyChanges([{ path: "README.md" }, { path: "docs/history/notes.md" }, { path: ".gitignore" }]);
  assert.equal(s.docs_only, true);
  for (const key of ["frontend", "backend", "bundle", "audit", "tooling"]) assert.equal(s[key], false);
});

test("version-only metadata plus notes stays lightweight; dependency changes do not", () => {
  const before = JSON.stringify({ version: "0.6.0", dependencies: { react: "19" } });
  const after = JSON.stringify({ version: "0.6.1", dependencies: { react: "19" } });
  const s = classifyChanges([{ path: "package.json", before, after }, { path: "RELEASE_NOTES.md" }]);
  assert.equal(s.metadata_only, true);
  assert.equal(s.bundle, false);
  const deps = classifyChanges([{ path: "package.json", before,
    after: JSON.stringify({ version: "0.6.1", dependencies: { react: "20" } }) }]);
  assert.equal(deps.frontend, true);
  assert.equal(deps.audit, true);
  assert.equal(deps.bundle, true);
});

test("backend changes do not request frontend unit tests or npm audit", () => {
  const s = classifyChanges([{ path: "src-tauri/src/gps/mod.rs" }]);
  assert.equal(s.backend, true);
  assert.equal(s.bundle, true);
  assert.equal(s.frontend, false);
  assert.equal(s.audit, false);
});

test("frontend utilities and desktop media changes have different build needs", () => {
  const util = classifyChanges([{ path: "src/utils/format.ts" }]);
  assert.equal(util.frontend, true);
  assert.equal(util.bundle, false);
  const media = classifyChanges([{ path: "src/components/video/VideoGrid.tsx" }]);
  assert.equal(media.frontend, true);
  assert.equal(media.bundle, true);
  assert.equal(media.backend, false);
});

test("unknown files, renamed docs into runtime paths, and workflow changes cannot skip tests", () => {
  for (const path of ["new-build-file", ".github/workflows/ci.yml", "src/unusual.md"]) {
    const s = classifyChanges([{ path: "docs/old.md" }, { path }]);
    assert.equal(s.docs_only, false);
    assert.ok(s.frontend || s.backend || s.bundle);
  }
});

test("Cargo dependency changes are not confused with package-version changes", () => {
  const before = '[package]\nname = "tripviewer"\nversion = "0.6.0"\n\n[dependencies]\nserde = "1"\n';
  const after = before.replace("0.6.0", "0.6.1");
  assert.equal(classifyChanges([{ path: "src-tauri/Cargo.toml", before, after }]).metadata_only, true);
  const changed = after.replace('serde = "1"', 'serde = "2"');
  assert.equal(classifyChanges([{ path: "src-tauri/Cargo.toml", before, after: changed }]).backend, true);
});

test("changing only the frontend test runner does not request packaging or a dependency audit", () => {
  const before = JSON.stringify({ version: "0.6.0", scripts: { test: "old" } });
  const after = JSON.stringify({ version: "0.6.1", scripts: { test: "new" } });
  const s = classifyChanges([{ path: "package.json", before, after }]);
  assert.equal(s.frontend, true);
  assert.equal(s.bundle, false);
  assert.equal(s.audit, false);
});

test("standalone frontend and backend tests do not build a desktop package", () => {
  const frontend = classifyChanges([{ path: "src/components/video/health.test.ts" }]);
  assert.equal(frontend.frontend, true);
  assert.equal(frontend.bundle, false);
  const backend = classifyChanges([{ path: "src-tauri/tests/archive.rs" }]);
  assert.equal(backend.backend, true);
  assert.equal(backend.bundle, false);
});

test("native build-script changes still run Rust tests and package validation", () => {
  const s = classifyChanges([{ path: "src-tauri/build.rs" }]);
  assert.equal(s.backend, true);
  assert.equal(s.bundle, true);
});
