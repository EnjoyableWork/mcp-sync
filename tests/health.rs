mod support;

use assert_cmd::Command as AssertCommand;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use support::{SyntheticHome, process_exists};

const FIXTURE_ARGUMENT: &str = "--mcp-sync-health-fixture";
const HEALTHY_MODE: &str = "healthy";
const SILENT_MODE: &str = "silent";
const MALFORMED_MODE: &str = "malformed";
const REJECTING_MODE: &str = "rejecting";
const SHUTDOWN_TIMEOUT_MODE: &str = "shutdown-timeout";
const COMMAND_TIMEOUT: Duration = Duration::from_secs(8);

// Cargo runs this custom target as one sequential process. Keep separable
// protocol behavior in `src/health.rs` unit tests; only these real process
// journeys are isolated from the parallel unit worker pool that triggered #60.
fn main() {
    match std::env::args().nth(1).as_deref() {
        Some(FIXTURE_ARGUMENT) => run_fixture(
            &std::env::args()
                .nth(2)
                .expect("the native health fixture mode should be provided"),
        ),
        Some(argument) => panic!("unexpected native health harness argument: {argument}"),
        None => {
            prove_explicit_fixture_readiness_and_complete_handshake();
            prove_product_timeout_is_bounded_and_reaps_the_silent_child();
            prove_malformed_output_is_redacted_and_reaps_the_child();
            prove_shutdown_timeout_is_bounded_and_reaps_the_initialized_child();
            prove_json_rpc_rejection_is_structurally_redacted();
            prove_unknown_server_and_spawn_failure_are_nonzero_and_redacted();
        }
    }
}

fn run_fixture(mode: &str) {
    match mode {
        HEALTHY_MODE => run_healthy_fixture(),
        SILENT_MODE => run_silent_fixture(),
        MALFORMED_MODE => run_malformed_fixture(),
        REJECTING_MODE => run_rejecting_fixture(),
        SHUTDOWN_TIMEOUT_MODE => run_shutdown_timeout_fixture(),
        unexpected => panic!("unexpected native health fixture mode: {unexpected}"),
    }
}

fn run_healthy_fixture() {
    assert!(
        std::env::var_os("HOME").is_none(),
        "the configured server must not inherit HOME"
    );
    assert_eq!(
        std::env::var("PRIVATE_ENVIRONMENT").as_deref(),
        Ok("synthetic-health-private-value"),
        "the canonical environment should reach the fixture"
    );

    let mut input = BufReader::new(std::io::stdin().lock());
    let initialize = read_protocol_line(&mut input, "initialize");
    fs::write(fixture_path("READY_PATH"), b"ready")
        .expect("the fixture should publish observable readiness");
    wait_for_release(&fixture_path("RELEASE_PATH"));
    fs::write(fixture_path("REQUEST_PATH"), initialize)
        .expect("the fixture should retain the initialize request");

    println!(
        "{{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{{\"tools\":{{\"listChanged\":true}}}},\"serverInfo\":{{\"name\":\"synthetic-health-server\",\"version\":\"1.0\"}}}}}}"
    );
    std::io::stdout()
        .flush()
        .expect("the fixture initialize response should flush");

    let initialized = read_protocol_line(&mut input, "initialized notification");
    fs::write(fixture_path("NOTIFICATION_PATH"), initialized)
        .expect("the fixture should retain the initialized notification");
    fs::write(fixture_path("EXIT_MARKER"), b"exited")
        .expect("the fixture should publish clean exit");
}

fn run_silent_fixture() {
    publish_process_id();
    let mut input = BufReader::new(std::io::stdin().lock());
    let _initialize = read_protocol_line(&mut input, "initialize");
    loop {
        thread::sleep(Duration::from_secs(60));
    }
}

fn run_malformed_fixture() {
    publish_process_id();
    let mut input = BufReader::new(std::io::stdin().lock());
    let _initialize = read_protocol_line(&mut input, "initialize");
    println!(
        "not-json-{}",
        std::env::var("PRIVATE_STDOUT").expect("the private fixture value should be configured")
    );
    std::io::stdout()
        .flush()
        .expect("the malformed fixture response should flush");
    loop {
        thread::sleep(Duration::from_secs(60));
    }
}

