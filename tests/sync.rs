mod support;

use assert_cmd::Command as AssertCommand;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;
use std::time::Duration;
use support::{SyntheticHome, assert_file_matches};

const COMMAND_TIMEOUT: Duration = Duration::from_secs(5);
const CLAUDE_CURRENT: &[u8] = include_bytes!("fixtures/claude-desktop/current.json");
const CANONICAL_DESIRED: &[u8] = include_bytes!("fixtures/claude-desktop/desired.json");
const CURSOR_CURRENT: &[u8] = include_bytes!("fixtures/cursor/current.json");
const WINDSURF_CURRENT: &[u8] = include_bytes!("fixtures/windsurf/current.json");
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
    windsurf: Vec<u8>,
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
            "fixture-windsurf-old-secret",
            "fixture-windsurf-remove-secret",
            "fixture-windsurf-local-secret",
            "Bearer fixture-windsurf-remote-secret",
            "https://windsurf.example.invalid/mcp",
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
    let windsurf = String::from_utf8(WINDSURF_CURRENT.to_vec())
        .expect("the Windsurf fixture should be UTF-8")
        .replace(
            "fixture-windsurf-unchanged-secret",
            "fixture-unchanged-secret",
        )
        .replace("\"remote-only\"", "\"windsurf-remote-only\"")
        .into_bytes();
    let project_path = home.user_root().join("workspace/.cursor/mcp.json");

    home.write_file(&home.canonical_configuration(), &canonical);
    home.write_file(&home.claude_desktop_configuration(), &claude);
    home.write_file(&home.cursor_configuration(), &cursor);
    home.write_file(&home.windsurf_configuration(), &windsurf);
    home.write_file(&project_path, PROJECT_CURSOR);

    ExistingJourney {
        canonical,
        claude,
        cursor,
        windsurf,
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
    let windsurf: Value = serde_json::from_slice(
        &fs::read(home.windsurf_configuration()).expect("Windsurf output should be readable"),
    )
    .expect("Windsurf output should remain valid JSON");

    for target in [&claude, &cursor, &windsurf] {
        assert!(
            target["mcpServers"]["added"]["command"].as_str() == Some(process_command)
                && target["mcpServers"]["updated"]["command"].as_str()
                    == Some("/synthetic/bin/new")
                && target["mcpServers"]["updated"]["args"][1].as_str() == Some("two")
                && target["mcpServers"]["updated"]["env"]["ROTATE"].as_str()
                    == Some("fixture-new-secret"),
            "managed process values should match canonical state"
        );
    }

    assert!(
        claude["mcpServers"]["target-only"]["env"]["LOCAL_TOKEN"].as_str()
            == Some("fixture-local-secret")
            && cursor["mcpServers"]["target-only"]["env"]["LOCAL_TOKEN"].as_str()
                == Some("fixture-cursor-local-secret")
            && claude["mcpServers"]["updated"]["cwd"].as_str()
                == Some("/synthetic/workspace/preserved")
            && cursor["mcpServers"]["updated"]["envFile"].as_str()
                == Some("/synthetic/env/preserved.env")
            && cursor["mcpServers"]["updated"]["type"].as_str() == Some("stdio")
            && cursor["mcpServers"]["remote-only"]["headers"]["Authorization"].as_str()
                == Some("Bearer fixture-cursor-remote-secret")
            && windsurf["mcpServers"]["target-only"]["env"]["LOCAL_TOKEN"].as_str()
                == Some("fixture-windsurf-local-secret")
            && windsurf["mcpServers"]["updated"]["disabledTools"][0].as_str()
                == Some("fixture_update_tool")
            && windsurf["mcpServers"]["windsurf-remote-only"]["headers"]["Authorization"].as_str()
                == Some("Bearer fixture-windsurf-remote-secret")
            && windsurf["mcpServers"]["windsurf-remote-only"]["serverUrl"].as_str()
                == Some("https://windsurf.example.invalid/mcp"),
        "unowned, drift, and unmanaged native values should be preserved"
    );
    assert!(
        claude["futureTopLevel"]["preciseNumber"]
            .as_number()
            .is_some_and(|number| number.as_str() == "1234567890123456789012345678901234567890")
            && cursor["futureTopLevel"]["preciseNumber"]
                .as_number()
                .is_some_and(|number| {
                    number.as_str() == "1234567890123456789012345678901234567890"
                })
            && windsurf["futureTopLevel"]["preciseNumber"]
                .as_number()
                .is_some_and(|number| {
                    number.as_str() == "9876543210987654321098765432109876543210"
                }),
        "arbitrary-precision unowned numbers should be preserved"
    );
}

