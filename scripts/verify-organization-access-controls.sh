#!/usr/bin/env bash

set -euo pipefail

organization_access_assert_organization_policy() {
  jq -e '
      .two_factor_requirement_enabled == true and
      .default_repository_permission == "none" and
      .members_can_create_repositories == false and
      .members_can_create_public_repositories == false and
      .members_can_create_private_repositories == false and
      .members_can_create_internal_repositories == false and
      .members_can_fork_private_repositories == false
    ' >/dev/null
}

organization_access_assert_workflow_token_policy() {
  jq -e '
      .default_workflow_permissions == "read" and
      .can_approve_pull_request_reviews == false
    ' >/dev/null
}

organization_access_assert_deploy_key_boundary() {
  local mcp_sync_keys=$1
  local tap_keys=$2

  jq -e 'type == "array" and length == 0' \
    <<<"$mcp_sync_keys" >/dev/null &&
    jq -e 'type == "array" and length == 0' \
      <<<"$tap_keys" >/dev/null
}

organization_access_assert_secret_boundary() {
  local organization_secrets=$1
  local repository_secrets=$2
  local tap_secrets=$3
  local release_secrets=$4
  local release_control_secrets=$5

  jq -e '.total_count == 0 and (.secrets | length) == 0' \
    <<<"$organization_secrets" >/dev/null &&
    jq -e '.total_count == 0 and (.secrets | length) == 0' \
      <<<"$repository_secrets" >/dev/null &&
    jq -e '.total_count == 0 and (.secrets | length) == 0' \
      <<<"$tap_secrets" >/dev/null &&
    jq -e '.total_count == 0 and (.secrets | length) == 0' \
      <<<"$release_secrets" >/dev/null &&
    jq -e '.total_count == 0 and (.secrets | length) == 0' \
      <<<"$release_control_secrets" >/dev/null
}

organization_access_assert_private_evidence() {
  local evidence=$1
  local member_count=$2
  local owner_count=$3
  local outside_collaborator_count=$4
  local pending_invitation_count=$5
  local team_count=$6
  local all_repository_app_count=$7
  local write_capable_all_repository_app_count=$8

  jq -e \
    --argjson member_count "$member_count" \
    --argjson owner_count "$owner_count" \
    --argjson outside_collaborator_count "$outside_collaborator_count" \
    --argjson pending_invitation_count "$pending_invitation_count" \
    --argjson team_count "$team_count" \
    --argjson all_repository_app_count "$all_repository_app_count" \
    --argjson write_capable_all_repository_app_count \
      "$write_capable_all_repository_app_count" '
      (keys | sort) == ([
        "automation_access_reviewed",
        "expected_counts",
        "independent_recovery_paths_confirmed",
        "least_privilege_access_reviewed",
        "owner_choice_explicitly_accepted",
        "ownership_mode",
        "private_recovery_record_current",
        "recovery_exercise_passed",
        "reviewed_on",
        "schema",
        "secure_methods_only_confirmed"
      ] | sort) and
      .schema == 1 and
      (.reviewed_on | type == "string" and test("^[0-9]{4}-[0-9]{2}-[0-9]{2}$")) and
      .owner_choice_explicitly_accepted == true and
      .secure_methods_only_confirmed == true and
      .independent_recovery_paths_confirmed == true and
      .private_recovery_record_current == true and
      .recovery_exercise_passed == true and
      .least_privilege_access_reviewed == true and
      .automation_access_reviewed == true and
      (
        ($owner_count >= 2 and .ownership_mode == "two-trusted-owners") or
        ($owner_count == 1 and .ownership_mode == "single-owner-recovery")
      ) and
      (.expected_counts | keys | sort) == ([
        "all_repository_app_installations",
        "organization_members",
        "organization_owners",
        "outside_collaborators",
        "pending_invitations",
        "teams",
        "write_capable_all_repository_app_installations"
      ] | sort) and
      .expected_counts.organization_members == $member_count and
      .expected_counts.organization_owners == $owner_count and
      .expected_counts.outside_collaborators == $outside_collaborator_count and
      .expected_counts.pending_invitations == $pending_invitation_count and
      .expected_counts.teams == $team_count and
      .expected_counts.all_repository_app_installations == $all_repository_app_count and
      .expected_counts.write_capable_all_repository_app_installations ==
        $write_capable_all_repository_app_count
    ' <<<"$evidence" >/dev/null
}

organization_access_count_array_pages() {
  local pages

  pages="$(jq -c '.')" || return 1
  jq -e 'type == "array" and all(.[]; type == "array")' \
    <<<"$pages" >/dev/null || return 1
  jq '[.[][]] | length' <<<"$pages"
}

