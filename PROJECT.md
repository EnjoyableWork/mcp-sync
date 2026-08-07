# mcp-sync project plan

This is the canonical living document for product scope, delivery status,
tickets, deliverables, decisions, risks, and release gates. It is intentionally
lightweight enough to use before a hosted tracker is justified.

| Control | Current state |
| --- | --- |
| Document state | Active |
| Product state | Pre-MVP; runnable Rust CLI walking skeleton implemented |
| Current milestone | M0 — Walking skeleton |
| Overall status | `MCP-002` complete; `MCP-003` ready; no later implementation has begun |
| Current focus | Add deterministic quality gates and synthetic-home CI for `MCP-003` |
| Milestone target | Unscheduled; set after an owner and delivery capacity are known |
| Last reviewed | 2026-08-06 |
| Next review trigger | Completion of `MCP-003`, or any change to the MVP boundary |

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
| M0 | Survey and foundations | A runnable Rust skeleton, versioned config contract, quality gates, and fixture harness | Clean checkout can build, lint, test, and show CLI help; the first schema and test environment are documented | In progress |
| M1 | Town — MVP | One safe vertical journey across Claude Desktop and Cursor on macOS | Every MVP acceptance criterion below passes against a synthetic home directory | Proposed |
| M2 | City — dependable daily tool | README client coverage, health checks, cross-platform behavior, restore UX, and accessible release channels | Supported matrix passes in CI and one tagged release installs and smoke-tests through every advertised channel | Proposed |
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
| GitHub Releases | All platforms, including users without a package manager | Download a prebuilt macOS, Linux, or Windows archive | Canonical versioned binaries, checksums, release notes, and provenance; GitHub supports attaching binary assets to releases. |
| Homebrew tap | macOS and Linux users | `brew install EnjoyableWork/tap/mcp-sync` | Primary one-command Unix installation. An organization-owned tap keeps the formula under project control. |
| Windows Package Manager | Windows users | `winget install EnjoyableWork.mcp-sync` | Native discovery, installation, and upgrades through the public WinGet repository. |
| crates.io / Cargo | Rust developers with a toolchain | `cargo install enjoyable-mcp-sync` | Builds from the published source package and installs a binary still named `mcp-sync`. |

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
| Initial platform | macOS — working assumption |
| Initial clients | Claude Desktop and Cursor — working assumption; two clients are the minimum proof of sync |
| Canonical format | Versioned JSON at the documented `mcp-sync` config location |
| Commands | `init`, `add`, `list`, `sync --dry-run`, and `sync` |
| Import | Discover the two initial clients, normalize compatible server entries, and stop with an actionable conflict report when values disagree |
| Sync ownership | Add or update canonical server entries while preserving unrelated native configuration |
| Deletion | Do not silently delete target-only entries; report drift. Explicit prune behavior is post-MVP until its ownership contract is accepted |
| Mutation safety | Validate, show a redacted plan, back up existing files, replace each file atomically, and recover prior writes after a later target failure |
| Secrets | Preserve required values but redact them from terminal output, errors, fixtures, and logs |
| Network and process behavior | `init` and `sync` are local file operations and never start an MCP server; STDIO health testing is M2 |
| Distribution | Build and run from a source checkout for M1; packaged binaries are M2 |

### Golden MVP journey

Given a synthetic macOS home directory with realistic Claude Desktop and Cursor
configuration, including unrelated keys and at least one environment value:

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
- Delivered M1 behavior matches the corresponding README contract, and this
  tracker clearly records which north-star capabilities remain beyond M1.

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

## Deliverables

