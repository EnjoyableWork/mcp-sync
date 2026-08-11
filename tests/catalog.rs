mod support;

use assert_cmd::Command as AssertCommand;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;
use std::time::Duration;
use support::SyntheticHome;

const COMMAND_TIMEOUT: Duration = Duration::from_secs(5);
const EMPTY_CANONICAL: &[u8] = b"{\n  \"schemaVersion\": 1,\n  \"servers\": {}\n}\n";

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

fn list_command(home: &SyntheticHome) -> AssertCommand {
    let mut command = AssertCommand::from_std(home.command());
    command.arg("list").timeout(COMMAND_TIMEOUT);
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

fn assert_no_temporary_files(destination: &Path) {
    let parent = destination
        .parent()
        .expect("the canonical configuration has a parent");
    if !parent.exists() {
        return;
    }

    let temporary_file_exists = fs::read_dir(parent)
        .expect("the canonical directory should be readable")
        .filter_map(Result::ok)
        .map(|entry| entry.file_name())
        .any(|name| name.to_string_lossy().starts_with(".mcp-sync-"));
    assert!(
        !temporary_file_exists,
        "temporary files should be cleaned up"
    );
}

fn assert_output_omits(output: &str, private_values: &[&str]) {
    for private_value in private_values {
        assert!(
            !output.contains(private_value),
            "CLI output must omit command, argument, and environment values"
        );
    }
}

fn write_empty_canonical(home: &SyntheticHome) {
    home.write_file(&home.canonical_configuration(), EMPTY_CANONICAL);
}

#[test]
fn add_and_list_are_redacted_and_never_access_target_clients() {
    let home = SyntheticHome::new();
    write_empty_canonical(&home);
    let canonical_path = home.canonical_configuration();
    let backup = backup_path(&canonical_path);
    let claude_bytes = b"Claude target must not be read by add or list\n";
    let cursor_bytes = b"Cursor target must not be read by add or list\n";
    let windsurf_bytes = b"Windsurf target must not be read by add or list\n";
    let vscode_bytes = b"VS Code target must not be read by add or list\n";
    let codex_bytes = b"Codex target must not be read by add or list\n";
    let kiro_bytes = b"Kiro target must not be read by add or list\n";
    let project_path = home.user_root().join("workspace/.cursor/mcp.json");
    let project_bytes = b"project target must remain outside every global operation\n";
    home.write_file(&home.claude_desktop_configuration(), claude_bytes);
    home.write_file(&home.cursor_configuration(), cursor_bytes);
    home.write_file(&home.windsurf_configuration(), windsurf_bytes);
    home.write_file(&home.vscode_configuration(), vscode_bytes);
    home.write_file(&home.codex_configuration(), codex_bytes);
    home.write_file(&home.kiro_configuration(), kiro_bytes);
    home.write_file(&project_path, project_bytes);
    let process_marker = home.root().join("configured-server-was-started");
    let process_sentinel = home.root().join("synthetic-mcp-server");
    home.write_file(
        &process_sentinel,
        format!(
            "#!/bin/sh\nprintf started > '{}'\n",
            process_marker.display()
        ),
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        fs::set_permissions(&process_sentinel, fs::Permissions::from_mode(0o700))
            .expect("the process sentinel should be executable");
    }

    let empty_list = stdout(&run_success(list_command(&home)));
    assert!(
        empty_list == "No servers configured.\n",
        "empty list output should be stable"
    );
    assert!(
        !backup.exists(),
        "read-only listing should not create a backup"
    );

    let process_sentinel_text = process_sentinel.to_string_lossy().into_owned();
    let zeta_private = [
        process_sentinel_text.as_str(),
        "--zeta-private-argument",
        "zeta-literal-argument",
        "zeta-private-value",
    ];
    let zeta_output = stdout(&run_success(add_command(
        &home,
        "zeta",
        zeta_private[0],
        &[zeta_private[1], zeta_private[2]],
        &["TOKEN=zeta-private-value", "EMPTY="],
    )));
    assert!(
        zeta_output
            == "Added server \"zeta\" (command: <redacted>; arguments: 2; environment keys: 2).\n",
        "add output should contain only stable structural metadata"
    );
    assert_output_omits(&zeta_output, &zeta_private);
    assert!(
        fs::read(&backup).expect("the first add should create a backup") == EMPTY_CANONICAL,
        "the backup should contain the exact pre-add bytes"
    );

    let first_canonical = fs::read(&canonical_path).expect("canonical output should be readable");
    let parsed: Value = serde_json::from_slice(&first_canonical)
        .expect("canonical output should remain valid JSON");
    assert!(
        parsed["servers"]["zeta"]["command"] == zeta_private[0]
            && parsed["servers"]["zeta"]["args"][0] == zeta_private[1]
            && parsed["servers"]["zeta"]["args"][1] == zeta_private[2]
            && parsed["servers"]["zeta"]["env"]["TOKEN"] == zeta_private[3]
            && parsed["servers"]["zeta"]["env"]["EMPTY"] == "",
        "canonical state should preserve literal process values"
    );

    let alpha_output = stdout(&run_success(add_command(
        &home,
        "alpha",
        "alpha-private-command",
        &[],
        &[],
    )));
    assert!(
        alpha_output
            == "Added server \"alpha\" (command: <redacted>; arguments: 0; environment keys: 0).\n",
        "second add output should be stable and redacted"
    );
    assert_output_omits(&alpha_output, &["alpha-private-command"]);

    let canonical_before_list =
        fs::read(&canonical_path).expect("canonical state should remain readable");
    let backup_before_list = fs::read(&backup).expect("the latest backup should remain readable");
    assert!(
        backup_before_list == first_canonical,
        "each changed add should back up the exact immediately preceding state"
    );
    let listed = stdout(&run_success(list_command(&home)));
    assert!(
        listed
            == "\"alpha\": command: <redacted>; arguments: 0; environment keys: none\n\"zeta\": command: <redacted>; arguments: 2; environment keys: \"EMPTY\", \"TOKEN\"\n",
        "list output should be sorted and structurally redacted"
    );
    assert_output_omits(
        &listed,
        &[
            "alpha-private-command",
            zeta_private[0],
            zeta_private[1],
            zeta_private[2],
            zeta_private[3],
        ],
    );
    assert!(
        fs::read(&canonical_path).expect("list should leave canonical state readable")
            == canonical_before_list,
        "list must not rewrite canonical state"
    );
    assert!(
        fs::read(&backup).expect("list should leave the backup readable") == backup_before_list,
        "list must not rewrite the backup"
    );
    assert!(
        fs::read(home.claude_desktop_configuration())
            .expect("the Claude sentinel should remain readable")
            == claude_bytes,
        "add and list must not touch Claude Desktop"
    );
    assert!(
        fs::read(home.cursor_configuration()).expect("the Cursor sentinel should remain readable")
            == cursor_bytes,
        "add and list must not touch global Cursor state"
    );
    assert!(
        fs::read(home.windsurf_configuration())
            .expect("the Windsurf sentinel should remain readable")
            == windsurf_bytes,
        "add and list must not touch global Windsurf state"
    );
    assert!(
        fs::read(home.vscode_configuration()).expect("the VS Code sentinel should remain readable")
            == vscode_bytes,
        "add and list must not touch global VS Code state"
    );
    assert!(
        fs::read(home.codex_configuration()).expect("the Codex sentinel should remain readable")
            == codex_bytes,
        "add and list must not touch global Codex state"
    );
    assert!(
        fs::read(home.kiro_configuration()).expect("the Kiro sentinel should remain readable")
            == kiro_bytes,
        "add and list must not touch global Kiro state"
    );
    assert!(
        fs::read(project_path).expect("the project sentinel should remain readable")
            == project_bytes,
        "add and list must not touch project Cursor state"
    );
    assert!(
        !process_marker.exists(),
        "add and list must never execute a configured server"
    );
    assert_no_temporary_files(&canonical_path);
}

#[test]
fn insertion_order_is_deterministic_and_updates_are_recoverable_no_ops() {
    let first = SyntheticHome::new();
    let second = SyntheticHome::new();
    write_empty_canonical(&first);
    write_empty_canonical(&second);

    run_success(add_command(
        &first,
        "zeta",
        "zeta-private-command",
        &[],
        &[],
    ));
    run_success(add_command(
        &first,
        "alpha",
        "alpha-private-command",
        &["--alpha"],
        &["TOKEN=alpha-private-value"],
    ));
    run_success(add_command(
        &second,
        "alpha",
        "alpha-private-command",
        &["--alpha"],
        &["TOKEN=alpha-private-value"],
    ));
    run_success(add_command(
        &second,
        "zeta",
        "zeta-private-command",
        &[],
        &[],
    ));

    let canonical_path = first.canonical_configuration();
    let before_update = fs::read(&canonical_path).expect("canonical state should be readable");
    assert!(
        before_update
            == fs::read(second.canonical_configuration())
                .expect("the second canonical state should be readable"),
        "insertion order must not change canonical bytes"
    );

    let updated_private = [
        "updated-private-command",
        "--updated-private-argument",
        "updated-private-value",
    ];
    let updated = stdout(&run_success(add_command(
        &first,
        "alpha",
        updated_private[0],
        &[updated_private[1]],
        &["TOKEN=updated-private-value"],
    )));
    assert!(
        updated
            == "Updated server \"alpha\" (command: <redacted>; arguments: 1; environment keys: 1).\n",
        "update output should be stable and structural"
    );
    assert_output_omits(&updated, &updated_private);
    let after_update = fs::read(&canonical_path).expect("updated state should be readable");
    let backup = backup_path(&canonical_path);
    let after_update_backup = fs::read(&backup).expect("an update should create a backup");
    assert!(
        after_update_backup == before_update,
        "an update should preserve the exact prior canonical bytes"
    );
    let parsed: Value =
        serde_json::from_slice(&after_update).expect("updated canonical state should be valid");
    assert!(
        parsed["servers"]["alpha"]["command"] == updated_private[0]
            && parsed["servers"]["zeta"]["command"] == "zeta-private-command",
        "an update should replace one complete definition and preserve every other server"
    );

    let unchanged = stdout(&run_success(add_command(
        &first,
        "alpha",
        updated_private[0],
        &[updated_private[1]],
        &["TOKEN=updated-private-value"],
    )));
    assert!(
        unchanged
            == "Server \"alpha\" is already up to date (command: <redacted>; arguments: 1; environment keys: 1).\n",
        "semantic no-op output should be stable and structural"
    );
    assert_output_omits(&unchanged, &updated_private);
    assert!(
        fs::read(&canonical_path).expect("no-op state should remain readable") == after_update,
        "an exact upsert must not rewrite canonical bytes"
    );
    assert!(
        fs::read(&backup).expect("the no-op backup should remain readable") == after_update_backup,
        "an exact upsert must not create or rewrite a backup"
    );
    assert_no_temporary_files(&canonical_path);
}

#[test]
fn a_semantic_no_op_preserves_noncanonical_input_bytes() {
    let home = SyntheticHome::new();
    let noncanonical = br#"{"servers":{"alpha":{"command":"private-command"}},"schemaVersion":1}"#;
    let canonical_path = home.canonical_configuration();
    home.write_file(&canonical_path, noncanonical);

    let output = stdout(&run_success(add_command(
        &home,
        "alpha",
        "private-command",
        &[],
        &[],
    )));

    assert!(
        output
            == "Server \"alpha\" is already up to date (command: <redacted>; arguments: 0; environment keys: 0).\n",
        "semantic no-op output should be stable"
    );
    assert_output_omits(&output, &["private-command"]);
    assert!(
        fs::read(&canonical_path).expect("the original bytes should remain readable")
            == noncanonical,
        "semantic equality must skip byte normalization and replacement"
    );
    assert!(
        !backup_path(&canonical_path).exists(),
        "a semantic no-op should not create a backup"
    );
    assert_no_temporary_files(&canonical_path);
}

#[test]
fn missing_and_malformed_canonical_state_fail_without_mutation_or_value_output() {
    let missing = SyntheticHome::new();
    let missing_path = missing.canonical_configuration();
    let expected_missing = format!(
        "error: canonical configuration does not exist at `{}`; run `mcp-sync init` first\n",
        missing_path.display()
    );
    let missing_add = stderr(&run_failure(add_command(
        &missing,
        "alpha",
        "missing-private-command",
        &["missing-private-argument"],
        &["TOKEN=missing-private-value"],
    )));
    assert_output_omits(
        &missing_add,
        &[
            "missing-private-command",
            "missing-private-argument",
            "missing-private-value",
        ],
    );
    assert_eq!(
        missing_add, expected_missing,
        "missing add diagnostics should be exact and actionable"
    );
    let missing_list = stderr(&run_failure(list_command(&missing)));
    assert!(
        missing_list == expected_missing,
        "missing list diagnostics should be exact and actionable"
    );
    assert!(
        !missing_path.exists(),
        "failure must not initialize state implicitly"
    );

    let malformed = SyntheticHome::new();
    let malformed_path = malformed.canonical_configuration();
    let malformed_bytes = br#"{
  "schemaVersion": 1,
  "servers": {
    "alpha": {
      "command": "stored-private-command",
      "env": {"TOKEN": "stored-private-value",}
    }
  }
}
"#;
    malformed.write_file(&malformed_path, malformed_bytes);
    let malformed_output = stderr(&run_failure(add_command(
        &malformed,
        "beta",
        "requested-private-command",
        &["requested-private-argument"],
        &["TOKEN=requested-private-value"],
    )));
    assert!(
        malformed_output.starts_with("error: canonical configuration at `")
            && malformed_output.contains("is invalid: invalid JSON:")
            && malformed_output.ends_with("; restore or repair it before continuing\n"),
        "malformed canonical diagnostics should be structural and actionable"
    );
    assert_output_omits(
        &malformed_output,
        &[
            "stored-private-command",
            "stored-private-value",
            "requested-private-command",
            "requested-private-argument",
            "requested-private-value",
        ],
    );
    assert!(
        fs::read(&malformed_path).expect("malformed state should remain readable")
            == malformed_bytes,
        "malformed state must remain byte-for-byte unchanged"
    );
    assert!(
        !backup_path(&malformed_path).exists(),
        "validation failure must not create a backup"
    );

    let invalid_utf8 = SyntheticHome::new();
    let invalid_utf8_path = invalid_utf8.canonical_configuration();
    let invalid_utf8_bytes = [0xff, 0xfe, 0xfd];
    invalid_utf8.write_file(&invalid_utf8_path, invalid_utf8_bytes);
    let invalid_utf8_output = stderr(&run_failure(list_command(&invalid_utf8)));
    let expected_utf8 = format!(
        "error: canonical configuration at `{}` is not valid UTF-8; restore or repair it before continuing\n",
        invalid_utf8_path.display()
    );
    assert!(
        invalid_utf8_output == expected_utf8,
        "invalid UTF-8 diagnostics should be exact and actionable"
    );
    assert!(
        fs::read(&invalid_utf8_path).expect("invalid bytes should remain readable")
            == invalid_utf8_bytes,
        "list failure must not mutate invalid input"
    );
    assert!(!backup_path(&invalid_utf8_path).exists());
}

