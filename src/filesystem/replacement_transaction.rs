use super::{
    FileIoError, FileMutationError, FileOperation, FileSnapshot, backup_path,
    ensure_expected_bytes, ensure_regular_file, ensure_snapshot_unchanged,
    ensure_supported_file_type, prepare_temporary_file,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
#[cfg(any(unix, test))]
use std::fs::File;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Component, Path, PathBuf};

const TRANSACTION_VERSION: u32 = 1;
const TRANSACTION_SUFFIX: &str = ".mcp-sync-transaction.json";
const TEMPORARY_PREFIX: &str = ".mcp-sync-";
const TEMPORARY_SUFFIX: &str = ".tmp";
const MAX_TRANSACTION_BYTES: u64 = 16 * 1024;

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum TransactionPhase {
    Preparing,
    Prepared,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Fingerprint {
    byte_count: u64,
    sha256: String,
}

impl Fingerprint {
    fn from_bytes(bytes: &[u8]) -> Self {
        let digest = Sha256::digest(bytes);
        let mut sha256 = String::with_capacity(digest.len() * 2);
        const HEX: &[u8; 16] = b"0123456789abcdef";
        for byte in digest {
            sha256.push(HEX[usize::from(byte >> 4)] as char);
            sha256.push(HEX[usize::from(byte & 0x0f)] as char);
        }
        Self {
            byte_count: bytes.len() as u64,
            sha256,
        }
    }

    fn matches(&self, bytes: &[u8]) -> bool {
        self.byte_count == bytes.len() as u64 && *self == Self::from_bytes(bytes)
    }

    fn is_valid(&self) -> bool {
        self.sha256.len() == 64
            && self
                .sha256
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "kebab-case", deny_unknown_fields)]
enum SnapshotFingerprint {
    Missing,
    Existing { fingerprint: Fingerprint },
}

impl SnapshotFingerprint {
    fn from_snapshot(snapshot: &FileSnapshot) -> Self {
        match snapshot {
            FileSnapshot::Missing => Self::Missing,
            FileSnapshot::Existing(bytes) => Self::Existing {
                fingerprint: Fingerprint::from_bytes(bytes),
            },
        }
    }

    fn is_valid(&self) -> bool {
        match self {
            Self::Missing => true,
            Self::Existing { fingerprint } => fingerprint.is_valid(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReplacementTransaction {
    version: u32,
    phase: TransactionPhase,
    original_target: Fingerprint,
    replacement_target: Fingerprint,
    previous_backup: SnapshotFingerprint,
    replacement_stage: String,
    backup_stage: String,
}

impl ReplacementTransaction {
    fn new(
        expected_current: &[u8],
        replacement: &[u8],
        previous_backup: &FileSnapshot,
        replacement_stage: String,
        backup_stage: String,
    ) -> Self {
        Self {
            version: TRANSACTION_VERSION,
            phase: TransactionPhase::Preparing,
            original_target: Fingerprint::from_bytes(expected_current),
            replacement_target: Fingerprint::from_bytes(replacement),
            previous_backup: SnapshotFingerprint::from_snapshot(previous_backup),
            replacement_stage,
            backup_stage,
        }
    }

    fn validate(&self, path: &Path) -> Result<(), FileMutationError> {
        if self.version != TRANSACTION_VERSION
            || !self.original_target.is_valid()
            || !self.replacement_target.is_valid()
            || !self.previous_backup.is_valid()
            || self.original_target == self.replacement_target
            || !valid_stage_name(&self.replacement_stage)
            || !valid_stage_name(&self.backup_stage)
            || self.replacement_stage == self.backup_stage
        {
            return Err(FileMutationError::InvalidReplacementTransaction {
                path: path.to_owned(),
            });
        }
        Ok(())
    }

    fn serialized(&self, path: &Path) -> Result<Vec<u8>, FileMutationError> {
        let mut bytes = serde_json::to_vec(self).map_err(|_| {
            FileMutationError::InvalidReplacementTransaction {
                path: path.to_owned(),
            }
        })?;
        bytes.push(b'\n');
        Ok(bytes)
    }

    fn replacement_stage_path(&self, path: &Path) -> Result<PathBuf, FileMutationError> {
        stage_path(path, &self.replacement_stage)
    }

    fn backup_stage_path(&self, path: &Path) -> Result<PathBuf, FileMutationError> {
        stage_path(path, &self.backup_stage)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RecoveryOutcome {
    NoPendingTransaction,
    Aborted,
    Completed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DurableBoundary {
    PreparingJournal,
    PreparedJournal,
    TargetPublished,
    BackupPublished,
}

pub(super) fn ensure_no_pending_for_read(path: &Path) -> Result<(), FileIoError> {
    let transaction = transaction_path(path);
    match fs::symlink_metadata(&transaction) {
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(FileIoError::new(
            FileOperation::InspectTransaction,
            &transaction,
            source,
        )),
        Ok(_) => Err(FileIoError::new(
            FileOperation::InspectTransaction,
            &transaction,
            io::Error::other(
                "an incomplete mcp-sync replacement requires recovery by a mutating command",
            ),
        )),
    }
}

pub(crate) fn recover_pending_replacements<'a>(
    paths: impl IntoIterator<Item = &'a Path>,
) -> Result<(), FileMutationError> {
    for path in paths {
        recover_pending_replacement(path)?;
    }
    Ok(())
}

pub(super) fn replace_existing_with_backup_snapshot_after(
    path: &Path,
    expected_current: &[u8],
    replacement: &[u8],
    previous_backup: FileSnapshot,
    after_prepared: impl FnOnce(),
    after_target_published: impl FnOnce(),
) -> Result<FileSnapshot, FileMutationError> {
    ensure_regular_file(path, true)?;
    ensure_transaction_absent(path)?;
    let backup = backup_path(path);
    ensure_expected_bytes(path, expected_current)?;
    ensure_snapshot_unchanged(&backup, &previous_backup)?;

    let mut replacement_stage = create_empty_stage(path)?;
    let mut backup_stage = create_empty_stage(path)?;
    let replacement_stage_name = stage_name(path, replacement_stage.path())?;
    let backup_stage_name = stage_name(path, backup_stage.path())?;
    let mut journal = ReplacementTransaction::new(
        expected_current,
        replacement,
        &previous_backup,
        replacement_stage_name,
        backup_stage_name,
    );
    journal.validate(path)?;
    let mut journal_bytes = journal.serialized(path)?;

    if let Err(failure) = publish_new_journal(path, &journal_bytes) {
        return fail_before_commit(
            path,
            failure,
            None,
            replacement_stage,
            backup_stage,
            &journal,
        );
    }
    if let Err(failure) = sync_parent_directory(path) {
        return fail_before_commit(
            path,
            failure,
            Some(&journal_bytes),
            replacement_stage,
            backup_stage,
            &journal,
        );
    }
    observe_test_boundary(DurableBoundary::PreparingJournal);

    if let Err(failure) = write_and_sync_stage(path, &mut replacement_stage, replacement)
        .and_then(|()| write_and_sync_stage(path, &mut backup_stage, expected_current))
    {
        return fail_before_commit(
            path,
            failure.into(),
            Some(&journal_bytes),
            replacement_stage,
            backup_stage,
            &journal,
        );
    }

    journal.phase = TransactionPhase::Prepared;
    let prepared_bytes = journal.serialized(path)?;
    if let Err(failure) = replace_journal(path, &journal_bytes, &prepared_bytes) {
        return fail_before_commit(
            path,
            failure,
            Some(&journal_bytes),
            replacement_stage,
            backup_stage,
            &journal,
        );
    }
    journal_bytes = prepared_bytes;
    if let Err(failure) = sync_parent_directory(path) {
        return fail_before_commit(
            path,
            failure,
            Some(&journal_bytes),
            replacement_stage,
            backup_stage,
            &journal,
        );
    }
    observe_test_boundary(DurableBoundary::PreparedJournal);
    after_prepared();

    let transaction = transaction_path(path);
    if let Err(failure) = ensure_regular_bytes(path, expected_current)
        .and_then(|()| ensure_snapshot_unchanged(&backup, &previous_backup))
        .and_then(|()| ensure_regular_bytes(&transaction, &journal_bytes))
        .and_then(|()| ensure_regular_bytes(replacement_stage.path(), replacement))
        .and_then(|()| ensure_regular_bytes(backup_stage.path(), expected_current))
    {
        return fail_before_commit(
            path,
            failure,
            Some(&journal_bytes),
            replacement_stage,
            backup_stage,
            &journal,
        );
    }

    let persisted_target = match replacement_stage.persist(path) {
        Ok(file) => file,
        Err(error) => {
            let failure = FileIoError::new(FileOperation::ReplaceFile, path, error.error).into();
            return fail_before_commit(
                path,
                failure,
                Some(&journal_bytes),
                error.file,
                backup_stage,
                &journal,
            );
        }
    };
    drop(persisted_target);
    if let Err(failure) = sync_parent_directory(path) {
        return fail_after_target_commit(
            path,
            expected_current,
            replacement,
            failure,
            backup_stage,
            &journal_bytes,
        );
    }
    observe_test_boundary(DurableBoundary::TargetPublished);
    after_target_published();

    if let Err(failure) = ensure_regular_bytes(path, replacement)
        .and_then(|()| ensure_snapshot_unchanged(&backup, &previous_backup))
        .and_then(|()| ensure_regular_bytes(&transaction, &journal_bytes))
        .and_then(|()| ensure_regular_bytes(backup_stage.path(), expected_current))
    {
        return fail_after_target_commit(
            path,
            expected_current,
            replacement,
            failure,
            backup_stage,
            &journal_bytes,
        );
    }

    let backup_result = match &previous_backup {
        FileSnapshot::Missing => backup_stage.persist_noclobber(&backup),
        FileSnapshot::Existing(_) => backup_stage.persist(&backup),
    };
    let persisted_backup = match backup_result {
        Ok(file) => file,
        Err(error) => {
            let operation = match previous_backup {
                FileSnapshot::Missing => FileOperation::PublishNewFile,
                FileSnapshot::Existing(_) => FileOperation::ReplaceFile,
            };
            let failure = FileIoError::new(operation, &backup, error.error).into();
            return fail_after_target_commit(
                path,
                expected_current,
                replacement,
                failure,
                error.file,
                &journal_bytes,
            );
        }
    };
    drop(persisted_backup);
    if sync_parent_directory(path).is_ok() {
        observe_test_boundary(DurableBoundary::BackupPublished);

        // Publishing the backup is the final externally visible commit. From
        // this point onward the caller must receive its rollback receipt even
        // if journal cleanup fails. A retained, exact journal is safe: reads
        // refuse it and the next locked mutation finishes the cleanup.
        let _ = cleanup_transaction(path, &journal, &journal_bytes, false);
    }
    Ok(previous_backup)
}

fn recover_pending_replacement(path: &Path) -> Result<RecoveryOutcome, FileMutationError> {
    let transaction_path = transaction_path(path);
    let Some((journal, journal_bytes)) = read_journal(path, &transaction_path)? else {
        return Ok(RecoveryOutcome::NoPendingTransaction);
    };
    let backup = backup_path(path);
    let target_bytes = read_required_regular(path)?;
    let target_is_original = journal.original_target.matches(&target_bytes);
    let target_is_replacement = journal.replacement_target.matches(&target_bytes);
    let backup_is_previous = snapshot_matches(&backup, &journal.previous_backup)?;
    let backup_is_original = snapshot_matches(
        &backup,
        &SnapshotFingerprint::Existing {
            fingerprint: journal.original_target.clone(),
        },
    )?;

    match journal.phase {
        TransactionPhase::Preparing if target_is_original && backup_is_previous => {
            cleanup_transaction(path, &journal, &journal_bytes, true)?;
            Ok(RecoveryOutcome::Aborted)
        }
        TransactionPhase::Preparing => Err(FileMutationError::AmbiguousReplacementTransaction {
            path: path.to_owned(),
        }),
        TransactionPhase::Prepared if target_is_original && backup_is_previous => {
            cleanup_transaction(path, &journal, &journal_bytes, false)?;
            Ok(RecoveryOutcome::Aborted)
        }
        TransactionPhase::Prepared if target_is_replacement && backup_is_original => {
            cleanup_transaction(path, &journal, &journal_bytes, false)?;
            Ok(RecoveryOutcome::Completed)
        }
        TransactionPhase::Prepared if target_is_replacement && backup_is_previous => {
            complete_backup_publication(path, &journal, &journal_bytes)?;
            Ok(RecoveryOutcome::Completed)
        }
        TransactionPhase::Prepared => Err(FileMutationError::AmbiguousReplacementTransaction {
            path: path.to_owned(),
        }),
    }
}

fn complete_backup_publication(
    path: &Path,
    journal: &ReplacementTransaction,
    journal_bytes: &[u8],
) -> Result<(), FileMutationError> {
    let backup = backup_path(path);
    let backup_stage_path = journal.backup_stage_path(path)?;
    read_stage(&backup_stage_path, &journal.original_target, false)?.ok_or_else(|| {
        FileMutationError::InvalidReplacementTransaction {
            path: path.to_owned(),
        }
    })?;
    ensure_fingerprint(path, &journal.replacement_target)?;
    ensure_snapshot_fingerprint(&backup, &journal.previous_backup)?;
    ensure_regular_bytes(&transaction_path(path), journal_bytes)?;
    ensure_fingerprint(&backup_stage_path, &journal.original_target)?;
    let backup_stage = take_existing_stage(&backup_stage_path)?;

    if let Err(failure) = ensure_fingerprint(path, &journal.replacement_target)
        .and_then(|()| ensure_snapshot_fingerprint(&backup, &journal.previous_backup))
        .and_then(|()| ensure_regular_bytes(&transaction_path(path), journal_bytes))
        .and_then(|()| ensure_fingerprint(&backup_stage_path, &journal.original_target))
    {
        preserve_named_stage(backup_stage);
        return Err(failure);
    }

    let operation = match journal.previous_backup {
        SnapshotFingerprint::Missing => FileOperation::PublishNewFile,
        SnapshotFingerprint::Existing { .. } => FileOperation::ReplaceFile,
    };
    let persisted = match journal.previous_backup {
        SnapshotFingerprint::Missing => backup_stage.persist_noclobber(&backup),
        SnapshotFingerprint::Existing { .. } => backup_stage.persist(&backup),
    }
    .map_err(|error| {
        preserve_named_stage(error.file);
        FileIoError::new(operation, &backup, error.error)
    })?;
    drop(persisted);
    sync_parent_directory(path)?;
    ensure_fingerprint(&backup, &journal.original_target)?;
    cleanup_transaction(path, journal, journal_bytes, false)
}

fn compensate_after_target_commit(
    path: &Path,
    replacement: &[u8],
    failure: FileMutationError,
    backup_stage: tempfile::NamedTempFile,
    journal_bytes: &[u8],
) -> Result<FileSnapshot, FileMutationError> {
    if let Err(recovery) = ensure_regular_bytes(path, replacement) {
        preserve_named_stage(backup_stage);
        return Err(FileMutationError::RecoveryFailed {
            path: path.to_owned(),
            failure: Box::new(failure),
            recovery: Box::new(recovery),
        });
    }

    let compensation = backup_stage.persist(path).map(drop).map_err(|error| {
        preserve_named_stage(error.file);
        FileIoError::new(FileOperation::ReplaceFile, path, error.error).into()
    });

    match compensation {
        Ok(()) => match remove_journal_if_unchanged(path, journal_bytes)
            .and_then(|()| sync_parent_directory(path))
        {
            Ok(()) => Err(failure),
            Err(recovery) => Err(FileMutationError::RecoveryFailed {
                path: path.to_owned(),
                failure: Box::new(failure),
                recovery: Box::new(recovery),
            }),
        },
        Err(recovery) => Err(FileMutationError::RecoveryFailed {
            path: path.to_owned(),
            failure: Box::new(failure),
            recovery: Box::new(recovery),
        }),
    }
}

fn fail_after_target_commit(
    path: &Path,
    original: &[u8],
    replacement: &[u8],
    failure: FileMutationError,
    backup_stage: tempfile::NamedTempFile,
    journal_bytes: &[u8],
) -> Result<FileSnapshot, FileMutationError> {
    if let Err(recovery) = ensure_regular_bytes(backup_stage.path(), original) {
        preserve_named_stage(backup_stage);
        return Err(FileMutationError::RecoveryFailed {
            path: path.to_owned(),
            failure: Box::new(failure),
            recovery: Box::new(recovery),
        });
    }

    compensate_after_target_commit(path, replacement, failure, backup_stage, journal_bytes)
}

fn fail_before_commit(
    path: &Path,
    failure: FileMutationError,
    journal_bytes: Option<&[u8]>,
    replacement_stage: tempfile::NamedTempFile,
    backup_stage: tempfile::NamedTempFile,
    journal: &ReplacementTransaction,
) -> Result<FileSnapshot, FileMutationError> {
    let recovery = abort_owned_transaction(
        path,
        journal_bytes,
        replacement_stage,
        backup_stage,
        journal,
    );
    match recovery {
        Ok(()) => Err(failure),
        Err(recovery) => Err(FileMutationError::RecoveryFailed {
            path: path.to_owned(),
            failure: Box::new(failure),
            recovery: Box::new(recovery),
        }),
    }
}

fn abort_owned_transaction(
    path: &Path,
    journal_bytes: Option<&[u8]>,
    replacement_stage: tempfile::NamedTempFile,
    backup_stage: tempfile::NamedTempFile,
    journal: &ReplacementTransaction,
) -> Result<(), FileMutationError> {
    let allow_empty = journal.phase == TransactionPhase::Preparing;
    close_owned_stage(
        path,
        replacement_stage,
        &journal.replacement_target,
        allow_empty,
    )?;
    close_owned_stage(path, backup_stage, &journal.original_target, allow_empty)?;
    if let Some(journal_bytes) = journal_bytes {
        remove_journal_if_unchanged(path, journal_bytes)?;
        sync_parent_directory(path)?;
    }
    Ok(())
}

fn cleanup_transaction(
    path: &Path,
    journal: &ReplacementTransaction,
    journal_bytes: &[u8],
    allow_empty: bool,
) -> Result<(), FileMutationError> {
    cleanup_stage_if_present(
        path,
        &journal.replacement_stage_path(path)?,
        &journal.replacement_target,
        allow_empty,
    )?;
    cleanup_stage_if_present(
        path,
        &journal.backup_stage_path(path)?,
        &journal.original_target,
        allow_empty,
    )?;
    remove_journal_if_unchanged(path, journal_bytes)?;
    sync_parent_directory(path)
}

fn cleanup_stage_if_present(
    transaction_target: &Path,
    stage: &Path,
    expected: &Fingerprint,
    allow_empty: bool,
) -> Result<(), FileMutationError> {
    let Some(observed) = read_stage(stage, expected, allow_empty)? else {
        return Ok(());
    };
    match read_stage(stage, expected, allow_empty)? {
        None => Ok(()),
        Some(current) if current == observed => fs::remove_file(stage)
            .map_err(|source| FileIoError::new(FileOperation::RemoveFile, stage, source).into()),
        Some(_) => Err(FileMutationError::InvalidReplacementTransaction {
            path: transaction_target.to_owned(),
        }),
    }
    .map_err(|error| match error {
        FileMutationError::InvalidReplacementTransaction { .. } => {
            FileMutationError::InvalidReplacementTransaction {
                path: transaction_target.to_owned(),
            }
        }
        other => other,
    })
}

fn close_owned_stage(
    transaction_target: &Path,
    stage: tempfile::NamedTempFile,
    expected: &Fingerprint,
    allow_empty: bool,
) -> Result<(), FileMutationError> {
    let valid = read_stage(stage.path(), expected, allow_empty)
        .map(|state| state.is_some())
        .unwrap_or(false);
    if !valid {
        preserve_named_stage(stage);
        return Err(FileMutationError::InvalidReplacementTransaction {
            path: transaction_target.to_owned(),
        });
    }
    let stage_path = stage.path().to_owned();
    stage
        .close()
        .map_err(|source| FileIoError::new(FileOperation::RemoveFile, &stage_path, source).into())
}

fn preserve_named_stage(stage: tempfile::NamedTempFile) {
    match stage.keep() {
        Ok((file, _)) => drop(file),
        Err(mut error) => error.file.disable_cleanup(true),
    }
}

fn read_journal(
    path: &Path,
    transaction_path: &Path,
) -> Result<Option<(ReplacementTransaction, Vec<u8>)>, FileMutationError> {
    let metadata = match fs::symlink_metadata(transaction_path) {
        Ok(metadata) => metadata,
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(source) => {
            return Err(FileIoError::new(
                FileOperation::InspectTransaction,
                transaction_path,
                source,
            )
            .into());
        }
    };
    ensure_supported_file_type(transaction_path, metadata.file_type())?;
    if metadata.len() > MAX_TRANSACTION_BYTES {
        return Err(FileMutationError::InvalidReplacementTransaction {
            path: path.to_owned(),
        });
    }
    let bytes = fs::read(transaction_path).map_err(|source| {
        FileIoError::new(FileOperation::ReadTransaction, transaction_path, source)
    })?;
    let journal: ReplacementTransaction = serde_json::from_slice(&bytes).map_err(|_| {
        FileMutationError::InvalidReplacementTransaction {
            path: path.to_owned(),
        }
    })?;
    journal.validate(path)?;
    if journal.serialized(path)? != bytes {
        return Err(FileMutationError::InvalidReplacementTransaction {
            path: path.to_owned(),
        });
    }
    Ok(Some((journal, bytes)))
}

fn read_required_regular(path: &Path) -> Result<Vec<u8>, FileMutationError> {
    ensure_regular_file(path, true)?;
    fs::read(path).map_err(|source| FileIoError::read(path, source).into())
}

fn read_stage(
    path: &Path,
    expected: &Fingerprint,
    allow_empty: bool,
) -> Result<Option<Vec<u8>>, FileMutationError> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(source) => {
            return Err(FileIoError::new(FileOperation::InspectFile, path, source).into());
        }
    };
    ensure_supported_file_type(path, metadata.file_type())?;
    let bytes = fs::read(path).map_err(|source| FileIoError::read(path, source))?;
    if expected.matches(&bytes) || (allow_empty && bytes.is_empty()) {
        Ok(Some(bytes))
    } else {
        Err(FileMutationError::InvalidReplacementTransaction {
            path: path.to_owned(),
        })
    }
}

fn snapshot_matches(
    path: &Path,
    expected: &SnapshotFingerprint,
) -> Result<bool, FileMutationError> {
    match (fs::symlink_metadata(path), expected) {
        (Err(source), SnapshotFingerprint::Missing) if source.kind() == io::ErrorKind::NotFound => {
            Ok(true)
        }
        (Err(source), _) if source.kind() == io::ErrorKind::NotFound => Ok(false),
        (Err(source), _) => Err(FileIoError::new(FileOperation::InspectFile, path, source).into()),
        (Ok(metadata), _) => {
            ensure_supported_file_type(path, metadata.file_type())?;
            let bytes = fs::read(path).map_err(|source| FileIoError::read(path, source))?;
            Ok(match expected {
                SnapshotFingerprint::Missing => false,
                SnapshotFingerprint::Existing { fingerprint } => fingerprint.matches(&bytes),
            })
        }
    }
}

fn ensure_snapshot_fingerprint(
    path: &Path,
    expected: &SnapshotFingerprint,
) -> Result<(), FileMutationError> {
    if snapshot_matches(path, expected)? {
        Ok(())
    } else {
        Err(FileMutationError::AmbiguousReplacementTransaction {
            path: path.to_owned(),
        })
    }
}

fn ensure_fingerprint(path: &Path, expected: &Fingerprint) -> Result<(), FileMutationError> {
    let bytes = read_required_regular(path)?;
    if expected.matches(&bytes) {
        Ok(())
    } else {
        Err(FileMutationError::AmbiguousReplacementTransaction {
            path: path.to_owned(),
        })
    }
}

fn ensure_regular_bytes(path: &Path, expected: &[u8]) -> Result<(), FileMutationError> {
    ensure_regular_file(path, true)?;
    ensure_expected_bytes(path, expected)
}

fn create_empty_stage(path: &Path) -> Result<tempfile::NamedTempFile, FileMutationError> {
    prepare_temporary_file(path, &[]).map_err(Into::into)
}

fn write_and_sync_stage(
    target: &Path,
    stage: &mut tempfile::NamedTempFile,
    contents: &[u8],
) -> Result<(), FileIoError> {
    stage
        .write_all(contents)
        .map_err(|source| FileIoError::new(FileOperation::WriteTemporaryFile, target, source))?;
    stage
        .as_file()
        .sync_all()
        .map_err(|source| FileIoError::new(FileOperation::SyncTemporaryFile, target, source))
}

fn publish_new_journal(path: &Path, bytes: &[u8]) -> Result<(), FileMutationError> {
    let transaction = transaction_path(path);
    ensure_missing(&transaction)?;
    let temporary = prepare_temporary_file(&transaction, bytes)?;
    temporary.persist_noclobber(&transaction).map_err(|error| {
        FileIoError::new(FileOperation::PublishTransaction, &transaction, error.error)
    })?;
    Ok(())
}

fn replace_journal(
    path: &Path,
    expected: &[u8],
    replacement: &[u8],
) -> Result<(), FileMutationError> {
    let transaction = transaction_path(path);
    let temporary = prepare_temporary_file(&transaction, replacement)?;
    ensure_regular_bytes(&transaction, expected)?;
    temporary.persist(&transaction).map_err(|error| {
        FileIoError::new(FileOperation::ReplaceTransaction, &transaction, error.error)
    })?;
    Ok(())
}

fn remove_journal_if_unchanged(path: &Path, expected: &[u8]) -> Result<(), FileMutationError> {
    let transaction = transaction_path(path);
    ensure_regular_file(&transaction, true)?;
    ensure_expected_bytes(&transaction, expected)?;
    fs::remove_file(&transaction)
        .map_err(|source| FileIoError::new(FileOperation::RemoveFile, &transaction, source).into())
}

fn take_existing_stage(path: &Path) -> Result<tempfile::NamedTempFile, FileMutationError> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .map_err(|source| FileIoError::new(FileOperation::OpenTransactionStage, path, source))?;
    let temporary_path = tempfile::TempPath::try_from_path(path)
        .map_err(|source| FileIoError::new(FileOperation::OpenTransactionStage, path, source))?;
    Ok(tempfile::NamedTempFile::from_parts(file, temporary_path))
}

fn ensure_transaction_absent(path: &Path) -> Result<(), FileMutationError> {
    let transaction = transaction_path(path);
    match fs::symlink_metadata(&transaction) {
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(source) => {
            Err(FileIoError::new(FileOperation::InspectTransaction, &transaction, source).into())
        }
        Ok(_) => Err(FileMutationError::PendingReplacement {
            path: path.to_owned(),
        }),
    }
}

fn ensure_missing(path: &Path) -> Result<(), FileMutationError> {
    match fs::symlink_metadata(path) {
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(FileIoError::new(FileOperation::InspectFile, path, source).into()),
        Ok(metadata) => {
            ensure_supported_file_type(path, metadata.file_type())?;
            Err(FileMutationError::PendingReplacement {
                path: path.to_owned(),
            })
        }
    }
}

fn stage_name(transaction_target: &Path, stage: &Path) -> Result<String, FileMutationError> {
    let name = stage
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| FileMutationError::InvalidReplacementTransaction {
            path: transaction_target.to_owned(),
        })?;
    if valid_stage_name(name) {
        Ok(name.to_owned())
    } else {
        Err(FileMutationError::InvalidReplacementTransaction {
            path: transaction_target.to_owned(),
        })
    }
}

fn valid_stage_name(name: &str) -> bool {
    let Some(random) = name
        .strip_prefix(TEMPORARY_PREFIX)
        .and_then(|name| name.strip_suffix(TEMPORARY_SUFFIX))
    else {
        return false;
    };
    random.len() == 6
        && random.bytes().all(|byte| byte.is_ascii_alphanumeric())
        && matches!(
            Path::new(name).components().collect::<Vec<_>>().as_slice(),
            [Component::Normal(_)]
        )
}

fn stage_path(transaction_target: &Path, name: &str) -> Result<PathBuf, FileMutationError> {
    if !valid_stage_name(name) {
        return Err(FileMutationError::InvalidReplacementTransaction {
            path: transaction_target.to_owned(),
        });
    }
    let parent = transaction_target.parent().ok_or_else(|| {
        FileMutationError::InvalidReplacementTransaction {
            path: transaction_target.to_owned(),
        }
    })?;
    Ok(parent.join(name))
}

pub(crate) fn transaction_path(path: &Path) -> PathBuf {
    let mut transaction = path.as_os_str().to_owned();
    transaction.push(TRANSACTION_SUFFIX);
    PathBuf::from(transaction)
}

#[cfg(test)]
pub(crate) fn test_recovery_shape(path: &Path) -> String {
    let transaction = transaction_path(path);
    let journal = match read_journal(path, &transaction) {
        Ok(Some((journal, _))) => journal,
        Ok(None) => return "journal=missing".to_owned(),
        Err(FileMutationError::Io(_)) => return "journal=unreadable".to_owned(),
        Err(FileMutationError::UnsupportedFileType { .. }) => {
            return "journal=unsupported".to_owned();
        }
        Err(_) => return "journal=invalid".to_owned(),
    };
    let backup = backup_path(path);
    let replacement_stage = journal.replacement_stage_path(path);
    let backup_stage = journal.backup_stage_path(path);

    format!(
        "journal=valid,target={},backup={},replacement-stage={},backup-stage={}",
        test_target_shape(path, &journal),
        test_backup_shape(&backup, &journal),
        replacement_stage
            .as_deref()
            .map_or("invalid-name", |stage| test_file_shape(
                stage,
                &journal.replacement_target
            )),
        backup_stage
            .as_deref()
            .map_or("invalid-name", |stage| test_file_shape(
                stage,
                &journal.original_target
            )),
    )
}

#[cfg(test)]
fn test_target_shape(path: &Path, journal: &ReplacementTransaction) -> &'static str {
    match test_read_regular(path) {
        TestRegularFile::Missing => "missing",
        TestRegularFile::Unreadable => "unreadable",
        TestRegularFile::Unsupported => "unsupported",
        TestRegularFile::Bytes(bytes) if journal.original_target.matches(&bytes) => "original",
        TestRegularFile::Bytes(bytes) if journal.replacement_target.matches(&bytes) => {
            "replacement"
        }
        TestRegularFile::Bytes(_) => "different",
    }
}

#[cfg(test)]
fn test_backup_shape(path: &Path, journal: &ReplacementTransaction) -> &'static str {
    match test_read_regular(path) {
        TestRegularFile::Missing
            if matches!(journal.previous_backup, SnapshotFingerprint::Missing) =>
        {
            "previous"
        }
        TestRegularFile::Missing => "missing",
        TestRegularFile::Unreadable => "unreadable",
        TestRegularFile::Unsupported => "unsupported",
        TestRegularFile::Bytes(bytes)
            if matches!(
                &journal.previous_backup,
                SnapshotFingerprint::Existing { fingerprint } if fingerprint.matches(&bytes)
            ) =>
        {
            "previous"
        }
        TestRegularFile::Bytes(bytes) if journal.original_target.matches(&bytes) => "original",
        TestRegularFile::Bytes(_) => "different",
    }
}

#[cfg(test)]
fn test_file_shape(path: &Path, expected: &Fingerprint) -> &'static str {
    match test_read_regular(path) {
        TestRegularFile::Missing => "missing",
        TestRegularFile::Unreadable => "unreadable",
        TestRegularFile::Unsupported => "unsupported",
        TestRegularFile::Bytes(bytes) if expected.matches(&bytes) => "matching",
        TestRegularFile::Bytes(_) => "different",
    }
}

