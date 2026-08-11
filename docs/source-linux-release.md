# Source and GNU/Linux release runbook

This is the operational contract for `MCP-028` and `MCP-029`. `MCP-028`
proves the path without creating a tag, GitHub Release, crates.io version, or
Homebrew commit. `MCP-029` is the separately authorized publication step.
The funded Apple/Windows pipeline remains available in the
[signed-native runbook](release.md).

## Fixed `v0.1.0` identity and boundary

The initial zero-cost version is `0.1.0`, tagged `v0.1.0`, from
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
`.github/workflows/source-linux-release.yml`. `SHA256SUMS` covers the other six
files exactly. There is no project-issued macOS or Windows binary in this
release and there is no WinGet package. macOS and Windows users build the exact
Cargo package locally; Homebrew builds that same package on macOS and
GNU/Linux.

## End-to-end release sequence

A release selects an already committed and tested state; code is not committed
"onto" a release. The protected sequence is:

1. Finish the version, release notes, and release automation on a branch, then
   merge that branch to `main`.
2. Let CI, the source and GNU/Linux release preflight, and the retained signed
   six-target preflight prove the exact `main` commit.
3. Run `scripts/verify-release-repository-controls.sh` for the exact `main`
   commit, then dispatch `.github/workflows/release-authorize.yml` from `main`
   with `confirm_repository_controls=true`. The owner reviews and approves its
   `release-control` deployment. This records authority but creates no tag and
   publishes nothing.
4. Create one annotated `v0.1.0` tag that points to that exact commit and push
   only the tag. The tag is the immutable version marker.
5. The tag starts `.github/workflows/source-linux-release.yml`. Immediately
   before approving its protected `release` deployment, rerun the operator-side
   repository-control verifier for the tagged commit. After approval, GitHub
   requires the successful exact-commit authorization, builds, attests,
   verifies, and publishes the immutable GitHub Release.
6. From the exact tag, compare the local Cargo package with the immutable
   release asset, perform Cargo's publication dry run, publish to crates.io,
   and verify that crates.io serves identical bytes.
7. Dispatch `.github/workflows/source-linux-release-publish-homebrew.yml` from
   the tag. After another `release` approval, it copies the exact attested
   formula to the organization tap with the tap-scoped deploy key. The one-time
   `v0.1.0` recovery exception below applies only when the immutable tag's
   original workflow definition cannot complete this step.
8. Dispatch `.github/workflows/source-linux-release-channels.yml`. This final
   read-only workflow installs from every public channel represented by the
   release and exercises recovery on every supported native host.
9. Record the immutable release, registry, tap, and smoke evidence in
   `PROJECT.md`; only then mark `MCP-029` and M2 complete.

The separate approvals are intentional. They keep tag creation, GitHub Release
publication, crates.io publication, and tap mutation independently reviewable
and prevent one broad credential from publishing everywhere.

## Trigger separation and repository controls

Before a stable tag exists, verify all of the following:

- GitHub Release immutability is enabled for `EnjoyableWork/mcp-sync`.
- The active `Protect stable release tags` ruleset selects `refs/tags/v*` and
  restricts creation, update, and deletion to the recorded emergency bypass.
- The `release` environment requires review and permits only `v*` tag
  deployments.
- The `release-control` environment requires review and permits only `main`
  deployments.
- The exact current `main` commit has green CI, green
  `Source and GNU/Linux release preflight`, and green retained six-target
  `Release preflight` runs.
- No interactive administration token or Apple/Microsoft signing credential is
  present in the source and GNU/Linux workflow.

GitHub's immutable-release setting endpoint requires repository
administration read access, which the standard Actions `GITHUB_TOKEN` does not
provide. Keep that credential boundary outside Actions. From a clean checkout
whose `HEAD` is the exact current `main` commit, run:

```bash
scripts/verify-release-repository-controls.sh "$(git rev-parse HEAD)"
```

