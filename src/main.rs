use clap::Parser;

/// Synchronize local Model Context Protocol server configurations.
#[derive(Debug, Parser)]
#[command(name = "mcp-sync", version)]
struct Cli;

fn main() {
    Cli::parse();
}
