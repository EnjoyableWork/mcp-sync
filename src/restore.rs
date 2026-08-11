use crate::claude_desktop::{
    ClaudeDesktopAdapter, ClaudeDesktopAdapterError, ClaudeDesktopDocument,
};
use crate::codex::{CodexAdapter, CodexAdapterError, CodexDocument};
use crate::config::{CanonicalConfig, ConfigError};
use crate::cursor::{CursorAdapter, CursorAdapterError, CursorDocument};
use crate::filesystem::{
    BackupRestorer, ExpectedFile, FileMutationError, RestoreFileSystem, backup_path,
};
use crate::kiro::{KiroAdapter, KiroAdapterError, KiroDocument};
use crate::paths::ConfigurationPaths;
use crate::vscode::{VsCodeAdapter, VsCodeAdapterError, VsCodeDocument};
use crate::windsurf::{WindsurfAdapter, WindsurfAdapterError, WindsurfDocument};
use std::error::Error;
use std::fmt;
use std::io;
use std::path::PathBuf;

/// One global configuration path owned by the restore command.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RestoreTarget {
    Canonical,
    ClaudeDesktop,
    Cursor,
    Windsurf,
    VsCode,
    Codex,
    Kiro,
}

impl RestoreTarget {
    fn configuration_path(self, paths: &ConfigurationPaths) -> PathBuf {
        match self {
            Self::Canonical => paths.canonical_configuration().to_owned(),
            Self::ClaudeDesktop => ClaudeDesktopAdapter::from_paths(paths)
                .configuration_path()
                .to_owned(),
            Self::Cursor => CursorAdapter::from_paths(paths)
                .configuration_path()
                .to_owned(),
            Self::Windsurf => WindsurfAdapter::from_paths(paths)
                .configuration_path()
                .to_owned(),
            Self::VsCode => VsCodeAdapter::from_paths(paths)
                .configuration_path()
                .to_owned(),
            Self::Codex => CodexAdapter::from_paths(paths)
                .configuration_path()
                .to_owned(),
            Self::Kiro => KiroAdapter::from_paths(paths)
                .configuration_path()
                .to_owned(),
        }
    }
}

impl fmt::Display for RestoreTarget {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Canonical => "canonical configuration",
            Self::ClaudeDesktop => "Claude Desktop",
            Self::Cursor => "Cursor",
            Self::Windsurf => "Windsurf",
            Self::VsCode => "VS Code",
            Self::Codex => "Codex",
            Self::Kiro => "Kiro",
        })
    }
}

/// Read and validate one adjacent backup before any restoration is attempted.
///
/// The returned plan retains exact private bytes for guarded apply. Its custom
/// debug representation exposes paths, states, and byte counts only.
pub fn plan_restore(
    paths: &ConfigurationPaths,
    filesystem: &impl RestoreFileSystem,
    target: RestoreTarget,
) -> Result<RestorePlan, RestoreError> {
    let path = target.configuration_path(paths);
    let backup = backup_path(&path);
    let observed = match filesystem.read_optional_regular_file(&path) {
        Ok(Some(bytes)) => ObservedTarget::Existing(bytes),
        Ok(None) => ObservedTarget::Missing,
        Err(source) => return Err(RestoreError::ReadTarget { target, source }),
    };
    let backup_bytes = match filesystem.read_required_regular_file(&backup) {
        Ok(bytes) => bytes,
        Err(source) if mutation_is_not_found(&source) => {
            return Err(RestoreError::MissingBackup {
                target,
                path: backup,
            });
        }
        Err(source) => return Err(RestoreError::ReadBackup { target, source }),
    };

    validate_backup(target, &backup_bytes).map_err(|source| RestoreError::InvalidBackup {
        target,
        path: backup.clone(),
        source,
    })?;

    Ok(RestorePlan {
        target,
        path,
        backup_path: backup,
        observed,
        backup: backup_bytes,
    })
}

fn mutation_is_not_found(error: &FileMutationError) -> bool {
    matches!(error, FileMutationError::Io(source) if source.kind() == io::ErrorKind::NotFound)
}

/// Return the structural result of an already validated restore plan without
/// changing either file.
pub fn dry_run(plan: &RestorePlan) -> RestoreReport {
    plan.report(RestoreMode::DryRun)
}

