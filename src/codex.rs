use crate::config::{CanonicalConfig, CanonicalServer, ConfigError};
use crate::filesystem::{FileIoError, FileSystem};
use crate::paths::ConfigurationPaths;
use crate::reconciliation::{ReconciliationOutcomeKind, ReconciliationPlan};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use toml_edit::{Array, DocumentMut, Item, Table, TableLike, Value, value};

const CODEX_DIRECTORY: &str = ".codex";
const CODEX_CONFIGURATION_FILE: &str = "config.toml";
const MCP_SERVERS_FIELD: &str = "mcp_servers";
const COMMAND_FIELD: &str = "command";
const ARGUMENTS_FIELD: &str = "args";
const ENVIRONMENT_FIELD: &str = "env";
const URL_FIELD: &str = "url";

/// The global Codex-host MCP target on macOS and Linux.
///
/// Discovery resolves exactly `~/.codex/config.toml` through the injected
/// user home. The ChatGPT desktop app, Codex CLI, and Codex IDE extension
/// share this user-level configuration. Project `.codex/config.toml` layers,
/// profile files, system configuration, OAuth credential stores, and every
/// other path remain outside this adapter boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodexAdapter {
    configuration_path: PathBuf,
}

impl CodexAdapter {
    pub fn from_paths(paths: &ConfigurationPaths) -> Self {
        Self {
            configuration_path: paths
                .user_home()
                .join(CODEX_DIRECTORY)
                .join(CODEX_CONFIGURATION_FILE),
        }
    }

    pub fn configuration_path(&self) -> &Path {
        &self.configuration_path
    }

    pub fn discover(
        &self,
        filesystem: &impl FileSystem,
    ) -> Result<CodexDiscovery, CodexAdapterError> {
        match filesystem.read(&self.configuration_path) {
            Ok(bytes) => CodexDocument::parse(&bytes)
                .map(Box::new)
                .map(CodexDiscovery::Found),
            Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(CodexDiscovery::Missing),
            Err(source) => Err(CodexAdapterError::Read { source }),
        }
    }
}

#[derive(Debug)]
pub enum CodexDiscovery {
    Missing,
    Found(Box<CodexDocument>),
}

impl CodexDiscovery {
    #[cfg(test)]
    pub fn document(&self) -> Option<&CodexDocument> {
        match self {
            Self::Missing => None,
            Self::Found(document) => Some(document.as_ref()),
        }
    }

    pub fn into_document(self) -> CodexDocument {
        match self {
            Self::Missing => CodexDocument::empty(),
            Self::Found(document) => *document,
        }
    }
}

/// A parsed, editable Codex TOML document plus its canonical-compatible view.
///
/// Only `command`, `args`, and `env` are canonical-owned inside unambiguous
/// command-based entries. URL-based remote entries, mixed command/URL entries,
/// opaque target-only entries, authentication, headers, tool policy, timeout,
/// working-directory, environment-forwarding, and every other native field
/// remain unmanaged TOML. The editable document preserves comments, ordering,
/// formatting, and unrelated settings without a JSON conversion.
pub struct CodexDocument {
    document: DocumentMut,
    canonical_config: CanonicalConfig,
    unmanaged_server_names: BTreeSet<String>,
    original_bytes: Vec<u8>,
}

impl CodexDocument {
    pub fn parse(document: &[u8]) -> Result<Self, CodexAdapterError> {
        let text = std::str::from_utf8(document).map_err(|_| CodexAdapterError::InvalidEncoding)?;
        let parsed =
            DocumentMut::from_str(text).map_err(|error| CodexAdapterError::InvalidToml {
                message: error.message().to_owned(),
                byte_offset: error.span().map(|span| span.start),
            })?;
        let decoded = decode_servers(&parsed)?;
        let canonical_config = CanonicalConfig::new(decoded.local_servers)
            .map_err(|source| CodexAdapterError::InvalidCanonical { source })?;

        Ok(Self {
            document: parsed,
            canonical_config,
            unmanaged_server_names: decoded.unmanaged_server_names,
            original_bytes: document.to_vec(),
        })
    }

    fn empty() -> Self {
        Self {
            document: DocumentMut::new(),
            canonical_config: CanonicalConfig::new(BTreeMap::new())
                .expect("an empty canonical server map is valid"),
            unmanaged_server_names: BTreeSet::new(),
            original_bytes: Vec::new(),
        }
    }

