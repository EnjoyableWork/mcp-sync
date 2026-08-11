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
cargo_publish_request_workflow_sha=${REQUEST_WORKFLOW_SHA:-}
cargo_publish_request_ref_protected=${REQUEST_REF_PROTECTED:-}
cargo_publish_request_version=${REQUEST_VERSION:-}
cargo_publish_request_tag=${REQUEST_TAG:-}
cargo_publish_request_kind=${REQUEST_RELEASE_KIND:-}
cargo_publish_request_mode=${REQUEST_MODE:-}

cargo_publish_request_fail() {
  echo "$1" >&2
  exit 1
}

if [[ "$cargo_publish_request_repository" != EnjoyableWork/mcp-sync ]]; then
  cargo_publish_request_fail "Cargo publication is restricted to EnjoyableWork/mcp-sync"
fi
case "$cargo_publish_request_kind" in
  source-linux | funded) ;;
  *) cargo_publish_request_fail "Cargo publication release kind is not supported" ;;
esac
case "$cargo_publish_request_mode" in
  authorization-only | publish) ;;
  *) cargo_publish_request_fail "Cargo publication mode is not supported" ;;
esac

cargo_publish_request_script_root="$(
  cd "$(dirname "${BASH_SOURCE[0]}")"
  pwd
)"
if [[ "$cargo_publish_request_mode" == authorization-only ]]; then
  "$cargo_publish_request_script_root/validate-release-version.sh" \
    rehearsal \
    "$cargo_publish_request_tag" \
    "$cargo_publish_request_version" >/dev/null
else
  "$cargo_publish_request_script_root/validate-release-version.sh" \
    published \
    "$cargo_publish_request_tag" \
    "$cargo_publish_request_version" >/dev/null
fi

if [[ "$cargo_publish_request_event" != workflow_dispatch ]]; then
  cargo_publish_request_fail "Cargo publication event is not authorized"
fi

if [[ "$cargo_publish_request_mode" == authorization-only ]]; then
  if [[ "$cargo_publish_request_version" != 0.1.0 ||
    "$cargo_publish_request_tag" != v0.1.0 ||
    "$cargo_publish_request_kind" != source-linux ]]; then
    cargo_publish_request_fail "Authorization-only mode is limited to the fixed MCP-039 v0.1.0 rehearsal"
  fi
  if [[ "$cargo_publish_request_ref_type" != branch ||
    "$cargo_publish_request_ref" != refs/heads/main ||
    "$cargo_publish_request_ref_protected" != true ||
    "$cargo_publish_request_sha" != "$cargo_publish_request_workflow_sha" ]]; then
    cargo_publish_request_fail "The MCP-039 rehearsal must use the exact protected main workflow revision"
  fi
elif [[ "$cargo_publish_request_ref_type" != tag ||
  "$cargo_publish_request_ref" != "refs/tags/$cargo_publish_request_tag" ||
  "$cargo_publish_request_ref_protected" != true ]]; then
  cargo_publish_request_fail "Cargo publication must target the exact existing release tag"
fi

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
