use crate::claude_desktop::{
    ClaudeDesktopAdapter, ClaudeDesktopAdapterError, ClaudeDesktopDiscovery, ClaudeDesktopDocument,
};
use crate::config::{CanonicalConfig, ConfigError};
use crate::cursor::{CursorAdapter, CursorAdapterError, CursorDiscovery, CursorDocument};
use crate::filesystem::{
    ExpectedFile, FileIoError, FileMutationError, FileSystem, TransactionalFileUpdater, backup_path,
};
use crate::paths::MacOsConfigurationPaths;
use crate::reconciliation::{ReconciliationOutcome, ReconciliationPlan, ServerChanges, reconcile};
use std::error::Error;
use std::fmt;
use std::io;
use std::path::PathBuf;

/// Read and validate canonical plus native state, then render every desired
/// target byte before any mutation is attempted.
///
/// The returned plan contains the exact observed and replacement bytes used by
/// guarded apply. Its debug implementation reveals only paths, byte counts,
/// and the already-redacted reconciliation structure.
pub fn plan_sync(
    paths: &MacOsConfigurationPaths,
    filesystem: &impl FileSystem,
) -> Result<SyncPlan, SyncError> {
    let desired = load_canonical(paths, filesystem)?;

    let claude_adapter = ClaudeDesktopAdapter::for_macos(paths);
    let claude_discovery = claude_adapter
        .discover(filesystem)
        .map_err(|source| SyncError::DiscoverClaude { source })?;
    let claude = plan_claude(claude_adapter, claude_discovery, &desired)?;

    let cursor_adapter = CursorAdapter::for_macos(paths);
    let cursor_discovery = cursor_adapter
        .discover(filesystem)
        .map_err(|source| SyncError::DiscoverCursor { source })?;
    let cursor = plan_cursor(cursor_adapter, cursor_discovery, &desired)?;

    Ok(SyncPlan {
        targets: vec![claude, cursor],
    })
}

/// Return the redacted view of an already validated plan without changing any
/// file or independently recalculating target output.
pub fn dry_run(plan: &SyncPlan) -> SyncReport {
    SyncReport {
        mode: SyncMode::DryRun,
        targets: plan
            .targets
            .iter()
            .map(|target| {
                TargetReport::new(
                    target,
                    if target.requires_mutation() {
                        match target.observed {
                            ObservedFile::Missing => TargetStatus::WouldCreate,
                            ObservedFile::Existing(_) => TargetStatus::WouldUpdate {
                                backup: backup_path(&target.path),
                            },
                        }
                    } else {
                        TargetStatus::Unchanged
                    },
                )
            })
            .collect(),
    }
}

/// Apply the exact bytes in an already validated plan as one recoverable
/// transaction.
///
/// Each changed target produces an opaque filesystem receipt. If a later
/// target fails, receipts are rolled back in reverse order. Apply never reads
/// or reconciles native configuration again.
pub fn apply_sync<F>(plan: &SyncPlan, filesystem: &F) -> Result<SyncReport, SyncError>
where
    F: TransactionalFileUpdater,
{
    let mut reports = plan
        .targets
        .iter()
        .map(|target| {
            TargetReport::new(
                target,
                if target.requires_mutation() {
                    TargetStatus::NotAttempted
                } else {
                    TargetStatus::Unchanged
                },
            )
        })
        .collect::<Vec<_>>();
    let mut applied = Vec::<(usize, F::Receipt)>::new();

    for (index, target) in plan.targets.iter().enumerate() {
        if !target.requires_mutation() {
            continue;
        }

        let mutation = target.mutation_kind();
        let expected = match &target.observed {
            ObservedFile::Missing => ExpectedFile::Missing,
            ObservedFile::Existing(bytes) => ExpectedFile::Existing(bytes),
        };

        match filesystem.apply_file_change(&target.path, expected, &target.replacement) {
            Ok(receipt) => {
                reports[index].status = match mutation {
                    MutationKind::Create => TargetStatus::Created,
                    MutationKind::Update => TargetStatus::Updated {
                        backup: backup_path(&target.path),
                    },
                };
                applied.push((index, receipt));
            }
            Err(source) => {
                reports[index].status = TargetStatus::Failed { mutation, source };

                for (applied_index, receipt) in applied.iter().rev() {
                    let applied_mutation = plan.targets[*applied_index].mutation_kind();
                    let rollback = filesystem.rollback_file_change(receipt);
                    reports[*applied_index].status = match rollback {
                        Ok(()) => TargetStatus::RolledBack {
                            mutation: applied_mutation,
                        },
                        Err(source) => TargetStatus::RollbackFailed {
                            mutation: applied_mutation,
                            backup: (applied_mutation == MutationKind::Update)
                                .then(|| backup_path(&plan.targets[*applied_index].path)),
                            source,
                        },
                    };
                }

                return Err(SyncError::ApplyTransaction { targets: reports });
            }
        }
    }

    Ok(SyncReport {
        mode: SyncMode::Apply,
        targets: reports,
    })
}

