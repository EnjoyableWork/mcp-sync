mod support;

use assert_cmd::Command as AssertCommand;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;
use std::time::Duration;
use support::{SyntheticHome, assert_file_matches};

const COMMAND_TIMEOUT: Duration = Duration::from_secs(10);

const OLD_CANONICAL: &[u8] = br#"{
  "schemaVersion": 1,
  "servers": {
    "recovery-server": {
      "command": "old-private-command",
      "args": ["--old-private-argument"],
      "env": {
        "TOKEN": "old-private-token"
      }
    }
  }
}
"#;

const OLD_CLAUDE: &[u8] = br#"{
  "mcpServers": {
    "recovery-server": {
      "command": "old-private-command",
      "args": ["--old-private-argument"],
      "env": {"TOKEN": "old-private-token"},
      "nativePrivate": "claude-private-value"
    }
  },
  "preserved": "claude-root-private-value"
}
"#;

const OLD_CURSOR: &[u8] = br#"{
  "mcpServers": {
    "recovery-server": {
      "type": "stdio",
      "command": "old-private-command",
      "args": ["--old-private-argument"],
      "env": {"TOKEN": "old-private-token"},
      "envFile": "cursor-private-value"
    }
  }
}
"#;

const OLD_WINDSURF: &[u8] = br#"{
  "mcpServers": {
    "recovery-server": {
      "command": "old-private-command",
      "args": ["--old-private-argument"],
      "env": {"TOKEN": "old-private-token"},
      "disabledTools": ["windsurf-private-value"]
    }
  }
}
"#;

const OLD_VSCODE: &[u8] = br#"{
  "servers": {
    "recovery-server": {
      "type": "stdio",
      "command": "old-private-command",
      "args": ["--old-private-argument"],
      "env": {"TOKEN": "old-private-token"},
      "envFile": "vscode-private-value"
    }
  },
  "inputs": [{"id": "vscode-input-private-value", "type": "promptString"}]
}
"#;

const OLD_CODEX: &[u8] = br#"# retained Codex private configuration
model = "codex-root-private-value"

[mcp_servers.recovery-server]
command = "old-private-command"
args = ["--old-private-argument"]
cwd = "codex-private-value"

[mcp_servers.recovery-server.env]
TOKEN = "old-private-token"
"#;

const OLD_KIRO: &[u8] = br#"{
  // retained Kiro private configuration comment
  "mcpServers": {
    "recovery-server": {
      "command": "old-private-command",
      "args": ["--old-private-argument"],
      "env": {"TOKEN": "old-private-token"},
      "cwd": "kiro-private-value",
      "disabled": false
    },
    "remote-private": {
      "url": "https://kiro-restore-private.invalid/mcp",
      "headers": {"Authorization": "Bearer kiro-restore-private-token"}
    },
  },
}
"#;

const PRIVATE_VALUES: &[&str] = &[
    "old-private-command",
    "--old-private-argument",
    "old-private-token",
    "new-private-command",
    "--new-private-argument",
    "new-private-token",
    "claude-private-value",
    "claude-root-private-value",
    "cursor-private-value",
    "windsurf-private-value",
    "vscode-private-value",
    "vscode-input-private-value",
    "codex-root-private-value",
    "codex-private-value",
    "kiro-private-value",
    "https://kiro-restore-private.invalid/mcp",
    "kiro-restore-private-token",
    "private-invalid-backup-sentinel",
    "corrupt-current-private-sentinel",
];

fn backup_path(path: &Path) -> PathBuf {
    let mut path = path.as_os_str().to_owned();
    path.push(".bak");
    PathBuf::from(path)
}

fn add_update_command(home: &SyntheticHome) -> AssertCommand {
    let mut command = AssertCommand::from_std(home.command());
    command
        .args([
            "add",
            "recovery-server",
            "--command",
            "new-private-command",
            "--arg",
            "--new-private-argument",
            "--env",
            "TOKEN=new-private-token",
        ])
        .timeout(COMMAND_TIMEOUT);
    command
}

fn sync_command(home: &SyntheticHome) -> AssertCommand {
    let mut command = AssertCommand::from_std(home.command());
    command.arg("sync").timeout(COMMAND_TIMEOUT);
    command
}

