# mcp-sync project plan

This is the canonical living document for product scope, delivery status,
tickets, deliverables, decisions, risks, and release gates. It is intentionally
lightweight enough to use before a hosted tracker is justified.

| Control | Current state |
| --- | --- |
| Document state | Active |
| Product state | The source-checkout macOS, GNU/Linux, and Windows implementation reconciles global Claude Desktop, Cursor, Windsurf legacy Cascade, native VS Code default user-profile, and global Codex host configuration, exposes one bounded named-server STDIO initialize health boundary, and provides parser-validated restore with one-generation adjacent backup retention for canonical state plus all five targets. Deterministic fixtures and native x64/ARM64 GNU/Linux and Windows MSVC CI prove the pre-restore five-target configuration behavior; the restore journey passes locally on ARM64 and Intel macOS, resolves every supported platform path in tests, and compiles every test surface for all six supported architecture targets. The credential-free source and GNU/Linux release preflight proves deterministic Cargo and source-built Homebrew outputs, exact-package installation and recovery across all six native hosts, Homebrew installation and recovery across four native Unix hosts, and the two represented GNU/Linux archive/SBOM paths while the retained six-target release preflight remains green. The immutable `v0.1.0` GitHub Release, byte-identical `enjoyable-mcp-sync` `0.1.0` crates.io package, and exact source-building `EnjoyableWork/tap/mcp-sync` formula are public; the final installed-channel matrix passes two GitHub archive, six Cargo, and four Homebrew native install and recovery journeys. Claude Desktop and Cursor retain macOS current-client sync smoke evidence, and no Linux or Windows target has a current-client smoke claim |
| Current milestone | M3 — Enterprise assurance and adoption, complete; no successor active |
| Overall status | M0 through M3 and `MCP-001` through `MCP-035` are `Done`; no later main-story milestone or side quest is active, and only funded `SIDE-006` remains proposed |
| Current focus | Maintain the completed M3 controls, BadgeApp `baseline-1` status, public assurance evidence, exact-artifact SLSA statement, and immutable M2 release while collecting adoption feedback; select and activate any later arc deliberately under its own accepted ticket and Goal |
| Milestone target | Completed 2026-08-09 |
| Last reviewed | 2026-08-09 |
| Next review trigger | An organization-access, recovery, Actions, dependency-update, workflow-trust, distribution-authentication, repository-artifact, `main` ruleset/check-context, repository-security-control, community-contract, repository-inventory, or licensing change; an assurance-framework, issuer-status, badge, or public-proof-link revision; a security or release incident; evidence drift; deliberate selection of a later main-story arc; or explicit funding and activation of `SIDE-006` |

## Document roles

The repository deliberately separates the finished-product story from current
delivery evidence:

| Artifact | Role | How to interpret it |
| --- | --- | --- |
| [README.md](README.md) | North-star product specification and public marketing page | Describes the end-state user experience in finished-product language. It is the front door for people discovering the project and the destination implementation should strive toward; it is not a progress report. |
| [PROJECT.md](PROJECT.md) | Living delivery truth | Records what exists now, what comes next, which parts of the README vision remain unbuilt, and the evidence required to advance. |
| Code, tests, and releases | Implemented and released truth | Prove which parts of the north-star contract actually work. README prose alone is never implementation evidence. |

Do not dilute the README into an internal status diary or replace its product
story with ticket mechanics. Change it when the desired end product changes,
when its public experience becomes more precise, or when distribution strategy
changes. Track the implementation gap here. Before a release channel is
actually published, its package identity, install command, artifacts, and smoke
tests must be verified by their owning release ticket: `MCP-029` for the
zero-cost market-validation release and `SIDE-006` for the funded signed-native
destination. Public assurance is current-state evidence rather than a
north-star exception. An accepted assurance target's owning ticket must verify
its exact framework version, level, scope, assessment date, and durable public
evidence before any README claim appears. When the framework provides an
official badge or conformance mark for the achieved scope, obtaining that
issuer-provided proof and linking it from the README is part of completing the
target, not optional follow-up. When no official proof exists, use an explicit
scoped evidence link and never substitute a project-created shield that could
be mistaken for certification. Correct or remove every claim, badge, or mark
when its evidence or issuer status becomes stale.

## How to operate this document

Use this document as the repository’s Linear/Jira equivalent for now.

1. Start only the earliest incomplete main-story ticket marked `Ready`, or an
   explicitly classified side quest that satisfies the rules below.
2. Give every material change a ticket. Keep one outcome per ticket and split
   work that can be delivered or reverted independently.
3. Move the ticket through the status model below and link durable evidence
   before marking it `Done`.
4. Update affected deliverables, decisions, risks, and milestone gates in the
   same change as the implementation.
5. Review this document after each completed ticket and at least weekly while
   active development is underway.
6. Update current state in place. Git preserves history, so this file should not
   become an append-only activity log.

Work-in-progress limit: one main-story ticket, plus at most one independent side
quest that does not delay, redefine, or consume required capacity from the main
story. A ticket cannot be `In progress` without an owner.

If GitHub Issues, Linear, or another tracker is adopted, that system may own
individual issue discussion. This file remains the concise repository-level
source for milestone health, accepted scope, deliverables, decisions, risks,
and links to the active backlog.

## Main story and side quests

The **main story** is the ordered critical path to a usable product. Complete it
in sequence. Only the earliest incomplete main-story ticket may be `Ready` or
`In progress`; every later ticket remains `Proposed` even if it could be coded
in isolation.

Do not jump over a difficult or blocked main-story ticket to work on a later
one. A blocker stops the main sequence until it is resolved or an accepted
decision changes the story. A main-story ticket leaves the sequence only when:

- an accepted decision proves its outcome is irrelevant to the product path
  and marks it `Cancelled`;
- an accepted decision replaces it with a clearly identified ticket at the
  same point in the sequence and marks it `Superseded`; or
- earlier work already satisfies every acceptance criterion, in which case the
  ticket is closed as `Done` with evidence rather than skipped.

The main story is complete through M2 and accepted for M3:

| Arc | Required ticket order |
| --- | --- |
| M0 — Foundations | `MCP-001` → `MCP-002` → `MCP-003` → `MCP-004` → `MCP-005` |
| M1 — Town MVP | `MCP-006` → `MCP-007` → `MCP-008` → `MCP-009` → `MCP-010` → `MCP-011` → `MCP-012` → `MCP-013` |
| M2 — Dependable city | `MCP-014` → `MCP-015` → `MCP-016` → `MCP-017` → `MCP-018` → `MCP-019` → `MCP-020` → `MCP-028` → `MCP-029`; `MCP-021` is superseded at the post-`MCP-020` story point by `DEC-036` |
| M3 — Enterprise assurance and adoption | `MCP-030` → `MCP-031` → `MCP-032` → `MCP-033` → `MCP-034` → `MCP-035`; `DEC-038` promotes and supersedes the optional `SIDE-001` through `SIDE-005` forms without reopening M2 |

A **side quest** is useful, optional work that the main story does not require.
Side quests use `SIDE-NNN` IDs and obey all of these rules:

- no main-story ticket, milestone gate, safety invariant, or release claim
  depends on the side quest;
- the main story remains correct, safe, and releasable if the side quest is
  never completed;
- it cannot be used to work around a blocker or quietly introduce later-story
  architecture;
- it may run only within the work-in-progress limit and must yield capacity to
  the main story; and
- if it becomes necessary, promote it into the main sequence through an
  accepted decision before continuing dependent work.

There are no active side quests. `DEC-038` promotes and supersedes
`SIDE-001` through `SIDE-005` with ordered M3 tickets because repository
assurance is now an explicit adoption outcome rather than optional work.
`SIDE-006` remains the funding-dependent signed-native distribution stretch
and does not gate M3. The deferred post-M3 items are expansion candidates, not
executable work. Before one starts, classify it as the next ordered main-story
arc or as a genuinely independent `SIDE-NNN` ticket.

## Goal-mode execution

