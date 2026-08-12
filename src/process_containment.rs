use std::io;
use std::process::{Child, ChildStdin, ChildStdout, Command, ExitStatus};
use std::time::{Duration, Instant};

/// A configured process whose descendants remain inside one cleanup boundary.
///
/// The platform implementations deliberately live at the process adapter. The
/// CLI and health protocol layers never receive process identifiers or native
/// handles, so neither can accidentally expose them in diagnostics.
pub(crate) struct ContainedChild {
    child: Child,
    containment: PlatformContainment,
    cleanup_timeout: Duration,
    cleaned: bool,
    #[cfg(test)]
    fail_next_cleanup: bool,
}

impl ContainedChild {
    pub(crate) fn spawn(command: &mut Command, cleanup_timeout: Duration) -> io::Result<Self> {
        let (child, containment) = PlatformContainment::spawn(command, cleanup_timeout)?;
        Ok(Self {
            child,
            containment,
            cleanup_timeout,
            cleaned: false,
            #[cfg(test)]
            fail_next_cleanup: false,
        })
    }

    pub(crate) fn stdin(&mut self) -> &mut Option<ChildStdin> {
        &mut self.child.stdin
    }

    pub(crate) fn stdout(&mut self) -> &mut Option<ChildStdout> {
        &mut self.child.stdout
    }

    pub(crate) fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        self.child.try_wait()
    }

    pub(crate) fn finish_after_exit(&mut self, timeout: Duration) -> io::Result<()> {
        let deadline = cleanup_deadline(timeout);
        let mut failure = None;
        record_first(&mut failure, self.containment.terminate());
        record_first(
            &mut failure,
            self.containment
                .wait_until_empty(remaining_cleanup_time(deadline)),
        );
        if failure.is_none() {
            record_first(&mut failure, self.containment.finish());
        }
        self.cleaned = failure.is_none();
        failure.map_or(Ok(()), Err)
    }

    pub(crate) fn terminate(&mut self, timeout: Duration) -> io::Result<()> {
        #[cfg(test)]
        if std::mem::take(&mut self.fail_next_cleanup) {
            return Err(io::Error::other(
                "injected transient process-containment cleanup failure",
            ));
        }

        let deadline = cleanup_deadline(timeout);
        let mut failure = None;
        record_first(&mut failure, self.containment.terminate());
        // The containment primitive should already have targeted the complete
        // tree. This bounded direct-child fallback preserves the existing reap
        // guarantee even when native tree cleanup reports an error.
        record_first(
            &mut failure,
            terminate_and_reap_direct(&mut self.child, remaining_cleanup_time(deadline)),
        );
        record_first(
            &mut failure,
            self.containment
                .wait_until_empty(remaining_cleanup_time(deadline)),
        );
        if failure.is_none() {
            record_first(&mut failure, self.containment.finish());
        }
        self.cleaned = failure.is_none();
        failure.map_or(Ok(()), Err)
    }

    #[cfg(test)]
    #[allow(dead_code)] // Used by the harness-free integration fixture's path-included copy.
    pub(crate) fn fail_next_cleanup_for_test(&mut self) {
        self.fail_next_cleanup = true;
    }

    #[cfg(all(test, target_os = "macos"))]
    #[allow(dead_code)] // Used by the harness-free integration fixture's path-included copy.
    pub(crate) fn forget_descendants_for_pipe_discovery_test(&mut self) {
        self.containment
            .forget_descendants_for_pipe_discovery_test();
    }
}

impl Drop for ContainedChild {
    fn drop(&mut self) {
        if !self.cleaned && self.terminate(self.cleanup_timeout).is_err() {
            // A transient native inspection or signaling failure must not
            // consume the final ownership backstop. Retry once while the
            // direct child handle is still available for bounded reap.
            let _ = self.terminate(self.cleanup_timeout);
        }
    }
}

fn record_first(failure: &mut Option<io::Error>, result: io::Result<()>) {
    if let Err(source) = result
        && failure.is_none()
    {
        *failure = Some(source);
    }
}

fn combine_setup_error(source: io::Error, cleanup_failure: Option<io::Error>) -> io::Error {
    cleanup_failure.map_or(source, |cleanup| {
        io::Error::new(
            cleanup.kind(),
            "process containment setup failed and cleanup did not complete",
        )
    })
}

fn cleanup_deadline(timeout: Duration) -> Instant {
    Instant::now()
        .checked_add(timeout)
        .unwrap_or_else(Instant::now)
}

fn remaining_cleanup_time(deadline: Instant) -> Duration {
    deadline.saturating_duration_since(Instant::now())
}

fn terminate_and_reap_direct(child: &mut Child, timeout: Duration) -> io::Result<()> {
    if child.try_wait()?.is_some() {
        return Ok(());
    }
    if let Err(source) = child.kill() {
        return match child.try_wait()? {
            Some(_) => Ok(()),
            None => Err(source),
        };
    }

    let deadline = cleanup_deadline(timeout);
    loop {
        if child.try_wait()?.is_some() {
            return Ok(());
        }
        let now = Instant::now();
        if now >= deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "the direct child did not exit within the cleanup bound",
            ));
        }
        std::thread::sleep(Duration::from_millis(5).min(deadline.duration_since(now)));
    }
}

#[cfg(unix)]
type PlatformContainment = unix::UnixContainment;

#[cfg(windows)]
type PlatformContainment = windows::WindowsContainment;