organization_access_main() {
  local organization_access_repository=${1:-}
  local organization_access_organization=EnjoyableWork
  local organization_access_tap_repository=EnjoyableWork/homebrew-tap
  local organization_access_evidence_path=${MCP_SYNC_PRIVATE_ORGANIZATION_EVIDENCE:-}
  local organization_access_script_directory
  local organization_access_api_headers=(
    --header 'Accept: application/vnd.github+json'
    --header 'X-GitHub-Api-Version: 2026-03-10'
  )

  if [[ "$organization_access_repository" != EnjoyableWork/mcp-sync ]]; then
    echo "usage: MCP_SYNC_PRIVATE_ORGANIZATION_EVIDENCE=/private/evidence.json $0 EnjoyableWork/mcp-sync" >&2
    return 2
  fi
  for organization_access_command in gh jq git id stat; do
    if ! command -v "$organization_access_command" >/dev/null 2>&1; then
      echo "organization access verification requires $organization_access_command" >&2
      return 2
    fi
  done
  if ! gh auth status --hostname github.com >/dev/null 2>&1; then
    echo "GitHub CLI must be authenticated with organization administration read access" >&2
    return 1
  fi
  organization_access_script_directory="$(
    cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
    pwd
  )"
  if [[ -z "$organization_access_evidence_path" ||
    ! -f "$organization_access_evidence_path" ||
    -L "$organization_access_evidence_path" ]]; then
    echo "private organization evidence is missing or unsafe; details remain outside the repository" >&2
    return 1
  fi

  local organization_access_repository_root
  local organization_access_evidence_directory
  local organization_access_evidence_resolved
  local organization_access_evidence_mode
  local organization_access_evidence_owner
  organization_access_repository_root="$(git rev-parse --show-toplevel)"
  organization_access_evidence_directory="$(
    cd -- "$(dirname -- "$organization_access_evidence_path")"
    pwd -P
  )"
  organization_access_evidence_resolved="$organization_access_evidence_directory/$(basename -- "$organization_access_evidence_path")"
  case "$organization_access_evidence_resolved" in
    "$organization_access_repository_root" | "$organization_access_repository_root"/*)
      echo "private organization evidence must remain outside the repository" >&2
      return 1
      ;;
  esac
  organization_access_evidence_mode="$({
    stat -f '%Lp' "$organization_access_evidence_resolved" 2>/dev/null ||
      stat -c '%a' "$organization_access_evidence_resolved"
  })"
  if [[ ! "$organization_access_evidence_mode" =~ ^[0-7]{3,4}$ ]] ||
    ((8#$organization_access_evidence_mode & 077)); then
    echo "private organization evidence permissions are too broad" >&2
    return 1
  fi
  organization_access_evidence_owner="$({
    stat -f '%u' "$organization_access_evidence_resolved" 2>/dev/null ||
      stat -c '%u' "$organization_access_evidence_resolved"
  })"
  if [[ "$organization_access_evidence_owner" != "$(id -u)" ]]; then
    echo "private organization evidence must be owned by the current operator" >&2
    return 1
  fi

  local organization_access_organization_state
  organization_access_organization_state="$(
    gh api "orgs/$organization_access_organization" \
      "${organization_access_api_headers[@]}"
  )"
  if ! organization_access_assert_organization_policy \
    <<<"$organization_access_organization_state"; then
    echo "organization MFA, base-access, repository-creation, or private-fork policy requires remediation" >&2
    return 1
  fi

  local organization_access_scope
  local organization_access_workflow_policy
  for organization_access_scope in \
    "orgs/$organization_access_organization" \
    "repos/$organization_access_repository" \
    "repos/$organization_access_tap_repository"; do
    organization_access_workflow_policy="$(
      gh api "$organization_access_scope/actions/permissions/workflow" \
        "${organization_access_api_headers[@]}"
    )"
    if ! organization_access_assert_workflow_token_policy \
      <<<"$organization_access_workflow_policy"; then
      echo "workflow token defaults require remediation" >&2
      return 1
    fi
  done

  local organization_access_current_main
  organization_access_current_main="$(
    gh api "repos/$organization_access_repository/commits/main" \
      "${organization_access_api_headers[@]}" \
      --jq '.sha'
  )"
  if ! "$organization_access_script_directory/verify-release-repository-controls.sh" \
    "$organization_access_current_main" >/dev/null; then
    echo "protected release credential controls require remediation" >&2
    return 1
  fi

  local organization_access_mcp_sync_keys
  local organization_access_tap_keys
  organization_access_mcp_sync_keys="$(
    gh api "repos/$organization_access_repository/keys?per_page=100" \
      "${organization_access_api_headers[@]}"
  )"
  organization_access_tap_keys="$(
    gh api "repos/$organization_access_tap_repository/keys?per_page=100" \
      "${organization_access_api_headers[@]}"
  )"
  if ! organization_access_assert_deploy_key_boundary \
    "$organization_access_mcp_sync_keys" "$organization_access_tap_keys"; then
    echo "deploy-key scope or state requires private maintainer remediation" >&2
    return 1
  fi

  local organization_access_organization_secrets
  local organization_access_repository_secrets
  local organization_access_tap_secrets
  local organization_access_release_secrets
  local organization_access_release_control_secrets
  organization_access_organization_secrets="$(
    gh api "orgs/$organization_access_organization/actions/secrets" \
      "${organization_access_api_headers[@]}"
  )"
  organization_access_repository_secrets="$(
    gh api "repos/$organization_access_repository/actions/secrets" \
      "${organization_access_api_headers[@]}"
  )"
  organization_access_tap_secrets="$(
    gh api "repos/$organization_access_tap_repository/actions/secrets" \
      "${organization_access_api_headers[@]}"
  )"
  organization_access_release_secrets="$(
    gh api "repos/$organization_access_repository/environments/release/secrets" \
      "${organization_access_api_headers[@]}"
  )"
  organization_access_release_control_secrets="$(
    gh api "repos/$organization_access_repository/environments/release-control/secrets" \
      "${organization_access_api_headers[@]}"
  )"
  if ! organization_access_assert_secret_boundary \
    "$organization_access_organization_secrets" \
    "$organization_access_repository_secrets" \
    "$organization_access_tap_secrets" \
    "$organization_access_release_secrets" \
    "$organization_access_release_control_secrets"; then
    echo "Actions credential scope requires private maintainer remediation" >&2
    return 1
  fi

  local organization_access_members
  local organization_access_owners
  local organization_access_outside_collaborators
  local organization_access_pending_invitations
  local organization_access_teams
  local organization_access_installations
  organization_access_members="$(
    gh api --paginate --slurp \
      "orgs/$organization_access_organization/members?filter=all&role=all&per_page=100" \
      "${organization_access_api_headers[@]}"
  )"
  organization_access_owners="$(
    gh api --paginate --slurp \
      "orgs/$organization_access_organization/members?filter=all&role=admin&per_page=100" \
      "${organization_access_api_headers[@]}"
  )"
  organization_access_outside_collaborators="$(
    gh api --paginate --slurp \
      "orgs/$organization_access_organization/outside_collaborators?per_page=100" \
      "${organization_access_api_headers[@]}"
  )"
  organization_access_pending_invitations="$(
    gh api --paginate --slurp \
      "orgs/$organization_access_organization/invitations?per_page=100" \
      "${organization_access_api_headers[@]}"
  )"
  organization_access_teams="$(
    gh api --paginate --slurp \
      "orgs/$organization_access_organization/teams?per_page=100" \
      "${organization_access_api_headers[@]}"
  )"
  organization_access_installations="$(
    gh api --paginate --slurp \
      "orgs/$organization_access_organization/installations?per_page=100" \
      "${organization_access_api_headers[@]}"
  )"

  local organization_access_member_count
  local organization_access_owner_count
  local organization_access_outside_collaborator_count
  local organization_access_pending_invitation_count
  local organization_access_team_count
  local organization_access_all_repository_app_count
  local organization_access_write_capable_all_repository_app_count
  organization_access_member_count="$(
    organization_access_count_array_pages <<<"$organization_access_members"
  )"
  organization_access_owner_count="$(
    organization_access_count_array_pages <<<"$organization_access_owners"
  )"
  organization_access_outside_collaborator_count="$(
    organization_access_count_array_pages <<<"$organization_access_outside_collaborators"
  )"
  organization_access_pending_invitation_count="$(
    organization_access_count_array_pages <<<"$organization_access_pending_invitations"
  )"
  organization_access_team_count="$(
    organization_access_count_array_pages <<<"$organization_access_teams"
  )"
  organization_access_all_repository_app_count="$(
    jq '[.[] | .installations[]? | select(.repository_selection == "all")] | length' \
      <<<"$organization_access_installations"
  )"
  organization_access_write_capable_all_repository_app_count="$(
    jq '[
        .[] | .installations[]? |
        select(.repository_selection == "all") |
        select(any(.permissions | to_entries[]; .value == "write"))
      ] | length' <<<"$organization_access_installations"
  )"

  local organization_access_private_evidence
  organization_access_private_evidence="$(<"$organization_access_evidence_resolved")"
  if ! organization_access_assert_private_evidence \
    "$organization_access_private_evidence" \
    "$organization_access_member_count" \
    "$organization_access_owner_count" \
    "$organization_access_outside_collaborator_count" \
    "$organization_access_pending_invitation_count" \
    "$organization_access_team_count" \
    "$organization_access_all_repository_app_count" \
    "$organization_access_write_capable_all_repository_app_count"; then
    echo "private access and recovery evidence is stale or incomplete; details remain outside the repository" >&2
    return 1
  fi

  echo "Verified non-disclosing organization access, automation, and recovery controls for $organization_access_repository."
}

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
  organization_access_main "$@"
fi
