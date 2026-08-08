# Stable release runbook

This runbook is the operational contract for `MCP-021`. It does not make an
unpublished channel current: the release, registry records, downstream package
repositories, and native smoke runs are the evidence for each live claim.

## Fixed first-release identity

The first stable version is `0.1.0`, tagged `v0.1.0`, from
`EnjoyableWork/mcp-sync`. Cargo publishes `enjoyable-mcp-sync`, Homebrew uses
`EnjoyableWork/homebrew-tap` and formula `mcp-sync`, WinGet uses
`EnjoyableWork.mcp-sync`, and every channel installs an executable named
`mcp-sync`.

The GitHub Release must contain exactly these six archives, six matching SPDX
JSON SBOMs, and `SHA256SUMS`:

```text
mcp-sync-v0.1.0-aarch64-apple-darwin.tar.gz
mcp-sync-v0.1.0-x86_64-apple-darwin.tar.gz
mcp-sync-v0.1.0-aarch64-unknown-linux-gnu.tar.gz
mcp-sync-v0.1.0-x86_64-unknown-linux-gnu.tar.gz
mcp-sync-v0.1.0-aarch64-pc-windows-msvc.zip
mcp-sync-v0.1.0-x86_64-pc-windows-msvc.zip
```

Each SBOM replaces the archive suffix with `.spdx.json` while retaining the
complete version and Rust target.

## Repository controls

Before a stable tag exists, verify all of the following:

- GitHub Release immutability is enabled for `EnjoyableWork/mcp-sync`.
- The active `Protect stable release tags` ruleset selects `refs/tags/v*`,
  restricts creation, update, and deletion, and retains only the recorded named
  administrator as the explicit creation and emergency bypass.
- The `release` environment requires review and permits only `v*` tag
  deployments. The single-owner reviewer model is an authorization pause, not
  an independent second-person approval claim.
- No interactive administration token is stored as a release credential.
- The current `main` commit has green CI and a green six-target
  `Release preflight` run.

The tag-only `Publish immutable release` workflow uses the `release`
environment and checks the immutable-release setting and ruleset again before
building. A named ruleset bypass actor creates the annotated tag manually only
after preflight; the workflow token does not receive tag-bypass authority.

## Protected signing inputs

Store only the following three values as `release` environment secrets:

```text
MACOS_CERTIFICATE_P12_BASE64
MACOS_CERTIFICATE_PASSWORD
APPLE_NOTARY_KEY_P8_BASE64
```

Store these non-secret identifiers as `release` environment variables:

```text
APPLE_NOTARY_KEY_ID
APPLE_NOTARY_ISSUER_ID
AZURE_CLIENT_ID
AZURE_TENANT_ID
AZURE_SUBSCRIPTION_ID
ARTIFACT_SIGNING_ENDPOINT
ARTIFACT_SIGNING_ACCOUNT
ARTIFACT_SIGNING_PROFILE
```

The Apple certificate must be a current Developer ID Application identity. The
notary key is a team App Store Connect API key whose issuer is available. The
Azure app registration uses GitHub OIDC and has only the Artifact Signing role
needed for the named Public Trust account and certificate profile. Do not put
credential values in a ticket, command argument, workflow log, or tracked file.

The macOS workflow imports the certificate into an ephemeral keychain, signs
the Mach-O with identifier `com.enjoyablework.mcp-sync`, hardened runtime, and
a secure timestamp, and removes the keychain and API key after use. Apple's
notary service accepts ZIP, DMG, or PKG submissions rather than the published
`.tar.gz`; therefore each native job submits a temporary ZIP containing the
exact signed executable and release documents, requires `Accepted`, performs a
local Gatekeeper assessment, and then packages those same bytes into the final
tar archive. No unnotarized binary substitution occurs after acceptance.

The Windows workflow authenticates to Azure through GitHub OIDC, signs the
native executable with Artifact Signing Public Trust and RFC3161/SHA-256
timestamping, and requires `Get-AuthenticodeSignature` to report a valid code
signing certificate plus a timestamp certificate before ZIP creation.

## Prepublication sequence

1. Work from a clean release-preparation branch whose package version and
   release notes agree.
2. Run the repository handoff gates and `cargo publish --dry-run --locked`.
3. Merge through the normal repository flow. Require green CI and six native
   package/installed-restore jobs from `.github/workflows/release-preflight.yml`
   on the exact `main` commit.
4. Recheck that `enjoyable-mcp-sync` is either unclaimed on crates.io or already
   belongs to this repository at the same version. Do not rely on an earlier
   search result.
