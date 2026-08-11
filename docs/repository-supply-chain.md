# Repository and distribution supply-chain controls

This document is the `MCP-033` contract for dependency maintenance, GitHub
Actions trust boundaries, authenticated distribution, and source-repository
artifact policy. It covers `EnjoyableWork/mcp-sync` and the existing `v0.1.0`
Cargo and Homebrew distribution paths. It does not modify that immutable
release, activate the funded signed-native pipeline, or claim a complete
assurance baseline or certification.

## Dependency maintenance

`.github/dependabot.yml` requests two weekly grouped version-update pull
requests:

- one Cargo group covering direct and transitive dependencies in `/`; and
- one GitHub Actions group covering workflows in `/`.

Dependabot security updates remain enabled independently. Bot-authored changes
use the same protected pull-request path and required checks as any other
change. A grouped update is a review prompt, not an automatic approval: review
the manifest or workflow diff, regenerated `Cargo.lock`, action update hints,
upstream release notes, and CI before merging.

## Immutable Actions references and selected-action policy

Every action reference committed under `.github/workflows` uses a full
40-character commit SHA followed by a human-readable update hint. The
credential-free `scripts/verify-workflow-supply-chain.sh` gate rejects a
mutable or unhinted reference, an action outside `.github/actions-policy.json`,
a checkout that persists credentials, a privileged pull-request trigger, or a
GitHub expression interpolated directly into a shell command. Its optional
`--verify-upstreams` mode proves that each hint resolves to the recorded SHA.

The reviewed inventory, extended for `MCP-039` on 2026-08-10, is:

| Action | Reviewed SHA | Update hint | Purpose |
| --- | --- | --- | --- |
| `actions/checkout` | `3d3c42e5aac5ba805825da76410c181273ba90b1` | `v7.0.1` | Source checkout with persisted credentials disabled |
| `actions/download-artifact` | `3e5f45b2cfb9172054b4087a40e8e0b5a5461e7c` | `v8.0.1` | Same-run preflight and release artifact collection |
| `actions/upload-artifact` | `043fb46d1a93c77aae656e7c1c64a875d1fc6a0a` | `v7.0.1` | Short-lived preflight and controlled release staging |
| `actions/attest-build-provenance` | `4d101475d8b20a2381f78447822ac1eab6504dd8` | `v4.2.2` | GitHub build provenance |
| `actions/attest` | `508db95dd578ae2727ebd6217d5ba78e4fbda05d` | `v4.2.1` | Transitive action pinned by the reviewed provenance action |
| `EmbarkStudios/cargo-deny-action` | `3c6349835b2b7b196a839186cb8b78e02f7b5f25` | `v2.1.1` | Advisory, license, ban, and source policy |
| `anchore/sbom-action` | `e22c389904149dbc22b58101806040fa8d37a610` | `v0.24.0` | SPDX SBOM generation |
| `Homebrew/actions/setup-homebrew` | `fd832223f9f99ebf0244dd20658680e5d4aca049` | `2026.08.03.2` | Native Homebrew preflight and channel setup |
| `Azure/login` | `f5d393ae46f8fde4be8b75f32e3fc50e654ad0ca` | `v3.0.1` | Dormant funded Windows signing authentication |
| `Azure/artifact-signing-action` | `c7ab2a863ab5f9a846ddb8265964877ef296ee82` | `v2.0.0` | Dormant funded Windows Public Trust signing |
| `rust-lang/crates-io-auth-action` | `c6f97d42243bad5fab37ca0427f495c86d5b1a18` | `v1.0.5` | Official short-lived crates.io Trusted Publishing authorization and automatic revocation |

The `MCP-039` review verified the official `rust-lang` repository and signed
`v1.0.5` commit, its Node 24 action definition, crates.io-only default
audience, masked token output, saved post-job state, and unconditional
revocation request. The workflow consumes that output only as
`CARGO_REGISTRY_TOKEN` on the single publish step; neither the OIDC assertion
nor temporary registry credential is logged, persisted, or promoted to a job
output.

The live repository policy allows only those exact action repositories or
subpaths, disables the broad GitHub-owned and verified-creator allowances, and
requires full-length SHA references. The `@*` suffixes in the selected-action
API are repository selectors, not permission to use mutable refs: the separate
SHA requirement rejects tags and branches. Default workflow permissions remain
read-only and workflows cannot approve pull requests.

GitHub-managed CodeQL default setup for Rust and Actions is deliberately not a
tracked workflow or selected-action entry. GitHub documents that default setup
is not affected by policies restricting public actions or reusable workflows;
the authenticated verifier separately requires that setup to remain enabled.
If the repository moves to CodeQL advanced setup, its committed actions enter
this inventory and SHA gate before the change merges.

## Untrusted code and metadata boundary

The workflows triggered by `pull_request` are `ci.yml`,
`release-preflight.yml`, and `source-linux-release-preflight.yml`. They have
read-only repository permission, receive no repository or environment secret,
request no OIDC or write authority, and cannot publish or consume protected
release assets. Their one-day preflight artifacts are unsigned test output and
are never inputs to a privileged publisher. Checkout credentials are disabled.

