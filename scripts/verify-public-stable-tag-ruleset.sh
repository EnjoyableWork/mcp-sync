#!/usr/bin/env bash

set -euo pipefail

release_ruleset_repository=${1:-}
release_ruleset_name='Protect stable release tags'

if [[ "$release_ruleset_repository" != EnjoyableWork/mcp-sync ]]; then
  echo "usage: $0 EnjoyableWork/mcp-sync" >&2
  exit 2
fi

release_rulesets="$(
  curl --fail --silent --show-error --location \
    --header 'Accept: application/vnd.github+json' \
    --header 'User-Agent: mcp-sync-release-controls/0.1' \
    --header 'X-GitHub-Api-Version: 2026-03-10' \
    "https://api.github.com/repos/$release_ruleset_repository/rulesets"
)"

release_ruleset_count="$(
  jq --arg name "$release_ruleset_name" \
    '[.[] | select(.name == $name)] | length' \
    <<<"$release_rulesets"
)"
if [[ "$release_ruleset_count" != 1 ]]; then
  echo "expected exactly one stable release tag ruleset" >&2
  exit 1
fi

release_ruleset_id="$(
  jq -r --arg name "$release_ruleset_name" \
    '.[] | select(.name == $name) | .id' \
    <<<"$release_rulesets"
)"
if [[ ! "$release_ruleset_id" =~ ^[0-9]+$ ]]; then
  echo "stable release tag ruleset has an invalid identifier" >&2
  exit 1
fi

curl --fail --silent --show-error --location \
  --header 'Accept: application/vnd.github+json' \
  --header 'User-Agent: mcp-sync-release-controls/0.1' \
  --header 'X-GitHub-Api-Version: 2026-03-10' \
  "https://api.github.com/repos/$release_ruleset_repository/rulesets/$release_ruleset_id" \
  | jq -e --arg name "$release_ruleset_name" '
      .name == $name and
      .enforcement == "active" and
      .target == "tag" and
      .conditions.ref_name.include == ["refs/tags/v*"] and
      ([.rules[].type] | sort) == (["creation", "deletion", "update"] | sort)
    ' >/dev/null

echo "Verified the public stable-tag ruleset for $release_ruleset_repository."
