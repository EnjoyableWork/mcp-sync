use crate::config::{CanonicalConfig, CanonicalServer, ConfigError, parse_unique_json_value};
use crate::filesystem::{FileIoError, FileSystem};
use crate::paths::MacOsConfigurationPaths;
use serde::Serialize;
use serde_json::{Map, Value};
use std::env;
use std::error::Error;
use std::fmt;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, ExitStatus, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

const INITIALIZE_REQUEST_ID: u64 = 1;
const CURRENT_HANDSHAKE_PROTOCOL_VERSION: &str = "2025-11-25";
const RESPONSE_TIMEOUT: Duration = Duration::from_secs(5);
const SHUTDOWN_TIMEOUT: Duration = Duration::from_millis(500);
const PROCESS_POLL_INTERVAL: Duration = Duration::from_millis(10);
const MAX_RESPONSE_BYTES: usize = 1024 * 1024;

const INITIALIZED_NOTIFICATION: &[u8] =
    b"{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n";

/// Run one explicit, bounded MCP STDIO initialize health test.
///
/// This is the sole application boundary that executes a configured server.
/// Canonical process values cross into the process adapter but never enter the
/// returned report, diagnostics, or debug output.
pub fn test_server(
    paths: &MacOsConfigurationPaths,
    filesystem: &impl FileSystem,
    tester: &impl InitializeTester,
    name: &str,
) -> Result<HealthReport, HealthError> {
    let path = paths.canonical_configuration();
    let bytes = match filesystem.read(path) {
        Ok(bytes) => bytes,
        Err(source) if source.kind() == io::ErrorKind::NotFound => {
            return Err(HealthError::NotInitialized {
                path: path.to_owned(),
            });
        }
        Err(source) => return Err(HealthError::Read { source }),
    };
    let document = std::str::from_utf8(&bytes).map_err(|_| HealthError::InvalidEncoding {
        path: path.to_owned(),
    })?;
    let config =
        CanonicalConfig::parse_json(document).map_err(|source| HealthError::InvalidCanonical {
            path: path.to_owned(),
            source,
        })?;
    let server = config
        .servers()
        .get(name)
        .ok_or_else(|| HealthError::UnknownServer {
            name: name.to_owned(),
        })?;

    let protocol = tester
        .test_initialize(server)
        .map_err(|source| HealthError::TestFailed {
            name: name.to_owned(),
            source,
        })?;

    Ok(HealthReport {
        name: name.to_owned(),
        protocol,
    })
}

/// Replaceable boundary for the only command allowed to start a configured
/// MCP server.
pub trait InitializeTester {
    fn test_initialize(
        &self,
        server: &CanonicalServer,
    ) -> Result<NegotiatedProtocol, InitializeError>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct OsInitializeTester;

impl InitializeTester for OsInitializeTester {
    fn test_initialize(
        &self,
        server: &CanonicalServer,
    ) -> Result<NegotiatedProtocol, InitializeError> {
        run_initialize(
            server,
            HealthLimits {
                response_timeout: RESPONSE_TIMEOUT,
                shutdown_timeout: SHUTDOWN_TIMEOUT,
                maximum_response_bytes: MAX_RESPONSE_BYTES,
            },
        )
    }
}

#[derive(Clone, Copy)]
struct HealthLimits {
    response_timeout: Duration,
    shutdown_timeout: Duration,
    maximum_response_bytes: usize,
}

fn run_initialize(
    server: &CanonicalServer,
    limits: HealthLimits,
) -> Result<NegotiatedProtocol, InitializeError> {
    let request = initialize_request()?;
    let mut session = OsStdioSession::spawn(server, limits.maximum_response_bytes)?;

    let interaction = (|| {
        session.send(&request, MessagePhase::InitializeRequest)?;
        let response = session.receive(limits.response_timeout)?;
        let protocol = validate_initialize_response(&response)?;
        session.send(
            INITIALIZED_NOTIFICATION,
            MessagePhase::InitializedNotification,
        )?;
        Ok(protocol)
    })();

    match interaction {
        Ok(protocol) => {
            session.shutdown(limits.shutdown_timeout)?;
            Ok(protocol)
        }
        Err(failure) => match session.terminate(limits.shutdown_timeout) {
            Ok(()) => Err(failure),
            Err(cleanup) => Err(InitializeError::CleanupFailed {
                failure: Box::new(failure),
                cleanup,
            }),
        },
    }
}

#[derive(Serialize)]
struct InitializeRequest<'a> {
    jsonrpc: &'static str,
    id: u64,
    method: &'static str,
    params: InitializeParams<'a>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct InitializeParams<'a> {
    protocol_version: &'static str,
    capabilities: Map<String, Value>,
    client_info: ClientInfo<'a>,
}

#[derive(Serialize)]
struct ClientInfo<'a> {
    name: &'static str,
    version: &'a str,
}

fn initialize_request() -> Result<Vec<u8>, InitializeError> {
    let request = InitializeRequest {
        jsonrpc: "2.0",
        id: INITIALIZE_REQUEST_ID,
        method: "initialize",
        params: InitializeParams {
            protocol_version: CURRENT_HANDSHAKE_PROTOCOL_VERSION,
            capabilities: Map::new(),
            client_info: ClientInfo {
                name: "mcp-sync",
                version: env!("CARGO_PKG_VERSION"),
            },
        },
    };
    let mut bytes = serde_json::to_vec(&request)
        .map_err(|_| InitializeError::CannotSerializeInitializeRequest)?;
    bytes.push(b'\n');
    Ok(bytes)
}

