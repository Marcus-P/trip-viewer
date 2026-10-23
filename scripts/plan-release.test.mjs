import { test } from "node:test";
import assert from "node:assert/strict";
import { planRelease } from "./plan-release.mjs";

const sha = "a".repeat(40);
const other = "b".repeat(40);
test("a version merge publishes one stable release", () => {
  assert.deepEqual(planRelease({ version: "0.8.1", parentVersion: "0.8.0", sha }),
    { version: "0.8.1", tag: "v0.8.1", prerelease: false });
});
test("an unchanged-version push gets immutable downloads even before the stable tag exists", () => {
  for (const versionTagCommit of [null, other]) {
    assert.deepEqual(planRelease({ version: "0.8.1", parentVersion: "0.8.1", sha, versionTagCommit }),
      { version: "0.8.1", tag: `main-${sha}`, prerelease: true });
  }
});
test("two unchanged-version commits never share a download tag", () => {
  const plan = (sha) => planRelease({ version: "0.8.1", parentVersion: "0.8.1", sha });
  assert.notEqual(plan(sha).tag, plan(other).tag);
});
test("a published stable commit keeps its tag on retry, never becoming a snapshot", () => {
  assert.equal(planRelease({ version: "0.8.1", parentVersion: "0.8.1", sha,
    versionTagCommit: sha }).tag, "v0.8.1");
});
test("a version-changing merge cannot overwrite another commit's stable version", () => {
  assert.throws(() => planRelease({ version: "0.8.1", parentVersion: "0.8.0", sha,
    versionTagCommit: other }), /another commit/);
});
test("invalid source identities and unstable versions are rejected", () => {
  assert.throws(() => planRelease({ version: "0.8.1", sha: "main" }), /full commit/);
  assert.throws(() => planRelease({ version: "0.8.1-beta", sha }), /stable/);
});