| ID | Deliverable | Milestone | Owner | Target | Status | Completion evidence |
| --- | --- | --- | --- | --- | --- | --- |
| D-01 | Project operating model and north-star product page | M0 | Repository | 2026-08-06 | Done | [AGENTS.md](AGENTS.md), this tracker, and [README.md](README.md) |
| D-02 | Runnable Rust CLI skeleton | M0 | Codex | 2026-08-06 | Done | [Cargo.toml](Cargo.toml), [src/main.rs](src/main.rs), [CLI smoke tests](tests/cli.rs), [Cargo.lock](Cargo.lock), and locked build/install verification |
| D-03 | Deterministic test and CI foundation | M0 | Unassigned | Unscheduled | Ready | Format, lint, unit, integration, and synthetic-home checks in CI |
| D-04 | Versioned canonical configuration contract | M0 | Unassigned | Unscheduled | Proposed | Schema types, examples, validation tests, and version policy |
| D-05 | Two-client import and conflict reporting | M1 | Unassigned | Unscheduled | Proposed | Claude/Cursor fixtures and import journey evidence |
| D-06 | Redacted plan and safe multi-target apply | M1 | Unassigned | Unscheduled | Proposed | Dry-run, no-op, backup, atomic write, and rollback tests |
| D-07 | Complete M1 CLI journey and user guide | M1 | Unassigned | Unscheduled | Proposed | Golden journey and README verification |
| D-08 | Five-client, cross-platform support matrix | M2 | Unassigned | Unscheduled | Proposed | Platform/client CI matrix with native JSON and TOML fixtures |
| D-09 | Bounded STDIO health testing | M2 | Unassigned | Unscheduled | Proposed | Protocol, timeout, cleanup, and redaction tests |
| D-10 | Accessible release channels and recovery runbook | M2 | Unassigned | Unscheduled | Proposed | GitHub binaries, Homebrew, WinGet, Cargo, checksums, install smoke tests, and restore exercise |

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
| MCP-003 | Add format, Clippy, test, and CI quality gates using a synthetic home | M0 | P0 | Unassigned | Ready | `MCP-002` | Documented local checks pass in CI without touching real config |
| MCP-004 | Define the versioned canonical server model and JSON validation contract | M0 | P0 | Unassigned | Proposed | `MCP-003` | Types, example config, deterministic round-trip tests, unknown-version error |
| MCP-005 | Introduce injectable config-path and filesystem boundaries | M0 | P0 | Unassigned | Proposed | `MCP-004` | macOS path fixtures, synthetic-home enforcement, contextual I/O errors |
| MCP-006 | Build the pure normalized reconciliation and redacted plan engine | M1 | P0 | Unassigned | Proposed | `MCP-005` | Deterministic add/update/no-op/drift plans and redaction tests |
| MCP-007 | Implement the Claude Desktop macOS adapter | M1 | P0 | Unassigned | Proposed | `MCP-006` | Native fixture round trips while unrelated keys survive |
| MCP-008 | Implement the Cursor macOS adapter | M1 | P0 | Unassigned | Proposed | `MCP-007` | Native fixture round trips while unrelated keys survive |
| MCP-009 | Implement `init` discovery, import, normalization, and conflict reporting | M1 | P0 | Unassigned | Proposed | `MCP-008` | Deterministic two-client import and no-mutation conflict journey |
| MCP-010 | Implement `add` and redacted `list` against the canonical config | M1 | P0 | Unassigned | Proposed | `MCP-009` | Upsert/list CLI tests, atomic canonical write, no secret output |
| MCP-011 | Implement `sync --dry-run` and safe apply with backup and transaction recovery | M1 | P0 | Unassigned | Proposed | `MCP-010` | Plan/apply parity, no-op, atomic write, failure rollback tests |
| MCP-012 | Prove the golden MVP journey and failure matrix | M1 | P0 | Unassigned | Proposed | `MCP-011` | All M1 acceptance criteria pass through the built binary |
| MCP-013 | Verify M1 against the north-star README and publish detailed usage and recovery guidance | M1 | P0 | Unassigned | Proposed | `MCP-012` | Delivered commands match their README contract and the guide records current operational limitations |
| MCP-014 | Add the Windsurf target adapter | M2 | P1 | Unassigned | Proposed | `MCP-013` | Fixture, merge-boundary, discovery, and journey coverage |
| MCP-015 | Add the VS Code target adapter and define extension-shape boundaries | M2 | P1 | Unassigned | Proposed | `MCP-014` | Supported extension contract plus fixtures and journey coverage |
| MCP-016 | Add the Codex adapter for the shared ChatGPT desktop, Codex CLI, and IDE host configuration | M2 | P1 | Unassigned | Proposed | `MCP-015` | The global TOML fixture round-trips, unrelated Codex settings and unsupported MCP fields survive, and the shared server map has discovery and journey coverage |
| MCP-017 | Add bounded MCP STDIO initialize health testing | M2 | P1 | Unassigned | Proposed | `MCP-016` | Handshake, timeout, child cleanup, malformed output, redaction tests |
| MCP-018 | Add Linux path and behavior support | M2 | P1 | Unassigned | Proposed | `MCP-017` | Linux CI and supported-client path fixtures pass |
| MCP-019 | Add Windows path and replacement behavior support | M2 | P1 | Unassigned | Proposed | `MCP-018` | Windows CI, path, backup, replacement, and rollback tests pass |
| MCP-020 | Add explicit restore and backup-retention UX | M2 | P1 | Unassigned | Proposed | `MCP-019` | Restore journey and retention rules documented and tested |
| MCP-021 | Publish accessible, provenance-linked release channels | M2 | P1 | Unassigned | Proposed | `MCP-020` | The same tagged version installs from GitHub Releases, Homebrew, WinGet, and Cargo and completes a smoke journey on each supported platform |

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
| MCP-007 | Complete MCP-007: implement the Claude Desktop macOS adapter for native discovery, parsing, rendering, and bounded merge behavior while preserving unrelated configuration. Follow AGENTS.md and PROJECT.md, preserve unrelated worktree changes, and do not begin another client adapter. Finish only when MCP-007's fixture and evidence criteria plus the ticket-done gate pass, durable evidence is recorded, MCP-007 is Done, and only MCP-008 is Ready. |
| MCP-008 | Complete MCP-008: implement the Cursor macOS adapter for native discovery, parsing, rendering, and bounded merge behavior while preserving unrelated configuration. Follow AGENTS.md and PROJECT.md, preserve unrelated worktree changes, and do not begin import orchestration or another client adapter. Finish only when MCP-008's fixture and evidence criteria plus the ticket-done gate pass, durable evidence is recorded, MCP-008 is Done, and only MCP-009 is Ready. |
| MCP-009 | Complete MCP-009: implement deterministic init discovery, import, normalization, and actionable conflict reporting across the two M1 clients without mutating anything on conflict. Follow AGENTS.md and PROJECT.md, preserve unrelated worktree changes, and do not begin add, list, or sync behavior. Finish only when MCP-009's journey evidence and the ticket-done gate pass, durable evidence is recorded, MCP-009 is Done, and only MCP-010 is Ready. |
| MCP-010 | Complete MCP-010: implement canonical-config add and structurally redacted list behavior with deterministic upsert semantics, atomic canonical writes, and no secret output. Follow AGENTS.md and PROJECT.md, preserve unrelated worktree changes, and do not begin target sync behavior. Finish only when MCP-010's CLI evidence and the ticket-done gate pass, durable evidence is recorded, MCP-010 is Done, and only MCP-011 is Ready. |
| MCP-011 | Complete MCP-011: implement sync dry-run and safe multi-target apply using the same validated plan, recoverable backups, no-op detection, atomic replacement, per-target reporting, and transaction rollback. Follow AGENTS.md and PROJECT.md, preserve unrelated worktree changes, and do not weaken safety invariants or begin later client work. Finish only when MCP-011's success and forced-failure evidence plus the ticket-done gate pass, durable evidence is recorded, MCP-011 is Done, and only MCP-012 is Ready. |
| MCP-012 | Complete MCP-012: prove the golden M1 journey and its failure matrix through the built binary against a synthetic home, including redaction, idempotence, rollback, and non-zero error behavior. Follow AGENTS.md and PROJECT.md, preserve unrelated worktree changes, and fix any M1 defect revealed without adding M2 scope. Finish only when every M1 acceptance criterion and the ticket-done gate pass with durable evidence, MCP-012 is Done, and only MCP-013 is Ready. |
| MCP-013 | Complete MCP-013: verify delivered M1 behavior against the north-star README and publish accurate usage and recovery guidance while keeping current limitations in delivery documentation rather than diluting the README's product role. Follow AGENTS.md and PROJECT.md, preserve unrelated worktree changes, and do not claim or begin M2 behavior. Finish only when MCP-013's documentation evidence and the ticket-done gate pass, durable evidence is recorded, MCP-013 is Done, and only MCP-014 is Ready. |
| MCP-014 | Complete MCP-014: add the Windsurf target adapter with native discovery, parse and render translation, a documented merge boundary, and fixture and journey coverage that preserves unrelated data. Follow AGENTS.md and PROJECT.md, preserve unrelated worktree changes, and do not begin VS Code or other later targets. Finish only when MCP-014's evidence criteria and the ticket-done gate pass, durable evidence is recorded, MCP-014 is Done, and only MCP-015 is Ready. |
| MCP-015 | Complete MCP-015: add the VS Code target adapter and define exactly which extension configuration shape mcp-sync supports, with native fixtures, preservation boundaries, and journey coverage. Follow AGENTS.md and PROJECT.md, preserve unrelated worktree changes, and do not begin Codex or health testing. Finish only when MCP-015's evidence criteria and the ticket-done gate pass, durable evidence is recorded, MCP-015 is Done, and only MCP-016 is Ready. |
| MCP-016 | Complete MCP-016: add the Codex TOML adapter for the shared ChatGPT desktop app, Codex CLI, and IDE host configuration using the accepted OPEN-07 scope, structurally preserving unrelated settings and unsupported MCP fields. Follow AGENTS.md and PROJECT.md, preserve unrelated worktree changes, and never use a lossy JSON conversion or begin health testing. Finish only when MCP-016's fixtures, discovery and journey evidence, and ticket-done gate pass, durable evidence is recorded, MCP-016 is Done, and only MCP-017 is Ready. |
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