struct OsStdioSession {
    child: Option<Child>,
    stdin: Option<ChildStdin>,
    response: Receiver<Result<Vec<u8>, ResponseReadError>>,
    reader: Option<JoinHandle<()>>,
}

impl OsStdioSession {
    fn spawn(
        server: &CanonicalServer,
        maximum_response_bytes: usize,
    ) -> Result<Self, InitializeError> {
        let mut command = Command::new(server.command());
        command
            .args(server.args())
            .env_clear()
            .envs(server.env())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());

        // A literal command name still needs a search path. No other ambient
        // variables cross the process boundary unless canonical state names
        // them explicitly.
        if !server.env().contains_key("PATH")
            && let Some(path) = env::var_os("PATH")
        {
            command.env("PATH", path);
        }

        let mut child = command
            .spawn()
            .map_err(|source| InitializeError::CannotStart { source })?;
        let Some(stdin) = child.stdin.take() else {
            terminate_unmanaged_child(&mut child);
            return Err(InitializeError::CannotOpenStdin);
        };
        let Some(stdout) = child.stdout.take() else {
            drop(stdin);
            terminate_unmanaged_child(&mut child);
            return Err(InitializeError::CannotOpenStdout);
        };

        let (sender, response) = mpsc::channel();
        let reader = thread::spawn(move || {
            let mut stdout = BufReader::new(stdout);
            let result = read_bounded_response(&mut stdout, maximum_response_bytes);
            let should_drain = result.is_ok();
            if sender.send(result).is_ok() && should_drain {
                // Keep the protocol pipe open and drained until shutdown so a
                // compliant server cannot block while emitting notifications.
                let _ = io::copy(&mut stdout, &mut io::sink());
            }
        });

        Ok(Self {
            child: Some(child),
            stdin: Some(stdin),
            response,
            reader: Some(reader),
        })
    }

    fn send(&mut self, message: &[u8], phase: MessagePhase) -> Result<(), InitializeError> {
        let stdin = self
            .stdin
            .as_mut()
            .ok_or(InitializeError::ProcessInputClosed { phase })?;
        stdin
            .write_all(message)
            .and_then(|()| stdin.flush())
            .map_err(|source| InitializeError::CannotWrite { phase, source })
    }

    fn receive(&mut self, timeout: Duration) -> Result<Vec<u8>, InitializeError> {
        let received = self.response.recv_timeout(timeout);
        match received {
            Ok(result) => result.map_err(InitializeError::from),
            Err(RecvTimeoutError::Timeout) => Err(InitializeError::ResponseTimedOut { timeout }),
            Err(RecvTimeoutError::Disconnected) => {
                self.join_finished_reader()?;
                Err(InitializeError::ResponseReaderStopped)
            }
        }
    }

    fn shutdown(&mut self, timeout: Duration) -> Result<(), InitializeError> {
        drop(self.stdin.take());
        let wait_result = {
            let child = self.child.as_mut().ok_or(InitializeError::ProcessLost)?;
            wait_for_exit(child, timeout)
        };
        let status = match wait_result {
            Ok(status) => status,
            Err(source) => {
                let failure = InitializeError::CannotWaitForShutdown { source };
                return match self.terminate(timeout) {
                    Ok(()) => Err(failure),
                    Err(cleanup) => Err(InitializeError::CleanupFailed {
                        failure: Box::new(failure),
                        cleanup,
                    }),
                };
            }
        };

        match status {
            Some(status) => {
                self.child.take();
                self.finish_reader(timeout)
                    .map_err(|cleanup| InitializeError::CleanupFailed {
                        failure: Box::new(InitializeError::ProcessLost),
                        cleanup,
                    })?;
                if status.success() {
                    Ok(())
                } else {
                    Err(InitializeError::UnsuccessfulShutdown)
                }
            }
            None => {
                self.force_terminate()
                    .map_err(|cleanup| InitializeError::CleanupFailed {
                        failure: Box::new(InitializeError::ShutdownTimedOut { timeout }),
                        cleanup,
                    })?;
                self.finish_reader(timeout)
                    .map_err(|cleanup| InitializeError::CleanupFailed {
                        failure: Box::new(InitializeError::ShutdownTimedOut { timeout }),
                        cleanup,
                    })?;
                Err(InitializeError::ShutdownTimedOut { timeout })
            }
        }
    }

    fn terminate(&mut self, reader_timeout: Duration) -> Result<(), CleanupError> {
        drop(self.stdin.take());
        self.force_terminate()?;
        self.finish_reader(reader_timeout)
    }

    fn force_terminate(&mut self) -> Result<(), CleanupError> {
        let Some(mut child) = self.child.take() else {
            return Ok(());
        };

        let result = match child.try_wait().map_err(CleanupError::Inspect) {
            Err(error) => Err(error),
            Ok(Some(_)) => Ok(()),
            Ok(None) => match child.kill() {
                Ok(()) => child.wait().map(drop).map_err(CleanupError::Wait),
                Err(kill_error) => match child.try_wait().map_err(CleanupError::Inspect) {
                    Ok(Some(_)) => Ok(()),
                    Ok(None) => Err(CleanupError::Kill(kill_error)),
                    Err(error) => Err(error),
                },
            },
        };
        if result.is_err() {
            self.child = Some(child);
        }
        result
    }

    fn join_finished_reader(&mut self) -> Result<(), InitializeError> {
        let Some(reader) = self.reader.take() else {
            return Ok(());
        };
        reader
            .join()
            .map_err(|_| InitializeError::ResponseReaderPanicked)
    }

    fn finish_reader(&mut self, timeout: Duration) -> Result<(), CleanupError> {
        let Some(reader) = self.reader.take() else {
            return Ok(());
        };
        let deadline = Instant::now()
            .checked_add(timeout)
            .unwrap_or_else(Instant::now);
        while !reader.is_finished() {
            let now = Instant::now();
            if now >= deadline {
                return Err(CleanupError::ReaderTimedOut);
            }
            thread::sleep(PROCESS_POLL_INTERVAL.min(deadline.duration_since(now)));
        }
        reader.join().map_err(|_| CleanupError::ReaderPanicked)
    }
}

