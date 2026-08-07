mod support;

use assert_cmd::Command as AssertCommand;
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;
use std::time::Duration;
use support::{SyntheticHome, assert_bytes_match, assert_file_matches};

const COMMAND_TIMEOUT: Duration = Duration::from_secs(5);

fn json_document(value: &Value) -> Vec<u8> {
    let mut bytes = serde_json::to_vec_pretty(value).expect("synthetic JSON should serialize");
    bytes.push(b'\n');
    bytes
}

fn cli_command(home: &SyntheticHome, arguments: &[&str]) -> AssertCommand {
    let mut command = AssertCommand::from_std(home.command());
    command.args(arguments).timeout(COMMAND_TIMEOUT);
    command
}

fn add_command(
    home: &SyntheticHome,
    name: &str,
    command_value: &str,
    arguments: &[&str],
    environment: &[&str],
) -> AssertCommand {
    let mut command = AssertCommand::from_std(home.command());
    command
        .arg("add")
        .arg(name)
        .arg("--command")
        .arg(command_value);
    for argument in arguments {
        command.arg("--arg").arg(argument);
    }
    for assignment in environment {
        command.arg("--env").arg(assignment);
    }
    command.timeout(COMMAND_TIMEOUT);
    command
}

fn run_success(mut command: AssertCommand) -> Output {
    let output = command.assert().get_output().clone();
    assert!(
        output.status.code() == Some(0),
        "the built CLI should exit zero"
    );
    assert!(output.stderr.is_empty(), "success should not write stderr");
    output
}

fn run_failure(mut command: AssertCommand) -> Output {
    let output = command.assert().get_output().clone();
    assert!(
        output.status.code() == Some(1),
        "an application failure should exit with code 1"
    );
    assert!(output.stdout.is_empty(), "failure should not write stdout");
    output
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be UTF-8")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr should be UTF-8")
}

fn read(path: &Path) -> Vec<u8> {
    fs::read(path).unwrap_or_else(|error| {
        panic!(
            "synthetic file {} should be readable: {error}",
            path.display()
        )
    })
}

fn assert_private_values_absent(outputs: &[&str], private_values: &[&str]) {
    for (index, private_value) in private_values.iter().enumerate() {
        for output in outputs {
            assert!(
                !output.contains(private_value),
                "CLI output exposed private sentinel at index {index}"
            );
        }
    }
}

fn assert_no_temporary_files(path: &Path) {
    let Some(parent) = path.parent() else {
        return;
    };
    if !parent.exists() {
        return;
    }
    let temporary_exists = fs::read_dir(parent)
        .expect("a synthetic target directory should be readable")
        .filter_map(Result::ok)
        .any(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with(".mcp-sync-")
        });
    assert!(!temporary_exists, "temporary files should be cleaned up");
}

fn backup_path(path: &Path) -> PathBuf {
    let mut backup = path.as_os_str().to_owned();
    backup.push(".bak");
    PathBuf::from(backup)
}

fn process_sentinel(home: &SyntheticHome, name: &str, marker: &Path) -> String {
    let path = home.root().join(name);
    home.write_file(
        &path,
        format!("#!/bin/sh\nprintf started > '{}'\n", marker.display()),
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700))
            .expect("the process sentinel should be executable");
    }
    path.to_string_lossy().into_owned()
}

fn add_target_only_server(path: &Path, name: &str, command: &str, private_value: &str) {
    let mut document: Value =
        serde_json::from_slice(&read(path)).expect("the native fixture should be valid JSON");
    document["mcpServers"]
        .as_object_mut()
        .expect("the native fixture should contain a server map")
        .insert(
            name.to_owned(),
            json!({
                "command": command,
                "args": ["--target-only-private-argument"],
                "env": {"TARGET_ONLY_TOKEN": private_value},
                "futureNativeField": "target-only-unowned-private-value"
            }),
        );
    let bytes = json_document(&document);
    fs::write(path, bytes).expect("the native fixture should be updated");
}

