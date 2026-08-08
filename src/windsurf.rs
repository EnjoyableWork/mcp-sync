use crate::config::{CanonicalConfig, CanonicalServer, ConfigError, parse_unique_json_value};
use crate::filesystem::{FileIoError, FileSystem};
use crate::paths::ConfigurationPaths;
use crate::reconciliation::{ReconciliationOutcomeKind, ReconciliationPlan};
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

const CODEIUM_DIRECTORY: &str = ".codeium";
const WINDSURF_DIRECTORY: &str = "windsurf";
const WINDSURF_CONFIGURATION_FILE: &str = "mcp_config.json";
const MCP_SERVERS_FIELD: &str = "mcpServers";
const COMMAND_FIELD: &str = "command";
const ARGUMENTS_FIELD: &str = "args";
const ENVIRONMENT_FIELD: &str = "env";

/// The documented Windsurf global Cascade target on macOS, Linux, and Windows.
///
/// Discovery resolves exactly `~/.codeium/windsurf/mcp_config.json` through
/// the injected home path. Current vendor documentation distinguishes this
/// legacy Cascade configuration from the separate Devin Local agent
/// configuration. This adapter intentionally owns only the former. Discovery
/// is read-only; a missing file is a normal state and every other filesystem
/// failure remains contextual.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WindsurfAdapter {
    configuration_path: PathBuf,
}

impl WindsurfAdapter {
    pub fn from_paths(paths: &ConfigurationPaths) -> Self {
        Self {
            configuration_path: paths
                .user_home()
                .join(CODEIUM_DIRECTORY)
                .join(WINDSURF_DIRECTORY)
                .join(WINDSURF_CONFIGURATION_FILE),
        }
    }

    pub fn configuration_path(&self) -> &Path {
        &self.configuration_path
    }

    pub fn discover(
        &self,
        filesystem: &impl FileSystem,
    ) -> Result<WindsurfDiscovery, WindsurfAdapterError> {
        match filesystem.read(&self.configuration_path) {
            Ok(bytes) => WindsurfDocument::parse(&bytes).map(WindsurfDiscovery::Found),
            Err(source) if source.kind() == io::ErrorKind::NotFound => {
                Ok(WindsurfDiscovery::Missing)
            }
            Err(source) => Err(WindsurfAdapterError::Read { source }),
        }
    }
}

#[derive(Debug)]
pub enum WindsurfDiscovery {
    Missing,
    Found(WindsurfDocument),
}

impl WindsurfDiscovery {
    #[cfg(test)]
    pub fn document(&self) -> Option<&WindsurfDocument> {
        match self {
            Self::Missing => None,
            Self::Found(document) => Some(document),
        }
    }

    pub fn into_document(self) -> WindsurfDocument {
        match self {
            Self::Missing => WindsurfDocument::empty(),
            Self::Found(document) => document,
        }
    }
}

/// A validated Windsurf global document plus its exact original bytes.
///
/// Canonical v1 models only command-based local STDIO servers. For those
/// entries the adapter owns `mcpServers.<name>.command`, `args`, and `env`.
/// Windsurf fields such as `disabledTools`, every unknown top-level or
/// per-server field, target-only local servers, and commandless remote
/// `serverUrl` or `url` entries remain native data. Commandless entries are
/// reported as unmanaged and are never rewritten into local entries by a
/// colliding add plan.
pub struct WindsurfDocument {
    root: Map<String, Value>,
    canonical_config: CanonicalConfig,
    unmanaged_server_names: BTreeSet<String>,
    original_bytes: Vec<u8>,
}

impl WindsurfDocument {
    pub fn parse(document: &[u8]) -> Result<Self, WindsurfAdapterError> {
        let value = parse_unique_json_value(document).map_err(|error| {
            WindsurfAdapterError::InvalidJson {
                message: error.to_string(),
            }
        })?;
        let Value::Object(root) = value else {
            return Err(WindsurfDocumentError::RootMustBeObject.into());
        };

        let decoded = decode_servers(root.get(MCP_SERVERS_FIELD))?;
        let canonical_config = CanonicalConfig::new(decoded.local_servers)
            .map_err(|source| WindsurfAdapterError::InvalidCanonical { source })?;

        Ok(Self {
            root,
            canonical_config,
            unmanaged_server_names: decoded.unmanaged_server_names,
            original_bytes: document.to_vec(),
        })
    }

    fn empty() -> Self {
        Self {
            root: Map::new(),
            canonical_config: CanonicalConfig::new(BTreeMap::new())
                .expect("an empty canonical server map is valid"),
            unmanaged_server_names: BTreeSet::new(),
            original_bytes: b"{}\n".to_vec(),
        }
    }

