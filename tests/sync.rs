mod support;

use assert_cmd::Command as AssertCommand;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;
use std::time::Duration;
use support::SyntheticHome;

const COMMAND_TIMEOUT: Duration = Duration::from_secs(5);
const CLAUDE_CURRENT: &[u8] = include_bytes!("fixtures/claude-desktop/current.json");
const CANONICAL_DESIRED: &[u8] = include_bytes!("fixtures/claude-desktop/desired.json");
const CURSOR_CURRENT: &[u8] = include_bytes!("fixtures/cursor/current.json");
const PROJECT_CURSOR: &[u8] = include_bytes!("fixtures/cursor/project.json");

fn sync_command(home: &SyntheticHome, dry_run: bool) -> AssertCommand {
    let mut command = AssertCommand::from_std(home.command());
    command.arg("sync");
    if dry_run {
        command.arg("--dry-run");
    }
    command.timeout(COMMAND_TIMEOUT);
    command
}

fn run_success(mut command: AssertCommand) -> Output {
    let assertion = command.assert();
    let output = assertion.get_output().clone();
    assert!(output.status.success(), "the CLI command should succeed");
    assert!(output.stderr.is_empty(), "success should not write stderr");
    output
}

fn run_failure(mut command: AssertCommand) -> Output {
    let assertion = command.assert();
    let output = assertion.get_output().clone();
    assert!(!output.status.success(), "the CLI command should fail");
    assert!(output.stdout.is_empty(), "failure should not write stdout");
    output
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be UTF-8")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr should be UTF-8")
}

fn backup_path(path: &Path) -> PathBuf {
    let mut path = path.as_os_str().to_owned();
    path.push(".bak");
    PathBuf::from(path)
}

struct ExistingJourney {
    canonical: Vec<u8>,
    claude: Vec<u8>,
    cursor: Vec<u8>,
    project_path: PathBuf,
    process_marker: PathBuf,
    process_command: String,
}

impl ExistingJourney {
    fn private_values(&self) -> Vec<&str> {
        vec![
            &self.process_command,
            "/synthetic/bin/unchanged",
            "/synthetic/bin/old",
            "/synthetic/bin/new",
            "/synthetic/bin/local",
            "--mode",
            "--old",
            "--new",
            "fixture-unchanged-secret",
            "fixture-old-secret",
            "fixture-new-secret",
            "fixture-added-secret",
            "fixture-added-environment-secret",
            "fixture-remove-secret",
            "fixture-local-secret",
            "fixture-cursor-old-secret",
            "fixture-cursor-remove-secret",
            "fixture-cursor-local-secret",
            "Bearer fixture-cursor-remote-secret",
            "https://mcp.example.invalid/api",
            "/synthetic/workspace/preserved",
            "/synthetic/env/preserved.env",
        ]
    }
}

fn prepare_existing_journey(home: &SyntheticHome) -> ExistingJourney {
    let process_marker = home.root().join("configured-server-was-started");
    let process_command_path = home.root().join("synthetic-private-mcp-server");
    home.write_file(
        &process_command_path,
        format!(
            "#!/bin/sh\nprintf started > '{}'\n",
            process_marker.display()
        ),
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        fs::set_permissions(&process_command_path, fs::Permissions::from_mode(0o700))
            .expect("the process sentinel should be executable");
    }

    let process_command = process_command_path.to_string_lossy().into_owned();
    let mut canonical: Value = serde_json::from_slice(CANONICAL_DESIRED)
        .expect("the desired canonical fixture should parse");
    canonical["servers"]["added"]["command"] = Value::String(process_command.clone());
    let mut canonical =
        serde_json::to_vec_pretty(&canonical).expect("the canonical fixture should serialize");
    canonical.push(b'\n');

    let claude = CLAUDE_CURRENT.to_vec();
    let cursor = String::from_utf8(CURSOR_CURRENT.to_vec())
        .expect("the Cursor fixture should be UTF-8")
        .replace(
            "fixture-cursor-unchanged-secret",
            "fixture-unchanged-secret",
        )
        .into_bytes();
    let project_path = home.user_root().join("workspace/.cursor/mcp.json");

    home.write_file(&home.canonical_configuration(), &canonical);
    home.write_file(&home.claude_desktop_configuration(), &claude);
    home.write_file(&home.cursor_configuration(), &cursor);
    home.write_file(&project_path, PROJECT_CURSOR);

    ExistingJourney {
        canonical,
        claude,
        cursor,
        project_path,
        process_marker,
        process_command,
    }
}

