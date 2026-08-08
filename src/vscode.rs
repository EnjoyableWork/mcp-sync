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

const CODE_DIRECTORY: &str = "Code";
const USER_DIRECTORY: &str = "User";
const VSCODE_CONFIGURATION_FILE: &str = "mcp.json";
const SERVERS_FIELD: &str = "servers";
const TYPE_FIELD: &str = "type";
const STDIO_TYPE: &str = "stdio";
const COMMAND_FIELD: &str = "command";
const ARGUMENTS_FIELD: &str = "args";
const ENVIRONMENT_FIELD: &str = "env";

/// The native VS Code default user-profile MCP target on macOS, Linux, and
/// Windows.
///
/// Discovery resolves exactly
/// the platform user-data root through the injected paths. It intentionally
/// does not discover workspace
/// `.vscode/mcp.json`, named profiles, remote profiles, VS Code Insiders,
/// portable installations, Cline or Roo extension storage, or Agent Host and
/// Copilot CLI configuration. Discovery is read-only; a missing file is a
/// normal state and every other filesystem failure remains contextual.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VsCodeAdapter {
    configuration_path: PathBuf,
}

impl VsCodeAdapter {
    pub fn from_paths(paths: &ConfigurationPaths) -> Self {
        Self {
            configuration_path: paths
                .user_data_home()
                .join(CODE_DIRECTORY)
                .join(USER_DIRECTORY)
                .join(VSCODE_CONFIGURATION_FILE),
        }
    }

    pub fn configuration_path(&self) -> &Path {
        &self.configuration_path
    }

    pub fn discover(
        &self,
        filesystem: &impl FileSystem,
    ) -> Result<VsCodeDiscovery, VsCodeAdapterError> {
        match filesystem.read(&self.configuration_path) {
            Ok(bytes) => VsCodeDocument::parse(&bytes).map(VsCodeDiscovery::Found),
            Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(VsCodeDiscovery::Missing),
            Err(source) => Err(VsCodeAdapterError::Read { source }),
        }
    }
}

#[derive(Debug)]
pub enum VsCodeDiscovery {
    Missing,
    Found(VsCodeDocument),
}

impl VsCodeDiscovery {
    #[cfg(test)]
    pub fn document(&self) -> Option<&VsCodeDocument> {
        match self {
            Self::Missing => None,
            Self::Found(document) => Some(document),
        }
    }

    pub fn into_document(self) -> VsCodeDocument {
        match self {
            Self::Missing => VsCodeDocument::empty(),
            Self::Found(document) => document,
        }
    }
}

/// A validated native VS Code user-profile document plus its original bytes.
///
/// Canonical v1 owns only `servers.<name>.command`, `args`, and string-valued
/// `env` for compatible local STDIO entries. An omitted `type` is accepted for
/// the minimal shape emitted by VS Code itself; a new canonical entry receives
/// an explicit `"type": "stdio"`. Existing `type`, `cwd`, `envFile`, `dev`,
/// `sandboxEnabled`, root `inputs` and `sandbox`, and every unknown field remain
/// native data. Remote entries and otherwise valid local entries containing
/// number or null environment values are preserved and reported as unmanaged.
pub struct VsCodeDocument {
    root: Map<String, Value>,
    canonical_config: CanonicalConfig,
    unmanaged_server_names: BTreeSet<String>,
    original_bytes: Vec<u8>,
}

