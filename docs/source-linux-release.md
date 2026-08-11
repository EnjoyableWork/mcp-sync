# Source and GNU/Linux release runbook

This is the current operational contract for the zero-cost mcp-sync release
path. It publishes one immutable GitHub Release, source-based Cargo and
Homebrew channels, and native GNU/Linux ARM64 and x64 archives. It publishes no
project-issued macOS or Windows binary and no WinGet package. The dormant
funded Apple/Windows path remains separate in the
[signed-native runbook](release.md).

## Identity and immutable artifact boundary

The repository is `EnjoyableWork/mcp-sync`, the Cargo package is
`enjoyable-mcp-sync`, the Homebrew formula is
`EnjoyableWork/homebrew-tap/Formula/mcp-sync.rb`, and every channel installs an
executable named `mcp-sync`.

A source/GNU/Linux release for `{version}` uses annotated tag `v{version}` and
contains exactly seven uploaded assets:

```text
SHA256SUMS
enjoyable-mcp-sync-{version}.crate
mcp-sync.rb
mcp-sync-v{version}-aarch64-unknown-linux-gnu.spdx.json
mcp-sync-v{version}-aarch64-unknown-linux-gnu.tar.gz
mcp-sync-v{version}-x86_64-unknown-linux-gnu.spdx.json
mcp-sync-v{version}-x86_64-unknown-linux-gnu.tar.gz
```

`.github/workflows/source-linux-release.yml` attests every asset.
`SHA256SUMS` covers the other six files exactly. A published tag, release, or
asset is never moved, replaced, or deleted to correct a defect; publish a newer
version instead.

## Stable-version policy

Every release after `0.1.0` must satisfy all of these conditions:

- the tag and Cargo package version are the same canonical stable SemVer, with
  no leading-zero component or prerelease/build suffix;
- the candidate is later than `0.1.0` and is not older than any published
  stable `enjoyable-mcp-sync` version;
- `docs/releases/v{version}.md` is a regular, non-symbolic-link file;
- crates.io still identifies the package with this repository, exposes the
  unyanked `0.1.0` baseline, and reports `trustpub_only: true`; and
- initial tag authorization finds no existing tag, GitHub Release, or Cargo
  version for the candidate.

`scripts/validate-release-version.sh` is the shared policy boundary. Its
`future` mode deliberately permits equality with the newest registry version
only so a partial run can prove and recover an already-published candidate. It
never permits a downgrade. `published` verifies an existing channel version,
and `rehearsal` is fixed to immutable `v0.1.0`.

## Repository and preflight gates

Before authorizing a tag, require all of the following on the exact current
`main` commit:

- CI;
- `Source and GNU/Linux release preflight`;
- the retained six-target `Release preflight`;
- GitHub Release immutability;
- the active `Protect stable release tags` ruleset for `refs/tags/v*`;
- the reviewed `release-control` environment restricted to `main`; and
- the reviewed `release` environment restricted to `v*` tags.

From a clean checkout at exact current `main`, run:

```bash
scripts/verify-release-repository-controls.sh "$(git rev-parse HEAD)"
```

This operator-side command retains repository-administration reads outside
Actions. The workflows independently recheck the public stable-tag ruleset.
Do not approve a deployment using a stale verifier result.

The source/GNU/Linux preflight is credential-free and nonpublishing. It proves
deterministic Cargo packaging and formula generation, native source installs on
all six supported OS/CPU hosts, source-built Homebrew on four Unix hosts, both
native GNU/Linux archives and SPDX 2.3 SBOMs, the exact seven-asset boundary,
and rejection of extra macOS, Windows, or unrelated assets.

## Repeat-release sequence

1. Prepare the version, lockfile, release notes, and automation on a branch.
   Merge normally, then wait for all three exact-main gates above.
2. Run the operator verifier and dispatch
   `.github/workflows/release-authorize.yml` from `main` with
   `confirm_repository_controls=true`. Approve `release-control` only while
   the verifier result remains current. Authorization creates no ref and
   publishes nothing.