fn load_canonical(
    paths: &MacOsConfigurationPaths,
    filesystem: &impl FileSystem,
) -> Result<CanonicalConfig, SyncError> {
    let path = paths.canonical_configuration();
    let bytes = match filesystem.read(path) {
        Ok(bytes) => bytes,
        Err(source) if source.kind() == io::ErrorKind::NotFound => {
            return Err(SyncError::NotInitialized {
                path: path.to_owned(),
            });
        }
        Err(source) => return Err(SyncError::ReadCanonical { source }),
    };
    let document = std::str::from_utf8(&bytes).map_err(|_| SyncError::InvalidEncoding {
        path: path.to_owned(),
    })?;
    CanonicalConfig::parse_json(document).map_err(|source| SyncError::InvalidCanonical {
        path: path.to_owned(),
        source,
    })
}

fn plan_claude(
    adapter: ClaudeDesktopAdapter,
    discovery: ClaudeDesktopDiscovery,
    desired: &CanonicalConfig,
) -> Result<TargetPlan, SyncError> {
    let (observed, document) = match discovery {
        ClaudeDesktopDiscovery::Missing => (
            ObservedFile::Missing,
            ClaudeDesktopDiscovery::Missing.into_document(),
        ),
        ClaudeDesktopDiscovery::Found(document) => (
            ObservedFile::Existing(document.original_bytes().to_vec()),
            document,
        ),
    };
    let reconciliation = reconcile(document.canonical_config(), desired);
    let rendered = document
        .render_plan(&reconciliation)
        .map_err(|source| SyncError::RenderClaude { source })?;
    let changed = rendered.changed();
    let replacement = rendered.into_bytes();
    let verified = ClaudeDesktopDocument::parse(&replacement)
        .map_err(|source| SyncError::RenderClaude { source })?;
    if !rendered_config_matches_plan(
        document.canonical_config(),
        desired,
        verified.canonical_config(),
    ) {
        return Err(SyncError::InconsistentRenderedPlan {
            target: Target::ClaudeDesktop,
        });
    }

    TargetPlan::new(
        Target::ClaudeDesktop,
        adapter.configuration_path().to_owned(),
        observed,
        replacement,
        reconciliation,
        changed,
        Vec::new(),
    )
}

fn plan_cursor(
    adapter: CursorAdapter,
    discovery: CursorDiscovery,
    desired: &CanonicalConfig,
) -> Result<TargetPlan, SyncError> {
    let (observed, document) = match discovery {
        CursorDiscovery::Missing => (
            ObservedFile::Missing,
            CursorDiscovery::Missing.into_document(),
        ),
        CursorDiscovery::Found(document) => (
            ObservedFile::Existing(document.original_bytes().to_vec()),
            document,
        ),
    };
    let unmanaged_entries = document
        .unmanaged_server_names()
        .into_iter()
        .map(str::to_owned)
        .collect();
    let reconciliation = reconcile(document.canonical_config(), desired);
    let rendered = document
        .render_plan(&reconciliation)
        .map_err(|source| SyncError::RenderCursor { source })?;
    let changed = rendered.changed();
    let replacement = rendered.into_bytes();
    let verified =
        CursorDocument::parse(&replacement).map_err(|source| SyncError::RenderCursor { source })?;
    let verified_unmanaged = verified
        .unmanaged_server_names()
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if !rendered_config_matches_plan(
        document.canonical_config(),
        desired,
        verified.canonical_config(),
    ) || verified_unmanaged != unmanaged_entries
    {
        return Err(SyncError::InconsistentRenderedPlan {
            target: Target::Cursor,
        });
    }

    TargetPlan::new(
        Target::Cursor,
        adapter.configuration_path().to_owned(),
        observed,
        replacement,
        reconciliation,
        changed,
        unmanaged_entries,
    )
}