fn assert_output_omits(output: &str, private_values: &[&str]) {
    for private in private_values {
        assert!(
            !output.contains(private),
            "sync output must omit commands, arguments, environment values, and unowned native values"
        );
    }
}

fn assert_no_temporary_files(path: &Path) {
    let parent = path.parent().expect("a target path has a parent");
    if !parent.exists() {
        return;
    }
    let has_temporary = fs::read_dir(parent)
        .expect("the target directory should be readable")
        .filter_map(Result::ok)
        .any(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with(".mcp-sync-")
        });
    assert!(!has_temporary, "temporary files should be cleaned up");
}

fn assert_native_result(home: &SyntheticHome, process_command: &str) {
    let claude: Value = serde_json::from_slice(
        &fs::read(home.claude_desktop_configuration())
            .expect("Claude Desktop output should be readable"),
    )
    .expect("Claude Desktop output should remain valid JSON");
    let cursor: Value = serde_json::from_slice(
        &fs::read(home.cursor_configuration()).expect("Cursor output should be readable"),
    )
    .expect("Cursor output should remain valid JSON");

    for target in [&claude, &cursor] {
        assert_eq!(target["mcpServers"]["added"]["command"], process_command);
        assert_eq!(
            target["mcpServers"]["updated"]["command"],
            "/synthetic/bin/new"
        );
        assert_eq!(target["mcpServers"]["updated"]["args"][1], "two");
        assert_eq!(
            target["mcpServers"]["updated"]["env"]["ROTATE"],
            "fixture-new-secret"
        );
    }

    assert_eq!(
        claude["mcpServers"]["target-only"]["env"]["LOCAL_TOKEN"],
        "fixture-local-secret"
    );
    assert_eq!(
        cursor["mcpServers"]["target-only"]["env"]["LOCAL_TOKEN"],
        "fixture-cursor-local-secret"
    );

    assert_eq!(
        claude["mcpServers"]["updated"]["cwd"],
        "/synthetic/workspace/preserved"
    );
    assert_eq!(
        claude["futureTopLevel"]["preciseNumber"].to_string(),
        "1234567890123456789012345678901234567890"
    );
    assert_eq!(
        cursor["mcpServers"]["updated"]["envFile"],
        "/synthetic/env/preserved.env"
    );
    assert_eq!(cursor["mcpServers"]["updated"]["type"], "stdio");
    assert_eq!(
        cursor["mcpServers"]["remote-only"]["headers"]["Authorization"],
        "Bearer fixture-cursor-remote-secret"
    );
    assert_eq!(
        cursor["futureTopLevel"]["preciseNumber"].to_string(),
        "1234567890123456789012345678901234567890"
    );
}

