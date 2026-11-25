import test from "node:test";
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { upstreamStatus, needsCurrentUpstream } from "./check-upstream.mjs";

test("freshness distinguishes divergence from containing current upstream", t => {
  const cwd = mkdtempSync(join(tmpdir(), "tripviewer-upstream-"));
  t.after(() => rmSync(cwd, { recursive: true, force: true }));
  const env = { ...process.env, GIT_AUTHOR_NAME: "Fixture", GIT_COMMITTER_NAME: "Fixture",
    GIT_AUTHOR_EMAIL: "fixture@example.invalid", GIT_COMMITTER_EMAIL: "fixture@example.invalid" };
  const git = (...args) => execFileSync("git", args, { cwd, env, encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] }).trim();
  git("init");
  git("-c", "commit.gpgsign=false", "commit", "--allow-empty", "-m", "base");
  const base = git("rev-parse", "HEAD");
  git("checkout", "-b", "upstream");
  git("-c", "commit.gpgsign=false", "commit", "--allow-empty", "-m", "upstream addition");
  const upstream = git("rev-parse", "HEAD");
  git("checkout", "-b", "fork", base);
  git("-c", "commit.gpgsign=false", "commit", "--allow-empty", "-m", "fork addition");
  const before = git("rev-parse", "HEAD");
  assert.deepEqual([upstreamStatus(upstream, "HEAD", cwd).behind, upstreamStatus(upstream, "HEAD", cwd).ahead], [1, 1]);
  assert.equal(git("rev-parse", "HEAD"), before, "check must not modify HEAD");
  git("-c", "commit.gpgsign=false", "merge", "--no-edit", upstream);
  assert.equal(upstreamStatus(upstream, "HEAD", cwd).current, true);
});

test("code and packaging require freshness; policy-only checks report drift", () => {
  for (const key of ["frontend", "backend", "bundle", "audit"]) assert.equal(needsCurrentUpstream({ [key]: true }), true);
  assert.equal(needsCurrentUpstream({ docs: true, tooling: true }), false);
});
