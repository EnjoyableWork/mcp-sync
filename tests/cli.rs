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
    assert!(stdout.contains("--version"));
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
    synthetic_home.assert_command_is_isolated(&synthetic_home.command());
}