/// Apply one validated restore plan through the guarded filesystem boundary.
pub fn apply_restore(
    plan: &RestorePlan,
    filesystem: &impl BackupRestorer,
) -> Result<RestoreReport, RestoreError> {
    if plan.is_unchanged() {
        return Ok(plan.report(RestoreMode::Apply));
    }

    filesystem
        .restore_backup_if_unchanged(&plan.path, plan.expected_target(), &plan.backup)
        .map_err(|source| RestoreError::Apply {
            target: plan.target,
            source,
        })?;
    Ok(plan.report(RestoreMode::Apply))
}

fn validate_backup(target: RestoreTarget, backup: &[u8]) -> Result<(), BackupValidationError> {
    match target {
        RestoreTarget::Canonical => {
            let document = std::str::from_utf8(backup)
                .map_err(|_| BackupValidationError::InvalidCanonicalEncoding)?;
            CanonicalConfig::parse_json(document)
                .map(drop)
                .map_err(BackupValidationError::Canonical)
        }
        RestoreTarget::ClaudeDesktop => ClaudeDesktopDocument::parse(backup)
            .map(drop)
            .map_err(BackupValidationError::ClaudeDesktop),
        RestoreTarget::Cursor => CursorDocument::parse(backup)
            .map(drop)
            .map_err(BackupValidationError::Cursor),
        RestoreTarget::Windsurf => WindsurfDocument::parse(backup)
            .map(drop)
            .map_err(BackupValidationError::Windsurf),
        RestoreTarget::VsCode => VsCodeDocument::parse(backup)
            .map(drop)
            .map_err(BackupValidationError::VsCode),
        RestoreTarget::Codex => CodexDocument::parse(backup)
            .map(drop)
            .map_err(BackupValidationError::Codex),
        RestoreTarget::Kiro => KiroDocument::parse(backup)
            .map(drop)
            .map_err(BackupValidationError::Kiro),
    }
}

pub struct RestorePlan {
    target: RestoreTarget,
    path: PathBuf,
    backup_path: PathBuf,
    observed: ObservedTarget,
    backup: Vec<u8>,
}

impl RestorePlan {
    fn expected_target(&self) -> ExpectedFile<'_> {
        match &self.observed {
            ObservedTarget::Missing => ExpectedFile::Missing,
            ObservedTarget::Existing(bytes) => ExpectedFile::Existing(bytes),
        }
    }

    fn is_unchanged(&self) -> bool {
        matches!(&self.observed, ObservedTarget::Existing(bytes) if bytes == &self.backup)
    }

    fn report(&self, mode: RestoreMode) -> RestoreReport {
        let status = match (&self.observed, self.is_unchanged(), mode) {
            (_, true, _) => RestoreStatus::Unchanged,
            (ObservedTarget::Missing, false, RestoreMode::DryRun) => RestoreStatus::WouldRecreate,
            (ObservedTarget::Missing, false, RestoreMode::Apply) => RestoreStatus::Recreated,
            (ObservedTarget::Existing(_), false, RestoreMode::DryRun) => {
                RestoreStatus::WouldRestore
            }
            (ObservedTarget::Existing(_), false, RestoreMode::Apply) => RestoreStatus::Restored,
        };
        RestoreReport {
            target: self.target,
            path: self.path.clone(),
            backup_path: self.backup_path.clone(),
            status,
        }
    }
}

impl fmt::Debug for RestorePlan {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RestorePlan")
            .field("target", &self.target)
            .field("path", &self.path)
            .field("backup_path", &self.backup_path)
            .field("observed", &self.observed)
            .field("backup_byte_count", &self.backup.len())
            .finish()
    }
}

enum ObservedTarget {
    Missing,
    Existing(Vec<u8>),
}

impl fmt::Debug for ObservedTarget {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing => formatter.write_str("Missing"),
            Self::Existing(bytes) => formatter
                .debug_struct("Existing")
                .field("byte_count", &bytes.len())
                .finish(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RestoreMode {
    DryRun,
    Apply,
}

pub struct RestoreReport {
    target: RestoreTarget,
    path: PathBuf,
    backup_path: PathBuf,
    status: RestoreStatus,
}

impl fmt::Debug for RestoreReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RestoreReport")
            .field("target", &self.target)
            .field("path", &self.path)
            .field("backup_path", &self.backup_path)
            .field("status", &self.status)
            .finish()
    }
}

