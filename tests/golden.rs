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

fn add_target_only_server(
    path: &Path,
    server_map_field: &str,
    name: &str,
    command: &str,
    private_value: &str,
) {
    let mut document: Value =
        serde_json::from_slice(&read(path)).expect("the native fixture should be valid JSON");
    document[server_map_field]
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

fn add_codex_target_only_server(path: &Path, command: &str, private_value: &str) {
    let mut document = String::from_utf8(read(path)).expect("the Codex fixture should be UTF-8");
    document.push_str(&format!(
        "\n[mcp_servers.target-only]\ncommand = {}\nargs = [\"--target-only-private-argument\"]\ncwd = \"/synthetic/codex-target-only\"\n\n[mcp_servers.target-only.env]\nTARGET_ONLY_TOKEN = {}\n",
        serde_json::to_string(command).expect("the synthetic command should quote as TOML"),
        serde_json::to_string(private_value)
            .expect("the synthetic environment value should quote as TOML")
    ));
    document
        .parse::<toml_edit::DocumentMut>()
        .expect("the updated Codex fixture should be valid TOML");
    fs::write(path, document).expect("the Codex fixture should be updated");
}

fn kiro_parse_options() -> jsonc_parser::ParseOptions {
    jsonc_parser::ParseOptions {
        allow_comments: true,
        allow_loose_object_property_names: false,
        allow_trailing_commas: true,
        allow_missing_commas: false,
        allow_single_quoted_strings: false,
        allow_hexadecimal_numbers: false,
        allow_unary_plus_numbers: false,
    }
}

fn parse_kiro_bytes(bytes: &[u8]) -> (String, Value) {
    let text = String::from_utf8(bytes.to_vec()).expect("Kiro bytes should be UTF-8");
    let root = jsonc_parser::cst::CstRootNode::parse(&text, &kiro_parse_options())
        .expect("Kiro bytes should be valid comment-bearing JSON");
    let value = root
        .value()
        .and_then(|value| value.to_serde_value())
        .expect("Kiro bytes should contain one JSON value");
    (text, value)
}

fn add_kiro_target_only_server(path: &Path, command: &str, private_value: &str) {
    use jsonc_parser::cst::CstInputValue;

    let text = String::from_utf8(read(path)).expect("the Kiro fixture should be UTF-8");
    let root = jsonc_parser::cst::CstRootNode::parse(&text, &kiro_parse_options())
        .expect("the Kiro fixture should parse");
    let servers = root
        .object_value()
        .and_then(|root| root.object_value("mcpServers"))
        .expect("the Kiro fixture should contain mcpServers");
    servers.append(
        "target-only",
        CstInputValue::Object(vec![
            (
                "command".to_owned(),
                CstInputValue::String(command.to_owned()),
            ),
            (
                "args".to_owned(),
                CstInputValue::Array(vec![CstInputValue::String(
                    "--target-only-private-argument".to_owned(),
                )]),
            ),
            (
                "env".to_owned(),
                CstInputValue::Object(vec![(
                    "TARGET_ONLY_TOKEN".to_owned(),
                    CstInputValue::String(private_value.to_owned()),
                )]),
            ),
            (
                "futureNativeField".to_owned(),
                CstInputValue::String("target-only-unowned-private-value".to_owned()),
            ),
        ]),
    );
    fs::write(path, root.to_string()).expect("the Kiro fixture should be updated");
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
    let vscode = json_document(&json!({
        "servers": {
            "delta": {
                "type": "stdio",
                "command": "delta-private-command",
                "args": ["--delta-private-argument"],
                "env": {"DELTA_TOKEN": "delta-private-value"},
                "cwd": "${workspaceFolder}"
            },
            "vscode-native-env": {
                "type": "stdio",
                "command": "vscode-native-private-command",
                "env": {"OPTIONAL": null, "PORT": 43117}
            },
            "vscode-remote-only": {
                "type": "http",
                "url": "https://vscode-remote-private.invalid/mcp",
                "headers": {"Authorization": "Bearer vscode-remote-private-value"}
            },
            "shared": {
                "command": imported_command,
                "args": ["--shared-imported-private-argument"],
                "env": {"SHARED_TOKEN": "shared-imported-private-value"},
                "envFile": "${userHome}/shared-unowned-private.env"
            }
        },
        "inputs": [{"type": "promptString", "id": "vscode-unowned-private-input"}],
        "sandbox": {"network": {"allowedDomains": ["vscode-unowned-private.invalid"]}},
        "vscodeMetadata": {"private": "vscode-unowned-private-value"}
    }));
    let quoted_imported_command =
        serde_json::to_string(&imported_command).expect("the synthetic command should quote");
    let codex = format!(
        r#"# Codex global comment must survive.
model = "codex-unowned-private-model" # unrelated inline comment
future_root = {{ private = "codex-unowned-private-value" }}

[mcp_servers.epsilon]
command = "epsilon-private-command"
args = ["--epsilon-private-argument"]
cwd = "/synthetic/epsilon-unowned-private"

[mcp_servers.epsilon.env]
EPSILON_TOKEN = "epsilon-private-value"

[mcp_servers.shared]
command = {quoted_imported_command}
args = ["--shared-imported-private-argument"]
enabled = false
startup_timeout_sec = 21

[mcp_servers.shared.env]
SHARED_TOKEN = "shared-imported-private-value"

[mcp_servers.codex-remote-only]
url = "https://codex-remote-private.invalid/mcp"
bearer_token_env_var = "CODEX_REMOTE_TOKEN"
http_headers = {{ Authorization = "Bearer codex-remote-private-value" }}

[mcp_servers.codex-mixed-transport]
command = "codex-mixed-private-command"
url = "https://codex-mixed-private.invalid/mcp"
args = ["--codex-mixed-private-argument"]

[mcp_servers.codex-opaque]
enabled = false
future_transport = "codex-opaque-private-value"

[plugins."golden@test".mcp_servers.hosted]
enabled = true
private = "codex-plugin-private-value"
"#
    )
    .into_bytes();
    let kiro = format!(
        r#"{{
  // Kiro global comment must survive.
  "kiroMetadata": {{"private": "kiro-unowned-private-value"}},
  "mcpServers": {{
    "zeta": {{
      "command": "zeta-private-command",
      "args": ["--zeta-private-argument"],
      "env": {{"ZETA_TOKEN": "zeta-private-value"}},
      "disabled": false
    }},
    "shared": {{
      "command": {quoted_imported_command},
      "args": ["--shared-imported-private-argument"],
      "env": {{"SHARED_TOKEN": "shared-imported-private-value"}},
      "cwd": "/synthetic/kiro-shared-unowned-private",
      "autoApprove": ["read"]
    }},
    "kiro-remote-only": {{
      "url": "https://kiro-remote-private.invalid/mcp",
      "headers": {{"Authorization": "Bearer kiro-remote-private-value"}}
    }},
    "kiro-reference-only": {{
      "command": "${{KIRO_GOLDEN_RUNNER}}",
      "env": {{"TOKEN": "${{KIRO_GOLDEN_TOKEN}}"}},
      "disabled": true
    }},
    "kiro-native-env": {{
      "command": "kiro-native-private-command",
      "env": {{"PORT": 43118}}
    }},
  }},
}}
"#
    )
    .into_bytes();
    let cursor_project_path = home.user_root().join("workspace/.cursor/mcp.json");
    let cursor_project = b"{\"projectPrivateSentinel\":\"project-unowned-private-value\"}\n";
    let codex_project_path = home.user_root().join("workspace/.codex/config.toml");
    let codex_project =
        b"[mcp_servers.project-only]\ncommand = \"codex-project-private-command\"\n";
    let codex_profile_path = home.user_root().join(".codex/profiles/private.toml");
    let codex_profile = b"profile_private = \"codex-profile-private-value\"\n";
    let codex_auth_path = home.user_root().join(".codex/auth.json");
    let codex_auth = b"{\"tokens\":{\"access_token\":\"codex-auth-private-value\"}}\n";
    let kiro_workspace_path = home.user_root().join("workspace/.kiro/settings/mcp.json");
    let kiro_workspace = b"{\"workspacePrivate\":\"kiro-workspace-private-value\"}\n";
    let kiro_crew_path = home.user_root().join(".kiro/crew/mcp.json");
    let kiro_crew = b"{\"crewPrivate\":\"kiro-crew-private-value\"}\n";
    let kiro_agent_path = home.user_root().join(".kiro/agents/kirocrew.json");
    let kiro_agent = b"{\"agentPrivate\":\"kiro-agent-private-value\"}\n";
    home.write_file(&home.claude_desktop_configuration(), &claude);
    home.write_file(&home.cursor_configuration(), &cursor);
    home.write_file(&home.windsurf_configuration(), &windsurf);
    home.write_file(&home.vscode_configuration(), &vscode);
    home.write_file(&home.codex_configuration(), &codex);
    home.write_file(&home.kiro_configuration(), &kiro);
    home.write_file(&cursor_project_path, cursor_project);
    home.write_file(&codex_project_path, codex_project);
    home.write_file(&codex_profile_path, codex_profile);
    home.write_file(&codex_auth_path, codex_auth);
    home.write_file(&kiro_workspace_path, kiro_workspace);
    home.write_file(&kiro_crew_path, kiro_crew);
    home.write_file(&kiro_agent_path, kiro_agent);

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
        "delta-private-command",
        "--delta-private-argument",
        "delta-private-value",
        "epsilon-private-command",
        "--epsilon-private-argument",
        "epsilon-private-value",
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
        "vscode-native-private-command",
        "43117",
        "https://vscode-remote-private.invalid/mcp",
        "Bearer vscode-remote-private-value",
        "https://codex-remote-private.invalid/mcp",
        "Bearer codex-remote-private-value",
        "codex-mixed-private-command",
        "--codex-mixed-private-argument",
        "https://codex-mixed-private.invalid/mcp",
        "codex-opaque-private-value",
        "zeta-private-command",
        "--zeta-private-argument",
        "zeta-private-value",
        "kiro-native-private-command",
        "43118",
        "https://kiro-remote-private.invalid/mcp",
        "Bearer kiro-remote-private-value",
        "${KIRO_GOLDEN_RUNNER}",
        "${KIRO_GOLDEN_TOKEN}",
        "/synthetic/alpha-unowned-private",
        "/synthetic/beta-unowned-private.env",
        "/synthetic/epsilon-unowned-private",
        "/synthetic/shared-unowned-private",
        "/synthetic/shared-unowned-private.env",
        "${userHome}/shared-unowned-private.env",
        "claude-unowned-private-value",
        "cursor-unowned-private-value",
        "windsurf-unowned-private-value",
        "vscode-unowned-private-value",
        "codex-unowned-private-model",
        "codex-unowned-private-value",
        "codex-plugin-private-value",
        "kiro-unowned-private-value",
        "/synthetic/kiro-shared-unowned-private",
        "vscode-unowned-private-input",
        "vscode-unowned-private.invalid",
        "project-unowned-private-value",
        "codex-project-private-command",
        "codex-profile-private-value",
        "codex-auth-private-value",
        "kiro-workspace-private-value",
        "kiro-crew-private-value",
        "kiro-agent-private-value",
        "claude-target-only-private-command",
        "cursor-target-only-private-command",
        "windsurf-target-only-private-command",
        "vscode-target-only-private-command",
        "codex-target-only-private-command",
        "kiro-target-only-private-command",
        "claude-target-only-private-value",
        "cursor-target-only-private-value",
        "windsurf-target-only-private-value",
        "vscode-target-only-private-value",
        "codex-target-only-private-value",
        "kiro-target-only-private-value",
        "target-only-unowned-private-value",
    ];

    let init_output = stdout(&run_success(cli_command(&home, &["init"])));
    assert!(
        init_output.starts_with(
            "Initialized canonical configuration with 7 servers from 6 client configurations.\n"
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
    assert!(
        init_output.contains(
            "Skipped 2 unsupported VS Code entries: \"vscode-native-env\", \"vscode-remote-only\"."
        ),
        "init should report unsupported VS Code structures by name"
    );
    assert!(
        init_output.contains(
            "Skipped 3 unsupported Codex entries: \"codex-mixed-transport\", \"codex-opaque\", \"codex-remote-only\"."
        ),
        "init should report unsupported Codex structures by name"
    );
    assert!(
        init_output.contains(
            "Skipped 3 unsupported Kiro entries: \"kiro-native-env\", \"kiro-reference-only\", \"kiro-remote-only\"."
        ),
        "init should report unsupported Kiro structures by name"
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
        &home.vscode_configuration(),
        &vscode,
        "init must preserve VS Code bytes",
    );
    assert_file_matches(
        &home.codex_configuration(),
        &codex,
        "init must preserve Codex bytes",
    );
    assert_file_matches(
        &home.kiro_configuration(),
        &kiro,
        "init must preserve exact Kiro JSONC bytes",
    );
    assert_file_matches(
        &cursor_project_path,
        cursor_project,
        "init must preserve project Cursor bytes",
    );
    assert_file_matches(
        &codex_project_path,
        codex_project,
        "init must preserve project Codex bytes",
    );
    assert_file_matches(
        &codex_profile_path,
        codex_profile,
        "init must preserve alternate Codex profile bytes",
    );
    assert_file_matches(
        &codex_auth_path,
        codex_auth,
        "init must preserve Codex credential-store bytes",
    );
    assert_file_matches(
        &kiro_workspace_path,
        kiro_workspace,
        "init must preserve workspace Kiro bytes",
    );
    assert_file_matches(
        &kiro_crew_path,
        kiro_crew,
        "init must preserve Crew-only bytes",
    );
    assert_file_matches(
        &kiro_agent_path,
        kiro_agent,
        "init must preserve generated-agent bytes",
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
        "\"delta\":",
        "\"epsilon\":",
        "\"gamma\":",
        "\"shared\":",
        "\"zeta\":",
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
        "mcpServers",
        "target-only",
        "claude-target-only-private-command",
        "claude-target-only-private-value",
    );
    add_target_only_server(
        &home.cursor_configuration(),
        "mcpServers",
        "target-only",
        "cursor-target-only-private-command",
        "cursor-target-only-private-value",
    );
    add_target_only_server(
        &home.windsurf_configuration(),
        "mcpServers",
        "target-only",
        "windsurf-target-only-private-command",
        "windsurf-target-only-private-value",
    );
    add_target_only_server(
        &home.vscode_configuration(),
        "servers",
        "target-only",
        "vscode-target-only-private-command",
        "vscode-target-only-private-value",
    );
    add_codex_target_only_server(
        &home.codex_configuration(),
        "codex-target-only-private-command",
        "codex-target-only-private-value",
    );
    add_kiro_target_only_server(
        &home.kiro_configuration(),
        "kiro-target-only-private-command",
        "kiro-target-only-private-value",
    );

    let canonical_path = home.canonical_configuration();
    let canonical_backup = backup_path(&canonical_path);
    let claude_path = home.claude_desktop_configuration();
    let cursor_path = home.cursor_configuration();
    let windsurf_path = home.windsurf_configuration();
    let vscode_path = home.vscode_configuration();
    let codex_path = home.codex_configuration();
    let kiro_path = home.kiro_configuration();
    let claude_backup = backup_path(&claude_path);
    let cursor_backup = backup_path(&cursor_path);
    let windsurf_backup = backup_path(&windsurf_path);
    let vscode_backup = backup_path(&vscode_path);
    let codex_backup = backup_path(&codex_path);
    let kiro_backup = backup_path(&kiro_path);
    let canonical_before_sync = read(&canonical_path);
    let canonical_backup_before_sync = read(&canonical_backup);
    let claude_before_sync = read(&claude_path);
    let cursor_before_sync = read(&cursor_path);
    let windsurf_before_sync = read(&windsurf_path);
    let vscode_before_sync = read(&vscode_path);
    let codex_before_sync = read(&codex_path);
    let kiro_before_sync = read(&kiro_path);

    let dry_output = stdout(&run_success(cli_command(&home, &["sync", "--dry-run"])));
    assert!(
        dry_output.starts_with("Dry run validated 6 targets; no files changed.\n"),
        "dry-run should validate the complete six-target plan"
    );
    assert!(
        dry_output.contains("Claude Desktop: would update with recoverable backup")
            && dry_output.contains("Cursor: would update with recoverable backup")
            && dry_output.contains("Windsurf: would update with recoverable backup")
            && dry_output.contains("VS Code: would update with recoverable backup")
            && dry_output.contains("Codex: would update with recoverable backup")
            && dry_output.contains("Kiro: would update with recoverable backup")
            && dry_output.contains("preserve target-only \"target-only\"")
            && dry_output.contains("preserve unmanaged \"remote-only\"")
            && dry_output.contains("preserve unmanaged \"windsurf-remote-only\"")
            && dry_output.contains("preserve unmanaged \"vscode-native-env\"")
            && dry_output.contains("preserve unmanaged \"vscode-remote-only\"")
            && dry_output.contains("preserve unmanaged \"codex-mixed-transport\"")
            && dry_output.contains("preserve unmanaged \"codex-opaque\"")
            && dry_output.contains("preserve unmanaged \"codex-remote-only\"")
            && dry_output.contains("preserve unmanaged \"kiro-native-env\"")
            && dry_output.contains("preserve unmanaged \"kiro-reference-only\"")
            && dry_output.contains("preserve unmanaged \"kiro-remote-only\""),
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
    assert_file_matches(
        &vscode_path,
        &vscode_before_sync,
        "dry-run must preserve VS Code",
    );
    assert_file_matches(
        &codex_path,
        &codex_before_sync,
        "dry-run must preserve Codex",
    );
    assert_file_matches(
        &kiro_path,
        &kiro_before_sync,
        "dry-run must preserve exact Kiro bytes",
    );
    assert_file_matches(
        &codex_project_path,
        codex_project,
        "dry-run must preserve project Codex bytes",
    );
    assert_file_matches(
        &codex_profile_path,
        codex_profile,
        "dry-run must preserve alternate Codex profile bytes",
    );
    assert_file_matches(
        &codex_auth_path,
        codex_auth,
        "dry-run must preserve Codex credential-store bytes",
    );
    assert_file_matches(
        &kiro_workspace_path,
        kiro_workspace,
        "dry-run must preserve workspace Kiro bytes",
    );
    assert_file_matches(
        &kiro_crew_path,
        kiro_crew,
        "dry-run must preserve Crew-only bytes",
    );
    assert_file_matches(
        &kiro_agent_path,
        kiro_agent,
        "dry-run must preserve generated-agent bytes",
    );
    assert!(
        !claude_backup.exists()
            && !cursor_backup.exists()
            && !windsurf_backup.exists()
            && !vscode_backup.exists()
            && !codex_backup.exists()
            && !kiro_backup.exists()
    );

    let apply_output = stdout(&run_success(cli_command(&home, &["sync"])));
    assert!(
        apply_output.starts_with("Sync completed for 6 targets.\n")
            && apply_output.contains("Claude Desktop: updated with recoverable backup")
            && apply_output.contains("Cursor: updated with recoverable backup")
            && apply_output.contains("Windsurf: updated with recoverable backup")
            && apply_output.contains("VS Code: updated with recoverable backup")
            && apply_output.contains("Codex: updated with recoverable backup")
            && apply_output.contains("Kiro: updated with recoverable backup"),
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
        &vscode_backup,
        &vscode_before_sync,
        "VS Code backup must contain exact pre-sync bytes",
    );
    assert_file_matches(
        &codex_backup,
        &codex_before_sync,
        "Codex backup must contain exact pre-sync bytes",
    );
    assert_file_matches(
        &kiro_backup,
        &kiro_before_sync,
        "Kiro backup must contain exact pre-sync comment-bearing bytes",
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
        &cursor_project_path,
        cursor_project,
        "sync must preserve project Cursor bytes",
    );
    assert_file_matches(
        &codex_project_path,
        codex_project,
        "sync must preserve project Codex bytes",
    );
    assert_file_matches(
        &codex_profile_path,
        codex_profile,
        "sync must preserve alternate Codex profile bytes",
    );
    assert_file_matches(
        &codex_auth_path,
        codex_auth,
        "sync must preserve Codex credential-store bytes",
    );
    assert_file_matches(
        &kiro_workspace_path,
        kiro_workspace,
        "sync must preserve workspace Kiro bytes",
    );
    assert_file_matches(
        &kiro_crew_path,
        kiro_crew,
        "sync must preserve Crew-only bytes",
    );
    assert_file_matches(
        &kiro_agent_path,
        kiro_agent,
        "sync must preserve generated-agent bytes",
    );

    let claude_after: Value =
        serde_json::from_slice(&read(&claude_path)).expect("Claude output should be valid JSON");
    let cursor_after: Value =
        serde_json::from_slice(&read(&cursor_path)).expect("Cursor output should be valid JSON");
    let windsurf_after: Value = serde_json::from_slice(&read(&windsurf_path))
        .expect("Windsurf output should be valid JSON");
    let vscode_after: Value =
        serde_json::from_slice(&read(&vscode_path)).expect("VS Code output should be valid JSON");
    let codex_after_bytes = read(&codex_path);
    let codex_after_text =
        String::from_utf8(codex_after_bytes.clone()).expect("Codex output should be UTF-8");
    let codex_after = codex_after_text
        .parse::<toml_edit::DocumentMut>()
        .expect("Codex output should be valid TOML");
    let (kiro_after_text, kiro_after) = parse_kiro_bytes(&read(&kiro_path));
    for target in [&claude_after, &cursor_after, &windsurf_after] {
        assert!(
            target["mcpServers"]["shared"]["command"].as_str() == Some(updated_command.as_str())
                && target["mcpServers"]["added"]["command"].as_str()
                    == Some(added_command.as_str())
                && target["mcpServers"]["alpha"].is_object()
                && target["mcpServers"]["beta"].is_object()
                && target["mcpServers"]["delta"].is_object()
                && target["mcpServers"]["epsilon"].is_object()
                && target["mcpServers"]["gamma"].is_object()
                && target["mcpServers"]["zeta"].is_object(),
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
    assert!(
        vscode_after["servers"]["shared"]["command"].as_str() == Some(updated_command.as_str())
            && vscode_after["servers"]["added"]["command"].as_str() == Some(added_command.as_str())
            && vscode_after["servers"]["added"]["type"].as_str() == Some("stdio")
            && vscode_after["servers"]["alpha"].is_object()
            && vscode_after["servers"]["beta"].is_object()
            && vscode_after["servers"]["delta"].is_object()
            && vscode_after["servers"]["epsilon"].is_object()
            && vscode_after["servers"]["gamma"].is_object()
            && vscode_after["servers"]["zeta"].is_object(),
        "VS Code should contain the complete desired managed definitions"
    );
    assert!(
        vscode_after["vscodeMetadata"]["private"].as_str() == Some("vscode-unowned-private-value")
            && vscode_after["servers"]["shared"]["envFile"].as_str()
                == Some("${userHome}/shared-unowned-private.env")
            && vscode_after["servers"]["vscode-remote-only"]["headers"]["Authorization"].as_str()
                == Some("Bearer vscode-remote-private-value")
            && vscode_after["servers"]["vscode-native-env"]["env"]["PORT"].as_u64() == Some(43117)
            && vscode_after["servers"]["target-only"]["env"]["TARGET_ONLY_TOKEN"].as_str()
                == Some("vscode-target-only-private-value")
            && vscode_after["inputs"][0]["id"].as_str() == Some("vscode-unowned-private-input")
            && vscode_after["sandbox"]["network"]["allowedDomains"][0].as_str()
                == Some("vscode-unowned-private.invalid"),
        "VS Code root, local, unmanaged, and target-only native data should survive"
    );
    let codex_servers = codex_after["mcp_servers"]
        .as_table_like()
        .expect("Codex should retain its server map");
    assert!(
        codex_after["mcp_servers"]["shared"]["command"].as_str() == Some(updated_command.as_str())
            && codex_after["mcp_servers"]["added"]["command"].as_str()
                == Some(added_command.as_str())
            && ["alpha", "beta", "delta", "epsilon", "gamma", "zeta"]
                .into_iter()
                .all(|name| codex_servers.contains_key(name)),
        "Codex should contain the complete desired managed definitions"
    );
    assert!(
        codex_after["model"].as_str() == Some("codex-unowned-private-model")
            && codex_after["future_root"]["private"].as_str()
                == Some("codex-unowned-private-value")
            && codex_after["mcp_servers"]["shared"]["enabled"].as_bool() == Some(false)
            && codex_after["mcp_servers"]["shared"]["startup_timeout_sec"].as_integer() == Some(21)
            && codex_after["mcp_servers"]["codex-remote-only"]["http_headers"]["Authorization"]
                .as_str()
                == Some("Bearer codex-remote-private-value")
            && codex_after["mcp_servers"]["codex-mixed-transport"]["url"].as_str()
                == Some("https://codex-mixed-private.invalid/mcp")
            && codex_after["mcp_servers"]["codex-opaque"]["future_transport"].as_str()
                == Some("codex-opaque-private-value")
            && codex_after["mcp_servers"]["target-only"]["env"]["TARGET_ONLY_TOKEN"].as_str()
                == Some("codex-target-only-private-value")
            && codex_after["plugins"]["golden@test"]["mcp_servers"]["hosted"]["private"].as_str()
                == Some("codex-plugin-private-value")
            && codex_after_text.contains("# Codex global comment must survive.")
            && codex_after_text.contains("# unrelated inline comment"),
        "Codex comments, root settings, unowned local fields, unmanaged entries, plug-ins, and target-only data should survive"
    );
    assert!(
        kiro_after["mcpServers"]["shared"]["command"].as_str() == Some(updated_command.as_str())
            && kiro_after["mcpServers"]["added"]["command"].as_str()
                == Some(added_command.as_str())
            && ["alpha", "beta", "delta", "epsilon", "gamma", "zeta"]
                .into_iter()
                .all(|name| kiro_after["mcpServers"][name].is_object()),
        "Kiro should contain the complete desired managed definitions"
    );
    assert!(
        kiro_after["kiroMetadata"]["private"].as_str() == Some("kiro-unowned-private-value")
            && kiro_after["mcpServers"]["shared"]["cwd"].as_str()
                == Some("/synthetic/kiro-shared-unowned-private")
            && kiro_after["mcpServers"]["shared"]["autoApprove"][0].as_str() == Some("read")
            && kiro_after["mcpServers"]["kiro-remote-only"]["headers"]["Authorization"].as_str()
                == Some("Bearer kiro-remote-private-value")
            && kiro_after["mcpServers"]["kiro-reference-only"]["command"].as_str()
                == Some("${KIRO_GOLDEN_RUNNER}")
            && kiro_after["mcpServers"]["kiro-native-env"]["env"]["PORT"].as_u64() == Some(43118)
            && kiro_after["mcpServers"]["target-only"]["env"]["TARGET_ONLY_TOKEN"].as_str()
                == Some("kiro-target-only-private-value")
            && kiro_after_text.contains("// Kiro global comment must survive."),
        "Kiro comments, root settings, unowned fields, references, remote entries, native values, and target-only data should survive"
    );

    let claude_after_bytes = read(&claude_path);
    let cursor_after_bytes = read(&cursor_path);
    let windsurf_after_bytes = read(&windsurf_path);
    let vscode_after_bytes = read(&vscode_path);
    let kiro_after_bytes = read(&kiro_path);
    let claude_backup_after = read(&claude_backup);
    let cursor_backup_after = read(&cursor_backup);
    let windsurf_backup_after = read(&windsurf_backup);
    let vscode_backup_after = read(&vscode_backup);
    let codex_backup_after = read(&codex_backup);
    let kiro_backup_after = read(&kiro_backup);
    let no_op_output = stdout(&run_success(cli_command(&home, &["sync"])));
    assert!(
        no_op_output
            .matches("unchanged; no write or backup")
            .count()
            == 6,
        "repeat sync should report six target no-ops"
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
        &vscode_path,
        &vscode_after_bytes,
        "repeat sync must preserve VS Code bytes",
    );
    assert_file_matches(
        &codex_path,
        &codex_after_bytes,
        "repeat sync must preserve Codex bytes",
    );
    assert_file_matches(
        &kiro_path,
        &kiro_after_bytes,
        "repeat sync must preserve exact Kiro bytes",
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
    assert_file_matches(
        &vscode_backup,
        &vscode_backup_after,
        "repeat sync must preserve the VS Code backup",
    );
    assert_file_matches(
        &codex_backup,
        &codex_backup_after,
        "repeat sync must preserve the Codex backup",
    );
    assert_file_matches(
        &kiro_backup,
        &kiro_backup_after,
        "repeat sync must preserve the Kiro backup",
    );
    assert_file_matches(
        &codex_project_path,
        codex_project,
        "repeat sync must preserve project Codex bytes",
    );
    assert_file_matches(
        &codex_profile_path,
        codex_profile,
        "repeat sync must preserve alternate Codex profile bytes",
    );
    assert_file_matches(
        &codex_auth_path,
        codex_auth,
        "repeat sync must preserve Codex credential-store bytes",
    );
    assert_file_matches(
        &kiro_workspace_path,
        kiro_workspace,
        "repeat sync must preserve workspace Kiro bytes",
    );
    assert_file_matches(
        &kiro_crew_path,
        kiro_crew,
        "repeat sync must preserve Crew-only bytes",
    );
    assert_file_matches(
        &kiro_agent_path,
        kiro_agent,
        "repeat sync must preserve generated-agent bytes",
    );
    for marker in [&imported_marker, &updated_marker, &added_marker] {
        assert!(
            !marker.exists(),
            "configuration commands must never execute servers"
        );
    }
    for path in [
        &canonical_path,
        &claude_path,
        &cursor_path,
        &windsurf_path,
        &vscode_path,
        &codex_path,
        &kiro_path,
    ] {
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
    let delta = json!({
        "type": "stdio",
        "command": "delta-private-command",
        "args": ["--delta-private-argument"],
        "env": {"DELTA_TOKEN": "delta-private-value"}
    });
    let zeta = json!({
        "command": "zeta-private-command",
        "args": ["--zeta-private-argument"],
        "env": {"ZETA_TOKEN": "zeta-private-value"}
    });
    let shared = json!({
        "command": "shared-private-command",
        "args": ["--shared-private-argument"],
        "env": {"SHARED_TOKEN": "shared-private-value"}
    });
    let codex = br#"[mcp_servers.epsilon]
command = "epsilon-private-command"
args = ["--epsilon-private-argument"]
env = { EPSILON_TOKEN = "epsilon-private-value" }

[mcp_servers.shared]
command = "shared-private-command"
args = ["--shared-private-argument"]
env = { SHARED_TOKEN = "shared-private-value" }
"#;
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
    first.write_file(
        &first.vscode_configuration(),
        json_document(&json!({"servers": {"delta": delta, "shared": shared}})),
    );
    first.write_file(&first.codex_configuration(), codex);
    first.write_file(
        &first.kiro_configuration(),
        json_document(&json!({"mcpServers": {"zeta": zeta, "shared": shared}})),
    );
    second.write_file(
        &second.claude_desktop_configuration(),
        json_document(&json!({"mcpServers": {"delta": {
            "command": "delta-private-command",
            "args": ["--delta-private-argument"],
            "env": {"DELTA_TOKEN": "delta-private-value"}
        }, "shared": shared}})),
    );
    second.write_file(
        &second.cursor_configuration(),
        json_document(&json!({"mcpServers": {"gamma": gamma, "shared": shared}})),
    );
    second.write_file(
        &second.windsurf_configuration(),
        json_document(&json!({"mcpServers": {"beta": beta, "shared": shared}})),
    );
    second.write_file(
        &second.vscode_configuration(),
        json_document(&json!({"servers": {"alpha": alpha, "shared": shared}})),
    );
    second.write_file(&second.codex_configuration(), codex);
    second.write_file(
        &second.kiro_configuration(),
        json_document(&json!({"mcpServers": {"zeta": zeta, "shared": shared}})),
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
        "delta-private-command",
        "--delta-private-argument",
        "delta-private-value",
        "epsilon-private-command",
        "--epsilon-private-argument",
        "epsilon-private-value",
        "zeta-private-command",
        "--zeta-private-argument",
        "zeta-private-value",
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
        home.write_file(&home.vscode_configuration(), native);
        home.write_file(&home.codex_configuration(), native);
        home.write_file(&home.kiro_configuration(), native);
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
        assert_file_matches(
            &home.vscode_configuration(),
            native,
            "missing canonical failure must preserve VS Code",
        );
        assert_file_matches(
            &home.codex_configuration(),
            native,
            "missing canonical failure must preserve Codex",
        );
        assert_file_matches(
            &home.kiro_configuration(),
            native,
            "missing canonical failure must preserve Kiro",
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
        let windsurf = json_document(&json!({
            "mcpServers": {
                "shared": {
                    "command": "windsurf-current-private-command",
                    "args": ["--windsurf-current-private-argument"],
                    "env": {"TOKEN": "windsurf-current-private-value"}
                }
            }
        }));
        let malformed_vscode =
            b"{\"servers\":{\"shared\":{\"type\":\"stdio\",\"command\":\"malformed-private-command\"";
        let codex = b"[mcp_servers.shared]\ncommand = \"codex-current-private-command\"\nargs = [\"--codex-current-private-argument\"]\nenv = { TOKEN = \"codex-current-private-value\" }\n";
        let kiro = b"{\n  // Kiro preflight preservation\n  \"mcpServers\": {\n    \"shared\": {\n      \"command\": \"kiro-current-private-command\",\n      \"args\": [\"--kiro-current-private-argument\"],\n      \"env\": {\"TOKEN\": \"kiro-current-private-value\"},\n      \"disabled\": false\n    }\n  }\n}\n";
        home.write_file(&home.canonical_configuration(), &canonical);
        home.write_file(&home.claude_desktop_configuration(), &claude);
        home.write_file(&home.cursor_configuration(), &cursor);
        home.write_file(&home.windsurf_configuration(), &windsurf);
        home.write_file(&home.vscode_configuration(), malformed_vscode);
        home.write_file(&home.codex_configuration(), codex);
        home.write_file(&home.kiro_configuration(), kiro);
        let diagnostic = stderr(&run_failure(cli_command(&home, &["sync"])));
        assert!(
            diagnostic.contains("cannot plan VS Code sync: invalid VS Code JSON:")
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
                "windsurf-current-private-command",
                "--windsurf-current-private-argument",
                "windsurf-current-private-value",
                "malformed-private-command",
                "codex-current-private-command",
                "--codex-current-private-argument",
                "codex-current-private-value",
                "kiro-current-private-command",
                "--kiro-current-private-argument",
                "kiro-current-private-value",
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
            &windsurf,
            "preflight failure must preserve Windsurf",
        );
        assert_file_matches(
            &home.vscode_configuration(),
            malformed_vscode,
            "preflight failure must preserve malformed VS Code",
        );
        assert_file_matches(
            &home.codex_configuration(),
            codex,
            "preflight failure must preserve Codex",
        );
        assert_file_matches(
            &home.kiro_configuration(),
            kiro,
            "preflight failure must preserve Kiro",
        );
        assert!(!backup_path(&home.claude_desktop_configuration()).exists());
        assert!(!backup_path(&home.cursor_configuration()).exists());
        assert!(!backup_path(&home.windsurf_configuration()).exists());
        assert!(!backup_path(&home.vscode_configuration()).exists());
        assert!(!backup_path(&home.codex_configuration()).exists());
        assert!(!backup_path(&home.kiro_configuration()).exists());
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
