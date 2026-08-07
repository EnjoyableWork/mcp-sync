# mcp-sync repository guidance

## Product and source of truth

`mcp-sync` is a local configuration CLI for defining MCP servers once and
reconciling that desired state into the native configuration files used by MCP
clients.

The repository is pre-MVP. The README is a deliberate target-state exception:
it presents the intended finished product in confident public product language
and should not be rewritten as a progress report. Everywhere else, do not
describe a planned command, client adapter, platform, safety property, or
distribution channel as currently implemented until verification exists.

- [README.md](README.md) is the north-star product specification and public
  marketing page. It describes the destination, not current delivery status.
- [PROJECT.md](PROJECT.md) is the canonical delivery plan, ticket board,
  decision log, risk register, and current implementation record.
- Code, tests, and release artifacts are the authority for implemented
  behavior. When they disagree with prose, correct the prose as part of the
  same change.

Write the product name as lowercase `mcp-sync` except where capitalization is
required by a platform or sentence structure. Format paths, commands,
configuration keys, filenames, and protocol values as code.

## Priorities

In descending order:

1. Preserve user configuration and secrets.
2. Produce correct, deterministic plans and recoverable writes.
3. Keep the north-star product story coherent and current delivery records
   evidence-based.
4. Deliver the smallest working vertical slice quickly.
5. Preserve clear extension points for demonstrated variation.

Speed does not justify silently dropping configuration, exposing environment
values, weakening validation, or claiming unverified support in current-state
records, release notes, or runtime output.

## Delivery strategy

Rust is the selected implementation language. Follow the current milestone and
focus order in [PROJECT.md](PROJECT.md); do not implement the entire roadmap as
one change.

The initial shape is a modular monolith: one installable binary crate with
cohesive internal modules. A Cargo workspace, separately published library,
dynamic plug-in system, daemon, GUI, or remote service requires a current
consumer and an accepted decision in `PROJECT.md`.

The implemented foundation is one Rust 2024 binary crate with a package named
`enjoyable-mcp-sync` and an installed binary named `mcp-sync`. The CLI now
supports help, version, create-only `init`, complete-definition canonical
`add`, structurally redacted `list`, and two-target `sync --dry-run` / `sync`
journeys. It also contains the strict canonical JSON v1 model, an injected
macOS configuration-path resolver, a replaceable filesystem boundary with
read, no-clobber creation, guarded replacement, and reversible transaction
ports, and a pure deterministic reconciliation engine with structurally
redacted plans. Fixture-backed global Claude Desktop and Cursor macOS adapters
discover and parse native JSON, then render plan-driven updates in memory while
preserving unowned fields. `init` deterministically
imports compatible local definitions, reports structural conflicts without
writing, skips named commandless Cursor entries that canonical v1 cannot
represent, and creates only a previously missing canonical file through a
same-directory temporary file. `add` validates one complete definition before
reading canonical state, performs a deterministic add/update, skips semantic
no-ops, and backs up then atomically replaces only the canonical regular file;
`list` exposes names, counts, and escaped environment key names but no process
values. `sync --dry-run` validates and structurally reports one fully rendered
two-target plan without mutation; `sync` applies those exact bytes with no-op
detection, recoverable backups, atomic replacement, per-target outcomes, and
reverse-order rollback after a later failure. It preserves target-only and
unowned native data, reports commandless Cursor entries without exposing their
values, and never touches a project-level Cursor file. No implemented command
starts configured server processes. A combined built-binary synthetic-home
suite proves the complete M1 command flow, deterministic import, redaction,
idempotence, native preservation, non-zero failures, and transaction rollback.
Controlled current-stable Cursor and Claude Desktop smokes both accept the
rendered global definitions and complete MCP initialization; the Claude journey
uses a no-clobber backup and verified exact restore around its temporary native
file. Other client adapters and process behavior remain later-ticket scope. Use
the existing Clap command tree as CLI behavior grows; do not introduce a second
parser.

“Extensible” currently means:

- one client-independent canonical model;
- one pure reconciliation engine that computes a plan before any mutation;
- narrow target adapters for client-specific paths and native formats;
- explicit filesystem and process boundaries that tests can replace;
- versioned configuration with deliberate migrations; and
- typed errors and stable command outcomes.

It does not mean adding speculative traits, factories, generic utility layers,
or package boundaries. Introduce an abstraction when a real variation exists or
when it protects a critical invariant.

## Project tracking workflow

Read `PROJECT.md` before material planning or implementation work.

- The repository owner explicitly requires one canonical Goal-mode objective
  per main-story ticket. Before beginning a main-story ticket in a runtime that
  supports goals, read the current thread goal and reconcile it exactly with
  the objective in PROJECT's main-story goal catalog. Set it when none is
  unfinished; edit a mismatched objective for the same eligible ticket when
  supported. If a goal for another unfinished ticket exists or the runtime
  prevents replacement, stop and report the mismatch instead of silently
  abandoning it or starting implementation.