impl Drop for OsStdioSession {
    fn drop(&mut self) {
        drop(self.stdin.take());
        let terminated = self.force_terminate().is_ok();
        if terminated && self.reader.as_ref().is_some_and(JoinHandle::is_finished) {
            let _ = self.join_finished_reader();
        }
    }
}

fn terminate_unmanaged_child(child: &mut Child) {
    if child.try_wait().ok().flatten().is_none() {
        let _ = child.kill();
        let _ = child.wait();
    }
}

fn wait_for_exit(child: &mut Child, timeout: Duration) -> io::Result<Option<ExitStatus>> {
    let deadline = Instant::now()
        .checked_add(timeout)
        .unwrap_or_else(Instant::now);
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(Some(status));
        }
        let now = Instant::now();
        if now >= deadline {
            return Ok(None);
        }
        thread::sleep(PROCESS_POLL_INTERVAL.min(deadline.duration_since(now)));
    }
}

fn read_bounded_response(
    stdout: &mut BufReader<ChildStdout>,
    maximum_response_bytes: usize,
) -> Result<Vec<u8>, ResponseReadError> {
    let limit = u64::try_from(maximum_response_bytes)
        .unwrap_or(u64::MAX)
        .saturating_add(1);
    let mut reader = stdout.take(limit);
    let mut response = Vec::new();
    let read = reader
        .read_until(b'\n', &mut response)
        .map_err(ResponseReadError::Io)?;
    if read == 0 {
        return Err(ResponseReadError::Closed);
    }
    if response.len() > maximum_response_bytes {
        return Err(ResponseReadError::TooLarge {
            maximum: maximum_response_bytes,
        });
    }
    if response.last() != Some(&b'\n') {
        return Err(ResponseReadError::MissingDelimiter);
    }
    Ok(response)
}

fn validate_initialize_response(response: &[u8]) -> Result<NegotiatedProtocol, InitializeError> {
    let value = parse_unique_json_value(response)
        .map_err(|_| InitializeError::InvalidResponse(ResponseViolation::MalformedJson))?;
    let object = value.as_object().ok_or(InitializeError::InvalidResponse(
        ResponseViolation::RootMustBeObject,
    ))?;

    if object.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
        return Err(InitializeError::InvalidResponse(
            ResponseViolation::InvalidJsonRpcVersion,
        ));
    }
    if object.get("id").and_then(Value::as_u64) != Some(INITIALIZE_REQUEST_ID) {
        return Err(InitializeError::InvalidResponse(
            ResponseViolation::MismatchedIdentifier,
        ));
    }

    match (object.get("result"), object.get("error")) {
        (Some(_), Some(_)) => Err(InitializeError::InvalidResponse(
            ResponseViolation::ResultAndError,
        )),
        (None, None) => Err(InitializeError::InvalidResponse(
            ResponseViolation::MissingResultOrError,
        )),
        (None, Some(error)) => validate_error_response(error),
        (Some(result), None) => validate_result(result),
    }
}

fn validate_error_response(error: &Value) -> Result<NegotiatedProtocol, InitializeError> {
    let Some(error) = error.as_object() else {
        return Err(InitializeError::InvalidResponse(
            ResponseViolation::MalformedError,
        ));
    };
    let code_is_integer = error
        .get("code")
        .and_then(Value::as_i64)
        .or_else(|| error.get("code").and_then(Value::as_u64).map(|_| 0))
        .is_some();
    let message_is_string = error.get("message").and_then(Value::as_str).is_some();
    if !code_is_integer || !message_is_string {
        return Err(InitializeError::InvalidResponse(
            ResponseViolation::MalformedError,
        ));
    }
    Err(InitializeError::InitializeRejected)
}