#[test]
fn golden_source_checkout_journey_runs_all_commands_as_one_safe_flow() {
    let home = SyntheticHome::new();
    let imported_marker = home.root().join("imported-server-started");
    let updated_marker = home.root().join("updated-server-started");
    let added_marker = home.root().join("added-server-started");
    let imported_command = process_sentinel(&home, "imported-private-server", &imported_marker);
    let updated_command = process_sentinel(&home, "updated-private-server", &updated_marker);
    let added_command = process_sentinel(&home, "added-private-server", &added_marker);

    let claude = json_document(&json!({
        "mcpServers": {
            "alpha": {
                "command": "alpha-private-command",
                "args": ["--alpha-private-argument"],
                "env": {"ALPHA_TOKEN": "alpha-private-value"},
                "cwd": "/synthetic/alpha-unowned-private"
            },
            "shared": {
                "command": imported_command,
                "args": ["--shared-imported-private-argument"],
                "env": {"SHARED_TOKEN": "shared-imported-private-value"},
                "cwd": "/synthetic/shared-unowned-private"
            }
        },
        "preferences": {"privatePreference": "claude-unowned-private-value"},
        "futureTopLevel": {"enabled": true}
    }));
    let cursor = json_document(&json!({
        "mcpServers": {
            "beta": {
                "command": "beta-private-command",
                "args": ["--beta-private-argument"],
                "env": {"BETA_TOKEN": "beta-private-value"},
                "envFile": "/synthetic/beta-unowned-private.env"
            },
            "remote-only": {
                "url": "https://remote-private.invalid/mcp",
                "headers": {"Authorization": "Bearer remote-private-value"}
            },
            "shared": {
                "command": imported_command,
                "args": ["--shared-imported-private-argument"],
                "env": {"SHARED_TOKEN": "shared-imported-private-value"},
                "envFile": "/synthetic/shared-unowned-private.env"
            }
        },
        "cursorMetadata": {"private": "cursor-unowned-private-value"}
    }));
    let windsurf = json_document(&json!({
        "mcpServers": {
            "gamma": {
                "command": "gamma-private-command",
                "args": ["--gamma-private-argument"],
                "env": {"GAMMA_TOKEN": "gamma-private-value"},
                "disabledTools": ["gamma_unowned_tool"]
            },
            "windsurf-remote-only": {
                "serverUrl": "https://windsurf-remote-private.invalid/mcp",
                "headers": {"Authorization": "Bearer windsurf-remote-private-value"},
                "disabledTools": ["remote_unowned_tool"]
            },
            "shared": {
                "command": imported_command,
                "args": ["--shared-imported-private-argument"],
                "env": {"SHARED_TOKEN": "shared-imported-private-value"},
                "disabledTools": ["shared_unowned_tool"]
            }
        },
        "windsurfMetadata": {"private": "windsurf-unowned-private-value"}
    }));
    let project_path = home.user_root().join("workspace/.cursor/mcp.json");
    let project = b"{\"projectPrivateSentinel\":\"project-unowned-private-value\"}\n";
    home.write_file(&home.claude_desktop_configuration(), &claude);
    home.write_file(&home.cursor_configuration(), &cursor);
    home.write_file(&home.windsurf_configuration(), &windsurf);
    home.write_file(&project_path, project);

    let private_values = [
        imported_command.as_str(),
        updated_command.as_str(),
        added_command.as_str(),
        "alpha-private-command",
        "--alpha-private-argument",
        "alpha-private-value",
        "beta-private-command",
        "--beta-private-argument",
        "beta-private-value",
        "gamma-private-command",
        "--gamma-private-argument",
        "gamma-private-value",
        "--shared-imported-private-argument",
        "shared-imported-private-value",
        "--shared-updated-private-argument",
        "shared-updated-private-value",
        "--added-private-argument",
        "added-private-value",
        "https://remote-private.invalid/mcp",
        "Bearer remote-private-value",
        "https://windsurf-remote-private.invalid/mcp",
        "Bearer windsurf-remote-private-value",
        "/synthetic/alpha-unowned-private",
        "/synthetic/beta-unowned-private.env",
        "/synthetic/shared-unowned-private",
        "/synthetic/shared-unowned-private.env",
        "claude-unowned-private-value",
        "cursor-unowned-private-value",
        "windsurf-unowned-private-value",
        "project-unowned-private-value",
        "claude-target-only-private-command",
        "cursor-target-only-private-command",
        "windsurf-target-only-private-command",
        "claude-target-only-private-value",
        "cursor-target-only-private-value",
        "windsurf-target-only-private-value",
        "target-only-unowned-private-value",
    ];

    let init_output = stdout(&run_success(cli_command(&home, &["init"])));
    assert!(
        init_output.starts_with(
            "Initialized canonical configuration with 4 servers from 3 client configurations.\n"
        ),
        "init should report the deterministic imported shape"
    );
    assert!(
        init_output.contains("Skipped 1 unsupported Cursor entry: \"remote-only\"."),
        "init should report unsupported Cursor structure by name"
    );
    assert!(
        init_output.contains("Skipped 1 unsupported Windsurf entry: \"windsurf-remote-only\"."),
        "init should report unsupported Windsurf structure by name"
    );
    assert_private_values_absent(&[&init_output], &private_values);
    assert_file_matches(
        &home.claude_desktop_configuration(),
        &claude,
        "init must preserve Claude Desktop bytes",
    );
    assert_file_matches(
        &home.cursor_configuration(),
        &cursor,
        "init must preserve Cursor bytes",
    );
    assert_file_matches(
        &home.windsurf_configuration(),
        &windsurf,
        "init must preserve Windsurf bytes",
    );
    assert_file_matches(
        &project_path,
        project,
        "init must preserve project Cursor bytes",
    );

    let update_output = stdout(&run_success(add_command(
        &home,
        "shared",
        &updated_command,
        &["--shared-updated-private-argument"],
        &["SHARED_TOKEN=shared-updated-private-value"],
    )));
    let add_output = stdout(&run_success(add_command(
        &home,
        "added",
        &added_command,
        &["--added-private-argument"],
        &["ADDED_TOKEN=added-private-value"],
    )));
    assert!(
        update_output.starts_with("Updated server \"shared\""),
        "add should update one complete definition"
    );
    assert!(
        add_output.starts_with("Added server \"added\""),
        "add should insert one complete definition"
    );
    assert_private_values_absent(&[&update_output, &add_output], &private_values);

    let list_output = stdout(&run_success(cli_command(&home, &["list"])));
    let listed_names = [
        "\"added\":",
        "\"alpha\":",
        "\"beta\":",
        "\"gamma\":",
        "\"shared\":",
    ];
    let listed_offsets = listed_names.map(|name| {
        list_output
            .find(name)
            .unwrap_or_else(|| panic!("list should contain structural name {name}"))
    });
    assert!(
        listed_offsets.windows(2).all(|pair| pair[0] < pair[1]),
        "list should sort server names"
    );
    assert!(
        list_output.contains("environment keys: \"ADDED_TOKEN\"")
            && list_output.contains("environment keys: \"SHARED_TOKEN\""),
        "list should expose only structural environment key names"
    );
    assert_private_values_absent(&[&list_output], &private_values);

    add_target_only_server(
        &home.claude_desktop_configuration(),
        "target-only",
        "claude-target-only-private-command",
        "claude-target-only-private-value",
    );
    add_target_only_server(
        &home.cursor_configuration(),
        "target-only",
        "cursor-target-only-private-command",
        "cursor-target-only-private-value",
    );
    add_target_only_server(
        &home.windsurf_configuration(),
        "target-only",
        "windsurf-target-only-private-command",
        "windsurf-target-only-private-value",
    );

    let canonical_path = home.canonical_configuration();
    let canonical_backup = backup_path(&canonical_path);
    let claude_path = home.claude_desktop_configuration();
    let cursor_path = home.cursor_configuration();
    let windsurf_path = home.windsurf_configuration();
    let claude_backup = backup_path(&claude_path);
    let cursor_backup = backup_path(&cursor_path);
    let windsurf_backup = backup_path(&windsurf_path);
    let canonical_before_sync = read(&canonical_path);
    let canonical_backup_before_sync = read(&canonical_backup);
    let claude_before_sync = read(&claude_path);
    let cursor_before_sync = read(&cursor_path);
    let windsurf_before_sync = read(&windsurf_path);

    let dry_output = stdout(&run_success(cli_command(&home, &["sync", "--dry-run"])));
    assert!(
        dry_output.starts_with("Dry run validated 3 targets; no files changed.\n"),
        "dry-run should validate the complete three-target plan"
    );
    assert!(
        dry_output.contains("Claude Desktop: would update with recoverable backup")
            && dry_output.contains("Cursor: would update with recoverable backup")
            && dry_output.contains("Windsurf: would update with recoverable backup")
            && dry_output.contains("preserve target-only \"target-only\"")
            && dry_output.contains("preserve unmanaged \"remote-only\"")
            && dry_output.contains("preserve unmanaged \"windsurf-remote-only\""),
        "dry-run should report every target and preservation outcome"
    );
    assert_private_values_absent(&[&dry_output], &private_values);
    assert_file_matches(
        &canonical_path,
        &canonical_before_sync,
        "dry-run must preserve canonical bytes",
    );
    assert_file_matches(
        &canonical_backup,
        &canonical_backup_before_sync,
        "dry-run must preserve the canonical backup",
    );
    assert_file_matches(
        &claude_path,
        &claude_before_sync,
        "dry-run must preserve Claude",
    );
    assert_file_matches(
        &cursor_path,
        &cursor_before_sync,
        "dry-run must preserve Cursor",
    );
    assert_file_matches(
        &windsurf_path,
        &windsurf_before_sync,
        "dry-run must preserve Windsurf",
    );
    assert!(!claude_backup.exists() && !cursor_backup.exists() && !windsurf_backup.exists());

    let apply_output = stdout(&run_success(cli_command(&home, &["sync"])));
    assert!(
        apply_output.starts_with("Sync completed for 3 targets.\n")
            && apply_output.contains("Claude Desktop: updated with recoverable backup")
            && apply_output.contains("Cursor: updated with recoverable backup")
            && apply_output.contains("Windsurf: updated with recoverable backup"),
        "sync should report successful per-target application"
    );
    assert_private_values_absent(&[&apply_output], &private_values);
    assert_file_matches(
        &claude_backup,
        &claude_before_sync,
        "Claude backup must contain exact pre-sync bytes",
    );
    assert_file_matches(
        &cursor_backup,
        &cursor_before_sync,
        "Cursor backup must contain exact pre-sync bytes",
    );
    assert_file_matches(
        &windsurf_backup,
        &windsurf_before_sync,
        "Windsurf backup must contain exact pre-sync bytes",
    );
    assert_file_matches(
        &canonical_path,
        &canonical_before_sync,
        "sync must not rewrite canonical state",
    );
    assert_file_matches(
        &canonical_backup,
        &canonical_backup_before_sync,
        "sync must not rewrite the canonical backup",
    );
    assert_file_matches(
        &project_path,
        project,
        "sync must preserve project Cursor bytes",
    );

    let claude_after: Value =
        serde_json::from_slice(&read(&claude_path)).expect("Claude output should be valid JSON");
    let cursor_after: Value =
        serde_json::from_slice(&read(&cursor_path)).expect("Cursor output should be valid JSON");
    let windsurf_after: Value = serde_json::from_slice(&read(&windsurf_path))
        .expect("Windsurf output should be valid JSON");
    for target in [&claude_after, &cursor_after, &windsurf_after] {
        assert!(
            target["mcpServers"]["shared"]["command"].as_str() == Some(updated_command.as_str())
                && target["mcpServers"]["added"]["command"].as_str()
                    == Some(added_command.as_str())
                && target["mcpServers"]["alpha"].is_object()
                && target["mcpServers"]["beta"].is_object()
                && target["mcpServers"]["gamma"].is_object(),
            "each native target should contain the complete desired managed definitions"
        );
    }
    assert!(
        claude_after["preferences"]["privatePreference"].as_str()
            == Some("claude-unowned-private-value")
            && claude_after["mcpServers"]["shared"]["cwd"].as_str()
                == Some("/synthetic/shared-unowned-private")
            && claude_after["mcpServers"]["target-only"]["env"]["TARGET_ONLY_TOKEN"].as_str()
                == Some("claude-target-only-private-value"),
        "Claude unowned and target-only data should survive"
    );
    assert!(
        cursor_after["cursorMetadata"]["private"].as_str() == Some("cursor-unowned-private-value")
            && cursor_after["mcpServers"]["shared"]["envFile"].as_str()
                == Some("/synthetic/shared-unowned-private.env")
            && cursor_after["mcpServers"]["remote-only"]["headers"]["Authorization"].as_str()
                == Some("Bearer remote-private-value")
            && cursor_after["mcpServers"]["target-only"]["env"]["TARGET_ONLY_TOKEN"].as_str()
                == Some("cursor-target-only-private-value"),
        "Cursor unowned, unmanaged, and target-only data should survive"
    );
    assert!(
        windsurf_after["windsurfMetadata"]["private"].as_str()
            == Some("windsurf-unowned-private-value")
            && windsurf_after["mcpServers"]["shared"]["disabledTools"][0].as_str()
                == Some("shared_unowned_tool")
            && windsurf_after["mcpServers"]["windsurf-remote-only"]["headers"]["Authorization"]
                .as_str()
                == Some("Bearer windsurf-remote-private-value")
            && windsurf_after["mcpServers"]["target-only"]["env"]["TARGET_ONLY_TOKEN"].as_str()
                == Some("windsurf-target-only-private-value"),
        "Windsurf unowned, unmanaged, and target-only data should survive"
    );

    let claude_after_bytes = read(&claude_path);
    let cursor_after_bytes = read(&cursor_path);
    let windsurf_after_bytes = read(&windsurf_path);
    let claude_backup_after = read(&claude_backup);
    let cursor_backup_after = read(&cursor_backup);
    let windsurf_backup_after = read(&windsurf_backup);
    let no_op_output = stdout(&run_success(cli_command(&home, &["sync"])));
    assert!(
        no_op_output
            .matches("unchanged; no write or backup")
            .count()
            == 3,
        "repeat sync should report three target no-ops"
    );
    assert_private_values_absent(&[&no_op_output], &private_values);
    assert_file_matches(
        &claude_path,
        &claude_after_bytes,
        "repeat sync must preserve Claude bytes",
    );
    assert_file_matches(
        &cursor_path,
        &cursor_after_bytes,
        "repeat sync must preserve Cursor bytes",
    );
    assert_file_matches(
        &windsurf_path,
        &windsurf_after_bytes,
        "repeat sync must preserve Windsurf bytes",
    );
    assert_file_matches(
        &claude_backup,
        &claude_backup_after,
        "repeat sync must preserve the Claude backup",
    );
    assert_file_matches(
        &cursor_backup,
        &cursor_backup_after,
        "repeat sync must preserve the Cursor backup",
    );
    assert_file_matches(
        &windsurf_backup,
        &windsurf_backup_after,
        "repeat sync must preserve the Windsurf backup",
    );
    for marker in [&imported_marker, &updated_marker, &added_marker] {
        assert!(
            !marker.exists(),
            "configuration commands must never execute servers"
        );
    }
    for path in [&canonical_path, &claude_path, &cursor_path, &windsurf_path] {
        assert_no_temporary_files(path);
    }
}

