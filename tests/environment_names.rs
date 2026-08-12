mod support;

use assert_cmd::Command as AssertCommand;
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;
use std::time::Duration;
use support::{SyntheticHome, assert_file_matches};

const COMMAND_TIMEOUT: Duration = Duration::from_secs(5);
const EMPTY_CANONICAL: &[u8] = b"{\n  \"schemaVersion\": 1,\n  \"servers\": {}\n}\n";

#[derive(Clone, Copy)]
enum UnmanagedClient {
    Cursor,
    Windsurf,
    VsCode,
    Codex,
    Kiro,
}

impl UnmanagedClient {
    fn label(self) -> &'static str {
        match self {
            Self::Cursor => "Cursor",
            Self::Windsurf => "Windsurf",
            Self::VsCode => "VS Code",
            Self::Codex => "Codex",
            Self::Kiro => "Kiro",
        }
    }

    fn path(self, home: &SyntheticHome) -> PathBuf {
        match self {
            Self::Cursor => home.cursor_configuration(),
            Self::Windsurf => home.windsurf_configuration(),
            Self::VsCode => home.vscode_configuration(),
            Self::Codex => home.codex_configuration(),
            Self::Kiro => home.kiro_configuration(),
        }
    }

    fn invalid_document(self) -> Vec<u8> {
        match self {
            Self::Cursor => {
                include_bytes!("fixtures/cursor/invalid-environment-name.json").to_vec()
            }
            Self::Windsurf => {
                include_bytes!("fixtures/windsurf/invalid-environment-name.json").to_vec()
            }
            Self::VsCode => {
                include_bytes!("fixtures/vscode/invalid-environment-name.json").to_vec()
            }
            Self::Codex => include_bytes!("fixtures/codex/invalid-environment-name.toml").to_vec(),
            Self::Kiro => include_bytes!("fixtures/kiro/invalid-environment-name.json").to_vec(),
        }
    }
}

fn json_document(value: &Value) -> Vec<u8> {
    let mut bytes = serde_json::to_vec_pretty(value).expect("synthetic JSON should serialize");
    bytes.push(b'\n');
    bytes
}

fn command(home: &SyntheticHome, arguments: &[&str]) -> AssertCommand {
    let mut command = AssertCommand::from_std(home.command());
    command.args(arguments).timeout(COMMAND_TIMEOUT);
    command
}

fn run_success(mut command: AssertCommand) -> Output {
    let output = command.assert().get_output().clone();
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    output
}

