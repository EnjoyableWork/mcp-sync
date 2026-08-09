#!/usr/bin/env bash

set -euo pipefail

organization_access_preparation_repository=${1:-}
organization_access_preparation_organization=EnjoyableWork
organization_access_preparation_script_directory="$(
  cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
  pwd
)"
organization_access_preparation_api_headers=(
  --header 'Accept: application/vnd.github+json'
  --header 'X-GitHub-Api-Version: 2026-03-10'
)

if [[ "$organization_access_preparation_repository" != EnjoyableWork/mcp-sync ]]; then
  echo "usage: $0 EnjoyableWork/mcp-sync" >&2
  exit 2
fi
for organization_access_preparation_command in gh jq date; do
  if ! command -v "$organization_access_preparation_command" >/dev/null 2>&1; then
    echo "organization access evidence preparation requires $organization_access_preparation_command" >&2
    exit 2
  fi
done
if ! gh auth status --hostname github.com >/dev/null 2>&1; then
  echo "GitHub CLI must be authenticated with organization administration read access" >&2
  exit 1
fi

# Both scripts are checked directly by the quality gate.
# shellcheck disable=SC1091
source "$organization_access_preparation_script_directory/verify-organization-access-controls.sh"

organization_access_preparation_count() {
  local endpoint=$1

  gh api --paginate --slurp "$endpoint" \
    "${organization_access_preparation_api_headers[@]}" |
    organization_access_count_array_pages
}

organization_access_preparation_member_count="$(
  organization_access_preparation_count \
    "orgs/$organization_access_preparation_organization/members?filter=all&role=all&per_page=100"
)"
organization_access_preparation_owner_count="$(
  organization_access_preparation_count \
    "orgs/$organization_access_preparation_organization/members?filter=all&role=admin&per_page=100"
)"
organization_access_preparation_outside_collaborator_count="$(
  organization_access_preparation_count \
    "orgs/$organization_access_preparation_organization/outside_collaborators?per_page=100"
)"
organization_access_preparation_pending_invitation_count="$(
  organization_access_preparation_count \
    "orgs/$organization_access_preparation_organization/invitations?per_page=100"
)"
organization_access_preparation_team_count="$(
  organization_access_preparation_count \
    "orgs/$organization_access_preparation_organization/teams?per_page=100"
)"
organization_access_preparation_installations="$(
  gh api --paginate --slurp \
    "orgs/$organization_access_preparation_organization/installations?per_page=100" \
    "${organization_access_preparation_api_headers[@]}"
)"
organization_access_preparation_all_repository_app_count="$(
  jq '[.[] | .installations[]? | select(.repository_selection == "all")] | length' \
    <<<"$organization_access_preparation_installations"
)"
organization_access_preparation_write_capable_all_repository_app_count="$(
  jq '[
      .[] | .installations[]? |
      select(.repository_selection == "all") |
      select(any(.permissions | to_entries[]; .value == "write"))
    ] | length' <<<"$organization_access_preparation_installations"
)"

if ((organization_access_preparation_owner_count >= 2)); then
  organization_access_preparation_ownership_mode=two-trusted-owners
elif ((organization_access_preparation_owner_count == 1)); then
  organization_access_preparation_ownership_mode=single-owner-recovery
else
  echo "organization ownership count is invalid; no evidence template was produced" >&2
  exit 1
fi

echo "Template only: review privately, set booleans true only after completion, and store outside the repository with owner-only permissions." >&2
jq -n \
  --arg reviewed_on "$(date -u +%F)" \
  --arg ownership_mode "$organization_access_preparation_ownership_mode" \
  --argjson organization_members "$organization_access_preparation_member_count" \
  --argjson organization_owners "$organization_access_preparation_owner_count" \
  --argjson outside_collaborators "$organization_access_preparation_outside_collaborator_count" \
  --argjson pending_invitations "$organization_access_preparation_pending_invitation_count" \
  --argjson teams "$organization_access_preparation_team_count" \
  --argjson all_repository_app_installations \
    "$organization_access_preparation_all_repository_app_count" \
  --argjson write_capable_all_repository_app_installations \
    "$organization_access_preparation_write_capable_all_repository_app_count" '
    {
      schema: 1,
      reviewed_on: $reviewed_on,
      ownership_mode: $ownership_mode,
      owner_choice_explicitly_accepted: false,
      secure_methods_only_confirmed: false,
      independent_recovery_paths_confirmed: false,
      private_recovery_record_current: false,
      recovery_exercise_passed: false,
      least_privilege_access_reviewed: false,
      automation_access_reviewed: false,
      expected_counts: {
        organization_members: $organization_members,
        organization_owners: $organization_owners,
        outside_collaborators: $outside_collaborators,
        pending_invitations: $pending_invitations,
        teams: $teams,
        all_repository_app_installations: $all_repository_app_installations,
        write_capable_all_repository_app_installations:
          $write_capable_all_repository_app_installations
      }
    }
  '