impl fmt::Display for RestoreReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.status {
            RestoreStatus::WouldRestore => write!(
                formatter,
                "{} at `{}` would be restored from `{}`; the current target would become the retained backup",
                self.target,
                self.path.display(),
                self.backup_path.display()
            ),
            RestoreStatus::WouldRecreate => write!(
                formatter,
                "missing {} at `{}` would be recreated from retained backup `{}`",
                self.target,
                self.path.display(),
                self.backup_path.display()
            ),
            RestoreStatus::Unchanged => write!(
                formatter,
                "{} at `{}` already matches retained backup `{}`; no files changed",
                self.target,
                self.path.display(),
                self.backup_path.display()
            ),
            RestoreStatus::Restored => write!(
                formatter,
                "{} at `{}` restored from `{}`; the previous target is now the retained backup",
                self.target,
                self.path.display(),
                self.backup_path.display()
            ),
            RestoreStatus::Recreated => write!(
                formatter,
                "{} at `{}` recreated from retained backup `{}`; the backup was preserved",
                self.target,
                self.path.display(),
                self.backup_path.display()
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RestoreStatus {
    WouldRestore,
    WouldRecreate,
    Unchanged,
    Restored,
    Recreated,
}

pub(crate) enum BackupValidationError {
    InvalidCanonicalEncoding,
    Canonical(ConfigError),
    ClaudeDesktop(ClaudeDesktopAdapterError),
    Cursor(CursorAdapterError),
    Windsurf(WindsurfAdapterError),
    VsCode(VsCodeAdapterError),
    Codex(CodexAdapterError),
    Kiro(KiroAdapterError),
}

impl fmt::Debug for BackupValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("invalid configuration backup")
    }
}

impl fmt::Display for BackupValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("backup does not satisfy the selected configuration contract")
    }
}

impl Error for BackupValidationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidCanonicalEncoding => None,
            Self::Canonical(source) => Some(source),
            Self::ClaudeDesktop(source) => Some(source),
            Self::Cursor(source) => Some(source),
            Self::Windsurf(source) => Some(source),
            Self::VsCode(source) => Some(source),
            Self::Codex(source) => Some(source),
            Self::Kiro(source) => Some(source),
        }
    }
}

#[derive(Debug)]
pub enum RestoreError {
    ReadTarget {
        target: RestoreTarget,
        source: FileMutationError,
    },
    MissingBackup {
        target: RestoreTarget,
        path: PathBuf,
    },
    ReadBackup {
        target: RestoreTarget,
        source: FileMutationError,
    },
    InvalidBackup {
        target: RestoreTarget,
        path: PathBuf,
        source: BackupValidationError,
    },
    Apply {
        target: RestoreTarget,
        source: FileMutationError,
    },
}

impl fmt::Display for RestoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadTarget { target, source } => {
                write!(
                    formatter,
                    "cannot inspect current {target} target: {source}"
                )
            }
            Self::MissingBackup { target, path } => write!(
                formatter,
                "cannot restore {target}: adjacent backup `{}` does not exist",
                path.display()
            ),
            Self::ReadBackup { target, source } => {
                write!(formatter, "cannot inspect {target} backup: {source}")
            }
            Self::InvalidBackup { target, path, .. } => write!(
                formatter,
                "cannot restore {target}: backup `{}` is not a valid {target} document",
                path.display()
            ),
            Self::Apply { target, source } => {
                write!(formatter, "could not restore {target}: {source}")
            }
        }
    }
}

