<h1 align="center">mcp-sync :zap:</h1>

<p align="center">
  <strong>Define your MCP servers once. Keep every local AI client in sync.</strong>
</p>

<p align="center">
  A safety-first CLI for reconciling local MCP server configuration across
  Claude Desktop, Cursor, Windsurf, VS Code, and Codex.
</p>

<p align="center">
  <a href="https://github.com/EnjoyableWork/mcp-sync/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/EnjoyableWork/mcp-sync/actions/workflows/ci.yml/badge.svg"></a>
  <a href="https://www.bestpractices.dev/projects/14014"><img alt="OpenSSF Baseline" src="https://www.bestpractices.dev/projects/14014/baseline"></a>
  <a href="https://docs.rs/crate/enjoyable-mcp-sync/latest"><img alt="crates.io version" src="https://img.shields.io/crates/v/enjoyable-mcp-sync.svg?logo=rust&amp;logoColor=white"></a>
  <a href="LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-blue.svg"></a>
</p>

<p align="center">
  <a href="#why-mcp-sync">Why mcp-sync?</a> ·
  <a href="#install">Install</a> ·
  <a href="#quick-start">Quick start</a> ·
  <a href="#safety-by-construction">Safety</a> ·
  <a href="#security-and-trust">Trust</a>
</p>

```console
$ mcp-sync init
Initialized canonical configuration with 3 servers from 2 client configurations.

$ mcp-sync sync --dry-run
Claude Desktop: would update with recoverable backup at "<path>"
Cursor: would update with recoverable backup at "<path>"
Windsurf: would update with recoverable backup at "<path>"
VS Code: would update with recoverable backup at "<path>"
Codex: would update with recoverable backup at "<path>"
```

Plan first. Preserve client-owned settings. Apply one deterministic change
across every target—or roll it back.

## Why mcp-sync?

Every MCP client has its own configuration file, native shape, and ownership
rules. Copying the same server definition between them creates drift, stale
credentials, malformed JSON or TOML, and changes that are difficult to review
or undo.

`mcp-sync` gives local MCP configuration one versioned source of truth and
translates only the fields it owns into each client's native format.

| Define once | See the change first | Recover deliberately |
| --- | --- | --- |
| Import compatible definitions or add one complete local STDIO server. | Validate and inspect a structurally redacted five-client plan before writing. | Keep adjacent backups, roll back partial multi-target writes, and restore one file explicitly. |

No daemon sits between a client and its server. `mcp-sync` updates local
configuration, gets out of the way, and starts a configured process only when
you explicitly run `mcp-sync test`.

## Install

Choose the channel that fits your platform and workflow:

