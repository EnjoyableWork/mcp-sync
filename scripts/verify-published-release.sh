#!/usr/bin/env bash

set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "usage: $0 <published asset directory> <stable version>" >&2
  exit 2
fi

asset_directory=$1
asset_version=$2

if [[ ! "$asset_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "asset version must be a stable semantic version" >&2
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

expected_assets=(SHA256SUMS "$asset_cargo_package")
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
  echo "published release asset set is incomplete or contains unexpected files" >&2
  exit 1
fi

manifest_entries=$(sed -n 's/^[[:xdigit:]]\{64\}  \([^/][^/]*\)$/\1/p' "$asset_directory/SHA256SUMS")
expected_manifest_entries=$(
  printf '%s\n' "${expected_assets[@]}" \
    | sed '/^SHA256SUMS$/d' \
    | LC_ALL=C sort
)
manifest_line_count=$(wc -l < "$asset_directory/SHA256SUMS" | tr -d '[:space:]')
expected_manifest_line_count=$(printf '%s\n' "$expected_manifest_entries" | wc -l | tr -d '[:space:]')
if [[ "$manifest_line_count" != "$expected_manifest_line_count" ]] ||
  [[ "$(printf '%s\n' "$manifest_entries" | LC_ALL=C sort)" != "$expected_manifest_entries" ]]; then
  echo "SHA256SUMS does not contain the exact release payload" >&2
  exit 1
fi

(
  cd "$asset_directory"
  sha256sum --check SHA256SUMS >/dev/null
)

for asset_target in "${asset_targets[@]}"; do
  jq -e '
    .spdxVersion == "SPDX-2.3" and
    (.documentNamespace | type == "string") and
    (.packages | type == "array" and length > 0)
  ' "$asset_directory/$asset_prefix-$asset_target.spdx.json" >/dev/null
done