- A goal does not make a ticket eligible and cannot override this file,
  `PROJECT.md`, dependencies, open decisions, or evidence gates. Move a ticket
  to `In progress` only after the goal matches. Mark the goal complete only
  after the ticket is `Done` with evidence; do not activate the successor until
  intentionally starting it. Agents without Goal mode use the catalog
  objective directly as their task contract.
- Do not assign a goal token budget unless the user explicitly supplies one.
- Associate each material change with a ticket ID. Create a ticket in the
  tracker first when no existing ticket covers the outcome.
- Follow the main-story sequence in `PROJECT.md` exactly. Do not begin a later
  main-story ticket until its immediate predecessor is `Done`.
- A blocked main-story ticket blocks the sequence. It may be cancelled or
  superseded only through an accepted decision that records why the outcome is
  irrelevant or names the replacement at the same point in the story.
- Use `SIDE-NNN` only for optional work that no main-story acceptance criterion,
  safety invariant, milestone gate, or release claim depends on. If a side
  quest becomes necessary, promote it into the ordered main story before
  continuing dependent work.
- A ticket may move to `In progress` only when its outcome, acceptance evidence,
  dependencies, immediate story predecessor, and owner are clear.
- Keep work in progress to one main-story ticket and at most one independent
  side quest that cannot delay or redefine the main story.
- A ticket is `Done` only when its acceptance criteria are met and its evidence
  column points to durable code, tests, documentation, or release verification.
- Update the current focus, deliverable status, decisions, risks, and review
  metadata in place when a change affects them. Git is the edit history; do not
  turn the tracker into a chronological diary.
- Record a blocker with the exact missing decision, dependency, or authority.
  “Needs more work” is not a blocker.
- If an external issue tracker is adopted, keep `PROJECT.md` as the repository
  index and milestone truth rather than maintaining two independent backlogs.

## Rust architecture

Keep dependency direction toward the domain:

```text
CLI parsing -> application use cases -> domain model and reconciliation
                                  |-> target adapters
                                  |-> filesystem adapter
                                  `-> process adapter (when health checks exist)
