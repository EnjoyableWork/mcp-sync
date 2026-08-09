# Project community, repositories, and licensing

This document inventories the public `mcp-sync` project boundary and records
the contribution, discussion, channel, and MIT licensing evidence maintained by
`MCP-032`. It describes current public state rather than the README's
north-star distribution destination.

The evidence below is scoped input to the later complete OpenSSF OSPS Baseline
self-assessment. It is not an assurance badge, independent certification, or a
claim that controls outside this ticket are complete.

## Official project channels

These are the official public channels for `mcp-sync`. Every published URI uses
HTTPS.

| Purpose | Official channel | Current boundary |
| --- | --- | --- |
| Authoritative source, history, and documentation | [EnjoyableWork/mcp-sync](https://github.com/EnjoyableWork/mcp-sync) | Product source, tests, public policy, delivery plan, and release automation |
| Defects, proposed changes, and usage obstacles | [GitHub Issues](https://github.com/EnjoyableWork/mcp-sync/issues) | Public, best-effort discussion through the structured issue forms |
| Proposed implementation changes | [GitHub pull requests](https://github.com/EnjoyableWork/mcp-sync/pulls) | Public review and protected merge path |
| Vulnerability reports | [GitHub private vulnerability reporting](https://github.com/EnjoyableWork/mcp-sync/security/advisories/new) | Private route defined by `SECURITY.md`; no vulnerability details belong in public channels |
| Canonical immutable release | [GitHub Releases](https://github.com/EnjoyableWork/mcp-sync/releases) | `v0.1.0` currently supplies source and GNU/Linux x64/ARM64 artifacts only |
| Rust source package | [crates.io `enjoyable-mcp-sync`](https://crates.io/crates/enjoyable-mcp-sync) | Installs the `mcp-sync` executable from source on supported build hosts |
| Homebrew formula | [EnjoyableWork/homebrew-tap](https://github.com/EnjoyableWork/homebrew-tap) | Source-building `Formula/mcp-sync.rb` for macOS and GNU/Linux |

GitHub Discussions is not an enabled project channel. Public issue and
pull-request threads provide the maintained discussion mechanism. There is no
official direct-message support channel or unencrypted project mirror.

The target-state README also describes funded, signed macOS and Windows
artifacts and WinGet. Those are not current distribution channels: they remain
funding-dependent `SIDE-006` scope and cannot be advertised as published until
that ticket's signing and native verification gates pass.

## Repository and codebase inventory

The project spans exactly two organization-owned public repositories today:

| Repository | In-scope codebase | Authority and license |
| --- | --- | --- |
| [EnjoyableWork/mcp-sync](https://github.com/EnjoyableWork/mcp-sync) | The complete repository | Authoritative product codebase. Root `LICENSE` applies the MIT License to the source; `Cargo.toml` declares `MIT`. |
| [EnjoyableWork/homebrew-tap](https://github.com/EnjoyableWork/homebrew-tap) | `Formula/mcp-sync.rb` | Distribution codebase only. The tap's root `LICENSE` applies the MIT License to repository source, and the formula declares the distributed software license as `MIT`. |

The other public repositories currently visible under the organization—
`EnjoyableWork/courtside-mcp`, `EnjoyableWork/enjoyable-mcp`, and
`EnjoyableWork/mcp-doctor`—are separate products and are not `mcp-sync`
codebases. Third-party MCP client repositories, registries, package-manager
infrastructure, forks, and user configuration files are also outside this
project's codebase inventory.

Adding another source or distribution repository requires this inventory,
licensing, ownership, release verification, and the applicable project plan to
be updated before the new repository is treated as official.

## Public contribution and support boundary

[`CONTRIBUTING.md`](../CONTRIBUTING.md) explains how to report defects, discuss
usage obstacles and proposed changes, prepare a checkout, test a change, and
submit a pull request. The structured bug and feature forms are the maintained
entry points, and public issue comments carry the resulting discussion.

Support is best effort. Only the latest public release receives security fixes;
development snapshots are not supported releases. Client-specific behavior
outside the documented `mcp-sync` ownership boundary normally belongs with the
client project. Reports and examples must use generic names plus synthetic,
redacted values. Vulnerabilities use the private route in
[`SECURITY.md`](../SECURITY.md), never a public issue or pull request.

[`CODE_OF_CONDUCT.md`](../CODE_OF_CONDUCT.md) defines behavior and enforcement
for project spaces. Its public contact form requests only a private follow-up;
incident details and identities must not be posted in the public request.

## Source and release licensing

`mcp-sync` uses the [OSI-approved MIT License](https://opensource.org/license/mit).
The authoritative license text is maintained in the root [`LICENSE`](../LICENSE),
and the package manifest declares `license = "MIT"`.

Current release licensing is carried with the software rather than inferred
from a badge or SBOM field:

- the immutable `v0.1.0` tagged source contains the root `LICENSE`;
- the published `enjoyable-mcp-sync-0.1.0.crate` contains that `LICENSE`;
- both published GNU/Linux binary archives contain that `LICENSE` beside the
  executable and release documents;
- the published Homebrew formula declares `license "MIT"`, and its source is
  covered by the tap repository's root `LICENSE`; and
- the generated source and binary packaging scripts fail their archive checks
  if the required `LICENSE` is absent.

Checksums and SPDX SBOMs are release metadata; they do not replace the license
shipped with each software archive. The current SBOM generator reports
`NOASSERTION` for package license fields, so those fields are deliberately not
used as licensing evidence. Future funded macOS and Windows archives already
use the same required `LICENSE` packaging boundary, but they are not published
`v0.1.0` assets.

## Inbound contribution terms

Contributions use an inbound-equals-outbound model: an intentionally submitted
contribution is provided under the project's MIT License, and the contributor
must have the right to provide it. The project requires neither copyright
assignment nor a Contributor License Agreement (CLA).

The project also does not currently require Developer Certificate of Origin
(DCO) sign-off. The CLA and DCO choices are separate: no CLA does not imply a
DCO requirement, and a DCO sign-off is not a CLA. Any later adoption requires
a new recorded decision, updated public guidance, and compatible enforcement
before it applies.

There is no `CODEOWNERS` file while the repository has no independent reviewer.
Adding one now would imply a review path that does not exist. Revisit ownership
and required review together when a second trusted reviewer is available.

## OpenSSF OSPS Baseline evidence mapping

This ticket uses the exact OpenSSF OSPS Baseline `v2026.02.19` Level 1
requirements named below.

| Control | MCP-032 evidence |
| --- | --- |
| `OSPS-BR-03.01` | The official-channel table and credential-free URI audit allow only HTTPS channel URIs. |
| `OSPS-DO-02.01` | `CONTRIBUTING.md` and the structured bug form explain safe defect reporting for the released project. |
| `OSPS-GV-02.01` | Public issues and pull requests support discussion of proposed changes and usage obstacles; the feature form provides the entry point. |
| `OSPS-GV-03.01` | `CONTRIBUTING.md` documents the complete contribution process. |
| `OSPS-LE-02.01` | The source uses the OSI-approved MIT License. |
| `OSPS-LE-02.02` | Every current released software archive carries the MIT `LICENSE`; the formula declares `MIT`. |
| `OSPS-LE-03.01` | Both in-scope source repositories maintain a root `LICENSE`. |
| `OSPS-LE-03.02` | The tagged source, Cargo package, and GNU/Linux archives each include the license alongside released software. |
| `OSPS-QA-04.01` | The two-repository codebase inventory above includes the authoritative source and the in-scope distribution formula and explicitly excludes unrelated repositories. |

Run the credential-free public verifier after a default-branch change:

```bash
./scripts/verify-public-project-contract.sh EnjoyableWork/mcp-sync 0.1.0
```

The verifier checks GitHub community-profile recognition, HTTPS-only official
URI files, both repository licenses, the exact immutable release asset set,
license presence in every software archive, crates.io byte identity, and the
published tap formula. Focused repository tests enforce the local contract.

Reverify this document when a repository or official channel changes, a
community template moves, GitHub changes community-profile recognition, a
license or contribution policy changes, a release asset or packaging script
changes, the supported-release boundary changes, or the named OSPS Baseline
version changes.
