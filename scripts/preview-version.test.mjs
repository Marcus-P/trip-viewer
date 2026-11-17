import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';
import { execFileSync } from 'node:child_process';
import { previewVersion, setVersion } from './preview-version.mjs';

test('acceptance versions remain below their stable release and retain unique run IDs', () => {
  assert.equal(previewVersion('0.12.0-rc.1', '44152470559'), '0.12.0-rc.44152470559');
  assert.equal(previewVersion('0.12.0', '44152470560'), '0.12.0-rc.44152470560');
  for (const value of ['0', '-1', 'abc', '1+sha', '01']) assert.throws(() => previewVersion('0.12.0', value));
  if (process.platform === 'linux') {
    execFileSync('dpkg', ['--compare-versions', '0.12.0~rc.44152470559', 'lt', '0.12.0']);
    execFileSync('dpkg', ['--compare-versions', '0.11.0', 'lt', '0.12.0~rc.44152470559']);
  }
});

test('Windows CRLF stamping and finalizing preserve dependencies, history and version consistency', () => {
  const root = mkdtempSync(join(tmpdir(), 'trip-preview-'));
  try {
    mkdirSync(join(root, 'src-tauri'));
    const write = (name, value) => writeFileSync(join(root, name), value);
    const read = (name) => readFileSync(join(root, name), 'utf8');
    write('package.json', JSON.stringify({version:'0.12.0-rc.1', dependencies:{fixture:'1.0.0'}}));
    write('package-lock.json', JSON.stringify({version:'0.12.0-rc.1', packages:{'':{version:'0.12.0-rc.1'}, fixture:{version:'1.0.0'}}}));
    write('src-tauri/Cargo.toml', '[package]\nname = "tripviewer"\nversion = "0.12.0-rc.1"\n[dependencies]\nfixture = "1.0.0"\n');
    write('src-tauri/Cargo.lock', '[[package]]\nname = "tripviewer"\nversion = "0.12.0-rc.1"\n[[package]]\nname = "fixture"\nversion = "1.0.0"\n');
    write('RELEASE_VERSION', '0.12.0-rc.1\n');
    write('RELEASE_NOTES.md', '# Trip Viewer 0.12.0-rc.1\n\nChanges\n\n# Trip Viewer 0.11.0\nHistory\n');
    // Windows Git checkouts may use CRLF even when repository blobs use LF.
    for (const name of ['package.json', 'package-lock.json', 'src-tauri/Cargo.toml', 'src-tauri/Cargo.lock', 'RELEASE_VERSION', 'RELEASE_NOTES.md']) {
      write(name, read(name).replace(/\n/g, '\r\n'));
    }
    const url = pathToFileURL(root + '/');
    for (const version of ['0.12.0-rc.44152470559', '0.12.0']) {
      setVersion(url, version);
      assert.equal(JSON.parse(read('package.json')).version, version);
      assert.equal(JSON.parse(read('package-lock.json')).packages[''].version, version);
      assert.ok(read('src-tauri/Cargo.toml').includes('version = "' + version + '"'));
      assert.ok(read('src-tauri/Cargo.lock').includes('name = "tripviewer"\nversion = "' + version + '"'));
      assert.equal(read('RELEASE_VERSION'), version + '\n');
      assert.ok(read('RELEASE_NOTES.md').startsWith('# Trip Viewer ' + version + '\n'));
      assert.ok(read('RELEASE_NOTES.md').includes('# Trip Viewer 0.11.0\nHistory'));
      assert.equal(JSON.parse(read('package.json')).dependencies.fixture, '1.0.0');
      assert.equal(JSON.parse(read('package-lock.json')).packages.fixture.version, '1.0.0');
    }
  } finally { rmSync(root, { recursive:true, force:true }); }
});

test('acceptance pipelines stamp versions, stable releases keep committed metadata', () => {
  const desktop = readFileSync(new URL('../.github/workflows/desktop-packages.yml', import.meta.url), 'utf8');
  const ci = readFileSync(new URL('../.github/workflows/ci.yml', import.meta.url), 'utf8');
  assert.match(desktop, /if: inputs.acceptance\s+shell: bash\s+run: node scripts\/preview-version.mjs/);
  assert.match(ci, /if: needs.scope.outputs.bundle == 'true'\s+run: node scripts\/preview-version.mjs/);
  assert.match(desktop, /acceptance:\s+type: boolean\s+default: false/);
});
