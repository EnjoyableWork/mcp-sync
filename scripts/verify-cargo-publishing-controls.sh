#!/usr/bin/env bash

set -euo pipefail

cargo_controls_commit=${1:-}
cargo_controls_publisher_source=${2:-}

if [[ ! "$cargo_controls_commit" =~ ^[0-9a-f]{40}$ ]] ||
  [[ -z "$cargo_controls_publisher_source" ]] ||
  [[ $# -ne 2 ]]; then
  echo "usage: $0 <exact-main-commit> <trusted-publisher-json-file|->" >&2
  exit 2
fi
if ! gh auth status --hostname github.com >/dev/null 2>&1; then
  echo "GitHub CLI must be authenticated with repository administration read access" >&2
  exit 1
fi
for cargo_controls_command in base64 cmp curl gh git jq; do
  if ! command -v "$cargo_controls_command" >/dev/null 2>&1; then
    echo "Cargo publishing control verification requires $cargo_controls_command" >&2
    exit 2
  fi
done

cargo_controls_root="$(git rev-parse --show-toplevel)"
cargo_controls_temp="$(
  mktemp -d "${TMPDIR:-/tmp}/mcp-sync-cargo-controls.XXXXXX"
)"
cleanup_cargo_controls() {
  case "$cargo_controls_temp" in
    "${TMPDIR:-/tmp}/mcp-sync-cargo-controls."*)
      rm -rf -- "$cargo_controls_temp"
      ;;
    *)
      echo "refusing to remove an unexpected Cargo control path" >&2
      return 1
      ;;
  esac
}
trap cleanup_cargo_controls EXIT

cargo_controls_head="$(git -C "$cargo_controls_root" rev-parse HEAD)"
cargo_controls_main="$(
  gh api repos/EnjoyableWork/mcp-sync/commits/main \
    -H 'X-GitHub-Api-Version: 2026-03-10' \
    --jq .sha
)"
if [[ "$cargo_controls_head" != "$cargo_controls_commit" ||
  "$cargo_controls_main" != "$cargo_controls_commit" ]] ||
  ! git -C "$cargo_controls_root" diff --quiet ||
  ! git -C "$cargo_controls_root" diff --cached --quiet; then
  echo "Cargo publishing control verification requires a clean checkout of exact main" >&2
  exit 1
fi

cargo_controls_repository_state="$cargo_controls_temp/repository.json"
gh api repos/EnjoyableWork/mcp-sync \
  -H 'X-GitHub-Api-Version: 2026-03-10' \
  >"$cargo_controls_repository_state"
cargo_controls_owner_id="$(
  jq -r '
    select(.full_name == "EnjoyableWork/mcp-sync" and
           .owner.login == "EnjoyableWork" and
           (.owner.id | type) == "number") |
    .owner.id
  ' "$cargo_controls_repository_state"
)"
if [[ ! "$cargo_controls_owner_id" =~ ^[0-9]+$ ]]; then
  echo "live GitHub repository owner identity could not be verified" >&2
  exit 1
fi

cargo_controls_auth_action="$cargo_controls_temp/crates-io-auth-action.json"
gh api \
  repos/rust-lang/crates-io-auth-action/commits/c6f97d42243bad5fab37ca0427f495c86d5b1a18 \
  -H 'X-GitHub-Api-Version: 2026-03-10' \
  >"$cargo_controls_auth_action"
jq -e '
  .sha == "c6f97d42243bad5fab37ca0427f495c86d5b1a18" and
  .commit.verification.verified == true and
  .commit.verification.reason == "valid" and
  .commit.verification.signature != null and
  .commit.verification.payload != null
' "$cargo_controls_auth_action" >/dev/null

cargo_controls_forbidden_credential_pattern='((CRATES?(_?IO)?|CARGO|REGISTRY).*(TOKEN|KEY|CREDENTIAL)|(TOKEN|KEY|CREDENTIAL).*(CRATES?(_?IO)?|CARGO|REGISTRY))'
verify_no_cargo_credential_names() {
  local cargo_controls_endpoint=$1
  local cargo_controls_collection=$2
  local cargo_controls_label=$3
  local cargo_controls_state="$cargo_controls_temp/$cargo_controls_label.json"

  gh api "$cargo_controls_endpoint" \
    -H 'X-GitHub-Api-Version: 2026-03-10' \
    >"$cargo_controls_state"
  jq -e \
    --arg collection "$cargo_controls_collection" \
    --arg forbidden "$cargo_controls_forbidden_credential_pattern" '
      (.total_count | type) == "number" and
      (.[$collection] | type) == "array" and
      .total_count == (.[$collection] | length) and
      all(.[$collection][];
        (.name | type) == "string" and
        ((.name | ascii_upcase | test($forbidden)) | not))
    ' "$cargo_controls_state" >/dev/null
}

