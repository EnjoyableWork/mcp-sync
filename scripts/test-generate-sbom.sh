#!/usr/bin/env bash

set -euo pipefail

sbom_test_script_directory="$(
  cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
  pwd
)"
sbom_test_repository_root="$(
  cd -- "$sbom_test_script_directory/.."
  pwd
)"
sbom_test_temp_parent=${TMPDIR:-/tmp}
sbom_test_root_prefix="${sbom_test_temp_parent%/}/mcp-sync-syft-test."
sbom_test_root=$(mktemp -d "${sbom_test_root_prefix}XXXXXX")
sbom_test_fixture_root="$sbom_test_root/fixture"
sbom_test_download_root="$sbom_test_root/download"
sbom_test_input="$sbom_test_root/input.zip"

cleanup_sbom_test_root() {
  if [[ "$sbom_test_root" != "$sbom_test_root_prefix"* ]]; then
    echo "refusing to remove an unexpected Syft test path" >&2
    return 1
  fi

  if [[ -d "$sbom_test_root" ]]; then
    rm -rf -- "$sbom_test_root"
  fi
}
trap cleanup_sbom_test_root EXIT

mkdir -p -- "$sbom_test_fixture_root" "$sbom_test_download_root"
printf 'synthetic release archive\n' >"$sbom_test_input"

cat >"$sbom_test_fixture_root/syft" <<'SYFT_FIXTURE'
#!/usr/bin/env bash
set -euo pipefail

case "${1:-}" in
  version)
    printf 'Version: 1.50.0\n'
    ;;
  scan)
    if [[ $# -ne 4 || "$2" != file:* || "$3" != --output || "$4" != spdx-json=* ]]; then
      echo "unexpected synthetic Syft invocation" >&2
      exit 1
    fi
    output=${4#spdx-json=}
    printf '{"spdxVersion":"SPDX-2.3","documentNamespace":"https://example.invalid/synthetic","packages":[{"name":"synthetic"}]}\n' >"$output"
    ;;
  *)
    echo "unexpected synthetic Syft command" >&2
    exit 1
    ;;
esac
SYFT_FIXTURE
chmod +x "$sbom_test_fixture_root/syft"
tar -C "$sbom_test_fixture_root" -czf \
  "$sbom_test_download_root/syft_1.50.0_darwin_arm64.tar.gz" \
  syft
if command -v sha256sum >/dev/null 2>&1; then
  sbom_test_asset_sha=$(sha256sum "$sbom_test_download_root/syft_1.50.0_darwin_arm64.tar.gz" | awk '{print $1}')
else
  sbom_test_asset_sha=$(shasum -a 256 "$sbom_test_download_root/syft_1.50.0_darwin_arm64.tar.gz" | awk '{print $1}')
fi

sbom_test_manifest="$sbom_test_root/syft-assets.txt"
awk -v replacement="$sbom_test_asset_sha" '
  $1 == "darwin" && $2 == "arm64" { $4 = replacement }
  { print }
' "$sbom_test_script_directory/syft-assets.txt" >"$sbom_test_manifest"

sbom_test_curl="$sbom_test_root/fake-curl"
cat >"$sbom_test_curl" <<'CURL_FIXTURE'
#!/usr/bin/env bash
set -euo pipefail

output=
url=
write_out=
while [[ $# -gt 0 ]]; do
  case "$1" in
    --output)
      output=$2
      shift 2
      ;;
    --write-out)
      write_out=$2
      shift 2
      ;;
    --*)
      if [[ "$1" == --proto || "$1" == --proto-redir || "$1" == --connect-timeout ||
        "$1" == --max-time ]]; then
        shift 2
      else
        shift
      fi
      ;;
    *)
      url=$1
      shift
      ;;
  esac
done
if [[ "$write_out" != '%{http_code}' ]]; then
  echo "synthetic curl did not receive the HTTP status contract" >&2
  exit 2
fi

attempt=1
if [[ -f "$MCP_SYNC_SYFT_CURL_ATTEMPTS" ]]; then
  attempt=$(($(<"$MCP_SYNC_SYFT_CURL_ATTEMPTS") + 1))
