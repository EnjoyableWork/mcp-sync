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
const VSCODE_CURRENT: &[u8] = include_bytes!("fixtures/vscode/current.json");
const CODEX_CURRENT: &[u8] = include_bytes!("fixtures/codex/current.toml");
const KIRO_CURRENT: &[u8] = include_bytes!("fixtures/kiro/current.json");
const PROJECT_CURSOR: &[u8] = include_bytes!("fixtures/cursor/project.json");
const PROJECT_CODEX: &[u8] = include_bytes!("fixtures/codex/project.toml");

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
    vscode: Vec<u8>,
    codex: Vec<u8>,
    kiro: Vec<u8>,
    project_path: PathBuf,
    project_vscode_path: PathBuf,
    project_vscode: Vec<u8>,
    project_codex_path: PathBuf,
    codex_profile_path: PathBuf,
    codex_profile: Vec<u8>,
    codex_auth_path: PathBuf,
    codex_auth: Vec<u8>,
    excluded_kiro_paths: Vec<PathBuf>,
    excluded_kiro: Vec<u8>,
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
            "fixture-vscode-old-secret",
            "fixture-vscode-remove-secret",
            "fixture-vscode-local-secret",
            "fixture-vscode-remote-secret",
            "fixture-vscode-added-secret",
            "fixture-vscode-added-environment-secret",
            "fixture-vscode-new-secret",
            "fixture-vscode-unchanged-secret",
            "https://vscode.example.invalid/mcp",
            "Bearer fixture-vscode-remote-secret",
            "/synthetic/workspace/preserved",
            "/synthetic/env/preserved.env",
            "${userHome}/.synthetic-vscode.env",
            "fixture-codex-root-private",
            "fixture-codex-old-secret",
            "fixture-codex-shared-old",
            "fixture-codex-local-secret",
            "fixture-codex-mixed-secret",
            "fixture-codex-remote-secret",
            "fixture-codex-future-private",
            "https://codex.example.invalid/mcp",
            "https://codex.example.invalid/mixed",
            "codex-profile-private-value",
            "codex-auth-private-value",
            "fixture-kiro-old-secret",
            "fixture-kiro-remove-secret",
            "fixture-kiro-local-secret",
            "fixture-kiro-remote-secret",
            "fixture-kiro-added-secret",
            "fixture-kiro-added-environment-secret",
            "fixture-kiro-new-secret",
            "https://kiro.example.invalid/mcp",
            "${KIRO_FIXTURE_RUNNER}",
            "${KIRO_FIXTURE_ARGUMENT}",
            "${KIRO_FIXTURE_TOKEN}",
            "kiro-excluded-private-value",
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
    let vscode = String::from_utf8(VSCODE_CURRENT.to_vec())
        .expect("the VS Code fixture should be UTF-8")
        .replace(
            "fixture-vscode-unchanged-secret",
            "fixture-unchanged-secret",
        )
        .replace("\"remote-only\"", "\"vscode-remote-only\"")
        .into_bytes();
    let codex = String::from_utf8(CODEX_CURRENT.to_vec())
        .expect("the Codex fixture should be UTF-8")
        .replace("args = [\"--serve\"]", "args = [\"--mode\", \"safe\"]")
        .replace("fixture-codex-unchanged-secret", "fixture-unchanged-secret")
        .into_bytes();
    let kiro = String::from_utf8(KIRO_CURRENT.to_vec())
        .expect("the Kiro fixture should be UTF-8")
        .replace("fixture-kiro-unchanged-secret", "fixture-unchanged-secret")
        .replace("\"remote-only\"", "\"kiro-remote-only\"")
        .replace("\"reference-only\"", "\"kiro-reference-only\"")
        .replace("\"numeric-env\"", "\"kiro-numeric-env\"")
        .into_bytes();
    let project_path = home.user_root().join("workspace/.cursor/mcp.json");
    let project_vscode_path = home.user_root().join("workspace/.vscode/mcp.json");
    let project_vscode = b"{\"projectVsCodeSentinel\":\"unchanged\"}\n".to_vec();
    let project_codex_path = home.user_root().join("workspace/.codex/config.toml");
    let codex_profile_path = home.user_root().join(".codex/review.config.toml");
    let codex_profile = b"model = \"codex-profile-private-value\"\n".to_vec();
    let codex_auth_path = home.user_root().join(".codex/auth.json");
    let codex_auth = b"{\"token\":\"codex-auth-private-value\"}\n".to_vec();
    let excluded_kiro_paths = vec![
        home.user_root().join("workspace/.kiro/settings/mcp.json"),
        home.user_root().join(".kiro/crew/mcp.json"),
        home.user_root().join(".kiro/agents/kirocrew.json"),
        home.user_root().join(".kiro/agents/synthetic.json"),
    ];
    let excluded_kiro = b"{\"private\":\"kiro-excluded-private-value\"}\n".to_vec();

    home.write_file(&home.canonical_configuration(), &canonical);
    home.write_file(&home.claude_desktop_configuration(), &claude);
    home.write_file(&home.cursor_configuration(), &cursor);
    home.write_file(&home.windsurf_configuration(), &windsurf);
    home.write_file(&home.vscode_configuration(), &vscode);
    home.write_file(&home.codex_configuration(), &codex);
    home.write_file(&home.kiro_configuration(), &kiro);
    home.write_file(&project_path, PROJECT_CURSOR);
    home.write_file(&project_vscode_path, &project_vscode);
    home.write_file(&project_codex_path, PROJECT_CODEX);
    home.write_file(&codex_profile_path, &codex_profile);
    home.write_file(&codex_auth_path, &codex_auth);
    for path in &excluded_kiro_paths {
        home.write_file(path, &excluded_kiro);
    }

    ExistingJourney {
        canonical,
        claude,
        cursor,
        windsurf,
        vscode,
        codex,
        kiro,
        project_path,
        project_vscode_path,
        project_vscode,
        project_codex_path,
        codex_profile_path,
        codex_profile,
        codex_auth_path,
        codex_auth,
        excluded_kiro_paths,
        excluded_kiro,
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

fn assert_codex_exclusions_unchanged(fixture: &ExistingJourney, context: &str) {
    assert_file_matches(
        &fixture.project_codex_path,
        PROJECT_CODEX,
        &format!("{context} should preserve project Codex bytes"),
    );
    assert_file_matches(
        &fixture.codex_profile_path,
        &fixture.codex_profile,
        &format!("{context} should preserve Codex profile bytes"),
    );
    assert_file_matches(
        &fixture.codex_auth_path,
        &fixture.codex_auth,
        &format!("{context} should preserve Codex credential bytes"),
    );
}

fn assert_kiro_exclusions_unchanged(fixture: &ExistingJourney, context: &str) {
    for path in &fixture.excluded_kiro_paths {
        assert_file_matches(
            path,
            &fixture.excluded_kiro,
            &format!("{context} should preserve excluded Kiro workspace, agent, and Crew bytes"),
        );
    }
}

fn parse_kiro(path: &Path) -> (String, Value) {
    let text = fs::read_to_string(path).expect("Kiro output should be readable UTF-8");
    let root = jsonc_parser::cst::CstRootNode::parse(
        &text,
        &jsonc_parser::ParseOptions {
            allow_comments: true,
            allow_loose_object_property_names: false,
            allow_trailing_commas: true,
            allow_missing_commas: false,
            allow_single_quoted_strings: false,
            allow_hexadecimal_numbers: false,
            allow_unary_plus_numbers: false,
        },
    )
    .expect("Kiro output should remain valid comment-bearing JSON");
    let value = root
        .value()
        .and_then(|value| value.to_serde_value())
        .expect("Kiro output should have a JSON value");
    (text, value)
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
    let vscode: Value = serde_json::from_slice(
        &fs::read(home.vscode_configuration()).expect("VS Code output should be readable"),
    )
    .expect("VS Code output should remain valid JSON");
    let codex_text = fs::read_to_string(home.codex_configuration())
        .expect("Codex output should be readable UTF-8");
    let codex = codex_text
        .parse::<toml_edit::DocumentMut>()
        .expect("Codex output should remain valid TOML");
    let (kiro_text, kiro) = parse_kiro(&home.kiro_configuration());

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
        vscode["servers"]["added"]["command"].as_str() == Some(process_command)
            && vscode["servers"]["added"]["type"].as_str() == Some("stdio")
            && vscode["servers"]["updated"]["command"].as_str() == Some("/synthetic/bin/new")
            && vscode["servers"]["updated"]["args"][1].as_str() == Some("two")
            && vscode["servers"]["updated"]["env"]["ROTATE"].as_str() == Some("fixture-new-secret"),
        "managed VS Code process values should match canonical state"
    );
    assert!(
        codex["mcp_servers"]["added"]["command"].as_str() == Some(process_command)
            && codex["mcp_servers"]["updated"]["command"].as_str() == Some("/synthetic/bin/new")
            && codex["mcp_servers"]["updated"]["args"]
                .as_array()
                .and_then(|args| args.get(1))
                .and_then(toml_edit::Value::as_str)
                == Some("two")
            && codex["mcp_servers"]["updated"]["env"]["ROTATE"].as_str()
                == Some("fixture-new-secret"),
        "managed Codex process values should match canonical state"
    );
    assert!(
        kiro["mcpServers"]["added"]["command"].as_str() == Some(process_command)
            && kiro["mcpServers"]["updated"]["command"].as_str() == Some("/synthetic/bin/new")
            && kiro["mcpServers"]["updated"]["args"][1].as_str() == Some("two")
            && kiro["mcpServers"]["updated"]["env"]["ROTATE"].as_str()
                == Some("fixture-new-secret"),
        "managed Kiro process values should match canonical state"
    );

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
                == Some("https://windsurf.example.invalid/mcp")
            && vscode["servers"]["target-only"]["env"]["LOCAL_TOKEN"].as_str()
                == Some("fixture-vscode-local-secret")
            && vscode["servers"]["updated"]["envFile"].as_str()
                == Some("${userHome}/.synthetic-vscode.env")
            && vscode["servers"]["updated"]["dev"]["watch"].as_str() == Some("src/**/*.ts")
            && vscode["servers"]["vscode-remote-only"]["headers"]["Authorization"].as_str()
                == Some("Bearer fixture-vscode-remote-secret")
            && vscode["servers"]["numeric-env"]["env"]["PORT"].as_u64() == Some(3000)
            && vscode["inputs"][0]["id"].as_str() == Some("fixture-api-key")
            && vscode["sandbox"]["network"]["allowedDomains"][0].as_str()
                == Some("fixture.example.invalid")
            && codex["model"].as_str() == Some("synthetic-model")
            && codex["mcp_oauth_callback_port"].as_integer() == Some(5555)
            && codex["mcp_servers"]["target-only"]["env"]["LOCAL_TOKEN"].as_str()
                == Some("fixture-codex-local-secret")
            && codex["mcp_servers"]["updated"]["cwd"].as_str() == Some("/synthetic/updated")
            && codex["mcp_servers"]["updated"]["enabled"].as_bool() == Some(false)
            && codex["mcp_servers"]["remote-only"]["auth"].as_str() == Some("oauth")
            && codex["mcp_servers"]["remote-only"]["http_headers"]["X-Fixture"].as_str()
                == Some("fixture-codex-remote-secret")
            && codex["mcp_servers"]["mixed-transport"]["url"].as_str()
                == Some("https://codex.example.invalid/mixed")
            && codex["mcp_servers"]["opaque"]["future_transport"].as_str()
                == Some("fixture-codex-future-private")
            && codex["plugins"]["fixture@test"]["mcp_servers"]["hosted"]["enabled"].as_bool()
                == Some(true)
            && kiro["mcpServers"]["target-only"]["env"]["LOCAL_TOKEN"].as_str()
                == Some("fixture-kiro-local-secret")
            && kiro["mcpServers"]["updated"]["cwd"].as_str() == Some("/synthetic/kiro/preserved")
            && kiro["mcpServers"]["updated"]["autoApprove"][0].as_str() == Some("read")
            && kiro["mcpServers"]["updated"]["disabledTools"][0].as_str() == Some("write")
            && kiro["mcpServers"]["kiro-remote-only"]["headers"]["Authorization"].as_str()
                == Some("Bearer fixture-kiro-remote-secret")
            && kiro["mcpServers"]["kiro-reference-only"]["command"].as_str()
                == Some("${KIRO_FIXTURE_RUNNER}")
            && kiro["mcpServers"]["kiro-numeric-env"]["env"]["PORT"].as_u64() == Some(43117),
        "unowned, drift, and unmanaged native values should be preserved"
    );
    for comment in [
        "# Synthetic global Codex configuration. This comment must survive.",
        "# managed-field comment survives replacement",
        "# unowned local field",
    ] {
        assert!(
            codex_text.contains(comment),
            "Codex structural editing should preserve fixture comments"
        );
    }
    for comment in [
        "// Kiro global-user fixture comment",
        "// compatible local entries are owned only at command, args, and env",
    ] {
        assert!(
            kiro_text.contains(comment),
            "Kiro CST editing should preserve every fixture comment"
        );
    }
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
                })
            && vscode["futureTopLevel"]["preciseNumber"]
                .as_number()
                .is_some_and(|number| {
                    number.as_str() == "1234567890123456789012345678901234567890"
                })
            && kiro["futureTopLevel"]["preciseNumber"]
                .as_number()
                .is_some_and(|number| {
                    number.as_str() == "1234567890123456789012345678901234567890"
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
    let vscode_backup = backup_path(&home.vscode_configuration());
    let codex_backup = backup_path(&home.codex_configuration());
    let kiro_backup = backup_path(&home.kiro_configuration());

    let dry_output = stdout(&run_success(sync_command(&home, true)));

    assert!(dry_output.starts_with("Dry run validated 6 targets; no files changed.\n"));
    assert!(dry_output.contains("Claude Desktop: would update with recoverable backup"));
    assert!(dry_output.contains("Cursor: would update with recoverable backup"));
    assert!(dry_output.contains("Windsurf: would update with recoverable backup"));
    assert!(dry_output.contains("VS Code: would update with recoverable backup"));
    assert!(dry_output.contains("Codex: would update with recoverable backup"));
    assert!(dry_output.contains("Kiro: would update with recoverable backup"));
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
    assert!(dry_output.contains(
        "(add: 1; update: 1; unchanged: 1; drift preserved: 1; unmanaged preserved: 2)."
    ));
    assert!(dry_output.contains(
        "(add: 1; update: 1; unchanged: 1; drift preserved: 1; unmanaged preserved: 3)."
    ));
    assert!(dry_output.contains("add \"added\" (arguments: 1; environment keys: \"ADD_TOKEN\")"));
    assert!(dry_output.contains("update \"updated\" (command; arguments 1 -> 2"));
    assert!(dry_output.contains("environment keys updated \"ROTATE\""));
    assert!(dry_output.contains("preserve target-only \"target-only\""));
    assert!(dry_output.contains("preserve unmanaged \"remote-only\""));
    assert!(dry_output.contains("preserve unmanaged \"kiro-numeric-env\""));
    assert!(dry_output.contains("preserve unmanaged \"kiro-reference-only\""));
    assert!(dry_output.contains("preserve unmanaged \"kiro-remote-only\""));
    assert!(dry_output.contains("preserve unmanaged \"windsurf-remote-only\""));
    assert!(dry_output.contains("preserve unmanaged \"numeric-env\""));
    assert!(dry_output.contains("preserve unmanaged \"vscode-remote-only\""));
    assert!(dry_output.contains("preserve unmanaged \"mixed-transport\""));
    assert!(dry_output.contains("preserve unmanaged \"opaque\""));
    assert!(dry_output.contains("preserve unmanaged \"remote-only\""));
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
        &home.vscode_configuration(),
        &fixture.vscode,
        "dry-run should preserve VS Code bytes",
    );
    assert_file_matches(
        &home.codex_configuration(),
        &fixture.codex,
        "dry-run should preserve Codex bytes",
    );
    assert_file_matches(
        &home.kiro_configuration(),
        &fixture.kiro,
        "dry-run should preserve exact Kiro JSONC bytes",
    );
    assert_file_matches(
        &fixture.project_path,
        PROJECT_CURSOR,
        "dry-run should preserve project Cursor bytes",
    );
    assert_file_matches(
        &fixture.project_vscode_path,
        &fixture.project_vscode,
        "dry-run should preserve project VS Code bytes",
    );
    assert_codex_exclusions_unchanged(&fixture, "dry-run");
    assert_kiro_exclusions_unchanged(&fixture, "dry-run");
    assert!(!claude_backup.exists());
    assert!(!cursor_backup.exists());
    assert!(!windsurf_backup.exists());
    assert!(!vscode_backup.exists());
    assert!(!codex_backup.exists());
    assert!(!kiro_backup.exists());
    assert!(!fixture.process_marker.exists());

    let apply_output = stdout(&run_success(sync_command(&home, false)));

    assert!(apply_output.starts_with("Sync completed for 6 targets.\n"));
    assert!(apply_output.contains("Claude Desktop: updated with recoverable backup"));
    assert!(apply_output.contains("Cursor: updated with recoverable backup"));
    assert!(apply_output.contains("Windsurf: updated with recoverable backup"));
    assert!(apply_output.contains("VS Code: updated with recoverable backup"));
    assert!(apply_output.contains("Codex: updated with recoverable backup"));
    assert!(apply_output.contains("Kiro: updated with recoverable backup"));
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
        &vscode_backup,
        &fixture.vscode,
        "VS Code backup should contain exact prior bytes",
    );
    assert_file_matches(
        &codex_backup,
        &fixture.codex,
        "Codex backup should contain exact prior bytes",
    );
    assert_file_matches(
        &kiro_backup,
        &fixture.kiro,
        "Kiro backup should contain exact prior comment-bearing bytes",
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
    assert_file_matches(
        &fixture.project_vscode_path,
        &fixture.project_vscode,
        "apply should preserve project VS Code bytes",
    );
    assert_codex_exclusions_unchanged(&fixture, "apply");
    assert_kiro_exclusions_unchanged(&fixture, "apply");
    assert!(!fixture.process_marker.exists());
    assert_native_result(&home, &fixture.process_command);
    assert_no_temporary_files(&home.claude_desktop_configuration());
    assert_no_temporary_files(&home.cursor_configuration());
    assert_no_temporary_files(&home.windsurf_configuration());
    assert_no_temporary_files(&home.vscode_configuration());
    assert_no_temporary_files(&home.codex_configuration());
    assert_no_temporary_files(&home.kiro_configuration());

    let claude_after_apply = fs::read(home.claude_desktop_configuration()).unwrap();
    let cursor_after_apply = fs::read(home.cursor_configuration()).unwrap();
    let windsurf_after_apply = fs::read(home.windsurf_configuration()).unwrap();
    let vscode_after_apply = fs::read(home.vscode_configuration()).unwrap();
    let codex_after_apply = fs::read(home.codex_configuration()).unwrap();
    let kiro_after_apply = fs::read(home.kiro_configuration()).unwrap();
    let claude_backup_after_apply = fs::read(&claude_backup).unwrap();
    let cursor_backup_after_apply = fs::read(&cursor_backup).unwrap();
    let windsurf_backup_after_apply = fs::read(&windsurf_backup).unwrap();
    let vscode_backup_after_apply = fs::read(&vscode_backup).unwrap();
    let codex_backup_after_apply = fs::read(&codex_backup).unwrap();
    let kiro_backup_after_apply = fs::read(&kiro_backup).unwrap();
    let no_op_output = stdout(&run_success(sync_command(&home, false)));

    assert_eq!(
        no_op_output
            .matches("unchanged; no write or backup")
            .count(),
        6
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
    assert!(no_op_output.contains(
        "(add: 0; update: 0; unchanged: 3; drift preserved: 1; unmanaged preserved: 2)."
    ));
    assert!(no_op_output.contains(
        "(add: 0; update: 0; unchanged: 3; drift preserved: 1; unmanaged preserved: 3)."
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
        &home.vscode_configuration(),
        &vscode_after_apply,
        "no-op should preserve VS Code bytes",
    );
    assert_file_matches(
        &home.codex_configuration(),
        &codex_after_apply,
        "no-op should preserve Codex bytes",
    );
    assert_file_matches(
        &home.kiro_configuration(),
        &kiro_after_apply,
        "no-op should preserve exact Kiro bytes",
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
        &vscode_backup,
        &vscode_backup_after_apply,
        "no-op should preserve VS Code backup bytes",
    );
    assert_file_matches(
        &codex_backup,
        &codex_backup_after_apply,
        "no-op should preserve Codex backup bytes",
    );
    assert_file_matches(
        &kiro_backup,
        &kiro_backup_after_apply,
        "no-op should preserve Kiro backup bytes",
    );
    assert_file_matches(
        &fixture.project_path,
        PROJECT_CURSOR,
        "no-op should preserve project Cursor bytes",
    );
    assert_file_matches(
        &fixture.project_vscode_path,
        &fixture.project_vscode,
        "no-op should preserve project VS Code bytes",
    );
    assert_codex_exclusions_unchanged(&fixture, "no-op");
    assert_kiro_exclusions_unchanged(&fixture, "no-op");
    assert!(!fixture.process_marker.exists());
}

#[test]
fn sync_uses_only_the_absolute_kiro_home_profile_selected_for_the_process() {
    let home = SyntheticHome::new();
    let kiro_home = home.root().join("selected-kiro-profile");
    let selected_path = kiro_home.join("settings/mcp.json");
    let selected_crew_path = kiro_home.join("crew/mcp.json");
    let selected_agent_path = kiro_home.join("agents/kirocrew.json");
    let default_bytes = b"default Kiro file must not be inspected\n";
    let excluded_bytes = b"{\"private\":\"selected-profile-exclusion\"}\n";

    home.write_file(&home.canonical_configuration(), CANONICAL_DESIRED);
    home.write_file(&home.kiro_configuration(), default_bytes);
    home.write_file(&selected_path, KIRO_CURRENT);
    home.write_file(&selected_crew_path, excluded_bytes);
    home.write_file(&selected_agent_path, excluded_bytes);

    let command = |dry_run: bool| {
        let mut process = home.command();
        process.env("KIRO_HOME", &kiro_home);
        let mut command = AssertCommand::from_std(process);
        command.arg("sync");
        if dry_run {
            command.arg("--dry-run");
        }
        command.timeout(COMMAND_TIMEOUT);
        command
    };

    let dry_output = stdout(&run_success(command(true)));

    assert!(dry_output.contains("Kiro: would update with recoverable backup"));
    assert!(!dry_output.contains("selected-profile-exclusion"));
    assert_file_matches(
        &home.kiro_configuration(),
        default_bytes,
        "KIRO_HOME dry-run must not inspect or mutate the default Kiro profile",
    );
    assert_file_matches(
        &selected_path,
        KIRO_CURRENT,
        "KIRO_HOME dry-run must preserve exact selected-profile bytes",
    );
    assert!(!backup_path(&selected_path).exists());

    let apply_output = stdout(&run_success(command(false)));

    assert!(apply_output.contains("Kiro: updated with recoverable backup"));
    assert!(!apply_output.contains("selected-profile-exclusion"));
    assert_file_matches(
        &home.kiro_configuration(),
        default_bytes,
        "KIRO_HOME apply must leave the default Kiro profile untouched",
    );
    assert!(!backup_path(&home.kiro_configuration()).exists());
    assert_file_matches(
        &backup_path(&selected_path),
        KIRO_CURRENT,
        "KIRO_HOME apply must retain exact prior selected-profile bytes",
    );
    for path in [&selected_crew_path, &selected_agent_path] {
        assert_file_matches(
            path,
            excluded_bytes,
            "KIRO_HOME apply must not mutate Crew-only or generated-agent stores",
        );
        assert!(!backup_path(path).exists());
    }
    let (selected_text, selected) = parse_kiro(&selected_path);
    assert!(selected_text.contains("// Kiro global-user fixture comment"));
    assert_eq!(
        selected["mcpServers"]["updated"]["command"].as_str(),
        Some("/synthetic/bin/new")
    );
    assert_eq!(
        selected["mcpServers"]["reference-only"]["command"].as_str(),
        Some("${KIRO_FIXTURE_RUNNER}")
    );
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
    assert!(diagnostic.contains("VS Code: not attempted after an earlier failure"));
    assert!(diagnostic.contains("Codex: not attempted after an earlier failure"));
    assert!(diagnostic.contains("Kiro: not attempted after an earlier failure"));
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
        &home.vscode_configuration(),
        &fixture.vscode,
        "a second-target failure should preserve VS Code bytes",
    );
    assert!(!backup_path(&home.vscode_configuration()).exists());
    assert_file_matches(
        &home.codex_configuration(),
        &fixture.codex,
        "a second-target failure should preserve Codex bytes",
    );
    assert!(!backup_path(&home.codex_configuration()).exists());
    assert_file_matches(
        &home.kiro_configuration(),
        &fixture.kiro,
        "a second-target failure should preserve Kiro bytes",
    );
    assert!(!backup_path(&home.kiro_configuration()).exists());
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
    assert_file_matches(
        &fixture.project_vscode_path,
        &fixture.project_vscode,
        "failed sync should preserve project VS Code bytes",
    );
    assert_codex_exclusions_unchanged(&fixture, "second-target failure");
    assert_kiro_exclusions_unchanged(&fixture, "second-target failure");
    assert!(!fixture.process_marker.exists());
    assert_no_temporary_files(&home.claude_desktop_configuration());
    assert_no_temporary_files(&home.cursor_configuration());
    assert_no_temporary_files(&home.windsurf_configuration());
    assert_no_temporary_files(&home.vscode_configuration());
    assert_no_temporary_files(&home.codex_configuration());
    assert_no_temporary_files(&home.kiro_configuration());
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
    assert!(diagnostic.contains("VS Code: not attempted after an earlier failure"));
    assert!(diagnostic.contains("Codex: not attempted after an earlier failure"));
    assert!(diagnostic.contains("Kiro: not attempted after an earlier failure"));
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
        &home.vscode_configuration(),
        &fixture.vscode,
        "a third-target failure should preserve VS Code bytes",
    );
    assert!(!backup_path(&home.vscode_configuration()).exists());
    assert_file_matches(
        &home.codex_configuration(),
        &fixture.codex,
        "a third-target failure should preserve Codex bytes",
    );
    assert!(!backup_path(&home.codex_configuration()).exists());
    assert_file_matches(
        &home.kiro_configuration(),
        &fixture.kiro,
        "a third-target failure should preserve Kiro bytes",
    );
    assert!(!backup_path(&home.kiro_configuration()).exists());
    assert_file_matches(
        &home.canonical_configuration(),
        &fixture.canonical,
        "failed four-target sync should preserve canonical bytes",
    );
    assert_file_matches(
        &fixture.project_path,
        PROJECT_CURSOR,
        "failed four-target sync should preserve project Cursor bytes",
    );
    assert_file_matches(
        &fixture.project_vscode_path,
        &fixture.project_vscode,
        "failed five-target sync should preserve project VS Code bytes",
    );
    assert_codex_exclusions_unchanged(&fixture, "third-target failure");
    assert_kiro_exclusions_unchanged(&fixture, "third-target failure");
    assert!(!fixture.process_marker.exists());
    for target in [
        home.claude_desktop_configuration(),
        home.cursor_configuration(),
        home.windsurf_configuration(),
        home.vscode_configuration(),
        home.codex_configuration(),
        home.kiro_configuration(),
    ] {
        assert_no_temporary_files(&target);
    }
}