#[test]
fn dry_run_apply_and_repeat_no_op_share_one_redacted_per_target_contract() {
    let home = SyntheticHome::new();
    let fixture = prepare_existing_journey(&home);
    let claude_backup = backup_path(&home.claude_desktop_configuration());
    let cursor_backup = backup_path(&home.cursor_configuration());

    let dry_output = stdout(&run_success(sync_command(&home, true)));

    assert!(dry_output.starts_with("Dry run validated 2 targets; no files changed.\n"));
    assert!(dry_output.contains("Claude Desktop: would update with recoverable backup"));
    assert!(dry_output.contains("Cursor: would update with recoverable backup"));
    assert_eq!(
        dry_output
            .matches("Claude Desktop: would update with recoverable backup")
            .count(),
        1
    );
    assert!(dry_output.contains(
        "(add: 1; update: 1; unchanged: 1; drift preserved: 1; unmanaged preserved: 0)."
    ));
    assert!(dry_output.contains(
        "(add: 1; update: 1; unchanged: 1; drift preserved: 1; unmanaged preserved: 1)."
    ));
    assert!(dry_output.contains("add \"added\" (arguments: 1; environment keys: \"ADD_TOKEN\")"));
    assert!(dry_output.contains("update \"updated\" (command; arguments 1 -> 2"));
    assert!(dry_output.contains("environment keys updated \"ROTATE\""));
    assert!(dry_output.contains("preserve target-only \"target-only\""));
    assert!(dry_output.contains("preserve unmanaged \"remote-only\""));
    assert_output_omits(&dry_output, &fixture.private_values());
    assert_eq!(
        fs::read(home.canonical_configuration()).unwrap(),
        fixture.canonical
    );
    assert_eq!(
        fs::read(home.claude_desktop_configuration()).unwrap(),
        fixture.claude
    );
    assert_eq!(
        fs::read(home.cursor_configuration()).unwrap(),
        fixture.cursor
    );
    assert_eq!(fs::read(&fixture.project_path).unwrap(), PROJECT_CURSOR);
    assert!(!claude_backup.exists());
    assert!(!cursor_backup.exists());
    assert!(!fixture.process_marker.exists());

    let apply_output = stdout(&run_success(sync_command(&home, false)));

    assert!(apply_output.starts_with("Sync completed for 2 targets.\n"));
    assert!(apply_output.contains("Claude Desktop: updated with recoverable backup"));
    assert!(apply_output.contains("Cursor: updated with recoverable backup"));
    assert_output_omits(&apply_output, &fixture.private_values());
    assert_eq!(fs::read(&claude_backup).unwrap(), fixture.claude);
    assert_eq!(fs::read(&cursor_backup).unwrap(), fixture.cursor);
    assert_eq!(
        fs::read(home.canonical_configuration()).unwrap(),
        fixture.canonical
    );
    assert_eq!(fs::read(&fixture.project_path).unwrap(), PROJECT_CURSOR);
    assert!(!fixture.process_marker.exists());
    assert_native_result(&home, &fixture.process_command);
    assert_no_temporary_files(&home.claude_desktop_configuration());
    assert_no_temporary_files(&home.cursor_configuration());

    let claude_after_apply = fs::read(home.claude_desktop_configuration()).unwrap();
    let cursor_after_apply = fs::read(home.cursor_configuration()).unwrap();
    let claude_backup_after_apply = fs::read(&claude_backup).unwrap();
    let cursor_backup_after_apply = fs::read(&cursor_backup).unwrap();
    let no_op_output = stdout(&run_success(sync_command(&home, false)));

    assert_eq!(
        no_op_output
            .matches("unchanged; no write or backup")
            .count(),
        2
    );
    assert_eq!(
        no_op_output
            .matches(
                "(add: 0; update: 0; unchanged: 3; drift preserved: 1; unmanaged preserved: 0)."
            )
            .count(),
        1
    );
    assert!(no_op_output.contains(
        "(add: 0; update: 0; unchanged: 3; drift preserved: 1; unmanaged preserved: 1)."
    ));
    assert_output_omits(&no_op_output, &fixture.private_values());
    assert_eq!(
        fs::read(home.claude_desktop_configuration()).unwrap(),
        claude_after_apply
    );
    assert_eq!(
        fs::read(home.cursor_configuration()).unwrap(),
        cursor_after_apply
    );
    assert_eq!(fs::read(&claude_backup).unwrap(), claude_backup_after_apply);
    assert_eq!(fs::read(&cursor_backup).unwrap(), cursor_backup_after_apply);
    assert_eq!(fs::read(&fixture.project_path).unwrap(), PROJECT_CURSOR);
    assert!(!fixture.process_marker.exists());
}

