mod support;

#[allow(dead_code)]
#[path = "../src/process_containment.rs"]
mod containment_under_test;

use assert_cmd::Command as AssertCommand;
use serde_json::json;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpListener};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use support::{SyntheticHome, process_exists};

const ROOT_MODE: &str = "--mcp-sync-containment-root";
const DESCENDANT_MODE: &str = "--mcp-sync-containment-descendant";
const LEAF_MODE: &str = "--mcp-sync-containment-leaf";
const RESPONSE_TIMEOUT_MODE: &str = "response-timeout";
const INVALID_RESPONSE_MODE: &str = "invalid-response";
const CLEAN_EXIT_MODE: &str = "clean-exit";
const SHUTDOWN_TIMEOUT_MODE: &str = "shutdown-timeout";
const MARKER_ENVIRONMENT: &str = "MCP_SYNC_CONTAINMENT_MARKER";
const PRIVATE_ENVIRONMENT: &str = "MCP_SYNC_CONTAINMENT_PRIVATE";
const PRIVATE_VALUE: &str = "synthetic-private-containment-value";
const FIXTURE_READY_TIMEOUT: Duration = Duration::from_secs(3);
const COMMAND_TIMEOUT: Duration = Duration::from_secs(8);

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some(ROOT_MODE) => run_root_fixture(
            &std::env::args()
                .nth(2)
                .expect("the root fixture mode should be provided"),
        ),
        Some(DESCENDANT_MODE) => run_session_escape_intermediary(),
        Some(LEAF_MODE) => run_detached_leaf(),
        Some(argument) => panic!("unexpected fixture argument: {argument}"),
        None => prove_built_binary_contains_detached_descendants_on_every_outcome(),
    }
}

fn run_root_fixture(mode: &str) {
    assert_eq!(
        std::env::var(PRIVATE_ENVIRONMENT).as_deref(),
        Ok(PRIVATE_VALUE),
        "the configured environment should reach only the fixture tree"
    );
    let marker = marker_path();
    let executable = std::env::current_exe().expect("fixture executable should resolve");
    let mut descendant = Command::new(executable)
        .arg(DESCENDANT_MODE)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::null())
        .spawn()
        .expect("the detached fixture descendant should start");

    wait_for_marker_lines(&marker, 3, FIXTURE_READY_TIMEOUT);
    descendant
        .wait()
        .expect("the intermediary should exit after publishing leaf readiness");
    let mut marker_file = fs::OpenOptions::new()
        .append(true)
        .open(&marker)
        .expect("the root fixture should reopen the synthetic marker");
    writeln!(marker_file, "{}", std::process::id())
        .expect("the root fixture should publish its process identifier");

    let mut input = BufReader::new(std::io::stdin().lock());
    let mut initialize = String::new();
    input
        .read_line(&mut initialize)
        .expect("the fixture should read initialize");
    assert!(initialize.ends_with('\n'));

    match mode {
        RESPONSE_TIMEOUT_MODE => resist_shutdown(&mut descendant),
        INVALID_RESPONSE_MODE => {
            println!("not-json");
            std::io::stdout()
                .flush()
                .expect("the invalid fixture response should flush");
            resist_shutdown(&mut descendant);
        }
        CLEAN_EXIT_MODE | SHUTDOWN_TIMEOUT_MODE => {}
        unexpected => panic!("unexpected root fixture mode: {unexpected}"),
    }

    println!(
        "{{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{{}},\"serverInfo\":{{\"name\":\"containment-fixture\",\"version\":\"1.0\"}}}}}}"
    );
    std::io::stdout()
        .flush()
        .expect("the fixture response should flush");

    let mut initialized = String::new();
    input
        .read_line(&mut initialized)
        .expect("the fixture should read initialized");
    assert!(initialized.ends_with('\n'));

    if mode == SHUTDOWN_TIMEOUT_MODE {
        resist_shutdown(&mut descendant);
    }
}

fn resist_shutdown(descendant: &mut std::process::Child) -> ! {
    // The root deliberately resists stdin closure after its already-reaped
    // intermediary leaves a session-escaping stdout holder behind. The product
    // must terminate the root and that reparented leaf together.
    loop {
        let _ = descendant.try_wait();
        thread::sleep(Duration::from_secs(60));
    }
}

