#!/usr/bin/env bash

set -euo pipefail

release_script_directory="$(
  cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
  pwd
)"
release_repository_root="$(
  cd -- "$release_script_directory/.."
  pwd
)"

if [[ $# -ne 4 ]]; then
  echo "usage: $0 <version> <Rust target> <mcp-sync executable> <output directory>" >&2
  exit 2
fi

release_version=$1
release_target=$2
release_executable=$3
release_output_directory=$4

if [[ ! "$release_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "release version must be a stable semantic version" >&2
  exit 1
fi

case "$release_target" in
  aarch64-apple-darwin | x86_64-apple-darwin | aarch64-unknown-linux-gnu | x86_64-unknown-linux-gnu) ;;
  *)
    echo "unsupported Unix release target" >&2
    exit 1
    ;;
esac

if [[ ! -x "$release_executable" ]]; then
  echo "release executable is missing or not executable" >&2
  exit 1
fi

release_reported_version=$("$release_executable" --version)
if [[ "$release_reported_version" != "mcp-sync $release_version" ]]; then
  echo "release executable version does not match the requested archive version" >&2
  exit 1
fi

mkdir -p -- "$release_output_directory"
release_output_directory=$(cd "$release_output_directory" && pwd)
release_temp_parent="${TMPDIR:-/tmp}"
release_stage_prefix="${release_temp_parent%/}/mcp-sync-release-package."
release_stage=$(mktemp -d "${release_stage_prefix}XXXXXX")
release_archive="mcp-sync-v${release_version}-${release_target}.tar.gz"
release_archive_path="$release_output_directory/$release_archive"
release_archive_temp=$(mktemp "$release_output_directory/.${release_archive}.XXXXXX")

cleanup_release_stage() {
  if [[ "$release_stage" != "$release_stage_prefix"* ]]; then
    echo "refusing to remove an unexpected release staging path" >&2
    return 1
  fi

  if [[ -d "$release_stage" ]]; then
    rm -rf -- "$release_stage"
  fi
  if [[ -f "$release_archive_temp" ]]; then
    rm -f -- "$release_archive_temp"
  fi
}
trap cleanup_release_stage EXIT

install -m 0755 "$release_executable" "$release_stage/mcp-sync"
install -m 0644 \
  "$release_repository_root/LICENSE" \
  "$release_repository_root/README.md" \
  "$release_repository_root/Cargo.lock" \
  "$release_stage/"

COPYFILE_DISABLE=1 tar -C "$release_stage" -czf "$release_archive_temp" \
  mcp-sync LICENSE README.md Cargo.lock
mv -f -- "$release_archive_temp" "$release_archive_path"

printf '%s\n' "$release_archive_path"