| Channel | Platforms | Install |
| --- | --- | --- |
| Homebrew | macOS, Linux | `brew install EnjoyableWork/tap/mcp-sync` |
| Windows Package Manager | Windows | `winget install EnjoyableWork.mcp-sync` |
| Cargo | macOS, Linux, Windows | `cargo install enjoyable-mcp-sync` |
| GitHub Releases | macOS, Linux, Windows | [Download the latest archive](https://github.com/EnjoyableWork/mcp-sync/releases/latest) |

The crates.io package has the distinct registry name
`enjoyable-mcp-sync`, but installs the same `mcp-sync` executable.
Stable releases provide separate ARM64 and x64 archives for macOS, Linux, and
Windows. macOS binaries are signed and notarized, Windows binaries are
Authenticode-signed, and every immutable release includes SHA-256 checksums,
SPDX SBOMs, and verifiable build provenance.

## Quick start

### 1. Create the canonical configuration

Discover supported client files and import compatible local server definitions:

```bash
mcp-sync init
```

The canonical file is created only when it does not already exist. Structural
conflicts are reported without writing.

### 2. Add a complete server definition

```bash
mcp-sync add project-files \
  --command example-mcp-server \
  --arg --transport \
  --arg stdio \
  --env ACCESS_MODE=read-only
```

Arguments are literal and ordered, so repeat `--arg` once per process
argument instead of passing a shell command string. `--cmd` is available as a
shorter alias for `--command`.

### 3. Review and test it

```bash
mcp-sync list
mcp-sync test project-files
```

`list` shows names, counts, and environment key names without printing
commands, arguments, or environment values. `test` performs a bounded MCP
STDIO `initialize` exchange, validates the response, and closes or terminates
the child before returning.

> [!IMPORTANT]
> Only `test` starts the selected configured process. `init`, `list`,
> `sync`, and `restore` do not execute MCP servers.

### 4. Preview, then sync

```bash
mcp-sync sync --dry-run
mcp-sync sync
```

The dry run validates and reports the exact redacted plan without mutation.
Apply renders those same definitions into every supported target, skips
semantic no-ops, and reports each target outcome.

### 5. Restore when needed

```bash
mcp-sync restore cursor --dry-run
mcp-sync restore cursor
```

Restore accepts only `canonical`, `claude-desktop`, `cursor`, `windsurf`,
`vscode`, or `codex`. It validates the retained adjacent backup with the
selected JSON or TOML parser before changing anything.

## How it works

```text
                         canonical JSON
                    one versioned definition
                               │
                        validate + plan
                               │
               preserve unowned native settings
                               │
        ┌──────────────┬───────┼───────┬────────────┐
        ▼              ▼       ▼       ▼            ▼
  Claude Desktop    Cursor  Windsurf  VS Code      Codex
       JSON          JSON     JSON      JSON         TOML
```

### Canonical configuration

The client-independent source of truth describes local STDIO servers with one
literal executable, an ordered argument array, and literal environment strings:

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

`command`, `args`, and `env` values are preserved exactly rather than
parsed as shell syntax or expanded as variables. Canonical output has
deterministic key and field ordering, so unchanged configuration remains
stable. See the complete [canonical v1 example](examples/config.v1.json).

The canonical file lives at
`${XDG_CONFIG_HOME:-$HOME/.config}/mcp-sync/config.json` on macOS and Linux,
or `%LOCALAPPDATA%\mcp-sync\config.json` on Windows.

## Client coverage

All five targets support macOS, Linux, and Windows:

| Target | Managed global configuration | Native format |
| --- | --- | --- |
| Claude Desktop | `claude_desktop_config.json` | JSON `mcpServers` |
| Cursor | `.cursor/mcp.json` | JSON `mcpServers` |
| Windsurf | Legacy Cascade `.codeium/windsurf/mcp_config.json` | JSON `mcpServers` |
| VS Code | Default user-profile `mcp.json` | JSON `servers` |
| Codex | Shared `.codex/config.toml` host configuration | TOML `mcp_servers` |

<details>
<summary><strong>Managed paths by platform</strong></summary>

| Target | macOS | Linux | Windows |
| --- | --- | --- | --- |
| Claude Desktop | `~/Library/Application Support/Claude/claude_desktop_config.json` | `${XDG_CONFIG_HOME:-$HOME/.config}/Claude/claude_desktop_config.json` | `%APPDATA%\Claude\claude_desktop_config.json` |
| Cursor | `~/.cursor/mcp.json` | `~/.cursor/mcp.json` | `%USERPROFILE%\.cursor\mcp.json` |
| Windsurf | `~/.codeium/windsurf/mcp_config.json` | `~/.codeium/windsurf/mcp_config.json` | `%USERPROFILE%\.codeium\windsurf\mcp_config.json` |
| VS Code | `~/Library/Application Support/Code/User/mcp.json` | `${XDG_CONFIG_HOME:-$HOME/.config}/Code/User/mcp.json` | `%APPDATA%\Code\User\mcp.json` |
| Codex | `~/.codex/config.toml` | `~/.codex/config.toml` | `%USERPROFILE%\.codex\config.toml` |

</details>

The ChatGPT desktop app, Codex CLI, and Codex IDE extension share the same
global host configuration, so one Codex target keeps their local STDIO
definitions aligned while leaving project layers, remote/OAuth entries, and
credential stores untouched.

The VS Code target manages only the native default user profile. Workspace and
named-profile files, remote and Insiders profiles, portable installations,
Cline, Roo Code, and Agent Host/Copilot CLI configuration remain outside its
boundary. The Windsurf target manages the documented legacy Cascade file, not
Devin Local agent configuration.

## Safety by construction

- **Plan before apply.** Every target is parsed, validated, and rendered in
  memory before the first write. `sync --dry-run` exposes that plan without
  mutation.
- **Preserve what mcp-sync does not own.** Client-only fields, settings,
  comments, target-only servers, and unmanaged remote transports survive a
  sync. There is no implicit prune.
- **Keep secrets structural.** Commands, argument values, environment values,
  headers, server output, and raw configuration stay out of ordinary
  diagnostics.
- **Replace recoverably.** Changed regular files receive one adjacent `.bak`
  generation and are replaced through a same-directory temporary file.
- **Treat five targets as one transaction.** If a later write fails, earlier
  target changes are rolled back in reverse order; overall success is never
  reported for an unrecovered partial apply.
- **Fail closed.** Malformed, unreadable, symbolic-link, non-regular, stale, or
  concurrently changed inputs are refused instead of flattened or overwritten.

Backup retention is deliberately one adjacent generation per managed file,
with no time-based expiration. A successful changed `add`, `sync`, or
existing-target `restore` retains the bytes immediately preceding that
operation. Creations have no prior-file backup; no-ops and failed or rolled-back
operations preserve the existing slot. Copy both files to access-controlled
storage before another change when you need longer history.

For the complete operational and recovery contract, see the
[usage and recovery guide](docs/m1-usage-and-recovery.md).

## Security and trust

[![OpenSSF Baseline](https://www.bestpractices.dev/projects/14014/baseline)](https://www.bestpractices.dev/projects/14014)

On 2026-08-09, all 24 OpenSSF OSPS Baseline `v2026.02.19` Level 1 controls were
recorded `Met` in the official public
[`baseline-1` assessment](https://www.bestpractices.dev/en/projects/14014/baseline-1).
This is a maintainer self-certification, not OSPS Level 2 or 3, independent
certification, regulatory compliance, or an OpenSSF endorsement.

The [security-assurance record](docs/security-assurance.md) links the evidence,
scope, limitations, assessment date, and revalidation triggers. It separately
limits its SLSA v1.0 Build Level 2 statement to seven verified `v0.1.0` GitHub
Release assets rather than every release, channel, or project practice.

Suspected vulnerabilities must use
[GitHub private vulnerability reporting](https://github.com/EnjoyableWork/mcp-sync/security/advisories/new),
never a public issue or pull request. See [SECURITY.md](SECURITY.md) for the
supported-version and disclosure policy.

## Contributing

Contributions are welcome. Start with [CONTRIBUTING.md](CONTRIBUTING.md), ask
for help through [SUPPORT.md](SUPPORT.md), and follow the
[Code of Conduct](CODE_OF_CONDUCT.md). The
[project and licensing inventory](docs/project-community-and-licensing.md)
records the authoritative repositories and MIT source-and-release boundary.

<details>
<summary><strong>Build and verify from source</strong></summary>

On macOS or Linux:

```bash
cargo build --locked
cargo run --locked -- --help
./scripts/check.sh
cargo deny --all-features --locked check
```

On Windows PowerShell:

```powershell
cargo build --locked
cargo run --locked -- --help
.\scripts\check.ps1
cargo deny --all-features --locked check
```

The platform-native quality scripts run formatting, Clippy, unit, and
integration checks against disposable configuration roots. The dependency
policy uses the CI-pinned `cargo-deny` 0.20.2 release:

```bash
cargo install --locked cargo-deny --version 0.20.2
```

</details>

## License

Distributed under the [MIT License](LICENSE).