    pub fn canonical_config(&self) -> &CanonicalConfig {
        &self.canonical_config
    }

    /// Exact bytes observed during discovery for guarded sync application.
    pub fn original_bytes(&self) -> &[u8] {
        &self.original_bytes
    }

    /// Names of native MCP entries outside canonical JSON v1.
    ///
    /// Only names are exposed. URLs, headers, authentication, OAuth settings,
    /// commands, arguments, and environment values remain private.
    pub fn unmanaged_server_names(&self) -> Vec<&str> {
        self.unmanaged_server_names
            .iter()
            .map(String::as_str)
            .collect()
    }

    pub fn preserved_root_fields(&self) -> Vec<&str> {
        self.document
            .iter()
            .map(|(field, _)| field)
            .filter(|field| *field != MCP_SERVERS_FIELD)
            .collect()
    }

    #[cfg(test)]
    pub fn preserved_server_fields(&self, server_name: &str) -> Option<Vec<&str>> {
        let servers = self.document.get(MCP_SERVERS_FIELD)?.as_table_like()?;
        let fields = servers.get(server_name)?.as_table_like()?;
        let is_managed_local = self.canonical_config.servers().contains_key(server_name);

        Some(
            fields
                .iter()
                .map(|(field, _)| field)
                .filter(|field| {
                    !is_managed_local
                        || !matches!(*field, COMMAND_FIELD | ARGUMENTS_FIELD | ENVIRONMENT_FIELD)
                })
                .collect(),
        )
    }

    pub fn render_plan(
        &self,
        plan: &ReconciliationPlan,
    ) -> Result<RenderedCodexDocument, CodexAdapterError> {
        self.validate_plan(plan)?;
        if !plan.requires_mutation() {
            return Ok(RenderedCodexDocument::new(
                self.original_bytes.clone(),
                false,
            ));
        }

        let mut document = self.document.clone();
        let servers = ensure_servers_table(&mut document)?;

        for entry in plan.entries() {
            if !matches!(
                entry.outcome().kind(),
                ReconciliationOutcomeKind::Add | ReconciliationOutcomeKind::Update
            ) {
                continue;
            }

            let desired = entry.desired_server().ok_or_else(|| {
                CodexAdapterError::PlanMissingDesiredServer {
                    server: entry.server_name().to_owned(),
                }
            })?;

            if entry.outcome().kind() == ReconciliationOutcomeKind::Add {
                servers.insert(entry.server_name(), Item::Table(Table::new()));
            }

            let fields = servers
                .get_mut(entry.server_name())
                .and_then(Item::as_table_like_mut)
                .ok_or_else(|| CodexDocumentError::ServerMustBeTable {
                    server: entry.server_name().to_owned(),
                })?;
            write_managed_fields(entry.server_name(), fields, desired)?;
        }

        let bytes = document.to_string().into_bytes();
        Self::parse(&bytes)?;
        let changed = bytes != self.original_bytes;
        Ok(RenderedCodexDocument::new(bytes, changed))
    }

