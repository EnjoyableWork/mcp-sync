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
/// destination directory and must never replace an existing path. Replacement,
/// backup, and rollback belong to the later sync apply boundary.
pub trait FileCreator {
    fn create_new(&self, path: &Path, contents: &[u8]) -> Result<(), FileIoError>;
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

        temporary
            .persist_noclobber(path)
            .map_err(|error| FileIoError::new(FileOperation::PublishNewFile, path, error.error))?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileOperation {
    Read,
    CreateParentDirectory,
    CreateTemporaryFile,
    WriteTemporaryFile,
    SyncTemporaryFile,
    PublishNewFile,
}

impl fmt::Display for FileOperation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read => formatter.write_str("read"),
            Self::CreateParentDirectory => formatter.write_str("create parent directory for"),
            Self::CreateTemporaryFile => formatter.write_str("create temporary file for"),
            Self::WriteTemporaryFile => formatter.write_str("write temporary file for"),
            Self::SyncTemporaryFile => formatter.write_str("synchronize temporary file for"),
            Self::PublishNewFile => formatter.write_str("create"),
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

#[cfg(test)]
mod tests {
    use super::{FileCreator, FileIoError, FileOperation, FileSystem, OsFileSystem};
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
