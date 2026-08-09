#!/usr/bin/env bash

set -euo pipefail

repository_artifact_root=${1:-}

if [[ $# -gt 1 ]]; then
  echo "usage: $0 [repository-root]" >&2
  exit 2
fi
if [[ -z "$repository_artifact_root" ]]; then
  repository_artifact_root="$(git rev-parse --show-toplevel)"
fi
if ! git -C "$repository_artifact_root" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  echo "repository artifact verification requires a Git worktree" >&2
  exit 2
fi
for repository_artifact_command in file git tr; do
  if ! command -v "$repository_artifact_command" >/dev/null 2>&1; then
    echo "repository artifact verification requires $repository_artifact_command" >&2
    exit 2
  fi
done

repository_artifact_count=0
repository_artifact_failures=0

while IFS= read -r -d '' repository_artifact_entry; do
  repository_artifact_metadata=${repository_artifact_entry%%$'\t'*}
  repository_artifact_path=${repository_artifact_entry#*$'\t'}
  repository_artifact_mode=${repository_artifact_metadata%% *}
  repository_artifact_remainder=${repository_artifact_metadata#* }
  repository_artifact_object=${repository_artifact_remainder%% *}
  repository_artifact_count=$((repository_artifact_count + 1))

  if [[ "$repository_artifact_mode" == 160000 ]]; then
    printf 'tracked submodule content is not permitted: %q\n' \
      "$repository_artifact_path" >&2
    repository_artifact_failures=$((repository_artifact_failures + 1))
    continue
  fi

  repository_artifact_lower_path="$({
    printf '%s' "$repository_artifact_path" | tr '[:upper:]' '[:lower:]'
  })"
  case "$repository_artifact_lower_path" in
    *.7z | *.a | *.bin | *.cab | *.class | *.crate | *.deb | *.dll | *.dmg | \
      *.dylib | *.exe | *.gz | *.jar | *.msi | *.o | *.obj | *.pdb | *.pkg | \
      *.pyc | *.rar | *.rpm | *.so | *.tar | *.tgz | *.wasm | *.whl | *.xz | \
      *.zip | *.zst)
      printf 'tracked generated or executable artifact extension is not permitted: %q\n' \
        "$repository_artifact_path" >&2
      repository_artifact_failures=$((repository_artifact_failures + 1))
      continue
      ;;
  esac

  repository_artifact_size="$(
    git -C "$repository_artifact_root" cat-file -s "$repository_artifact_object"
  )"
  if [[ "$repository_artifact_size" == 0 ]]; then
    continue
  fi
  repository_artifact_encoding="$({
    git -C "$repository_artifact_root" cat-file blob "$repository_artifact_object" |
      file -b --mime-encoding -
  })"
  if [[ "$repository_artifact_encoding" == binary ]]; then
    printf 'tracked binary content is not permitted: %q\n' \
      "$repository_artifact_path" >&2
    repository_artifact_failures=$((repository_artifact_failures + 1))
  fi
done < <(git -C "$repository_artifact_root" ls-files --stage -z)

if [[ "$repository_artifact_failures" -ne 0 ]]; then
  printf 'repository artifact policy rejected %d tracked path(s)\n' \
    "$repository_artifact_failures" >&2
  exit 1
fi

printf 'Verified %d tracked paths contain no generated executables or binary artifacts.\n' \
  "$repository_artifact_count"
