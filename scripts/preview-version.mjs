import { readFileSync, writeFileSync } from "node:fs";
import { pathToFileURL } from "node:url";

export function previewVersion(version, build) {
  const match = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-[0-9A-Za-z.-]+)?$/.exec(version);
  if (!match || !/^[1-9]\d*$/.test(String(build))) throw Error("Expected a semantic version and positive build ID");
  return match.slice(1).join(".") + "-rc." + build;
}

export function setVersion(root, version) {
  const file = (name) => new URL(name, root);
  const read = (name) => readFileSync(file(name), "utf8").replace(/\r\n/g, "\n");
  const old = JSON.parse(read("package.json")).version;
  for (const name of ["package.json", "package-lock.json"]) {
    const value = JSON.parse(read(name)); value.version = version;
    if (name === "package-lock.json") value.packages[""].version = version;
    writeFileSync(file(name), JSON.stringify(value, null, 2) + "\n");
  }
  writeFileSync(file("src-tauri/Cargo.toml"), read("src-tauri/Cargo.toml").replace(/(\[package\][\s\S]*?\nversion = )"[^"]+"/, '$1"' + version + '"'));
  writeFileSync(file("src-tauri/Cargo.lock"), read("src-tauri/Cargo.lock").replace(/(name = "tripviewer"\nversion = )"[^"]+"/, '$1"' + version + '"'));
  writeFileSync(file("RELEASE_VERSION"), version + "\n");
  writeFileSync(file("RELEASE_NOTES.md"), read("RELEASE_NOTES.md").replace("# Trip Viewer " + old + "\n", "# Trip Viewer " + version + "\n"));
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const root = new URL("../", import.meta.url);
  const current = JSON.parse(readFileSync(new URL("package.json", root), "utf8")).version;
  const next = process.argv[2] === "--stable" ? current.split("-")[0] : previewVersion(current, process.env.GITHUB_RUN_ID);
  setVersion(root, next);
  console.log("Build version: " + next);
}
