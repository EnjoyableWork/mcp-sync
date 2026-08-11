use crate::config::{CanonicalConfig, CanonicalServer, ConfigError};
use crate::filesystem::{FileIoError, FileSystem};
use crate::paths::ConfigurationPaths;
use crate::reconciliation::{ReconciliationOutcomeKind, ReconciliationPlan};
use jsonc_parser::ParseOptions;
use jsonc_parser::cst::{CstInputValue, CstNode, CstRootNode};
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

const SETTINGS_DIRECTORY: &str = "settings";
const KIRO_CONFIGURATION_FILE: &str = "mcp.json";
const MCP_SERVERS_FIELD: &str = "mcpServers";
const COMMAND_FIELD: &str = "command";
const ARGUMENTS_FIELD: &str = "args";
const ENVIRONMENT_FIELD: &str = "env";
const REMOTE_OR_DISCRIMINATOR_FIELDS: [&str; 5] =
    ["url", "headers", "oauth", "oauthScopes", "type"];

/// Kiro's selected global-user MCP target.
///
/// The default is `~/.kiro/settings/mcp.json`. An absolute, traversal-free
/// `KIRO_HOME` supplied through the injected path resolver selects
/// `<KIRO_HOME>/settings/mcp.json` instead. Workspace, agent, Kiro Crew,
/// organization-managed, remote-service, generated-agent, and credential
/// stores are deliberately outside discovery.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KiroAdapter {
    configuration_path: PathBuf,
}

impl KiroAdapter {
    pub fn from_paths(paths: &ConfigurationPaths) -> Self {
        Self {
            configuration_path: paths
                .kiro_home()
                .join(SETTINGS_DIRECTORY)
                .join(KIRO_CONFIGURATION_FILE),
        }
    }

    pub fn configuration_path(&self) -> &Path {
        &self.configuration_path
    }

    pub fn discover(
        &self,
        filesystem: &impl FileSystem,
    ) -> Result<KiroDiscovery, KiroAdapterError> {
        match filesystem.read(&self.configuration_path) {
            Ok(bytes) => KiroDocument::parse(&bytes).map(KiroDiscovery::Found),
            Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(KiroDiscovery::Missing),
            Err(source) => Err(KiroAdapterError::Read { source }),
        }
    }
}

#[derive(Debug)]
pub enum KiroDiscovery {
    Missing,
    Found(KiroDocument),
}

impl KiroDiscovery {
    #[cfg(test)]
    pub fn document(&self) -> Option<&KiroDocument> {
        match self {
            Self::Missing => None,
            Self::Found(document) => Some(document),
        }
    }

    pub fn into_document(self) -> KiroDocument {
        match self {
            Self::Missing => KiroDocument::empty(),
            Self::Found(document) => document,
        }
    }
}

/// A validated Kiro global-user document plus its exact original bytes.
///
/// Kiro currently parses comment-bearing JSON and expands valid
/// `${VARIABLE}` references recursively before launching a server. Canonical
/// v1 cannot preserve reference semantics, so any reference-bearing owned
/// field remains unmanaged. The same is true of mixed/remote transports,
/// malformed local shapes, non-string environment values, and owned arrays or
/// objects containing comments that a replacement could detach. For compatible
/// local STDIO entries, only `command`, `args`, and string-valued `env` are
/// owned. Every other field and every comment is retained through CST edits.
pub struct KiroDocument {
    root: Map<String, Value>,
    canonical_config: CanonicalConfig,
    unmanaged_server_names: BTreeSet<String>,
    original_comments: Vec<String>,
    original_bytes: Vec<u8>,
}

