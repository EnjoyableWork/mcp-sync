#!/usr/bin/env bash

set -euo pipefail

market_verify_script_directory="$(
  cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
  pwd
)"

if [[ $# -ne 2 ]]; then
  echo "usage: $0 <asset directory> <stable version>" >&2
  exit 2
fi

market_asset_directory=$1
market_asset_version=$2

if [[ ! "$market_asset_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "market asset version must be a stable semantic version" >&2
  exit 1
fi
if [[ ! -d "$market_asset_directory" || -L "$market_asset_directory" ]]; then
  echo "market asset directory is missing or symbolic" >&2
  exit 1
fi

market_asset_directory=$(cd -- "$market_asset_directory" && pwd)
market_asset_prefix="mcp-sync-v$market_asset_version"
market_cargo_package="enjoyable-mcp-sync-$market_asset_version.crate"
market_targets=(
  aarch64-unknown-linux-gnu
  x86_64-unknown-linux-gnu
)

market_expected_assets=(
  "$market_cargo_package"
  mcp-sync.rb
)
for market_target in "${market_targets[@]}"; do
  market_expected_assets+=(
    "$market_asset_prefix-$market_target.tar.gz"
    "$market_asset_prefix-$market_target.spdx.json"
  )
done

market_observed_assets=$(
  find "$market_asset_directory" -maxdepth 1 -type f -print \
    | sed 's#^.*/##' \
    | LC_ALL=C sort
)
market_sorted_expected_assets=$(printf '%s\n' "${market_expected_assets[@]}" | LC_ALL=C sort)
if [[ "$market_observed_assets" != "$market_sorted_expected_assets" ]]; then
  echo "market release asset set is incomplete or contains unexpected files" >&2
  exit 1
fi

for market_target in "${market_targets[@]}"; do
  jq -e '
    .spdxVersion == "SPDX-2.3" and
    (.documentNamespace | type == "string") and
    (.packages | type == "array" and length > 0)
  ' "$market_asset_directory/$market_asset_prefix-$market_target.spdx.json" >/dev/null
done

market_temp_parent=${TMPDIR:-/tmp}
market_temp_prefix="${market_temp_parent%/}/mcp-sync-market-verify."
market_temp=$(mktemp -d "${market_temp_prefix}XXXXXX")
cleanup_market_verify() {
  if [[ "$market_temp" != "$market_temp_prefix"* ]]; then
    echo "refusing to remove an unexpected market-verification path" >&2
    return 1
  fi
  if [[ -d "$market_temp" ]]; then
    rm -rf -- "$market_temp"
  fi
}
trap cleanup_market_verify EXIT

market_generated="$market_temp/generated"
"$market_verify_script_directory/generate-market-release-channels.sh" \
  "$market_asset_directory/$market_cargo_package" \
  "$market_asset_version" \
  "$market_generated" >/dev/null
cmp --silent \
  "$market_asset_directory/$market_cargo_package" \
  "$market_generated/cargo/$market_cargo_package"
cmp --silent \
  "$market_asset_directory/mcp-sync.rb" \
  "$market_generated/homebrew/Formula/mcp-sync.rb"

market_sha256() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1"
  else
    shasum -a 256 "$1"
  fi
}

(
  cd -- "$market_asset_directory"
  for market_expected_asset in "${market_expected_assets[@]}"; do
    market_sha256 "$market_expected_asset"
  done | LC_ALL=C sort -k2 >SHA256SUMS
)
