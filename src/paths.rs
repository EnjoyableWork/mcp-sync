use std::env;
use std::ffi::OsString;
use std::fmt;
use std::path::{Component, Path, PathBuf};

const HOME: &str = "HOME";
const XDG_CONFIG_HOME: &str = "XDG_CONFIG_HOME";

/// Supplies only the environment values needed to resolve configuration
/// locations. Tests provide an isolated implementation instead of reading the
/// test runner's process environment.
pub trait Environment {
    fn value(&self, name: &'static str) -> Option<OsString>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ProcessEnvironment;

impl Environment for ProcessEnvironment {
    fn value(&self, name: &'static str) -> Option<OsString> {
        env::var_os(name)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MacOsConfigurationPaths {
    user_home: PathBuf,
    configuration_home: PathBuf,
    application_support: PathBuf,
    canonical_configuration: PathBuf,
}

impl MacOsConfigurationPaths {
    pub fn resolve(environment: &impl Environment) -> Result<Self, PathResolutionError> {
        let user_home = required_absolute_path(environment, HOME)?;
        let configuration_home = optional_absolute_path(environment, XDG_CONFIG_HOME)?
            .unwrap_or_else(|| user_home.join(".config"));
        let application_support = user_home.join("Library/Application Support");
        let canonical_configuration = configuration_home.join("mcp-sync/config.json");

        Ok(Self {
            user_home,
            configuration_home,
            application_support,
            canonical_configuration,
        })
    }

    pub fn user_home(&self) -> &Path {
        &self.user_home
    }

    #[cfg(test)]
    pub fn configuration_home(&self) -> &Path {
        &self.configuration_home
    }

    pub fn application_support(&self) -> &Path {
        &self.application_support
    }

    pub fn canonical_configuration(&self) -> &Path {
        &self.canonical_configuration
    }
}

fn required_absolute_path(
    environment: &impl Environment,
    variable: &'static str,
) -> Result<PathBuf, PathResolutionError> {
    let value = environment
        .value(variable)
        .filter(|value| !value.is_empty())
        .ok_or(PathResolutionError::MissingVariable { variable })?;

    validate_absolute_path(variable, PathBuf::from(value))
}

fn optional_absolute_path(
    environment: &impl Environment,
    variable: &'static str,
) -> Result<Option<PathBuf>, PathResolutionError> {
    environment
        .value(variable)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .map(|path| validate_absolute_path(variable, path))
        .transpose()
}

fn validate_absolute_path(
    variable: &'static str,
    path: PathBuf,
) -> Result<PathBuf, PathResolutionError> {
    if !path.is_absolute() {
        return Err(PathResolutionError::NonAbsoluteVariable { variable });
    }

    if path
        .components()
        .any(|component| component == Component::ParentDir)
    {
        return Err(PathResolutionError::ParentTraversal { variable });
    }

    Ok(path)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PathResolutionError {
    MissingVariable { variable: &'static str },
    NonAbsoluteVariable { variable: &'static str },
    ParentTraversal { variable: &'static str },
}

impl fmt::Display for PathResolutionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingVariable { variable } => {
                write!(formatter, "configuration path requires `{variable}`")
            }
            Self::NonAbsoluteVariable { variable } => {
                write!(
                    formatter,
                    "configuration path variable `{variable}` must be absolute"
                )
            }
            Self::ParentTraversal { variable } => write!(
                formatter,
                "configuration path variable `{variable}` must not contain parent traversal"
            ),
        }
    }
}

impl std::error::Error for PathResolutionError {}

#[cfg(test)]
mod tests {
    use super::{Environment, MacOsConfigurationPaths, PathResolutionError};
    use std::collections::BTreeMap;
    use std::ffi::OsString;
    use std::path::{Path, PathBuf};

    #[derive(Default)]
    struct FixtureEnvironment {
        values: BTreeMap<&'static str, OsString>,
    }

    impl FixtureEnvironment {
        fn with_path(mut self, name: &'static str, path: impl Into<PathBuf>) -> Self {
            self.values.insert(name, path.into().into_os_string());
            self
        }

        fn with_value(mut self, name: &'static str, value: impl Into<OsString>) -> Self {
            self.values.insert(name, value.into());
            self
        }
    }

    impl Environment for FixtureEnvironment {
        fn value(&self, name: &'static str) -> Option<OsString> {
            self.values.get(name).cloned()
        }
    }

    struct MacOsPathFixture {
        root: tempfile::TempDir,
        user_home: PathBuf,
    }

    impl MacOsPathFixture {
        fn new() -> Self {
            let root = tempfile::tempdir().expect("temporary path fixture should be created");
            let user_home = root.path().join("user");
            std::fs::create_dir_all(user_home.join("Library/Application Support"))
                .expect("macOS fixture directories should be created");

            Self { root, user_home }
        }

        fn environment(&self) -> FixtureEnvironment {
            FixtureEnvironment::default().with_path("HOME", &self.user_home)
        }

        fn assert_isolated(&self, path: &Path) {
            assert!(path.is_absolute(), "resolved paths should be absolute");
            assert!(
                path.starts_with(self.root.path()),
                "resolved path should remain inside its disposable fixture root"
            );
        }
    }

    #[test]
    fn macos_defaults_resolve_under_the_injected_home() {
        let fixture = MacOsPathFixture::new();

        let paths = MacOsConfigurationPaths::resolve(&fixture.environment())
            .expect("synthetic macOS paths should resolve");

        assert_eq!(paths.user_home(), fixture.user_home);
        assert_eq!(
            paths.configuration_home(),
            fixture.user_home.join(".config")
        );
        assert_eq!(
            paths.application_support(),
            fixture.user_home.join("Library/Application Support")
        );
        assert_eq!(
            paths.canonical_configuration(),
            fixture.user_home.join(".config/mcp-sync/config.json")
        );

        for path in [
            paths.user_home(),
            paths.configuration_home(),
            paths.application_support(),
            paths.canonical_configuration(),
        ] {
            fixture.assert_isolated(path);
        }
    }

    #[test]
    fn macos_honors_an_injected_xdg_configuration_home() {
        let fixture = MacOsPathFixture::new();
        let xdg_configuration_home = fixture.root.path().join("xdg-config");
        let environment = fixture
            .environment()
            .with_path("XDG_CONFIG_HOME", &xdg_configuration_home);

        let paths = MacOsConfigurationPaths::resolve(&environment)
            .expect("the XDG override should resolve");

        assert_eq!(paths.configuration_home(), xdg_configuration_home);
        assert_eq!(
            paths.canonical_configuration(),
            fixture.root.path().join("xdg-config/mcp-sync/config.json")
        );
        assert_eq!(
            paths.application_support(),
            fixture.user_home.join("Library/Application Support")
        );
        fixture.assert_isolated(paths.canonical_configuration());
        fixture.assert_isolated(paths.application_support());
    }

    #[test]
    fn an_empty_xdg_configuration_home_uses_the_home_default() {
        let fixture = MacOsPathFixture::new();
        let environment = fixture
            .environment()
            .with_value("XDG_CONFIG_HOME", OsString::new());

        let paths = MacOsConfigurationPaths::resolve(&environment)
            .expect("an empty XDG override should use the default");

        assert_eq!(
            paths.canonical_configuration(),
            fixture.user_home.join(".config/mcp-sync/config.json")
        );
        fixture.assert_isolated(paths.canonical_configuration());
    }

    #[test]
    fn missing_home_is_a_contextual_resolution_error() {
        let error = MacOsConfigurationPaths::resolve(&FixtureEnvironment::default())
            .expect_err("HOME should be required");

        assert_eq!(
            error,
            PathResolutionError::MissingVariable { variable: "HOME" }
        );
        assert_eq!(error.to_string(), "configuration path requires `HOME`");
    }

    #[test]
    fn relative_environment_paths_are_rejected() {
        let fixture = MacOsPathFixture::new();
        let cases = [
            (
                FixtureEnvironment::default().with_path("HOME", "relative-home"),
                "HOME",
            ),
            (
                fixture
                    .environment()
                    .with_path("XDG_CONFIG_HOME", "relative-config"),
                "XDG_CONFIG_HOME",
            ),
        ];

        for (environment, variable) in cases {
            let error = MacOsConfigurationPaths::resolve(&environment)
                .expect_err("relative configuration paths should fail");
            assert_eq!(error, PathResolutionError::NonAbsoluteVariable { variable });
        }
    }

    #[test]
    fn parent_traversal_in_environment_paths_is_rejected() {
        let fixture = MacOsPathFixture::new();
        let traversing_home = fixture.root.path().join("user/../outside");
        let environment = FixtureEnvironment::default().with_path("HOME", traversing_home);

        let error = MacOsConfigurationPaths::resolve(&environment)
            .expect_err("parent traversal should fail");

        assert_eq!(
            error,
            PathResolutionError::ParentTraversal { variable: "HOME" }
        );
    }
}
