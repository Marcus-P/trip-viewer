import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { resolve, join } from "node:path";
import { fileURLToPath } from "node:url";

// Owner-specific opt-in hooks. Private addresses belong in local Git config,
// never in this source file or in diagnostics printed by the hooks.
const marker = "# Trip Viewer commit privacy guard";
const ownerEmail = "14142325+Marcus-P@users.noreply.github.com";
const git = (...args) => execFileSync("git", args, {
  encoding: "utf8", stdio: ["ignore", "pipe", "pipe"],
}).trimEnd();
const config = (key) => {
  try { return git("config", "--get-all", key).split("\n").filter(Boolean); }
  catch { return []; }
};
const quote = (text) => "'" + text.replaceAll("'", "'\\''") + "'";
const fail = (message) => { throw Error(message); };

function install() {
  if (!/github\.com[:/]Marcus-P\/trip-viewer(?:\.git)?$/i.test(git("remote", "get-url", "origin"))) {
    fail("This setup is for the owner's fork; other projects choose their own policy.");
  }
  if (config("core.hooksPath").length) fail("Existing hooksPath needs manual integration; it was not changed.");
  const common = resolve(git("rev-parse", "--git-common-dir"));
  const hooks = join(common, "hooks");
  const modes = { "pre-commit": "commit", "commit-msg": "message", "pre-push": "push" };
  for (const name of Object.keys(modes)) {
    const path = join(hooks, name);
    if (existsSync(path) && !readFileSync(path, "utf8").includes(marker)) {
      fail("Existing " + name + " hook needs manual integration; no hooks were changed.");
    }
  }
  mkdirSync(hooks, { recursive: true });
  const helper = join(common, "privacy-guard.mjs");
  const source = readFileSync(fileURLToPath(import.meta.url), "utf8");
  writeFileSync(helper, source, { mode: 0o600 });
  for (const [name, mode] of Object.entries(modes)) {
    const command = [process.execPath, helper, mode].map((x) => quote(x.replaceAll("\\", "/"))).join(" ");
    writeFileSync(join(hooks, name), "#!/bin/sh\n" + marker + "\nexec " + command + " \"$@\"\n", { mode: 0o700 });
  }
  git("config", "--local", "user.name", "Marcus-P");
  git("config", "--local", "user.email", ownerEmail);
  git("config", "--local", "privacy.expectedEmail", ownerEmail);
  console.log("Repository-local no-reply identity and commit/push guards installed.");
}

function check(mode) {
  const expected = config("privacy.expectedEmail")[0];
  if (!expected) fail("Privacy guard has no expected identity; rerun its setup.");
  const blocked = config("privacy.blockedEmail").map((x) => x.toLowerCase());
  const inspect = (text) => {
    if (blocked.some((x) => text.toLowerCase().includes(x))) {
      fail("Blocked private address detected. Inspect locally; its value is intentionally omitted.");
    }
  };
  const additions = (text) => text.split("\n").filter((x) => x.startsWith("+") && !x.startsWith("+++")).join("\n");
  if (mode === "commit") {
    const committer = git("var", "GIT_COMMITTER_IDENT");
    if (committer.match(/<([^<>]+)> \d+ [+-]\d+$/)?.[1] !== expected) {
      fail("Committer must use the configured GitHub no-reply identity.");
    }
    inspect(git("var", "GIT_AUTHOR_IDENT"));
    inspect(additions(git("diff", "--cached", "--no-ext-diff", "--unified=0")));
  } else if (mode === "message") {
    inspect(readFileSync(process.argv[3], "utf8"));
  } else if (mode === "push") {
    const commits = new Set();
    for (const line of readFileSync(0, "utf8").trim().split("\n").filter(Boolean)) {
      const [, local, , remote] = line.split(/\s+/);
      if (/^0+$/.test(local)) continue;
      const args = ["rev-list", local, "--not", "--remotes"];
      if (!/^0+$/.test(remote)) {
        try { git("cat-file", "-e", remote + "^{commit}"); args.push(remote); } catch { /* Use known remote refs. */ }
      }
      for (const sha of git(...args).split("\n").filter(Boolean)) commits.add(sha);
    }
    for (const sha of commits) {
      inspect(git("show", "-s", "--format=%ae%n%ce%n%B", sha));
      inspect(additions(git("show", "--format=", "--no-ext-diff", "--diff-merges=first-parent", "--unified=0", sha)));
    }
  } else fail("Unknown privacy guard mode.");
}

try {
  const mode = process.argv[2];
  if (mode === "setup") install(); else check(mode);
} catch (error) {
  // Do not echo subprocess arguments, stdout or errors that may contain PII.
  console.error(error.status === undefined ? error.message : "Privacy guard could not inspect Git state; operation stopped.");
  process.exitCode = 1;
}