fn validate_result(result: &Value) -> Result<NegotiatedProtocol, InitializeError> {
    let result = result.as_object().ok_or(InitializeError::InvalidResponse(
        ResponseViolation::ResultMustBeObject,
    ))?;
    let protocol = result
        .get("protocolVersion")
        .and_then(Value::as_str)
        .and_then(NegotiatedProtocol::from_wire)
        .ok_or(InitializeError::InvalidResponse(
            ResponseViolation::UnsupportedProtocolVersion,
        ))?;
    if !result.get("capabilities").is_some_and(Value::is_object) {
        return Err(InitializeError::InvalidResponse(
            ResponseViolation::CapabilitiesMustBeObject,
        ));
    }
    let server_info = result.get("serverInfo").and_then(Value::as_object).ok_or(
        InitializeError::InvalidResponse(ResponseViolation::ServerInfoMustBeObject),
    )?;
    if server_info.get("name").and_then(Value::as_str).is_none()
        || server_info.get("version").and_then(Value::as_str).is_none()
    {
        return Err(InitializeError::InvalidResponse(
            ResponseViolation::InvalidServerInfo,
        ));
    }
    Ok(protocol)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NegotiatedProtocol {
    V2025_11_25,
    V2025_06_18,
    V2025_03_26,
    V2024_11_05,
}

impl NegotiatedProtocol {
    fn from_wire(value: &str) -> Option<Self> {
        match value {
            "2025-11-25" => Some(Self::V2025_11_25),
            "2025-06-18" => Some(Self::V2025_06_18),
            "2025-03-26" => Some(Self::V2025_03_26),
            "2024-11-05" => Some(Self::V2024_11_05),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::V2025_11_25 => "2025-11-25",
            Self::V2025_06_18 => "2025-06-18",
            Self::V2025_03_26 => "2025-03-26",
            Self::V2024_11_05 => "2024-11-05",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HealthReport {
    name: String,
    protocol: NegotiatedProtocol,
}

impl fmt::Display for HealthReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Server {:?} initialized successfully via STDIO (protocol version {}).",
            self.name,
            self.protocol.as_str()
        )
    }
}

#[derive(Debug)]
pub enum HealthError {
    NotInitialized {
        path: PathBuf,
    },
    Read {
        source: FileIoError,
    },
    InvalidEncoding {
        path: PathBuf,
    },
    InvalidCanonical {
        path: PathBuf,
        source: ConfigError,
    },
    UnknownServer {
        name: String,
    },
    TestFailed {
        name: String,
        source: InitializeError,
    },
}

impl fmt::Display for HealthError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotInitialized { path } => write!(
                formatter,
                "canonical configuration does not exist at `{}`; run `mcp-sync init` first",
                path.display()
            ),
            Self::Read { source } => {
                write!(formatter, "cannot read canonical configuration: {source}")
            }
            Self::InvalidEncoding { path } => write!(
                formatter,
                "canonical configuration at `{}` is not valid UTF-8; restore or repair it before continuing",
                path.display()
            ),
            Self::InvalidCanonical { path, source } => write!(
                formatter,
                "canonical configuration at `{}` is invalid: {source}; restore or repair it before continuing",
                path.display()
            ),
            Self::UnknownServer { name } => write!(
                formatter,
                "server {name:?} is not present in canonical configuration; run `mcp-sync list` to review configured names"
            ),
            Self::TestFailed { name, source } => {
                write!(
                    formatter,
                    "health test for server {name:?} failed: {source}"
                )
            }
        }
    }
}

impl Error for HealthError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Read { source } => Some(source),
            Self::InvalidCanonical { source, .. } => Some(source),
            Self::TestFailed { source, .. } => Some(source),
            Self::NotInitialized { .. }
            | Self::InvalidEncoding { .. }
            | Self::UnknownServer { .. } => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessagePhase {
    InitializeRequest,
    InitializedNotification,
}

impl MessagePhase {
    fn description(self) -> &'static str {
        match self {
            Self::InitializeRequest => "initialize request",
            Self::InitializedNotification => "initialized notification",
        }
    }
}

#[derive(Debug)]
pub enum InitializeError {
    CannotSerializeInitializeRequest,
    CannotStart {
        source: io::Error,
    },
    CannotOpenStdin,
    CannotOpenStdout,
    ProcessInputClosed {
        phase: MessagePhase,
    },
    CannotWrite {
        phase: MessagePhase,
        source: io::Error,
    },
    ResponseTimedOut {
        timeout: Duration,
    },
    ResponseClosed,
    ResponseTooLarge {
        maximum: usize,
    },
    ResponseMissingDelimiter,
    ResponseRead {
        source: io::Error,
    },
    ResponseReaderStopped,
    ResponseReaderPanicked,
    InvalidResponse(ResponseViolation),
    InitializeRejected,
    ProcessLost,
    CannotWaitForShutdown {
        source: io::Error,
    },
    ShutdownTimedOut {
        timeout: Duration,
    },
    UnsuccessfulShutdown,
    CleanupFailed {
        failure: Box<Self>,
        cleanup: CleanupError,
    },
}

