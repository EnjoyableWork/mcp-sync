use std::error::Error;
use std::fmt;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};

mod replacement_transaction;

/// Read operations required above the operating-system boundary.
pub trait FileSystem {
    fn read(&self, path: &Path) -> Result<Vec<u8>, FileIoError>;
}

/// The narrow create-only operation required by `mcp-sync init`.
///
/// Implementations must publish complete bytes from a temporary file in the
/// destination directory and must never replace an existing path. Existing-file
/// replacement uses the separate `FileReplacer` port; multi-file sync uses the
/// reversible `TransactionalFileUpdater` boundary.
pub trait FileCreator {
    fn create_new(&self, path: &Path, contents: &[u8]) -> Result<(), FileIoError>;
}

/// Atomic replacement for an existing regular file.
///
/// Implementations must validate that the destination is still byte-for-byte
/// equal to `expected_current`, publish a recoverable `.bak` copy, and replace
/// the destination from a synchronized same-directory temporary file. Symbolic
/// links and non-regular destination or backup paths must be rejected.
pub trait FileReplacer {
    fn replace_if_unchanged(
        &self,
        path: &Path,
        expected_current: &[u8],
        replacement: &[u8],
    ) -> Result<(), FileMutationError>;
}

/// Guarded restoration from one adjacent backup generation.
///
/// Implementations must require the `.bak` path to remain byte-for-byte equal
/// to `expected_backup`. A missing target is created without consuming the
/// backup. An existing target is replaced atomically and its exact prior bytes
/// become the new backup, so a valid prior target can be restored again.
/// Symbolic links and non-regular target or backup paths must be rejected.
pub trait BackupRestorer {
    fn restore_backup_if_unchanged(
        &self,
        path: &Path,
        expected_target: ExpectedFile<'_>,
        expected_backup: &[u8],
    ) -> Result<(), FileMutationError>;
}

/// Regular-file reads used to plan a restore without following symbolic links.
///
/// The optional read is for a target that may need recreation. The required
/// read is for its adjacent backup. Both return exact private bytes and reject
/// directories, symbolic links, and other non-regular file types.
pub trait RestoreFileSystem {
    fn read_optional_regular_file(&self, path: &Path)
    -> Result<Option<Vec<u8>>, FileMutationError>;

    fn read_required_regular_file(&self, path: &Path) -> Result<Vec<u8>, FileMutationError>;
}

/// The exact file state observed while a higher-level transaction is planned.
///
/// Existing bytes may contain credentials. Implementations must use them only
/// for guarded mutation and must never include them in diagnostics or debug
/// output.
#[derive(Clone, Copy)]
pub enum ExpectedFile<'a> {
    Missing,
    Existing(&'a [u8]),
}

/// Reversible target-file mutation used by multi-target sync.
///
/// A successful apply returns an opaque receipt containing enough private
/// state to restore both the target and any backup that predated the
/// transaction. Rollback must refuse to overwrite a target that no longer
/// matches the bytes written by apply.
pub trait TransactionalFileUpdater {
    type Receipt;

    fn apply_file_change(
        &self,
        path: &Path,
        expected: ExpectedFile<'_>,
        replacement: &[u8],
    ) -> Result<Self::Receipt, FileMutationError>;

    fn rollback_file_change(&self, receipt: &Self::Receipt) -> Result<(), FileMutationError>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct OsFileSystem;

impl FileSystem for OsFileSystem {
    fn read(&self, path: &Path) -> Result<Vec<u8>, FileIoError> {
        replacement_transaction::ensure_no_pending_for_read(path)?;
        std::fs::read(path).map_err(|source| FileIoError::read(path, source))
    }
}

impl FileCreator for OsFileSystem {
    fn create_new(&self, path: &Path, contents: &[u8]) -> Result<(), FileIoError> {
        let parent = path.parent().ok_or_else(|| {
            FileIoError::new(
                FileOperation::CreateParentDirectory,
                path,
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "destination has no parent directory",
                ),
            )
        })?;

        std::fs::create_dir_all(parent).map_err(|source| {
            FileIoError::new(FileOperation::CreateParentDirectory, path, source)
        })?;

        let temporary = prepare_temporary_file(path, contents)?;

        temporary
            .persist_noclobber(path)
            .map_err(|error| FileIoError::new(FileOperation::PublishNewFile, path, error.error))?;
        Ok(())
    }
}

impl FileReplacer for OsFileSystem {
    fn replace_if_unchanged(
        &self,
        path: &Path,
        expected_current: &[u8],
        replacement: &[u8],
    ) -> Result<(), FileMutationError> {
        replace_existing_with_backup(path, expected_current, replacement).map(drop)
    }
}

impl BackupRestorer for OsFileSystem {
    fn restore_backup_if_unchanged(
        &self,
        path: &Path,
        expected_target: ExpectedFile<'_>,
        expected_backup: &[u8],
    ) -> Result<(), FileMutationError> {
        match expected_target {
            ExpectedFile::Missing => restore_missing_from_backup(path, expected_backup),
            ExpectedFile::Existing(current) => {
                let previous_backup = FileSnapshot::Existing(expected_backup.to_vec());
                replace_existing_with_backup_snapshot_after(
                    path,
                    current,
                    expected_backup,
                    previous_backup,
                    || {},
                )
                .map(drop)
            }
        }
    }
}

impl RestoreFileSystem for OsFileSystem {
    fn read_optional_regular_file(
        &self,
        path: &Path,
    ) -> Result<Option<Vec<u8>>, FileMutationError> {
        replacement_transaction::ensure_no_pending_for_read(path)?;
        match snapshot_optional_regular_file(path)? {
            FileSnapshot::Missing => Ok(None),
            FileSnapshot::Existing(bytes) => Ok(Some(bytes)),
        }
    }

    fn read_required_regular_file(&self, path: &Path) -> Result<Vec<u8>, FileMutationError> {
        replacement_transaction::ensure_no_pending_for_read(path)?;
        match snapshot_optional_regular_file(path)? {
            FileSnapshot::Missing => Err(FileIoError::new(
                FileOperation::InspectFile,
                path,
                io::Error::new(io::ErrorKind::NotFound, "required file does not exist"),
            )
            .into()),
            FileSnapshot::Existing(bytes) => Ok(bytes),
        }
    }
}

