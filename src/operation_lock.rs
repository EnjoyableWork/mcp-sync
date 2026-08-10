use std::error::Error;
use std::fmt;
use std::fs::{self, File, OpenOptions, TryLockError};
use std::io;
use std::path::Path;

/// The open handle is the lifetime of one exclusive mutating operation.
///
/// The lock file itself is persistent coordination metadata. Dropping this
/// guard closes the handle and releases the operating-system lock; normal
/// operation never deletes the file.
#[must_use = "the operation lock must be held until planning, apply, and rollback finish"]
pub struct OperationLockGuard {
    _file: File,
}

impl fmt::Debug for OperationLockGuard {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("OperationLockGuard")
    }
}

/// Try to own the mutation boundary for one canonical configuration root.
///
/// Acquisition never waits. The empty persistent file carries no owner or
/// process data, so contention diagnostics cannot disclose either.
pub fn try_acquire(path: &Path) -> Result<OperationLockGuard, OperationLockError> {
    let parent = path.parent().ok_or(OperationLockError::MissingParent)?;
    fs::create_dir_all(parent).map_err(|source| OperationLockError::CreateDirectory { source })?;

    let file = match OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(path)
    {
        Ok(file) => file,
        Err(source) if source.kind() == io::ErrorKind::AlreadyExists => {
            validate_path(path)?;
            OpenOptions::new()
                .read(true)
                .write(true)
                .open(path)
                .map_err(|source| OperationLockError::Open { source })?
        }
        Err(source) => return Err(OperationLockError::Open { source }),
    };

    validate_file(&file)?;
    match file.try_lock() {
        Ok(()) => {}
        Err(TryLockError::WouldBlock) => return Err(OperationLockError::Contended),
        Err(TryLockError::Error(source)) => {
            return Err(OperationLockError::Acquire { source });
        }
    }

    // Recheck after acquisition so an ordinary replacement between inspection
    // and open fails closed instead of silently accepting an unsafe artifact.
    validate_path(path)?;
    validate_file(&file)?;

    Ok(OperationLockGuard { _file: file })
}

fn validate_path(path: &Path) -> Result<(), OperationLockError> {
    let metadata =
        fs::symlink_metadata(path).map_err(|source| OperationLockError::Inspect { source })?;
    validate_metadata(&metadata)
}

fn validate_file(file: &File) -> Result<(), OperationLockError> {
    let metadata = file
        .metadata()
        .map_err(|source| OperationLockError::Inspect { source })?;
    validate_metadata(&metadata)
}

fn validate_metadata(metadata: &fs::Metadata) -> Result<(), OperationLockError> {
    if !metadata.file_type().is_file() {
        return Err(OperationLockError::NonRegular);
    }
    if metadata.len() != 0 {
        return Err(OperationLockError::NonEmpty);
    }
    Ok(())
}

#[derive(Debug)]
pub enum OperationLockError {
    MissingParent,
    CreateDirectory { source: io::Error },
    Inspect { source: io::Error },
    NonRegular,
    NonEmpty,
    Open { source: io::Error },
    Contended,
    Acquire { source: io::Error },
}

impl fmt::Display for OperationLockError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingParent => formatter.write_str(
                "cannot establish exclusive configuration access because the operation lock has no parent directory",
            ),
            Self::CreateDirectory { source } => write!(
                formatter,
                "cannot create the mcp-sync operation-lock directory: {source}"
            ),
            Self::Inspect { source } => write!(
                formatter,
                "cannot inspect the mcp-sync operation lock safely: {source}"
            ),
            Self::NonRegular => formatter.write_str(
                "the mcp-sync operation lock is not a regular file; replace it only when no mcp-sync operation is running",
            ),
            Self::NonEmpty => formatter.write_str(
                "the mcp-sync operation lock is not empty; preserve it for inspection and replace it only when no mcp-sync operation is running",
            ),
            Self::Open { source } => {
                write!(formatter, "cannot open the mcp-sync operation lock: {source}")
            }
            Self::Contended => formatter.write_str(
                "another mutating mcp-sync operation is already in progress; retry after it finishes",
            ),
            Self::Acquire { source } => write!(
                formatter,
                "cannot acquire exclusive access for this mcp-sync operation: {source}"
            ),
        }
    }
}

