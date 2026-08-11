#!/usr/bin/env bash

set -euo pipefail

if [[ $# -ne 0 ]]; then
  echo "usage: $0" >&2
  exit 2
fi

cargo_publish_test_root="$(git rev-parse --show-toplevel)"
cargo_publish_test_validator="$cargo_publish_test_root/scripts/validate-cargo-publish-request.sh"
cargo_publish_test_temp="$(
  mktemp -d "${TMPDIR:-/tmp}/mcp-sync-cargo-publish-policy.XXXXXX"
)"
cleanup_cargo_publish_test() {
  case "$cargo_publish_test_temp" in
    "${TMPDIR:-/tmp}/mcp-sync-cargo-publish-policy."*)
      rm -rf -- "$cargo_publish_test_temp"
      ;;
    *)
      echo "refusing to remove an unexpected Cargo publisher test path" >&2
      return 1
      ;;
  esac
}
trap cleanup_cargo_publish_test EXIT

cargo_publish_test_base=(
  REQUEST_REPOSITORY=EnjoyableWork/mcp-sync
  REQUEST_REF_TYPE=tag
  REQUEST_SHA=1111111111111111111111111111111111111111
  REQUEST_WORKFLOW_SHA=1111111111111111111111111111111111111111
  REQUEST_REF_PROTECTED=true
)

cargo_publish_expect_acceptance() {
  local cargo_publish_test_name=$1
  shift
  local cargo_publish_test_output="$cargo_publish_test_temp/$cargo_publish_test_name.output"
  if ! env \
    "${cargo_publish_test_base[@]}" \
    GITHUB_OUTPUT="$cargo_publish_test_output" \
    "$@" \
    "$cargo_publish_test_validator" >/dev/null 2>&1; then
    echo "Cargo publisher policy rejected accepted case: $cargo_publish_test_name" >&2
    exit 1
  fi
  for cargo_publish_test_field in version tag release_kind mode; do
    if [[ "$(grep -c "^$cargo_publish_test_field=" "$cargo_publish_test_output")" != 1 ]]; then
      echo "Cargo publisher policy emitted an invalid accepted output" >&2
      exit 1
    fi
  done
}

cargo_publish_expect_rejection() {
  local cargo_publish_test_name=$1
  shift
  local cargo_publish_test_output="$cargo_publish_test_temp/$cargo_publish_test_name.output"
  if env \
    "${cargo_publish_test_base[@]}" \
    GITHUB_OUTPUT="$cargo_publish_test_output" \
    "$@" \
    "$cargo_publish_test_validator" >/dev/null 2>&1; then
    echo "Cargo publisher policy accepted rejected case: $cargo_publish_test_name" >&2
    exit 1
  fi
  if [[ -s "$cargo_publish_test_output" ]]; then
    echo "Cargo publisher policy emitted outputs for a rejected request" >&2
    exit 1
  fi
}

cargo_publish_expect_acceptance manual-authorization \
  REQUEST_EVENT=workflow_dispatch \
  REQUEST_REF=refs/heads/main \
  REQUEST_REF_TYPE=branch \
  REQUEST_VERSION=0.1.0 \
  REQUEST_TAG=v0.1.0 \
  REQUEST_RELEASE_KIND=source-linux \
  REQUEST_MODE=authorization-only

cargo_publish_expect_acceptance manual-publication \
  REQUEST_EVENT=workflow_dispatch \
  REQUEST_REF=refs/tags/v1.2.3 \
  REQUEST_VERSION=1.2.3 \
  REQUEST_TAG=v1.2.3 \
  REQUEST_RELEASE_KIND=funded \
  REQUEST_MODE=publish

cargo_publish_expect_rejection wrong-repository \
  REQUEST_REPOSITORY=somewhere/mcp-sync \
  REQUEST_EVENT=workflow_dispatch \
  REQUEST_REF=refs/tags/v1.2.3 \
  REQUEST_VERSION=1.2.3 \
  REQUEST_TAG=v1.2.3 \
  REQUEST_RELEASE_KIND=funded \
  REQUEST_MODE=publish

cargo_publish_expect_rejection branch-ref \
  REQUEST_EVENT=workflow_dispatch \
  REQUEST_REF=refs/heads/main \
  REQUEST_REF_TYPE=branch \
  REQUEST_VERSION=1.2.3 \
  REQUEST_TAG=v1.2.3 \
  REQUEST_RELEASE_KIND=funded \
  REQUEST_MODE=publish

