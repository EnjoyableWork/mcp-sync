use clap::{Parser, Subcommand};
use std::error::Error;
use std::fmt;
use std::process::ExitCode;

#[allow(
    dead_code,
    reason = "the adapter's render boundary is consumed by later sync use cases"
)]
mod claude_desktop;
#[allow(
    dead_code,
    reason = "MCP-004 establishes the canonical boundary before later use cases consume it"
)]
mod config;
#[allow(
    dead_code,
    reason = "the adapter's render boundary is consumed by later sync use cases"
)]
mod cursor;
#[allow(
    dead_code,
    reason = "context accessors support focused boundary tests and later apply use cases"
)]
mod filesystem;
mod init;
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
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Discover client configurations and create the canonical configuration.
    Init,
}

fn run(command: Command) -> Result<init::InitReport, ApplicationError> {
    match command {
        Command::Init => {
            let paths = paths::MacOsConfigurationPaths::resolve(&paths::ProcessEnvironment)
                .map_err(ApplicationError::ResolvePaths)?;
            init::initialize(&paths, &filesystem::OsFileSystem).map_err(ApplicationError::Init)
        }
    }
}

fn main() -> ExitCode {
    match run(Cli::parse().command) {
        Ok(report) => {
            println!("{report}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

#[derive(Debug)]
enum ApplicationError {
    ResolvePaths(paths::PathResolutionError),
    Init(init::InitError),
}

impl fmt::Display for ApplicationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ResolvePaths(error) => error.fmt(formatter),
            Self::Init(error) => error.fmt(formatter),
        }
    }
}

impl Error for ApplicationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ResolvePaths(error) => Some(error),
            Self::Init(error) => Some(error),
        }
    }
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