#[cfg(test)]
enum TestRegularFile {
    Missing,
    Unreadable,
    Unsupported,
    Bytes(Vec<u8>),
}

#[cfg(test)]
fn test_read_regular(path: &Path) -> TestRegularFile {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(source) if source.kind() == io::ErrorKind::NotFound => {
            return TestRegularFile::Missing;
        }
        Err(_) => return TestRegularFile::Unreadable,
    };
    if !metadata.file_type().is_file() {
        return TestRegularFile::Unsupported;
    }
    fs::read(path).map_or(TestRegularFile::Unreadable, TestRegularFile::Bytes)
}

fn sync_parent_directory(path: &Path) -> Result<(), FileMutationError> {
    let parent = path.parent().ok_or_else(|| {
        FileIoError::new(
            FileOperation::SyncDirectory,
            path,
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "destination has no parent directory",
            ),
        )
    })?;

    #[cfg(unix)]
    let directory = File::open(parent)
        .map_err(|source| FileIoError::new(FileOperation::SyncDirectory, parent, source))?;

    #[cfg(unix)]
    return directory
        .sync_all()
        .map_err(|source| FileIoError::new(FileOperation::SyncDirectory, parent, source).into());

    // The Windows standard-library boundary exposes no documented, portable
    // equivalent to synchronizing a directory entry. File and journal contents
    // are synchronized before their atomic namespace operations, so the
    // process-termination guarantee does not depend on an unsupported directory
    // flush. Whole-machine and power-loss durability are outside the transaction
    // contract.
    #[cfg(windows)]
    {
        let _ = parent;
        Ok(())
    }
}