#[test]
fn invalid_add_input_fails_before_canonical_mutation_and_never_echoes_values() {
    let home = SyntheticHome::new();
    write_empty_canonical(&home);
    let canonical_path = home.canonical_configuration();

    let missing_separator = stderr(&run_failure(add_command(
        &home,
        "alpha",
        "private-command",
        &[],
        &["private-value-without-separator"],
    )));
    assert!(
        missing_separator
            == "error: environment assignment 1 must use `KEY=VALUE`; no value was written\n",
        "missing-separator diagnostics should not echo input"
    );
    assert_output_omits(
        &missing_separator,
        &["private-command", "private-value-without-separator"],
    );

    let duplicate = stderr(&run_failure(add_command(
        &home,
        "alpha",
        "private-command",
        &[],
        &["TOKEN=first-private-value", "TOKEN=second-private-value"],
    )));
    assert!(
        duplicate
            == "error: environment key \"TOKEN\" was supplied more than once; provide each key once\n",
        "duplicate-key diagnostics should identify only the structural key"
    );
    assert_output_omits(
        &duplicate,
        &[
            "private-command",
            "first-private-value",
            "second-private-value",
        ],
    );

    let padded_name = stderr(&run_failure(add_command(
        &home,
        " padded-name ",
        "private-command",
        &[],
        &[],
    )));
    assert!(
        padded_name
            == "error: cannot add the requested server definition: server name at position 0 must not have surrounding whitespace\n",
        "invalid-name diagnostics should be structural"
    );
    assert_output_omits(&padded_name, &["private-command"]);

    let invalid_command = stderr(&run_failure(add_command(
        &home,
        "alpha",
        "private\ncommand",
        &[],
        &[],
    )));
    assert!(
        invalid_command
            == "error: cannot add the requested server definition: command for server \"alpha\" must not contain control characters\n",
        "invalid-command diagnostics should identify only the field"
    );
    assert_output_omits(&invalid_command, &["private\ncommand"]);

    assert!(
        fs::read(&canonical_path).expect("canonical state should remain readable")
            == EMPTY_CANONICAL,
        "invalid requests must not mutate canonical state"
    );
    assert!(!backup_path(&canonical_path).exists());
    assert_no_temporary_files(&canonical_path);
}