verify_no_cargo_credential_names \
  'repos/EnjoyableWork/mcp-sync/actions/secrets?per_page=100' \
  secrets \
  repository-secrets
verify_no_cargo_credential_names \
  'repos/EnjoyableWork/mcp-sync/actions/variables?per_page=100' \
  variables \
  repository-variables
verify_no_cargo_credential_names \
  'repos/EnjoyableWork/mcp-sync/environments/release/secrets?per_page=100' \
  secrets \
  release-secrets
verify_no_cargo_credential_names \
  'repos/EnjoyableWork/mcp-sync/environments/release/variables?per_page=100' \
  variables \
  release-variables
verify_no_cargo_credential_names \
  'orgs/EnjoyableWork/actions/secrets?per_page=100' \
  secrets \
  organization-secrets
verify_no_cargo_credential_names \
  'orgs/EnjoyableWork/actions/variables?per_page=100' \
  variables \
  organization-variables

"$cargo_controls_root/scripts/verify-release-repository-controls.sh" \
  "$cargo_controls_commit"
"$cargo_controls_root/scripts/verify-repository-supply-chain-controls.sh" \
  EnjoyableWork/mcp-sync \
  0.1.0

cargo_controls_remote_workflow="$cargo_controls_temp/cargo-publish.yml"
gh api \
  'repos/EnjoyableWork/mcp-sync/contents/.github/workflows/cargo-publish.yml?ref=main' \
  -H 'X-GitHub-Api-Version: 2026-03-10' \
  --jq .content \
  | base64 --decode >"$cargo_controls_remote_workflow"
cmp --silent \
  "$cargo_controls_root/.github/workflows/cargo-publish.yml" \
  "$cargo_controls_remote_workflow"

cargo_controls_workflow_state="$cargo_controls_temp/workflow.json"
gh api \
  repos/EnjoyableWork/mcp-sync/actions/workflows/cargo-publish.yml \
  -H 'X-GitHub-Api-Version: 2026-03-10' \
  >"$cargo_controls_workflow_state"
jq -e '
  .name == "Publish Cargo package with trusted publishing" and
  .path == ".github/workflows/cargo-publish.yml" and
  .state == "active"
' "$cargo_controls_workflow_state" >/dev/null

cargo_controls_crate_state="$cargo_controls_temp/crate.json"
curl --fail --silent --show-error --location --retry 5 \
  --proto '=https' --proto-redir '=https' --tlsv1.2 \
  --output "$cargo_controls_crate_state" \
  --header 'User-Agent: mcp-sync-cargo-control-verifier/0.1 (+https://github.com/EnjoyableWork/mcp-sync)' \
  'https://crates.io/api/v1/crates/enjoyable-mcp-sync'
jq -e '
  .crate.id == "enjoyable-mcp-sync" and
  .crate.repository == "https://github.com/EnjoyableWork/mcp-sync" and
  .crate.trustpub_only == true and
  any(.versions[]; .num == "0.1.0" and .yanked == false)
' "$cargo_controls_crate_state" >/dev/null

cargo_controls_publisher_state="$cargo_controls_temp/trusted-publisher.json"
if [[ "$cargo_controls_publisher_source" == - ]]; then
  jq -e . >"$cargo_controls_publisher_state"
elif [[ -f "$cargo_controls_publisher_source" &&
  ! -L "$cargo_controls_publisher_source" ]]; then
  jq -e . "$cargo_controls_publisher_source" >"$cargo_controls_publisher_state"
else
  echo "trusted-publisher readback must be a regular JSON file or standard input" >&2
  exit 1
fi
jq -e \
  --argjson expected_owner_id "$cargo_controls_owner_id" '
  (.github_configs | length) == 1 and
  .meta.total == 1 and
  .github_configs[0].krate == "enjoyable-mcp-sync" and
  .github_configs[0].repository_owner == "EnjoyableWork" and
  .github_configs[0].repository_owner_id == $expected_owner_id and
  .github_configs[0].repository_name == "mcp-sync" and
  .github_configs[0].workflow_filename == "cargo-publish.yml" and
  .github_configs[0].environment == "release"
' "$cargo_controls_publisher_state" >/dev/null

printf 'Verified exact-main Cargo workflow, release protections, absent registry-token fallback, signed selected action, trusted publisher, trusted-only setting, and immutable v0.1.0 identity.\n'