#[test]
fn dry_run_apply_and_repeat_no_op_share_one_redacted_per_target_contract() {
    let home = SyntheticHome::new();
    let fixture = prepare_existing_journey(&home);
    let claude_backup = backup_path(&home.claude_desktop_configuration());
    let cursor_backup = backup_path(&home.cursor_configuration());
    let windsurf_backup = backup_path(&home.windsurf_configuration());

    let dry_output = stdout(&run_success(sync_command(&home, true)));

    assert!(dry_output.starts_with("Dry run validated 3 targets; no files changed.\n"));
    assert!(dry_output.contains("Claude Desktop: would update with recoverable backup"));
    assert!(dry_output.contains("Cursor: would update with recoverable backup"));
    assert!(dry_output.contains("Windsurf: would update with recoverable backup"));
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
    assert!(dry_output.contains("preserve unmanaged \"windsurf-remote-only\""));
    assert_output_omits(&dry_output, &fixture.private_values());
    assert_file_matches(
        &home.canonical_configuration(),
        &fixture.canonical,
        "dry-run should preserve canonical bytes",
    );
    assert_file_matches(
        &home.claude_desktop_configuration(),
        &fixture.claude,
        "dry-run should preserve Claude bytes",
    );
    assert_file_matches(
        &home.cursor_configuration(),
        &fixture.cursor,
        "dry-run should preserve Cursor bytes",
    );
    assert_file_matches(
        &home.windsurf_configuration(),
        &fixture.windsurf,
        "dry-run should preserve Windsurf bytes",
    );
    assert_file_matches(
        &fixture.project_path,
        PROJECT_CURSOR,
        "dry-run should preserve project Cursor bytes",
    );
    assert!(!claude_backup.exists());
    assert!(!cursor_backup.exists());
    assert!(!windsurf_backup.exists());
    assert!(!fixture.process_marker.exists());

    let apply_output = stdout(&run_success(sync_command(&home, false)));

    assert!(apply_output.starts_with("Sync completed for 3 targets.\n"));
    assert!(apply_output.contains("Claude Desktop: updated with recoverable backup"));
    assert!(apply_output.contains("Cursor: updated with recoverable backup"));
    assert!(apply_output.contains("Windsurf: updated with recoverable backup"));
    assert_output_omits(&apply_output, &fixture.private_values());
    assert_file_matches(
        &claude_backup,
        &fixture.claude,
        "Claude backup should contain exact prior bytes",
    );
    assert_file_matches(
        &cursor_backup,
        &fixture.cursor,
        "Cursor backup should contain exact prior bytes",
    );
    assert_file_matches(
        &windsurf_backup,
        &fixture.windsurf,
        "Windsurf backup should contain exact prior bytes",
    );
    assert_file_matches(
        &home.canonical_configuration(),
        &fixture.canonical,
        "apply should preserve canonical bytes",
    );
    assert_file_matches(
        &fixture.project_path,
        PROJECT_CURSOR,
        "apply should preserve project Cursor bytes",
    );
    assert!(!fixture.process_marker.exists());
    assert_native_result(&home, &fixture.process_command);
    assert_no_temporary_files(&home.claude_desktop_configuration());
    assert_no_temporary_files(&home.cursor_configuration());
    assert_no_temporary_files(&home.windsurf_configuration());

    let claude_after_apply = fs::read(home.claude_desktop_configuration()).unwrap();
    let cursor_after_apply = fs::read(home.cursor_configuration()).unwrap();
    let windsurf_after_apply = fs::read(home.windsurf_configuration()).unwrap();
    let claude_backup_after_apply = fs::read(&claude_backup).unwrap();
    let cursor_backup_after_apply = fs::read(&cursor_backup).unwrap();
    let windsurf_backup_after_apply = fs::read(&windsurf_backup).unwrap();
    let no_op_output = stdout(&run_success(sync_command(&home, false)));

    assert_eq!(
        no_op_output
            .matches("unchanged; no write or backup")
            .count(),
        3
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
    assert_file_matches(
        &home.claude_desktop_configuration(),
        &claude_after_apply,
        "no-op should preserve Claude bytes",
    );
    assert_file_matches(
        &home.cursor_configuration(),
        &cursor_after_apply,
        "no-op should preserve Cursor bytes",
    );
    assert_file_matches(
        &home.windsurf_configuration(),
        &windsurf_after_apply,
        "no-op should preserve Windsurf bytes",
    );
    assert_file_matches(
        &claude_backup,
        &claude_backup_after_apply,
        "no-op should preserve Claude backup bytes",
    );
    assert_file_matches(
        &cursor_backup,
        &cursor_backup_after_apply,
        "no-op should preserve Cursor backup bytes",
    );
    assert_file_matches(
        &windsurf_backup,
        &windsurf_backup_after_apply,
        "no-op should preserve Windsurf backup bytes",
    );
    assert_file_matches(
        &fixture.project_path,
        PROJECT_CURSOR,
        "no-op should preserve project Cursor bytes",
    );
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
    assert!(diagnostic.contains("Windsurf: not attempted after an earlier failure"));
    assert!(diagnostic.contains("Per-target outcomes:"));
    assert_output_omits(&diagnostic, &fixture.private_values());
    assert!(!diagnostic.contains("older private Claude backup bytes"));
    assert_file_matches(
        &home.claude_desktop_configuration(),
        &fixture.claude,
        "rollback should restore Claude bytes",
    );
    assert_file_matches(
        &claude_backup,
        previous_claude_backup,
        "rollback should restore the previous Claude backup",
    );
    assert_file_matches(
        &home.cursor_configuration(),
        &fixture.cursor,
        "failed Cursor target should remain unchanged",
    );
    assert!(cursor_backup.is_dir());
    assert_file_matches(
        &home.windsurf_configuration(),
        &fixture.windsurf,
        "a second-target failure should preserve Windsurf bytes",
    );
    assert!(!backup_path(&home.windsurf_configuration()).exists());
    assert_file_matches(
        &home.canonical_configuration(),
        &fixture.canonical,
        "failed sync should preserve canonical bytes",
    );
    assert_file_matches(
        &fixture.project_path,
        PROJECT_CURSOR,
        "failed sync should preserve project Cursor bytes",
    );
    assert!(!fixture.process_marker.exists());
    assert_no_temporary_files(&home.claude_desktop_configuration());
    assert_no_temporary_files(&home.cursor_configuration());
    assert_no_temporary_files(&home.windsurf_configuration());
}

#[test]
fn a_real_third_target_failure_restores_both_prior_targets_and_backups() {
    let home = SyntheticHome::new();
    let fixture = prepare_existing_journey(&home);
    let claude_backup = backup_path(&home.claude_desktop_configuration());
    let cursor_backup = backup_path(&home.cursor_configuration());
    let windsurf_backup = backup_path(&home.windsurf_configuration());
    let previous_claude_backup = b"older private Claude backup bytes\n";
    let previous_cursor_backup = b"older private Cursor backup bytes\n";
    home.write_file(&claude_backup, previous_claude_backup);
    home.write_file(&cursor_backup, previous_cursor_backup);
    fs::create_dir(&windsurf_backup).expect("the blocking Windsurf backup should be created");

    let output = run_failure(sync_command(&home, false));
    let diagnostic = stderr(&output);

    assert!(diagnostic.starts_with("error: sync transaction failed while applying Windsurf:"));
    assert!(diagnostic.contains("Claude Desktop: rolled back after update"));
    assert!(diagnostic.contains("Cursor: rolled back after update"));
    assert!(diagnostic.contains("Windsurf: update failed: refusing to replace directory"));
    assert_output_omits(&diagnostic, &fixture.private_values());
    assert!(!diagnostic.contains("older private Claude backup bytes"));
    assert!(!diagnostic.contains("older private Cursor backup bytes"));
    assert_file_matches(
        &home.claude_desktop_configuration(),
        &fixture.claude,
        "third-target rollback should restore Claude bytes",
    );
    assert_file_matches(
        &home.cursor_configuration(),
        &fixture.cursor,
        "third-target rollback should restore Cursor bytes",
    );
    assert_file_matches(
        &home.windsurf_configuration(),
        &fixture.windsurf,
        "failed Windsurf target should remain unchanged",
    );
    assert_file_matches(
        &claude_backup,
        previous_claude_backup,
        "third-target rollback should restore the previous Claude backup",
    );
    assert_file_matches(
        &cursor_backup,
        previous_cursor_backup,
        "third-target rollback should restore the previous Cursor backup",
    );
    assert!(windsurf_backup.is_dir());
    assert_file_matches(
        &home.canonical_configuration(),
        &fixture.canonical,
        "failed three-target sync should preserve canonical bytes",
    );
    assert_file_matches(
        &fixture.project_path,
        PROJECT_CURSOR,
        "failed three-target sync should preserve project Cursor bytes",
    );
    assert!(!fixture.process_marker.exists());
    for target in [
        home.claude_desktop_configuration(),
        home.cursor_configuration(),
        home.windsurf_configuration(),
    ] {
        assert_no_temporary_files(&target);
    }
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
    assert!(diagnostic.contains("Windsurf: not attempted after an earlier failure"));
    assert_output_omits(&diagnostic, &fixture.private_values());
    assert!(!home.claude_desktop_configuration().exists());
    assert!(!backup_path(&home.claude_desktop_configuration()).exists());
    assert_file_matches(
        &home.cursor_configuration(),
        &fixture.cursor,
        "creation rollback should preserve Cursor bytes",
    );
    assert!(cursor_backup.is_dir());
    assert_file_matches(
        &home.windsurf_configuration(),
        &fixture.windsurf,
        "creation rollback should preserve Windsurf bytes",
    );
    assert!(!backup_path(&home.windsurf_configuration()).exists());
    assert_file_matches(
        &fixture.project_path,
        PROJECT_CURSOR,
        "creation rollback should preserve project Cursor bytes",
    );
    assert!(!fixture.process_marker.exists());
    assert_no_temporary_files(&home.claude_desktop_configuration());
    assert_no_temporary_files(&home.cursor_configuration());
    assert_no_temporary_files(&home.windsurf_configuration());
}

#[test]
fn malformed_final_windsurf_state_stops_before_any_apply_mutation() {
    let home = SyntheticHome::new();
    let fixture = prepare_existing_journey(&home);
    let malformed = b"{\"mcpServers\":{\"bad\":{\"command\":\"private-malformed-command\"";
    home.write_file(&home.windsurf_configuration(), malformed);

    let output = run_failure(sync_command(&home, false));
    let diagnostic = stderr(&output);

    assert!(diagnostic.starts_with("error: cannot plan Windsurf sync: invalid Windsurf JSON:"));
    assert!(diagnostic.ends_with("; no target files were changed\n"));
    assert!(!diagnostic.contains("private-malformed-command"));
    assert_output_omits(&diagnostic, &fixture.private_values());
    assert_file_matches(
        &home.claude_desktop_configuration(),
        &fixture.claude,
        "preflight failure should preserve Claude bytes",
    );
    assert_file_matches(
        &home.cursor_configuration(),
        &fixture.cursor,
        "preflight failure should preserve Cursor bytes",
    );
    assert_file_matches(
        &home.windsurf_configuration(),
        malformed,
        "preflight failure should preserve malformed Windsurf bytes",
    );
    assert!(!backup_path(&home.claude_desktop_configuration()).exists());
    assert!(!backup_path(&home.cursor_configuration()).exists());
    assert!(!backup_path(&home.windsurf_configuration()).exists());
    assert_file_matches(
        &fixture.project_path,
        PROJECT_CURSOR,
        "preflight failure should preserve project Cursor bytes",
    );
    assert!(!fixture.process_marker.exists());
}

#[test]
fn missing_or_malformed_canonical_state_fails_before_native_discovery() {
    let home = SyntheticHome::new();
    let claude = b"Claude target must not be parsed before canonical validation\n";
    let cursor = b"Cursor target must not be parsed before canonical validation\n";
    let windsurf = b"Windsurf target must not be parsed before canonical validation\n";
    home.write_file(&home.claude_desktop_configuration(), claude);
    home.write_file(&home.cursor_configuration(), cursor);
    home.write_file(&home.windsurf_configuration(), windsurf);

    let missing = stderr(&run_failure(sync_command(&home, true)));
    assert!(missing.contains("canonical configuration does not exist"));
    assert!(missing.contains("run `mcp-sync init` first"));

    let malformed = b"{\"schemaVersion\":1,\"servers\":{\"private\":\"malformed-private-value\"";
    home.write_file(&home.canonical_configuration(), malformed);
    let invalid = stderr(&run_failure(sync_command(&home, false)));

    assert!(invalid.contains("canonical configuration"));
    assert!(invalid.contains("is invalid: invalid JSON:"));
    assert!(!invalid.contains("malformed-private-value"));
    assert_file_matches(
        &home.claude_desktop_configuration(),
        claude,
        "canonical validation failure should preserve Claude bytes",
    );
    assert_file_matches(
        &home.cursor_configuration(),
        cursor,
        "canonical validation failure should preserve Cursor bytes",
    );
    assert_file_matches(
        &home.windsurf_configuration(),
        windsurf,
        "canonical validation failure should preserve Windsurf bytes",
    );
    assert!(!backup_path(&home.claude_desktop_configuration()).exists());
    assert!(!backup_path(&home.cursor_configuration()).exists());
    assert!(!backup_path(&home.windsurf_configuration()).exists());
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
    assert_file_matches(
        &home.claude_desktop_configuration(),
        &fixture.claude,
        "collision failure should preserve Claude bytes",
    );
    assert_file_matches(
        &home.cursor_configuration(),
        &fixture.cursor,
        "collision failure should preserve Cursor bytes",
    );
    assert_file_matches(
        &home.windsurf_configuration(),
        &fixture.windsurf,
        "collision failure should preserve Windsurf bytes",
    );
    assert!(!backup_path(&home.claude_desktop_configuration()).exists());
    assert!(!backup_path(&home.cursor_configuration()).exists());
    assert!(!backup_path(&home.windsurf_configuration()).exists());
    assert_file_matches(
        &fixture.project_path,
        PROJECT_CURSOR,
        "collision failure should preserve project Cursor bytes",
    );
    assert!(!fixture.process_marker.exists());
}

#[test]
fn an_unmanaged_windsurf_name_collision_fails_the_complete_plan_before_apply() {
    let home = SyntheticHome::new();
    let fixture = prepare_existing_journey(&home);
    let mut canonical: Value =
        serde_json::from_slice(&fixture.canonical).expect("canonical fixture should parse");
    canonical["servers"]["windsurf-remote-only"] = canonical["servers"]["added"].clone();
    let mut canonical =
        serde_json::to_vec_pretty(&canonical).expect("collision fixture should serialize");
    canonical.push(b'\n');
    home.write_file(&home.canonical_configuration(), &canonical);

    let output = run_failure(sync_command(&home, false));
    let diagnostic = stderr(&output);

    assert!(diagnostic.contains(
        "cannot render the validated Windsurf sync plan: desired local server \"windsurf-remote-only\" collides with an unmanaged Windsurf server"
    ));
    assert!(diagnostic.ends_with("; no target files were changed\n"));
    assert_output_omits(&diagnostic, &fixture.private_values());
    assert_file_matches(
        &home.claude_desktop_configuration(),
        &fixture.claude,
        "Windsurf collision should preserve Claude bytes",
    );
    assert_file_matches(
        &home.cursor_configuration(),
        &fixture.cursor,
        "Windsurf collision should preserve Cursor bytes",
    );
    assert_file_matches(
        &home.windsurf_configuration(),
        &fixture.windsurf,
        "Windsurf collision should preserve Windsurf bytes",
    );
    for target in [
        home.claude_desktop_configuration(),
        home.cursor_configuration(),
        home.windsurf_configuration(),
    ] {
        assert!(!backup_path(&target).exists());
    }
    assert_file_matches(
        &fixture.project_path,
        PROJECT_CURSOR,
        "Windsurf collision should preserve project Cursor bytes",
    );
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
    assert!(diagnostic.contains("Windsurf: not attempted after an earlier failure"));
    assert!(diagnostic.contains("Permission denied"));
    assert_output_omits(&diagnostic, &fixture.private_values());
    assert_file_matches(
        &home.claude_desktop_configuration(),
        &fixture.claude,
        "permission rollback should restore Claude bytes",
    );
    assert_file_matches(
        &home.cursor_configuration(),
        &fixture.cursor,
        "permission failure should preserve Cursor bytes",
    );
    assert!(!backup_path(&home.claude_desktop_configuration()).exists());
    assert!(!backup_path(&home.cursor_configuration()).exists());
    assert_file_matches(
        &home.windsurf_configuration(),
        &fixture.windsurf,
        "permission failure should preserve Windsurf bytes",
    );
    assert!(!backup_path(&home.windsurf_configuration()).exists());
    assert_file_matches(
        &fixture.project_path,
        PROJECT_CURSOR,
        "permission failure should preserve project Cursor bytes",
    );
    assert!(!fixture.process_marker.exists());
}
