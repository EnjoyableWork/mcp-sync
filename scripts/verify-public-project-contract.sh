#!/usr/bin/env bash

set -euo pipefail

public_contract_repository=${1:-}
public_contract_version=${2:-}
public_contract_tap_repository=EnjoyableWork/homebrew-tap

if [[ "$public_contract_repository" != EnjoyableWork/mcp-sync ]] ||
  [[ "$public_contract_version" != 0.1.0 ]] || [[ $# -ne 2 ]]; then
  echo "usage: $0 EnjoyableWork/mcp-sync 0.1.0" >&2
  exit 2
fi

for public_contract_command in curl jq tar cmp grep; do
  if ! command -v "$public_contract_command" >/dev/null 2>&1; then
    echo "required verification command is unavailable: $public_contract_command" >&2
    exit 1
  fi
done

public_contract_script_directory="$(
  cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
  pwd
)"
public_contract_repository_root="$(
  cd -- "$public_contract_script_directory/.."
  pwd
)"

public_contract_api_headers=(
  --header 'Accept: application/vnd.github+json'
  --header 'User-Agent: mcp-sync-public-project-contract/0.1'
  --header 'X-GitHub-Api-Version: 2026-03-10'
)

public_contract_get_json() {
  curl --fail --silent --show-error --location \
    "${public_contract_api_headers[@]}" \
    "$1"
}

public_contract_community="$({
  public_contract_get_json \
    "https://api.github.com/repos/$public_contract_repository/community/profile"
})"

jq -e '
    .health_percentage >= 87 and
    .files.readme != null and
    .files.license.name == "MIT License" and
    .files.code_of_conduct_file != null and
    .files.contributing != null and
    .files.pull_request_template != null
  ' <<<"$public_contract_community" >/dev/null

# GitHub's community-profile REST API reports 87% and leaves
# files.issue_template null for this repository even though the signed-in
# Community Standards UI marks the valid YAML issue forms as Added. Require the
# other REST-visible files and the observed lower bound above, then verify the
# exact default-branch form inventory directly instead of treating that API
# field as a missing-template signal.
public_contract_issue_templates="$({
  public_contract_get_json \
    "https://api.github.com/repos/$public_contract_repository/contents/.github/ISSUE_TEMPLATE"
})"
jq -e '
    ([.[].name] | sort) ==
      (["01-bug-report.yml", "02-feature-request.yml", "03-conduct-contact.yml", "config.yml"] | sort) and
    all(.[];
      .type == "file" and
      (.download_url | startswith("https://")))
  ' <<<"$public_contract_issue_templates" >/dev/null

public_contract_support="$({
  public_contract_get_json \
    "https://api.github.com/repos/$public_contract_repository/contents/SUPPORT.md"
})"
jq -e '
    .name == "SUPPORT.md" and
    .type == "file" and
    (.download_url | startswith("https://"))
  ' <<<"$public_contract_support" >/dev/null

public_contract_security="$({
  public_contract_get_json \
    "https://api.github.com/repos/$public_contract_repository/contents/SECURITY.md"
})"
jq -e '
    .name == "SECURITY.md" and
    .type == "file" and
    (.download_url | startswith("https://"))
  ' <<<"$public_contract_security" >/dev/null

for public_contract_public_repository in \
  "$public_contract_repository" \
  "$public_contract_tap_repository"; do
  public_contract_repository_metadata="$({
    public_contract_get_json \
      "https://api.github.com/repos/$public_contract_public_repository"
  })"
  jq -e \
    --arg expected_name "$public_contract_public_repository" \
    '.full_name == $expected_name and
     .visibility == "public" and
     .archived == false and
     .default_branch == "main"' \
    <<<"$public_contract_repository_metadata" >/dev/null
done

public_contract_license_metadata="$({
  public_contract_get_json \
    "https://api.github.com/repos/$public_contract_repository/license"
})"
public_contract_tap_license_metadata="$({
  public_contract_get_json \
    "https://api.github.com/repos/$public_contract_tap_repository/license"
})"

for public_contract_license_metadata_document in \
  "$public_contract_license_metadata" \
  "$public_contract_tap_license_metadata"; do
  jq -e '
      .name == "LICENSE" and
      .license.spdx_id == "MIT" and
      (.download_url | startswith("https://"))
    ' <<<"$public_contract_license_metadata_document" >/dev/null
done

public_contract_local_uri_files=(
  "$public_contract_repository_root/README.md"
  "$public_contract_repository_root/SECURITY.md"
  "$public_contract_repository_root/CONTRIBUTING.md"
  "$public_contract_repository_root/CODE_OF_CONDUCT.md"
  "$public_contract_repository_root/SUPPORT.md"
  "$public_contract_repository_root/PROJECT.md"
  "$public_contract_repository_root/Cargo.toml"
  "$public_contract_repository_root/docs/"*.md
  "$public_contract_repository_root/docs/releases/"*.md
  "$public_contract_repository_root/.github/ISSUE_TEMPLATE/"*.yml
  "$public_contract_repository_root/.github/pull_request_template.md"
)
if grep -En '(http://|git://|ftp://|ssh://|git@)' \
  "${public_contract_local_uri_files[@]}" >/dev/null; then
  echo "an official project channel file contains a non-HTTPS transport" >&2
  exit 1
fi

grep -Fx 'license = "MIT"' "$public_contract_repository_root/Cargo.toml" >/dev/null

