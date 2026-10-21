import { execFileSync } from "node:child_process";
import { appendFileSync } from "node:fs";
import { pathToFileURL } from "node:url";

const metadata = new Set([
  "package.json", "package-lock.json", "src-tauri/Cargo.toml",
  "src-tauri/Cargo.lock", "RELEASE_VERSION",
]);

function withoutVersion(path, text) {
  if (text === null) return null;
  if (path === "RELEASE_VERSION") return "<version>";
  if (path === "package.json" || path === "package-lock.json") {
    const data = JSON.parse(text);
    delete data.version;
    if (path === "package-lock.json" && data.packages?.[""]) {
      delete data.packages[""].version;
    }
    return JSON.stringify(data);
  }
  if (path === "src-tauri/Cargo.lock") {
    return text.replace(/(\[\[package\]\]\r?\nname = "tripviewer"\r?\nversion = )"[^"]+"/, '$1"<version>"');
  }
  return text.replace(/\[package\][\s\S]*?(?=\n\[|$)/, (section) =>
    section.replace(/^version\s*=\s*"[^"]+"/m, 'version = "<version>"'));
}

export function classifyChanges(changes) {
  const scope = { docs: false, docs_only: false, metadata_only: false, frontend: false,
    backend: false, bundle: false, audit: false, tooling: false };
  let lightweight = changes.length > 0;
  let hasMetadata = false;
  for (const { path, before = null, after = null } of changes) {
    if (/^[^/]+\.md$/.test(path) || /^docs\/.*\.md$/.test(path) ||
        path === ".github/FUNDING.yml" || path === ".gitignore") {
      scope.docs = true;
      continue;
    }
    if (metadata.has(path) && before !== null && after !== null &&
        withoutVersion(path, before) === withoutVersion(path, after)) {
      hasMetadata = true;
      continue;
    }
    lightweight = false;
    if (path.startsWith("src/")) {
      scope.frontend = true;
      if (!/\.test\.tsx?$/.test(path) && (/^src\/(engine|updates|ipc|components\/(video|map|timeline|updates))\//.test(path) ||
          path === "src/App.tsx")) scope.bundle = true;
    } else if (path.startsWith("src-tauri/src/") || path === "src-tauri/build.rs") {
      scope.backend = scope.bundle = true;
    } else if (/^src-tauri\/(tests|examples)\//.test(path)) {
      scope.backend = true;
    } else if (/^scripts\/(change-scope|check-version|check-docs|check-interfaces)(\.test)?\.mjs$/.test(path)) {
      scope.tooling = true;
    } else if (/^scripts\/(install-|prepare-|validate-)/.test(path)) {
      scope.bundle = scope.tooling = true;
    } else if (path === "package.json" || path === "package-lock.json" ||
               /^((vite|tsconfig)[^/]*|index\.html)$/.test(path)) {
      scope.frontend = true;
      if (path === "package-lock.json") scope.audit = scope.bundle = true;
      if (path === "package.json") {
        const previous = before === null ? {} : JSON.parse(before);
        const current = after === null ? {} : JSON.parse(after);
        const differs = (key) => JSON.stringify(previous[key]) !== JSON.stringify(current[key]);
        if (["dependencies", "optionalDependencies", "overrides"].some(differs)) scope.audit = scope.bundle = true;
        if (differs("devDependencies") || ["build", "tauri", "pretauri"].some((key) =>
          previous.scripts?.[key] !== current.scripts?.[key])) scope.bundle = true;
      }
    } else if (path.startsWith("src-tauri/") || path.startsWith("icon/")) {
      scope.bundle = true;
      if (path.endsWith("Cargo.toml") || path.endsWith("Cargo.lock")) scope.backend = true;
    } else {
      // Unknown files and workflow changes fail closed: prove the full pipeline.
      scope.frontend = scope.backend = scope.bundle = scope.audit = scope.tooling = true;
    }
  }
  scope.docs_only = lightweight && !hasMetadata;
  scope.metadata_only = lightweight && hasMetadata;
  if (changes.length === 0) scope.docs = true;
  return scope;
}

export function scopeFromGit(base, head = "HEAD") {
  if (!base) {
    return { docs: true, docs_only: false, metadata_only: false, frontend: true,
      backend: true, bundle: true, audit: true, tooling: true };
  }
  if (!/^[a-f0-9]{40}$/i.test(base)) throw Error("CHECK_BASE_SHA must be a full commit SHA");
  const names = execFileSync("git", ["diff", "--no-renames", "--name-only", "-z", base, head],
    { encoding: "utf8" }).split("\0").filter(Boolean);
  const read = (ref, path) => {
    try { return execFileSync("git", ["show", ref + ":" + path], { encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] }); }
    catch { return null; }
  };
  return classifyChanges(names.map((path) => ({ path,
    before: metadata.has(path) ? read(base, path) : null,
    after: metadata.has(path) ? read(head, path) : null })));
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const scope = scopeFromGit(process.env.CHECK_BASE_SHA);
  console.log(JSON.stringify(scope));
  if (process.env.GITHUB_OUTPUT) appendFileSync(process.env.GITHUB_OUTPUT,
    Object.entries(scope).map(([name, value]) => name + "=" + value).join("\n") + "\n");
}
