#!/usr/bin/env bash

set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <mcp-sync executable>" >&2
  exit 2
fi

trust_executable=$1

if [[ ! -x "$trust_executable" ]]; then
  echo "macOS trust executable is missing or not executable" >&2
  exit 1
fi

codesign --verify --deep --strict "$trust_executable"
trust_details=$(codesign -d --verbose=4 "$trust_executable" 2>&1)

grep -q '^Identifier=com.enjoyablework.mcp-sync$' <<<"$trust_details"
grep -Eq '^CodeDirectory .*flags=.*runtime' <<<"$trust_details"
grep -q '^Timestamp=' <<<"$trust_details"
spctl --assess --type execute --verbose=4 "$trust_executable" >/dev/null