    pub fn canonical_config(&self) -> &CanonicalConfig {
        &self.canonical_config
    }

    /// Exact bytes observed during discovery for guarded sync application.
    pub fn original_bytes(&self) -> &[u8] {
        &self.original_bytes
    }

    /// Names of native entries not representable by canonical JSON v1.
    ///
    /// Windsurf places remote HTTP/SSE definitions in the same `mcpServers`
    /// map as command-based servers. These names let application use cases
    /// report unsupported native capabilities without exposing URLs, headers,
    /// authentication material, or other values.
    pub fn unmanaged_server_names(&self) -> Vec<&str> {
        self.unmanaged_server_names
            .iter()
            .map(String::as_str)
            .collect()
    }

    pub fn preserved_root_fields(&self) -> Vec<&str> {
        self.root
            .keys()
            .filter(|field| field.as_str() != MCP_SERVERS_FIELD)
            .map(String::as_str)
            .collect()
    }

    #[cfg(test)]
    pub fn preserved_server_fields(&self, server_name: &str) -> Option<Vec<&str>> {
        let fields = self
            .root
            .get(MCP_SERVERS_FIELD)?
            .as_object()?
            .get(server_name)?
            .as_object()?;
        let is_managed_local = self.canonical_config.servers().contains_key(server_name);

        Some(
            fields
                .keys()
                .filter(|field| {
                    !is_managed_local
                        || !matches!(
                            field.as_str(),
                            COMMAND_FIELD | ARGUMENTS_FIELD | ENVIRONMENT_FIELD
                        )
                })
                .map(String::as_str)
                .collect(),
        )
    }

    pub fn render_plan(
        &self,
        plan: &ReconciliationPlan,
    ) -> Result<RenderedWindsurfDocument, WindsurfAdapterError> {
        self.validate_plan(plan)?;
        if !plan.requires_mutation() {
            return Ok(RenderedWindsurfDocument::new(
                self.original_bytes.clone(),
                false,
            ));
        }

        let mut root = self.root.clone();
        let servers = root
            .entry(MCP_SERVERS_FIELD.to_owned())
            .or_insert_with(|| Value::Object(Map::new()))
            .as_object_mut()
            .ok_or(WindsurfDocumentError::McpServersMustBeObject)?;

        for entry in plan.entries() {
            if !matches!(
                entry.outcome().kind(),
                ReconciliationOutcomeKind::Add | ReconciliationOutcomeKind::Update
            ) {
                continue;
            }

            let desired = entry.desired_server().ok_or_else(|| {
                WindsurfAdapterError::PlanMissingDesiredServer {
                    server: entry.server_name().to_owned(),
                }
            })?;
            let native_server = servers
                .entry(entry.server_name().to_owned())
                .or_insert_with(|| Value::Object(Map::new()))
                .as_object_mut()
                .ok_or_else(|| WindsurfDocumentError::ServerMustBeObject {
                    server: entry.server_name().to_owned(),
                })?;

            write_managed_fields(native_server, desired);
        }

        let mut bytes = serde_json::to_vec_pretty(&Value::Object(root)).map_err(|error| {
            WindsurfAdapterError::Serialization {
                message: error.to_string(),
            }
        })?;
        bytes.push(b'\n');

        Self::parse(&bytes)?;
        let changed = bytes != self.original_bytes;
        Ok(RenderedWindsurfDocument::new(bytes, changed))
    }

    fn validate_plan(&self, plan: &ReconciliationPlan) -> Result<(), WindsurfAdapterError> {
        let mut planned_names = BTreeSet::new();

        for entry in plan.entries() {
            planned_names.insert(entry.server_name());

            if self.unmanaged_server_names.contains(entry.server_name()) {
                return Err(WindsurfAdapterError::UnmanagedServerCollision {
                    server: entry.server_name().to_owned(),
                });
            }

            let present = self
                .canonical_config
                .servers()
                .contains_key(entry.server_name());
            let should_be_present = entry.outcome().kind() != ReconciliationOutcomeKind::Add;

            if present != should_be_present {
                return Err(WindsurfAdapterError::PlanPresenceMismatch {
                    server: entry.server_name().to_owned(),
                    outcome: entry.outcome().kind(),
                    present,
                });
            }

            if matches!(
                entry.outcome().kind(),
                ReconciliationOutcomeKind::Add | ReconciliationOutcomeKind::Update
            ) && entry.desired_server().is_none()
            {
                return Err(WindsurfAdapterError::PlanMissingDesiredServer {
                    server: entry.server_name().to_owned(),
                });
            }
        }

        if let Some(server) = self
            .canonical_config
            .servers()
            .keys()
            .find(|server| !planned_names.contains(server.as_str()))
        {
            return Err(WindsurfAdapterError::PlanOmittedCurrentServer {
                server: server.clone(),
            });
        }

        Ok(())
    }
}

