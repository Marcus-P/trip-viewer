import { execFileSync } from "node:child_process";
import { pathToFileURL } from "node:url";
import { scopeFromGit } from "./change-scope.mjs";

const git = (args, cwd) => execFileSync("git", args, {
  cwd, encoding: "utf8", stdio: ["ignore", "pipe", "pipe"],
}).trim();

export function upstreamStatus(upstream, head = "HEAD", cwd) {
  const [behind, ahead] = git(["rev-list", "--left-right", "--count", `${upstream}...${head}`], cwd)
    .split(/\s+/).map(Number);
  return { upstream: git(["rev-parse", upstream], cwd), head: git(["rev-parse", head], cwd),
    behind, ahead, current: behind === 0 };
}

export function needsCurrentUpstream(scope) {
  return ["frontend", "backend", "bundle", "audit"].some(key => scope[key]);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    // Fixed public source; never execute or merge upstream code in this check.
    git(["fetch", "--quiet", "--no-tags", "https://github.com/chrisl8/trip-viewer.git",
      "+refs/heads/main:refs/upstream-check/main"]);
    const head = process.env.UPSTREAM_HEAD_SHA || "HEAD";
    const base = process.env.UPSTREAM_BASE_SHA;
    const required = base ? needsCurrentUpstream(scopeFromGit(base, head)) : true;
    const result = upstreamStatus("refs/upstream-check/main", head);
    console.log(JSON.stringify({ ...result, required }, null, 2));
    if (!result.current) {
      const message = "Source branch is behind current chrisl8/trip-viewer main by " + result.behind +
        " commit(s). Integrate and review upstream before application work; this check never merges automatically.";
      console.log((required ? "::error::" : "::warning::") + message);
      if (required) process.exitCode = 1;
    }
  } catch {
    console.error("Upstream freshness could not be verified; check failed. No merge or history update was performed.");
    process.exitCode = 1;
  }
}
