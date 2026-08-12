use clap::{ArgAction, Args, Parser, Subcommand, ValueEnum};
use std::error::Error;
use std::fmt;
use std::path::PathBuf;
use std::process::ExitCode;

mod catalog;
mod claude_desktop;
mod codex;
mod config;
mod cursor;
mod filesystem;
mod health;
mod init;
mod kiro;
mod operation_lock;
mod paths;
mod process_containment;
mod reconciliation;
mod restore;
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
    /// Restore one managed configuration from its validated adjacent backup.
    Restore(RestoreCommand),
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

#[derive(Args)]
struct RestoreCommand {
    /// Managed global configuration whose adjacent backup should be restored.
    #[arg(value_enum)]
    configuration: RestoreTargetArgument,

    /// Validate and report the restore without changing the target or backup.
    #[arg(long)]
    dry_run: bool,
}

#[derive(Clone, Copy, ValueEnum)]
enum RestoreTargetArgument {
    Canonical,
    ClaudeDesktop,
    Cursor,
    Windsurf,
    #[value(name = "vscode", alias = "vs-code")]
    VsCode,
    Codex,
    Kiro,
}

impl From<RestoreTargetArgument> for restore::RestoreTarget {
    fn from(target: RestoreTargetArgument) -> Self {
        match target {
            RestoreTargetArgument::Canonical => Self::Canonical,
            RestoreTargetArgument::ClaudeDesktop => Self::ClaudeDesktop,
            RestoreTargetArgument::Cursor => Self::Cursor,
            RestoreTargetArgument::Windsurf => Self::Windsurf,
            RestoreTargetArgument::VsCode => Self::VsCode,
            RestoreTargetArgument::Codex => Self::Codex,
            RestoreTargetArgument::Kiro => Self::Kiro,
        }
    }
}

fn run(command: Command) -> Result<CommandReport, ApplicationError> {
    let paths = paths::ConfigurationPaths::resolve(&paths::ProcessEnvironment)
        .map_err(ApplicationError::ResolvePaths)?;

    match command {
        Command::Init => {
            let _operation_lock = acquire_operation_lock(&paths)?;
            recover_pending_replacements(&paths)?;
            init::initialize(&paths, &filesystem::OsFileSystem)
                .map(CommandReport::Init)
                .map_err(ApplicationError::Init)
        }
        Command::Add(command) => {
            let request = catalog::AddRequest::new(
                command.name,
                command.command,
                command.arguments,
                command.environment,
            )
            .validate()
            .map_err(ApplicationError::Catalog)?;
            let _operation_lock = acquire_operation_lock(&paths)?;
            recover_pending_replacements(&paths)?;
            catalog::add_server(&paths, &filesystem::OsFileSystem, request)
                .map(CommandReport::Add)
                .map_err(ApplicationError::Catalog)
        }
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
        Command::Restore(command) => {
            if command.dry_run {
                let plan = restore::plan_restore(
                    &paths,
                    &filesystem::OsFileSystem,
                    command.configuration.into(),
                )
                .map_err(ApplicationError::Restore)?;
                Ok(CommandReport::Restore(restore::dry_run(&plan)))
            } else {
                let _operation_lock = acquire_operation_lock(&paths)?;
                recover_pending_replacements(&paths)?;
                let plan = restore::plan_restore(
                    &paths,
                    &filesystem::OsFileSystem,
                    command.configuration.into(),
                )
                .map_err(ApplicationError::Restore)?;
                restore::apply_restore(&plan, &filesystem::OsFileSystem)
                    .map(CommandReport::Restore)
                    .map_err(ApplicationError::Restore)
            }
        }
        Command::Sync(command) => {
            if command.dry_run {
                let plan = sync::plan_sync(&paths, &filesystem::OsFileSystem)
                    .map_err(ApplicationError::Sync)?;
                Ok(CommandReport::Sync(sync::dry_run(&plan)))
            } else {
                let _operation_lock = acquire_operation_lock(&paths)?;
                recover_pending_replacements(&paths)?;
                let plan = sync::plan_sync(&paths, &filesystem::OsFileSystem)
                    .map_err(ApplicationError::Sync)?;
                sync::apply_sync(&plan, &filesystem::OsFileSystem)
                    .map(CommandReport::Sync)
                    .map_err(ApplicationError::Sync)
            }
        }
    }
}