/// Opaque operating-system receipt for one applied target-file change.
///
/// The receipt deliberately exposes no byte accessors and its custom debug
/// output reports only state and byte counts.
pub struct FileUpdateReceipt {
    path: PathBuf,
    original: FileSnapshot,
    replacement: Vec<u8>,
    previous_backup: Option<FileSnapshot>,
}

impl fmt::Debug for FileUpdateReceipt {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FileUpdateReceipt")
            .field("path", &self.path)
            .field("original", &self.original.redacted_description())
            .field("replacement_byte_count", &self.replacement.len())
            .field(
                "previous_backup",
                &self
                    .previous_backup
                    .as_ref()
                    .map(FileSnapshot::redacted_description),
            )
            .finish()
    }
}

impl TransactionalFileUpdater for OsFileSystem {
    type Receipt = FileUpdateReceipt;

    fn apply_file_change(
        &self,
        path: &Path,
        expected: ExpectedFile<'_>,
        replacement: &[u8],
    ) -> Result<Self::Receipt, FileMutationError> {
        match expected {
            ExpectedFile::Missing => {
                self.create_new(path, replacement)?;
                Ok(FileUpdateReceipt {
                    path: path.to_owned(),
                    original: FileSnapshot::Missing,
                    replacement: replacement.to_vec(),
                    previous_backup: None,
                })
            }
            ExpectedFile::Existing(original) => {
                let previous_backup = replace_existing_with_backup(path, original, replacement)?;
                Ok(FileUpdateReceipt {
                    path: path.to_owned(),
                    original: FileSnapshot::Existing(original.to_vec()),
                    replacement: replacement.to_vec(),
                    previous_backup: Some(previous_backup),
                })
            }
        }
    }

    fn rollback_file_change(&self, receipt: &Self::Receipt) -> Result<(), FileMutationError> {
        match &receipt.original {
            FileSnapshot::Missing => remove_if_unchanged(&receipt.path, &receipt.replacement),
            FileSnapshot::Existing(original) => {
                replace_without_backup_if_unchanged(&receipt.path, &receipt.replacement, original)?;

                let previous_backup = receipt
                    .previous_backup
                    .as_ref()
                    .expect("an existing target receipt records its prior backup state");
                restore_snapshot_if_unchanged(
                    &backup_path(&receipt.path),
                    original,
                    previous_backup,
                )
            }
        }
    }
}

#[derive(Clone)]
enum FileSnapshot {
    Missing,
    Existing(Vec<u8>),
}

impl FileSnapshot {
    fn redacted_description(&self) -> String {
        match self {
            Self::Missing => "missing".to_owned(),
            Self::Existing(bytes) => format!("existing ({} bytes)", bytes.len()),
        }
    }
}

impl fmt::Debug for FileSnapshot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.redacted_description())
    }
}

fn replace_existing_with_backup(
    path: &Path,
    expected_current: &[u8],
    replacement: &[u8],
) -> Result<FileSnapshot, FileMutationError> {
    replace_existing_with_backup_after(path, expected_current, replacement, || {})
}

fn replace_existing_with_backup_after(
    path: &Path,
    expected_current: &[u8],
    replacement: &[u8],
    after_prepared: impl FnOnce(),
) -> Result<FileSnapshot, FileMutationError> {
    ensure_regular_file(path, true)?;
    let backup_path = backup_path(path);
    let previous_backup = snapshot_optional_regular_file(&backup_path)?;

    replace_existing_with_backup_snapshot_after(
        path,
        expected_current,
        replacement,
        previous_backup,
        after_prepared,
    )
}

fn replace_existing_with_backup_snapshot_after(
    path: &Path,
    expected_current: &[u8],
    replacement: &[u8],
    previous_backup: FileSnapshot,
    after_prepared: impl FnOnce(),
) -> Result<FileSnapshot, FileMutationError> {
    replacement_transaction::replace_existing_with_backup_snapshot_after(
        path,
        expected_current,
        replacement,
        previous_backup,
        after_prepared,
        || {},
    )
}

#[cfg(test)]
fn replace_existing_with_backup_after_target(
    path: &Path,
    expected_current: &[u8],
    replacement: &[u8],
    after_target_published: impl FnOnce(),
) -> Result<FileSnapshot, FileMutationError> {
    ensure_regular_file(path, true)?;
    let previous_backup = snapshot_optional_regular_file(&backup_path(path))?;
    replacement_transaction::replace_existing_with_backup_snapshot_after(
        path,
        expected_current,
        replacement,
        previous_backup,
        || {},
        after_target_published,
    )
}

pub(crate) fn recover_pending_replacements<'a>(
    paths: impl IntoIterator<Item = &'a Path>,
) -> Result<(), FileMutationError> {
    replacement_transaction::recover_pending_replacements(paths)
}

#[cfg(test)]
pub(crate) fn replacement_transaction_path(path: &Path) -> PathBuf {
    replacement_transaction::transaction_path(path)
}

#[cfg(test)]
pub(crate) fn replacement_recovery_test_shape(path: &Path) -> String {
    replacement_transaction::test_recovery_shape(path)
}

fn restore_missing_from_backup(
    path: &Path,
    expected_backup: &[u8],
) -> Result<(), FileMutationError> {
    restore_missing_from_backup_after(path, expected_backup, || {})
}

fn restore_missing_from_backup_after(
    path: &Path,
    expected_backup: &[u8],
    after_publish: impl FnOnce(),
) -> Result<(), FileMutationError> {
    let backup_path = backup_path(path);
    ensure_snapshot_unchanged(path, &FileSnapshot::Missing)?;
    ensure_snapshot_unchanged(
        &backup_path,
        &FileSnapshot::Existing(expected_backup.to_vec()),
    )?;
    let replacement_file = prepare_temporary_file(path, expected_backup)?;

    // Recheck both inputs immediately before no-clobber publication. The
    // backup is deliberately retained because there was no current target to
    // rotate into its slot.
    ensure_snapshot_unchanged(path, &FileSnapshot::Missing)?;
    ensure_expected_bytes(&backup_path, expected_backup)?;
    replacement_file
        .persist_noclobber(path)
        .map_err(|error| FileIoError::new(FileOperation::PublishNewFile, path, error.error))?;
    after_publish();

    // If the backup changed across publication, remove only the exact target
    // bytes created here and report the stale plan. Never clobber the changed
    // backup while compensating.
    if let Err(failure) = ensure_expected_bytes(&backup_path, expected_backup) {
        return match remove_if_unchanged(path, expected_backup) {
            Ok(()) => Err(failure),
            Err(recovery) => Err(FileMutationError::RecoveryFailed {
                path: path.to_owned(),
                failure: Box::new(failure),
                recovery: Box::new(recovery),
            }),
        };
    }

    Ok(())
}

