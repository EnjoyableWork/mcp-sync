use crate::config::{
    CanonicalConfig, CanonicalServer, ConfigError, ValidationError, validate_environment_name,
};
use crate::filesystem::{FileIoError, FileMutationError, FileReplacer, FileSystem};
use crate::paths::ConfigurationPaths;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::io;
use std::path::PathBuf;

/// A complete canonical server definition supplied by `mcp-sync add`.
///
/// Environment assignments remain raw until the application boundary can
/// reject ambiguous or duplicate keys without exposing their values.
pub struct AddRequest {
    name: String,
    command: String,
    arguments: Vec<String>,
    environment_assignments: Vec<String>,
}

impl AddRequest {
    pub fn new(
        name: String,
        command: String,
        arguments: Vec<String>,
        environment_assignments: Vec<String>,
    ) -> Self {
        Self {
            name,
            command,
            arguments,
            environment_assignments,
        }
    }

    /// Validate all user-controlled fields before any filesystem boundary is
    /// reached. The CLI acquires its cross-process mutation lock only after
    /// this step succeeds.
    pub fn validate(self) -> Result<ValidatedAddRequest, CatalogError> {
        let environment = parse_environment(&self.name, self.environment_assignments)?;
        let server = CanonicalServer::new(self.command, self.arguments, environment);
        CanonicalConfig::new(BTreeMap::from([(self.name.clone(), server.clone())]))
            .map_err(|source| CatalogError::InvalidRequestedDefinition { source })?;
        Ok(ValidatedAddRequest {
            name: self.name,
            server,
        })
    }
}

impl fmt::Debug for AddRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AddRequest")
            .field("name", &self.name)
            .field("command", &"<redacted>")
            .field("argument_count", &self.arguments.len())
            .field(
                "environment_assignment_count",
                &self.environment_assignments.len(),
            )
            .finish()
    }
}

pub struct ValidatedAddRequest {
    name: String,
    server: CanonicalServer,
}

impl fmt::Debug for ValidatedAddRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ValidatedAddRequest")
            .field("name", &self.name)
            .field("command", &"<redacted>")
            .field("argument_count", &self.server.args().len())
            .field("environment_key_count", &self.server.env().len())
            .finish()
    }
}

/// Add or replace one complete named definition in canonical state.
///
/// An exact match is a semantic no-op and never reaches the replacement port.
/// Every actual change is validated and serialized before the atomic boundary
/// receives the original and desired bytes together.
pub fn add_server(
    paths: &ConfigurationPaths,
    filesystem: &(impl FileSystem + FileReplacer),
    request: ValidatedAddRequest,
) -> Result<AddReport, CatalogError> {
    let loaded = load_canonical(paths, filesystem)?;
    let outcome = match loaded.config.servers().get(&request.name) {
        Some(current) if current == &request.server => UpsertOutcome::Unchanged,
        Some(_) => UpsertOutcome::Updated,
        None => UpsertOutcome::Added,
    };

    let report = AddReport::new(outcome, &request.name, &request.server);
    if outcome == UpsertOutcome::Unchanged {
        return Ok(report);
    }

    let mut servers = loaded.config.servers().clone();
    servers.insert(request.name, request.server);
    let desired = CanonicalConfig::new(servers)
        .map_err(|source| CatalogError::InvalidRequestedDefinition { source })?;
    let desired_bytes = desired
        .to_canonical_json()
        .map_err(|source| CatalogError::Serialize { source })?;

    filesystem
        .replace_if_unchanged(
            paths.canonical_configuration(),
            &loaded.bytes,
            desired_bytes.as_bytes(),
        )
        .map_err(|source| CatalogError::Replace { source })?;
    Ok(report)
}

/// Read deterministic, structurally redacted summaries from canonical state.
pub fn list_servers(
    paths: &ConfigurationPaths,
    filesystem: &impl FileSystem,
) -> Result<ListReport, CatalogError> {
    let loaded = load_canonical(paths, filesystem)?;
    let servers = loaded
        .config
        .servers()
        .iter()
        .map(|(name, server)| ServerSummary {
            name: name.clone(),
            argument_count: server.args().len(),
            environment_keys: server.env().keys().cloned().collect(),
        })
        .collect();
    Ok(ListReport { servers })
}

struct LoadedCanonical {
    bytes: Vec<u8>,
    config: CanonicalConfig,
}

