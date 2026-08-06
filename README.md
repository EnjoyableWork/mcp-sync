# mcp-sync

The unified configuration engine for Model Context Protocol (MCP) servers.

Define your local MCP servers once. `mcp-sync` validates process health and automatically updates configuration files across Claude Desktop, Cursor, Windsurf, and VS Code in a single command.

## ⚡ The Problem

Adding an MCP server—such as Postgres, GitHub, or Brave Search—to your local AI toolchain currently requires manual editing across multiple isolated configuration files:

- **Claude Desktop:** `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Cursor:** `~/.cursor/mcp.json`
- **Windsurf:** `~/.codeium/windsurf/mcp_config.json`
- **VS Code (Cline/Roo/Copilot):** `~/.config/Code/User/globalStorage/.../settings.json`

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
         ┌──────────────────┬─────────┴─────────┬──────────────────┐
         ▼                  ▼                   ▼                  ▼
┌──────────────────┐┌───────────────┐┌──────────────────┐┌──────────────────┐
│  Claude Desktop  ││    Cursor     ││     Windsurf     ││     VS Code      │
│  config.json     ││  mcp.json     ││  mcp_config.json ││  settings.json   │
└──────────────────┘└───────────────┘└──────────────────┘└──────────────────┘
```

## ✨ Features

- 🔍 **Auto-Discovery:** Scrapes and imports existing MCP configurations on first launch.
- 🔄 **Multi-Client Sync:** Maps master configurations into native client JSON formats instantly.
- 🩺 **STDIO Health Testing:** Dry-runs server binaries using JSON-RPC handshakes before writing configs.
- 🛡️ **Atomic Writes & Backups:** Automatically creates `.bak` copies before modifying target files.
- ⚡ **Zero Latency:** Operates purely as a configuration tool and gets out of the way during runtime.

## 📦 Installation

Run instantly with `npx`—no permanent installation required:

```bash
npx mcp-sync --help
```

Or install globally via npm, Yarn, or pnpm:

```bash
npm install -g mcp-sync
```

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
  --cmd "npx" \
  --args "-y @modelcontextprotocol/server-postgres" \
  --env "POSTGRES_URL=postgresql://localhost:5432/dev"
```

### 3. Test the process connection

Verify that the server boots cleanly via STDIO and responds to an MCP `initialize` request:

```bash
mcp-sync test postgres
# Output: ✔ Connecting via STDIO... Connected! (Protocol Version: 2024-11-05)
```

### 4. Sync configuration across all clients

Push the updated master server definitions to all detected local IDEs and AI applications:

```bash
mcp-sync sync
```

## 💻 Supported Clients

| Target client | Supported platforms | Configuration file managed |
| --- | --- | --- |
| Claude Desktop | macOS, Windows | `claude_desktop_config.json` |
| Cursor | macOS, Linux, Windows | `mcp.json` |
| Windsurf | macOS, Linux, Windows | `mcp_config.json` |
| VS Code (Cline / Copilot) | macOS, Linux, Windows | `cline_mcp_settings.json` |

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
│  ~/.config/       │               │ • Claude Desktop Config  │
│  mcp-sync/        │               │ • Cursor mcp.json        │
│  config.json      │               │ • Windsurf mcp_config    │
└───────────────────┘               └──────────────────────────┘
```

## 🤝 Contributing

Contributions are welcome! Feel free to submit a pull request:

1. Fork the repository.
2. Create your feature branch: `git checkout -b feature/amazing-feature`.
3. Commit your changes: `git commit -m 'Add amazing feature'`.
4. Push to the branch: `git push origin feature/amazing-feature`.
5. Open a pull request.

## 📄 License

Distributed under the MIT License. See [LICENSE](LICENSE) for details.