fn acquire_operation_lock(
    paths: &paths::ConfigurationPaths,
) -> Result<operation_lock::OperationLockGuard, ApplicationError> {
    operation_lock::try_acquire(paths.operation_lock()).map_err(ApplicationError::OperationLock)
}

fn recover_pending_replacements(paths: &paths::ConfigurationPaths) -> Result<(), ApplicationError> {
    let managed_paths = managed_configuration_paths(paths);
    filesystem::recover_pending_replacements(managed_paths.iter().map(PathBuf::as_path))
        .map_err(ApplicationError::ReplacementRecovery)
}

fn managed_configuration_paths(paths: &paths::ConfigurationPaths) -> [PathBuf; 7] {
    [
        paths.canonical_configuration().to_owned(),
        claude_desktop::ClaudeDesktopAdapter::from_paths(paths)
            .configuration_path()
            .to_owned(),
        cursor::CursorAdapter::from_paths(paths)
            .configuration_path()
            .to_owned(),
        windsurf::WindsurfAdapter::from_paths(paths)
            .configuration_path()
            .to_owned(),
        vscode::VsCodeAdapter::from_paths(paths)
            .configuration_path()
            .to_owned(),
        codex::CodexAdapter::from_paths(paths)
            .configuration_path()
            .to_owned(),
        kiro::KiroAdapter::from_paths(paths)
            .configuration_path()
            .to_owned(),
    ]
}

enum CommandReport {
    Init(init::InitReport),
    Add(catalog::AddReport),
    List(catalog::ListReport),
    Test(health::HealthReport),
    Restore(restore::RestoreReport),
    Sync(sync::SyncReport),
}

impl fmt::Display for CommandReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Init(report) => report.fmt(formatter),
            Self::Add(report) => report.fmt(formatter),
            Self::List(report) => report.fmt(formatter),
            Self::Test(report) => report.fmt(formatter),
            Self::Restore(report) => report.fmt(formatter),
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
    OperationLock(operation_lock::OperationLockError),
    ReplacementRecovery(filesystem::FileMutationError),
    Init(init::InitError),
    Catalog(catalog::CatalogError),
    Health(health::HealthError),
    Restore(restore::RestoreError),
    Sync(sync::SyncError),
}

impl fmt::Display for ApplicationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ResolvePaths(error) => error.fmt(formatter),
            Self::OperationLock(error) => error.fmt(formatter),
            Self::ReplacementRecovery(error) => error.fmt(formatter),
            Self::Init(error) => error.fmt(formatter),
            Self::Catalog(error) => error.fmt(formatter),
            Self::Health(error) => error.fmt(formatter),
            Self::Restore(error) => error.fmt(formatter),
            Self::Sync(error) => error.fmt(formatter),
        }
    }
}

