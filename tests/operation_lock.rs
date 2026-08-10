mod support;

use assert_cmd::Command as AssertCommand;
use serde_json::json;
use std::env;
use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::process::{Child, Command as ProcessCommand, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use support::{SyntheticHome, assert_file_matches};

const COMMAND_TIMEOUT: Duration = Duration::from_secs(5);
const HOLDER_START_TIMEOUT: Duration = Duration::from_secs(15);
const HOLDER_EXIT_TIMEOUT: Duration = Duration::from_secs(5);
const HOLDER_SELF_TIMEOUT: Duration = Duration::from_secs(30);
const HOLDER_LOCK_PATH: &str = "MCP_SYNC_LOCK_HOLDER_PATH";
const HOLDER_READY_PATH: &str = "MCP_SYNC_LOCK_HOLDER_READY";
const HOLDER_RELEASE_PATH: &str = "MCP_SYNC_LOCK_HOLDER_RELEASE";
const CONTENDED_STDERR: &str =
    "error: another mutating mcp-sync operation is already in progress; retry after it finishes\n";

fn command(home: &SyntheticHome, arguments: &[&str]) -> AssertCommand {
    let mut command = AssertCommand::from_std(home.command());
    command.args(arguments).timeout(COMMAND_TIMEOUT);
    command
}

fn run_success(home: &SyntheticHome, arguments: &[&str]) -> Output {
    let output = command(home, arguments).assert().get_output().clone();
    assert!(output.status.success(), "the CLI command should succeed");
    assert!(output.stderr.is_empty(), "success should not write stderr");
    output
}

fn run_failure(home: &SyntheticHome, arguments: &[&str]) -> Output {
    let output = command(home, arguments).assert().get_output().clone();
    assert!(!output.status.success(), "the CLI command should fail");
    assert!(output.stdout.is_empty(), "failure should not write stdout");
    output
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr should be UTF-8")
}

fn backup_path(path: &Path) -> PathBuf {
    let mut path = path.as_os_str().to_owned();
    path.push(".bak");
    PathBuf::from(path)
}

fn json_bytes(value: serde_json::Value) -> Vec<u8> {
    let mut bytes = serde_json::to_vec_pretty(&value).expect("synthetic JSON should serialize");
    bytes.push(b'\n');
    bytes
}

fn canonical(command: &str) -> Vec<u8> {
    json_bytes(json!({
        "schemaVersion": 1,
        "servers": {
            "shared": {
                "command": command,
                "args": ["--stdio"],
                "env": {"TOKEN": format!("{command}-private-value")}
            }
        }
    }))
}

fn target_bytes(command: &str) -> [Vec<u8>; 5] {
    let common = json!({
        "shared": {
            "command": command,
            "args": ["--stdio"],
            "env": {"TOKEN": format!("{command}-private-value")}
        }
    });
    [
        json_bytes(json!({"mcpServers": common.clone()})),
        json_bytes(json!({"mcpServers": common.clone()})),
        json_bytes(json!({"mcpServers": common.clone()})),
        json_bytes(json!({"servers": {
            "shared": {
                "type": "stdio",
                "command": command,
                "args": ["--stdio"],
                "env": {"TOKEN": format!("{command}-private-value")}
            }
        }})),
        format!(
            "[mcp_servers.shared]\ncommand = {command:?}\nargs = [\"--stdio\"]\nenv = {{ TOKEN = {value:?} }}\n",
            value = format!("{command}-private-value")
        )
        .into_bytes(),
    ]
}

fn target_paths(home: &SyntheticHome) -> [PathBuf; 5] {
    [
        home.claude_desktop_configuration(),
        home.cursor_configuration(),
        home.windsurf_configuration(),
        home.vscode_configuration(),
        home.codex_configuration(),
    ]
}

fn seed_sync_state(home: &SyntheticHome, canonical_command: &str, target_command: &str) {
    home.write_file(
        &home.canonical_configuration(),
        canonical(canonical_command),
    );
    for (path, bytes) in target_paths(home)
        .into_iter()
        .zip(target_bytes(target_command))
    {
        home.write_file(&path, bytes);
    }
}

fn snapshot(paths: impl IntoIterator<Item = PathBuf>) -> Vec<(PathBuf, Vec<u8>)> {
    paths
        .into_iter()
        .map(|path| {
            let bytes = fs::read(&path)
                .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
            (path, bytes)
        })
        .collect()
}

fn assert_snapshot_unchanged(snapshot: &[(PathBuf, Vec<u8>)], context: &str) {
    for (path, expected) in snapshot {
        assert_file_matches(path, expected, context);
    }
}

struct LockHolder {
    child: Option<Child>,
    release_path: PathBuf,
}

impl LockHolder {
    fn spawn(home: &SyntheticHome) -> Self {
        let lock_path = home.operation_lock();
        home.write_file(&lock_path, b"");
        let ready_path = home.root().join("operation-lock-holder.ready");
        let release_path = home.root().join("operation-lock-holder.release");

        let mut command = ProcessCommand::new(
            env::current_exe().expect("the operation-lock test binary path should resolve"),
        );
        command
            .env_clear()
            .arg("--exact")
            .arg("operation_lock_holder_process")
            .arg("--nocapture")
            .env(HOLDER_LOCK_PATH, &lock_path)
            .env(HOLDER_READY_PATH, &ready_path)
            .env(HOLDER_RELEASE_PATH, &release_path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        if let Some(profile) = env::var_os("LLVM_PROFILE_FILE") {
            command.env("LLVM_PROFILE_FILE", profile);
        }

        let mut child = command
            .spawn()
            .expect("the cross-process lock holder should start");
        let deadline = Instant::now() + HOLDER_START_TIMEOUT;
        while !ready_path.is_file() {
            if let Some(status) = child
                .try_wait()
                .expect("the lock-holder process should be observable")
            {
                panic!("the lock-holder process exited before acquiring the lock: {status}");
            }
            if Instant::now() >= deadline {
                child
                    .kill()
                    .expect("an unresponsive lock holder should be terminated");
                child
                    .wait()
                    .expect("the terminated lock holder should reap");
                panic!("the lock-holder process should acquire the lock within the startup bound");
            }
            thread::sleep(Duration::from_millis(10));
        }

        Self {
            child: Some(child),
            release_path,
        }
    }

    fn release(mut self) {
        fs::write(&self.release_path, b"release")
            .expect("the lock-holder release signal should be written");
        let mut child = self
            .child
            .take()
            .expect("the lock-holder process should still be owned");
        let deadline = Instant::now() + HOLDER_EXIT_TIMEOUT;
        loop {
            if let Some(status) = child
                .try_wait()
                .expect("the lock-holder process should be observable")
            {
                assert!(status.success(), "the lock holder should exit cleanly");
                return;
            }
            if Instant::now() >= deadline {
                child
                    .kill()
                    .expect("an unresponsive lock holder should be terminated");
                child
                    .wait()
                    .expect("the terminated lock holder should reap");
                panic!("the lock holder did not exit within the shutdown bound");
            }
            thread::sleep(Duration::from_millis(10));
        }
    }

    fn terminate(mut self) {
        let mut child = self
            .child
            .take()
            .expect("the lock-holder process should still be owned");
        child
            .kill()
            .expect("the lock holder should be force-terminable");
        child
            .wait()
            .expect("the terminated lock-holder process should reap");
    }
}

impl Drop for LockHolder {
    fn drop(&mut self) {
        let Some(mut child) = self.child.take() else {
            return;
        };
        let _ = fs::write(&self.release_path, b"release");
        let _ = child.kill();
        let _ = child.wait();
    }
}

#[test]
fn operation_lock_holder_process() {
    let Some(lock_path) = env::var_os(HOLDER_LOCK_PATH) else {
        return;
    };
    let ready_path = PathBuf::from(
        env::var_os(HOLDER_READY_PATH).expect("the holder should receive a ready path"),
    );
    let release_path = PathBuf::from(
        env::var_os(HOLDER_RELEASE_PATH).expect("the holder should receive a release path"),
    );
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(PathBuf::from(lock_path))
        .expect("the holder should open the synthetic operation lock");
    file.lock()
        .expect("the holder should acquire the operating-system lock");
    fs::write(ready_path, b"ready").expect("the holder should publish readiness");

    let deadline = Instant::now() + HOLDER_SELF_TIMEOUT;
    while !release_path.is_file() {
        assert!(
            Instant::now() < deadline,
            "the parent test should release the lock holder"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn every_mutating_command_fails_fast_while_read_only_modes_remain_available() {
    let home = SyntheticHome::new();
    seed_sync_state(&home, "new-command", "old-command");
    let canonical_backup = backup_path(&home.canonical_configuration());
    home.write_file(&canonical_backup, canonical("prior-command"));
    let mut managed_paths = vec![home.canonical_configuration(), canonical_backup];
    managed_paths.extend(target_paths(&home));
    let before = snapshot(managed_paths);

    let holder = LockHolder::spawn(&home);

    for arguments in [
        vec!["init"],
        vec!["add", "shared", "--command", "changed-private-command"],
        vec!["restore", "canonical"],
        vec!["sync"],
    ] {
        let diagnostic = stderr(&run_failure(&home, &arguments));
        assert_eq!(diagnostic, CONTENDED_STDERR);
        assert!(!diagnostic.contains(&home.operation_lock().to_string_lossy().into_owned()));
        assert!(!diagnostic.contains("changed-private-command"));
    }

    let invalid_add = stderr(&run_failure(
        &home,
        &[
            "add",
            " padded-name ",
            "--command",
            "invalid-private-command",
        ],
    ));
    assert!(invalid_add.contains("cannot add the requested server definition"));
    assert!(!invalid_add.contains("already in progress"));
    assert!(!invalid_add.contains("invalid-private-command"));

    run_success(&home, &["list"]);
    let missing_test = stderr(&run_failure(&home, &["test", "missing"]));
    assert!(missing_test.contains("server \"missing\""));
    assert!(!missing_test.contains("already in progress"));
    run_success(&home, &["sync", "--dry-run"]);
    run_success(&home, &["restore", "canonical", "--dry-run"]);
    assert_snapshot_unchanged(&before, "contention and read-only modes");

    let independent = SyntheticHome::new();
    seed_sync_state(&independent, "new-command", "old-command");
    run_success(&independent, &["sync"]);

    holder.release();
    run_success(&home, &["sync"]);
    assert!(home.operation_lock().is_file());
    assert!(
        fs::read(home.operation_lock())
            .expect("the persistent operation lock should be readable")
            .is_empty()
    );
}

#[test]
fn mutating_commands_contend_before_reading_unusable_managed_state() {
    let home = SyntheticHome::new();
    let holder = LockHolder::spawn(&home);
    fs::create_dir(home.canonical_configuration())
        .expect("directory-shaped canonical state should be created");

    for arguments in [
        vec!["init"],
        vec!["add", "shared", "--command", "private-command"],
        vec!["restore", "canonical"],
        vec!["sync"],
    ] {
        let diagnostic = stderr(&run_failure(&home, &arguments));
        assert_eq!(diagnostic, CONTENDED_STDERR);
        assert!(!diagnostic.contains("private-command"));
    }

    assert!(
        home.canonical_configuration().is_dir(),
        "contention must leave unusable managed state untouched"
    );
    holder.release();
}

#[test]
fn issue_45_partial_generation_cannot_be_observed_by_a_second_sync() {
    let home = SyntheticHome::new();
    let oracle = SyntheticHome::new();
    seed_sync_state(&home, "new-command", "old-command");
    seed_sync_state(&oracle, "new-command", "old-command");

    run_success(&oracle, &["sync"]);
    let expected_targets = snapshot(target_paths(&oracle));
    let old_targets = target_bytes("old-command");

    let holder = LockHolder::spawn(&home);
    let claude_path = home.claude_desktop_configuration();
    let claude_backup = backup_path(&claude_path);
    home.write_file(&claude_path, &expected_targets[0].1);
    home.write_file(&claude_backup, &old_targets[0]);
    let mut partial_paths = target_paths(&home).to_vec();
    partial_paths.push(claude_backup.clone());
    let partial_generation = snapshot(partial_paths);

    let diagnostic = stderr(&run_failure(&home, &["sync"]));
    assert_eq!(diagnostic, CONTENDED_STDERR);
    assert_snapshot_unchanged(
        &partial_generation,
        "a second sync must not plan or mutate a partial generation",
    );

    // Model the older transaction's rollback while it still owns the lock.
    home.write_file(&claude_path, &old_targets[0]);
    fs::remove_file(&claude_backup).expect("the simulated rollback should restore no prior backup");
    holder.release();

    run_success(&home, &["sync"]);
    for ((path, (_, expected)), old) in target_paths(&home)
        .into_iter()
        .zip(expected_targets)
        .zip(old_targets)
    {
        assert_file_matches(
            &path,
            &expected,
            "the final target generation should converge",
        );
        assert_file_matches(
            &backup_path(&path),
            &old,
            "the final recovery generation should be coherent",
        );
    }
    let settled = run_success(&home, &["sync", "--dry-run"]);
    let settled = String::from_utf8(settled.stdout).expect("dry-run output should be UTF-8");
    assert!(settled.contains("Dry run validated 5 targets; no files changed."));
    assert_eq!(settled.matches("unchanged; no write or backup").count(), 5);
}

#[test]
fn process_termination_releases_the_lock_without_deleting_it() {
    let home = SyntheticHome::new();
    seed_sync_state(&home, "new-command", "old-command");
    let holder = LockHolder::spawn(&home);

    holder.terminate();

    run_success(&home, &["sync"]);
    assert!(home.operation_lock().is_file());
    assert!(
        fs::read(home.operation_lock())
            .expect("the persistent lock should remain readable")
            .is_empty()
    );
}

#[test]
fn unsafe_lock_artifacts_fail_closed_without_exposing_contents_or_paths() {
    let home = SyntheticHome::new();
    seed_sync_state(&home, "new-command", "old-command");
    let canonical_before = fs::read(home.canonical_configuration())
        .expect("canonical state should be readable before the refusal");
    let lock_path = home.operation_lock();
    let private_lock_value = "private-lock-content-must-not-appear";
    home.write_file(&lock_path, private_lock_value);

    let non_empty = stderr(&run_failure(
        &home,
        &["add", "shared", "--command", "changed-private-command"],
    ));
    assert!(non_empty.contains("operation lock is not empty"));
    assert!(!non_empty.contains(private_lock_value));
    assert!(!non_empty.contains(&lock_path.to_string_lossy().into_owned()));
    assert_file_matches(
        &home.canonical_configuration(),
        &canonical_before,
        "a non-empty lock refusal should preserve canonical state",
    );

    fs::remove_file(&lock_path).expect("the non-empty lock fixture should be removable");
    fs::create_dir(&lock_path).expect("the directory-shaped lock fixture should be created");
    let non_regular = stderr(&run_failure(&home, &["sync"]));
    assert!(non_regular.contains("operation lock is not a regular file"));
    assert!(!non_regular.contains(&lock_path.to_string_lossy().into_owned()));
    assert_file_matches(
        &home.canonical_configuration(),
        &canonical_before,
        "a non-regular lock refusal should preserve canonical state",
    );
}

#[test]
fn invalid_add_input_precedes_even_an_unsafe_lock_artifact() {
    let home = SyntheticHome::new();
    let lock_path = home.operation_lock();
    home.write_file(&lock_path, b"private-lock-content");

    let diagnostic = stderr(&run_failure(
        &home,
        &["add", " padded-name ", "--command", "private-command"],
    ));

    assert!(diagnostic.contains("cannot add the requested server definition"));
    assert!(!diagnostic.contains("operation lock"));
    assert!(!diagnostic.contains("private-command"));
    assert!(!home.canonical_configuration().exists());
    assert_file_matches(
        &lock_path,
        b"private-lock-content",
        "invalid input should not access or replace the unsafe lock artifact",
    );
}
