#!/usr/bin/env bash

set -euo pipefail

if [[ $# -ne 3 ]]; then
  echo "usage: $0 <candidate version> <candidate formula> <published formula>" >&2
  exit 2
fi

homebrew_candidate_version=$1
homebrew_candidate_formula=$2
homebrew_published_formula=$3
homebrew_policy_root="$(
  cd "$(dirname "${BASH_SOURCE[0]}")"
  pwd
)"
homebrew_version_validator="$homebrew_policy_root/validate-release-version.sh"

homebrew_formula_version() {
  local formula_path=$1
  local formula_label=$2
  local formula_url
  local formula_url_count
  local formula_tag_version
  local formula_package_version

  if [[ ! -f "$formula_path" || -L "$formula_path" ]]; then
    echo "$formula_label formula must be a regular, non-symbolic-link file" >&2
    return 1
  fi
  formula_url="$(
    sed -n 's/^  url "\(.*\)"$/\1/p' "$formula_path"
  )"
  formula_url_count="$(
    sed -n 's/^  url "\(.*\)"$/\1/p' "$formula_path" | awk 'END { print NR }'
  )"
  if [[ "$formula_url_count" -ne 1 ]]; then
    echo "$formula_label formula must contain exactly one canonical source URL" >&2
    return 1
  fi
  if [[ "$formula_url" =~ ^https://github\.com/EnjoyableWork/mcp-sync/releases/download/v([^/]+)/enjoyable-mcp-sync-([^/]+)\.crate$ ]]; then
    formula_tag_version=${BASH_REMATCH[1]}
    formula_package_version=${BASH_REMATCH[2]}
  else
    echo "$formula_label formula does not use the canonical immutable release URL" >&2
    return 1
  fi
  if [[ "$formula_tag_version" != "$formula_package_version" ]]; then
    echo "$formula_label formula tag and package versions do not match" >&2
    return 1
  fi
  "$homebrew_version_validator" \
    published \
    "v$formula_tag_version" \
    "$formula_tag_version" >/dev/null
  printf '%s\n' "$formula_tag_version"
}

"$homebrew_version_validator" \
  published \
  "v$homebrew_candidate_version" \
  "$homebrew_candidate_version" >/dev/null
homebrew_formula_candidate_version="$(
  homebrew_formula_version "$homebrew_candidate_formula" candidate
)"
if [[ "$homebrew_formula_candidate_version" != "$homebrew_candidate_version" ]]; then
  echo "candidate formula version does not match the requested release" >&2
  exit 1
fi

if [[ ! -e "$homebrew_published_formula" && ! -L "$homebrew_published_formula" ]]; then
  printf 'create\n'
  exit 0
fi
if [[ ! -f "$homebrew_published_formula" || -L "$homebrew_published_formula" ]]; then
  echo "published formula must be a regular, non-symbolic-link file" >&2
  exit 1
fi
if cmp --silent "$homebrew_candidate_formula" "$homebrew_published_formula"; then
  printf 'noop\n'
  exit 0
fi

homebrew_formula_published_version="$(
  homebrew_formula_version "$homebrew_published_formula" published
)"
if [[ "$homebrew_formula_published_version" == "$homebrew_candidate_version" ]]; then
  echo "refusing to replace different formula bytes for the same immutable version" >&2
  exit 1
fi
"$homebrew_version_validator" \
  future \
  "v$homebrew_candidate_version" \
  "$homebrew_candidate_version" \
  "$homebrew_formula_published_version" >/dev/null

printf 'update %s\n' "$homebrew_formula_published_version"
