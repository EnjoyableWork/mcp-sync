use crate::config::{CanonicalConfig, CanonicalServer, ConfigError, parse_unique_json_value};
use crate::filesystem::{FileIoError, FileSystem};
use crate::paths::MacOsConfigurationPaths;
use crate::reconciliation::{ReconciliationOutcomeKind, ReconciliationPlan};
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

const CLAUDE_DIRECTORY: &str = "Claude";
const CLAUDE_CONFIGURATION_FILE: &str = "claude_desktop_config.json";
const MCP_SERVERS_FIELD: &str = "mcpServers";
const COMMAND_FIELD: &str = "command";
const ARGUMENTS_FIELD: &str = "args";
const ENVIRONMENT_FIELD: &str = "env";

/// The current Claude Desktop global configuration target on macOS.
///
/// Discovery is read-only. Missing configuration is a normal state; every
/// other filesystem failure remains contextual and actionable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClaudeDesktopAdapter {
    configuration_path: PathBuf,
}

impl ClaudeDesktopAdapter {
    pub fn for_macos(paths: &MacOsConfigurationPaths) -> Self {
        Self {
            configuration_path: paths
                .application_support()
                .join(CLAUDE_DIRECTORY)
                .join(CLAUDE_CONFIGURATION_FILE),
        }
    }

    pub fn configuration_path(&self) -> &Path {
        &self.configuration_path
    }

    pub fn discover(
        &self,
        filesystem: &impl FileSystem,
    ) -> Result<ClaudeDesktopDiscovery, ClaudeDesktopAdapterError> {
        match filesystem.read(&self.configuration_path) {
            Ok(bytes) => ClaudeDesktopDocument::parse(&bytes).map(ClaudeDesktopDiscovery::Found),
            Err(source) if source.kind() == io::ErrorKind::NotFound => {
                Ok(ClaudeDesktopDiscovery::Missing)
            }
            Err(source) => Err(ClaudeDesktopAdapterError::Read { source }),
        }
    }
}

#[derive(Debug)]
pub enum ClaudeDesktopDiscovery {
    Missing,
    Found(ClaudeDesktopDocument),
}

impl ClaudeDesktopDiscovery {
    pub fn document(&self) -> Option<&ClaudeDesktopDocument> {
        match self {
            Self::Missing => None,
            Self::Found(document) => Some(document),
        }
    }

    pub fn into_document(self) -> ClaudeDesktopDocument {
        match self {
            Self::Missing => ClaudeDesktopDocument::empty(),
            Self::Found(document) => document,
        }
    }
}

/// A validated Claude Desktop document plus its exact original bytes.
///
/// The adapter owns `mcpServers.<name>.command`, `args`, and `env`. Every other
/// top-level and per-server field remains native data and is preserved when a
/// plan is rendered.
pub struct ClaudeDesktopDocument {
    root: Map<String, Value>,
    canonical_config: CanonicalConfig,
    original_bytes: Vec<u8>,
}

impl ClaudeDesktopDocument {
    pub fn parse(document: &[u8]) -> Result<Self, ClaudeDesktopAdapterError> {
        let value = parse_unique_json_value(document).map_err(|error| {
            ClaudeDesktopAdapterError::InvalidJson {
                message: error.to_string(),
            }
        })?;
        let Value::Object(root) = value else {
            return Err(ClaudeDesktopDocumentError::RootMustBeObject.into());
        };

        let servers = decode_servers(root.get(MCP_SERVERS_FIELD))?;
        let canonical_config = CanonicalConfig::new(servers)
            .map_err(|source| ClaudeDesktopAdapterError::InvalidCanonical { source })?;

        Ok(Self {
            root,
            canonical_config,
            original_bytes: document.to_vec(),
        })
    }

    fn empty() -> Self {
        Self {
            root: Map::new(),
            canonical_config: CanonicalConfig::new(BTreeMap::new())
                .expect("an empty canonical server map is valid"),
            original_bytes: b"{}\n".to_vec(),
        }
    }

    pub fn canonical_config(&self) -> &CanonicalConfig {
        &self.canonical_config
    }

    pub fn preserved_root_fields(&self) -> Vec<&str> {
        self.root
            .keys()
            .filter(|field| field.as_str() != MCP_SERVERS_FIELD)
            .map(String::as_str)
            .collect()
    }