fn run_rejecting_fixture() {
    let mut input = BufReader::new(std::io::stdin().lock());
    let _initialize = read_protocol_line(&mut input, "initialize");
    let private_response =
        std::env::var("PRIVATE_RESPONSE").expect("the private response should be configured");
    eprintln!(
        "{}",
        std::env::var("PRIVATE_STDERR").expect("the private stderr should be configured")
    );
    println!(
        "{}",
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "error": {
                "code": -32603,
                "message": &private_response,
                "data": {"token": &private_response},
            }
        })
    );
    std::io::stdout()
        .flush()
        .expect("the rejecting fixture response should flush");
    loop {
        thread::sleep(Duration::from_secs(60));
    }
}

fn run_shutdown_timeout_fixture() {
    publish_process_id();
    let mut input = BufReader::new(std::io::stdin().lock());
    let _initialize = read_protocol_line(&mut input, "initialize");
    println!(
        "{{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{{}},\"serverInfo\":{{\"name\":\"fixture\",\"version\":\"1.0\"}}}}}}"
    );
    std::io::stdout()
        .flush()
        .expect("the fixture initialize response should flush");
    let _initialized = read_protocol_line(&mut input, "initialized notification");
    loop {
        thread::sleep(Duration::from_secs(60));
    }
}

fn read_protocol_line(input: &mut impl BufRead, phase: &str) -> String {
    let mut line = String::new();
    input
        .read_line(&mut line)
        .unwrap_or_else(|error| panic!("the fixture should read {phase}: {error}"));
    assert!(line.ends_with('\n'), "{phase} should be newline-delimited");
    line
}

fn fixture_path(name: &'static str) -> PathBuf {
    PathBuf::from(
        std::env::var_os(name).unwrap_or_else(|| panic!("the native fixture should define {name}")),
    )
}

fn publish_process_id() {
    fs::write(fixture_path("PID_PATH"), std::process::id().to_string())
        .expect("the native fixture should publish its process identifier");
}

