import { readFileSync } from "node:fs";
import { execFileSync } from "node:child_process";

const read = (path) => readFileSync(new URL(`../${path}`, import.meta.url), "utf8");
const pkg = JSON.parse(read("package.json"));
const lock = JSON.parse(read("package-lock.json"));
const cargo = read("src-tauri/Cargo.toml").match(/\[package\]([\s\S]*?)(?=\n\[)/)?.[1]
  .match(/^version\s*=\s*"([^"]+)"/m)?.[1];
const cargoLock = read("src-tauri/Cargo.lock").match(/name = "tripviewer"\nversion = "([^"]+)"/)?.[1];
const tauri = JSON.parse(read("src-tauri/tauri.conf.json"));
const versions = {
  "package.json": pkg.version,
  "package-lock.json": lock.version,
  "package-lock.json root package": lock.packages?.[""]?.version,
  "Cargo.toml": cargo,
  "Cargo.lock": cargoLock,
  "RELEASE_VERSION": read("RELEASE_VERSION").trim(),
};
const parse = (value) => {
  if (typeof value !== "string" || !/^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/.test(value)) {
    throw new Error(`Expected stable MAJOR.MINOR.PATCH, got ${value}`);
  }
  return value.split(".").map(BigInt);
};
try {
  const current = parse(pkg.version);
  if (!read("RELEASE_NOTES.md").startsWith("# Trip Viewer " + pkg.version + "\n")) {
    throw new Error("RELEASE_NOTES.md must start with the proposed application version");
  }
  for (const [file, version] of Object.entries(versions)) {
    if (version !== pkg.version) throw new Error(`${file}: ${version} differs from ${pkg.version}`);
  }
  if (tauri.version !== "../package.json") throw new Error("Tauri must read its version from ../package.json");
  const base = process.env.VERSION_BASE_SHA;
  if (base) {
    if (!/^[a-f0-9]{40}$/i.test(base)) throw new Error("VERSION_BASE_SHA must be a full Git commit SHA");
    const old = JSON.parse(execFileSync("git", ["show", `${base}:package.json`], { encoding: "utf8" })).version;
    const previous = parse(old);
    const notesChanged = execFileSync("git", ["diff", "--name-only", base, "--", "RELEASE_NOTES.md"], { encoding: "utf8" }).trim();
    if (!notesChanged) throw new Error("Every merge must update RELEASE_NOTES.md");
    const changed = current.findIndex((part, index) => part !== previous[index]);
    if (changed < 0 || current[changed] <= previous[changed]) {
      throw new Error(`Merge version must increase: base ${old}, proposed ${pkg.version}`);
    }
    if (current[changed] !== previous[changed] + 1n || current.slice(changed + 1).some(part => part !== 0n)) {
      throw new Error(`Use one patch, minor or major increment and reset lower components: ${old} -> ${pkg.version}`);
    }
    console.log(`Merge version: ${old} -> ${pkg.version}`);
  }
  console.log(`Version files agree: ${pkg.version}`);
} catch (error) {
  console.error(error.message);
  process.exitCode = 1;
}