fn snapshot_optional_regular_file(path: &Path) -> Result<FileSnapshot, FileMutationError> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) => {
            ensure_supported_file_type(path, metadata.file_type())?;
            std::fs::read(path)
                .map(FileSnapshot::Existing)
                .map_err(|source| FileIoError::read(path, source).into())
        }
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(FileSnapshot::Missing),
        Err(source) => Err(FileIoError::new(FileOperation::InspectFile, path, source).into()),
    }
}

fn ensure_snapshot_unchanged(
    path: &Path,
    expected: &FileSnapshot,
) -> Result<(), FileMutationError> {
    match expected {
        FileSnapshot::Missing => match std::fs::symlink_metadata(path) {
            Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(source) => Err(FileIoError::new(FileOperation::InspectFile, path, source).into()),
            Ok(metadata) => {
                ensure_supported_file_type(path, metadata.file_type())?;
                Err(FileMutationError::ConcurrentModification {
                    path: path.to_owned(),
                })
            }
        },
        FileSnapshot::Existing(bytes) => {
            ensure_regular_file(path, true)?;
            ensure_expected_bytes(path, bytes)
        }
    }
}

fn restore_snapshot_if_unchanged(
    path: &Path,
    expected_current: &[u8],
    previous: &FileSnapshot,
) -> Result<(), FileMutationError> {
    match previous {
        FileSnapshot::Missing => remove_if_unchanged(path, expected_current),
        FileSnapshot::Existing(previous) => {
            replace_without_backup_if_unchanged(path, expected_current, previous)
        }
    }
}

fn replace_without_backup_if_unchanged(
    path: &Path,
    expected_current: &[u8],
    replacement: &[u8],
) -> Result<(), FileMutationError> {
    ensure_regular_file(path, true)?;
    let replacement_file = prepare_temporary_file(path, replacement)?;
    ensure_expected_bytes(path, expected_current)?;
    replacement_file
        .persist(path)
        .map_err(|error| FileIoError::new(FileOperation::ReplaceFile, path, error.error).into())
        .map(drop)
}

fn remove_if_unchanged(path: &Path, expected_current: &[u8]) -> Result<(), FileMutationError> {
    ensure_regular_file(path, true)?;
    ensure_expected_bytes(path, expected_current)?;
    std::fs::remove_file(path)
        .map_err(|source| FileIoError::new(FileOperation::RemoveFile, path, source).into())
}

fn prepare_temporary_file(
    path: &Path,
    contents: &[u8],
) -> Result<tempfile::NamedTempFile, FileIoError> {
    let parent = path.parent().ok_or_else(|| {
        FileIoError::new(
            FileOperation::CreateTemporaryFile,
            path,
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "destination has no parent directory",
            ),
        )
    })?;
    let mut temporary = tempfile::Builder::new()
        .prefix(".mcp-sync-")
        .suffix(".tmp")
        .tempfile_in(parent)
        .map_err(|source| FileIoError::new(FileOperation::CreateTemporaryFile, path, source))?;

    temporary
        .write_all(contents)
        .map_err(|source| FileIoError::new(FileOperation::WriteTemporaryFile, path, source))?;
    temporary
        .as_file()
        .sync_all()
        .map_err(|source| FileIoError::new(FileOperation::SyncTemporaryFile, path, source))?;
    Ok(temporary)
}

fn ensure_expected_bytes(path: &Path, expected: &[u8]) -> Result<(), FileMutationError> {
    let current = std::fs::read(path).map_err(|source| FileIoError::read(path, source))?;
    if current == expected {
        Ok(())
    } else {
        Err(FileMutationError::ConcurrentModification {
            path: path.to_owned(),
        })
    }
}

fn ensure_regular_file(path: &Path, required: bool) -> Result<(), FileMutationError> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(source) if !required && source.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(source) => {
            return Err(FileIoError::new(FileOperation::InspectFile, path, source).into());
        }
    };
    ensure_supported_file_type(path, metadata.file_type())
}

fn ensure_supported_file_type(
    path: &Path,
    file_type: std::fs::FileType,
) -> Result<(), FileMutationError> {
    let kind = if file_type.is_symlink() {
        Some(UnsupportedFileKind::SymbolicLink)
    } else if file_type.is_dir() {
        Some(UnsupportedFileKind::Directory)
    } else if file_type.is_file() {
        None
    } else {
        Some(UnsupportedFileKind::Other)
    };

    match kind {
        Some(kind) => Err(FileMutationError::UnsupportedFileType {
            path: path.to_owned(),
            kind,
        }),
        None => Ok(()),
    }
}

pub(crate) fn backup_path(path: &Path) -> PathBuf {
    let mut path = path.as_os_str().to_owned();
    path.push(".bak");
    PathBuf::from(path)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileOperation {
    Read,
    InspectFile,
    InspectTransaction,
    ReadTransaction,
    CreateParentDirectory,
    CreateTemporaryFile,
    WriteTemporaryFile,
    SyncTemporaryFile,
    SyncDirectory,
    PublishNewFile,
    PublishTransaction,
    ReplaceFile,
    ReplaceTransaction,
    OpenTransactionStage,
    RemoveFile,
}

impl fmt::Display for FileOperation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read => formatter.write_str("read"),
            Self::InspectFile => formatter.write_str("inspect"),
            Self::InspectTransaction => formatter.write_str("inspect replacement transaction for"),
            Self::ReadTransaction => formatter.write_str("read replacement transaction for"),
            Self::CreateParentDirectory => formatter.write_str("create parent directory for"),
            Self::CreateTemporaryFile => formatter.write_str("create temporary file for"),
            Self::WriteTemporaryFile => formatter.write_str("write temporary file for"),
            Self::SyncTemporaryFile => formatter.write_str("synchronize temporary file for"),
            Self::SyncDirectory => formatter.write_str("synchronize containing directory for"),
            Self::PublishNewFile => formatter.write_str("create"),
            Self::PublishTransaction => formatter.write_str("publish replacement transaction for"),
            Self::ReplaceFile => formatter.write_str("atomically replace"),
            Self::ReplaceTransaction => formatter.write_str("advance replacement transaction for"),
            Self::OpenTransactionStage => formatter.write_str("open replacement stage for"),
            Self::RemoveFile => formatter.write_str("remove"),
        }
    }
}