#[test]
fn built_binary_import_is_deterministic_when_client_assignments_are_reversed() {
    let first = SyntheticHome::new();
    let second = SyntheticHome::new();
    let alpha = json!({
        "command": "alpha-private-command",
        "args": ["--alpha-private-argument"],
        "env": {"ALPHA_TOKEN": "alpha-private-value"}
    });
    let beta = json!({
        "command": "beta-private-command",
        "args": ["--beta-private-argument"],
        "env": {"BETA_TOKEN": "beta-private-value"}
    });
    let gamma = json!({
        "command": "gamma-private-command",
        "args": ["--gamma-private-argument"],
        "env": {"GAMMA_TOKEN": "gamma-private-value"}
    });
    let shared = json!({
        "command": "shared-private-command",
        "args": ["--shared-private-argument"],
        "env": {"SHARED_TOKEN": "shared-private-value"}
    });
    first.write_file(
        &first.claude_desktop_configuration(),
        json_document(&json!({"mcpServers": {"alpha": alpha, "shared": shared}})),
    );
    first.write_file(
        &first.cursor_configuration(),
        json_document(&json!({"mcpServers": {"beta": beta, "shared": shared}})),
    );
    first.write_file(
        &first.windsurf_configuration(),
        json_document(&json!({"mcpServers": {"gamma": gamma, "shared": shared}})),
    );
    second.write_file(
        &second.claude_desktop_configuration(),
        json_document(&json!({"mcpServers": {"gamma": gamma, "shared": shared}})),
    );
    second.write_file(
        &second.cursor_configuration(),
        json_document(&json!({"mcpServers": {"alpha": alpha, "shared": shared}})),
    );
    second.write_file(
        &second.windsurf_configuration(),
        json_document(&json!({"mcpServers": {"beta": beta, "shared": shared}})),
    );

    let first_output = stdout(&run_success(cli_command(&first, &["init"])));
    let second_output = stdout(&run_success(cli_command(&second, &["init"])));
    let private_values = [
        "alpha-private-command",
        "--alpha-private-argument",
        "alpha-private-value",
        "beta-private-command",
        "--beta-private-argument",
        "beta-private-value",
        "gamma-private-command",
        "--gamma-private-argument",
        "gamma-private-value",
        "shared-private-command",
        "--shared-private-argument",
        "shared-private-value",
    ];
    assert_private_values_absent(&[&first_output, &second_output], &private_values);
    assert_bytes_match(
        &read(&first.canonical_configuration()),
        &read(&second.canonical_configuration()),
        "reversing client assignment must not change canonical bytes",
    );
}