fn restore_command(home: &SyntheticHome, target: &str, dry_run: bool) -> AssertCommand {
    let mut command = AssertCommand::from_std(home.command());
    command.arg("restore").arg(target);
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

fn assert_redacted(output: &str) {
    for private in PRIVATE_VALUES {
        assert!(
            !output.contains(private),
            "restore output must not expose configuration or backup values"
        );
    }
}

fn assert_no_temporary_files(path: &Path) {
    let parent = path.parent().expect("a managed path has a parent");
    if !parent.exists() {
        return;
    }
    let has_temporary = fs::read_dir(parent)
        .expect("the managed directory should be readable")
        .filter_map(Result::ok)
        .any(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with(".mcp-sync-")
        });
    assert!(
        !has_temporary,
        "temporary restore files should be cleaned up"
    );
}

fn targets(home: &SyntheticHome) -> Vec<(&'static str, PathBuf, &'static [u8])> {
    vec![
        ("canonical", home.canonical_configuration(), OLD_CANONICAL),
        (
            "claude-desktop",
            home.claude_desktop_configuration(),
            OLD_CLAUDE,
        ),
        ("cursor", home.cursor_configuration(), OLD_CURSOR),
        ("windsurf", home.windsurf_configuration(), OLD_WINDSURF),
        ("vscode", home.vscode_configuration(), OLD_VSCODE),
        ("codex", home.codex_configuration(), OLD_CODEX),
        ("kiro", home.kiro_configuration(), OLD_KIRO),
    ]
}

fn write_old_journey(home: &SyntheticHome) {
    for (_, path, old) in targets(home) {
        home.write_file(&path, old);
    }
}

#[test]
fn built_binary_restore_journey_is_dry_run_safe_exact_redacted_and_reversible() {
    let home = SyntheticHome::new();
    write_old_journey(&home);

    let add_output = stdout(&run_success(add_update_command(&home)));
    let sync_output = stdout(&run_success(sync_command(&home)));
    assert_redacted(&add_output);
    assert_redacted(&sync_output);

    for (argument, path, old) in targets(&home) {
        let backup = backup_path(&path);
        assert_file_matches(
            &backup,
            old,
            "the write preceding restore should retain exact old bytes",
        );
        let updated = fs::read(&path).expect("the updated target should be readable");
        assert_ne!(updated, old, "the journey must create a real restore delta");

        let before_dry_target = updated.clone();
        let before_dry_backup = fs::read(&backup).expect("the backup should remain readable");
        let dry_output = stdout(&run_success(restore_command(&home, argument, true)));
        assert!(dry_output.contains("would be restored"));
        assert!(dry_output.contains("would become the retained backup"));
        assert_redacted(&dry_output);
        assert_file_matches(
            &path,
            &before_dry_target,
            "restore dry-run must preserve target bytes",
        );
        assert_file_matches(
            &backup,
            &before_dry_backup,
            "restore dry-run must preserve backup bytes",
        );

        let restore_output = stdout(&run_success(restore_command(&home, argument, false)));
        assert!(restore_output.contains("restored from"));
        assert!(restore_output.contains("previous target is now the retained backup"));
        assert_redacted(&restore_output);
        assert_file_matches(&path, old, "restore must publish exact retained bytes");
        assert_file_matches(
            &backup,
            &updated,
            "restore must retain the exact immediately preceding target",
        );
        assert_no_temporary_files(&path);

        let undo_output = stdout(&run_success(restore_command(&home, argument, false)));
        assert_redacted(&undo_output);
        assert_file_matches(
            &path,
            &updated,
            "a second restore should recover the immediately preceding valid target",
        );
        assert_file_matches(
            &backup,
            old,
            "a second restore should rotate the one retained generation",
        );
        assert_no_temporary_files(&path);
    }
}

