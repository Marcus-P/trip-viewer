import { execFileSync } from "node:child_process";
import { appendFileSync, readFileSync } from "node:fs";
import { pathToFileURL } from "node:url";

export function planRelease({ version, sha, parentVersion, versionTagCommit = null }) {
  if (!/^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/.test(version)) {
    throw Error("Release version must be stable MAJOR.MINOR.PATCH");
  }
  if (!/^[a-f0-9]{40}$/.test(sha)) throw Error("Release source must be a full commit SHA");
  const prerelease = parentVersion === version && versionTagCommit !== sha;
  if (!prerelease && versionTagCommit && versionTagCommit !== sha) {
    throw Error("Version tag already belongs to another commit; increase RELEASE_VERSION");
  }
  return { version, tag: prerelease ? `main-${sha}` : `v${version}`, prerelease };
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const git = (...args) => {
    try { return execFileSync("git", args, { encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] }).trim(); }
    catch { return null; }
  };
  const version = readFileSync("RELEASE_VERSION", "utf8").trim();
  const plan = planRelease({ version, sha: git("rev-parse", "HEAD"),
    parentVersion: git("show", "HEAD^:RELEASE_VERSION"),
    versionTagCommit: git("rev-parse", "--verify", `refs/tags/v${version}^{commit}`) });
  if (git("rev-parse", "--verify", `refs/tags/${plan.tag}^{commit}`)) {
    throw Error(`Tag ${plan.tag} already exists; published downloads must not be overwritten`);
  }
  console.log(JSON.stringify(plan));
  if (process.env.GITHUB_OUTPUT) appendFileSync(process.env.GITHUB_OUTPUT,
    Object.entries(plan).map(([key, value]) => `${key}=${value}`).join("\n") + "\n");
}