3. As the recorded stable-tag bypass actor, create one signed or annotated
   `v{version}` tag at that exact authorized `main` commit and push only the
   tag. The tag push publishes nothing by itself.
4. Choose exactly one GitHub Release producer. For this path, rerun the
   operator verifier and dispatch the source/GNU/Linux workflow on the tag:

   ```bash
   gh workflow run source-linux-release.yml \
     --repo EnjoyableWork/mcp-sync \
     --ref "v{version}" \
     -f version="{version}" \
     -f mode=publish
   ```

   Do not also dispatch the funded signed workflow for the same tag. Approve
   the `release` deployment only after confirming the exact tag and commit.
5. Require the publisher to finish with an immutable, attested, exact
   seven-asset GitHub Release. GitHub is intentionally first: the candidate
   Cargo version is not a prerequisite.
6. Dispatch the dedicated Cargo publisher on the same tag:

   ```bash
   gh workflow run cargo-publish.yml \
     --repo EnjoyableWork/mcp-sync \
     --ref "v{version}" \
     -f version="{version}" \
     -f tag="v{version}" \
     -f release_kind=source-linux \
     -f mode=publish
   ```

7. After Cargo and registry bytes converge, dispatch the Homebrew publisher on
   the same tag:

   ```bash
   gh workflow run source-linux-release-publish-homebrew.yml \
     --repo EnjoyableWork/mcp-sync \
     --ref "v{version}" \
     -f version="{version}"
   ```

8. Run the credential-free public-channel verifier from current protected
   `main`:

   ```bash
   gh workflow run source-linux-release-channels.yml \
     --repo EnjoyableWork/mcp-sync \
     --ref main \
     -f version="{version}"
   ```

9. Record the immutable release, registry, tap commit, and complete native
   smoke evidence in `PROJECT.md`.

The four mutating workflows—source/GNU/Linux release, funded signed release,
Cargo publication, and Homebrew publication—share one non-cancelling
`mcp-sync-release` concurrency group. Do not bypass that serialization with a
manual channel mutation.

## GitHub Release construction and recovery

The source/GNU/Linux producer validates the annotated protected tag, exact
current-main commit, successful exact-commit authorization, release notes,
registry identity, stable-version inventory, and public ruleset before it
builds. It then packages the Cargo source and Homebrew formula twice, builds
and smokes both native GNU/Linux archives, generates target SPDX SBOMs,
attests all six payload files plus `SHA256SUMS`, verifies a downloaded draft,
publishes it, requires immutable state and release attestation, and verifies
the immutable bytes again.

An interrupted draft may be repaired and reverified. If the candidate Cargo
version already exists, the producer proceeds only when the corresponding
GitHub Release is already immutable; after rebuilding, it downloads the
registry `.crate` and requires exact equality with the immutable release
asset. A yanked or mismatched candidate fails closed.

## Cargo Trusted Publishing and retry behavior

`.github/workflows/cargo-publish.yml` is the only Cargo publisher for both
source/GNU/Linux and funded releases after `0.1.0`. crates.io Trusted
Publishing is bound exactly as follows:

| Trusted-publisher field | Required value |
| --- | --- |
| GitHub owner | `EnjoyableWork` |
| GitHub repository | `mcp-sync` |
| Workflow filename | `cargo-publish.yml` |
| GitHub environment | `release` |

The protected publish job alone receives `id-token: write`. The reviewed
`rust-lang/crates-io-auth-action` obtains a short-lived credential, exposes it
only as `CARGO_REGISTRY_TOKEN` to the single `cargo publish --locked` step, and
revokes it in its post-job handler. **Require trusted publishing for all new versions**
remains enabled. There is no API-token fallback, and no crates.io token may be
created, requested, or stored. Operator revalidation uses
`scripts/verify-cargo-publishing-controls.sh` and records only the aggregate zero-token result,
never credential or account-session material.

