mod support;

use assert_cmd::Command as AssertCommand;
use serde_json::{Value, json};
use std::fs;
use std::path::Path;
use std::time::Duration;
use support::SyntheticHome;

const COMMAND_TIMEOUT: Duration = Duration::from_secs(5);

fn json_document(value: &Value) -> Vec<u8> {
    let mut bytes = serde_json::to_vec_pretty(value).expect("synthetic JSON should serialize");
    bytes.push(b'\n');
    bytes
}

fn init_command(home: &SyntheticHome) -> AssertCommand {
    let mut command = AssertCommand::from_std(home.command());
    command.arg("init").timeout(COMMAND_TIMEOUT);
    command
}

fn assert_no_temporary_files(destination: &Path) {
    let parent = destination
        .parent()
        .expect("the canonical configuration has a parent");
    if !parent.exists() {
        return;
    }

    let unexpected = fs::read_dir(parent)
        .expect("the canonical directory should be readable")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name())
        .filter(|name| name.to_string_lossy().starts_with(".mcp-sync-"))
        .collect::<Vec<_>>();
    assert!(
        unexpected.is_empty(),
        "temporary files should be cleaned up"
    );
}

#[test]
fn init_imports_both_clients_without_executing_servers_or_touching_native_files() {
    let home = SyntheticHome::new();
    let marker = home.root().join("configured-server-was-started");
    let sentinel = home.root().join("synthetic-mcp-server");
    home.write_file(
        &sentinel,
        format!("#!/bin/sh\nprintf started > '{}'\n", marker.display()),
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        fs::set_permissions(&sentinel, fs::Permissions::from_mode(0o700))
            .expect("the sentinel server should be executable");
    }

    let claude = json_document(&json!({
        "mcpServers": {
            "alpha": {
                "command": sentinel.to_string_lossy(),
                "args": ["--alpha"],
                "env": {"SYNTHETIC_TOKEN": "claude-synthetic-value"},
                "cwd": "/synthetic/unowned"
            },
            "shared": {
                "command": "shared-server",
                "args": ["--stdio"],
                "env": {"MODE": "synthetic"}
            }
        },
        "preferences": {"synthetic": true}
    }));
    let cursor = json_document(&json!({
        "mcpServers": {
            "beta": {
                "command": "beta-server",
                "args": [],
                "env": {}
            },
            "remote-only": {
                "url": "https://invalid.example.test/mcp",
                "headers": {"Authorization": "Bearer cursor-synthetic-value"}
            },
            "shared": {
                "command": "shared-server",
                "args": ["--stdio"],
                "env": {"MODE": "synthetic"},
                "envFile": ".synthetic-env"
            }
        },
        "cursorMetadata": 7
    }));
    let project_path = home.user_root().join("workspace/.cursor/mcp.json");
    let project_bytes = b"{\"projectSentinel\":true}\n";
    home.write_file(&home.claude_desktop_configuration(), &claude);
    home.write_file(&home.cursor_configuration(), &cursor);
    home.write_file(&project_path, project_bytes);

    init_command(&home)
        .assert()
        .success()
        .stdout(
            "Initialized canonical configuration with 3 servers from 2 client configurations.\n\
             Skipped 1 unsupported Cursor entry: \"remote-only\".\n",
        )
        .stderr("");

    let canonical_path = home.canonical_configuration();
    let canonical_bytes = fs::read(&canonical_path)
        .expect("successful initialization should create the canonical configuration");
    let canonical: Value =
        serde_json::from_slice(&canonical_bytes).expect("canonical output should be valid JSON");
    assert_eq!(
        canonical,
        json!({
            "schemaVersion": 1,
            "servers": {
                "alpha": {
                    "command": sentinel.to_string_lossy(),
                    "args": ["--alpha"],
                    "env": {"SYNTHETIC_TOKEN": "claude-synthetic-value"}
                },
                "beta": {
                    "command": "beta-server",
                    "args": [],
                    "env": {}
                },
                "shared": {
                    "command": "shared-server",
                    "args": ["--stdio"],
                    "env": {"MODE": "synthetic"}
                }
            }
        })
    );
    assert!(canonical_bytes.ends_with(b"\n"));
    assert_eq!(
        fs::read(home.claude_desktop_configuration())
            .expect("Claude Desktop configuration should remain readable"),
        claude
    );
    assert_eq!(
        fs::read(home.cursor_configuration()).expect("Cursor configuration should remain readable"),
        cursor
    );
    assert_eq!(
        fs::read(&project_path).expect("project Cursor sentinel should remain readable"),
        project_bytes
    );
    assert!(!marker.exists(), "init must not execute configured servers");
    assert_no_temporary_files(&canonical_path);
}