#[cfg(test)]
fn observe_test_boundary(boundary: DurableBoundary) {
    const REQUEST_ENV: &str = "MCP_SYNC_TEST_REPLACEMENT_BOUNDARY";
    const SENTINEL_ENV: &str = "MCP_SYNC_TEST_REPLACEMENT_SENTINEL";
    let expected = match boundary {
        DurableBoundary::PreparingJournal => "preparing-journal",
        DurableBoundary::PreparedJournal => "prepared-journal",
        DurableBoundary::TargetPublished => "target-published",
        DurableBoundary::BackupPublished => "backup-published",
    };
    if std::env::var(REQUEST_ENV).as_deref() != Ok(expected) {
        return;
    }
    let sentinel = std::env::var_os(SENTINEL_ENV)
        .map(PathBuf::from)
        .expect("the abrupt-process child requires a sentinel path");
    fs::write(&sentinel, expected.as_bytes())
        .expect("the abrupt-process child should publish its boundary sentinel");
    File::open(&sentinel)
        .and_then(|file| file.sync_all())
        .expect("the abrupt-process boundary sentinel should synchronize");
    loop {
        std::thread::park();
    }
}

#[cfg(not(test))]
fn observe_test_boundary(_boundary: DurableBoundary) {}

#[cfg(test)]
mod tests {
    use super::super::UnsupportedFileKind;
    use super::{
        DurableBoundary, FileMutationError, FileSnapshot, Fingerprint, RecoveryOutcome,
        ReplacementTransaction, TRANSACTION_VERSION, TransactionPhase, backup_path,
        create_empty_stage, observe_test_boundary, preserve_named_stage, publish_new_journal,
        recover_pending_replacement, replace_journal, sync_parent_directory, transaction_path,
        valid_stage_name, write_and_sync_stage,
    };
    use std::path::Path;