```

- The domain model and reconciliation logic must not depend on terminal output,
  operating-system paths, or a specific client schema.
- Target adapters translate between the canonical model and one client’s native
  representation. Keep discovery, parsing, rendering, and capability metadata
  cohesive for that target.
- Separate plan from apply. Planning reads and validates state and produces a
  deterministic change set; only the apply boundary may mutate files.
- Prefer typed structs and enums at boundaries. Validate external configuration
  before it reaches domain operations.
- Treat user input, client files, child-process output, and paths as untrusted.
  Return contextual errors rather than panicking on recoverable input or I/O
  failures.
- Keep functions focused and modules named after capabilities. Avoid `utils`,
  `helpers`, or `common` grab bags.
- Avoid `unsafe`. If it becomes necessary, isolate it, document the invariant,
  and add focused tests.
- Introduce asynchronous execution only for a demonstrated need such as bounded
  concurrent health checks; do not make file reconciliation asynchronous by
  default.

## Configuration and filesystem safety

Configuration files can contain credentials and unrelated settings. Treat data
integrity and redaction as product requirements.

- Never log or display environment-variable values, tokens, headers, command
  secrets, or complete unredacted configuration. Plans may show key names and
  structural changes with values redacted.
- Never access real user configuration in automated tests. Resolve all home,
  config, and target paths through an injectable environment and use temporary
  fixture directories.
- Preserve fields outside the adapter’s documented ownership boundary.
  Unknown top-level keys and target-specific data must not disappear as a side
  effect of sync.
- Manage only the global Cursor file at `~/.cursor/mcp.json`. Never discover or
  mutate a project-level `.cursor/mcp.json`; preserve commandless remote entries
  structurally, and reject a local canonical addition that collides with one.
- Treat Codex as one TOML target shared by the ChatGPT desktop app, Codex CLI,
  and IDE extension. Preserve non-MCP settings and unsupported MCP fields
  structurally; never round-trip `~/.codex/config.toml` through a lossy JSON
  representation.
- The initial sync behavior must not silently delete target-only servers.
  Report drift; any future prune behavior must be explicit, planned, backed up,
  and covered by recovery tests.
- Validate every desired output before writing it. Skip writes when bytes or
  semantic content are unchanged, as defined by the adapter contract.
- Write through a same-directory temporary file and use the safest atomic
  replacement available on the platform. Create a recoverable backup before
  replacing an existing file.
- A multi-target apply must report per-target outcomes and recover already
  changed targets if a later target fails. Never report overall success after a
  partial, unrecovered mutation.
- Inspect and test behavior for missing files, malformed JSON or TOML,
  permissions, symlinks, non-regular files, interrupted writes, backup
  collisions, and concurrent modification before claiming safe sync.
- `init` and `sync` must not execute configured MCP server commands. A health
  command is an explicit execution boundary and must use timeouts, terminate
  child processes, and avoid inheriting unnecessary input or secrets.

## Dependencies and configuration format

- Prefer the standard library and small, actively maintained crates with a
  narrow purpose.
- Before adding a crate, check existing capabilities, transitive dependencies,
  maintenance, license compatibility, security advisories, supported platforms,
  and effect on binary size or startup time.
- Commit `Cargo.lock` because `mcp-sync` is an application. Keep direct
  dependency requirements intentional and use stable releases unless an
  accepted ticket requires otherwise.
- The product and installed binary are `mcp-sync`, but the crates.io package
  name `mcp-sync` belongs to an unrelated project. `MCP-002` verified the
  distinct package name `enjoyable-mcp-sync`; recheck availability immediately
  before publication and never publish or document `cargo install mcp-sync`
  for this repository.
- Do not add an npm package, Node.js wrapper, or JavaScript distribution path
  without a superseding accepted decision. Target GitHub Releases, the
  organization Homebrew tap, WinGet, and the distinctly named Cargo package.
- `DEC-024` fixes the first public release at six separate 64-bit targets:
  ARM64 and x64 for macOS, GNU/Linux, and Windows MSVC. Do not add 32-bit,
  musl/Alpine, or universal-macOS artifacts without native CI, install evidence,
  and a later accepted support decision.
- Treat GitHub Releases as the canonical immutable channel. Stable publication
  requires the complete SHA-256 manifest, target-specific SPDX SBOMs, build
  attestations, and verified native smoke results before Homebrew, WinGet, or
  Cargo is updated to the same version. Never move a published tag or replace
  immutable assets; issue a new version.
- Stable macOS artifacts require Developer ID Application signing, hardened
  runtime, secure timestamping, and accepted notarization. Stable Windows
  executables require timestamped Public Trust Authenticode signing. Missing
  signing credentials block `MCP-021`; do not silently publish unsigned
  advertised artifacts or weaken checks for release convenience.
- Preserve the accepted public identities: GitHub `EnjoyableWork/mcp-sync`,
  Cargo `enjoyable-mcp-sync`, Homebrew `EnjoyableWork/tap/mcp-sync`, WinGet
  `EnjoyableWork.mcp-sync`, executable `mcp-sync`, and macOS signing identifier
  `com.enjoyablework.mcp-sync`.
- Do not implement cryptography, shell parsing, atomic-file semantics, or MCP
  protocol framing casually when a reviewed, maintained implementation is the
  safer choice.
- Keep the canonical configuration versioned from its first committed schema.
  Parsing may be liberal only when normalization is deterministic; writes must
  use one documented canonical representation.

## Testing and verification

Tests should prove behavior at the narrowest useful layer:

- unit tests for validation, normalization, merge rules, redaction, and plan
  generation;
- adapter fixture tests for every supported native client shape;
- filesystem integration tests in temporary directories for backup, atomic
  replacement, rollback, permissions, and no-op behavior; and
- CLI journey tests that exercise the built binary against a synthetic home
  directory.

Use deterministic fixtures and injected clocks/process runners where needed.
Do not call live MCP servers or depend on installed desktop clients in the
default test suite. A bug involving data loss, redaction, conflict handling, or
rollback requires a regression test.

The normal substantive-change handoff checks are:

1. `cargo fmt --all -- --check`
2. `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
3. `cargo test --workspace --all-targets --all-features --locked`
4. `cargo deny --all-features --locked check`
5. Any additional schema or release checks affected by the change

For documentation-only work, also run `git diff --check` and inspect headings,
tables, relative links, code fences, and claims.

## Documentation

- Preserve the README as the polished target-state product page. Do not add
  internal progress notes, ticket mechanics, implementation disclaimers, or a
  current-status banner unless the user explicitly changes its role.
- Keep every other document as canonical current-state guidance. Replace stale
  claims instead of layering corrections on top of them, and clearly label
  working assumptions, implemented behavior, and production-verified behavior.
- Update the README when the desired end-state install commands, supported
  platforms/clients, configuration paths, command behavior, or product promise
  changes. Track the gap between that promise and reality in `PROJECT.md`.
- Before publishing a release, verify every advertised installation channel and
  supported-client journey represented by that release; the README alone is
  never evidence that either exists.
- Update `PROJECT.md` whenever a ticket, milestone, deliverable, decision, risk,
  or MVP boundary changes.
- Use generic, synthetic server names and redacted values in examples. Never
  paste local user paths, private repository names, credentials, or real client
  configuration into tracked artifacts.
- Create a new document only for a durable contract, decision, runbook, or user
  workflow that would make `PROJECT.md` harder to operate.

## Version control and handoff

- Preserve unrelated changes in a dirty worktree. Inspect `git status --short`
  and the relevant diff before editing or staging.
- Use `rg` or `rg --files` for repository discovery before introducing parallel
  implementations.
- Follow Conventional Commits: `<type>[optional scope]: <imperative summary>`.
  Keep commits coherent and do not mix unrelated cleanup with a ticket.
- Do not commit, push, rewrite history, create tags, publish releases, or open
  pull requests unless the user requests it.
- At handoff, state the ticket outcome, files changed, checks run, and any
  remaining assumption, risk, or unverified gate.
