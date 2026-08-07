#!/usr/bin/env bash

set -euo pipefail

mcp_sync_script_directory="$(
  cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
  pwd
)"
mcp_sync_repository_root="$(
  cd -- "${mcp_sync_script_directory}/.."
  pwd
)"
mcp_sync_caller_user_root="${HOME:?HOME must identify the caller Rust toolchain root}"
mcp_sync_cargo_home="${CARGO_HOME:-${mcp_sync_caller_user_root}/.cargo}"
mcp_sync_rustup_home="${RUSTUP_HOME:-${mcp_sync_caller_user_root}/.rustup}"
mcp_sync_tool_path="${PATH:?PATH must locate the Rust toolchain}"
mcp_sync_temp_parent="${TMPDIR:-/tmp}"
mcp_sync_synthetic_root="$(
  mktemp -d "${mcp_sync_temp_parent%/}/mcp-sync-quality.XXXXXX"
)"
mcp_sync_synthetic_prefix="${mcp_sync_temp_parent%/}/mcp-sync-quality."
mcp_sync_synthetic_user_root="${mcp_sync_synthetic_root}/user"

mcp_sync_cleanup() {
  if [[ "${mcp_sync_synthetic_root}" != "${mcp_sync_synthetic_prefix}"* ]]; then
    printf 'Refusing to remove unexpected quality-gate path: %s\n' \
      "${mcp_sync_synthetic_root}" >&2
    return 1
  fi

  if [[ -d "${mcp_sync_synthetic_root}" ]]; then
    rm -rf -- "${mcp_sync_synthetic_root}"
  fi
}

trap mcp_sync_cleanup EXIT

mkdir -p -- \
  "${mcp_sync_synthetic_user_root}/.cache" \
  "${mcp_sync_synthetic_user_root}/.config" \
  "${mcp_sync_synthetic_user_root}/.local/share" \
  "${mcp_sync_synthetic_user_root}/.local/state" \
  "${mcp_sync_synthetic_user_root}/AppData/Local" \
  "${mcp_sync_synthetic_user_root}/AppData/Roaming" \
  "${mcp_sync_synthetic_user_root}/Library/Application Support" \
  "${mcp_sync_synthetic_root}/runtime" \
  "${mcp_sync_synthetic_root}/tmp" \
  "${mcp_sync_synthetic_root}/xdg-config-dirs"

mcp_sync_run_isolated() {
  env -i \
    APPDATA="${mcp_sync_synthetic_user_root}/AppData/Roaming" \
    CARGO_HOME="${mcp_sync_cargo_home}" \
    CARGO_INCREMENTAL=0 \
    CARGO_TERM_COLOR=never \
    CFFIXED_USER_HOME="${mcp_sync_synthetic_user_root}" \
    HOME="${mcp_sync_synthetic_user_root}" \
    LANG=C \
    LC_ALL=C \
    LOCALAPPDATA="${mcp_sync_synthetic_user_root}/AppData/Local" \
    MCP_SYNC_TEST_HOME="${mcp_sync_synthetic_user_root}" \
    MCP_SYNC_TEST_MODE=1 \
    MCP_SYNC_TEST_ROOT="${mcp_sync_synthetic_root}" \
    NO_COLOR=1 \
    PATH="${mcp_sync_tool_path}" \
    RUSTUP_HOME="${mcp_sync_rustup_home}" \
    TEMP="${mcp_sync_synthetic_root}/tmp" \
    TMP="${mcp_sync_synthetic_root}/tmp" \
    TMPDIR="${mcp_sync_synthetic_root}/tmp" \
    TZ=UTC \
    USERPROFILE="${mcp_sync_synthetic_user_root}" \
    XDG_CACHE_HOME="${mcp_sync_synthetic_user_root}/.cache" \
    XDG_CONFIG_DIRS="${mcp_sync_synthetic_root}/xdg-config-dirs" \
    XDG_CONFIG_HOME="${mcp_sync_synthetic_user_root}/.config" \
    XDG_DATA_HOME="${mcp_sync_synthetic_user_root}/.local/share" \
    XDG_RUNTIME_DIR="${mcp_sync_synthetic_root}/runtime" \
    XDG_STATE_HOME="${mcp_sync_synthetic_user_root}/.local/state" \
    "$@"
}

cd -- "${mcp_sync_repository_root}"

printf 'Running quality gates with disposable user configuration roots.\n'
mcp_sync_run_isolated cargo fmt --all -- --check
mcp_sync_run_isolated cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
mcp_sync_run_isolated cargo test --workspace --all-targets --all-features --locked
printf 'Formatting, Clippy, and tests passed through the synthetic home.\n'