#[test]
fn a_real_fourth_target_failure_restores_all_prior_targets_and_backups_in_reverse() {
    let home = SyntheticHome::new();
    let fixture = prepare_existing_journey(&home);
    let claude_backup = backup_path(&home.claude_desktop_configuration());
    let cursor_backup = backup_path(&home.cursor_configuration());
    let windsurf_backup = backup_path(&home.windsurf_configuration());
    let vscode_backup = backup_path(&home.vscode_configuration());
    let previous_claude_backup = b"older private Claude backup bytes\n";
    let previous_cursor_backup = b"older private Cursor backup bytes\n";
    let previous_windsurf_backup = b"older private Windsurf backup bytes\n";
    home.write_file(&claude_backup, previous_claude_backup);
    home.write_file(&cursor_backup, previous_cursor_backup);
    home.write_file(&windsurf_backup, previous_windsurf_backup);
    fs::create_dir(&vscode_backup).expect("the blocking VS Code backup should be created");

    let output = run_failure(sync_command(&home, false));
    let diagnostic = stderr(&output);

    assert!(diagnostic.starts_with("error: sync transaction failed while applying VS Code:"));
    assert!(diagnostic.contains("Claude Desktop: rolled back after update"));
    assert!(diagnostic.contains("Cursor: rolled back after update"));
    assert!(diagnostic.contains("Windsurf: rolled back after update"));
    assert!(diagnostic.contains("VS Code: update failed: refusing to replace directory"));
    assert!(diagnostic.contains("Codex: not attempted after an earlier failure"));
    assert!(diagnostic.contains("Kiro: not attempted after an earlier failure"));
    assert_output_omits(&diagnostic, &fixture.private_values());
    for private_backup in [
        "older private Claude backup bytes",
        "older private Cursor backup bytes",
        "older private Windsurf backup bytes",
    ] {
        assert!(!diagnostic.contains(private_backup));
    }
    for (path, bytes, label) in [
        (
            home.claude_desktop_configuration(),
            fixture.claude.as_slice(),
            "Claude Desktop",
        ),
        (
            home.cursor_configuration(),
            fixture.cursor.as_slice(),
            "Cursor",
        ),
        (
            home.windsurf_configuration(),
            fixture.windsurf.as_slice(),
            "Windsurf",
        ),
        (
            home.vscode_configuration(),
            fixture.vscode.as_slice(),
            "VS Code",
        ),
        (
            home.codex_configuration(),
            fixture.codex.as_slice(),
            "Codex",
        ),
        (home.kiro_configuration(), fixture.kiro.as_slice(), "Kiro"),
    ] {
        assert_file_matches(
            &path,
            bytes,
            &format!("fourth-target rollback should restore or preserve {label} bytes"),
        );
        assert_no_temporary_files(&path);
    }
    assert_file_matches(
        &claude_backup,
        previous_claude_backup,
        "fourth-target rollback should restore the previous Claude backup",
    );
    assert_file_matches(
        &cursor_backup,
        previous_cursor_backup,
        "fourth-target rollback should restore the previous Cursor backup",
    );
    assert_file_matches(
        &windsurf_backup,
        previous_windsurf_backup,
        "fourth-target rollback should restore the previous Windsurf backup",
    );
    assert!(vscode_backup.is_dir());
    assert!(!backup_path(&home.codex_configuration()).exists());
    assert!(!backup_path(&home.kiro_configuration()).exists());
    assert_file_matches(
        &home.canonical_configuration(),
        &fixture.canonical,
        "failed fourth-target sync should preserve canonical bytes",
    );
    assert_file_matches(
        &fixture.project_vscode_path,
        &fixture.project_vscode,
        "failed fourth-target sync should preserve project VS Code bytes",
    );
    assert_codex_exclusions_unchanged(&fixture, "fourth-target failure");
    assert_kiro_exclusions_unchanged(&fixture, "fourth-target failure");
    assert!(!fixture.process_marker.exists());
}

