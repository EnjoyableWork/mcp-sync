#!/usr/bin/env bash

set -euo pipefail

distribution_authentication_repository=${1:-}
distribution_authentication_version=${2:-}

if [[ "$distribution_authentication_repository" != EnjoyableWork/mcp-sync ||
  "$distribution_authentication_version" != 0.1.1 ]]; then
  echo "usage: $0 EnjoyableWork/mcp-sync 0.1.1" >&2
  exit 2
fi
if ! gh auth status --hostname github.com >/dev/null 2>&1; then
  echo "GitHub CLI must be authenticated for release and attestation verification" >&2
  exit 1
fi
for distribution_authentication_command in gh jq curl cmp; do
  if ! command -v "$distribution_authentication_command" >/dev/null 2>&1; then
    echo "distribution authentication verification requires $distribution_authentication_command" >&2
    exit 2
  fi
done

distribution_authentication_root="$(git rev-parse --show-toplevel)"
distribution_authentication_tag="v$distribution_authentication_version"
distribution_authentication_temp="$(
  mktemp -d "${TMPDIR:-/tmp}/mcp-sync-distribution-authentication.XXXXXX"
)"
cleanup_distribution_authentication() {
  rm -rf -- "$distribution_authentication_temp"
}
trap cleanup_distribution_authentication EXIT

distribution_authentication_headers=(
  --header 'Accept: application/vnd.github+json'
  --header 'X-GitHub-Api-Version: 2026-03-10'
)

distribution_authentication_release="$(
  gh api \
    "repos/$distribution_authentication_repository/releases/tags/$distribution_authentication_tag" \
    "${distribution_authentication_headers[@]}"
)"
jq -e \
  --arg tag "$distribution_authentication_tag" \
  '.tag_name == $tag and .draft == false and .prerelease == false and
   .immutable == true and (.assets | length) == 7' \
  <<<"$distribution_authentication_release" >/dev/null

distribution_authentication_ref="$(
  gh api \
    "repos/$distribution_authentication_repository/git/ref/tags/$distribution_authentication_tag" \
    "${distribution_authentication_headers[@]}"
)"
jq -e '.object.type == "tag"' <<<"$distribution_authentication_ref" >/dev/null
distribution_authentication_tag_object="$(
  jq -r '.object.sha' <<<"$distribution_authentication_ref"
)"
distribution_authentication_tag_payload="$(
  gh api \
    "repos/$distribution_authentication_repository/git/tags/$distribution_authentication_tag_object" \
    "${distribution_authentication_headers[@]}"
)"
jq -e '.object.type == "commit"' \
  <<<"$distribution_authentication_tag_payload" >/dev/null
distribution_authentication_commit="$(
  jq -r '.object.sha' <<<"$distribution_authentication_tag_payload"
)"

distribution_authentication_assets="$distribution_authentication_temp/assets"
mkdir -p "$distribution_authentication_assets"
gh release download "$distribution_authentication_tag" \
  --repo "$distribution_authentication_repository" \
  --dir "$distribution_authentication_assets"
"$distribution_authentication_root/scripts/verify-published-source-linux-release.sh" \
  "$distribution_authentication_assets" \
  "$distribution_authentication_version"

gh release verify "$distribution_authentication_tag" \
  --repo "$distribution_authentication_repository" \
  --format json >/dev/null