    pub fn preserved_server_fields(&self, server_name: &str) -> Option<Vec<&str>> {
        let fields = self
            .root
            .get(MCP_SERVERS_FIELD)?
            .as_object()?
            .get(server_name)?
            .as_object()?;

        Some(
            fields
                .keys()
                .filter(|field| {
                    !matches!(
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
    ) -> Result<RenderedClaudeDesktopDocument, ClaudeDesktopAdapterError> {
        self.validate_plan(plan)?;
        if !plan.requires_mutation() {
            return Ok(RenderedClaudeDesktopDocument::new(
                self.original_bytes.clone(),
                false,
            ));
        }

        let mut root = self.root.clone();
        let servers = root
            .entry(MCP_SERVERS_FIELD.to_owned())
            .or_insert_with(|| Value::Object(Map::new()))
            .as_object_mut()
            .ok_or(ClaudeDesktopDocumentError::McpServersMustBeObject)?;

        for entry in plan.entries() {
            if !matches!(
                entry.outcome().kind(),
                ReconciliationOutcomeKind::Add | ReconciliationOutcomeKind::Update
            ) {
                continue;
            }

            let desired = entry.desired_server().ok_or_else(|| {
                ClaudeDesktopAdapterError::PlanMissingDesiredServer {
                    server: entry.server_name().to_owned(),
                }
            })?;
            let native_server = servers
                .entry(entry.server_name().to_owned())
                .or_insert_with(|| Value::Object(Map::new()))
                .as_object_mut()
                .ok_or_else(|| ClaudeDesktopDocumentError::ServerMustBeObject {
                    server: entry.server_name().to_owned(),
                })?;

            write_managed_fields(native_server, desired);
        }

        let mut bytes = serde_json::to_vec_pretty(&Value::Object(root)).map_err(|error| {
            ClaudeDesktopAdapterError::Serialization {
                message: error.to_string(),
            }
        })?;
        bytes.push(b'\n');

        Self::parse(&bytes)?;
        let changed = bytes != self.original_bytes;
        Ok(RenderedClaudeDesktopDocument::new(bytes, changed))
    }

    fn validate_plan(&self, plan: &ReconciliationPlan) -> Result<(), ClaudeDesktopAdapterError> {
        let mut planned_names = BTreeSet::new();

        for entry in plan.entries() {
            planned_names.insert(entry.server_name());
            let present = self
                .canonical_config
                .servers()
                .contains_key(entry.server_name());
            let should_be_present = entry.outcome().kind() != ReconciliationOutcomeKind::Add;

            if present != should_be_present {
                return Err(ClaudeDesktopAdapterError::PlanPresenceMismatch {
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
                return Err(ClaudeDesktopAdapterError::PlanMissingDesiredServer {
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
            return Err(ClaudeDesktopAdapterError::PlanOmittedCurrentServer {
                server: server.clone(),
            });
        }

        Ok(())
    }
}

impl fmt::Debug for ClaudeDesktopDocument {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ClaudeDesktopDocument")
            .field(
                "server_names",
                &self.canonical_config.servers().keys().collect::<Vec<_>>(),
            )
            .field("preserved_root_fields", &self.preserved_root_fields())
            .field("byte_count", &self.original_bytes.len())
            .finish()
    }
}

pub struct RenderedClaudeDesktopDocument {
    bytes: Vec<u8>,
    changed: bool,
}

impl RenderedClaudeDesktopDocument {
    fn new(bytes: Vec<u8>, changed: bool) -> Self {
        Self { bytes, changed }
    }

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

impl fmt::Debug for RenderedClaudeDesktopDocument {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RenderedClaudeDesktopDocument")
            .field("byte_count", &self.bytes.len())
            .field("changed", &self.changed)
            .finish()
    }
}

fn decode_servers(
    value: Option<&Value>,
) -> Result<BTreeMap<String, CanonicalServer>, ClaudeDesktopAdapterError> {
    let Some(value) = value else {
        return Ok(BTreeMap::new());
    };
    let Value::Object(servers) = value else {
        return Err(ClaudeDesktopDocumentError::McpServersMustBeObject.into());
    };

    servers
        .iter()
        .map(|(name, value)| decode_server(name, value).map(|server| (name.clone(), server)))
        .collect()
}

fn decode_server(name: &str, value: &Value) -> Result<CanonicalServer, ClaudeDesktopAdapterError> {
    let Value::Object(fields) = value else {
        return Err(ClaudeDesktopDocumentError::ServerMustBeObject {
            server: name.to_owned(),
        }
        .into());
    };

    let command =
        fields
            .get(COMMAND_FIELD)
            .ok_or_else(|| ClaudeDesktopDocumentError::MissingCommand {
                server: name.to_owned(),
            })?;
    let Value::String(command) = command else {
        return Err(ClaudeDesktopDocumentError::CommandMustBeString {
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
) -> Result<Vec<String>, ClaudeDesktopAdapterError> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let Value::Array(arguments) = value else {
        return Err(ClaudeDesktopDocumentError::ArgumentsMustBeArray {
            server: server.to_owned(),
        }
        .into());
    };

    arguments
        .iter()
        .enumerate()
        .map(|(index, argument)| match argument {
            Value::String(argument) => Ok(argument.clone()),
            _ => Err(ClaudeDesktopDocumentError::ArgumentMustBeString {
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
) -> Result<BTreeMap<String, String>, ClaudeDesktopAdapterError> {
    let Some(value) = value else {
        return Ok(BTreeMap::new());
    };
    let Value::Object(environment) = value else {
        return Err(ClaudeDesktopDocumentError::EnvironmentMustBeObject {
            server: server.to_owned(),
        }
        .into());
    };

    environment
        .iter()
        .enumerate()
        .map(|(position, (key, value))| match value {
            Value::String(value) => Ok((key.clone(), value.clone())),
            _ => Err(ClaudeDesktopDocumentError::EnvironmentValueMustBeString {
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
pub enum ClaudeDesktopAdapterError {
    Read {
        source: FileIoError,
    },
    InvalidJson {
        message: String,
    },
    InvalidDocument(ClaudeDesktopDocumentError),
    InvalidCanonical {
        source: ConfigError,
    },
    Serialization {
        message: String,
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

impl fmt::Display for ClaudeDesktopAdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { source } => source.fmt(formatter),
            Self::InvalidJson { message } => {
                write!(formatter, "invalid Claude Desktop JSON: {message}")
            }
            Self::InvalidDocument(error) => error.fmt(formatter),
            Self::InvalidCanonical { source } => {
                write!(
                    formatter,
                    "invalid Claude Desktop MCP server definition: {source}"
                )
            }
            Self::Serialization { message } => {
                write!(
                    formatter,
                    "cannot serialize Claude Desktop configuration: {message}"
                )
            }
            Self::PlanPresenceMismatch {
                server,
                outcome,
                present,
            } => write!(
                formatter,
                "reconciliation plan outcome {outcome:?} for server {server:?} does not match Claude Desktop document presence ({present})"
            ),
            Self::PlanOmittedCurrentServer { server } => write!(
                formatter,
                "reconciliation plan omits current Claude Desktop server {server:?}"
            ),
            Self::PlanMissingDesiredServer { server } => write!(
                formatter,
                "reconciliation plan has no desired definition for Claude Desktop server {server:?}"
            ),
        }
    }
}

impl Error for ClaudeDesktopAdapterError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Read { source } => Some(source),
            Self::InvalidDocument(error) => Some(error),
            Self::InvalidCanonical { source } => Some(source),
            Self::InvalidJson { .. }
            | Self::Serialization { .. }
            | Self::PlanPresenceMismatch { .. }
            | Self::PlanOmittedCurrentServer { .. }
            | Self::PlanMissingDesiredServer { .. } => None,
        }
    }
}

impl From<ClaudeDesktopDocumentError> for ClaudeDesktopAdapterError {
    fn from(error: ClaudeDesktopDocumentError) -> Self {
        Self::InvalidDocument(error)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClaudeDesktopDocumentError {
    RootMustBeObject,
    McpServersMustBeObject,
    ServerMustBeObject { server: String },
    MissingCommand { server: String },
    CommandMustBeString { server: String },
    ArgumentsMustBeArray { server: String },
    ArgumentMustBeString { server: String, index: usize },
    EnvironmentMustBeObject { server: String },
    EnvironmentValueMustBeString { server: String, position: usize },
}

impl fmt::Display for ClaudeDesktopDocumentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RootMustBeObject => {
                formatter.write_str("Claude Desktop configuration must be an object")
            }
            Self::McpServersMustBeObject => {
                formatter.write_str("Claude Desktop field `mcpServers` must be an object")
            }
            Self::ServerMustBeObject { server } => {
                write!(
                    formatter,
                    "Claude Desktop server {server:?} must be an object"
                )
            }
            Self::MissingCommand { server } => write!(
                formatter,
                "Claude Desktop server {server:?} is missing required field `command`"
            ),
            Self::CommandMustBeString { server } => write!(
                formatter,
                "Claude Desktop server {server:?} field `command` must be a string"
            ),
            Self::ArgumentsMustBeArray { server } => write!(
                formatter,
                "Claude Desktop server {server:?} field `args` must be an array"
            ),
            Self::ArgumentMustBeString { server, index } => write!(
                formatter,
                "Claude Desktop server {server:?} argument at index {index} must be a string"
            ),
            Self::EnvironmentMustBeObject { server } => write!(
                formatter,
                "Claude Desktop server {server:?} field `env` must be an object"
            ),
            Self::EnvironmentValueMustBeString { server, position } => write!(
                formatter,
                "Claude Desktop server {server:?} environment value at position {position} must be a string"
            ),
        }
    }
}

impl Error for ClaudeDesktopDocumentError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths::Environment;
    use crate::reconciliation::{ReconciliationOutcomeKind, reconcile};
    use std::ffi::OsString;

    const CURRENT_FIXTURE: &[u8] = include_bytes!("../tests/fixtures/claude-desktop/current.json");
    const DESIRED_FIXTURE: &str = include_str!("../tests/fixtures/claude-desktop/desired.json");
    const MERGED_FIXTURE: &[u8] = include_bytes!("../tests/fixtures/claude-desktop/merged.json");

    struct FixtureEnvironment {
        home: PathBuf,
    }

    impl Environment for FixtureEnvironment {
        fn value(&self, name: &'static str) -> Option<OsString> {
            (name == "HOME").then(|| self.home.clone().into_os_string())
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
                    io::Error::new(*kind, "synthetic Claude Desktop read failure"),
                )),
            }
        }
    }

    fn adapter_fixture() -> (tempfile::TempDir, ClaudeDesktopAdapter) {
        let root = tempfile::tempdir().expect("temporary adapter fixture should be created");
        let paths = MacOsConfigurationPaths::resolve(&FixtureEnvironment {
            home: root.path().join("user"),
        })
        .expect("synthetic macOS paths should resolve");
        let adapter = ClaudeDesktopAdapter::for_macos(&paths);
        (root, adapter)
    }

    fn desired_config() -> CanonicalConfig {
        CanonicalConfig::parse_json(DESIRED_FIXTURE)
            .expect("desired fixture should be valid canonical JSON")
    }

    #[test]
    fn macos_discovery_path_matches_the_current_global_contract() {
        let (root, adapter) = adapter_fixture();

        assert_eq!(
            adapter.configuration_path(),
            root.path()
                .join("user/Library/Application Support/Claude/claude_desktop_config.json")
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
            .expect("the fixture should be discovered");

        assert_eq!(document.canonical_config().servers().len(), 3);
        assert_eq!(
            document.preserved_root_fields(),
            ["futureTopLevel", "globalShortcut", "preferences"]
        );
        assert_eq!(
            document.preserved_server_fields("updated").unwrap(),
            ["cwd", "futureOption"]
        );
    }

    #[test]
    fn a_missing_native_file_is_a_normal_empty_discovery_state() {
        let (_root, adapter) = adapter_fixture();
        let filesystem = FixtureFileSystem {
            expected_path: adapter.configuration_path().to_owned(),
            result: Err(io::ErrorKind::NotFound),
        };

        let discovery = adapter
            .discover(&filesystem)
            .expect("a missing target should not be an error");

        assert!(discovery.document().is_none());
        assert!(
            discovery
                .into_document()
                .canonical_config()
                .servers()
                .is_empty()
        );
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

        let ClaudeDesktopAdapterError::Read { source } = error else {
            panic!("the adapter should preserve the filesystem error");
        };
        assert_eq!(source.kind(), io::ErrorKind::PermissionDenied);
        assert_eq!(source.path(), adapter.configuration_path());
    }

    #[test]
    fn fixture_plan_round_trips_and_preserves_every_unmanaged_field() {
        let document =
            ClaudeDesktopDocument::parse(CURRENT_FIXTURE).expect("current fixture should be valid");
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

        let merged = ClaudeDesktopDocument::parse(first.bytes())
            .expect("rendered fixture should parse again");
        assert_eq!(
            merged.preserved_root_fields(),
            ["futureTopLevel", "globalShortcut", "preferences"]
        );
        assert_eq!(
            merged.preserved_server_fields("updated").unwrap(),
            ["cwd", "futureOption"]
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
        let document =
            ClaudeDesktopDocument::parse(CURRENT_FIXTURE).expect("current fixture should be valid");
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
    fn a_missing_document_can_render_a_new_native_server_map() {
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
            .expect("additions should render into an empty native document");
        let reparsed = ClaudeDesktopDocument::parse(rendered.bytes())
            .expect("new native document should be valid");

        assert!(rendered.changed());
        assert_eq!(reparsed.canonical_config(), &desired);
        assert!(reparsed.preserved_root_fields().is_empty());
    }

    #[test]
    fn missing_mcp_servers_normalizes_to_empty_without_losing_root_data() {
        let bytes = br#"{
  "futureSetting": {
    "enabled": true
  }
}
"#;
        let document = ClaudeDesktopDocument::parse(bytes)
            .expect("a document without mcpServers should be valid");
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
    fn omitted_native_arguments_and_environment_normalize_to_empty_collections() {
        let document = ClaudeDesktopDocument::parse(
            br#"{"mcpServers":{"minimal":{"command":"synthetic-command"}}}"#,
        )
        .expect("optional native fields may be omitted");
        let server = document
            .canonical_config()
            .servers()
            .get("minimal")
            .expect("minimal server should normalize");

        assert_eq!(server.command(), "synthetic-command");
        assert!(server.args().is_empty());
        assert!(server.env().is_empty());
    }

    #[test]
    fn malformed_json_and_invalid_utf8_are_rejected_without_echoing_values() {
        let malformed = br#"{"future":"fixture-json-secret","mcpServers":"#;
        let invalid_utf8 = [0xff, 0xfe, 0xfd];

        for bytes in [malformed.as_slice(), invalid_utf8.as_slice()] {
            let error = ClaudeDesktopDocument::parse(bytes)
                .expect_err("invalid native JSON bytes should be rejected");
            let output = format!("{error:?}\n{error}");

            assert!(matches!(
                error,
                ClaudeDesktopAdapterError::InvalidJson { .. }
            ));
            assert!(!output.contains("fixture-json-secret"));
        }
    }

    #[test]
    fn invalid_native_shapes_have_structural_errors() {
        let cases = [
            ("root", br#"[]"#.as_slice()),
            ("server map", br#"{"mcpServers":[]}"#.as_slice()),
            ("server", br#"{"mcpServers":{"fixture":[]}}"#.as_slice()),
            (
                "missing command",
                br#"{"mcpServers":{"fixture":{}}}"#.as_slice(),
            ),
            (
                "command",
                br#"{"mcpServers":{"fixture":{"command":7}}}"#.as_slice(),
            ),
            (
                "arguments",
                br#"{"mcpServers":{"fixture":{"command":"safe","args":{}}}}"#.as_slice(),
            ),
            (
                "argument",
                br#"{"mcpServers":{"fixture":{"command":"safe","args":[7]}}}"#.as_slice(),
            ),
            (
                "environment",
                br#"{"mcpServers":{"fixture":{"command":"safe","env":[]}}}"#.as_slice(),
            ),
            (
                "environment value",
                br#"{"mcpServers":{"fixture":{"command":"safe","env":{"TOKEN":7}}}}"#.as_slice(),
            ),
        ];

        for (label, bytes) in cases {
            let error = ClaudeDesktopDocument::parse(bytes).unwrap_err();
            assert!(
                matches!(error, ClaudeDesktopAdapterError::InvalidDocument(_)),
                "unexpected {label} error: {error}"
            );
        }

        let error = ClaudeDesktopDocument::parse(br#"{"mcpServers":{"fixture":{"command":""}}}"#)
            .expect_err("invalid canonical process text should be rejected");
        assert!(matches!(
            error,
            ClaudeDesktopAdapterError::InvalidCanonical { .. }
        ));
    }

    #[test]
    fn duplicate_keys_are_rejected_at_every_native_object_level() {
        let cases = [
            br#"{"mcpServers":{},"mcpServers":{}}"#.as_slice(),
            br#"{"mcpServers":{"same":{"command":"safe"},"same":{"command":"safe"}}}"#.as_slice(),
            br#"{"mcpServers":{"fixture":{"command":"safe","command":"safe"}}}"#.as_slice(),
            br#"{"mcpServers":{"fixture":{"command":"safe","env":{"TOKEN":"one","TOKEN":"two"}}}}"#
                .as_slice(),
        ];

        for bytes in cases {
            let error =
                ClaudeDesktopDocument::parse(bytes).expect_err("duplicate keys should be rejected");
            let ClaudeDesktopAdapterError::InvalidJson { message } = error else {
                panic!("duplicate keys should be invalid JSON");
            };
            assert!(message.contains("duplicate JSON object key"));
        }
    }

    #[test]
    fn diagnostics_and_debug_output_never_echo_native_values() {
        let document =
            ClaudeDesktopDocument::parse(CURRENT_FIXTURE).expect("current fixture should be valid");
        let rendered = document
            .render_plan(&reconcile(document.canonical_config(), &desired_config()))
            .expect("fixture plan should render");
        let malformed =
            br#"{"mcpServers":{"fixture":{"command":"safe","args":"fixture-diagnostic-secret"}}}"#;
        let error = ClaudeDesktopDocument::parse(malformed)
            .expect_err("the malformed secret-bearing field should be rejected");
        let output = format!("{document:?}\n{rendered:?}\n{error:?}\n{error}");

        for secret in [
            "fixture-unchanged-secret",
            "fixture-old-secret",
            "fixture-local-secret",
            "fixture-added-secret",
            "fixture-new-secret",
            "fixture-diagnostic-secret",
            "/synthetic/bin/old",
            "/synthetic/bin/new",
        ] {
            assert!(!output.contains(secret));
        }
        assert!(output.contains("server_names"));
        assert!(output.contains("changed"));
    }

    #[test]
    fn rendering_rejects_a_plan_for_a_different_native_presence_set() {
        let document =
            ClaudeDesktopDocument::parse(CURRENT_FIXTURE).expect("current fixture should be valid");
        let empty = CanonicalConfig::new(BTreeMap::new())
            .expect("an empty canonical config should be valid");
        let plan = reconcile(&empty, document.canonical_config());

        let error = document
            .render_plan(&plan)
            .expect_err("an add plan must not overwrite existing native servers");

        assert!(matches!(
            error,
            ClaudeDesktopAdapterError::PlanPresenceMismatch {
                outcome: ReconciliationOutcomeKind::Add,
                present: true,
                ..
            }
        ));
    }

    #[test]
    fn rendering_rejects_a_plan_that_omits_current_native_servers() {
        let document =
            ClaudeDesktopDocument::parse(CURRENT_FIXTURE).expect("current fixture should be valid");
        let empty = CanonicalConfig::new(BTreeMap::new())
            .expect("an empty canonical config should be valid");
        let empty_plan = reconcile(&empty, &empty);

        let error = document
            .render_plan(&empty_plan)
            .expect_err("a partial plan must not bypass target-only preservation");

        assert!(matches!(
            error,
            ClaudeDesktopAdapterError::PlanOmittedCurrentServer { .. }
        ));
    }

    #[test]
    fn rendered_bytes_can_be_consumed_without_exposing_them_through_debug() {
        let document =
            ClaudeDesktopDocument::parse(CURRENT_FIXTURE).expect("current fixture should be valid");
        let rendered = document
            .render_plan(&reconcile(document.canonical_config(), &desired_config()))
            .expect("fixture plan should render");
        let debug = format!("{rendered:?}");

        assert!(debug.contains("byte_count"));
        assert!(!debug.contains("fixture-added-secret"));
        assert_eq!(rendered.into_bytes(), MERGED_FIXTURE);
    }
}