The command uses the controlled operator host's existing authenticated `gh`
session to require the exact current `main` SHA, enabled release immutability,
the public stable-tag ruleset, and the exact `main` / `v*` policies plus required
reviewer on both protected environments. It reads no secret values. A passing
check authorizes dispatch with `confirm_repository_controls=true`; a failed or
stale check requires denying the pending deployment. Run it once immediately
before the `release-control` authorization dispatch and again immediately
before approving the tag-triggered `release` deployment.

Actions independently verifies the public ruleset without a credential. The
tag-triggered publisher also requires a successful protected authorization run
for its exact commit, and publication is not successful unless the public
release reports `immutable: true` and its release attestation verifies.

A push of the exact annotated `v0.1.0` tag can invoke only
`.github/workflows/source-linux-release.yml`. The funded
`.github/workflows/release.yml` has no tag trigger; it requires manual dispatch
on a selected stable tag and `confirm_funded_signing=true`. Do not weaken this
separation by adding a broad `v*` push trigger to the funded workflow.

## Non-publishing preflight

`Source and GNU/Linux release preflight` runs on pull requests, pushes to
`main`, and manual dispatch without secrets or write permissions. It must prove
all of the following before `MCP-028` is complete:

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
6. `scripts/verify-source-linux-release-assets.sh` and
   `scripts/verify-published-source-linux-release.sh` accept the exact dry-run
   payload and reject an extra macOS, Windows, or unrelated file.
7. The retained `Release preflight` continues to build and smoke all six
   unsigned test artifacts without credentials, proving that the future signed
   pipeline has not decayed.

The source and GNU/Linux preflight also executes the public stable-tag ruleset
verifier without a credential, proving the same API path used by protected
authorization and publication jobs.

The preflight artifacts have one-day retention and are not releases. They must
not be described or shared as supported project-issued macOS or Windows
binaries.

## Human authority required for `MCP-029`

No Apple Developer Program or Windows signing purchase is required. The owner
must still:

1. sign in to crates.io through the intended publisher account, verify its
   email address, recheck that `enjoyable-mcp-sync` remains available, create a
   short-lived first-publication token with only the `publish-new` endpoint
   scope and exact crate-name pattern `enjoyable-mcp-sync`, and store it only
   in Cargo's credential store on the controlled publisher host. crates.io
   evaluates crate patterns when the token is used, so the exact future crate
   may be scoped before it exists. Do not add `publish-update`, ownership,
   yank, or trusted-publisher administration to this one-use credential;
2. ensure public repository `EnjoyableWork/homebrew-tap` exists and give the
   release identity only the write access needed for `Formula/mcp-sync.rb`.
   Store its repository-scoped write deploy key as
   `HOMEBREW_TAP_DEPLOY_KEY` in the protected `release` environment; and
3. run the operator-side repository-control verifier for the exact current
   `main` commit, dispatch `.github/workflows/release-authorize.yml` from that
   commit with `confirm_repository_controls=true`, review its evidence, then
   deliberately create and push the one annotated `v0.1.0` tag it authorizes.

Never place a token value in a command argument, ticket, tracked file, or
workflow log. Revoke every first-publication token after the published registry
bytes are verified.

## GitHub Release publication

The `v0.1.0` tag invokes the protected source and GNU/Linux workflow. Approve
its `release` deployment only after rerunning the operator-side repository
control check for the tagged commit. The workflow revalidates the tag, current
`main` commit, package version, successful exact-commit authorization, public
tag ruleset, registry identity, and existing release state. It then:

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

## Historical Cargo first publication

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

Native `cargo install enjoyable-mcp-sync --version '=0.1.0' --locked` plus the
installed restore smoke must pass on all six supported OS/CPU hosts. The
first-publication token was revoked and removed; this section is historical
evidence and is not an authorized procedure for another version.

## Cargo publication after `0.1.0`