cargo_publish_expect_rejection mismatched-tag \
  REQUEST_EVENT=workflow_dispatch \
  REQUEST_REF=refs/tags/v1.2.4 \
  REQUEST_VERSION=1.2.3 \
  REQUEST_TAG=v1.2.4 \
  REQUEST_RELEASE_KIND=funded \
  REQUEST_MODE=publish

cargo_publish_expect_rejection immutable-version \
  REQUEST_EVENT=workflow_dispatch \
  REQUEST_REF=refs/tags/v0.1.0 \
  REQUEST_VERSION=0.1.0 \
  REQUEST_TAG=v0.1.0 \
  REQUEST_RELEASE_KIND=source-linux \
  REQUEST_MODE=publish

cargo_publish_expect_rejection token-mode \
  REQUEST_EVENT=workflow_dispatch \
  REQUEST_REF=refs/tags/v1.2.3 \
  REQUEST_VERSION=1.2.3 \
  REQUEST_TAG=v1.2.3 \
  REQUEST_RELEASE_KIND=funded \
  REQUEST_MODE=api-token

cargo_publish_expect_rejection deployment-event \
  REQUEST_EVENT=deployment \
  REQUEST_REF=refs/heads/main \
  REQUEST_REF_TYPE=branch \
  REQUEST_VERSION=0.1.0 \
  REQUEST_TAG=v0.1.0 \
  REQUEST_RELEASE_KIND=source-linux \
  REQUEST_MODE=authorization-only

cargo_publish_expect_rejection authorization-tag-ref \
  REQUEST_EVENT=workflow_dispatch \
  REQUEST_REF=refs/tags/v0.1.0 \
  REQUEST_VERSION=0.1.0 \
  REQUEST_TAG=v0.1.0 \
  REQUEST_RELEASE_KIND=source-linux \
  REQUEST_MODE=authorization-only

cargo_publish_expect_rejection authorization-unprotected-main \
  REQUEST_EVENT=workflow_dispatch \
  REQUEST_REF=refs/heads/main \
  REQUEST_REF_TYPE=branch \
  REQUEST_REF_PROTECTED=false \
  REQUEST_VERSION=0.1.0 \
  REQUEST_TAG=v0.1.0 \
  REQUEST_RELEASE_KIND=source-linux \
  REQUEST_MODE=authorization-only

cargo_publish_expect_rejection authorization-wrong-workflow-sha \
  REQUEST_EVENT=workflow_dispatch \
  REQUEST_REF=refs/heads/main \
  REQUEST_REF_TYPE=branch \
  REQUEST_WORKFLOW_SHA=2222222222222222222222222222222222222222 \
  REQUEST_VERSION=0.1.0 \
  REQUEST_TAG=v0.1.0 \
  REQUEST_RELEASE_KIND=source-linux \
  REQUEST_MODE=authorization-only

cargo_publish_expect_rejection authorization-wrong-version \
  REQUEST_EVENT=workflow_dispatch \
  REQUEST_REF=refs/heads/main \
  REQUEST_REF_TYPE=branch \
  REQUEST_VERSION=1.2.3 \
  REQUEST_TAG=v1.2.3 \
  REQUEST_RELEASE_KIND=source-linux \
  REQUEST_MODE=authorization-only

cargo_publish_expect_rejection publication-unprotected-tag \
  REQUEST_EVENT=workflow_dispatch \
  REQUEST_REF=refs/tags/v1.2.3 \
  REQUEST_REF_PROTECTED=false \
  REQUEST_VERSION=1.2.3 \
  REQUEST_TAG=v1.2.3 \
  REQUEST_RELEASE_KIND=funded \
  REQUEST_MODE=publish

cargo_publish_expect_rejection leading-zero-publication \
  REQUEST_EVENT=workflow_dispatch \
  REQUEST_REF=refs/tags/v01.2.3 \
  REQUEST_VERSION=01.2.3 \
  REQUEST_TAG=v01.2.3 \
  REQUEST_RELEASE_KIND=source-linux \
  REQUEST_MODE=publish

cargo_publish_expect_rejection prerelease-publication \
  REQUEST_EVENT=workflow_dispatch \
  REQUEST_REF=refs/tags/v1.2.3-rc.1 \
  REQUEST_VERSION=1.2.3-rc.1 \
  REQUEST_TAG=v1.2.3-rc.1 \
  REQUEST_RELEASE_KIND=source-linux \
  REQUEST_MODE=publish

printf 'Verified Cargo publisher request acceptance and fail-closed rejection policy.\n'