impl fmt::Display for InitializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CannotSerializeInitializeRequest => {
                formatter.write_str("could not construct the MCP initialize request")
            }
            Self::CannotStart { source } => {
                write!(
                    formatter,
                    "could not start the configured process: {source}"
                )
            }
            Self::CannotOpenStdin => {
                formatter.write_str("could not open the configured process input")
            }
            Self::CannotOpenStdout => {
                formatter.write_str("could not open the configured process output")
            }
            Self::ProcessInputClosed { phase } => write!(
                formatter,
                "process input closed before the MCP {} could be sent",
                phase.description()
            ),
            Self::CannotWrite { phase, source } => write!(
                formatter,
                "could not send the MCP {}: {source}",
                phase.description()
            ),
            Self::ResponseTimedOut { timeout } => write!(
                formatter,
                "the process did not return an initialize response within {}; it was terminated",
                display_duration(*timeout)
            ),
            Self::ResponseClosed => formatter
                .write_str("the process closed stdout before returning an initialize response"),
            Self::ResponseTooLarge { maximum } => write!(
                formatter,
                "the initialize response exceeded the {maximum}-byte safety limit"
            ),
            Self::ResponseMissingDelimiter => formatter.write_str(
                "the initialize response was not terminated by the required newline delimiter",
            ),
            Self::ResponseRead { source } => {
                write!(
                    formatter,
                    "could not read the initialize response: {source}"
                )
            }
            Self::ResponseReaderStopped => {
                formatter.write_str("the initialize response reader stopped unexpectedly")
            }
            Self::ResponseReaderPanicked => {
                formatter.write_str("the initialize response reader failed unexpectedly")
            }
            Self::InvalidResponse(violation) => write!(
                formatter,
                "the process returned an invalid initialize response: {violation}"
            ),
            Self::InitializeRejected => {
                formatter.write_str("the process rejected the initialize request")
            }
            Self::ProcessLost => formatter.write_str("the configured process lifecycle was lost"),
            Self::CannotWaitForShutdown { source } => {
                write!(formatter, "could not observe process shutdown: {source}")
            }
            Self::ShutdownTimedOut { timeout } => write!(
                formatter,
                "the initialized process did not exit within {} after stdin closed; it was terminated",
                display_duration(*timeout)
            ),
            Self::UnsuccessfulShutdown => formatter
                .write_str("the initialized process exited unsuccessfully after stdin closed"),
            Self::CleanupFailed { failure, cleanup } => write!(
                formatter,
                "{failure}; additionally, process cleanup failed: {cleanup}"
            ),
        }
    }
}

impl Error for InitializeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CannotStart { source }
            | Self::CannotWrite { source, .. }
            | Self::ResponseRead { source }
            | Self::CannotWaitForShutdown { source } => Some(source),
            Self::InvalidResponse(violation) => Some(violation),
            Self::CleanupFailed { failure, .. } => Some(failure),
            _ => None,
        }
    }
}

fn display_duration(duration: Duration) -> String {
    if duration.subsec_nanos() == 0 {
        format!("{} seconds", duration.as_secs())
    } else {
        format!("{} milliseconds", duration.as_millis())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResponseViolation {
    MalformedJson,
    RootMustBeObject,
    InvalidJsonRpcVersion,
    MismatchedIdentifier,
    ResultAndError,
    MissingResultOrError,
    MalformedError,
    ResultMustBeObject,
    UnsupportedProtocolVersion,
    CapabilitiesMustBeObject,
    ServerInfoMustBeObject,
    InvalidServerInfo,
}

impl fmt::Display for ResponseViolation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::MalformedJson => "stdout was not one duplicate-free JSON-RPC message",
            Self::RootMustBeObject => "the JSON-RPC message must be an object",
            Self::InvalidJsonRpcVersion => "field `jsonrpc` must be `2.0`",
            Self::MismatchedIdentifier => "the response identifier did not match the request",
            Self::ResultAndError => "the response contained both `result` and `error`",
            Self::MissingResultOrError => "the response contained neither `result` nor `error`",
            Self::MalformedError => "field `error` was not a valid JSON-RPC error object",
            Self::ResultMustBeObject => "field `result` must be an object",
            Self::UnsupportedProtocolVersion => {
                "field `protocolVersion` was missing, invalid, or unsupported"
            }
            Self::CapabilitiesMustBeObject => "field `capabilities` must be an object",
            Self::ServerInfoMustBeObject => "field `serverInfo` must be an object",
            Self::InvalidServerInfo => {
                "field `serverInfo` must contain string `name` and `version` fields"
            }
        })
    }
}

impl Error for ResponseViolation {}

#[derive(Debug)]
pub enum CleanupError {
    Inspect(io::Error),
    Kill(io::Error),
    Wait(io::Error),
    ReaderPanicked,
    ReaderTimedOut,
}

impl fmt::Display for CleanupError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Inspect(source) => write!(formatter, "could not inspect the child: {source}"),
            Self::Kill(source) => write!(formatter, "could not terminate the child: {source}"),
            Self::Wait(source) => write!(formatter, "could not reap the child: {source}"),
            Self::ReaderPanicked => formatter.write_str("the stdout reader failed during cleanup"),
            Self::ReaderTimedOut => {
                formatter.write_str("the stdout reader did not stop within the cleanup bound")
            }
        }
    }
}

impl Error for CleanupError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Inspect(source) | Self::Kill(source) | Self::Wait(source) => Some(source),
            Self::ReaderPanicked | Self::ReaderTimedOut => None,
        }
    }
}

enum ResponseReadError {
    Closed,
    TooLarge { maximum: usize },
    MissingDelimiter,
    Io(io::Error),
}

impl From<ResponseReadError> for InitializeError {
    fn from(error: ResponseReadError) -> Self {
        match error {
            ResponseReadError::Closed => Self::ResponseClosed,
            ResponseReadError::TooLarge { maximum } => Self::ResponseTooLarge { maximum },
            ResponseReadError::MissingDelimiter => Self::ResponseMissingDelimiter,
            ResponseReadError::Io(source) => Self::ResponseRead { source },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CURRENT_HANDSHAKE_PROTOCOL_VERSION, HealthLimits, InitializeError, InitializeTester,
        NegotiatedProtocol, ResponseViolation, initialize_request, run_initialize, test_server,
        validate_initialize_response,
    };
    use crate::config::CanonicalServer;
    use crate::filesystem::OsFileSystem;
    use crate::paths::{Environment, MacOsConfigurationPaths};
    use serde_json::Value;
    use std::cell::Cell;
    use std::collections::BTreeMap;
    use std::ffi::OsString;
    use std::fs;
    use std::path::Path;
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};