#[cfg(unix)]
mod unix {
    use super::{
        cleanup_deadline, combine_setup_error, record_first, remaining_cleanup_time,
        terminate_and_reap_direct,
    };
    use rustix::process::{Pid, Signal, kill_process_group};
    use std::collections::{HashMap, HashSet};
    use std::io;
    #[cfg(target_os = "macos")]
    use std::os::fd::AsRawFd;
    use std::os::unix::process::CommandExt;
    use std::process::{Child, Command};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex, MutexGuard};
    use std::thread::{self, JoinHandle};
    use std::time::{Duration, Instant};

    const MONITOR_INTERVAL: Duration = Duration::from_millis(5);
    const FREEZE_PASSES: usize = 8;
    const REQUIRED_STABLE_FREEZE_PASSES: usize = 2;

    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    struct ProcessIdentity {
        pid: u32,
        started: ProcessStart,
    }

    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    struct ProcessStart {
        major: u64,
        minor: u64,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct ProcessInfo {
        identity: ProcessIdentity,
        parent_pid: u32,
        process_group: u32,
    }

    #[cfg(target_os = "macos")]
    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    struct PipeEndpoint {
        handle: u64,
        peer_handle: u64,
    }

    #[cfg(target_os = "macos")]
    impl PipeEndpoint {
        fn peer(self) -> Self {
            Self {
                handle: self.peer_handle,
                peer_handle: self.handle,
            }
        }
    }

    struct TrackedProcess {
        identity: ProcessIdentity,
        target: SignalTarget,
    }

    #[derive(Default)]
    struct MonitorState {
        tracked: HashMap<ProcessIdentity, TrackedProcess>,
        first_error: Option<io::ErrorKind>,
    }

    pub(super) struct UnixContainment {
        root: ProcessIdentity,
        state: Arc<Mutex<MonitorState>>,
        stop: Arc<AtomicBool>,
        monitor: Option<JoinHandle<()>>,
        #[cfg(target_os = "linux")]
        baseline_children: HashSet<ProcessIdentity>,
        #[cfg(target_os = "macos")]
        baseline_processes: HashSet<ProcessIdentity>,
        #[cfg(target_os = "macos")]
        stdout_writer: Option<PipeEndpoint>,
        #[cfg(target_os = "linux")]
        subreaper: SubreaperGuard,
        cleanup_timeout: Duration,
        finished: bool,
    }

    impl UnixContainment {
        pub(super) fn spawn(
            command: &mut Command,
            cleanup_timeout: Duration,
        ) -> io::Result<(Child, Self)> {
            command.process_group(0);

            #[cfg(target_os = "linux")]
            let baseline_children = direct_children(std::process::id())?;
            #[cfg(target_os = "macos")]
            let baseline_processes = process_snapshot()?
                .into_iter()
                .map(|process| process.identity)
                .collect();
            #[cfg(target_os = "linux")]
            let mut subreaper = SubreaperGuard::enable()?;

            let mut child = match command.spawn() {
                Ok(child) => child,
                Err(source) => {
                    #[cfg(target_os = "linux")]
                    return Err(combine_setup_error(source, subreaper.restore().err()));
                    #[cfg(not(target_os = "linux"))]
                    return Err(source);
                }
            };

            let root = match process_info(child.id()) {
                Ok(Some(info)) => info.identity,
                Ok(None) => {
                    let source = io::Error::other(
                        "process containment could not identify the configured process",
                    );
                    return Err(cleanup_unidentified_setup(
                        &mut child,
                        #[cfg(target_os = "linux")]
                        &mut subreaper,
                        cleanup_timeout,
                        source,
                    ));
                }
                Err(source) => {
                    return Err(cleanup_unidentified_setup(
                        &mut child,
                        #[cfg(target_os = "linux")]
                        &mut subreaper,
                        cleanup_timeout,
                        source,
                    ));
                }
            };
            let target = match SignalTarget::open(root) {
                Ok(Some(target)) => target,
                Ok(None) => {
                    let source = io::Error::other(
                        "process containment lost the configured process during setup",
                    );
                    return Err(cleanup_identified_group_setup(
                        &mut child,
                        #[cfg(target_os = "linux")]
                        &mut subreaper,
                        root,
                        cleanup_timeout,
                        source,
                    ));
                }
                Err(source) => {
                    return Err(cleanup_identified_group_setup(
                        &mut child,
                        #[cfg(target_os = "linux")]
                        &mut subreaper,
                        root,
                        cleanup_timeout,
                        source,
                    ));
                }
            };

            #[cfg(target_os = "macos")]
            let stdout_writer = match child.stdout.as_ref() {
                Some(stdout) => match pipe_endpoint(std::process::id(), stdout.as_raw_fd()) {
                    Ok(Some(endpoint)) => Some(endpoint.peer()),
                    Ok(None) => {
                        let source = io::Error::other(
                            "process containment could not identify the stdout pipe",
                        );
                        return Err(cleanup_identified_group_setup(
                            &mut child,
                            root,
                            cleanup_timeout,
                            source,
                        ));
                    }
                    Err(source) => {
                        return Err(cleanup_identified_group_setup(
                            &mut child,
                            root,
                            cleanup_timeout,
                            source,
                        ));
                    }
                },
                None => None,
            };

            let state = Arc::new(Mutex::new(MonitorState {
                tracked: HashMap::from([(
                    root,
                    TrackedProcess {
                        identity: root,
                        target,
                    },
                )]),
                first_error: None,
            }));
            let stop = Arc::new(AtomicBool::new(false));
            let monitor_state = Arc::clone(&state);
            let monitor_stop = Arc::clone(&stop);
            #[cfg(target_os = "linux")]
            let monitor_baseline = baseline_children.clone();

            let mut containment = Self {
                root,
                state,
                stop,
                monitor: None,
                #[cfg(target_os = "linux")]
                baseline_children,
                #[cfg(target_os = "macos")]
                baseline_processes,
                #[cfg(target_os = "macos")]
                stdout_writer,
                #[cfg(target_os = "linux")]
                subreaper,
                cleanup_timeout,
                finished: false,
            };
            let monitor = thread::Builder::new()
                .name("mcp-sync-process-monitor".to_owned())
                .spawn(move || {
                    while !monitor_stop.load(Ordering::Acquire) {
                        #[cfg(target_os = "linux")]
                        let capture = capture_descendants(root, &monitor_baseline, &monitor_state);
                        #[cfg(not(target_os = "linux"))]
                        let capture = capture_descendants(root, &monitor_state);
                        if let Err(source) = capture {
                            remember_monitor_error(&monitor_state, source.kind());
                        }
                        thread::park_timeout(MONITOR_INTERVAL);
                    }
                });

            match monitor {
                Ok(monitor) => {
                    containment.monitor = Some(monitor);
                    Ok((child, containment))
                }
                Err(source) => Err(cleanup_identified_setup(
                    &mut child,
                    &mut containment,
                    cleanup_timeout,
                    source,
                )),
            }
        }

        pub(super) fn terminate(&mut self) -> io::Result<()> {
            let mut failure = None;

            record_first(&mut failure, self.capture_cleanup_targets().map(drop));

            record_first(&mut failure, signal_group(self.root, Signal::STOP));

            let mut stable_passes = 0;
            for _ in 0..FREEZE_PASSES {
                let captured = self.capture_cleanup_targets();

                let added = match captured {
                    Ok(added) => added,
                    Err(source) => {
                        record_first(&mut failure, Err(source));
                        0
                    }
                };
                record_first(&mut failure, signal_tracked(&self.state, Signal::STOP));
                if added == 0 {
                    stable_passes += 1;
                    if stable_passes >= REQUIRED_STABLE_FREEZE_PASSES {
                        break;
                    }
                } else {
                    stable_passes = 0;
                }
            }

            self.stop_monitor();
            record_first(&mut failure, take_monitor_error(&self.state));
            record_first(&mut failure, signal_tracked(&self.state, Signal::KILL));
            record_first(&mut failure, signal_group(self.root, Signal::KILL));

            failure.map_or(Ok(()), Err)
        }

        pub(super) fn wait_until_empty(&mut self, timeout: Duration) -> io::Result<()> {
            let deadline = Instant::now()
                .checked_add(timeout)
                .unwrap_or_else(Instant::now);
            loop {
                #[cfg(target_os = "linux")]
                reap_adopted_descendants(&self.state);

                if tracked_are_gone(&self.state)? {
                    return Ok(());
                }
                let now = Instant::now();
                if now >= deadline {
                    return Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        "contained processes did not exit within the cleanup bound",
                    ));
                }
                thread::sleep(MONITOR_INTERVAL.min(deadline.duration_since(now)));
            }
        }

        pub(super) fn finish(&mut self) -> io::Result<()> {
            if self.finished {
                return Ok(());
            }
            self.stop_monitor();
            #[cfg(target_os = "linux")]
            self.subreaper.restore()?;
            self.finished = true;
            Ok(())
        }

        fn stop_monitor(&mut self) {
            self.stop.store(true, Ordering::Release);
            if let Some(monitor) = self.monitor.take() {
                monitor.thread().unpark();
                let _ = monitor.join();
            }
        }

        fn capture_cleanup_targets(&self) -> io::Result<usize> {
            #[cfg(target_os = "linux")]
            {
                capture_descendants(self.root, &self.baseline_children, &self.state)
            }
            #[cfg(target_os = "macos")]
            {
                capture_macos_cleanup_targets(
                    self.root,
                    &self.baseline_processes,
                    self.stdout_writer,
                    &self.state,
                )
            }
        }

        #[cfg(all(test, target_os = "macos"))]
        pub(super) fn forget_descendants_for_pipe_discovery_test(&mut self) {
            self.stop_monitor();
            lock_state(&self.state)
                .tracked
                .retain(|identity, _| *identity == self.root);
        }
    }

    impl Drop for UnixContainment {
        fn drop(&mut self) {
            if !self.finished {
                let deadline = cleanup_deadline(self.cleanup_timeout);
                let _ = self.terminate();
                let _ = self.wait_until_empty(remaining_cleanup_time(deadline));
                let _ = self.finish();
            }
        }
    }

    fn cleanup_unidentified_setup(
        child: &mut Child,
        #[cfg(target_os = "linux")] subreaper: &mut SubreaperGuard,
        timeout: Duration,
        source: io::Error,
    ) -> io::Error {
        let mut cleanup_failure = terminate_untracked_group(child, timeout).err();
        if cleanup_failure.is_some() {
            cleanup_failure = terminate_untracked_group(child, timeout).err();
        }
        #[cfg(target_os = "linux")]
        record_first(&mut cleanup_failure, subreaper.restore());
        combine_setup_error(source, cleanup_failure)
    }

    fn cleanup_identified_setup(
        child: &mut Child,
        containment: &mut UnixContainment,
        timeout: Duration,
        source: io::Error,
    ) -> io::Error {
        let mut cleanup_failure = cleanup_identified_processes(child, containment, timeout).err();
        if cleanup_failure.is_some() {
            cleanup_failure = cleanup_identified_processes(child, containment, timeout).err();
        }
        if cleanup_failure.is_none() {
            record_first(&mut cleanup_failure, containment.finish());
        }
        combine_setup_error(source, cleanup_failure)
    }

    fn cleanup_identified_group_setup(
        child: &mut Child,
        #[cfg(target_os = "linux")] subreaper: &mut SubreaperGuard,
        root: ProcessIdentity,
        timeout: Duration,
        source: io::Error,
    ) -> io::Error {
        let mut cleanup_failure = terminate_identified_group(child, root, timeout).err();
        if cleanup_failure.is_some() {
            cleanup_failure = terminate_identified_group(child, root, timeout).err();
        }
        #[cfg(target_os = "linux")]
        record_first(&mut cleanup_failure, subreaper.restore());
        combine_setup_error(source, cleanup_failure)
    }

    fn cleanup_identified_processes(
        child: &mut Child,
        containment: &mut UnixContainment,
        timeout: Duration,
    ) -> io::Result<()> {
        let deadline = cleanup_deadline(timeout);
        let mut failure = None;
        record_first(&mut failure, containment.terminate());
        record_first(
            &mut failure,
            terminate_and_reap_direct(child, remaining_cleanup_time(deadline)),
        );
        record_first(
            &mut failure,
            containment.wait_until_empty(remaining_cleanup_time(deadline)),
        );
        failure.map_or(Ok(()), Err)
    }

    fn terminate_untracked_group(child: &mut Child, timeout: Duration) -> io::Result<()> {
        let deadline = cleanup_deadline(timeout);
        let mut failure = None;
        // The direct child has not yet been reaped, so its numeric identifier
        // cannot be recycled under the normal child-wait contract. Signal its
        // still-rooted process group before the bounded direct reap so setup
        // failure also catches a fork left behind by an early root exit.
        record_first(
            &mut failure,
            signal_unidentified_group(child.id(), Signal::KILL),
        );
        record_first(
            &mut failure,
            terminate_and_reap_direct(child, remaining_cleanup_time(deadline)),
        );
        failure.map_or(Ok(()), Err)
    }

    fn terminate_identified_group(
        child: &mut Child,
        root: ProcessIdentity,
        timeout: Duration,
    ) -> io::Result<()> {
        let deadline = cleanup_deadline(timeout);
        let mut failure = None;
        record_first(&mut failure, signal_group(root, Signal::KILL));
        record_first(
            &mut failure,
            terminate_and_reap_direct(child, remaining_cleanup_time(deadline)),
        );
        failure.map_or(Ok(()), Err)
    }

    fn signal_group(root: ProcessIdentity, signal: Signal) -> io::Result<()> {
        let snapshot = process_snapshot()?;
        if !snapshot_contains_group_root(root, &snapshot) {
            return Ok(());
        }
        signal_snapshot_group(root.pid, snapshot, signal)
    }

    fn snapshot_contains_group_root(root: ProcessIdentity, snapshot: &[ProcessInfo]) -> bool {
        snapshot
            .iter()
            .any(|process| process.identity == root && process.process_group == root.pid)
    }

    fn signal_unidentified_group(pid: u32, signal: Signal) -> io::Result<()> {
        let snapshot = process_snapshot()?;
        if !snapshot
            .iter()
            .any(|process| process.identity.pid == pid && process.process_group == pid)
        {
            return Ok(());
        }
        signal_snapshot_group(pid, snapshot, signal)
    }

    fn signal_snapshot_group(
        pid: u32,
        snapshot: Vec<ProcessInfo>,
        signal: Signal,
    ) -> io::Result<()> {
        let members: Vec<_> = snapshot
            .into_iter()
            .filter(|process| process.process_group == pid)
            .map(|process| process.identity)
            .collect();
        if members.is_empty() {
            return Ok(());
        }
        let Some(pid) = raw_pid(pid) else {
            return Ok(());
        };
        match kill_process_group(pid, signal) {
            Ok(()) | Err(rustix::io::Errno::SRCH) => Ok(()),
            #[cfg(target_os = "macos")]
            Err(rustix::io::Errno::PERM) => {
                let mut failure = None;
                for identity in members {
                    match SignalTarget::open(identity) {
                        Ok(Some(target)) => {
                            record_first(&mut failure, target.signal(identity, signal));
                        }
                        Ok(None) => {}
                        Err(source) => record_first(&mut failure, Err(source)),
                    }
                }
                failure.map_or(Ok(()), Err)
            }
            Err(source) => Err(io::Error::new(
                io::Error::from(source).kind(),
                "process-group signaling failed",
            )),
        }
    }

    fn signal_tracked(state: &Arc<Mutex<MonitorState>>, signal: Signal) -> io::Result<()> {
        let state = lock_state(state);
        let mut failure = None;
        for process in state.tracked.values() {
            record_first(
                &mut failure,
                process.target.signal(process.identity, signal),
            );
        }
        failure.map_or(Ok(()), Err)
    }

    #[cfg(target_os = "linux")]
    fn capture_descendants(
        root: ProcessIdentity,
        baseline_children: &HashSet<ProcessIdentity>,
        state: &Arc<Mutex<MonitorState>>,
    ) -> io::Result<usize> {
        capture_descendants_from_snapshot(
            root,
            baseline_children,
            std::process::id(),
            process_snapshot()?,
            state,
        )
    }

    #[cfg(not(target_os = "linux"))]
    fn capture_descendants(
        root: ProcessIdentity,
        state: &Arc<Mutex<MonitorState>>,
    ) -> io::Result<usize> {
        capture_descendants_from_snapshot(root, process_snapshot()?, state)
    }

    #[cfg(target_os = "linux")]
    fn capture_descendants_from_snapshot(
        root: ProcessIdentity,
        baseline_children: &HashSet<ProcessIdentity>,
        subreaper_pid: u32,
        snapshot: Vec<ProcessInfo>,
        state: &Arc<Mutex<MonitorState>>,
    ) -> io::Result<usize> {
        let mut state = lock_state(state);
        let before = state.tracked.len();
        let mut descendants = descendant_identities(root, &state.tracked, &snapshot);
        descendants.extend(snapshot.iter().filter_map(|process| {
            (process.parent_pid == subreaper_pid && !baseline_children.contains(&process.identity))
                .then_some(process.identity)
        }));
        add_targets(&mut state, descendants)?;
        Ok(state.tracked.len().saturating_sub(before))
    }

    #[cfg(not(target_os = "linux"))]
    fn capture_descendants_from_snapshot(
        root: ProcessIdentity,
        snapshot: Vec<ProcessInfo>,
        state: &Arc<Mutex<MonitorState>>,
    ) -> io::Result<usize> {
        let mut state = lock_state(state);
        let before = state.tracked.len();
        let descendants = descendant_identities(root, &state.tracked, &snapshot);
        add_targets(&mut state, descendants)?;
        Ok(state.tracked.len().saturating_sub(before))
    }

    #[cfg(target_os = "macos")]
    fn capture_macos_cleanup_targets(
        root: ProcessIdentity,
        baseline_processes: &HashSet<ProcessIdentity>,
        stdout_writer: Option<PipeEndpoint>,
        state: &Arc<Mutex<MonitorState>>,
    ) -> io::Result<usize> {
        let snapshot = process_snapshot()?;
        let pipe_holders = match stdout_writer {
            Some(endpoint) => pipe_holder_identities(endpoint, baseline_processes, &snapshot)?,
            None => HashSet::new(),
        };
        let mut state = lock_state(state);
        let before = state.tracked.len();
        let mut identities = descendant_identities(root, &state.tracked, &snapshot);
        identities.extend(pipe_holders);
        add_targets(&mut state, identities)?;
        Ok(state.tracked.len().saturating_sub(before))
    }

    fn descendant_identities(
        root: ProcessIdentity,
        tracked: &HashMap<ProcessIdentity, TrackedProcess>,
        snapshot: &[ProcessInfo],
    ) -> HashSet<ProcessIdentity> {
        let current: HashMap<u32, ProcessIdentity> = snapshot
            .iter()
            .map(|process| (process.identity.pid, process.identity))
            .collect();
        let mut ancestors: HashSet<u32> = tracked
            .keys()
            .filter(|identity| current.get(&identity.pid) == Some(identity))
            .map(|identity| identity.pid)
            .collect();
        if current.get(&root.pid) == Some(&root) {
            ancestors.insert(root.pid);
        }

        let mut descendants = HashSet::new();
        loop {
            let before = descendants.len();
            for process in snapshot {
                if ancestors.contains(&process.parent_pid) {
                    ancestors.insert(process.identity.pid);
                    descendants.insert(process.identity);
                }
            }
            if descendants.len() == before {
                return descendants;
            }
        }
    }

    fn add_targets(
        state: &mut MonitorState,
        identities: HashSet<ProcessIdentity>,
    ) -> io::Result<()> {
        for identity in identities {
            if state.tracked.contains_key(&identity) {
                continue;
            }
            if let Some(target) = SignalTarget::open(identity)? {
                state
                    .tracked
                    .insert(identity, TrackedProcess { identity, target });
            }
        }
        Ok(())
    }

    fn tracked_are_gone(state: &Arc<Mutex<MonitorState>>) -> io::Result<bool> {
        for identity in lock_state(state).tracked.keys() {
            if process_info(identity.pid)?.is_some_and(|info| info.identity == *identity) {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn remember_monitor_error(state: &Arc<Mutex<MonitorState>>, kind: io::ErrorKind) {
        let mut state = lock_state(state);
        state.first_error.get_or_insert(kind);
    }

    fn take_monitor_error(state: &Arc<Mutex<MonitorState>>) -> io::Result<()> {
        let kind = lock_state(state).first_error.take();
        kind.map_or(Ok(()), |kind| {
            Err(io::Error::new(
                kind,
                "process-tree monitoring failed during the health check",
            ))
        })
    }

    fn lock_state(state: &Arc<Mutex<MonitorState>>) -> MutexGuard<'_, MonitorState> {
        state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn raw_pid(pid: u32) -> Option<Pid> {
        i32::try_from(pid).ok().and_then(Pid::from_raw)
    }

    #[cfg(target_os = "linux")]
    struct SubreaperGuard {
        restore_to_disabled: bool,
        restored: bool,
    }

    #[cfg(target_os = "linux")]
    impl SubreaperGuard {
        fn enable() -> io::Result<Self> {
            let already_enabled = rustix::process::child_subreaper()?.is_some();
            if !already_enabled {
                rustix::process::set_child_subreaper(Some(Pid::INIT))?;
            }
            Ok(Self {
                restore_to_disabled: !already_enabled,
                restored: false,
            })
        }

        fn restore(&mut self) -> io::Result<()> {
            if self.restored {
                return Ok(());
            }
            if self.restore_to_disabled {
                rustix::process::set_child_subreaper(None)?;
            }
            self.restored = true;
            Ok(())
        }
    }

    #[cfg(target_os = "linux")]
    impl Drop for SubreaperGuard {
        fn drop(&mut self) {
            let _ = self.restore();
        }
    }

    #[cfg(target_os = "linux")]
    fn direct_children(parent_pid: u32) -> io::Result<HashSet<ProcessIdentity>> {
        Ok(process_snapshot()?
            .into_iter()
            .filter_map(|process| (process.parent_pid == parent_pid).then_some(process.identity))
            .collect())
    }

    #[cfg(target_os = "linux")]
    fn reap_adopted_descendants(state: &Arc<Mutex<MonitorState>>) {
        use rustix::process::{WaitOptions, waitpid};

        for identity in lock_state(state).tracked.keys() {
            let Some(pid) = raw_pid(identity.pid) else {
                continue;
            };
            match waitpid(Some(pid), WaitOptions::NOHANG) {
                Ok(_) | Err(rustix::io::Errno::CHILD | rustix::io::Errno::SRCH) => {}
                Err(_) => {}
            }
        }
    }

    #[cfg(target_os = "linux")]
    enum SignalTarget {
        PidFd(std::os::fd::OwnedFd),
        VerifiedPid,
    }

    #[cfg(target_os = "linux")]
    impl SignalTarget {
        fn open(identity: ProcessIdentity) -> io::Result<Option<Self>> {
            let Some(pid) = raw_pid(identity.pid) else {
                return Ok(None);
            };
            let target =
                match rustix::process::pidfd_open(pid, rustix::process::PidfdFlags::empty()) {
                    Ok(pidfd) => Self::PidFd(pidfd),
                    Err(rustix::io::Errno::NOSYS | rustix::io::Errno::INVAL) => Self::VerifiedPid,
                    Err(rustix::io::Errno::SRCH) => return Ok(None),
                    Err(source) => return Err(source.into()),
                };
            Ok(
                (process_info(identity.pid)?.map(|info| info.identity) == Some(identity))
                    .then_some(target),
            )
        }

        fn signal(&self, identity: ProcessIdentity, signal: Signal) -> io::Result<()> {
            let result = match self {
                Self::PidFd(pidfd) => rustix::process::pidfd_send_signal(pidfd, signal),
                Self::VerifiedPid => {
                    if process_info(identity.pid)?.map(|info| info.identity) != Some(identity) {
                        return Ok(());
                    }
                    let Some(pid) = raw_pid(identity.pid) else {
                        return Ok(());
                    };
                    rustix::process::kill_process(pid, signal)
                }
            };
            match result {
                Ok(()) | Err(rustix::io::Errno::SRCH) => Ok(()),
                Err(source) => Err(io::Error::new(
                    io::Error::from(source).kind(),
                    "contained-process signaling failed",
                )),
            }
        }
    }

    #[cfg(target_os = "macos")]
    struct SignalTarget;

    #[cfg(target_os = "macos")]
    impl SignalTarget {
        fn open(identity: ProcessIdentity) -> io::Result<Option<Self>> {
            Ok(
                (process_info(identity.pid)?.map(|info| info.identity) == Some(identity))
                    .then_some(Self),
            )
        }

        fn signal(&self, identity: ProcessIdentity, signal: Signal) -> io::Result<()> {
            if process_info(identity.pid)?.map(|info| info.identity) != Some(identity) {
                return Ok(());
            }
            let Some(pid) = raw_pid(identity.pid) else {
                return Ok(());
            };
            match rustix::process::kill_process(pid, signal) {
                Ok(()) | Err(rustix::io::Errno::SRCH) => Ok(()),
                Err(source) => Err(io::Error::new(
                    io::Error::from(source).kind(),
                    "contained-process signaling failed",
                )),
            }
        }
    }

    #[cfg(target_os = "macos")]
    #[repr(C)]
    struct ProcFileInfo {
        open_flags: u32,
        status: u32,
        offset: libc::off_t,
        file_type: i32,
        guard_flags: u32,
    }

    #[cfg(target_os = "macos")]
    #[repr(C)]
    struct PipeInfo {
        stat: libc::vinfo_stat,
        handle: u64,
        peer_handle: u64,
        status: i32,
        reserved: i32,
    }

    #[cfg(target_os = "macos")]
    #[repr(C)]
    struct PipeFdInfo {
        file: ProcFileInfo,
        pipe: PipeInfo,
    }

    #[cfg(target_os = "macos")]
    fn pipe_holder_identities(
        endpoint: PipeEndpoint,
        baseline_processes: &HashSet<ProcessIdentity>,
        snapshot: &[ProcessInfo],
    ) -> io::Result<HashSet<ProcessIdentity>> {
        let mut holders = HashSet::new();
        for process in snapshot {
            if baseline_processes.contains(&process.identity) {
                continue;
            }
            for descriptor in process_file_descriptors(process.identity.pid)? {
                if descriptor.proc_fdtype != u32::try_from(libc::PROX_FDTYPE_PIPE).unwrap_or(6) {
                    continue;
                }
                if pipe_endpoint(process.identity.pid, descriptor.proc_fd)? == Some(endpoint) {
                    holders.insert(process.identity);
                    break;
                }
            }
        }
        Ok(holders)
    }

    #[cfg(target_os = "macos")]
    fn process_file_descriptors(pid: u32) -> io::Result<Vec<libc::proc_fdinfo>> {
        let Ok(raw_pid) = i32::try_from(pid) else {
            return Ok(Vec::new());
        };
        let required = unsafe {
            // SAFETY: A null buffer with zero length is the documented size
            // query for the `PROC_PIDLISTFDS` process-information flavor.
            libc::proc_pidinfo(raw_pid, libc::PROC_PIDLISTFDS, 0, std::ptr::null_mut(), 0)
        };
        if required == 0 {
            return Ok(Vec::new());
        }
        if required < 0 {
            let source = io::Error::last_os_error();
            return if skippable_macos_process_inspection_error(&source) {
                Ok(Vec::new())
            } else {
                Err(source)
            };
        }

        let entry_size = std::mem::size_of::<libc::proc_fdinfo>();
        let required = usize::try_from(required).map_err(io::Error::other)?;
        let capacity = required
            .div_ceil(entry_size)
            .checked_add(32)
            .ok_or_else(|| io::Error::other("process descriptor count overflowed"))?;
        let buffer_bytes = capacity
            .checked_mul(entry_size)
            .ok_or_else(|| io::Error::other("process descriptor buffer overflowed"))?;
        let buffer_size = i32::try_from(buffer_bytes).map_err(io::Error::other)?;
        let mut descriptors = Vec::<libc::proc_fdinfo>::with_capacity(capacity);
        let written = unsafe {
            // SAFETY: `descriptors` owns uninitialized capacity for exactly
            // `buffer_size` bytes. libproc writes only within that allocation;
            // the vector length is set below only for complete returned entries.
            libc::proc_pidinfo(
                raw_pid,
                libc::PROC_PIDLISTFDS,
                0,
                descriptors.as_mut_ptr().cast(),
                buffer_size,
            )
        };
        if written == 0 {
            return Ok(Vec::new());
        }
        if written < 0 {
            let source = io::Error::last_os_error();
            return if skippable_macos_process_inspection_error(&source) {
                Ok(Vec::new())
            } else {
                Err(source)
            };
        }
        let written = usize::try_from(written).map_err(io::Error::other)?;
        if written > buffer_bytes {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "process descriptor inspection exceeded its buffer",
            ));
        }
        unsafe {
            // SAFETY: libproc initialized `written` bytes above. Truncating to
            // complete entries means every value included in the vector is
            // fully initialized and uses the C layout supplied by `libc`.
            descriptors.set_len(written / entry_size);
        }
        Ok(descriptors)
    }

    #[cfg(target_os = "macos")]
    fn pipe_endpoint(pid: u32, descriptor: i32) -> io::Result<Option<PipeEndpoint>> {
        const PROC_PIDFDPIPEINFO: i32 = 6;

        let Ok(raw_pid) = i32::try_from(pid) else {
            return Ok(None);
        };
        let expected = std::mem::size_of::<PipeFdInfo>();
        let size = i32::try_from(expected).map_err(io::Error::other)?;
        let mut info = std::mem::MaybeUninit::<PipeFdInfo>::zeroed();
        let read = unsafe {
            // SAFETY: `info` is exact C-layout storage for the public
            // `PROC_PIDFDPIPEINFO` flavor and is assumed initialized only
            // after libproc reports that every byte was written.
            libc::proc_pidfdinfo(
                raw_pid,
                descriptor,
                PROC_PIDFDPIPEINFO,
                info.as_mut_ptr().cast(),
                size,
            )
        };
        if read <= 0 {
            let source = io::Error::last_os_error();
            return if skippable_macos_process_inspection_error(&source) {
                Ok(None)
            } else {
                Err(source)
            };
        }
        if usize::try_from(read).ok() != Some(expected) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "stdout pipe inspection returned an unexpected size",
            ));
        }
        let info = unsafe {
            // SAFETY: The exact-size check above proves libproc initialized
            // the complete `PipeFdInfo` value.
            info.assume_init()
        };
        Ok(Some(PipeEndpoint {
            handle: info.pipe.handle,
            peer_handle: info.pipe.peer_handle,
        }))
    }

    #[cfg(target_os = "macos")]
    fn skippable_macos_process_inspection_error(source: &io::Error) -> bool {
        matches!(
            source.kind(),
            io::ErrorKind::NotFound | io::ErrorKind::PermissionDenied
        ) || matches!(
            source.raw_os_error(),
            Some(libc::ESRCH | libc::ENOENT | libc::EBADF | libc::EPERM | libc::EACCES)
        )
    }

    #[cfg(target_os = "linux")]
    fn process_snapshot() -> io::Result<Vec<ProcessInfo>> {
        let mut processes = Vec::new();
        for entry in std::fs::read_dir("/proc")? {
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => continue,
            };
            let Some(pid) = entry
                .file_name()
                .to_str()
                .and_then(|name| name.parse::<u32>().ok())
            else {
                continue;
            };
            if let Some(info) = linux_process_info_at(pid, &entry.path())? {
                processes.push(info);
            }
        }
        Ok(processes)
    }

    #[cfg(target_os = "linux")]
    fn process_info(pid: u32) -> io::Result<Option<ProcessInfo>> {
        linux_process_info_at(pid, &std::path::Path::new("/proc").join(pid.to_string()))
    }

    #[cfg(target_os = "linux")]
    fn linux_process_info_at(
        pid: u32,
        process_path: &std::path::Path,
    ) -> io::Result<Option<ProcessInfo>> {
        let stat = match std::fs::read_to_string(process_path.join("stat")) {
            Ok(stat) => stat,
            Err(source) if skippable_linux_process_inspection_error(&source) => return Ok(None),
            Err(source) => return Err(source),
        };
        parse_linux_stat(pid, &stat).map(Some)
    }

    #[cfg(target_os = "linux")]
    fn skippable_linux_process_inspection_error(source: &io::Error) -> bool {
        matches!(
            source.kind(),
            io::ErrorKind::NotFound | io::ErrorKind::PermissionDenied
        ) || source.raw_os_error() == Some(libc::ESRCH)
    }

    #[cfg(target_os = "linux")]
    fn parse_linux_stat(pid: u32, stat: &str) -> io::Result<ProcessInfo> {
        let fields = stat
            .rfind(')')
            .and_then(|end| stat.get(end.saturating_add(2)..))
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid process stat"))?
            .split_ascii_whitespace()
            .collect::<Vec<_>>();
        let parent_pid = fields
            .get(1)
            .and_then(|field| field.parse().ok())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid process parent"))?;
        let process_group = fields
            .get(2)
            .and_then(|field| field.parse().ok())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid process group"))?;
        let started = fields
            .get(19)
            .and_then(|field| field.parse().ok())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid process start"))?;
        Ok(ProcessInfo {
            identity: ProcessIdentity {
                pid,
                started: ProcessStart {
                    major: started,
                    minor: 0,
                },
            },
            parent_pid,
            process_group,
        })
    }

    #[cfg(target_os = "macos")]
    fn process_snapshot() -> io::Result<Vec<ProcessInfo>> {
        const PROC_ALL_PIDS: u32 = 1;
        let required = unsafe {
            // SAFETY: A null buffer with zero length is the documented size
            // query for libproc's process-list API.
            libc::proc_listpids(PROC_ALL_PIDS, 0, std::ptr::null_mut(), 0)
        };
        if required <= 0 {
            return Err(io::Error::last_os_error());
        }
        let extra = 64 * std::mem::size_of::<u32>();
        let bytes = usize::try_from(required).unwrap_or(0).saturating_add(extra);
        let mut pids = vec![0_u32; bytes / std::mem::size_of::<u32>()];
        let buffer_size =
            i32::try_from(pids.len() * std::mem::size_of::<u32>()).map_err(io::Error::other)?;
        let written = unsafe {
            // SAFETY: `pids` is initialized writable storage and the byte
            // length passed to libproc exactly matches its allocation.
            libc::proc_listpids(PROC_ALL_PIDS, 0, pids.as_mut_ptr().cast(), buffer_size)
        };
        if written <= 0 {
            return Err(io::Error::last_os_error());
        }
        pids.truncate(usize::try_from(written).unwrap_or(0) / std::mem::size_of::<u32>());
        Ok(pids
            .into_iter()
            .filter(|pid| *pid != 0)
            .filter_map(|pid| process_info(pid).ok().flatten())
            .collect())
    }

    #[cfg(target_os = "macos")]
    fn process_info(pid: u32) -> io::Result<Option<ProcessInfo>> {
        let Ok(raw_pid) = i32::try_from(pid) else {
            return Ok(None);
        };
        let mut info = std::mem::MaybeUninit::<libc::proc_bsdinfo>::zeroed();
        let expected = std::mem::size_of::<libc::proc_bsdinfo>();
        let size = i32::try_from(expected).map_err(io::Error::other)?;
        let read = unsafe {
            // SAFETY: `info` points to storage of the exact flavor-specific
            // size required by `PROC_PIDTBSDINFO`. It is assumed initialized
            // only after libproc reports that every byte was written.
            libc::proc_pidinfo(
                raw_pid,
                libc::PROC_PIDTBSDINFO,
                0,
                info.as_mut_ptr().cast(),
                size,
            )
        };
        if usize::try_from(read).ok() != Some(expected) {
            return Ok(None);
        }
        let info = unsafe {
            // SAFETY: The exact-size check above proves libproc initialized
            // the complete `proc_bsdinfo` value.
            info.assume_init()
        };
        Ok(Some(ProcessInfo {
            identity: ProcessIdentity {
                pid: info.pbi_pid,
                started: ProcessStart {
                    major: info.pbi_start_tvsec,
                    minor: info.pbi_start_tvusec,
                },
            },
            parent_pid: info.pbi_ppid,
            process_group: info.pbi_pgid,
        }))
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn identity(pid: u32, started: u64) -> ProcessIdentity {
            ProcessIdentity {
                pid,
                started: ProcessStart {
                    major: started,
                    minor: 0,
                },
            }
        }

        fn info(pid: u32, parent_pid: u32, started: u64) -> ProcessInfo {
            ProcessInfo {
                identity: identity(pid, started),
                parent_pid,
                process_group: pid,
            }
        }

        #[test]
        fn descendant_closure_follows_forks_after_process_group_escape() {
            let root = identity(40, 1);
            let tracked = HashMap::new();
            let snapshot = [
                info(40, 10, 1),
                info(41, 40, 2),
                info(42, 41, 3),
                info(90, 10, 4),
            ];

            let descendants = descendant_identities(root, &tracked, &snapshot);

            assert_eq!(
                descendants,
                HashSet::from([identity(41, 2), identity(42, 3)])
            );
        }

        #[test]
        fn reused_pid_is_not_treated_as_the_tracked_process() {
            let root = identity(40, 1);
            let tracked = HashMap::from([(
                identity(41, 2),
                TrackedProcess {
                    identity: identity(41, 2),
                    target: SignalTarget::open(identity(41, 2))
                        .ok()
                        .flatten()
                        .unwrap_or_else(test_signal_target),
                },
            )]);
            let snapshot = [info(41, 10, 99), info(42, 41, 3)];

            let descendants = descendant_identities(root, &tracked, &snapshot);

            assert!(descendants.is_empty());
        }

        #[test]
        fn reused_root_pid_does_not_authorize_process_group_signaling() {
            let root = identity(40, 1);
            let snapshot = [info(40, 10, 99), info(41, 40, 2)];

            assert!(!snapshot_contains_group_root(root, &snapshot));
        }

        #[cfg(target_os = "linux")]
        fn test_signal_target() -> SignalTarget {
            SignalTarget::VerifiedPid
        }

        #[cfg(target_os = "macos")]
        fn test_signal_target() -> SignalTarget {
            SignalTarget
        }

        #[cfg(target_os = "linux")]
        #[test]
        fn linux_stat_parser_uses_parent_and_kernel_start_ticks() {
            let stat =
                "42 (name with ) parenthesis) S 17 42 42 0 -1 0 0 0 0 0 0 0 0 0 0 0 0 0 12345 0";

            let parsed = parse_linux_stat(42, stat).expect("synthetic stat should parse");

            assert_eq!(parsed, info(42, 17, 12345));
        }

        #[cfg(target_os = "linux")]
        #[test]
        fn linux_snapshot_skips_only_expected_process_inspection_races() {
            for error in [
                io::Error::from(io::ErrorKind::NotFound),
                io::Error::from(io::ErrorKind::PermissionDenied),
                io::Error::from_raw_os_error(libc::ESRCH),
            ] {
                assert!(skippable_linux_process_inspection_error(&error));
            }
            assert!(!skippable_linux_process_inspection_error(
                &io::Error::from_raw_os_error(libc::EIO)
            ));
        }

        #[test]
        fn native_snapshot_identifies_the_current_process() {
            let current = process_info(std::process::id())
                .expect("the current process should be inspectable")
                .expect("the current process should exist");

            assert_eq!(current.identity.pid, std::process::id());
            assert_ne!(current.identity.started.major, 0);
        }
    }
}

