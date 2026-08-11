#!/usr/bin/env bash

set -euo pipefail

if [[ $# -ne 0 ]]; then
  echo "usage: $0" >&2
  exit 2
fi

release_policy_root="$(git rev-parse --show-toplevel)"
release_policy_validator="$release_policy_root/scripts/validate-release-version.sh"

release_policy_expect_acceptance() {
  local case_name=$1
  shift
  local output
  if ! output="$($release_policy_validator "$@" 2>/dev/null)"; then
    echo "release version policy rejected accepted case: $case_name" >&2
    exit 1
  fi
  if [[ "$output" != "$3" ]]; then
    echo "release version policy emitted the wrong accepted version: $case_name" >&2
    exit 1
  fi
}

release_policy_expect_rejection() {
  local case_name=$1
  shift
  if "$release_policy_validator" "$@" >/dev/null 2>&1; then
    echo "release version policy accepted rejected case: $case_name" >&2
    exit 1
  fi
}

release_policy_expect_acceptance patch-release \
  future v0.1.1 0.1.1 0.1.0
release_policy_expect_acceptance partial-publication-recovery \
  future v0.1.1 0.1.1 0.1.0 0.1.1
release_policy_expect_acceptance minor-release \
  future v0.2.0 0.2.0 0.1.0 0.1.9
release_policy_expect_acceptance major-release \
  future v1.0.0 1.0.0 0.1.0 0.99.99
release_policy_expect_acceptance unbounded-numeric-components \
  future \
  v18446744073709551616.0.0 \
  18446744073709551616.0.0 \
  18446744073709551615.99.99
release_policy_expect_acceptance initial-published-release \
  published v0.1.0 0.1.0
release_policy_expect_acceptance later-published-release \
  published v12.34.56 12.34.56
release_policy_expect_acceptance fixed-live-rehearsal \
  rehearsal v0.1.0 0.1.0

release_policy_expect_rejection missing-inventory \
  future v0.1.1 0.1.1
release_policy_expect_rejection immutable-initial-version \
  future v0.1.0 0.1.0 0.1.0
release_policy_expect_rejection before-initial-version \
  future v0.0.9 0.0.9 0.1.0
release_policy_expect_rejection manifest-mismatch \
  future v0.1.1 0.1.2 0.1.0
release_policy_expect_rejection older-than-published \
  future v0.1.1 0.1.1 0.2.0
release_policy_expect_rejection missing-prefix \
  future 0.1.1 0.1.1 0.1.0
release_policy_expect_rejection leading-zero \
  future v01.2.3 01.2.3 0.1.0
release_policy_expect_rejection prerelease \
  future v1.2.3-rc.1 1.2.3-rc.1 0.1.0
release_policy_expect_rejection malformed-inventory \
  future v1.2.3 1.2.3 not-stable
release_policy_expect_rejection published-before-baseline \
  published v0.0.99 0.0.99
release_policy_expect_rejection later-rehearsal \
  rehearsal v0.1.1 0.1.1
release_policy_expect_rejection unknown-mode \
  unknown v1.0.0 1.0.0

printf 'Verified canonical, monotonic, and recovery-safe release version policy.\n'