No workflow uses `pull_request_target` or `workflow_run`. Fork and other
untrusted pull requests therefore cannot combine an elevated event context
with contributor-controlled code. The repository's first-time-contributor
approval policy controls when a public-fork workflow starts; approval does not
grant secrets or a write token.

Untrusted pull-request titles, branch names, commit messages, and author data
are not interpolated into shell commands. Workflow expressions enter shell
steps only through environment variables. Static matrix values are bounded by
the committed workflow. Manual release inputs are checked for exact accepted
values before use. In particular, the Homebrew recovery workflow now validates
the exact `0.1.0` request and protected tag or recovery ref in an unprivileged
job before entering the `release` environment, checking out release code, or
making its tap-scoped deploy key available.

The Cargo request validator similarly accepts only an explicit, fully matched
version, annotated tag, release kind, and mode before the protected job can
start. Its `deployment` trigger is restricted to the exact MCP-039
authorization-only rehearsal against the existing immutable `v0.1.0` tag; it
cannot select publication mode, create another version, or weaken the same
repository, ref, and release-environment checks.

Privileged publishing remains limited to explicit tag or manual workflows,
least-privilege job permissions, protected environments, exact repository and
ref checks, and the existing release authorization contract. No pull-request
artifact crosses into those jobs.

## Authenticated distribution chain

GitHub Releases remains the canonical immutable channel. For `v0.1.0`:

1. the annotated protected tag identifies one release commit;
2. GitHub's release attestation verifies the immutable release record;
3. every one of the seven release assets has GitHub build provenance bound to
   the exact tag, source commit, and `source-linux-release.yml` signer workflow;
4. `SHA256SUMS` covers the packaged payloads;
5. crates.io is reached over HTTPS and serves a package byte-identical to the
   attested `.crate` release asset; and
6. the organization Homebrew tap is read over HTTPS and exposes a formula
   byte-identical to the attested release formula. Its protected publisher uses
   a tap-only SSH deploy key, a pinned GitHub Ed25519 host key, and strict host
   checking.

The `0.1.0` crate is the only token-published exception: its one-use,
scope-minimized first-publication token was revoked and removed.
Cargo publication after `0.1.0` uses one protected
`.github/workflows/cargo-publish.yml` path. Its required crates.io Trusted
Publisher identity is exactly `EnjoyableWork/mcp-sync`, workflow filename
`cargo-publish.yml`, and environment `release`; **Require trusted publishing
for all new versions** must remain enabled. Before requesting authorization,
the workflow verifies the annotated tag, immutable GitHub release and
attestation, the attested `.crate`, deterministic local packaging, and exact
local-to-release bytes. Only its protected publish job receives OIDC, and only
the actual `cargo publish` step receives the official action's short-lived,
masked registry credential.

No crates.io API token may be created, stored, or used as fallback.
The owner-side completion audit also requires crates.io **Account Settings →
API Tokens** to remain empty before and after the authorization-only rehearsal;
only the aggregate zero-token result is recorded.

After publication, the workflow waits for crates.io, compares the downloaded
registry `.crate` byte-for-byte with the attested GitHub release asset, and
proves exact-version installation plus recovery output on all six retained
native targets. The authorization-only mode exercises the protected OIDC and
publisher identity without consuming the returned registry credential or
creating a crate version; the fixed MCP-039 rehearsal also re-reads crates.io
and succeeds only while unyanked `0.1.0` remains the sole version.

`scripts/verify-distribution-authentication.sh EnjoyableWork/mcp-sync 0.1.0`
rechecks this chain without publishing or replacing anything. Cargo and
Homebrew consumers still depend on their local TLS and package-manager trust
stores; this evidence establishes the authenticated official path and exact
cross-channel bytes, not the security of every downstream host.

There is no current WinGet distribution and no project-issued macOS or Windows
binary. The funded signing and downstream workflows remain intact and dormant
under `SIDE-006`.

## Generated executable and binary-artifact policy

Generated executables, packages, archives, SBOM output, and other binary build
artifacts must not be committed. Release automation creates them in ignored
temporary or `release-assets` directories and publishes them through the
attested release boundary instead.

`scripts/verify-repository-artifacts.sh` inspects Git's tracked blobs rather
than only the working-tree filenames. It rejects submodule entries, known
generated package or executable extensions, and content identified as binary;
reviewable executable text scripts remain allowed. The CI `Dependency policy`
job runs this gate on every push and pull request. `.gitignore` provides an
earlier local guard, while `scripts/test-repository-artifact-policy.sh` proves
that reviewable text passes and both extension-disguised and extensionless
binary artifacts fail. A future need for a tracked visual or other binary asset
requires a reviewed policy change with a narrow, explainable exception rather
than silently weakening the gate.

## Verification and review

Run the credential-free local gates with:

```sh
./scripts/verify-workflow-supply-chain.sh
./scripts/verify-repository-artifacts.sh
./scripts/test-repository-artifact-policy.sh
./scripts/test-cargo-publish-policy.sh
./scripts/test-cargo-publish-workflow-policy.sh
```

An authenticated repository administrator can revalidate upstream action tags,
the live Actions policy, token and fork settings, CodeQL default setup, merged
Dependabot configuration, the repository artifact boundary, and the published
distribution chain without reading secret values:

```sh
./scripts/verify-repository-supply-chain-controls.sh \
  EnjoyableWork/mcp-sync 0.1.0
```

Re-run the operator verifier after an action, workflow, dependency-update
policy, GitHub Actions policy, fork policy, CodeQL setup, release pipeline,
official distribution channel, or repository-artifact policy change.

After the protected Cargo workflow is present on exact `main`, save the
non-sensitive crates.io Trusted Publisher readback to a temporary regular JSON
file and verify the complete live control set with:

```sh
./scripts/verify-cargo-publishing-controls.sh \
  <exact-main-commit> <trusted-publisher-json-file>
```

The verifier requires a clean checkout of that exact `main` commit, compares
the live workflow bytes, confirms its active GitHub identity and existing
release protections, rejects Cargo or registry credential names across the
repository, `release` environment, and organization Actions stores, checks the
public `trustpub_only` state, and accepts only one exact Trusted Publisher
record. It reads no credential value.

## Live control record

Authenticated read-only API verification on 2026-08-09 confirms that
`EnjoyableWork/mcp-sync` has Actions enabled with `allowed_actions` set to
`selected` and `sha_pinning_required` set to `true`. The selected-action state
matches `.github/actions-policy.json`: both broad allowance flags are `false`
and only the ten reviewed patterns are present. Default workflow permission is
`read`, workflows cannot approve pull requests, public-fork execution requires
approval for first-time contributors, and CodeQL default setup remains enabled
for Rust and Actions.

GitHub's repository API rejects preloading selected patterns while
`allowed_actions` is `all`. Activation therefore set selected/full-SHA mode and
then immediately installed the exact inventory. The brief intermediate state
allowed only GitHub-owned actions with full SHAs, so it was narrower than the
starting state and did not expose another action, credential, or asset. Final
state was read back and compared structurally before CI was retriggered.

Protected [PR #38](https://github.com/EnjoyableWork/mcp-sync/pull/38) merged
normally as exact `main` commit
[`d934d38`](https://github.com/EnjoyableWork/mcp-sync/commit/d934d38edd40a4c38bacfd79994221548882e79a).
That commit passes [CI](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31304197031),
[CodeQL](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31304196954),
the retained [six-target release preflight](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31304196996),
the complete [source and GNU/Linux preflight](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31304197063),
and GitHub's two accepted Dependabot configuration evaluations for
[Cargo](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31304200669)
and [GitHub Actions](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31304199968).
The exact-main operator verifier then passed all live settings, upstream action,
tracked-artifact, and authenticated-distribution checks without inspecting a
secret value.

Final [PR #39](https://github.com/EnjoyableWork/mcp-sync/pull/39) closes a
completion-audit gap in future-drift detection: a pull-request workflow is
rejected for any write permission, dot or bracket secret access, a reusable
workflow secret map, protected-environment access, or cross-run artifact
credentials. This is broader than enumerating only currently used permission
names and keeps the read-only, secretless fork boundary true as workflows grow.

An intentionally invalid `1.2.3` manual request provides the negative-path
record. Its [workflow run](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31304302717)
failed in the unprivileged validator, skipped the protected publish job, and
created no `release` deployment. The ordered set of four pre-existing release
deployment identifiers had the same SHA-256 fingerprint
`bd51aa991323f0c8c8c51c14d13a739a183a265fb9aad4860d342b9c9233afb2`
before and after the exercise. No release asset, tag, crate, formula, protected
credential, or environment policy was changed.

## OpenSSF OSPS Baseline evidence mapping

This is evidence for five Level 1 controls in the
[OpenSSF OSPS Baseline `v2026.02.19`](https://baseline.openssf.org/versions/2026-02-19.html),
not a project-wide conformance or certification claim.

| Control | `MCP-033` evidence |
| --- | --- |
| `OSPS-BR-01.01` | Untrusted workflow metadata is either unused in shell commands or crosses a typed environment boundary and is validated before security-sensitive use; the static gate rejects direct expression interpolation. |
| `OSPS-BR-01.03` | Pull-request and fork code receives read-only permissions, no secrets, no OIDC or release authority, and no protected artifacts; privileged publishers have no untrusted trigger. |
| `OSPS-BR-03.02` | The immutable GitHub release and build attestations, HTTPS Cargo and Homebrew reads, exact cross-channel bytes, and pinned SSH publisher path form the authenticated official distribution chain. |
| `OSPS-QA-05.01` | CI and the tracked-blob verifier reject generated executable and package artifacts from version control. |
| `OSPS-QA-05.02` | CI rejects unreviewable binary blobs and submodule entries; its focused failure test proves both content- and extension-based rejection. |
