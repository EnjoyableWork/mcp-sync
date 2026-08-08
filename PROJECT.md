# mcp-sync project plan

This is the canonical living document for product scope, delivery status,
tickets, deliverables, decisions, risks, and release gates. It is intentionally
lightweight enough to use before a hosted tracker is justified.

| Control | Current state |
| --- | --- |
| Document state | Active |
| Product state | The source-checkout macOS and GNU/Linux implementation reconciles global Claude Desktop, Cursor, Windsurf legacy Cascade, native VS Code default user-profile, and global Codex host configuration, and exposes one bounded named-server STDIO initialize health boundary; deterministic fixtures and native x64/ARM64 GNU/Linux CI prove the five-target configuration behavior without weakening its safety boundaries; Claude Desktop and Cursor retain macOS current-client sync smoke evidence, no Linux target has a current-client smoke claim, and the repository remains pre-release |
| Current milestone | M2 — City, in progress |
| Overall status | `MCP-018` is `Done`; only `MCP-019` is `Ready`, and every later main-story ticket remains `Proposed` |
| Current focus | Keep `MCP-019` as the sole ready successor; do not begin Windows work until its exact goal is intentionally activated and its owner is clear |
| Milestone target | Unscheduled; set after an owner and delivery capacity are known |
| Last reviewed | 2026-08-08 |
| Next review trigger | Start, completion, or blocking of `MCP-019`, or any change to the supported-client boundary |

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
tests must be verified by `MCP-021`.

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

The main story is currently defined through M2:

| Arc | Required ticket order |
| --- | --- |
| M0 — Foundations | `MCP-001` → `MCP-002` → `MCP-003` → `MCP-004` → `MCP-005` |
| M1 — Town MVP | `MCP-006` → `MCP-007` → `MCP-008` → `MCP-009` → `MCP-010` → `MCP-011` → `MCP-012` → `MCP-013` |
| M2 — Dependable city | `MCP-014` → `MCP-015` → `MCP-016` → `MCP-017` → `MCP-018` → `MCP-019` → `MCP-020` → `MCP-021` |

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

There are no active side quests. The deferred M3 and M4 items are expansion
candidates, not executable work. Before one starts, classify it as the next
ordered main-story arc or as a genuinely independent `SIDE-NNN` ticket.

## Goal-mode execution

