use std::error::Error;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

/// The filesystem operations currently required above the operating-system
/// boundary.
///
/// Mutating operations will be added only when their atomicity and recovery
/// contracts are introduced by later tickets.
pub trait FileSystem {
    fn read(&self, path: &Path) -> Result<Vec<u8>, FileIoError>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct OsFileSystem;

impl FileSystem for OsFileSystem {
    fn read(&self, path: &Path) -> Result<Vec<u8>, FileIoError> {
        std::fs::read(path).map_err(|source| FileIoError::read(path, source))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileOperation {
    Read,
}

impl fmt::Display for FileOperation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read => formatter.write_str("read"),
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
    pub(crate) fn read(path: &Path, source: io::Error) -> Self {
        Self {
            operation: FileOperation::Read,
            path: path.to_owned(),
            source,
        }
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
    use super::{FileIoError, FileOperation, FileSystem, OsFileSystem};
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