    #[derive(Clone, Copy)]
    enum SeedBoundary {
        PreparingJournal,
        PreparedJournal,
        TargetPublished,
        BackupPublished,
    }

    #[test]
    fn sha256_fingerprint_is_deterministic_and_validated() {
        let fingerprint = Fingerprint::from_bytes(b"abc");
        assert_eq!(fingerprint.byte_count, 3);
        assert_eq!(
            fingerprint.sha256,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert!(fingerprint.is_valid());
        assert!(fingerprint.matches(b"abc"));
        assert!(!fingerprint.matches(b"abd"));
    }

    #[test]
    fn journal_serialization_contains_only_versioned_structural_metadata() {
        let private_original = b"private original configuration value\n";
        let private_replacement = b"private replacement configuration value\n";
        let private_previous_backup = b"private previous backup value\n";
        let path = Path::new("config.json");
        let journal = ReplacementTransaction::new(
            private_original,
            private_replacement,
            &FileSnapshot::Existing(private_previous_backup.to_vec()),
            ".mcp-sync-abc123.tmp".to_owned(),
            ".mcp-sync-def456.tmp".to_owned(),
        );

        let bytes = journal.serialized(path).unwrap();
        let text = std::str::from_utf8(&bytes).unwrap();
        for private in [
            private_original.as_slice(),
            private_replacement.as_slice(),
            private_previous_backup.as_slice(),
        ] {
            assert!(
                !bytes.windows(private.len()).any(|window| window == private),
                "transaction metadata must never contain configuration bytes"
            );
        }

        let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let object = value.as_object().unwrap();
        assert_eq!(object.len(), 7);
        for field in [
            "version",
            "phase",
            "original_target",
            "replacement_target",
            "previous_backup",
            "replacement_stage",
            "backup_stage",
        ] {
            assert!(object.contains_key(field), "missing journal field {field}");
        }
        assert_eq!(object["version"], TRANSACTION_VERSION);
        assert_eq!(object["phase"], "preparing");
        assert_eq!(object["replacement_stage"], ".mcp-sync-abc123.tmp");
        assert_eq!(object["backup_stage"], ".mcp-sync-def456.tmp");
        assert!(!text.contains("private "));

        for field in ["original_target", "replacement_target"] {
            let fingerprint = object[field].as_object().unwrap();
            assert_eq!(fingerprint.len(), 2);
            assert!(fingerprint.contains_key("byte_count"));
            assert!(fingerprint.contains_key("sha256"));
        }
        let previous = object["previous_backup"].as_object().unwrap();
        assert_eq!(previous.len(), 2);
        assert_eq!(previous["state"], "existing");
        let fingerprint = previous["fingerprint"].as_object().unwrap();
        assert_eq!(fingerprint.len(), 2);
        assert!(fingerprint.contains_key("byte_count"));
        assert!(fingerprint.contains_key("sha256"));
    }

    #[test]
    fn stage_names_are_single_owned_components() {
        assert!(valid_stage_name(".mcp-sync-abc123.tmp"));
        assert!(!valid_stage_name("../.mcp-sync-abc123.tmp"));
        assert!(!valid_stage_name(".mcp-sync-abc123.tmp/child"));
        assert!(!valid_stage_name(".mcp-sync-short.tmp"));
        assert!(!valid_stage_name("foreign.tmp"));
    }

    #[test]
    fn recovery_aborts_before_commit_and_completes_after_commit_with_existing_backup() {
        for boundary in [
            SeedBoundary::PreparingJournal,
            SeedBoundary::PreparedJournal,
            SeedBoundary::TargetPublished,
            SeedBoundary::BackupPublished,
        ] {
            let fixture = tempfile::tempdir().expect("transaction fixture should be created");
            let path = fixture.path().join("config.json");
            let original = b"target generation B\n";
            let previous_backup = b"backup generation A\n";
            let replacement = b"target generation C\n";
            seed_transaction(
                &path,
                original,
                Some(previous_backup),
                replacement,
                boundary,
            );

            let outcome = recover_pending_replacement(&path)
                .expect("a journaled transaction boundary should recover");

            match boundary {
                SeedBoundary::PreparingJournal | SeedBoundary::PreparedJournal => {
                    assert_eq!(outcome, RecoveryOutcome::Aborted);
                    assert_eq!(std::fs::read(&path).unwrap(), original);
                    assert_eq!(std::fs::read(backup_path(&path)).unwrap(), previous_backup);
                }
                SeedBoundary::TargetPublished | SeedBoundary::BackupPublished => {
                    assert_eq!(outcome, RecoveryOutcome::Completed);
                    assert_eq!(std::fs::read(&path).unwrap(), replacement);
                    assert_eq!(std::fs::read(backup_path(&path)).unwrap(), original);
                }
            }
            assert_transaction_artifacts_absent(fixture.path());
            assert_eq!(
                recover_pending_replacement(&path).unwrap(),
                RecoveryOutcome::NoPendingTransaction,
                "retrying a completed recovery should be a no-op"
            );
        }
    }

    #[test]
    fn recovery_aborts_before_commit_and_completes_after_commit_without_prior_backup() {
        for boundary in [
            SeedBoundary::PreparingJournal,
            SeedBoundary::PreparedJournal,
            SeedBoundary::TargetPublished,
            SeedBoundary::BackupPublished,
        ] {
            let fixture = tempfile::tempdir().expect("transaction fixture should be created");
            let path = fixture.path().join("config.json");
            let original = b"target generation B\n";
            let replacement = b"target generation C\n";
            seed_transaction(&path, original, None, replacement, boundary);

            let outcome = recover_pending_replacement(&path)
                .expect("a journaled transaction boundary should recover");

            match boundary {
                SeedBoundary::PreparingJournal | SeedBoundary::PreparedJournal => {
                    assert_eq!(outcome, RecoveryOutcome::Aborted);
                    assert_eq!(std::fs::read(&path).unwrap(), original);
                    assert!(!backup_path(&path).exists());
                }
                SeedBoundary::TargetPublished | SeedBoundary::BackupPublished => {
                    assert_eq!(outcome, RecoveryOutcome::Completed);
                    assert_eq!(std::fs::read(&path).unwrap(), replacement);
                    assert_eq!(std::fs::read(backup_path(&path)).unwrap(), original);
                }
            }
            assert_transaction_artifacts_absent(fixture.path());
            assert_eq!(
                recover_pending_replacement(&path).unwrap(),
                RecoveryOutcome::NoPendingTransaction,
                "retrying a completed recovery should be a no-op"
            );
        }
    }

    #[test]
    fn malformed_journal_and_changed_target_fail_closed_without_cleanup() {
        let fixture = tempfile::tempdir().expect("transaction fixture should be created");
        let path = fixture.path().join("config.json");
        let backup = backup_path(&path);
        std::fs::write(&path, b"target generation B\n").unwrap();
        std::fs::write(&backup, b"backup generation A\n").unwrap();
        std::fs::write(
            transaction_path(&path),
            b"private malformed journal contents\n",
        )
        .unwrap();

        let malformed = recover_pending_replacement(&path)
            .expect_err("malformed transaction metadata must fail closed");
        assert!(matches!(
            malformed,
            super::FileMutationError::InvalidReplacementTransaction { .. }
        ));
        assert!(!malformed.to_string().contains("private malformed"));
        assert_eq!(std::fs::read(&path).unwrap(), b"target generation B\n");
        assert_eq!(std::fs::read(&backup).unwrap(), b"backup generation A\n");

        std::fs::remove_file(transaction_path(&path)).unwrap();
        seed_transaction(
            &path,
            b"target generation B\n",
            Some(b"backup generation A\n"),
            b"target generation C\n",
            SeedBoundary::PreparedJournal,
        );
        std::fs::write(&path, b"external generation D\n").unwrap();

        let changed = recover_pending_replacement(&path)
            .expect_err("an external target edit must make recovery ambiguous");
        assert!(matches!(
            changed,
            super::FileMutationError::AmbiguousReplacementTransaction { .. }
        ));
        assert_eq!(std::fs::read(&path).unwrap(), b"external generation D\n");
        assert_eq!(std::fs::read(&backup).unwrap(), b"backup generation A\n");
        assert!(transaction_path(&path).exists());
    }

    #[test]
    fn external_backup_edits_after_target_commit_fail_closed_for_both_prior_states() {
        for previous_backup in [None, Some(b"backup generation A\n".as_slice())] {
            let fixture = tempfile::tempdir().expect("transaction fixture should be created");
            let path = fixture.path().join("config.json");
            let backup = backup_path(&path);
            let original = b"target generation B\n";
            let replacement = b"target generation C\n";
            let external = b"external backup generation D\n";
            seed_transaction(
                &path,
                original,
                previous_backup,
                replacement,
                SeedBoundary::TargetPublished,
            );
            std::fs::write(&backup, external).unwrap();

            let error = recover_pending_replacement(&path)
                .expect_err("an external backup edit must make recovery ambiguous");

            assert!(matches!(
                error,
                FileMutationError::AmbiguousReplacementTransaction { .. }
            ));
            assert_eq!(std::fs::read(&path).unwrap(), replacement);
            assert_eq!(std::fs::read(&backup).unwrap(), external);
            assert!(transaction_path(&path).is_file());
        }
    }

    #[test]
    fn a_missing_required_postcommit_stage_fails_closed_for_both_prior_backup_states() {
        for previous_backup in [None, Some(b"backup generation A\n".as_slice())] {
            let fixture = tempfile::tempdir().expect("transaction fixture should be created");
            let path = fixture.path().join("config.json");
            let backup = backup_path(&path);
            let original = b"target generation B\n";
            let replacement = b"target generation C\n";
            seed_transaction(
                &path,
                original,
                previous_backup,
                replacement,
                SeedBoundary::TargetPublished,
            );
            let transaction = transaction_path(&path);
            let journal_bytes = std::fs::read(&transaction).unwrap();
            let journal: ReplacementTransaction = serde_json::from_slice(&journal_bytes).unwrap();
            let required_stage = journal.backup_stage_path(&path).unwrap();
            std::fs::remove_file(&required_stage).unwrap();

            let error = recover_pending_replacement(&path)
                .expect_err("the exact original-target stage is required after target commit");

            assert!(matches!(
                error,
                FileMutationError::InvalidReplacementTransaction { .. }
            ));
            assert_eq!(std::fs::read(&path).unwrap(), replacement);
            match previous_backup {
                Some(previous) => assert_eq!(std::fs::read(&backup).unwrap(), previous),
                None => assert!(!backup.exists()),
            }
            assert_eq!(std::fs::read(&transaction).unwrap(), journal_bytes);
            assert!(!required_stage.exists());
        }
    }

    #[test]
    fn oversized_and_traversal_bearing_journals_fail_closed() {
        let fixture = tempfile::tempdir().expect("transaction fixture should be created");
        let path = fixture.path().join("config.json");
        std::fs::write(&path, b"target generation B\n").unwrap();
        let transaction = transaction_path(&path);
        std::fs::write(&transaction, vec![b'x'; 16 * 1024 + 1]).unwrap();

        let oversized = recover_pending_replacement(&path)
            .expect_err("an oversized journal must fail closed before parsing");
        assert!(matches!(
            oversized,
            FileMutationError::InvalidReplacementTransaction { .. }
        ));
        assert_eq!(
            std::fs::metadata(&transaction).unwrap().len(),
            16 * 1024 + 1
        );
        std::fs::remove_file(&transaction).unwrap();

        seed_transaction(
            &path,
            b"target generation B\n",
            None,
            b"target generation C\n",
            SeedBoundary::PreparedJournal,
        );
        let mut journal: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&transaction).unwrap()).unwrap();
        journal["replacement_stage"] = serde_json::json!("../.mcp-sync-abc123.tmp");
        std::fs::write(&transaction, serde_json::to_vec(&journal).unwrap()).unwrap();

