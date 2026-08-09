#!/usr/bin/env bash

set -euo pipefail

main_control_repository=${1:-}

if [[ "$main_control_repository" != EnjoyableWork/mcp-sync ]]; then
  echo "usage: $0 EnjoyableWork/mcp-sync" >&2
  exit 2
fi
if ! gh auth status --hostname github.com >/dev/null 2>&1; then
  echo "GitHub CLI must be authenticated with repository administration read access" >&2
  exit 1
fi

main_control_script_directory="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
main_control_api_version=2026-03-10

"$main_control_script_directory/verify-public-main-ruleset.sh" \
  "$main_control_repository" >/dev/null

repository_state="$(
  gh api "repos/$main_control_repository" \
    -H "X-GitHub-Api-Version: $main_control_api_version"
)"
jq -e '
    .default_branch == "main" and
    .allow_merge_commit == true and
    .allow_squash_merge == true and
    .allow_rebase_merge == true and
    .allow_auto_merge == false and
    .delete_branch_on_merge == true and
    .web_commit_signoff_required == false
  ' <<<"$repository_state" >/dev/null

echo "Verified the operator-side default-branch controls for $main_control_repository."