#[derive(Debug)]
pub struct FileIoError {
    operation: FileOperation,
    path: PathBuf,
    source: io::Error,
}

impl FileIoError {
    fn new(operation: FileOperation, path: &Path, source: io::Error) -> Self {
        Self {
            operation,
            path: path.to_owned(),
            source,
        }
    }

    pub(crate) fn read(path: &Path, source: io::Error) -> Self {
        Self::new(FileOperation::Read, path, source)
    }

    #[cfg(test)]
    pub fn operation(&self) -> FileOperation {
        self.operation
    }

    #[cfg(test)]
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn kind(&self) -> io::ErrorKind {
        self.source.kind()
    }
}

impl fmt::Display for FileIoError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "could not {} file `{}`: {}",
            self.operation,
            self.path.display(),
            self.source
        )
    }
}

impl Error for FileIoError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnsupportedFileKind {
    SymbolicLink,
    Directory,
    Other,
}

impl fmt::Display for UnsupportedFileKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SymbolicLink => formatter.write_str("symbolic link"),
            Self::Directory => formatter.write_str("directory"),
            Self::Other => formatter.write_str("non-regular file"),
        }
    }
}

#[derive(Debug)]
pub enum FileMutationError {
    Io(FileIoError),
    ConcurrentModification {
        path: PathBuf,
    },
    UnsupportedFileType {
        path: PathBuf,
        kind: UnsupportedFileKind,
    },
    PendingReplacement {
        path: PathBuf,
    },
    InvalidReplacementTransaction {
        path: PathBuf,
    },
    AmbiguousReplacementTransaction {
        path: PathBuf,
    },
    RecoveryFailed {
        path: PathBuf,
        failure: Box<FileMutationError>,
        recovery: Box<FileMutationError>,
    },
}

impl fmt::Display for FileMutationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(source) => source.fmt(formatter),
            Self::ConcurrentModification { path } => write!(
                formatter,
                "file `{}` changed while the update was being prepared; no replacement was applied",
                path.display()
            ),
            Self::UnsupportedFileType { path, kind } => write!(
                formatter,
                "refusing to replace {kind} `{}`; a regular file is required",
                path.display()
            ),
            Self::PendingReplacement { path } => write!(
                formatter,
                "file `{}` has an incomplete mcp-sync replacement; run a mutating command to recover it before retrying",
                path.display()
            ),
            Self::InvalidReplacementTransaction { path } => write!(
                formatter,
                "file `{}` has invalid or unsafe mcp-sync replacement metadata; no recovery was attempted",
                path.display()
            ),
            Self::AmbiguousReplacementTransaction { path } => write!(
                formatter,
                "file `{}` no longer matches a recoverable mcp-sync replacement state; no file was overwritten or removed",
                path.display()
            ),
            Self::RecoveryFailed {
                path,
                failure,
                recovery,
            } => write!(
                formatter,
                "file update for `{}` failed ({failure}) and recovery could not complete ({recovery})",
                path.display()
            ),
        }
    }
}

impl Error for FileMutationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(source) => Some(source),
            Self::RecoveryFailed { failure, .. } => Some(failure.as_ref()),
            Self::ConcurrentModification { .. }
            | Self::UnsupportedFileType { .. }
            | Self::PendingReplacement { .. }
            | Self::InvalidReplacementTransaction { .. }
            | Self::AmbiguousReplacementTransaction { .. } => None,
        }
    }
}

impl From<FileIoError> for FileMutationError {
    fn from(error: FileIoError) -> Self {
        Self::Io(error)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BackupRestorer, ExpectedFile, FileCreator, FileIoError, FileMutationError, FileOperation,
        FileReplacer, FileSnapshot, FileSystem, OsFileSystem, TransactionalFileUpdater,
        UnsupportedFileKind, backup_path, ensure_regular_file, ensure_snapshot_unchanged,
        replace_existing_with_backup_after, replace_existing_with_backup_after_target,
        restore_missing_from_backup_after,
    };
    use std::error::Error;
    use std::io;
    use std::path::Path;

    struct FixtureFileSystem {
        result: Result<Vec<u8>, io::ErrorKind>,
    }

    impl FileSystem for FixtureFileSystem {
        fn read(&self, path: &Path) -> Result<Vec<u8>, FileIoError> {
            match &self.result {
                Ok(bytes) => Ok(bytes.clone()),
                Err(kind) => Err(FileIoError::read(
                    path,
                    io::Error::new(*kind, "synthetic filesystem failure"),
                )),
            }
        }
    }

    #[test]
    fn operating_system_adapter_reads_only_from_the_supplied_fixture_path() {
        let fixture = tempfile::tempdir().expect("temporary filesystem fixture should be created");
        let path = fixture.path().join("config.json");
        std::fs::write(&path, b"synthetic configuration")
            .expect("synthetic configuration should be written");

        let bytes = OsFileSystem
            .read(&path)
            .expect("synthetic configuration should be readable");

        assert_eq!(bytes, b"synthetic configuration");
        assert!(path.starts_with(fixture.path()));
    }

    #[test]
    fn operating_system_adapter_publishes_complete_new_files_without_temp_artifacts() {
        let fixture = tempfile::tempdir().expect("temporary filesystem fixture should be created");
        let path = fixture.path().join("nested/config.json");

        OsFileSystem
            .create_new(&path, b"synthetic configuration\n")
            .expect("a new file should be created");

        assert_eq!(
            std::fs::read(&path).expect("the created file should be readable"),
            b"synthetic configuration\n"
        );
        let entries = std::fs::read_dir(path.parent().expect("the path has a parent"))
            .expect("the destination directory should be readable")
            .map(|entry| {
                entry
                    .expect("the directory entry should be readable")
                    .file_name()
            })
            .collect::<Vec<_>>();
        assert_eq!(entries, ["config.json"]);
    }

    #[test]
    fn operating_system_adapter_never_replaces_an_existing_file() {
        let fixture = tempfile::tempdir().expect("temporary filesystem fixture should be created");
        let path = fixture.path().join("config.json");
        std::fs::write(&path, b"original bytes\n").expect("the existing fixture should be written");

        let error = OsFileSystem
            .create_new(&path, b"replacement bytes\n")
            .expect_err("create-only publication must refuse an existing path");

        assert_eq!(error.operation(), FileOperation::PublishNewFile);
        assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(
            std::fs::read(&path).expect("the existing fixture should remain readable"),
            b"original bytes\n"
        );
        let entries = std::fs::read_dir(fixture.path())
            .expect("the fixture directory should be readable")
            .count();
        assert_eq!(entries, 1, "a failed publish should clean up its temp file");
    }

