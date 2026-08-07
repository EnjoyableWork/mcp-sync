use std::collections::BTreeMap;
use std::env;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

static NEXT_SYNTHETIC_HOME_ID: AtomicUsize = AtomicUsize::new(0);

pub struct SyntheticHome {
    root: PathBuf,
    user_root: PathBuf,
}

impl SyntheticHome {
    pub fn new() -> Self {
        let root = Self::create_unique_root();
        let user_root = root.join("user");

        for directory in [
            user_root.join(".cache"),
            user_root.join(".config"),
            user_root.join(".local/share"),
            user_root.join(".local/state"),
            user_root.join("AppData/Local"),
            user_root.join("AppData/Roaming"),
            user_root.join("Library/Application Support"),
            root.join("runtime"),
            root.join("tmp"),
            root.join("xdg-config-dirs"),
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
        command
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
            assert!(
                Path::new(configured_path).starts_with(&self.root),
                "{name} should remain inside the synthetic root"
            );
        }
    }

    fn create_unique_root() -> PathBuf {
        for _ in 0..1_024 {
            let identifier = NEXT_SYNTHETIC_HOME_ID.fetch_add(1, Ordering::Relaxed);
            let candidate =
                env::temp_dir().join(format!("mcp-sync-test-{}-{identifier}", process::id()));

            match fs::create_dir(&candidate) {
                Ok(()) => return candidate,
                Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
                Err(error) => {
                    panic!(
                        "synthetic configuration root {} should be created: {error}",
                        candidate.display()
                    )
                }
            }
        }

        panic!("a unique synthetic configuration root should be available");
    }

    fn user_locations(&self) -> [(&'static str, PathBuf); 16] {
        [
            ("APPDATA", self.user_root.join("AppData/Roaming")),
            ("CFFIXED_USER_HOME", self.user_root.clone()),
            ("HOME", self.user_root.clone()),
            ("LOCALAPPDATA", self.user_root.join("AppData/Local")),
            ("MCP_SYNC_TEST_HOME", self.user_root.clone()),
            ("MCP_SYNC_TEST_ROOT", self.root.clone()),
            ("TEMP", self.root.join("tmp")),
            ("TMP", self.root.join("tmp")),
            ("TMPDIR", self.root.join("tmp")),
            ("USERPROFILE", self.user_root.clone()),
            ("XDG_CACHE_HOME", self.user_root.join(".cache")),
            ("XDG_CONFIG_DIRS", self.root.join("xdg-config-dirs")),
            ("XDG_CONFIG_HOME", self.user_root.join(".config")),
            ("XDG_DATA_HOME", self.user_root.join(".local/share")),
            ("XDG_RUNTIME_DIR", self.root.join("runtime")),
            ("XDG_STATE_HOME", self.user_root.join(".local/state")),
        ]
    }
}

impl Default for SyntheticHome {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for SyntheticHome {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_dir_all(&self.root)
            && error.kind() != ErrorKind::NotFound
            && !thread::panicking()
        {
            panic!(
                "synthetic configuration root {} should be removed: {error}",
                self.root.display()
            );
        }
    }
}