fn rendered_config_matches_plan(
    current: &CanonicalConfig,
    desired: &CanonicalConfig,
    rendered: &CanonicalConfig,
) -> bool {
    let expected_count = current
        .servers()
        .keys()
        .chain(desired.servers().keys())
        .collect::<std::collections::BTreeSet<_>>()
        .len();

    rendered.servers().len() == expected_count
        && desired
            .servers()
            .iter()
            .all(|(name, server)| rendered.servers().get(name) == Some(server))
        && current.servers().iter().all(|(name, server)| {
            desired.servers().contains_key(name) || rendered.servers().get(name) == Some(server)
        })
}

pub struct SyncPlan {
    targets: Vec<TargetPlan>,
}

impl SyncPlan {
    #[cfg(test)]
    pub fn target_count(&self) -> usize {
        self.targets.len()
    }
}

impl fmt::Debug for SyncPlan {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SyncPlan")
            .field("targets", &self.targets)
            .finish()
    }
}

struct TargetPlan {
    target: Target,
    path: PathBuf,
    observed: ObservedFile,
    replacement: Vec<u8>,
    reconciliation: ReconciliationPlan,
    unmanaged_entries: Vec<String>,
}

impl TargetPlan {
    fn new(
        target: Target,
        path: PathBuf,
        observed: ObservedFile,
        replacement: Vec<u8>,
        reconciliation: ReconciliationPlan,
        rendered_changed: bool,
        unmanaged_entries: Vec<String>,
    ) -> Result<Self, SyncError> {
        if reconciliation.requires_mutation() != rendered_changed {
            return Err(SyncError::InconsistentRenderedPlan { target });
        }

        Ok(Self {
            target,
            path,
            observed,
            replacement,
            reconciliation,
            unmanaged_entries,
        })
    }

    fn requires_mutation(&self) -> bool {
        self.reconciliation.requires_mutation()
    }

    fn mutation_kind(&self) -> MutationKind {
        match self.observed {
            ObservedFile::Missing => MutationKind::Create,
            ObservedFile::Existing(_) => MutationKind::Update,
        }
    }
}

impl fmt::Debug for TargetPlan {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TargetPlan")
            .field("target", &self.target)
            .field("path", &self.path)
            .field("observed", &self.observed)
            .field("replacement_byte_count", &self.replacement.len())
            .field("reconciliation", &self.reconciliation)
            .field("unmanaged_entries", &self.unmanaged_entries)
            .finish()
    }
}

enum ObservedFile {
    Missing,
    Existing(Vec<u8>),
}