impl VsCodeDocument {
    pub fn parse(document: &[u8]) -> Result<Self, VsCodeAdapterError> {
        let value =
            parse_unique_json_value(document).map_err(|error| VsCodeAdapterError::InvalidJson {
                message: error.to_string(),
            })?;
        let Value::Object(root) = value else {
            return Err(VsCodeDocumentError::RootMustBeObject.into());
        };

        let decoded = decode_servers(root.get(SERVERS_FIELD))?;
        let canonical_config = CanonicalConfig::new(decoded.local_servers)
            .map_err(|source| VsCodeAdapterError::InvalidCanonical { source })?;

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
    /// Only names are exposed. URLs, headers, OAuth data, non-string
    /// environment values, input definitions, and other values remain private.
    pub fn unmanaged_server_names(&self) -> Vec<&str> {
        self.unmanaged_server_names
            .iter()
            .map(String::as_str)
            .collect()
    }

    pub fn preserved_root_fields(&self) -> Vec<&str> {
        self.root
            .keys()
            .filter(|field| field.as_str() != SERVERS_FIELD)
            .map(String::as_str)
            .collect()
    }

    #[cfg(test)]
    pub fn preserved_server_fields(&self, server_name: &str) -> Option<Vec<&str>> {
        let fields = self
            .root
            .get(SERVERS_FIELD)?
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
    ) -> Result<RenderedVsCodeDocument, VsCodeAdapterError> {
        self.validate_plan(plan)?;
        if !plan.requires_mutation() {
            return Ok(RenderedVsCodeDocument::new(
                self.original_bytes.clone(),
                false,
            ));
        }

        let mut root = self.root.clone();
        let servers = root
            .entry(SERVERS_FIELD.to_owned())
            .or_insert_with(|| Value::Object(Map::new()))
            .as_object_mut()
            .ok_or(VsCodeDocumentError::ServersMustBeObject)?;

        for entry in plan.entries() {
            if !matches!(
                entry.outcome().kind(),
                ReconciliationOutcomeKind::Add | ReconciliationOutcomeKind::Update
            ) {
                continue;
            }

            let desired = entry.desired_server().ok_or_else(|| {
                VsCodeAdapterError::PlanMissingDesiredServer {
                    server: entry.server_name().to_owned(),
                }
            })?;
            let is_add = entry.outcome().kind() == ReconciliationOutcomeKind::Add;
            let native_server = servers
                .entry(entry.server_name().to_owned())
                .or_insert_with(|| Value::Object(Map::new()))
                .as_object_mut()
                .ok_or_else(|| VsCodeDocumentError::ServerMustBeObject {
                    server: entry.server_name().to_owned(),
                })?;

            if is_add {
                native_server.insert(TYPE_FIELD.to_owned(), Value::String(STDIO_TYPE.to_owned()));
            }
            write_managed_fields(native_server, desired);
        }

        let mut bytes = serde_json::to_vec_pretty(&Value::Object(root)).map_err(|error| {
            VsCodeAdapterError::Serialization {
                message: error.to_string(),
            }
        })?;
        bytes.push(b'\n');

        Self::parse(&bytes)?;
        let changed = bytes != self.original_bytes;
        Ok(RenderedVsCodeDocument::new(bytes, changed))
    }

    fn validate_plan(&self, plan: &ReconciliationPlan) -> Result<(), VsCodeAdapterError> {
        let mut planned_names = BTreeSet::new();

        for entry in plan.entries() {
            planned_names.insert(entry.server_name());

            if self.unmanaged_server_names.contains(entry.server_name()) {
                return Err(VsCodeAdapterError::UnmanagedServerCollision {
                    server: entry.server_name().to_owned(),
                });
            }

            let present = self
                .canonical_config
                .servers()
                .contains_key(entry.server_name());
            let should_be_present = entry.outcome().kind() != ReconciliationOutcomeKind::Add;

            if present != should_be_present {
                return Err(VsCodeAdapterError::PlanPresenceMismatch {
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
                return Err(VsCodeAdapterError::PlanMissingDesiredServer {
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
            return Err(VsCodeAdapterError::PlanOmittedCurrentServer {
                server: server.clone(),
            });
        }

        Ok(())
    }
}

impl fmt::Debug for VsCodeDocument {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("VsCodeDocument")
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

pub struct RenderedVsCodeDocument {
    bytes: Vec<u8>,
    changed: bool,
}

impl RenderedVsCodeDocument {
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

impl fmt::Debug for RenderedVsCodeDocument {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RenderedVsCodeDocument")
            .field("byte_count", &self.bytes.len())
            .field("changed", &self.changed)
            .finish()
    }
}

struct DecodedServers {
    local_servers: BTreeMap<String, CanonicalServer>,
    unmanaged_server_names: BTreeSet<String>,
}

enum DecodedServer {
    Local(CanonicalServer),
    Unmanaged,
}

enum DecodedEnvironment {
    Compatible(BTreeMap<String, String>),
    CanonicalIncompatible,
}

fn decode_servers(value: Option<&Value>) -> Result<DecodedServers, VsCodeAdapterError> {
    let Some(value) = value else {
        return Ok(DecodedServers {
            local_servers: BTreeMap::new(),
            unmanaged_server_names: BTreeSet::new(),
        });
    };
    let Value::Object(servers) = value else {
        return Err(VsCodeDocumentError::ServersMustBeObject.into());
    };

    let mut local_servers = BTreeMap::new();
    let mut unmanaged_server_names = BTreeSet::new();

    for (name, value) in servers {
        let Value::Object(fields) = value else {
            return Err(VsCodeDocumentError::ServerMustBeObject {
                server: name.clone(),
            }
            .into());
        };

        match decode_server(name, fields)? {
            DecodedServer::Local(server) => {
                local_servers.insert(name.clone(), server);
            }
            DecodedServer::Unmanaged => {
                unmanaged_server_names.insert(name.clone());
            }
        }
    }

    Ok(DecodedServers {
        local_servers,
        unmanaged_server_names,
    })
}

fn decode_server(
    name: &str,
    fields: &Map<String, Value>,
) -> Result<DecodedServer, VsCodeAdapterError> {
    let server_type = match fields.get(TYPE_FIELD) {
        None => None,
        Some(Value::String(server_type)) => Some(server_type.as_str()),
        Some(_) => {
            return Err(VsCodeDocumentError::TypeMustBeString {
                server: name.to_owned(),
            }
            .into());
        }
    };

    match server_type {
        Some(STDIO_TYPE) => {}
        None if fields.contains_key(COMMAND_FIELD) => {}
        _ => return Ok(DecodedServer::Unmanaged),
    }

    let command = fields
        .get(COMMAND_FIELD)
        .and_then(Value::as_str)
        .ok_or_else(|| VsCodeDocumentError::CommandMustBeString {
            server: name.to_owned(),
        })?
        .to_owned();
    let args = decode_arguments(name, fields.get(ARGUMENTS_FIELD))?;
    let env = match decode_environment(name, fields.get(ENVIRONMENT_FIELD))? {
        DecodedEnvironment::Compatible(env) => env,
        DecodedEnvironment::CanonicalIncompatible => return Ok(DecodedServer::Unmanaged),
    };

    Ok(DecodedServer::Local(CanonicalServer::new(
        command, args, env,
    )))
}

fn decode_arguments(
    server: &str,
    value: Option<&Value>,
) -> Result<Vec<String>, VsCodeDocumentError> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let Value::Array(values) = value else {
        return Err(VsCodeDocumentError::ArgumentsMustBeArray {
            server: server.to_owned(),
        });
    };

    values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            value.as_str().map(str::to_owned).ok_or_else(|| {
                VsCodeDocumentError::ArgumentMustBeString {
                    server: server.to_owned(),
                    index,
                }
            })
        })
        .collect()
}

fn decode_environment(
    server: &str,
    value: Option<&Value>,
) -> Result<DecodedEnvironment, VsCodeDocumentError> {
    let Some(value) = value else {
        return Ok(DecodedEnvironment::Compatible(BTreeMap::new()));
    };
    let Value::Object(values) = value else {
        return Err(VsCodeDocumentError::EnvironmentMustBeObject {
            server: server.to_owned(),
        });
    };

    let mut environment = BTreeMap::new();
    let mut canonical_compatible = true;

    for (position, (key, value)) in values.iter().enumerate() {
        match value {
            Value::String(value) => {
                environment.insert(key.clone(), value.clone());
            }
            Value::Number(_) | Value::Null => {
                canonical_compatible = false;
            }
            _ => {
                return Err(
                    VsCodeDocumentError::EnvironmentValueMustBeStringNumberOrNull {
                        server: server.to_owned(),
                        position,
                    },
                );
            }
        }
    }

    if canonical_compatible {
        Ok(DecodedEnvironment::Compatible(environment))
    } else {
        Ok(DecodedEnvironment::CanonicalIncompatible)
    }
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
pub enum VsCodeAdapterError {
    Read {
        source: FileIoError,
    },
    InvalidJson {
        message: String,
    },
    InvalidDocument(VsCodeDocumentError),
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

impl fmt::Display for VsCodeAdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { source } => {
                write!(formatter, "cannot read VS Code configuration: {source}")
            }
            Self::InvalidJson { message } => {
                write!(formatter, "invalid VS Code JSON: {message}")
            }
            Self::InvalidDocument(error) => error.fmt(formatter),
            Self::InvalidCanonical { source } => {
                write!(
                    formatter,
                    "invalid local VS Code server definition: {source}"
                )
            }
            Self::Serialization { message } => {
                write!(
                    formatter,
                    "cannot serialize VS Code configuration: {message}"
                )
            }
            Self::UnmanagedServerCollision { server } => write!(
                formatter,
                "desired local server {server:?} collides with an unmanaged VS Code server"
            ),
            Self::PlanPresenceMismatch {
                server,
                outcome,
                present,
            } => write!(
                formatter,
                "reconciliation plan outcome {outcome:?} for VS Code server {server:?} does not match native presence ({present})"
            ),
            Self::PlanOmittedCurrentServer { server } => write!(
                formatter,
                "reconciliation plan omits current VS Code server {server:?}"
            ),
            Self::PlanMissingDesiredServer { server } => write!(
                formatter,
                "reconciliation plan has no desired definition for VS Code server {server:?}"
            ),
        }
    }
}

impl Error for VsCodeAdapterError {
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

impl From<VsCodeDocumentError> for VsCodeAdapterError {
    fn from(error: VsCodeDocumentError) -> Self {
        Self::InvalidDocument(error)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VsCodeDocumentError {
    RootMustBeObject,
    ServersMustBeObject,
    ServerMustBeObject { server: String },
    TypeMustBeString { server: String },
    CommandMustBeString { server: String },
    ArgumentsMustBeArray { server: String },
    ArgumentMustBeString { server: String, index: usize },
    EnvironmentMustBeObject { server: String },
    EnvironmentValueMustBeStringNumberOrNull { server: String, position: usize },
}

impl fmt::Display for VsCodeDocumentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RootMustBeObject => {
                formatter.write_str("VS Code configuration must be an object")
            }
            Self::ServersMustBeObject => {
                formatter.write_str("VS Code field `servers` must be an object")
            }
            Self::ServerMustBeObject { server } => {
                write!(formatter, "VS Code server {server:?} must be an object")
            }
            Self::TypeMustBeString { server } => write!(
                formatter,
                "VS Code server {server:?} field `type` must be a string"
            ),
            Self::CommandMustBeString { server } => write!(
                formatter,
                "VS Code local server {server:?} field `command` must be a string"
            ),
            Self::ArgumentsMustBeArray { server } => write!(
                formatter,
                "VS Code local server {server:?} field `args` must be an array"
            ),
            Self::ArgumentMustBeString { server, index } => write!(
                formatter,
                "VS Code local server {server:?} argument at index {index} must be a string"
            ),
            Self::EnvironmentMustBeObject { server } => write!(
                formatter,
                "VS Code local server {server:?} field `env` must be an object"
            ),
            Self::EnvironmentValueMustBeStringNumberOrNull { server, position } => write!(
                formatter,
                "VS Code local server {server:?} environment value at position {position} must be a string, number, or null"
            ),
        }
    }
}

impl Error for VsCodeDocumentError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths::{Environment, Platform};
    use crate::reconciliation::{ReconciliationOutcomeKind, reconcile};
    use std::ffi::OsString;

    const CURRENT_FIXTURE: &[u8] = include_bytes!("../tests/fixtures/vscode/current.json");
    const DESIRED_FIXTURE: &str = include_str!("../tests/fixtures/vscode/desired.json");
    const MERGED_FIXTURE: &[u8] = include_bytes!("../tests/fixtures/vscode/merged.json");

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
                    io::Error::new(*kind, "synthetic VS Code read failure"),
                )),
            }
        }
    }

    fn adapter_fixture_for(platform: Platform) -> (tempfile::TempDir, VsCodeAdapter) {
        let root = tempfile::tempdir().expect("temporary adapter fixture should be created");
        let paths = ConfigurationPaths::resolve_for(
            platform,
            &FixtureEnvironment {
                home: root.path().join("user"),
            },
        )
        .expect("synthetic platform paths should resolve");
        let adapter = VsCodeAdapter::from_paths(&paths);
        (root, adapter)
    }

    fn adapter_fixture() -> (tempfile::TempDir, VsCodeAdapter) {
        adapter_fixture_for(Platform::MacOs)
    }

    fn desired_config() -> CanonicalConfig {
        CanonicalConfig::parse_json(DESIRED_FIXTURE)
            .expect("desired fixture should be valid canonical JSON")
    }

    #[test]
    fn macos_discovery_path_is_only_the_native_default_user_profile_contract() {
        let (root, adapter) = adapter_fixture();
        let home = root.path().join("user");

        assert_eq!(
            adapter.configuration_path(),
            home.join("Library/Application Support/Code/User/mcp.json")
        );
        assert!(adapter.configuration_path().starts_with(root.path()));
        for excluded in [
            home.join("workspace/.vscode/mcp.json"),
            home.join("workspace/.mcp.json"),
            home.join("Library/Application Support/Code/User/profiles/profile/mcp.json"),
            home.join("Library/Application Support/Code - Insiders/User/mcp.json"),
            home.join("Library/Application Support/Code/User/globalStorage/saoudrizwan.claude-dev/settings/cline_mcp_settings.json"),
            home.join(".cline/data/settings/cline_mcp_settings.json"),
            home.join(".copilot/mcp-config.json"),
        ] {
            assert_ne!(adapter.configuration_path(), excluded);
        }
    }

    #[test]
    fn linux_discovery_path_is_only_the_native_default_user_profile_contract() {
        let (root, adapter) = adapter_fixture_for(Platform::Linux);
        let home = root.path().join("user");

        assert_eq!(
            adapter.configuration_path(),
            home.join(".config/Code/User/mcp.json")
        );
        assert!(adapter.configuration_path().starts_with(root.path()));
        for excluded in [
            home.join("workspace/.vscode/mcp.json"),
            home.join("workspace/.mcp.json"),
            home.join(".config/Code/User/profiles/profile/mcp.json"),
            home.join(".config/Code - Insiders/User/mcp.json"),
            home.join(".config/Code/User/globalStorage/saoudrizwan.claude-dev/settings/cline_mcp_settings.json"),
            home.join(".cline/data/settings/cline_mcp_settings.json"),
            home.join(".copilot/mcp-config.json"),
        ] {
            assert_ne!(adapter.configuration_path(), excluded);
        }
    }

    #[test]
    fn windows_discovery_path_is_only_the_native_default_user_profile_contract() {
        let (root, adapter) = adapter_fixture_for(Platform::Windows);
        let home = root.path().join("user");

        assert_eq!(
            adapter.configuration_path(),
            home.join("AppData/Roaming/Code/User/mcp.json")
        );
        assert!(adapter.configuration_path().starts_with(root.path()));
        for excluded in [
            home.join("workspace/.vscode/mcp.json"),
            home.join("workspace/.mcp.json"),
            home.join("AppData/Roaming/Code/User/profiles/profile/mcp.json"),
            home.join("AppData/Roaming/Code - Insiders/User/mcp.json"),
            home.join("AppData/Roaming/Code/User/globalStorage/saoudrizwan.claude-dev/settings/cline_mcp_settings.json"),
            home.join(".cline/data/settings/cline_mcp_settings.json"),
            home.join(".copilot/mcp-config.json"),
        ] {
            assert_ne!(adapter.configuration_path(), excluded);
        }
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
        assert_eq!(
            document.unmanaged_server_names(),
            ["numeric-env", "remote-only"]
        );
        assert_eq!(
            document.preserved_root_fields(),
            ["futureTopLevel", "inputs", "sandbox"]
        );
        assert_eq!(
            document.preserved_server_fields("updated").unwrap(),
            ["dev", "envFile"]
        );
        assert_eq!(
            document.preserved_server_fields("remote-only").unwrap(),
            ["headers", "oauth", "type", "url"]
        );
        assert_eq!(
            document.preserved_server_fields("numeric-env").unwrap(),
            ["args", "command", "env", "futureOption", "type"]
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

        let VsCodeAdapterError::Read { source } = error else {
            panic!("the adapter should preserve the filesystem error");
        };
        assert_eq!(source.kind(), io::ErrorKind::PermissionDenied);
        assert_eq!(source.path(), adapter.configuration_path());
    }

    #[test]
    fn fixture_plan_round_trips_and_preserves_every_unmanaged_field() {
        let document = VsCodeDocument::parse(CURRENT_FIXTURE)
            .expect("current VS Code fixture should be valid");
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

        let merged = VsCodeDocument::parse(first.bytes())
            .expect("rendered VS Code fixture should parse again");
        assert_eq!(
            merged.unmanaged_server_names(),
            ["numeric-env", "remote-only"]
        );
        assert_eq!(
            merged.preserved_root_fields(),
            ["futureTopLevel", "inputs", "sandbox"]
        );
        assert_eq!(
            merged.preserved_server_fields("updated").unwrap(),
            ["dev", "envFile"]
        );
        assert_eq!(
            merged.preserved_server_fields("remote-only").unwrap(),
            ["headers", "oauth", "type", "url"]
        );
        assert_eq!(
            merged.preserved_server_fields("target-only").unwrap(),
            ["cwd", "type"]
        );

        let added = serde_json::from_slice::<Value>(first.bytes())
            .expect("rendered fixture should be JSON");
        assert_eq!(added["servers"]["added"]["type"], STDIO_TYPE);

        let settled = reconcile(merged.canonical_config(), &desired);
        assert!(!settled.requires_mutation());
        assert!(settled.has_drift());
        assert_eq!(settled.summary().no_op(), 3);
        assert_eq!(settled.summary().drift(), 1);
    }

    #[test]
    fn no_op_and_drift_rendering_preserves_original_bytes_exactly() {
        let document = VsCodeDocument::parse(CURRENT_FIXTURE)
            .expect("current VS Code fixture should be valid");
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
    fn a_missing_document_renders_explicit_stdio_entries() {
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
            .expect("additions should render into an empty VS Code document");
        let native: Value =
            serde_json::from_slice(rendered.bytes()).expect("new document should be JSON");
        let reparsed =
            VsCodeDocument::parse(rendered.bytes()).expect("new VS Code document should be valid");

        assert!(rendered.changed());
        assert_eq!(reparsed.canonical_config(), &desired);
        assert!(
            native["servers"]
                .as_object()
                .unwrap()
                .values()
                .all(|server| server["type"] == STDIO_TYPE)
        );
        assert!(reparsed.preserved_root_fields().is_empty());
        assert!(reparsed.unmanaged_server_names().is_empty());
    }

    #[test]
    fn missing_server_map_normalizes_to_empty_without_losing_root_data() {
        let bytes = br#"{
  "inputs": [],
  "futureSetting": {
    "enabled": true
  }
}
"#;
        let document =
            VsCodeDocument::parse(bytes).expect("a document without servers should be valid");
        let plan = reconcile(document.canonical_config(), document.canonical_config());

        let rendered = document
            .render_plan(&plan)
            .expect("an empty plan should preserve the document");

        assert!(document.canonical_config().servers().is_empty());
        assert_eq!(
            document.preserved_root_fields(),
            ["futureSetting", "inputs"]
        );
        assert!(!rendered.changed());
        assert_eq!(rendered.bytes(), bytes);
    }

    #[test]
    fn stdio_defaults_interpolation_and_unowned_fields_remain_literal_and_bounded() {
        let document = VsCodeDocument::parse(
            br#"{"servers":{"implicit":{"command":"${input:command}","env":{"TOKEN":"${env:SYNTHETIC_TOKEN}"},"envFile":"${userHome}/.env"},"explicit":{"type":"stdio","command":"safe","cwd":"${workspaceFolder}"}}}"#,
        )
        .expect("supported local STDIO forms should parse");
        let implicit = document
            .canonical_config()
            .servers()
            .get("implicit")
            .expect("implicit STDIO server should normalize");

        assert_eq!(implicit.command(), "${input:command}");
        assert!(implicit.args().is_empty());
        assert_eq!(
            implicit.env().get("TOKEN").unwrap(),
            "${env:SYNTHETIC_TOKEN}"
        );
        assert_eq!(
            document.preserved_server_fields("implicit").unwrap(),
            ["envFile"]
        );
        assert_eq!(
            document.preserved_server_fields("explicit").unwrap(),
            ["cwd", "type"]
        );
    }

    #[test]
    fn remote_unknown_opaque_and_native_scalar_environment_entries_are_unmanaged() {
        let bytes = br#"{
  "servers": {
    "http": {"type":"http","url":"https://vscode.example.invalid/http"},
    "sse": {"type":"sse","url":"https://vscode.example.invalid/sse"},
    "future": {"type":"future-transport","private":"fixture-future-value"},
    "opaque": {},
    "implicit-opaque": {"url":"https://vscode.example.invalid/implicit"},
    "native-env": {"type":"stdio","command":"safe","env":{"PORT":3000,"OPTIONAL":null}}
  }
}
"#;
        let document = VsCodeDocument::parse(bytes)
            .expect("valid unsupported VS Code entries should remain native data");
        let empty =
            CanonicalConfig::new(BTreeMap::new()).expect("an empty canonical config is valid");
        let rendered = document
            .render_plan(&reconcile(document.canonical_config(), &empty))
            .expect("an unmanaged-only document should render without mutation");

        assert!(document.canonical_config().servers().is_empty());
        assert_eq!(
            document.unmanaged_server_names(),
            [
                "future",
                "http",
                "implicit-opaque",
                "native-env",
                "opaque",
                "sse"
            ]
        );
        assert!(!rendered.changed());
        assert_eq!(rendered.bytes(), bytes);
    }

    #[test]
    fn desired_local_server_cannot_overwrite_any_unmanaged_entry() {
        let document = VsCodeDocument::parse(CURRENT_FIXTURE)
            .expect("current VS Code fixture should be valid");

        for name in ["remote-only", "numeric-env"] {
            let desired = CanonicalConfig::new(BTreeMap::from([(
                name.to_owned(),
                CanonicalServer::new("/synthetic/bin/collision", Vec::new(), BTreeMap::new()),
            )]))
            .expect("the colliding desired server should be canonical");
            let error = document
                .render_plan(&reconcile(document.canonical_config(), &desired))
                .expect_err("a local add must not replace unmanaged native data");

            assert!(matches!(
                error,
                VsCodeAdapterError::UnmanagedServerCollision { ref server } if server == name
            ));
        }
    }

    #[test]
    fn malformed_json_and_invalid_utf8_are_rejected_without_echoing_values() {
        let malformed = br#"{"future":"fixture-vscode-json-secret","servers":"#;
        let invalid_utf8 = [0xff, 0xfe, 0xfd];

        for bytes in [malformed.as_slice(), invalid_utf8.as_slice()] {
            let error = VsCodeDocument::parse(bytes)
                .expect_err("invalid native JSON bytes should be rejected");
            let output = format!("{error:?}\n{error}");

            assert!(matches!(error, VsCodeAdapterError::InvalidJson { .. }));
            assert!(!output.contains("fixture-vscode-json-secret"));
        }
    }

    #[test]
    fn invalid_native_local_shapes_have_structural_errors() {
        let cases = [
            br#"[]"#.as_slice(),
            br#"{"servers":[]}"#.as_slice(),
            br#"{"servers":{"fixture":[]}}"#.as_slice(),
            br#"{"servers":{"fixture":{"type":7}}}"#.as_slice(),
            br#"{"servers":{"fixture":{"type":"stdio","command":7}}}"#.as_slice(),
            br#"{"servers":{"fixture":{"command":"safe","args":{}}}}"#.as_slice(),
            br#"{"servers":{"fixture":{"command":"safe","args":[7]}}}"#.as_slice(),
            br#"{"servers":{"fixture":{"command":"safe","env":[]}}}"#.as_slice(),
            br#"{"servers":{"fixture":{"command":"safe","env":{"TOKEN":true}}}}"#.as_slice(),
            br#"{"servers":{"fixture":{"command":"safe","env":{"TOKEN":[]}}}}"#.as_slice(),
            br#"{"servers":{"fixture":{"command":"safe","env":{"TOKEN":{}}}}}"#.as_slice(),
        ];

        for bytes in cases {
            let error = VsCodeDocument::parse(bytes).unwrap_err();
            assert!(matches!(error, VsCodeAdapterError::InvalidDocument(_)));
        }

        let error = VsCodeDocument::parse(br#"{"servers":{"fixture":{"command":""}}}"#)
            .expect_err("invalid canonical process text should be rejected");
        assert!(matches!(error, VsCodeAdapterError::InvalidCanonical { .. }));
    }

    #[test]
    fn duplicate_keys_are_rejected_at_every_native_object_level() {
        let cases = [
            br#"{"servers":{},"servers":{}}"#.as_slice(),
            br#"{"servers":{"same":{"command":"safe"},"same":{"command":"safe"}}}"#
                .as_slice(),
            br#"{"servers":{"fixture":{"command":"safe","command":"safe"}}}"#.as_slice(),
            br#"{"servers":{"fixture":{"command":"safe","env":{"TOKEN":"one","TOKEN":"two"}}}}"#
                .as_slice(),
            br#"{"inputs":[{"id":"one","id":"two"}]}"#.as_slice(),
            br#"{"servers":{"remote":{"type":"http","url":"https://example.invalid","headers":{"TOKEN":"one","TOKEN":"two"}}}}"#
                .as_slice(),
        ];

        for bytes in cases {
            let error =
                VsCodeDocument::parse(bytes).expect_err("duplicate keys should be rejected");
            let VsCodeAdapterError::InvalidJson { message } = error else {
                panic!("duplicate keys should be invalid JSON");
            };
            assert!(message.contains("duplicate JSON object key"));
        }
    }

    #[test]
    fn diagnostics_and_debug_output_never_echo_native_values() {
        let document = VsCodeDocument::parse(CURRENT_FIXTURE)
            .expect("current VS Code fixture should be valid");
        let rendered = document
            .render_plan(&reconcile(document.canonical_config(), &desired_config()))
            .expect("fixture plan should render");
        let malformed = br#"{"servers":{"fixture":{"command":"safe","args":"fixture-vscode-diagnostic-secret"}}}"#;
        let error = VsCodeDocument::parse(malformed)
            .expect_err("the malformed secret-bearing field should be rejected");
        let output = format!("{document:?}\n{rendered:?}\n{error:?}\n{error}");

        for secret in [
            "fixture-vscode-unchanged-secret",
            "fixture-vscode-old-secret",
            "fixture-vscode-local-secret",
            "fixture-vscode-remote-secret",
            "fixture-vscode-added-secret",
            "fixture-vscode-new-secret",
            "fixture-vscode-diagnostic-secret",
            "https://vscode.example.invalid/mcp",
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
        let document = VsCodeDocument::parse(CURRENT_FIXTURE)
            .expect("current VS Code fixture should be valid");
        let empty =
            CanonicalConfig::new(BTreeMap::new()).expect("an empty canonical config is valid");
        let plan = reconcile(&empty, document.canonical_config());

        let error = document
            .render_plan(&plan)
            .expect_err("an add plan must not overwrite existing native servers");

        assert!(matches!(
            error,
            VsCodeAdapterError::PlanPresenceMismatch {
                outcome: ReconciliationOutcomeKind::Add,
                present: true,
                ..
            }
        ));
    }

    #[test]
    fn rendering_rejects_a_plan_that_omits_current_local_servers() {
        let document = VsCodeDocument::parse(CURRENT_FIXTURE)
            .expect("current VS Code fixture should be valid");
        let empty =
            CanonicalConfig::new(BTreeMap::new()).expect("an empty canonical config is valid");
        let empty_plan = reconcile(&empty, &empty);

        let error = document
            .render_plan(&empty_plan)
            .expect_err("a partial plan must not bypass target-only preservation");

        assert!(matches!(
            error,
            VsCodeAdapterError::PlanOmittedCurrentServer { .. }
        ));
    }
}