    #[test]
    fn create_failures_retain_the_target_and_leave_existing_bytes_unchanged() {
        let fixture = tempfile::tempdir().expect("temporary filesystem fixture should be created");
        let blocking_file = fixture.path().join("not-a-directory");
        let path = blocking_file.join("config.json");
        std::fs::write(&blocking_file, b"blocking sentinel\n")
            .expect("the blocking fixture should be written");

        let error = OsFileSystem
            .create_new(&path, b"new bytes\n")
            .expect_err("a file cannot be used as the destination directory");

        assert_eq!(error.operation(), FileOperation::CreateParentDirectory);
        assert_eq!(error.path(), path);
        assert!(matches!(
            error.kind(),
            io::ErrorKind::AlreadyExists | io::ErrorKind::NotADirectory
        ));
        assert_eq!(
            std::fs::read(blocking_file).expect("the blocking fixture should remain readable"),
            b"blocking sentinel\n"
        );
    }

    #[test]
    fn atomic_replacement_creates_an_exact_backup_and_cleans_up_temporary_files() {
        let fixture = tempfile::tempdir().expect("temporary filesystem fixture should be created");
        let path = fixture.path().join("config.json");
        let original = b"original configuration\n";
        std::fs::write(&path, original).expect("the original fixture should be written");

        OsFileSystem
            .replace_if_unchanged(&path, original, b"replacement configuration\n")
            .expect("a matching regular file should be replaced");

        assert_eq!(
            std::fs::read(&path).expect("the replacement should be readable"),
            b"replacement configuration\n"
        );
        assert_eq!(
            std::fs::read(backup_path(&path)).expect("the backup should be readable"),
            original
        );
        let entries = std::fs::read_dir(fixture.path())
            .expect("the fixture directory should be readable")
            .map(|entry| {
                entry
                    .expect("the directory entry should be readable")
                    .file_name()
            })
            .collect::<Vec<_>>();
        assert_eq!(entries.len(), 2);
        assert!(
            entries
                .iter()
                .all(|name| !name.to_string_lossy().starts_with(".mcp-sync-"))
        );
    }

    #[test]
    fn existing_target_restore_swaps_exact_bytes_and_is_reversible() {
        let fixture = tempfile::tempdir().expect("temporary filesystem fixture should be created");
        let path = fixture.path().join("config.json");
        let backup = backup_path(&path);
        let current = b"current private configuration\n";
        let retained = b"retained private configuration\n";
        std::fs::write(&path, current).expect("the current fixture should be written");
        std::fs::write(&backup, retained).expect("the retained fixture should be written");

        OsFileSystem
            .restore_backup_if_unchanged(&path, ExpectedFile::Existing(current), retained)
            .expect("a matching restore pair should be swapped");

        assert_eq!(
            std::fs::read(&path).expect("the restored target should be readable"),
            retained
        );
        assert_eq!(
            std::fs::read(&backup).expect("the rotated backup should be readable"),
            current
        );
        assert_no_temporary_files(fixture.path());

        OsFileSystem
            .restore_backup_if_unchanged(&path, ExpectedFile::Existing(retained), current)
            .expect("the immediately preceding valid target should be restorable again");

        assert_eq!(
            std::fs::read(&path).expect("the second restored target should be readable"),
            current
        );
        assert_eq!(
            std::fs::read(&backup).expect("the second rotated backup should be readable"),
            retained
        );
        assert_no_temporary_files(fixture.path());
    }

    #[test]
    fn missing_target_restore_recreates_exact_bytes_and_retains_the_backup() {
        let fixture = tempfile::tempdir().expect("temporary filesystem fixture should be created");
        let path = fixture.path().join("config.json");
        let backup = backup_path(&path);
        let retained = b"retained private configuration\n";
        std::fs::write(&backup, retained).expect("the retained fixture should be written");

        OsFileSystem
            .restore_backup_if_unchanged(&path, ExpectedFile::Missing, retained)
            .expect("a missing target should be recreated from its matching backup");

        assert_eq!(
            std::fs::read(&path).expect("the recreated target should be readable"),
            retained
        );
        assert_eq!(
            std::fs::read(&backup).expect("the retained backup should remain readable"),
            retained
        );
        assert_no_temporary_files(fixture.path());
    }

    #[test]
    fn restore_refuses_stale_target_or_backup_without_changing_either_file() {
        let fixture = tempfile::tempdir().expect("temporary filesystem fixture should be created");
        let path = fixture.path().join("config.json");
        let backup = backup_path(&path);
        let current = b"current private configuration\n";
        let retained = b"retained private configuration\n";
        std::fs::write(&path, current).expect("the current fixture should be written");
        std::fs::write(&backup, retained).expect("the retained fixture should be written");

        let target_error = OsFileSystem
            .restore_backup_if_unchanged(
                &path,
                ExpectedFile::Existing(b"stale current bytes\n"),
                retained,
            )
            .expect_err("a stale target observation must be refused");
        assert!(matches!(
            target_error,
            FileMutationError::ConcurrentModification { path: ref changed } if changed == &path
        ));
        assert_eq!(std::fs::read(&path).unwrap(), current);
        assert_eq!(std::fs::read(&backup).unwrap(), retained);

        let backup_error = OsFileSystem
            .restore_backup_if_unchanged(
                &path,
                ExpectedFile::Existing(current),
                b"stale backup bytes\n",
            )
            .expect_err("a stale backup observation must be refused");
        assert!(matches!(
            backup_error,
            FileMutationError::ConcurrentModification { path: ref changed } if changed == &backup
        ));
        assert_eq!(std::fs::read(&path).unwrap(), current);
        assert_eq!(std::fs::read(&backup).unwrap(), retained);
        assert_no_temporary_files(fixture.path());
    }