#[allow(clippy::zombie_processes)]
fn run_session_escape_intermediary() {
    #[cfg(unix)]
    {
        rustix::process::setsid().expect("the intermediary should escape the root process group");
    }

    let executable = std::env::current_exe().expect("fixture executable should resolve");
    // Intentionally relinquish the leaf handle so this fixture proves the
    // containment boundary after the intermediate parent exits and reparents it.
    let _leaf = Command::new(executable)
        .arg(LEAF_MODE)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::null())
        .spawn()
        .expect("the reparenting fixture leaf should start");
    let marker = marker_path();
    wait_for_file(&marker, FIXTURE_READY_TIMEOUT);
    let mut marker_file = fs::OpenOptions::new()
        .append(true)
        .open(&marker)
        .expect("the intermediary should reopen the synthetic marker");
    writeln!(marker_file, "{}", std::process::id())
        .expect("the intermediary should publish its process identifier");
    // Exit immediately after the leaf is ready. The leaf is then reparented
    // outside the original ancestry while retaining stdout and its listener.
}

fn run_detached_leaf() {
    #[cfg(unix)]
    {
        rustix::process::setsid().expect("the leaf should escape the intermediary process group");
        unsafe {
            // SAFETY: This single-threaded synthetic fixture installs the
            // standard ignore disposition before it performs any other work.
            libc::signal(libc::SIGHUP, libc::SIG_IGN);
            libc::signal(libc::SIGTERM, libc::SIG_IGN);
        }
    }

    let listener = TcpListener::bind(("127.0.0.1", 0))
        .expect("the detached descendant should reserve a local fixture port");
    let address = listener
        .local_addr()
        .expect("the fixture listener should have an address");
    fs::write(
        marker_path(),
        format!("{}\n{}\n", std::process::id(), address.port()),
    )
    .expect("the detached descendant should publish its synthetic marker");

    loop {
        let _ = listener.local_addr();
        thread::sleep(Duration::from_secs(60));
    }
}

fn prove_built_binary_contains_detached_descendants_on_every_outcome() {
    for scenario in [
        Scenario {
            mode: RESPONSE_TIMEOUT_MODE,
            marker_name: "response-timeout.marker",
            success: false,
            expected_stdout: "",
            expected_stderr: "error: health test for server \"detached\" failed: the process did not return an initialize response within 5 seconds; it was terminated\n",
        },
        Scenario {
            mode: INVALID_RESPONSE_MODE,
            marker_name: "invalid-response.marker",
            success: false,
            expected_stdout: "",
            expected_stderr: "error: health test for server \"detached\" failed: the process returned an invalid initialize response: stdout was not one duplicate-free JSON-RPC message\n",
        },
        Scenario {
            mode: CLEAN_EXIT_MODE,
            marker_name: "clean-exit.marker",
            success: true,
            expected_stdout: "Server \"detached\" initialized successfully via STDIO (protocol version 2025-11-25).\n",
            expected_stderr: "",
        },
        Scenario {
            mode: SHUTDOWN_TIMEOUT_MODE,
            marker_name: "shutdown-timeout.marker",
            success: false,
            expected_stdout: "",
            expected_stderr: "error: health test for server \"detached\" failed: the initialized process did not exit within 500 milliseconds after stdin closed; it was terminated\n",
        },
    ] {
        prove_scenario(scenario);
    }
    prove_containment_setup_failure_cleanup();
    prove_containment_cleanup_failure_drop_retry();
    prove_containment_drop_backstop();
    #[cfg(target_os = "macos")]
    prove_macos_pipe_discovery_without_ancestry_history();
}

#[derive(Clone, Copy)]
struct Scenario {
    mode: &'static str,
    marker_name: &'static str,
    success: bool,
    expected_stdout: &'static str,
    expected_stderr: &'static str,
}