#[test]
fn a_real_second_target_failure_restores_the_first_target_and_its_prior_backup() {
    let home = SyntheticHome::new();
    let fixture = prepare_existing_journey(&home);
    let claude_backup = backup_path(&home.claude_desktop_configuration());
    let previous_claude_backup = b"older private Claude backup bytes\n";
    home.write_file(&claude_backup, previous_claude_backup);
    let cursor_backup = backup_path(&home.cursor_configuration());
    fs::create_dir(&cursor_backup).expect("the blocking Cursor backup should be created");

    let output = run_failure(sync_command(&home, false));
    let diagnostic = stderr(&output);

    assert!(diagnostic.starts_with("error: sync transaction failed while applying Cursor:"));
    assert!(diagnostic.contains("Claude Desktop: rolled back after update"));
    assert!(diagnostic.contains("Cursor: update failed: refusing to replace directory"));
    assert!(diagnostic.contains("Per-target outcomes:"));
    assert_output_omits(&diagnostic, &fixture.private_values());
    assert!(!diagnostic.contains("older private Claude backup bytes"));
    assert_eq!(
        fs::read(home.claude_desktop_configuration()).unwrap(),
        fixture.claude
    );
    assert_eq!(fs::read(&claude_backup).unwrap(), previous_claude_backup);
    assert_eq!(
        fs::read(home.cursor_configuration()).unwrap(),
        fixture.cursor
    );
    assert!(cursor_backup.is_dir());
    assert_eq!(
        fs::read(home.canonical_configuration()).unwrap(),
        fixture.canonical
    );
    assert_eq!(fs::read(&fixture.project_path).unwrap(), PROJECT_CURSOR);
    assert!(!fixture.process_marker.exists());
    assert_no_temporary_files(&home.claude_desktop_configuration());
    assert_no_temporary_files(&home.cursor_configuration());
}

#[test]
fn a_created_first_target_is_removed_when_the_second_target_fails() {
    let home = SyntheticHome::new();
    let fixture = prepare_existing_journey(&home);
    fs::remove_file(home.claude_desktop_configuration())
        .expect("the Claude Desktop target should become missing");
    let cursor_backup = backup_path(&home.cursor_configuration());
    fs::create_dir(&cursor_backup).expect("the blocking Cursor backup should be created");

    let output = run_failure(sync_command(&home, false));
    let diagnostic = stderr(&output);

    assert!(diagnostic.contains("Claude Desktop: rolled back after creation"));
    assert!(diagnostic.contains("Cursor: update failed"));
    assert_output_omits(&diagnostic, &fixture.private_values());
    assert!(!home.claude_desktop_configuration().exists());
    assert!(!backup_path(&home.claude_desktop_configuration()).exists());
    assert_eq!(
        fs::read(home.cursor_configuration()).unwrap(),
        fixture.cursor
    );
    assert!(cursor_backup.is_dir());
    assert_eq!(fs::read(&fixture.project_path).unwrap(), PROJECT_CURSOR);
    assert!(!fixture.process_marker.exists());
    assert_no_temporary_files(&home.claude_desktop_configuration());
    assert_no_temporary_files(&home.cursor_configuration());
}

#[test]
fn malformed_later_target_state_stops_before_any_apply_mutation() {
    let home = SyntheticHome::new();
    let fixture = prepare_existing_journey(&home);
    let malformed = b"{\"mcpServers\":{\"bad\":{\"command\":\"private-malformed-command\"";
    home.write_file(&home.cursor_configuration(), malformed);

    let output = run_failure(sync_command(&home, false));
    let diagnostic = stderr(&output);

    assert!(diagnostic.starts_with("error: cannot plan Cursor sync: invalid Cursor JSON:"));
    assert!(diagnostic.ends_with("; no target files were changed\n"));
    assert!(!diagnostic.contains("private-malformed-command"));
    assert_output_omits(&diagnostic, &fixture.private_values());
    assert_eq!(
        fs::read(home.claude_desktop_configuration()).unwrap(),
        fixture.claude
    );
    assert_eq!(fs::read(home.cursor_configuration()).unwrap(), malformed);
    assert!(!backup_path(&home.claude_desktop_configuration()).exists());
    assert!(!backup_path(&home.cursor_configuration()).exists());
    assert_eq!(fs::read(&fixture.project_path).unwrap(), PROJECT_CURSOR);
    assert!(!fixture.process_marker.exists());
}