    #[test]
    fn restore_refuses_non_regular_backup_without_changing_the_target() {
        let fixture = tempfile::tempdir().expect("temporary filesystem fixture should be created");
        let path = fixture.path().join("config.json");
        let backup = backup_path(&path);
        let current = b"current private configuration\n";
        std::fs::write(&path, current).expect("the current fixture should be written");
        std::fs::create_dir(&backup).expect("the blocking backup directory should be created");

        let error = OsFileSystem
            .restore_backup_if_unchanged(
                &path,
                ExpectedFile::Existing(current),
                b"planned backup bytes\n",
            )
            .expect_err("a backup directory must not be restored");

        assert!(matches!(
            error,
            FileMutationError::UnsupportedFileType {
                path: ref error_path,
                kind: UnsupportedFileKind::Directory,
            } if error_path == &backup
        ));
        assert_eq!(std::fs::read(&path).unwrap(), current);
        assert!(backup.is_dir());
        assert_no_temporary_files(fixture.path());
    }

    #[test]
    fn missing_target_restore_removes_its_created_file_if_the_backup_changes_during_publish() {
        let fixture = tempfile::tempdir().expect("temporary filesystem fixture should be created");
        let path = fixture.path().join("config.json");
        let backup = backup_path(&path);
        let retained = b"retained private configuration\n";
        let concurrent = b"concurrent private backup\n";
        std::fs::write(&backup, retained).expect("the retained fixture should be written");

        let error = restore_missing_from_backup_after(&path, retained, || {
            std::fs::write(&backup, concurrent)
                .expect("the injected interruption should change the backup")
        })
        .expect_err("a backup edit across target publication must fail");

        assert!(matches!(
            error,
            FileMutationError::ConcurrentModification { path: ref changed } if changed == &backup
        ));
        assert!(
            !path.exists(),
            "the exact target created by the failed restore should be removed"
        );
        assert_eq!(
            std::fs::read(&backup).expect("the concurrent backup should remain readable"),
            concurrent
        );
        assert_no_temporary_files(fixture.path());
    }

    #[test]
    fn transactional_replacement_rollback_restores_target_and_preexisting_backup_exactly() {
        let fixture = tempfile::tempdir().expect("temporary filesystem fixture should be created");
        let path = fixture.path().join("config.json");
        let backup = backup_path(&path);
        let original = b"original private configuration\n";
        let replacement = b"replacement private configuration\n";
        let previous_backup = b"older private backup\n";
        std::fs::write(&path, original).expect("the original fixture should be written");
        std::fs::write(&backup, previous_backup)
            .expect("the previous backup fixture should be written");

        let receipt = OsFileSystem
            .apply_file_change(&path, ExpectedFile::Existing(original), replacement)
            .expect("the transactional replacement should apply");

        assert!(
            std::fs::read(&path).expect("the replacement should be readable") == replacement,
            "transactional apply should publish exact replacement bytes"
        );
        assert!(
            std::fs::read(&backup).expect("the new backup should be readable") == original,
            "transactional apply should back up exact original bytes"
        );
        let debug = format!("{receipt:?}");
        for private in [
            "original private configuration",
            "replacement private configuration",
            "older private backup",
        ] {
            assert!(!debug.contains(private));
        }

        OsFileSystem
            .rollback_file_change(&receipt)
            .expect("rollback should restore both pre-transaction files");

        assert!(
            std::fs::read(&path).expect("the original should be restored") == original,
            "rollback should restore exact original bytes"
        );
        assert!(
            std::fs::read(&backup).expect("the prior backup should be restored") == previous_backup,
            "rollback should restore exact pre-transaction backup bytes"
        );
        assert_no_temporary_files(fixture.path());
    }

    #[test]
    fn transactional_replacement_rollback_removes_a_new_transaction_backup() {
        let fixture = tempfile::tempdir().expect("temporary filesystem fixture should be created");
        let path = fixture.path().join("config.json");
        let original = b"original configuration\n";
        let replacement = b"replacement configuration\n";
        std::fs::write(&path, original).expect("the original fixture should be written");

        let receipt = OsFileSystem
            .apply_file_change(&path, ExpectedFile::Existing(original), replacement)
            .expect("the transactional replacement should apply");
        assert!(backup_path(&path).is_file());

        OsFileSystem
            .rollback_file_change(&receipt)
            .expect("rollback should restore the target and absent backup state");

        assert_eq!(
            std::fs::read(&path).expect("the original should be restored"),
            original
        );
        assert!(!backup_path(&path).exists());
        assert_no_temporary_files(fixture.path());
    }

    #[test]
    fn transactional_creation_rollback_removes_only_the_exact_created_file() {
        let fixture = tempfile::tempdir().expect("temporary filesystem fixture should be created");
        let path = fixture.path().join("nested/config.json");
        let replacement = b"new configuration\n";

        let receipt = OsFileSystem
            .apply_file_change(&path, ExpectedFile::Missing, replacement)
            .expect("the transactional creation should apply");
        assert_eq!(
            std::fs::read(&path).expect("the created target should be readable"),
            replacement
        );

        OsFileSystem
            .rollback_file_change(&receipt)
            .expect("rollback should remove the exact created target");

        assert!(!path.exists());
        assert!(!backup_path(&path).exists());
        assert_no_temporary_files(path.parent().expect("the target has a parent"));
    }

    #[test]
    fn rollback_refuses_to_clobber_a_concurrently_changed_target_and_keeps_its_backup() {
        let fixture = tempfile::tempdir().expect("temporary filesystem fixture should be created");
        let path = fixture.path().join("config.json");
        let original = b"original configuration\n";
        let replacement = b"replacement configuration\n";
        let concurrent = b"concurrent configuration\n";
        std::fs::write(&path, original).expect("the original fixture should be written");
        let receipt = OsFileSystem
            .apply_file_change(&path, ExpectedFile::Existing(original), replacement)
            .expect("the transactional replacement should apply");
        std::fs::write(&path, concurrent).expect("the concurrent edit should be written");

        let error = OsFileSystem
            .rollback_file_change(&receipt)
            .expect_err("rollback must refuse to overwrite a concurrent edit");

        assert!(matches!(
            error,
            FileMutationError::ConcurrentModification {
                path: ref error_path,
            } if error_path == &path
        ));
        assert_eq!(
            std::fs::read(&path).expect("the concurrent target should remain readable"),
            concurrent
        );
        assert_eq!(
            std::fs::read(backup_path(&path)).expect("the recovery backup should remain readable"),
            original
        );
        assert_no_temporary_files(fixture.path());
    }

