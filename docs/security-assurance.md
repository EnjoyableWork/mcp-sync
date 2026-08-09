# Security assurance

This document is the public, durable `MCP-035` security-assurance contract for
`mcp-sync`. It records a point-in-time project self-assessment, the evidence
boundary behind it, the exact artifact-level provenance statement, and the
conditions that require reassessment or removal of public claims.

## Assurance statement

On 2026-08-09, the project self-assessed all 24 OpenSSF Open Source Project
Security (OSPS) Baseline [`v2026.02.19` Level 1
controls](https://baseline.openssf.org/versions/2026-02-19-checklist.md) as
passing. All 24 controls apply; none is treated as not applicable or satisfied
through an unresolved exception.

This is a maintainer self-assessment, not an independent certification,
regulatory determination, or OpenSSF endorsement. The official BadgeApp
[`baseline-1` assessment](https://www.bestpractices.dev/en/projects/14014/baseline-1)
and dynamic badge are the public proof of the corresponding self-certification:

[![OpenSSF Baseline](https://www.bestpractices.dev/projects/14014/baseline)](https://www.bestpractices.dev/projects/14014)

BadgeApp first recorded `baseline-1` on 2026-08-09 at 21:00:26 UTC. Its public
assessment reports all 24 controls `Met` against criteria version
`v2026.02.19`; the image above is the issuer's dynamic status rather than a
project-created shield.

## Assessment scope

The assessment covers:

- the authoritative public [`EnjoyableWork/mcp-sync`](https://github.com/EnjoyableWork/mcp-sync)
  source repository and its complete public history;
- the in-scope [`EnjoyableWork/homebrew-tap`](https://github.com/EnjoyableWork/homebrew-tap)
  distribution codebase identified by the project inventory;
- GitHub organization, repository, Actions, protected-environment, and private
  recovery practices that directly protect those repositories and the
  `mcp-sync` release path; and
- the immutable [`v0.1.0`](https://github.com/EnjoyableWork/mcp-sync/releases/tag/v0.1.0)
  release plus its represented Cargo and Homebrew distribution copies.

The repository and live-setting evidence was revalidated on 2026-08-09 against
public `main` commit
[`d106f4b`](https://github.com/EnjoyableWork/mcp-sync/commit/d106f4b065dcc0701bf14cce9e9a6899e3c7a4c7)
and the then-current GitHub state. The final `MCP-035` merge and exact-main
badge verification are recorded in [`PROJECT.md`](../PROJECT.md); that later
record does not change the point-in-time basis of the underlying controls.

The assessment does not cover other `EnjoyableWork` products, third-party MCP
clients, user systems, future releases, unpublished macOS or Windows binaries,
the dormant funded signing pipeline, OSPS Level 2 or 3, OpenSSF Best Practices
Passing, NIST SSDF, OpenChain, or any legal or regulatory compliance regime.

## Level 1 control crosswalk

The result in every row is **Pass**. Evidence links point to maintained public
contracts, tests, scripts, or immutable records; private GitHub findings,
identities, authentication factors, credentials, and recovery details are
deliberately excluded.

The root [`.bestpractices.json`](../.bestpractices.json) mirrors these results
as machine-readable BadgeApp automation proposals. BadgeApp treats those
values as suggestions for an authorized human to review; the issued public
assessment, not the proposal file by itself, is the official badge-status
record.

| Control | Result | Evidence and exact reasoning |
| --- | --- | --- |
| `OSPS-AC-01.01` | Pass | The [organization access contract](organization-access-and-recovery.md#authentication-and-collaborator-access-policy) requires organization-wide secure-method MFA before sensitive GitHub access. The same-day private confirmation and non-disclosing live verifier passed without publishing a person or factor. |
| `OSPS-AC-02.01` | Pass | The [collaborator policy](organization-access-and-recovery.md#authentication-and-collaborator-access-policy) sets organization base access to `none` and requires manual, repository-specific least-privilege grants. The non-disclosing live verifier confirmed the policy and aggregate access state. |
| `OSPS-AC-03.01` | Pass | The active [`Protect main`](https://github.com/EnjoyableWork/mcp-sync/rules/20601003) ruleset requires a pull request and strict app-bound checks. The [governance record](repository-governance.md#mcp-030-verification-record) includes a rejected direct-update exercise. |
| `OSPS-AC-03.02` | Pass | The same ruleset blocks deletion, and the [governance record](repository-governance.md#mcp-030-verification-record) includes a rejected deletion exercise plus restored emergency-administration state. |
| `OSPS-BR-01.01` | Pass | The [untrusted metadata boundary](repository-supply-chain.md#untrusted-code-and-metadata-boundary) uses typed environment boundaries and exact input validation. [`verify-workflow-supply-chain.sh`](../scripts/verify-workflow-supply-chain.sh) rejects direct GitHub-expression interpolation into shell commands. |
| `OSPS-BR-01.03` | Pass | Pull-request workflows are read-only and secretless, use no protected environment, OIDC, privileged trigger, or cross-run credentialed artifact, and cannot feed a publisher. The [supply-chain contract](repository-supply-chain.md#untrusted-code-and-metadata-boundary) and static verifier cover every workflow. |
| `OSPS-BR-03.01` | Pass | The [official-channel inventory](project-community-and-licensing.md#official-project-channels) contains only HTTPS public URIs. The public project-contract verifier rejects an unencrypted channel in every maintained channel document. |
| `OSPS-BR-03.02` | Pass | The [authenticated distribution chain](repository-supply-chain.md#authenticated-distribution-chain) combines HTTPS channels, immutable release state, signed GitHub provenance, exact hashes, byte-identical Cargo and Homebrew payloads, and a pinned-host tap publisher. The live distribution verifier passed for `v0.1.0`. |
| `OSPS-BR-07.01` | Pass | The [repository security boundary](repository-security.md#openssf-osps-mapping) combines local secret-file exclusions, provider-pattern secret scanning, push protection, and a non-disclosing zero-open-finding gate. The live repository-security verifier passed. |
| `OSPS-DO-01.01` | Pass | The [README](../README.md) documents installation, configuration, and every basic CLI journey: `init`, `add`, `list`, `test`, `sync --dry-run`, `sync`, and `restore`. The [usage and recovery guide](m1-usage-and-recovery.md) documents safe operation, limits, recovery, and failure behavior in detail. |
| `OSPS-DO-02.01` | Pass | [`CONTRIBUTING.md`](../CONTRIBUTING.md#report-a-defect), [`SUPPORT.md`](../SUPPORT.md), and the public structured bug form explain how to report defects while routing vulnerabilities privately. |
| `OSPS-GV-02.01` | Pass | The [public project contract](project-community-and-licensing.md#public-contribution-and-support-boundary) identifies GitHub Issues and pull-request conversations as the public mechanisms for proposed changes and usage obstacles. |
| `OSPS-GV-03.01` | Pass | [`CONTRIBUTING.md`](../CONTRIBUTING.md) explains issue selection, safety and redaction rules, checkout preparation, required tests, contribution terms, and the protected pull-request process. |
| `OSPS-LE-02.01` | Pass | Source is licensed under the OSI-approved [MIT License](../LICENSE), and [`Cargo.toml`](../Cargo.toml) declares `MIT`. |
| `OSPS-LE-02.02` | Pass | The [release-license record](project-community-and-licensing.md#source-and-release-licensing) and public verifier confirm that the Cargo package and both GNU/Linux software archives carry the same MIT license and the Homebrew formula declares it. |
| `OSPS-LE-03.01` | Pass | The [two-codebase inventory](project-community-and-licensing.md#repository-and-codebase-inventory) and public verifier confirm a root `LICENSE` in both `mcp-sync` and `homebrew-tap`. |
| `OSPS-LE-03.02` | Pass | The immutable tagged source, published crate, and both GNU/Linux software archives each contain exactly one matching `LICENSE` alongside the released software. Packaging and public verification fail if it is absent. |
| `OSPS-QA-01.01` | Pass | The authoritative source is publicly readable at the stable [`https://github.com/EnjoyableWork/mcp-sync`](https://github.com/EnjoyableWork/mcp-sync) URL; live repository metadata confirms it is public and not archived. |
| `OSPS-QA-01.02` | Pass | GitHub exposes the repository's attributable, timestamped [public commit history](https://github.com/EnjoyableWork/mcp-sync/commits/main/). The local Git object history and public commit API retain author, committer, and time metadata. |
| `OSPS-QA-02.01` | Pass | [`Cargo.toml`](../Cargo.toml) enumerates every direct Rust dependency, and the committed [`Cargo.lock`](../Cargo.lock) records the resolved direct and transitive graph. Locked Cargo metadata and dependency-policy checks pass. |
| `OSPS-QA-04.01` | Pass | The [repository and codebase inventory](project-community-and-licensing.md#repository-and-codebase-inventory) names the complete product source and Homebrew distribution codebase, states each role, and explicitly separates unrelated organization repositories. |
| `OSPS-QA-05.01` | Pass | The [artifact policy](repository-supply-chain.md#generated-executable-and-binary-artifact-policy) forbids generated executables and packages in source history. CI runs the tracked-blob verifier, whose focused rejection exercise passed. |
| `OSPS-QA-05.02` | Pass | The same policy and [`verify-repository-artifacts.sh`](../scripts/verify-repository-artifacts.sh) reject unreviewable binary blobs and submodules while allowing reviewable text scripts. Both disguised and extensionless binary rejection cases pass. |
| `OSPS-VM-02.01` | Pass | The root [`SECURITY.md`](../SECURITY.md#security-contacts-and-private-reporting) names the maintainers as security contacts and provides a stable private vulnerability-reporting route plus a detail-free fallback. |

## Revalidation record

The following checks passed on 2026-08-09. Their success output is generic;
private payloads were discarded or kept outside the repository:

```sh
./scripts/verify-public-main-ruleset.sh EnjoyableWork/mcp-sync
./scripts/verify-main-repository-controls.sh EnjoyableWork/mcp-sync
./scripts/verify-repository-security-controls.sh EnjoyableWork/mcp-sync
./scripts/verify-public-project-contract.sh EnjoyableWork/mcp-sync 0.1.0
./scripts/verify-workflow-supply-chain.sh --verify-upstreams
./scripts/verify-repository-artifacts.sh
./scripts/test-repository-artifact-policy.sh
./scripts/verify-distribution-authentication.sh EnjoyableWork/mcp-sync 0.1.0
```

The owner-only organization-access verifier also passed against freshly read
aggregate state and the already-confirmed same-day private recovery assertions.
Its temporary owner-only evidence was removed from the working location after
verification and no private field or count is reproduced here.

The four controls whose evidence predates M3 were revalidated directly:

- `OSPS-DO-01.01`: every current basic command is represented in the README and
  detailed usage/recovery guide, and the built-binary journeys cover the same
  surface;
- `OSPS-QA-01.01`: the authoritative repository remains publicly readable,
  stable, and unarchived;
- `OSPS-QA-01.02`: its public Git history remains readable with author,
  committer, and timestamp metadata; and
- `OSPS-QA-02.01`: locked Cargo metadata agrees with the direct dependency list
  in the committed manifest.

## SLSA v1.0 Build Level 2 artifact statement

The seven exact assets attached to the immutable `v0.1.0` GitHub Release meet
the [SLSA v1.0 Build Level 2](https://slsa.dev/spec/v1.0/levels#build-l2)
requirements under this artifact-only scope. This statement uses the v1.0
definition named by `MCP-035`; it does not claim a newer SLSA specification,
Build Level 3, registry metadata, installer behavior, future releases, or every
output from the project.

| Attested `v0.1.0` asset | Verified SHA-256 |
| --- | --- |
| `enjoyable-mcp-sync-0.1.0.crate` | `dc48488c20725abc4d773834544acd6965b78b2b2a963b0386b09752a2c3288e` |
| `mcp-sync.rb` | `d4b364917faf1d8497a57cf44c2a3e351aa388a1260c678bb1da71f8f7e58f7f` |
| `mcp-sync-v0.1.0-aarch64-unknown-linux-gnu.tar.gz` | `5cc6627fdba5b7cb6ad51c1fd9a88dea4e7ddcec140deb0f22fe7e3f5051f1ce` |
| `mcp-sync-v0.1.0-aarch64-unknown-linux-gnu.spdx.json` | `95aee76134327702a509c270dcd8c221d1794347b08da6f6580417ea6723ea41` |
| `mcp-sync-v0.1.0-x86_64-unknown-linux-gnu.tar.gz` | `bbb5d685f24e61985c74887bd603de5bb262815627cad39be6e0c8efc099fd8b` |
| `mcp-sync-v0.1.0-x86_64-unknown-linux-gnu.spdx.json` | `3c4564c6fd8b91fe88c2fcd5b70aa305453a069e87d27f0fae13c9e9cc9db8e6` |
| `SHA256SUMS` | `b6a4fd20110bf8a2f4dbf3952239e7ee6bc8698afb73c3aff58637a44bc7720d` |

For each asset, `gh attestation verify` authenticated a Sigstore-backed
in-toto Statement v1 with a `https://slsa.dev/provenance/v1` predicate. The
statement records the GitHub Actions workflow, hosted runner environment
(`github-hosted`),
build process, `refs/tags/v0.1.0`, and source commit
`08365900497bf6fde3749695078794e0f3ecf790`. The certificate binds the same
public repository, tag, source digest, and
[`source-linux-release.yml`](../.github/workflows/source-linux-release.yml)
workflow through GitHub's OIDC issuer. Downstream verification checked that
signature and identity, the immutable release attestation, every subject
digest, and the exact source and workflow constraints. The [successful release
run](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31284055636) is the
public hosted-build record.

The crates.io package and Homebrew formula are byte-identical to their attested
GitHub Release assets. That identity extends the verified bytes through those
two HTTPS distribution paths; it does not turn crates.io, Homebrew, or a local
source build into a separately assessed SLSA build platform.

## Maintenance and claim-removal contract

Reassess the complete baseline at least annually, no later than 2027-08-09, and
immediately after any of the following:

- the named OSPS checklist, BadgeApp criteria, badge status, or badge URL
  changes;
- organization ownership, authentication, collaborator, application,
  credential, repository, or private recovery state changes;
- the default branch, ruleset, contribution process, codebase inventory,
  license, security contact, reporting route, or GitHub security controls
  change;
- a dependency, workflow, action, protected environment, release pipeline,
  packaging rule, artifact policy, or official distribution channel changes;
- a security, secret, access, release, or recovery incident occurs; or
- any evidence link, verifier, public claim, or exact-artifact statement stops
  matching current state.

If a control no longer passes, mark it accurately in BadgeApp, remove or
qualify the README badge and trust statement, and update this document and
`PROJECT.md` in the same corrective change. Do not leave a passing image or
claim visible while remediation is pending. Restore it only after every named
control and public-proof check passes again.
