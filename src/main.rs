use clap::Parser;

#[allow(
    dead_code,
    reason = "MCP-007 establishes the Claude Desktop adapter before application use cases consume it"
)]
mod claude_desktop;
#[allow(
    dead_code,
    reason = "MCP-004 establishes the canonical boundary before later use cases consume it"
)]
mod config;
#[allow(
    dead_code,
    reason = "MCP-005 establishes the filesystem boundary before later use cases consume it"
)]
mod filesystem;
#[allow(
    dead_code,
    reason = "MCP-005 establishes macOS paths before later adapters consume them"
)]
mod paths;
#[allow(
    dead_code,
    reason = "MCP-006 establishes pure reconciliation before application use cases consume it"
)]
mod reconciliation;

/// Synchronize local Model Context Protocol server configurations.
#[derive(Debug, Parser)]
#[command(name = "mcp-sync", version)]
struct Cli;

fn main() {
    Cli::parse();
}

#[cfg(test)]
mod tests {
    use super::Cli;
    use clap::CommandFactory;

    #[test]
    fn command_definition_is_valid() {
        Cli::command().debug_assert();
    }
}