#[test]
fn a_real_fifth_target_failure_restores_all_prior_targets_and_backups_in_reverse() {
    let home = SyntheticHome::new();
    let fixture = prepare_existing_journey(&home);
    let claude_backup = backup_path(&home.claude_desktop_configuration());
    let cursor_backup = backup_path(&home.cursor_configuration());
    let windsurf_backup = backup_path(&home.windsurf_configuration());
    let vscode_backup = backup_path(&home.vscode_configuration());
    let codex_backup = backup_path(&home.codex_configuration());
    let previous_claude_backup = b"older private Claude backup bytes\n";
    let previous_cursor_backup = b"older private Cursor backup bytes\n";
    let previous_windsurf_backup = b"older private Windsurf backup bytes\n";
    let previous_vscode_backup = b"older private VS Code backup bytes\n";
    home.write_file(&claude_backup, previous_claude_backup);
    home.write_file(&cursor_backup, previous_cursor_backup);
    home.write_file(&windsurf_backup, previous_windsurf_backup);
    home.write_file(&vscode_backup, previous_vscode_backup);
    fs::create_dir(&codex_backup).expect("the blocking Codex backup should be created");

    let output = run_failure(sync_command(&home, false));
    let diagnostic = stderr(&output);

    assert!(diagnostic.starts_with("error: sync transaction failed while applying Codex:"));
    assert!(diagnostic.contains("Claude Desktop: rolled back after update"));
    assert!(diagnostic.contains("Cursor: rolled back after update"));
    assert!(diagnostic.contains("Windsurf: rolled back after update"));
    assert!(diagnostic.contains("VS Code: rolled back after update"));
    assert!(diagnostic.contains("Codex: update failed: refusing to replace directory"));
    assert!(diagnostic.contains("Kiro: not attempted after an earlier failure"));
    assert_output_omits(&diagnostic, &fixture.private_values());
    for private_backup in [
        "older private Claude backup bytes",
        "older private Cursor backup bytes",
        "older private Windsurf backup bytes",
        "older private VS Code backup bytes",
    ] {
        assert!(!diagnostic.contains(private_backup));
    }
    for (path, bytes, label) in [
        (
            home.claude_desktop_configuration(),
            fixture.claude.as_slice(),
            "Claude Desktop",
        ),
        (
            home.cursor_configuration(),
            fixture.cursor.as_slice(),
            "Cursor",
        ),
        (
            home.windsurf_configuration(),
            fixture.windsurf.as_slice(),
            "Windsurf",
        ),
        (
            home.vscode_configuration(),
            fixture.vscode.as_slice(),
            "VS Code",
        ),
        (
            home.codex_configuration(),
            fixture.codex.as_slice(),
            "Codex",
        ),
        (home.kiro_configuration(), fixture.kiro.as_slice(), "Kiro"),
    ] {
        assert_file_matches(
            &path,
            bytes,
            &format!("fifth-target rollback should restore or preserve {label} bytes"),
        );
        assert_no_temporary_files(&path);
    }
    for (path, bytes, label) in [
        (&claude_backup, previous_claude_backup.as_slice(), "Claude"),
        (&cursor_backup, previous_cursor_backup.as_slice(), "Cursor"),
        (
            &windsurf_backup,
            previous_windsurf_backup.as_slice(),
            "Windsurf",
        ),
        (&vscode_backup, previous_vscode_backup.as_slice(), "VS Code"),
    ] {
        assert_file_matches(
            path,
            bytes,
            &format!("fifth-target rollback should restore the previous {label} backup"),
        );
    }
    assert!(codex_backup.is_dir());
    assert!(!backup_path(&home.kiro_configuration()).exists());
    assert_file_matches(
        &home.canonical_configuration(),
        &fixture.canonical,
        "failed fifth-target sync should preserve canonical bytes",
    );
    assert_file_matches(
        &fixture.project_path,
        PROJECT_CURSOR,
        "failed fifth-target sync should preserve project Cursor bytes",
    );
    assert_file_matches(
        &fixture.project_vscode_path,
        &fixture.project_vscode,
        "failed fifth-target sync should preserve project VS Code bytes",
    );
    assert_codex_exclusions_unchanged(&fixture, "fifth-target failure");
    assert_kiro_exclusions_unchanged(&fixture, "fifth-target failure");
    assert!(!fixture.process_marker.exists());
}