fn run_failure(mut command: AssertCommand) -> Output {
    let output = command.assert().get_output().clone();
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
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

fn managed_target_paths(home: &SyntheticHome) -> [PathBuf; 6] {
    [
        home.claude_desktop_configuration(),
        home.cursor_configuration(),
        home.windsurf_configuration(),
        home.vscode_configuration(),
        home.codex_configuration(),
        home.kiro_configuration(),
    ]
}

fn assert_omits(output: &str, private_values: &[&str]) {
    for private in private_values {
        assert!(
            !output.contains(private),
            "diagnostics must omit private environment and process text"
        );
    }
}

#[test]
fn add_rejects_an_empty_environment_name_before_lock_or_managed_state_access() {
    let home = SyntheticHome::new();
    let canonical_parent = home
        .canonical_configuration()
        .parent()
        .expect("canonical path should have a parent")
        .to_owned();
    assert!(!canonical_parent.exists());

    let output = run_failure(command(
        &home,
        &[
            "add",
            "alpha",
            "--command",
            "private-command",
            "--arg",
            "private-argument",
            "--env",
            "=private-value",
        ],
    ));
    let diagnostic = stderr(&output);

    assert!(diagnostic.contains("environment name 0 for server \"alpha\""));
    assert!(diagnostic.contains("must not be empty"));
    assert_omits(
        &diagnostic,
        &["private-command", "private-argument", "private-value"],
    );
    assert!(!home.canonical_configuration().exists());
    assert!(!home.operation_lock().exists());
    assert!(
        !canonical_parent.exists(),
        "validation should fail before creating managed-state directories"
    );
}

#[test]
fn init_rejects_an_invalid_claude_environment_name_without_mutation_or_disclosure() {
    let home = SyntheticHome::new();
    let claude = include_bytes!("fixtures/claude-desktop/invalid-environment-name.json");
    home.write_file(&home.claude_desktop_configuration(), claude);

    let output = run_failure(command(&home, &["init"]));
    let diagnostic = stderr(&output);

    assert!(diagnostic.contains("invalid Claude Desktop MCP server definition"));
    assert!(diagnostic.contains("environment name 0 for server \"collision\""));
    assert!(diagnostic.contains("must not contain `=`"));
    assert_omits(
        &diagnostic,
        &[
            "synthetic-command",
            "synthetic-argument",
            "SYNTHETIC=NAME",
            "synthetic-value",
        ],
    );
    assert!(!home.canonical_configuration().exists());
    assert_file_matches(
        &home.claude_desktop_configuration(),
        claude,
        "failed Claude import must preserve exact native bytes",
    );
    assert!(home.operation_lock().is_file());
    assert!(fs::read(home.operation_lock()).unwrap().is_empty());
}

#[test]
fn init_skips_invalid_environment_names_for_five_unmanaged_clients() {
    let home = SyntheticHome::new();
    let cases = [
        UnmanagedClient::Cursor,
        UnmanagedClient::Windsurf,
        UnmanagedClient::VsCode,
        UnmanagedClient::Codex,
        UnmanagedClient::Kiro,
    ];
    let native = cases
        .into_iter()
        .map(|client| {
            let bytes = client.invalid_document();
            home.write_file(&client.path(&home), &bytes);
            (client, bytes)
        })
        .collect::<Vec<_>>();

    let output = run_success(command(&home, &["init"]));
    let report = stdout(&output);

    assert_eq!(
        report,
        "Initialized canonical configuration with 0 servers from 5 client configurations.\n\
Skipped 1 unsupported Cursor entry: \"collision\".\n\
Skipped 1 unsupported Windsurf entry: \"collision\".\n\
Skipped 1 unsupported VS Code entry: \"collision\".\n\
Skipped 1 unsupported Codex entry: \"collision\".\n\
Skipped 1 unsupported Kiro entry: \"collision\".\n"
    );
    assert_omits(
        &report,
        &[
            "SYNTHETIC=NAME",
            "synthetic-value",
            "synthetic-argument",
            "synthetic-command",
        ],
    );
    assert_file_matches(
        &home.canonical_configuration(),
        EMPTY_CANONICAL,
        "invalid native definitions must not enter canonical state",
    );
    for (client, bytes) in native {
        assert_file_matches(
            &client.path(&home),
            &bytes,
            "init must preserve every unmanaged native entry exactly",
        );
    }
}

#[test]
fn sync_refuses_each_invalid_unmanaged_collision_before_any_target_mutation() {
    for client in [
        UnmanagedClient::Cursor,
        UnmanagedClient::Windsurf,
        UnmanagedClient::VsCode,
        UnmanagedClient::Codex,
        UnmanagedClient::Kiro,
    ] {
        let home = SyntheticHome::new();
        let canonical = json_document(&json!({
            "schemaVersion": 1,
            "servers": {
                "collision": {
                    "command": "private-desired-command",
                    "args": ["private-desired-argument"],
                    "env": {"VALID": "private-desired-value"}
                }
            }
        }));
        let native = client.invalid_document();
        home.write_file(&home.canonical_configuration(), &canonical);
        home.write_file(&client.path(&home), &native);

        let output = run_failure(command(&home, &["sync"]));
        let diagnostic = stderr(&output);

        assert!(diagnostic.contains(client.label()));
        assert!(diagnostic.contains("unmanaged"));
        assert!(diagnostic.contains("collision"));
        assert!(diagnostic.contains("no target files were changed"));
        assert_omits(
            &diagnostic,
            &[
                "SYNTHETIC=NAME",
                "synthetic-value",
                "synthetic-argument",
                "synthetic-command",
                "private-desired-command",
                "private-desired-argument",
                "private-desired-value",
            ],
        );
        assert_file_matches(
            &home.canonical_configuration(),
            &canonical,
            "collision planning must preserve canonical bytes",
        );
        assert_file_matches(
            &client.path(&home),
            &native,
            "collision planning must preserve unmanaged native bytes",
        );
        for path in managed_target_paths(&home) {
            if path != client.path(&home) {
                assert!(
                    !path.exists(),
                    "planning failure must not create an earlier or later target"
                );
            }
            assert!(!backup_path(&path).exists());
        }
    }
}

#[test]
fn canonical_invalid_name_blocks_list_sync_and_test_without_mutation_or_execution() {
    let home = SyntheticHome::new();
    let marker = home.root().join("invalid-server-started");
    let (process_command, process_arguments) = process_sentinel(&home, &marker);
    let canonical = json_document(&json!({
        "schemaVersion": 1,
        "servers": {
            "invalid": {
                "command": process_command,
                "args": process_arguments,
                "env": {"private=name": "private-value"}
            }
        }
    }));
    let target_sentinels = managed_target_paths(&home)
        .into_iter()
        .enumerate()
        .map(|(index, path)| {
            let bytes = format!("target-{index}-private-sentinel\n").into_bytes();
            home.write_file(&path, &bytes);
            (path, bytes)
        })
        .collect::<Vec<_>>();
    home.write_file(&home.canonical_configuration(), &canonical);

    for arguments in [
        vec!["list"],
        vec!["sync", "--dry-run"],
        vec!["sync"],
        vec!["test", "invalid"],
    ] {
        let output = run_failure(command(&home, &arguments));
        let diagnostic = stderr(&output);
        assert!(diagnostic.contains("environment name 0 for server \"invalid\""));
        assert!(diagnostic.contains("must not contain `=`"));
        assert_omits(
            &diagnostic,
            &[
                "private=name",
                "private-value",
                "private-sentinel",
                marker.to_string_lossy().as_ref(),
            ],
        );
        assert_file_matches(
            &home.canonical_configuration(),
            &canonical,
            "invalid canonical reads must preserve exact canonical bytes",
        );
        for (path, bytes) in &target_sentinels {
            assert_file_matches(
                path,
                bytes,
                "invalid canonical reads must not change any native target",
            );
            assert!(!backup_path(path).exists());
        }
        assert!(
            !marker.exists(),
            "invalid canonical state must never execute"
        );
    }
}

#[cfg(unix)]
fn process_sentinel(home: &SyntheticHome, marker: &Path) -> (String, Vec<String>) {
    let script = home.root().join("invalid-environment-server.sh");
    home.write_file(
        &script,
        format!("#!/bin/sh\nprintf started > '{}'\n", marker.display()),
    );
    use std::os::unix::fs::PermissionsExt as _;
    fs::set_permissions(&script, fs::Permissions::from_mode(0o700))
        .expect("the process sentinel should be executable");
    (script.to_string_lossy().into_owned(), Vec::new())
}

#[cfg(windows)]
fn process_sentinel(home: &SyntheticHome, marker: &Path) -> (String, Vec<String>) {
    let script = home.root().join("invalid-environment-server.ps1");
    home.write_file(
        &script,
        format!(
            "[IO.File]::WriteAllText('{}', 'started')\n",
            marker.display().to_string().replace('\'', "''")
        ),
    );
    (
        "powershell.exe".to_owned(),
        vec![
            "-NoLogo".to_owned(),
            "-NoProfile".to_owned(),
            "-NonInteractive".to_owned(),
            "-File".to_owned(),
            script.to_string_lossy().into_owned(),
        ],
    )
}