while IFS= read -r -d '' distribution_authentication_asset; do
  distribution_authentication_asset_name=${distribution_authentication_asset##*/}
  distribution_authentication_attestation="$(
    gh attestation verify "$distribution_authentication_asset" \
      --repo "$distribution_authentication_repository" \
      --signer-workflow "$distribution_authentication_repository/.github/workflows/source-linux-release.yml" \
      --source-ref "refs/tags/$distribution_authentication_tag" \
      --source-digest "$distribution_authentication_commit" \
      --format json
  )"
  jq -e \
    --arg asset "$distribution_authentication_asset_name" \
    --arg repository "$distribution_authentication_repository" \
    --arg source_ref "refs/tags/$distribution_authentication_tag" \
    --arg source_digest "$distribution_authentication_commit" '
      type == "array" and length > 0 and
      any(.[];
        .verificationResult.statement._type ==
          "https://in-toto.io/Statement/v1" and
        .verificationResult.statement.predicateType ==
          "https://slsa.dev/provenance/v1" and
        any(.verificationResult.statement.subject[];
          .name == $asset and
          (.digest.sha256 | type == "string" and
           test("^[0-9a-f]{64}$"))) and
        .verificationResult.statement.predicate.buildDefinition.buildType ==
          "https://actions.github.io/buildtypes/workflow/v1" and
        .verificationResult.statement.predicate.buildDefinition.externalParameters.workflow.repository ==
          ("https://github.com/" + $repository) and
        .verificationResult.statement.predicate.buildDefinition.externalParameters.workflow.path ==
          ".github/workflows/source-linux-release.yml" and
        .verificationResult.statement.predicate.buildDefinition.externalParameters.workflow.ref ==
          $source_ref and
        .verificationResult.statement.predicate.buildDefinition.internalParameters.github.runner_environment ==
          "github-hosted" and
        any(.verificationResult.statement.predicate.buildDefinition.resolvedDependencies[];
          .digest.gitCommit == $source_digest) and
        .verificationResult.signature.certificate.issuer ==
          "https://token.actions.githubusercontent.com" and
        .verificationResult.signature.certificate.runnerEnvironment ==
          "github-hosted" and
        .verificationResult.signature.certificate.githubWorkflowRepository ==
          $repository and
        .verificationResult.signature.certificate.githubWorkflowRef ==
          $source_ref and
        .verificationResult.signature.certificate.githubWorkflowSHA ==
          $source_digest and
        .verificationResult.signature.certificate.sourceRepositoryRef ==
          $source_ref and
        .verificationResult.signature.certificate.sourceRepositoryDigest ==
          $source_digest)
    ' <<<"$distribution_authentication_attestation" >/dev/null
done < <(find "$distribution_authentication_assets" -maxdepth 1 -type f -print0)

distribution_authentication_crates_metadata="$distribution_authentication_temp/crates.json"
curl --fail --silent --show-error --location --retry 5 \
  --proto '=https' --proto-redir '=https' --tlsv1.2 \
  --output "$distribution_authentication_crates_metadata" \
  --header 'User-Agent: mcp-sync-supply-chain-verifier/0.1 (+https://github.com/EnjoyableWork/mcp-sync)' \
  'https://crates.io/api/v1/crates/enjoyable-mcp-sync'
jq -e \
  --arg version "$distribution_authentication_version" \
  '.crate.repository == "https://github.com/EnjoyableWork/mcp-sync" and
   any(.versions[]; .num == $version and .yanked == false)' \
  "$distribution_authentication_crates_metadata" >/dev/null

distribution_authentication_crate="$distribution_authentication_temp/enjoyable-mcp-sync.crate"
curl --fail --silent --show-error --location --retry 5 \
  --proto '=https' --proto-redir '=https' --tlsv1.2 \
  --output "$distribution_authentication_crate" \
  --header 'User-Agent: mcp-sync-supply-chain-verifier/0.1 (+https://github.com/EnjoyableWork/mcp-sync)' \
  "https://crates.io/api/v1/crates/enjoyable-mcp-sync/$distribution_authentication_version/download"
cmp --silent \
  "$distribution_authentication_assets/enjoyable-mcp-sync-$distribution_authentication_version.crate" \
  "$distribution_authentication_crate"

distribution_authentication_formula="$distribution_authentication_temp/mcp-sync.rb"
curl --fail --silent --show-error --location --retry 5 \
  --proto '=https' --proto-redir '=https' --tlsv1.2 \
  --output "$distribution_authentication_formula" \
  'https://raw.githubusercontent.com/EnjoyableWork/homebrew-tap/main/Formula/mcp-sync.rb'
cmp --silent \
  "$distribution_authentication_assets/mcp-sync.rb" \
  "$distribution_authentication_formula"

printf 'Verified immutable release and SLSA v1.0 Build Level 2 provenance plus byte-identical HTTPS Cargo and Homebrew distribution for %s.\n' \
  "$distribution_authentication_tag"