    #[test]
    fn prepared_replacement_preserves_the_backup_without_clobbering_a_concurrent_edit() {
        let fixture = tempfile::tempdir().expect("temporary filesystem fixture should be created");
        let path = fixture.path().join("config.json");
        let backup = backup_path(&path);
        let original = b"original configuration\n";
        let previous_backup = b"older backup configuration\n";
        let concurrent = b"concurrent configuration\n";
        std::fs::write(&path, original).expect("the original fixture should be written");
        std::fs::write(&backup, previous_backup)
            .expect("the previous backup fixture should be written");

        let error =
            replace_existing_with_backup_after(&path, original, b"planned replacement\n", || {
                std::fs::write(&path, concurrent)
                    .expect("the injected interruption should change the target")
            })
            .expect_err("a change after journal preparation must abort replacement");

        assert!(matches!(
            error,
            FileMutationError::ConcurrentModification {
                path: ref error_path,
            } if error_path == &path
        ));
        assert_eq!(
            std::fs::read(&path).expect("the concurrent target should remain readable"),
            concurrent
        );
        assert_eq!(
            std::fs::read(&backup).expect("the prior backup should be restored"),
            previous_backup
        );
        assert_no_temporary_files(fixture.path());
    }

    #[test]
    fn precommit_replacement_refuses_concurrent_target_and_backup_edits_without_compensation() {
        let fixture = tempfile::tempdir().expect("temporary filesystem fixture should be created");
        let path = fixture.path().join("config.json");
        let backup = backup_path(&path);
        let original = b"original private configuration\n";
        let previous_backup = b"older private backup\n";
        let concurrent_target = b"concurrent private target\n";
        let concurrent_backup = b"concurrent private backup\n";
        std::fs::write(&path, original).expect("the original fixture should be written");
        std::fs::write(&backup, previous_backup)
            .expect("the previous backup fixture should be written");

        let error = replace_existing_with_backup_after(
            &path,
            original,
            b"planned private replacement\n",
            || {
                std::fs::write(&path, concurrent_target)
                    .expect("the injected interruption should change the target");
                std::fs::write(&backup, concurrent_backup)
                    .expect("the injected interruption should change the backup");
            },
        )
        .expect_err("concurrent target and backup edits must abort before the commit point");

        assert!(matches!(
            &error,
            FileMutationError::ConcurrentModification {
                path: failure_path,
            } if failure_path == &path
        ));
        let diagnostic = error.to_string();
        assert!(
            diagnostic.contains("changed while the update was being prepared"),
            "the precommit refusal should remain explicit and actionable"
        );
        for private in [
            "original private configuration",
            "older private backup",
            "concurrent private target",
            "concurrent private backup",
            "planned private replacement",
        ] {
            assert!(!diagnostic.contains(private));
        }
        assert!(
            std::fs::read(&path).expect("the concurrent target should remain readable")
                == concurrent_target,
            "recovery must not clobber a concurrent target edit"
        );
        assert!(
            std::fs::read(&backup).expect("the concurrent backup should remain readable")
                == concurrent_backup,
            "recovery must not clobber a concurrent backup edit"
        );
        assert_no_temporary_files(fixture.path());
    }

    #[test]
    fn postcommit_backup_edit_compensates_the_target_without_clobbering_the_edit() {
        for previous_backup in [None, Some(b"older private backup\n".as_slice())] {
            let fixture =
                tempfile::tempdir().expect("temporary filesystem fixture should be created");
            let path = fixture.path().join("config.json");
            let backup = backup_path(&path);
            let original = b"original private configuration\n";
            let replacement = b"planned private replacement\n";
            let concurrent_backup = b"concurrent private backup\n";
            std::fs::write(&path, original).unwrap();
            if let Some(previous) = previous_backup {
                std::fs::write(&backup, previous).unwrap();
            }

            let error =
                replace_existing_with_backup_after_target(&path, original, replacement, || {
                    std::fs::write(&backup, concurrent_backup).unwrap()
                })
                .expect_err("a postcommit backup edit must fail with safe target compensation");

            assert!(matches!(
                error,
                FileMutationError::ConcurrentModification {
                    path: ref error_path,
                } if error_path == &backup
            ));
            assert_eq!(std::fs::read(&path).unwrap(), original);
            assert_eq!(std::fs::read(&backup).unwrap(), concurrent_backup);
            assert!(!super::replacement_transaction_path(&path).exists());
            assert_no_temporary_files(fixture.path());
        }
    }

    #[test]
    fn postcommit_target_edit_preserves_the_original_stage_and_fails_redacted() {
        let fixture = tempfile::tempdir().expect("temporary filesystem fixture should be created");
        let path = fixture.path().join("config.json");
        let backup = backup_path(&path);
        let original = b"original private configuration\n";
        let previous_backup = b"older private backup\n";
        let replacement = b"planned private replacement\n";
        let concurrent_target = b"concurrent private target\n";
        std::fs::write(&path, original).unwrap();
        std::fs::write(&backup, previous_backup).unwrap();

        let error = replace_existing_with_backup_after_target(&path, original, replacement, || {
            std::fs::write(&path, concurrent_target).unwrap()
        })
        .expect_err("a postcommit target edit must make compensation fail closed");

        assert!(matches!(&error, FileMutationError::RecoveryFailed { .. }));
        assert_eq!(std::fs::read(&path).unwrap(), concurrent_target);
        assert_eq!(std::fs::read(&backup).unwrap(), previous_backup);
        let transaction = super::replacement_transaction_path(&path);
        let journal_bytes = std::fs::read(&transaction).unwrap();
        let journal: serde_json::Value = serde_json::from_slice(&journal_bytes).unwrap();
        let stage_name = journal["backup_stage"].as_str().unwrap();
        let original_stage = path.parent().unwrap().join(stage_name);
        assert_eq!(std::fs::read(&original_stage).unwrap(), original);

        let diagnostic = error.to_string();
        for private in [
            "original private configuration",
            "older private backup",
            "planned private replacement",
            "concurrent private target",
            "sha256",
        ] {
            assert!(!diagnostic.contains(private));
        }
    }

    #[test]
    fn missing_snapshot_guard_rejects_created_paths_and_inspection_failures() {
        let fixture = tempfile::tempdir().expect("temporary filesystem fixture should be created");
        let path = fixture.path().join("snapshot.json");

        ensure_snapshot_unchanged(&path, &FileSnapshot::Missing)
            .expect("an optional path should remain missing");

        std::fs::write(&path, b"concurrent snapshot bytes\n")
            .expect("a concurrent snapshot should be created");
        let created = ensure_snapshot_unchanged(&path, &FileSnapshot::Missing)
            .expect_err("a newly created path must invalidate a missing snapshot");
        assert!(matches!(
            created,
            FileMutationError::ConcurrentModification {
                path: ref error_path,
            } if error_path == &path
        ));

        let invalid = fixture.path().join("invalid\0snapshot.json");
        let inspection = ensure_snapshot_unchanged(&invalid, &FileSnapshot::Missing)
            .expect_err("inspection errors other than not-found must not look like absence");
        assert!(matches!(
            inspection,
            FileMutationError::Io(ref source) if source.kind() != io::ErrorKind::NotFound
        ));
    }

