#!/usr/bin/env bash

set -euo pipefail

channel_script_directory="$(
  cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
  pwd
)"

if [[ $# -ne 3 ]]; then
  echo "usage: $0 <published asset directory> <stable version> <output directory>" >&2
  exit 2
fi

channel_asset_directory=$1
channel_version=$2
channel_output_directory=$3

if [[ ! "$channel_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "channel version must be a stable semantic version" >&2
  exit 1
fi
if [[ ! -d "$channel_asset_directory" ]]; then
  echo "published asset directory is missing" >&2
  exit 1
fi
if [[ -e "$channel_output_directory" ]]; then
  echo "channel output directory already exists" >&2
  exit 1
fi

channel_asset_directory=$(cd "$channel_asset_directory" && pwd)
channel_output_parent=$(dirname "$channel_output_directory")
channel_output_name=$(basename "$channel_output_directory")
mkdir -p -- "$channel_output_parent"
channel_output_parent=$(cd "$channel_output_parent" && pwd)
channel_output_directory="$channel_output_parent/$channel_output_name"
channel_stage_prefix="$channel_output_parent/.mcp-sync-release-channels."
channel_stage=$(mktemp -d "${channel_stage_prefix}XXXXXX")

cleanup_channel_stage() {
  if [[ -z "${channel_stage:-}" ]]; then
    return
  fi
  if [[ "$channel_stage" != "$channel_stage_prefix"* ]]; then
    echo "refusing to remove an unexpected channel staging path" >&2
    return 1
  fi
  if [[ -d "$channel_stage" ]]; then
    rm -rf -- "$channel_stage"
  fi
}
trap cleanup_channel_stage EXIT

"$channel_script_directory/verify-published-release.sh" \
  "$channel_asset_directory" \
  "$channel_version"

channel_manifest="$channel_asset_directory/SHA256SUMS"
channel_asset_prefix="mcp-sync-v$channel_version"

release_hash() {
  local asset_name=$1
  local -a matching_hashes=()

  while IFS= read -r asset_hash; do
    matching_hashes+=("$asset_hash")
  done < <(awk -v asset="$asset_name" '$2 == asset { print $1 }' "$channel_manifest")

  if [[ ${#matching_hashes[@]} -ne 1 ]] ||
    [[ ! "${matching_hashes[0]}" =~ ^[[:xdigit:]]{64}$ ]]; then
    echo "release hash is missing or ambiguous for $asset_name" >&2
    return 1
  fi

  printf '%s\n' "${matching_hashes[0]}"
}

macos_arm_archive="$channel_asset_prefix-aarch64-apple-darwin.tar.gz"
macos_x64_archive="$channel_asset_prefix-x86_64-apple-darwin.tar.gz"
linux_arm_archive="$channel_asset_prefix-aarch64-unknown-linux-gnu.tar.gz"
linux_x64_archive="$channel_asset_prefix-x86_64-unknown-linux-gnu.tar.gz"
windows_arm_archive="$channel_asset_prefix-aarch64-pc-windows-msvc.zip"
windows_x64_archive="$channel_asset_prefix-x86_64-pc-windows-msvc.zip"

macos_arm_hash=$(release_hash "$macos_arm_archive")
macos_x64_hash=$(release_hash "$macos_x64_archive")
linux_arm_hash=$(release_hash "$linux_arm_archive")
linux_x64_hash=$(release_hash "$linux_x64_archive")
windows_arm_hash=$(release_hash "$windows_arm_archive")
windows_x64_hash=$(release_hash "$windows_x64_archive")

formula_directory="$channel_stage/homebrew/Formula"
winget_directory="$channel_stage/winget/manifests/e/EnjoyableWork/mcp-sync/$channel_version"
mkdir -p -- "$formula_directory" "$winget_directory"

cat >"$formula_directory/mcp-sync.rb" <<EOF
# typed: false
# frozen_string_literal: true

class McpSync < Formula
  desc "Keep MCP server configuration synchronized across supported clients"
  homepage "https://github.com/EnjoyableWork/mcp-sync"
  version "$channel_version"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/EnjoyableWork/mcp-sync/releases/download/v$channel_version/$macos_arm_archive"
      sha256 "$macos_arm_hash"
    end
    on_intel do
      url "https://github.com/EnjoyableWork/mcp-sync/releases/download/v$channel_version/$macos_x64_archive"
      sha256 "$macos_x64_hash"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/EnjoyableWork/mcp-sync/releases/download/v$channel_version/$linux_arm_archive"
      sha256 "$linux_arm_hash"
    end
    on_intel do
      url "https://github.com/EnjoyableWork/mcp-sync/releases/download/v$channel_version/$linux_x64_archive"
      sha256 "$linux_x64_hash"
    end
  end

  def install
    bin.install "mcp-sync"
  end

  test do
    assert_equal "mcp-sync #{version}", shell_output("#{bin}/mcp-sync --version").strip
  end
end
EOF

cat >"$winget_directory/EnjoyableWork.mcp-sync.yaml" <<EOF
# yaml-language-server: \$schema=https://aka.ms/winget-manifest.version.1.12.0.schema.json

PackageIdentifier: EnjoyableWork.mcp-sync
PackageVersion: $channel_version
DefaultLocale: en-US
ManifestType: version
ManifestVersion: 1.12.0
EOF

cat >"$winget_directory/EnjoyableWork.mcp-sync.locale.en-US.yaml" <<EOF
# yaml-language-server: \$schema=https://aka.ms/winget-manifest.defaultLocale.1.12.0.schema.json

PackageIdentifier: EnjoyableWork.mcp-sync
PackageVersion: $channel_version
PackageLocale: en-US
Publisher: EnjoyableWork
PublisherUrl: https://github.com/EnjoyableWork
PublisherSupportUrl: https://github.com/EnjoyableWork/mcp-sync/issues
PackageName: mcp-sync
PackageUrl: https://github.com/EnjoyableWork/mcp-sync
License: MIT
LicenseUrl: https://github.com/EnjoyableWork/mcp-sync/blob/v$channel_version/LICENSE
ShortDescription: Keep MCP server configuration synchronized across supported clients.
Moniker: mcp-sync
Tags:
  - cli
  - developer-tools
  - mcp
  - model-context-protocol
ReleaseNotesUrl: https://github.com/EnjoyableWork/mcp-sync/releases/tag/v$channel_version
ManifestType: defaultLocale
ManifestVersion: 1.12.0
EOF

cat >"$winget_directory/EnjoyableWork.mcp-sync.installer.yaml" <<EOF
# yaml-language-server: \$schema=https://aka.ms/winget-manifest.installer.1.12.0.schema.json

PackageIdentifier: EnjoyableWork.mcp-sync
PackageVersion: $channel_version
InstallerType: zip
NestedInstallerType: portable
Scope: user
UpgradeBehavior: install
Commands:
  - mcp-sync
Installers:
  - Architecture: arm64
    InstallerUrl: https://github.com/EnjoyableWork/mcp-sync/releases/download/v$channel_version/$windows_arm_archive
    InstallerSha256: $windows_arm_hash
    NestedInstallerFiles:
      - RelativeFilePath: mcp-sync.exe
        PortableCommandAlias: mcp-sync
  - Architecture: x64
    InstallerUrl: https://github.com/EnjoyableWork/mcp-sync/releases/download/v$channel_version/$windows_x64_archive
    InstallerSha256: $windows_x64_hash
    NestedInstallerFiles:
      - RelativeFilePath: mcp-sync.exe
        PortableCommandAlias: mcp-sync
ManifestType: installer
ManifestVersion: 1.12.0
EOF

mv -- "$channel_stage" "$channel_output_directory"
channel_stage=

printf '%s\n' "$channel_output_directory"
