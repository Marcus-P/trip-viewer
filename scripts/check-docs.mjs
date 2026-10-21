import { readFileSync, existsSync } from "node:fs";
import { execFileSync, spawnSync } from "node:child_process";
import { dirname, resolve } from "node:path";

const files = execFileSync("git", ["ls-files", "-z", "*.md"], { encoding: "utf8" })
  .split("\0").filter((name) => name && !name.startsWith("docs/history/"));
let links = 0;
let shellBlocks = 0;
for (const name of files) {
  if (!existsSync(name)) continue;
  const text = readFileSync(name, "utf8");
  if (text.includes("\r")) throw Error(name + ": use LF line endings");
  for (const block of text.matchAll(/^\x60{3}bash\r?\n([\s\S]*?)^\x60{3}/gm)) {
    const result = spawnSync("bash", ["-n"], { input: block[1], encoding: "utf8" });
    if (result.status !== 0) throw Error(name + ": invalid shell example: " + (result.stderr || result.error));
    shellBlocks++;
  }
  for (const match of text.matchAll(/\]\(([^)\n]+)\)/g)) {
    let target = match[1];
    if (/^[a-z][a-z0-9+.-]*:|^#/i.test(target)) continue;
    target = target.split("#")[0].replace(/^<|>$/g, "");
    if (!target) continue;
    if (!existsSync(resolve(dirname(name), decodeURIComponent(target)))) {
      throw Error(name + ": missing local link " + target);
    }
    links++;
  }
}
console.log(files.length + " current Markdown files, " + links + " local links, " + shellBlocks + " shell examples checked");
