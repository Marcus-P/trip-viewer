import test from "node:test";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, writeFileSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const script = fileURLToPath(new URL("./commit-privacy.mjs", import.meta.url));
const address = "private-fixture@example.invalid";
function repository(t, setup = true) {
  const cwd = mkdtempSync(join(tmpdir(), "tripviewer-privacy-"));
  t.after(() => rmSync(cwd, { recursive: true, force: true }));
  const run = (command, args, options = {}) => spawnSync(command, args, {
    cwd, encoding: "utf8", ...options,
    env: { ...process.env, GIT_CONFIG_GLOBAL: "/dev/null", GIT_CONFIG_NOSYSTEM: "1", ...options.env },
  });
  const git = (...args) => run("git", args);
  assert.equal(git("init", "-q").status, 0);
  assert.equal(git("remote", "add", "origin", "https://github.com/Marcus-P/trip-viewer.git").status, 0);
  if (setup) assert.equal(run(process.execPath, [script, "setup"]).status, 0);
  git("config", "privacy.blockedEmail", address);
  git("config", "commit.gpgsign", "false");
  return { cwd, run, git };
}

test("safe owner commits and preserved contributor authorship are accepted", (t) => {
  const { git } = repository(t);
  assert.equal(git("config", "user.email").stdout.trim(), "14142325+Marcus-P@users.noreply.github.com");
  assert.equal(git("commit", "--allow-empty", "-m", "safe").status, 0);
  assert.equal(git("commit", "--allow-empty", "--author=Contributor <contributor@example.invalid>", "-m", "contribution").status, 0);
  assert.equal(git("log", "-1", "--format=%ae").stdout.trim(), "contributor@example.invalid");
});

test("wrong committer, private author, staged content and message are rejected without disclosure", (t) => {
  const { cwd, run, git } = repository(t);
  for (const variable of ["GIT_COMMITTER_EMAIL", "GIT_AUTHOR_EMAIL"]) {
    const r = run("git", ["commit", "--allow-empty", "-m", "safe"], { env: { [variable]: address } });
    assert.notEqual(r.status, 0);
    assert.ok(!(r.stdout + r.stderr).includes(address));
  }
  writeFileSync(join(cwd, "fixture.txt"), address);
  git("add", "fixture.txt");
  assert.notEqual(git("commit", "-m", "safe").status, 0);
  git("rm", "--cached", "fixture.txt");
  const r = git("commit", "--allow-empty", "-m", address);
  assert.notEqual(r.status, 0);
  assert.ok(!(r.stdout + r.stderr).includes(address));
});

test("push checks new commits while tolerating unchanged published history", (t) => {
  const { run, git } = repository(t);
  assert.equal(run("git", ["-c", "core.hooksPath=/dev/null", "commit", "--allow-empty", "-m", "old"],
    { env: { GIT_AUTHOR_EMAIL: address } }).status, 0);
  const old = git("rev-parse", "HEAD").stdout.trim();
  const input = (sha, remote) => `refs/heads/test ${sha} refs/heads/test ${remote}\n`;
  const blocked = run(process.execPath, [script, "push"], { input: input(old, "0".repeat(40)) });
  assert.notEqual(blocked.status, 0);
  assert.ok(!blocked.stderr.includes(address));
  git("update-ref", "refs/remotes/origin/main", old);
  assert.equal(git("commit", "--allow-empty", "-m", "new safe commit").status, 0);
  const head = git("rev-parse", "HEAD").stdout.trim();
  assert.equal(run(process.execPath, [script, "push"], { input: input(head, old) }).status, 0);
});

test("setup preserves an existing hook", (t) => {
  const { cwd, run } = repository(t, false);
  const hook = join(cwd, ".git", "hooks", "pre-push");
  writeFileSync(hook, "# existing hook\n");
  assert.notEqual(run(process.execPath, [script, "setup"]).status, 0);
  assert.equal(readFileSync(hook, "utf8"), "# existing hook\n");
});
