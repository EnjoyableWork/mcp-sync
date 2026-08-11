#!/usr/bin/env bash

set -euo pipefail

workflow_supply_chain_mode=${1:-}

if [[ $# -gt 1 || ( -n "$workflow_supply_chain_mode" && "$workflow_supply_chain_mode" != --verify-upstreams ) ]]; then
  echo "usage: $0 [--verify-upstreams]" >&2
  exit 2
fi

workflow_supply_chain_root="$(git rev-parse --show-toplevel)"
workflow_supply_chain_policy="$workflow_supply_chain_root/.github/actions-policy.json"
workflow_supply_chain_files="$workflow_supply_chain_root/.github/workflows"

for workflow_supply_chain_command in awk curl cut find git grep jq sed sort; do
  if ! command -v "$workflow_supply_chain_command" >/dev/null 2>&1; then
    echo "workflow supply-chain verification requires $workflow_supply_chain_command" >&2
    exit 2
  fi
done

jq -e '
    .github_owned_allowed == false and
    .verified_allowed == false and
    (.patterns_allowed | type == "array" and length == 11 and
      length == (unique | length) and . == sort and
      all(.[]; test("^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+(/[A-Za-z0-9_.-]+)?@\\*$"))) and
    (.reviewed_transitive_actions == [{
      action: "actions/attest",
      parent_action: "actions/attest-build-provenance",
      parent_sha: "4d101475d8b20a2381f78447822ac1eab6504dd8",
      sha: "508db95dd578ae2727ebd6217d5ba78e4fbda05d",
      update_hint: "v4.2.1"
    }])
  ' "$workflow_supply_chain_policy" >/dev/null

workflow_supply_chain_action_regex='^uses:[[:space:]]+([A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+(/[A-Za-z0-9_.-]+)?)@([0-9a-f]{40})[[:space:]]+#[[:space:]]+([A-Za-z0-9][A-Za-z0-9._-]*)'
workflow_supply_chain_expression_marker="\${{"
workflow_supply_chain_records=()
workflow_supply_chain_count=0

while IFS= read -r -d '' workflow_supply_chain_file; do
  if grep -nE '^[[:space:]]+(pull_request_target|workflow_run):' \
    "$workflow_supply_chain_file" >/dev/null; then
    echo "privileged or chained untrusted trigger is not permitted: $workflow_supply_chain_file" >&2
    exit 1
  fi

  if grep -qE '^  pull_request:' "$workflow_supply_chain_file"; then
    if grep -qE \
      '^[[:space:]]*permissions:.*write|^[[:space:]]+[A-Za-z0-9_-]+:[[:space:]]*write([[:space:]#]|$)' \
      "$workflow_supply_chain_file"; then
      echo "pull-request workflow contains write authority: $workflow_supply_chain_file" >&2
      exit 1
    fi
    for workflow_supply_chain_forbidden in \
      'secrets(\.|\[)' \
      '^[[:space:]]+secrets:' \
      '^[[:space:]]+environment:' \
      '^[[:space:]]+run-id:' \
      '^[[:space:]]+github-token:'; do
      if grep -qE "$workflow_supply_chain_forbidden" "$workflow_supply_chain_file"; then
        echo "pull-request workflow contains privileged credentials or cross-run assets: $workflow_supply_chain_file" >&2
        exit 1
      fi
    done
  fi

  workflow_supply_chain_in_run=false
  workflow_supply_chain_run_indent=0
  while IFS= read -r workflow_supply_chain_line || [[ -n "$workflow_supply_chain_line" ]]; do
    workflow_supply_chain_leading=${workflow_supply_chain_line%%[! ]*}
    workflow_supply_chain_indent=${#workflow_supply_chain_leading}
    workflow_supply_chain_trimmed=${workflow_supply_chain_line#"$workflow_supply_chain_leading"}

    if [[ "$workflow_supply_chain_in_run" == true &&
      -n "$workflow_supply_chain_trimmed" &&
      "$workflow_supply_chain_indent" -le "$workflow_supply_chain_run_indent" ]]; then
      workflow_supply_chain_in_run=false
    fi
    if [[ "$workflow_supply_chain_in_run" == true && "$workflow_supply_chain_line" == *"$workflow_supply_chain_expression_marker"* ]]; then
      echo "GitHub expressions must enter shell steps through a validated env boundary: $workflow_supply_chain_file" >&2
      exit 1
    fi
    if [[ "$workflow_supply_chain_trimmed" == run:* ]]; then
      if [[ "$workflow_supply_chain_trimmed" == *"$workflow_supply_chain_expression_marker"* ]]; then
        echo "inline GitHub expressions are not permitted in shell commands: $workflow_supply_chain_file" >&2
        exit 1
      fi
      if [[ "$workflow_supply_chain_trimmed" == 'run: |'* ||
        "$workflow_supply_chain_trimmed" == 'run: >'* ]]; then
        workflow_supply_chain_in_run=true
        workflow_supply_chain_run_indent=$workflow_supply_chain_indent
      fi
    fi

    if [[ "$workflow_supply_chain_trimmed" != uses:* ]]; then
      continue
    fi
    if [[ "$workflow_supply_chain_trimmed" == uses:\ ./* ]]; then
      continue
    fi
    if [[ ! "$workflow_supply_chain_trimmed" =~ $workflow_supply_chain_action_regex ]]; then
      echo "action must use a reviewed full commit SHA and update hint: $workflow_supply_chain_file: $workflow_supply_chain_trimmed" >&2
      exit 1
    fi

    workflow_supply_chain_action=${BASH_REMATCH[1]}
    workflow_supply_chain_sha=${BASH_REMATCH[3]}
    workflow_supply_chain_hint=${BASH_REMATCH[4]}
    if ! jq -e --arg pattern "$workflow_supply_chain_action@*" \
      '.patterns_allowed | index($pattern) != null' \
      "$workflow_supply_chain_policy" >/dev/null; then
      echo "action is absent from the selected-action policy: $workflow_supply_chain_action" >&2
      exit 1
    fi
    workflow_supply_chain_records+=(
      "$workflow_supply_chain_action $workflow_supply_chain_sha $workflow_supply_chain_hint"
    )
    workflow_supply_chain_count=$((workflow_supply_chain_count + 1))
  done <"$workflow_supply_chain_file"

  while IFS=: read -r workflow_supply_chain_line_number _; do
    workflow_supply_chain_checkout_end=$((workflow_supply_chain_line_number + 5))
    if ! sed -n "${workflow_supply_chain_line_number},${workflow_supply_chain_checkout_end}p" \
      "$workflow_supply_chain_file" | grep -q 'persist-credentials: false'; then
      echo "checkout must disable persisted credentials: $workflow_supply_chain_file:$workflow_supply_chain_line_number" >&2
      exit 1
    fi
  done < <(grep -nE 'uses:[[:space:]]+actions/checkout@' "$workflow_supply_chain_file" || true)
done < <(find "$workflow_supply_chain_files" -type f -name '*.yml' -print0)

if [[ "$workflow_supply_chain_count" -eq 0 ]]; then
  echo "no workflow action references were inspected" >&2
  exit 1
fi

while IFS= read -r workflow_supply_chain_pattern; do
  workflow_supply_chain_action=${workflow_supply_chain_pattern%@\*}
  if [[ "$workflow_supply_chain_action" == actions/attest ]]; then
    continue
  fi
  if ! printf '%s\n' "${workflow_supply_chain_records[@]}" |
    grep -qE "^${workflow_supply_chain_action//\//\\/} "; then
    echo "selected-action policy contains an unused action: $workflow_supply_chain_action" >&2
    exit 1
  fi
done < <(jq -r '.patterns_allowed[]' "$workflow_supply_chain_policy")

workflow_supply_chain_homebrew="$workflow_supply_chain_files/source-linux-release-publish-homebrew.yml"
workflow_supply_chain_validate_block="$(
  sed -n '/^  validate:/,/^  publish:/p' "$workflow_supply_chain_homebrew"
)"
workflow_supply_chain_homebrew_publish="$(
  sed -n '/^  publish:/,$p' "$workflow_supply_chain_homebrew"
)"
for workflow_supply_chain_required in \
  'permissions: {}' \
  "REQUESTED_VERSION: \${{ inputs.version }}" \
  "DISPATCH_REF: \${{ github.ref }}" \
  "DISPATCH_REF_PROTECTED: \${{ github.ref_protected }}" \
  "DISPATCH_REF_TYPE: \${{ github.ref_type }}" \
  'stable_version_pattern=' \
  'later Homebrew publication requires the exact canonical release tag' \
  'release_version=%s'; do
  if ! grep -F "$workflow_supply_chain_required" \
    <<<"$workflow_supply_chain_validate_block" >/dev/null; then
    echo "Homebrew publisher lacks pre-environment input validation: $workflow_supply_chain_required" >&2
    exit 1
  fi
done

for workflow_supply_chain_release_writer in \
  source-linux-release.yml \
  release.yml \
  cargo-publish.yml \
  source-linux-release-publish-homebrew.yml; do
  if ! grep -F 'group: mcp-sync-release' \
    "$workflow_supply_chain_files/$workflow_supply_chain_release_writer" >/dev/null; then
    echo "release writer is outside the global serialization boundary: $workflow_supply_chain_release_writer" >&2
    exit 1
  fi
done

workflow_supply_chain_source_release="$workflow_supply_chain_files/source-linux-release.yml"
workflow_supply_chain_source_request="$(
  sed -n '/^  request:/,/^  validate:/p' "$workflow_supply_chain_source_release"
)"
for workflow_supply_chain_required in \
  'permissions: {}' \
  "REQUEST_REF: \${{ github.ref }}" \
  "REQUEST_REF_PROTECTED: \${{ github.ref_protected }}" \
  "REQUEST_REF_TYPE: \${{ github.ref_type }}" \
  "REQUESTED_VERSION: \${{ inputs.version }}" \
  'canonical later version on its exact protected tag' \
  'version=%s'; do
  if ! grep -F "$workflow_supply_chain_required" \
    <<<"$workflow_supply_chain_source_request" >/dev/null; then
    echo "source release lacks pre-environment request validation: $workflow_supply_chain_required" >&2
    exit 1
  fi
done
for workflow_supply_chain_forbidden in 'environment:' 'secrets.' 'uses: actions/checkout'; do
  if grep -F "$workflow_supply_chain_forbidden" \
    <<<"$workflow_supply_chain_source_request" >/dev/null; then
    echo "source release request validation must remain unprivileged: $workflow_supply_chain_forbidden" >&2
    exit 1
  fi
done
for workflow_supply_chain_forbidden in 'environment:' 'secrets.' 'uses: actions/checkout'; do
  if grep -F "$workflow_supply_chain_forbidden" \
    <<<"$workflow_supply_chain_validate_block" >/dev/null; then
    echo "Homebrew validation job must remain unprivileged: $workflow_supply_chain_forbidden" >&2
    exit 1
  fi
done
for workflow_supply_chain_required in \
  'scripts/validate-homebrew-formula-update.sh' \
  'prior-homebrew-release.json' \
  "cmp --silent \"\$tap_formula\" \"\$prior_formula_directory/mcp-sync.rb\"" \
  'ls-remote origin refs/heads/main' \
  'secrets.HOMEBREW_TAP_DEPLOY_KEY'; do
  if ! grep -F "$workflow_supply_chain_required" \
    <<<"$workflow_supply_chain_homebrew_publish" >/dev/null; then
    echo "Homebrew publisher lacks monotonic exact-byte handoff behavior: $workflow_supply_chain_required" >&2
    exit 1
  fi
done

workflow_supply_chain_cargo="$workflow_supply_chain_files/cargo-publish.yml"
workflow_supply_chain_cargo_validate="$({
  sed -n '/^  validate:/,/^  publish:/p' "$workflow_supply_chain_cargo"
})"
workflow_supply_chain_cargo_publish="$({
  sed -n '/^  publish:/,/^  cargo-unix:/p' "$workflow_supply_chain_cargo"
})"

for workflow_supply_chain_required in \
  'workflow_dispatch:' \
  'version:' \
  'tag:' \
  'release_kind:' \
  'mode:'; do
  if ! grep -F -- "$workflow_supply_chain_required" \
    "$workflow_supply_chain_cargo" >/dev/null; then
    echo "Cargo publisher lacks an explicit manual input: $workflow_supply_chain_required" >&2
    exit 1
  fi
done
for workflow_supply_chain_required in \
  "REQUEST_WORKFLOW_SHA: \${{ github.workflow_sha }}" \
  "REQUEST_REF_PROTECTED: \${{ github.ref_protected }}" \
  'scripts/validate-cargo-publish-request.sh'; do
  if ! grep -F -- "$workflow_supply_chain_required" \
    <<<"$workflow_supply_chain_cargo_validate" >/dev/null; then
    echo "Cargo publisher lacks unprivileged request validation: $workflow_supply_chain_required" >&2
    exit 1
  fi
done
for workflow_supply_chain_forbidden in 'environment:' 'id-token: write' 'crates-io-auth-action'; do
  if grep -F -- "$workflow_supply_chain_forbidden" \
    <<<"$workflow_supply_chain_cargo_validate" >/dev/null; then
    echo "Cargo publisher validation reaches protected authorization: $workflow_supply_chain_forbidden" >&2
    exit 1
  fi
done

for workflow_supply_chain_required in \
  'name: release' \
  'deployment: false' \
  'id-token: write' \
  'gh release verify' \
  'gh attestation verify' \
  'scripts/verify-published-source-linux-release.sh' \
  'scripts/verify-published-release.sh' \
  'cargo package --locked' \
  'Cargo authorization rehearsal does not use exact protected main' \
  "\"\$GITHUB_WORKFLOW_SHA\" != \"\$GITHUB_SHA\"" \
  "cmp --silent \"\$first_package\" \"\$local_package\"" \
  "cmp --silent \"\$release_package\" \"\$local_package\"" \
  '.crate.trustpub_only == true' \
  '.crate["trustpub_only"] == true' \
  'scripts/validate-release-version.sh' \
  'future' \
  'enjoyable-mcp-sync-existing.crate' \
  "if ! cmp --silent \"\$release_package\" \"\$existing_registry_package\"; then" \
  'existing Cargo version differs from the immutable GitHub Release bytes' \
  "printf 'publish=false\\n'" \
  "needs.validate.outputs.mode == 'authorization-only' ||" \
  "steps.package.outputs.publish == 'true'" \
  'rust-lang/crates-io-auth-action@c6f97d42243bad5fab37ca0427f495c86d5b1a18 # v1.0.5' \
  "CARGO_REGISTRY_TOKEN: \${{ steps.crates_io.outputs.token }}" \
  'cargo publish --locked --registry crates-io' \
  'Require the MCP-039 rehearsal to create no Cargo version' \
  'enjoyable-mcp-sync-versions-before.json' \
  'enjoyable-mcp-sync-versions-after.json' \
  "if ! cmp --silent \"\$RUNNER_TEMP/enjoyable-mcp-sync-versions-before.json\" \"\$RUNNER_TEMP/enjoyable-mcp-sync-versions-after.json\"; then" \
  "cmp --silent \"\$release_package\" \"\$registry_package\""; do
  if ! grep -F -- "$workflow_supply_chain_required" \
    <<<"$workflow_supply_chain_cargo_publish" >/dev/null; then
    echo "Cargo publisher lacks protected fail-closed behavior: $workflow_supply_chain_required" >&2
    exit 1
  fi
done

if grep -qE '^  deployment:[[:space:]]*$' "$workflow_supply_chain_cargo"; then
  echo "Cargo publisher contains a forbidden deployment trigger" >&2
  exit 1
fi

if [[ "$(grep -c 'id-token: write' "$workflow_supply_chain_cargo")" != 1 ]] ||
  [[ "$(grep -c 'rust-lang/crates-io-auth-action@' "$workflow_supply_chain_cargo")" != 1 ]] ||
  [[ "$(grep -c 'CARGO_REGISTRY_TOKEN:' "$workflow_supply_chain_cargo")" != 1 ]]; then
  echo "Cargo publisher must expose OIDC and its temporary credential through one protected path" >&2
  exit 1
fi
for workflow_supply_chain_forbidden in \
  'pull_request:' \
  'pull_request_target:' \
  'push:' \
  'repository_dispatch:' \
  'workflow_run:' \
  'secrets.' \
  'CRATES_IO_TOKEN' \
  'cargo login' \
  '--token'; do
  if grep -F -- "$workflow_supply_chain_forbidden" \
    "$workflow_supply_chain_cargo" >/dev/null; then
    echo "Cargo publisher contains a forbidden trigger or token fallback: $workflow_supply_chain_forbidden" >&2
    exit 1
  fi
done

workflow_supply_chain_cargo_verify_line="$(
  grep -n 'Verify immutable release, attested package, and deterministic local bytes' \
    "$workflow_supply_chain_cargo" | cut -d: -f1
)"
workflow_supply_chain_cargo_auth_line="$(
  grep -n 'Obtain short-lived crates.io authorization through OIDC' \
    "$workflow_supply_chain_cargo" | cut -d: -f1
)"
if [[ -z "$workflow_supply_chain_cargo_verify_line" ||
  -z "$workflow_supply_chain_cargo_auth_line" ||
  "$workflow_supply_chain_cargo_verify_line" -ge "$workflow_supply_chain_cargo_auth_line" ]]; then
  echo "Cargo publisher must verify immutable bytes before requesting OIDC authorization" >&2
  exit 1
fi

if [[ "$workflow_supply_chain_mode" == --verify-upstreams ]]; then
  while IFS= read -r workflow_supply_chain_record; do
    read -r workflow_supply_chain_action workflow_supply_chain_sha workflow_supply_chain_hint \
      <<<"$workflow_supply_chain_record"
    workflow_supply_chain_repository="$(
      cut -d/ -f1-2 <<<"$workflow_supply_chain_action"
    )"
    workflow_supply_chain_remote_refs="$(
      git ls-remote \
        "https://github.com/$workflow_supply_chain_repository.git" \
        "refs/tags/$workflow_supply_chain_hint" \
        "refs/tags/$workflow_supply_chain_hint^{}"
    )"
    workflow_supply_chain_resolved="$(
      awk '
        $2 ~ /\^\{\}$/ { peeled = $1 }
        $2 !~ /\^\{\}$/ { direct = $1 }
        END { if (peeled != "") print peeled; else print direct }
      ' <<<"$workflow_supply_chain_remote_refs"
    )"
    if [[ "$workflow_supply_chain_resolved" != "$workflow_supply_chain_sha" ]]; then
      echo "update hint does not resolve to the reviewed action SHA: $workflow_supply_chain_action@$workflow_supply_chain_hint" >&2
      exit 1
    fi
  done < <(printf '%s\n' "${workflow_supply_chain_records[@]}" | sort -u)

  workflow_supply_chain_transitive_parent_sha="$(
    jq -r '.reviewed_transitive_actions[0].parent_sha' "$workflow_supply_chain_policy"
  )"
  workflow_supply_chain_transitive_sha="$(
    jq -r '.reviewed_transitive_actions[0].sha' "$workflow_supply_chain_policy"
  )"
  workflow_supply_chain_transitive_hint="$(
    jq -r '.reviewed_transitive_actions[0].update_hint' "$workflow_supply_chain_policy"
  )"
  workflow_supply_chain_parent_definition="$(
    curl --fail --silent --show-error --location \
      --proto '=https' --proto-redir '=https' --tlsv1.2 \
      "https://raw.githubusercontent.com/actions/attest-build-provenance/$workflow_supply_chain_transitive_parent_sha/action.yml"
  )"
  if ! grep -F \
    "uses: actions/attest@$workflow_supply_chain_transitive_sha # $workflow_supply_chain_transitive_hint" \
    <<<"$workflow_supply_chain_parent_definition" >/dev/null; then
    echo "reviewed provenance action no longer contains the recorded transitive pin" >&2
    exit 1
  fi
  workflow_supply_chain_transitive_resolved="$(
    git ls-remote https://github.com/actions/attest.git \
      "refs/tags/$workflow_supply_chain_transitive_hint" \
      "refs/tags/$workflow_supply_chain_transitive_hint^{}" |
      awk '
        $2 ~ /\^\{\}$/ { peeled = $1 }
        $2 !~ /\^\{\}$/ { direct = $1 }
        END { if (peeled != "") print peeled; else print direct }
      '
  )"
  if [[ "$workflow_supply_chain_transitive_resolved" != "$workflow_supply_chain_transitive_sha" ]]; then
    echo "transitive action update hint does not resolve to the reviewed SHA" >&2
    exit 1
  fi
fi

printf 'Verified %d immutable workflow action references, untrusted boundaries, and the selected-action policy.\n' \
  "$workflow_supply_chain_count"