impl fmt::Debug for ObservedFile {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing => formatter.write_str("Missing"),
            Self::Existing(bytes) => formatter
                .debug_tuple("Existing")
                .field(&format_args!("{} bytes", bytes.len()))
                .finish(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Target {
    ClaudeDesktop,
    Cursor,
}

impl fmt::Display for Target {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ClaudeDesktop => formatter.write_str("Claude Desktop"),
            Self::Cursor => formatter.write_str("Cursor"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MutationKind {
    Create,
    Update,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SyncMode {
    DryRun,
    Apply,
}

pub struct SyncReport {
    mode: SyncMode,
    targets: Vec<TargetReport>,
}

impl fmt::Debug for SyncReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SyncReport")
            .field("mode", &self.mode)
            .field("targets", &self.targets)
            .finish()
    }
}

impl fmt::Display for SyncReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.mode {
            SyncMode::DryRun => write!(
                formatter,
                "Dry run validated {} targets; no files changed.",
                self.targets.len()
            )?,
            SyncMode::Apply => write!(
                formatter,
                "Sync completed for {} targets.",
                self.targets.len()
            )?,
        }

        for target in &self.targets {
            write!(formatter, "\n{target}")?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct TargetReport {
    target: Target,
    status: TargetStatus,
    entries: Vec<RedactedPlanEntry>,
    add: usize,
    update: usize,
    no_op: usize,
    drift: usize,
    unmanaged_entries: Vec<String>,
}

impl TargetReport {
    fn new(plan: &TargetPlan, status: TargetStatus) -> Self {
        let summary = plan.reconciliation.summary();
        Self {
            target: plan.target,
            status,
            entries: plan
                .reconciliation
                .entries()
                .iter()
                .map(|entry| RedactedPlanEntry {
                    server_name: entry.server_name().to_owned(),
                    outcome: entry.outcome().clone(),
                })
                .collect(),
            add: summary.add(),
            update: summary.update(),
            no_op: summary.no_op(),
            drift: summary.drift(),
            unmanaged_entries: plan.unmanaged_entries.clone(),
        }
    }
}

impl fmt::Display for TargetReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.target, self.status)?;
        write!(
            formatter,
            " (add: {}; update: {}; unchanged: {}; drift preserved: {}; unmanaged preserved: {}).",
            self.add,
            self.update,
            self.no_op,
            self.drift,
            self.unmanaged_entries.len()
        )?;
        for entry in &self.entries {
            write!(formatter, "\n  - {entry}")?;
        }
        for server in &self.unmanaged_entries {
            write!(formatter, "\n  - preserve unmanaged {server:?}")?;
        }
        Ok(())
    }
}

#[derive(Debug)]
enum TargetStatus {
    WouldCreate,
    WouldUpdate {
        backup: PathBuf,
    },
    Unchanged,
    Created,
    Updated {
        backup: PathBuf,
    },
    NotAttempted,
    Failed {
        mutation: MutationKind,
        source: FileMutationError,
    },
    RolledBack {
        mutation: MutationKind,
    },
    RollbackFailed {
        mutation: MutationKind,
        backup: Option<PathBuf>,
        source: FileMutationError,
    },
}

impl fmt::Display for TargetStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WouldCreate => formatter.write_str("would create (no previous file to back up)"),
            Self::WouldUpdate { backup } => {
                write!(
                    formatter,
                    "would update with recoverable backup at {backup:?}"
                )
            }
            Self::Unchanged => formatter.write_str("unchanged; no write or backup"),
            Self::Created => formatter.write_str("created (no previous file to back up)"),
            Self::Updated { backup } => {
                write!(formatter, "updated with recoverable backup at {backup:?}")
            }
            Self::NotAttempted => formatter.write_str("not attempted after an earlier failure"),
            Self::Failed { mutation, source } => {
                write!(formatter, "{} failed: {source}", mutation.verb())
            }
            Self::RolledBack { mutation } => write!(
                formatter,
                "rolled back after {}; pre-transaction target and backup state restored",
                mutation.noun()
            ),
            Self::RollbackFailed {
                mutation,
                backup,
                source,
            } => {
                write!(
                    formatter,
                    "ROLLBACK FAILED after {}: {source}",
                    mutation.noun()
                )?;
                match backup {
                    Some(path) => write!(
                        formatter,
                        "; inspect the target and recovery backup at {path:?} before retrying"
                    ),
                    None => formatter.write_str(
                        "; the newly created target may remain and has no prior-file backup",
                    ),
                }
            }
        }
    }
}

impl MutationKind {
    fn verb(self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Update => "update",
        }
    }

    fn noun(self) -> &'static str {
        match self {
            Self::Create => "creation",
            Self::Update => "update",
        }
    }
}

#[derive(Clone, Debug)]
struct RedactedPlanEntry {
    server_name: String,
    outcome: ReconciliationOutcome,
}

impl fmt::Display for RedactedPlanEntry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.outcome {
            ReconciliationOutcome::Add { desired } => write!(
                formatter,
                "add {:?} (arguments: {}; environment keys: {})",
                self.server_name,
                desired.argument_count(),
                quoted_list(desired.environment_keys())
            ),
            ReconciliationOutcome::Update { changes } => write!(
                formatter,
                "update {:?} ({})",
                self.server_name,
                describe_changes(changes)
            ),
            ReconciliationOutcome::NoOp => write!(formatter, "unchanged {:?}", self.server_name),
            ReconciliationOutcome::Drift { current } => write!(
                formatter,
                "preserve target-only {:?} (arguments: {}; environment keys: {})",
                self.server_name,
                current.argument_count(),
                quoted_list(current.environment_keys())
            ),
        }
    }
}