#[test]
fn kiro_restore_uses_only_the_absolute_kiro_home_profile_selected_for_the_process() {
    let home = SyntheticHome::new();
    let default = b"default Kiro target must not be inspected\n";
    let kiro_home = home.root().join("kiro-restore-profile");
    let path = kiro_home.join("settings/mcp.json");
    let backup = backup_path(&path);
    let current = br#"{
  // current relocated Kiro profile
  "mcpServers": {
    "current": {"command": "new-private-command"}
  }
}
"#;
    home.write_file(&home.kiro_configuration(), default);
    home.write_file(&path, current);
    home.write_file(&backup, OLD_KIRO);

    let command = |dry_run: bool| {
        let mut process = home.command();
        process.env("KIRO_HOME", &kiro_home);
        let mut command = AssertCommand::from_std(process);
        command.arg("restore").arg("kiro");
        if dry_run {
            command.arg("--dry-run");
        }
        command.timeout(COMMAND_TIMEOUT);
        command
    };

    let dry_output = stdout(&run_success(command(true)));
    assert!(dry_output.contains("Kiro"));
    assert!(dry_output.contains("would be restored"));
    assert_redacted(&dry_output);
    assert_file_matches(
        &home.kiro_configuration(),
        default,
        "KIRO_HOME restore dry-run must not inspect or change the default target",
    );
    assert_file_matches(
        &path,
        current,
        "dry-run must preserve relocated target bytes",
    );
    assert_file_matches(
        &backup,
        OLD_KIRO,
        "dry-run must preserve relocated backup bytes",
    );

    let output = stdout(&run_success(command(false)));
    assert!(output.contains("Kiro"));
    assert!(output.contains("restored from"));
    assert_redacted(&output);
    assert_file_matches(
        &home.kiro_configuration(),
        default,
        "KIRO_HOME restore must leave the default target untouched",
    );
    assert_file_matches(
        &path,
        OLD_KIRO,
        "KIRO_HOME restore must publish the selected retained bytes exactly",
    );
    assert_file_matches(
        &backup,
        current,
        "KIRO_HOME restore must retain the selected profile's exact prior target",
    );
    assert_no_temporary_files(&path);
}

#[test]
fn built_binary_recreates_missing_targets_without_consuming_retained_backups() {
    let home = SyntheticHome::new();

    for (argument, path, retained) in targets(&home) {
        let backup = backup_path(&path);
        home.write_file(&backup, retained);

        let dry_output = stdout(&run_success(restore_command(&home, argument, true)));
        assert!(dry_output.contains("would be recreated"));
        assert!(!path.exists());
        assert_file_matches(
            &backup,
            retained,
            "missing-target dry-run must preserve retained bytes",
        );

        let restore_output = stdout(&run_success(restore_command(&home, argument, false)));
        assert!(restore_output.contains("recreated from retained backup"));
        assert!(restore_output.contains("backup was preserved"));
        assert_redacted(&restore_output);
        assert_file_matches(&path, retained, "restore should recreate exact bytes");
        assert_file_matches(
            &backup,
            retained,
            "recreation should not consume or rotate the retained backup",
        );

        let no_op_target = fs::read(&path).unwrap();
        let no_op_backup = fs::read(&backup).unwrap();
        let no_op_output = stdout(&run_success(restore_command(&home, argument, false)));
        assert!(no_op_output.contains("already matches retained backup"));
        assert!(no_op_output.contains("no files changed"));
        assert_file_matches(&path, &no_op_target, "no-op should preserve target bytes");
        assert_file_matches(&backup, &no_op_backup, "no-op should preserve backup bytes");
        assert_no_temporary_files(&path);

        let corrupt_current = b"corrupt-current-private-sentinel\n";
        home.write_file(&path, corrupt_current);
        let recovery_output = stdout(&run_success(restore_command(&home, argument, false)));
        assert!(recovery_output.contains("restored from"));
        assert_redacted(&recovery_output);
        assert_file_matches(
            &path,
            retained,
            "a valid backup should recover a malformed current target",
        );
        assert_file_matches(
            &backup,
            corrupt_current,
            "recovery should retain even malformed immediately preceding target bytes",
        );
        assert_no_temporary_files(&path);
    }
}

#[test]
fn built_binary_refuses_missing_invalid_and_non_regular_backups_without_mutation() {
    let home = SyntheticHome::new();
    let current = b"current-private-restore-sentinel\n";
    let invalid = b"{\"private-invalid-backup-sentinel\": [\n";

    for (argument, path, _) in targets(&home) {
        let backup = backup_path(&path);
        home.write_file(&path, current);
        home.write_file(&backup, invalid);

        let output = run_failure(restore_command(&home, argument, false));
        let diagnostic = stderr(&output);
        assert!(diagnostic.contains("is not a valid"));
        assert_redacted(&diagnostic);
        assert!(!diagnostic.contains("current-private-restore-sentinel"));
        assert_file_matches(
            &path,
            current,
            "invalid backup must not change target bytes",
        );
        assert_file_matches(
            &backup,
            invalid,
            "invalid backup must not change retained bytes",
        );
        assert_no_temporary_files(&path);
    }

    let canonical = home.canonical_configuration();
    let canonical_backup = backup_path(&canonical);
    fs::remove_file(&canonical_backup).expect("the synthetic backup should be removable");
    let missing = stderr(&run_failure(restore_command(&home, "canonical", false)));
    assert!(missing.contains("adjacent backup"));
    assert!(missing.contains("does not exist"));
    assert_file_matches(
        &canonical,
        current,
        "missing backup must not change the current target",
    );
    let transaction = PathBuf::from(format!("{}.mcp-sync-transaction.json", canonical.display()));
    assert!(
        !transaction.exists(),
        "a missing restore source must fail during planning before a replacement transaction starts"
    );

    fs::create_dir(&canonical_backup).expect("a blocking backup directory should be created");
    let non_regular = stderr(&run_failure(restore_command(&home, "canonical", false)));
    assert!(non_regular.contains("cannot inspect canonical configuration backup"));
    assert_file_matches(
        &canonical,
        current,
        "non-regular backup must not change the current target",
    );
    assert!(canonical_backup.is_dir());
    assert_no_temporary_files(&canonical);
}