#[cfg(windows)]
mod windows {
    use super::{
        cleanup_deadline, combine_setup_error, record_first, remaining_cleanup_time,
        terminate_and_reap_direct,
    };
    // The Windows implementation is kept in this module so its native handles
    // cannot cross into the protocol or CLI layers.
    use std::io;
    use std::os::windows::io::AsRawHandle;
    use std::os::windows::process::CommandExt;
    use std::process::{Child, Command};
    use std::time::{Duration, Instant};
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, TH32CS_SNAPTHREAD, THREADENTRY32, Thread32First, Thread32Next,
    };
    use windows_sys::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        JOBOBJECT_BASIC_ACCOUNTING_INFORMATION, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
        JobObjectBasicAccountingInformation, JobObjectExtendedLimitInformation,
        QueryInformationJobObject, SetInformationJobObject, TerminateJobObject,
    };
    use windows_sys::Win32::System::Threading::{
        CREATE_SUSPENDED, GetProcessId, OpenThread, ResumeThread, THREAD_SUSPEND_RESUME,
    };

    #[derive(Debug)]
    struct JobHandle(HANDLE);

    impl Drop for JobHandle {
        fn drop(&mut self) {
            if !self.0.is_null() && self.0 != INVALID_HANDLE_VALUE {
                // SAFETY: `JobHandle` owns this non-null, non-sentinel handle
                // and closes it exactly once from `Drop`.
                let _ = unsafe { CloseHandle(self.0) };
            }
        }
    }

    pub(super) struct WindowsContainment {
        job: JobHandle,
        finished: bool,
    }

    impl WindowsContainment {
        pub(super) fn spawn(
            command: &mut Command,
            cleanup_timeout: Duration,
        ) -> io::Result<(Child, Self)> {
            let job = create_kill_on_close_job()?;
            command.creation_flags(CREATE_SUSPENDED);
            let mut child = command.spawn()?;
            let process = child.as_raw_handle().cast();

            // SAFETY: The Job Object is owned by this containment value and
            // `process` is the live suspended child's borrowed process handle.
            if unsafe { AssignProcessToJobObject(job.0, process) } == 0 {
                let source = io::Error::last_os_error();
                let mut cleanup_failure = terminate_setup_child(&mut child, cleanup_timeout).err();
                if cleanup_failure.is_some() {
                    cleanup_failure = terminate_setup_child(&mut child, cleanup_timeout).err();
                }
                return Err(combine_setup_error(source, cleanup_failure));
            }
            let mut containment = Self {
                job,
                finished: false,
            };
            if let Err(source) = resume_process_threads(process) {
                let mut cleanup_failure =
                    cleanup_setup_job_child(&mut child, &mut containment, cleanup_timeout).err();
                if cleanup_failure.is_some() {
                    cleanup_failure =
                        cleanup_setup_job_child(&mut child, &mut containment, cleanup_timeout)
                            .err();
                }
                return Err(combine_setup_error(source, cleanup_failure));
            }

            Ok((child, containment))
        }

        pub(super) fn terminate(&mut self) -> io::Result<()> {
            // SAFETY: `self.job` remains owned and open until this
            // containment value is dropped.
            if unsafe { TerminateJobObject(self.job.0, 1) } != 0 {
                return Ok(());
            }
            let source = io::Error::last_os_error();
            match active_processes(self.job.0) {
                Ok(0) => Ok(()),
                Ok(_) => Err(source),
                Err(query) => Err(query),
            }
        }

        pub(super) fn wait_until_empty(&mut self, timeout: Duration) -> io::Result<()> {
            let deadline = Instant::now()
                .checked_add(timeout)
                .unwrap_or_else(Instant::now);
            loop {
                if active_processes(self.job.0)? == 0 {
                    return Ok(());
                }
                let now = Instant::now();
                if now >= deadline {
                    return Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        "contained processes did not exit within the cleanup bound",
                    ));
                }
                std::thread::sleep(Duration::from_millis(10).min(deadline.duration_since(now)));
            }
        }

        pub(super) fn finish(&mut self) -> io::Result<()> {
            self.finished = true;
            Ok(())
        }
    }

    fn create_kill_on_close_job() -> io::Result<JobHandle> {
        // SAFETY: Null security attributes and name request a private Job
        // Object with default security, as documented by Win32.
        let handle = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
        if handle.is_null() {
            return Err(io::Error::last_os_error());
        }
        let job = JobHandle(handle);
        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        // SAFETY: `limits` is initialized storage of the exact information
        // class size and remains alive for the duration of this call.
        let configured = unsafe {
            SetInformationJobObject(
                job.0,
                JobObjectExtendedLimitInformation,
                &limits as *const _ as _,
                u32::try_from(std::mem::size_of_val(&limits)).map_err(io::Error::other)?,
            )
        };
        if configured == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(job)
    }

    fn resume_process_threads(process: HANDLE) -> io::Result<()> {
        // SAFETY: `process` is borrowed from the live suspended `Child`.
        let process_id = unsafe { GetProcessId(process) };
        if process_id == 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: This call accepts scalar flags and returns a newly owned
        // snapshot handle or the documented invalid-handle sentinel.
        let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) };
        if snapshot == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }
        let snapshot = JobHandle(snapshot);
        let mut entry = THREADENTRY32 {
            dwSize: u32::try_from(std::mem::size_of::<THREADENTRY32>())
                .map_err(io::Error::other)?,
            ..Default::default()
        };
        // SAFETY: `entry` is initialized with the required `dwSize`, and the
        // owned snapshot remains open throughout enumeration.
        let mut next = unsafe { Thread32First(snapshot.0, &mut entry) } != 0;
        let mut resumed = 0_u32;
        while next {
            if entry.th32OwnerProcessID == process_id {
                // SAFETY: The enumerated thread identifier is passed as a
                // scalar and the returned handle is validated before use.
                let thread = unsafe { OpenThread(THREAD_SUSPEND_RESUME, 0, entry.th32ThreadID) };
                if thread.is_null() {
                    return Err(io::Error::last_os_error());
                }
                let thread = JobHandle(thread);
                // SAFETY: `thread` owns a live handle opened with the exact
                // resume permission required by this call.
                if unsafe { ResumeThread(thread.0) } == u32::MAX {
                    return Err(io::Error::last_os_error());
                }
                resumed = resumed.saturating_add(1);
            }
            // SAFETY: The same initialized entry and live snapshot from the
            // first enumeration call remain valid here.
            next = unsafe { Thread32Next(snapshot.0, &mut entry) } != 0;
        }
        if resumed == 0 {
            Err(io::Error::other(
                "process containment could not resume the configured process",
            ))
        } else {
            Ok(())
        }
    }

    fn active_processes(job: HANDLE) -> io::Result<u32> {
        let mut accounting = JOBOBJECT_BASIC_ACCOUNTING_INFORMATION::default();
        // SAFETY: `accounting` is initialized writable storage of the exact
        // queried information-class size, and `job` is a live borrowed handle.
        let queried = unsafe {
            QueryInformationJobObject(
                job,
                JobObjectBasicAccountingInformation,
                &mut accounting as *mut _ as _,
                u32::try_from(std::mem::size_of_val(&accounting)).map_err(io::Error::other)?,
                std::ptr::null_mut(),
            )
        };
        if queried == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(accounting.ActiveProcesses)
    }

    fn cleanup_setup_job_child(
        child: &mut Child,
        containment: &mut WindowsContainment,
        timeout: Duration,
    ) -> io::Result<()> {
        let deadline = cleanup_deadline(timeout);
        let mut failure = None;
        record_first(&mut failure, containment.terminate());
        record_first(
            &mut failure,
            terminate_and_reap_direct(child, remaining_cleanup_time(deadline)),
        );
        record_first(
            &mut failure,
            containment.wait_until_empty(remaining_cleanup_time(deadline)),
        );
        failure.map_or(Ok(()), Err)
    }

    fn terminate_setup_child(child: &mut Child, timeout: Duration) -> io::Result<()> {
        terminate_and_reap_direct(child, timeout)
    }
}