#[test]
fn init_reports_an_exact_redacted_conflict_and_mutates_nothing() {
    let home = SyntheticHome::new();
    let marker = home.root().join("conflicting-server-was-started");
    let claude_command = home.root().join("claude-private-command");
    home.write_file(
        &claude_command,
        format!("#!/bin/sh\nprintf started > '{}'\n", marker.display()),
    );
    let claude = json_document(&json!({
        "mcpServers": {
            "shared": {
                "command": claude_command.to_string_lossy(),
                "args": ["--claude-private-argument"],
                "env": {"SYNTHETIC_TOKEN": "claude-private-value"}
            }
        }
    }));
    let cursor = json_document(&json!({
        "mcpServers": {
            "shared": {
                "command": "cursor-private-command",
                "args": ["--cursor-private-argument"],
                "env": {
                    "CURSOR_ONLY": "cursor-only-private-value",
                    "SYNTHETIC_TOKEN": "cursor-private-value"
                }
            }
        }
    }));
    let project_path = home.user_root().join("workspace/.cursor/mcp.json");
    let project_bytes = b"{\"projectSentinel\":\"unchanged\"}\n";
    home.write_file(&home.claude_desktop_configuration(), &claude);
    home.write_file(&home.cursor_configuration(), &cursor);
    home.write_file(&project_path, project_bytes);

    let expected_stderr = "error: cannot initialize because server \"shared\" differs between Claude Desktop and Cursor in command, arguments, environment keys, and environment values; make the definitions identical, rename one, or remove one, then rerun `mcp-sync init`\n";
    init_command(&home)
        .assert()
        .failure()
        .stdout("")
        .stderr(expected_stderr);

    assert!(!home.canonical_configuration().exists());
    assert!(!home.user_root().join(".config/mcp-sync").exists());
    assert_eq!(
        fs::read(home.claude_desktop_configuration())
            .expect("Claude Desktop configuration should remain readable"),
        claude
    );
    assert_eq!(
        fs::read(home.cursor_configuration()).expect("Cursor configuration should remain readable"),
        cursor
    );
    assert_eq!(
        fs::read(&project_path).expect("project Cursor sentinel should remain readable"),
        project_bytes
    );
    assert!(
        !marker.exists(),
        "conflict handling must not execute servers"
    );
    for private_value in [
        claude_command.to_string_lossy().as_ref(),
        "cursor-private-command",
        "--claude-private-argument",
        "--cursor-private-argument",
        "claude-private-value",
        "cursor-private-value",
        "cursor-only-private-value",
    ] {
        assert!(!expected_stderr.contains(private_value));
    }
}

#[test]
fn init_rejects_malformed_native_json_without_creating_canonical_state() {
    let home = SyntheticHome::new();
    let malformed = br#"{
  "mcpServers": {
    "malformed": {
      "command": "synthetic-command",
      "env": {"SYNTHETIC_TOKEN": "must-not-appear",}
    }
  }
}
"#;
    let cursor = json_document(&json!({
        "mcpServers": {
            "cursor-only": {"command": "synthetic-cursor-command"}
        }
    }));
    home.write_file(&home.claude_desktop_configuration(), malformed);
    home.write_file(&home.cursor_configuration(), &cursor);

    let mut command = init_command(&home);
    let assertion = command.assert().failure().stdout("");
    let stderr = String::from_utf8(assertion.get_output().stderr.clone())
        .expect("diagnostics should be UTF-8");

    assert!(stderr.starts_with(
        "error: cannot import Claude Desktop configuration: invalid Claude Desktop JSON:"
    ));
    assert!(stderr.ends_with("; fix the file or its permissions, then rerun `mcp-sync init`\n"));
    assert!(!stderr.contains("must-not-appear"));
    assert!(!home.canonical_configuration().exists());
    assert_eq!(
        fs::read(home.claude_desktop_configuration())
            .expect("malformed Claude bytes should remain readable"),
        malformed
    );
    assert_eq!(
        fs::read(home.cursor_configuration()).expect("Cursor bytes should remain readable"),
        cursor
    );
}

#[test]
fn init_creates_a_valid_empty_config_when_both_clients_are_missing() {
    let home = SyntheticHome::new();

    init_command(&home)
        .assert()
        .success()
        .stdout(
            "Initialized canonical configuration with 0 servers from 0 client configurations.\n",
        )
        .stderr("");

    assert_eq!(
        fs::read_to_string(home.canonical_configuration())
            .expect("empty initialization should create a canonical config"),
        "{\n  \"schemaVersion\": 1,\n  \"servers\": {}\n}\n"
    );
}

#[test]
fn init_never_replaces_an_existing_canonical_configuration() {
    let home = SyntheticHome::new();
    let canonical = b"existing canonical sentinel\n";
    home.write_file(&home.canonical_configuration(), canonical);
    home.write_file(
        &home.claude_desktop_configuration(),
        b"native configuration must not be inspected\n",
    );

    let expected_stderr = format!(
        "error: canonical configuration already exists at `{}`; move or remove it before rerunning `mcp-sync init`\n",
        home.canonical_configuration().display()
    );
    init_command(&home)
        .assert()
        .failure()
        .stdout("")
        .stderr(expected_stderr);

    assert_eq!(
        fs::read(home.canonical_configuration())
            .expect("the existing canonical sentinel should remain readable"),
        canonical
    );
    assert_no_temporary_files(&home.canonical_configuration());
}

#[cfg(unix)]
#[test]
fn init_reports_a_native_permission_failure_without_mutation() {
    use std::os::unix::fs::PermissionsExt as _;

    let home = SyntheticHome::new();
    let claude_path = home.claude_desktop_configuration();
    let claude = json_document(&json!({
        "mcpServers": {"private": {"command": "synthetic-command"}}
    }));
    home.write_file(&claude_path, &claude);
    fs::set_permissions(&claude_path, fs::Permissions::from_mode(0o000))
        .expect("the fixture should become unreadable");

    let mut command = init_command(&home);
    let assertion = command.assert().failure().stdout("");
    let stderr = String::from_utf8(assertion.get_output().stderr.clone())
        .expect("diagnostics should be UTF-8");

    fs::set_permissions(&claude_path, fs::Permissions::from_mode(0o600))
        .expect("the fixture permissions should be restored");
    assert!(stderr.starts_with("error: cannot import Claude Desktop configuration:"));
    assert!(stderr.contains("could not read file"));
    assert!(stderr.contains("Permission denied"));
    assert!(!home.canonical_configuration().exists());
    assert_eq!(
        fs::read(claude_path).expect("restored Claude bytes should be readable"),
        claude
    );
}