fn describe_changes(changes: &ServerChanges) -> String {
    let mut descriptions = Vec::new();
    if changes.command_changed() {
        descriptions.push("command".to_owned());
    }
    if let Some(arguments) = changes.arguments() {
        descriptions.push(format!(
            "arguments {} -> {}",
            arguments.current_count(),
            arguments.desired_count()
        ));
    }
    if !changes.environment().added_keys().is_empty() {
        descriptions.push(format!(
            "environment keys added {}",
            quoted_list(changes.environment().added_keys())
        ));
    }
    if !changes.environment().updated_keys().is_empty() {
        descriptions.push(format!(
            "environment keys updated {}",
            quoted_list(changes.environment().updated_keys())
        ));
    }
    if !changes.environment().removed_keys().is_empty() {
        descriptions.push(format!(
            "environment keys removed {}",
            quoted_list(changes.environment().removed_keys())
        ));
    }
    descriptions.join("; ")
}

fn quoted_list(items: &[String]) -> String {
    if items.is_empty() {
        "none".to_owned()
    } else {
        items
            .iter()
            .map(|item| format!("{item:?}"))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[derive(Debug)]
pub enum SyncError {
    NotInitialized { path: PathBuf },
    ReadCanonical { source: FileIoError },
    InvalidEncoding { path: PathBuf },
    InvalidCanonical { path: PathBuf, source: ConfigError },
    DiscoverClaude { source: ClaudeDesktopAdapterError },
    RenderClaude { source: ClaudeDesktopAdapterError },
    DiscoverCursor { source: CursorAdapterError },
    RenderCursor { source: CursorAdapterError },
    InconsistentRenderedPlan { target: Target },
    ApplyTransaction { targets: Vec<TargetReport> },
}

impl fmt::Display for SyncError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotInitialized { path } => write!(
                formatter,
                "canonical configuration does not exist at `{}`; run `mcp-sync init` first",
                path.display()
            ),
            Self::ReadCanonical { source } => {
                write!(
                    formatter,
                    "cannot read canonical configuration for sync: {source}"
                )
            }
            Self::InvalidEncoding { path } => write!(
                formatter,
                "canonical configuration at `{}` is not valid UTF-8; restore or repair it before syncing",
                path.display()
            ),
            Self::InvalidCanonical { path, source } => write!(
                formatter,
                "canonical configuration at `{}` is invalid: {source}; restore or repair it before syncing",
                path.display()
            ),
            Self::DiscoverClaude { source } => write!(
                formatter,
                "cannot plan Claude Desktop sync: {source}; no target files were changed"
            ),
            Self::RenderClaude { source } => write!(
                formatter,
                "cannot render the validated Claude Desktop sync plan: {source}; no target files were changed"
            ),
            Self::DiscoverCursor { source } => write!(
                formatter,
                "cannot plan Cursor sync: {source}; no target files were changed"
            ),
            Self::RenderCursor { source } => write!(
                formatter,
                "cannot render the validated Cursor sync plan: {source}; no target files were changed"
            ),
            Self::InconsistentRenderedPlan { target } => write!(
                formatter,
                "{target} adapter produced bytes inconsistent with its validated reconciliation plan; no target files were changed"
            ),
            Self::ApplyTransaction { targets } => {
                let failed = targets.iter().find_map(|target| match &target.status {
                    TargetStatus::Failed { source, .. } => Some((target.target, source)),
                    _ => None,
                });
                if let Some((target, source)) = failed {
                    write!(
                        formatter,
                        "sync transaction failed while applying {target}: {source}"
                    )?;
                } else {
                    formatter.write_str("sync transaction failed")?;
                }
                formatter.write_str("\nPer-target outcomes:")?;
                for target in targets {
                    write!(formatter, "\n{target}")?;
                }
                Ok(())
            }
        }
    }
}

