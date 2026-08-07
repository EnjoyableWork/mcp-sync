use std::error::Error;
use std::fmt;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Read operations required above the operating-system boundary.
pub trait FileSystem {
    fn read(&self, path: &Path) -> Result<Vec<u8>, FileIoError>;
}

/// The narrow create-only operation required by `mcp-sync init`.
///
/// Implementations must publish complete bytes from a temporary file in the
/// destination directory and must never replace an existing path. Existing-file
/// replacement uses the separate `FileReplacer` port; multi-file rollback
/// belongs to the later sync apply boundary.
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

#[derive(Clone, Copy, Debug, Default)]
pub struct OsFileSystem;

impl FileSystem for OsFileSystem {
    fn read(&self, path: &Path) -> Result<Vec<u8>, FileIoError> {
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
        ensure_regular_file(path, true)?;
        let backup_path = backup_path(path);
        ensure_regular_file(&backup_path, false)?;

        let replacement_file = prepare_temporary_file(path, replacement)?;
        let backup_file = prepare_temporary_file(&backup_path, expected_current)?;

        ensure_expected_bytes(path, expected_current)?;
        backup_file.persist(&backup_path).map_err(|error| {
            FileIoError::new(FileOperation::ReplaceFile, &backup_path, error.error)
        })?;

        // Check again after publishing the backup so an edit observed during
        // preparation cannot be overwritten by the final atomic rename.
        ensure_regular_file(path, true)?;
        ensure_expected_bytes(path, expected_current)?;
        replacement_file
            .persist(path)
            .map_err(|error| FileIoError::new(FileOperation::ReplaceFile, path, error.error))?;
        Ok(())
    }
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
    let file_type = metadata.file_type();
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
    CreateParentDirectory,
    CreateTemporaryFile,
    WriteTemporaryFile,
    SyncTemporaryFile,
    PublishNewFile,
    ReplaceFile,
}

impl fmt::Display for FileOperation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read => formatter.write_str("read"),
            Self::InspectFile => formatter.write_str("inspect"),
            Self::CreateParentDirectory => formatter.write_str("create parent directory for"),
            Self::CreateTemporaryFile => formatter.write_str("create temporary file for"),
            Self::WriteTemporaryFile => formatter.write_str("write temporary file for"),
            Self::SyncTemporaryFile => formatter.write_str("synchronize temporary file for"),
            Self::PublishNewFile => formatter.write_str("create"),
            Self::ReplaceFile => formatter.write_str("atomically replace"),
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

    pub fn operation(&self) -> FileOperation {
        self.operation
    }

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
        }
    }
}

impl Error for FileMutationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(source) => Some(source),
            Self::ConcurrentModification { .. } | Self::UnsupportedFileType { .. } => None,
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
        FileCreator, FileIoError, FileMutationError, FileOperation, FileReplacer, FileSystem,
        OsFileSystem, UnsupportedFileKind, backup_path,
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