    fn validate_plan(&self, plan: &ReconciliationPlan) -> Result<(), CodexAdapterError> {
        let mut planned_names = BTreeSet::new();

        for entry in plan.entries() {
            planned_names.insert(entry.server_name());

            if self.unmanaged_server_names.contains(entry.server_name()) {
                return Err(CodexAdapterError::UnmanagedServerCollision {
                    server: entry.server_name().to_owned(),
                });
            }

            let present = self
                .canonical_config
                .servers()
                .contains_key(entry.server_name());
            let should_be_present = entry.outcome().kind() != ReconciliationOutcomeKind::Add;

            if present != should_be_present {
                return Err(CodexAdapterError::PlanPresenceMismatch {
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
                return Err(CodexAdapterError::PlanMissingDesiredServer {
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
            return Err(CodexAdapterError::PlanOmittedCurrentServer {
                server: server.clone(),
            });
        }

        Ok(())
    }
}

impl fmt::Debug for CodexDocument {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CodexDocument")
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

pub struct RenderedCodexDocument {
    bytes: Vec<u8>,
    changed: bool,
}

impl RenderedCodexDocument {
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

impl fmt::Debug for RenderedCodexDocument {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RenderedCodexDocument")
            .field("byte_count", &self.bytes.len())
            .field("changed", &self.changed)
            .finish()
    }
}

struct DecodedServers {
    local_servers: BTreeMap<String, CanonicalServer>,
    unmanaged_server_names: BTreeSet<String>,
}

fn decode_servers(document: &DocumentMut) -> Result<DecodedServers, CodexAdapterError> {
    let Some(item) = document.get(MCP_SERVERS_FIELD) else {
        return Ok(DecodedServers {
            local_servers: BTreeMap::new(),
            unmanaged_server_names: BTreeSet::new(),
        });
    };
    let servers = item
        .as_table_like()
        .ok_or(CodexDocumentError::McpServersMustBeTable)?;

    let mut local_servers = BTreeMap::new();
    let mut unmanaged_server_names = BTreeSet::new();

    for (name, item) in servers.iter() {
        let fields = item
            .as_table_like()
            .ok_or_else(|| CodexDocumentError::ServerMustBeTable {
                server: name.to_owned(),
            })?;

        if fields.contains_key(COMMAND_FIELD) && !fields.contains_key(URL_FIELD) {
            local_servers.insert(name.to_owned(), decode_local_server(name, fields)?);
        } else {
            unmanaged_server_names.insert(name.to_owned());
        }
    }

    Ok(DecodedServers {
        local_servers,
        unmanaged_server_names,
    })
}

fn decode_local_server(
    name: &str,
    fields: &dyn TableLike,
) -> Result<CanonicalServer, CodexAdapterError> {
    let command = fields
        .get(COMMAND_FIELD)
        .and_then(Item::as_str)
        .ok_or_else(|| CodexDocumentError::CommandMustBeString {
            server: name.to_owned(),
        })?
        .to_owned();
    let arguments = decode_arguments(name, fields.get(ARGUMENTS_FIELD))?;
    let environment = decode_environment(name, fields.get(ENVIRONMENT_FIELD))?;

    Ok(CanonicalServer::new(command, arguments, environment))
}

fn decode_arguments(server: &str, item: Option<&Item>) -> Result<Vec<String>, CodexAdapterError> {
    let Some(item) = item else {
        return Ok(Vec::new());
    };
    let arguments = item
        .as_array()
        .ok_or_else(|| CodexDocumentError::ArgumentsMustBeArray {
            server: server.to_owned(),
        })?;

    arguments
        .iter()
        .enumerate()
        .map(|(index, argument)| {
            argument.as_str().map(str::to_owned).ok_or_else(|| {
                CodexDocumentError::ArgumentMustBeString {
                    server: server.to_owned(),
                    index,
                }
                .into()
            })
        })
        .collect()
}

fn decode_environment(
    server: &str,
    item: Option<&Item>,
) -> Result<BTreeMap<String, String>, CodexAdapterError> {
    let Some(item) = item else {
        return Ok(BTreeMap::new());
    };
    let environment =
        item.as_table_like()
            .ok_or_else(|| CodexDocumentError::EnvironmentMustBeTable {
                server: server.to_owned(),
            })?;

    environment
        .iter()
        .enumerate()
        .map(|(position, (key, item))| {
            item.as_str()
                .map(|value| (key.to_owned(), value.to_owned()))
                .ok_or_else(|| {
                    CodexDocumentError::EnvironmentValueMustBeString {
                        server: server.to_owned(),
                        position,
                    }
                    .into()
                })
        })
        .collect()
}

fn ensure_servers_table(
    document: &mut DocumentMut,
) -> Result<&mut dyn TableLike, CodexAdapterError> {
    if !document.contains_key(MCP_SERVERS_FIELD) {
        document.insert(MCP_SERVERS_FIELD, Item::Table(Table::new()));
    }
    document
        .get_mut(MCP_SERVERS_FIELD)
        .and_then(Item::as_table_like_mut)
        .ok_or_else(|| CodexDocumentError::McpServersMustBeTable.into())
}

fn write_managed_fields(
    name: &str,
    fields: &mut dyn TableLike,
    server: &CanonicalServer,
) -> Result<(), CodexAdapterError> {
    insert_value_preserving_decor(fields, COMMAND_FIELD, Value::from(server.command()));

    let arguments = server.args().iter().cloned().collect::<Array>();
    insert_value_preserving_decor(fields, ARGUMENTS_FIELD, Value::Array(arguments));

    if !fields.contains_key(ENVIRONMENT_FIELD) {
        fields.insert(ENVIRONMENT_FIELD, Item::Table(Table::new()));
    }
    let environment = fields
        .get_mut(ENVIRONMENT_FIELD)
        .and_then(Item::as_table_like_mut)
        .ok_or_else(|| CodexDocumentError::EnvironmentMustBeTable {
            server: name.to_owned(),
        })?;
    let obsolete_keys = environment
        .iter()
        .map(|(key, _)| key.to_owned())
        .filter(|key| !server.env().contains_key(key))
        .collect::<Vec<_>>();
    for key in obsolete_keys {
        environment.remove(&key);
    }
    for (key, value) in server.env() {
        insert_value_preserving_decor(environment, key, Value::from(value.as_str()));
    }

    Ok(())
}

fn insert_value_preserving_decor(table: &mut dyn TableLike, key: &str, mut new_value: Value) {
    if let Some(old_value) = table.get(key).and_then(Item::as_value) {
        new_value.decor_mut().clone_from(old_value.decor());
    }
    table.insert(key, value(new_value));
}

#[derive(Debug)]
pub enum CodexAdapterError {
    Read {
        source: FileIoError,
    },
    InvalidEncoding,
    InvalidToml {
        message: String,
        byte_offset: Option<usize>,
    },
    InvalidDocument(CodexDocumentError),
    InvalidCanonical {
        source: ConfigError,
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

impl fmt::Display for CodexAdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { source } => write!(formatter, "cannot read Codex configuration: {source}"),
            Self::InvalidEncoding => formatter.write_str("Codex configuration is not valid UTF-8"),
            Self::InvalidToml {
                message,
                byte_offset,
            } => match byte_offset {
                Some(offset) => write!(
                    formatter,
                    "invalid Codex TOML near byte {offset}: {message}"
                ),
                None => write!(formatter, "invalid Codex TOML: {message}"),
            },
            Self::InvalidDocument(error) => error.fmt(formatter),
            Self::InvalidCanonical { source } => {
                write!(formatter, "invalid local Codex server definition: {source}")
            }
            Self::UnmanagedServerCollision { server } => write!(
                formatter,
                "desired local server {server:?} collides with an unmanaged Codex server"
            ),
            Self::PlanPresenceMismatch {
                server,
                outcome,
                present,
            } => write!(
                formatter,
                "reconciliation plan outcome {outcome:?} for Codex server {server:?} does not match native presence ({present})"
            ),
            Self::PlanOmittedCurrentServer { server } => write!(
                formatter,
                "reconciliation plan omits current Codex server {server:?}"
            ),
            Self::PlanMissingDesiredServer { server } => write!(
                formatter,
                "reconciliation plan has no desired definition for Codex server {server:?}"
            ),
        }
    }
}

impl Error for CodexAdapterError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Read { source } => Some(source),
            Self::InvalidDocument(error) => Some(error),
            Self::InvalidCanonical { source } => Some(source),
            Self::InvalidEncoding
            | Self::InvalidToml { .. }
            | Self::UnmanagedServerCollision { .. }
            | Self::PlanPresenceMismatch { .. }
            | Self::PlanOmittedCurrentServer { .. }
            | Self::PlanMissingDesiredServer { .. } => None,
        }
    }
}

impl From<CodexDocumentError> for CodexAdapterError {
    fn from(error: CodexDocumentError) -> Self {
        Self::InvalidDocument(error)
    }
}

#[derive(Debug)]
pub enum CodexDocumentError {
    McpServersMustBeTable,
    ServerMustBeTable { server: String },
    CommandMustBeString { server: String },
    ArgumentsMustBeArray { server: String },
    ArgumentMustBeString { server: String, index: usize },
    EnvironmentMustBeTable { server: String },
    EnvironmentValueMustBeString { server: String, position: usize },
}

impl fmt::Display for CodexDocumentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::McpServersMustBeTable => {
                formatter.write_str("Codex `mcp_servers` must be a TOML table")
            }
            Self::ServerMustBeTable { server } => {
                write!(formatter, "Codex server {server:?} must be a TOML table")
            }
            Self::CommandMustBeString { server } => write!(
                formatter,
                "local Codex server {server:?} must have a string `command`"
            ),
            Self::ArgumentsMustBeArray { server } => write!(
                formatter,
                "local Codex server {server:?} must have an array `args`"
            ),
            Self::ArgumentMustBeString { server, index } => write!(
                formatter,
                "local Codex server {server:?} argument at index {index} must be a string"
            ),
            Self::EnvironmentMustBeTable { server } => write!(
                formatter,
                "local Codex server {server:?} must have a table `env`"
            ),
            Self::EnvironmentValueMustBeString { server, position } => write!(
                formatter,
                "local Codex server {server:?} environment value at position {position} must be a string"
            ),
        }
    }
}