        let traversal = recover_pending_replacement(&path)
            .expect_err("a journal stage outside the target parent must fail closed");
        assert!(matches!(
            traversal,
            FileMutationError::InvalidReplacementTransaction { .. }
        ));
        assert!(transaction.is_file());
        assert_eq!(std::fs::read(&path).unwrap(), b"target generation B\n");
    }

    #[test]
    fn semantically_equivalent_but_noncanonical_journal_bytes_fail_closed() {
        let fixture = tempfile::tempdir().expect("transaction fixture should be created");
        let path = fixture.path().join("config.json");
        seed_transaction(
            &path,
            b"target generation B\n",
            Some(b"backup generation A\n"),
            b"target generation C\n",
            SeedBoundary::PreparedJournal,
        );
        let transaction = transaction_path(&path);
        let mut changed = vec![b' '];
        changed.extend(std::fs::read(&transaction).unwrap());
        std::fs::write(&transaction, changed).unwrap();

        let error = recover_pending_replacement(&path)
            .expect_err("only the exact canonical journal representation is owned");

        assert!(matches!(
            error,
            FileMutationError::InvalidReplacementTransaction { .. }
        ));
        assert!(transaction.is_file());
        assert_eq!(std::fs::read(&path).unwrap(), b"target generation B\n");
        assert_eq!(
            std::fs::read(backup_path(&path)).unwrap(),
            b"backup generation A\n"
        );
    }

    #[test]
    fn directory_shaped_journal_and_stage_artifacts_are_never_removed() {
        let fixture = tempfile::tempdir().expect("transaction fixture should be created");
        let path = fixture.path().join("config.json");
        let transaction = transaction_path(&path);
        std::fs::write(&path, b"target generation B\n").unwrap();
        std::fs::create_dir(&transaction).unwrap();

        let journal_error = recover_pending_replacement(&path)
            .expect_err("a directory-shaped journal must fail closed");
        assert!(matches!(
            journal_error,
            FileMutationError::UnsupportedFileType {
                path: ref unsafe_path,
                kind: UnsupportedFileKind::Directory,
            } if unsafe_path == &transaction
        ));
        assert!(transaction.is_dir());
        std::fs::remove_dir(&transaction).unwrap();

        seed_transaction(
            &path,
            b"target generation B\n",
            None,
            b"target generation C\n",
            SeedBoundary::PreparedJournal,
        );
        let journal: ReplacementTransaction =
            serde_json::from_slice(&std::fs::read(&transaction).unwrap()).unwrap();
        let stage = journal.replacement_stage_path(&path).unwrap();
        std::fs::remove_file(&stage).unwrap();
        std::fs::create_dir(&stage).unwrap();

        let stage_error = recover_pending_replacement(&path)
            .expect_err("a directory-shaped stage must fail closed");
        assert!(matches!(
            stage_error,
            FileMutationError::UnsupportedFileType {
                path: ref unsafe_path,
                kind: UnsupportedFileKind::Directory,
            } if unsafe_path == &stage
        ));
        assert!(stage.is_dir());
        assert!(transaction.is_file());
        assert_eq!(std::fs::read(&path).unwrap(), b"target generation B\n");
    }

    #[cfg(unix)]
    #[test]
    fn symbolic_link_journal_and_stage_artifacts_are_never_followed_or_removed() {
        use std::os::unix::fs::symlink;

        let fixture = tempfile::tempdir().expect("transaction fixture should be created");
        let path = fixture.path().join("config.json");
        let transaction = transaction_path(&path);
        let external = fixture.path().join("external-private-data");
        let external_bytes = b"external private bytes\n";
        std::fs::write(&path, b"target generation B\n").unwrap();
        std::fs::write(&external, external_bytes).unwrap();
        symlink(&external, &transaction).unwrap();

        let journal_error = recover_pending_replacement(&path)
            .expect_err("a symbolic-link journal must fail closed");
        assert!(matches!(
            journal_error,
            FileMutationError::UnsupportedFileType {
                path: ref unsafe_path,
                kind: UnsupportedFileKind::SymbolicLink,
            } if unsafe_path == &transaction
        ));
        assert_eq!(std::fs::read(&external).unwrap(), external_bytes);
        assert!(
            std::fs::symlink_metadata(&transaction)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        std::fs::remove_file(&transaction).unwrap();

        seed_transaction(
            &path,
            b"target generation B\n",
            None,
            b"target generation C\n",
            SeedBoundary::PreparedJournal,
        );
        let journal: ReplacementTransaction =
            serde_json::from_slice(&std::fs::read(&transaction).unwrap()).unwrap();
        let stage = journal.replacement_stage_path(&path).unwrap();
        std::fs::remove_file(&stage).unwrap();
        symlink(&external, &stage).unwrap();

        let stage_error =
            recover_pending_replacement(&path).expect_err("a symbolic-link stage must fail closed");
        assert!(matches!(
            stage_error,
            FileMutationError::UnsupportedFileType {
                path: ref unsafe_path,
                kind: UnsupportedFileKind::SymbolicLink,
            } if unsafe_path == &stage
        ));
        assert_eq!(std::fs::read(&external).unwrap(), external_bytes);
        assert!(
            std::fs::symlink_metadata(&stage)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert!(transaction.is_file());
    }

    #[test]
    fn a_corrupted_owned_stage_is_never_removed() {
        let fixture = tempfile::tempdir().expect("transaction fixture should be created");
        let path = fixture.path().join("config.json");
        seed_transaction(
            &path,
            b"target generation B\n",
            Some(b"backup generation A\n"),
            b"target generation C\n",
            SeedBoundary::PreparedJournal,
        );
        let journal_bytes = std::fs::read(transaction_path(&path)).unwrap();
        let journal: ReplacementTransaction = serde_json::from_slice(&journal_bytes).unwrap();
        let stage = journal.replacement_stage_path(&path).unwrap();
        std::fs::write(&stage, b"external stage bytes\n").unwrap();

        let error = recover_pending_replacement(&path)
            .expect_err("a digest-mismatched stage must fail closed");
        assert!(matches!(
            error,
            super::FileMutationError::InvalidReplacementTransaction { .. }
        ));
        assert_eq!(std::fs::read(&stage).unwrap(), b"external stage bytes\n");
        assert!(transaction_path(&path).exists());
    }

    fn seed_transaction(
        path: &Path,
        original: &[u8],
        previous_backup: Option<&[u8]>,
        replacement: &[u8],
        boundary: SeedBoundary,
    ) {
        std::fs::write(path, original).expect("the target generation should be seeded");
        let backup = backup_path(path);
        let previous_snapshot = match previous_backup {
            Some(bytes) => {
                std::fs::write(&backup, bytes).expect("the prior backup should be seeded");
                FileSnapshot::Existing(bytes.to_vec())
            }
            None => FileSnapshot::Missing,
        };
        let mut replacement_stage =
            create_empty_stage(path).expect("the replacement stage should be created");
        let mut backup_stage =
            create_empty_stage(path).expect("the backup stage should be created");
        let mut journal = ReplacementTransaction::new(
            original,
            replacement,
            &previous_snapshot,
            replacement_stage
                .path()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned(),
            backup_stage
                .path()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned(),
        );
        let preparing_bytes = journal.serialized(path).unwrap();
        publish_new_journal(path, &preparing_bytes).unwrap();
        sync_parent_directory(path).unwrap();
        if matches!(boundary, SeedBoundary::PreparingJournal) {
            preserve_named_stage(replacement_stage);
            preserve_named_stage(backup_stage);
            return;
        }

        write_and_sync_stage(path, &mut replacement_stage, replacement).unwrap();
        write_and_sync_stage(path, &mut backup_stage, original).unwrap();
        journal.phase = TransactionPhase::Prepared;
        let prepared_bytes = journal.serialized(path).unwrap();
        replace_journal(path, &preparing_bytes, &prepared_bytes).unwrap();
        sync_parent_directory(path).unwrap();
        if matches!(boundary, SeedBoundary::PreparedJournal) {
            preserve_named_stage(replacement_stage);
            preserve_named_stage(backup_stage);
            return;
        }

        replacement_stage.persist(path).unwrap();
        sync_parent_directory(path).unwrap();
        if matches!(boundary, SeedBoundary::TargetPublished) {
            preserve_named_stage(backup_stage);
            return;
        }

        match previous_snapshot {
            FileSnapshot::Missing => backup_stage.persist_noclobber(&backup).unwrap(),
            FileSnapshot::Existing(_) => backup_stage.persist(&backup).unwrap(),
        };
        sync_parent_directory(path).unwrap();
    }

    fn assert_transaction_artifacts_absent(directory: &Path) {
        let remaining = std::fs::read_dir(directory)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|name| {
                name.starts_with(".mcp-sync-") || name.ends_with(".mcp-sync-transaction.json")
            })
            .collect::<Vec<_>>();
        assert!(
            remaining.is_empty(),
            "unexpected transaction artifacts: {remaining:?}"
        );
    }

    #[test]
    fn test_observer_is_inert_without_a_requested_boundary() {
        for boundary in [
            DurableBoundary::PreparingJournal,
            DurableBoundary::PreparedJournal,
            DurableBoundary::TargetPublished,
            DurableBoundary::BackupPublished,
        ] {
            observe_test_boundary(boundary);
        }
    }
}