fn load_canonical(
    paths: &ConfigurationPaths,
    filesystem: &impl FileSystem,
) -> Result<LoadedCanonical, CatalogError> {
    let path = paths.canonical_configuration();
    let bytes = match filesystem.read(path) {
        Ok(bytes) => bytes,
        Err(source) if source.kind() == io::ErrorKind::NotFound => {
            return Err(CatalogError::NotInitialized {
                path: path.to_owned(),
            });
        }
        Err(source) => return Err(CatalogError::Read { source }),
    };
    let document = std::str::from_utf8(&bytes).map_err(|_| CatalogError::InvalidEncoding {
        path: path.to_owned(),
    })?;
    let config =
        CanonicalConfig::parse_json(document).map_err(|source| CatalogError::InvalidCanonical {
            path: path.to_owned(),
            source,
        })?;
    Ok(LoadedCanonical { bytes, config })
}

fn parse_environment(
    server: &str,
    assignments: Vec<String>,
) -> Result<BTreeMap<String, String>, CatalogError> {
    let mut environment = BTreeMap::new();
    for (index, assignment) in assignments.into_iter().enumerate() {
        let position = index + 1;
        let Some((key, value)) = assignment.split_once('=') else {
            return Err(EnvironmentAssignmentError::MissingSeparator { position }.into());
        };
        if let Err(violation) = validate_environment_name(key) {
            return Err(CatalogError::InvalidRequestedDefinition {
                source: ConfigError::InvalidModel(ValidationError::InvalidEnvironmentName {
                    server: server.to_owned(),
                    position: index,
                    violation,
                }),
            });
        }
        if environment.contains_key(key) {
            return Err(EnvironmentAssignmentError::DuplicateKey {
                key: key.to_owned(),
            }
            .into());
        }
        environment.insert(key.to_owned(), value.to_owned());
    }
    Ok(environment)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UpsertOutcome {
    Added,
    Updated,
    Unchanged,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AddReport {
    outcome: UpsertOutcome,
    name: String,
    argument_count: usize,
    environment_key_count: usize,
}

impl AddReport {
    fn new(outcome: UpsertOutcome, name: &str, server: &CanonicalServer) -> Self {
        Self {
            outcome,
            name: name.to_owned(),
            argument_count: server.args().len(),
            environment_key_count: server.env().len(),
        }
    }
}

impl fmt::Display for AddReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.outcome {
            UpsertOutcome::Added | UpsertOutcome::Updated => {
                let action = if self.outcome == UpsertOutcome::Added {
                    "Added"
                } else {
                    "Updated"
                };
                write!(
                    formatter,
                    "{action} server {:?} (command: <redacted>; arguments: {}; environment keys: {}).",
                    self.name, self.argument_count, self.environment_key_count
                )
            }
            UpsertOutcome::Unchanged => write!(
                formatter,
                "Server {:?} is already up to date (command: <redacted>; arguments: {}; environment keys: {}).",
                self.name, self.argument_count, self.environment_key_count
            ),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ServerSummary {
    name: String,
    argument_count: usize,
    environment_keys: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListReport {
    servers: Vec<ServerSummary>,
}

impl fmt::Display for ListReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.servers.is_empty() {
            return formatter.write_str("No servers configured.");
        }

        for (index, server) in self.servers.iter().enumerate() {
            if index > 0 {
                formatter.write_str("\n")?;
            }
            let environment_keys = if server.environment_keys.is_empty() {
                "none".to_owned()
            } else {
                server
                    .environment_keys
                    .iter()
                    .map(|key| format!("{key:?}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            write!(
                formatter,
                "{:?}: command: <redacted>; arguments: {}; environment keys: {environment_keys}",
                server.name, server.argument_count
            )?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EnvironmentAssignmentError {
    MissingSeparator { position: usize },
    DuplicateKey { key: String },
}

impl fmt::Display for EnvironmentAssignmentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSeparator { position } => write!(
                formatter,
                "environment assignment {position} must use `KEY=VALUE`; no value was written"
            ),
            Self::DuplicateKey { key } => write!(
                formatter,
                "environment key {key:?} was supplied more than once; provide each key once"
            ),
        }
    }
}

impl Error for EnvironmentAssignmentError {}

#[derive(Debug)]
pub enum CatalogError {
    NotInitialized { path: PathBuf },
    Read { source: FileIoError },
    InvalidEncoding { path: PathBuf },
    InvalidCanonical { path: PathBuf, source: ConfigError },
    InvalidEnvironment { source: EnvironmentAssignmentError },
    InvalidRequestedDefinition { source: ConfigError },
    Serialize { source: ConfigError },
    Replace { source: FileMutationError },
}

impl fmt::Display for CatalogError {
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
            Self::InvalidEnvironment { source } => source.fmt(formatter),
            Self::InvalidRequestedDefinition { source } => {
                write!(
                    formatter,
                    "cannot add the requested server definition: {source}"
                )
            }
            Self::Serialize { source } => {
                write!(
                    formatter,
                    "cannot serialize canonical configuration update: {source}"
                )
            }
            Self::Replace { source } => {
                write!(formatter, "cannot update canonical configuration: {source}")
            }
        }
    }
}

impl Error for CatalogError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::NotInitialized { .. } | Self::InvalidEncoding { .. } => None,
            Self::Read { source } => Some(source),
            Self::InvalidCanonical { source, .. }
            | Self::InvalidRequestedDefinition { source }
            | Self::Serialize { source } => Some(source),
            Self::InvalidEnvironment { source } => Some(source),
            Self::Replace { source } => Some(source),
        }
    }
}

