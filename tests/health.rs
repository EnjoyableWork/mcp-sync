mod support;

use assert_cmd::Command as AssertCommand;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;
use support::SyntheticHome;

const COMMAND_TIMEOUT: Duration = Duration::from_secs(8);
static PROCESS_FIXTURE_LOCK: Mutex<()> = Mutex::new(());

fn process_fixture_lock() -> MutexGuard<'static, ()> {
    PROCESS_FIXTURE_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn health_command(home: &SyntheticHome, name: &str) -> AssertCommand {
    let mut command = AssertCommand::from_std(home.command());
    command.arg("test").arg(name).timeout(COMMAND_TIMEOUT);
    command
}

fn run_success(mut command: AssertCommand) -> Output {
    let assertion = command.assert();
    let output = assertion.get_output().clone();
    assert!(output.status.success(), "the health command should succeed");
    assert!(output.stderr.is_empty(), "success should not write stderr");
    output
}

fn run_failure(mut command: AssertCommand) -> Output {
    let assertion = command.assert();
    let output = assertion.get_output().clone();
    assert!(!output.status.success(), "the health command should fail");
    assert!(output.stdout.is_empty(), "failure should not write stdout");
    output
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be UTF-8")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr should be UTF-8")
}

fn write_server(
    home: &SyntheticHome,
    name: &str,
    script: &Path,
    environment: BTreeMap<String, String>,
) -> String {
    #[cfg(windows)]
    let environment = {
        let mut environment = environment;
        environment.insert(
            "SystemRoot".to_owned(),
            std::env::var("SystemRoot").expect("Windows should define SystemRoot"),
        );
        environment
    };
    let (command, arguments) = server_launch(script);
    let mut servers = serde_json::Map::new();
    servers.insert(
        name.to_owned(),
        json!({
            "command": command,
            "args": arguments,
            "env": environment,
        }),
    );
    let document = json!({"schemaVersion": 1, "servers": servers});
    let mut bytes = serde_json::to_vec_pretty(&document)
        .expect("synthetic canonical configuration should serialize");
    bytes.push(b'\n');
    home.write_file(&home.canonical_configuration(), bytes);
    command
}

fn script_path(home: &SyntheticHome, name: &str) -> PathBuf {
    let extension = if cfg!(windows) { "ps1" } else { "sh" };
    home.root().join(format!("{name}.{extension}"))
}

#[cfg(unix)]
fn server_launch(script: &Path) -> (String, Vec<String>) {
    (
        "/bin/sh".to_owned(),
        vec![script.to_string_lossy().into_owned()],
    )
}

#[cfg(windows)]
fn powershell_path() -> PathBuf {
    PathBuf::from(std::env::var_os("SystemRoot").expect("Windows should define SystemRoot"))
        .join("System32/WindowsPowerShell/v1.0/powershell.exe")
}

#[cfg(windows)]
fn server_launch(script: &Path) -> (String, Vec<String>) {
    (
        powershell_path().to_string_lossy().into_owned(),
        vec![
            "-NoLogo".to_owned(),
            "-NoProfile".to_owned(),
            "-NonInteractive".to_owned(),
            "-ExecutionPolicy".to_owned(),
            "Bypass".to_owned(),
            "-File".to_owned(),
            script.to_string_lossy().into_owned(),
        ],
    )
}

fn assert_output_omits(output: &str, private_values: &[&str]) {
    for private in private_values {
        assert!(
            !output.contains(private),
            "health output must omit process and protocol values"
        );
    }
}