impl Error for SyncError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ReadCanonical { source } => Some(source),
            Self::InvalidCanonical { source, .. } => Some(source),
            Self::DiscoverClaude { source } | Self::RenderClaude { source } => Some(source),
            Self::DiscoverCursor { source } | Self::RenderCursor { source } => Some(source),
            Self::ApplyTransaction { targets } => targets.iter().find_map(|target| {
                if let TargetStatus::Failed { source, .. } = &target.status {
                    Some(source as &(dyn Error + 'static))
                } else {
                    None
                }
            }),
            Self::NotInitialized { .. }
            | Self::InvalidEncoding { .. }
            | Self::InconsistentRenderedPlan { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths::Environment;
    use std::ffi::OsString;
    use std::fs;
    use std::path::Path;

    struct FixtureEnvironment(PathBuf);

    impl Environment for FixtureEnvironment {
        fn value(&self, name: &'static str) -> Option<OsString> {
            (name == "HOME").then(|| self.0.clone().into_os_string())
        }
    }

    fn fixture_paths(root: &Path) -> MacOsConfigurationPaths {
        MacOsConfigurationPaths::resolve(&FixtureEnvironment(root.join("user")))
            .expect("synthetic macOS paths should resolve")
    }

    fn canonical_with_private_values() -> &'static [u8] {
        br#"{
  "schemaVersion": 1,
  "servers": {
    "private-server": {
      "command": "private-command-value",
      "args": ["private-argument-value"],
      "env": {"PRIVATE_KEY": "private-environment-value"}
    }
  }
}
"#
    }

    #[test]
    fn one_plan_drives_redacted_dry_run_and_guarded_apply() {
        let fixture = tempfile::tempdir().expect("temporary sync fixture should be created");
        let paths = fixture_paths(fixture.path());
        fs::create_dir_all(
            paths
                .canonical_configuration()
                .parent()
                .expect("canonical path has a parent"),
        )
        .expect("canonical directory should be created");
        fs::write(
            paths.canonical_configuration(),
            canonical_with_private_values(),
        )
        .expect("canonical fixture should be written");

        let plan = plan_sync(&paths, &crate::filesystem::OsFileSystem)
            .expect("both missing targets should plan");
        let debug = format!("{plan:?}");
        let dry_run = dry_run(&plan).to_string();

        assert_eq!(plan.target_count(), 2);
        for private in [
            "private-command-value",
            "private-argument-value",
            "private-environment-value",
        ] {
            assert!(!debug.contains(private));
            assert!(!dry_run.contains(private));
        }
        assert!(dry_run.contains("Dry run validated 2 targets; no files changed."));
        assert!(dry_run.contains("Claude Desktop: would create"));
        assert!(dry_run.contains("Cursor: would create"));
        assert!(!paths.user_home().join(".cursor/mcp.json").exists());

        let report = apply_sync(&plan, &crate::filesystem::OsFileSystem)
            .expect("the exact planned bytes should apply");
        assert!(report.to_string().contains("Sync completed for 2 targets."));
        assert!(paths.user_home().join(".cursor/mcp.json").is_file());
        assert!(
            paths
                .application_support()
                .join("Claude/claude_desktop_config.json")
                .is_file()
        );

        let settled = plan_sync(&paths, &crate::filesystem::OsFileSystem)
            .expect("the applied targets should replan");
        let settled_report = apply_sync(&settled, &crate::filesystem::OsFileSystem)
            .expect("a settled plan should be a no-op");
        assert!(
            settled_report
                .to_string()
                .contains("unchanged; no write or backup")
        );
    }

    #[test]
    fn apply_refuses_stale_planned_bytes_without_replanning_or_touching_later_targets() {
        let fixture = tempfile::tempdir().expect("temporary sync fixture should be created");
        let paths = fixture_paths(fixture.path());
        let claude_path = ClaudeDesktopAdapter::for_macos(&paths)
            .configuration_path()
            .to_owned();
        let cursor_path = CursorAdapter::for_macos(&paths)
            .configuration_path()
            .to_owned();
        for path in [paths.canonical_configuration(), &claude_path, &cursor_path] {
            fs::create_dir_all(path.parent().expect("a fixture path has a parent"))
                .expect("fixture directories should be created");
        }
        fs::write(
            paths.canonical_configuration(),
            canonical_with_private_values(),
        )
        .expect("canonical fixture should be written");
        fs::write(&claude_path, b"{}\n").expect("Claude fixture should be written");
        fs::write(&cursor_path, b"{}\n").expect("Cursor fixture should be written");

        let plan = plan_sync(&paths, &crate::filesystem::OsFileSystem)
            .expect("the original exact bytes should plan");
        let concurrent = b"{\"concurrent\":true}\n";
        fs::write(&claude_path, concurrent).expect("the concurrent edit should be written");

        let error = apply_sync(&plan, &crate::filesystem::OsFileSystem)
            .expect_err("stale planned bytes must abort the transaction");
        let diagnostic = error.to_string();

        assert!(diagnostic.contains("Claude Desktop: update failed"));
        assert!(diagnostic.contains("Cursor: not attempted after an earlier failure"));
        assert_eq!(
            fs::read(&claude_path).expect("the concurrent target should remain readable"),
            concurrent
        );
        assert_eq!(
            fs::read(&cursor_path).expect("the later target should remain readable"),
            b"{}\n"
        );
        assert!(!backup_path(&claude_path).exists());
        assert!(!backup_path(&cursor_path).exists());
        for private in [
            "private-command-value",
            "private-argument-value",
            "private-environment-value",
        ] {
            assert!(!diagnostic.contains(private));
        }
    }

    struct ForcedFailureFileSystem {
        apply_count: std::cell::Cell<usize>,
        rollback_count: std::cell::Cell<usize>,
        fail_rollback: bool,
    }

    impl TransactionalFileUpdater for ForcedFailureFileSystem {
        type Receipt = usize;

        fn apply_file_change(
            &self,
            path: &Path,
            _expected: ExpectedFile<'_>,
            _replacement: &[u8],
        ) -> Result<Self::Receipt, FileMutationError> {
            let count = self.apply_count.get();
            self.apply_count.set(count + 1);
            if count == 1 {
                Err(FileMutationError::ConcurrentModification {
                    path: path.to_owned(),
                })
            } else {
                Ok(count)
            }
        }

        fn rollback_file_change(&self, _receipt: &Self::Receipt) -> Result<(), FileMutationError> {
            self.rollback_count.set(self.rollback_count.get() + 1);
            if self.fail_rollback {
                Err(FileMutationError::ConcurrentModification {
                    path: PathBuf::from("/synthetic/first-target.json"),
                })
            } else {
                Ok(())
            }
        }
    }

    fn two_create_plan() -> (tempfile::TempDir, SyncPlan) {
        let fixture = tempfile::tempdir().expect("temporary sync fixture should be created");
        let paths = fixture_paths(fixture.path());
        fs::create_dir_all(
            paths
                .canonical_configuration()
                .parent()
                .expect("canonical path has a parent"),
        )
        .expect("canonical directory should be created");
        fs::write(
            paths.canonical_configuration(),
            canonical_with_private_values(),
        )
        .expect("canonical fixture should be written");
        let plan = plan_sync(&paths, &crate::filesystem::OsFileSystem)
            .expect("both missing targets should plan");
        (fixture, plan)
    }

    #[test]
    fn a_forced_second_target_failure_rolls_back_the_first_in_reverse_order() {
        let (_fixture, plan) = two_create_plan();
        let filesystem = ForcedFailureFileSystem {
            apply_count: std::cell::Cell::new(0),
            rollback_count: std::cell::Cell::new(0),
            fail_rollback: false,
        };

        let error = apply_sync(&plan, &filesystem)
            .expect_err("the injected second target failure should abort apply");
        let diagnostic = error.to_string();

        assert_eq!(filesystem.apply_count.get(), 2);
        assert_eq!(filesystem.rollback_count.get(), 1);
        assert!(std::error::Error::source(&error).is_some());
        assert!(diagnostic.contains("Claude Desktop: rolled back after creation"));
        assert!(diagnostic.contains("Cursor: create failed"));
        for private in [
            "private-command-value",
            "private-argument-value",
            "private-environment-value",
        ] {
            assert!(!diagnostic.contains(private));
        }
    }

    #[test]
    fn rollback_failure_is_non_successful_and_explicit_per_target() {
        let (_fixture, plan) = two_create_plan();
        let filesystem = ForcedFailureFileSystem {
            apply_count: std::cell::Cell::new(0),
            rollback_count: std::cell::Cell::new(0),
            fail_rollback: true,
        };

        let error = apply_sync(&plan, &filesystem)
            .expect_err("rollback failure must keep the transaction unsuccessful");
        let diagnostic = error.to_string();

        assert_eq!(filesystem.rollback_count.get(), 1);
        assert!(diagnostic.contains("Claude Desktop: ROLLBACK FAILED after creation"));
        assert!(diagnostic.contains("Cursor: create failed"));
    }
}