#[test]
fn a_real_sixth_target_failure_restores_all_five_prior_targets_and_backups_in_reverse() {
    let home = SyntheticHome::new();
    let fixture = prepare_existing_journey(&home);
    let backups = [
        (
            backup_path(&home.claude_desktop_configuration()),
            b"older private Claude backup bytes\n".as_slice(),
            "Claude Desktop",
        ),
        (
            backup_path(&home.cursor_configuration()),
            b"older private Cursor backup bytes\n".as_slice(),
            "Cursor",
        ),
        (
            backup_path(&home.windsurf_configuration()),
            b"older private Windsurf backup bytes\n".as_slice(),
            "Windsurf",
        ),
        (
            backup_path(&home.vscode_configuration()),
            b"older private VS Code backup bytes\n".as_slice(),
            "VS Code",
        ),
        (
            backup_path(&home.codex_configuration()),
            b"older private Codex backup bytes\n".as_slice(),
            "Codex",
        ),
    ];
    for (path, bytes, _) in &backups {
        home.write_file(path, bytes);
    }
    let kiro_backup = backup_path(&home.kiro_configuration());
    fs::create_dir(&kiro_backup).expect("the blocking Kiro backup should be created");

    let diagnostic = stderr(&run_failure(sync_command(&home, false)));

    assert!(diagnostic.starts_with("error: sync transaction failed while applying Kiro:"));
    for target in ["Claude Desktop", "Cursor", "Windsurf", "VS Code", "Codex"] {
        assert!(diagnostic.contains(&format!("{target}: rolled back after update")));
    }
    assert!(diagnostic.contains("Kiro: update failed: refusing to replace directory"));
    assert_output_omits(&diagnostic, &fixture.private_values());
    for private_backup in [
        "older private Claude backup bytes",
        "older private Cursor backup bytes",
        "older private Windsurf backup bytes",
        "older private VS Code backup bytes",
        "older private Codex backup bytes",
    ] {
        assert!(!diagnostic.contains(private_backup));
    }

    for (path, bytes, label) in [
        (
            home.claude_desktop_configuration(),
            fixture.claude.as_slice(),
            "Claude Desktop",
        ),
        (
            home.cursor_configuration(),
            fixture.cursor.as_slice(),
            "Cursor",
        ),
        (
            home.windsurf_configuration(),
            fixture.windsurf.as_slice(),
            "Windsurf",
        ),
        (
            home.vscode_configuration(),
            fixture.vscode.as_slice(),
            "VS Code",
        ),
        (
            home.codex_configuration(),
            fixture.codex.as_slice(),
            "Codex",
        ),
        (home.kiro_configuration(), fixture.kiro.as_slice(), "Kiro"),
    ] {
        assert_file_matches(
            &path,
            bytes,
            &format!("sixth-target rollback should restore or preserve {label} bytes"),
        );
        assert_no_temporary_files(&path);
    }
    for (path, bytes, label) in &backups {
        assert_file_matches(
            path,
            bytes,
            &format!("sixth-target rollback should restore the previous {label} backup"),
        );
    }
    assert!(kiro_backup.is_dir());
    assert_file_matches(
        &home.canonical_configuration(),
        &fixture.canonical,
        "failed sixth-target sync should preserve canonical bytes",
    );
    assert_codex_exclusions_unchanged(&fixture, "sixth-target failure");
    assert_kiro_exclusions_unchanged(&fixture, "sixth-target failure");
    assert!(!fixture.process_marker.exists());
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
    assert!(diagnostic.contains("VS Code: not attempted after an earlier failure"));
    assert!(diagnostic.contains("Codex: not attempted after an earlier failure"));
    assert!(diagnostic.contains("Kiro: not attempted after an earlier failure"));
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
        &home.vscode_configuration(),
        &fixture.vscode,
        "permission failure should preserve VS Code bytes",
    );
    assert!(!backup_path(&home.vscode_configuration()).exists());
    assert_file_matches(
        &home.codex_configuration(),
        &fixture.codex,
        "creation rollback should preserve Codex bytes",
    );
    assert!(!backup_path(&home.codex_configuration()).exists());
    assert_file_matches(
        &home.kiro_configuration(),
        &fixture.kiro,
        "creation rollback should preserve Kiro bytes",
    );
    assert!(!backup_path(&home.kiro_configuration()).exists());
    assert_file_matches(
        &home.vscode_configuration(),
        &fixture.vscode,
        "creation rollback should preserve VS Code bytes",
    );
    assert!(!backup_path(&home.vscode_configuration()).exists());
    assert_file_matches(
        &fixture.project_path,
        PROJECT_CURSOR,
        "creation rollback should preserve project Cursor bytes",
    );
    assert_file_matches(
        &fixture.project_vscode_path,
        &fixture.project_vscode,
        "creation rollback should preserve project VS Code bytes",
    );
    assert_codex_exclusions_unchanged(&fixture, "creation rollback");
    assert_kiro_exclusions_unchanged(&fixture, "creation rollback");
    assert!(!fixture.process_marker.exists());
    assert_no_temporary_files(&home.claude_desktop_configuration());
    assert_no_temporary_files(&home.cursor_configuration());
    assert_no_temporary_files(&home.windsurf_configuration());
    assert_no_temporary_files(&home.vscode_configuration());
    assert_no_temporary_files(&home.codex_configuration());
    assert_no_temporary_files(&home.kiro_configuration());
}