#[test]
fn a_non_regular_backup_blocks_replacement_without_changing_canonical_state() {
    let home = SyntheticHome::new();
    write_empty_canonical(&home);
    let canonical_path = home.canonical_configuration();
    let backup = backup_path(&canonical_path);
    fs::create_dir(&backup).expect("a synthetic backup collision should be created");

    let output = stderr(&run_failure(add_command(
        &home,
        "alpha",
        "private-command",
        &["private-argument"],
        &["TOKEN=private-value"],
    )));
    let expected = format!(
        "error: cannot update canonical configuration: refusing to replace directory `{}`; a regular file is required\n",
        backup.display()
    );
    assert_output_omits(
        &output,
        &["private-command", "private-argument", "private-value"],
    );
    assert_eq!(
        output, expected,
        "backup-collision diagnostics should be exact and actionable"
    );
    assert!(
        fs::read(&canonical_path).expect("canonical state should remain readable")
            == EMPTY_CANONICAL,
        "a backup collision must leave canonical state unchanged"
    );
    assert!(
        backup.is_dir(),
        "the colliding backup directory must survive"
    );
    assert_no_temporary_files(&canonical_path);
}

#[cfg(unix)]
#[test]
fn a_symbolic_link_canonical_path_is_never_replaced() {
    use std::os::unix::fs::symlink;

    let home = SyntheticHome::new();
    let canonical_path = home.canonical_configuration();
    let target = home.root().join("canonical-target.json");
    home.write_file(&target, EMPTY_CANONICAL);
    fs::create_dir_all(
        canonical_path
            .parent()
            .expect("the canonical path should have a parent"),
    )
    .expect("the canonical directory should be created");
    symlink(&target, &canonical_path).expect("the synthetic canonical link should be created");

    let output = stderr(&run_failure(add_command(
        &home,
        "alpha",
        "private-command",
        &["private-argument"],
        &["TOKEN=private-value"],
    )));
    let expected = format!(
        "error: cannot update canonical configuration: refusing to replace symbolic link `{}`; a regular file is required\n",
        canonical_path.display()
    );
    assert!(
        output == expected,
        "symbolic-link diagnostics should be exact and actionable"
    );
    assert_output_omits(
        &output,
        &["private-command", "private-argument", "private-value"],
    );
    assert!(
        fs::symlink_metadata(&canonical_path)
            .expect("the link should remain inspectable")
            .file_type()
            .is_symlink(),
        "the canonical symbolic link must remain a link"
    );
    assert!(
        fs::read(&target).expect("the target should remain readable") == EMPTY_CANONICAL,
        "the symbolic-link target must remain unchanged"
    );
    assert!(!backup_path(&canonical_path).exists());
    assert_no_temporary_files(&canonical_path);
}

