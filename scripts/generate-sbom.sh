#!/usr/bin/env bash

set -euo pipefail

sbom_script_directory="$(
  cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
  pwd
)"
sbom_asset_manifest=${MCP_SYNC_SYFT_ASSET_MANIFEST:-$sbom_script_directory/syft-assets.txt}
sbom_syft_version=1.50.0

if [[ $# -ne 2 ]]; then
  echo "usage: $0 <release archive> <SPDX JSON output>" >&2
  exit 2
fi

sbom_input=$1
sbom_output=$2

if [[ ! -f "$sbom_input" ]]; then
  echo "SBOM input archive is missing" >&2
  exit 1
fi
if [[ ! -f "$sbom_asset_manifest" ]]; then
  echo "Syft asset manifest is missing" >&2
  exit 1
fi

sbom_host_os=${MCP_SYNC_SYFT_HOST_OS:-$(uname -s)}
sbom_host_architecture=${MCP_SYNC_SYFT_HOST_ARCHITECTURE:-$(uname -m)}
case "$sbom_host_os" in
  Darwin | darwin) sbom_host_os=darwin ;;
  Linux | linux) sbom_host_os=linux ;;
  *)
    echo "unsupported Syft host operating system" >&2
    exit 1
    ;;
esac
case "$sbom_host_architecture" in
  arm64 | aarch64) sbom_host_architecture=arm64 ;;
  x86_64 | amd64) sbom_host_architecture=x86_64 ;;
  *)
    echo "unsupported Syft host architecture" >&2
    exit 1
    ;;
esac

sbom_manifest_record=$(
  awk -v host_os="$sbom_host_os" -v host_architecture="$sbom_host_architecture" '
    $1 == host_os && $2 == host_architecture {
      print $3 " " $4
      matches += 1
    }
    END {
      if (matches != 1) {
        exit 1
      }
    }
  ' "$sbom_asset_manifest"
) || {
  echo "Syft asset manifest does not contain exactly one host mapping" >&2
  exit 1
}
read -r sbom_asset_name sbom_expected_sha256 <<<"$sbom_manifest_record"

if [[ ! "$sbom_asset_name" =~ ^syft_${sbom_syft_version}_(darwin|linux)_(amd64|arm64)\.tar\.gz$ ]] ||
  [[ ! "$sbom_expected_sha256" =~ ^[0-9a-f]{64}$ ]]; then
  echo "Syft asset manifest contains an invalid Unix record" >&2
  exit 1
fi

sbom_output_directory=$(dirname -- "$sbom_output")
mkdir -p -- "$sbom_output_directory"
sbom_output_directory=$(cd -- "$sbom_output_directory" && pwd)
sbom_output="$sbom_output_directory/$(basename -- "$sbom_output")"
sbom_input=$(cd -- "$(dirname -- "$sbom_input")" && pwd)/$(basename -- "$sbom_input")

sbom_temp_parent=${RUNNER_TEMP:-${TMPDIR:-/tmp}}
sbom_stage_prefix="${sbom_temp_parent%/}/mcp-sync-syft."
sbom_stage=$(mktemp -d "${sbom_stage_prefix}XXXXXX")
sbom_download="$sbom_stage/$sbom_asset_name"
sbom_download_partial="$sbom_stage/$sbom_asset_name.partial"
sbom_output_temp=$(mktemp "$sbom_output_directory/.mcp-sync-sbom.XXXXXX")

cleanup_sbom_stage() {
  if [[ "$sbom_stage" != "$sbom_stage_prefix"* ]]; then
    echo "refusing to remove an unexpected Syft staging path" >&2
    return 1
  fi

  if [[ -d "$sbom_stage" ]]; then
    rm -rf -- "$sbom_stage"
  fi
  if [[ -f "$sbom_output_temp" ]]; then
    rm -f -- "$sbom_output_temp"
  fi
}
trap cleanup_sbom_stage EXIT

sbom_download_base=${MCP_SYNC_SYFT_DOWNLOAD_BASE_URL:-https://github.com/anchore/syft/releases/download/v${sbom_syft_version}}
sbom_download_url="${sbom_download_base%/}/$sbom_asset_name"
sbom_curl=${MCP_SYNC_SYFT_CURL:-curl}
sbom_sleep=${MCP_SYNC_SYFT_SLEEP:-sleep}
sbom_maximum_attempts=5

for ((sbom_attempt = 1; sbom_attempt <= sbom_maximum_attempts; sbom_attempt += 1)); do
  set +e
  sbom_http_status=$(
    "$sbom_curl" \
      --fail \
      --location \
      --silent \
      --show-error \
      --proto '=https' \
      --proto-redir '=https' \
      --tlsv1.2 \
      --connect-timeout 15 \
      --max-time 120 \
      --write-out '%{http_code}' \
      --output "$sbom_download_partial" \
      "$sbom_download_url"
  )
  sbom_curl_status=$?
  set -e

  if [[ "$sbom_curl_status" -eq 0 && "$sbom_http_status" =~ ^2[0-9]{2}$ ]]; then
    mv -f -- "$sbom_download_partial" "$sbom_download"
    break
  fi

  sbom_transient=false
  case "$sbom_http_status" in
    408 | 429 | 500 | 502 | 503 | 504) sbom_transient=true ;;
  esac
  case "$sbom_curl_status" in
    5 | 6 | 7 | 16 | 18 | 28 | 35 | 52 | 55 | 56 | 92) sbom_transient=true ;;
  esac
  if [[ "$sbom_transient" != true || "$sbom_attempt" -eq "$sbom_maximum_attempts" ]]; then
    echo "Syft asset acquisition failed" >&2
    exit 1
  fi

  sbom_delay=$((1 << (sbom_attempt - 1)))
  echo "transient Syft download failure; retrying attempt $((sbom_attempt + 1)) of $sbom_maximum_attempts" >&2
  "$sbom_sleep" "$sbom_delay"
done

if command -v sha256sum >/dev/null 2>&1; then
  sbom_actual_sha256=$(sha256sum "$sbom_download" | awk '{print $1}')
else
  sbom_actual_sha256=$(shasum -a 256 "$sbom_download" | awk '{print $1}')
fi
if [[ "$sbom_actual_sha256" != "$sbom_expected_sha256" ]]; then
  echo "downloaded Syft asset failed SHA-256 verification" >&2
  exit 1
fi

tar -xzf "$sbom_download" -C "$sbom_stage" syft
sbom_syft="$sbom_stage/syft"
if [[ ! -x "$sbom_syft" ]]; then
  echo "verified Syft archive did not contain an executable" >&2
  exit 1
fi
if ! "$sbom_syft" version | awk -v version="$sbom_syft_version" '
  $1 == "Version:" && $2 == version { matches += 1 }
  END { exit matches == 1 ? 0 : 1 }
'; then
  echo "Syft executable version does not match the pinned release" >&2
  exit 1
fi

SYFT_CHECK_FOR_APP_UPDATE=false "$sbom_syft" scan \
  "file:$sbom_input" \
  --output "spdx-json=$sbom_output_temp"
if [[ ! -s "$sbom_output_temp" ]]; then
  echo "Syft did not produce an SPDX JSON document" >&2
  exit 1
fi
mv -f -- "$sbom_output_temp" "$sbom_output"