impl From<EnvironmentAssignmentError> for CatalogError {
    fn from(source: EnvironmentAssignmentError) -> Self {
        Self::InvalidEnvironment { source }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::EnvironmentNameViolation;
    use crate::filesystem::FileMutationError;
    use crate::paths::Environment;
    use std::cell::{Cell, RefCell};
    use std::ffi::OsString;
    use std::path::Path;

    struct FixtureEnvironment(PathBuf);

    impl Environment for FixtureEnvironment {
        fn value(&self, name: &'static str) -> Option<OsString> {
            (name == "HOME").then(|| self.0.clone().into_os_string())
        }
    }

    struct RecordingFileSystem {
        current: Vec<u8>,
        reads: Cell<usize>,
        replacements: RefCell<Vec<(Vec<u8>, Vec<u8>)>>,
        reject_replacement: bool,
    }

    impl RecordingFileSystem {
        fn new(current: Vec<u8>) -> Self {
            Self {
                current,
                reads: Cell::new(0),
                replacements: RefCell::new(Vec::new()),
                reject_replacement: false,
            }
        }

        fn rejecting_replacement(current: Vec<u8>) -> Self {
            Self {
                reject_replacement: true,
                ..Self::new(current)
            }
        }
    }

    impl FileSystem for RecordingFileSystem {
        fn read(&self, _path: &Path) -> Result<Vec<u8>, FileIoError> {
            self.reads.set(self.reads.get() + 1);
            Ok(self.current.clone())
        }
    }

    impl FileReplacer for RecordingFileSystem {
        fn replace_if_unchanged(
            &self,
            _path: &Path,
            expected_current: &[u8],
            replacement: &[u8],
        ) -> Result<(), FileMutationError> {
            self.replacements
                .borrow_mut()
                .push((expected_current.to_vec(), replacement.to_vec()));
            if self.reject_replacement {
                Err(FileMutationError::ConcurrentModification {
                    path: PathBuf::from("synthetic/config.json"),
                })
            } else {
                Ok(())
            }
        }
    }

    fn paths() -> (tempfile::TempDir, ConfigurationPaths) {
        let root = tempfile::tempdir().expect("a temporary fixture should be created");
        let paths = ConfigurationPaths::resolve_for(
            crate::paths::Platform::MacOs,
            &FixtureEnvironment(root.path().join("synthetic-user")),
        )
        .expect("synthetic paths should resolve");
        (root, paths)
    }

    fn canonical(entries: Vec<(&str, CanonicalServer)>) -> Vec<u8> {
        CanonicalConfig::new(
            entries
                .into_iter()
                .map(|(name, server)| (name.to_owned(), server))
                .collect(),
        )
        .expect("the fixture config should be valid")
        .to_canonical_json()
        .expect("the fixture config should serialize")
        .into_bytes()
    }

    #[test]
    fn environment_assignments_preserve_embedded_equals_and_empty_values() {
        let environment = parse_environment(
            "alpha",
            vec![
                "EMPTY=".to_owned(),
                "URL=scheme://host/path?left=right".to_owned(),
            ],
        )
        .expect("literal assignments should parse");

        assert_eq!(environment.get("EMPTY").map(String::as_str), Some(""));
        assert_eq!(
            environment.get("URL").map(String::as_str),
            Some("scheme://host/path?left=right")
        );
    }

    #[test]
    fn invalid_environment_diagnostics_never_echo_values() {
        let private_value = "must-not-appear";
        let missing = parse_environment("alpha", vec![private_value.to_owned()])
            .expect_err("an assignment without equals should fail")
            .to_string();
        let duplicate = parse_environment(
            "alpha",
            vec![
                format!("TOKEN={private_value}"),
                "TOKEN=second-must-not-appear".to_owned(),
            ],
        )
        .expect_err("a duplicate key should fail")
        .to_string();

        assert_eq!(
            missing,
            "environment assignment 1 must use `KEY=VALUE`; no value was written"
        );
        assert_eq!(
            duplicate,
            "environment key \"TOKEN\" was supplied more than once; provide each key once"
        );
        assert!(!missing.contains(private_value));
        assert!(!duplicate.contains(private_value));
        assert!(!duplicate.contains("second-must-not-appear"));
    }

    #[test]
    fn add_request_debug_output_contains_only_structural_metadata() {
        let request = AddRequest::new(
            "alpha".to_owned(),
            "private-command".to_owned(),
            vec!["private-argument".to_owned()],
            vec!["TOKEN=private-value".to_owned()],
        );

        let debug = format!("{request:?}");

        assert!(debug.contains("name: \"alpha\""));
        assert!(debug.contains("command: \"<redacted>\""));
        assert!(debug.contains("argument_count: 1"));
        assert!(debug.contains("environment_assignment_count: 1"));
        for private_value in ["private-command", "private-argument", "private-value"] {
            assert!(!debug.contains(private_value));
        }

        let validated = AddRequest::new(
            "alpha".to_owned(),
            "private-command".to_owned(),
            vec!["private-argument".to_owned()],
            vec!["TOKEN=private-value".to_owned()],
        )
        .validate()
        .expect("the redaction fixture should validate");
        let validated_debug = format!("{validated:?}");

        assert!(validated_debug.contains("name: \"alpha\""));
        assert!(validated_debug.contains("command: \"<redacted>\""));
        assert!(validated_debug.contains("argument_count: 1"));
        assert!(validated_debug.contains("environment_key_count: 1"));
        for private_value in ["private-command", "private-argument", "private-value"] {
            assert!(!validated_debug.contains(private_value));
        }
    }

    #[test]
    fn an_exact_upsert_is_a_no_op_before_the_replacement_boundary() {
        let (_root, paths) = paths();
        let server = CanonicalServer::new(
            "private-command",
            vec!["private-argument".to_owned()],
            BTreeMap::from([("TOKEN".to_owned(), "private-value".to_owned())]),
        );
        let current = canonical(vec![("alpha", server)]);
        let filesystem = RecordingFileSystem::new(current.clone());

        let report = add_server(
            &paths,
            &filesystem,
            AddRequest::new(
                "alpha".to_owned(),
                "private-command".to_owned(),
                vec!["private-argument".to_owned()],
                vec!["TOKEN=private-value".to_owned()],
            )
            .validate()
            .expect("the exact request should validate"),
        )
        .expect("an exact upsert should succeed");

        assert_eq!(report.outcome, UpsertOutcome::Unchanged);
        assert_eq!(filesystem.reads.get(), 1);
        assert!(filesystem.replacements.borrow().is_empty());
        assert!(!report.to_string().contains("private"));
        assert!(
            filesystem.current == current,
            "a no-op should preserve exact canonical bytes"
        );
    }

    #[test]
    fn concurrent_modification_is_actionable_and_never_exposes_requested_values() {
        let (_root, paths) = paths();
        let current = canonical(Vec::new());
        let filesystem = RecordingFileSystem::rejecting_replacement(current.clone());

        let error = add_server(
            &paths,
            &filesystem,
            AddRequest::new(
                "alpha".to_owned(),
                "private-command".to_owned(),
                vec!["private-argument".to_owned()],
                vec!["TOKEN=private-value".to_owned()],
            )
            .validate()
            .expect("the changed request should validate"),
        )
        .expect_err("a concurrent edit should stop the update");
        let diagnostic = error.to_string();

        assert!(diagnostic.contains("changed while the update was being prepared"));
        assert!(diagnostic.contains("no replacement was applied"));
        for private_value in ["private-command", "private-argument", "private-value"] {
            assert!(!diagnostic.contains(private_value));
        }
        assert!(
            filesystem.current == current,
            "a rejected replacement should preserve exact canonical bytes"
        );
        assert_eq!(filesystem.replacements.borrow().len(), 1);
    }

    #[test]
    fn updates_preserve_other_servers_and_pass_original_plus_canonical_bytes_to_the_writer() {
        let (_root, paths) = paths();
        let current = canonical(vec![
            (
                "zeta",
                CanonicalServer::new("zeta-command", Vec::new(), BTreeMap::new()),
            ),
            (
                "alpha",
                CanonicalServer::new("old-command", Vec::new(), BTreeMap::new()),
            ),
        ]);
        let filesystem = RecordingFileSystem::new(current.clone());

        let report = add_server(
            &paths,
            &filesystem,
            AddRequest::new(
                "alpha".to_owned(),
                "new-command".to_owned(),
                vec!["--new".to_owned()],
                vec!["TOKEN=new-value".to_owned()],
            )
            .validate()
            .expect("the update request should validate"),
        )
        .expect("a valid update should succeed");

        assert_eq!(report.outcome, UpsertOutcome::Updated);
        let replacements = filesystem.replacements.borrow();
        assert_eq!(replacements.len(), 1);
        assert!(
            replacements[0].0 == current,
            "the writer should receive exact original canonical bytes"
        );
        let desired = CanonicalConfig::parse_json(
            std::str::from_utf8(&replacements[0].1).expect("desired bytes should be UTF-8"),
        )
        .expect("desired bytes should be valid canonical JSON");
        assert_eq!(
            desired
                .servers()
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            ["alpha", "zeta"]
        );
        assert!(
            desired.servers()["zeta"].command() == "zeta-command"
                && desired.servers()["alpha"].command() == "new-command",
            "the update should preserve one definition and replace the requested one"
        );
    }

    #[test]
    fn listing_is_sorted_and_contains_only_structural_metadata() {
        let (_root, paths) = paths();
        let private_values = [
            "alpha-private-command",
            "alpha-private-argument",
            "alpha-private-value",
            "zeta-private-command",
        ];
        let current = canonical(vec![
            (
                "zeta",
                CanonicalServer::new(private_values[3], Vec::new(), BTreeMap::new()),
            ),
            (
                "alpha",
                CanonicalServer::new(
                    private_values[0],
                    vec![private_values[1].to_owned()],
                    BTreeMap::from([
                        ("LINE\nBREAK".to_owned(), "line-private-value".to_owned()),
                        ("TOKEN".to_owned(), private_values[2].to_owned()),
                    ]),
                ),
            ),
        ]);
        let filesystem = RecordingFileSystem::new(current);

        let output = list_servers(&paths, &filesystem)
            .expect("valid canonical state should list")
            .to_string();

        assert_eq!(
            output,
            "\"alpha\": command: <redacted>; arguments: 1; environment keys: \"LINE\\nBREAK\", \"TOKEN\"\n\"zeta\": command: <redacted>; arguments: 0; environment keys: none"
        );
        for private_value in private_values {
            assert!(!output.contains(private_value));
        }
        assert!(!output.contains("line-private-value"));
        assert!(filesystem.replacements.borrow().is_empty());
    }

    #[test]
    fn invalid_requested_definitions_fail_before_reading_canonical_state() {
        let (_root, _paths) = paths();
        let filesystem = RecordingFileSystem::new(canonical(Vec::new()));

        let error = AddRequest::new(
            " valid-name ".to_owned(),
            "private-command".to_owned(),
            Vec::new(),
            Vec::new(),
        )
        .validate()
        .expect_err("a padded name should be rejected");

        assert!(matches!(
            error,
            CatalogError::InvalidRequestedDefinition { .. }
        ));
        assert_eq!(filesystem.reads.get(), 0);
        assert!(filesystem.replacements.borrow().is_empty());
        assert!(!error.to_string().contains("private-command"));
    }

    #[test]
    fn empty_cli_environment_name_fails_before_managed_state_access() {
        let (_root, _paths) = paths();
        let filesystem = RecordingFileSystem::new(canonical(Vec::new()));

        let error = AddRequest::new(
            "alpha".to_owned(),
            "private-command".to_owned(),
            vec!["private-argument".to_owned()],
            vec!["=private-value".to_owned()],
        )
        .validate()
        .expect_err("an empty environment name should be rejected");

        assert!(matches!(
            error,
            CatalogError::InvalidRequestedDefinition {
                source: ConfigError::InvalidModel(ValidationError::InvalidEnvironmentName {
                    violation: EnvironmentNameViolation::Empty,
                    ..
                })
            }
        ));
        assert_eq!(filesystem.reads.get(), 0);
        assert!(filesystem.replacements.borrow().is_empty());
        let diagnostic = error.to_string();
        for private in ["private-command", "private-argument", "private-value"] {
            assert!(!diagnostic.contains(private));
        }
    }
}
