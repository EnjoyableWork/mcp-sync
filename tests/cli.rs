use std::process::{Command, Output};

fn run_cli(argument: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mcp-sync"))
        .arg(argument)
        .output()
        .expect("mcp-sync should start")
}

#[test]
fn help_describes_the_installed_binary() {
    let output = run_cli("--help");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8(output.stdout).expect("help output should be UTF-8");
    assert!(stdout.contains("Synchronize local Model Context Protocol server configurations"));
    assert!(stdout.contains("Usage: mcp-sync"));
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
