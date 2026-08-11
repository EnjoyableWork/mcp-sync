#!/usr/bin/env bash

set -euo pipefail

if [[ $# -ne 0 ]]; then
  echo "usage: $0" >&2
  exit 2
fi

homebrew_test_root="$(git rev-parse --show-toplevel)"
homebrew_test_validator="$homebrew_test_root/scripts/validate-homebrew-formula-update.sh"
homebrew_test_temp="$(
  mktemp -d "${TMPDIR:-/tmp}/mcp-sync-homebrew-formula-policy.XXXXXX"
)"
cleanup_homebrew_test() {
  case "$homebrew_test_temp" in
    "${TMPDIR:-/tmp}/mcp-sync-homebrew-formula-policy."*)
      rm -rf -- "$homebrew_test_temp"
      ;;
    *)
      echo "refusing to remove an unexpected Homebrew policy test path" >&2
      return 1
      ;;
  esac
}
trap cleanup_homebrew_test EXIT

homebrew_test_formula() {
  local formula_path=$1
  local tag_version=$2
  local package_version=${3:-$tag_version}
  local formula_hash=${4:-0000000000000000000000000000000000000000000000000000000000000000}
  printf '%s\n' \
    'class McpSync < Formula' \
    "  url \"https://github.com/EnjoyableWork/mcp-sync/releases/download/v$tag_version/enjoyable-mcp-sync-$package_version.crate\"" \
    "  sha256 \"$formula_hash\"" \
    'end' >"$formula_path"
}

homebrew_test_expect() {
  local case_name=$1
  local expected=$2
  shift 2
  local output
  if ! output="$($homebrew_test_validator "$@" 2>/dev/null)"; then
    echo "Homebrew formula policy rejected accepted case: $case_name" >&2
    exit 1
  fi
  if [[ "$output" != "$expected" ]]; then
    echo "Homebrew formula policy returned the wrong decision: $case_name" >&2
    exit 1
  fi
}

homebrew_test_reject() {
  local case_name=$1
  shift
  if "$homebrew_test_validator" "$@" >/dev/null 2>&1; then
    echo "Homebrew formula policy accepted rejected case: $case_name" >&2
    exit 1
  fi
}

published="$homebrew_test_temp/published.rb"
candidate="$homebrew_test_temp/candidate.rb"
missing="$homebrew_test_temp/missing.rb"
homebrew_test_formula "$published" 0.1.0
homebrew_test_formula "$candidate" 0.1.1 0.1.1 1111111111111111111111111111111111111111111111111111111111111111

homebrew_test_expect create create 0.1.1 "$candidate" "$missing"
homebrew_test_expect noop noop 0.1.0 "$published" "$published"
homebrew_test_expect monotonic-update 'update 0.1.0' 0.1.1 "$candidate" "$published"

newer="$homebrew_test_temp/newer.rb"
homebrew_test_formula "$newer" 1.0.0
homebrew_test_reject downgrade 0.1.1 "$candidate" "$newer"

same_version_mismatch="$homebrew_test_temp/same-version-mismatch.rb"
homebrew_test_formula "$same_version_mismatch" 0.1.1 0.1.1 2222222222222222222222222222222222222222222222222222222222222222
homebrew_test_reject same-version-mismatch \
  0.1.1 "$candidate" "$same_version_mismatch"

mismatched_url="$homebrew_test_temp/mismatched-url.rb"
homebrew_test_formula "$mismatched_url" 0.1.1 0.1.2
homebrew_test_reject mismatched-url 0.1.1 "$mismatched_url" "$published"

malformed="$homebrew_test_temp/malformed.rb"
printf '%s\n' 'class McpSync < Formula' 'end' >"$malformed"
homebrew_test_reject malformed-formula 0.1.1 "$malformed" "$published"
homebrew_test_reject leading-zero 00.1.1 "$candidate" "$published"
homebrew_test_reject prerelease 0.1.1-rc.1 "$candidate" "$published"

published_link="$homebrew_test_temp/published-link.rb"
ln -s "$published" "$published_link"
homebrew_test_reject symbolic-link 0.1.1 "$candidate" "$published_link"

printf 'Verified create, no-op, monotonic update, and fail-closed Homebrew formula policy.\n'
