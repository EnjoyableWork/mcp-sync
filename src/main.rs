use clap::Parser;

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