Before publication, the workflow verifies the immutable release and
attestation, reproduces the package twice, compares local and release bytes,
checks the complete stable registry inventory, and performs a credential-free
dry run. After a new publish call, it waits for an unyanked version, downloads
it, requires byte equality, and then runs native Cargo install/recovery smokes
on all six supported hosts.

Rerunning after crates.io accepted the candidate is safe. The workflow first
requires the existing registry `.crate` to be byte-identical to the immutable
GitHub asset. If it is exact, it records recovery, requests no OIDC credential,
makes no publish call, and continues through the same registry and native
installation verification. A byte mismatch, yank, older candidate, missing
immutable release, or disabled trusted-only setting stops the run.

## Homebrew handoff and public verification

The release's attested `mcp-sync.rb` points to its immutable `.crate` and pins
the exact SHA-256. The Homebrew workflow validates its canonical protected tag
before entering the `release` environment, rechecks the immutable release,
every attestation, the registry identity, and exact Cargo bytes, then uses only
the existing tap-scoped `HOMEBREW_TAP_DEPLOY_KEY`.

An identical public formula is a no-op. A later formula may replace an older
one only when the existing tap file has a canonical release URL, is
byte-identical to the formula in its own prior immutable GitHub Release, and
the candidate version is strictly newer. The workflow refuses a downgrade,
same-version byte change, malformed formula, non-fast-forward tap race, or any
other file change. It then verifies the exact public formula bytes.

The final source/GNU/Linux channel workflow is read-only and credential-free.
It runs from exact protected `main`, checks out the requested immutable tag,
verifies all seven assets and attestations, compares crates.io and tap bytes,
smokes both GitHub GNU/Linux archives, performs six native Cargo installs, and
performs four native source-building Homebrew installs.

## Nonpublishing live rehearsal

The repeat-release rehearsal reuses the already public immutable `v0.1.0`
evidence and creates no tag, release, crate, formula, or credential:

```bash
gh workflow run source-linux-release.yml \
  --repo EnjoyableWork/mcp-sync \
  --ref main \
  -f version=0.1.0 \
  -f mode=rehearse
```

It requires exact current protected `main`, verifies the historical release
and all attestations, compares the public Cargo and Homebrew bytes, proves
synthetic mismatches are rejected, and runs the positive/negative stable
version policy cases.

The separate Cargo `authorization-only` rehearsal remains fixed to
`v0.1.0`. It may run from exact protected `main` only while the operator has
temporarily added the single exact `main` branch deployment policy to the
`release` environment as documented by
`scripts/verify-release-repository-controls.sh --allow-mcp-039-main-rehearsal`.
Remove that temporary policy immediately
afterward and re-run the strict verifier. The rehearsal compares the registry
version inventory before and after OIDC authorization and never invokes
`cargo publish`.

## Historical `v0.1.0` record

`v0.1.0` was the one first-publication exception. Its scope-minimized
`publish-new` API token was revoked server-side and removed locally after the
registry bytes were verified. That history is not an authorized procedure for
another version; never create a replacement token.

The first Homebrew attempt failed before credential use or tap mutation. The
retained annotated operational tag `v-mcp-029-homebrew-recovery-1` allowed the
corrected workflow to check out canonical `v0.1.0` and publish only its absent,
byte-identical formula. That tag is not a product release and is never moved or
deleted. Its exception remains accepted only for version `0.1.0`; every later
Homebrew publication uses its canonical `v{version}` tag.

## Failure and correction

Do not approve stale authorization, publish a draft with incomplete evidence,
or continue after any version, hash, attestation, SBOM, checksum, registry,
formula, or native-smoke failure. Drafts may be repaired before publication.
Immutable releases and published package versions are evidence, not mutable
deployment slots. Preserve a failed or partial run, correct the cause, and
either retry the exact byte-safe step or issue a newer version.
