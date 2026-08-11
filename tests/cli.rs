mod support;

use std::process::Output;
use support::SyntheticHome;

fn run_cli(argument: &str) -> Output {
    let synthetic_home = SyntheticHome::new();
    let mut command = synthetic_home.command();
    command.arg(argument);
    command.output().expect("mcp-sync should start")
}

#[test]
fn help_describes_the_installed_binary() {
    let output = run_cli("--help");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8(output.stdout).expect("help output should be UTF-8");
    assert!(stdout.contains("Synchronize local Model Context Protocol server configurations"));
    assert!(stdout.contains("Usage: mcp-sync"));
    assert!(stdout.contains("init"));
    assert!(stdout.contains("add"));
    assert!(stdout.contains("list"));
    assert!(stdout.contains("test"));
    assert!(stdout.contains("restore"));
    assert!(stdout.contains("sync"));
    assert!(stdout.contains("--version"));
}

#[test]
fn restore_help_limits_selection_and_exposes_dry_run() {
    let synthetic_home = SyntheticHome::new();
    let output = synthetic_home
        .command()
        .args(["restore", "--help"])
        .output()
        .expect("restore help should start");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).expect("restore help should be UTF-8");
    assert!(stdout.contains("validated adjacent backup"));
    assert!(stdout.contains("canonical, claude-desktop, cursor, windsurf, vscode, codex, kiro"));
    assert!(stdout.contains("--dry-run"));
    assert!(stdout.contains("without changing the target or backup"));
}

#[test]
fn restore_rejects_arbitrary_paths_before_configuration_access() {
    let synthetic_home = SyntheticHome::new();
    let output = synthetic_home
        .command()
        .args(["restore", "../outside-config.json"])
        .output()
        .expect("invalid restore selection should return");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).expect("restore error should be UTF-8");
    assert!(stderr.contains("invalid value"));
    assert!(stderr.contains("possible values"));
    assert!(
        !synthetic_home.canonical_configuration().exists(),
        "Clap selection validation should happen before path resolution or file access"
    );
}

#[test]
fn version_uses_the_binary_name_and_package_version() {
    let output = run_cli("--version");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8(output.stdout).expect("version output should be UTF-8");
    assert_eq!(stdout, format!("mcp-sync {}\n", env!("CARGO_PKG_VERSION")));
}

#[test]
fn cli_processes_receive_only_synthetic_user_locations() {
    let synthetic_home = SyntheticHome::new();
    assert_ne!(
        synthetic_home.configuration_home(),
        synthetic_home.user_root().join(".config"),
        "the integration harness should exercise an explicit XDG override"
    );
    synthetic_home.assert_command_is_isolated(&synthetic_home.command());
}
