# Tech stack
- Rust is the selected implementation language and a single binary crate is the
  accepted initial shape. No Cargo manifest, dependencies, source tree, build
  configuration, or executable exists yet.
- The product and binary remain `mcp-sync`. The crates.io package `mcp-sync` is
  owned by an unrelated project, so `enjoyable-mcp-sync` is the working Cargo
  package name pending verification during `MCP-002`.
- Versioned JSON is the working assumption for canonical configuration; the
  exact schema is intentionally deferred to `MCP-004` in `PROJECT.md`.
- Target adapters preserve each client's native representation. Current
  north-star targets use JSON except Codex, whose shared host configuration is
  TOML at `~/.codex/config.toml`.
- README commands describe the target user experience and are not locally
  runnable. Distribution targets GitHub binaries, Homebrew, WinGet, and Cargo;
  this repository will not be an npm package. Do not infer Node.js project
  tooling from an example MCP server command that happens to use `npx`.
- Current repository artifacts are Markdown documentation, project metadata,
  and an MIT license.