#[test]
fn missing_or_malformed_canonical_state_fails_before_native_discovery() {
    let home = SyntheticHome::new();
    let claude = b"Claude target must not be parsed before canonical validation\n";
    let cursor = b"Cursor target must not be parsed before canonical validation\n";
    home.write_file(&home.claude_desktop_configuration(), claude);
    home.write_file(&home.cursor_configuration(), cursor);

    let missing = stderr(&run_failure(sync_command(&home, true)));
    assert!(missing.contains("canonical configuration does not exist"));
    assert!(missing.contains("run `mcp-sync init` first"));

    let malformed = b"{\"schemaVersion\":1,\"servers\":{\"private\":\"malformed-private-value\"";
    home.write_file(&home.canonical_configuration(), malformed);
    let invalid = stderr(&run_failure(sync_command(&home, false)));

    assert!(invalid.contains("canonical configuration"));
    assert!(invalid.contains("is invalid: invalid JSON:"));
    assert!(!invalid.contains("malformed-private-value"));
    assert_eq!(
        fs::read(home.claude_desktop_configuration()).unwrap(),
        claude
    );
    assert_eq!(fs::read(home.cursor_configuration()).unwrap(), cursor);
    assert!(!backup_path(&home.claude_desktop_configuration()).exists());
    assert!(!backup_path(&home.cursor_configuration()).exists());
}

#[test]
fn an_unmanaged_cursor_name_collision_fails_the_complete_plan_before_apply() {
    let home = SyntheticHome::new();
    let fixture = prepare_existing_journey(&home);
    let mut canonical: Value =
        serde_json::from_slice(&fixture.canonical).expect("canonical fixture should parse");
    canonical["servers"]["remote-only"] = canonical["servers"]["added"].clone();
    let mut canonical =
        serde_json::to_vec_pretty(&canonical).expect("collision fixture should serialize");
    canonical.push(b'\n');
    home.write_file(&home.canonical_configuration(), &canonical);

    let output = run_failure(sync_command(&home, false));
    let diagnostic = stderr(&output);

    assert!(diagnostic.contains(
        "cannot render the validated Cursor sync plan: desired local server \"remote-only\" collides with an unmanaged Cursor server"
    ));
    assert!(diagnostic.ends_with("; no target files were changed\n"));
    assert_output_omits(&diagnostic, &fixture.private_values());
    assert_eq!(
        fs::read(home.claude_desktop_configuration()).unwrap(),
        fixture.claude
    );
    assert_eq!(
        fs::read(home.cursor_configuration()).unwrap(),
        fixture.cursor
    );
    assert!(!backup_path(&home.claude_desktop_configuration()).exists());
    assert!(!backup_path(&home.cursor_configuration()).exists());
    assert_eq!(fs::read(&fixture.project_path).unwrap(), PROJECT_CURSOR);
    assert!(!fixture.process_marker.exists());
}

#[cfg(unix)]
#[test]
fn a_second_target_permission_failure_rolls_back_the_first_target() {
    use std::os::unix::fs::PermissionsExt as _;

    let home = SyntheticHome::new();
    let fixture = prepare_existing_journey(&home);
    let cursor_directory = home
        .cursor_configuration()
        .parent()
        .expect("the Cursor path has a parent")
        .to_owned();
    fs::set_permissions(&cursor_directory, fs::Permissions::from_mode(0o500))
        .expect("the Cursor directory should become read-only");
    let permission_probe = cursor_directory.join("permission-probe");
    if fs::write(&permission_probe, b"probe").is_ok() {
        fs::remove_file(permission_probe).expect("the permission probe should be removed");
        fs::set_permissions(&cursor_directory, fs::Permissions::from_mode(0o700))
            .expect("the Cursor directory permissions should be restored");
        return;
    }

    let output = run_failure(sync_command(&home, false));
    fs::set_permissions(&cursor_directory, fs::Permissions::from_mode(0o700))
        .expect("the Cursor directory permissions should be restored");
    let diagnostic = stderr(&output);

    assert!(diagnostic.starts_with("error: sync transaction failed while applying Cursor:"));
    assert!(diagnostic.contains("Claude Desktop: rolled back after update"));
    assert!(diagnostic.contains("Cursor: update failed"));
    assert!(diagnostic.contains("Permission denied"));
    assert_output_omits(&diagnostic, &fixture.private_values());
    assert_eq!(
        fs::read(home.claude_desktop_configuration()).unwrap(),
        fixture.claude
    );
    assert_eq!(
        fs::read(home.cursor_configuration()).unwrap(),
        fixture.cursor
    );
    assert!(!backup_path(&home.claude_desktop_configuration()).exists());
    assert!(!backup_path(&home.cursor_configuration()).exists());
    assert_eq!(fs::read(&fixture.project_path).unwrap(), PROJECT_CURSOR);
    assert!(!fixture.process_marker.exists());
}
