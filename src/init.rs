use crate::claude_desktop::{
    ClaudeDesktopAdapter, ClaudeDesktopAdapterError, ClaudeDesktopDiscovery,
};
use crate::config::{CanonicalConfig, CanonicalServer, ConfigError};
use crate::cursor::{CursorAdapter, CursorAdapterError, CursorDiscovery};
use crate::filesystem::{FileCreator, FileIoError, FileSystem};
use crate::paths::MacOsConfigurationPaths;
use crate::windsurf::{WindsurfAdapter, WindsurfAdapterError, WindsurfDiscovery};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;
use std::io;
use std::path::PathBuf;

/// Discover every implemented macOS client and create the first canonical
/// configuration.
///
/// Every read, parse, normalization, conflict, and serialization step finishes
/// before the create-only filesystem boundary is called. Native client files
/// are never written by this use case.
pub fn initialize(
    paths: &MacOsConfigurationPaths,
    filesystem: &(impl FileSystem + FileCreator),
) -> Result<InitReport, InitError> {
    let canonical_path = paths.canonical_configuration();
    match filesystem.read(canonical_path) {
        Ok(_) => {
            return Err(InitError::AlreadyInitialized {
                path: canonical_path.to_owned(),
            });
        }
        Err(source) if source.kind() == io::ErrorKind::NotFound => {}
        Err(source) => return Err(InitError::ReadCanonical { source }),
    }

    let claude = ClaudeDesktopAdapter::for_macos(paths)
        .discover(filesystem)
        .map_err(|source| InitError::DiscoverClaude { source })?;
    let cursor = CursorAdapter::for_macos(paths)
        .discover(filesystem)
        .map_err(|source| InitError::DiscoverCursor { source })?;
    let windsurf = WindsurfAdapter::for_macos(paths)
        .discover(filesystem)
        .map_err(|source| InitError::DiscoverWindsurf { source })?;

    let mut imports = Vec::with_capacity(3);
    let mut unmanaged_entries = BTreeMap::<Client, BTreeSet<String>>::new();
    if let ClaudeDesktopDiscovery::Found(document) = claude {
        imports.push(ClientImport::new(
            Client::ClaudeDesktop,
            document.canonical_config().clone(),
        ));
    }

    if let CursorDiscovery::Found(document) = cursor {
        unmanaged_entries.insert(
            Client::Cursor,
            document
                .unmanaged_server_names()
                .into_iter()
                .map(str::to_owned)
                .collect(),
        );
        imports.push(ClientImport::new(
            Client::Cursor,
            document.canonical_config().clone(),
        ));
    }
    if let WindsurfDiscovery::Found(document) = windsurf {
        unmanaged_entries.insert(
            Client::Windsurf,
            document
                .unmanaged_server_names()
                .into_iter()
                .map(str::to_owned)
                .collect(),
        );
        imports.push(ClientImport::new(
            Client::Windsurf,
            document.canonical_config().clone(),
        ));
    }

    let discovered_clients = imports.len();
    let normalized = normalize_imports(imports, unmanaged_entries)
        .map_err(|source| InitError::Conflicts { source })?;
    let config = CanonicalConfig::new(normalized.servers)
        .map_err(|source| InitError::BuildCanonical { source })?;
    let contents = config
        .to_canonical_json()
        .map_err(|source| InitError::SerializeCanonical { source })?;

    if let Err(source) = filesystem.create_new(canonical_path, contents.as_bytes()) {
        if source.kind() == io::ErrorKind::AlreadyExists {
            return Err(InitError::AlreadyInitialized {
                path: canonical_path.to_owned(),
            });
        }
        return Err(InitError::CreateCanonical { source });
    }

    Ok(InitReport {
        imported_servers: config.servers().len(),
        discovered_clients,
        skipped_entries: normalized.skipped_entries,
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InitReport {
    imported_servers: usize,
    discovered_clients: usize,
    skipped_entries: Vec<SkippedClientEntries>,
}

impl fmt::Display for InitReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Initialized canonical configuration with {} {} from {} client {}.",
            self.imported_servers,
            plural(self.imported_servers, "server", "servers"),
            self.discovered_clients,
            plural(self.discovered_clients, "configuration", "configurations")
        )?;

        for skipped in &self.skipped_entries {
            let names = skipped
                .names
                .iter()
                .map(|name| format!("{name:?}"))
                .collect::<Vec<_>>()
                .join(", ");
            write!(
                formatter,
                "\nSkipped {} unsupported {} {}: {names}.",
                skipped.names.len(),
                skipped.client,
                plural(skipped.names.len(), "entry", "entries")
            )?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SkippedClientEntries {
    client: Client,
    names: Vec<String>,
}

fn plural<'a>(count: usize, singular: &'a str, plural: &'a str) -> &'a str {
    if count == 1 { singular } else { plural }
}

#[derive(Debug)]
pub enum InitError {
    AlreadyInitialized { path: PathBuf },
    ReadCanonical { source: FileIoError },
    DiscoverClaude { source: ClaudeDesktopAdapterError },
    DiscoverCursor { source: CursorAdapterError },
    DiscoverWindsurf { source: WindsurfAdapterError },
    Conflicts { source: ImportConflicts },
    BuildCanonical { source: ConfigError },
    SerializeCanonical { source: ConfigError },
    CreateCanonical { source: FileIoError },
}

impl fmt::Display for InitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyInitialized { path } => write!(
                formatter,
                "canonical configuration already exists at `{}`; move or remove it before rerunning `mcp-sync init`",
                path.display()
            ),
            Self::ReadCanonical { source } => write!(
                formatter,
                "cannot inspect the canonical configuration before initialization: {source}"
            ),
            Self::DiscoverClaude { source } => write!(
                formatter,
                "cannot import Claude Desktop configuration: {source}; fix the file or its permissions, then rerun `mcp-sync init`"
            ),
            Self::DiscoverCursor { source } => write!(
                formatter,
                "cannot import Cursor configuration: {source}; fix the file or its permissions, then rerun `mcp-sync init`"
            ),
            Self::DiscoverWindsurf { source } => write!(
                formatter,
                "cannot import Windsurf configuration: {source}; fix the file or its permissions, then rerun `mcp-sync init`"
            ),
            Self::Conflicts { source } => source.fmt(formatter),
            Self::BuildCanonical { source } => {
                write!(
                    formatter,
                    "cannot normalize imported configuration: {source}"
                )
            }
            Self::SerializeCanonical { source } => {
                write!(
                    formatter,
                    "cannot serialize imported configuration: {source}"
                )
            }
            Self::CreateCanonical { source } => {
                write!(formatter, "cannot create canonical configuration: {source}")
            }
        }
    }
}

