#!/usr/bin/env bash

set -euo pipefail

if [[ $# -ne 0 ]]; then
  echo "usage: $0" >&2
  exit 2
fi

cargo_workflow_test_root="$(git rev-parse --show-toplevel)"
cargo_workflow_test_temp="$(
  mktemp -d "${TMPDIR:-/tmp}/mcp-sync-cargo-workflow-policy.XXXXXX"
)"
cleanup_cargo_workflow_test() {
  case "$cargo_workflow_test_temp" in
    "${TMPDIR:-/tmp}/mcp-sync-cargo-workflow-policy."*)
      rm -rf -- "$cargo_workflow_test_temp"
      ;;
    *)
      echo "refusing to remove an unexpected Cargo workflow test path" >&2
      return 1
      ;;
  esac
}
trap cleanup_cargo_workflow_test EXIT

cargo_workflow_prepare_case() {
  local cargo_workflow_case_name=$1
  local cargo_workflow_case_root="$cargo_workflow_test_temp/$cargo_workflow_case_name"
  mkdir -p "$cargo_workflow_case_root/scripts"
  cp -R "$cargo_workflow_test_root/.github" "$cargo_workflow_case_root/"
  cp \
    "$cargo_workflow_test_root/scripts/verify-workflow-supply-chain.sh" \
    "$cargo_workflow_case_root/scripts/"
  chmod +x "$cargo_workflow_case_root/scripts/verify-workflow-supply-chain.sh"
  git -C "$cargo_workflow_case_root" init --quiet
  printf '%s\n' "$cargo_workflow_case_root"
}

cargo_workflow_replace_once() {
  local cargo_workflow_file=$1
  local cargo_workflow_old=$2
  local cargo_workflow_new=$3
  local cargo_workflow_replacement="$cargo_workflow_file.replacement"
  local cargo_workflow_replaced=false

  while IFS= read -r cargo_workflow_line || [[ -n "$cargo_workflow_line" ]]; do
    if [[ "$cargo_workflow_replaced" == false &&
      "$cargo_workflow_line" == *"$cargo_workflow_old"* ]]; then
      printf '%s\n' "${cargo_workflow_line/"$cargo_workflow_old"/"$cargo_workflow_new"}"
      cargo_workflow_replaced=true
    else
      printf '%s\n' "$cargo_workflow_line"
    fi
  done <"$cargo_workflow_file" >"$cargo_workflow_replacement"

  if [[ "$cargo_workflow_replaced" != true ]]; then
    echo "Cargo workflow mutation target was not found" >&2
    exit 1
  fi
  mv "$cargo_workflow_replacement" "$cargo_workflow_file"
}

cargo_workflow_expect_rejection() {
  local cargo_workflow_case_name=$1
  local cargo_workflow_old=$2
  local cargo_workflow_new=$3
  local cargo_workflow_case_root
  cargo_workflow_case_root="$(cargo_workflow_prepare_case "$cargo_workflow_case_name")"
  cargo_workflow_replace_once \
    "$cargo_workflow_case_root/.github/workflows/cargo-publish.yml" \
    "$cargo_workflow_old" \
    "$cargo_workflow_new"
  if (
    cd "$cargo_workflow_case_root"
    ./scripts/verify-workflow-supply-chain.sh >/dev/null 2>&1
  ); then
    echo "workflow policy accepted rejected Cargo case: $cargo_workflow_case_name" >&2
    exit 1
  fi
}

cargo_workflow_accept_root="$(cargo_workflow_prepare_case accepted)"
(
  cd "$cargo_workflow_accept_root"
  ./scripts/verify-workflow-supply-chain.sh >/dev/null
)

cargo_workflow_expect_rejection mutable-auth-action \
  'rust-lang/crates-io-auth-action@c6f97d42243bad5fab37ca0427f495c86d5b1a18' \
  'rust-lang/crates-io-auth-action@v1.0.5'
cargo_workflow_expect_rejection wrong-environment \
  'name: release' \
  'name: staging'
cargo_workflow_expect_rejection missing-local-release-comparison \
  "cmp --silent \"\$release_package\" \"\$local_package\"" \
  'true'
cargo_workflow_expect_rejection missing-registry-comparison \
  "cmp --silent \"\$release_package\" \"\$existing_registry_package\"" \
  'true'
cargo_workflow_expect_rejection missing-rehearsal-nonpublication-proof \
  "if ! cmp --silent \"\$RUNNER_TEMP/enjoyable-mcp-sync-versions-before.json\" \"\$RUNNER_TEMP/enjoyable-mcp-sync-versions-after.json\"; then" \
  'true'
cargo_workflow_expect_rejection missing-trusted-only-publication-gate \
  '.crate.trustpub_only == true' \
  'true'
cargo_workflow_expect_rejection reusable-token-fallback \
  'CARGO_REGISTRY_TOKEN:' \
  'CRATES_IO_TOKEN:'
cargo_workflow_expect_rejection untrusted-trigger \
  'workflow_dispatch:' \
  'pull_request:'
cargo_workflow_expect_rejection historical-deployment-trigger \
  'workflow_dispatch:' \
  'deployment:'
cargo_workflow_expect_rejection missing-oidc \
  'id-token: write' \
  'contents: read'
cargo_workflow_expect_rejection recovery-requests-oidc \
  "needs.validate.outputs.mode == 'authorization-only' ||" \
  'always() ||'
cargo_workflow_expect_rejection missing-global-release-serialization \
  'group: mcp-sync-release' \
  'group: cargo-publish'

printf 'Verified Cargo publisher workflow acceptance and fail-closed policy mutations.\n'
