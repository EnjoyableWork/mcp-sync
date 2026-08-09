#!/usr/bin/env bash

set -euo pipefail

repository_security_repository=${1:-}

if [[ "$repository_security_repository" != EnjoyableWork/mcp-sync ]]; then
  echo "usage: $0 EnjoyableWork/mcp-sync" >&2
  exit 2
fi
if ! gh auth status --hostname github.com >/dev/null 2>&1; then
  echo "GitHub CLI must be authenticated with repository administration read access" >&2
  exit 1
fi

repository_security_api_headers=(
  --header 'Accept: application/vnd.github+json'
  --header 'X-GitHub-Api-Version: 2026-03-10'
)

repository_security_assert_empty_baseline() {
  local baseline_name=$1
  local baseline_endpoint=$2
  local baseline_payload

  if ! baseline_payload="$(
    gh api "$baseline_endpoint" "${repository_security_api_headers[@]}"
  )"; then
    echo "$baseline_name baseline could not be verified; details remain in GitHub" >&2
    return 1
  fi
  if ! jq -e 'type == "array" and length == 0' \
    <<<"$baseline_payload" >/dev/null; then
    echo "$baseline_name baseline requires private maintainer remediation; details remain in GitHub" >&2
    return 1
  fi
}

repository_state="$(
  gh api "repos/$repository_security_repository" \
    "${repository_security_api_headers[@]}"
)"
jq -e '
    .visibility == "public" and
    .archived == false and
    .security_and_analysis.dependabot_security_updates.status == "enabled" and
    .security_and_analysis.secret_scanning.status == "enabled" and
    .security_and_analysis.secret_scanning_push_protection.status == "enabled" and
    .security_and_analysis.secret_scanning_non_provider_patterns.status == "disabled" and
    .security_and_analysis.secret_scanning_validity_checks.status == "disabled"
  ' <<<"$repository_state" >/dev/null

organization_state="$(
  gh api orgs/EnjoyableWork "${repository_security_api_headers[@]}"
)"
jq -e '.plan.name == "free"' <<<"$organization_state" >/dev/null

private_reporting_state="$(
  gh api "repos/$repository_security_repository/private-vulnerability-reporting" \
    "${repository_security_api_headers[@]}"
)"
jq -e '.enabled == true' <<<"$private_reporting_state" >/dev/null

gh api "repos/$repository_security_repository/vulnerability-alerts" \
  "${repository_security_api_headers[@]}" >/dev/null

dependency_graph_state="$(
  gh api "repos/$repository_security_repository/dependency-graph/sbom" \
    "${repository_security_api_headers[@]}"
)"
jq -e '.sbom.packages | type == "array" and length > 0' \
  <<<"$dependency_graph_state" >/dev/null

codeql_state="$(
  gh api "repos/$repository_security_repository/code-scanning/default-setup" \
    "${repository_security_api_headers[@]}"
)"
jq -e '
    .state == "configured" and
    .query_suite == "default" and
    ([.languages[]] | sort) == (["actions", "rust"] | sort)
  ' <<<"$codeql_state" >/dev/null

repository_security_assert_empty_baseline \
  'Dependabot' \
  "repos/$repository_security_repository/dependabot/alerts?state=open&per_page=100"
repository_security_assert_empty_baseline \
  'Code scanning' \
  "repos/$repository_security_repository/code-scanning/alerts?state=open&per_page=100"
repository_security_assert_empty_baseline \
  'Secret scanning' \
  "repos/$repository_security_repository/secret-scanning/alerts?state=open&per_page=100"

echo "Verified the non-disclosing repository security baseline for $repository_security_repository."