impl Error for InitError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::AlreadyInitialized { .. } => None,
            Self::ReadCanonical { source } | Self::CreateCanonical { source } => Some(source),
            Self::DiscoverClaude { source } => Some(source),
            Self::DiscoverCursor { source } => Some(source),
            Self::DiscoverWindsurf { source } => Some(source),
            Self::Conflicts { source } => Some(source),
            Self::BuildCanonical { source } | Self::SerializeCanonical { source } => Some(source),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Client {
    ClaudeDesktop,
    Cursor,
    Windsurf,
}

impl fmt::Display for Client {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ClaudeDesktop => formatter.write_str("Claude Desktop"),
            Self::Cursor => formatter.write_str("Cursor"),
            Self::Windsurf => formatter.write_str("Windsurf"),
        }
    }
}

#[derive(Clone)]
struct ClientImport {
    client: Client,
    config: CanonicalConfig,
}

impl ClientImport {
    fn new(client: Client, config: CanonicalConfig) -> Self {
        Self { client, config }
    }
}

#[derive(Debug)]
struct NormalizedImport {
    servers: BTreeMap<String, CanonicalServer>,
    skipped_entries: Vec<SkippedClientEntries>,
}

fn normalize_imports(
    mut imports: Vec<ClientImport>,
    unmanaged_entries: BTreeMap<Client, BTreeSet<String>>,
) -> Result<NormalizedImport, ImportConflicts> {
    imports.sort_by_key(|source| source.client);

    let mut imported = BTreeMap::<String, (Client, CanonicalServer)>::new();
    let mut conflicts = Vec::new();

    for source in imports {
        for (name, server) in source.config.servers() {
            match imported.get(name) {
                Some((first_client, first_server)) if first_server != server => {
                    conflicts.push(ImportConflict::DefinitionsDiffer {
                        server: name.clone(),
                        first_client: *first_client,
                        second_client: source.client,
                        differences: server_differences(first_server, server),
                    });
                }
                Some(_) => {}
                None => {
                    imported.insert(name.clone(), (source.client, server.clone()));
                }
            }
        }
    }

    for (unmanaged_client, names) in &unmanaged_entries {
        for name in names {
            if let Some((local_client, _)) = imported.get(name) {
                conflicts.push(ImportConflict::UnsupportedNativeCollision {
                    server: name.clone(),
                    local_client: *local_client,
                    unmanaged_client: *unmanaged_client,
                });
            }
        }
    }

    if !conflicts.is_empty() {
        conflicts.sort_by(|left, right| {
            left.server_name()
                .cmp(right.server_name())
                .then_with(|| left.kind_order().cmp(&right.kind_order()))
        });
        return Err(ImportConflicts { conflicts });
    }

    Ok(NormalizedImport {
        servers: imported
            .into_iter()
            .map(|(name, (_, server))| (name, server))
            .collect(),
        skipped_entries: unmanaged_entries
            .into_iter()
            .filter_map(|(client, names)| {
                (!names.is_empty()).then(|| SkippedClientEntries {
                    client,
                    names: names.into_iter().collect(),
                })
            })
            .collect(),
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ServerDifference {
    Command,
    Arguments,
    EnvironmentKeys,
    EnvironmentValues,
}

impl ServerDifference {
    fn description(self) -> &'static str {
        match self {
            Self::Command => "command",
            Self::Arguments => "arguments",
            Self::EnvironmentKeys => "environment keys",
            Self::EnvironmentValues => "environment values",
        }
    }
}

fn server_differences(first: &CanonicalServer, second: &CanonicalServer) -> Vec<ServerDifference> {
    let mut differences = Vec::with_capacity(4);
    if first.command() != second.command() {
        differences.push(ServerDifference::Command);
    }
    if first.args() != second.args() {
        differences.push(ServerDifference::Arguments);
    }

    let first_keys = first.env().keys().collect::<BTreeSet<_>>();
    let second_keys = second.env().keys().collect::<BTreeSet<_>>();
    if first_keys != second_keys {
        differences.push(ServerDifference::EnvironmentKeys);
    }
    if first
        .env()
        .iter()
        .any(|(key, value)| second.env().get(key).is_some_and(|other| other != value))
    {
        differences.push(ServerDifference::EnvironmentValues);
    }
    differences
}

#[derive(Debug, PartialEq, Eq)]
enum ImportConflict {
    DefinitionsDiffer {
        server: String,
        first_client: Client,
        second_client: Client,
        differences: Vec<ServerDifference>,
    },
    UnsupportedNativeCollision {
        server: String,
        local_client: Client,
        unmanaged_client: Client,
    },
}

impl ImportConflict {
    fn server_name(&self) -> &str {
        match self {
            Self::DefinitionsDiffer { server, .. }
            | Self::UnsupportedNativeCollision { server, .. } => server,
        }
    }

    fn kind_order(&self) -> u8 {
        match self {
            Self::DefinitionsDiffer { .. } => 0,
            Self::UnsupportedNativeCollision { .. } => 1,
        }
    }
}

impl fmt::Display for ImportConflict {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DefinitionsDiffer {
                server,
                first_client,
                second_client,
                differences,
            } => write!(
                formatter,
                "server {server:?} differs between {first_client} and {second_client} in {}",
                human_list(
                    differences
                        .iter()
                        .map(|difference| difference.description())
                        .collect()
                )
            ),
            Self::UnsupportedNativeCollision {
                server,
                local_client,
                unmanaged_client,
            } => write!(
                formatter,
                "server {server:?} is both a local {local_client} definition and an unsupported commandless {unmanaged_client} entry"
            ),
        }
    }
}

