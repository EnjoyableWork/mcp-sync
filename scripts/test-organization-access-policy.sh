#!/usr/bin/env bash

set -euo pipefail

organization_access_test_directory="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
# Both scripts are checked directly by the quality gate.
# shellcheck disable=SC1091
source "$organization_access_test_directory/verify-organization-access-controls.sh"

organization_access_test_expect_failure() {
  local test_name=$1
  shift

  if "$@"; then
    echo "expected organization access policy rejection: $test_name" >&2
    return 1
  fi
}

organization_access_test_policy='{
  "two_factor_requirement_enabled": true,
  "default_repository_permission": "none",
  "members_can_create_repositories": false,
  "members_can_create_public_repositories": false,
  "members_can_create_private_repositories": false,
  "members_can_create_internal_repositories": false,
  "members_can_fork_private_repositories": false
}'
organization_access_assert_organization_policy \
  <<<"$organization_access_test_policy"
organization_access_test_expect_failure \
  'missing MFA' \
  organization_access_assert_organization_policy \
  <<<"$(jq '.two_factor_requirement_enabled = false' <<<"$organization_access_test_policy")"
organization_access_test_expect_failure \
  'implicit repository read access' \
  organization_access_assert_organization_policy \
  <<<"$(jq '.default_repository_permission = "read"' <<<"$organization_access_test_policy")"
organization_access_test_expect_failure \
  'member repository creation' \
  organization_access_assert_organization_policy \
  <<<"$(jq '.members_can_create_public_repositories = true' <<<"$organization_access_test_policy")"

organization_access_test_workflow_policy='{
  "default_workflow_permissions": "read",
  "can_approve_pull_request_reviews": false
}'
organization_access_assert_workflow_token_policy \
  <<<"$organization_access_test_workflow_policy"
organization_access_test_expect_failure \
  'write-default workflow token' \
  organization_access_assert_workflow_token_policy \
  <<<"$(jq '.default_workflow_permissions = "write"' <<<"$organization_access_test_workflow_policy")"

organization_access_test_mcp_keys='[]'
organization_access_test_tap_keys='[
  {
    "read_only": false,
    "verified": true,
    "enabled": true,
    "last_used": "2026-08-08T00:00:00Z"
  }
]'
organization_access_assert_deploy_key_boundary \
  "$organization_access_test_mcp_keys" \
  "$organization_access_test_tap_keys"
organization_access_test_expect_failure \
  'broad second deploy key' \
  organization_access_assert_deploy_key_boundary \
  "$organization_access_test_mcp_keys" \
  "$(jq '. += [.[0]]' <<<"$organization_access_test_tap_keys")"

organization_access_test_empty_secrets='{"total_count": 0, "secrets": []}'
organization_access_test_release_secrets='{
  "total_count": 1,
  "secrets": [{"name": "HOMEBREW_TAP_DEPLOY_KEY"}]
}'
organization_access_assert_secret_boundary \
  "$organization_access_test_empty_secrets" \
  "$organization_access_test_empty_secrets" \
  "$organization_access_test_empty_secrets" \
  "$organization_access_test_release_secrets" \
  "$organization_access_test_empty_secrets"
organization_access_test_expect_failure \
  'extra environment secret' \
  organization_access_assert_secret_boundary \
  "$organization_access_test_empty_secrets" \
  "$organization_access_test_empty_secrets" \
  "$organization_access_test_empty_secrets" \
  "$(jq '.total_count = 2 | .secrets += [{"name": "UNEXPECTED"}]' <<<"$organization_access_test_release_secrets")" \
  "$organization_access_test_empty_secrets"

organization_access_test_evidence='{
  "schema": 1,
  "reviewed_on": "2026-08-09",
  "ownership_mode": "single-owner-recovery",
  "owner_choice_explicitly_accepted": true,
  "secure_methods_only_confirmed": true,
  "independent_recovery_paths_confirmed": true,
  "private_recovery_record_current": true,
  "recovery_exercise_passed": true,
  "least_privilege_access_reviewed": true,
  "automation_access_reviewed": true,
  "expected_counts": {
    "organization_members": 1,
    "organization_owners": 1,
    "outside_collaborators": 0,
    "pending_invitations": 0,
    "teams": 0,
    "all_repository_app_installations": 1,
    "write_capable_all_repository_app_installations": 1
  }
}'
organization_access_assert_private_evidence \
  "$organization_access_test_evidence" 1 1 0 0 0 1 1
organization_access_test_expect_failure \
  'unaccepted owner choice' \
  organization_access_assert_private_evidence \
  "$(jq '.owner_choice_explicitly_accepted = false' <<<"$organization_access_test_evidence")" \
  1 1 0 0 0 1 1
organization_access_test_expect_failure \
  'untested recovery' \
  organization_access_assert_private_evidence \
  "$(jq '.recovery_exercise_passed = false' <<<"$organization_access_test_evidence")" \
  1 1 0 0 0 1 1
organization_access_test_expect_failure \
  'stale aggregate access count' \
  organization_access_assert_private_evidence \
  "$organization_access_test_evidence" 1 1 0 0 0 2 1
organization_access_test_expect_failure \
  'identity-bearing extra field' \
  organization_access_assert_private_evidence \
  "$(jq '.owner_identity = "forbidden"' <<<"$organization_access_test_evidence")" \
  1 1 0 0 0 1 1
organization_access_test_expect_failure \
  'single-owner evidence for two owners' \
  organization_access_assert_private_evidence \
  "$(jq '.expected_counts.organization_members = 2 | .expected_counts.organization_owners = 2' <<<"$organization_access_test_evidence")" \
  2 2 0 0 0 1 1

organization_access_test_two_owner_evidence="$(
  jq '
      .ownership_mode = "two-trusted-owners" |
      .expected_counts.organization_members = 2 |
      .expected_counts.organization_owners = 2
    ' <<<"$organization_access_test_evidence"
)"
organization_access_assert_private_evidence \
  "$organization_access_test_two_owner_evidence" 2 2 0 0 0 1 1

if [[ "$(organization_access_count_array_pages <<<'[[{}, {}], []]')" != 2 ]]; then
  echo 'paginated aggregate count should ignore identities and count records' >&2
  exit 1
fi
organization_access_test_expect_failure \
  'malformed paginated aggregate' \
  organization_access_count_array_pages \
  <<<'{"not": "pages"}'

echo 'Organization access policy synthetic acceptance and rejection checks passed.'