impl fmt::Debug for WindsurfDocument {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WindsurfDocument")
            .field(
                "local_server_names",
                &self.canonical_config.servers().keys().collect::<Vec<_>>(),
            )
            .field("unmanaged_server_count", &self.unmanaged_server_names.len())
            .field("preserved_root_fields", &self.preserved_root_fields())
            .field("byte_count", &self.original_bytes.len())
            .finish()
    }
}

pub struct RenderedWindsurfDocument {
    bytes: Vec<u8>,
    changed: bool,
}

impl RenderedWindsurfDocument {
    fn new(bytes: Vec<u8>, changed: bool) -> Self {
        Self { bytes, changed }
    }

    #[cfg(test)]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    pub fn changed(&self) -> bool {
        self.changed
    }
}

impl fmt::Debug for RenderedWindsurfDocument {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RenderedWindsurfDocument")
            .field("byte_count", &self.bytes.len())
            .field("changed", &self.changed)
            .finish()
    }
}

struct DecodedServers {
    local_servers: BTreeMap<String, CanonicalServer>,
    unmanaged_server_names: BTreeSet<String>,
}

fn decode_servers(value: Option<&Value>) -> Result<DecodedServers, WindsurfAdapterError> {
    let Some(value) = value else {
        return Ok(DecodedServers {
            local_servers: BTreeMap::new(),
            unmanaged_server_names: BTreeSet::new(),
        });
    };
    let Value::Object(servers) = value else {
        return Err(WindsurfDocumentError::McpServersMustBeObject.into());
    };

    let mut local_servers = BTreeMap::new();
    let mut unmanaged_server_names = BTreeSet::new();

    for (name, value) in servers {
        let Value::Object(fields) = value else {
            return Err(WindsurfDocumentError::ServerMustBeObject {
                server: name.clone(),
            }
            .into());
        };

        if fields.contains_key(COMMAND_FIELD) {
            local_servers.insert(name.clone(), decode_local_server(name, fields)?);
        } else {
            unmanaged_server_names.insert(name.clone());
        }
    }

    Ok(DecodedServers {
        local_servers,
        unmanaged_server_names,
    })
}

fn decode_local_server(
    name: &str,
    fields: &Map<String, Value>,
) -> Result<CanonicalServer, WindsurfAdapterError> {
    let command = fields
        .get(COMMAND_FIELD)
        .expect("the caller classifies local entries by command presence");
    let Value::String(command) = command else {
        return Err(WindsurfDocumentError::CommandMustBeString {
            server: name.to_owned(),
        }
        .into());
    };

    let arguments = decode_arguments(name, fields.get(ARGUMENTS_FIELD))?;
    let environment = decode_environment(name, fields.get(ENVIRONMENT_FIELD))?;
    Ok(CanonicalServer::new(
        command.clone(),
        arguments,
        environment,
    ))
}

fn decode_arguments(
    server: &str,
    value: Option<&Value>,
) -> Result<Vec<String>, WindsurfAdapterError> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let Value::Array(arguments) = value else {
        return Err(WindsurfDocumentError::ArgumentsMustBeArray {
            server: server.to_owned(),
        }
        .into());
    };

    arguments
        .iter()
        .enumerate()
        .map(|(index, argument)| match argument {
            Value::String(argument) => Ok(argument.clone()),
            _ => Err(WindsurfDocumentError::ArgumentMustBeString {
                server: server.to_owned(),
                index,
            }
            .into()),
        })
        .collect()
}

fn decode_environment(
    server: &str,
    value: Option<&Value>,
) -> Result<BTreeMap<String, String>, WindsurfAdapterError> {
    let Some(value) = value else {
        return Ok(BTreeMap::new());
    };
    let Value::Object(environment) = value else {
        return Err(WindsurfDocumentError::EnvironmentMustBeObject {
            server: server.to_owned(),
        }
        .into());
    };

    environment
        .iter()
        .enumerate()
        .map(|(position, (key, value))| match value {
            Value::String(value) => Ok((key.clone(), value.clone())),
            _ => Err(WindsurfDocumentError::EnvironmentValueMustBeString {
                server: server.to_owned(),
                position,
            }
            .into()),
        })
        .collect()
}