impl KiroDocument {
    pub fn parse(document: &[u8]) -> Result<Self, KiroAdapterError> {
        let text = std::str::from_utf8(document).map_err(|_| KiroAdapterError::InvalidEncoding)?;
        let cst = parse_cst(text)?;
        let value_node = cst.value().ok_or(KiroDocumentError::RootMustBeObject)?;
        validate_unique_properties(&value_node)?;
        let Value::Object(root) = value_node
            .to_serde_value()
            .ok_or(KiroDocumentError::RootMustBeObject)?
        else {
            return Err(KiroDocumentError::RootMustBeObject.into());
        };
        let root_cst = value_node
            .as_object()
            .ok_or(KiroDocumentError::RootMustBeObject)?;
        let servers_cst = match root_cst.get(MCP_SERVERS_FIELD) {
            Some(property) => Some(
                property
                    .object_value()
                    .ok_or(KiroDocumentError::McpServersMustBeObject)?,
            ),
            None => None,
        };

        let decoded = decode_servers(root.get(MCP_SERVERS_FIELD), servers_cst.as_ref())?;
        let canonical_config = CanonicalConfig::new(decoded.local_servers)
            .map_err(|source| KiroAdapterError::InvalidCanonical { source })?;

        Ok(Self {
            root,
            canonical_config,
            unmanaged_server_names: decoded.unmanaged_server_names,
            original_comments: collect_comments(&cst),
            original_bytes: document.to_vec(),
        })
    }

    fn empty() -> Self {
        Self::parse(b"{}\n").expect("an empty Kiro document is valid")
    }

    pub fn canonical_config(&self) -> &CanonicalConfig {
        &self.canonical_config
    }

    pub fn original_bytes(&self) -> &[u8] {
        &self.original_bytes
    }

