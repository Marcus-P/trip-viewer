#!/usr/bin/env bash
# Run using the repository owner's GitHub CLI login with Administration:write.
set -euo pipefail
repo="Marcus-P/trip-viewer"
script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
login="$(gh api user --jq .login)"
if [[ "$login" != "Marcus-P" ]]; then
  echo "Run this using the Marcus-P GitHub account (current login: $login)." >&2
  exit 1
fi
identity="$(gh api "repos/$repo" --jq '[.owner.type, .owner.login, .default_branch] | join("/")')"
if [[ "$identity" != "User/Marcus-P/main" ]]; then
  echo "Repository ownership/default branch changed; review rules before applying." >&2
  exit 1
fi
for file in main-history-rules.json main-owner-rules.json; do
  payload="$script_dir/github/$file"
  name="$(node -e 'process.stdout.write(JSON.parse(require("fs").readFileSync(process.argv[1], "utf8")).name)' "$payload")"
  id="$(gh api "repos/$repo/rulesets" --paginate --jq ".[] | select(.name == \"$name\") | .id")"
  if [[ -n "$id" ]]; then
    echo "Checking existing ruleset $id: $name"
  else
    id="$(gh api --method POST "repos/$repo/rulesets" --input "$payload" --jq .id)"
    echo "Created ruleset $id: $name"
  fi
  gh api "repos/$repo/rulesets/$id" | node -e '
    const fs = require("fs");
    const expected = JSON.parse(fs.readFileSync(process.argv[1], "utf8"));
    const actual = JSON.parse(fs.readFileSync(0, "utf8"));
    function matches(e, a) {
      if (Array.isArray(e)) return Array.isArray(a) && e.length === a.length && e.every(v => a.some(x => matches(v, x)));
      if (e && typeof e === "object") return a && Object.keys(e).every(k => matches(e[k], a[k]));
      return e === a;
    }
    if (!matches(expected, actual)) {
      console.error("Ruleset differs from the requested configuration; review it in GitHub Settings. Existing rules were not overwritten.");
      process.exit(1);
    }
    console.log("Verified active ruleset: " + actual.name);
  ' "$payload"
done
echo "Verified: main requires checked PRs, rejects force-push/deletion, and only the personal repository owner can merge."