fn prove_scenario(scenario: Scenario) {
    let home = SyntheticHome::new();
    let marker = home.root().join(scenario.marker_name);
    let executable = std::env::current_exe().expect("fixture executable should resolve");
    let environment = serde_json::Map::from_iter([
        (
            MARKER_ENVIRONMENT.to_owned(),
            json!(marker.to_string_lossy()),
        ),
        (PRIVATE_ENVIRONMENT.to_owned(), json!(PRIVATE_VALUE)),
    ]);
    #[cfg(windows)]
    let environment = {
        let mut environment = environment;
        environment.insert(
            "SystemRoot".to_owned(),
            json!(std::env::var("SystemRoot").expect("Windows should define SystemRoot")),
        );
        environment
    };
    let document = json!({
        "schemaVersion": 1,
        "servers": {
            "detached": {
                "command": executable,
                "args": [ROOT_MODE, scenario.mode],
                "env": environment,
            }
        }
    });
    let mut bytes = serde_json::to_vec_pretty(&document)
        .expect("the synthetic canonical configuration should serialize");
    bytes.push(b'\n');
    home.write_file(&home.canonical_configuration(), bytes);

    let mut command = home.command();
    command.arg("test").arg("detached");
    let output = AssertCommand::from_std(command)
        .timeout(COMMAND_TIMEOUT)
        .output()
        .expect("the bounded health command should return");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert_eq!(
        output.status.success(),
        scenario.success,
        "the scenario should return its expected status; stdout={stdout:?}; stderr={stderr:?}"
    );
    assert_eq!(stdout, scenario.expected_stdout);
    assert_eq!(stderr, scenario.expected_stderr);

    let marker_contents = wait_for_marker_lines(&marker, 4, FIXTURE_READY_TIMEOUT);
    let mut marker_lines = marker_contents.lines();
    let descendant_process_text = marker_lines
        .next()
        .expect("the marker should contain a process identifier");
    let process_id = descendant_process_text
        .parse::<u32>()
        .expect("the fixture process identifier should be numeric");
    let port_text = marker_lines
        .next()
        .expect("the marker should contain a port");
    let port = port_text
        .parse::<u16>()
        .expect("the fixture port should be numeric");
    let mut cleanup = FixtureCleanup::new(process_id);
    let intermediary_process_text = marker_lines
        .next()
        .expect("the marker should contain the intermediary process identifier");
    let intermediary_process_id = intermediary_process_text
        .parse::<u32>()
        .expect("the intermediary fixture process identifier should be numeric");
    cleanup.add(intermediary_process_id);
    let root_process_text = marker_lines
        .next()
        .expect("the marker should contain the root process identifier");
    let root_process_id = root_process_text
        .parse::<u32>()
        .expect("the root fixture process identifier should be numeric");
    cleanup.add(root_process_id);
    assert!(
        marker_lines.next().is_none(),
        "the marker should contain only the expected fixture identities"
    );

    for private in [
        executable.to_string_lossy().as_ref(),
        ROOT_MODE,
        scenario.mode,
        PRIVATE_VALUE,
        marker.to_string_lossy().as_ref(),
        marker_contents.trim(),
        descendant_process_text,
        intermediary_process_text,
        root_process_text,
        port_text,
    ] {
        assert!(
            !stdout.contains(private) && !stderr.contains(private),
            "containment diagnostics must remain structurally redacted"
        );
    }

    wait_for_process_exit(root_process_id, Duration::from_secs(2));
    wait_for_process_exit(intermediary_process_id, Duration::from_secs(2));
    wait_for_process_exit(process_id, Duration::from_secs(2));
    wait_for_port_release(
        SocketAddr::from(([127, 0, 0, 1], port)),
        Duration::from_secs(2),
    );
    cleanup.disarm();
}

fn prove_containment_setup_failure_cleanup() {
    let mut fixture = DirectContainmentFixture::new("setup-failure.marker");

    // Health setup failures for stdin, stdout, or reader creation invoke this
    // exact explicit containment path before an OsStdioSession can exist.
    fixture
        .child
        .as_mut()
        .expect("the setup-failure fixture should retain its child")
        .terminate(Duration::from_millis(500))
        .expect("setup-failure cleanup should terminate and reap the tree");

    fixture.assert_cleaned();
}

fn prove_containment_drop_backstop() {
    DirectContainmentFixture::new("drop-backstop.marker").assert_cleaned();
}

fn prove_containment_cleanup_failure_drop_retry() {
    let mut fixture = DirectContainmentFixture::new("cleanup-failure-drop-retry.marker");
    let child = fixture
        .child
        .as_mut()
        .expect("the cleanup-failure fixture should retain its child");
    child.fail_next_cleanup_for_test();

    let error = child
        .terminate(Duration::from_millis(500))
        .expect_err("the injected transient cleanup failure should be returned");
    assert_eq!(
        error.to_string(),
        "injected transient process-containment cleanup failure"
    );

    // `assert_cleaned` drops the still-owned containment value. Its independent
    // backstop must retry and remove the complete escaped fixture tree.
    fixture.assert_cleaned();
}

#[cfg(target_os = "macos")]
fn prove_macos_pipe_discovery_without_ancestry_history() {
    let mut fixture = DirectContainmentFixture::new("pipe-discovery.marker");
    let child = fixture
        .child
        .as_mut()
        .expect("the pipe-discovery fixture should retain its child");

    // Fixture readiness now includes the intermediary's observed exit, so the
    // detached leaf is no longer discoverable from the root's ancestry. Erase
    // monitor history as a forced interleaving: cleanup can succeed only by
    // identifying the exact inherited stdout writer.
    child.forget_descendants_for_pipe_discovery_test();
    child
        .terminate(Duration::from_millis(500))
        .expect("pipe identity should recover a missed reparented stdout holder");

    fixture.assert_cleaned();
}

struct DirectContainmentFixture {
    _home: SyntheticHome,
    child: Option<containment_under_test::ContainedChild>,
    descendant_process_id: u32,
    intermediary_process_id: u32,
    root_process_id: u32,
    port: u16,
    cleanup: FixtureCleanup,
}

