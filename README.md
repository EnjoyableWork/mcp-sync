# mcp-sync

The unified configuration engine for Model Context Protocol (MCP) servers.

Define your local MCP servers once. `mcp-sync` validates process health and automatically updates configuration files across Claude Desktop, Cursor, Windsurf, VS Code, and Codex in a single command.

## ⚡ The Problem

Adding an MCP server—such as Postgres, GitHub, or Brave Search—to your local AI
toolchain currently requires manual editing across multiple isolated
configuration files. On macOS, the global files are:

- **Claude Desktop:** `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Cursor:** `~/.cursor/mcp.json`
- **Windsurf:** `~/.codeium/windsurf/mcp_config.json`
- **VS Code (native MCP / GitHub Copilot):** `~/Library/Application Support/Code/User/mcp.json`
- **Codex (ChatGPT desktop app / Codex CLI / IDE extension):** `~/.codex/config.toml`

On Linux, Claude Desktop and VS Code use
`$XDG_CONFIG_HOME/Claude/claude_desktop_config.json` and
`$XDG_CONFIG_HOME/Code/User/mcp.json`, falling back to the corresponding paths
under `~/.config` when `XDG_CONFIG_HOME` is unset or empty. Cursor, Windsurf,
and Codex retain the home-relative paths shown above.

This fragmentation leads to syntax errors, environment variable drift, broken credentials, and wasted setup time whenever a new server or API key is updated.

## 🚀 The Solution

`mcp-sync` establishes a single local source of truth at `~/.config/mcp-sync/config.json`. Add or update servers in one place, dry-run connection health, and push native configurations to all local AI clients simultaneously.

```text
                         ┌─────────────────────────┐
                         │   ~/.config/mcp-sync    │
                         │    (Master Config)      │
                         └────────────┬────────────┘
                                      │
                                  mcp-sync sync
                                      │
                ┌─────────────────────┴─────────────────────┐
                │            Native target adapters         │
                └─────────────────────┬─────────────────────┘
                                      │
┌────────────────┐ ┌────────────┐ ┌────────────────┐ ┌───────────────┐ ┌────────────────┐
│ Claude Desktop │ │ Cursor     │ │ Windsurf       │ │ VS Code       │ │ Codex          │
│ config.json    │ │ mcp.json   │ │ mcp_config.json│ │ mcp.json      │ │ config.toml    │
└────────────────┘ └────────────┘ └────────────────┘ └───────────────┘ └────────────────┘
```

## 🧭 Canonical Configuration

The source of truth is versioned, client-independent JSON. Version 1 describes
local STDIO servers with one literal executable, an ordered argument array, and
literal environment strings:

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

`command`, `args`, and `env` values are preserved exactly rather than parsed as
shell syntax or expanded as variables. Canonical output has deterministic key
and field ordering so unchanged configuration remains stable. A complete
checked-in example is available at [examples/config.v1.json](examples/config.v1.json).

## ✨ Features

- 🔍 **Auto-Discovery:** Scrapes and imports existing MCP configurations on first launch.
- 🔄 **Multi-Client Sync:** Maps master configurations into each client's native format instantly.
- 🩺 **STDIO Health Testing:** Dry-runs server binaries using JSON-RPC handshakes before writing configs.
- 🛡️ **Atomic Writes & Backups:** Automatically creates `.bak` copies before modifying target files.
- ⚡ **Zero Latency:** Operates purely as a configuration tool and gets out of the way during runtime.

## 📦 Installation

### Homebrew (macOS and Linux)

```bash
brew install EnjoyableWork/tap/mcp-sync
```

### Windows Package Manager

```powershell
winget install EnjoyableWork.mcp-sync
```

### Cargo

The Cargo package has a distinct registry name but installs the same
`mcp-sync` executable:

```bash
cargo install enjoyable-mcp-sync
```

### Prebuilt binaries

Download a prebuilt archive for macOS, Linux, or Windows from the
[latest GitHub release](https://github.com/EnjoyableWork/mcp-sync/releases/latest).
Stable releases provide separate ARM64 and x64 archives for each operating
system. macOS binaries are signed and notarized, Windows binaries are
Authenticode-signed, and every immutable release includes SHA-256 checksums,
SPDX SBOMs, and verifiable build provenance.

## 📖 Quickstart & Usage

### 1. Initialize the master configuration

Auto-detect installed clients and import existing MCP server configurations into `~/.config/mcp-sync/config.json`:

```bash
mcp-sync init
```

### 2. Add an MCP server

Add a new server with a command binary, arguments, and optional environment variables:

```bash
mcp-sync add postgres \
  --command "npx" \
  --arg "-y" \
  --arg "@modelcontextprotocol/server-postgres" \
  --env "POSTGRES_URL=postgresql://localhost:5432/dev"