The historical first-publication token is not an authorized path for another
version. `.github/workflows/cargo-publish.yml` is the one Cargo publisher for
both source/GNU/Linux and funded immutable releases. It uses
[crates.io Trusted Publishing](https://crates.io/docs/trusted-publishing)
with this exact identity:

| Trusted-publisher field | Required value |
| --- | --- |
| GitHub owner | `EnjoyableWork` |
| GitHub repository | `mcp-sync` |
| Workflow filename | `cargo-publish.yml` |
| GitHub environment | `release` |

The official `rust-lang/crates-io-auth-action` is reviewed and pinned to commit
`c6f97d42243bad5fab37ca0427f495c86d5b1a18` (`v1.0.5`). Only the `publish`
job enters the protected `release` environment and requests `id-token: write`.
Its temporary token is masked, consumed only as `CARGO_REGISTRY_TOKEN` by the
single `cargo publish --locked` step, and revoked by the action's post-job
handler. The repository and environment contain no crates.io credential, and
there is no API-token fallback.

Before configuring the publisher, inspect crates.io **Account Settings → API
Tokens** and require the empty-token state. Do not create a token for setup,
rehearsal, recovery, or fallback. Recheck the same zero-token state after the
authorization-only rehearsal. Record only the aggregate zero-token result;
never copy or persist account-session data.

Before a real publication, configure the exact trusted publisher above while
signed in to crates.io. Do not create an API token. The `MCP-039` foundation
rehearsal targets the existing immutable `v0.1.0` tag without creating or
moving a ref. Create one GitHub deployment with this exact request:

```sh
gh api --method POST repos/EnjoyableWork/mcp-sync/deployments \
  --input - <<'JSON'
{
  "ref": "v0.1.0",
  "task": "mcp-sync:cargo-publish-authorization",
  "auto_merge": false,
  "required_contexts": [],
  "payload": {
    "contract": "MCP-039",
    "version": "0.1.0",
    "tag": "v0.1.0",
    "release_kind": "source-linux",
    "mode": "authorization-only"
  },
  "environment": "release",
  "description": "MCP-039 authorization-only rehearsal",
  "transient_environment": false,
  "production_environment": true
}
JSON
```

GitHub loads the `deployment` event workflow from protected `main` while the
deployment's `GITHUB_REF` remains the existing tag, so the tag-only `release`
environment contract stays intact. The unprivileged validator accepts only the
task and payload above; any other deployment event skips the protected path.
Review the pending `release` job and approve it only after the repository-control
verifier still passes for exact `main`. The workflow verifies the immutable
seven-asset release, its release attestation, the `.crate` provenance, and two
deterministic local packages before requesting OIDC. In authorization-only mode
it never runs `cargo publish`, re-reads crates.io to require that unyanked
`0.1.0` remains the sole version, and then relies on successful action cleanup
to revoke the temporary credential.

After that rehearsal succeeds, enable **Require trusted publishing for all new versions**
on `enjoyable-mcp-sync`. Read back the authenticated trusted-publisher
list from
`/api/v1/trusted_publishing/github_configs?crate=enjoyable-mcp-sync` and the
public `trustpub_only: true` crate field from
`/api/v1/crates/enjoyable-mcp-sync`. Save the first response only to a temporary
regular JSON file, then validate both readbacks and the GitHub controls without
printing identities or credential material:

```sh
./scripts/verify-cargo-publishing-controls.sh \
  <exact-current-main-commit> \
  <temporary-trusted-publisher-readback.json>
```

Delete the temporary response after the verifier passes.

For every later version, first finish and verify the protected immutable GitHub
Release with an attested `enjoyable-mcp-sync-{version}.crate` asset. Dispatch
`cargo-publish.yml` on that exact existing tag with explicit `version`, `tag`,
the owning `release_kind` (`source-linux` or `funded`), and `mode=publish`.
The unprivileged job rejects branches, mismatched tags, unsupported producers,
unrecognized events, and any attempt to republish `0.1.0` before environment
access. The protected job then reproduces and compares local and release
`.crate` bytes, performs a credential-free `cargo publish --dry-run --locked`,
requires the public trusted-publishing-only flag still to be enabled, publishes
explicitly to `crates-io` through short-lived OIDC, downloads and compares the
registry bytes, and gates success on native Cargo install and recovery journeys
for all six supported OS/CPU hosts. Any identity, authorization, release,
attestation, determinism, publication, registry, or native-smoke failure stops
the workflow.

## Source-building Homebrew publication

The attested `mcp-sync.rb` points to the immutable GitHub Release `.crate` and
pins its SHA-256. Copy that file byte-for-byte to
`EnjoyableWork/homebrew-tap/Formula/mcp-sync.rb`; do not regenerate or hand-edit
it after the release is immutable. It declares Rust only as a build dependency
and installs through Homebrew's locked standard Cargo arguments.

After the exact crates.io package is public and byte-equal to the immutable
release asset, dispatch
`.github/workflows/source-linux-release-publish-homebrew.yml` from tag `v0.1.0`
with version `0.1.0`. The protected job rechecks the immutable release, every
attestation, and the exact crates.io bytes before using the tap-scoped deploy
key. It creates only `Formula/mcp-sync.rb`, refuses to replace different
published bytes, and accepts an already-identical formula as a no-op. It has no
write permission to `EnjoyableWork/mcp-sync` and does not publish Cargo, WinGet,
or a binary.

### One-time protected `v0.1.0` publisher recovery

The first `v0.1.0` Homebrew dispatch failed before reading the tap deploy key
or mutating the tap because its crates.io package download did not identify its
HTTP client and crates.io returned `403`. The immutable release and registry
package remain byte-identical. Rerunning that frozen workflow definition would
repeat the same request, while moving `v0.1.0` or replacing an asset is
forbidden.

Use this exact recovery path once:

1. Merge the request-header correction through review and require successful
   exact-`main` CI, source and GNU/Linux preflight, and retained release
   preflight runs.
2. Run `scripts/verify-release-repository-controls.sh` for that exact current
   `main` commit. Reconfirm that `v0.1.0` remains immutable and attested, the
   crates.io package is not yanked and is byte-equal, and the tap formula is
   absent.
3. After separate owner authorization, create the annotated operational tag
   `v-mcp-029-homebrew-recovery-1` at that exact current `main` commit and push
   only that tag. It is protected release-automation evidence, not a product
   version: do not create a GitHub Release or Cargo version for it, and never
   move or delete it.
4. Dispatch `.github/workflows/source-linux-release-publish-homebrew.yml` from
   `v-mcp-029-homebrew-recovery-1` with version `0.1.0`, then review and approve
   its existing `release` environment deployment.

The recovery path accepts only that exact annotated operational tag, requires
it to resolve to the current `main` commit, requires all three exact-main gates
to pass, and rechecks the public stable-tag ruleset. It then checks out only
canonical `v0.1.0`, verifies its immutable release, attestations, and crates.io
bytes, and can create only the absent byte-identical tap formula. It neither
weakens the environment's tag-only policy nor permits a second product release.
Future versions use their canonical release tag and must send an explicit
project user agent on every crates.io request.

Require Homebrew style, strict online audit, formula test, explicit
`--build-from-source` installation, and the installed restore journey on native
ARM64 and x64 macOS and GNU/Linux hosts. This is not a bottle and does not
claim a project-issued or notarized macOS executable.

After Cargo and Homebrew are public, dispatch
`.github/workflows/source-linux-release-channels.yml` with version `0.1.0`. The
read-only, credential-free workflow requires the immutable seven-file release,
all attestations, exact crates.io `.crate` bytes, exact tap formula bytes,
native GitHub Linux archive smokes, six native Cargo installs, and four native
Homebrew source installs.

Record the immutable release, registry record, tap commit, and every native
smoke run in `PROJECT.md`. `MCP-029` and M2 remain open until all represented
paths install the same protected version.

## Failure and correction

Do not authorize or approve a deployment when the operator-side repository
control check fails or targets a stale commit. Do not publish a draft when a
package hash, formula, SBOM, attestation, checksum, or installed smoke fails. A
draft can be repaired and reverified because it is not public or immutable.
Once published, never delete or replace an asset, move the tag, or overwrite
package metadata. Correct a release defect with a new version and preserve the
evidence explaining the superseding release.
