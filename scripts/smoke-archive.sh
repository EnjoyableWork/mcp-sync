#!/usr/bin/env bash

set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "usage: $0 <release archive> <expected version>" >&2
  exit 2
fi

smoke_archive=$1
smoke_version=$2

if [[ ! -f "$smoke_archive" ]]; then
  echo "release archive is missing" >&2
  exit 1
fi

smoke_archive=$(cd "$(dirname "$smoke_archive")" && pwd)/$(basename "$smoke_archive")
smoke_extract_root=$(mktemp -d)

cleanup_smoke_extract_root() {
  rm -rf -- "$smoke_extract_root"
}
trap cleanup_smoke_extract_root EXIT

tar -xzf "$smoke_archive" -C "$smoke_extract_root"

for required_file in mcp-sync LICENSE README.md Cargo.lock; do
  if [[ ! -f "$smoke_extract_root/$required_file" ]]; then
    echo "release archive is missing $required_file" >&2
    exit 1
  fi
done

"$(dirname "$0")/smoke-installed.sh" \
  "$smoke_extract_root/mcp-sync" \
  "$smoke_version"