```

Arguments are literal and ordered, so repeat `--arg` once per process argument
instead of passing a shell command string. `--cmd` is also accepted as a shorter
alias for `--command`.

### 3. Review configured servers safely

List server names and structural metadata without printing commands, arguments,
or environment values:

```bash
mcp-sync list
# "postgres": command: <redacted>; arguments: 2; environment keys: "POSTGRES_URL"
```

### 4. Test the process connection

Verify that the server boots cleanly via STDIO and responds to an MCP `initialize` request:

```bash
mcp-sync test postgres
# Server "postgres" initialized successfully via STDIO (protocol version 2025-11-25).
```

Only `test` starts the selected configured process. It validates a bounded,
newline-delimited JSON-RPC initialize exchange, reports protocol structure
without echoing process or server-output values, and always closes or
terminates the child before returning. `init` and `sync` remain configuration
operations and never start configured servers.

### 5. Sync configuration across all clients

Preview one validated, structurally redacted plan without changing any files:

```bash
mcp-sync sync --dry-run
```

Then push the same planned server definitions to all detected local IDEs and
AI applications:

```bash
mcp-sync sync
```

Targets that are already semantically current are not rewritten. Existing
files receive recoverable `.bak` copies before atomic replacement, and if a
later target fails, earlier target changes are rolled back and reported per
target.

## 💻 Supported Clients

| Target client | Supported platforms | Configuration file managed |
| --- | --- | --- |
| Claude Desktop | macOS, Linux, Windows | `claude_desktop_config.json` |
| Cursor | macOS, Linux, Windows | `mcp.json` |
| Windsurf | macOS, Linux, Windows | `mcp_config.json` |
| VS Code (native MCP / GitHub Copilot) | macOS, Linux, Windows | User-profile `mcp.json` |
| Codex (ChatGPT desktop app / Codex CLI / IDE extension) | macOS, Linux, Windows | `~/.codex/config.toml` |

The ChatGPT desktop app, Codex CLI, and Codex IDE extension share the same host
configuration, so one Codex target keeps their MCP server definitions aligned
while preserving unrelated Codex settings.

The VS Code target uses the editor's native user-profile `mcp.json` `servers`
shape. It is distinct from Cline, Roo Code, and other extensions that maintain
their own configuration stores.

## 🏗️ Technical Architecture

```text
┌─────────────────────────────────────────────────────────────┐
│                       mcp-sync CLI                          │
│  ┌──────────────┐    ┌──────────────┐    ┌───────────────┐  │
│  │ Config Engine│ ──►│ Health Check │ ──►│ Client Sync   │  │
│  └──────┬───────┘    └──────────────┘    └──────┬────────┘  │
└─────────┼───────────────────────────────────────┼───────────┘
          │                                       │
          ▼                                       ▼
┌───────────────────┐               ┌──────────────────────────┐
│  Master Config    │               │ Client Targets           │
│  ~/.config/       │               │ • Claude Desktop JSON    │
│  mcp-sync/        │               │ • Cursor / Windsurf JSON │
│  config.json      │               │ • VS Code JSON           │
│                   │               │ • Codex TOML              │
└───────────────────┘               └──────────────────────────┘
```

## 🤝 Contributing

To build and verify `mcp-sync` from a source checkout:

```bash
cargo build --locked
cargo run --locked -- --help
./scripts/check.sh
cargo deny --all-features --locked check
```

`./scripts/check.sh` is the canonical local quality gate. It runs deterministic
formatting, Clippy, unit, and integration checks with the committed lockfile.
Every command receives disposable `HOME`, XDG, macOS application-support,
Linux user-data, and Windows-style user configuration roots, and the CLI
integration harness clears inherited process state before supplying its own
synthetic home. The dependency-policy command requires the CI-pinned
`cargo-deny` 0.20.2 release; install that exact version with
`cargo install --locked cargo-deny --version 0.20.2`. Its committed policy checks
security advisories, licenses, duplicate or banned dependencies, and dependency
sources.

Contributions are welcome! Feel free to submit a pull request:

1. Fork the repository.
2. Create your feature branch: `git checkout -b feature/amazing-feature`.
3. Commit your changes: `git commit -m 'Add amazing feature'`.
4. Push to the branch: `git push origin feature/amazing-feature`.
5. Open a pull request.

## 📄 License

Distributed under the MIT License. See [LICENSE](LICENSE) for details.
