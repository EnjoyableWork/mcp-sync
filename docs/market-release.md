# Zero-cost market-release runbook

This is the operational contract for `MCP-028` and `MCP-029`. `MCP-028`
proves the path without creating a tag, GitHub Release, crates.io version, or
Homebrew commit. `MCP-029` is the separately authorized publication step.
The funded Apple/Windows pipeline remains available in the
[signed-native runbook](release.md).

## Fixed `v0.1.0` identity and boundary

The market-validation version is `0.1.0`, tagged `v0.1.0`, from
`EnjoyableWork/mcp-sync`. Cargo publishes `enjoyable-mcp-sync`, Homebrew uses
`EnjoyableWork/homebrew-tap` and formula `mcp-sync`, and every path installs an
executable named `mcp-sync`.

The immutable GitHub Release must contain exactly these seven uploaded assets:

```text
SHA256SUMS
enjoyable-mcp-sync-0.1.0.crate
mcp-sync.rb
mcp-sync-v0.1.0-aarch64-unknown-linux-gnu.spdx.json
mcp-sync-v0.1.0-aarch64-unknown-linux-gnu.tar.gz
mcp-sync-v0.1.0-x86_64-unknown-linux-gnu.spdx.json
mcp-sync-v0.1.0-x86_64-unknown-linux-gnu.tar.gz
```

Each uploaded asset receives GitHub build-provenance attestation from
`.github/workflows/market-release.yml`. `SHA256SUMS` covers the other six files
exactly. There is no project-issued macOS or Windows binary in this release and
there is no WinGet package. macOS and Windows users build the exact Cargo
package locally; Homebrew builds that same package on macOS and GNU/Linux.

## Trigger separation and repository controls

Before a stable tag exists, verify all of the following:

- GitHub Release immutability is enabled for `EnjoyableWork/mcp-sync`.
- The active `Protect stable release tags` ruleset selects `refs/tags/v*` and
  restricts creation, update, and deletion to the recorded emergency bypass.
- The `release` environment requires review and permits only `v*` tag
  deployments.
- The exact current `main` commit has green CI, green
  `Market release preflight`, and green retained six-target `Release preflight`
  runs.
- No interactive administration token or Apple/Microsoft signing credential is
  present in the market workflow.

A push of the exact annotated `v0.1.0` tag can invoke only
`.github/workflows/market-release.yml`. The funded
`.github/workflows/release.yml` has no tag trigger; it requires manual dispatch
on a selected stable tag and `confirm_funded_signing=true`. Do not weaken this
separation by adding a broad `v*` push trigger to the funded workflow.

## Non-publishing preflight

`Market release preflight` runs on pull requests, pushes to `main`, and manual
dispatch without secrets or write permissions. It must prove all of the
following before `MCP-028` is complete:

1. Two consecutive `cargo package --locked` operations produce the same
   `enjoyable-mcp-sync-0.1.0.crate` SHA-256.
2. Two generator runs produce byte-identical Cargo and Homebrew outputs.
3. The exact `.crate` installs and passes the installed restore journey on
   native ARM64 and x64 macOS, GNU/Linux, and Windows MSVC hosts.
4. The exact generated formula passes Homebrew style, audit, source install,
   formula test, and installed restore on native ARM64 and x64 macOS and
   GNU/Linux hosts. Preflight seeds Homebrew's disposable runner cache with the
   local `.crate`; it does not publish or substitute a different formula.
5. Only native GNU/Linux ARM64 and x64 archives are assembled, smoke-tested,
   and paired with valid SPDX 2.3 JSON SBOMs.
6. `scripts/verify-market-release-assets.sh` and
   `scripts/verify-published-market-release.sh` accept the exact dry-run payload
   and reject an extra macOS, Windows, or unrelated file.
7. The retained `Release preflight` continues to build and smoke all six
   unsigned test artifacts without credentials, proving that the future signed
   pipeline has not decayed.

The preflight artifacts have one-day retention and are not releases. They must
not be described or shared as supported project-issued macOS or Windows
binaries.

