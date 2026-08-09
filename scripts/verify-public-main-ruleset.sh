#!/usr/bin/env bash

set -euo pipefail

main_ruleset_repository=${1:-}
main_ruleset_name='Protect main'

if [[ "$main_ruleset_repository" != EnjoyableWork/mcp-sync ]]; then
  echo "usage: $0 EnjoyableWork/mcp-sync" >&2
  exit 2
fi

main_ruleset_api_headers=(
  --header 'Accept: application/vnd.github+json'
  --header 'User-Agent: mcp-sync-repository-controls/0.1'
  --header 'X-GitHub-Api-Version: 2026-03-10'
)
main_rulesets="$({
  curl --fail --silent --show-error --location \
    "${main_ruleset_api_headers[@]}" \
    "https://api.github.com/repos/$main_ruleset_repository/rulesets"
})"

main_ruleset_count="$(
  jq --arg name "$main_ruleset_name" \
    '[.[] | select(.name == $name)] | length' \
    <<<"$main_rulesets"
)"
if [[ "$main_ruleset_count" != 1 ]]; then
  echo "expected exactly one default-branch ruleset" >&2
  exit 1
fi

main_ruleset_id="$(
  jq -r --arg name "$main_ruleset_name" \
    '.[] | select(.name == $name) | .id' \
    <<<"$main_rulesets"
)"
if [[ ! "$main_ruleset_id" =~ ^[0-9]+$ ]]; then
  echo "default-branch ruleset has an invalid identifier" >&2
  exit 1
fi

main_ruleset="$({
  curl --fail --silent --show-error --location \
    "${main_ruleset_api_headers[@]}" \
    "https://api.github.com/repos/$main_ruleset_repository/rulesets/$main_ruleset_id"
})"

jq -e --arg name "$main_ruleset_name" '
    .name == $name and
    .enforcement == "active" and
    .target == "branch" and
    .conditions.ref_name.include == ["~DEFAULT_BRANCH"] and
    .conditions.ref_name.exclude == [] and
    (.bypass_actors | length) == 0 and
    ([.rules[].type] | sort) ==
      (["deletion", "non_fast_forward", "pull_request", "required_status_checks"] | sort) and
    ([.rules[] | select(.type == "pull_request")] | length) == 1 and
    any(.rules[];
      .type == "pull_request" and
      (.parameters.allowed_merge_methods | sort) == (["merge", "rebase", "squash"] | sort) and
      .parameters.dismiss_stale_reviews_on_push == false and
      .parameters.require_code_owner_review == false and
      .parameters.require_last_push_approval == false and
      .parameters.required_approving_review_count == 0 and
      .parameters.required_review_thread_resolution == true) and
    ([.rules[] | select(.type == "required_status_checks")] | length) == 1 and
    any(.rules[];
      .type == "required_status_checks" and
      .parameters.do_not_enforce_on_create == false and
      .parameters.strict_required_status_checks_policy == true and
      ([.parameters.required_status_checks[] | {context, integration_id}] | sort_by(.context)) ==
        ([
          {context: "Dependency policy", integration_id: 15368},
          {context: "Linux ARM64 — format, Clippy, and test", integration_id: 15368},
          {context: "Linux x64 — format, Clippy, and test", integration_id: 15368},
          {context: "Windows ARM64 — format, Clippy, and test", integration_id: 15368},
          {context: "Windows x64 — format, Clippy, and test", integration_id: 15368}
        ] | sort_by(.context)))
  ' <<<"$main_ruleset" >/dev/null

echo "Verified the public default-branch ruleset for $main_ruleset_repository."
