#!/usr/bin/env bash

set -euo pipefail

published_market_script_directory="$(
  cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
  pwd
)"

if [[ $# -ne 2 ]]; then
  echo "usage: $0 <published asset directory> <stable version>" >&2
  exit 2
fi

published_market_directory=$1
published_market_version=$2

if [[ ! "$published_market_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "published market version must be a stable semantic version" >&2
  exit 1
fi
if [[ ! -d "$published_market_directory" || -L "$published_market_directory" ]]; then
  echo "published market asset directory is missing or symbolic" >&2
  exit 1
fi

published_market_directory=$(cd -- "$published_market_directory" && pwd)
published_market_prefix="mcp-sync-v$published_market_version"
published_market_payload=(
  "enjoyable-mcp-sync-$published_market_version.crate"
  mcp-sync.rb
  "$published_market_prefix-aarch64-unknown-linux-gnu.tar.gz"
  "$published_market_prefix-aarch64-unknown-linux-gnu.spdx.json"
  "$published_market_prefix-x86_64-unknown-linux-gnu.tar.gz"
  "$published_market_prefix-x86_64-unknown-linux-gnu.spdx.json"
)
published_market_expected=(SHA256SUMS "${published_market_payload[@]}")

published_market_observed=$(
  find "$published_market_directory" -maxdepth 1 -type f -print \
    | sed 's#^.*/##' \
    | LC_ALL=C sort
)
published_market_sorted_expected=$(printf '%s\n' "${published_market_expected[@]}" | LC_ALL=C sort)
if [[ "$published_market_observed" != "$published_market_sorted_expected" ]]; then
  echo "published market release is incomplete or contains unexpected files" >&2
  exit 1
fi

published_market_entries=$(
  sed -n 's/^[[:xdigit:]]\{64\}  \([^/][^/]*\)$/\1/p' \
    "$published_market_directory/SHA256SUMS"
)
published_market_expected_entries=$(printf '%s\n' "${published_market_payload[@]}" | LC_ALL=C sort)
published_market_line_count=$(wc -l <"$published_market_directory/SHA256SUMS" | tr -d '[:space:]')
published_market_expected_count=$(printf '%s\n' "$published_market_expected_entries" | wc -l | tr -d '[:space:]')
if [[ "$published_market_line_count" != "$published_market_expected_count" ]] ||
  [[ "$(printf '%s\n' "$published_market_entries" | LC_ALL=C sort)" != "$published_market_expected_entries" ]]; then
  echo "SHA256SUMS does not contain the exact market-release payload" >&2
  exit 1
fi

(
  cd -- "$published_market_directory"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum --check SHA256SUMS >/dev/null
  else
    shasum -a 256 --check SHA256SUMS >/dev/null
  fi
)

published_market_temp_parent=${TMPDIR:-/tmp}
published_market_temp_prefix="${published_market_temp_parent%/}/mcp-sync-published-market."
published_market_temp=$(mktemp -d "${published_market_temp_prefix}XXXXXX")
cleanup_published_market() {
  if [[ "$published_market_temp" != "$published_market_temp_prefix"* ]]; then
    echo "refusing to remove an unexpected published-market path" >&2
    return 1
  fi
  if [[ -d "$published_market_temp" ]]; then
    rm -rf -- "$published_market_temp"
  fi
}
trap cleanup_published_market EXIT

for published_market_asset in "${published_market_payload[@]}"; do
  cp -- "$published_market_directory/$published_market_asset" "$published_market_temp/"
done
"$published_market_script_directory/verify-market-release-assets.sh" \
  "$published_market_temp" \
  "$published_market_version"
cmp --silent \
  "$published_market_directory/SHA256SUMS" \
  "$published_market_temp/SHA256SUMS"