#[test]
fn malformed_final_vscode_state_stops_before_any_apply_mutation() {
    let home = SyntheticHome::new();
    let fixture = prepare_existing_journey(&home);
    let malformed =
        b"{\"servers\":{\"bad\":{\"type\":\"stdio\",\"command\":\"private-malformed-command\"";
    home.write_file(&home.vscode_configuration(), malformed);

    let output = run_failure(sync_command(&home, false));
    let diagnostic = stderr(&output);

    assert!(diagnostic.starts_with("error: cannot plan VS Code sync: invalid VS Code JSON:"));
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
        &fixture.windsurf,
        "preflight failure should preserve Windsurf bytes",
    );
    assert_file_matches(
        &home.vscode_configuration(),
        malformed,
        "preflight failure should preserve malformed VS Code bytes",
    );
    assert_file_matches(
        &home.codex_configuration(),
        &fixture.codex,
        "VS Code preflight failure should preserve Codex bytes",
    );
    assert_file_matches(
        &home.kiro_configuration(),
        &fixture.kiro,
        "VS Code preflight failure should preserve Kiro bytes",
    );
    assert!(!backup_path(&home.claude_desktop_configuration()).exists());
    assert!(!backup_path(&home.cursor_configuration()).exists());
    assert!(!backup_path(&home.windsurf_configuration()).exists());
    assert!(!backup_path(&home.vscode_configuration()).exists());
    assert!(!backup_path(&home.codex_configuration()).exists());
    assert!(!backup_path(&home.kiro_configuration()).exists());
    assert_file_matches(
        &fixture.project_path,
        PROJECT_CURSOR,
        "preflight failure should preserve project Cursor bytes",
    );
    assert_file_matches(
        &fixture.project_vscode_path,
        &fixture.project_vscode,
        "preflight failure should preserve project VS Code bytes",
    );
    assert_codex_exclusions_unchanged(&fixture, "VS Code preflight failure");
    assert_kiro_exclusions_unchanged(&fixture, "VS Code preflight failure");
    assert!(!fixture.process_marker.exists());
}