#[cfg(unix)]
#[test]
fn built_binary_refuses_a_backup_symlink_after_validation_without_touching_either_path() {
    use std::os::unix::fs::symlink;

    let home = SyntheticHome::new();
    let path = home.canonical_configuration();
    let backup = backup_path(&path);
    let linked_backup = home.root().join("linked-private-backup.json");
    let current = b"{\n  \"schemaVersion\": 1,\n  \"servers\": {}\n}\n";
    home.write_file(&path, current);
    home.write_file(&linked_backup, OLD_CANONICAL);
    symlink(&linked_backup, &backup).expect("the synthetic backup symlink should be created");

    let diagnostic = stderr(&run_failure(restore_command(&home, "canonical", false)));

    assert!(diagnostic.contains("refusing to replace symbolic link"));
    assert_redacted(&diagnostic);
    assert_file_matches(
        &path,
        current,
        "a backup symlink refusal must preserve the target",
    );
    assert!(
        fs::symlink_metadata(&backup)
            .expect("the backup symlink should remain")
            .file_type()
            .is_symlink()
    );
    assert_file_matches(
        &linked_backup,
        OLD_CANONICAL,
        "a backup symlink refusal must preserve the link target",
    );
    assert_no_temporary_files(&path);
}

#[cfg(unix)]
#[test]
fn built_binary_refuses_a_target_symlink_before_reading_restore_bytes() {
    use std::os::unix::fs::symlink;

    let home = SyntheticHome::new();
    let path = home.canonical_configuration();
    let backup = backup_path(&path);
    let linked_target = home.root().join("linked-private-target.json");
    home.write_file(&linked_target, OLD_CANONICAL);
    home.write_file(
        &backup,
        b"{\n  \"schemaVersion\": 1,\n  \"servers\": {}\n}\n",
    );
    symlink(&linked_target, &path).expect("the synthetic target symlink should be created");

    let diagnostic = stderr(&run_failure(restore_command(&home, "canonical", true)));

    assert!(diagnostic.contains("refusing to replace symbolic link"));
    assert_redacted(&diagnostic);
    assert!(
        fs::symlink_metadata(&path)
            .expect("the target symlink should remain")
            .file_type()
            .is_symlink()
    );
    assert_file_matches(
        &linked_target,
        OLD_CANONICAL,
        "a target symlink refusal must preserve the link target",
    );
    assert_no_temporary_files(&path);
}

#[cfg(unix)]
#[test]
fn built_binary_refuses_an_unreadable_backup_without_changing_the_target() {
    use std::os::unix::fs::PermissionsExt as _;

    let home = SyntheticHome::new();
    let path = home.canonical_configuration();
    let backup = backup_path(&path);
    let current = b"{\n  \"schemaVersion\": 1,\n  \"servers\": {}\n}\n";
    home.write_file(&path, current);
    home.write_file(&backup, OLD_CANONICAL);
    fs::set_permissions(&backup, fs::Permissions::from_mode(0o000))
        .expect("the backup should become unreadable");

    let diagnostic = stderr(&run_failure(restore_command(&home, "canonical", false)));

    assert!(diagnostic.contains("cannot inspect canonical configuration backup"));
    assert_redacted(&diagnostic);
    assert_file_matches(
        &path,
        current,
        "an unreadable backup must not change the target",
    );
    fs::set_permissions(&backup, fs::Permissions::from_mode(0o600))
        .expect("the backup permissions should be restored for verification");
    assert_file_matches(
        &backup,
        OLD_CANONICAL,
        "an unreadable-backup refusal must preserve retained bytes",
    );
    assert_no_temporary_files(&path);
}