    /// Native entries whose semantics canonical v1 cannot reproduce safely.
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
    ) -> Result<RenderedKiroDocument, KiroAdapterError> {
        self.validate_plan(plan)?;
        if !plan.requires_mutation() {
            return Ok(RenderedKiroDocument::new(
                self.original_bytes.clone(),
                false,
            ));
        }

        let text = std::str::from_utf8(&self.original_bytes)
            .map_err(|_| KiroAdapterError::InvalidEncoding)?;
        let cst = parse_cst(text)?;
        let root = cst
            .object_value()
            .ok_or(KiroDocumentError::RootMustBeObject)?;
        let servers = match root.object_value(MCP_SERVERS_FIELD) {
            Some(servers) => servers,
            None if root.get(MCP_SERVERS_FIELD).is_none() => {
                root.append(MCP_SERVERS_FIELD, CstInputValue::Object(Vec::new()));
                root.object_value(MCP_SERVERS_FIELD)
                    .expect("the appended MCP server map is an object")
            }
            None => return Err(KiroDocumentError::McpServersMustBeObject.into()),
        };

        for entry in plan.entries() {
            if !matches!(
                entry.outcome().kind(),
                ReconciliationOutcomeKind::Add | ReconciliationOutcomeKind::Update
            ) {
                continue;
            }

            let desired = entry.desired_server().ok_or_else(|| {
                KiroAdapterError::PlanMissingDesiredServer {
                    server: entry.server_name().to_owned(),
                }
            })?;
            let native_server = match servers.object_value(entry.server_name()) {
                Some(server) => server,
                None if servers.get(entry.server_name()).is_none() => {
                    servers.append(entry.server_name(), CstInputValue::Object(Vec::new()));
                    servers
                        .object_value(entry.server_name())
                        .expect("the appended Kiro server is an object")
                }
                None => {
                    return Err(KiroAdapterError::UnmanagedServerCollision {
                        server: entry.server_name().to_owned(),
                    });
                }
            };

            write_managed_fields(&native_server, desired);
        }

        let bytes = cst.to_string().into_bytes();
        let verified = Self::parse(&bytes)?;
        if verified.original_comments != self.original_comments {
            return Err(KiroAdapterError::CommentPreservationMismatch);
        }
        let changed = bytes != self.original_bytes;
        Ok(RenderedKiroDocument::new(bytes, changed))
    }

    fn validate_plan(&self, plan: &ReconciliationPlan) -> Result<(), KiroAdapterError> {
        let mut planned_names = BTreeSet::new();

        for entry in plan.entries() {
            planned_names.insert(entry.server_name());

            if self.unmanaged_server_names.contains(entry.server_name()) {
                return Err(KiroAdapterError::UnmanagedServerCollision {
                    server: entry.server_name().to_owned(),
                });
            }

            let present = self
                .canonical_config
                .servers()
                .contains_key(entry.server_name());
            let should_be_present = entry.outcome().kind() != ReconciliationOutcomeKind::Add;

            if present != should_be_present {
                return Err(KiroAdapterError::PlanPresenceMismatch {
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
                return Err(KiroAdapterError::PlanMissingDesiredServer {
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
            return Err(KiroAdapterError::PlanOmittedCurrentServer {
                server: server.clone(),
            });
        }

        Ok(())
    }
}

impl fmt::Debug for KiroDocument {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("KiroDocument")
            .field(
                "local_server_names",
                &self.canonical_config.servers().keys().collect::<Vec<_>>(),
            )
            .field("unmanaged_server_count", &self.unmanaged_server_names.len())
            .field("preserved_root_fields", &self.preserved_root_fields())
            .field("comment_count", &self.original_comments.len())
            .field("byte_count", &self.original_bytes.len())
            .finish()
    }
}

pub struct RenderedKiroDocument {
    bytes: Vec<u8>,
    changed: bool,
}

impl RenderedKiroDocument {
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

impl fmt::Debug for RenderedKiroDocument {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RenderedKiroDocument")
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

fn decode_servers(
    value: Option<&Value>,
    cst: Option<&jsonc_parser::cst::CstObject>,
) -> Result<DecodedServers, KiroAdapterError> {
    let Some(value) = value else {
        return Ok(DecodedServers {
            local_servers: BTreeMap::new(),
            unmanaged_server_names: BTreeSet::new(),
        });
    };
    let Value::Object(servers) = value else {
        return Err(KiroDocumentError::McpServersMustBeObject.into());
    };
    let cst = cst.ok_or(KiroDocumentError::McpServersMustBeObject)?;

    let mut local_servers = BTreeMap::new();
    let mut unmanaged_server_names = BTreeSet::new();

    for (name, value) in servers {
        let Some(server_cst) = cst.get(name).and_then(|property| property.object_value()) else {
            unmanaged_server_names.insert(name.clone());
            continue;
        };
        let Value::Object(fields) = value else {
            unmanaged_server_names.insert(name.clone());
            continue;
        };

        match decode_server(fields, &server_cst) {
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

fn decode_server(fields: &Map<String, Value>, cst: &jsonc_parser::cst::CstObject) -> DecodedServer {
    if REMOTE_OR_DISCRIMINATOR_FIELDS
        .iter()
        .any(|field| fields.contains_key(*field))
    {
        return DecodedServer::Unmanaged;
    }

    let Some(command) = fields.get(COMMAND_FIELD).and_then(Value::as_str) else {
        return DecodedServer::Unmanaged;
    };
    if command.is_empty() || contains_kiro_reference(command) {
        return DecodedServer::Unmanaged;
    }

    let arguments = match fields.get(ARGUMENTS_FIELD) {
        None => Vec::new(),
        Some(Value::Array(values)) => {
            let Some(value_node) = cst
                .get(ARGUMENTS_FIELD)
                .and_then(|property| property.value())
            else {
                return DecodedServer::Unmanaged;
            };
            if contains_comment(&value_node) {
                return DecodedServer::Unmanaged;
            }
            let mut arguments = Vec::with_capacity(values.len());
            for value in values {
                let Some(argument) = value.as_str() else {
                    return DecodedServer::Unmanaged;
                };
                if contains_kiro_reference(argument) {
                    return DecodedServer::Unmanaged;
                }
                arguments.push(argument.to_owned());
            }
            arguments
        }
        Some(_) => return DecodedServer::Unmanaged,
    };

    let environment = match fields.get(ENVIRONMENT_FIELD) {
        None => BTreeMap::new(),
        Some(Value::Object(values)) => {
            let Some(value_node) = cst
                .get(ENVIRONMENT_FIELD)
                .and_then(|property| property.value())
            else {
                return DecodedServer::Unmanaged;
            };
            if contains_comment(&value_node) {
                return DecodedServer::Unmanaged;
            }
            let mut environment = BTreeMap::new();
            for (key, value) in values {
                let Some(value) = value.as_str() else {
                    return DecodedServer::Unmanaged;
                };
                if contains_kiro_reference(value) {
                    return DecodedServer::Unmanaged;
                }
                environment.insert(key.clone(), value.to_owned());
            }
            environment
        }
        Some(_) => return DecodedServer::Unmanaged,
    };

    DecodedServer::Local(CanonicalServer::new(
        command.to_owned(),
        arguments,
        environment,
    ))
}

fn write_managed_fields(fields: &jsonc_parser::cst::CstObject, server: &CanonicalServer) {
    set_or_append(
        fields,
        COMMAND_FIELD,
        CstInputValue::String(server.command().to_owned()),
    );
    set_or_append(
        fields,
        ARGUMENTS_FIELD,
        CstInputValue::Array(
            server
                .args()
                .iter()
                .cloned()
                .map(CstInputValue::String)
                .collect(),
        ),
    );
    set_or_append(
        fields,
        ENVIRONMENT_FIELD,
        CstInputValue::Object(
            server
                .env()
                .iter()
                .map(|(key, value)| (key.clone(), CstInputValue::String(value.clone())))
                .collect(),
        ),
    );
}

fn set_or_append(object: &jsonc_parser::cst::CstObject, name: &str, value: CstInputValue) {
    match object.get(name) {
        Some(property) => property.set_value(value),
        None => {
            object.append(name, value);
        }
    }
}

fn parse_cst(text: &str) -> Result<CstRootNode, KiroAdapterError> {
    CstRootNode::parse(
        text,
        &ParseOptions {
            allow_comments: true,
            allow_loose_object_property_names: false,
            allow_trailing_commas: true,
            allow_missing_commas: false,
            allow_single_quoted_strings: false,
            allow_hexadecimal_numbers: false,
            allow_unary_plus_numbers: false,
        },
    )
    .map_err(|error| KiroAdapterError::InvalidJson {
        message: error.to_string(),
    })
}

fn validate_unique_properties(node: &CstNode) -> Result<(), KiroDocumentError> {
    if let Some(object) = node.as_object() {
        let mut names = BTreeSet::new();
        for property in object.properties() {
            let name = property
                .name()
                .ok_or(KiroDocumentError::MalformedProperty)?
                .decoded_value()
                .map_err(|_| KiroDocumentError::MalformedProperty)?;
            if !names.insert(name) {
                return Err(KiroDocumentError::DuplicateProperty);
            }
            if let Some(value) = property.value() {
                validate_unique_properties(&value)?;
            }
        }
    } else if let Some(array) = node.as_array() {
        for element in array.elements() {
            validate_unique_properties(&element)?;
        }
    }
    Ok(())
}

fn contains_comment(node: &CstNode) -> bool {
    node.is_comment() || node.children().iter().any(contains_comment)
}

fn collect_comments(root: &CstRootNode) -> Vec<String> {
    let mut comments = Vec::new();
    for child in root.children() {
        collect_node_comments(&child, &mut comments);
    }
    comments
}

fn collect_node_comments(node: &CstNode, comments: &mut Vec<String>) {
    if node.is_comment() {
        comments.push(node.to_string());
    }
    for child in node.children() {
        collect_node_comments(&child, comments);
    }
}

fn contains_kiro_reference(value: &str) -> bool {
    let bytes = value.as_bytes();
    let mut index = 0;

    while index + 3 < bytes.len() {
        if bytes[index] != b'$' || bytes[index + 1] != b'{' {
            index += 1;
            continue;
        }

        let first = bytes[index + 2];
        if !first.is_ascii_alphabetic() && first != b'_' {
            index += 2;
            continue;
        }

        let mut cursor = index + 3;
        while cursor < bytes.len()
            && (bytes[cursor].is_ascii_alphanumeric() || bytes[cursor] == b'_')
        {
            cursor += 1;
        }
        if cursor < bytes.len() && bytes[cursor] == b'}' {
            return true;
        }
        index += 2;
    }

    false
}

#[derive(Debug)]
pub enum KiroAdapterError {
    Read {
        source: FileIoError,
    },
    InvalidEncoding,
    InvalidJson {
        message: String,
    },
    InvalidDocument(KiroDocumentError),
    InvalidCanonical {
        source: ConfigError,
    },
    CommentPreservationMismatch,
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

impl fmt::Display for KiroAdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { source } => source.fmt(formatter),
            Self::InvalidEncoding => formatter.write_str("Kiro configuration is not valid UTF-8"),
            Self::InvalidJson { message } => write!(formatter, "invalid Kiro JSON: {message}"),
            Self::InvalidDocument(error) => error.fmt(formatter),
            Self::InvalidCanonical { source } => {
                write!(formatter, "invalid local Kiro server definition: {source}")
            }
            Self::CommentPreservationMismatch => formatter
                .write_str("Kiro render could not preserve every existing document comment"),
            Self::UnmanagedServerCollision { server } => write!(
                formatter,
                "cannot reconcile local server {server:?} because Kiro already has an unmanaged entry with that name"
            ),
            Self::PlanPresenceMismatch {
                server,
                outcome,
                present,
            } => write!(
                formatter,
                "Kiro plan for {server:?} has {outcome:?} outcome inconsistent with current presence {present}"
            ),
            Self::PlanOmittedCurrentServer { server } => write!(
                formatter,
                "Kiro plan omitted current local server {server:?}"
            ),
            Self::PlanMissingDesiredServer { server } => write!(
                formatter,
                "Kiro plan for {server:?} requires a desired local definition"
            ),
        }
    }
}

impl Error for KiroAdapterError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Read { source } => Some(source),
            Self::InvalidDocument(source) => Some(source),
            Self::InvalidCanonical { source } => Some(source),
            Self::InvalidEncoding
            | Self::InvalidJson { .. }
            | Self::CommentPreservationMismatch
            | Self::UnmanagedServerCollision { .. }
            | Self::PlanPresenceMismatch { .. }
            | Self::PlanOmittedCurrentServer { .. }
            | Self::PlanMissingDesiredServer { .. } => None,
        }
    }
}

impl From<KiroDocumentError> for KiroAdapterError {
    fn from(error: KiroDocumentError) -> Self {
        Self::InvalidDocument(error)
    }
}

#[derive(Debug)]
pub enum KiroDocumentError {
    RootMustBeObject,
    McpServersMustBeObject,
    DuplicateProperty,
    MalformedProperty,
}

impl fmt::Display for KiroDocumentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RootMustBeObject => {
                formatter.write_str("Kiro configuration root must be a JSON object")
            }
            Self::McpServersMustBeObject => {
                formatter.write_str("Kiro `mcpServers` must be a JSON object")
            }
            Self::DuplicateProperty => {
                formatter.write_str("Kiro JSON must not contain duplicate object properties")
            }
            Self::MalformedProperty => {
                formatter.write_str("Kiro JSON contains a malformed object property")
            }
        }
    }
}