fn health_command(home: &SyntheticHome, name: &str) -> AssertCommand {
    let mut command = AssertCommand::from_std(home.command());
    command.arg("test").arg(name).timeout(COMMAND_TIMEOUT);
    command
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
    mode: &str,
    environment: BTreeMap<String, String>,
) -> String {
    let executable = std::env::current_exe().expect("the native fixture executable should resolve");
    let command = executable.to_string_lossy().into_owned();
    let mut servers = serde_json::Map::new();
    servers.insert(
        name.to_owned(),
        json!({
            "command": command.clone(),
            "args": [FIXTURE_ARGUMENT, mode],
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

fn assert_output_omits(output: &str, private_values: &[&str]) {
    for private in private_values {
        assert!(
            !output.contains(private),
            "health output must omit process and protocol values"
        );
    }
}

fn prove_explicit_fixture_readiness_and_complete_handshake() {
    let home = SyntheticHome::new();
    let ready_path = home.root().join("fixture.ready");
    let release_path = home.root().join("fixture.release");
    let request_path = home.root().join("initialize-request.json");
    let notification_path = home.root().join("initialized-notification.json");
    let exit_marker = home.root().join("server-exited");
    let private_environment = "synthetic-health-private-value";
    let process_command = write_server(
        &home,
        "healthy",
        HEALTHY_MODE,
        BTreeMap::from([
            (
                "READY_PATH".to_owned(),
                ready_path.to_string_lossy().into_owned(),
            ),
            (
                "RELEASE_PATH".to_owned(),
                release_path.to_string_lossy().into_owned(),
            ),
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

    let mut command = home.command();
    command
        .args(["test", "healthy"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command
        .spawn()
        .expect("the health CLI should start for the readiness handshake");
    let deadline = Instant::now() + COMMAND_TIMEOUT;
    wait_for_file_while_running(&ready_path, &mut child, deadline);
    fs::write(&release_path, b"release")
        .expect("the test should explicitly acknowledge fixture readiness");
    let output = wait_for_output(child, deadline);

    assert!(output.status.success(), "the health command should succeed");
    assert!(output.stderr.is_empty(), "success should not write stderr");
    let output_text = stdout(&output);
    assert_eq!(
        output_text,
        "Server \"healthy\" initialized successfully via STDIO (protocol version 2025-11-25).\n"
    );
    assert_output_omits(&output_text, &[&process_command, private_environment]);
    assert_eq!(
        fs::read_to_string(exit_marker).expect("the healthy child should exit"),
        "exited"
    );

    let request: Value = serde_json::from_slice(
        &fs::read(request_path).expect("the fixture should capture initialize"),
    )
    .expect("initialize request should be JSON");
    assert_eq!(request["jsonrpc"], "2.0");
    assert_eq!(request["id"], 1);
    assert_eq!(request["method"], "initialize");
    assert_eq!(request["params"]["protocolVersion"], "2025-11-25");
    assert_eq!(request["params"]["capabilities"], json!({}));
    assert_eq!(request["params"]["clientInfo"]["name"], "mcp-sync");

    let notification: Value = serde_json::from_slice(
        &fs::read(notification_path).expect("the fixture should capture initialized"),
    )
    .expect("initialized notification should be JSON");
    assert_eq!(notification["jsonrpc"], "2.0");
    assert_eq!(notification["method"], "notifications/initialized");
    assert!(notification.get("id").is_none());
}

fn prove_product_timeout_is_bounded_and_reaps_the_silent_child() {
    let home = SyntheticHome::new();
    let pid_path = home.root().join("silent.pid");
    let private_argument = "synthetic-timeout-private-argument";
    let private_environment = "synthetic-timeout-private-environment";
    let process_command = write_server(
        &home,
        "silent",
        SILENT_MODE,
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
    assert_output_omits(
        &error,
        &[&process_command, private_argument, private_environment],
    );
    let process_id = read_process_id(&pid_path);
    assert!(
        !process_exists(process_id),
        "the silent child must be reaped"
    );
}

fn prove_malformed_output_is_redacted_and_reaps_the_child() {
    let home = SyntheticHome::new();
    let pid_path = home.root().join("malformed.pid");
    let private_stdout = "synthetic-malformed-private-output";
    let process_command = write_server(
        &home,
        "malformed",
        MALFORMED_MODE,
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
    assert_output_omits(&error, &[&process_command, private_stdout]);
    let process_id = read_process_id(&pid_path);
    assert!(
        !process_exists(process_id),
        "the malformed child must be reaped"
    );
}

fn prove_shutdown_timeout_is_bounded_and_reaps_the_initialized_child() {
    let home = SyntheticHome::new();
    let pid_path = home.root().join("shutdown-timeout.pid");
    let process_command = write_server(
        &home,
        "shutdown-resistant",
        SHUTDOWN_TIMEOUT_MODE,
        BTreeMap::from([(
            "PID_PATH".to_owned(),
            pid_path.to_string_lossy().into_owned(),
        )]),
    );

    let output = run_failure(health_command(&home, "shutdown-resistant"));
    let error = stderr(&output);
    assert_eq!(
        error,
        "error: health test for server \"shutdown-resistant\" failed: the initialized process did not exit within 500 milliseconds after stdin closed; it was terminated\n"
    );
    assert_output_omits(&error, &[&process_command]);
    let process_id = read_process_id(&pid_path);
    assert!(
        !process_exists(process_id),
        "the shutdown-resistant child must be reaped"
    );
}

fn prove_json_rpc_rejection_is_structurally_redacted() {
    let home = SyntheticHome::new();
    let private_response = "synthetic-rejection-private-response";
    let private_stderr = "synthetic-rejection-private-stderr";
    let process_command = write_server(
        &home,
        "rejecting",
        REJECTING_MODE,
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
    assert_output_omits(
        &error,
        &[&process_command, private_response, private_stderr],
    );
}

fn prove_unknown_server_and_spawn_failure_are_nonzero_and_redacted() {
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

fn wait_for_file_while_running(path: &Path, child: &mut Child, deadline: Instant) {
    loop {
        if path.is_file() {
            return;
        }
        if let Some(status) = child
            .try_wait()
            .expect("the health CLI should remain inspectable")
        {
            panic!("the health CLI exited before fixture readiness with {status}");
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("the native fixture did not publish observable readiness");
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn wait_for_output(mut child: Child, deadline: Instant) -> Output {
    let status = loop {
        if let Some(status) = child
            .try_wait()
            .expect("the health CLI should remain inspectable")
        {
            break status;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("the health CLI exceeded its outer test watchdog");
        }
        thread::sleep(Duration::from_millis(10));
    };
    let mut stdout = Vec::new();
    child
        .stdout
        .take()
        .expect("the health CLI stdout should be piped")
        .read_to_end(&mut stdout)
        .expect("the health CLI stdout should be readable");
    let mut stderr = Vec::new();
    child
        .stderr
        .take()
        .expect("the health CLI stderr should be piped")
        .read_to_end(&mut stderr)
        .expect("the health CLI stderr should be readable");
    Output {
        status,
        stdout,
        stderr,
    }
}

fn wait_for_release(path: &Path) {
    while !path.is_file() {
        thread::sleep(Duration::from_millis(10));
    }
}

fn read_process_id(path: &Path) -> u32 {
    fs::read_to_string(path)
        .expect("the native fixture should publish its process identifier")
        .parse()
        .expect("the native fixture process identifier should be numeric")
}