impl Error for ApplicationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ResolvePaths(error) => Some(error),
            Self::OperationLock(error) => Some(error),
            Self::ReplacementRecovery(error) => Some(error),
            Self::Init(error) => Some(error),
            Self::Catalog(error) => Some(error),
            Self::Health(error) => Some(error),
            Self::Restore(error) => Some(error),
            Self::Sync(error) => Some(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AddCommand, Cli, Command, RestoreCommand, RestoreTargetArgument, SyncCommand, filesystem,
        operation_lock, paths, run,
    };
    use clap::CommandFactory;
    use std::collections::BTreeMap;
    use std::ffi::OsString;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::{Command as ProcessCommand, Output, Stdio};
    use std::thread;
    use std::time::{Duration, Instant};

    const CHILD_MODE: &str = "MCP_SYNC_TEST_REPLACEMENT_CHILD_MODE";
    const FAULT_BOUNDARY: &str = "MCP_SYNC_TEST_REPLACEMENT_BOUNDARY";
    const FAULT_SENTINEL: &str = "MCP_SYNC_TEST_REPLACEMENT_SENTINEL";
    const CHILD_TEST: &str = "tests::abrupt_replacement_process_child";

    const CANONICAL_A: &[u8] = br#"{
  "schemaVersion": 1,
  "servers": {
    "fixture-server": {
      "command": "older-private-command",
      "args": [],
      "env": {}
    }
  }
}
"#;
    const CANONICAL_B: &[u8] = br#"{
  "schemaVersion": 1,
  "servers": {
    "fixture-server": {
      "command": "old-private-command",
      "args": [],
      "env": {}
    }
  }
}
"#;
    const CANONICAL_C: &[u8] = br#"{
  "schemaVersion": 1,
  "servers": {
    "fixture-server": {
      "command": "new-private-command",
      "args": [],
      "env": {}
    }
  }
}
"#;
    const CLAUDE_A: &[u8] = br#"{
  "mcpServers": {
    "fixture-server": {
      "command": "older-private-command",
      "args": [],
      "env": {}
    }
  }
}
"#;
    const CLAUDE_B: &[u8] = br#"{
  "mcpServers": {
    "fixture-server": {
      "command": "old-private-command",
      "args": [],
      "env": {}
    }
  }
}
"#;

    #[derive(Clone, Copy, Debug)]
    enum Journey {
        Add,
        Sync,
        Restore,
    }

    impl Journey {
        fn initial_mode(self) -> &'static str {
            match self {
                Self::Add => "initial-add",
                Self::Sync => "initial-sync",
                Self::Restore => "initial-restore",
            }
        }

        fn read_mode(self) -> &'static str {
            match self {
                Self::Add => "read-add",
                Self::Sync => "read-sync",
                Self::Restore => "read-restore",
            }
        }
    }

    #[derive(Clone, Copy, Debug)]
    enum Boundary {
        PreparingJournal,
        PreparedJournal,
        TargetPublished,
        BackupPublished,
    }

    impl Boundary {
        fn value(self) -> &'static str {
            match self {
                Self::PreparingJournal => "preparing-journal",
                Self::PreparedJournal => "prepared-journal",
                Self::TargetPublished => "target-published",
                Self::BackupPublished => "backup-published",
            }
        }

        fn target_committed(self) -> bool {
            matches!(self, Self::TargetPublished | Self::BackupPublished)
        }

        fn backup_committed(self) -> bool {
            matches!(self, Self::BackupPublished)
        }
    }

    struct FixtureEnvironment {
        values: BTreeMap<&'static str, OsString>,
    }

    impl paths::Environment for FixtureEnvironment {
        fn value(&self, name: &'static str) -> Option<OsString> {
            self.values.get(name).cloned()
        }
    }

    struct AbruptFixture {
        _root: tempfile::TempDir,
        environment: FixtureEnvironment,
        paths: paths::ConfigurationPaths,
    }

    impl AbruptFixture {
        fn new() -> Self {
            let root = tempfile::tempdir().expect("the abrupt-process root should be created");
            let user = root.path().join("user");
            let xdg = root.path().join("xdg");
            let local = root.path().join("local");
            let roaming = root.path().join("roaming");
            let kiro = root.path().join("kiro");
            for directory in [&user, &xdg, &local, &roaming, &kiro] {
                fs::create_dir_all(directory)
                    .expect("the abrupt-process directory should be created");
            }
            let environment = FixtureEnvironment {
                values: BTreeMap::from([
                    ("HOME", user.clone().into_os_string()),
                    ("XDG_CONFIG_HOME", xdg.clone().into_os_string()),
                    ("USERPROFILE", user.into_os_string()),
                    ("LOCALAPPDATA", local.into_os_string()),
                    ("APPDATA", roaming.into_os_string()),
                    ("KIRO_HOME", kiro.into_os_string()),
                ]),
            };
            let paths = paths::ConfigurationPaths::resolve(&environment)
                .expect("the abrupt-process paths should resolve");
            Self {
                _root: root,
                environment,
                paths,
            }
        }

        fn command(&self, mode: &str) -> ProcessCommand {
            let mut command = ProcessCommand::new(
                std::env::current_exe().expect("the unit-test executable should resolve"),
            );
            command
                .arg("--ignored")
                .arg("--exact")
                .arg(CHILD_TEST)
                .arg("--nocapture")
                .env_clear()
                .env(CHILD_MODE, mode)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            for (name, value) in &self.environment.values {
                command.env(name, value);
            }
            command
        }

        fn write(&self, path: &Path, contents: &[u8]) {
            fs::create_dir_all(path.parent().expect("a fixture file should have a parent"))
                .expect("the fixture parent should be created");
            fs::write(path, contents).expect("the fixture file should be written");
        }
    }

    #[test]
    fn command_definition_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn abrupt_process_boundaries_recover_all_existing_file_command_journeys() {
        for journey in [Journey::Add, Journey::Sync, Journey::Restore] {
            let backup_states: &[bool] = if matches!(journey, Journey::Restore) {
                &[true]
            } else {
                &[false, true]
            };
            for &has_previous_backup in backup_states {
                for boundary in [
                    Boundary::PreparingJournal,
                    Boundary::PreparedJournal,
                    Boundary::TargetPublished,
                    Boundary::BackupPublished,
                ] {
                    exercise_abrupt_boundary(journey, has_previous_backup, boundary);
                }
            }
        }
    }

    fn exercise_abrupt_boundary(journey: Journey, has_previous_backup: bool, boundary: Boundary) {
        let fixture = AbruptFixture::new();
        let (target, original, previous_backup) =
            seed_journey(&fixture, journey, has_previous_backup);
        let backup = filesystem::backup_path(&target);
        let sentinel = target
            .parent()
            .expect("the affected file should have a parent")
            .join("fault-boundary.ready");
        let mut child = fixture.command(journey.initial_mode());
        child
            .env(FAULT_BOUNDARY, boundary.value())
            .env(FAULT_SENTINEL, &sentinel);
        let child = child
            .spawn()
            .expect("the abrupt-process child should start");
        let output = kill_after_sentinel(child, &sentinel);
        assert_redacted(&output);

        let committed_target =
            fs::read(&target).expect("the affected target should remain readable");
        if boundary.target_committed() {
            assert_ne!(
                committed_target, original,
                "the target commit point should be visible"
            );
        } else {
            assert_eq!(
                committed_target, original,
                "precommit death must preserve the target"
            );
        }
        if boundary.backup_committed() {
            assert_eq!(
                fs::read(&backup).expect("the committed backup should exist"),
                original,
                "backup publication must retain the exact original target"
            );
        } else {
            match previous_backup.as_deref() {
                Some(previous) => assert_eq!(
                    fs::read(&backup).expect("the prior backup should remain readable"),
                    previous,
                    "the prior backup must remain authoritative before backup publication"
                ),
                None => assert!(
                    !backup.exists(),
                    "a previously missing backup must remain absent"
                ),
            }
        }
        assert!(
            filesystem::replacement_transaction_path(&target).exists(),
            "the killed transaction should retain its recovery journal"
        );

        let read_output = fixture
            .command(journey.read_mode())
            .output()
            .expect("the lock-free refusal child should finish");
        assert!(
            read_output.status.success(),
            "the child assertion should pass"
        );
        assert_redacted(&read_output);
        assert_eq!(
            fs::read(&target).unwrap(),
            committed_target,
            "a lock-free refusal must not alter the affected target"
        );
        if boundary.backup_committed() {
            assert_eq!(fs::read(&backup).unwrap(), original);
        } else {
            match previous_backup.as_deref() {
                Some(previous) => assert_eq!(fs::read(&backup).unwrap(), previous),
                None => assert!(!backup.exists()),
            }
        }
        assert!(
            filesystem::replacement_transaction_path(&target).exists(),
            "a lock-free refusal must retain the recovery journal"
        );

        let recovery_output = fixture
            .command("recover")
            .output()
            .expect("the recovery child should finish");
        assert_redacted(&recovery_output);
        assert!(
            recovery_output.status.success(),
            "the child recovery assertion should pass for {journey:?}, previous backup \
             {has_previous_backup}, and {boundary:?}: {}",
            String::from_utf8_lossy(&recovery_output.stderr)
        );

        if boundary.target_committed() {
            assert_eq!(fs::read(&target).unwrap(), committed_target);
            let recovered_backup = fs::read(&backup).unwrap_or_else(|source| {
                panic!(
                    "the recovered backup should exist for {journey:?}, previous backup \
                     {has_previous_backup}, and {boundary:?}: {source}"
                )
            });
            assert_eq!(
                recovered_backup, original,
                "recovery should publish the original target for {journey:?}, previous backup \
                 {has_previous_backup}, and {boundary:?}"
            );
        } else {
            assert_eq!(fs::read(&target).unwrap(), original);
            match previous_backup.as_deref() {
                Some(previous) => assert_eq!(fs::read(&backup).unwrap(), previous),
                None => assert!(!backup.exists()),
            }
        }
        assert!(!filesystem::replacement_transaction_path(&target).exists());
        assert_no_owned_stages(target.parent().unwrap());
    }

    #[test]
    fn pending_recovery_waits_behind_the_same_root_operation_lock() {
        let fixture = AbruptFixture::new();
        let (target, original, _) = seed_journey(&fixture, Journey::Add, false);
        let sentinel = target.parent().unwrap().join("fault-boundary.ready");
        let mut child = fixture.command(Journey::Add.initial_mode());
        child
            .env(FAULT_BOUNDARY, Boundary::PreparedJournal.value())
            .env(FAULT_SENTINEL, &sentinel);
        let output = kill_after_sentinel(child.spawn().unwrap(), &sentinel);
        assert_redacted(&output);

        let guard = operation_lock::try_acquire(fixture.paths.operation_lock())
            .expect("the parent should hold the same canonical-root lock");
        let contended = fixture.command("recover-contended").output().unwrap();
        assert!(contended.status.success());
        assert_redacted(&contended);
        assert_eq!(fs::read(&target).unwrap(), original);
        assert!(filesystem::replacement_transaction_path(&target).exists());
        drop(guard);

        let recovered = fixture.command("recover").output().unwrap();
        assert!(recovered.status.success());
        assert_redacted(&recovered);
        assert_eq!(fs::read(&target).unwrap(), original);
        assert!(!filesystem::backup_path(&target).exists());
        assert!(!filesystem::replacement_transaction_path(&target).exists());
        assert_no_owned_stages(target.parent().unwrap());
    }

    fn seed_journey(
        fixture: &AbruptFixture,
        journey: Journey,
        has_previous_backup: bool,
    ) -> (PathBuf, Vec<u8>, Option<Vec<u8>>) {
        let canonical = fixture.paths.canonical_configuration();
        match journey {
            Journey::Add => {
                fixture.write(canonical, CANONICAL_B);
                let backup = filesystem::backup_path(canonical);
                if has_previous_backup {
                    fixture.write(&backup, CANONICAL_A);
                }
                (
                    canonical.to_owned(),
                    CANONICAL_B.to_vec(),
                    has_previous_backup.then(|| CANONICAL_A.to_vec()),
                )
            }
            Journey::Sync => {
                fixture.write(canonical, CANONICAL_C);
                let target =
                    super::claude_desktop::ClaudeDesktopAdapter::from_paths(&fixture.paths)
                        .configuration_path()
                        .to_owned();
                fixture.write(&target, CLAUDE_B);
                if has_previous_backup {
                    fixture.write(&filesystem::backup_path(&target), CLAUDE_A);
                }
                (
                    target,
                    CLAUDE_B.to_vec(),
                    has_previous_backup.then(|| CLAUDE_A.to_vec()),
                )
            }
            Journey::Restore => {
                fixture.write(canonical, CANONICAL_B);
                fixture.write(&filesystem::backup_path(canonical), CANONICAL_A);
                (
                    canonical.to_owned(),
                    CANONICAL_B.to_vec(),
                    Some(CANONICAL_A.to_vec()),
                )
            }
        }
    }

    fn kill_after_sentinel(mut child: std::process::Child, sentinel: &Path) -> Output {
        let deadline = Instant::now() + Duration::from_secs(10);
        while !sentinel.exists() {
            if let Some(status) = child
                .try_wait()
                .expect("the child status should be readable")
            {
                let output = child
                    .wait_with_output()
                    .expect("the exited child output should be readable");
                panic!(
                    "the abrupt-process child exited before its boundary ({status}): {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            assert!(
                Instant::now() < deadline,
                "the child did not reach its fault boundary"
            );
            thread::sleep(Duration::from_millis(10));
        }
        child
            .kill()
            .expect("the child should be terminated abruptly");
        child
            .wait_with_output()
            .expect("the terminated child output should be readable")
    }

    fn assert_redacted(output: &Output) {
        let combined = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        for private in [
            "older-private-command",
            "old-private-command",
            "new-private-command",
        ] {
            assert!(
                !combined.contains(private),
                "child output must remain redacted"
            );
        }
        assert!(
            !combined.contains("sha256"),
            "fingerprints must not reach diagnostics"
        );
    }

    fn assert_no_owned_stages(parent: &Path) {
        let remaining = fs::read_dir(parent)
            .expect("the target directory should be readable")
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|name| name.starts_with(".mcp-sync-") && name.ends_with(".tmp"))
            .collect::<Vec<_>>();
        assert!(
            remaining.is_empty(),
            "owned transaction stages remain: {remaining:?}"
        );
    }

    #[test]
    #[ignore = "spawned as an abrupt-process fixture by the parent regression"]
    fn abrupt_replacement_process_child() {
        let mode = std::env::var(CHILD_MODE).expect("the child mode should be provided");
        match mode.as_str() {
            "initial-add" => {
                let _ = run(Command::Add(AddCommand {
                    name: "fixture-server".to_owned(),
                    command: "new-private-command".to_owned(),
                    arguments: Vec::new(),
                    environment: Vec::new(),
                }));
                panic!("the add fixture should stop at a replacement boundary");
            }
            "initial-sync" => {
                let _ = run(Command::Sync(SyncCommand { dry_run: false }));
                panic!("the sync fixture should stop at a replacement boundary");
            }
            "initial-restore" => {
                let _ = run(Command::Restore(RestoreCommand {
                    configuration: RestoreTargetArgument::Canonical,
                    dry_run: false,
                }));
                panic!("the restore fixture should stop at a replacement boundary");
            }
            "read-add" => assert_pending_read(Command::List),
            "read-sync" => assert_pending_read(Command::Sync(SyncCommand { dry_run: true })),
            "read-restore" => assert_pending_read(Command::Restore(RestoreCommand {
                configuration: RestoreTargetArgument::Canonical,
                dry_run: true,
            })),
            "recover" => {
                let recovery_shape = pending_recovery_shape();
                match run(Command::Init) {
                    Err(super::ApplicationError::Init(
                        super::init::InitError::AlreadyInitialized { .. },
                    )) => {}
                    Err(super::ApplicationError::ReplacementRecovery(error)) => panic!(
                        "replacement recovery failed structurally: {}; {recovery_shape}",
                        mutation_error_shape(&error),
                    ),
                    Err(super::ApplicationError::ResolvePaths(_)) => {
                        panic!("post-recovery init returned a path-resolution error")
                    }
                    Err(super::ApplicationError::OperationLock(_)) => {
                        panic!("post-recovery init returned an operation-lock error")
                    }
                    Err(super::ApplicationError::Init(_)) => {
                        panic!("post-recovery init returned an unexpected init error")
                    }
                    Err(super::ApplicationError::Catalog(_)) => {
                        panic!("post-recovery init returned a catalog error")
                    }
                    Err(super::ApplicationError::Health(_)) => {
                        panic!("post-recovery init returned a health error")
                    }
                    Err(super::ApplicationError::Restore(_)) => {
                        panic!("post-recovery init returned a restore error")
                    }
                    Err(super::ApplicationError::Sync(_)) => {
                        panic!("post-recovery init returned a sync error")
                    }
                    Ok(_) => panic!("init should remain create-only after transaction recovery"),
                }
            }
            "recover-contended" => match run(Command::Init) {
                Ok(_) => panic!("same-root recovery must not bypass the operation lock"),
                Err(error) => {
                    let diagnostic = error.to_string();
                    assert!(diagnostic.contains("another mutating mcp-sync operation"));
                    assert!(!diagnostic.contains("private-command"));
                    assert!(!diagnostic.contains("sha256"));
                }
            },
            _ => panic!("unknown abrupt-process child mode"),
        }
    }

    fn assert_pending_read(command: Command) {
        match run(command) {
            Ok(_) => panic!("a lock-free command must not read an incomplete replacement"),
            Err(error) => {
                let diagnostic = error.to_string();
                assert!(diagnostic.contains("incomplete mcp-sync replacement"));
                assert!(!diagnostic.contains("private-command"));
                assert!(!diagnostic.contains("sha256"));
            }
        }
    }

    fn mutation_error_shape(error: &filesystem::FileMutationError) -> String {
        match error {
            filesystem::FileMutationError::Io(source) => format!(
                "io(operation={}, kind={:?})",
                source.operation(),
                source.kind()
            ),
            filesystem::FileMutationError::ConcurrentModification { .. } => {
                "concurrent-modification".to_owned()
            }
            filesystem::FileMutationError::UnsupportedFileType { kind, .. } => {
                format!("unsupported-file-type({kind})")
            }
            filesystem::FileMutationError::PendingReplacement { .. } => {
                "pending-replacement".to_owned()
            }
            filesystem::FileMutationError::InvalidReplacementTransaction { .. } => {
                "invalid-replacement-transaction".to_owned()
            }
            filesystem::FileMutationError::AmbiguousReplacementTransaction { .. } => {
                "ambiguous-replacement-transaction".to_owned()
            }
            filesystem::FileMutationError::RecoveryFailed {
                failure, recovery, ..
            } => format!(
                "recovery-failed(failure={}, recovery={})",
                mutation_error_shape(failure),
                mutation_error_shape(recovery)
            ),
        }
    }

    fn pending_recovery_shape() -> String {
        let Ok(configuration_paths) =
            paths::ConfigurationPaths::resolve(&paths::ProcessEnvironment)
        else {
            return "paths=unresolved".to_owned();
        };
        super::managed_configuration_paths(&configuration_paths)
            .into_iter()
            .find(|path| filesystem::replacement_transaction_path(path).exists())
            .map_or_else(
                || "journal=missing".to_owned(),
                |path| filesystem::replacement_recovery_test_shape(&path),
            )
    }
}
