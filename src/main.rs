use clap::{ArgAction, Args, Parser, Subcommand};
use std::error::Error;
use std::fmt;
use std::process::ExitCode;

mod catalog;
mod claude_desktop;
mod codex;
mod config;
mod cursor;
mod filesystem;
mod health;
mod init;
mod paths;
mod reconciliation;
mod sync;
mod vscode;
mod windsurf;

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
    /// Test one canonical server through a bounded MCP STDIO initialize exchange.
    Test(TestCommand),
    /// Reconcile canonical servers into every supported target.
    Sync(SyncCommand),
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

#[derive(Args)]
struct SyncCommand {
    /// Validate and report the exact plan without changing any files.
    #[arg(long)]
    dry_run: bool,
}

#[derive(Args)]
struct TestCommand {
    /// Canonical name of the server definition to test.
    name: String,
}

fn run(command: Command) -> Result<CommandReport, ApplicationError> {
    let paths = paths::ConfigurationPaths::resolve(&paths::ProcessEnvironment)
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
        Command::Test(command) => health::test_server(
            &paths,
            &filesystem::OsFileSystem,
            &health::OsInitializeTester,
            &command.name,
        )
        .map(CommandReport::Test)
        .map_err(ApplicationError::Health),
        Command::Sync(command) => {
            let plan = sync::plan_sync(&paths, &filesystem::OsFileSystem)
                .map_err(ApplicationError::Sync)?;
            if command.dry_run {
                Ok(CommandReport::Sync(sync::dry_run(&plan)))
            } else {
                sync::apply_sync(&plan, &filesystem::OsFileSystem)
                    .map(CommandReport::Sync)
                    .map_err(ApplicationError::Sync)
            }
        }
    }
}

enum CommandReport {
    Init(init::InitReport),
    Add(catalog::AddReport),
    List(catalog::ListReport),
    Test(health::HealthReport),
    Sync(sync::SyncReport),
}

impl fmt::Display for CommandReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Init(report) => report.fmt(formatter),
            Self::Add(report) => report.fmt(formatter),
            Self::List(report) => report.fmt(formatter),
            Self::Test(report) => report.fmt(formatter),
            Self::Sync(report) => report.fmt(formatter),
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
    Health(health::HealthError),
    Sync(sync::SyncError),
}

impl fmt::Display for ApplicationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ResolvePaths(error) => error.fmt(formatter),
            Self::Init(error) => error.fmt(formatter),
            Self::Catalog(error) => error.fmt(formatter),
            Self::Health(error) => error.fmt(formatter),
            Self::Sync(error) => error.fmt(formatter),
        }
    }
}

impl Error for ApplicationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ResolvePaths(error) => Some(error),
            Self::Init(error) => Some(error),
            Self::Catalog(error) => Some(error),
            Self::Health(error) => Some(error),
            Self::Sync(error) => Some(error),
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