impl Error for CodexDocumentError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths::{Environment, Platform};
    use crate::reconciliation::{ReconciliationOutcomeKind, reconcile};
    use std::ffi::OsString;

    const CURRENT_FIXTURE: &[u8] = include_bytes!("../tests/fixtures/codex/current.toml");
    const DESIRED_FIXTURE: &str = include_str!("../tests/fixtures/codex/desired.json");
    const MERGED_FIXTURE: &[u8] = include_bytes!("../tests/fixtures/codex/merged.toml");

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
                    io::Error::new(*kind, "synthetic Codex read failure"),
                )),
            }
        }
    }

    fn adapter_fixture_for(platform: Platform) -> (tempfile::TempDir, CodexAdapter) {
        let root = tempfile::tempdir().expect("temporary adapter fixture should be created");
        let paths = ConfigurationPaths::resolve_for(
            platform,
            &FixtureEnvironment {
                home: root.path().join("user"),
            },
        )
        .expect("synthetic platform paths should resolve");
        let adapter = CodexAdapter::from_paths(&paths);
        (root, adapter)
    }

    fn adapter_fixture() -> (tempfile::TempDir, CodexAdapter) {
        adapter_fixture_for(Platform::MacOs)
    }

    fn desired_config() -> CanonicalConfig {
        CanonicalConfig::parse_json(DESIRED_FIXTURE)
            .expect("desired fixture should be valid canonical JSON")
    }

    #[test]
    fn macos_discovery_path_is_only_the_global_user_contract() {
        let (root, adapter) = adapter_fixture();
        let home = root.path().join("user");

        assert_eq!(
            adapter.configuration_path(),
            home.join(".codex/config.toml")
        );
        assert!(adapter.configuration_path().starts_with(root.path()));
        for excluded in [
            home.join("workspace/.codex/config.toml"),
            home.join("workspace/nested/.codex/config.toml"),
            home.join(".codex/review.config.toml"),
            home.join(".codex/auth.json"),
            PathBuf::from("/etc/codex/config.toml"),
        ] {
            assert_ne!(adapter.configuration_path(), excluded);
        }
    }

    #[test]
    fn linux_discovery_path_is_only_the_global_user_contract() {
        let (root, adapter) = adapter_fixture_for(Platform::Linux);
        let home = root.path().join("user");

        assert_eq!(
            adapter.configuration_path(),
            home.join(".codex/config.toml")
        );
        assert!(adapter.configuration_path().starts_with(root.path()));
        for excluded in [
            home.join("workspace/.codex/config.toml"),
            home.join("workspace/nested/.codex/config.toml"),
            home.join(".codex/review.config.toml"),
            home.join(".codex/auth.json"),
            PathBuf::from("/etc/codex/config.toml"),
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
            .expect("the global Codex fixture should be discovered");

        assert_eq!(document.canonical_config().servers().len(), 3);
        assert_eq!(
            document.unmanaged_server_names(),
            ["mixed-transport", "opaque", "remote-only"]
        );
        assert_eq!(
            document.preserved_root_fields(),
            ["model", "mcp_oauth_callback_port", "future_root", "plugins"]
        );
        assert_eq!(
            document.preserved_server_fields("updated").unwrap(),
            ["cwd", "enabled", "env_vars"]
        );
        assert_eq!(
            document.preserved_server_fields("remote-only").unwrap(),
            ["url", "auth", "bearer_token_env_var", "http_headers"]
        );
        assert_eq!(
            document.preserved_server_fields("mixed-transport").unwrap(),
            ["command", "url", "args", "env"]
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
        assert!(document.original_bytes().is_empty());
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

        let CodexAdapterError::Read { source } = error else {
            panic!("the adapter should preserve the filesystem error");
        };
        assert_eq!(source.kind(), io::ErrorKind::PermissionDenied);
        assert_eq!(source.path(), adapter.configuration_path());
    }

    #[test]
    fn fixture_plan_is_deterministic_and_preserves_unmanaged_structure() {
        let document =
            CodexDocument::parse(CURRENT_FIXTURE).expect("current Codex fixture should be valid");
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

        let merged =
            CodexDocument::parse(first.bytes()).expect("rendered Codex fixture should parse again");
        assert_eq!(
            merged.unmanaged_server_names(),
            ["mixed-transport", "opaque", "remote-only"]
        );
        assert_eq!(
            merged.preserved_server_fields("updated").unwrap(),
            ["cwd", "enabled", "env_vars"]
        );
        assert_eq!(
            merged.preserved_server_fields("target-only").unwrap(),
            ["cwd"]
        );
        let rendered_text = std::str::from_utf8(first.bytes()).unwrap();
        for preserved in [
            "# Synthetic global Codex configuration. This comment must survive.",
            "# unrelated inline comment",
            "# OAuth callback setting is unmanaged",
            "# managed-field comment survives replacement",
            "# managed array comment survives replacement",
            "# retained-key comment survives",
            "fixture-codex-remote-secret",
            "fixture-codex-future-private",
            "default_tools_approval_mode = \"prompt\"",
        ] {
            assert!(rendered_text.contains(preserved));
        }
        assert!(!rendered_text.contains("fixture-codex-old-secret"));

        let settled = reconcile(merged.canonical_config(), &desired);
        assert!(!settled.requires_mutation());
        assert!(settled.has_drift());
        assert_eq!(settled.summary().no_op(), 3);
        assert_eq!(settled.summary().drift(), 1);
    }

    #[test]
    fn no_op_and_drift_rendering_preserves_original_bytes_exactly() {
        let document =
            CodexDocument::parse(CURRENT_FIXTURE).expect("current Codex fixture should be valid");
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
    fn a_missing_document_renders_native_local_stdio_tables() {
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
            .expect("additions should render into an empty Codex document");
        let reparsed =
            CodexDocument::parse(rendered.bytes()).expect("new Codex document should be valid");
        let text = std::str::from_utf8(rendered.bytes()).unwrap();

        assert!(rendered.changed());
        assert_eq!(reparsed.canonical_config(), &desired);
        assert!(text.contains("[mcp_servers.added]"));
        assert!(text.contains("[mcp_servers.added.env]"));
        assert!(reparsed.preserved_root_fields().is_empty());
        assert!(reparsed.unmanaged_server_names().is_empty());
    }

    #[test]
    fn missing_server_map_normalizes_to_empty_without_losing_root_data() {
        let bytes = b"# root comment\nmodel = \"synthetic\"\nfuture = { enabled = true }\n";
        let document =
            CodexDocument::parse(bytes).expect("a document without servers should be valid");
        let plan = reconcile(document.canonical_config(), document.canonical_config());

        let rendered = document
            .render_plan(&plan)
            .expect("an empty plan should preserve the document");

        assert!(document.canonical_config().servers().is_empty());
        assert_eq!(document.preserved_root_fields(), ["model", "future"]);
        assert!(!rendered.changed());
        assert_eq!(rendered.bytes(), bytes);
    }

    #[test]
    fn standard_and_inline_local_tables_normalize_literal_values_and_preserve_extras() {
        let bytes = br#"mcp_servers.inline = { command = "${env:COMMAND}", args = ["--inline"], env = { TOKEN = "${env:SYNTHETIC_TOKEN}" }, cwd = "/synthetic" }

[mcp_servers.standard]
command = "safe"
enabled = false
"#;
        let document =
            CodexDocument::parse(bytes).expect("standard and inline local tables should parse");
        let inline = document
            .canonical_config()
            .servers()
            .get("inline")
            .expect("inline local server should normalize");

        assert_eq!(inline.command(), "${env:COMMAND}");
        assert_eq!(inline.args(), ["--inline"]);
        assert_eq!(inline.env().get("TOKEN").unwrap(), "${env:SYNTHETIC_TOKEN}");
        assert_eq!(document.preserved_server_fields("inline").unwrap(), ["cwd"]);
        assert_eq!(
            document.preserved_server_fields("standard").unwrap(),
            ["enabled"]
        );
    }

    #[test]
    fn an_inline_server_map_supports_bounded_updates_and_additions() {
        let bytes = br#"mcp_servers = { current = { command = "old", args = ["--old"], env = { TOKEN = "old" }, cwd = "/synthetic" }, remote = { url = "https://codex.example.invalid/mcp", private = "fixture-codex-inline-remote-secret" } }
"#;
        let document = CodexDocument::parse(bytes).expect("the inline server map should normalize");
        let desired = CanonicalConfig::new(BTreeMap::from([
            (
                "added".to_owned(),
                CanonicalServer::new("added", Vec::new(), BTreeMap::new()),
            ),
            (
                "current".to_owned(),
                CanonicalServer::new(
                    "new",
                    vec!["--new".to_owned()],
                    BTreeMap::from([("TOKEN".to_owned(), "new".to_owned())]),
                ),
            ),
        ]))
        .expect("the desired inline-map fixture should be canonical");

        let rendered = document
            .render_plan(&reconcile(document.canonical_config(), &desired))
            .expect("inline tables should accept bounded structural edits");
        let reparsed =
            CodexDocument::parse(rendered.bytes()).expect("the rendered inline map should parse");
        let text = std::str::from_utf8(rendered.bytes()).unwrap();

        assert_eq!(reparsed.canonical_config(), &desired);
        assert_eq!(reparsed.unmanaged_server_names(), ["remote"]);
        assert_eq!(
            reparsed.preserved_server_fields("current").unwrap(),
            ["cwd"]
        );
        assert!(text.contains("fixture-codex-inline-remote-secret"));
    }

    #[test]
    fn remote_mixed_and_opaque_tables_are_unmanaged_without_inspecting_private_values() {
        let bytes = br#"[mcp_servers.http]
url = "https://codex.example.invalid/http"
auth = "oauth"

[mcp_servers.mixed]
command = 7
url = "https://codex.example.invalid/mixed"
private = "fixture-codex-mixed-diagnostic-secret"

[mcp_servers.future]
transport = "future"
private = "fixture-codex-future-diagnostic-secret"
"#;
        let document = CodexDocument::parse(bytes)
            .expect("unsupported table-shaped entries should remain native data");
        let empty =
            CanonicalConfig::new(BTreeMap::new()).expect("an empty canonical config is valid");
        let rendered = document
            .render_plan(&reconcile(document.canonical_config(), &empty))
            .expect("an unmanaged-only document should render without mutation");

        assert!(document.canonical_config().servers().is_empty());
        assert_eq!(
            document.unmanaged_server_names(),
            ["future", "http", "mixed"]
        );
        assert!(!rendered.changed());
        assert_eq!(rendered.bytes(), bytes);
    }

    #[test]
    fn desired_local_server_cannot_overwrite_any_unmanaged_entry() {
        let document =
            CodexDocument::parse(CURRENT_FIXTURE).expect("current Codex fixture should be valid");

        for name in ["remote-only", "mixed-transport", "opaque"] {
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
                CodexAdapterError::UnmanagedServerCollision { ref server } if server == name
            ));
        }
    }

    #[test]
    fn malformed_toml_and_invalid_utf8_are_rejected_without_echoing_values() {
        let malformed = b"private = \"fixture-codex-toml-secret\"\nmcp_servers = {";
        let invalid_utf8 = [0xff, 0xfe, 0xfd];

        let malformed_error =
            CodexDocument::parse(malformed).expect_err("malformed native TOML should be rejected");
        let malformed_output = format!("{malformed_error:?}\n{malformed_error}");
        assert!(matches!(
            malformed_error,
            CodexAdapterError::InvalidToml { .. }
        ));
        assert!(!malformed_output.contains("fixture-codex-toml-secret"));

        let encoding_error =
            CodexDocument::parse(&invalid_utf8).expect_err("invalid UTF-8 should be rejected");
        assert!(matches!(encoding_error, CodexAdapterError::InvalidEncoding));
    }

    #[test]
    fn invalid_native_local_shapes_have_structural_errors() {
        let cases = [
            "mcp_servers = []\n",
            "[mcp_servers]\nfixture = []\n",
            "[mcp_servers.fixture]\ncommand = 7\n",
            "[mcp_servers.fixture]\ncommand = \"safe\"\nargs = {}\n",
            "[mcp_servers.fixture]\ncommand = \"safe\"\nargs = [7]\n",
            "[mcp_servers.fixture]\ncommand = \"safe\"\nenv = []\n",
            "[mcp_servers.fixture]\ncommand = \"safe\"\n[mcp_servers.fixture.env]\nTOKEN = true\n",
        ];

        for text in cases {
            let error = CodexDocument::parse(text.as_bytes()).unwrap_err();
            assert!(matches!(error, CodexAdapterError::InvalidDocument(_)));
        }

        let error = CodexDocument::parse(b"[mcp_servers.fixture]\ncommand = \"\"\n")
            .expect_err("invalid canonical process text should be rejected");
        assert!(matches!(error, CodexAdapterError::InvalidCanonical { .. }));
    }

    #[test]
    fn duplicate_toml_keys_are_rejected_without_source_lines() {
        let cases = [
            "mcp_servers = {}\nmcp_servers = {}\n",
            "[mcp_servers.fixture]\ncommand = \"safe\"\ncommand = \"private-command\"\n",
            "[mcp_servers.fixture]\ncommand = \"safe\"\n[mcp_servers.fixture.env]\nTOKEN = \"one\"\nTOKEN = \"private-two\"\n",
        ];

        for text in cases {
            let error = CodexDocument::parse(text.as_bytes())
                .expect_err("duplicate TOML keys should be rejected");
            let output = format!("{error:?}\n{error}");
            assert!(matches!(error, CodexAdapterError::InvalidToml { .. }));
            assert!(!output.contains("private-command"));
            assert!(!output.contains("private-two"));
        }
    }

    #[test]
    fn diagnostics_and_debug_output_never_echo_native_values() {
        let document =
            CodexDocument::parse(CURRENT_FIXTURE).expect("current Codex fixture should be valid");
        let rendered = document
            .render_plan(&reconcile(document.canonical_config(), &desired_config()))
            .expect("fixture plan should render");
        let malformed = b"[mcp_servers.fixture]\ncommand = \"safe\"\nargs = \"fixture-codex-diagnostic-secret\"\n";
        let error = CodexDocument::parse(malformed)
            .expect_err("the malformed secret-bearing field should be rejected");
        let output = format!("{document:?}\n{rendered:?}\n{error:?}\n{error}");

        for secret in [
            "fixture-codex-unchanged-secret",
            "fixture-codex-old-secret",
            "fixture-codex-local-secret",
            "fixture-codex-remote-secret",
            "fixture-codex-added-secret",
            "fixture-codex-new-secret",
            "fixture-codex-diagnostic-secret",
            "https://codex.example.invalid/mcp",
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
        let document =
            CodexDocument::parse(CURRENT_FIXTURE).expect("current Codex fixture should be valid");
        let empty =
            CanonicalConfig::new(BTreeMap::new()).expect("an empty canonical config is valid");
        let plan = reconcile(&empty, document.canonical_config());

        let error = document
            .render_plan(&plan)
            .expect_err("an add plan must not overwrite existing native servers");

        assert!(matches!(
            error,
            CodexAdapterError::PlanPresenceMismatch {
                outcome: ReconciliationOutcomeKind::Add,
                present: true,
                ..
            }
        ));
    }

    #[test]
    fn rendering_rejects_a_plan_that_omits_current_local_servers() {
        let document =
            CodexDocument::parse(CURRENT_FIXTURE).expect("current Codex fixture should be valid");
        let empty =
            CanonicalConfig::new(BTreeMap::new()).expect("an empty canonical config is valid");
        let empty_plan = reconcile(&empty, &empty);

        let error = document
            .render_plan(&empty_plan)
            .expect_err("a partial plan must not bypass target-only preservation");

        assert!(matches!(
            error,
            CodexAdapterError::PlanOmittedCurrentServer { .. }
        ));
    }
}