fn write_managed_fields(fields: &mut Map<String, Value>, server: &CanonicalServer) {
    fields.insert(
        COMMAND_FIELD.to_owned(),
        Value::String(server.command().to_owned()),
    );
    fields.insert(
        ARGUMENTS_FIELD.to_owned(),
        Value::Array(server.args().iter().cloned().map(Value::String).collect()),
    );
    fields.insert(
        ENVIRONMENT_FIELD.to_owned(),
        Value::Object(
            server
                .env()
                .iter()
                .map(|(key, value)| (key.clone(), Value::String(value.clone())))
                .collect(),
        ),
    );
}

#[derive(Debug)]
pub enum WindsurfAdapterError {
    Read {
        source: FileIoError,
    },
    InvalidJson {
        message: String,
    },
    InvalidDocument(WindsurfDocumentError),
    InvalidCanonical {
        source: ConfigError,
    },
    Serialization {
        message: String,
    },
    UnmanagedServerCollision {
        server: String,
    },
    PlanPresenceMismatch {
        server: String,
        outcome: ReconciliationOutcomeKind,
        present: bool,
    },
    PlanOmittedCurrentServer {
        server: String,
    },
    PlanMissingDesiredServer {
        server: String,
    },
}

impl fmt::Display for WindsurfAdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { source } => source.fmt(formatter),
            Self::InvalidJson { message } => write!(formatter, "invalid Windsurf JSON: {message}"),
            Self::InvalidDocument(error) => error.fmt(formatter),
            Self::InvalidCanonical { source } => {
                write!(
                    formatter,
                    "invalid Windsurf MCP server definition: {source}"
                )
            }
            Self::Serialization { message } => {
                write!(
                    formatter,
                    "cannot serialize Windsurf configuration: {message}"
                )
            }
            Self::UnmanagedServerCollision { server } => write!(
                formatter,
                "desired local server {server:?} collides with an unmanaged Windsurf server"
            ),
            Self::PlanPresenceMismatch {
                server,
                outcome,
                present,
            } => write!(
                formatter,
                "reconciliation plan outcome {outcome:?} for server {server:?} does not match Windsurf document presence ({present})"
            ),
            Self::PlanOmittedCurrentServer { server } => write!(
                formatter,
                "reconciliation plan omits current Windsurf server {server:?}"
            ),
            Self::PlanMissingDesiredServer { server } => write!(
                formatter,
                "reconciliation plan has no desired definition for Windsurf server {server:?}"
            ),
        }
    }
}

impl Error for WindsurfAdapterError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Read { source } => Some(source),
            Self::InvalidDocument(error) => Some(error),
            Self::InvalidCanonical { source } => Some(source),
            Self::InvalidJson { .. }
            | Self::Serialization { .. }
            | Self::UnmanagedServerCollision { .. }
            | Self::PlanPresenceMismatch { .. }
            | Self::PlanOmittedCurrentServer { .. }
            | Self::PlanMissingDesiredServer { .. } => None,
        }
    }
}

impl From<WindsurfDocumentError> for WindsurfAdapterError {
    fn from(error: WindsurfDocumentError) -> Self {
        Self::InvalidDocument(error)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WindsurfDocumentError {
    RootMustBeObject,
    McpServersMustBeObject,
    ServerMustBeObject { server: String },
    CommandMustBeString { server: String },
    ArgumentsMustBeArray { server: String },
    ArgumentMustBeString { server: String, index: usize },
    EnvironmentMustBeObject { server: String },
    EnvironmentValueMustBeString { server: String, position: usize },
}

impl fmt::Display for WindsurfDocumentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RootMustBeObject => {
                formatter.write_str("Windsurf configuration must be an object")
            }
            Self::McpServersMustBeObject => {
                formatter.write_str("Windsurf field `mcpServers` must be an object")
            }
            Self::ServerMustBeObject { server } => {
                write!(formatter, "Windsurf server {server:?} must be an object")
            }
            Self::CommandMustBeString { server } => write!(
                formatter,
                "Windsurf server {server:?} field `command` must be a string"
            ),
            Self::ArgumentsMustBeArray { server } => write!(
                formatter,
                "Windsurf server {server:?} field `args` must be an array"
            ),
            Self::ArgumentMustBeString { server, index } => write!(
                formatter,
                "Windsurf server {server:?} argument at index {index} must be a string"
            ),
            Self::EnvironmentMustBeObject { server } => write!(
                formatter,
                "Windsurf server {server:?} field `env` must be an object"
            ),
            Self::EnvironmentValueMustBeString { server, position } => write!(
                formatter,
                "Windsurf server {server:?} environment value at position {position} must be a string"
            ),
        }
    }
}

