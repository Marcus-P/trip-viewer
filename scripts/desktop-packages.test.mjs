import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

test("Windows-only acceptance keeps both default release platforms", () => {
  const workflow = readFileSync(new URL("../.github/workflows/desktop-packages.yml", import.meta.url), "utf8");
  const choices = workflow.match(/fromJSON\(inputs.windows-only && '([^']+)' \|\| '([^']+)'\)/);
  assert.ok(choices);
  assert.deepEqual(JSON.parse(choices[1]).map((p) => p.platform), ["Windows-x64"]);
  assert.deepEqual(JSON.parse(choices[2]).map((p) => p.platform), ["Windows-x64", "macOS-universal"]);
  assert.match(workflow, /workflow_call:\n    inputs:\n      windows-only:\n        type: boolean\n        default: false/);
  assert.match(workflow, /retention-days: \$\{\{ inputs.acceptance && 7 \|\| 1 \}\}/);
  assert.ok(!/^  ubuntu:/m.test(workflow), "a quick package run must not emit the full required CI gate");
});