public_contract_temp_parent=${TMPDIR:-/tmp}
public_contract_temp_prefix="${public_contract_temp_parent%/}/mcp-sync-public-contract."
public_contract_temp="$(mktemp -d "${public_contract_temp_prefix}XXXXXX")"
cleanup_public_contract() {
  if [[ "$public_contract_temp" != "$public_contract_temp_prefix"* ]]; then
    echo "refusing to remove an unexpected public-contract path" >&2
    return 1
  fi
  if [[ -d "$public_contract_temp" ]]; then
    rm -rf -- "$public_contract_temp"
  fi
}
trap cleanup_public_contract EXIT

public_contract_release="$({
  public_contract_get_json \
    "https://api.github.com/repos/$public_contract_repository/releases/tags/v$public_contract_version"
})"
public_contract_release_prefix="mcp-sync-v$public_contract_version"
public_contract_expected_assets=(
  "enjoyable-mcp-sync-$public_contract_version.crate"
  mcp-sync.rb
  "$public_contract_release_prefix-aarch64-unknown-linux-gnu.tar.gz"
  "$public_contract_release_prefix-aarch64-unknown-linux-gnu.spdx.json"
  "$public_contract_release_prefix-x86_64-unknown-linux-gnu.tar.gz"
  "$public_contract_release_prefix-x86_64-unknown-linux-gnu.spdx.json"
  SHA256SUMS
)

jq -e \
  --arg tag "v$public_contract_version" \
  --argjson expected_count "${#public_contract_expected_assets[@]}" \
  '.tag_name == $tag and .draft == false and .immutable == true and
   (.assets | length) == $expected_count and
   all(.assets[]; .browser_download_url | startswith("https://"))' \
  <<<"$public_contract_release" >/dev/null

for public_contract_asset in "${public_contract_expected_assets[@]}"; do
  public_contract_asset_count="$(
    jq --arg name "$public_contract_asset" \
      '[.assets[] | select(.name == $name)] | length' \
      <<<"$public_contract_release"
  )"
  if [[ "$public_contract_asset_count" != 1 ]]; then
    echo "published release asset set does not match the licensing contract" >&2
    exit 1
  fi
  public_contract_asset_url="$(
    jq -r --arg name "$public_contract_asset" \
      '.assets[] | select(.name == $name) | .browser_download_url' \
      <<<"$public_contract_release"
  )"
  curl --fail --silent --show-error --location \
    --header 'User-Agent: mcp-sync-public-project-contract/0.1' \
    --output "$public_contract_temp/$public_contract_asset" \
    "$public_contract_asset_url"
done

"$public_contract_script_directory/verify-published-source-linux-release.sh" \
  "$public_contract_temp" \
  "$public_contract_version"

public_contract_tag_license="$public_contract_temp/tag-LICENSE"
curl --fail --silent --show-error --location \
  --header 'User-Agent: mcp-sync-public-project-contract/0.1' \
  --output "$public_contract_tag_license" \
  "https://raw.githubusercontent.com/$public_contract_repository/v$public_contract_version/LICENSE"
cmp --silent "$public_contract_repository_root/LICENSE" "$public_contract_tag_license"

public_contract_tap_license="$public_contract_temp/tap-LICENSE"
curl --fail --silent --show-error --location \
  --header 'User-Agent: mcp-sync-public-project-contract/0.1' \
  --output "$public_contract_tap_license" \
  "https://raw.githubusercontent.com/$public_contract_tap_repository/main/LICENSE"
cmp --silent "$public_contract_tag_license" "$public_contract_tap_license"

public_contract_archives=(
  "$public_contract_temp/enjoyable-mcp-sync-$public_contract_version.crate"
  "$public_contract_temp/$public_contract_release_prefix-aarch64-unknown-linux-gnu.tar.gz"
  "$public_contract_temp/$public_contract_release_prefix-x86_64-unknown-linux-gnu.tar.gz"
)
for public_contract_archive in "${public_contract_archives[@]}"; do
  public_contract_license_members="$(
    tar -tzf "$public_contract_archive" | grep -E '(^|/)LICENSE$' || true
  )"
  if [[ "$(wc -l <<<"$public_contract_license_members" | tr -d '[:space:]')" != 1 ]] ||
    [[ -z "$public_contract_license_members" ]]; then
    echo "a published software archive does not contain exactly one LICENSE" >&2
    exit 1
  fi
  tar -xOzf "$public_contract_archive" "$public_contract_license_members" \
    | cmp --silent - "$public_contract_tag_license"
done

public_contract_crates_package="$public_contract_temp/crates-io.crate"
curl --fail --silent --show-error --location \
  --header 'User-Agent: mcp-sync-public-project-contract/0.1' \
  --output "$public_contract_crates_package" \
  "https://crates.io/api/v1/crates/enjoyable-mcp-sync/$public_contract_version/download"
cmp --silent \
  "$public_contract_temp/enjoyable-mcp-sync-$public_contract_version.crate" \
  "$public_contract_crates_package"

public_contract_tap_formula="$public_contract_temp/tap-mcp-sync.rb"
curl --fail --silent --show-error --location \
  --header 'User-Agent: mcp-sync-public-project-contract/0.1' \
  --output "$public_contract_tap_formula" \
  "https://raw.githubusercontent.com/$public_contract_tap_repository/main/Formula/mcp-sync.rb"
cmp --silent "$public_contract_temp/mcp-sync.rb" "$public_contract_tap_formula"
grep -Fx '  license "MIT"' "$public_contract_tap_formula" >/dev/null

echo "Verified the public community, repository, URI, and licensing contract for $public_contract_repository v$public_contract_version."
