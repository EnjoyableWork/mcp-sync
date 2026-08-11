#!/usr/bin/env bash

set -euo pipefail

release_control_repository=EnjoyableWork/mcp-sync
release_control_commit=${1:-}
release_control_mode=${2:-}

if [[ ! "$release_control_commit" =~ ^[0-9a-f]{40}$ ]] ||
  [[ $# -gt 2 ]] ||
  [[ -n "$release_control_mode" &&
    "$release_control_mode" != --allow-mcp-039-main-rehearsal ]]; then
  echo "usage: $0 <exact-current-main-commit> [--allow-mcp-039-main-rehearsal]" >&2
  exit 2
fi
release_control_allow_rehearsal=false
if [[ "$release_control_mode" == --allow-mcp-039-main-rehearsal ]]; then
  release_control_allow_rehearsal=true
fi
if ! gh auth status --hostname github.com >/dev/null 2>&1; then
  echo "GitHub CLI must be authenticated as a repository administrator" >&2
  exit 1
fi

release_control_script_directory="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
release_control_api_version=2026-03-10

current_main="$(
  gh api "repos/$release_control_repository/commits/main" \
    -H "X-GitHub-Api-Version: $release_control_api_version" \
    --jq '.sha'
)"
if [[ "$current_main" != "$release_control_commit" ]]; then
  echo "expected commit is not the exact current main commit" >&2
  exit 1
fi

immutable_releases="$(
  gh api "repos/$release_control_repository/immutable-releases" \
    -H "X-GitHub-Api-Version: $release_control_api_version" \
    --jq '.enabled'
)"
if [[ "$immutable_releases" != true ]]; then
  echo "repository release immutability is not enabled" >&2
  exit 1
fi

release_ruleset_name='Protect stable release tags'
release_ruleset_id="$(
  gh api "repos/$release_control_repository/rulesets" \
    -H "X-GitHub-Api-Version: $release_control_api_version" \
    --jq ".[] | select(.name == \"$release_ruleset_name\") | .id"
)"
if [[ ! "$release_ruleset_id" =~ ^[0-9]+$ ]]; then
  echo "expected exactly one stable release tag ruleset" >&2
  exit 1
fi
gh api "repos/$release_control_repository/rulesets/$release_ruleset_id" \
  -H "X-GitHub-Api-Version: $release_control_api_version" \
  | jq -e --arg name "$release_ruleset_name" '
      .name == $name and
      .enforcement == "active" and
      .target == "tag" and
      .conditions.ref_name.include == ["refs/tags/v*"] and
      ([.rules[].type] | sort) == (["creation", "deletion", "update"] | sort) and
      (.bypass_actors | length) == 1 and
      .bypass_actors[0].actor_type == "User" and
      .bypass_actors[0].bypass_mode == "always"
    ' >/dev/null

verify_release_environment() {
  local environment_name=$1
  local policy_name=$2
  local policy_type=$3
  local allow_rehearsal=${4:-false}
  local environment_state
  local environment_policies

  environment_state="$(
    gh api "repos/$release_control_repository/environments/$environment_name" \
      -H 'X-GitHub-Api-Version: 2022-11-28'
  )"
  jq -e --arg name "$environment_name" '
      .name == $name and
      .deployment_branch_policy.protected_branches == false and
      .deployment_branch_policy.custom_branch_policies == true and
      (.protection_rules as $rules |
        ([$rules[].type] | sort) == ["branch_policy", "required_reviewers"] and
        ($rules | map(select(.type == "required_reviewers")) | length) == 1 and
        ($rules | map(select(.type == "required_reviewers"))[0] |
          .prevent_self_review == false and
          (.reviewers | length) == 1 and
          .reviewers[0].type == "User"))
    ' <<<"$environment_state" >/dev/null

  environment_policies="$(
    gh api \
      "repos/$release_control_repository/environments/$environment_name/deployment-branch-policies" \
      -H 'X-GitHub-Api-Version: 2022-11-28'
  )"
  if [[ "$allow_rehearsal" == true ]]; then
    jq -e '
        .total_count == 2 and
        ([.branch_policies[] | {name, type}] | sort_by(.type, .name)) ==
          ([{name: "main", type: "branch"}, {name: "v*", type: "tag"}] |
            sort_by(.type, .name))
      ' <<<"$environment_policies" >/dev/null
  else
    jq -e \
      --arg name "$policy_name" \
      --arg type "$policy_type" '
        .total_count == 1 and
        (.branch_policies | length) == 1 and
        .branch_policies[0].name == $name and
        .branch_policies[0].type == $type
      ' <<<"$environment_policies" >/dev/null
  fi
}

"$release_control_script_directory/verify-public-stable-tag-ruleset.sh" \
  "$release_control_repository" >/dev/null
verify_release_environment release-control main branch
verify_release_environment release 'v*' tag "$release_control_allow_rehearsal"

if [[ "$release_control_allow_rehearsal" == true ]]; then
  printf 'Verified release repository controls with the exact temporary MCP-039 main rehearsal policy for %s at %s.\n' \
    "$release_control_repository" \
    "$release_control_commit"
else
  printf 'Verified release repository controls for %s at %s.\n' \
    "$release_control_repository" \
    "$release_control_commit"
fi