#[test]
fn malformed_final_codex_state_stops_before_any_apply_mutation_without_value_output() {
    let home = SyntheticHome::new();
    let fixture = prepare_existing_journey(&home);
    let malformed =
        b"private = \"codex-malformed-private-value\"\n[mcp_servers.bad\ncommand = \"safe\"\n";
    home.write_file(&home.codex_configuration(), malformed);

    let output = run_failure(sync_command(&home, false));
    let diagnostic = stderr(&output);

    assert!(diagnostic.starts_with("error: cannot plan Codex sync: invalid Codex TOML near byte "));
    assert!(diagnostic.ends_with("; no target files were changed\n"));
    assert!(!diagnostic.contains("codex-malformed-private-value"));
    assert_output_omits(&diagnostic, &fixture.private_values());
    for (path, bytes, label) in [
        (
            home.claude_desktop_configuration(),
            fixture.claude.as_slice(),
            "Claude Desktop",
        ),
        (
            home.cursor_configuration(),
            fixture.cursor.as_slice(),
            "Cursor",
        ),
        (
            home.windsurf_configuration(),
            fixture.windsurf.as_slice(),
            "Windsurf",
        ),
        (
            home.vscode_configuration(),
            fixture.vscode.as_slice(),
            "VS Code",
        ),
        (home.codex_configuration(), malformed.as_slice(), "Codex"),
        (home.kiro_configuration(), fixture.kiro.as_slice(), "Kiro"),
    ] {
        assert_file_matches(
            &path,
            bytes,
            &format!("Codex preflight failure should preserve {label} bytes"),
        );
        assert!(!backup_path(&path).exists());
    }
    assert_file_matches(
        &home.canonical_configuration(),
        &fixture.canonical,
        "Codex preflight failure should preserve canonical bytes",
    );
    assert_file_matches(
        &fixture.project_path,
        PROJECT_CURSOR,
        "Codex preflight failure should preserve project Cursor bytes",
    );
    assert_file_matches(
        &fixture.project_vscode_path,
        &fixture.project_vscode,
        "Codex preflight failure should preserve project VS Code bytes",
    );
    assert_codex_exclusions_unchanged(&fixture, "Codex preflight failure");
    assert_kiro_exclusions_unchanged(&fixture, "Codex preflight failure");
    assert!(!fixture.process_marker.exists());
}