## Human authority required for `MCP-029`

No Apple Developer Program or Windows signing purchase is required. The owner
must still:

1. sign in to crates.io through the intended publisher account, verify its
   email address, recheck that `enjoyable-mcp-sync` remains available, create a
   scope-minimized first-publication token, and store it only in the protected
   publication mechanism;
2. ensure public repository `EnjoyableWork/homebrew-tap` exists and give the
   release identity only the write access needed for `Formula/mcp-sync.rb`; and
3. dispatch `.github/workflows/release-authorize.yml` from the exact current
   `main` commit, review its evidence, then deliberately create and push the
   one annotated `v0.1.0` tag it authorizes.

Never place a token value in a command argument, ticket, tracked file, or
workflow log.

## GitHub Release publication

The `v0.1.0` tag invokes the protected market workflow. It revalidates the tag,
current `main` commit, package version, immutable-release setting, tag ruleset,
registry identity, and existing release state. It then:

1. packages the Cargo source twice and requires equal hashes;
2. generates the source-building Homebrew formula twice and requires equal
   bytes;
3. builds, packages, and exercises only native GNU/Linux ARM64 and x64
   archives;
4. generates SPDX SBOMs and attestations for both Linux archives, both SBOMs,
   the `.crate`, and the formula;
5. creates and attests the exact checksum manifest;
6. assembles a draft release, downloads and verifies every byte, and publishes
   only after the seven-file contract passes; and
7. requires immutable release state and a valid release attestation, then
   downloads and verifies the immutable bytes again.

The workflow never reads Apple or Microsoft credentials and never builds a
macOS or Windows release executable.

## Cargo first publication

crates.io cannot establish this crate's trusted publisher before the crate
exists. From a fresh checkout of the immutable tag, download the attested
`enjoyable-mcp-sync-0.1.0.crate`, then run:

```bash
cargo package --locked
cmp target/package/enjoyable-mcp-sync-0.1.0.crate \
  <downloaded-release>/enjoyable-mcp-sync-0.1.0.crate
cargo publish --dry-run --locked
cargo publish --locked
```

Revoke the first-publication token after success. Download the published crate
from crates.io and require byte equality with the immutable release asset. A
mismatch is not repaired by replacing either artifact: yank the bad registry
version if necessary, preserve evidence, and issue a new version.

Configure a repository/environment-bound crates.io trusted publisher for later
versions only after crate ownership exists. Native `cargo install
enjoyable-mcp-sync --version '=0.1.0' --locked` plus the installed restore smoke
must pass on all six supported OS/CPU hosts.

## Source-building Homebrew publication

The attested `mcp-sync.rb` points to the immutable GitHub Release `.crate` and
pins its SHA-256. Copy that file byte-for-byte to
`EnjoyableWork/homebrew-tap/Formula/mcp-sync.rb`; do not regenerate or hand-edit
it after the release is immutable. It declares Rust only as a build dependency
and installs through Homebrew's locked standard Cargo arguments.

Require Homebrew style, strict online audit, formula test, explicit
`--build-from-source` installation, and the installed restore journey on native
ARM64 and x64 macOS and GNU/Linux hosts. This is not a bottle and does not
claim a project-issued or notarized macOS executable.

After Cargo and Homebrew are public, dispatch
`.github/workflows/market-release-channels.yml` with version `0.1.0`. The
read-only, credential-free workflow requires the immutable seven-file release,
all attestations, exact crates.io `.crate` bytes, exact tap formula bytes,
native GitHub Linux archive smokes, six native Cargo installs, and four native
Homebrew source installs.

Record the immutable release, registry record, tap commit, and every native
smoke run in `PROJECT.md`. `MCP-029` and M2 remain open until all represented
paths install the same protected version.

## Failure and correction

Do not publish a draft when a package hash, formula, SBOM, attestation,
checksum, or installed smoke fails. A draft can be repaired and reverified
because it is not public or immutable. Once published, never delete or replace
an asset, move the tag, or overwrite package metadata. Correct a release defect
with a new version and preserve the evidence explaining the superseding
release.
