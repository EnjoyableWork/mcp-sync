#![allow(
    dead_code,
    reason = "shared integration support exposes helpers used by different test crates"
)]

use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

#[track_caller]
pub fn assert_bytes_match(actual: &[u8], expected: &[u8], context: &str) {
    let first_difference = actual
        .iter()
        .zip(expected)
        .position(|(actual, expected)| actual != expected)
        .unwrap_or_else(|| actual.len().min(expected.len()));
    assert!(
        actual == expected,
        "{context}: bytes differ at offset {first_difference} (actual length {}, expected length {})",
        actual.len(),
        expected.len()
    );
}

#[track_caller]
pub fn assert_file_matches(path: &Path, expected: &[u8], context: &str) {
    let actual = fs::read(path).unwrap_or_else(|error| {
        panic!(
            "synthetic file {} should be readable: {error}",
            path.display()
        )
    });
    assert_bytes_match(&actual, expected, context);
}

pub struct SyntheticHome {
    root: TempDir,
    user_root: PathBuf,
}

impl SyntheticHome {
    pub fn new() -> Self {
        let root = tempfile::Builder::new()
            .prefix("mcp-sync-test-")
            .tempdir()
            .expect("synthetic configuration root should be created");
        let user_root = root.path().join("user");

        for directory in [
            user_root.join(".cache"),
            user_root.join(".config"),
            user_root.join(".local/share"),
            user_root.join(".local/state"),
            user_root.join("AppData/Local"),
            user_root.join("AppData/Roaming"),
            user_root.join("Library/Application Support"),
            root.path().join("runtime"),
            root.path().join("tmp"),
            root.path().join("xdg-config-dirs"),
        ] {
            fs::create_dir_all(&directory).unwrap_or_else(|error| {
                panic!(
                    "synthetic configuration directory {} should be created: {error}",
                    directory.display()
                )
            });
        }

        Self { root, user_root }
    }

    pub fn command(&self) -> Command {
        let coverage_profile = std::env::var_os("LLVM_PROFILE_FILE");
        let mut command = Command::new(env!("CARGO_BIN_EXE_mcp-sync"));
        command.env_clear();

        for (name, path) in self.user_locations() {
            command.env(name, path);
        }

        command.env("LANG", "C");
        command.env("LC_ALL", "C");
        command.env("MCP_SYNC_TEST_MODE", "1");
        command.env("NO_COLOR", "1");
        command.env("TZ", "UTC");
        if let Some(coverage_profile) = coverage_profile {
            command.env("LLVM_PROFILE_FILE", coverage_profile);
        }
        command
    }

    pub fn root(&self) -> &Path {
        self.root.path()
    }

    pub fn user_root(&self) -> &Path {
        &self.user_root
    }

    pub fn canonical_configuration(&self) -> PathBuf {
        self.user_root.join(".config/mcp-sync/config.json")
    }

    pub fn claude_desktop_configuration(&self) -> PathBuf {
        self.user_root
            .join("Library/Application Support/Claude/claude_desktop_config.json")
    }

    pub fn cursor_configuration(&self) -> PathBuf {
        self.user_root.join(".cursor/mcp.json")
    }

    pub fn windsurf_configuration(&self) -> PathBuf {
        self.user_root.join(".codeium/windsurf/mcp_config.json")
    }

    pub fn write_file(&self, path: &Path, contents: impl AsRef<[u8]>) {
        assert!(
            path.starts_with(self.root.path()),
            "synthetic files must remain inside the disposable root"
        );
        fs::create_dir_all(path.parent().expect("a synthetic file path has a parent"))
            .unwrap_or_else(|error| {
                panic!(
                    "synthetic parent directory {} should be created: {error}",
                    path.display()
                )
            });
        fs::write(path, contents).unwrap_or_else(|error| {
            panic!(
                "synthetic file {} should be written: {error}",
                path.display()
            )
        });
    }

    pub fn assert_command_is_isolated(&self, command: &Command) {
        let configured_environment: BTreeMap<OsString, OsString> = command
            .get_envs()
            .filter_map(|(name, value)| value.map(|value| (name.to_owned(), value.to_owned())))
            .collect();

        for (name, expected_path) in self.user_locations() {
            let configured_path = configured_environment
                .get(OsStr::new(name))
                .unwrap_or_else(|| panic!("{name} should be configured for the CLI process"));

            assert_eq!(configured_path, expected_path.as_os_str());
            self.assert_path_is_isolated(name, Path::new(configured_path));
        }
    }

    fn assert_path_is_isolated(&self, name: &str, path: &Path) {
        assert!(path.is_absolute(), "{name} should be an absolute path");
        assert!(
            path.starts_with(self.root.path()),
            "{name} should remain inside the synthetic root"
        );
    }

    fn user_locations(&self) -> [(&'static str, PathBuf); 16] {
        [
            ("APPDATA", self.user_root.join("AppData/Roaming")),
            ("CFFIXED_USER_HOME", self.user_root.clone()),
            ("HOME", self.user_root.clone()),
            ("LOCALAPPDATA", self.user_root.join("AppData/Local")),
            ("MCP_SYNC_TEST_HOME", self.user_root.clone()),
            ("MCP_SYNC_TEST_ROOT", self.root.path().to_owned()),
            ("TEMP", self.root.path().join("tmp")),
            ("TMP", self.root.path().join("tmp")),
            ("TMPDIR", self.root.path().join("tmp")),
            ("USERPROFILE", self.user_root.clone()),
            ("XDG_CACHE_HOME", self.user_root.join(".cache")),
            ("XDG_CONFIG_DIRS", self.root.path().join("xdg-config-dirs")),
            ("XDG_CONFIG_HOME", self.user_root.join(".config")),
            ("XDG_DATA_HOME", self.user_root.join(".local/share")),
            ("XDG_RUNTIME_DIR", self.root.path().join("runtime")),
            ("XDG_STATE_HOME", self.user_root.join(".local/state")),
        ]
    }
}

impl Default for SyntheticHome {
    fn default() -> Self {
        Self::new()
    }
}