#[cfg(unix)]
#[test]
fn a_write_permission_failure_preserves_canonical_state_and_creates_no_backup() {
    use std::os::unix::fs::PermissionsExt as _;

    let home = SyntheticHome::new();
    write_empty_canonical(&home);
    home.write_file(&home.operation_lock(), b"");
    let canonical_path = home.canonical_configuration();
    let parent = canonical_path
        .parent()
        .expect("the canonical path should have a parent");
    fs::set_permissions(parent, fs::Permissions::from_mode(0o500))
        .expect("the canonical directory should become read-only");

    let output = run_failure(add_command(
        &home,
        "alpha",
        "private-command",
        &["private-argument"],
        &["TOKEN=private-value"],
    ));
    fs::set_permissions(parent, fs::Permissions::from_mode(0o700))
        .expect("the canonical directory permissions should be restored");
    let diagnostic = stderr(&output);

    assert!(
        diagnostic.starts_with("error: cannot update canonical configuration:")
            && diagnostic.contains("create temporary file")
            && diagnostic.contains(&canonical_path.to_string_lossy().to_string()),
        "permission diagnostics should identify the failed operation and path"
    );
    assert_output_omits(
        &diagnostic,
        &["private-command", "private-argument", "private-value"],
    );
    assert!(
        fs::read(&canonical_path).expect("canonical state should remain readable")
            == EMPTY_CANONICAL,
        "a permission failure must leave canonical state unchanged"
    );
    assert!(!backup_path(&canonical_path).exists());
    assert_no_temporary_files(&canonical_path);
}