    const SHORT_LIMITS: HealthLimits = HealthLimits {
        response_timeout: Duration::from_millis(200),
        shutdown_timeout: Duration::from_millis(200),
        maximum_response_bytes: 64 * 1024,
    };

    struct FixtureEnvironment {
        home: OsString,
    }

    impl Environment for FixtureEnvironment {
        fn value(&self, name: &'static str) -> Option<OsString> {
            (name == "HOME").then(|| self.home.clone())
        }
    }

    struct RecordingTester {
        called: Cell<bool>,
    }

    impl InitializeTester for RecordingTester {
        fn test_initialize(
            &self,
            server: &CanonicalServer,
        ) -> Result<NegotiatedProtocol, InitializeError> {
            self.called.set(true);
            assert_eq!(server.command(), "synthetic-private-command");
            assert_eq!(server.args(), ["synthetic-private-argument"]);
            assert_eq!(
                server.env().get("TOKEN").map(String::as_str),
                Some("synthetic-private-value")
            );
            Ok(NegotiatedProtocol::V2025_11_25)
        }
    }

    fn fixture_paths(root: &Path) -> MacOsConfigurationPaths {
        MacOsConfigurationPaths::resolve(&FixtureEnvironment {
            home: root.as_os_str().to_owned(),
        })
        .expect("disposable paths should resolve")
    }

