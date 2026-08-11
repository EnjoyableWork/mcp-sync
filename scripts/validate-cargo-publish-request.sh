#!/usr/bin/env bash

set -euo pipefail

if [[ $# -ne 0 ]]; then
  echo "usage: $0" >&2
  exit 2
fi

cargo_publish_request_repository=${REQUEST_REPOSITORY:-}
cargo_publish_request_event=${REQUEST_EVENT:-}
cargo_publish_request_ref=${REQUEST_REF:-}
cargo_publish_request_ref_type=${REQUEST_REF_TYPE:-}
cargo_publish_request_sha=${REQUEST_SHA:-}
cargo_publish_request_version=${REQUEST_VERSION:-}
cargo_publish_request_tag=${REQUEST_TAG:-}
cargo_publish_request_kind=${REQUEST_RELEASE_KIND:-}
cargo_publish_request_mode=${REQUEST_MODE:-}
cargo_publish_request_deployment_ref=${DEPLOYMENT_REF:-}
cargo_publish_request_deployment_sha=${DEPLOYMENT_SHA:-}
cargo_publish_request_deployment_task=${DEPLOYMENT_TASK:-}
cargo_publish_request_deployment_environment=${DEPLOYMENT_ENVIRONMENT:-}
cargo_publish_request_contract=${DEPLOYMENT_CONTRACT:-}

cargo_publish_request_fail() {
  echo "$1" >&2
  exit 1
}

if [[ "$cargo_publish_request_repository" != EnjoyableWork/mcp-sync ]]; then
  cargo_publish_request_fail "Cargo publication is restricted to EnjoyableWork/mcp-sync"
fi
if [[ ! "$cargo_publish_request_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  cargo_publish_request_fail "Cargo publication requires a stable semantic version"
fi
if [[ "$cargo_publish_request_tag" != "v$cargo_publish_request_version" ]]; then
  cargo_publish_request_fail "Cargo publication tag must exactly match the requested version"
fi
if [[ "$cargo_publish_request_ref_type" != tag ]] ||
  [[ "$cargo_publish_request_ref" != "refs/tags/$cargo_publish_request_tag" ]]; then
  cargo_publish_request_fail "Cargo publication must target the exact existing release tag"
fi
case "$cargo_publish_request_kind" in
  source-linux | funded) ;;
  *) cargo_publish_request_fail "Cargo publication release kind is not supported" ;;
esac
case "$cargo_publish_request_mode" in
  authorization-only | publish) ;;
  *) cargo_publish_request_fail "Cargo publication mode is not supported" ;;
esac

case "$cargo_publish_request_event" in
  workflow_dispatch)
    if [[ -n "$cargo_publish_request_deployment_ref" ||
      -n "$cargo_publish_request_deployment_sha" ||
      -n "$cargo_publish_request_deployment_task" ||
      -n "$cargo_publish_request_deployment_environment" ||
      -n "$cargo_publish_request_contract" ]]; then
      cargo_publish_request_fail "Manual Cargo publication received unexpected deployment data"
    fi
    ;;
  deployment)
    if [[ "$cargo_publish_request_deployment_task" != mcp-sync:cargo-publish-authorization ]] ||
      [[ "$cargo_publish_request_deployment_environment" != release ]] ||
      [[ "$cargo_publish_request_contract" != MCP-039 ]]; then
      cargo_publish_request_fail "Cargo authorization deployment does not match the rehearsal contract"
    fi
    if [[ "$cargo_publish_request_deployment_ref" != "$cargo_publish_request_tag" ]] ||
      [[ "$cargo_publish_request_deployment_sha" != "$cargo_publish_request_sha" ]]; then
      cargo_publish_request_fail "Cargo authorization deployment does not match its exact tag commit"
    fi
    if [[ "$cargo_publish_request_mode" != authorization-only ||
      "$cargo_publish_request_version" != 0.1.0 ||
      "$cargo_publish_request_tag" != v0.1.0 ||
      "$cargo_publish_request_kind" != source-linux ]]; then
      cargo_publish_request_fail "Deployment events are limited to the non-publishing v0.1.0 rehearsal"
    fi
    ;;
  *) cargo_publish_request_fail "Cargo publication event is not authorized" ;;
esac

if [[ "$cargo_publish_request_mode" == publish &&
  "$cargo_publish_request_version" == 0.1.0 ]]; then
  cargo_publish_request_fail "Cargo 0.1.0 is immutable and cannot be republished"
fi

if [[ -z "${GITHUB_OUTPUT:-}" ]]; then
  cargo_publish_request_fail "Cargo publication output boundary is unavailable"
fi
{
  printf 'version=%s\n' "$cargo_publish_request_version"
  printf 'tag=%s\n' "$cargo_publish_request_tag"
  printf 'release_kind=%s\n' "$cargo_publish_request_kind"
  printf 'mode=%s\n' "$cargo_publish_request_mode"
} >>"$GITHUB_OUTPUT"
