mod support;

use assert_cmd::Command as AssertCommand;
use serde_json::{Value, json};
use std::fs;
use std::path::Path;
use std::process::Output;
use std::time::Duration;
use support::{SyntheticHome, assert_file_matches};

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
fn init_imports_all_clients_without_executing_servers_or_touching_native_files() {
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
    let windsurf = json_document(&json!({
        "mcpServers": {
            "gamma": {
                "command": "gamma-server",
                "args": ["--gamma"],
                "env": {"WINDSURF_TOKEN": "windsurf-synthetic-value"},
                "disabledTools": ["preserve_tool"]
            },
            "remote-windsurf": {
                "serverUrl": "https://windsurf.invalid.example.test/mcp",
                "headers": {"Authorization": "Bearer windsurf-remote-synthetic-value"}
            },
            "shared": {
                "command": "shared-server",
                "args": ["--stdio"],
                "env": {"MODE": "synthetic"}
            }
        },
        "windsurfMetadata": {"synthetic": true}
    }));
    let vscode = json_document(&json!({
        "servers": {
            "delta": {
                "type": "stdio",
                "command": "delta-server",
                "args": ["--delta"],
                "env": {"VSCODE_TOKEN": "vscode-synthetic-value"},
                "cwd": "${workspaceFolder}"
            },
            "native-env": {
                "type": "stdio",
                "command": "native-env-server",
                "env": {"OPTIONAL": null, "PORT": 3000}
            },
            "remote-vscode": {
                "type": "http",
                "url": "https://vscode.invalid.example.test/mcp",
                "headers": {"Authorization": "Bearer vscode-remote-synthetic-value"}
            },
            "shared": {
                "command": "shared-server",
                "args": ["--stdio"],
                "env": {"MODE": "synthetic"},
                "envFile": "${userHome}/.synthetic-env"
            }
        },
        "inputs": [{"type": "promptString", "id": "synthetic-input"}],
        "sandbox": {"enabled": true}
    }));
    let codex = br#"# global Codex fixture comment
model = "synthetic-model"

[mcp_servers.epsilon]
command = "epsilon-server"
args = ["--epsilon"]
cwd = "/synthetic/codex-unowned"

[mcp_servers.epsilon.env]
CODEX_TOKEN = "codex-synthetic-value"

[mcp_servers.shared]
command = "shared-server"
args = ["--stdio"]
enabled = false

[mcp_servers.shared.env]
MODE = "synthetic"

[mcp_servers.remote-codex]
url = "https://codex.invalid.example.test/mcp"
auth = "oauth"
http_headers = { Authorization = "Bearer codex-remote-synthetic-value" }

[mcp_servers.mixed-codex]
command = "codex-mixed-private-command"
url = "https://codex.invalid.example.test/mixed"

[mcp_servers.opaque-codex]
enabled = false
future_transport = "codex-future-private-value"
"#;
    let cursor_project_path = home.user_root().join("workspace/.cursor/mcp.json");
    let cursor_project_bytes = b"{\"projectSentinel\":true}\n";
    let codex_project_path = home.user_root().join("workspace/.codex/config.toml");
    let codex_project_bytes = include_bytes!("fixtures/codex/project.toml");
    let codex_profile_path = home.user_root().join(".codex/review.config.toml");
    let codex_profile_bytes = b"model = \"profile-sentinel\"\n";
    let codex_auth_path = home.user_root().join(".codex/auth.json");
    let codex_auth_bytes = b"{\"credentialSentinel\":\"must-remain-private\"}\n";
    let excluded_vscode_bytes = b"not native default-profile VS Code configuration\n";
    let excluded_vscode_paths = [
        home.user_root().join("workspace/.vscode/mcp.json"),
        home.user_root().join("workspace/.mcp.json"),
        home.user_data_home()
            .join("Code/User/profiles/synthetic-profile/mcp.json"),
        home.user_data_home().join("Code - Insiders/User/mcp.json"),
        home.user_data_home().join(
            "Code/User/globalStorage/saoudrizwan.claude-dev/settings/cline_mcp_settings.json",
        ),
        home.user_root()
            .join(".cline/data/settings/cline_mcp_settings.json"),
        home.user_root().join(".copilot/mcp-config.json"),
    ];
    home.write_file(&home.claude_desktop_configuration(), &claude);
    home.write_file(&home.cursor_configuration(), &cursor);
    home.write_file(&home.windsurf_configuration(), &windsurf);
    home.write_file(&home.vscode_configuration(), &vscode);
    home.write_file(&home.codex_configuration(), codex);
    home.write_file(&cursor_project_path, cursor_project_bytes);
    home.write_file(&codex_project_path, codex_project_bytes);
    home.write_file(&codex_profile_path, codex_profile_bytes);
    home.write_file(&codex_auth_path, codex_auth_bytes);
    for path in &excluded_vscode_paths {
        home.write_file(path, excluded_vscode_bytes);
    }

    let output = stdout(&run_success(init_command(&home)));
    assert!(
        output
            == "Initialized canonical configuration with 6 servers from 5 client configurations.\n\
                Skipped 1 unsupported Cursor entry: \"remote-only\".\n\
                Skipped 1 unsupported Windsurf entry: \"remote-windsurf\".\n\
                Skipped 2 unsupported VS Code entries: \"native-env\", \"remote-vscode\".\n\
                Skipped 3 unsupported Codex entries: \"mixed-codex\", \"opaque-codex\", \"remote-codex\".\n",
        "init success output should be exact and structural"
    );

    let canonical_path = home.canonical_configuration();
    let canonical_bytes = fs::read(&canonical_path)
        .expect("successful initialization should create the canonical configuration");
    let canonical: Value =
        serde_json::from_slice(&canonical_bytes).expect("canonical output should be valid JSON");
    assert!(
        canonical
            == json!({
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
                    "gamma": {
                        "command": "gamma-server",
                        "args": ["--gamma"],
                        "env": {"WINDSURF_TOKEN": "windsurf-synthetic-value"}
                    },
                    "delta": {
                        "command": "delta-server",
                        "args": ["--delta"],
                        "env": {"VSCODE_TOKEN": "vscode-synthetic-value"}
                    },
                    "epsilon": {
                        "command": "epsilon-server",
                        "args": ["--epsilon"],
                        "env": {"CODEX_TOKEN": "codex-synthetic-value"}
                    },
                    "shared": {
                        "command": "shared-server",
                        "args": ["--stdio"],
                        "env": {"MODE": "synthetic"}
                    }
                }
            }),
        "canonical import should preserve the normalized definitions"
    );
    assert!(canonical_bytes.ends_with(b"\n"));
    assert_file_matches(
        &home.claude_desktop_configuration(),
        &claude,
        "init should preserve Claude Desktop bytes",
    );
    assert_file_matches(
        &home.cursor_configuration(),
        &cursor,
        "init should preserve Cursor bytes",
    );
    assert_file_matches(
        &home.windsurf_configuration(),
        &windsurf,
        "init should preserve Windsurf bytes",
    );
    assert_file_matches(
        &home.vscode_configuration(),
        &vscode,
        "init should preserve VS Code bytes",
    );
    assert_file_matches(
        &home.codex_configuration(),
        codex,
        "init should preserve global Codex bytes",
    );
    assert_file_matches(
        &cursor_project_path,
        cursor_project_bytes,
        "init should preserve project Cursor bytes",
    );
    assert_file_matches(
        &codex_project_path,
        codex_project_bytes,
        "init should preserve project Codex bytes",
    );
    assert_file_matches(
        &codex_profile_path,
        codex_profile_bytes,
        "init should preserve Codex profile bytes",
    );
    assert_file_matches(
        &codex_auth_path,
        codex_auth_bytes,
        "init should never access or mutate Codex credential bytes",
    );
    for path in &excluded_vscode_paths {
        assert_file_matches(
            path,
            excluded_vscode_bytes,
            "init should preserve every excluded VS Code configuration shape",
        );
    }
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
    let vscode = json_document(&json!({
        "servers": {
            "vscode-only": {
                "type": "stdio",
                "command": "vscode-private-command",
                "args": ["--vscode-private-argument"],
                "env": {"VSCODE_TOKEN": "vscode-private-value"}
            }
        }
    }));
    let windsurf = json_document(&json!({
        "mcpServers": {
            "windsurf-only": {
                "command": "windsurf-private-command",
                "args": ["--windsurf-private-argument"],
                "env": {"WINDSURF_TOKEN": "windsurf-private-value"}
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
    home.write_file(&home.windsurf_configuration(), &windsurf);
    home.write_file(&home.vscode_configuration(), &vscode);
    home.write_file(&project_path, project_bytes);

    let expected_stderr = "error: cannot initialize because server \"shared\" differs between Claude Desktop and Cursor in command, arguments, environment keys, and environment values; make the definitions identical, rename one, or remove one, then rerun `mcp-sync init`\n";
    let diagnostic = stderr(&run_failure(init_command(&home)));
    assert!(
        diagnostic == expected_stderr,
        "conflict output should be exact and structural"
    );

    assert!(!home.canonical_configuration().exists());
    assert!(
        !home
            .canonical_configuration()
            .parent()
            .expect("the canonical path should have a parent")
            .exists()
    );
    assert_file_matches(
        &home.claude_desktop_configuration(),
        &claude,
        "conflict handling should preserve Claude Desktop bytes",
    );
    assert_file_matches(
        &home.cursor_configuration(),
        &cursor,
        "conflict handling should preserve Cursor bytes",
    );
    assert_file_matches(
        &home.windsurf_configuration(),
        &windsurf,
        "conflict handling should preserve Windsurf bytes",
    );
    assert_file_matches(
        &home.vscode_configuration(),
        &vscode,
        "conflict handling should preserve VS Code bytes",
    );
    assert_file_matches(
        &project_path,
        project_bytes,
        "conflict handling should preserve project Cursor bytes",
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
        "windsurf-private-command",
        "--windsurf-private-argument",
        "windsurf-private-value",
        "vscode-private-command",
        "--vscode-private-argument",
        "vscode-private-value",
    ] {
        assert!(!expected_stderr.contains(private_value));
    }
}

#[test]
fn init_rejects_malformed_final_vscode_json_without_creating_canonical_state() {
    let home = SyntheticHome::new();
    let malformed = br#"{
  "servers": {
    "malformed": {
      "type": "stdio",
      "command": "synthetic-command",
      "env": {"SYNTHETIC_TOKEN": "must-not-appear",}
    }
  }
}
"#;
    let claude = json_document(&json!({
        "mcpServers": {
            "claude-only": {"command": "synthetic-claude-command"}
        }
    }));
    let cursor = json_document(&json!({
        "mcpServers": {
            "cursor-only": {"command": "synthetic-cursor-command"}
        }
    }));
    let windsurf = json_document(&json!({
        "mcpServers": {
            "windsurf-only": {"command": "synthetic-windsurf-command"}
        }
    }));
    home.write_file(&home.claude_desktop_configuration(), &claude);
    home.write_file(&home.cursor_configuration(), &cursor);
    home.write_file(&home.windsurf_configuration(), &windsurf);
    home.write_file(&home.vscode_configuration(), malformed);

    let stderr = stderr(&run_failure(init_command(&home)));

    assert!(
        stderr.starts_with("error: cannot import VS Code configuration: invalid VS Code JSON:")
    );
    assert!(stderr.ends_with("; fix the file or its permissions, then rerun `mcp-sync init`\n"));
    assert!(!stderr.contains("must-not-appear"));
    assert!(!home.canonical_configuration().exists());
    assert_file_matches(
        &home.claude_desktop_configuration(),
        &claude,
        "malformed input should preserve Claude Desktop bytes",
    );
    assert_file_matches(
        &home.cursor_configuration(),
        &cursor,
        "malformed input should preserve Cursor bytes",
    );
    assert_file_matches(
        &home.windsurf_configuration(),
        &windsurf,
        "malformed later input should preserve Windsurf bytes",
    );
    assert_file_matches(
        &home.vscode_configuration(),
        malformed,
        "malformed input should preserve VS Code bytes",
    );
}

#[test]
fn init_rejects_malformed_codex_toml_without_exposing_values_or_mutating_any_layer() {
    let home = SyntheticHome::new();
    let claude = json_document(&json!({
        "mcpServers": {"claude-only": {"command": "synthetic-claude-command"}}
    }));
    let vscode = json_document(&json!({
        "servers": {"vscode-only": {"type": "stdio", "command": "synthetic-vscode-command"}}
    }));
    let malformed = b"private = \"must-not-appear\"\n[mcp_servers.fixture\ncommand = \"safe\"\n";
    let project_path = home.user_root().join("workspace/.codex/config.toml");
    let project_bytes = include_bytes!("fixtures/codex/project.toml");
    let auth_path = home.user_root().join(".codex/auth.json");
    let auth_bytes = b"{\"token\":\"credential-must-not-appear\"}\n";
    home.write_file(&home.claude_desktop_configuration(), &claude);
    home.write_file(&home.vscode_configuration(), &vscode);
    home.write_file(&home.codex_configuration(), malformed);
    home.write_file(&project_path, project_bytes);
    home.write_file(&auth_path, auth_bytes);

    let diagnostic = stderr(&run_failure(init_command(&home)));

    assert!(
        diagnostic
            .starts_with("error: cannot import Codex configuration: invalid Codex TOML near byte ")
    );
    assert!(
        diagnostic.ends_with("; fix the file or its permissions, then rerun `mcp-sync init`\n")
    );
    for private in ["must-not-appear", "credential-must-not-appear"] {
        assert!(!diagnostic.contains(private));
    }
    assert!(!home.canonical_configuration().exists());
    assert_file_matches(
        &home.claude_desktop_configuration(),
        &claude,
        "malformed Codex input should preserve Claude Desktop bytes",
    );
    assert_file_matches(
        &home.vscode_configuration(),
        &vscode,
        "malformed Codex input should preserve VS Code bytes",
    );
    assert_file_matches(
        &home.codex_configuration(),
        malformed,
        "malformed Codex input should remain unchanged",
    );
    assert_file_matches(
        &project_path,
        project_bytes,
        "malformed global input should preserve project Codex bytes",
    );
    assert_file_matches(
        &auth_path,
        auth_bytes,
        "malformed global input should preserve Codex credential bytes",
    );
}

#[test]
fn init_rejects_a_local_collision_with_an_unmanaged_vscode_entry() {
    let home = SyntheticHome::new();
    let claude = json_document(&json!({
        "mcpServers": {
            "native-collision": {
                "command": "local-private-command",
                "args": ["--local-private-argument"],
                "env": {"TOKEN": "local-private-value"}
            }
        }
    }));
    let vscode = json_document(&json!({
        "servers": {
            "native-collision": {
                "type": "stdio",
                "command": "native-private-command",
                "env": {"OPTIONAL": null, "PORT": 43117}
            }
        }
    }));
    home.write_file(&home.claude_desktop_configuration(), &claude);
    home.write_file(&home.vscode_configuration(), &vscode);

    let diagnostic = stderr(&run_failure(init_command(&home)));

    assert_eq!(
        diagnostic,
        "error: cannot initialize because server \"native-collision\" is both a local Claude Desktop definition and an unsupported VS Code entry; make the definitions identical, rename one, or remove one, then rerun `mcp-sync init`\n"
    );
    for private in [
        "local-private-command",
        "--local-private-argument",
        "local-private-value",
        "native-private-command",
        "43117",
    ] {
        assert!(!diagnostic.contains(private));
    }
    assert!(!home.canonical_configuration().exists());
    assert_file_matches(
        &home.claude_desktop_configuration(),
        &claude,
        "collision handling should preserve Claude Desktop bytes",
    );
    assert_file_matches(
        &home.vscode_configuration(),
        &vscode,
        "collision handling should preserve VS Code bytes",
    );
}

#[test]
fn init_rejects_a_local_collision_with_an_unmanaged_codex_entry() {
    let home = SyntheticHome::new();
    let claude = json_document(&json!({
        "mcpServers": {
            "remote-collision": {
                "command": "local-private-command",
                "args": ["--local-private-argument"],
                "env": {"TOKEN": "local-private-value"}
            }
        }
    }));
    let codex = br#"[mcp_servers.remote-collision]
url = "https://codex.invalid.example.test/private"
auth = "oauth"
http_headers = { Authorization = "Bearer codex-private-value" }
"#;
    let project_path = home.user_root().join("workspace/.codex/config.toml");
    let project_bytes = include_bytes!("fixtures/codex/project.toml");
    home.write_file(&home.claude_desktop_configuration(), &claude);
    home.write_file(&home.codex_configuration(), codex);
    home.write_file(&project_path, project_bytes);

    let diagnostic = stderr(&run_failure(init_command(&home)));

    assert_eq!(
        diagnostic,
        "error: cannot initialize because server \"remote-collision\" is both a local Claude Desktop definition and an unsupported Codex entry; make the definitions identical, rename one, or remove one, then rerun `mcp-sync init`\n"
    );
    for private in [
        "local-private-command",
        "--local-private-argument",
        "local-private-value",
        "https://codex.invalid.example.test/private",
        "codex-private-value",
    ] {
        assert!(!diagnostic.contains(private));
    }
    assert!(!home.canonical_configuration().exists());
    assert_file_matches(
        &home.claude_desktop_configuration(),
        &claude,
        "Codex collision handling should preserve Claude Desktop bytes",
    );
    assert_file_matches(
        &home.codex_configuration(),
        codex,
        "Codex collision handling should preserve global Codex bytes",
    );
    assert_file_matches(
        &project_path,
        project_bytes,
        "Codex collision handling should preserve project Codex bytes",
    );
}

#[test]
fn init_creates_a_valid_empty_config_when_all_clients_are_missing() {
    let home = SyntheticHome::new();

    let output = stdout(&run_success(init_command(&home)));
    assert!(
        output
            == "Initialized canonical configuration with 0 servers from 0 client configurations.\n",
        "empty initialization output should be exact"
    );

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
    let diagnostic = stderr(&run_failure(init_command(&home)));
    assert!(
        diagnostic == expected_stderr,
        "existing canonical state should produce an exact diagnostic"
    );

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

    let stderr = stderr(&run_failure(init_command(&home)));

    fs::set_permissions(&claude_path, fs::Permissions::from_mode(0o600))
        .expect("the fixture permissions should be restored");
    assert!(stderr.starts_with("error: cannot import Claude Desktop configuration:"));
    assert!(stderr.contains("could not read file"));
    assert!(stderr.contains("Permission denied"));
    assert!(!home.canonical_configuration().exists());
    assert_file_matches(
        &claude_path,
        &claude,
        "permission failure should preserve Claude Desktop bytes",
    );
}