    fn canonical_document() -> &'static str {
        r#"{
  "schemaVersion": 1,
  "servers": {
    "alpha": {
      "command": "synthetic-private-command",
      "args": ["synthetic-private-argument"],
      "env": {"TOKEN": "synthetic-private-value"}
    }
  }
}
"#
    }

    #[test]
    fn application_selects_one_server_and_reports_only_safe_protocol_metadata() {
        let root = tempfile::tempdir().expect("temporary health root should be created");
        let paths = fixture_paths(root.path());
        let canonical = paths.canonical_configuration();
        fs::create_dir_all(
            canonical
                .parent()
                .expect("canonical path should have a parent"),
        )
        .expect("canonical parent should be created");
        fs::write(canonical, canonical_document()).expect("canonical fixture should be written");
        let tester = RecordingTester {
            called: Cell::new(false),
        };

        let report = test_server(&paths, &OsFileSystem, &tester, "alpha")
            .expect("the selected server should pass");

        assert!(tester.called.get());
        let output = report.to_string();
        assert_eq!(
            output,
            "Server \"alpha\" initialized successfully via STDIO (protocol version 2025-11-25)."
        );
        for private in [
            "synthetic-private-command",
            "synthetic-private-argument",
            "synthetic-private-value",
        ] {
            assert!(!output.contains(private));
            assert!(!format!("{report:?}").contains(private));
        }
    }

    #[test]
    fn unknown_name_never_reaches_the_process_boundary() {
        let root = tempfile::tempdir().expect("temporary health root should be created");
        let paths = fixture_paths(root.path());
        let canonical = paths.canonical_configuration();
        fs::create_dir_all(
            canonical
                .parent()
                .expect("canonical path should have a parent"),
        )
        .expect("canonical parent should be created");
        fs::write(canonical, canonical_document()).expect("canonical fixture should be written");
        let tester = RecordingTester {
            called: Cell::new(false),
        };

        let error = test_server(&paths, &OsFileSystem, &tester, "missing")
            .expect_err("an unknown server should fail before execution");

        assert!(!tester.called.get());
        assert_eq!(
            error.to_string(),
            "server \"missing\" is not present in canonical configuration; run `mcp-sync list` to review configured names"
        );
    }

    #[test]
    fn initialize_request_and_notification_follow_the_handshake_contract() {
        let request = initialize_request().expect("the fixed request should serialize");
        assert_eq!(request.last(), Some(&b'\n'));
        let request: Value = serde_json::from_slice(&request).expect("request should be JSON");
        assert_eq!(request["jsonrpc"], "2.0");
        assert_eq!(request["id"], 1);
        assert_eq!(request["method"], "initialize");
        assert_eq!(
            request["params"]["protocolVersion"],
            CURRENT_HANDSHAKE_PROTOCOL_VERSION
        );
        assert_eq!(request["params"]["capabilities"], serde_json::json!({}));
        assert_eq!(request["params"]["clientInfo"]["name"], "mcp-sync");
        assert_eq!(
            request["params"]["clientInfo"]["version"],
            env!("CARGO_PKG_VERSION")
        );

        let notification: Value = serde_json::from_slice(super::INITIALIZED_NOTIFICATION)
            .expect("notification should be JSON");
        assert_eq!(notification["jsonrpc"], "2.0");
        assert_eq!(notification["method"], "notifications/initialized");
        assert!(notification.get("id").is_none());
    }

    #[test]
    fn every_supported_handshake_revision_is_negotiated_to_a_safe_enum() {
        let cases = [
            ("2025-11-25", NegotiatedProtocol::V2025_11_25),
            ("2025-06-18", NegotiatedProtocol::V2025_06_18),
            ("2025-03-26", NegotiatedProtocol::V2025_03_26),
            ("2024-11-05", NegotiatedProtocol::V2024_11_05),
        ];
        for (wire_version, expected) in cases {
            let response = format!(
                "{{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{{\"protocolVersion\":\"{wire_version}\",\"capabilities\":{{}},\"serverInfo\":{{\"name\":\"fixture\",\"version\":\"1.0\"}},\"futureField\":true}}}}\n"
            );
            assert_eq!(
                validate_initialize_response(response.as_bytes())
                    .expect("a supported response should validate"),
                expected
            );
        }
    }

    #[test]
    fn malformed_and_structurally_invalid_protocol_responses_are_rejected() {
        let cases = [
            (
                b"not-json\n".as_slice(),
                ResponseViolation::MalformedJson,
            ),
            (b"[]\n".as_slice(), ResponseViolation::RootMustBeObject),
            (
                br#"{"jsonrpc":"1.0","id":1,"result":{}}
"#,
                ResponseViolation::InvalidJsonRpcVersion,
            ),
            (
                br#"{"jsonrpc":"2.0","id":2,"result":{}}
"#,
                ResponseViolation::MismatchedIdentifier,
            ),
            (
                br#"{"jsonrpc":"2.0","id":1,"result":{},"error":{"code":-1,"message":"x"}}
"#,
                ResponseViolation::ResultAndError,
            ),
            (
                br#"{"jsonrpc":"2.0","id":1}
"#,
                ResponseViolation::MissingResultOrError,
            ),
            (
                br#"{"jsonrpc":"2.0","id":1,"error":{"message":"x"}}
"#,
                ResponseViolation::MalformedError,
            ),
            (
                br#"{"jsonrpc":"2.0","id":1,"result":[]}
"#,
                ResponseViolation::ResultMustBeObject,
            ),
            (
                br#"{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"private-future-value","capabilities":{},"serverInfo":{"name":"x","version":"1"}}}
"#,
                ResponseViolation::UnsupportedProtocolVersion,
            ),
            (
                br#"{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2025-11-25","capabilities":[],"serverInfo":{"name":"x","version":"1"}}}
"#,
                ResponseViolation::CapabilitiesMustBeObject,
            ),
            (
                br#"{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2025-11-25","capabilities":{},"serverInfo":[]}}
"#,
                ResponseViolation::ServerInfoMustBeObject,
            ),
            (
                br#"{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2025-11-25","capabilities":{},"serverInfo":{"name":"x"}}}
"#,
                ResponseViolation::InvalidServerInfo,
            ),
            (
                br#"{"jsonrpc":"2.0","jsonrpc":"2.0","id":1,"result":{}}
"#,
                ResponseViolation::MalformedJson,
            ),
        ];

        for (response, expected) in cases {
            let error = validate_initialize_response(response)
                .expect_err("the invalid response should be rejected");
            assert!(
                matches!(error, InitializeError::InvalidResponse(actual) if actual == expected),
                "unexpected structural failure: {error}"
            );
        }
    }

    #[test]
    fn valid_error_response_is_redacted_in_display_and_debug_output() {
        let private = "synthetic-private-error-detail";
        let response = format!(
            "{{\"jsonrpc\":\"2.0\",\"id\":1,\"error\":{{\"code\":-32603,\"message\":\"{private}\",\"data\":{{\"token\":\"{private}\"}}}}}}\n"
        );

        let error = validate_initialize_response(response.as_bytes())
            .expect_err("a server error should fail the health test");

        assert!(matches!(error, InitializeError::InitializeRejected));
        assert!(!error.to_string().contains(private));
        assert!(!format!("{error:?}").contains(private));
    }

    #[cfg(unix)]
    fn shell_server(script: &str, environment: BTreeMap<String, String>) -> CanonicalServer {
        CanonicalServer::new(
            "/bin/sh",
            vec!["-c".to_owned(), script.to_owned()],
            environment,
        )
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

    #[cfg(unix)]
    #[test]
    fn operating_system_boundary_sequences_handshake_and_minimizes_environment() {
        let root = tempfile::tempdir().expect("temporary process root should be created");
        let request_path = root.path().join("initialize.json");
        let notification_path = root.path().join("initialized.json");
        let environment = BTreeMap::from([
            (
                "REQUEST_PATH".to_owned(),
                request_path.to_string_lossy().into_owned(),
            ),
            (
                "NOTIFICATION_PATH".to_owned(),
                notification_path.to_string_lossy().into_owned(),
            ),
            ("SYNTHETIC_TOKEN".to_owned(), "private-value".to_owned()),
        ]);
        let server = shell_server(
            r#"
if [ "${HOME+x}" = x ]; then exit 70; fi
if [ "$SYNTHETIC_TOKEN" != "private-value" ]; then exit 71; fi
IFS= read -r initialize || exit 72
printf '%s' "$initialize" > "$REQUEST_PATH"
printf '%s\n' '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2025-11-25","capabilities":{"tools":{}},"serverInfo":{"name":"fixture","version":"1.0"}}}'
IFS= read -r initialized || exit 73
printf '%s' "$initialized" > "$NOTIFICATION_PATH"
exit 0
"#,
            environment,
        );

        let protocol = run_initialize(&server, SHORT_LIMITS)
            .expect("the operating-system handshake should succeed");

        assert_eq!(protocol, NegotiatedProtocol::V2025_11_25);
        let request: Value = serde_json::from_slice(
            &fs::read(request_path).expect("server should capture initialize request"),
        )
        .expect("captured request should be JSON");
        assert_eq!(request["method"], "initialize");
        assert_eq!(request["params"]["protocolVersion"], "2025-11-25");
        let notification: Value = serde_json::from_slice(
            &fs::read(notification_path).expect("server should capture initialized notification"),
        )
        .expect("captured notification should be JSON");
        assert_eq!(notification["method"], "notifications/initialized");
    }

    #[cfg(unix)]
    #[test]
    fn response_timeout_force_terminates_and_reaps_the_child() {
        let root = tempfile::tempdir().expect("temporary process root should be created");
        let pid_path = root.path().join("pid");
        let server = shell_server(
            r#"
printf '%s' "$$" > "$PID_PATH"
while :; do :; done
"#,
            BTreeMap::from([(
                "PID_PATH".to_owned(),
                pid_path.to_string_lossy().into_owned(),
            )]),
        );
        let limits = HealthLimits {
            response_timeout: Duration::from_millis(100),
            shutdown_timeout: Duration::from_millis(200),
            maximum_response_bytes: 1024,
        };
        let started = Instant::now();

        let error = run_initialize(&server, limits)
            .expect_err("a silent process should time out and be terminated");

        assert!(matches!(error, InitializeError::ResponseTimedOut { .. }));
        assert!(started.elapsed() < Duration::from_secs(1));
        let pid = fs::read_to_string(pid_path).expect("server should publish its pid");
        assert!(!process_exists(&pid), "the timed-out child must be reaped");
    }

    #[cfg(unix)]
    #[test]
    fn malformed_output_is_redacted_and_force_cleans_the_child() {
        let root = tempfile::tempdir().expect("temporary process root should be created");
        let pid_path = root.path().join("pid");
        let private = "synthetic-private-stdout";
        let server = shell_server(
            r#"
printf '%s' "$$" > "$PID_PATH"
printf 'not-json-%s\n' "$PRIVATE_VALUE"
while :; do :; done
"#,
            BTreeMap::from([
                (
                    "PID_PATH".to_owned(),
                    pid_path.to_string_lossy().into_owned(),
                ),
                ("PRIVATE_VALUE".to_owned(), private.to_owned()),
            ]),
        );

        let error = run_initialize(&server, SHORT_LIMITS)
            .expect_err("malformed protocol output should fail and terminate the child");

        assert!(matches!(
            error,
            InitializeError::InvalidResponse(ResponseViolation::MalformedJson)
        ));
        assert!(!error.to_string().contains(private));
        assert!(!format!("{error:?}").contains(private));
        let pid = fs::read_to_string(pid_path).expect("server should publish its pid");
        assert!(!process_exists(&pid), "the malformed child must be reaped");
    }

    #[cfg(unix)]
    #[test]
    fn initialized_process_that_refuses_shutdown_is_force_cleaned_and_fails() {
        let root = tempfile::tempdir().expect("temporary process root should be created");
        let pid_path = root.path().join("pid");
        let server = shell_server(
            r#"
printf '%s' "$$" > "$PID_PATH"
IFS= read -r initialize || exit 80
printf '%s\n' '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2025-11-25","capabilities":{},"serverInfo":{"name":"fixture","version":"1.0"}}}'
IFS= read -r initialized || exit 81
while :; do :; done
"#,
            BTreeMap::from([(
                "PID_PATH".to_owned(),
                pid_path.to_string_lossy().into_owned(),
            )]),
        );
        let limits = HealthLimits {
            response_timeout: Duration::from_millis(200),
            shutdown_timeout: Duration::from_millis(100),
            maximum_response_bytes: 1024,
        };

        let error = run_initialize(&server, limits)
            .expect_err("a process that ignores stdin closure should fail bounded shutdown");

        assert!(matches!(error, InitializeError::ShutdownTimedOut { .. }));
        let pid = fs::read_to_string(pid_path).expect("server should publish its pid");
        assert!(
            !process_exists(&pid),
            "the shutdown-resistant child must be reaped"
        );
    }

    #[cfg(unix)]
    #[test]
    fn oversized_and_undelimited_messages_fail_without_unbounded_reads() {
        let oversized_payload = "x".repeat(129);
        let oversized_script = format!("printf '%s\\n' '{oversized_payload}'");
        let oversized = shell_server(&oversized_script, BTreeMap::<String, String>::new());
        let small_limit = HealthLimits {
            response_timeout: Duration::from_millis(200),
            shutdown_timeout: Duration::from_millis(200),
            maximum_response_bytes: 128,
        };
        let oversized_error = run_initialize(&oversized, small_limit)
            .expect_err("a response one byte above the limit should fail");
        assert!(
            matches!(
                oversized_error,
                InitializeError::ResponseTooLarge { maximum: 128 }
            ),
            "unexpected oversized-response failure: {oversized_error}"
        );

        let undelimited = shell_server(
            "printf '%s' '{\"jsonrpc\":\"2.0\"}'",
            BTreeMap::<String, String>::new(),
        );
        assert!(matches!(
            run_initialize(&undelimited, SHORT_LIMITS),
            Err(InitializeError::ResponseMissingDelimiter)
        ));
    }
}
