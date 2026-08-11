#!/usr/bin/env bash

set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "usage: $0 <asset directory> <stable version>" >&2
  exit 2
fi

asset_directory=$1
asset_version=$2

if [[ ! "$asset_version" =~ ^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$ ]]; then
  echo "asset version must be a canonical stable semantic version" >&2
  exit 1
fi
if [[ ! -d "$asset_directory" ]]; then
  echo "asset directory is missing" >&2
  exit 1
fi

asset_directory=$(cd "$asset_directory" && pwd)
asset_prefix="mcp-sync-v$asset_version"
asset_cargo_package="enjoyable-mcp-sync-$asset_version.crate"
asset_targets=(
  aarch64-apple-darwin
  x86_64-apple-darwin
  aarch64-unknown-linux-gnu
  x86_64-unknown-linux-gnu
  aarch64-pc-windows-msvc
  x86_64-pc-windows-msvc
)

expected_assets=("$asset_cargo_package")
for asset_target in "${asset_targets[@]}"; do
  case "$asset_target" in
    *-pc-windows-msvc) asset_archive="$asset_prefix-$asset_target.zip" ;;
    *) asset_archive="$asset_prefix-$asset_target.tar.gz" ;;
  esac
  expected_assets+=("$asset_archive" "$asset_prefix-$asset_target.spdx.json")
done

observed_assets=$(
  find "$asset_directory" -maxdepth 1 -type f -print \
    | sed 's#^.*/##' \
    | LC_ALL=C sort
)
sorted_expected_assets=$(printf '%s\n' "${expected_assets[@]}" | LC_ALL=C sort)

if [[ "$observed_assets" != "$sorted_expected_assets" ]]; then
  echo "release asset set is incomplete or contains unexpected files" >&2
  exit 1
fi

for asset_target in "${asset_targets[@]}"; do
  sbom_path="$asset_directory/$asset_prefix-$asset_target.spdx.json"
  jq -e '
    .spdxVersion == "SPDX-2.3" and
    (.documentNamespace | type == "string") and
    (.packages | type == "array" and length > 0)
  ' "$sbom_path" >/dev/null
done

(
  cd "$asset_directory"
  for expected_asset in "${expected_assets[@]}"; do
    sha256sum "$expected_asset"
  done | LC_ALL=C sort -k2 > SHA256SUMS
  sha256sum --check SHA256SUMS >/dev/null
)
