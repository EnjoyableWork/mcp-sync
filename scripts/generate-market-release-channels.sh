#!/usr/bin/env bash

set -euo pipefail

if [[ $# -ne 3 ]]; then
  echo "usage: $0 <cargo package> <stable version> <output directory>" >&2
  exit 2
fi

market_cargo_package=$1
market_version=$2
market_output_directory=$3

if [[ ! "$market_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "market version must be a stable semantic version" >&2
  exit 1
fi
if [[ ! -f "$market_cargo_package" || -L "$market_cargo_package" ]]; then
  echo "Cargo package must be a regular, non-symbolic-link file" >&2
  exit 1
fi
if [[ -e "$market_output_directory" ]]; then
  echo "market channel output directory already exists" >&2
  exit 1
fi

market_package_name="enjoyable-mcp-sync-$market_version.crate"
if [[ "$(basename -- "$market_cargo_package")" != "$market_package_name" ]]; then
  echo "Cargo package filename does not match the market version" >&2
  exit 1
fi

market_cargo_package="$(cd -- "$(dirname -- "$market_cargo_package")" && pwd)/$market_package_name"
market_output_parent=$(dirname -- "$market_output_directory")
market_output_name=$(basename -- "$market_output_directory")
mkdir -p -- "$market_output_parent"
market_output_parent=$(cd -- "$market_output_parent" && pwd)
market_output_directory="$market_output_parent/$market_output_name"
market_stage_prefix="$market_output_parent/.mcp-sync-market-channels."
market_stage=$(mktemp -d "${market_stage_prefix}XXXXXX")

cleanup_market_stage() {
  if [[ -z "${market_stage:-}" ]]; then
    return
  fi
  if [[ "$market_stage" != "$market_stage_prefix"* ]]; then
    echo "refusing to remove an unexpected market-channel staging path" >&2
    return 1
  fi
  if [[ -d "$market_stage" ]]; then
    rm -rf -- "$market_stage"
  fi
}
trap cleanup_market_stage EXIT

market_package_root="enjoyable-mcp-sync-$market_version"
market_entry_count=0
while IFS= read -r market_entry; do
  market_entry_count=$((market_entry_count + 1))
  if [[ "$market_entry" != "$market_package_root/"* ]]; then
    echo "Cargo package contains a path outside its versioned root" >&2
    exit 1
  fi
  market_relative_entry=${market_entry#"$market_package_root/"}
  if [[ "/$market_relative_entry/" == *"/../"* ]]; then
    echo "Cargo package contains a parent-directory traversal" >&2
    exit 1
  fi
done < <(tar -tzf "$market_cargo_package")
if [[ "$market_entry_count" -eq 0 ]]; then
  echo "Cargo package is empty" >&2
  exit 1
fi

market_inspection_root="$market_stage/inspection"
mkdir -p -- "$market_inspection_root"
tar -xzf "$market_cargo_package" -C "$market_inspection_root"
market_manifest="$market_inspection_root/$market_package_root/Cargo.toml"
market_lockfile="$market_inspection_root/$market_package_root/Cargo.lock"
if [[ ! -f "$market_manifest" || ! -f "$market_lockfile" ]]; then
  echo "Cargo package does not contain its manifest and lockfile" >&2
  exit 1
fi

market_metadata="$market_stage/metadata.json"
cargo metadata \
  --locked \
  --offline \
  --no-deps \
  --format-version 1 \
  --manifest-path "$market_manifest" >"$market_metadata"
jq -e \
  --arg version "$market_version" \
  '.packages | length == 1 and
   .[0].name == "enjoyable-mcp-sync" and
   .[0].version == $version and
   .[0].license == "MIT" and
   .[0].repository == "https://github.com/EnjoyableWork/mcp-sync" and
   any(.[0].targets[]; .name == "mcp-sync" and any(.kind[]; . == "bin"))' \
  "$market_metadata" >/dev/null

market_sha256() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{ print tolower($1) }'
  else
    shasum -a 256 "$1" | awk '{ print tolower($1) }'
  fi
}

market_package_hash=$(market_sha256 "$market_cargo_package")
if [[ ! "$market_package_hash" =~ ^[[:xdigit:]]{64}$ ]]; then
  echo "Cargo package SHA-256 could not be determined" >&2
  exit 1
fi

market_cargo_directory="$market_stage/cargo"
market_formula_directory="$market_stage/homebrew/Formula"
mkdir -p -- "$market_cargo_directory" "$market_formula_directory"
install -m 0644 "$market_cargo_package" "$market_cargo_directory/$market_package_name"

cat >"$market_formula_directory/mcp-sync.rb" <<EOF
# typed: false
# frozen_string_literal: true

class McpSync < Formula
  desc "Keep MCP server configuration synchronized across supported clients"
  homepage "https://github.com/EnjoyableWork/mcp-sync"
  url "https://github.com/EnjoyableWork/mcp-sync/releases/download/v$market_version/$market_package_name"
  sha256 "$market_package_hash"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: ".")
  end

  test do
    assert_equal "mcp-sync #{version}", shell_output("#{bin}/mcp-sync --version").strip
  end
end
EOF

rm -rf -- "$market_inspection_root"
rm -f -- "$market_metadata"
mv -- "$market_stage" "$market_output_directory"
market_stage=

printf '%s\n' "$market_output_directory"
