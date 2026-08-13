#!/usr/bin/env bash

set -euo pipefail

repository_supply_chain_repository=${1:-}
repository_supply_chain_version=${2:-}

if [[ "$repository_supply_chain_repository" != EnjoyableWork/mcp-sync ||
  "$repository_supply_chain_version" != 0.1.1 ]]; then
  echo "usage: $0 EnjoyableWork/mcp-sync 0.1.1" >&2
  exit 2
fi
if ! gh auth status --hostname github.com >/dev/null 2>&1; then
  echo "GitHub CLI must be authenticated with repository administration read access" >&2
  exit 1
fi
for repository_supply_chain_command in gh jq git; do
  if ! command -v "$repository_supply_chain_command" >/dev/null 2>&1; then
    echo "repository supply-chain verification requires $repository_supply_chain_command" >&2
    exit 2
  fi
done

repository_supply_chain_root="$(git rev-parse --show-toplevel)"
repository_supply_chain_policy="$repository_supply_chain_root/.github/actions-policy.json"
repository_supply_chain_headers=(
  --header 'Accept: application/vnd.github+json'
  --header 'X-GitHub-Api-Version: 2026-03-10'
)

repository_supply_chain_head="$(git -C "$repository_supply_chain_root" rev-parse HEAD)"
repository_supply_chain_main="$(
  gh api \
    "repos/$repository_supply_chain_repository/commits/main" \
    "${repository_supply_chain_headers[@]}" \
    --jq '.sha'
)"
if [[ "$repository_supply_chain_head" != "$repository_supply_chain_main" ]] ||
  ! git -C "$repository_supply_chain_root" diff --quiet ||
  ! git -C "$repository_supply_chain_root" diff --cached --quiet; then
  echo "repository supply-chain verification requires a clean checkout of exact main" >&2
  exit 1
fi

"$repository_supply_chain_root/scripts/verify-workflow-supply-chain.sh" \
  --verify-upstreams
"$repository_supply_chain_root/scripts/verify-repository-artifacts.sh"
"$repository_supply_chain_root/scripts/test-repository-artifact-policy.sh"

repository_supply_chain_actions="$(
  gh api \
    "repos/$repository_supply_chain_repository/actions/permissions" \
    "${repository_supply_chain_headers[@]}"
)"
jq -e '
    .enabled == true and
    .allowed_actions == "selected" and
    .sha_pinning_required == true
  ' <<<"$repository_supply_chain_actions" >/dev/null

repository_supply_chain_selected="$(
  gh api \
    "repos/$repository_supply_chain_repository/actions/permissions/selected-actions" \
    "${repository_supply_chain_headers[@]}"
)"
repository_supply_chain_expected_patterns="$(
  jq -c '.patterns_allowed | sort' "$repository_supply_chain_policy"
)"
jq -e \
  --argjson expected_patterns "$repository_supply_chain_expected_patterns" \
  '.github_owned_allowed == false and
   .verified_allowed == false and
   (.patterns_allowed | sort) == $expected_patterns' \
  <<<"$repository_supply_chain_selected" >/dev/null

repository_supply_chain_workflow_permissions="$(
  gh api \
    "repos/$repository_supply_chain_repository/actions/permissions/workflow" \
    "${repository_supply_chain_headers[@]}"
)"
jq -e '
    .default_workflow_permissions == "read" and
    .can_approve_pull_request_reviews == false
  ' <<<"$repository_supply_chain_workflow_permissions" >/dev/null

repository_supply_chain_fork_policy="$(
  gh api \
    "repos/$repository_supply_chain_repository/actions/permissions/fork-pr-contributor-approval" \
    "${repository_supply_chain_headers[@]}"
)"
jq -e '.approval_policy == "first_time_contributors"' \
  <<<"$repository_supply_chain_fork_policy" >/dev/null

repository_supply_chain_codeql="$(
  gh api \
    "repos/$repository_supply_chain_repository/code-scanning/default-setup" \
    "${repository_supply_chain_headers[@]}"
)"
jq -e '
    .state == "configured" and
    .query_suite == "default" and
    ([.languages[]] | sort) == (["actions", "rust"] | sort)
  ' <<<"$repository_supply_chain_codeql" >/dev/null

repository_supply_chain_dependabot="$(
  gh api \
    "repos/$repository_supply_chain_repository/contents/.github/dependabot.yml?ref=main" \
    "${repository_supply_chain_headers[@]}"
)"
repository_supply_chain_dependabot_blob="$(
  git -C "$repository_supply_chain_root" hash-object .github/dependabot.yml
)"
jq -e \
  --arg expected_blob "$repository_supply_chain_dependabot_blob" \
  '.type == "file" and .size > 0 and .sha == $expected_blob' \
  <<<"$repository_supply_chain_dependabot" >/dev/null

"$repository_supply_chain_root/scripts/verify-distribution-authentication.sh" \
  "$repository_supply_chain_repository" \
  "$repository_supply_chain_version"

printf 'Verified live selected-action, SHA, token, fork, CodeQL, update, distribution, and repository-artifact controls for %s.\n' \
  "$repository_supply_chain_repository"