5. Verify all protected signing input names exist without displaying their
   values. Confirm Apple and Microsoft identity validation is active rather
   than merely requested.
6. As the named ruleset bypass actor, create one annotated `v0.1.0` tag at the
   exact current `main` commit and push only that tag. Never move or reuse it.
7. Approve the `release` environment deployment after confirming the displayed
   tag and commit. The tag workflow builds natively, signs, notarizes, packages,
   exercises installed restore, generates and verifies SBOMs and attestations,
   assembles the exact draft, verifies downloaded bytes, and only then
   publishes it.
8. Require the release API to report `immutable: true`; verify the release
   attestation and all 13 downloaded assets before publishing any downstream
   channel.

## Cargo first publication

crates.io cannot establish a trusted publisher before the crate exists. From a
fresh checkout of the immutable release tag, first compare `HEAD`, the tag
target, `Cargo.toml`, and the GitHub Release source commit. Then run:

```bash
cargo package --locked
cargo publish --dry-run --locked
cargo publish --locked
```

Inject the one-time, scope-minimized crates.io token through the operator's
protected credential mechanism; do not include it in shell history or a
workflow. Revoke it immediately after publication. Verify
`cargo install enjoyable-mcp-sync --version '=0.1.0' --locked` natively on all
six supported OS/CPU combinations and exercise `scripts/smoke-installed.sh` or
`scripts/smoke-installed.ps1` against the installed executable. Configure a
repository/environment-bound crates.io trusted publisher for later versions
only after ownership exists.

## Homebrew and WinGet

Create `EnjoyableWork/homebrew-tap` only after the immutable GitHub Release is
verified. Formula `mcp-sync` must select the matching macOS or GNU/Linux archive
for ARM64 or x64, use the exact release URL and SHA-256 from `SHA256SUMS`, install
only the executable, and pass `brew audit`, formula tests, version output, and
the installed restore smoke on all four native Unix combinations.

Download the complete immutable asset set into one otherwise-empty directory,
verify it with `scripts/verify-published-release.sh`, and generate both
downstream definitions from that exact `SHA256SUMS` file:

```bash
scripts/generate-release-channels.sh \
  <downloaded-release-assets> \
  0.1.0 \
  <new-output-directory>
```

The generator refuses a pre-existing output directory and writes through a
temporary sibling. Copy
`homebrew/Formula/mcp-sync.rb` into the newly created organization tap. Copy
the generated `winget/manifests/e/EnjoyableWork/mcp-sync/0.1.0` directory into
the matching path on the submission branch in the personal `winget-pkgs` fork.
Do not hand-edit URLs, hashes, identifiers, architectures, or versions after
generation; regenerate from the verified immutable assets if any input is
wrong.

Generate WinGet ZIP/portable manifests for publisher `EnjoyableWork`, package
identifier `EnjoyableWork.mcp-sync`, package name `mcp-sync`, and version
`0.1.0`. Pin the ARM64 and x64 immutable GitHub archive URLs and hashes. Validate
with the current WinGet tooling, submit through a personal `winget-pkgs` fork,
wait for the public manifest to merge, and then require native ARM64 and x64
`winget install --id EnjoyableWork.mcp-sync --version 0.1.0 --exact` plus the
installed restore smoke.

After Cargo, the tap formula, and the WinGet manifests are public, dispatch
`.github/workflows/release-channels.yml` with version `0.1.0`. This read-only,
credential-free workflow first requires the immutable release and all 13
attestations, an unyanked crates.io record owned by this repository, and exact
byte equality between freshly generated downstream definitions and their
published Homebrew and WinGet copies. It then installs and exercises the
GitHub archive and Cargo package on all six native targets, the Homebrew formula
on both macOS and both GNU/Linux targets, and the public WinGet package on both
Windows targets. The macOS and Windows binary-channel jobs recheck native code
signing and timestamp trust before the installed restore journey.

Record the immutable release, registry records, downstream commits or pull
requests, and every native smoke run in `PROJECT.md`. MCP-021 and M2 remain open
until all four channels install the same protected version.

## Failure and correction

Do not publish a draft when a signature, timestamp, notarization, SBOM,
attestation, checksum, or installed smoke fails. A draft may be repaired and
reverified because it is not public or immutable. Once published, never delete
or replace an asset, move the tag, or overwrite package metadata. Correct any
release defect with a new version and preserve the failed evidence needed to
explain the superseding release.
