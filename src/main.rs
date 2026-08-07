use clap::{ArgAction, Args, Parser, Subcommand};
use std::error::Error;
use std::fmt;
use std::process::ExitCode;

mod catalog;
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
#[derive(Parser)]
#[command(name = "mcp-sync", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Discover client configurations and create the canonical configuration.
    Init,
    /// Add or replace one complete server definition in canonical configuration.
    Add(AddCommand),
    /// List canonical servers without exposing commands, arguments, or values.
    List,
}

#[derive(Args)]
struct AddCommand {
    /// Canonical name of the server definition.
    name: String,

    /// Executable or command to store (redacted from all command output).
    #[arg(long, visible_alias = "cmd", value_name = "EXECUTABLE")]
    command: String,

    /// One literal ordered argument; repeat this option for multiple arguments.
    #[arg(
        long = "arg",
        value_name = "ARGUMENT",
        action = ArgAction::Append,
        allow_hyphen_values = true
    )]
    arguments: Vec<String>,

    /// One literal environment assignment; repeat as KEY=VALUE.
    #[arg(long = "env", value_name = "KEY=VALUE", action = ArgAction::Append)]
    environment: Vec<String>,
}

fn run(command: Command) -> Result<CommandReport, ApplicationError> {
    let paths = paths::MacOsConfigurationPaths::resolve(&paths::ProcessEnvironment)
        .map_err(ApplicationError::ResolvePaths)?;

    match command {
        Command::Init => init::initialize(&paths, &filesystem::OsFileSystem)
            .map(CommandReport::Init)
            .map_err(ApplicationError::Init),
        Command::Add(command) => catalog::add_server(
            &paths,
            &filesystem::OsFileSystem,
            catalog::AddRequest::new(
                command.name,
                command.command,
                command.arguments,
                command.environment,
            ),
        )
        .map(CommandReport::Add)
        .map_err(ApplicationError::Catalog),
        Command::List => catalog::list_servers(&paths, &filesystem::OsFileSystem)
            .map(CommandReport::List)
            .map_err(ApplicationError::Catalog),
    }
}

enum CommandReport {
    Init(init::InitReport),
    Add(catalog::AddReport),
    List(catalog::ListReport),
}

impl fmt::Display for CommandReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Init(report) => report.fmt(formatter),
            Self::Add(report) => report.fmt(formatter),
            Self::List(report) => report.fmt(formatter),
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
    Catalog(catalog::CatalogError),
}

impl fmt::Display for ApplicationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ResolvePaths(error) => error.fmt(formatter),
            Self::Init(error) => error.fmt(formatter),
            Self::Catalog(error) => error.fmt(formatter),
        }
    }
}

impl Error for ApplicationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ResolvePaths(error) => Some(error),
            Self::Init(error) => Some(error),
            Self::Catalog(error) => Some(error),
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