impl DirectContainmentFixture {
    fn new(marker_name: &str) -> Self {
        let home = SyntheticHome::new();
        let marker = home.root().join(marker_name);
        let executable = std::env::current_exe().expect("fixture executable should resolve");
        let mut command = Command::new(&executable);
        command
            .args([ROOT_MODE, RESPONSE_TIMEOUT_MODE])
            .env_clear()
            .env(MARKER_ENVIRONMENT, &marker)
            .env(PRIVATE_ENVIRONMENT, PRIVATE_VALUE)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        #[cfg(windows)]
        command.env(
            "SystemRoot",
            std::env::var_os("SystemRoot").expect("Windows should define SystemRoot"),
        );

        let child =
            containment_under_test::ContainedChild::spawn(&mut command, Duration::from_millis(500))
                .expect("the direct fixture should start inside containment");
        let marker_contents = wait_for_marker_lines(&marker, 4, FIXTURE_READY_TIMEOUT);
        let mut marker_lines = marker_contents.lines();
        let descendant_process_id = marker_lines
            .next()
            .expect("the marker should contain a descendant process identifier")
            .parse::<u32>()
            .expect("the descendant process identifier should be numeric");
        let port = marker_lines
            .next()
            .expect("the marker should contain a port")
            .parse::<u16>()
            .expect("the fixture port should be numeric");
        let intermediary_process_id = marker_lines
            .next()
            .expect("the marker should contain an intermediary process identifier")
            .parse::<u32>()
            .expect("the intermediary process identifier should be numeric");
        let root_process_id = marker_lines
            .next()
            .expect("the marker should contain a root process identifier")
            .parse::<u32>()
            .expect("the root process identifier should be numeric");
        assert!(
            marker_lines.next().is_none(),
            "the marker should contain only the expected fixture identities"
        );
        let mut cleanup = FixtureCleanup::new(descendant_process_id);
        cleanup.add(intermediary_process_id);
        cleanup.add(root_process_id);

        Self {
            _home: home,
            child: Some(child),
            descendant_process_id,
            intermediary_process_id,
            root_process_id,
            port,
            cleanup,
        }
    }

    fn assert_cleaned(mut self) {
        drop(self.child.take());

        wait_for_process_exit(self.root_process_id, Duration::from_secs(2));
        wait_for_process_exit(self.intermediary_process_id, Duration::from_secs(2));
        wait_for_process_exit(self.descendant_process_id, Duration::from_secs(2));
        wait_for_port_release(
            SocketAddr::from(([127, 0, 0, 1], self.port)),
            Duration::from_secs(2),
        );
        self.cleanup.disarm();
    }
}

fn wait_for_process_exit(process_id: u32, timeout: Duration) {
    let deadline = Instant::now() + timeout;
    while process_exists(process_id) {
        assert!(
            Instant::now() < deadline,
            "every fixture process must be reaped"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

fn marker_path() -> PathBuf {
    PathBuf::from(
        std::env::var_os(MARKER_ENVIRONMENT).expect("the fixture marker path should be configured"),
    )
}

fn wait_for_file(path: &Path, timeout: Duration) {
    let deadline = Instant::now() + timeout;
    while !path.is_file() {
        assert!(
            Instant::now() < deadline,
            "the detached descendant should become ready"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

fn wait_for_marker_lines(path: &Path, expected_lines: usize, timeout: Duration) -> String {
    let deadline = Instant::now() + timeout;
    loop {
        if let Ok(contents) = fs::read_to_string(path)
            && contents.lines().count() == expected_lines
        {
            return contents;
        }
        assert!(
            Instant::now() < deadline,
            "the complete process marker should become ready"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

fn wait_for_port_release(address: SocketAddr, timeout: Duration) {
    let deadline = Instant::now() + timeout;
    loop {
        if TcpListener::bind(address).is_ok() {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "the detached descendant must release its listener"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

struct FixtureCleanup {
    process_ids: Vec<u32>,
}

impl FixtureCleanup {
    fn new(process_id: u32) -> Self {
        Self {
            process_ids: vec![process_id],
        }
    }

    fn add(&mut self, process_id: u32) {
        self.process_ids.push(process_id);
    }

    fn disarm(&mut self) {
        self.process_ids.clear();
    }
}

impl Drop for FixtureCleanup {
    fn drop(&mut self) {
        for process_id in self.process_ids.drain(..) {
            #[cfg(unix)]
            {
                let _ = Command::new("/bin/kill")
                    .args(["-KILL", &process_id.to_string()])
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();
            }
            #[cfg(windows)]
            {
                let _ = Command::new("taskkill.exe")
                    .args(["/PID", &process_id.to_string(), "/T", "/F"])
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();
            }
        }
    }
}