#[test]
fn malformed_final_kiro_state_stops_before_any_apply_mutation_without_value_output() {
    let home = SyntheticHome::new();
    let fixture = prepare_existing_journey(&home);
    let malformed =
        b"{\n// private-kiro-malformed-comment\n\"mcpServers\":{\"bad\":{\"command\":\"private-kiro-malformed-command\"";
    home.write_file(&home.kiro_configuration(), malformed);

    let diagnostic = stderr(&run_failure(sync_command(&home, false)));

    assert!(diagnostic.starts_with("error: cannot plan Kiro sync: invalid Kiro JSON:"));
    assert!(diagnostic.ends_with("; no target files were changed\n"));
    for private in [
        "private-kiro-malformed-comment",
        "private-kiro-malformed-command",
    ] {
        assert!(!diagnostic.contains(private));
    }
    assert_output_omits(&diagnostic, &fixture.private_values());
    for (path, bytes, label) in [
        (
            home.claude_desktop_configuration(),
            fixture.claude.as_slice(),
            "Claude Desktop",
        ),
        (
            home.cursor_configuration(),
            fixture.cursor.as_slice(),
            "Cursor",
        ),
        (
            home.windsurf_configuration(),
            fixture.windsurf.as_slice(),
            "Windsurf",
        ),
        (
            home.vscode_configuration(),
            fixture.vscode.as_slice(),
            "VS Code",
        ),
        (
            home.codex_configuration(),
            fixture.codex.as_slice(),
            "Codex",
        ),
        (home.kiro_configuration(), malformed.as_slice(), "Kiro"),
    ] {
        assert_file_matches(
            &path,
            bytes,
            &format!("Kiro preflight failure should preserve {label} bytes"),
        );
        assert!(!backup_path(&path).exists());
    }
    assert_file_matches(
        &home.canonical_configuration(),
        &fixture.canonical,
        "Kiro preflight failure should preserve canonical bytes",
    );
    assert_codex_exclusions_unchanged(&fixture, "Kiro preflight failure");
    assert_kiro_exclusions_unchanged(&fixture, "Kiro preflight failure");
    assert!(!fixture.process_marker.exists());
}

#[test]
fn missing_or_malformed_canonical_state_fails_before_native_discovery() {
    let home = SyntheticHome::new();
    let claude = b"Claude target must not be parsed before canonical validation\n";
    let cursor = b"Cursor target must not be parsed before canonical validation\n";
    let windsurf = b"Windsurf target must not be parsed before canonical validation\n";
    let vscode = b"VS Code target must not be parsed before canonical validation\n";
    let codex = b"Codex target must not be parsed before canonical validation\n";
    let kiro = b"Kiro target must not be parsed before canonical validation\n";
    home.write_file(&home.claude_desktop_configuration(), claude);
    home.write_file(&home.cursor_configuration(), cursor);
    home.write_file(&home.windsurf_configuration(), windsurf);
    home.write_file(&home.vscode_configuration(), vscode);
    home.write_file(&home.codex_configuration(), codex);
    home.write_file(&home.kiro_configuration(), kiro);

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
    assert_file_matches(
        &home.vscode_configuration(),
        vscode,
        "canonical validation failure should preserve VS Code bytes",
    );
    assert_file_matches(
        &home.codex_configuration(),
        codex,
        "canonical validation failure should preserve Codex bytes",
    );
    assert_file_matches(
        &home.kiro_configuration(),
        kiro,
        "canonical validation failure should preserve Kiro bytes",
    );
    assert!(!backup_path(&home.claude_desktop_configuration()).exists());
    assert!(!backup_path(&home.cursor_configuration()).exists());
    assert!(!backup_path(&home.windsurf_configuration()).exists());
    assert!(!backup_path(&home.vscode_configuration()).exists());
    assert!(!backup_path(&home.codex_configuration()).exists());
    assert!(!backup_path(&home.kiro_configuration()).exists());
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
    assert_file_matches(
        &home.vscode_configuration(),
        &fixture.vscode,
        "collision failure should preserve VS Code bytes",
    );
    assert_file_matches(
        &home.codex_configuration(),
        &fixture.codex,
        "Cursor collision should preserve Codex bytes",
    );
    assert_file_matches(
        &home.kiro_configuration(),
        &fixture.kiro,
        "Cursor collision should preserve Kiro bytes",
    );
    assert!(!backup_path(&home.claude_desktop_configuration()).exists());
    assert!(!backup_path(&home.cursor_configuration()).exists());
    assert!(!backup_path(&home.windsurf_configuration()).exists());
    assert!(!backup_path(&home.vscode_configuration()).exists());
    assert!(!backup_path(&home.codex_configuration()).exists());
    assert!(!backup_path(&home.kiro_configuration()).exists());
    assert_file_matches(
        &fixture.project_path,
        PROJECT_CURSOR,
        "collision failure should preserve project Cursor bytes",
    );
    assert_file_matches(
        &fixture.project_vscode_path,
        &fixture.project_vscode,
        "collision failure should preserve project VS Code bytes",
    );
    assert_codex_exclusions_unchanged(&fixture, "Cursor collision");
    assert_kiro_exclusions_unchanged(&fixture, "Cursor collision");
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
    assert_file_matches(
        &home.vscode_configuration(),
        &fixture.vscode,
        "Windsurf collision should preserve VS Code bytes",
    );
    assert_file_matches(
        &home.codex_configuration(),
        &fixture.codex,
        "Windsurf collision should preserve Codex bytes",
    );
    for target in [
        home.claude_desktop_configuration(),
        home.cursor_configuration(),
        home.windsurf_configuration(),
        home.vscode_configuration(),
        home.codex_configuration(),
    ] {
        assert!(!backup_path(&target).exists());
    }
    assert_file_matches(
        &fixture.project_path,
        PROJECT_CURSOR,
        "Windsurf collision should preserve project Cursor bytes",
    );
    assert_file_matches(
        &fixture.project_vscode_path,
        &fixture.project_vscode,
        "Windsurf collision should preserve project VS Code bytes",
    );
    assert_codex_exclusions_unchanged(&fixture, "Windsurf collision");
    assert_kiro_exclusions_unchanged(&fixture, "Windsurf collision");
    assert!(!fixture.process_marker.exists());
}