[Codex Goal mode](https://learn.chatgpt.com/docs/long-running-work) is the
thread-level execution contract for one active ticket. It does not replace
repository truth: [README.md](README.md) defines the destination, this document
defines scope and order, and [AGENTS.md](AGENTS.md) defines durable engineering
constraints. A goal cannot waive a dependency, acceptance criterion, safety
rule, open decision, or required evidence.

Every main-story ticket has one exact canonical objective in the
[main-story goal catalog](#main-story-goal-catalog). To set one manually, paste
the objective after [`/goal`](https://learn.chatgpt.com/docs/developer-commands.md?surface=cli).
Goal-capable agents are explicitly authorized and required to reconcile the
active thread goal before beginning main-story work:

1. Identify the earliest incomplete main-story ticket. It must already be
   `Ready`; a goal never makes an otherwise ineligible ticket ready.
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
   the plan may not broaden the objective.
6. Mark the goal complete only after the ticket-done gate passes, durable
   evidence is recorded here, the ticket is `Done`, and only its immediate
   successor is made `Ready`. Do not activate the successor until work on it is
   intentionally started.

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
| M2 | City — dependable daily tool | README client coverage, health checks, cross-platform behavior, restore UX, and accessible release channels | Supported matrix passes in CI and one tagged release installs and smoke-tests through every advertised channel | In progress |
| M3 | Metropolis — extensible ecosystem | More clients, profiles, migrations, secret references, and a proven adapter contract | New adapters can be added without changing reconciliation invariants; upgrade paths are tested | Deferred |
| M4 | Country — managed platform hypothesis | Optional team/fleet policy, richer interfaces, and ecosystem distribution | Requires validated demand, a separate product plan, and compatibility commitments | Deferred |

Each milestone must leave the preceding milestone usable. Later work may replace
an internal implementation only when the public behavior is preserved or a
tested migration and rollback path exists.

## Distribution and accessibility target

`mcp-sync` will not be an npm package. It will be a Rust binary distributed
through several channels so users do not need a Rust toolchain merely to run
it. Every channel must install the same `mcp-sync` executable from one tagged,
tested release.

| Channel | Audience | Target installation | Role |
| --- | --- | --- | --- |
| GitHub Releases | All platforms, including users without a package manager | Download a prebuilt macOS, Linux, or Windows archive | Canonical immutable release containing versioned binaries, checksums, SBOMs, release notes, and provenance. |
| Homebrew tap | macOS and Linux users | `brew install EnjoyableWork/tap/mcp-sync` | Primary one-command Unix installation. An organization-owned tap keeps the formula under project control. |
| Windows Package Manager | Windows users | `winget install EnjoyableWork.mcp-sync` | Native discovery, installation, and upgrades through the public WinGet repository. |
| crates.io / Cargo | Rust developers with a toolchain | `cargo install enjoyable-mcp-sync` | Builds from the published source package and installs a binary still named `mcp-sync`. |

### Release artifact and trust contract

`OPEN-06` is resolved for M2 by `DEC-024`. The first stable public release is
`v0.1.0`; release candidates may use `v0.1.0-rc.N`, but Homebrew, WinGet, and
the normal Cargo install path must not point at a candidate. Every stable
channel consumes the same commit and version first published as the canonical
GitHub Release.

The initial binary matrix contains six separate 64-bit artifacts:

| Operating system | CPU | Rust target | Release archive |
| --- | --- | --- | --- |
| macOS | Apple Silicon / ARM64 | `aarch64-apple-darwin` | `mcp-sync-v{semver}-aarch64-apple-darwin.tar.gz` |
| macOS | Intel / x64 | `x86_64-apple-darwin` | `mcp-sync-v{semver}-x86_64-apple-darwin.tar.gz` |
| Linux | ARM64 | `aarch64-unknown-linux-gnu` | `mcp-sync-v{semver}-aarch64-unknown-linux-gnu.tar.gz` |
| Linux | x64 | `x86_64-unknown-linux-gnu` | `mcp-sync-v{semver}-x86_64-unknown-linux-gnu.tar.gz` |
| Windows | ARM64 | `aarch64-pc-windows-msvc` | `mcp-sync-v{semver}-aarch64-pc-windows-msvc.zip` |
| Windows | x64 | `x86_64-pc-windows-msvc` | `mcp-sync-v{semver}-x86_64-pc-windows-msvc.zip` |

These are supported [Rust platform targets](https://doc.rust-lang.org/rustc/platform-support.html),
but Rust's target tier is not product verification: `MCP-018` establishes
native GNU/Linux source-checkout behavior, `MCP-019` must do the same for
Windows, and `MCP-021` must build and smoke-test each advertised OS/CPU
artifact. The first release has no 32-bit, Alpine/musl, or combined
universal-macOS artifact. Add one only through a later accepted support
decision with native CI and install evidence.

Stable release trust is mandatory rather than best-effort:

- Sign each macOS Mach-O with the `com.enjoyablework.mcp-sync` identifier, a
  Developer ID Application certificate, hardened runtime, and a secure
  timestamp before packaging. Submit both macOS archives through Apple's
  current `notarytool` flow and require accepted notarization plus local
  `codesign` and Gatekeeper verification. Apple documents Developer ID signing
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

Homebrew selects the matching macOS or GNU/Linux archive and pins its SHA-256.
WinGet uses ZIP/portable manifests for the x64 and ARM64 Windows archives and
pins each installer SHA-256; WinGet currently supports both
[ZIP and portable packages](https://learn.microsoft.com/en-us/windows/package-manager/winget/).
Cargo publishes version `0.1.0` from the same tag only after the required live
name recheck. Release credentials belong in protected CI environments with
least privilege. Missing Apple or Windows signing authority blocks `MCP-021`;
it never silently downgrades the stable trust contract.

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
| Initial platform | macOS — accepted for M1; Linux followed in `MCP-018`, and Windows remains sequenced in `MCP-019` |
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
| README alignment and explicit post-M1 gaps | [README and operational-guidance audit](#readme-and-operational-guidance-audit), [M1 boundary](#m1-mvp-boundary), and the ordered `MCP-014` through `MCP-021` tracker rows |

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

- Runtime path selection recognizes only macOS and Linux. Linux resolves a
  non-empty absolute `XDG_CONFIG_HOME`, or `$HOME/.config` otherwise, for the
  canonical file and Electron-style user data. Claude Desktop uses
  `<config-home>/Claude/claude_desktop_config.json`, VS Code uses
  `<config-home>/Code/User/mcp.json`, and Cursor, Windsurf, and Codex retain
  their documented home-relative paths. Windows remains an explicit
  unsupported-platform error until `MCP-019`.
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
failure playbook, and guarded manual JSON and TOML restore procedures. The
documented JSON restore commands passed against disposable files; the guide
now adds a separately validated TOML procedure for Codex. They are guidance
for the absence of M1 restore UX, not a hidden `restore` implementation.

The README remains the product destination under `DEC-009`. Its corresponding
M1 surfaces were audited without adding a current-status banner or weakening
its finished-product story:

| README surface | Delivery audit |
| --- | --- |
| Canonical JSON v1 | Matches the implemented strict local-STDIO schema and deterministic serialization contract |
| `init`, complete-definition `add`, and redacted `list` | Match the delivered CLI, validation, import, no-clobber, and structural-output behavior |
| `sync --dry-run`, `sync`, backups, rollback, and no-ops | Match the delivered five-target macOS and Linux plan/apply transaction; Claude Desktop and Cursor additionally retain macOS current-client verification |
| `mcp-sync test` and STDIO health testing | Matches the explicit named-server five-second initialize boundary, structural protocol validation and redaction, and guaranteed direct-child cleanup; `init` and `sync` remain configuration-only |
| Windsurf | Delivered for the documented global legacy Cascade JSON contract with fixture and built-binary evidence; no Devin Local or current-client smoke claim |
| VS Code | Delivered for the native default user-profile `mcp.json` `servers` contract with fixture and built-binary evidence; project/profile/remote/Insiders/portable/extension-owned/Agent Host surfaces remain excluded, and there is no current-client smoke claim |
| Codex | Delivered for global `~/.codex/config.toml` with TOML-native fixture and built-binary evidence; project layers, remote/OAuth entries, credential stores, and unsupported fields remain outside ownership, and there is no current-client smoke claim |
| Linux | Implemented for all five global targets with deterministic path and built-binary behavior coverage plus a successful native x64/ARM64 whole-suite CI run; there is no Linux current-client smoke claim |
| Windows | North-star matrix only; sequenced through `MCP-019` |
| Explicit restore and backup retention | North-star UX only; sequenced to `MCP-020`, while the M1 guide records safe manual recovery and current one-slot behavior |
| GitHub Releases, Homebrew, WinGet, and Cargo | North-star distribution only; verification and publication remain gated by `MCP-021` |
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
| D-03 | Deterministic test and CI foundation | M0 | Codex | 2026-08-06 | Done | [Local quality gate](scripts/check.sh), [native x64/ARM64 Linux CI workflow](.github/workflows/ci.yml), [`tempfile`-owned synthetic-home CLI harness](tests/support/mod.rs), [macOS and Linux path fixtures](src/paths.rs), and [successful baseline CI run](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31137308671) |
| D-04 | Versioned canonical configuration contract | M0 | Codex | 2026-08-06 | Done | [Strict canonical model and tests](src/config.rs), [canonical v1 example](examples/config.v1.json), [public configuration contract](README.md), [accepted version policy](#canonical-configuration-v1-decision), and [dependency policy](deny.toml) |
| D-05 | Two-client import and conflict reporting | M1 | Codex | 2026-08-07 | Done | [Initialization use case](src/init.rs), [CLI command](src/main.rs), [global Claude Desktop adapter](src/claude_desktop.rs), [global Cursor adapter](src/cursor.rs), [create-only filesystem boundary](src/filesystem.rs), and [built-binary journeys](tests/init.rs) |
| D-06 | Redacted plan and safe multi-target apply | M1 | Codex | 2026-08-07 | Done | [Plan-once sync use case and focused transaction tests](src/sync.rs), [reversible exact-byte filesystem boundary and recovery tests](src/filesystem.rs), [seven isolated built-binary sync journeys](tests/sync.rs), [wired CLI](src/main.rs), and [public sync contract](README.md) |
| D-07 | Complete M1 CLI journey and user guide | M1 | Codex | 2026-08-07 | Done | The combined [golden built-binary journey and failure matrix](tests/golden.rs), command-specific journeys, audited [README contract](README.md), refreshed [controlled current-stable two-client smoke](#controlled-current-client-evidence), and tested [M1 usage and recovery guide](docs/m1-usage-and-recovery.md) complete the M1 journey |
| D-08 | Five-client, cross-platform support matrix | M2 | Codex | Unscheduled | In progress | All five global macOS and Linux targets pass deterministic fixture and built-binary import/sync coverage: four JSON adapters plus the TOML-native Codex adapter. Successful native x64/ARM64 whole-suite [push](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31240607296) and [pull-request](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31240608728) CI runs complete `MCP-018`; no Linux current-client smoke claim is made. Windsurf, native default-profile VS Code, and Codex have no current-client smoke claim on either implemented platform; Windows and the accepted six-target release matrix remain sequenced |
| D-09 | Bounded STDIO health testing | M2 | Codex | 2026-08-07 | Done | [Bounded initialize use case, replaceable process boundary, structural protocol parser, and 11 focused tests](src/health.rs), [five synthetic-home built-binary success and failure journeys](tests/health.rs), [Clap wiring](src/main.rs), and [source-checkout health contract](docs/m1-usage-and-recovery.md) |
| D-10 | Accessible release channels and recovery runbook | M2 | Unassigned | Unscheduled | Proposed | Six signed or platform-appropriate binaries, immutable GitHub Release, SHA-256 manifest, SPDX SBOMs, attestations, Homebrew, WinGet, Cargo, per-target install smoke tests, and restore exercise |

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
| MCP-018 | Add Linux path and behavior support | M2 | P1 | Codex | Done | `MCP-017` | The [platform-neutral resolver and eight path tests](src/paths.rs), exact Linux path fixtures in all five target adapters, distinct-XDG [built-binary harness](tests/support/mod.rs), request-sequenced [bounded-response regression](src/health.rs), and [native x64/ARM64 CI matrix](.github/workflows/ci.yml) prove the complete source-checkout configuration and bounded-health behavior on GNU/Linux. Both the [push](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31240607296) and [pull-request](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31240608728) event paths pass exact-host x64 and ARM64 jobs with all 228 tests, formatting, warning-free Clippy, and dependency policy. The focused regression also passed 100 consecutive local runs. Local macOS and native ARM64 gates, portable JSON/TOML recovery exercises, redaction, documentation, and diff checks pass. Windows, musl/Alpine, release packaging, and Linux current-client smoke remain explicitly unclaimed |
| MCP-019 | Add Windows path and replacement behavior support | M2 | P1 | Unassigned | Ready | `MCP-018` | Native Windows MSVC x64 and ARM64 CI, path, backup, replacement, rollback, and portable-package tests pass |
| MCP-020 | Add explicit restore and backup-retention UX | M2 | P1 | Unassigned | Proposed | `MCP-019` | Restore journey and retention rules documented and tested |
| MCP-021 | Publish accessible, provenance-linked release channels | M2 | P1 | Unassigned | Proposed | `MCP-020` | `DEC-024` passes in full: all six artifacts build and smoke-test natively, macOS and Windows trust checks pass, the immutable release contains checksums, per-target SPDX SBOMs, and attestations, and the exact same `v0.1.0` installs through GitHub Releases, Homebrew, WinGet, and Cargo under the accepted identifiers |

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
| MCP-020 | Complete MCP-020: add explicit restore and backup-retention user experience with documented retention rules and a tested recovery journey across supported platforms. Follow AGENTS.md and PROJECT.md, preserve unrelated worktree changes, and do not publish release channels early. Finish only when MCP-020's recovery evidence and the ticket-done gate pass, durable evidence is recorded, MCP-020 is Done, and only MCP-021 is Ready. |
| MCP-021 | Complete MCP-021: publish accessible, provenance-linked GitHub Release, Homebrew, WinGet, and Cargo channels that all install the same tagged mcp-sync version and pass supported-platform smoke journeys. Follow AGENTS.md and PROJECT.md, preserve unrelated worktree changes, and do not activate deferred expansion work. Finish only when MCP-021's release evidence, the M2 release gate, and the ticket-done gate pass, durable evidence is recorded, and MCP-021 is Done. |

### Expansion candidates

These are deliberately not executable tickets yet. When M2 nears completion,
select and sequence the next main-story arc; classify anything truly optional
as a `SIDE-NNN` ticket instead.

| ID | Candidate outcome | Possible arc | Classification trigger | Status |
| --- | --- | --- | --- | --- |
| MCP-022 | Add schema migration infrastructure after the first real schema change | M3 | A real schema change requires an upgrade path | Deferred |
| MCP-023 | Add external secret-reference support without becoming a secret store | M3 | Validated demand and an accepted threat model | Deferred |
| MCP-024 | Add named profiles and selective target policy | M3 | Validated multi-context workflow | Deferred |
| MCP-025 | Formalize a third-party adapter API | M3 | At least one real external adapter consumer | Deferred |
| MCP-026 | Explore managed team/fleet configuration | M4 | Approved product discovery and security boundary | Deferred |
| MCP-027 | Explore a GUI or desktop management surface | M4 | User research proves a distinct interface need | Deferred |

### Side-quest board

No side quests are active. The first optional ticket will be `SIDE-001` and
must satisfy the side-quest rules before it is marked `Ready`.

### Immediate focus

1. `MCP-018` is complete with deterministic five-client Linux path and
   built-binary coverage plus successful native GNU/Linux x64 and ARM64 CI on
   both push and pull-request event paths.
2. Only `MCP-019` is `Ready`. Assign an owner and reconcile its exact
   goal-catalog objective before moving it to `In progress`; keep `MCP-020` and
   every later ticket `Proposed`.
3. Preserve every configuration, process, protocol, redaction, backup, and
   rollback invariant when Windows work is intentionally started; do not begin
   it as part of `MCP-018` completion.

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
| DEC-014 | Use `enjoyable-mcp-sync` as the Cargo package and `mcp-sync` as its binary | Accepted | 2026-08-06 | The desired package name returned no exact crates.io record while the unrelated `mcp-sync` name is occupied; recheck availability immediately before MCP-021 publication |
| DEC-015 | Use Clap as the single CLI parser | Accepted | 2026-08-06 | Clap 4.6.6 provides maintained, cross-platform help, version, and future command parsing under MIT OR Apache-2.0 without introducing a disposable parser |
| DEC-016 | Introduce testing tools only with the first ticket that demonstrates their need | Accepted | 2026-08-06 | The [testing tool introduction plan](#testing-tool-introduction-plan) keeps `cargo test` as the baseline, makes each adopting ticket own configuration and evidence, and prevents speculative dependencies while preserving explicit review points for conditional tools |
| DEC-017 | Implement canonical JSON with Serde behind a strict duplicate-detecting and structurally redacted boundary | Accepted | 2026-08-06 | `serde` and `serde_json` provide maintained serialization primitives, while explicit shape and semantic validation prevent permissive map behavior, ordered maps produce deterministic bytes, and custom debug/errors prevent process values from leaking |
| DEC-018 | Resolve macOS configuration roots from an injected environment and keep the initial filesystem port read-only | Accepted | 2026-08-06 | Required absolute, traversal-free `HOME` and optional `XDG_CONFIG_HOME` inputs make path behavior deterministic without touching real user configuration; deferring mutation methods prevents atomicity, backup, and rollback contracts from being guessed before their owning tickets |
| DEC-019 | Manage only `command`, `args`, and `env` inside each Claude Desktop `mcpServers` entry | Accepted | 2026-08-06 | Matches canonical JSON v1 and the current local STDIO contract while preserving unknown top-level data, target-only servers, and native fields such as `cwd`; the adapter consumes an existing plan and never interprets remote account connectors or desktop extensions as this local file format |
| DEC-020 | Manage only command-based local entries in global Cursor configuration | Accepted | 2026-08-07 | Cursor stores global and project configuration separately and places local STDIO plus remote HTTP/SSE entries in `mcpServers`. M1 resolves only `~/.cursor/mcp.json`, owns only `command`, `args`, and `env` within compatible local entries, preserves `envFile` and all remote or unknown data, and refuses local additions that collide with unmanaged remote names; project-level `.cursor/mcp.json` remains outside the adapter boundary |
| DEC-021 | Make initialization a deterministic, create-only import transaction | Accepted | 2026-08-07 | `init` checks that canonical state is absent, completes every implemented client read and the entire pure merge before mutation, reports structural disagreements or unsupported-name collisions, and publishes validated bytes through a synced same-directory temporary file without replacing an existing path. Native files remain read-only; guarded canonical replacement is implemented by `MCP-010`, while target transaction rollback is implemented by `MCP-011` |
| DEC-022 | Treat canonical `add` as a complete-definition upsert and `list` as structural metadata only | Accepted | 2026-08-07 | Repeated literal `--arg` and `--env KEY=VALUE` inputs avoid shell parsing; an existing name is replaced as one validated unit, exact semantic equality skips all writes, and changed state receives an exact `.bak` before guarded atomic replacement. Normal output may show escaped names, argument counts, and escaped environment key names, but never commands, arguments, or environment values |
| DEC-023 | Preserve target-only entries in M1 and require explicit ownership provenance before any future prune | Accepted | 2026-08-07 | M1 sync may add desired names and update only `command`, `args`, and `env` inside compatible local entries while preserving every unowned field. Canonical absence produces non-mutating drift, never deletion; unmanaged remote-name collisions remain errors. A future prune requires a separately accepted command, provenance model, recovery contract, and ticket rather than inferring ownership from absence |
| DEC-024 | Adopt the six-target signed, attested, and immutable release contract | Accepted | 2026-08-07 | [The release contract](#release-artifact-and-trust-contract) fixes separate ARM64 and x64 artifacts for macOS, GNU/Linux, and Windows; mandatory Apple signing/notarization and Windows Public Trust signing; SHA-256, per-target SPDX SBOMs, build attestations, and immutable GitHub Releases; stable public identifiers; and `v0.1.0` as the first release. Missing signing authority blocks stable publication rather than producing unsigned advertised artifacts |
| DEC-025 | Satisfy the controlled two-client M1 criterion inside `MCP-012` | Accepted | 2026-08-07 | The repository owner's exact active objective requires every M1 acceptance criterion before `MCP-012` is `Done`, so the current-client gate cannot remain exclusively in its successor. `MCP-012` owns the first passing backup-protected Claude Desktop and Cursor verification; `MCP-013` remains the immediate successor for the north-star README audit, refreshed client verification as needed, and detailed usage and recovery guidance |
| DEC-026 | Manage only Windsurf's documented global legacy Cascade configuration | Accepted | 2026-08-07 | The official contract places command-based STDIO and commandless `serverUrl`/`url` transports together in `~/.codeium/windsurf/mcp_config.json`, while the vendor now distinguishes the Devin Local agent's separate configuration. The adapter owns only `command`, `args`, and `env` for compatible local entries; it preserves headers, authentication, `disabledTools`, interpolation literals, unknown data, unmanaged remote entries, and target-only servers, refuses a local collision, and makes no Devin Local or current-client smoke claim |
| DEC-027 | Manage only VS Code's native default user-profile MCP configuration | Accepted | 2026-08-07 | Official VS Code documentation places native user-profile MCP configuration in `mcp.json` under root `servers`. On macOS the adapter resolves exactly `~/Library/Application Support/Code/User/mcp.json`, accepts explicit or omitted local STDIO `type`, owns only `command`, `args`, and string-valued `env`, emits explicit `"type": "stdio"` for additions, preserves root `inputs`/`sandbox`, interpolation and unknown fields, and treats remote/unknown transports plus number/null environment entries as unmanaged collision-protected data. Workspace, named/remote profiles, Insiders, portable installations, Cline/Roo stores, and Agent Host/Copilot CLI configuration remain distinct and excluded; fixture and built-binary evidence is not a current-client smoke claim |
| DEC-028 | Manage only canonical-compatible local STDIO entries in Codex's global user configuration | Accepted | 2026-08-07 | [Codex configuration precedence](https://developers.openai.com/codex/config-basic/#configuration-precedence) makes trusted project `.codex/config.toml` files higher-precedence layers, while the [Codex MCP contract](https://developers.openai.com/codex/mcp/) places local STDIO and remote HTTP/OAuth servers in the same `mcp_servers` map shared by the ChatGPT desktop app, Codex CLI, and IDE extension. `MCP-016` therefore resolves only global `~/.codex/config.toml`, owns only `command`, `args`, and `env` in unambiguous local STDIO entries, and uses a TOML-native structural edit that preserves comments, unrelated settings, and all unowned local fields. Project layers remain undiscovered and untouched; `url`-based remote entries, authentication, headers, OAuth settings, unknown or mixed transports, and credential stores remain unmanaged and structurally preserved; a canonical local definition that collides with any unmanaged name fails before mutation. A successful global sync does not claim to override a higher-precedence project definition, perform OAuth login, or prove current-client behavior. |
| DEC-029 | Implement one explicit final-session MCP initialize compatibility boundary | Accepted | 2026-08-07 | `mcp-sync test <name>` is the only command that executes a configured server. It sends the final session-based `2025-11-25` initialize shape, accepts the four final handshake revisions through `2024-11-05`, bounds one newline-delimited response to five seconds and 1 MiB, validates JSON-RPC and required result structure before sending `notifications/initialized`, and allows 500 milliseconds for stdin-driven shutdown before force termination and reaping. The child receives canonical environment entries plus inherited `PATH` only when canonical state omits it; stderr and raw response/error data remain undisclosed. The current stateless `2026-07-28` protocol is a materially different boundary and is not claimed by this initialize-specific ticket. |
| DEC-030 | Resolve Linux user data through XDG and require native x64/ARM64 whole-suite CI | Accepted | 2026-08-07 | Linux uses a non-empty absolute `XDG_CONFIG_HOME`, falling back to `$HOME/.config`, for canonical configuration plus Claude Desktop and VS Code user data; Cursor, Windsurf, and Codex keep their documented home-relative paths. The platform-neutral resolver and existing adapters preserve all ownership and mutation rules. [GitHub's standard hosted-runner contract](https://docs.github.com/en/actions/how-tos/write-workflows/choose-where-workflows-run/choose-the-runner-for-a-job) supplies native `ubuntu-24.04` x64 and `ubuntu-24.04-arm` ARM64 runners, and each must assert its Rust host and pass the complete synthetic-home quality script. This proves GNU libc source-checkout behavior only: it does not add musl/Alpine, Windows, packaging, or a Linux current-client smoke claim. |

### Open decisions

`OPEN-07` is resolved by `DEC-028`. No open decision currently blocks
`MCP-019`.

## Risk register

| ID | Risk | Impact | Likelihood | Current mitigation | Trigger for escalation | Status |
| --- | --- | --- | --- | --- | --- | --- |
| RISK-01 | A merge or partial failure loses user configuration | Critical | Medium | Pure plan/apply separation, guarded exact-byte writes, recoverable backups, reversible receipts, reverse-order five-target rollback, non-mutating drift, bounded client ownership, unmanaged-name collision refusal, interrupted-write compensation, and the complete synthetic failure matrix are implemented. A forced Codex fifth-target failure restores Claude Desktop, Cursor, Windsurf, and VS Code plus their pre-existing backups exactly. Current Cursor verification is isolated; current Claude verification uses a no-clobber backup and verified exact restore | Any unrecoverable fixture mutation or ambiguous ownership case | Mitigated — five-target synthetic proof; monitor later targets |
| RISK-02 | Secrets leak through plans, errors, logs, fixtures, or snapshots | High | Medium | Import conflicts, canonical add/list reports, sync plans, per-target transaction reports, rollback errors, four JSON adapters, the TOML-native Codex document/render debug surfaces, and health-process failures expose structure only. Health journeys prove commands, arguments, environment values, raw stdout/stderr, malformed output, JSON-RPC error messages/data, and unchecked protocol versions remain undisclosed; the broader built-binary sentinels and safe byte/structural assertions cover the complete synthetic matrix without printing fixture contents. Current-client smokes read only synthetic output and structural status; no real configuration, credential, or account value was displayed | Any test or output path observes a secret value | Mitigated — configuration and health synthetic proof |
| RISK-03 | Native client schemas or paths drift | High | Medium | All five implemented global contracts and macOS/Linux paths are fixture-backed. Current Cursor `3.15.6` and current Claude Desktop `1.26832.0` accept and initialize rendered global entries on macOS. Linux has deterministic built-binary coverage and a successful native x64/ARM64 whole-suite CI run; there is no Linux current-client smoke claim. Windsurf's legacy Cascade, VS Code's native default-profile, and Codex's global TOML contracts retain explicit exclusions and have no current-client smoke claim on either platform | A client update invalidates fixture/discovery behavior or changes one of the documented global paths or schemas | Mitigated — monitor client updates and all five documented contracts |
| RISK-04 | Cross-platform file replacement behaves differently | High | Medium | No-clobber creation, guarded atomic replacement, reversible target receipts, and reverse-order rollback are isolated behind filesystem ports with disposable backup, stale-byte, interrupted-write, symlink, non-regular, permission, and cleanup tests. The complete suite passes on macOS and on native GNU/Linux x64 and ARM64 CI; Windows replacement and rollback remain gated by `MCP-019` | Platform work requires weakening atomicity or rollback | Open — Windows evidence remains |
| RISK-05 | Health checks hang or leave child processes running | High | Medium | The only execution boundary enforces a five-second response limit, 1-MiB message limit, 500-millisecond clean-shutdown allowance, force termination, reap, and a drop backstop. Focused and built-binary tests verify direct child PIDs are gone after response timeout, malformed output, and refused shutdown | A health test exceeds its bound or leaves the tested child running | Mitigated — bounded process and failure proof; monitor later platforms |
| RISK-06 | Broad client/platform scope delays the first usable proof | High | High | The two-client M1 proof is complete; strict story sequence, WIP limit, and milestone gates constrain M2 to one adapter or capability slice at a time | Any M2 ticket starts before its immediate predecessor is done | Mitigated — enforce M2 sequence |
| RISK-07 | Premature plug-in or workspace abstractions slow iteration | Medium | Medium | One crate; abstractions require real variation or consumer | New public trait/package has only one hypothetical implementation | Mitigated by guidance |
| RISK-08 | The tracker becomes stale and stops reflecting delivery truth | Medium | Medium | Same-change updates, evidence links, weekly/completion review triggers, and the completed M1 README/current-state audit | PROJECT contradicts code/release evidence, or README changes the north-star without a decision | Mitigated — monitor each ticket |
| RISK-09 | A side quest consumes main-story capacity or becomes a hidden prerequisite | Medium | Medium | Separate IDs, strict classification, one-side-quest WIP cap, required promotion | A main ticket or release gate depends on `SIDE-NNN`, or later story work starts early | Mitigated by plan |
| RISK-10 | The product or package is confused with unrelated projects using the `mcp-sync` name | High | High | Verified distinct Cargo package name, publisher-qualified install commands, provenance links, and a required pre-publication registry recheck | The selected name becomes unavailable or users cannot distinguish the publisher | Open — recheck at `MCP-021` |
| RISK-11 | Codex's shared TOML file loses non-MCP settings or target-only MCP capabilities | Critical | Medium | `DEC-028` fixes a TOML-native, global-only local-STDIO ownership boundary. Exact fixtures and built-binary journeys preserve comments, formatting, unrelated settings, project/alternate-file/credential isolation, unowned local fields, remote HTTP/OAuth/mixed/opaque entries, target-only data, and unmanaged-name collision refusal without a JSON conversion | Any Codex fixture requires lossy conversion, reads OAuth credentials, mutates a project layer, or drops an unowned field | Mitigated — fixture and five-target built-binary proof; monitor Codex contract drift |
| RISK-12 | Required platform signing authority is unavailable or identity validation delays release | High | Medium | `DEC-024` makes Apple Developer ID plus notarization and Windows Public Trust signing explicit `MCP-021` prerequisites; credentials stay in protected CI environments, and a CA-issued Authenticode certificate is the Windows fallback | `MCP-021` starts without active Apple notarization credentials or validated Windows signing authority | Open — release blocker |

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

### Milestone release

A milestone is complete only when every exit gate is evidenced, all P0 tickets
in its boundary are `Done` or explicitly removed through an accepted decision,
critical risks are closed or accepted with a concrete consequence, and the
preceding working behavior still passes. Main-story tickets must have completed
in order. Side quests never gate a milestone release.

## Tracking templates

Use the following compact shape when adding work:

```text
Ticket: MCP-NNN | SIDE-NNN — observable outcome
Track: Main story | Side quest
Milestone / priority:
Owner / target:
Status:
Immediate predecessor (main story only):
Canonical goal objective (main story only):
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
