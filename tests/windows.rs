#![cfg(windows)]

mod support;

use assert_cmd::Command as AssertCommand;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use support::{SyntheticHome, assert_file_matches};

const COMMAND_TIMEOUT: Duration = Duration::from_secs(10);

fn backup_path(path: &Path) -> PathBuf {
    let mut path = path.as_os_str().to_owned();
    path.push(".bak");
    PathBuf::from(path)
}

fn portable_command(home: &SyntheticHome, executable: &Path) -> AssertCommand {
    let mut command = AssertCommand::from_std(home.command_for(executable));
    command.timeout(COMMAND_TIMEOUT);
    command
}

#[test]
fn a_copied_portable_binary_uses_windows_profile_roots_and_safe_replacement() {
    let home = SyntheticHome::new();
    let package_directory = home.root().join("portable-package");
    fs::create_dir(&package_directory).expect("the portable package directory should be created");
    let portable_executable = package_directory.join("mcp-sync.exe");
    fs::copy(env!("CARGO_BIN_EXE_mcp-sync"), &portable_executable)
        .expect("the built executable should be copied into the portable package");

    let original_claude = br#"{
  "mcpServers": {
    "portable": {
      "command": "portable-server.exe",
      "args": ["--stdio"],
      "env": {
        "PORTABLE_TOKEN": "synthetic-private-value"
      }
    }
  },
  "preserved": true
}
"#;
    home.write_file(&home.claude_desktop_configuration(), original_claude);

    portable_command(&home, &portable_executable)
        .arg("init")
        .assert()
        .success()
        .stderr("");

    let canonical = home.canonical_configuration();
    assert_eq!(
        canonical,
        home.user_root().join("AppData/Local/mcp-sync/config.json")
    );
    assert!(canonical.is_file());
    assert!(
        !package_directory.join("mcp-sync/config.json").exists(),
        "portable execution must not make configuration relative to the executable"
    );
    let canonical_before_add =
        fs::read(&canonical).expect("the imported canonical configuration should remain readable");

    portable_command(&home, &portable_executable)
        .args([
            "add",
            "portable",
            "--command",
            "portable-server-v2.exe",
            "--arg",
            "--stdio",
            "--arg",
            "--portable",
            "--env",
            "PORTABLE_TOKEN=rotated-synthetic-private-value",
        ])
        .assert()
        .success()
        .stderr("");
    assert_file_matches(
        &backup_path(&canonical),
        &canonical_before_add,
        "Windows canonical replacement should retain exact prior bytes",
    );

    portable_command(&home, &portable_executable)
        .arg("sync")
        .assert()
        .success()
        .stderr("");

    assert_file_matches(
        &backup_path(&home.claude_desktop_configuration()),
        original_claude,
        "Windows target replacement should retain exact prior bytes",
    );
    for target in [
        home.cursor_configuration(),
        home.windsurf_configuration(),
        home.vscode_configuration(),
        home.codex_configuration(),
    ] {
        assert!(
            target.is_file(),
            "sync should create every missing Windows target"
        );
        assert!(
            !backup_path(&target).exists(),
            "create-only target publication should not invent a backup"
        );
    }

    portable_command(&home, &portable_executable)
        .args(["sync", "--dry-run"])
        .assert()
        .success()
        .stderr("");
}