fi
printf '%s\n' "$attempt" >"$MCP_SYNC_SYFT_CURL_ATTEMPTS"
printf '%s\n' "$url" >"$MCP_SYNC_SYFT_CURL_LOG"
sequence_item=$(cut -d, -f"$attempt" <<<"$MCP_SYNC_SYFT_FAKE_SEQUENCE")
http_status=${sequence_item%:*}
exit_status=${sequence_item#*:}
printf '%s' "$http_status"
if [[ "$exit_status" -eq 0 ]]; then
  cp "$MCP_SYNC_SYFT_FAKE_ASSET" "$output"
fi
exit "$exit_status"
CURL_FIXTURE
chmod +x "$sbom_test_curl"

sbom_test_sleep="$sbom_test_root/fake-sleep"
cat >"$sbom_test_sleep" <<'SLEEP_FIXTURE'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$1" >>"$MCP_SYNC_SYFT_SLEEP_LOG"
SLEEP_FIXTURE
chmod +x "$sbom_test_sleep"

sbom_test_output="$sbom_test_root/output.spdx.json"
MCP_SYNC_SYFT_HOST_OS=darwin \
MCP_SYNC_SYFT_HOST_ARCHITECTURE=arm64 \
MCP_SYNC_SYFT_ASSET_MANIFEST="$sbom_test_manifest" \
MCP_SYNC_SYFT_DOWNLOAD_BASE_URL=https://example.invalid/syft \
MCP_SYNC_SYFT_CURL="$sbom_test_curl" \
MCP_SYNC_SYFT_CURL_LOG="$sbom_test_root/curl.log" \
MCP_SYNC_SYFT_CURL_ATTEMPTS="$sbom_test_root/curl.attempts" \
MCP_SYNC_SYFT_FAKE_ASSET="$sbom_test_download_root/syft_1.50.0_darwin_arm64.tar.gz" \
MCP_SYNC_SYFT_FAKE_SEQUENCE=503:22,000:56,200:0 \
MCP_SYNC_SYFT_SLEEP="$sbom_test_sleep" \
MCP_SYNC_SYFT_SLEEP_LOG="$sbom_test_root/sleep.log" \
  "$sbom_test_repository_root/scripts/generate-sbom.sh" \
  "$sbom_test_input" \
  "$sbom_test_output"

jq -e '
  .spdxVersion == "SPDX-2.3" and
  .documentNamespace == "https://example.invalid/synthetic" and
  (.packages | length == 1)
' "$sbom_test_output" >/dev/null
if [[ "$(<"$sbom_test_root/curl.log")" != \
  'https://example.invalid/syft/syft_1.50.0_darwin_arm64.tar.gz' ]]; then
  echo "SBOM generator selected an unexpected immutable asset URL" >&2
  exit 1
fi
if [[ "$(<"$sbom_test_root/curl.attempts")" != 3 ]] ||
  [[ "$(paste -sd, "$sbom_test_root/sleep.log")" != 1,2 ]]; then
  echo "SBOM generator did not apply the exact transient retry policy" >&2
  exit 1
fi

printf 'corrupt asset\n' >"$sbom_test_root/corrupt.tar.gz"
if MCP_SYNC_SYFT_HOST_OS=darwin \
  MCP_SYNC_SYFT_HOST_ARCHITECTURE=arm64 \
  MCP_SYNC_SYFT_ASSET_MANIFEST="$sbom_test_manifest" \
  MCP_SYNC_SYFT_DOWNLOAD_BASE_URL=https://example.invalid/syft \
  MCP_SYNC_SYFT_CURL="$sbom_test_curl" \
  MCP_SYNC_SYFT_CURL_LOG="$sbom_test_root/corrupt-curl.log" \
  MCP_SYNC_SYFT_CURL_ATTEMPTS="$sbom_test_root/corrupt-curl.attempts" \
  MCP_SYNC_SYFT_FAKE_ASSET="$sbom_test_root/corrupt.tar.gz" \
  MCP_SYNC_SYFT_FAKE_SEQUENCE=200:0 \
  MCP_SYNC_SYFT_SLEEP="$sbom_test_sleep" \
  MCP_SYNC_SYFT_SLEEP_LOG="$sbom_test_root/corrupt-sleep.log" \
  "$sbom_test_repository_root/scripts/generate-sbom.sh" \
  "$sbom_test_input" \
  "$sbom_test_root/corrupt.spdx.json" 2>"$sbom_test_root/corrupt.stderr"; then
  echo "SBOM generator accepted a checksum mismatch" >&2
  exit 1
fi

if MCP_SYNC_SYFT_HOST_OS=darwin \
  MCP_SYNC_SYFT_HOST_ARCHITECTURE=arm64 \
  MCP_SYNC_SYFT_ASSET_MANIFEST="$sbom_test_manifest" \
  MCP_SYNC_SYFT_DOWNLOAD_BASE_URL=https://example.invalid/syft \
  MCP_SYNC_SYFT_CURL="$sbom_test_curl" \
  MCP_SYNC_SYFT_CURL_LOG="$sbom_test_root/permanent-curl.log" \
  MCP_SYNC_SYFT_CURL_ATTEMPTS="$sbom_test_root/permanent-curl.attempts" \
  MCP_SYNC_SYFT_FAKE_ASSET="$sbom_test_download_root/syft_1.50.0_darwin_arm64.tar.gz" \
  MCP_SYNC_SYFT_FAKE_SEQUENCE=404:22 \
  MCP_SYNC_SYFT_SLEEP="$sbom_test_sleep" \
  MCP_SYNC_SYFT_SLEEP_LOG="$sbom_test_root/permanent-sleep.log" \
  "$sbom_test_repository_root/scripts/generate-sbom.sh" \
  "$sbom_test_input" \
  "$sbom_test_root/permanent.spdx.json" 2>"$sbom_test_root/permanent.stderr"; then
  echo "SBOM generator accepted a permanent HTTP failure" >&2
  exit 1
fi
if [[ "$(<"$sbom_test_root/permanent-curl.attempts")" != 1 ]] ||
  [[ -e "$sbom_test_root/permanent-sleep.log" ]]; then
  echo "SBOM generator retried a permanent HTTP failure" >&2
  exit 1
fi

if MCP_SYNC_SYFT_HOST_OS=darwin \
  MCP_SYNC_SYFT_HOST_ARCHITECTURE=arm64 \
  MCP_SYNC_SYFT_ASSET_MANIFEST="$sbom_test_manifest" \
  MCP_SYNC_SYFT_DOWNLOAD_BASE_URL=https://example.invalid/syft \
  MCP_SYNC_SYFT_CURL="$sbom_test_curl" \
  MCP_SYNC_SYFT_CURL_LOG="$sbom_test_root/exhausted-curl.log" \
  MCP_SYNC_SYFT_CURL_ATTEMPTS="$sbom_test_root/exhausted-curl.attempts" \
  MCP_SYNC_SYFT_FAKE_ASSET="$sbom_test_download_root/syft_1.50.0_darwin_arm64.tar.gz" \
  MCP_SYNC_SYFT_FAKE_SEQUENCE=503:22,503:22,503:22,503:22,503:22 \
  MCP_SYNC_SYFT_SLEEP="$sbom_test_sleep" \
  MCP_SYNC_SYFT_SLEEP_LOG="$sbom_test_root/exhausted-sleep.log" \
  "$sbom_test_repository_root/scripts/generate-sbom.sh" \
  "$sbom_test_input" \
  "$sbom_test_root/exhausted.spdx.json" 2>"$sbom_test_root/exhausted.stderr"; then
  echo "SBOM generator accepted exhausted transient failures" >&2
  exit 1
fi
if [[ "$(<"$sbom_test_root/exhausted-curl.attempts")" != 5 ]] ||
  [[ "$(paste -sd, "$sbom_test_root/exhausted-sleep.log")" != 1,2,4,8 ]]; then
  echo "SBOM generator did not enforce the exact exhausted retry bound" >&2
  exit 1
fi

if ! grep -F 'failed SHA-256 verification' "$sbom_test_root/corrupt.stderr" >/dev/null; then
  echo "SBOM generator did not report checksum rejection" >&2
  exit 1
fi

if MCP_SYNC_SYFT_HOST_OS=linux \
  MCP_SYNC_SYFT_HOST_ARCHITECTURE=riscv64 \
  MCP_SYNC_SYFT_ASSET_MANIFEST="$sbom_test_manifest" \
  MCP_SYNC_SYFT_CURL="$sbom_test_curl" \
  "$sbom_test_repository_root/scripts/generate-sbom.sh" \
  "$sbom_test_input" \
  "$sbom_test_root/unsupported.spdx.json" 2>"$sbom_test_root/unsupported.stderr"; then
  echo "SBOM generator accepted an unsupported host" >&2
  exit 1
fi
if ! grep -F 'unsupported Syft host architecture' "$sbom_test_root/unsupported.stderr" >/dev/null; then
  echo "SBOM generator did not report unsupported host rejection" >&2
  exit 1
fi

printf 'Syft SBOM generation policy tests passed.\n'
