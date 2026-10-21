import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";

const lib = readFileSync("src-tauri/src/lib.rs", "utf8");
const handler = lib.split("tauri::generate_handler![")[1]?.split("])")[0];
if (!handler) throw Error("Tauri command registry not found");
const registered = new Set(handler.split(",").map((entry) =>
  entry.trim().split("::").at(-1)).filter(Boolean));
function* files(dir) {
  for (const item of readdirSync(dir, { withFileTypes: true })) {
    const path = join(dir, item.name);
    if (item.isDirectory()) yield* files(path);
    else if (/\.tsx?$/.test(path)) yield path;
  }
}
let checked = 0;
for (const file of files("src")) {
  if (/\.test\.tsx?$/.test(file)) continue;
  for (const match of readFileSync(file, "utf8").matchAll(/\binvoke(?:<[^;]*?>)?\s*\(\s*["']([a-z_]+)["']/g)) {
    if (!registered.has(match[1])) throw Error(file + ": unregistered Tauri command " + match[1]);
    checked++;
  }
}
const migrations = readFileSync("src-tauri/src/db/migrations.rs", "utf8");
const included = new Set([...migrations.matchAll(/migrations\/([^"]+\.sql)/g)].map((m) => m[1]));
const actual = readdirSync("src-tauri/src/db/migrations").filter((name) => name.endsWith(".sql"));
for (const file of actual) if (!included.has(file)) throw Error("Unregistered DB migration " + file);
if (included.size !== actual.length) throw Error("DB migration registry and files differ");
console.log(checked + " frontend IPC calls resolve; " + actual.length + " migrations registered");