    #[test]
    fn regular_file_guard_distinguishes_optional_absence_from_required_or_invalid_paths() {
        let fixture = tempfile::tempdir().expect("temporary filesystem fixture should be created");
        let missing = fixture.path().join("missing.json");

        ensure_regular_file(&missing, false).expect("an optional regular file may be absent");
        let required = ensure_regular_file(&missing, true)
            .expect_err("a required regular file must not be absent");
        assert!(matches!(
            required,
            FileMutationError::Io(ref source) if source.kind() == io::ErrorKind::NotFound
        ));

        let invalid_path = fixture.path().join("invalid\0optional.json");
        let invalid = ensure_regular_file(&invalid_path, false)
            .expect_err("optional paths must preserve non-not-found inspection failures");
        assert!(matches!(
            invalid,
            FileMutationError::Io(ref source) if source.kind() != io::ErrorKind::NotFound
        ));
    }

    fn assert_no_temporary_files(directory: &Path) {
        let has_temporary = std::fs::read_dir(directory)
            .expect("the fixture directory should be readable")
            .filter_map(Result::ok)
            .any(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".mcp-sync-")
            });
        assert!(!has_temporary, "temporary files should be cleaned up");
    }

    #[test]
    fn replacement_refuses_stale_expected_bytes_before_creating_a_backup() {
        let fixture = tempfile::tempdir().expect("temporary filesystem fixture should be created");
        let path = fixture.path().join("config.json");
        std::fs::write(&path, b"newer configuration\n")
            .expect("the newer fixture should be written");

        let error = OsFileSystem
            .replace_if_unchanged(&path, b"stale configuration\n", b"replacement\n")
            .expect_err("a stale expected value must not be replaced");

        assert!(matches!(
            error,
            FileMutationError::ConcurrentModification {
                path: ref error_path,
            }
                if error_path == &path
        ));
        assert_eq!(
            std::fs::read(&path).expect("the newer fixture should remain readable"),
            b"newer configuration\n"
        );
        assert!(!backup_path(&path).exists());
        assert_eq!(
            std::fs::read_dir(fixture.path())
                .expect("the fixture directory should be readable")
                .count(),
            1
        );
    }

    #[test]
    fn replacement_refuses_a_non_regular_backup_without_changing_the_source() {
        let fixture = tempfile::tempdir().expect("temporary filesystem fixture should be created");
        let path = fixture.path().join("config.json");
        let backup = backup_path(&path);
        std::fs::write(&path, b"original configuration\n")
            .expect("the original fixture should be written");
        std::fs::create_dir(&backup).expect("the blocking backup directory should be created");

        let error = OsFileSystem
            .replace_if_unchanged(
                &path,
                b"original configuration\n",
                b"replacement configuration\n",
            )
            .expect_err("a backup directory must not be replaced");

        assert!(matches!(
            error,
            FileMutationError::UnsupportedFileType {
                path: ref error_path,
                kind: UnsupportedFileKind::Directory,
            } if error_path == &backup
        ));
        assert_eq!(
            std::fs::read(&path).expect("the original fixture should remain readable"),
            b"original configuration\n"
        );
        assert!(backup.is_dir());
    }

    #[cfg(unix)]
    #[test]
    fn replacement_refuses_symbolic_links_without_touching_their_targets() {
        use std::os::unix::fs::symlink;

        let fixture = tempfile::tempdir().expect("temporary filesystem fixture should be created");
        let target = fixture.path().join("target.json");
        let path = fixture.path().join("config.json");
        std::fs::write(&target, b"target configuration\n")
            .expect("the target fixture should be written");
        symlink(&target, &path).expect("the symbolic link fixture should be created");

        let error = OsFileSystem
            .replace_if_unchanged(
                &path,
                b"target configuration\n",
                b"replacement configuration\n",
            )
            .expect_err("a symbolic link must not be replaced");

        assert!(matches!(
            error,
            FileMutationError::UnsupportedFileType {
                path: ref error_path,
                kind: UnsupportedFileKind::SymbolicLink,
            } if error_path == &path
        ));
        assert!(
            std::fs::symlink_metadata(&path)
                .expect("the symbolic link should remain")
                .file_type()
                .is_symlink()
        );
        assert_eq!(
            std::fs::read(&target).expect("the target should remain readable"),
            b"target configuration\n"
        );
        assert!(!backup_path(&path).exists());
    }

    #[test]
    fn operating_system_failures_retain_operation_path_kind_and_source() {
        let fixture = tempfile::tempdir().expect("temporary filesystem fixture should be created");
        let path = fixture.path().join("missing.json");

        let error = OsFileSystem
            .read(&path)
            .expect_err("a missing fixture should fail");

        assert_eq!(error.operation(), FileOperation::Read);
        assert_eq!(error.path(), path);
        assert_eq!(error.kind(), io::ErrorKind::NotFound);
        assert!(error.source().is_some());
        assert!(error.to_string().contains("could not read file"));
        assert!(error.to_string().contains("missing.json"));
    }

    #[test]
    fn injected_filesystems_preserve_context_for_non_os_failures() {
        let filesystem = FixtureFileSystem {
            result: Err(io::ErrorKind::PermissionDenied),
        };
        let path = Path::new("/synthetic/config.json");

        let error = filesystem
            .read(path)
            .expect_err("the injected permission failure should be returned");

        assert_eq!(error.operation(), FileOperation::Read);
        assert_eq!(error.path(), path);
        assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
        assert!(error.to_string().contains("synthetic filesystem failure"));
    }

    #[test]
    fn injected_filesystems_can_supply_deterministic_bytes_without_io() {
        let filesystem = FixtureFileSystem {
            result: Ok(b"fixture bytes".to_vec()),
        };

        let bytes = filesystem
            .read(Path::new("/synthetic/config.json"))
            .expect("the injected bytes should be returned");

        assert_eq!(bytes, b"fixture bytes");
    }
}