impl Error for KiroDocumentError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths::{Environment, Platform};
    use crate::reconciliation::{ReconciliationOutcomeKind, reconcile};
    use std::ffi::OsString;

    const CURRENT_FIXTURE: &[u8] = include_bytes!("../tests/fixtures/kiro/current.json");
    const DESIRED_FIXTURE: &str = include_str!("../tests/fixtures/kiro/desired.json");
    const MERGED_FIXTURE: &[u8] = include_bytes!("../tests/fixtures/kiro/merged.json");

    struct FixtureEnvironment {
        home: PathBuf,
        kiro_home: Option<PathBuf>,
    }

    impl Environment for FixtureEnvironment {
        fn value(&self, name: &'static str) -> Option<OsString> {
            let path = match name {
                "HOME" | "USERPROFILE" => self.home.clone(),
                "LOCALAPPDATA" => self.home.join("AppData/Local"),
                "APPDATA" => self.home.join("AppData/Roaming"),
                "KIRO_HOME" => return self.kiro_home.clone().map(PathBuf::into_os_string),
                _ => return None,
            };
            Some(path.into_os_string())
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
                    io::Error::new(*kind, "synthetic Kiro read failure"),
                )),
            }
        }
    }

    fn adapter_fixture_for(
        platform: Platform,
        relocated: bool,
    ) -> (tempfile::TempDir, KiroAdapter) {
        let root = tempfile::tempdir().expect("temporary Kiro fixture should be created");
        let home = root.path().join("user");
        let kiro_home = relocated.then(|| root.path().join("kiro-profile"));
        let paths =
            ConfigurationPaths::resolve_for(platform, &FixtureEnvironment { home, kiro_home })
                .expect("synthetic Kiro paths should resolve");
        (root, KiroAdapter::from_paths(&paths))
    }

    fn desired_fixture() -> CanonicalConfig {
        CanonicalConfig::parse_json(DESIRED_FIXTURE)
            .expect("the desired Kiro fixture should be canonical")
    }

    fn config(entries: Vec<(&str, CanonicalServer)>) -> CanonicalConfig {
        CanonicalConfig::new(
            entries
                .into_iter()
                .map(|(name, server)| (name.to_owned(), server))
                .collect(),
        )
        .expect("the synthetic canonical configuration should be valid")
    }

    fn server(command: &str, args: &[&str], env: &[(&str, &str)]) -> CanonicalServer {
        CanonicalServer::new(
            command.to_owned(),
            args.iter().map(|value| (*value).to_owned()).collect(),
            env.iter()
                .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
                .collect(),
        )
    }

    #[test]
    fn every_platform_resolves_only_the_default_global_user_file() {
        for platform in [Platform::MacOs, Platform::Linux, Platform::Windows] {
            let (root, adapter) = adapter_fixture_for(platform, false);
            assert_eq!(
                adapter.configuration_path(),
                root.path().join("user/.kiro/settings/mcp.json")
            );
            assert!(adapter.configuration_path().starts_with(root.path()));
        }
    }

    #[test]
    fn every_platform_resolves_the_injected_kiro_home_file() {
        for platform in [Platform::MacOs, Platform::Linux, Platform::Windows] {
            let (root, adapter) = adapter_fixture_for(platform, true);
            assert_eq!(
                adapter.configuration_path(),
                root.path().join("kiro-profile/settings/mcp.json")
            );
            assert!(adapter.configuration_path().starts_with(root.path()));
        }
    }

    #[test]
    fn discovery_reads_only_the_selected_global_file_and_handles_absence() {
        let (_root, adapter) = adapter_fixture_for(Platform::MacOs, false);
        let found = adapter
            .discover(&FixtureFileSystem {
                expected_path: adapter.configuration_path().to_owned(),
                result: Ok(CURRENT_FIXTURE.to_vec()),
            })
            .expect("the selected global fixture should be discovered");
        let document = found.document().expect("the fixture should be present");
        assert_eq!(document.canonical_config().servers().len(), 3);
        assert_eq!(
            document.unmanaged_server_names(),
            ["numeric-env", "reference-only", "remote-only"]
        );

        let missing = adapter
            .discover(&FixtureFileSystem {
                expected_path: adapter.configuration_path().to_owned(),
                result: Err(io::ErrorKind::NotFound),
            })
            .expect("a missing selected global file is an empty state");
        assert!(missing.document().is_none());
        assert!(
            missing
                .into_document()
                .canonical_config()
                .servers()
                .is_empty()
        );
    }

    #[test]
    fn discovery_preserves_non_missing_read_failures() {
        let (_root, adapter) = adapter_fixture_for(Platform::Linux, true);
        let error = adapter
            .discover(&FixtureFileSystem {
                expected_path: adapter.configuration_path().to_owned(),
                result: Err(io::ErrorKind::PermissionDenied),
            })
            .expect_err("a selected global read failure should remain contextual");
        let KiroAdapterError::Read { source } = error else {
            panic!("the filesystem failure should remain a Kiro read error");
        };
        assert_eq!(source.kind(), io::ErrorKind::PermissionDenied);
        assert_eq!(source.path(), adapter.configuration_path());
    }

    #[test]
    fn fixture_plan_is_deterministic_and_preserves_comments_and_unmanaged_data() {
        let document =
            KiroDocument::parse(CURRENT_FIXTURE).expect("the current Kiro fixture should parse");
        let desired = desired_fixture();
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
            .expect("the fixture plan should render");
        let second = document
            .render_plan(&plan)
            .expect("the fixture plan should render deterministically");
        assert!(first.changed());
        assert_eq!(first.bytes(), second.bytes());
        assert_eq!(first.bytes(), MERGED_FIXTURE);

        let merged =
            KiroDocument::parse(first.bytes()).expect("the merged Kiro fixture should parse again");
        assert_eq!(
            merged.unmanaged_server_names(),
            ["numeric-env", "reference-only", "remote-only"]
        );
        assert_eq!(merged.preserved_root_fields(), ["futureTopLevel"]);
        assert_eq!(
            merged.preserved_server_fields("updated").unwrap(),
            ["autoApprove", "cwd", "disabledTools", "futureOption"]
        );
        let text = std::str::from_utf8(first.bytes()).expect("merged bytes should be UTF-8");
        assert!(text.contains("// Kiro global-user fixture comment"));
        assert!(text.contains("// compatible local entries"));

        let settled = reconcile(merged.canonical_config(), &desired);
        assert!(!settled.requires_mutation());
        assert!(settled.has_drift());
        assert_eq!(
            merged
                .render_plan(&settled)
                .expect("the settled plan should remain exact")
                .bytes(),
            first.bytes()
        );
    }

    #[test]
    fn parses_compatible_local_entries_and_preserves_unowned_structure() {
        let document = KiroDocument::parse(
            br#"{
  // root comment
  "future": { "private": "value" },
  "mcpServers": {
    "local": {
      "command": "runner",
      "args": ["--stdio"],
      "env": { "TOKEN": "private" },
      "disabled": true,
      "autoApprove": ["read"],
      "disabledTools": ["write"]
    }
  }
}
"#,
        )
        .expect("the Kiro JSONC document should parse");

        assert_eq!(
            document.canonical_config(),
            &config(vec![(
                "local",
                server("runner", &["--stdio"], &[("TOKEN", "private")])
            )])
        );
        assert_eq!(document.unmanaged_server_names(), Vec::<&str>::new());
        assert_eq!(document.preserved_root_fields(), vec!["future"]);
        assert_eq!(
            document.preserved_server_fields("local"),
            Some(vec!["autoApprove", "disabled", "disabledTools"])
        );
    }

    #[test]
    fn classifies_every_unrepresentable_entry_without_exposing_values() {
        let document = KiroDocument::parse(
            br#"{
  "mcpServers": {
    "remote": { "url": "https://private.invalid", "headers": { "Authorization": "private" } },
    "mixed": { "command": "runner", "url": "https://private.invalid" },
    "registry": { "type": "registry", "command": "runner" },
    "reference-command": { "command": "${RUNNER}" },
    "reference-arg": { "command": "runner", "args": ["${ARG}"] },
    "reference-env": { "command": "runner", "env": { "TOKEN": "${TOKEN}" } },
    "numeric-env": { "command": "runner", "env": { "PORT": 42 } },
    "commented-args": { "command": "runner", "args": [/* semantic */ "one"] },
    "commented-env": { "command": "runner", "env": { /* semantic */ "A": "one" } },
    "bad-command": { "command": 42 },
    "bad-args": { "command": "runner", "args": "one" },
    "opaque": "private"
  }
}
"#,
        )
        .expect("unrepresentable entries should be preserved rather than rejected");

        assert!(document.canonical_config().servers().is_empty());
        assert_eq!(
            document.unmanaged_server_names(),
            vec![
                "bad-args",
                "bad-command",
                "commented-args",
                "commented-env",
                "mixed",
                "numeric-env",
                "opaque",
                "reference-arg",
                "reference-command",
                "reference-env",
                "registry",
                "remote",
            ]
        );
        let debug = format!("{document:?}");
        assert!(!debug.contains("private.invalid"));
        assert!(!debug.contains("Authorization"));
        assert!(!debug.contains("${TOKEN}"));
    }

    #[test]
    fn valid_reference_detection_matches_kiro_identifier_rules() {
        for value in ["${A}", "prefix-${_A2}-suffix", "${long_name_3}"] {
            assert!(contains_kiro_reference(value), "{value:?} should match");
        }
        for value in ["$A", "${}", "${2A}", "${A-B}", "${A", "literal"] {
            assert!(
                !contains_kiro_reference(value),
                "{value:?} should remain literal"
            );
        }
    }

    #[test]
    fn render_updates_only_owned_fields_and_preserves_every_comment() {
        let original = br#"{
  // root comment
  "future": { "private": "root" },
  "mcpServers": {
    // server comment
    "local": {
      "command" /* command key */: /* command value */ "old", // command tail
      "args": ["old"],
      "env": { "OLD": "private" },
      "disabled": true,
      "unknown": { "private": "server" }
    },
    "remote": { "url": "https://private.invalid", "headers": { "Authorization": "private" } }
  }
}
"#;
        let document = KiroDocument::parse(original).expect("the source should parse");
        let desired = config(vec![
            (
                "local",
                server("new", &["--new"], &[("NEW", "private-new")]),
            ),
            ("added", server("add", &[], &[])),
        ]);
        let plan = reconcile(document.canonical_config(), &desired);

        let rendered = document
            .render_plan(&plan)
            .expect("the plan should render without losing comments");
        assert!(rendered.changed());
        let text = std::str::from_utf8(rendered.bytes()).expect("rendered JSON is UTF-8");
        for comment in [
            "// root comment",
            "// server comment",
            "/* command key */",
            "/* command value */",
            "// command tail",
        ] {
            assert!(text.contains(comment), "missing preserved {comment}");
        }

        let parsed = KiroDocument::parse(rendered.bytes()).expect("rendered JSON should reparse");
        assert_eq!(parsed.canonical_config(), &desired);
        assert_eq!(parsed.unmanaged_server_names(), vec!["remote"]);
        let value = parsed.root.get(MCP_SERVERS_FIELD).unwrap();
        assert_eq!(value["local"]["disabled"], Value::Bool(true));
        assert_eq!(value["local"]["unknown"]["private"], "server");
        assert_eq!(value["remote"]["headers"]["Authorization"], "private");
        assert_eq!(parsed.root["future"]["private"], "root");
    }

    #[test]
    fn semantic_no_op_returns_the_exact_original_bytes() {
        let original = b"{\n  // exact\n  \"mcpServers\": {\n    \"local\": {\"command\": \"runner\"}\n  }\n}\n";
        let document = KiroDocument::parse(original).expect("the source should parse");
        let desired = document.canonical_config().clone();
        let plan = reconcile(document.canonical_config(), &desired);

        let rendered = document
            .render_plan(&plan)
            .expect("the no-op should render");

        assert!(!rendered.changed());
        assert_eq!(rendered.bytes(), original);
    }

    #[test]
    fn unmanaged_name_collision_is_rejected_before_rendering_values() {
        let document = KiroDocument::parse(
            br#"{"mcpServers":{"collision":{"url":"https://private.invalid","headers":{"Authorization":"private"}}}}"#,
        )
        .expect("the remote entry should parse");
        let desired = config(vec![("collision", server("private-command", &[], &[]))]);
        let plan = reconcile(document.canonical_config(), &desired);

        let error = document
            .render_plan(&plan)
            .expect_err("the unmanaged collision should fail");
        let diagnostic = error.to_string();

        assert!(diagnostic.contains("collision"));
        assert!(!diagnostic.contains("private.invalid"));
        assert!(!diagnostic.contains("Authorization"));
        assert!(!diagnostic.contains("private-command"));
    }

    #[test]
    fn rejects_duplicate_properties_and_non_object_roots() {
        for source in [
            br#"{"mcpServers":{},"mcpServers":{}}"#.as_slice(),
            br#"{"mcpServers":{"a":{"command":"one","command":"two"}}}"#.as_slice(),
            br#"[]"#.as_slice(),
            br#"{"mcpServers":[]}"#.as_slice(),
        ] {
            assert!(KiroDocument::parse(source).is_err());
        }
    }

    #[test]
    fn parser_matches_kiro_json_with_comments_boundary() {
        for accepted in [
            br#"{/* comment */"mcpServers":{},}"#.as_slice(),
            br#"{"mcpServers":{"a":{"command":"runner","args":[],},},}"#.as_slice(),
        ] {
            KiroDocument::parse(accepted).expect("comments and trailing commas should parse");
        }

        for rejected in [
            br#"{'mcpServers':{}}"#.as_slice(),
            br#"{mcpServers:{}}"#.as_slice(),
            br#"{"mcpServers":{} "other":true}"#.as_slice(),
            br#"{"number":+1}"#.as_slice(),
            br#"{"number":0x10}"#.as_slice(),
        ] {
            assert!(KiroDocument::parse(rejected).is_err());
        }
    }
}