impl Error for WindsurfDocumentError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths::{Environment, Platform};
    use crate::reconciliation::{ReconciliationOutcomeKind, reconcile};
    use std::ffi::OsString;

    const CURRENT_FIXTURE: &[u8] = include_bytes!("../tests/fixtures/windsurf/current.json");
    const DESIRED_FIXTURE: &str = include_str!("../tests/fixtures/windsurf/desired.json");
    const MERGED_FIXTURE: &[u8] = include_bytes!("../tests/fixtures/windsurf/merged.json");

    struct FixtureEnvironment {
        home: PathBuf,
    }

    impl Environment for FixtureEnvironment {
        fn value(&self, name: &'static str) -> Option<OsString> {
            match name {
                "HOME" | "USERPROFILE" => Some(self.home.clone().into_os_string()),
                "LOCALAPPDATA" => Some(self.home.join("AppData/Local").into_os_string()),
                "APPDATA" => Some(self.home.join("AppData/Roaming").into_os_string()),
                _ => None,
            }
        }
    }

    struct FixtureFileSystem {
        expected_path: PathBuf,
        result: Result<Vec<u8>, io::ErrorKind>,
    }

    impl FileSystem for FixtureFileSystem {
        fn read(&self, path: &Path) -> Result<Vec<u8>, FileIoError> {
            assert_eq!(path, self.expected_path);
            match &self.result {
                Ok(bytes) => Ok(bytes.clone()),
                Err(kind) => Err(FileIoError::read(
                    path,
                    io::Error::new(*kind, "synthetic Windsurf read failure"),
                )),
            }
        }
    }

    fn adapter_fixture_for(platform: Platform) -> (tempfile::TempDir, WindsurfAdapter) {
        let root = tempfile::tempdir().expect("temporary adapter fixture should be created");
        let paths = ConfigurationPaths::resolve_for(
            platform,
            &FixtureEnvironment {
                home: root.path().join("user"),
            },
        )
        .expect("synthetic platform paths should resolve");
        let adapter = WindsurfAdapter::from_paths(&paths);
        (root, adapter)
    }

    fn adapter_fixture() -> (tempfile::TempDir, WindsurfAdapter) {
        adapter_fixture_for(Platform::MacOs)
    }

    fn desired_config() -> CanonicalConfig {
        CanonicalConfig::parse_json(DESIRED_FIXTURE)
            .expect("desired fixture should be valid canonical JSON")
    }

    #[test]
    fn macos_discovery_path_matches_the_documented_global_contract() {
        let (root, adapter) = adapter_fixture();

        assert_eq!(
            adapter.configuration_path(),
            root.path().join("user/.codeium/windsurf/mcp_config.json")
        );
        assert!(adapter.configuration_path().starts_with(root.path()));
    }

    #[test]
    fn linux_discovery_path_matches_the_documented_global_contract() {
        let (root, adapter) = adapter_fixture_for(Platform::Linux);

        assert_eq!(
            adapter.configuration_path(),
            root.path().join("user/.codeium/windsurf/mcp_config.json")
        );
        assert!(adapter.configuration_path().starts_with(root.path()));
    }

    #[test]
    fn windows_discovery_path_matches_the_documented_global_contract() {
        let (root, adapter) = adapter_fixture_for(Platform::Windows);

        assert_eq!(
            adapter.configuration_path(),
            root.path().join("user/.codeium/windsurf/mcp_config.json")
        );
        assert!(adapter.configuration_path().starts_with(root.path()));
    }

    #[test]
    fn discovery_reads_the_exact_injected_path_and_parses_the_native_fixture() {
        let (_root, adapter) = adapter_fixture();
        let filesystem = FixtureFileSystem {
            expected_path: adapter.configuration_path().to_owned(),
            result: Ok(CURRENT_FIXTURE.to_vec()),
        };

        let discovery = adapter
            .discover(&filesystem)
            .expect("fixture discovery should succeed");
        let document = discovery
            .document()
            .expect("the global fixture should be discovered");

        assert_eq!(document.canonical_config().servers().len(), 3);
        assert_eq!(document.unmanaged_server_names(), ["remote-only"]);
        assert_eq!(
            document.preserved_root_fields(),
            ["futureTopLevel", "windsurfMetadata"]
        );
        assert_eq!(
            document.preserved_server_fields("updated").unwrap(),
            ["disabledTools", "futureOption"]
        );
        assert_eq!(
            document.preserved_server_fields("remote-only").unwrap(),
            ["disabledTools", "futureRemote", "headers", "serverUrl"]
        );
    }

    #[test]
    fn a_missing_global_file_is_a_normal_empty_discovery_state() {
        let (_root, adapter) = adapter_fixture();
        let filesystem = FixtureFileSystem {
            expected_path: adapter.configuration_path().to_owned(),
            result: Err(io::ErrorKind::NotFound),
        };

        let discovery = adapter
            .discover(&filesystem)
            .expect("a missing global target should not be an error");

        assert!(discovery.document().is_none());
        let document = discovery.into_document();
        assert!(document.canonical_config().servers().is_empty());
        assert!(document.unmanaged_server_names().is_empty());
    }

    #[test]
    fn discovery_preserves_non_missing_read_failures() {
        let (_root, adapter) = adapter_fixture();
        let filesystem = FixtureFileSystem {
            expected_path: adapter.configuration_path().to_owned(),
            result: Err(io::ErrorKind::PermissionDenied),
        };

        let error = adapter
            .discover(&filesystem)
            .expect_err("permission failures should remain errors");

        let WindsurfAdapterError::Read { source } = error else {
            panic!("the adapter should preserve the filesystem error");
        };
        assert_eq!(source.kind(), io::ErrorKind::PermissionDenied);
        assert_eq!(source.path(), adapter.configuration_path());
    }

    #[test]
    fn fixture_plan_round_trips_and_preserves_every_unmanaged_field() {
        let document = WindsurfDocument::parse(CURRENT_FIXTURE)
            .expect("current Windsurf fixture should be valid");
        let desired = desired_config();
        let plan = reconcile(document.canonical_config(), &desired);

        assert_eq!(
            plan.entries()
                .iter()
                .map(|entry| (entry.server_name(), entry.outcome().kind()))
                .collect::<Vec<_>>(),
            [
                ("added", ReconciliationOutcomeKind::Add),
                ("target-only", ReconciliationOutcomeKind::Drift),
                ("unchanged", ReconciliationOutcomeKind::NoOp),
                ("updated", ReconciliationOutcomeKind::Update),
            ]
        );

        let first = document
            .render_plan(&plan)
            .expect("fixture plan should render");
        let second = document
            .render_plan(&plan)
            .expect("the same fixture plan should render deterministically");

        assert!(first.changed());
        assert_eq!(first.bytes(), second.bytes());
        assert_eq!(first.bytes(), MERGED_FIXTURE);

        let merged = WindsurfDocument::parse(first.bytes())
            .expect("rendered Windsurf fixture should parse again");
        assert_eq!(merged.unmanaged_server_names(), ["remote-only"]);
        assert_eq!(
            merged.preserved_root_fields(),
            ["futureTopLevel", "windsurfMetadata"]
        );
        assert_eq!(
            merged.preserved_server_fields("updated").unwrap(),
            ["disabledTools", "futureOption"]
        );
        assert_eq!(
            merged.preserved_server_fields("remote-only").unwrap(),
            ["disabledTools", "futureRemote", "headers", "serverUrl"]
        );
        assert_eq!(
            merged.preserved_server_fields("target-only").unwrap(),
            ["disabled"]
        );

        let settled = reconcile(merged.canonical_config(), &desired);
        assert!(!settled.requires_mutation());
        assert!(settled.has_drift());
        assert_eq!(settled.summary().no_op(), 3);
        assert_eq!(settled.summary().drift(), 1);
    }

    #[test]
    fn no_op_and_drift_rendering_preserves_original_bytes_exactly() {
        let document = WindsurfDocument::parse(CURRENT_FIXTURE)
            .expect("current Windsurf fixture should be valid");
        let mut desired_servers = document.canonical_config().servers().clone();
        desired_servers.remove("target-only");
        let desired =
            CanonicalConfig::new(desired_servers).expect("fixture servers should remain canonical");
        let plan = reconcile(document.canonical_config(), &desired);

        let rendered = document
            .render_plan(&plan)
            .expect("non-mutating work should render");

        assert!(!plan.requires_mutation());
        assert!(plan.has_drift());
        assert!(!rendered.changed());
        assert_eq!(rendered.bytes(), CURRENT_FIXTURE);
    }

    #[test]
    fn a_missing_document_can_render_a_new_global_server_map() {
        let (_root, adapter) = adapter_fixture();
        let filesystem = FixtureFileSystem {
            expected_path: adapter.configuration_path().to_owned(),
            result: Err(io::ErrorKind::NotFound),
        };
        let document = adapter
            .discover(&filesystem)
            .expect("missing discovery should succeed")
            .into_document();
        let desired = desired_config();
        let plan = reconcile(document.canonical_config(), &desired);

        let rendered = document
            .render_plan(&plan)
            .expect("additions should render into an empty Windsurf document");
        let reparsed = WindsurfDocument::parse(rendered.bytes())
            .expect("new Windsurf document should be valid");

        assert!(rendered.changed());
        assert_eq!(reparsed.canonical_config(), &desired);
        assert!(reparsed.preserved_root_fields().is_empty());
        assert!(reparsed.unmanaged_server_names().is_empty());
    }

    #[test]
    fn missing_server_map_normalizes_to_empty_without_losing_root_data() {
        let bytes = br#"{
  "futureSetting": {
    "enabled": true
  }
}
"#;
        let document =
            WindsurfDocument::parse(bytes).expect("a document without mcpServers should be valid");
        let plan = reconcile(document.canonical_config(), document.canonical_config());

        let rendered = document
            .render_plan(&plan)
            .expect("an empty plan should preserve the document");

        assert!(document.canonical_config().servers().is_empty());
        assert_eq!(document.preserved_root_fields(), ["futureSetting"]);
        assert!(!rendered.changed());
        assert_eq!(rendered.bytes(), bytes);
    }

    #[test]
    fn omitted_fields_and_native_interpolation_remain_literal_and_bounded() {
        let document = WindsurfDocument::parse(
            br#"{"mcpServers":{"minimal":{"command":"${file:~/synthetic-command}","env":{"TOKEN":"${env:SYNTHETIC_TOKEN}"},"disabledTools":["preserve"]}}}"#,
        )
        .expect("optional native fields may be omitted");
        let server = document
            .canonical_config()
            .servers()
            .get("minimal")
            .expect("minimal server should normalize");

        assert_eq!(server.command(), "${file:~/synthetic-command}");
        assert!(server.args().is_empty());
        assert_eq!(server.env().get("TOKEN").unwrap(), "${env:SYNTHETIC_TOKEN}");
        assert_eq!(
            document.preserved_server_fields("minimal").unwrap(),
            ["disabledTools"]
        );
    }

    #[test]
    fn commandless_remote_and_opaque_entries_are_preserved_as_unmanaged() {
        let bytes = br#"{
  "mcpServers": {
    "opaque": {},
    "remote-url": {
      "url": "https://windsurf.example.invalid/remote",
      "headers": {"Authorization": "Bearer fixture-unmanaged-secret"}
    },
    "remote-server-url": {
      "serverUrl": "https://windsurf.example.invalid/server-url"
    }
  }
}
"#;
        let document = WindsurfDocument::parse(bytes)
            .expect("commandless entries should remain unmanaged native data");
        let empty =
            CanonicalConfig::new(BTreeMap::new()).expect("an empty canonical config is valid");
        let rendered = document
            .render_plan(&reconcile(document.canonical_config(), &empty))
            .expect("an unmanaged-only document should render without mutation");

        assert!(document.canonical_config().servers().is_empty());
        assert_eq!(
            document.unmanaged_server_names(),
            ["opaque", "remote-server-url", "remote-url"]
        );
        assert!(!rendered.changed());
        assert_eq!(rendered.bytes(), bytes);
    }

    #[test]
    fn desired_local_server_cannot_overwrite_an_unmanaged_remote_entry() {
        let document = WindsurfDocument::parse(CURRENT_FIXTURE)
            .expect("current Windsurf fixture should be valid");
        let remote = CanonicalServer::new("/synthetic/bin/collision", Vec::new(), BTreeMap::new());
        let desired = CanonicalConfig::new(BTreeMap::from([("remote-only".to_owned(), remote)]))
            .expect("the colliding desired server should be canonical");
        let plan = reconcile(document.canonical_config(), &desired);

        let error = document
            .render_plan(&plan)
            .expect_err("a local add must not replace an unmanaged remote server");

        assert!(matches!(
            error,
            WindsurfAdapterError::UnmanagedServerCollision { ref server }
                if server == "remote-only"
        ));
    }

    #[test]
    fn malformed_json_and_invalid_utf8_are_rejected_without_echoing_values() {
        let malformed = br#"{"future":"fixture-windsurf-json-secret","mcpServers":"#;
        let invalid_utf8 = [0xff, 0xfe, 0xfd];

        for bytes in [malformed.as_slice(), invalid_utf8.as_slice()] {
            let error = WindsurfDocument::parse(bytes)
                .expect_err("invalid native JSON bytes should be rejected");
            let output = format!("{error:?}\n{error}");

            assert!(matches!(error, WindsurfAdapterError::InvalidJson { .. }));
            assert!(!output.contains("fixture-windsurf-json-secret"));
        }
    }

    #[test]
    fn invalid_native_local_shapes_have_structural_errors() {
        let cases = [
            br#"[]"#.as_slice(),
            br#"{"mcpServers":[]}"#.as_slice(),
            br#"{"mcpServers":{"fixture":[]}}"#.as_slice(),
            br#"{"mcpServers":{"fixture":{"command":7}}}"#.as_slice(),
            br#"{"mcpServers":{"fixture":{"command":"safe","args":{}}}}"#.as_slice(),
            br#"{"mcpServers":{"fixture":{"command":"safe","args":[7]}}}"#.as_slice(),
            br#"{"mcpServers":{"fixture":{"command":"safe","env":[]}}}"#.as_slice(),
            br#"{"mcpServers":{"fixture":{"command":"safe","env":{"TOKEN":7}}}}"#.as_slice(),
        ];

        for bytes in cases {
            let error = WindsurfDocument::parse(bytes).unwrap_err();
            assert!(matches!(error, WindsurfAdapterError::InvalidDocument(_)));
        }

        let error = WindsurfDocument::parse(br#"{"mcpServers":{"fixture":{"command":""}}}"#)
            .expect_err("invalid canonical process text should be rejected");
        assert!(matches!(
            error,
            WindsurfAdapterError::InvalidCanonical { .. }
        ));
    }

    #[test]
    fn duplicate_keys_are_rejected_at_every_native_object_level() {
        let cases = [
            br#"{"mcpServers":{},"mcpServers":{}}"#.as_slice(),
            br#"{"mcpServers":{"same":{"command":"safe"},"same":{"command":"safe"}}}"#
                .as_slice(),
            br#"{"mcpServers":{"fixture":{"command":"safe","command":"safe"}}}"#.as_slice(),
            br#"{"mcpServers":{"fixture":{"command":"safe","env":{"TOKEN":"one","TOKEN":"two"}}}}"#
                .as_slice(),
            br#"{"mcpServers":{"remote":{"serverUrl":"https://example.invalid","headers":{"TOKEN":"one","TOKEN":"two"}}}}"#
                .as_slice(),
        ];

        for bytes in cases {
            let error =
                WindsurfDocument::parse(bytes).expect_err("duplicate keys should be rejected");
            let WindsurfAdapterError::InvalidJson { message } = error else {
                panic!("duplicate keys should be invalid JSON");
            };
            assert!(message.contains("duplicate JSON object key"));
        }
    }

    #[test]
    fn diagnostics_and_debug_output_never_echo_native_values() {
        let document = WindsurfDocument::parse(CURRENT_FIXTURE)
            .expect("current Windsurf fixture should be valid");
        let rendered = document
            .render_plan(&reconcile(document.canonical_config(), &desired_config()))
            .expect("fixture plan should render");
        let malformed = br#"{"mcpServers":{"fixture":{"command":"safe","args":"fixture-windsurf-diagnostic-secret"}}}"#;
        let error = WindsurfDocument::parse(malformed)
            .expect_err("the malformed secret-bearing field should be rejected");
        let output = format!("{document:?}\n{rendered:?}\n{error:?}\n{error}");

        for secret in [
            "fixture-windsurf-unchanged-secret",
            "fixture-windsurf-old-secret",
            "fixture-windsurf-local-secret",
            "fixture-windsurf-remote-secret",
            "fixture-windsurf-added-secret",
            "fixture-windsurf-new-secret",
            "fixture-windsurf-diagnostic-secret",
            "https://windsurf.example.invalid/mcp",
            "/synthetic/bin/old",
            "/synthetic/bin/new",
        ] {
            assert!(!output.contains(secret));
        }
        assert!(output.contains("local_server_names"));
        assert!(output.contains("unmanaged_server_count"));
        assert!(output.contains("changed"));
    }

    #[test]
    fn rendering_rejects_a_plan_for_a_different_native_presence_set() {
        let document = WindsurfDocument::parse(CURRENT_FIXTURE)
            .expect("current Windsurf fixture should be valid");
        let empty =
            CanonicalConfig::new(BTreeMap::new()).expect("an empty canonical config is valid");
        let plan = reconcile(&empty, document.canonical_config());

        let error = document
            .render_plan(&plan)
            .expect_err("an add plan must not overwrite existing native servers");

        assert!(matches!(
            error,
            WindsurfAdapterError::PlanPresenceMismatch {
                outcome: ReconciliationOutcomeKind::Add,
                present: true,
                ..
            }
        ));
    }

    #[test]
    fn rendering_rejects_a_plan_that_omits_current_local_servers() {
        let document = WindsurfDocument::parse(CURRENT_FIXTURE)
            .expect("current Windsurf fixture should be valid");
        let empty =
            CanonicalConfig::new(BTreeMap::new()).expect("an empty canonical config is valid");
        let empty_plan = reconcile(&empty, &empty);

        let error = document
            .render_plan(&empty_plan)
            .expect_err("a partial plan must not bypass target-only preservation");

        assert!(matches!(
            error,
            WindsurfAdapterError::PlanOmittedCurrentServer { .. }
        ));
    }
}
