#!/usr/bin/env bash

set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "usage: $0 <mcp-sync executable> <expected version>" >&2
  exit 2
fi

smoke_executable=$1
smoke_version=$2

if [[ ! -x "$smoke_executable" ]]; then
  echo "installed smoke executable is missing or not executable" >&2
  exit 1
fi

smoke_executable=$(cd "$(dirname "$smoke_executable")" && pwd)/$(basename "$smoke_executable")
smoke_temp_parent="${TMPDIR:-/tmp}"
smoke_root_prefix="${smoke_temp_parent%/}/mcp-sync-release-smoke."
smoke_root=$(mktemp -d "${smoke_root_prefix}XXXXXX")
smoke_home="$smoke_root/home"
smoke_xdg="$smoke_root/xdg"
smoke_local="$smoke_root/local"
smoke_roaming="$smoke_root/roaming"
smoke_path=$PATH

cleanup_smoke_root() {
  if [[ "$smoke_root" != "$smoke_root_prefix"* ]]; then
    echo "refusing to remove an unexpected installed-smoke path" >&2
    return 1
  fi

  if [[ -d "$smoke_root" ]]; then
    rm -rf -- "$smoke_root"
  fi
}
trap cleanup_smoke_root EXIT

mkdir -p "$smoke_home" "$smoke_xdg" "$smoke_local" "$smoke_roaming"

run_mcp_sync() {
  env -i \
    HOME="$smoke_home" \
    USERPROFILE="$smoke_home" \
    XDG_CONFIG_HOME="$smoke_xdg" \
    LOCALAPPDATA="$smoke_local" \
    APPDATA="$smoke_roaming" \
    PATH="$smoke_path" \
    "$smoke_executable" "$@"
}

version_output=$(run_mcp_sync --version)
if [[ "$version_output" != "mcp-sync $smoke_version" ]]; then
  echo "installed executable reported an unexpected version" >&2
  exit 1
fi

run_mcp_sync init >/dev/null
run_mcp_sync add release-smoke \
  --command release-smoke-server \
  --arg=--stdio \
  --env SMOKE_TOKEN=synthetic-release-value >/dev/null
run_mcp_sync sync --dry-run >/dev/null
run_mcp_sync sync >/dev/null
run_mcp_sync restore canonical --dry-run >/dev/null
run_mcp_sync restore canonical >/dev/null

restored_list=$(run_mcp_sync list)
if [[ "$restored_list" == *release-smoke* ]]; then
  echo "first restore did not recover the empty canonical generation" >&2
  exit 1
fi

run_mcp_sync restore canonical >/dev/null
current_list=$(run_mcp_sync list)
if [[ "$current_list" != *release-smoke* ]]; then
  echo "second restore did not recover the newer canonical generation" >&2
  exit 1
fi

run_mcp_sync sync --dry-run >/dev/null