#[cfg(unix)]
fn process_exists(pid: &str) -> bool {
    Command::new("/bin/kill")
        .arg("-0")
        .arg(pid)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

#[cfg(windows)]
fn process_exists(pid: &str) -> bool {
    let pid = pid
        .trim()
        .parse::<u32>()
        .expect("the Windows child should publish a numeric process identifier");
    Command::new(powershell_path())
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            &format!(
                "if (Get-Process -Id {pid} -ErrorAction SilentlyContinue) {{ exit 0 }} else {{ exit 1 }}"
            ),
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

#[test]
fn built_binary_completes_initialize_and_initialized_with_redacted_output() {
    let _process_fixture = process_fixture_lock();
    let home = SyntheticHome::new();
    let script = script_path(&home, "healthy-server");
    let request_path = home.root().join("initialize-request.json");
    let notification_path = home.root().join("initialized-notification.json");
    let exit_marker = home.root().join("server-exited");
    let private_environment = "synthetic-health-private-value";
    let script_contents = if cfg!(windows) {
        r#"
if (Test-Path Env:HOME) { exit 40 }
if ($env:PRIVATE_ENVIRONMENT -ne "synthetic-health-private-value") { exit 41 }
$initialize = [Console]::In.ReadLine()
if ($null -eq $initialize) { exit 42 }
[IO.File]::WriteAllText($env:REQUEST_PATH, $initialize)
[Console]::Out.WriteLine('{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2025-11-25","capabilities":{"tools":{"listChanged":true}},"serverInfo":{"name":"synthetic-health-server","version":"1.0"}}}')
[Console]::Out.Flush()
$initialized = [Console]::In.ReadLine()
if ($null -eq $initialized) { exit 43 }
[IO.File]::WriteAllText($env:NOTIFICATION_PATH, $initialized)
[IO.File]::WriteAllText($env:EXIT_MARKER, 'exited')
exit 0
"#
    } else {
        r#"
if [ "${HOME+x}" = x ]; then exit 40; fi
if [ "$PRIVATE_ENVIRONMENT" != "synthetic-health-private-value" ]; then exit 41; fi
IFS= read -r initialize || exit 42
printf '%s' "$initialize" > "$REQUEST_PATH"
printf '%s\n' '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2025-11-25","capabilities":{"tools":{"listChanged":true}},"serverInfo":{"name":"synthetic-health-server","version":"1.0"}}}'
IFS= read -r initialized || exit 43
printf '%s' "$initialized" > "$NOTIFICATION_PATH"
printf 'exited' > "$EXIT_MARKER"
exit 0
"#
    };
    home.write_file(&script, script_contents);
    let process_command = write_server(
        &home,
        "healthy",
        &script,
        BTreeMap::from([
            (
                "REQUEST_PATH".to_owned(),
                request_path.to_string_lossy().into_owned(),
            ),
            (
                "NOTIFICATION_PATH".to_owned(),
                notification_path.to_string_lossy().into_owned(),
            ),
            (
                "EXIT_MARKER".to_owned(),
                exit_marker.to_string_lossy().into_owned(),
            ),
            (
                "PRIVATE_ENVIRONMENT".to_owned(),
                private_environment.to_owned(),
            ),
        ]),
    );

    let output = run_success(health_command(&home, "healthy"));

    let output_text = stdout(&output);
    assert_eq!(
        output_text,
        "Server \"healthy\" initialized successfully via STDIO (protocol version 2025-11-25).\n"
    );
    let script_text = script.to_string_lossy();
    assert_output_omits(
        &output_text,
        &[&process_command, script_text.as_ref(), private_environment],
    );
    assert_eq!(
        fs::read_to_string(exit_marker).expect("healthy child should exit"),
        "exited"
    );

    let request: Value = serde_json::from_slice(
        &fs::read(request_path).expect("server should capture the initialize request"),
    )
    .expect("initialize request should be JSON");
    assert_eq!(request["jsonrpc"], "2.0");
    assert_eq!(request["id"], 1);
    assert_eq!(request["method"], "initialize");
    assert_eq!(request["params"]["protocolVersion"], "2025-11-25");
    assert_eq!(request["params"]["capabilities"], json!({}));
    assert_eq!(request["params"]["clientInfo"]["name"], "mcp-sync");

    let notification: Value = serde_json::from_slice(
        &fs::read(notification_path).expect("server should capture the initialized notification"),
    )
    .expect("initialized notification should be JSON");
    assert_eq!(notification["jsonrpc"], "2.0");
    assert_eq!(notification["method"], "notifications/initialized");
    assert!(notification.get("id").is_none());
}

#[test]
fn built_binary_timeout_is_bounded_and_reaps_the_silent_child() {
    let _process_fixture = process_fixture_lock();
    let home = SyntheticHome::new();
    let script = script_path(&home, "silent-server");
    let pid_path = home.root().join("silent-server.pid");
    let private_argument = "synthetic-timeout-private-argument";
    let private_environment = "synthetic-timeout-private-environment";
    let script_contents = if cfg!(windows) {
        r#"
[IO.File]::WriteAllText($env:PID_PATH, [string]$PID)
$initialize = [Console]::In.ReadLine()
if ($null -eq $initialize) { exit 50 }
Start-Sleep -Seconds 30
"#
    } else {
        r#"
printf '%s' "$$" > "$PID_PATH"
IFS= read -r initialize || exit 50
IFS= read -r never
"#
    };
    home.write_file(&script, script_contents);
    let process_command = write_server(
        &home,
        "silent",
        &script,
        BTreeMap::from([
            (
                "PID_PATH".to_owned(),
                pid_path.to_string_lossy().into_owned(),
            ),
            ("PRIVATE_ARGUMENT".to_owned(), private_argument.to_owned()),
            (
                "PRIVATE_ENVIRONMENT".to_owned(),
                private_environment.to_owned(),
            ),
        ]),
    );
    let output = run_failure(health_command(&home, "silent"));

    let error = stderr(&output);
    assert_eq!(
        error,
        "error: health test for server \"silent\" failed: the process did not return an initialize response within 5 seconds; it was terminated\n"
    );
    let script_text = script.to_string_lossy();
    assert_output_omits(
        &error,
        &[
            &process_command,
            script_text.as_ref(),
            private_argument,
            private_environment,
        ],
    );
    let pid = fs::read_to_string(pid_path).expect("silent child should publish its pid");
    assert!(!process_exists(&pid), "the timed-out child must be reaped");
}

#[test]
fn built_binary_rejects_malformed_stdout_and_reaps_the_child_without_echoing_it() {
    let _process_fixture = process_fixture_lock();
    let home = SyntheticHome::new();
    let script = script_path(&home, "malformed-server");
    let pid_path = home.root().join("malformed-server.pid");
    let private_stdout = "synthetic-malformed-private-output";
    let script_contents = if cfg!(windows) {
        r#"
[IO.File]::WriteAllText($env:PID_PATH, [string]$PID)
$initialize = [Console]::In.ReadLine()
if ($null -eq $initialize) { exit 60 }
[Console]::Out.WriteLine('not-json-' + $env:PRIVATE_STDOUT)
[Console]::Out.Flush()
while ($true) { Start-Sleep -Milliseconds 10 }
"#
    } else {
        r#"
printf '%s' "$$" > "$PID_PATH"
IFS= read -r initialize || exit 60
printf 'not-json-%s\n' "$PRIVATE_STDOUT"
while :; do :; done
"#
    };
    home.write_file(&script, script_contents);
    let process_command = write_server(
        &home,
        "malformed",
        &script,
        BTreeMap::from([
            (
                "PID_PATH".to_owned(),
                pid_path.to_string_lossy().into_owned(),
            ),
            ("PRIVATE_STDOUT".to_owned(), private_stdout.to_owned()),
        ]),
    );

    let output = run_failure(health_command(&home, "malformed"));

    let error = stderr(&output);
    assert_eq!(
        error,
        "error: health test for server \"malformed\" failed: the process returned an invalid initialize response: stdout was not one duplicate-free JSON-RPC message\n"
    );
    let script_text = script.to_string_lossy();
    assert_output_omits(
        &error,
        &[&process_command, script_text.as_ref(), private_stdout],
    );
    let pid = fs::read_to_string(pid_path).expect("malformed child should publish its pid");
    assert!(!process_exists(&pid), "the malformed child must be reaped");
}

#[test]
fn built_binary_redacts_json_rpc_error_message_data_and_stderr() {
    let _process_fixture = process_fixture_lock();
    let home = SyntheticHome::new();
    let script = script_path(&home, "rejecting-server");
    let private_response = "synthetic-rejection-private-response";
    let private_stderr = "synthetic-rejection-private-stderr";
    let script_contents = if cfg!(windows) {
        r#"
$initialize = [Console]::In.ReadLine()
if ($null -eq $initialize) { exit 70 }
[Console]::Error.WriteLine($env:PRIVATE_STDERR)
$response = '{"jsonrpc":"2.0","id":1,"error":{"code":-32603,"message":"' + $env:PRIVATE_RESPONSE + '","data":{"token":"' + $env:PRIVATE_RESPONSE + '"}}}'
[Console]::Out.WriteLine($response)
[Console]::Out.Flush()
while ($true) { Start-Sleep -Milliseconds 10 }
"#
    } else {
        r#"
IFS= read -r initialize || exit 70
printf '%s\n' "$PRIVATE_STDERR" >&2
printf '{"jsonrpc":"2.0","id":1,"error":{"code":-32603,"message":"%s","data":{"token":"%s"}}}\n' "$PRIVATE_RESPONSE" "$PRIVATE_RESPONSE"
while :; do :; done
"#
    };
    home.write_file(&script, script_contents);
    let process_command = write_server(
        &home,
        "rejecting",
        &script,
        BTreeMap::from([
            ("PRIVATE_RESPONSE".to_owned(), private_response.to_owned()),
            ("PRIVATE_STDERR".to_owned(), private_stderr.to_owned()),
        ]),
    );

    let output = run_failure(health_command(&home, "rejecting"));

    let error = stderr(&output);
    assert_eq!(
        error,
        "error: health test for server \"rejecting\" failed: the process rejected the initialize request\n"
    );
    let script_text = script.to_string_lossy();
    assert_output_omits(
        &error,
        &[
            &process_command,
            script_text.as_ref(),
            private_response,
            private_stderr,
        ],
    );
}

#[test]
fn unknown_server_and_spawn_failure_are_nonzero_and_structurally_redacted() {
    let home = SyntheticHome::new();
    let private_command = home
        .root()
        .join("synthetic-private-missing-command")
        .to_string_lossy()
        .into_owned();
    let private_argument = "synthetic-private-spawn-argument";
    let private_environment = "synthetic-private-spawn-environment";
    let document = json!({
        "schemaVersion": 1,
        "servers": {
            "broken": {
                "command": private_command,
                "args": [private_argument],
                "env": {"TOKEN": private_environment},
            }
        }
    });
    let mut bytes = serde_json::to_vec_pretty(&document)
        .expect("synthetic canonical configuration should serialize");
    bytes.push(b'\n');
    home.write_file(&home.canonical_configuration(), bytes);

    let unknown = run_failure(health_command(&home, "unknown"));
    assert_eq!(
        stderr(&unknown),
        "error: server \"unknown\" is not present in canonical configuration; run `mcp-sync list` to review configured names\n"
    );

    let failed = run_failure(health_command(&home, "broken"));
    let error = stderr(&failed);
    assert!(error.starts_with(
        "error: health test for server \"broken\" failed: could not start the configured process:"
    ));
    assert_output_omits(
        &error,
        &[&private_command, private_argument, private_environment],
    );
}