impl Error for RestoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ReadTarget { source, .. } | Self::ReadBackup { source, .. } => Some(source),
            Self::InvalidBackup { source, .. } => Some(source),
            Self::Apply { source, .. } => Some(source),
            Self::MissingBackup { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filesystem::{BackupRestorer, FileIoError, RestoreFileSystem};
    use crate::paths::{Environment, Platform};
    use std::cell::RefCell;
    use std::collections::BTreeMap;
    use std::ffi::OsString;
    use std::path::Path;

    const PRIVATE_CURRENT: &[u8] = b"current-private-restore-sentinel\n";

    struct FixtureEnvironment {
        root: PathBuf,
    }

    impl Environment for FixtureEnvironment {
        fn value(&self, name: &'static str) -> Option<OsString> {
            let path = match name {
                "HOME" | "USERPROFILE" => self.root.join("user"),
                "XDG_CONFIG_HOME" => self.root.join("xdg"),
                "LOCALAPPDATA" => self.root.join("local"),
                "APPDATA" => self.root.join("roaming"),
                _ => return None,
            };
            Some(path.into_os_string())
        }
    }

    struct MemoryFileSystem {
        files: BTreeMap<PathBuf, Vec<u8>>,
    }

    impl RestoreFileSystem for MemoryFileSystem {
        fn read_optional_regular_file(
            &self,
            path: &Path,
        ) -> Result<Option<Vec<u8>>, FileMutationError> {
            Ok(self.files.get(path).cloned())
        }

        fn read_required_regular_file(&self, path: &Path) -> Result<Vec<u8>, FileMutationError> {
            self.files.get(path).cloned().ok_or_else(|| {
                FileIoError::read(
                    path,
                    io::Error::new(io::ErrorKind::NotFound, "synthetic missing file"),
                )
                .into()
            })
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    enum RecordedTarget {
        Missing,
        Existing(Vec<u8>),
    }

    #[derive(Debug, PartialEq, Eq)]
    struct RestoreCall {
        path: PathBuf,
        target: RecordedTarget,
        backup: Vec<u8>,
    }

    #[derive(Default)]
    struct RecordingRestorer {
        calls: RefCell<Vec<RestoreCall>>,
    }

    impl BackupRestorer for RecordingRestorer {
        fn restore_backup_if_unchanged(
            &self,
            path: &Path,
            expected_target: ExpectedFile<'_>,
            expected_backup: &[u8],
        ) -> Result<(), FileMutationError> {
            let target = match expected_target {
                ExpectedFile::Missing => RecordedTarget::Missing,
                ExpectedFile::Existing(bytes) => RecordedTarget::Existing(bytes.to_vec()),
            };
            self.calls.borrow_mut().push(RestoreCall {
                path: path.to_owned(),
                target,
                backup: expected_backup.to_vec(),
            });
            Ok(())
        }
    }

    fn paths(platform: Platform) -> (tempfile::TempDir, ConfigurationPaths) {
        let root = tempfile::tempdir().expect("temporary path fixture should be created");
        let paths = ConfigurationPaths::resolve_for(
            platform,
            &FixtureEnvironment {
                root: root.path().to_owned(),
            },
        )
        .expect("synthetic paths should resolve");
        (root, paths)
    }

    fn valid_backup(target: RestoreTarget) -> &'static [u8] {
        match target {
            RestoreTarget::Canonical => b"{\"schemaVersion\":1,\"servers\":{}}\n",
            RestoreTarget::ClaudeDesktop
            | RestoreTarget::Cursor
            | RestoreTarget::Windsurf
            | RestoreTarget::VsCode
            | RestoreTarget::Kiro => b"{}\n",
            RestoreTarget::Codex => b"# retained Codex configuration\n",
        }
    }

    fn filesystem_for(
        target: RestoreTarget,
        paths: &ConfigurationPaths,
        current: Option<&[u8]>,
        backup: &[u8],
    ) -> MemoryFileSystem {
        let path = target.configuration_path(paths);
        let mut files = BTreeMap::from([(backup_path(&path), backup.to_vec())]);
        if let Some(current) = current {
            files.insert(path, current.to_vec());
        }
        MemoryFileSystem { files }
    }

    #[test]
    fn every_restore_target_resolves_and_validates_on_all_supported_platforms() {
        for platform in [Platform::MacOs, Platform::Linux, Platform::Windows] {
            let (_root, paths) = paths(platform);
            for target in [
                RestoreTarget::Canonical,
                RestoreTarget::ClaudeDesktop,
                RestoreTarget::Cursor,
                RestoreTarget::Windsurf,
                RestoreTarget::VsCode,
                RestoreTarget::Codex,
                RestoreTarget::Kiro,
            ] {
                let path = target.configuration_path(&paths);
                let filesystem =
                    filesystem_for(target, &paths, Some(PRIVATE_CURRENT), valid_backup(target));

                let plan = plan_restore(&paths, &filesystem, target)
                    .expect("each selected platform backup should validate");

                assert_eq!(plan.path, path);
                assert_eq!(plan.backup_path, backup_path(&path));
                let debug = format!("{plan:?}");
                assert!(!debug.contains("current-private-restore-sentinel"));
                assert!(!debug.contains("retained Codex configuration"));
                assert!(dry_run(&plan).to_string().contains("would be restored"));
                let restorer = RecordingRestorer::default();
                let report = apply_restore(&plan, &restorer)
                    .expect("each selected platform restore should apply");
                assert!(report.to_string().contains("restored from"));
                assert_eq!(restorer.calls.borrow().len(), 1);
                assert_eq!(restorer.calls.borrow()[0].path, path);
            }
        }
    }

    #[test]
    fn apply_uses_the_exact_validated_pair_and_reports_only_structure() {
        let (_root, paths) = paths(Platform::MacOs);
        let target = RestoreTarget::Cursor;
        let retained = b"{\"mcpServers\":{\"private-name\":{\"command\":\"private-command\"}}}\n";
        let filesystem = filesystem_for(target, &paths, Some(PRIVATE_CURRENT), retained);
        let plan = plan_restore(&paths, &filesystem, target).expect("the backup should validate");
        let restorer = RecordingRestorer::default();

        let report = apply_restore(&plan, &restorer).expect("the restore should apply");

        assert_eq!(
            *restorer.calls.borrow(),
            [RestoreCall {
                path: target.configuration_path(&paths),
                target: RecordedTarget::Existing(PRIVATE_CURRENT.to_vec()),
                backup: retained.to_vec(),
            }]
        );
        let output = format!("{report}\n{report:?}");
        for private in [
            "current-private-restore-sentinel",
            "private-name",
            "private-command",
        ] {
            assert!(!output.contains(private));
        }
        assert!(output.contains("previous target is now the retained backup"));
    }

    #[test]
    fn missing_target_is_planned_as_recreation_and_equal_pair_is_a_no_op() {
        let (_root, paths) = paths(Platform::Linux);
        let retained = valid_backup(RestoreTarget::Canonical);
        let missing_filesystem = filesystem_for(RestoreTarget::Canonical, &paths, None, retained);
        let missing = plan_restore(&paths, &missing_filesystem, RestoreTarget::Canonical)
            .expect("a valid retained backup should recreate a missing target");
        assert!(dry_run(&missing).to_string().contains("would be recreated"));
        let restorer = RecordingRestorer::default();
        assert!(
            apply_restore(&missing, &restorer)
                .expect("the missing target restore should apply")
                .to_string()
                .contains("backup was preserved")
        );
        assert_eq!(restorer.calls.borrow()[0].target, RecordedTarget::Missing);

        let equal_filesystem =
            filesystem_for(RestoreTarget::Canonical, &paths, Some(retained), retained);
        let equal = plan_restore(&paths, &equal_filesystem, RestoreTarget::Canonical)
            .expect("an equal pair should validate");
        let no_op_restorer = RecordingRestorer::default();
        let report = apply_restore(&equal, &no_op_restorer).expect("a no-op should succeed");
        assert!(report.to_string().contains("no files changed"));
        assert!(no_op_restorer.calls.borrow().is_empty());
    }

    #[test]
    fn missing_or_invalid_backups_fail_without_exposing_private_input() {
        let (_root, paths) = paths(Platform::Windows);
        for target in [
            RestoreTarget::Canonical,
            RestoreTarget::ClaudeDesktop,
            RestoreTarget::Cursor,
            RestoreTarget::Windsurf,
            RestoreTarget::VsCode,
            RestoreTarget::Codex,
            RestoreTarget::Kiro,
        ] {
            let path = target.configuration_path(&paths);
            let missing_filesystem = MemoryFileSystem {
                files: BTreeMap::from([(path.clone(), PRIVATE_CURRENT.to_vec())]),
            };
            let missing = plan_restore(&paths, &missing_filesystem, target)
                .expect_err("a missing backup must fail");
            assert!(missing.to_string().contains("does not exist"));

            let private_invalid = b"invalid private-backup-restore-sentinel = [\n";
            let invalid_filesystem =
                filesystem_for(target, &paths, Some(PRIVATE_CURRENT), private_invalid);
            let invalid = plan_restore(&paths, &invalid_filesystem, target)
                .expect_err("an invalid backup must fail");
            let diagnostic = format!("{invalid}\n{invalid:?}");
            assert!(diagnostic.contains("is not a valid"));
            assert!(!diagnostic.contains("private-backup-restore-sentinel"));
            assert!(!diagnostic.contains("current-private-restore-sentinel"));
        }
    }
}