impl Error for OperationLockError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CreateDirectory { source }
            | Self::Inspect { source }
            | Self::Open { source }
            | Self::Acquire { source } => Some(source),
            Self::MissingParent | Self::NonRegular | Self::NonEmpty | Self::Contended => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{OperationLockError, try_acquire};
    use std::fs;
    use std::path::PathBuf;

    fn lock_path(root: &tempfile::TempDir, name: &str) -> PathBuf {
        root.path().join(name).join("mcp-sync/operation.lock")
    }

    #[test]
    fn acquisition_creates_persistent_empty_coordination_metadata() {
        let root = tempfile::tempdir().expect("temporary lock root should be created");
        let path = lock_path(&root, "first");

        let guard = try_acquire(&path).expect("the first operation should acquire its lock");

        assert!(path.is_file());
        assert_eq!(format!("{guard:?}"), "OperationLockGuard");
        assert!(!format!("{guard:?}").contains(&path.to_string_lossy().into_owned()));
        assert!(
            fs::read(&path)
                .expect("the lock should remain readable")
                .is_empty()
        );
        drop(guard);
        assert!(
            path.is_file(),
            "releasing the lock must not delete its inode"
        );
        assert!(
            fs::read(path)
                .expect("the lock should remain readable")
                .is_empty()
        );
    }

    #[test]
    fn same_root_fails_fast_until_the_guard_is_dropped() {
        let root = tempfile::tempdir().expect("temporary lock root should be created");
        let path = lock_path(&root, "same-root");
        let first = try_acquire(&path).expect("the first operation should acquire its lock");

        assert!(matches!(
            try_acquire(&path),
            Err(OperationLockError::Contended)
        ));

        drop(first);
        let second = try_acquire(&path).expect("drop should release the operating-system lock");
        drop(second);
    }

    #[test]
    fn different_canonical_roots_do_not_block_each_other() {
        let root = tempfile::tempdir().expect("temporary lock root should be created");
        let first_path = lock_path(&root, "first-root");
        let second_path = lock_path(&root, "second-root");

        let first = try_acquire(&first_path).expect("the first root should lock");
        let second = try_acquire(&second_path).expect("the second root should lock independently");

        drop(second);
        drop(first);
    }

    #[test]
    fn non_regular_and_non_empty_lock_artifacts_fail_closed() {
        let root = tempfile::tempdir().expect("temporary lock root should be created");
        let directory_path = lock_path(&root, "directory");
        fs::create_dir_all(&directory_path).expect("directory-shaped lock should be created");
        assert!(matches!(
            try_acquire(&directory_path),
            Err(OperationLockError::NonRegular)
        ));

        let non_empty_path = lock_path(&root, "non-empty");
        fs::create_dir_all(
            non_empty_path
                .parent()
                .expect("the lock path should have a parent"),
        )
        .expect("lock parent should be created");
        fs::write(&non_empty_path, b"must-not-appear")
            .expect("non-empty lock fixture should be written");
        let error = try_acquire(&non_empty_path)
            .expect_err("non-empty coordination metadata should fail closed");
        assert!(matches!(error, OperationLockError::NonEmpty));
        assert!(!error.to_string().contains("must-not-appear"));
    }

    #[cfg(unix)]
    #[test]
    fn symbolic_link_lock_artifacts_fail_closed() {
        use std::os::unix::fs::symlink;

        let root = tempfile::tempdir().expect("temporary lock root should be created");
        let path = lock_path(&root, "symlink");
        fs::create_dir_all(path.parent().expect("the lock path should have a parent"))
            .expect("lock parent should be created");
        let target = root.path().join("empty-target");
        fs::write(&target, b"").expect("symlink target should be created");
        symlink(target, &path).expect("symbolic link fixture should be created");

        assert!(matches!(
            try_acquire(&path),
            Err(OperationLockError::NonRegular)
        ));
    }
}