#[test]
fn add_accepts_the_documented_command_alias_and_literal_equals_values() {
    let home = SyntheticHome::new();
    write_empty_canonical(&home);
    let mut command = AssertCommand::from_std(home.command());
    command
        .args(["add", "alpha", "--cmd", "private-command"])
        .arg("--arg=--literal-hyphen")
        .arg("--arg=")
        .arg("--env=URL=scheme://host/path?left=right")
        .arg("--env=EMPTY=")
        .timeout(COMMAND_TIMEOUT);

    let output = stdout(&run_success(command));
    assert!(
        output
            == "Added server \"alpha\" (command: <redacted>; arguments: 2; environment keys: 2).\n",
        "the documented alias should map to the same redacted add behavior"
    );
    assert_output_omits(
        &output,
        &[
            "private-command",
            "--literal-hyphen",
            "scheme://host/path?left=right",
        ],
    );
    let canonical = fs::read(home.canonical_configuration())
        .expect("the canonical configuration should be readable");
    let parsed: Value =
        serde_json::from_slice(&canonical).expect("canonical output should be valid JSON");
    assert!(
        parsed["servers"]["alpha"]["args"][0] == "--literal-hyphen"
            && parsed["servers"]["alpha"]["args"][1] == ""
            && parsed["servers"]["alpha"]["env"]["URL"] == "scheme://host/path?left=right"
            && parsed["servers"]["alpha"]["env"]["EMPTY"] == "",
        "literal arguments and values should be preserved without shell parsing"
    );
}

#[test]
fn list_escapes_structural_names_and_environment_keys() {
    let home = SyntheticHome::new();
    let canonical = br#"{
  "schemaVersion": 1,
  "servers": {
    "quoted\"name": {
      "command": "private-command",
      "args": ["private-argument"],
      "env": {
        "LINE\nBREAK": "private-value"
      }
    }
  }
}
"#;
    home.write_file(&home.canonical_configuration(), canonical);

    let output = stdout(&run_success(list_command(&home)));
    assert!(
        output
            == "\"quoted\\\"name\": command: <redacted>; arguments: 1; environment keys: \"LINE\\nBREAK\"\n",
        "structural strings should be escaped on one terminal line"
    );
    assert_output_omits(
        &output,
        &["private-command", "private-argument", "private-value"],
    );
}
