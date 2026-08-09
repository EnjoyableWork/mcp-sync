#!/usr/bin/env bash

set -euo pipefail

repository_artifact_test_root="$(
  mktemp -d "${TMPDIR:-/tmp}/mcp-sync-artifact-policy.XXXXXX"
)"
repository_artifact_verifier="$(
  cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P
)/verify-repository-artifacts.sh"

cleanup_repository_artifact_test() {
  rm -rf -- "$repository_artifact_test_root"
}
trap cleanup_repository_artifact_test EXIT

create_artifact_test_repository() {
  local repository_name=$1
  local repository_path="$repository_artifact_test_root/$repository_name"

  git init --quiet "$repository_path"
  printf '%s\n' "$repository_path"
}

text_repository="$(create_artifact_test_repository text)"
printf '#!/usr/bin/env bash\nprintf "reviewable text\\n"\n' \
  >"$text_repository/reviewable.sh"
git -C "$text_repository" add -- reviewable.sh
"$repository_artifact_verifier" "$text_repository" >/dev/null

binary_repository="$(create_artifact_test_repository binary)"
printf 'MZ\000\000unreviewable' >"$binary_repository/generated-program"
git -C "$binary_repository" add -- generated-program
if "$repository_artifact_verifier" "$binary_repository" \
  >"$repository_artifact_test_root/binary-output" 2>&1; then
  echo "repository artifact policy accepted binary content" >&2
  exit 1
fi
grep -F 'tracked binary content is not permitted' \
  "$repository_artifact_test_root/binary-output" >/dev/null

extension_repository="$(create_artifact_test_repository extension)"
printf 'text with a generated executable extension\n' \
  >"$extension_repository/generated.exe"
git -C "$extension_repository" add -- generated.exe
if "$repository_artifact_verifier" "$extension_repository" \
  >"$repository_artifact_test_root/extension-output" 2>&1; then
  echo "repository artifact policy accepted a generated executable extension" >&2
  exit 1
fi
grep -F 'tracked generated or executable artifact extension is not permitted' \
  "$repository_artifact_test_root/extension-output" >/dev/null

echo "Verified repository artifact policy acceptance and rejection paths."