#[test]
fn built_binary_failure_matrix_is_nonzero_non_mutating_and_redacted() {
    {
        let home = SyntheticHome::new();
        let native = b"native-private-value must not be parsed before canonical state\n";
        home.write_file(&home.claude_desktop_configuration(), native);
        home.write_file(&home.cursor_configuration(), native);
        home.write_file(&home.windsurf_configuration(), native);
        let diagnostic = stderr(&run_failure(cli_command(&home, &["sync", "--dry-run"])));
        assert!(
            diagnostic.contains("canonical configuration does not exist")
                && diagnostic.contains("run `mcp-sync init` first"),
            "missing canonical state should be actionable"
        );
        assert_private_values_absent(&[&diagnostic], &["native-private-value"]);
        assert_file_matches(
            &home.claude_desktop_configuration(),
            native,
            "missing canonical failure must preserve Claude",
        );
        assert_file_matches(
            &home.cursor_configuration(),
            native,
            "missing canonical failure must preserve Cursor",
        );
        assert_file_matches(
            &home.windsurf_configuration(),
            native,
            "missing canonical failure must preserve Windsurf",
        );
    }

    {
        let home = SyntheticHome::new();
        let unsupported = br#"{
  "schemaVersion": 2,
  "servers": {
    "private": {
      "command": "schema-private-command",
      "env": {"TOKEN": "schema-private-value"}
    }
  }
}
"#;
        home.write_file(&home.canonical_configuration(), unsupported);
        let diagnostic = stderr(&run_failure(cli_command(&home, &["list"])));
        assert!(
            diagnostic.contains("unsupported canonical configuration schema version 2")
                && diagnostic.contains("supported version is 1"),
            "unknown schema versions should require deliberate migration"
        );
        assert_private_values_absent(
            &[&diagnostic],
            &["schema-private-command", "schema-private-value"],
        );
        assert_file_matches(
            &home.canonical_configuration(),
            unsupported,
            "unknown-version failure must preserve canonical bytes",
        );
    }

    {
        let home = SyntheticHome::new();
        let claude = json_document(&json!({
            "mcpServers": {
                "shared": {
                    "command": "claude-conflict-private-command",
                    "args": ["--claude-conflict-private-argument"],
                    "env": {"TOKEN": "claude-conflict-private-value"}
                }
            }
        }));
        let cursor = json_document(&json!({
            "mcpServers": {
                "shared": {
                    "command": "cursor-conflict-private-command",
                    "args": ["--cursor-conflict-private-argument"],
                    "env": {"TOKEN": "cursor-conflict-private-value"}
                }
            }
        }));
        home.write_file(&home.claude_desktop_configuration(), &claude);
        home.write_file(&home.cursor_configuration(), &cursor);
        let diagnostic = stderr(&run_failure(cli_command(&home, &["init"])));
        assert!(
            diagnostic.contains("server \"shared\" differs between Claude Desktop and Cursor")
                && diagnostic.contains("rerun `mcp-sync init`"),
            "import conflicts should be structural and actionable"
        );
        assert_private_values_absent(
            &[&diagnostic],
            &[
                "claude-conflict-private-command",
                "--claude-conflict-private-argument",
                "claude-conflict-private-value",
                "cursor-conflict-private-command",
                "--cursor-conflict-private-argument",
                "cursor-conflict-private-value",
            ],
        );
        assert!(!home.canonical_configuration().exists());
        assert_file_matches(
            &home.claude_desktop_configuration(),
            &claude,
            "conflict must preserve Claude",
        );
        assert_file_matches(
            &home.cursor_configuration(),
            &cursor,
            "conflict must preserve Cursor",
        );
    }

    {
        let home = SyntheticHome::new();
        let canonical = json_document(&json!({
            "schemaVersion": 1,
            "servers": {
                "shared": {
                    "command": "desired-private-command",
                    "args": ["--desired-private-argument"],
                    "env": {"TOKEN": "desired-private-value"}
                }
            }
        }));
        let claude = json_document(&json!({
            "mcpServers": {
                "shared": {
                    "command": "current-private-command",
                    "args": ["--current-private-argument"],
                    "env": {"TOKEN": "current-private-value"}
                }
            }
        }));
        let cursor = json_document(&json!({
            "mcpServers": {
                "shared": {
                    "command": "cursor-current-private-command",
                    "args": ["--cursor-current-private-argument"],
                    "env": {"TOKEN": "cursor-current-private-value"}
                }
            }
        }));
        let malformed_windsurf =
            b"{\"mcpServers\":{\"shared\":{\"command\":\"malformed-private-command\"";
        home.write_file(&home.canonical_configuration(), &canonical);
        home.write_file(&home.claude_desktop_configuration(), &claude);
        home.write_file(&home.cursor_configuration(), &cursor);
        home.write_file(&home.windsurf_configuration(), malformed_windsurf);
        let diagnostic = stderr(&run_failure(cli_command(&home, &["sync"])));
        assert!(
            diagnostic.contains("cannot plan Windsurf sync: invalid Windsurf JSON:")
                && diagnostic.ends_with("; no target files were changed\n"),
            "malformed later input should fail complete preflight"
        );
        assert_private_values_absent(
            &[&diagnostic],
            &[
                "desired-private-command",
                "--desired-private-argument",
                "desired-private-value",
                "current-private-command",
                "--current-private-argument",
                "current-private-value",
                "cursor-current-private-command",
                "--cursor-current-private-argument",
                "cursor-current-private-value",
                "malformed-private-command",
            ],
        );
        assert_file_matches(
            &home.claude_desktop_configuration(),
            &claude,
            "preflight failure must preserve Claude",
        );
        assert_file_matches(
            &home.cursor_configuration(),
            &cursor,
            "preflight failure must preserve Cursor bytes",
        );
        assert_file_matches(
            &home.windsurf_configuration(),
            malformed_windsurf,
            "preflight failure must preserve Windsurf",
        );
        assert!(!backup_path(&home.claude_desktop_configuration()).exists());
        assert!(!backup_path(&home.cursor_configuration()).exists());
        assert!(!backup_path(&home.windsurf_configuration()).exists());
    }

    {
        let home = SyntheticHome::new();
        let canonical = json_document(&json!({
            "schemaVersion": 1,
            "servers": {
                "shared": {
                    "command": "rollback-desired-private-command",
                    "args": ["--rollback-desired-private-argument"],
                    "env": {"TOKEN": "rollback-desired-private-value"}
                }
            }
        }));
        let claude = json_document(&json!({
            "mcpServers": {
                "shared": {
                    "command": "rollback-claude-private-command",
                    "args": ["--rollback-claude-private-argument"],
                    "env": {"TOKEN": "rollback-claude-private-value"},
                    "cwd": "/rollback/claude-unowned-private"
                }
            }
        }));
        let cursor = json_document(&json!({
            "mcpServers": {
                "shared": {
                    "command": "rollback-cursor-private-command",
                    "args": ["--rollback-cursor-private-argument"],
                    "env": {"TOKEN": "rollback-cursor-private-value"},
                    "envFile": "/rollback/cursor-unowned-private.env"
                }
            }
        }));
        let claude_path = home.claude_desktop_configuration();
        let cursor_path = home.cursor_configuration();
        let claude_backup = backup_path(&claude_path);
        let cursor_backup = backup_path(&cursor_path);
        let prior_backup = b"prior-Claude-backup-private-value\n";
        home.write_file(&home.canonical_configuration(), &canonical);
        home.write_file(&claude_path, &claude);
        home.write_file(&cursor_path, &cursor);
        home.write_file(&claude_backup, prior_backup);
        fs::create_dir(&cursor_backup).expect("a blocking Cursor backup should be created");

        let diagnostic = stderr(&run_failure(cli_command(&home, &["sync"])));
        assert!(
            diagnostic.starts_with("error: sync transaction failed while applying Cursor:")
                && diagnostic.contains("Claude Desktop: rolled back after update")
                && diagnostic.contains("Cursor: update failed")
                && diagnostic.contains("Per-target outcomes:"),
            "a later apply failure should report reverse recovery"
        );
        assert_private_values_absent(
            &[&diagnostic],
            &[
                "rollback-desired-private-command",
                "--rollback-desired-private-argument",
                "rollback-desired-private-value",
                "rollback-claude-private-command",
                "--rollback-claude-private-argument",
                "rollback-claude-private-value",
                "rollback-cursor-private-command",
                "--rollback-cursor-private-argument",
                "rollback-cursor-private-value",
                "/rollback/claude-unowned-private",
                "/rollback/cursor-unowned-private.env",
                "prior-Claude-backup-private-value",
            ],
        );
        assert_file_matches(&claude_path, &claude, "rollback must restore Claude bytes");
        assert_file_matches(
            &claude_backup,
            prior_backup,
            "rollback must restore the prior Claude backup",
        );
        assert_file_matches(&cursor_path, &cursor, "failed Cursor must remain unchanged");
        assert!(cursor_backup.is_dir());
        assert_no_temporary_files(&claude_path);
        assert_no_temporary_files(&cursor_path);
    }
}