#[test]
fn an_unmanaged_vscode_name_collision_fails_the_complete_plan_before_apply() {
    let home = SyntheticHome::new();
    let fixture = prepare_existing_journey(&home);
    let mut canonical: Value =
        serde_json::from_slice(&fixture.canonical).expect("canonical fixture should parse");
    canonical["servers"]["numeric-env"] = canonical["servers"]["added"].clone();
    let mut canonical =
        serde_json::to_vec_pretty(&canonical).expect("collision fixture should serialize");
    canonical.push(b'\n');
    home.write_file(&home.canonical_configuration(), &canonical);

    let output = run_failure(sync_command(&home, false));
    let diagnostic = stderr(&output);

    assert!(diagnostic.contains(
        "cannot render the validated VS Code sync plan: desired local server \"numeric-env\" collides with an unmanaged VS Code server"
    ));
    assert!(diagnostic.ends_with("; no target files were changed\n"));
    assert_output_omits(&diagnostic, &fixture.private_values());
    for (path, bytes, label) in [
        (
            home.claude_desktop_configuration(),
            fixture.claude.as_slice(),
            "Claude Desktop",
        ),
        (
            home.cursor_configuration(),
            fixture.cursor.as_slice(),
            "Cursor",
        ),
        (
            home.windsurf_configuration(),
            fixture.windsurf.as_slice(),
            "Windsurf",
        ),
        (
            home.vscode_configuration(),
            fixture.vscode.as_slice(),
            "VS Code",
        ),
        (
            home.codex_configuration(),
            fixture.codex.as_slice(),
            "Codex",
        ),
    ] {
        assert_file_matches(
            &path,
            bytes,
            &format!("VS Code collision should preserve {label} bytes"),
        );
        assert!(!backup_path(&path).exists());
    }
    assert_file_matches(
        &fixture.project_vscode_path,
        &fixture.project_vscode,
        "VS Code collision should preserve project VS Code bytes",
    );
    assert_codex_exclusions_unchanged(&fixture, "VS Code collision");
    assert_file_matches(
        &home.kiro_configuration(),
        &fixture.kiro,
        "VS Code collision should preserve Kiro bytes",
    );
    assert!(!backup_path(&home.kiro_configuration()).exists());
    assert_kiro_exclusions_unchanged(&fixture, "VS Code collision");
    assert!(!fixture.process_marker.exists());
}

#[test]
fn an_unmanaged_codex_name_collision_fails_the_complete_plan_before_apply() {
    let home = SyntheticHome::new();
    let fixture = prepare_existing_journey(&home);
    let mut canonical: Value =
        serde_json::from_slice(&fixture.canonical).expect("canonical fixture should parse");
    canonical["servers"]["mixed-transport"] = canonical["servers"]["added"].clone();
    let mut canonical =
        serde_json::to_vec_pretty(&canonical).expect("collision fixture should serialize");
    canonical.push(b'\n');
    home.write_file(&home.canonical_configuration(), &canonical);

    let output = run_failure(sync_command(&home, false));
    let diagnostic = stderr(&output);

    assert!(diagnostic.contains(
        "cannot render the validated Codex sync plan: desired local server \"mixed-transport\" collides with an unmanaged Codex server"
    ));
    assert!(diagnostic.ends_with("; no target files were changed\n"));
    assert_output_omits(&diagnostic, &fixture.private_values());
    for (path, bytes, label) in [
        (
            home.claude_desktop_configuration(),
            fixture.claude.as_slice(),
            "Claude Desktop",
        ),
        (
            home.cursor_configuration(),
            fixture.cursor.as_slice(),
            "Cursor",
        ),
        (
            home.windsurf_configuration(),
            fixture.windsurf.as_slice(),
            "Windsurf",
        ),
        (
            home.vscode_configuration(),
            fixture.vscode.as_slice(),
            "VS Code",
        ),
        (
            home.codex_configuration(),
            fixture.codex.as_slice(),
            "Codex",
        ),
    ] {
        assert_file_matches(
            &path,
            bytes,
            &format!("Codex collision should preserve {label} bytes"),
        );
        assert!(!backup_path(&path).exists());
    }
    assert_file_matches(
        &fixture.project_path,
        PROJECT_CURSOR,
        "Codex collision should preserve project Cursor bytes",
    );
    assert_file_matches(
        &fixture.project_vscode_path,
        &fixture.project_vscode,
        "Codex collision should preserve project VS Code bytes",
    );
    assert_codex_exclusions_unchanged(&fixture, "Codex collision");
    assert_file_matches(
        &home.kiro_configuration(),
        &fixture.kiro,
        "Codex collision should preserve Kiro bytes",
    );
    assert!(!backup_path(&home.kiro_configuration()).exists());
    assert_kiro_exclusions_unchanged(&fixture, "Codex collision");
    assert!(!fixture.process_marker.exists());
}

#[test]
fn an_unmanaged_kiro_reference_collision_fails_the_complete_plan_before_apply() {
    let home = SyntheticHome::new();
    let fixture = prepare_existing_journey(&home);
    let mut canonical: Value =
        serde_json::from_slice(&fixture.canonical).expect("canonical fixture should parse");
    canonical["servers"]["kiro-reference-only"] = canonical["servers"]["added"].clone();
    let mut canonical =
        serde_json::to_vec_pretty(&canonical).expect("collision fixture should serialize");
    canonical.push(b'\n');
    home.write_file(&home.canonical_configuration(), &canonical);

    let diagnostic = stderr(&run_failure(sync_command(&home, false)));

    assert!(diagnostic.contains(
        "cannot render the validated Kiro sync plan: cannot reconcile local server \"kiro-reference-only\" because Kiro already has an unmanaged entry with that name"
    ));
    assert!(diagnostic.ends_with("; no target files were changed\n"));
    assert_output_omits(&diagnostic, &fixture.private_values());
    for (path, bytes, label) in [
        (
            home.claude_desktop_configuration(),
            fixture.claude.as_slice(),
            "Claude Desktop",
        ),
        (
            home.cursor_configuration(),
            fixture.cursor.as_slice(),
            "Cursor",
        ),
        (
            home.windsurf_configuration(),
            fixture.windsurf.as_slice(),
            "Windsurf",
        ),
        (
            home.vscode_configuration(),
            fixture.vscode.as_slice(),
            "VS Code",
        ),
        (
            home.codex_configuration(),
            fixture.codex.as_slice(),
            "Codex",
        ),
        (home.kiro_configuration(), fixture.kiro.as_slice(), "Kiro"),
    ] {
        assert_file_matches(
            &path,
            bytes,
            &format!("Kiro collision should preserve {label} bytes"),
        );
        assert!(!backup_path(&path).exists());
    }
    assert_codex_exclusions_unchanged(&fixture, "Kiro collision");
    assert_kiro_exclusions_unchanged(&fixture, "Kiro collision");
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
    assert!(diagnostic.contains("VS Code: not attempted after an earlier failure"));
    assert!(diagnostic.contains("Codex: not attempted after an earlier failure"));
    assert!(diagnostic.contains("Permission denied"));
    assert!(diagnostic.contains("Kiro: not attempted after an earlier failure"));
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
        &home.vscode_configuration(),
        &fixture.vscode,
        "permission failure should preserve VS Code bytes",
    );
    assert!(!backup_path(&home.vscode_configuration()).exists());
    assert_file_matches(
        &home.codex_configuration(),
        &fixture.codex,
        "permission failure should preserve Codex bytes",
    );
    assert!(!backup_path(&home.codex_configuration()).exists());
    assert_file_matches(
        &home.kiro_configuration(),
        &fixture.kiro,
        "permission failure should preserve Kiro bytes",
    );
    assert!(!backup_path(&home.kiro_configuration()).exists());
    assert_file_matches(
        &fixture.project_path,
        PROJECT_CURSOR,
        "permission failure should preserve project Cursor bytes",
    );
    assert_file_matches(
        &fixture.project_vscode_path,
        &fixture.project_vscode,
        "permission failure should preserve project VS Code bytes",
    );
    assert_codex_exclusions_unchanged(&fixture, "permission failure");
    assert_kiro_exclusions_unchanged(&fixture, "permission failure");
    assert!(!fixture.process_marker.exists());
}