fn human_list(items: Vec<&str>) -> String {
    match items.as_slice() {
        [] => String::new(),
        [only] => (*only).to_owned(),
        [first, second] => format!("{first} and {second}"),
        _ => {
            let (last, initial) = items
                .split_last()
                .expect("a list with at least three items has a final item");
            format!("{}, and {last}", initial.join(", "))
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ImportConflicts {
    conflicts: Vec<ImportConflict>,
}

impl fmt::Display for ImportConflicts {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let [conflict] = self.conflicts.as_slice() {
            return write!(
                formatter,
                "cannot initialize because {conflict}; make the definitions identical, rename one, or remove one, then rerun `mcp-sync init`"
            );
        }

        formatter.write_str("cannot initialize because imported definitions conflict:")?;
        for conflict in &self.conflicts {
            write!(formatter, "\n- {conflict}")?;
        }
        formatter.write_str(
            "\nMake the definitions identical, rename them, or remove them, then rerun `mcp-sync init`",
        )
    }
}

impl Error for ImportConflicts {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn config(entries: Vec<(&str, CanonicalServer)>) -> CanonicalConfig {
        CanonicalConfig::new(
            entries
                .into_iter()
                .map(|(name, server)| (name.to_owned(), server))
                .collect(),
        )
        .expect("the synthetic import should be valid")
    }

    fn server(command: &str, argument: &str, environment_value: &str) -> CanonicalServer {
        CanonicalServer::new(
            command,
            vec![argument.to_owned()],
            BTreeMap::from([("SYNTHETIC_TOKEN".to_owned(), environment_value.to_owned())]),
        )
    }

    fn unmanaged(client: Client, names: &[&str]) -> BTreeMap<Client, BTreeSet<String>> {
        BTreeMap::from([(
            client,
            names.iter().map(|name| (*name).to_owned()).collect(),
        )])
    }

    #[test]
    fn compatible_imports_are_deterministic_regardless_of_discovery_order() {
        let shared = server("shared-command", "--shared", "shared-value");
        let claude = ClientImport::new(
            Client::ClaudeDesktop,
            config(vec![
                ("zeta", server("zeta-command", "--zeta", "zeta-value")),
                ("shared", shared.clone()),
            ]),
        );
        let cursor = ClientImport::new(
            Client::Cursor,
            config(vec![
                ("alpha", server("alpha-command", "--alpha", "alpha-value")),
                ("shared", shared),
            ]),
        );
        let windsurf = ClientImport::new(
            Client::Windsurf,
            config(vec![(
                "beta",
                server("beta-command", "--beta", "beta-value"),
            )]),
        );
        let unmanaged_entries = BTreeMap::from([
            (Client::Cursor, BTreeSet::from(["remote-only".to_owned()])),
            (
                Client::Windsurf,
                BTreeSet::from(["windsurf-remote".to_owned()]),
            ),
        ]);

        let forward = normalize_imports(
            vec![claude.clone(), cursor.clone(), windsurf.clone()],
            unmanaged_entries.clone(),
        )
        .expect("compatible imports should normalize");
        let reverse = normalize_imports(vec![windsurf, cursor, claude], unmanaged_entries)
            .expect("discovery order should not affect normalization");

        let forward_json = CanonicalConfig::new(forward.servers)
            .expect("the merged map should be valid")
            .to_canonical_json()
            .expect("the merged map should serialize");
        let reverse_json = CanonicalConfig::new(reverse.servers)
            .expect("the merged map should be valid")
            .to_canonical_json()
            .expect("the merged map should serialize");
        assert!(
            forward_json == reverse_json,
            "discovery order should not change canonical bytes"
        );
        assert_eq!(
            reverse.skipped_entries,
            [
                SkippedClientEntries {
                    client: Client::Cursor,
                    names: vec!["remote-only".to_owned()],
                },
                SkippedClientEntries {
                    client: Client::Windsurf,
                    names: vec!["windsurf-remote".to_owned()],
                },
            ]
        );
        assert!(forward_json.find("alpha").unwrap() < forward_json.find("shared").unwrap());
        assert!(forward_json.find("shared").unwrap() < forward_json.find("zeta").unwrap());
    }

    #[test]
    fn disagreements_report_only_stable_structure_and_never_values() {
        let claude_value = "claude-private-value";
        let cursor_value = "cursor-private-value";
        let claude = ClientImport::new(
            Client::ClaudeDesktop,
            config(vec![(
                "shared",
                server("claude-private-command", "--claude-private", claude_value),
            )]),
        );
        let cursor = ClientImport::new(
            Client::Cursor,
            config(vec![(
                "shared",
                CanonicalServer::new(
                    "cursor-private-command",
                    vec!["--cursor-private".to_owned()],
                    BTreeMap::from([
                        ("CURSOR_ONLY".to_owned(), "cursor-only-value".to_owned()),
                        ("SYNTHETIC_TOKEN".to_owned(), cursor_value.to_owned()),
                    ]),
                ),
            )]),
        );

        let error = normalize_imports(vec![cursor, claude], BTreeMap::new())
            .expect_err("different definitions should conflict");
        let message = error.to_string();

        assert_eq!(
            message,
            "cannot initialize because server \"shared\" differs between Claude Desktop and Cursor in command, arguments, environment keys, and environment values; make the definitions identical, rename one, or remove one, then rerun `mcp-sync init`"
        );
        for private_value in [
            claude_value,
            cursor_value,
            "claude-private-command",
            "cursor-private-command",
            "--claude-private",
            "--cursor-private",
            "cursor-only-value",
        ] {
            assert!(!message.contains(private_value));
        }
    }

    #[test]
    fn a_local_definition_cannot_hide_an_unsupported_native_entry() {
        let imports = vec![ClientImport::new(
            Client::ClaudeDesktop,
            config(vec![(
                "remote-collision",
                server("synthetic-command", "--synthetic", "synthetic-value"),
            )]),
        )];

        let error = normalize_imports(imports, unmanaged(Client::Windsurf, &["remote-collision"]))
            .expect_err("an unrepresentable collision should fail safely");

        assert_eq!(
            error.to_string(),
            "cannot initialize because server \"remote-collision\" is both a local Claude Desktop definition and an unsupported commandless Windsurf entry; make the definitions identical, rename one, or remove one, then rerun `mcp-sync init`"
        );
    }

    #[test]
    fn conflict_order_is_stable_by_server_name() {
        let claude = ClientImport::new(
            Client::ClaudeDesktop,
            config(vec![
                ("zeta", server("one", "--one", "one")),
                ("alpha", server("one", "--one", "one")),
            ]),
        );
        let cursor = ClientImport::new(
            Client::Cursor,
            config(vec![
                ("alpha", server("two", "--two", "two")),
                ("zeta", server("two", "--two", "two")),
            ]),
        );

        let message = normalize_imports(vec![cursor, claude], BTreeMap::new())
            .expect_err("both definitions should conflict")
            .to_string();

        assert!(message.find("\"alpha\"").unwrap() < message.find("\"zeta\"").unwrap());
    }

    #[test]
    fn report_grammar_covers_empty_and_single_imports() {
        let empty = InitReport {
            imported_servers: 0,
            discovered_clients: 0,
            skipped_entries: Vec::new(),
        };
        let single = InitReport {
            imported_servers: 1,
            discovered_clients: 1,
            skipped_entries: vec![
                SkippedClientEntries {
                    client: Client::Cursor,
                    names: vec!["remote-only".to_owned()],
                },
                SkippedClientEntries {
                    client: Client::Windsurf,
                    names: vec!["windsurf-remote".to_owned()],
                },
            ],
        };

        assert_eq!(
            empty.to_string(),
            "Initialized canonical configuration with 0 servers from 0 client configurations."
        );
        assert_eq!(
            single.to_string(),
            "Initialized canonical configuration with 1 server from 1 client configuration.\nSkipped 1 unsupported Cursor entry: \"remote-only\".\nSkipped 1 unsupported Windsurf entry: \"windsurf-remote\"."
        );
    }

    #[test]
    fn canonical_path_is_checked_before_any_client_discovery() {
        struct ExistingCanonicalFileSystem {
            canonical_path: PathBuf,
        }

        impl FileSystem for ExistingCanonicalFileSystem {
            fn read(&self, path: &Path) -> Result<Vec<u8>, FileIoError> {
                if path == self.canonical_path {
                    Ok(b"existing canonical bytes".to_vec())
                } else {
                    panic!("client discovery must not run when canonical config exists")
                }
            }
        }

        impl FileCreator for ExistingCanonicalFileSystem {
            fn create_new(&self, _path: &Path, _contents: &[u8]) -> Result<(), FileIoError> {
                panic!("an existing canonical config must never be replaced")
            }
        }

        struct FixtureEnvironment(PathBuf);

        impl crate::paths::Environment for FixtureEnvironment {
            fn value(&self, name: &'static str) -> Option<std::ffi::OsString> {
                (name == "HOME").then(|| self.0.clone().into_os_string())
            }
        }

        let root = tempfile::tempdir().expect("a temporary fixture should be created");
        let paths = MacOsConfigurationPaths::resolve(&FixtureEnvironment(
            root.path().join("synthetic-user"),
        ))
        .expect("synthetic paths should resolve");
        let filesystem = ExistingCanonicalFileSystem {
            canonical_path: paths.canonical_configuration().to_owned(),
        };

        let error = initialize(&paths, &filesystem)
            .expect_err("an existing canonical config should stop initialization");

        assert!(matches!(error, InitError::AlreadyInitialized { .. }));
    }
}