1. Assign and complete `MCP-003` without adding schema or client logic.
2. After `MCP-003` is `Done`, update its evidence and move only `MCP-004` to
   `Ready`.
3. Continue one row at a time; do not begin `MCP-005` or any later ticket early.
4. Re-review the M1 working assumptions before `MCP-007` begins.

## Decision log

| ID | Decision | Status | Date | Reason and consequence |
| --- | --- | --- | --- | --- |
| DEC-001 | Implement the product in Rust | Accepted | 2026-08-06 | User-selected direction; optimize for a portable native CLI, explicit types, and predictable local execution |
| DEC-002 | Begin as one modular binary crate | Accepted | 2026-08-06 | Fastest coherent delivery shape; split only for a demonstrated consumer or boundary |
| DEC-003 | Use this Markdown document as the initial project tracker | Accepted | 2026-08-06 | Keeps scope and evidence beside the code; migrate issue detail later without duplicating milestone truth |
| DEC-004 | Use versioned JSON as the canonical M1 format | Working assumption | 2026-08-06 | Matches the stated product contract and native clients; confirm exact schema during `MCP-004` |
| DEC-005 | Prove M1 on macOS with Claude Desktop and Cursor | Working assumption | 2026-08-06 | Two clients demonstrate the core value with the smallest platform matrix; adapters keep expansion isolated |
| DEC-006 | Preserve target-only entries and report drift rather than silently pruning in M1 | Working assumption | 2026-08-06 | Prevents accidental loss before managed ownership and explicit deletion semantics are designed |
| DEC-007 | Keep health testing outside M1 | Working assumption | 2026-08-06 | Safe reconciliation is the core proof; process lifecycle and protocol handling become the next vertical feature |
| DEC-008 | Complete the main story in strict order and isolate optional work as side quests | Accepted | 2026-08-06 | Prevents foundation gaps and hidden prerequisites; a blocked story ticket blocks later story work until resolved or formally replaced |
| DEC-009 | Preserve README as the north-star product specification and public marketing page | Accepted | 2026-08-06 | README describes the intended finished experience; PROJECT, code, tests, and releases track current delivery truth |
| DEC-010 | Use Rust-native and OS-native distribution rather than npm | Accepted | 2026-08-06 | Canonical GitHub binaries plus Homebrew, WinGet, and a distinctly named Cargo package make the executable accessible with or without a Rust toolchain |
| DEC-011 | Classify Codex as a supported client target with its own adapter | Accepted | 2026-08-06 | [Codex MCP configuration](https://developers.openai.com/codex/mcp/) is shared by the ChatGPT desktop app, Codex CLI, and IDE extension; TOML is real target variation that belongs behind the existing adapter boundary |
| DEC-012 | Use one canonical Goal-mode objective per active main-story ticket | Accepted | 2026-08-06 | Thread goals provide persistence for long work, while exact per-ticket objectives preserve the ordered scope and evidence gates without replacing repository truth; only one unfinished ticket goal may be active in a thread |
| DEC-013 | Track the current stable Rust toolchain and defer a minimum supported version until CI evidence exists | Accepted | 2026-08-06 | `rust-toolchain.toml` selects stable and MCP-002 passed on Rust 1.94.1; MCP-003 will establish CI policy before a `rust-version` is claimed |
| DEC-014 | Use `enjoyable-mcp-sync` as the Cargo package and `mcp-sync` as its binary | Accepted | 2026-08-06 | The desired package name returned no exact crates.io record while the unrelated `mcp-sync` name is occupied; recheck availability immediately before MCP-021 publication |
| DEC-015 | Use Clap as the single CLI parser | Accepted | 2026-08-06 | Clap 4.6.6 provides maintained, cross-platform help, version, and future command parsing under MIT OR Apache-2.0 without introducing a disposable parser |

### Open decisions

| ID | Decision needed | Needed by | Default if still open |
| --- | --- | --- | --- |
| OPEN-01 | Confirm or change the initial platform and two client targets | Before `MCP-007` and `MCP-008` | Continue with macOS, Claude Desktop, and Cursor |
| OPEN-02 | Define the exact canonical schema, server identity, argument representation, and environment-value contract | During `MCP-004` | Follow current README examples, add `schemaVersion`, and reject ambiguous shapes |
| OPEN-03 | Define adapter ownership at field level and the future explicit prune model | Before `MCP-011` | Upsert named canonical entries, preserve unknown and target-only data, report drift |
| OPEN-06 | Finalize release CPU/OS targets, signing, notarization, checksums, and package identifiers | Before `MCP-021` | GitHub Releases are canonical; Homebrew, WinGet, and Cargo consume the same tagged version |
| OPEN-07 | Define Codex coverage for project-scoped configuration and [remote/OAuth MCP options](https://developers.openai.com/codex/config-reference) | Before `MCP-016` | Manage local STDIO entries in global `~/.codex/config.toml`; preserve project-scoped configuration and unsupported target-only fields unchanged |

## Risk register

| ID | Risk | Impact | Likelihood | Current mitigation | Trigger for escalation | Status |
| --- | --- | --- | --- | --- | --- | --- |
| RISK-01 | A merge or partial failure loses user configuration | Critical | Medium | Plan/apply separation, field ownership, backups, atomic replacement, rollback, failure tests | Any unrecoverable fixture mutation or ambiguous ownership case | Open — M1 gate |
| RISK-02 | Secrets leak through plans, errors, logs, fixtures, or snapshots | High | Medium | Structural redaction and synthetic tests; no raw config output | Any test or output path observes a secret value | Open — M1 gate |
| RISK-03 | Native client schemas or paths drift | High | Medium | Per-client adapters and versioned fixtures; support claims require evidence | Client update invalidates fixture or discovery behavior | Open |
| RISK-04 | Cross-platform file replacement behaves differently | High | Medium | macOS first, injectable filesystem, later Linux/Windows CI | Platform work requires weakening atomicity or rollback | Open |
| RISK-05 | Health checks hang or leave child processes running | High | Medium | Deferred to its own bounded slice with timeout and cleanup contract | `MCP-017` begins | Deferred with feature |
| RISK-06 | Broad client/platform scope delays the first usable proof | High | High | Two-client M1 boundary, strict story sequence, WIP limit, milestone gates | `MCP-014` starts before `MCP-013` is done | Mitigated by plan |
| RISK-07 | Premature plug-in or workspace abstractions slow iteration | Medium | Medium | One crate; abstractions require real variation or consumer | New public trait/package has only one hypothetical implementation | Mitigated by guidance |
| RISK-08 | The tracker becomes stale and stops reflecting delivery truth | Medium | Medium | Same-change updates, evidence links, weekly/completion review triggers | PROJECT contradicts code/release evidence, or README changes the north-star without a decision | Open |
| RISK-09 | A side quest consumes main-story capacity or becomes a hidden prerequisite | Medium | Medium | Separate IDs, strict classification, one-side-quest WIP cap, required promotion | A main ticket or release gate depends on `SIDE-NNN`, or later story work starts early | Mitigated by plan |
| RISK-10 | The product or package is confused with unrelated projects using the `mcp-sync` name | High | High | Verified distinct Cargo package name, publisher-qualified install commands, provenance links, and a required pre-publication registry recheck | The selected name becomes unavailable or users cannot distinguish the publisher | Open — recheck at `MCP-021` |
| RISK-11 | Codex's shared TOML file loses non-MCP settings or target-only MCP capabilities | Critical | Medium | Structural TOML merge, a narrow ownership boundary, and fixtures covering unrelated settings plus unsupported HTTP/auth fields | Any Codex fixture requires lossy JSON conversion or drops an unowned field | Open — M2 gate |

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