[Codex Goal mode](https://learn.chatgpt.com/docs/long-running-work) is the
thread-level execution contract for one active ticket. It does not replace
repository truth: [README.md](README.md) defines the destination, this document
defines scope and order, and [AGENTS.md](AGENTS.md) defines durable engineering
constraints. A goal cannot waive a dependency, acceptance criterion, safety
rule, open decision, or required evidence.

Every main-story ticket and every classified side quest has one exact canonical
objective in the [main-story goal catalog](#main-story-goal-catalog) or
[side-quest goal catalog](#side-quest-goal-catalog). To set one manually, paste
the objective after [`/goal`](https://learn.chatgpt.com/docs/developer-commands.md?surface=cli).
Goal-capable agents are explicitly authorized and required to reconcile the
active thread goal before beginning either kind of ticket:

1. Identify the ticket being started. A main-story ticket must be the earliest
   incomplete main-story ticket; a side quest must satisfy every independence
   rule and the work-in-progress limit. Either ticket must already be `Ready`;
   a goal never makes an otherwise ineligible ticket ready.
2. Read the thread's current goal when the runtime supports it and compare the
   objective with the catalog after trimming only surrounding whitespace.
3. If there is no unfinished goal, set the catalog objective. If the active
   objective targets the same eligible ticket but differs, set or edit it to
   the catalog objective before doing implementation work. If it belongs to a
   different unfinished ticket—or the runtime refuses replacement—stop and
   report both ticket IDs so the user can run `/goal edit` or `/goal clear`;
   never silently abandon or overwrite unfinished work.
4. Do not attach a token budget unless the user explicitly requests one. Scope
   is controlled by the ticket and its evidence, not by an arbitrary token
   ceiling.
5. Only after the goal matches may the ticket receive an owner and move to
   `In progress`. Use an implementation plan for tactical steps inside the goal;
   the plan may not broaden the objective. One thread may carry only one
   unfinished goal, so concurrent main-story and side-quest work requires
   separate threads rather than replacing either active goal.
6. Mark the goal complete only after the ticket-done gate passes, durable
   evidence is recorded here, and the ticket is `Done`. For a main-story
   ticket, only its immediate successor becomes `Ready`, and that successor is
   not activated until work intentionally starts. A completed side quest has no
   automatic successor and cannot activate another side quest.

An agent without Goal mode follows the same catalog objective as its task
contract and continues normally. Goal state remains runtime/thread state rather
than a second status field in this repository.

## Status and priority semantics

| Status | Meaning |
| --- | --- |
| Proposed | The outcome is useful but scope, acceptance, or sequencing still needs review. |
| Ready | The outcome and acceptance evidence are clear, every dependency is done, and—on the main story—the immediately preceding ticket is done. |
| In progress | An owner is actively working on the ticket. |
| Blocked | Progress requires a named external dependency, decision, or authority recorded in the ticket. |
| Done | Acceptance criteria pass and durable evidence is linked. |
| Deferred | Deliberately outside the current release; the consequence is understood. |
| Cancelled | No longer intended; the reason or superseding decision is recorded. |
| Superseded | Replaced by a named ticket through an accepted decision at the same point in the sequence. |

| Priority | Meaning |
| --- | --- |
| P0 | Required for the current milestone or needed to protect user data. |
| P1 | Required for the next useful milestone. |
| P2 | Valuable expansion after the current product proof. |

Dates represent commitments only when both an owner and a target are present.
Unscheduled work is sequenced by milestone, priority, and dependencies rather
than an invented deadline.

## Product outcome

`mcp-sync` gives a developer one local source of truth for MCP server
definitions, then safely reconciles those definitions into each supported
client’s native configuration.

The proof is not “we can serialize JSON.” The proof is that a user can import
realistic existing configuration, understand a redacted plan, apply it to more
than one client without losing unrelated data, repeat the operation as a no-op,
and recover if a write fails.

### Product principles

- **Working software first:** complete one end-to-end journey before expanding
  client count, platforms, interfaces, or distribution.
- **Safety is part of the MVP:** dry-run, validation, redaction, backups,
  idempotence, and failure recovery are not later polish for a configuration
  writer.
- **Core plus adapters:** keep the canonical model and reconciliation rules
  client-independent; isolate real schema and path variation at target
  boundaries.
- **Explicit evolution:** version configuration and decisions so later changes
  can migrate without breaking the last working release.
- **Evidence over percentage:** progress is a passed gate with linked evidence,
  not a subjective completion percentage.
- **North-star product story:** preserve the README as the clear public picture
  of the finished product while this tracker records the route and current gap.
- **Assurance serves adoption:** implement controls because they protect users,
  contributors, maintainers, and the official supply chain; self-attest only
  after durable evidence exists, then expose the achieved assurance where it
  reduces evaluation and adoption friction. Treat an official issuer-provided
  badge or conformance mark as required completion evidence whenever the
  accepted target offers one, while preserving exact self-assessment language.
- **Narrow today, replaceable tomorrow:** optimize the first release for one
  useful path while keeping operating-system, filesystem, client, and process
  boundaries explicit.

## Growth model

The project grows by completing useful, stable layers rather than attempting
the final platform immediately.

| Milestone | Metaphor | Outcome | Exit gate | Status |
| --- | --- | --- | --- | --- |
| M0 | Survey and foundations | A runnable Rust skeleton, versioned config contract, quality gates, and fixture harness | Clean checkout can build, lint, test, and show CLI help; the first schema and test environment are documented | Done |
| M1 | Town — MVP | One safe vertical journey across Claude Desktop and Cursor on macOS | Every MVP acceptance criterion below passes against a synthetic home directory | Done |
| M2 | City — dependable daily tool | README client coverage, health checks, cross-platform behavior, restore UX, and a zero-cost market-validation release | Supported matrix passes in CI and one protected tagged release installs and smoke-tests through every channel represented by the M2 release without requiring paid platform signing | Done |
| M3 | Trusted project — enterprise assurance and adoption | Contributor-compatible repository controls plus a dated, scoped, evidence-backed zero-cost assurance baseline that helps organizations evaluate the project | All 24 OpenSSF OSPS Baseline `v2026.02.19` Level 1 controls pass, the self-attestation and supporting evidence are public, the official BadgeApp `baseline-1` self-certification badge is issued and linked from an exact README trust claim, and no higher-level or independent-certification claim is implied | Done — `MCP-030` through `MCP-035` Done |
| M4 | Metropolis — extensible ecosystem | More clients, profiles, migrations, secret references, and a proven adapter contract | New adapters can be added without changing reconciliation invariants; upgrade paths are tested | Deferred |
| M5 | Country — managed platform hypothesis | Optional team/fleet policy, richer interfaces, and ecosystem distribution | Requires validated demand, a separate product plan, and compatibility commitments | Deferred |

Each milestone must leave the preceding milestone usable. Later work may replace
an internal implementation only when the public behavior is preserved or a
tested migration and rollback path exists.

## Distribution and accessibility target

`mcp-sync` will not be an npm package. Its north-star distribution target is a
Rust binary available through several channels so users do not need a Rust
toolchain merely to run it. Every end-state channel must install the same
`mcp-sync` version from one tagged, tested release.

| Channel | Audience | Target installation | Role |
| --- | --- | --- | --- |
| GitHub Releases | All platforms, including users without a package manager | Download a prebuilt macOS, Linux, or Windows archive | Canonical immutable release containing versioned binaries, checksums, SBOMs, release notes, and provenance. |
| Homebrew tap | macOS and Linux users | `brew install EnjoyableWork/tap/mcp-sync` | Primary one-command Unix installation. An organization-owned tap keeps the formula under project control. |
| Windows Package Manager | Windows users | `winget install EnjoyableWork.mcp-sync` | Native discovery, installation, and upgrades through the public WinGet repository. |
| crates.io / Cargo | Rust developers with a toolchain | `cargo install enjoyable-mcp-sync` | Builds from the published source package and installs a binary still named `mcp-sync`. |

### Zero-cost `v0.1.0` market-validation contract

`DEC-036` narrows the M2 release to channels that can be published without an
Apple Developer Program membership or paid Windows code-signing authority. The
first public release remains `v0.1.0`; release candidates may use
`v0.1.0-rc.N`, but the normal Cargo and Homebrew paths must not point at a
candidate. The market-validation release must:

- create one protected, immutable GitHub Release for the exact `v0.1.0` tag;
- publish native GNU/Linux x64 and ARM64 archives with SHA-256 checksums,
  target-specific SPDX SBOMs, build attestations, and installed native smoke
  evidence;
- publish `enjoyable-mcp-sync` `0.1.0` from the same commit to crates.io after
  a live name recheck, then prove `cargo install enjoyable-mcp-sync` and the
  installed recovery journey on supported macOS, GNU/Linux, and Windows hosts;
- publish `EnjoyableWork/tap/mcp-sync` from the same source and version as a
  source-building formula, then prove native Homebrew installation and the
  installed recovery journey on supported macOS and GNU/Linux hosts; and
- state the release's installation boundary accurately: there are no
  project-issued macOS or Windows binaries and no WinGet package in `v0.1.0`.

The free path must not publish ad-hoc-signed or unsigned macOS or Windows
binaries. It must also prevent a `v0.1.0` tag from accidentally invoking the
credential-gated signed workflow, while retaining that workflow, its six-target
preflight, downstream metadata generator, signing checks, and runbook for
`SIDE-006`. Free does not mean authority-free: completing `MCP-029` used a
no-cost [crates.io account and scope-minimized first-publication token](https://doc.rust-lang.org/cargo/reference/publishing.html),
write access limited to the organization Homebrew tap, and the protected GitHub
release path; the one-use Cargo token is now revoked. The
[Homebrew formula contract](https://docs.brew.sh/Formula-Cookbook)
supports building from an immutable source archive; its native evidence must
include the build prerequisites and supported host boundary rather than
presenting the formula as a prebuilt bottle.

WinGet itself is not classified as a paid channel. It is deferred because its
accepted package contract consumes the project-issued Windows executable, and
that executable remains subject to the paid Public Trust signing requirement.
Cargo and the source-building Homebrew formula avoid that dependency by
compiling the published source on the user's host.

The human publication authorities used for the zero-cost release were explicit
and are now settled as follows:

| Authority | `MCP-029` action and final state | Paid signing required? |
| --- | --- | --- |
| crates.io | Completed: the verified publisher rechecked `enjoyable-mcp-sync`, used one scope-minimized first-publication token, published the exact immutable-release bytes, then revoked the token server-side and removed it locally | No |
| `EnjoyableWork/homebrew-tap` | Completed: one repository-scoped deploy key in the protected `release` environment created only the byte-identical `Formula/mcp-sync.rb` through the reviewed publisher | No |
| `EnjoyableWork/mcp-sync` | Completed: the owner authorized the protected `v0.1.0` release and the separately constrained operational recovery tag and deployment after their exact evidence gates passed | No |
| Apple and Windows signing | Not performed for `MCP-028` or `MCP-029`; provision these authorities only after explicit funding and activation of `SIDE-006` | Yes, deferred |

### Funded signed-native release contract

`DEC-024` originally made the full signed-native matrix the M2 release gate.
`DEC-036` supersedes that timing, not its safety bar: after funding and market
evidence justify activation, `SIDE-006` must publish the next unreleased stable
version through the complete contract below. Every funded channel consumes the
same commit and version first published as the canonical GitHub Release; it
must never reuse or replace `v0.1.0` assets.

The funded binary matrix contains six separate 64-bit artifacts:

| Operating system | CPU | Rust target | Release archive |
| --- | --- | --- | --- |
| macOS | Apple Silicon / ARM64 | `aarch64-apple-darwin` | `mcp-sync-v{semver}-aarch64-apple-darwin.tar.gz` |
| macOS | Intel / x64 | `x86_64-apple-darwin` | `mcp-sync-v{semver}-x86_64-apple-darwin.tar.gz` |
| Linux | ARM64 | `aarch64-unknown-linux-gnu` | `mcp-sync-v{semver}-aarch64-unknown-linux-gnu.tar.gz` |
| Linux | x64 | `x86_64-unknown-linux-gnu` | `mcp-sync-v{semver}-x86_64-unknown-linux-gnu.tar.gz` |
| Windows | ARM64 | `aarch64-pc-windows-msvc` | `mcp-sync-v{semver}-aarch64-pc-windows-msvc.zip` |
| Windows | x64 | `x86_64-pc-windows-msvc` | `mcp-sync-v{semver}-x86_64-pc-windows-msvc.zip` |

These are supported [Rust platform targets](https://doc.rust-lang.org/rustc/platform-support.html),
but Rust's target tier is not product verification: `MCP-018` and `MCP-019`
establish native GNU/Linux and Windows source-checkout behavior, respectively,
and `SIDE-006` must build and smoke-test each advertised OS/CPU
artifact. The funded release has no 32-bit, Alpine/musl, or combined
universal-macOS artifact. Add one only through a later accepted support
decision with native CI and install evidence.

Stable release trust is mandatory rather than best-effort:

- Sign each macOS Mach-O with the `com.enjoyablework.mcp-sync` identifier, a
  Developer ID Application certificate, hardened runtime, and a secure
  timestamp before packaging. For each architecture, submit a temporary ZIP
  containing the exact signed release payload through Apple's current
  `notarytool` flow, require accepted notarization plus local `codesign` and
  Gatekeeper verification, and only then package the same bytes into the
  advertised tar archive. Apple documents Developer ID signing
  and [notarization for software distributed outside the App Store](https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution).
- Authenticode-sign and timestamp each Windows executable before creating its
  ZIP, using [Azure Artifact Signing Public Trust](https://learn.microsoft.com/en-us/azure/artifact-signing/concept-trust-models)
  through its supported GitHub Actions integration. A publicly trusted
  CA-issued Authenticode certificate is the fallback if Artifact Signing is
  unavailable; unsigned Windows artifacts are not a stable-release fallback.
- Linux has no additional OS-native signature in the initial contract. All six
  archives receive the shared checksum and provenance controls below.
- Publish one `SHA256SUMS` manifest, a target-specific SPDX JSON SBOM beside
  each archive, and a GitHub build-provenance attestation for every archive and
  SBOM. Do not claim reproducible builds until independently reproduced bytes
  exist, and do not introduce a separate long-lived GPG signing key initially.
- Create the GitHub Release as a draft, attach and verify the complete asset
  set, then publish it with [release immutability](https://docs.github.com/en/code-security/concepts/supply-chain-security/immutable-releases)
  enabled. Immutability locks the tag and assets and adds a release attestation;
  explicit [artifact attestations](https://docs.github.com/en/actions/how-tos/secure-your-work/use-artifact-attestations/use-artifact-attestations)
  retain build-workflow provenance. A correction after publication requires a
  new version rather than replacement assets or a moved tag.
- Before creating a stable tag, activate a `v*` tag ruleset that limits
  creation to the authorized release path and prevents update or deletion
  outside an explicit emergency bypass. Publish through a protected release
  environment with tag deployment restrictions and least-privilege
  credentials, preferring OIDC or trusted publishing over long-lived tokens
  wherever a destination supports it. An interactive repository-administration
  credential is not a release credential and must not be stored in the
  repository or its workflows.

The public identities are fixed independently of the legal subject displayed
by a platform signing certificate:

| Surface | Identifier |
| --- | --- |
| Product, executable, and package moniker | `mcp-sync` |
| GitHub repository | `EnjoyableWork/mcp-sync` |
| Git tag | `v{semver}` |
| Cargo package | `enjoyable-mcp-sync` |
| Homebrew repository / tap / formula | `EnjoyableWork/homebrew-tap` / `EnjoyableWork/tap` / `mcp-sync` |
| WinGet package / publisher / package name | `EnjoyableWork.mcp-sync` / `EnjoyableWork` / `mcp-sync` |
| macOS signing identifier | `com.enjoyablework.mcp-sync` |

In the funded path, Homebrew selects the matching funded macOS or GNU/Linux
archive and pins its SHA-256. WinGet uses ZIP/portable manifests for the x64
and ARM64 signed Windows archives and
pins each installer SHA-256; WinGet currently supports both
[ZIP and portable packages](https://learn.microsoft.com/en-us/windows/package-manager/winget/).
Cargo publishes the same funded version after the required live name recheck.
Release credentials belong in protected CI environments with least privilege.
Missing Apple or Windows signing authority blocks `SIDE-006`; it does not
block `MCP-028`, `MCP-029`, or the zero-cost channels established by them, and
it never silently downgrades the signed-native trust contract.

The crates.io package name `mcp-sync` is already assigned to an
[unrelated Rust project](https://docs.rs/crate/mcp-sync/latest), and crates.io
names are first-come-first-served. A live registry query on 2026-08-06 found no
package named `enjoyable-mcp-sync`, so that distinct name is now used in
`Cargo.toml`; availability must be checked again immediately before publication
because crates.io does not reserve an unclaimed name. The product, repository,
and installed executable remain `mcp-sync`.

Distribution must follow the primary channel contracts:

- [Cargo installs binary crates from crates.io](https://doc.rust-lang.org/cargo/commands/cargo-install.html),
  but Rust documents Cargo installation as a developer convenience rather than
  a replacement for system packages.
- [Homebrew taps](https://docs.brew.sh/How-to-Create-and-Maintain-a-Tap) allow
  an organization to publish and maintain its own formula and support direct
  one-command installation.
- [GitHub Releases](https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases)
  provide the canonical tagged release and downloadable binary assets.
- [WinGet manifests](https://learn.microsoft.com/en-us/windows/package-manager/package/repository)
  make approved packages discoverable through the public Windows Package
  Manager repository.

## M1 MVP boundary

The following is a working baseline chosen to reach a meaningful proof quickly.
Items marked as assumptions can change through the decision log without
discarding the core architecture.

| Dimension | M1 boundary |
| --- | --- |
| Runtime | One compiled Rust command-line binary |
| Initial platform | macOS — accepted for M1; Linux followed in `MCP-018`, and Windows followed in `MCP-019` |
| Initial clients | Current stable Claude Desktop and Cursor — accepted; two clients are the minimum proof of sync |
| Managed client scope | Global user configuration only: `~/Library/Application Support/Claude/claude_desktop_config.json` and `~/.cursor/mcp.json`; project-level `.cursor/mcp.json` remains untouched |
| Canonical format | Versioned JSON at the documented `mcp-sync` config location |
| Commands | `init`, `add`, `list`, `sync --dry-run`, and `sync` |
| Import | Discover the two initial clients, normalize compatible server entries, and stop with an actionable conflict report when values disagree |
| Sync ownership | Add or update canonical server entries while preserving unrelated native configuration |
| Deletion | Do not silently delete target-only entries; report drift. Explicit prune behavior is post-MVP until its ownership contract is accepted |
| Mutation safety | Validate, show a redacted plan, back up existing files, replace each file atomically, and recover prior writes after a later target failure |
| Secrets | Preserve required values but redact them from terminal output, errors, fixtures, and logs |
| Network and process behavior | `init` and `sync` are local file operations and never start an MCP server; STDIO health testing is M2 |
| Distribution | Build and run from a source checkout for M1; packaged binaries are M2 |
| Support verification | Synthetic automated journeys first, followed by a controlled, backup-protected smoke test with current stable Claude Desktop and Cursor on an accessible Mac before claiming M1 support |

### Canonical configuration v1 decision

`OPEN-02` is resolved for M1 with this contract:

- The root object contains integer `schemaVersion: 1` and a required `servers`
  object. An empty `servers` object is valid.
- Each server's unique identity is its key in `servers`. Names must be
  non-empty, have no leading or trailing whitespace, and contain no control
  characters; no narrower character set is imposed.
- Version 1 models local STDIO servers only. A server has a required literal
  `command`, an ordered array of literal `args`, and a string-to-string `env`
  object. Working directories, remote transports, HTTP, and OAuth are deferred.
- `command` is one executable value and `args` are already separated; neither
  is shell-parsed, split, interpolated, or normalized. Empty argument strings
  remain valid, while values that cannot be passed to a process, such as NUL,
  are rejected.
- Validation is field-aware: server names and commands reject every Unicode
  control character, while arguments and environment keys or values reject NUL
  but preserve other process-safe characters such as newlines in environment
  values.
- Environment values are literal strings preserved exactly, including empty
  strings. Version 1 performs no variable expansion, inheritance, type
  coercion, or secret-reference resolution. Values may be stored but must not
  appear in diagnostics, plans, logs, or normal list output.
- Inputs must include `schemaVersion`, `servers`, and each server's `command`.
  Omitted `args` and `env` normalize to `[]` and `{}`. Unknown fields,
  duplicate JSON object keys, blank or padded names and commands, and unsafe
  control characters are rejected rather than guessed.
- Canonical writes use pretty-printed JSON with `schemaVersion` first, fixed
  server-field order, lexicographically sorted server and environment keys,
  explicit `args` and `env` fields, and one trailing newline.
- Any integer schema version other than `1` produces a distinct unsupported-
  version failure. There is no best-effort downgrade or migration framework
  until a real schema change requires `MCP-022`.

The canonical shape is therefore:

```json
{
  "schemaVersion": 1,
  "servers": {
    "project-files": {
      "command": "example-mcp-server",
      "args": ["--transport", "stdio"],
      "env": {
        "ACCESS_MODE": "read-only"
      }
    }
  }
}
```

### Golden MVP journey

Given a synthetic macOS home directory with realistic global Claude Desktop and
Cursor configuration, including unrelated keys and at least one environment
value:

1. `mcp-sync init` discovers both files and creates one valid canonical config,
   or exits without mutation and reports an exact import conflict.
2. `mcp-sync add` can add or update a server definition, and `mcp-sync list`
   shows names and structural metadata without secret values.
3. `mcp-sync sync --dry-run` validates every input, reports per-target changes
   with secrets redacted, and changes no bytes on disk.
4. `mcp-sync sync` writes valid native configuration for both clients,
   preserves unrelated data, creates recoverable backups, and reports each
   target outcome.
5. Repeating `mcp-sync sync` produces a no-op and does not create needless
   backups or rewrite files.
6. A forced failure on the second target restores the first target and exits
   non-zero with an actionable, redacted error.

### M1 acceptance criteria

- A clean checkout builds with the documented Rust toolchain and has documented
  commands for formatting, linting, and tests.
- The canonical schema has an explicit version, validation rules, deterministic
  serialization, and migration behavior for unknown versions.
- Import behavior is deterministic regardless of client discovery order.
- Dry-run and apply use the same generated plan; apply does not independently
  recalculate different desired output.
- Native files remain parseable and retain fields outside the adapter ownership
  boundary.
- Missing files, malformed JSON, conflicting imports, permission failures,
  interrupted writes, and rollback failures have tested, non-zero outcomes.
- Secret values do not appear in snapshots, normal output, error output, or
  test failure messages.
- Unit, adapter fixture, filesystem integration, and golden CLI journey tests
  pass without reading or writing real user configuration.
- After the synthetic suite passes, a controlled and backup-protected manual
  smoke test confirms the global configuration journey with the then-current
  stable Claude Desktop and Cursor releases on macOS.
- Delivered M1 behavior matches the corresponding README contract, and this
  tracker clearly records which north-star capabilities remain beyond M1.

`MCP-012` completes the technical acceptance gate with durable built-binary,
unit, adapter, filesystem, and controlled current-client evidence. `MCP-013`
repeats the merged-binary client journey, audits the README contract, and
publishes the current usage and recovery guide. Together they close the M1
milestone without moving health testing, additional clients or platforms,
restore UX, or distribution work forward from M2.

The completion audit maps every criterion to durable evidence:

| M1 criterion | Durable evidence |
| --- | --- |
| Stable-toolchain build and documented checks | [`rust-toolchain.toml`](rust-toolchain.toml), [source-checkout commands](README.md), and [`scripts/check.sh`](scripts/check.sh); locked build, format, Clippy, tests, and dependency policy pass |
| Versioned deterministic schema and unknown-version behavior | [Strict canonical v1 model and tests](src/config.rs) plus the [canonical example](examples/config.v1.json) |
| Discovery-order-independent import | [Pure import tests](src/init.rs) and the [reversed-client built-binary journey](tests/golden.rs) |
| One generated plan for dry-run and apply | [Plan-once application tests](src/sync.rs) and the [combined CLI journey](tests/golden.rs) |
| Parseable native output and bounded preservation | [Claude fixtures](tests/fixtures/claude-desktop), [Cursor fixtures](tests/fixtures/cursor), adapter tests, and the [combined CLI journey](tests/golden.rs) |
| Non-zero failure and recovery matrix | [Built-binary failure matrix](tests/golden.rs), [filesystem recovery tests](src/filesystem.rs), and [transaction tests](tests/sync.rs) |
| Structural redaction in every diagnostic surface | [Safe CLI assertions](tests/support/mod.rs), redaction-focused unit tests, and private-value sentinels throughout the built-binary journeys |
| Unit, fixture, filesystem, and golden tests under synthetic roots | [`SyntheticHome`](tests/support/mod.rs), all 140 locked tests, and the canonical quality script's disposable user environment |
| Backup-protected current-stable two-client verification | [Controlled current-client evidence](#controlled-current-client-evidence) for signed Cursor `3.15.6` and Claude Desktop `1.26832.0` |
| README alignment and explicit post-M1 gaps | [README and operational-guidance audit](#readme-and-operational-guidance-audit), [M1 boundary](#m1-mvp-boundary), and the ordered `MCP-014` through `MCP-020`, `MCP-028`, and `MCP-029` tracker rows |

## Target architecture

Start with one crate and preserve dependency direction toward pure domain code:

```text
┌──────────────────────────────────────────────────────────┐
│                     mcp-sync CLI                         │
│ argument parsing, output formatting, exit-code mapping   │
└────────────────────────────┬─────────────────────────────┘
                             ▼
┌──────────────────────────────────────────────────────────┐
│                  Application use cases                   │
│ init · add · list · plan sync · apply sync               │
└───────────────┬──────────────────────┬───────────────────┘
                ▼                      ▼
┌──────────────────────────┐  ┌────────────────────────────┐
│ Canonical model + plan   │  │ Ports                      │
│ validate · normalize     │  │ filesystem · paths         │
│ reconcile · redact       │  │ process runner (later)     │
└───────────────┬──────────┘  └──────────────┬─────────────┘
                ▼                            ▼
┌──────────────────────────┐  ┌────────────────────────────┐
│ Target adapters          │  │ Real OS implementations    │
│ native JSON · TOML       │  │ and deterministic fakes    │
└──────────────────────────┘  └────────────────────────────┘
```

Rules that protect later growth:

- The canonical model contains MCP concepts, not Claude-, Cursor-, or
  operating-system-specific fields.
- A target adapter owns native discovery, parse/render translation, supported
  capability reporting, and its documented merge boundary.
- Reconciliation is pure: current normalized state plus desired canonical state
  produces a deterministic plan.
- The apply layer consumes an already validated plan and owns backup, atomic
  replacement, rollback, and per-target reporting.
- CLI presentation is not a domain API. Structured domain errors map to stable
  user messages and exit codes at the edge.
- Split crates only when there is a real second binary, publishable API, or
  compile-time boundary whose maintenance cost is justified.

### Implemented path and filesystem boundary

`MCP-005` established the operating-system seams, `MCP-009` added create-only
canonical publication, `MCP-010` added guarded single-file replacement, and
`MCP-018` generalized path resolution across macOS and GNU/Linux:

- `ConfigurationPaths` selects only macOS or Linux at the process boundary and
  accepts an injected platform and environment in deterministic fixtures. Other
  operating systems fail closed until their own ticket. `HOME` is required, a
  non-empty `XDG_CONFIG_HOME` overrides its `.config` directory, and both inputs
  must be absolute and free of parent traversal. The canonical file is
  `<config-home>/mcp-sync/config.json` on both platforms. The user-data root is
  `<home>/Library/Application Support` on macOS and the resolved XDG
  configuration home on Linux.
- `FileSystem` exposes reads and the separate `FileCreator` port exposes only
  no-clobber creation. `FileReplacer` accepts the observed and desired bytes
  together, requires regular source and backup paths, rejects symbolic links
  and other non-regular files, checks for concurrent byte changes, publishes
  the exact prior bytes at `<path>.bak`, and atomically replaces from a synced
  same-directory temporary file. `TransactionalFileUpdater` adds opaque
  receipts that can restore an existing target and its pre-transaction backup
  or remove an unchanged newly created target. `OsFileSystem` cleans up
  ordinary temporary artifacts and preserves attempted operation, path, error
  kind, and source in typed errors.
- Eight path fixtures and the automated I/O tests inject deterministic values
  and use `tempfile`-owned directories. They prove macOS and Linux defaults,
  Linux XDG target relocation, exact create and backup bytes, no-clobber
  publication, atomic replacement, stale-byte refusal, non-regular backup
  rejection, symbolic-link refusal, interrupted-write compensation, exact
  target and prior-backup rollback, created-target removal, concurrent rollback
  refusal, and temporary-file cleanup without resolving or accessing the test
  runner's real home or configuration files.

### Implemented reconciliation boundary

`MCP-006` establishes a pure plan between normalized canonical states; it does
not discover clients, parse native formats, access paths, perform I/O, or apply
changes:

- Reconciliation compares validated `CanonicalConfig` values exactly and
  literally. It walks the lexicographically sorted union of server names and
  produces one deterministic `Add`, `Update`, `NoOp`, or `Drift` outcome for
  every name.
- Desired-only definitions are adds, unequal shared definitions are updates,
  and exact matches are stable no-ops. Target-only definitions are preserved as
  non-mutating drift; the plan has no deletion outcome.
- Public plan structure exposes server names, argument counts, environment key
  names, and changed-field categories, but never commands, arguments, or
  environment values. Validated desired definitions remain private plan data
  available only to target application work, and custom debug output omits
  their values.
- Seven focused examples and five bounded property suites prove sorted and
  complete outcomes, determinism, input immutability, insertion-order
  independence, stable no-ops, non-mutating drift, and structural redaction.

### Implemented Claude Desktop boundary

`MCP-007` implements the fixture-backed global Claude Desktop adapter, and
`MCP-018` extends its path contract from macOS to Linux. The controlled
current-client gate remains the macOS proof in `MCP-012` under `DEC-025`:

- The current contract was revalidated on 2026-08-06 against the official
  [local-server setup](https://modelcontextprotocol.io/docs/develop/connect-local-servers),
  [environment configuration](https://modelcontextprotocol.io/docs/tools/debugging),
  and [server examples](https://modelcontextprotocol.io/docs/develop/build-server).
  The [official Linux installation guide](https://support.claude.com/en/articles/10065433-install-claude-desktop)
  now covers x64 and ARM64. Inspection of vendor package `1.24012.11` from that
  guide's official APT repository (amd64 SHA-256
  `99c4bcf5e3f7d0ec44a49fbf24d7d659f2ea46e29c7ec61c77c7298522f57e76`)
  confirms that the application joins `claude_desktop_config.json` to
  Electron's `userData` path; [Electron defines Linux application data through
  `XDG_CONFIG_HOME` with a `~/.config` fallback](https://www.electronjs.org/docs/latest/api/app#appgetpathname).
  The adapter therefore discovers only
  `~/Library/Application Support/Claude/claude_desktop_config.json` on macOS or
  `<config-home>/Claude/claude_desktop_config.json` on Linux through injected
  paths and the read-only filesystem port. A missing file is a normal empty
  target; every other read failure retains its path, kind, and source.
- Local STDIO entries normalize from `mcpServers`; `command` is required while
  omitted `args` and `env` become empty collections. The shared JSON boundary
  rejects duplicate keys at every depth and preserves arbitrary-precision
  numbers in unowned native data.
- The ownership boundary is exactly `mcpServers.<name>.command`, `args`, and
  `env`. Unknown top-level fields and unknown per-server fields such as `cwd`
  survive structurally, and target-only servers remain untouched as drift.
  Remote account connectors and desktop extensions are separate Claude
  mechanisms and remain outside this local JSON adapter.
- Rendering consumes the validated reconciliation plan, rejects plans whose
  presence set does not match the parsed document, applies only add/update
  entries, validates its result by reparsing, and returns original bytes exactly
  for no-op/drift-only work. It performs no filesystem mutation.
- Three checked-in synthetic fixtures and seventeen focused tests prove exact
  discovery, normalization, deterministic rendering, bounded preservation,
  add/update/no-op/drift round trips, missing and malformed behavior, duplicate
  rejection, read failures, plan mismatch failures, and structural redaction.

### Implemented Cursor boundary

`MCP-008` implements the fixture-backed global Cursor adapter; `MCP-018`
confirms the same global path on Linux. The controlled current-client gate
remains the macOS proof in `MCP-012` under `DEC-025`:

- The current contract was revalidated on 2026-08-07 against the official
  [Cursor MCP documentation](https://cursor.com/docs/mcp). The adapter discovers
  only `~/.cursor/mcp.json` through the injected home and read-only
  filesystem port. It accepts no project root, and fixture evidence confirms
  project-level `.cursor/mcp.json` files remain byte-for-byte unchanged.
- Command-based local STDIO entries normalize from `mcpServers`; `command` is
  required for a local entry while omitted `args` and `env` become empty
  collections. Cursor's optional `envFile` remains unowned native data. The
  shared JSON boundary rejects duplicate keys at every depth and preserves
  arbitrary-precision numbers in unowned data.
- Commandless remote HTTP/SSE entries share Cursor's `mcpServers` map but remain
  outside canonical JSON v1. The adapter preserves them structurally, reports
  only their names as unmanaged capability metadata, and rejects a desired
  local addition with the same name instead of producing a mixed or destructive
  native definition.
- The ownership boundary is exactly `mcpServers.<name>.command`, `args`, and
  `env` inside compatible local entries. Unknown root fields, local fields such
  as `type` and `envFile`, remote URL/header/auth data, and target-only servers
  survive. Rendering consumes an already validated plan, reparses its result,
  returns exact original bytes for no-op/drift-only work, and performs no
  filesystem mutation.
- Four checked-in synthetic fixtures and twenty focused tests prove exact
  global discovery, local normalization, unmanaged remote preservation,
  collision refusal, deterministic rendering, add/update/no-op/drift behavior,
  project-file isolation, missing and malformed behavior, duplicate rejection,
  contextual read failures, plan mismatch failures, and structural redaction.

### Implemented Windsurf boundary

`MCP-014` implements the fixture-backed global Windsurf legacy Cascade adapter;
`MCP-018` confirms the same global path on Linux. This is native-contract and
built-binary evidence, not a current-client smoke claim:

- The contract was revalidated on 2026-08-07 against the official
  [Windsurf MCP documentation](https://docs.windsurf.com/windsurf/cascade/mcp).
  The adapter discovers only `~/.codeium/windsurf/mcp_config.json` through the
  injected home and read-only filesystem port. The vendor now labels
  this as legacy Cascade configuration and documents the Devin Local agent as
  a separate configuration surface, so this adapter makes no Devin Local
  support claim.
- Command-based local STDIO entries normalize from `mcpServers`; `command` is
  required while omitted `args` and `env` become empty collections. Literal
  interpolation syntax is preserved without expansion. The shared JSON
  boundary rejects duplicate keys at every depth and preserves arbitrary-
  precision numbers in unowned data.
- Commandless `serverUrl` or `url` HTTP/SSE entries remain outside canonical
  JSON v1. Their URLs, headers, authentication data, `disabledTools`, and
  unknown fields survive structurally; only sorted names are reported, and a
  desired local addition with the same name is rejected.
- The ownership boundary is exactly `mcpServers.<name>.command`, `args`, and
  `env` inside compatible local entries. Unknown root and per-server fields and
  target-only servers survive. Rendering consumes the validated plan,
  verifies its presence set, reparses the result, returns original bytes for
  no-op/drift-only work, and performs no filesystem mutation.
- Three checked-in synthetic fixtures and nineteen focused tests prove exact
  discovery, normalization, deterministic rendering, bounded preservation,
  unmanaged remote handling, collision refusal, interpolation preservation,
  add/update/no-op/drift behavior, missing and malformed behavior, duplicate
  rejection, contextual read failures, plan mismatch failures, reparsing, and
  structural redaction. The expanded built-binary suites prove deterministic
  three-client import and one plan-first, reverse-rollback transaction across
  Claude Desktop, Cursor, and Windsurf.

### Implemented VS Code boundary

`MCP-015` implements the fixture-backed native VS Code default user-profile
adapter; `MCP-018` extends its path contract from macOS to Linux. This is
native-contract and built-binary evidence, not a current-client smoke claim:

- The contract was fixed on 2026-08-07 against the official [VS Code MCP server
  guide](https://code.visualstudio.com/docs/agent-customization/mcp-servers) and
  [configuration
  reference](https://code.visualstudio.com/docs/agents/reference/mcp-configuration).
  [VS Code's platform settings documentation](https://code.visualstudio.com/docs/configure/settings#_settings-file-locations)
  fixes its Linux default user-data path under `~/.config/Code/User`. The
  adapter discovers exactly
  `~/Library/Application Support/Code/User/mcp.json` on macOS or
  `<config-home>/Code/User/mcp.json` on Linux through the injected user-data
  root. It does not discover workspace `.vscode/mcp.json` or `.mcp.json`, named
  profiles under `Code/User/profiles`, remote profiles, VS Code Insiders,
  portable installations, extension-owned Cline or Roo Code stores, Cline
  shared settings, or Agent Host/Copilot CLI configuration.
- Compatible local STDIO entries normalize from root `servers`. Explicit
  `"type": "stdio"` and VS Code's documented minimal command form with omitted
  `type` are accepted; omitted `args` and `env` become empty collections, and
  interpolation placeholders remain literal. New canonical entries receive an
  explicit `"type": "stdio"`.
- The ownership boundary is exactly `servers.<name>.command`, `args`, and
  string-valued `env` inside compatible local entries. Existing `type`, `cwd`,
  `envFile`, `dev`, `sandboxEnabled`, root `inputs` and `sandbox`, unknown root
  or server fields, arbitrary-precision numbers, and target-only servers
  survive structurally.
- Native HTTP/SSE and unknown transports, opaque entries, and valid local
  STDIO entries containing number or null environment values remain unmanaged
  because canonical JSON v1 cannot represent them without coercion. Only
  sorted names are reported; a desired local addition with the same name is
  rejected. Invalid native shapes such as boolean, array, or object environment
  values fail validation before mutation.
- Three checked-in synthetic fixtures and eighteen focused tests prove exact
  discovery and exclusions, normalization, deterministic rendering, explicit
  STDIO additions, bounded preservation, remote and scalar-environment
  handling, collision refusal, interpolation preservation,
  add/update/no-op/drift behavior, missing and malformed behavior, duplicate
  rejection, contextual read failures, plan mismatch failures, reparsing, and
  structural redaction. Expanded built-binary suites prove deterministic
  four-client import and one plan-first transaction across Claude Desktop,
  Cursor, Windsurf, and VS Code, including exact reverse recovery when the
  fourth target fails.

### Implemented Codex boundary

`MCP-016` implements the fixture-backed global Codex host adapter; `MCP-018`
confirms the same global path on Linux. This is native-contract and built-binary
evidence, not a current-client smoke claim:

- The contract was fixed on 2026-08-07 against official [Codex configuration
  precedence](https://developers.openai.com/codex/config-basic/#configuration-precedence)
  and [Codex MCP configuration](https://developers.openai.com/codex/mcp/).
  The adapter discovers exactly `~/.codex/config.toml` through the injected
  user home. That global file is shared by the ChatGPT desktop app, Codex CLI,
  and Codex IDE extension. Higher-precedence trusted-project
  `.codex/config.toml` layers, arbitrary alternate files, system paths, and
  OAuth credential stores remain undiscovered and untouched.
- Compatible local STDIO entries normalize from `mcp_servers` when they have a
  `command` and no `url`; omitted `args` and `env` become empty collections.
  Standard tables, inline tables, and literal interpolation-like text are
  handled without expansion. The adapter never converts TOML through JSON.
- The ownership boundary is exactly `mcp_servers.<name>.command`, `args`, and
  `env` inside compatible local entries. Comments, ordering, formatting,
  unrelated root settings, plug-in configuration, working directories,
  enablement, timeouts, environment forwarding, headers, authentication, and
  every unknown local or root field survive structurally.
- URL-based remote HTTP/OAuth entries, mixed command/URL entries, and opaque
  future transports remain unmanaged. Only sorted names are reported; a
  desired local addition with any unmanaged name is rejected before mutation.
  Rendering consumes the validated plan, verifies native presence and
  unmanaged-name sets, reparses its result, and returns exact original bytes
  for no-op/drift-only work.
- Three checked-in synthetic TOML/JSON fixtures and nineteen focused tests
  prove exact global discovery and exclusions, deterministic bounded edits,
  comment and unknown-field preservation, standard and inline table handling,
  remote/OAuth/mixed/opaque preservation, collision refusal, add/update/no-op/
  drift behavior, missing and malformed behavior, duplicate rejection,
  contextual read failures, plan mismatch failures, reparsing, and structural
  redaction. Expanded built-binary suites prove deterministic five-client
  import, project/profile/credential isolation, exact backup and no-op bytes,
  unmanaged collision and malformed-input preflight, and reverse recovery when
  the fifth target fails.

### Implemented initialization boundary

`MCP-009` implements the first end-to-end application journey through the
Cargo-built binary. This ticket supplies automated macOS-contract evidence;
the controlled current-client gate now belongs to `MCP-012` under `DEC-025`:

- `mcp-sync init` refuses an existing canonical path before client discovery,
  reads only the five implemented global client files, treats missing files
  as empty, and creates `<config-home>/mcp-sync/config.json` only after every
  input has been parsed, normalized, merged, validated, and serialized
  successfully.
- The pure import step sorts client sources and server names, unions unique
  local definitions, and imports an identical shared definition once. Unequal
  definitions produce a stable report naming the server, both clients, and
  only the differing field categories; commands, arguments, and environment
  values never enter diagnostics.
- Unmanaged Cursor, Windsurf, VS Code, and Codex entries remain unrepresentable
  in canonical JSON v1. Their sorted names are reported as skipped per client,
  while a same-name local definition is an actionable conflict rather than an
  implicit conversion, coercion, or overwrite.
- Conflict, malformed-input, permission, and existing-canonical outcomes are
  non-zero and stop before the create boundary. Native Claude Desktop, global
  Cursor, Windsurf, VS Code, Codex, and excluded project/profile/credential/
  extension fixture bytes remain unchanged, and `init` never starts a
  configured server process.
- Six focused import tests plus nine bounded `assert_cmd` journeys cover source-
  order determinism, exact redacted conflicts, empty discovery, malformed JSON
  and TOML, permission failure, no-clobber behavior, native/project/credential
  isolation, unmanaged-name collisions, temporary-file cleanup, and the
  executable non-launch sentinel through `SyntheticHome`.

### Implemented canonical catalog boundary

`MCP-010` implements canonical `add` and `list` as application use cases and
built-binary journeys without beginning target reconciliation:

- `mcp-sync add <name> --command <executable>` accepts the documented `--cmd`
  alias, repeated literal `--arg` values, and repeated `--env KEY=VALUE`
  assignments. Environment input splits only at the first equals sign,
  preserves empty and embedded-equals values, and rejects missing separators or
  duplicate keys without echoing values.
- One invocation supplies a complete definition. It validates that definition
  before reading canonical state, adds a missing name, replaces exactly one
  existing definition, preserves every other server, and serializes stable
  sorted canonical bytes. An exact semantic match is a no-op that preserves the
  original bytes and existing backup even when the input document was not
  canonically formatted.
- A changed definition is fully validated and serialized before the guarded
  replacement boundary receives it. The prior canonical bytes become the
  recoverable `.bak`; malformed or missing state, permissions, backup
  collisions, symbolic links, and detected concurrent edits return non-zero
  without replacing canonical state.
- `mcp-sync list` reads canonical state without mutation and sorts output by
  server name. It prints only escaped server names, argument counts, and
  escaped environment key names; commands, argument contents, and environment
  values stay redacted. `add` reports the same bounded structure.
- Eight focused catalog tests, four filesystem replacement tests, and ten
  five-second `assert_cmd` journeys prove deterministic insertion order,
  complete-definition update, semantic no-op, exact backup behavior, literal
  preservation, structural escaping and redaction, important failure paths,
  target/project-file isolation, temporary cleanup, and process non-execution
  through `SyntheticHome`.

### Implemented sync transaction boundary

`MCP-011` established the first target-writing journey across the two M1
clients; `MCP-014` extended it to Windsurf, `MCP-015` to VS Code, and `MCP-016`
to Codex without changing its safety boundary. The controlled current-client
gate for the M1 clients belongs to `MCP-012` under `DEC-025`:

- `mcp-sync sync --dry-run` validates canonical state first, discovers and
  parses all five native targets, renders and reparses every desired output,
  and reports sorted add, update, no-op, target-only drift, and unmanaged
  Cursor, Windsurf, VS Code, or Codex names without mutating files or exposing
  process values.
- Dry-run and apply consume the same plan type. The plan retains exact observed
  and replacement bytes behind redacted debug surfaces; apply has no read or
  reconciliation capability and cannot independently calculate different
  output. Claude Desktop, Cursor, Windsurf, VS Code, and Codex are applied in
  that deterministic order.
- Semantic no-ops preserve exact native bytes and existing backups. A missing
  changed target is created through no-clobber publication, while an existing
  changed regular file receives its exact prior bytes at `.bak` before synced
  same-directory atomic replacement. Every apply rechecks the planned bytes to
  refuse concurrent edits.
- Each successful mutation returns an opaque receipt. A later target failure
  rolls receipts back in reverse order: created targets are removed only when
  still byte-identical, and replaced targets plus any backup that predated the
  transaction are restored exactly. Rollback failures remain overall failures,
  identify the affected target, and retain an actionable recovery-backup path
  when one exists.
- Fourteen synthetic-home built-binary journeys plus focused sync and filesystem
  tests prove dry-run immutability, five-target success, exact backups, repeat
  no-op behavior, unowned-field and project/profile/credential preservation,
  missing-target recovery, forced second-, third-, fourth-, and fifth-target
  failure rollback, malformed later-input preflight, Cursor, Windsurf, VS Code,
  and Codex unmanaged-name collision refusal,
  stale-plan refusal, interrupted-write compensation, explicit rollback
  failure, structural redaction, temporary cleanup, and process non-execution.

### Implemented Linux platform boundary

`MCP-018` extends the complete source-checkout configuration and bounded health
journeys to GNU/Linux without changing any adapter ownership or filesystem
safety rule:

- Runtime path selection recognizes macOS, Linux, and Windows. Linux resolves a
  non-empty absolute `XDG_CONFIG_HOME`, or `$HOME/.config` otherwise, for the
  canonical file and Electron-style user data. Claude Desktop uses
  `<config-home>/Claude/claude_desktop_config.json`, VS Code uses
  `<config-home>/Code/User/mcp.json`, and Cursor, Windsurf, and Codex retain
  their documented home-relative paths. The separately proven Windows mapping
  does not alter these Linux paths.
- Five adapter path fixtures and three platform-root fixtures prove the exact
  Linux defaults, XDG relocation, and exclusion boundaries without reaching a
  real home. The shared built-binary harness makes `XDG_CONFIG_HOME` distinct
  from `$HOME/.config`, selects native macOS or Linux target paths, and continues
  to inject every user root; its import, catalog, health, dry-run, apply, no-op,
  preservation, redaction, permission, and reverse-order rollback journeys are
  unchanged.
- The bounded-response fixture reads the initialize line before emitting an
  oversized or undelimited response, so it tests framing limits without racing
  process exit against the request write. The [CI quality
  matrix](.github/workflows/ci.yml) uses native `ubuntu-24.04` x64 and
  `ubuntu-24.04-arm` ARM64 runners, asserts the exact Rust host triple, and runs
  the complete [`scripts/check.sh`](scripts/check.sh) synthetic-home gate on
  each architecture. The successful
  [`MCP-018` push run](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31240607296)
  and [pull-request run](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31240608728)
  prove both native jobs plus dependency policy under both event paths. This is
  GNU libc source-checkout behavior evidence, not release packaging,
  Alpine/musl support, or a Linux current-client smoke claim.

### Implemented Windows platform boundary

`MCP-019` extends the same source-checkout configuration, recovery, and bounded
health journeys to native Windows MSVC without changing adapter ownership or
starting restore UX:

- Runtime path selection requires injected, non-empty, absolute,
  traversal-free `%USERPROFILE%`, `%LOCALAPPDATA%`, and `%APPDATA%` values.
  Canonical state uses `%LOCALAPPDATA%\mcp-sync\config.json`; Claude Desktop and
  VS Code use their documented roaming paths; Cursor, Windsurf, and Codex keep
  their documented profile-relative paths. `HOME` and XDG variables neither
  substitute for nor override those Windows roots.
- The same `tempfile`-owned synchronized same-directory publication boundary
  performs no-clobber creation and the operating system's maintained
  replace-existing equivalent. Existing generic filesystem and five-target
  failure tests exercise exact backups, stale-byte refusal, interrupted-write
  compensation, rollback of created and replaced targets, restoration of a
  pre-existing backup, concurrent-edit refusal, and temporary cleanup on the
  native platform.
- Thirteen resolver tests, exact Windows path fixtures in all five adapters, and
  the platform-aware synthetic-home harness keep every read and write beneath
  disposable `%LOCALAPPDATA%`, `%APPDATA%`, and `%USERPROFILE%` roots. A
  Windows-only copied-executable journey runs `init`, guarded canonical `add`,
  five-target `sync`, repeat dry-run, and the `MCP-020` canonical plus Claude
  restore rotation away from the checkout while checking exact backups and
  target placement. Native PowerShell fixtures also exercise
  bounded health success, timeout, malformed output, redaction, and direct-child
  cleanup rather than compiling those operating-system tests out.
- The [CI quality matrix](.github/workflows/ci.yml) uses `windows-2025` x64 and
  public-preview `windows-11-arm` ARM64 runners, asserts the exact MSVC Rust
  host triple, and runs [`scripts/check.ps1`](scripts/check.ps1) through a
  disposable Windows profile. The successful
  [`MCP-019` push run](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31244563467)
  and [pull-request run](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31244565101)
  prove exact-host x64 and ARM64 jobs, the complete Windows-enabled test suite,
  warning-free Clippy, formatting, and dependency policy under both event
  paths. This is source-checkout behavior, not signed packaging, WinGet
  publication, or a Windows current-client smoke claim.

### Implemented restore and retention boundary

`MCP-020` adds one explicit recovery operation without broadening adapter
ownership or release scope:

- `mcp-sync restore <configuration> [--dry-run]` accepts only `canonical`,
  `claude-desktop`, `cursor`, `windsurf`, `vscode`, or `codex`. Each selection
  resolves through the same injected platform paths as normal discovery;
  arbitrary paths, project layers, named profiles, alternate products, and
  credential stores cannot be selected.
- Planning reads only regular target and backup files, allows the target to be
  missing or malformed, requires the adjacent `.bak`, and validates the exact
  backup bytes through the selected strict canonical JSON, native JSON, or
  Codex TOML parser. Plans, reports, debug output, and errors retain paths and
  structural state without process or native values.
- Apply consumes the validated exact-byte pair. Different existing bytes are
  swapped through synchronized same-directory temporary files, leaving the
  prior target as the new backup; a missing target is recreated with
  no-clobber publication while the backup remains unchanged; equal bytes are a
  no-op. Target and backup staleness, symbolic links, non-regular paths,
  permissions, invalid backups, interrupted publication, and compensation are
  fail-closed and covered by focused tests.
- Retention is exactly the immediately preceding generation with no automatic
  expiration. Successful changed `add`, `sync`, and existing-target `restore`
  operations rotate that slot. Creations have no prior-file backup; missing-
  target restore preserves its retained copy; and no-ops, refusals, recovered
  failures, and transaction rollback preserve the pre-operation slot. Longer
  history remains an explicit external archival responsibility.
- The [restore use case and four platform/parser tests](src/restore.rs),
  [guarded filesystem swap, recreation, stale-input, interruption, and
  file-type tests](src/filesystem.rs), [six built-binary exact-byte, malformed-
  current, missing-target, no-op, redaction, permission, and symlink journeys](tests/restore.rs),
  [fixed CLI contract](src/main.rs), and extended [portable Windows journey](tests/windows.rs)
  provide durable evidence. The complete 255-test suite passes on native ARM64
  macOS and under Intel macOS emulation; all test surfaces compile warning-free
  for both macOS, GNU/Linux, and Windows MSVC architectures. The existing
  native Linux and Windows CI matrix will exercise the same journeys on its
  next repository run; this ticket makes no new current-client or release
  claim.

### Proven synthetic M1 journey and failure matrix

`MCP-012` combines the previously isolated commands into one built-binary
journey and audits every automatable M1 failure class against disposable homes:

- [`tests/golden.rs`](tests/golden.rs) runs `init`, complete-definition `add`
  and update, structurally redacted `list`, `sync --dry-run`, transactional
  `sync`, and repeat no-op `sync` as one flow. It proves deterministic import,
  exact dry-run immutability and backups, valid native output, unowned and
  target-only preservation, project-level Cursor, VS Code, and Codex isolation,
  Codex alternate-file and credential-store isolation, redaction, and process
  non-execution.
- The same built-binary suite proves exit code `1`, empty stdout on failure,
  redacted diagnostics, and no unintended mutation for missing canonical
  state, an unsupported schema version, conflicting imports, malformed later
  target input, and a second-target backup failure with reverse rollback and
  restoration of a pre-existing first-target backup.
- Focused [`filesystem`](src/filesystem.rs), [`sync`](src/sync.rs),
  [`initialization`](tests/init.rs), and [`catalog`](tests/catalog.rs) tests
  complete the matrix for malformed JSON, TOML, and UTF-8, absent optional
  targets, required-path errors, permissions, symlinks, non-regular files,
  stale bytes, interrupted replacement and compensation, backup collisions,
  created-target rollback, rollback failure with an actionable recovery path,
  native field preservation, temporary cleanup, and safe test-failure diagnostics.
- The current locked suite contains 228 passing tests, including 36 stateful
  built-binary configuration journeys, and never resolves a real user
  configuration path. The earlier `MCP-012` M1 snapshot from `cargo llvm-cov`
  0.8.7 reported 92.77% line, 91.95% region, and 91.14% function coverage
  across the then-current `src`; coverage remains diagnostic rather than a
  gate and is not presented as post-`MCP-015` measurement.
- A copied-tree, 60-second-per-test bounded `cargo-mutants` 27.1.0 pass over
  reconciliation, plan verification, transactional apply, filesystem guards,
  and rollback evaluated 57 mutants: 49 were caught, eight return-default
  mutations could not compile, and none were missed or timed out. The first
  pass's surviving safety mutations produced the focused regression tests
  above before the clean rerun.

`cargo-nextest` was evaluated and not adopted: once compiled, the complete
140-test suite runs locally in about one second, has no retry or isolation need,
and remains clearest under the documented authoritative `cargo test` command.

### Controlled current-client evidence

The 2026-08-07 macOS ARM64 verification first established the client contract
in `MCP-012`, then `MCP-013` repeated the journey from merged `main` commit
`8877ed9`. The refresh builds the locked binary, runs `init`, `add`, `list`,
`sync --dry-run`, `sync`, and repeat no-op `sync` against a disposable home,
then gives those exact rendered global files to vendor-signed current clients.
It never prints a native configuration or copies authentication state:

- The [Cursor download page](https://cursor.com/en-US/download) identified
  `3.15` as current stable. The downloaded ARM64 `3.15.6` build had SHA-256
  `deaef4ff90e235c7f4c0aa3b7c4bd89e471c827a954586da2aace0ef44bb40a8`,
  passed strict code-signing validation and macOS notarization assessment, and
  ran with disposable home, user-data, extension, and workspace directories.
  It discovered both synced global entries, launched each through its dedicated
  MCP process, completed `initialize`, `notifications/initialized`, and
  `tools/list`, and left the synthetic project-level Cursor sentinel unchanged.
  Metadata for the real global Cursor file remained unchanged.
- Claude Desktop self-updated from `1.25927.0` to `1.26832.0`; the new universal
  build passed strict code-signing validation and notarization assessment, its
  updater reported it current, and the [official download page](https://claude.com/download)
  remained the current vendor source at refresh time. A no-clobber exact backup
  and automatic exit guard protected the real global file while the synthetic
  synced file was installed atomically for the authenticated smoke. The app
  launched both synced entries; each completed `initialize`,
  `notifications/initialized`, and `tools/list`, then stopped when the app quit.
  The guard restored the immediately preceding native digest and bytes exactly
  with its original mode, and the temporary backup was removed from the live
  path. No credential, account data, or configuration value was inspected or
  printed.
- The [current Claude local-server guidance](https://support.claude.com/en/articles/10949351-getting-started-with-local-mcp-servers-on-claude-desktop)
  places connection status in Developer settings. The observed client-to-server
  initialization and tool-list exchange provides direct runtime evidence for
  both synthetic entries without exposing the account or native document.

### README and operational-guidance audit

The [M1 usage and recovery guide](docs/m1-usage-and-recovery.md) publishes the
current source-checkout workflow, exact managed paths, redaction boundary,
plan/apply sequence, one-slot backup behavior, automatic transaction recovery,
failure playbook, built-in cross-platform restore journey, explicit retention
rules, and guarded manual JSON and TOML fallbacks. `MCP-020` replaces the prior
manual-only recovery gap while retaining those procedures for a temporarily
unavailable binary.

The README remains the product destination under `DEC-009`. Its corresponding
M1 surfaces were audited without adding a current-status banner or weakening
its finished-product story:

| README surface | Delivery audit |
| --- | --- |
| Canonical JSON v1 | Matches the implemented strict local-STDIO schema and deterministic serialization contract |
| `init`, complete-definition `add`, and redacted `list` | Match the delivered CLI, validation, import, no-clobber, and structural-output behavior |
| `sync --dry-run`, `sync`, backups, rollback, and no-ops | Match the delivered five-target macOS, Linux, and Windows plan/apply transaction; Claude Desktop and Cursor additionally retain macOS current-client verification |
| `mcp-sync test` and STDIO health testing | Matches the explicit named-server five-second initialize boundary, structural protocol validation and redaction, and guaranteed direct-child cleanup; `init`, `sync`, and `restore` remain configuration-only |
| Windsurf | Delivered for the documented global legacy Cascade JSON contract with fixture and built-binary evidence; no Devin Local or current-client smoke claim |
| VS Code | Delivered for the native default user-profile `mcp.json` `servers` contract with fixture and built-binary evidence; project/profile/remote/Insiders/portable/extension-owned/Agent Host surfaces remain excluded, and there is no current-client smoke claim |
| Codex | Delivered for global `~/.codex/config.toml` with TOML-native fixture and built-binary evidence; project layers, remote/OAuth entries, credential stores, and unsupported fields remain outside ownership, and there is no current-client smoke claim |
| Linux | Implemented for all five global targets with deterministic path and built-binary behavior coverage plus a successful native x64/ARM64 whole-suite CI run; there is no Linux current-client smoke claim |
| Windows | Implemented for all five global targets with deterministic path, replacement, rollback, copied-binary, and PowerShell health fixtures plus successful native MSVC x64/ARM64 whole-suite CI; there is no Windows current-client smoke claim |
| Explicit restore and backup retention | Matches the delivered six-selection parser-validated dry-run/apply journey, exact existing-target swap, missing-target recreation, no-op behavior, and documented one-generation retention contract; arbitrary paths and unmanaged configuration layers remain excluded |
| GitHub Releases, Homebrew, WinGet, and Cargo | North-star distribution only. `MCP-028` and `MCP-029` own the narrower zero-cost GitHub, Cargo, and source-built Homebrew release; project-issued macOS/Windows binaries and WinGet remain unclaimed until funded `SIDE-006` evidence exists |
| Source-checkout contributor commands | Match the stable-toolchain locked build, quality script, and dependency-policy checks |

`MCP-015` refined the README's desired VS Code surface from an ambiguous
Cline/Roo/global-storage reference to VS Code's native user-profile `mcp.json`
contract. Implementation gaps stay here and in the operational guide, where
they are current-state facts rather than qualifications to the public product
promise.

## Deliverables

| ID | Deliverable | Milestone | Owner | Target | Status | Completion evidence |
| --- | --- | --- | --- | --- | --- | --- |
| D-01 | Project operating model and north-star product page | M0 | Repository | 2026-08-06 | Done | [AGENTS.md](AGENTS.md), this tracker, and [README.md](README.md) |
| D-02 | Runnable Rust CLI skeleton | M0 | Codex | 2026-08-06 | Done | [Cargo.toml](Cargo.toml), [src/main.rs](src/main.rs), [CLI smoke tests](tests/cli.rs), [Cargo.lock](Cargo.lock), and locked build/install verification |
| D-03 | Deterministic test and CI foundation | M0 | Codex | 2026-08-06 | Done | [Local POSIX gate](scripts/check.sh), [native Windows gate](scripts/check.ps1), [native x64/ARM64 Linux and Windows CI workflow](.github/workflows/ci.yml), [`tempfile`-owned synthetic-home CLI harness](tests/support/mod.rs), [cross-platform path fixtures](src/paths.rs), and [successful baseline CI run](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31137308671) |
| D-04 | Versioned canonical configuration contract | M0 | Codex | 2026-08-06 | Done | [Strict canonical model and tests](src/config.rs), [canonical v1 example](examples/config.v1.json), [public configuration contract](README.md), [accepted version policy](#canonical-configuration-v1-decision), and [dependency policy](deny.toml) |
| D-05 | Two-client import and conflict reporting | M1 | Codex | 2026-08-07 | Done | [Initialization use case](src/init.rs), [CLI command](src/main.rs), [global Claude Desktop adapter](src/claude_desktop.rs), [global Cursor adapter](src/cursor.rs), [create-only filesystem boundary](src/filesystem.rs), and [built-binary journeys](tests/init.rs) |
| D-06 | Redacted plan and safe multi-target apply | M1 | Codex | 2026-08-07 | Done | [Plan-once sync use case and focused transaction tests](src/sync.rs), [reversible exact-byte filesystem boundary and recovery tests](src/filesystem.rs), [seven isolated built-binary sync journeys](tests/sync.rs), [wired CLI](src/main.rs), and [public sync contract](README.md) |
| D-07 | Complete M1 CLI journey and user guide | M1 | Codex | 2026-08-07 | Done | The combined [golden built-binary journey and failure matrix](tests/golden.rs), command-specific journeys, audited [README contract](README.md), refreshed [controlled current-stable two-client smoke](#controlled-current-client-evidence), and tested [M1 usage and recovery guide](docs/m1-usage-and-recovery.md) complete the M1 journey |
| D-08 | Five-client, cross-platform support matrix | M2 | Codex | 2026-08-08 | Done | All five global macOS, GNU/Linux, and Windows targets pass deterministic path, fixture, built-binary import/sync, preservation, and rollback coverage: four JSON adapters plus the TOML-native Codex adapter. Native x64/ARM64 whole-suite CI proves GNU/Linux through the `MCP-018` [push](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31240607296) and [pull-request](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31240608728) runs, and Windows MSVC through the `MCP-019` [push](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31244563467) and [pull-request](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31244565101) runs. Claude Desktop and Cursor retain macOS current-client evidence; no Linux or Windows client has a current-client smoke claim, and Windsurf, native default-profile VS Code, and Codex have none on any implemented platform. `MCP-028` and `MCP-029` complete the zero-cost release packaging and publication evidence |
| D-09 | Bounded STDIO health testing | M2 | Codex | 2026-08-07 | Done | [Bounded initialize use case, replaceable process boundary, structural protocol parser, and 11 focused tests](src/health.rs), [five synthetic-home built-binary success and failure journeys](tests/health.rs), [Clap wiring](src/main.rs), and [source-checkout health contract](docs/m1-usage-and-recovery.md) |
| D-10 | Explicit restore and one-generation backup retention | M2 | Codex | 2026-08-08 | Done | [Fixed restore CLI and validated use case](src/restore.rs), [exact-byte guarded filesystem boundary](src/filesystem.rs), [six isolated built-binary recovery journeys](tests/restore.rs), [portable Windows recovery journey](tests/windows.rs), [public command contract](README.md), [cross-platform usage and retention contract](docs/m1-usage-and-recovery.md), and `DEC-034`; format, warning-free Clippy, all 255 synthetic-home tests, `cargo deny`, ARM64 and Intel macOS suites, all six target compilation surfaces, and documentation, redaction, and process-boundary checks pass |
| D-11 | Funded signed-native release foundation | Post-M2 stretch | Unassigned | Funding available | Deferred | The active `v*` ruleset restricts stable-tag creation, update, and deletion to the named administrator bypass; the tag-only `release` environment requires review and accepts only `v*`; and repository release immutability is enabled. Merged [PR #24](https://github.com/EnjoyableWork/mcp-sync/pull/24) supplies the [packaging scripts](scripts/package-release.sh), [six-target unsigned preflight](.github/workflows/release-preflight.yml), [funded signed workflow](.github/workflows/release.yml), [release runbook](docs/release.md), and installed restore smoke. Merged [PR #25](https://github.com/EnjoyableWork/mcp-sync/pull/25) adds the [deterministic downstream generator](scripts/generate-release-channels.sh) and [read-only all-channel native smoke](.github/workflows/release-channels.yml). Exact release-code `main` commit [`c923e95`](https://github.com/EnjoyableWork/mcp-sync/commit/c923e9514c8fe73b5ea0f4fcd2ee63ce7083e9b3) passes [CI](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31265151474) and all six native [preflight jobs](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31265151478); `MCP-028` additionally proves the retained path on exact implementation commit [`133e5c2`](https://github.com/EnjoyableWork/mcp-sync/commit/133e5c2ce343f3d047c76b5d4f0bd589a9ee2604). `DEC-036` preserves this foundation for `SIDE-006`; completion remains deferred until funding supplies both signing authorities and the signed GitHub, binary-backed Homebrew, and WinGet paths pass their native release evidence |
| D-12 | Zero-cost market-validation release and installed recovery exercise | M2 | Codex | 2026-08-08 | Done | Exact release commit [`0836590`](https://github.com/EnjoyableWork/mcp-sync/commit/08365900497bf6fde3749695078794e0f3ecf790) passes [CI](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31283240167), [source and GNU/Linux preflight](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31283240166), retained [six-target preflight](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31283240132), and protected [authorization](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31283455962). Annotated tag `v0.1.0` resolves to that commit, and the [source and GNU/Linux publisher](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31284055636) publishes the [immutable seven-asset GitHub Release](https://github.com/EnjoyableWork/mcp-sync/releases/tag/v0.1.0) with verified release/build attestations, checksum manifest, and only GNU/Linux x64/ARM64 native archives. [`enjoyable-mcp-sync` `0.1.0`](https://crates.io/crates/enjoyable-mcp-sync/0.1.0) is public, not yanked, and byte-identical to the release asset; its one-use `publish-new` token is revoked server-side and removed locally. After the first Homebrew dispatch failed before credential use or tap mutation, merged [PR #31](https://github.com/EnjoyableWork/mcp-sync/pull/31) binds the corrected recovery commit [`2dbe791`](https://github.com/EnjoyableWork/mcp-sync/commit/2dbe7915817d9525cfe92ccd3949fc3327f04458) to successful exact-main [CI](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31285564467), [source and GNU/Linux preflight](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31285564476), and retained [six-target preflight](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31285564464). The separately authorized signed annotated operational tag [`v-mcp-029-homebrew-recovery-1`](https://github.com/EnjoyableWork/mcp-sync/tree/v-mcp-029-homebrew-recovery-1) resolves directly to that exact current-main commit without a product release; the protected [recovery publisher](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31287106846) preserves the unchanged tag-only environment and creates only the exact attested formula at tap commit [`ee65157`](https://github.com/EnjoyableWork/homebrew-tap/commit/ee65157539da4c65bd3f8401d7268c794d916760). The final read-only [13-job channel matrix](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31289188547) passes immutable metadata verification, two GNU/Linux GitHub archive smokes, six native Cargo installs, four native source-building Homebrew installs, and every installed recovery journey. No project-issued macOS or Windows binary, WinGet package, or paid-signing workflow was published or activated |
| D-13 | Evidence-backed enterprise assurance baseline | M3 | Codex | 2026-08-09 | Done | `MCP-030` through `MCP-034` establish and verify the contributor, repository, security, community, licensing, supply-chain, artifact, organization-access, strong-MFA, least-privilege, automation, and private-recovery controls without publishing identities or recovery details. Protected [PR #41](https://github.com/EnjoyableWork/mcp-sync/pull/41) publishes the complete [24-control crosswalk and maintenance contract](docs/security-assurance.md), reviewed [BadgeApp proposal](.bestpractices.json), official dynamic README badge, exact seven-asset SLSA v1.0 Build Level 2 statement, strengthened distribution verifier, focused regressions, and exact-main public verifier. Its merge commit [`315bf5c`](https://github.com/EnjoyableWork/mcp-sync/commit/315bf5c908d3a8d6f10f3f937fe2dcc9c4839c47) publicly renders the badge and evidence; the owner-authorized [assessment](https://www.bestpractices.dev/en/projects/14014/baseline-1) reports all 24 controls `Met`, criteria `v2026.02.19`, `baseline-1` achieved on 2026-08-09, and no lost-status timestamp. Exact-main [CI](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31336765585), [six-target release preflight](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31336765560), and [source and GNU/Linux preflight](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31336765599) pass. The corrected verifier passes against that exact public commit and confirms the dynamic shield, destination, 24/24 form, tracked proposal, contract, scope, terminology, and rendering. The statement makes no OSPS Level 2 or 3, independent-certification, regulatory, OpenChain, NIST-certification, or paid-signing claim; `MCP-035`, D-13, and M3 are complete without activating a successor. |

## Ticket board

`Unassigned` is intentional: it makes missing ownership visible rather than
implying a commitment.

### Main-story board

This table is the implementation order. Each ticket depends on its immediate
predecessor, so only the first incomplete row can become `Ready`.

| ID | Outcome | Milestone | Priority | Owner | Status | Predecessor | Done when / evidence |
| --- | --- | --- | --- | --- | --- | --- | --- |
| MCP-001 | Establish repository guidance, living tracker, Rust direction, and north-star README role | M0 | P0 | Repository | Done | — | [AGENTS.md](AGENTS.md), this tracker, [README.md](README.md), and documentation checks |
| MCP-002 | Bootstrap one Rust binary crate with CLI help and version output | M0 | P0 | Codex | Done | `MCP-001` | Registry name verified; [manifest](Cargo.toml), [binary](src/main.rs), [smoke tests](tests/cli.rs), and [lockfile](Cargo.lock); format, Clippy, tests, locked build, help/version, isolated install, RustSec audit, and dependency-license inventory pass |
| MCP-003 | Add format, Clippy, test, and CI quality gates using a synthetic home | M0 | P0 | Codex | Done | `MCP-002` | [Documented local gate](README.md), [quality script](scripts/check.sh), [CI workflow](.github/workflows/ci.yml), [unit and isolated CLI tests](tests), and [successful CI run](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31137308671) |
| MCP-004 | Define the versioned canonical server model and JSON validation contract | M0 | P0 | Codex | Done | `MCP-003` | [Strict model, duplicate-safe parser, canonical serializer, typed/redacted errors, and 17 focused tests](src/config.rs), [tested v1 example](examples/config.v1.json), [manifest](Cargo.toml), [lockfile](Cargo.lock), [dependency policy](deny.toml), and [pinned CI gate](.github/workflows/ci.yml); local format, Clippy, 21 tests, locked build, and all dependency checks pass |
| MCP-005 | Introduce injectable config-path and filesystem boundaries | M0 | P0 | Codex | Done | `MCP-004` | [Injected resolver and six disposable macOS path tests](src/paths.rs), [read-only port and four adapter/error tests](src/filesystem.rs), [`tempfile`-owned synthetic-home enforcement](tests/support/mod.rs), [manifest](Cargo.toml), and [lockfile](Cargo.lock); local format, Clippy, 31 tests, build/help, dependency review, and policy checks pass |
| MCP-006 | Build the pure normalized reconciliation and redacted plan engine | M1 | P0 | Codex | Done | `MCP-005` | [Pure sorted engine, redacted plan types, seven focused examples, and five bounded property suites](src/reconciliation.rs), [development dependency](Cargo.toml), and [lockfile](Cargo.lock); local format, Clippy, 43 tests, locked build, dependency review, and policy checks pass |
| MCP-007 | Implement the global Claude Desktop macOS adapter | M1 | P0 | Codex | Done | `MCP-006` | [Global discovery, strict native parsing, plan-driven bounded rendering, and 16 focused tests](src/claude_desktop.rs), [exact current/desired/merged fixtures](tests/fixtures/claude-desktop), [arbitrary-precision JSON configuration](Cargo.toml), [lockfile](Cargo.lock), and [read-only boundary](src/filesystem.rs); local format, Clippy, 59 tests, locked build, dependency review, and policy checks pass |
| MCP-008 | Implement the global Cursor macOS adapter | M1 | P0 | Codex | Done | `MCP-007` | [Global-only discovery, local/unmanaged parsing, plan-driven bounded rendering, and 19 focused tests](src/cursor.rs), [exact current/desired/merged and project-isolation fixtures](tests/fixtures/cursor), and [read-only filesystem boundary](src/filesystem.rs); format, Clippy, all 78 tests through a synthetic home, locked build, dependency policy, fixture, and documentation checks pass |
| MCP-009 | Implement `init` discovery, import, normalization, and conflict reporting | M1 | P0 | Codex | Done | `MCP-008` | [Deterministic import and redacted conflict use case](src/init.rs), [wired CLI](src/main.rs), [no-clobber canonical creation](src/filesystem.rs), [isolated built-binary success and no-mutation failure journeys](tests/init.rs), [synthetic-home helpers](tests/support/mod.rs), [manifest](Cargo.toml), and [lockfile](Cargo.lock); format, Clippy, all 93 tests, locked build, dependency policy, redaction, and documentation checks pass |
| MCP-010 | Implement `add` and redacted `list` against the canonical config | M1 | P0 | Codex | Done | `MCP-009` | [Deterministic and redacted catalog use cases](src/catalog.rs), [wired CLI](src/main.rs), [guarded backup and atomic replacement](src/filesystem.rs), [ten isolated built-binary journeys](tests/catalog.rs), and [public command contract](README.md); format, warning-free Clippy, all 115 tests through a synthetic home, locked build, dependency policy, redaction, filesystem safety, and documentation checks pass |
| MCP-011 | Implement `sync --dry-run` and safe apply with backup and transaction recovery | M1 | P0 | Codex | Done | `MCP-010` | [Plan-once dry-run/apply orchestration and four focused tests](src/sync.rs), [reversible atomic filesystem receipts and five focused recovery tests](src/filesystem.rs), [seven synthetic-home success, no-op, redaction, and forced-failure journeys](tests/sync.rs), [Clap wiring](src/main.rs), and [README contract](README.md); format, warning-free Clippy, all 131 tests, `cargo deny`, redaction, filesystem safety, and documentation checks pass |
| MCP-012 | Prove the golden MVP journey and failure matrix | M1 | P0 | Codex | Done | `MCP-011` | [Three combined built-binary journeys](tests/golden.rs), [safe synthetic-home diagnostics](tests/support/mod.rs), and focused [filesystem](src/filesystem.rs) / [sync](src/sync.rs) regressions prove the automatable M1 journey and failure matrix; [controlled current-client evidence](#controlled-current-client-evidence) proves signed current-stable Cursor `3.15.6` and Claude Desktop `1.26832.0` each initialize both rendered entries while the real files remain isolated or exactly restored. All 140 locked tests plus stable-toolchain build, format, warning-free Clippy, `cargo deny`, and documentation checks pass; coverage is 92.77% lines / 91.95% regions / 91.14% functions; bounded mutation results are 49 caught, 8 compile-unviable, 0 missed, and 0 timed out; the README audit and `cargo-nextest` decision are recorded |
| MCP-013 | Verify M1 against current clients and the north-star README, then publish detailed usage and recovery guidance | M1 | P0 | Codex | Done | `MCP-012` | The merged binary passes the full synthetic suite and refreshed [current-stable two-client smoke](#controlled-current-client-evidence); the [README audit and tested M1 usage and recovery guide](#readme-and-operational-guidance-audit) record the delivered contract, safe recovery, and current limitations. Stable-toolchain build, format, warning-free Clippy, all 140 locked tests, `cargo deny`, documentation structure and links, the manual restore procedure, redaction, and exact client-config restoration pass; `trycmd` was evaluated and not adopted |
| MCP-014 | Add the Windsurf target adapter | M2 | P1 | Codex | Done | `MCP-013` | [Global legacy Cascade discovery, strict native parsing, plan-driven bounded rendering, and 18 focused tests](src/windsurf.rs), [exact current/desired/merged fixtures](tests/fixtures/windsurf), [three-client initialization journeys](tests/init.rs), and [three-target plan/apply, preflight, collision, and reverse-rollback journeys](tests/sync.rs) extend the [combined golden flow](tests/golden.rs) while preserving unowned and unmanaged data. Stable-toolchain locked build, format, warning-free Clippy, all 160 tests through synthetic homes, `cargo deny`, fixture, redaction, README-alignment, and documentation checks pass; the legacy Cascade-only boundary and absence of a Devin Local or current-client smoke claim are explicit |
| MCP-015 | Add the VS Code target adapter and define extension-shape boundaries | M2 | P1 | Codex | Done | `MCP-014` | [Native default user-profile discovery, strict parsing, plan-driven bounded rendering, and 17 focused tests](src/vscode.rs), [exact current/desired/merged fixtures](tests/fixtures/vscode), [four-client initialization and exclusion journeys](tests/init.rs), and [four-target plan/apply, preflight, collision, and reverse-rollback journeys](tests/sync.rs) extend the [combined golden flow](tests/golden.rs) while preserving root, local, unmanaged, target-only, project, profile, and extension-owned data. Stable-toolchain locked build, format, warning-free Clippy, all 181 tests through synthetic homes, `cargo deny`, fixture, redaction, README-alignment, and documentation checks pass; the native default-profile-only boundary and absence of a current-client smoke claim are explicit |
| MCP-016 | Add the Codex adapter for the shared ChatGPT desktop, Codex CLI, and IDE host configuration | M2 | P1 | Codex | Done | `MCP-015` | [Global-only TOML discovery, structural parsing/rendering, and 18 focused tests](src/codex.rs), [exact current/desired/merged plus project-isolation fixtures](tests/fixtures/codex), [five-client initialization and exclusion journeys](tests/init.rs), and [five-target plan/apply, preflight, collision, exact backup/no-op, and reverse-rollback journeys](tests/sync.rs) extend the [combined golden flow](tests/golden.rs) while preserving comments, formatting, unrelated settings, unowned local fields, remote/OAuth/mixed/opaque entries, project layers, alternate files, and credential stores without JSON conversion. The explicit `toml_edit` parse/display dependency is recorded in [Cargo.toml](Cargo.toml) and [Cargo.lock](Cargo.lock), and the [source-checkout recovery guide](docs/m1-usage-and-recovery.md) includes a tested fail-closed TOML restore procedure. Stable-toolchain locked build, synthetic-home format, warning-free Clippy, all 205 tests, `cargo deny`, dependency review, fixture, redaction, README-alignment, recovery, and documentation checks pass; the global-only boundary and absence of a current-client smoke claim are explicit |
| MCP-017 | Add bounded MCP STDIO initialize health testing | M2 | P1 | Codex | Done | `MCP-016` | [Bounded initialize orchestration, replaceable operating-system process boundary, duplicate-safe protocol validation, typed redacted failures, and 11 focused tests](src/health.rs), [five built-binary handshake, timeout, malformed-output, server-error, spawn-failure, redaction, and direct-child reap journeys](tests/health.rs), [single-command Clap wiring](src/main.rs), [public contract](README.md), [operational contract](docs/m1-usage-and-recovery.md), and `DEC-029`. The response is bounded to five seconds and 1 MiB, shutdown to 500 milliseconds before forced reap, and only canonical environment plus a fallback `PATH` reaches the child. Static process-boundary inspection confirms only `test` can spawn; the existing built-binary `init` and `sync` sentinels remain green. Stable-toolchain locked build, synthetic-home format, warning-free Clippy, all 221 tests, `cargo deny`, help, protocol, process cleanup, redaction, README-alignment, and documentation checks pass; no stateless `2026-07-28`, remote transport, later operation, current-server smoke, or platform-expansion claim is made |
| MCP-018 | Add Linux path and behavior support | M2 | P1 | Codex | Done | `MCP-017` | The [platform-neutral resolver and eight path tests](src/paths.rs), exact Linux path fixtures in all five target adapters, distinct-XDG [built-binary harness](tests/support/mod.rs), request-sequenced [bounded-response regression](src/health.rs), and [native x64/ARM64 CI matrix](.github/workflows/ci.yml) prove the complete source-checkout configuration and bounded-health behavior on GNU/Linux. Both the [push](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31240607296) and [pull-request](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31240608728) event paths pass exact-host x64 and ARM64 jobs with all 228 tests, formatting, warning-free Clippy, and dependency policy. The focused regression also passed 100 consecutive local runs. Local macOS and native ARM64 gates, portable JSON/TOML recovery exercises, redaction, documentation, and diff checks pass. This ticket excludes Windows evidence, which is recorded under `MCP-019`, plus musl/Alpine, release packaging, and a Linux current-client smoke claim |
| MCP-019 | Add Windows path and replacement behavior support | M2 | P1 | Codex | Done | `MCP-018` | The [platform resolver](src/paths.rs), exact Windows fixtures in all five target adapters, [synthetic-home harness](tests/support/mod.rs), [copied-binary Windows journey](tests/windows.rs), [PowerShell quality gate](scripts/check.ps1), and unchanged [filesystem](src/filesystem.rs) / [five-target transaction](src/sync.rs) regressions prove deterministic paths, no-clobber creation, exact backups, guarded replacement, interrupted-write compensation, rollback of created and replaced targets, pre-existing-backup restoration, reverse-order later-target recovery, redaction, and bounded PowerShell health cleanup. Both the native MSVC x64/ARM64 [push](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31244563467) and [pull-request](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31244565101) event paths pass exact-host checks, the complete Windows-enabled suite, formatting, warning-free Clippy, and dependency policy. The local 238-test gate, `cargo deny`, both MSVC cross-target Clippy checks, fixture and documentation checks also pass. Restore UX, signed packaging, WinGet, and a Windows current-client smoke remain explicitly unclaimed |
| MCP-020 | Add explicit restore and backup-retention UX | M2 | P1 | Codex | Done | `MCP-019` | [Fixed-selection restore planning and application](src/restore.rs), [regular-file guarded swap, recreation, compensation, and focused failure tests](src/filesystem.rs), [Clap boundary tests](tests/cli.rs), [six isolated built-binary recovery journeys](tests/restore.rs), and the extended [portable Windows journey](tests/windows.rs) prove non-mutating dry runs, exact reversible replacement, missing-target recreation without backup consumption, no-ops, malformed-current recovery, fail-closed invalid, missing, unreadable, symbolic-link, non-regular, stale, and concurrently changed inputs, parser coverage for canonical state plus all five clients, structural redaction, and fixed global path resolution on macOS, GNU/Linux, and Windows. The [README](README.md), [operational guide](docs/m1-usage-and-recovery.md), and `DEC-034` document one adjacent generation with no expiry and external archival for longer history. Format, warning-free Clippy, all 255 tests through a synthetic home, `cargo deny`, native ARM64 plus Intel macOS suites, warning-free compilation of every test surface for all six supported targets, relative-link and structure checks, and static process-boundary inspection pass; no release channel or current-client smoke claim was added |
| MCP-021 | Publish accessible, provenance-linked release channels | M2 | P1 | Codex | Superseded | `MCP-020` | `DEC-036` replaces this all-at-once, paid-signing-dependent outcome with `MCP-028` and `MCP-029` at the same story point. Its merged tag protections, signed workflow, six-target preflight, packaging, downstream metadata, all-channel smoke, runbook, and exact-main evidence remain durable foundations; the uncompleted signed-native publication criteria move intact to funding-dependent `SIDE-006` rather than being weakened or discarded |
| MCP-028 | Establish the zero-cost market-validation release path | M2 | P1 | Codex | Done | `MCP-020` | Exact implementation commit [`133e5c2`](https://github.com/EnjoyableWork/mcp-sync/commit/133e5c2ce343f3d047c76b5d4f0bd589a9ee2604) and draft [PR #27](https://github.com/EnjoyableWork/mcp-sync/pull/27) add the exact-`v0.1.0` [source and GNU/Linux publisher](.github/workflows/source-linux-release.yml), credential-free [source and GNU/Linux preflight](.github/workflows/source-linux-release-preflight.yml), read-only [published-channel verifier](.github/workflows/source-linux-release-channels.yml), deterministic [Cargo/Homebrew generator](scripts/generate-source-linux-release-channels.sh), exact [prepublication](scripts/verify-source-linux-release-assets.sh) and [published](scripts/verify-published-source-linux-release.sh) payload validators, and accurate [source/Linux](docs/source-linux-release.md), [funded](docs/release.md), and [v0.1.0](docs/releases/v0.1.0.md) guidance. A normal tag cannot invoke the manually dispatched funded workflow, which requires explicit confirmation and rejects `v0.1.0`; the full six-target signed path remains intact. The exact commit passes [source and GNU/Linux preflight](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31268357596) on six Cargo hosts, four Homebrew hosts, both represented GNU/Linux archive/SBOM paths, and exact payload assembly; the retained [six-target release preflight](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31268357599), pull-request [CI](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31268357600), and push [CI](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31268355689) also pass. Local format, warning-free Clippy, all 264 tests, `cargo deny`, Actionlint, ShellCheck, six-target compilation, deterministic double packaging/generation, exact seven-asset acceptance plus three extra-asset rejection cases, exact-package installed recovery, Homebrew style, Cargo publication dry-run, and diff checks pass; no tag, release, crate, formula, or unsigned macOS/Windows binary was published |
| MCP-029 | Publish and verify the zero-cost `v0.1.0` market-validation release | M2 | P1 | Codex | Done | `MCP-028` | The protected [immutable `v0.1.0` GitHub Release](https://github.com/EnjoyableWork/mcp-sync/releases/tag/v0.1.0), byte-identical unyanked [`enjoyable-mcp-sync` `0.1.0`](https://crates.io/crates/enjoyable-mcp-sync/0.1.0), and exact [`EnjoyableWork/tap/mcp-sync` formula](https://github.com/EnjoyableWork/homebrew-tap/blob/ee65157539da4c65bd3f8401d7268c794d916760/Formula/mcp-sync.rb) are public. Release/build attestations, checksums, the tap-scoped protected recovery, and the final [13-job native channel matrix](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31289188547) prove two GNU/Linux archive, six Cargo, four Homebrew, and all represented installed recovery journeys. Public identities and accurate release limitations are reverified; no project-issued macOS or Windows binary, WinGet package, or paid-signing path was published or activated; durable evidence is recorded in `D-12`; and the M2 exit and ticket-done gates pass |
| MCP-030 | Protect the default branch and define a contributor-compatible merge policy | M3 | P1 | Codex | Done | `MCP-029` | Active public ruleset [`Protect main`](https://github.com/EnjoyableWork/mcp-sync/rules/20601003), the [repository-governance contract and path-test record](docs/repository-governance.md), credential-free [public](scripts/verify-public-main-ruleset.sh) and authenticated read-only [operator](scripts/verify-main-repository-controls.sh) verifiers, and [three focused regressions](tests/governance.rs) prove the exact no-bypass pull-request, strict five-check GitHub Actions app binding, conversation-resolution, deletion, and non-fast-forward policy. [PR #33](https://github.com/EnjoyableWork/mcp-sync/pull/33) proves the normal zero-approval protected merge path; its implementation head passes [CI](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31295984170), [six-target release preflight](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31295984167), and [source and GNU/Linux preflight](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31295984173). Administrator-visible rule suites `3609610550`, `3609600890`, `3609601135`, and `3609601321` record rejected direct-`main`, disposable fast-forward, force-push, and deletion attempts; the scoped emergency-selector drill restored and reverified the exact rule. Stable-tag and both release-environment fingerprints remained byte-for-byte unchanged. The accepted zero-approval, three-merge-method, optional-signature, and explicit-administration choices plus this evidence prove `OSPS-AC-03.01` and `OSPS-AC-03.02`. |
| MCP-031 | Establish vulnerability disclosure, security contacts, and secret prevention | M3 | P1 | Codex | Done | `MCP-030` | The GitHub-recognized [security policy](SECURITY.md), stable [private reporting route](https://github.com/EnjoyableWork/mcp-sync/security/advisories/new), [repository-security control and OSPS record](docs/repository-security.md), authenticated non-disclosing [operator verifier](scripts/verify-repository-security-controls.sh), and [three focused regressions](tests/governance.rs) define supported versions, security contacts without personal contact credentials, safe fallback routing, redacted report inputs, response targets, coordinated disclosure, exact live settings, and the later-ticket boundary. Dependency graph and alerts, Dependabot security updates, CodeQL default setup for Rust and Actions, provider-pattern secret scanning, push protection, and private vulnerability reporting are enabled; the verifier recorded a zero-open-finding baseline on 2026-08-09 without exposing payloads. GitHub Free leaves non-provider patterns and partner-pattern validity checks disabled, with the exact GitHub Team plus Secret Protection limitation documented; scheduled dependency updates remain `MCP-033` scope. Protected [PR #34](https://github.com/EnjoyableWork/mcp-sync/pull/34) passes [CodeQL](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31298035849), [CI](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31298036662), [source and GNU/Linux preflight](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31298036647), and [six-target release preflight](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31298036651); local format, warning-free Clippy, all 272 tests, `cargo deny`, ShellCheck, both repository-control verifiers, and documentation checks pass. Stable-tag and both release-environment fingerprints remain byte-for-byte unchanged. This evidence proves `OSPS-BR-07.01` and `OSPS-VM-02.01` only; no assurance badge, certification, or complete-baseline claim is published. |
| MCP-032 | Publish the public contribution, community, repository, and licensing contract | M3 | P1 | Codex | Done | `MCP-031` | The [public project contract and dated recognition record](docs/project-community-and-licensing.md), [contribution guide](CONTRIBUTING.md), [conduct contract](CODE_OF_CONDUCT.md), [support entry point](SUPPORT.md), three structured [issue forms](.github/ISSUE_TEMPLATE), [pull-request template](.github/pull_request_template.md), credential-free [live verifier](scripts/verify-public-project-contract.sh), and [focused regressions](tests/governance.rs) define the public workflow, redaction and support boundaries, private security routing, complete two-repository inventory, inbound-equals-outbound MIT terms, no CLA, separate non-required DCO choice, and no nominal `CODEOWNERS`. Merged [PR #35](https://github.com/EnjoyableWork/mcp-sync/pull/35) publishes the contract, [PR #36](https://github.com/EnjoyableWork/mcp-sync/pull/36) adds the root support surface, final [PR #37](https://github.com/EnjoyableWork/mcp-sync/pull/37) records the live recognition correction and ticket closeout, and tap [PR #1](https://github.com/EnjoyableWork/homebrew-tap/pull/1) adds the missing root MIT license without changing its formula. Signed-in inspection on exact `main` commit [`608fddf`](https://github.com/EnjoyableWork/mcp-sync/commit/608fddfecea6e4c68e0d20d699c5de6e6ce41ea6) shows GitHub marking every ticket-relevant Community Standards entry Added and rendering all three forms plus private-security routing; the REST API's simultaneous `87`/null YAML-form discrepancy and unrelated disabled content-report setting are recorded rather than hidden or changed. PR #35 passes [CodeQL](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31300162751), [CI](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31300164296), [source and GNU/Linux preflight](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31300164282), and [six-target release preflight](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31300164286); PR #36 passes [CodeQL](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31300640244), [CI](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31300641035), [source and GNU/Linux preflight](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31300641032), and [six-target release preflight](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31300641052). Local format, warning-free Clippy, all 277 tests, `cargo deny`, ShellCheck, documentation checks, the public project verifier, and the preserved default-branch and non-disclosing security verifiers pass. The audit proves HTTPS-only official channels, both root MIT licenses, exact tag/crate/GNU archive license bytes, immutable release and crates.io identity, and the MIT Homebrew formula without modifying `v0.1.0`, security settings, or public assurance claims. This evidence proves `OSPS-BR-03.01`, `OSPS-DO-02.01`, `OSPS-GV-02.01`, `OSPS-GV-03.01`, `OSPS-LE-02.01`, `OSPS-LE-02.02`, `OSPS-LE-03.01`, `OSPS-LE-03.02`, and applicable `OSPS-QA-04.01`. |
| MCP-033 | Automate dependency maintenance and harden CI and distribution supply chains | M3 | P1 | Codex | Done | `MCP-032` | The [repository supply-chain contract and scoped OSPS record](docs/repository-supply-chain.md), grouped [Dependabot configuration](.github/dependabot.yml), exact [selected-action inventory](.github/actions-policy.json), credential-free [workflow](scripts/verify-workflow-supply-chain.sh) and [tracked-artifact](scripts/verify-repository-artifacts.sh) gates, authenticated [operator verifier](scripts/verify-repository-supply-chain-controls.sh), exact [distribution verifier](scripts/verify-distribution-authentication.sh), focused [artifact-policy failure exercise](scripts/test-repository-artifact-policy.sh), and [six regressions](tests/supply_chain.rs) implement the accepted boundary. Protected [PR #38](https://github.com/EnjoyableWork/mcp-sync/pull/38) merged normally as exact `main` commit [`d934d38`](https://github.com/EnjoyableWork/mcp-sync/commit/d934d38edd40a4c38bacfd79994221548882e79a); that commit passes [CI](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31304197031), [CodeQL](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31304196954), the [six-target release preflight](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31304196996), the complete [source and GNU/Linux preflight](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31304197063), and both accepted Dependabot configuration evaluations ([Cargo](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31304200669), [Actions](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31304199968)) under the live selected/full-SHA policy. The exact-main operator audit resolves all 72 action hints to their reviewed SHAs, structurally verifies the ten-pattern live policy plus read-only tokens, secretless fork handling, CodeQL and merged update configuration, rejects generated executables and binary artifacts across all 118 tracked paths, and cryptographically authenticates the immutable release and every asset before proving byte-identical HTTPS Cargo and Homebrew distribution. An expected-rejection [invalid `1.2.3` dispatch](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31304302717) fails only in the unprivileged validator, skips the protected publisher, and leaves the four-record `release` deployment set unchanged. Final [PR #39](https://github.com/EnjoyableWork/mcp-sync/pull/39) broadens the fork-safety drift gate from named scopes to every write permission plus alternate secret syntax, protected environments, and cross-run artifact credentials before recording ticket closure. Local format, warning-free Clippy, all 283 tests, `cargo deny`, Actionlint, ShellCheck, repository-control verifiers, and documentation checks pass. This evidence proves only `OSPS-BR-01.01`, `OSPS-BR-01.03`, `OSPS-BR-03.02`, `OSPS-QA-05.01`, and `OSPS-QA-05.02`; immutable `v0.1.0`, stable check contexts, protected credentials, the dormant funded path, and the later public-claim gate remain unchanged. |
| MCP-034 | Establish organization access, credential, and ownership recovery policy | M3 | P1 | Codex | Done | `MCP-033` | The [completed non-sensitive contract and OSPS evidence](docs/organization-access-and-recovery.md), aggregate-only [private-evidence template generator](scripts/prepare-organization-access-evidence.sh), read-only [operator verifier](scripts/verify-organization-access-controls.sh), synthetic [acceptance and rejection exercise](scripts/test-organization-access-policy.sh), and [focused regressions](tests/organization_access.rs) enforce strong MFA, default-no-access manual grants, deliberate repository creation, real rather than nominal teams and owners, private recovery evidence, and bounded automation without printing identities or sensitive material. After explicit approval, authenticated update and read-back set base access to `none` and disabled member repository creation without removing current human access. The owner accepted the single-owner residual risk, privately confirmed secure-method-only 2FA, the application review, and a successful same-day recovery exercise, and the owner-only aggregate evidence drove a passing complete live verifier before its temporary copy was removed. [PR #40](https://github.com/EnjoyableWork/mcp-sync/pull/40) implementation head [`77fce4b`](https://github.com/EnjoyableWork/mcp-sync/commit/77fce4b072aaea47bfe0367201ddfd6a88e64527) passes [CI](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31328986122), [CodeQL](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31328983549), the [six-target release preflight](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31328986128), and the complete [source and GNU/Linux preflight](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31328986137); local checks and the non-disclosing audit prove `OSPS-AC-01.01` and `OSPS-AC-02.01` only, with no complete-baseline or public assurance claim. |
| MCP-035 | Self-assess, publish, and showcase the zero-cost enterprise assurance baseline | M3 | P1 | Codex | Done | `MCP-034` | The public [security-assurance contract](docs/security-assurance.md) evaluates all 24 OpenSSF OSPS Baseline `v2026.02.19` Level 1 controls, links public evidence or exact conditional reasoning, directly revalidates `OSPS-DO-01.01`, `OSPS-QA-01.01`, `OSPS-QA-01.02`, and `OSPS-QA-02.01`, and defines annual plus change- and incident-driven revalidation and claim-removal triggers without publishing private settings, identity details, reports, or secrets. After explicit owner OAuth and public-attribution authorization, the official [BadgeApp project `14014` assessment](https://www.bestpractices.dev/en/projects/14014/baseline-1) records 24/24 `Met`, criteria `v2026.02.19`, `baseline-1` achieved on 2026-08-09, and no lost status; its issuer-provided dynamic badge appears in the README and links to the official project record. Protected [PR #41](https://github.com/EnjoyableWork/mcp-sync/pull/41) merges as exact public `main` commit [`315bf5c`](https://github.com/EnjoyableWork/mcp-sync/commit/315bf5c908d3a8d6f10f3f937fe2dcc9c4839c47), where the corrected [public verifier](scripts/verify-public-security-assurance.sh) confirms the API status, 24 controls, badge SVG, destination, rendered README, proposal, contract, exact scope, and claim language. The same record limits SLSA v1.0 Build Level 2 to seven named and hash-bound `v0.1.0` GitHub Release assets whose provenance and cross-channel bytes reverify. Local format, warning-free Clippy, all 294 tests, `cargo deny`, ShellCheck, documentation checks, live repository and release controls, and distribution authentication pass; no higher OSPS level, independent certification, regulatory regime, OpenChain, NIST certification, paid signing, later arc, or side quest is claimed or activated. |

### Testing tool introduction plan

Testing tools enter the project with the first main-story ticket that has a
real use for them. They are implementation support for that ticket, not
independent side quests, and must not be installed early merely to prepare for
later work.

`Required` means the named ticket adopts the tool if the pre-adoption review
still finds it maintained, compatible, and appropriately scoped. Replacing a
required tool needs an accepted decision that preserves the same evidence.
`Conditional` means the ticket must evaluate the stated trigger and record the
choice, but should not add the dependency when ordinary Rust tests remain
clearer. `Diagnostic` tools initially inform test design rather than impose an
arbitrary numeric release gate.

| Tool or method | First introduction ticket | Commitment | Implementation contract |
| --- | --- | --- | --- |
| Rust `#[test]` and `cargo test` | `MCP-003` — Done | Baseline | Keep `cargo test --workspace --all-targets --all-features --locked` authoritative and use the narrowest useful test layer. A later runner may accelerate this suite but does not replace its semantics. |
| `serde` and `serde_json` | `MCP-004` — Done | Adopted product dependencies | The strict v1 types and shared duplicate-rejecting JSON boundary prove normalization, malformed input, deterministic bytes, literal round trips, redaction, and unsupported-version behavior. `MCP-007` enables `serde_json` arbitrary-precision numbers so unowned native values survive adapter reserialization without numeric loss. |
| [`toml_edit`](https://docs.rs/toml_edit/0.25.13/) | `MCP-016` — Done | Adopted product dependency | Version 0.25.13 uses only its explicit `parse` and `display` features to edit Codex TOML structurally while retaining comments, ordering, formatting, and unsupported data without a JSON conversion. Its current toml-rs repository, Rust 1.85 floor, MIT/Apache-2.0 license, eight-package locked graph, duplicate-version impact, advisories, and source policy were reviewed at adoption; the focused fixtures exercise standard and inline tables, deterministic serialization, duplicate rejection, reparsing, exact no-ops, and redacted errors. |
| `pretty_assertions` | `MCP-004` — evaluated | Not adopted | The exact documents and table-driven cases remain small and readable with standard assertions, so another development dependency would not materially improve diagnosis. Reconsider only if future nested comparisons become difficult to review. |
| `cargo-deny` | `MCP-004` — Done | Adopted development/CI tool | The committed policy has no broad exceptions, the official action and tool release are pinned for CI and local use, and advisories, allowed licenses, sources, bans, and duplicate versions pass. |
| `tempfile` | `MCP-005` — Done; runtime use added by `MCP-009` and extended by `MCP-010` and `MCP-011` | Adopted product and development dependency | Version 3.27.0 backs `SyntheticHome`, path cases, and filesystem fixtures with explicit owned lifetimes. `MCP-009` promotes the same reviewed crate to the product graph for securely named same-directory temporary files and no-clobber publication; `MCP-010` reuses that boundary for synced atomic replacement and backup publication, while `MCP-011` adds exact rollback receipts and compensation without another dependency. Its release, Rust floor, MIT/Apache-2.0 license, all-target transitive graph, duplicate-version impact, advisories, and source policy pass; every test path remains under its disposable root. |
| `proptest` | `MCP-006` — Done | Adopted development dependency | Version 1.11.0 runs five pure suites with 128 cases each and a bounded shrink limit using only its `std` feature. The suites prove exact outcomes, determinism, input immutability, insertion-order independence, stable no-ops, and generated-value redaction without I/O; default failure persistence retains minimized regressions. Its feature-complete, passively maintained status, compatibility, MIT/Apache-2.0 license, all-target transitive graph, duplicate-version impact, advisories, and source policy were reviewed at adoption. |
| Checked-in native fixtures | `MCP-007` — Done; reused by `MCP-008`, `MCP-011`, `MCP-012`, `MCP-014`, `MCP-015`, and `MCP-016` | Adopted baseline method | Small synthetic current, desired, and merged documents use unmistakably fake secrets and exact byte comparisons. Claude Desktop fixtures cover every reconciliation outcome and bounded preservation. Cursor adds a project sentinel and covers unmanaged remote entries, local-name collisions, project-file isolation, arbitrary-precision native data, deterministic rendering, reparsing, and transaction-level preservation. Windsurf reuses the pattern for legacy Cascade local entries, commandless `url`/`serverUrl`, headers, `disabledTools`, target-only data, interpolation literals, and collision refusal. VS Code adds native root `servers`, explicit and implicit STDIO, remote HTTP, number/null environment values, root `inputs`/`sandbox`, default-profile isolation, and explicit STDIO rendering. Codex adds exact TOML with comments and formatting, standard and inline tables, unrelated root and plug-in settings, unowned local fields, remote/OAuth/mixed/opaque entries, project isolation, collision refusal, and exact no-op bytes. The combined disposable-home journeys use byte diagnostics that never print private fixture contents. Reuse this pattern for later adapters. |
| `insta` | `MCP-007` — evaluated | Not adopted | The three exact native fixtures remain small and directly reviewable, so snapshot tooling would add approval state and a dependency without improving diagnosis. Reconsider only when later native documents become unwieldy under direct comparisons. |
| [`assert_cmd`](https://docs.rs/assert_cmd/2.2.2/assert_cmd/) | `MCP-009` — Done; reused by `MCP-010`, `MCP-011`, `MCP-012`, `MCP-014`, `MCP-015`, `MCP-016`, and `MCP-017` | Adopted development dependency | Version 2.2.2 runs the Cargo-built `mcp-sync` binary through `SyntheticHome` with inherited environment cleared, then asserts status, bounded stdout/stderr structure, redaction, process cleanup, and filesystem effects together. Configuration journeys retain a five-second command bound; health journeys use an eight-second outer bound around the product's five-second response limit. The current release and active repository, Rust 1.85 floor, MIT/Apache-2.0 license, cross-platform support, nine-package development-only graph addition, duplicate-version impact, advisories, and source policy were checked; the locked `cargo test` path now runs nine `init`, ten canonical catalog, fourteen sync, three combined golden/failure-matrix, and five health journeys. Reuse this harness for later CLI journeys. |
| `cargo-nextest` | `MCP-012` — evaluated | Not adopted | The settled 140-test M1 suite runs in about one second after compilation and has no demonstrated retry, partitioning, or isolation need. The documented `cargo test --workspace --all-targets --all-features --locked` command remains the clearest authoritative runner; reconsider only when suite behavior or runtime creates a concrete benefit. |
| [`cargo-llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov) | `MCP-012` — evidence complete | Adopted diagnostic development tool | Version 0.8.7 profiled all targets and the spawned built binary by forwarding only `LLVM_PROFILE_FILE` through the otherwise cleared integration environment. The final review reports 92.77% line, 91.95% region, and 91.14% function coverage across `src`; recovery and rendered-plan gaps received tests, while coverage remains diagnostic rather than a numeric release gate. No product dependency or CI threshold was added. |
| [`cargo-mutants`](https://mutants.rs/) | `MCP-012` — evidence complete | Adopted diagnostic development tool | Version 27.1.0 ran in its copied temporary tree with a 60-second per-test bound over reconciliation, rendered-plan verification, transactional apply, filesystem guards, and rollback. After surviving safety mutations produced focused tests, the final 57-mutant run caught 49; eight return-default mutations were compile-unviable, with zero missed and zero timed out. It made no product dependency or working-tree mutation. |
| `trycmd` | `MCP-013` — evaluated | Not adopted | The six implemented command surfaces have focused help assertions and now 41 stateful built-binary journeys through the `assert_cmd` synthetic-home harness. The guide explains those tested flows plus manually exercised JSON and TOML recovery procedures; compact transcript cases would duplicate weaker evidence and would not cover filesystem effects, process cleanup, rollback, or redaction. Reconsider only if future stateless command documentation becomes costly to keep synchronized. |

Every introduction ticket owns the complete adoption: recheck the current
release, maintenance, licenses, advisories, transitive cost, and platform
support; update `Cargo.toml` or pinned tool installation plus `Cargo.lock` where
applicable; add the first meaningful tests; wire the relevant local and CI
commands; document developer usage; and link durable evidence from the ticket
row. Tool versions are selected and recorded when that ticket starts so this
plan does not freeze stale versions in advance.

### Main-story goal catalog

These are the exact objectives for Goal mode. Paste the applicable objective
after `/goal`; do not include the Markdown table delimiters. Goals for `Done`
or non-`Ready` tickets remain dormant until the ordered workflow makes that
ticket eligible.

| Ticket | Canonical goal objective |
| --- | --- |
| MCP-001 | Complete MCP-001: establish repository guidance, a living delivery tracker, the Rust direction, and README's north-star product and marketing role. Follow AGENTS.md and PROJECT.md, preserve unrelated worktree changes, and do not begin later tickets. Finish only when MCP-001's evidence criteria and the ticket-done gate pass, durable evidence is recorded, MCP-001 is Done, and only MCP-002 is Ready. |
| MCP-002 | Complete MCP-002: bootstrap one Rust binary crate whose verified, distinct Cargo package installs the mcp-sync executable with working help and version output and a committed Cargo.lock. Follow AGENTS.md and PROJECT.md, preserve unrelated worktree changes, and do not implement configuration, client, sync, or later-ticket behavior. Finish only when MCP-002's evidence criteria and the ticket-done gate pass, durable evidence is recorded, MCP-002 is Done, and only MCP-003 is Ready. |
| MCP-003 | Complete MCP-003: add deterministic formatting, Clippy, test, and CI quality gates that operate through a synthetic home and never touch real user configuration. Follow AGENTS.md and PROJECT.md, preserve unrelated worktree changes, and do not begin schema or client work. Finish only when MCP-003's evidence criteria and the ticket-done gate pass locally and in CI, durable evidence is recorded, MCP-003 is Done, and only MCP-004 is Ready. |
| MCP-004 | Complete MCP-004: define the versioned, client-independent canonical MCP server model and JSON validation contract with documented examples, deterministic round trips, and explicit unknown-version failure. Follow AGENTS.md and PROJECT.md, preserve unrelated worktree changes, and do not add client adapters or later behavior. Finish only when MCP-004's evidence criteria and the ticket-done gate pass, durable evidence is recorded, MCP-004 is Done, and only MCP-005 is Ready. |
| MCP-005 | Complete MCP-005: introduce injectable configuration-path and filesystem boundaries with macOS path fixtures, synthetic-home enforcement, and contextual I/O failures. Follow AGENTS.md and PROJECT.md, preserve unrelated worktree changes, and never access real user configuration or begin reconciliation work. Finish only when MCP-005's evidence criteria and the ticket-done gate pass, durable evidence is recorded, MCP-005 is Done, and only MCP-006 is Ready. |
| MCP-006 | Complete MCP-006: build a pure normalized reconciliation and structurally redacted plan engine with deterministic add, update, no-op, and drift outcomes and no filesystem mutation. Follow AGENTS.md and PROJECT.md, preserve unrelated worktree changes, and do not begin target adapters. Finish only when MCP-006's evidence criteria and the ticket-done gate pass, durable evidence is recorded, MCP-006 is Done, and only MCP-007 is Ready. |
| MCP-007 | Complete MCP-007: implement the global Claude Desktop macOS adapter for native discovery, parsing, rendering, and bounded merge behavior while preserving unrelated configuration. Follow AGENTS.md and PROJECT.md, preserve unrelated worktree changes, and do not begin another client adapter. Finish only when MCP-007's fixture and evidence criteria plus the ticket-done gate pass, durable evidence is recorded, MCP-007 is Done, and only MCP-008 is Ready. |
| MCP-008 | Complete MCP-008: implement the global Cursor macOS adapter for native discovery, parsing, rendering, and bounded merge behavior while preserving unrelated configuration and leaving every project-level `.cursor/mcp.json` untouched. Follow AGENTS.md and PROJECT.md, preserve unrelated worktree changes, and do not begin import orchestration or another client adapter. Finish only when MCP-008's fixture and evidence criteria plus the ticket-done gate pass, durable evidence is recorded, MCP-008 is Done, and only MCP-009 is Ready. |
| MCP-009 | Complete MCP-009: implement deterministic init discovery, import, normalization, and actionable conflict reporting across the two M1 clients without mutating anything on conflict. Follow AGENTS.md and PROJECT.md, preserve unrelated worktree changes, and do not begin add, list, or sync behavior. Finish only when MCP-009's journey evidence and the ticket-done gate pass, durable evidence is recorded, MCP-009 is Done, and only MCP-010 is Ready. |
| MCP-010 | Complete MCP-010: implement canonical-config add and structurally redacted list behavior with deterministic upsert semantics, atomic canonical writes, and no secret output. Follow AGENTS.md and PROJECT.md, preserve unrelated worktree changes, and do not begin target sync behavior. Finish only when MCP-010's CLI evidence and the ticket-done gate pass, durable evidence is recorded, MCP-010 is Done, and only MCP-011 is Ready. |
| MCP-011 | Complete MCP-011: implement sync dry-run and safe multi-target apply using the same validated plan, recoverable backups, no-op detection, atomic replacement, per-target reporting, and transaction rollback. Follow AGENTS.md and PROJECT.md, preserve unrelated worktree changes, and do not weaken safety invariants or begin later client work. Finish only when MCP-011's success and forced-failure evidence plus the ticket-done gate pass, durable evidence is recorded, MCP-011 is Done, and only MCP-012 is Ready. |
| MCP-012 | Complete MCP-012: prove the golden M1 journey and its failure matrix through the built binary against a synthetic home, including redaction, idempotence, rollback, and non-zero error behavior. Follow AGENTS.md and PROJECT.md, preserve unrelated worktree changes, and fix any M1 defect revealed without adding M2 scope. Finish only when every M1 acceptance criterion and the ticket-done gate pass with durable evidence, MCP-012 is Done, and only MCP-013 is Ready. |
| MCP-013 | Complete MCP-013: verify delivered M1 behavior through a controlled, backup-protected smoke test with current stable Claude Desktop and Cursor on macOS, verify it against the north-star README, and publish accurate usage and recovery guidance while keeping current limitations in delivery documentation rather than diluting the README's product role. Follow AGENTS.md and PROJECT.md, preserve unrelated worktree changes, and do not claim or begin M2 behavior. Finish only when MCP-013's client and documentation evidence plus the ticket-done gate pass, durable evidence is recorded, MCP-013 is Done, and only MCP-014 is Ready. |
| MCP-014 | Complete MCP-014: add the Windsurf target adapter with native discovery, parse and render translation, a documented merge boundary, and fixture and journey coverage that preserves unrelated data. Follow AGENTS.md and PROJECT.md, preserve unrelated worktree changes, and do not begin VS Code or other later targets. Finish only when MCP-014's evidence criteria and the ticket-done gate pass, durable evidence is recorded, MCP-014 is Done, and only MCP-015 is Ready. |
| MCP-015 | Complete MCP-015: add the VS Code target adapter and define exactly which extension configuration shape mcp-sync supports, with native fixtures, preservation boundaries, and journey coverage. Follow AGENTS.md and PROJECT.md, preserve unrelated worktree changes, and do not begin Codex or health testing. Finish only when MCP-015's evidence criteria and the ticket-done gate pass, durable evidence is recorded, MCP-015 is Done, and only MCP-016 is Ready. |
| MCP-016 | Complete MCP-016: add the Codex TOML adapter for the shared ChatGPT desktop app, Codex CLI, and IDE host configuration using `DEC-028`'s global-only, local-STDIO scope, structurally preserving unrelated settings, project layers, remote/OAuth entries, and every unsupported MCP field while collision-protecting unmanaged names. Follow AGENTS.md and PROJECT.md, preserve unrelated worktree changes, and never use a lossy JSON conversion, access OAuth credential stores, or begin health testing. Finish only when MCP-016's fixtures, discovery and journey evidence, and ticket-done gate pass, durable evidence is recorded, MCP-016 is Done, and only MCP-017 is Ready. |
| MCP-017 | Complete MCP-017: add bounded MCP STDIO initialize health testing with protocol validation, timeouts, guaranteed child cleanup, malformed-output handling, and structural secret redaction. Follow AGENTS.md and PROJECT.md, preserve unrelated worktree changes, and do not let init or sync execute servers or begin platform expansion. Finish only when MCP-017's process and failure evidence plus the ticket-done gate pass, durable evidence is recorded, MCP-017 is Done, and only MCP-018 is Ready. |
| MCP-018 | Complete MCP-018: add Linux path and behavior support across the supported clients with deterministic fixtures and Linux CI coverage while preserving all existing safety and reconciliation behavior. Follow AGENTS.md and PROJECT.md, preserve unrelated worktree changes, and do not begin Windows work. Finish only when MCP-018's platform evidence and the ticket-done gate pass, durable evidence is recorded, MCP-018 is Done, and only MCP-019 is Ready. |
| MCP-019 | Complete MCP-019: add Windows path, backup, atomic-replacement-equivalent, rollback, and supported-client behavior with Windows CI and platform fixtures. Follow AGENTS.md and PROJECT.md, preserve unrelated worktree changes, and do not weaken recovery guarantees or begin restore UX. Finish only when MCP-019's platform and failure evidence plus the ticket-done gate pass, durable evidence is recorded, MCP-019 is Done, and only MCP-020 is Ready. |
| MCP-020 | Complete MCP-020: add explicit restore and backup-retention user experience with documented retention rules and a tested recovery journey across supported platforms. Follow AGENTS.md and PROJECT.md, preserve unrelated worktree changes, and do not publish release channels early. Finish only when MCP-020's recovery evidence and the ticket-done gate pass, durable evidence is recorded, MCP-020 is Done, and only the accepted release successor at the post-MCP-020 story point is Ready. |
| MCP-021 | Complete MCP-021: publish accessible, provenance-linked GitHub Release, Homebrew, WinGet, and Cargo channels that all install the same protected tagged mcp-sync version and pass supported-platform smoke journeys through the accepted tag ruleset, release environment, signing, and provenance controls. Follow AGENTS.md and PROJECT.md, preserve unrelated worktree changes, and do not activate deferred expansion work. Finish only when MCP-021's release evidence, the M2 release gate, and the ticket-done gate pass, durable evidence is recorded, and MCP-021 is Done. |
| MCP-028 | Complete MCP-028: establish and dry-run-prove a zero-cost market-validation release path that publishes only attested GNU/Linux x64/ARM64 binaries plus source-based Cargo and Homebrew outputs, cannot accidentally invoke paid signing from a normal `v0.1.0` release tag, and preserves the full six-target signed pipeline and its verification tooling for later funding. Follow AGENTS.md and PROJECT.md, preserve unrelated worktree changes, do not publish a tag, release, crate, formula, unsigned macOS binary, or unsigned Windows binary, and keep release notes and runbooks accurate about the current boundary. Finish only when MCP-028's deterministic generation, provenance, native install/recovery smoke, retained-pipeline, documentation, and ticket-done evidence pass, durable evidence is recorded, MCP-028 is Done, and only MCP-029 is Ready. |
| MCP-029 | Complete MCP-029: publish and verify the protected zero-cost v0.1.0 market-validation release from one commit through an immutable GitHub Release with attested GNU/Linux x64/ARM64 artifacts, crates.io as enjoyable-mcp-sync 0.1.0, and the source-building EnjoyableWork/tap/mcp-sync Homebrew formula, with accurate limitations and all represented native install and recovery smokes. Follow AGENTS.md and PROJECT.md, preserve unrelated worktree changes, recheck every public identity, never publish or imply a project-issued unsigned macOS or Windows binary, do not publish WinGet, and do not activate paid signing. Finish only when MCP-029's durable publication evidence, the M2 release gate, and the ticket-done gate pass, MCP-029 is Done, and no deferred expansion or side quest is implicitly activated. |
| MCP-030 | Complete MCP-030: protect `main` with a contributor-compatible GitHub ruleset that requires pull requests, strict up-to-date CI checks bound to the expected GitHub Actions app, resolved conversations, and blocked deletion and force pushes; deliberately record minimal bypass, approval-count, merge-method, and signed-commit choices; prove normal and emergency paths; and produce durable evidence for `OSPS-AC-03.01` and `OSPS-AC-03.02`. Follow AGENTS.md and PROJECT.md, preserve unrelated worktree changes, do not alter stable-tag or release-environment controls, and do not begin later assurance work. Finish only when MCP-030's settings, path tests, OSPS evidence, documentation checks, and ticket-done gate pass, durable evidence is recorded, MCP-030 is Done, and only MCP-031 is Ready. |
| MCP-031 | Complete MCP-031: establish vulnerability disclosure, supported-version and security-contact guidance, private reporting, coordinated response expectations, and the available GitHub dependency-graph, Dependabot security-update, CodeQL, secret-scanning, and push-protection controls, with an exact limitation for unavailable supplemental features and zero-cost controls that fully prove `OSPS-BR-07.01` and `OSPS-VM-02.01`. Follow AGENTS.md and PROJECT.md, preserve unrelated worktree changes, expose no alert, report, secret, identity, or contact credential, do not add dependency-update workflow scope owned by MCP-033, and do not publish an assurance badge or claim. Finish only when MCP-031's public contract, live settings, baseline evidence, documentation checks, and ticket-done gate pass, durable evidence is recorded, MCP-031 is Done, and only MCP-032 is Ready. |
| MCP-032 | Complete MCP-032: publish the contribution, community, repository, and licensing contract through GitHub-recognized contribution and conduct guidance, structured defect and feature forms, a pull-request template, HTTPS-only official URIs, an in-scope repository inventory, source and release-asset MIT licensing, the accepted no-CLA and separate DCO choice, redaction and support boundaries, and safe routing to MCP-031's private security path, proving `OSPS-BR-03.01`, `OSPS-DO-02.01`, `OSPS-GV-02.01`, `OSPS-GV-03.01`, `OSPS-LE-02.01`, `OSPS-LE-02.02`, `OSPS-LE-03.01`, `OSPS-LE-03.02`, and applicable `OSPS-QA-04.01`. Follow AGENTS.md and PROJECT.md, preserve unrelated worktree changes, add `CODEOWNERS` only if an independent reviewer exists, and do not change security settings or publish assurance claims. Finish only when MCP-032's files, GitHub recognition, licensing and URI audits, OSPS evidence, documentation checks, and ticket-done gate pass, durable evidence is recorded, MCP-032 is Done, and only MCP-033 is Ready. |
| MCP-033 | Complete MCP-033: automate weekly grouped Cargo and GitHub Actions updates, pin every action to a reviewed full commit SHA, narrow allowed-action and SHA policy, validate untrusted workflow metadata, keep privileged credentials and assets unavailable to fork or other untrusted code, verify cryptographically authenticated distribution, prevent committed generated executables and unreviewable binary artifacts, and prove `OSPS-BR-01.01`, `OSPS-BR-01.03`, `OSPS-BR-03.02`, `OSPS-QA-05.01`, and `OSPS-QA-05.02`. Follow AGENTS.md and PROJECT.md, preserve unrelated worktree changes, retain least-privilege tokens and stable check contexts, never modify immutable `v0.1.0`, and do not publish assurance claims. Finish only when MCP-033's automation, fork-safety, distribution and repository audits, CI, dependency policy, OSPS evidence, and ticket-done gate pass, durable evidence is recorded, MCP-033 is Done, and only MCP-034 is Ready. |
| MCP-034 | Complete MCP-034: establish organization access, credential, and ownership-recovery policy with strong MFA for sensitive resources, manual or least-privilege collaborator assignment, two trusted owners or an explicitly accepted and tested single-owner recovery plan, least-privilege teams where useful, deliberate repository creation, private recovery practices, and short-lived scope-minimized automation, proving `OSPS-AC-01.01` and `OSPS-AC-02.01`. Follow AGENTS.md and PROJECT.md, preserve unrelated worktree changes, obtain explicit approval before organization-wide access or policy changes, never remove access or publish identities, factors, tokens, or recovery material, and do not publish assurance claims. Finish only when MCP-034's live-setting and private-policy evidence, recovery exercise, OSPS evidence, documentation checks, and ticket-done gate pass, durable non-sensitive evidence is recorded, MCP-034 is Done, and only MCP-035 is Ready. |
| MCP-035 | Complete MCP-035: perform and publish a point-in-time self-assessment of all 24 OpenSSF OSPS Baseline `v2026.02.19` Level 1 controls, link durable evidence or exact conditional reasoning, revalidate existing user-guide, public-source, public-history, and direct-dependency evidence, publish a security-assurance contract, and complete the official public `baseline-1` assessment for `EnjoyableWork/mcp-sync` after the owner authorizes the badge service's GitHub OAuth and required public account attribution. Only after every control passes, obtain the official BadgeApp `baseline-1` self-certification badge and add a concise README Security and Trust surface whose dynamic shield links to that assessment; verify its status, destination, scope, terminology, and rendering on exact `main`; verify and scope an artifact-level SLSA v1.0 Build Level 2 statement from the existing GitHub attestations; and define revalidation triggers. Follow AGENTS.md and PROJECT.md, preserve unrelated worktree changes, disclose no private setting, non-required identity detail, report, or secret, and make no OSPS Level 2 or 3, independent-certification, regulatory, OpenChain, NIST-certification, or paid-signing claim. Finish only when every Level 1 control passes, the official badge is issued and linked, the exact version, level, scope, date, limitations, public evidence and claim language are verified, documentation checks and the ticket-done and M3 gates pass, durable evidence is recorded, MCP-035 and M3 are Done, and no later arc or side quest is implicitly activated. |

### Side-quest goal catalog

These are the exact objectives for Goal mode. Paste the applicable objective
after `/goal`; do not include the Markdown table delimiters. `SIDE-001`
through `SIDE-005` were superseded by ordered `MCP-030` through `MCP-034`
under `DEC-038`; their retired objectives must not be activated. A goal does
not make a remaining side quest independent, ready, or exempt from the
one-side-quest WIP cap, and completing one never activates another.

| Ticket | Canonical goal objective |
| --- | --- |
| SIDE-006 | Complete SIDE-006: activate the funded signed-native distribution pipeline by provisioning Apple Developer ID and notarization plus Windows Public Trust signing authorities, publishing one later immutable version as six signed and attested native artifacts with binary-backed Homebrew and WinGet updates, and proving every native installed recovery smoke while preserving the zero-cost Cargo, Homebrew, and GitHub paths. Follow AGENTS.md and PROJECT.md, preserve unrelated worktree changes, require explicit funding and activation, keep the work independent of M2, never reuse or replace v0.1.0 assets, never weaken signing or provenance checks, and do not activate another expansion item. Finish only when SIDE-006's signed-native publication evidence and the ticket-done gate pass, durable evidence is recorded, SIDE-006 is Done, and no other side quest is implicitly activated. |

### Expansion candidates

These are deliberately not executable tickets yet. M3 now owns enterprise
assurance and adoption; after it completes, select and sequence the next
main-story arc and classify anything truly optional as a `SIDE-NNN` ticket.

| ID | Candidate outcome | Possible arc | Classification trigger | Status |
| --- | --- | --- | --- | --- |
| MCP-022 | Add schema migration infrastructure after the first real schema change | M4 | A real schema change requires an upgrade path | Deferred |
| MCP-023 | Add external secret-reference support without becoming a secret store | M4 | Validated demand and an accepted threat model | Deferred |
| MCP-024 | Add named profiles and selective target policy | M4 | Validated multi-context workflow | Deferred |
| MCP-025 | Formalize a third-party adapter API | M4 | At least one real external adapter consumer | Deferred |
| MCP-026 | Explore managed team/fleet configuration | M5 | Approved product discovery and security boundary | Deferred |
| MCP-027 | Explore a GUI or desktop management surface | M5 | User research proves a distinct interface need | Deferred |

### Side-quest board

The following baseline was read through GitHub's API on 2026-08-08. It records
settings and counts, not credential values, organization-member identities, or
recovery material. `DEC-038` promotes the required outcomes of `SIDE-001`
through `SIDE-005` into the ordered M3 main story; their retired side-quest
objectives must not be activated. Only `SIDE-006` remains a proposed side quest,
with a dormant objective in the
[side-quest goal catalog](#side-quest-goal-catalog).

| ID | Outcome | Priority | Owner | Status | Audited baseline and done when |
| --- | --- | --- | --- | --- | --- |
| SIDE-001 | Protect the default branch and define a contributor-compatible merge policy | P2 | Unassigned | Superseded | Replaced by ordered main-story ticket `MCP-030` through `DEC-038`; retain this ID only as the audited origin of that scope. |
| SIDE-002 | Publish a complete public contribution and community-health contract | P2 | Unassigned | Superseded | Replaced by ordered main-story ticket `MCP-032` through `DEC-038`; its security-report routing depends on completed `MCP-031`. |
| SIDE-003 | Establish the public vulnerability-disclosure and GitHub code-security baseline | P2 | Unassigned | Superseded | Replaced by ordered main-story ticket `MCP-031` through `DEC-038`. |
| SIDE-004 | Automate dependency maintenance and harden the CI action supply chain | P2 | Unassigned | Superseded | Replaced by ordered main-story ticket `MCP-033` through `DEC-038`. |
| SIDE-005 | Establish organization access, credential, and ownership-recovery policy | P2 | Unassigned | Superseded | Replaced by ordered main-story ticket `MCP-034` through `DEC-038`. |
| SIDE-006 | Activate the funded six-target signed-native distribution pipeline | P2 | Unassigned | Proposed | `MCP-021` produced a fail-closed signed workflow, native six-target preflight, deterministic downstream metadata, all-channel smoke workflow, protected release controls, and a detailed runbook, but the required Apple and Windows signing authorities have a funding cost and no market evidence yet justifies it. Done only after explicit funding and activation provide Developer ID/notarization and Windows Public Trust credentials; a later immutable version publishes signed macOS and Windows plus attested GNU/Linux x64/ARM64 artifacts, binary-backed Homebrew, WinGet, checksums, SBOMs, and attestations; every native installation and recovery smoke passes; the existing zero-cost channels remain valid; and no `v0.1.0` tag or asset is moved or replaced. |

### M3 OSPS Level 1 control map

M3 targets the exact 24 Level 1 controls in the official
[OpenSSF OSPS Baseline `v2026.02.19` checklist](https://baseline.openssf.org/versions/2026-02-19-checklist.md).
This is a point-in-time, evidence-backed self-assessment for the repositories,
release process, and organization practices within the stated scope. Its
official BadgeApp `baseline-1` badge is proof of that self-certification,
not an independent certification, and M3 makes no Level 2 or Level 3 claim.

| Owning work | Level 1 controls | Count | Evidence boundary |
| --- | --- | --- | --- |
| `MCP-030` | `OSPS-AC-03.01`, `OSPS-AC-03.02` | 2 | [Verified direct-commit prevention and primary-branch deletion protection](docs/repository-governance.md) |
| `MCP-031` | `OSPS-BR-07.01`, `OSPS-VM-02.01` | 2 | [Security contacts, private vulnerability reporting, and secret prevention](docs/repository-security.md) |
| `MCP-032` | `OSPS-BR-03.01`, `OSPS-DO-02.01`, `OSPS-GV-02.01`, `OSPS-GV-03.01`, `OSPS-LE-02.01`, `OSPS-LE-02.02`, `OSPS-LE-03.01`, `OSPS-LE-03.02`, `OSPS-QA-04.01` | 9 | Public project, contribution, repository, and source-and-release licensing contracts |
| `MCP-033` | `OSPS-BR-01.01`, `OSPS-BR-01.03`, `OSPS-BR-03.02`, `OSPS-QA-05.01`, `OSPS-QA-05.02` | 5 | Dependency maintenance, CI isolation, authenticated distribution, and binary-artifact policy |
| `MCP-034` | `OSPS-AC-01.01`, `OSPS-AC-02.01` | 2 | [Verified organization access, authentication, least privilege, and recovery](docs/organization-access-and-recovery.md) |
| Existing evidence revalidated by `MCP-035` | `OSPS-DO-01.01`, `OSPS-QA-01.01`, `OSPS-QA-01.02`, `OSPS-QA-02.01` | 4 | User guide, public source and history, and direct-dependency inventory |
| **Total** | **Every Level 1 control in the named checklist version** | **24** | `MCP-035` must link durable evidence or exact applicability reasoning for every row before any achieved-status claim or README badge appears |

### Enterprise assurance target board

This is the cross-framework evidence view, not a second task queue. Only the
OSPS Level 1 row is a project-wide M3 exit target. `MCP-035` also performs a
bounded verification of the existing artifact provenance against SLSA Build
Level 2; the other rows are zero-cost candidate assurance targets whose
adoption value and evidence overlap can be assessed after M3. None is currently
claimed or automatically activated.

The following public-proof policy applies to this board and every assurance
target added later:

- Before activation, the owning ticket must identify the exact official proof
  mechanism, fee, issuer terms, human account or legal-assertion prerequisite,
  public evidence URL, and revalidation and removal triggers.
- If the target provides an official badge or conformance mark for the achieved
  scope, the ticket cannot be `Done` until that proof is issued, linked from the
  README to issuer or assessment evidence, and verified on exact `main`. If a
  required fee is unfunded, the target remains candidate or deferred rather
  than silently dropping the proof requirement.
- If the target has no applicable official badge, completion requires a
  conspicuous, accurately scoped README evidence link instead. Do not invent a
  shield or use issuer branding in a way that implies certification, approval,
  or a level the project did not receive.
- Revalidate both the underlying controls and the public proof on the owning
  ticket's review triggers. Correct or remove a stale, withdrawn, broken, or
  over-broad claim immediately.

| Target | Permitted assertion and scope | Direct fee | Adoption value | Owning work | Required public proof if activated | Status and public-claim gate |
| --- | --- | --- | --- | --- | --- | --- |
| [OpenSSF OSPS Baseline `v2026.02.19` Level 1](https://baseline.openssf.org/versions/2026-02-19-checklist.md) | Dated, scoped, evidence-backed project self-assessment | $0 | Broad minimum open-source security and governance signal | `MCP-030` through `MCP-035` | Official [BadgeApp](https://www.bestpractices.dev/) `baseline-1` self-certification badge linked to the public assessment, plus the dated project crosswalk | Accepted M3 gate; `MCP-035` and M3 cannot be `Done` before every mapped control passes and the issued badge and exact-main README link are verified |
| [OpenSSF Best Practices Passing](https://www.bestpractices.dev/en/criteria/0) | Public project self-certification against the badge criteria | $0 | Familiar open-source project-health signal | Candidate gap assessment after M3 | Official BadgeApp Passing badge linked to the public project assessment | Not active and not claimed; if accepted later, its ticket must verify every criterion and obtain and link the official badge before becoming `Done` |
| [SLSA v1.0 Build Level 2](https://slsa.dev/spec/v1.0/levels#build-l2) | Artifact-scoped statement only for release assets with verified provenance | $0 | Supply-chain provenance signal for enterprise artifact review | Existing `MCP-029` evidence; exact-scope audit in `MCP-035` | Signed provenance and attestations plus a public verification record for the exact artifacts; no issuer-awarded project badge is currently identified, so no certification-like shield may be invented | Evidence exists for the attested `v0.1.0` assets; public wording remains gated on scope verification and must not imply all channels or future releases |
| [NIST SSDF 1.1](https://csrc.nist.gov/pubs/sp/800/218/final) | Self-evaluated practice alignment, never “NIST certified” | $0 | Enterprise procurement vocabulary and control crosswalk | Candidate gap assessment after M3 | Dated public self-assessment and crosswalk; no official NIST certification badge is currently identified, so use a scoped evidence link rather than a certification-like shield | Not active and not claimed; a later accepted ticket must define applicable practices, evidence, gaps, assessment date, and recheck official proof availability |
| [OpenChain ISO/IEC 18974](https://openchainproject.org/security-assurance) and [ISO/IEC 5230](https://openchainproject.org/license-compliance) | Narrow organization-process self-certification only where the official conformance path permits it; never a product certification | $0 self-certification resources; implementation expertise may cost | Licensing and vulnerability-management process signal for larger organizations | Funding- and capacity-aware stretch after M3 | Applicable [official OpenChain conformance badge](https://openchainproject.org/news/2023/06/22/get-openchain-conformance-badges), displayed under issuer terms and linked to the current conformance evidence | Not active and not claimed; if accepted later, its ticket requires a defined organization scope, owner, evidence plan, successful self-certification, and official badge before becoming `Done` |

### Immediate focus

1. Maintain the completed `MCP-030` through `MCP-035` controls, public evidence,
   official BadgeApp `baseline-1` status, README trust surface, and exact-main
   verifier. Reassess or remove claims on every documented trigger.
2. Keep M4, M5, and every side quest inactive until a later outcome is
   deliberately selected, accepted, assigned, and activated under its own
   canonical Goal; completing M3 does not choose a successor.
3. Preserve immutable `v0.1.0`, its signed annotated release and operational
   recovery tags, seven attested assets, byte-identical Cargo package, exact
   tap formula, protected environments, and repository-scoped credentials.
   Never move a published tag or replace an immutable asset; issue a new
   version for any future release correction.
4. Monitor the public GitHub, Cargo, and source-building Homebrew channels and
   distinguish product-fit feedback from Rust/Homebrew prerequisite friction.
   Keep the published limitations accurate: `v0.1.0` has no project-issued
   macOS or Windows binary and no WinGet package.
5. Treat OSPS Level 1 as the only M3 assurance gate. Best Practices, NIST SSDF,
   OpenChain, higher OSPS levels, and independent certification remain
   non-gating candidates until separately accepted; activation must preserve
   the board's official-proof requirement. SLSA wording remains limited to the
   exact attested release artifacts verified by `MCP-035`.
6. Retain the funded six-target signed-native workflow, preflight, metadata,
   trust checks, and runbook without activating them. `SIDE-006` still requires
   explicit funding, assignment, and a separate exact Goal-mode objective.

## Decision log

| ID | Decision | Status | Date | Reason and consequence |
| --- | --- | --- | --- | --- |
| DEC-001 | Implement the product in Rust | Accepted | 2026-08-06 | User-selected direction; optimize for a portable native CLI, explicit types, and predictable local execution |
| DEC-002 | Begin as one modular binary crate | Accepted | 2026-08-06 | Fastest coherent delivery shape; split only for a demonstrated consumer or boundary |
| DEC-003 | Use this Markdown document as the initial project tracker | Accepted | 2026-08-06 | Keeps scope and evidence beside the code; migrate issue detail later without duplicating milestone truth |
| DEC-004 | Use the strict, client-independent [canonical JSON v1 contract](#canonical-configuration-v1-decision) for local STDIO servers | Accepted | 2026-08-06 | Named server-map entries contain literal command, ordered arguments, and literal environment strings; deterministic writes and explicit unsupported-version failures prevent ambiguous normalization |
| DEC-005 | Prove M1 on macOS with current stable Claude Desktop and Cursor, managing only global user configuration | Accepted | 2026-08-06 | The repository owner confirmed the complete `OPEN-01` recommendation and access to both clients for later verification. [Claude Desktop's global JSON](https://modelcontextprotocol.io/docs/develop/connect-local-servers) and [Cursor's global JSON](https://docs.cursor.com/context/model-context-protocol) exercise two real adapters without adding another platform or TOML; Cursor project configuration remains untouched, Linux and Windows stay sequenced later, and a controlled current-client macOS smoke test gates the support claim |
| DEC-006 | Reconcile exact normalized definitions as sorted add/update/no-op work and preserve target-only entries as non-mutating drift | Accepted | 2026-08-06 | One outcome per unioned server name makes plans deterministic; omitting deletion prevents accidental loss, and `DEC-023` now fixes the M1 ownership and prune boundary |
| DEC-007 | Keep health testing outside M1 | Working assumption | 2026-08-06 | Safe reconciliation is the core proof; process lifecycle and protocol handling become the next vertical feature |
| DEC-008 | Complete the main story in strict order and isolate optional work as side quests | Accepted | 2026-08-06 | Prevents foundation gaps and hidden prerequisites; a blocked story ticket blocks later story work until resolved or formally replaced |
| DEC-009 | Preserve README as the north-star product specification and public marketing page | Accepted | 2026-08-06 | README describes the intended finished experience; PROJECT, code, tests, and releases track current delivery truth |
| DEC-010 | Use Rust-native and OS-native distribution rather than npm | Accepted | 2026-08-06 | Canonical GitHub binaries plus Homebrew, WinGet, and a distinctly named Cargo package make the executable accessible with or without a Rust toolchain |
| DEC-011 | Classify Codex as a supported client target with its own adapter | Accepted | 2026-08-06 | [Codex MCP configuration](https://developers.openai.com/codex/mcp/) is shared by the ChatGPT desktop app, Codex CLI, and IDE extension; TOML is real target variation that belongs behind the existing adapter boundary |
| DEC-012 | Use one canonical Goal-mode objective per active main-story ticket | Accepted | 2026-08-06 | Thread goals provide persistence for long work, while exact per-ticket objectives preserve the ordered scope and evidence gates without replacing repository truth; only one unfinished ticket goal may be active in a thread |
| DEC-013 | Track the current stable Rust toolchain and leave the minimum supported version unspecified until a lower-bound CI policy exists | Accepted | 2026-08-06 | `rust-toolchain.toml` selects stable with rustfmt and Clippy, and MCP-003 verifies that toolchain on every CI run; do not claim `rust-version` from current-stable evidence alone |
| DEC-014 | Use `enjoyable-mcp-sync` as the Cargo package and `mcp-sync` as its binary | Accepted | 2026-08-06 | The unrelated `mcp-sync` name is occupied; `enjoyable-mcp-sync` was rechecked immediately before `MCP-029` publication and is now the published Cargo identity for the installed `mcp-sync` executable |
| DEC-015 | Use Clap as the single CLI parser | Accepted | 2026-08-06 | Clap 4.6.6 provides maintained, cross-platform help, version, and future command parsing under MIT OR Apache-2.0 without introducing a disposable parser |
| DEC-016 | Introduce testing tools only with the first ticket that demonstrates their need | Accepted | 2026-08-06 | The [testing tool introduction plan](#testing-tool-introduction-plan) keeps `cargo test` as the baseline, makes each adopting ticket own configuration and evidence, and prevents speculative dependencies while preserving explicit review points for conditional tools |
| DEC-017 | Implement canonical JSON with Serde behind a strict duplicate-detecting and structurally redacted boundary | Accepted | 2026-08-06 | `serde` and `serde_json` provide maintained serialization primitives, while explicit shape and semantic validation prevent permissive map behavior, ordered maps produce deterministic bytes, and custom debug/errors prevent process values from leaking |
| DEC-018 | Resolve macOS configuration roots from an injected environment and keep the initial filesystem port read-only | Accepted | 2026-08-06 | Required absolute, traversal-free `HOME` and optional `XDG_CONFIG_HOME` inputs make path behavior deterministic without touching real user configuration; deferring mutation methods prevents atomicity, backup, and rollback contracts from being guessed before their owning tickets |
| DEC-019 | Manage only `command`, `args`, and `env` inside each Claude Desktop `mcpServers` entry | Accepted | 2026-08-06 | Matches canonical JSON v1 and the current local STDIO contract while preserving unknown top-level data, target-only servers, and native fields such as `cwd`; the adapter consumes an existing plan and never interprets remote account connectors or desktop extensions as this local file format |
| DEC-020 | Manage only command-based local entries in global Cursor configuration | Accepted | 2026-08-07 | Cursor stores global and project configuration separately and places local STDIO plus remote HTTP/SSE entries in `mcpServers`. M1 resolves only `~/.cursor/mcp.json`, owns only `command`, `args`, and `env` within compatible local entries, preserves `envFile` and all remote or unknown data, and refuses local additions that collide with unmanaged remote names; project-level `.cursor/mcp.json` remains outside the adapter boundary |
| DEC-021 | Make initialization a deterministic, create-only import transaction | Accepted | 2026-08-07 | `init` checks that canonical state is absent, completes every implemented client read and the entire pure merge before mutation, reports structural disagreements or unsupported-name collisions, and publishes validated bytes through a synced same-directory temporary file without replacing an existing path. Native files remain read-only; guarded canonical replacement is implemented by `MCP-010`, while target transaction rollback is implemented by `MCP-011` |
| DEC-022 | Treat canonical `add` as a complete-definition upsert and `list` as structural metadata only | Accepted | 2026-08-07 | Repeated literal `--arg` and `--env KEY=VALUE` inputs avoid shell parsing; an existing name is replaced as one validated unit, exact semantic equality skips all writes, and changed state receives an exact `.bak` before guarded atomic replacement. Normal output may show escaped names, argument counts, and escaped environment key names, but never commands, arguments, or environment values |
| DEC-023 | Preserve target-only entries in M1 and require explicit ownership provenance before any future prune | Accepted | 2026-08-07 | M1 sync may add desired names and update only `command`, `args`, and `env` inside compatible local entries while preserving every unowned field. Canonical absence produces non-mutating drift, never deletion; unmanaged remote-name collisions remain errors. A future prune requires a separately accepted command, provenance model, recovery contract, and ticket rather than inferring ownership from absence |
| DEC-024 | Adopt the six-target signed, attested, and immutable release contract | Superseded | 2026-08-07 | `DEC-036` supersedes only this contract's all-at-once M2 timing and `v0.1.0` assignment. Its separate ARM64/x64 macOS, GNU/Linux, and Windows matrix; Apple signing/notarization; Windows Public Trust signing; checksums; target-specific SPDX SBOMs; build attestations; immutable-release rule; and stable identities remain the accepted safety contract for funding-dependent `SIDE-006`. Missing signing authority still blocks those native artifacts rather than permitting an unsigned fallback |
| DEC-025 | Satisfy the controlled two-client M1 criterion inside `MCP-012` | Accepted | 2026-08-07 | The repository owner's exact active objective requires every M1 acceptance criterion before `MCP-012` is `Done`, so the current-client gate cannot remain exclusively in its successor. `MCP-012` owns the first passing backup-protected Claude Desktop and Cursor verification; `MCP-013` remains the immediate successor for the north-star README audit, refreshed client verification as needed, and detailed usage and recovery guidance |
| DEC-026 | Manage only Windsurf's documented global legacy Cascade configuration | Accepted | 2026-08-07 | The official contract places command-based STDIO and commandless `serverUrl`/`url` transports together in `~/.codeium/windsurf/mcp_config.json`, while the vendor now distinguishes the Devin Local agent's separate configuration. The adapter owns only `command`, `args`, and `env` for compatible local entries; it preserves headers, authentication, `disabledTools`, interpolation literals, unknown data, unmanaged remote entries, and target-only servers, refuses a local collision, and makes no Devin Local or current-client smoke claim |
| DEC-027 | Manage only VS Code's native default user-profile MCP configuration | Accepted | 2026-08-07 | Official VS Code documentation places native user-profile MCP configuration in `mcp.json` under root `servers`. On macOS the adapter resolves exactly `~/Library/Application Support/Code/User/mcp.json`, accepts explicit or omitted local STDIO `type`, owns only `command`, `args`, and string-valued `env`, emits explicit `"type": "stdio"` for additions, preserves root `inputs`/`sandbox`, interpolation and unknown fields, and treats remote/unknown transports plus number/null environment entries as unmanaged collision-protected data. Workspace, named/remote profiles, Insiders, portable installations, Cline/Roo stores, and Agent Host/Copilot CLI configuration remain distinct and excluded; fixture and built-binary evidence is not a current-client smoke claim |
| DEC-028 | Manage only canonical-compatible local STDIO entries in Codex's global user configuration | Accepted | 2026-08-07 | [Codex configuration precedence](https://developers.openai.com/codex/config-basic/#configuration-precedence) makes trusted project `.codex/config.toml` files higher-precedence layers, while the [Codex MCP contract](https://developers.openai.com/codex/mcp/) places local STDIO and remote HTTP/OAuth servers in the same `mcp_servers` map shared by the ChatGPT desktop app, Codex CLI, and IDE extension. `MCP-016` therefore resolves only global `~/.codex/config.toml`, owns only `command`, `args`, and `env` in unambiguous local STDIO entries, and uses a TOML-native structural edit that preserves comments, unrelated settings, and all unowned local fields. Project layers remain undiscovered and untouched; `url`-based remote entries, authentication, headers, OAuth settings, unknown or mixed transports, and credential stores remain unmanaged and structurally preserved; a canonical local definition that collides with any unmanaged name fails before mutation. A successful global sync does not claim to override a higher-precedence project definition, perform OAuth login, or prove current-client behavior. |
| DEC-029 | Implement one explicit final-session MCP initialize compatibility boundary | Accepted | 2026-08-07 | `mcp-sync test <name>` is the only command that executes a configured server. It sends the final session-based `2025-11-25` initialize shape, accepts the four final handshake revisions through `2024-11-05`, bounds one newline-delimited response to five seconds and 1 MiB, validates JSON-RPC and required result structure before sending `notifications/initialized`, and allows 500 milliseconds for stdin-driven shutdown before force termination and reaping. The child receives canonical environment entries plus inherited `PATH` only when canonical state omits it; stderr and raw response/error data remain undisclosed. The current stateless `2026-07-28` protocol is a materially different boundary and is not claimed by this initialize-specific ticket. |
| DEC-030 | Resolve Linux user data through XDG and require native x64/ARM64 whole-suite CI | Accepted | 2026-08-07 | Linux uses a non-empty absolute `XDG_CONFIG_HOME`, falling back to `$HOME/.config`, for canonical configuration plus Claude Desktop and VS Code user data; Cursor, Windsurf, and Codex keep their documented home-relative paths. The platform-neutral resolver and existing adapters preserve all ownership and mutation rules. [GitHub's standard hosted-runner contract](https://docs.github.com/en/actions/how-tos/write-workflows/choose-where-workflows-run/choose-the-runner-for-a-job) supplies native `ubuntu-24.04` x64 and `ubuntu-24.04-arm` ARM64 runners, and each must assert its Rust host and pass the complete synthetic-home quality script. This proves GNU libc source-checkout behavior only: it does not add musl/Alpine, Windows, packaging, or a Linux current-client smoke claim. |
| DEC-031 | Protect stable release tags and publication credentials across every release path | Accepted | 2026-08-08 | Release immutability begins only after publication, so `MCP-021` activated a `v*` tag ruleset that restricts creation to the authorized release path and prevents update or deletion outside an explicit emergency bypass. `MCP-028` preserves those controls while separating free and paid automation, `MCP-029` published through the protected least-privilege release environment, and `SIDE-006` must use the same standard when activated. GitHub's immutable-release setting requires administration read access unavailable to the standard Actions token, so the controlled operator-side verifier bound that live setting plus both protected environment policies to the exact release `main` commit immediately before approval; Actions recorded explicit confirmation, verified the public ruleset, and required successful exact-commit authorization plus final immutable release state. OIDC or trusted publishing remains preferred where supported; an interactive administration PAT is never stored or reused as a release credential. |
| DEC-032 | Give every classified side quest a canonical Goal-mode objective | Accepted | 2026-08-08 | A side-quest objective provides the same persistent scope and evidence contract as a main-story objective without changing eligibility, independence, priority, or the one-side-quest WIP cap. One thread still carries only one unfinished goal, so concurrent main-story and side-quest work uses separate threads; completing a side quest activates no successor. |
| DEC-033 | Resolve Windows user paths through standard profile variables and require native MSVC x64/ARM64 evidence | Accepted | 2026-08-08 | Windows requires non-empty absolute, traversal-free `%USERPROFILE%`, `%LOCALAPPDATA%`, and `%APPDATA%`: home-relative Cursor, Windsurf, and Codex files remain beneath the profile; the machine-specific canonical configuration lives beneath local application data; and Claude Desktop plus VS Code use roaming application data according to their documented Windows contracts. The existing `tempfile` same-directory persistence boundary remains the maintained atomic-replacement-equivalent instead of introducing an unreviewed platform primitive, but native tests must prove overwrite, exact backup, compensation, and reverse-order rollback on both `x86_64-pc-windows-msvc` and `aarch64-pc-windows-msvc`. [GitHub's hosted-runner contract](https://docs.github.com/en/actions/reference/runners/github-hosted-runners) supplies `windows-2025` x64 and public-preview `windows-11-arm` ARM64 runners; preview availability is therefore an explicit CI dependency, not a release-support claim. |
| DEC-034 | Retain one adjacent backup generation and make restore a validated reversible operation | Accepted | 2026-08-08 | `mcp-sync restore <configuration>` selects only canonical, Claude Desktop, Cursor, Windsurf, VS Code, or Codex global paths and supports a non-mutating `--dry-run`. The adjacent `.bak` must be a regular file whose exact bytes pass the selected configuration parser before apply. An existing regular target is atomically replaced while its immediately preceding bytes become the new `.bak`; a missing target is created without consuming or rewriting the backup. Equal target and backup bytes are a no-op. Missing, invalid, symbolic-link, non-regular, stale, or concurrently changed inputs fail closed without exposing contents. Successful `add`, `sync`, and existing-target `restore` replacements retain exactly the immediately preceding generation with no time-based expiration; creations have no prior-file backup, no-ops and rolled-back or refused operations preserve prior retention state, and longer history remains an explicit external archival responsibility. |
| DEC-035 | Notarize the exact macOS payload through a temporary accepted ZIP before tar packaging | Accepted | 2026-08-08 | Apple's notary submission formats do not include the advertised `.tar.gz`. Each funded native macOS release job therefore signs the final Mach-O first, places that executable and the exact release documents in a temporary ZIP, requires `notarytool` status `Accepted` plus local `codesign` and Gatekeeper success, and then packages those unchanged bytes into the target-specific tar archive. The temporary transport is not published, no post-acceptance binary mutation is allowed, and this preserves the signed-and-notarized payload contract retained for `SIDE-006` by `DEC-036`. |
| DEC-036 | Ship a zero-cost market-validation release before funding signed-native distribution | Accepted | 2026-08-08 | Paid Apple and Windows signing authorities are a financial blocker before product demand is validated, but weakening native trust checks would create a misleading and difficult-to-reverse release precedent. `MCP-021` is therefore superseded at the post-`MCP-020` story point by `MCP-028` and `MCP-029`. `v0.1.0` uses an immutable GitHub Release with attested GNU/Linux x64/ARM64 binaries, crates.io source installation on supported platforms, and a source-building organization Homebrew formula; it publishes no project-issued macOS or Windows binary and no WinGet package. Reusable automation is named for this technical source and GNU/Linux artifact boundary rather than the temporary business purpose, so later signed-native activation does not require business-phase renaming. The existing signed workflow, six-target preflight, packaging, downstream generation, trust checks, all-channel smoke, and runbook remain maintained but inactive under proposed `SIDE-006`, which may start only after explicit funding and activation and must use a later version. This changes release sequencing and M2 scope, not the README's north-star destination or the safety bar for future native binaries. |
| DEC-037 | Recover post-publication automation without changing immutable `v0.1.0` | Accepted | 2026-08-08 | The first protected Homebrew dispatch failed before credential use or tap mutation because one crates.io download used curl's unidentified default client and received `403`; the immutable GitHub Release and crates.io bytes remained correct. Rerunning the frozen tag workflow would have repeated the failure, while moving `v0.1.0`, replacing assets, manually bypassing the tap-scoped key, or widening the protected environment would have weakened accepted controls. After the explicit-client correction merged and exact-main CI plus both preflights passed, the separately authorized signed annotated operational tag `v-mcp-029-homebrew-recovery-1` was fixed directly to that exact current `main` commit. It triggered no release publisher itself and is not a product version. The manual Homebrew dispatch from that exact tag remained behind the tag-only `release` environment, rechecked the public tag ruleset and all three recovery-commit gates, then checked out canonical `v0.1.0` and published only its absent byte-identical formula. The operational tag is retained without a GitHub Release or Cargo version and is never moved or deleted; future registry requests always identify the project client. |
| DEC-038 | Promote zero-cost enterprise assurance into an ordered M3 and gate public trust claims on evidence | Accepted | 2026-08-08 | Enterprise adoption makes the repository, disclosure, community, supply-chain, and organization outcomes audited under `SIDE-001` through `SIDE-005` part of the product path rather than optional polish. Their side-quest forms are superseded and promoted into ordered `MCP-030` through `MCP-034`; `MCP-035` closes the complete evidence, self-attestation, maintenance, and public-claim gate. M3 targets exactly all 24 OpenSSF OSPS Baseline `v2026.02.19` Level 1 controls at $0 through a dated, scoped, evidence-backed self-assessment. M2 remains closed and immutable, `SIDE-006` remains independent and funding-gated, and M3 does not claim OSPS Level 2 or 3, another assurance framework, independent certification, regulatory compliance, or paid platform signing. README badges or trust language may describe only achieved scope after `MCP-035`, and must be corrected or removed when evidence becomes stale. |
| DEC-039 | Protect `main` through strict pull requests without creating a single-maintainer deadlock | Accepted | 2026-08-09 | The repository currently has one administrator, no teams, and five stable CI contexts emitted by GitHub Actions integration `15368`. The active default-branch ruleset therefore requires pull requests, strict latest-base status checks bound to that integration, resolved conversations, deletion protection, and non-fast-forward protection, with no standing bypass actor. Required approvals remain zero until an independent trusted reviewer exists; code-owner and last-push approval are off. Merge, squash, and rebase remain allowed through pull requests. Verified commit signatures remain optional until external-contributor and GitHub-generated merge commits pass an end-to-end signing exercise; contribution sign-off remains separate `MCP-032` scope. Emergency administration requires an explicit, auditable ruleset edit by a repository administrator, constrained to the smallest ref, rule, and time window, followed by immediate restoration and verification; no routine direct push to `main` is an emergency path. This protects contributors from impossible self-approval while satisfying `OSPS-AC-03.01` and `OSPS-AC-03.02`. Revisit approvals when a second trusted reviewer exists, signing after the end-to-end exercise, and check bindings whenever a required job or GitHub integration identity changes. |
| DEC-040 | Use GitHub's private repository route and available zero-cost security controls as the initial disclosure and secret-prevention boundary | Accepted | 2026-08-09 | The maintainers are the public security contacts through GitHub private vulnerability reporting; no personal address, identity, or contact credential is published. Only the latest public release is supported, with explicit acknowledgement, triage, update, remediation, and coordinated-disclosure targets rather than a guaranteed SLA. Dependency graph and alerts, Dependabot security updates, CodeQL default setup for Rust and Actions, provider-pattern secret scanning, push protection, and private reporting are enabled and verified without disclosing findings. Scheduled dependency updates remain `MCP-033` scope. GitHub Free does not entitle this organization repository to non-provider patterns or partner-pattern validity checks, so both stay explicitly disabled until the plan and feature boundary changes. This decision proves only the two controls owned by `MCP-031`; it creates no badge, certification, or complete-baseline claim. |
| DEC-041 | Use public GitHub workflows and inbound-equals-outbound MIT terms without nominal review or sign-off controls | Accepted | 2026-08-09 | Structured GitHub Issues and pull-request conversations are the public mechanisms for defects, proposed changes, and usage obstacles; GitHub Discussions remains disabled, while a root `SUPPORT.md` provides the concise user entry point, vulnerabilities continue through `MCP-031`'s private route, and conduct reports begin with a detail-free public contact request followed by private handling. Contributions are submitted under the outbound MIT License without copyright assignment or a CLA; DCO sign-off is a separate choice and is not required or enforced. The authoritative `mcp-sync` repository and its in-scope `homebrew-tap` formula are the complete two-repository project inventory, and both maintain root MIT license files while released software archives carry `LICENSE`. Official project channel URIs use HTTPS. `CODEOWNERS` remains absent until a real independent reviewer exists, avoiding a nominal or impossible review path. This decision changes no repository security setting, dependency automation, immutable release asset, or public assurance claim. |
| DEC-042 | Use grouped update review, exact selected actions with mandatory SHAs, validation before privilege, and a text-only source-history gate | Accepted | 2026-08-09 | Cargo and GitHub Actions receive separate weekly grouped Dependabot pull requests through the normal protected path; no bot update is auto-merged. Every committed action uses a reviewed full commit SHA plus an update hint, while repository Actions policy permits only the ten direct or reviewed-transitive action paths in `.github/actions-policy.json`, disables broad GitHub-owned and verified-creator allowances, and separately requires full SHAs. GitHub-managed CodeQL default setup stays enabled and separately verified because GitHub exempts that generated workflow from public-action restrictions. Pull-request workflows remain read-only and secretless, direct GitHub-expression interpolation into shell commands is rejected, and the Homebrew publisher validates exact manual input and dispatch ref before protected-environment access. Tracked submodules, generated executable/package extensions, and binary blobs fail the stable `Dependency policy` check; release artifacts remain generated, attested, and published outside source history. The immutable `v0.1.0` release, exact Cargo and Homebrew bytes, dormant funded workflow, stable check contexts, and later public-claim gate do not change. |
| DEC-043 | Use default-no-access organization membership and a tested single-owner recovery exception until a trusted second owner exists | Accepted | 2026-08-09 | Explicit owner approval changed organization base repository permission from `read` to `none` and disabled member repository creation, so future grants and repositories require deliberate owner action. No current human access was removed. The current sole owner accepts the residual risk of single-owner continuity rather than promoting a nominal or untrusted owner, keeps the recovery plan and exercise record private, and must add a genuinely trusted second owner when one exists. Teams are created only for a real collaborator cohort, installed applications are privately reviewed for least privilege, normal automation remains short-lived and repository-scoped, and the one protected tap-only deploy key remains the documented cross-repository exception. This decision does not itself satisfy the secure-method, successful recovery, live-verifier, or later public-assurance gates. |
| DEC-044 | Make official public proof part of completing every accepted assurance target that provides it | Accepted | 2026-08-09 | Assurance work is intended to reduce adoption friction as well as improve controls, so invisible evidence alone does not complete a target when its issuer offers a badge or conformance mark for the achieved scope. The owning ticket must obtain that official proof, follow its terms, link it from the README to issuer or assessment evidence, verify it on exact `main`, and define revalidation and removal triggers. M3 therefore requires the official BadgeApp `baseline-1` self-certification badge after all 24 named controls pass; future accepted Best Practices or OpenChain targets likewise require their applicable official badges. Human OAuth, public-attribution, or legal-assertion prerequisites remain explicit owner gates and are not silently performed or copied into repository evidence. SLSA and NIST SSDF currently use accurately scoped public artifact or self-assessment evidence because no applicable issuer-awarded project certification badge has been identified; the project must recheck that fact at activation and must never fabricate an official-looking shield. An unfunded required proof keeps a target candidate or deferred until a later decision funds it; a stale or withdrawn proof is corrected or removed immediately. |

### Open decisions

`OPEN-07` is resolved by `DEC-028`, `DEC-036` resolves the release-funding
sequence, `DEC-037` fixes the completed immutable-release recovery path, and
`DEC-038` establishes the ordered M3 assurance arc, `DEC-039` resolves
`MCP-030`'s approval, merge-method, bypass, and signed-commit choices,
`DEC-040` fixes the `MCP-031` disclosure and zero-cost feature boundary, and
`DEC-041` fixes `MCP-032`'s public workflow, repository, review, and inbound
licensing choices, `DEC-042` fixes `MCP-033`'s dependency, action,
untrusted-workflow, distribution-authentication, and tracked-artifact policy,
and `DEC-043` fixes `MCP-034`'s default-no-access and single-owner recovery
choice. `DEC-044` makes issuer-provided public proof a completion requirement
for every accepted assurance target that offers it and fixes the specific
OpenSSF `baseline-1` badge requirement for `MCP-035` and M3.
`MCP-029`, M2, `MCP-030` through `MCP-035`, and M3 are complete. `MCP-034`'s
explicitly authorized organization update, accepted single-owner exception,
private secure-method, application, and successful-recovery confirmations,
owner-only aggregate evidence, and passing non-disclosing verifier satisfy its
ticket gate without publishing private details. `MCP-035`'s protected public
assessment, issued badge, exact-main verification, and scoped SLSA statement
satisfy the final M3 gate. No later main-story milestone or side quest is
implicitly active.
`DEC-031`, `DEC-036`, and
`DEC-037` remain the release boundary; protected publication access is durable
evidence under `RISK-17`, while paid signing authority under `RISK-12` blocks
only dormant `SIDE-006`.

## Risk register

| ID | Risk | Impact | Likelihood | Current mitigation | Trigger for escalation | Status |
| --- | --- | --- | --- | --- | --- | --- |
| RISK-01 | A merge or partial failure loses user configuration | Critical | Medium | Pure plan/apply separation, guarded exact-byte writes, recoverable backups, reversible receipts, reverse-order five-target rollback, non-mutating drift, bounded client ownership, unmanaged-name collision refusal, interrupted-write compensation, and the complete synthetic failure matrix are implemented. Fixed-target restore validates retained bytes, swaps existing target and backup exactly, recreates missing targets without consuming the backup, and compensates an interrupted creation. A forced Codex fifth-target failure still restores Claude Desktop, Cursor, Windsurf, and VS Code plus their pre-existing backups exactly. Current Cursor verification is isolated; current Claude verification uses a no-clobber backup and verified exact restore; every represented installed release channel completes its synthetic recovery journey | Any unrecoverable fixture mutation or ambiguous ownership case | Mitigated — sync, restore, and installed-channel recovery proof; monitor future releases |
| RISK-02 | Secrets leak through plans, errors, logs, fixtures, or snapshots | High | Medium | Import conflicts, canonical add/list reports, sync and restore plans/reports, per-target transaction reports, rollback errors, four JSON adapters, the TOML-native Codex document/render debug surfaces, and health-process failures expose structure only. Health journeys prove commands, arguments, environment values, raw stdout/stderr, malformed output, JSON-RPC error messages/data, and unchecked protocol versions remain undisclosed; restore journeys prove current, retained, malformed, unowned, and parser-error values remain undisclosed; the broader built-binary sentinels and safe byte/structural assertions cover the complete synthetic matrix without printing fixture contents. Current-client smokes read only synthetic output and structural status; no real configuration, credential, or account value was displayed | Any test or output path observes a secret value | Mitigated — configuration, restore, and health synthetic proof |
| RISK-03 | Native client schemas or paths drift | High | Medium | All five implemented global contracts and macOS, GNU/Linux, and Windows paths are fixture-backed. Current Cursor `3.15.6` and current Claude Desktop `1.26832.0` accept and initialize rendered global entries on macOS. GNU/Linux and Windows have deterministic built-binary coverage and successful native x64/ARM64 whole-suite CI; neither has a current-client smoke claim. Windsurf's legacy Cascade, VS Code's native default-profile, and Codex's global TOML contracts retain explicit exclusions and have no current-client smoke claim on any implemented platform | A client update invalidates fixture/discovery behavior or changes one of the documented global paths or schemas | Mitigated — monitor client updates and all five documented contracts |
| RISK-04 | Cross-platform file replacement behaves differently | High | Medium | No-clobber creation, guarded atomic replacement, one-generation restore, reversible target receipts, and reverse-order rollback are isolated behind filesystem ports with disposable backup, stale-byte, interrupted-write, symlink, non-regular, permission, compensation, and cleanup tests. The complete suite passes on native ARM64 and emulated x64 macOS, every test surface compiles warning-free for all six macOS/GNU/Linux/Windows targets, the native x64/ARM64 GNU/Linux and Windows CI matrix carries the expanded restore plus copied-binary Windows journey, and the final channel matrix passes installed recovery on every represented native host | Platform work requires weakening atomicity, restore, or rollback | Mitigated — three-platform source-checkout and published-channel contract; monitor future native runs |
| RISK-05 | Health checks hang or leave child processes running | High | Medium | The only execution boundary enforces a five-second response limit, 1-MiB message limit, 500-millisecond clean-shutdown allowance, force termination, reap, and a drop backstop. Focused and built-binary tests verify direct child PIDs are gone after response timeout, malformed output, and refused shutdown; native Windows x64/ARM64 PowerShell fixtures prove the same operating-system boundary. One operational-tag Windows x64 CI attempt hit the initialize-response timeout in the shutdown-resistant fixture; the termination path held, exact-main CI had passed at the same SHA, and the isolated [attempt-2 rerun](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31287080491) passed without a code change | The timing failure recurs, a health test exceeds its cleanup bound, or a tested child remains running | Mitigated — bounded cross-platform process proof; monitor timing recurrence |
| RISK-06 | Broad client/platform scope delays the first usable proof | High | High | M1 and the strictly sequenced M2 are complete; `DEC-038` accepts a bounded assurance-only M3, while the WIP limit and required classification prevent M4, M5, or funded side-quest scope from silently entering it | Work starts outside `MCP-030` through `MCP-035`, a later ticket starts early, or assurance work expands beyond the exact accepted target | Mitigated — M2 complete and M3 bounded; enforce intentional activation |
| RISK-07 | Premature plug-in or workspace abstractions slow iteration | Medium | Medium | One crate; abstractions require real variation or consumer | New public trait/package has only one hypothetical implementation | Mitigated by guidance |
| RISK-08 | The tracker becomes stale and stops reflecting delivery truth | Medium | Medium | Same-change updates, evidence links, weekly/completion review triggers, the completed M1 README/current-state audit, and `MCP-035` framework-version, assessment-date, scope, and evidence-drift triggers | PROJECT contradicts code or release evidence, an assurance statement outlives its evidence, or README changes the north-star or trust posture without a decision | Mitigated — monitor each ticket and assurance review trigger |
| RISK-09 | A side quest consumes main-story capacity or becomes a hidden prerequisite | Medium | Medium | Separate IDs, strict classification, canonical side-quest goals, one-side-quest WIP cap, separate goal threads for concurrent work, and `DEC-038` promotion of assurance work as soon as it became required | A main ticket or milestone gate depends on a remaining `SIDE-NNN`, a retired side-quest objective is activated, an active goal is silently replaced, or later story work starts early | Mitigated by classification and promotion rules |
| RISK-10 | The product or package is confused with unrelated projects using the `mcp-sync` name | High | Medium | Distinct package [`enjoyable-mcp-sync`](https://crates.io/crates/enjoyable-mcp-sync/0.1.0) is now published from the immutable repository release, while install commands remain publisher-qualified and the installed executable remains `mcp-sync`; provenance links and exact-byte verification bind the identities | Users install the unrelated `mcp-sync` crate or cannot connect `enjoyable-mcp-sync` to this repository | Mitigated for registry identity — monitor install feedback |
| RISK-11 | Codex's shared TOML file loses non-MCP settings or target-only MCP capabilities | Critical | Medium | `DEC-028` fixes a TOML-native, global-only local-STDIO ownership boundary. Exact fixtures and built-binary journeys preserve comments, formatting, unrelated settings, project/alternate-file/credential isolation, unowned local fields, remote HTTP/OAuth/mixed/opaque entries, target-only data, and unmanaged-name collision refusal without a JSON conversion | Any Codex fixture requires lossy conversion, reads OAuth credentials, mutates a project layer, or drops an unowned field | Mitigated — fixture and five-target built-binary proof; monitor Codex contract drift |
| RISK-12 | Required platform signing authority is unavailable or identity validation delays signed-native distribution | High | Medium | `DEC-036` removes paid signing from the M2 market-validation gate without weakening it: `v0.1.0` publishes no project-issued macOS or Windows binary, while the retained Developer ID/notarization and Windows Public Trust requirements, protected credentials, and CA-issued Authenticode fallback remain mandatory for `SIDE-006` | `SIDE-006` is proposed for activation without explicit funding, active Apple notarization credentials, or validated Windows signing authority | Accepted for M2 — blocks only funded `SIDE-006` |
| RISK-13 | An unprotected default branch permits direct, destructive, or insufficiently reviewed changes | High | Medium | The active public `Protect main` ruleset requires pull requests, strict latest-base success from five checks bound to GitHub Actions integration `15368`, and resolved conversations; blocks deletion and non-fast-forward updates; and has no standing bypass. Public and operator verifiers enforce configuration drift, while rejected direct-update, force-push, and deletion drills plus the normal protected PR and scoped emergency-administration path prove enforcement without changing release controls | A direct or force push reaches `main`, a branch is deleted, a merge bypasses expected checks or unresolved conversations, an expected check/app binding drifts, or emergency administration is not immediately restored | Mitigated — `MCP-030` protected and path-tested; monitor rule drift |
| RISK-14 | Contributors publicly disclose a vulnerability, credential, or unsafe diagnostic because reporting routes are unclear or unavailable | High | Medium | The recognized root security policy defines the private route, safe public fallback, redacted evidence, security contacts, response targets, and coordinated disclosure. Private vulnerability reporting, provider-pattern secret scanning, push protection, dependency alerts and security updates, and CodeQL are live and generically baseline-clean. Supplemental non-provider patterns and validity checks are unavailable on the current free plan and documented exactly. The GitHub-recognized bug, feature, and detail-free conduct forms plus chooser links now repeat the private-security and redaction boundary before public submission | A security report appears in a public issue, a pushed secret is accepted, the private route or security policy becomes unavailable, an open finding appears, or GitHub feature entitlement changes | Mitigated — `MCP-031` and `MCP-032` routes and controls complete; monitor drift |
| RISK-15 | Mutable Actions references or unmaintained dependencies compromise or silently weaken CI | High | Medium | Repository Actions requires full commit SHAs and permits only the ten exact direct or reviewed-transitive action paths in `.github/actions-policy.json`; broad GitHub-owned and verified-creator allowances are off. Merged `MCP-033` pins and upstream-verifies all 72 committed references, adds weekly grouped Cargo and Actions updates, validates action hints and untrusted workflow boundaries, keeps pull requests read-only and secretless, authenticates official distribution, and makes tracked executable/binary rejection part of the stable dependency check. Dependabot security updates and alerts remain enabled and baseline-clean; the protected implementation and exact-main CI, CodeQL, both release preflights, operator audit, and expected invalid-input isolation exercise all pass | An action hint stops resolving to its reviewed SHA, an action appears outside the inventory, fork code receives privilege, a generated or binary artifact is tracked, an actionable advisory remains unpatched, or dependency updates stop arriving | Mitigated — `MCP-033` merged and exact-main verified; monitor policy, workflow, distribution, artifact, and update drift |
| RISK-16 | A single organization owner becomes unavailable or an over-broad long-lived credential becomes a recovery dependency | High | Low | The 2026-08-09 aggregate audit and complete non-disclosing verifier prove organization-required secure-method 2FA, base access `none`, disabled member repository creation, read-only workflow-token defaults, no organization or repository Actions secrets, one protected-environment secret, and one used, verified, tap-only write deploy key. The sole owner explicitly accepted the residual risk, privately confirmed the installed-application review and successful recovery exercise, and must add a genuinely trusted second owner when one exists | The sole owner loses access, a maintainer cannot recover administration, a future member receives implicit private-repository access or creates an unreviewed repository, an installed application retains unnecessary scope, or one long-lived credential becomes necessary for routine automation | Mitigated with accepted single-owner residual risk — reverify on every trigger and add a trusted second owner when available |
| RISK-17 | A release tag or publication credential is changed or misused before immutable release controls take effect | Critical | Low | The active `v*` ruleset restricts tag creation, update, and deletion; both protected environments require review and constrain deployment to exact `main` or `v*` refs; release immutability is enabled; and `DEC-031` carries those controls through `MCP-028`, `MCP-029`, and later `SIDE-006`. Protected authorization bound canonical `v0.1.0` to its exact release commit; only that tag invoked the source/GNU/Linux publisher, while the funded workflow remained manual and rejected `v0.1.0`. The separately authorized signed annotated recovery tag resolves directly to corrected exact current `main`, where all three gates and the public ruleset pass; it has no product release and invoked only the manual Homebrew path. That protected job preserved the tag-only environment, verified immutable release and registry bytes before reading its one tap-repository deploy key, and created only the absent byte-identical formula. The release, crate, formula, and final read-only native channel matrix are now independently verified | Release automation creates, moves, deletes, or publishes a protected tag outside its exact authorized purpose; recovery accepts a stale commit or another ref; `v0.1.0` invokes paid signing; an approval proceeds after failed controls; or a downstream credential can mutate broader state | Mitigated — protected immutable release and constrained recovery complete; monitor future releases |
| RISK-18 | Source-based macOS and Windows installation creates more setup friction and weakens early conversion evidence | Medium | High | `v0.1.0` offers native GNU/Linux archives, Cargo installation across supported platforms, and source-built Homebrew on macOS/Linux; release notes state prerequisites and limitations, and market feedback must distinguish product-fit objections from installation friction. All represented native install and recovery journeys pass, and the funded native path remains available for deliberate activation after demand and budget justify it | Prospective users cannot complete installation, feedback repeatedly cites the toolchain boundary, or target users require WinGet or notarized macOS binaries before evaluation | Open — measure post-M2 market feedback; consider funding `SIDE-006` only after explicit activation |
| RISK-19 | A stale, over-broad, unofficial, or ambiguous self-attestation or badge misleads adopters | High | Medium | `DEC-038` limits M3 to the exact OSPS checklist version and Level 1 scope, while `DEC-044` requires issuer-provided proof where available and prohibits fabricated certification-like substitutes. `MCP-035` publishes the official dynamic BadgeApp badge linked to its 24/24 assessment, framework version, date, scope, limitations, evidence, issuer status, and annual plus event-driven revalidation and removal triggers; the exact-main verifier checks all of those fields and the rendered README. The seven-asset SLSA statement remains separate from project-wide claims | The framework or issuer status changes, a badge is withdrawn, a mapped control or evidence link fails, a security or release-pipeline change invalidates evidence, or public wording implies a higher level, certification, another framework, all artifacts, or current status that was not assessed | Mitigated — published and exact-main verified; monitor every revalidation trigger |

## Readiness and completion gates

### Ticket ready

A ticket is `Ready` when:

- it describes one observable outcome;
- acceptance evidence and failure behavior are explicit;
- a main-story ticket has a canonical goal objective that matches its outcome,
  boundary, evidence, and immediate successor;
- every dependency is `Done` and open decisions are recorded;
- a main-story ticket is the earliest incomplete ticket in the published
  sequence, or a side quest satisfies every independence rule;
- it fits the current milestone or is a necessary unblocker; and
- starting it will not violate the work-in-progress limit.

### Ticket done

A ticket is `Done` when:

- the observable outcome and important failure paths work;
- focused regression tests and the relevant broader checks pass;
- secrets and real user configuration were not used as test data;
- public behavior, architecture, and tracker documentation remain accurate;
- its evidence is linked from the ticket row; and
- no required rollout or verification step is being hidden behind “follow-up.”

### M3 enterprise assurance

M3 is complete only when:

- `MCP-030` through `MCP-035` are `Done` in order and D-13 links their durable
  public evidence;
- every one of the 24 OpenSSF OSPS Baseline `v2026.02.19` Level 1 controls in
  the [M3 control map](#m3-osps-level-1-control-map) passes, including each
  applicable conditional control, with no unresolved exception hidden behind a
  general statement;
- the public self-attestation states the framework version, Level 1, assessed
  repositories and organization/release boundaries, assessment date,
  limitations, evidence links, self-assessed status, and review triggers;
- the public BadgeApp assessment has achieved `baseline-1` against the named
  checklist version, and its official dynamic badge image is linked from the
  README to that assessment and verified on exact `main`;
- any README Security and Trust text or badge describes only achieved current
  self-certification evidence and does not imply Level 2 or 3, independent
  certification, regulatory compliance, another assurance framework, or paid
  native signing;
- any SLSA v1.0 Build Level 2 statement names only the exact `v0.1.0` artifacts
  whose provenance was verified and does not extend to all channels, releases,
  or project practices;
- no published evidence exposes private settings, vulnerability or secret
  findings, organization-member identities, authentication factors,
  credentials, or recovery material; and
- immutable `v0.1.0` and the complete M2 installed-channel evidence remain
  unchanged and valid.

### Milestone release

A milestone is complete only when every exit gate is evidenced, all P0 tickets
in its boundary are `Done` or explicitly removed through an accepted decision,
critical risks are closed or accepted with a concrete consequence, and the
preceding working behavior still passes. Main-story tickets must have completed
in order. Side quests never gate a milestone release. A documentation and
repository-assurance milestone such as M3 does not require a new product
release unless one of its accepted tickets explicitly makes that necessary.

## Tracking templates

Use the following compact shape when adding work:

```text
Ticket: MCP-NNN | SIDE-NNN — observable outcome
Track: Main story | Side quest
Milestone / priority:
Owner / target:
Status:
Immediate predecessor (main story only):
Canonical goal objective:
Acceptance evidence:
Blocker or risk link:
```

Use the decision log for choices that constrain more than one ticket:

```text
Decision: DEC-NNN — chosen direction
Status: Working assumption | Accepted | Superseded
Context:
Consequence:
Revisit trigger:
```
