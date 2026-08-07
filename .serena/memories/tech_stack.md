# Tech stack
- Rust 2024 is the implementation language. `rust-toolchain.toml` follows the
  stable toolchain; `MCP-002` passed on Rust/Cargo 1.94.1, and a minimum
  supported version remains deferred until `MCP-003` CI evidence exists.
- The single package is `enjoyable-mcp-sync` and installs one `mcp-sync` binary.
  A live crates.io query found that distinct name unclaimed on 2026-08-06; it
  must be rechecked before publication because the unrelated `mcp-sync` package
  is already owned.
- Clap 4.6.6 with its derive feature is the only direct dependency and the
  single CLI parser. The implemented command surface is limited to `--help` and
  `--version`.
- Versioned JSON is the working assumption for canonical configuration; the
  exact schema is intentionally deferred to `MCP-004` in `PROJECT.md`.
- Target adapters preserve each client's native representation. Current
  north-star targets use JSON except Codex, whose shared host configuration is
  TOML at `~/.codex/config.toml`.
- README product commands beyond help/version remain north-star behavior rather
  than implemented functionality. Distribution targets GitHub binaries,
  Homebrew, WinGet, and Cargo; this repository will not be an npm package. Do
  not infer Node.js project tooling from an example MCP server command that
  happens to use `npx`.
