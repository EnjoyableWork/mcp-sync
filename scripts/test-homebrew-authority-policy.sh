#!/usr/bin/env bash

set -euo pipefail

if [[ $# -ne 0 ]]; then
  echo "usage: $0" >&2
  exit 2
fi

homebrew_authority_root=$(git rev-parse --show-toplevel)
homebrew_authority_workflows="$homebrew_authority_root/.github/workflows"
homebrew_authority_obsolete="$homebrew_authority_workflows/source-linux-release-publish-homebrew.yml"

if [[ -e "$homebrew_authority_obsolete" ]]; then
  echo "mcp-sync must not retain the obsolete cross-repository Homebrew publisher" >&2
  exit 1
fi

for homebrew_authority_forbidden in \
  HOMEBREW_TAP_DEPLOY_KEY \
  git@github.com:EnjoyableWork/homebrew-tap.git \
  GIT_SSH_COMMAND= \
  PERSONAL_ACCESS_TOKEN \
  repository_dispatch: \
  workflow_call:; do
  if grep -R -F -- "$homebrew_authority_forbidden" \
    "$homebrew_authority_workflows" >/dev/null; then
    echo "source workflows retain forbidden Homebrew authority: $homebrew_authority_forbidden" >&2
    exit 1
  fi
done

homebrew_authority_source_release="$homebrew_authority_workflows/source-linux-release.yml"
# These are literal GitHub expression markers in the workflow source.
# shellcheck disable=SC2016
for homebrew_authority_required in \
  'group: mcp-sync-release' \
  'enjoyable-mcp-sync-${{ env.RELEASE_VERSION }}.crate' \
  'release-assets/mcp-sync.rb' \
  'rust_target: aarch64-unknown-linux-gnu' \
  'rust_target: x86_64-unknown-linux-gnu' \
  'SBOM_PATH: release-assets/mcp-sync-v${{ env.RELEASE_VERSION }}-${{ matrix.rust_target }}.spdx.json' \
  'gh attestation verify' \
  'No tag, release, crate, formula, or credential was created or changed.'; do
  if ! grep -F -- "$homebrew_authority_required" \
    "$homebrew_authority_source_release" >/dev/null; then
    echo "source release lost required immutable handoff evidence: $homebrew_authority_required" >&2
    exit 1
  fi
done

homebrew_authority_runbook="$homebrew_authority_root/docs/source-linux-release.md"
# Backticks are literal Markdown delimiters in the runbook policy marker.
# shellcheck disable=SC2016
for homebrew_authority_required in \
  'publish-mcp-sync.yml' \
  '--repo EnjoyableWork/homebrew-tap' \
  '--ref main' \
  '-f mode=publish' \
  'job-scoped `GITHUB_TOKEN`'; do
  if ! grep -F -- "$homebrew_authority_required" "$homebrew_authority_runbook" >/dev/null; then
    echo "source release runbook lacks the tap-owned handoff: $homebrew_authority_required" >&2
    exit 1
  fi
done
if grep -F 'source-linux-release-publish-homebrew.yml' \
  "$homebrew_authority_runbook" >/dev/null; then
  echo "source release runbook still names the obsolete publisher" >&2
  exit 1
fi

homebrew_authority_verifier="$homebrew_authority_root/scripts/verify-organization-access-controls.sh"
if grep -F 'HOMEBREW_TAP_DEPLOY_KEY' "$homebrew_authority_verifier" >/dev/null ||
  [[ "$(grep -F -c 'type == "array" and length == 0' "$homebrew_authority_verifier")" -lt 2 ]]; then
  echo "organization access policy must require zero source secrets and zero tap deploy keys" >&2
  exit 1
fi

"$homebrew_authority_root/scripts/test-homebrew-formula-update-policy.sh" >/dev/null
printf '%s\n' \
  'Verified credential-free source ownership, immutable formula generation, and manual tap-owned Homebrew handoff.'
