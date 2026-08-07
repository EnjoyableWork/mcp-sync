use serde::Serialize;
use serde::de::{self, Deserialize, Deserializer, Error as _, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Number, Value};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

pub const CANONICAL_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalConfig {
    schema_version: u32,
    servers: BTreeMap<String, CanonicalServer>,
}

impl CanonicalConfig {
    pub fn new(servers: BTreeMap<String, CanonicalServer>) -> Result<Self, ConfigError> {
        let config = Self {
            schema_version: CANONICAL_SCHEMA_VERSION,
            servers,
        };
        config.validate()?;
        Ok(config)
    }

    pub fn parse_json(document: &str) -> Result<Self, ConfigError> {
        let value = parse_unique_json_value(document.as_bytes()).map_err(|error| {
            ConfigError::InvalidJson {
                message: error.to_string(),
            }
        })?;

        Self::from_json_value(value)
    }

    pub fn to_canonical_json(&self) -> Result<String, ConfigError> {
        if self.schema_version != CANONICAL_SCHEMA_VERSION {
            return Err(ConfigError::UnsupportedSchemaVersion {
                found: self.schema_version.to_string(),
            });
        }
        self.validate()?;

        let mut document =
            serde_json::to_string_pretty(self).map_err(|error| ConfigError::Serialization {
                message: error.to_string(),
            })?;
        document.push('\n');
        Ok(document)
    }

    pub fn schema_version(&self) -> u32 {
        self.schema_version
    }

    pub fn servers(&self) -> &BTreeMap<String, CanonicalServer> {
        &self.servers
    }

    fn from_json_value(value: Value) -> Result<Self, ConfigError> {
        let mut root = match value {
            Value::Object(root) => root,
            _ => return Err(DocumentError::RootMustBeObject.into()),
        };

        let version = root
            .remove("schemaVersion")
            .ok_or(DocumentError::MissingSchemaVersion)?;
        validate_schema_version(&version)?;

        let servers = root
            .remove("servers")
            .ok_or(DocumentError::MissingServers)?;

        if let Some((field, _)) = root.into_iter().next() {
            return Err(DocumentError::UnknownRootField { field }.into());
        }

        let server_entries = match servers {
            Value::Object(entries) => entries,
            _ => return Err(DocumentError::ServersMustBeObject.into()),
        };

        let mut normalized_servers = BTreeMap::new();
        for (position, (name, value)) in server_entries.into_iter().enumerate() {
            validate_server_name(&name, position)?;
            let server = decode_server(&name, value)?;
            normalized_servers.insert(name, server);
        }

        Self::new(normalized_servers)
    }

    fn validate(&self) -> Result<(), ValidationError> {
        for (position, (name, server)) in self.servers.iter().enumerate() {
            validate_server_name(name, position)?;
            validate_server(name, server)?;
        }
        Ok(())
    }
}

impl fmt::Debug for CanonicalConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CanonicalConfig")
            .field("schema_version", &self.schema_version)
            .field("servers", &self.servers)
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq, Serialize)]
pub struct CanonicalServer {
    command: String,
    args: Vec<String>,
    env: BTreeMap<String, String>,
}

impl fmt::Debug for CanonicalServer {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CanonicalServer")
            .field("command", &"<redacted>")
            .field("argument_count", &self.args.len())
            .field("environment_keys", &self.env.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl CanonicalServer {
    pub fn new(
        command: impl Into<String>,
        args: Vec<String>,
        env: BTreeMap<String, String>,
    ) -> Self {
        Self {
            command: command.into(),
            args,
            env,
        }
    }

    pub fn command(&self) -> &str {
        &self.command
    }

    pub fn args(&self) -> &[String] {
        &self.args
    }

    pub fn env(&self) -> &BTreeMap<String, String> {
        &self.env
    }
}

fn validate_schema_version(value: &Value) -> Result<(), ConfigError> {
    let Value::Number(version) = value else {
        return Err(DocumentError::SchemaVersionMustBeInteger.into());
    };

    let is_integer = version.is_i64() || version.is_u64();
    if !is_integer {
        return Err(DocumentError::SchemaVersionMustBeInteger.into());
    }

    if version.as_u64() == Some(u64::from(CANONICAL_SCHEMA_VERSION)) {
        return Ok(());
    }

    Err(ConfigError::UnsupportedSchemaVersion {
        found: version.to_string(),
    })
}

fn decode_server(name: &str, value: Value) -> Result<CanonicalServer, ConfigError> {
    let mut fields = match value {
        Value::Object(fields) => fields,
        _ => {
            return Err(DocumentError::ServerMustBeObject {
                server: name.to_owned(),
            }
            .into());
        }
    };

    let command = fields
        .remove("command")
        .ok_or_else(|| DocumentError::MissingCommand {
            server: name.to_owned(),
        })?;
    let args = fields.remove("args");
    let env = fields.remove("env");

    if let Some((field, _)) = fields.into_iter().next() {
        return Err(DocumentError::UnknownServerField {
            server: name.to_owned(),
            field,
        }
        .into());
    }

    let command = match command {
        Value::String(command) => command,
        _ => {
            return Err(DocumentError::CommandMustBeString {
                server: name.to_owned(),
            }
            .into());
        }
    };

    let args = decode_arguments(name, args)?;
    let env = decode_environment(name, env)?;
    let server = CanonicalServer::new(command, args, env);
    validate_server(name, &server)?;
    Ok(server)
}

fn decode_arguments(name: &str, value: Option<Value>) -> Result<Vec<String>, ConfigError> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let Value::Array(values) = value else {
        return Err(DocumentError::ArgumentsMustBeArray {
            server: name.to_owned(),
        }
        .into());
    };

    values
        .into_iter()
        .enumerate()
        .map(|(index, value)| match value {
            Value::String(argument) => Ok(argument),
            _ => Err(DocumentError::ArgumentMustBeString {
                server: name.to_owned(),
                index,
            }
            .into()),
        })
        .collect()
}

fn decode_environment(
    name: &str,
    value: Option<Value>,
) -> Result<BTreeMap<String, String>, ConfigError> {
    let Some(value) = value else {
        return Ok(BTreeMap::new());
    };
    let Value::Object(values) = value else {
        return Err(DocumentError::EnvironmentMustBeObject {
            server: name.to_owned(),
        }
        .into());
    };

    values
        .into_iter()
        .enumerate()
        .map(|(position, (key, value))| match value {
            Value::String(value) => Ok((key, value)),
            _ => Err(DocumentError::EnvironmentValueMustBeString {
                server: name.to_owned(),
                position,
            }
            .into()),
        })
        .collect()
}

fn validate_server_name(name: &str, position: usize) -> Result<(), ValidationError> {
    let violation = if name.is_empty() {
        Some(TextViolation::Empty)
    } else if name.trim() != name {
        Some(TextViolation::SurroundingWhitespace)
    } else if name.chars().any(char::is_control) {
        Some(TextViolation::ControlCharacter)
    } else {
        None
    };

    match violation {
        Some(violation) => Err(ValidationError::InvalidServerName {
            position,
            violation,
        }),
        None => Ok(()),
    }
}

fn validate_server(name: &str, server: &CanonicalServer) -> Result<(), ValidationError> {
    let command_violation = if server.command.is_empty() {
        Some(TextViolation::Empty)
    } else if server.command.trim() != server.command {
        Some(TextViolation::SurroundingWhitespace)
    } else if server.command.chars().any(char::is_control) {
        Some(TextViolation::ControlCharacter)
    } else {
        None
    };

    if let Some(violation) = command_violation {
        return Err(ValidationError::InvalidCommand {
            server: name.to_owned(),
            violation,
        });
    }

    if let Some(index) = server
        .args
        .iter()
        .position(|argument| argument.contains('\0'))
    {
        return Err(ValidationError::ArgumentContainsNul {
            server: name.to_owned(),
            index,
        });
    }

    for (position, (key, value)) in server.env.iter().enumerate() {
        if key.contains('\0') {
            return Err(ValidationError::EnvironmentKeyContainsNul {
                server: name.to_owned(),
                position,
            });
        }
        if value.contains('\0') {
            return Err(ValidationError::EnvironmentValueContainsNul {
                server: name.to_owned(),
                position,
            });
        }
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    InvalidJson { message: String },
    UnsupportedSchemaVersion { found: String },
    InvalidDocument(DocumentError),
    InvalidModel(ValidationError),
    Serialization { message: String },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidJson { message } => write!(formatter, "invalid JSON: {message}"),
            Self::UnsupportedSchemaVersion { found } => write!(
                formatter,
                "unsupported canonical configuration schema version {found}; supported version is {CANONICAL_SCHEMA_VERSION}"
            ),
            Self::InvalidDocument(error) => error.fmt(formatter),
            Self::InvalidModel(error) => error.fmt(formatter),
            Self::Serialization { message } => {
                write!(
                    formatter,
                    "cannot serialize canonical configuration: {message}"
                )
            }
        }
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidDocument(error) => Some(error),
            Self::InvalidModel(error) => Some(error),
            Self::InvalidJson { .. }
            | Self::UnsupportedSchemaVersion { .. }
            | Self::Serialization { .. } => None,
        }
    }
}

impl From<DocumentError> for ConfigError {
    fn from(error: DocumentError) -> Self {
        Self::InvalidDocument(error)
    }
}

impl From<ValidationError> for ConfigError {
    fn from(error: ValidationError) -> Self {
        Self::InvalidModel(error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentError {
    RootMustBeObject,
    MissingSchemaVersion,
    SchemaVersionMustBeInteger,
    MissingServers,
    ServersMustBeObject,
    UnknownRootField { field: String },
    ServerMustBeObject { server: String },
    MissingCommand { server: String },
    CommandMustBeString { server: String },
    ArgumentsMustBeArray { server: String },
    ArgumentMustBeString { server: String, index: usize },
    EnvironmentMustBeObject { server: String },
    EnvironmentValueMustBeString { server: String, position: usize },
    UnknownServerField { server: String, field: String },
}

impl fmt::Display for DocumentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RootMustBeObject => {
                formatter.write_str("canonical configuration must be an object")
            }
            Self::MissingSchemaVersion => formatter
                .write_str("canonical configuration is missing required field `schemaVersion`"),
            Self::SchemaVersionMustBeInteger => {
                formatter.write_str("field `schemaVersion` must be an integer")
            }
            Self::MissingServers => {
                formatter.write_str("canonical configuration is missing required field `servers`")
            }
            Self::ServersMustBeObject => formatter.write_str("field `servers` must be an object"),
            Self::UnknownRootField { field } => {
                write!(formatter, "unknown canonical configuration field {field:?}")
            }
            Self::ServerMustBeObject { server } => {
                write!(formatter, "server {server:?} must be an object")
            }
            Self::MissingCommand { server } => {
                write!(
                    formatter,
                    "server {server:?} is missing required field `command`"
                )
            }
            Self::CommandMustBeString { server } => {
                write!(
                    formatter,
                    "field `command` for server {server:?} must be a string"
                )
            }
            Self::ArgumentsMustBeArray { server } => {
                write!(
                    formatter,
                    "field `args` for server {server:?} must be an array"
                )
            }
            Self::ArgumentMustBeString { server, index } => write!(
                formatter,
                "argument {index} for server {server:?} must be a string"
            ),
            Self::EnvironmentMustBeObject { server } => {
                write!(
                    formatter,
                    "field `env` for server {server:?} must be an object"
                )
            }
            Self::EnvironmentValueMustBeString { server, position } => write!(
                formatter,
                "environment value {position} for server {server:?} must be a string"
            ),
            Self::UnknownServerField { server, field } => {
                write!(formatter, "unknown field {field:?} for server {server:?}")
            }
        }
    }
}

impl Error for DocumentError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    InvalidServerName {
        position: usize,
        violation: TextViolation,
    },
    InvalidCommand {
        server: String,
        violation: TextViolation,
    },
    ArgumentContainsNul {
        server: String,
        index: usize,
    },
    EnvironmentKeyContainsNul {
        server: String,
        position: usize,
    },
    EnvironmentValueContainsNul {
        server: String,
        position: usize,
    },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidServerName {
                position,
                violation,
            } => write!(
                formatter,
                "server name at position {position} {}",
                violation.requirement()
            ),
            Self::InvalidCommand { server, violation } => write!(
                formatter,
                "command for server {server:?} {}",
                violation.requirement()
            ),
            Self::ArgumentContainsNul { server, index } => write!(
                formatter,
                "argument {index} for server {server:?} must not contain NUL"
            ),
            Self::EnvironmentKeyContainsNul { server, position } => write!(
                formatter,
                "environment key {position} for server {server:?} must not contain NUL"
            ),
            Self::EnvironmentValueContainsNul { server, position } => write!(
                formatter,
                "environment value {position} for server {server:?} must not contain NUL"
            ),
        }
    }
}

impl Error for ValidationError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextViolation {
    Empty,
    SurroundingWhitespace,
    ControlCharacter,
}

impl TextViolation {
    fn requirement(self) -> &'static str {
        match self {
            Self::Empty => "must not be empty",
            Self::SurroundingWhitespace => "must not have surrounding whitespace",
            Self::ControlCharacter => "must not contain control characters",
        }
    }
}

struct UniqueJsonValue(Value);

/// Rejects duplicate keys before using `Value`'s arbitrary-precision decoder.
///
/// The validation pass deliberately discards its value: `serde_json::Value`
/// owns the feature-aware number representation used by the returned tree.
pub(crate) fn parse_unique_json_value(document: &[u8]) -> Result<Value, serde_json::Error> {
    let _: UniqueJsonValue = serde_json::from_slice(document)?;
    serde_json::from_slice(document)
}

impl<'de> Deserialize<'de> for UniqueJsonValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(UniqueJsonValueVisitor)
    }
}

struct UniqueJsonValueVisitor;

impl<'de> Visitor<'de> for UniqueJsonValueVisitor {
    type Value = UniqueJsonValue;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON value without duplicate object keys")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
        Ok(UniqueJsonValue(Value::Bool(value)))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
        Ok(UniqueJsonValue(Value::Number(value.into())))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
        Ok(UniqueJsonValue(Value::Number(value.into())))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Number::from_f64(value)
            .map(Value::Number)
            .map(UniqueJsonValue)
            .ok_or_else(|| E::custom("non-finite JSON number"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_string(value.to_owned())
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
        Ok(UniqueJsonValue(Value::String(value)))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(UniqueJsonValue(Value::Null))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(UniqueJsonValue(Value::Null))
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::with_capacity(sequence.size_hint().unwrap_or(0));
        while let Some(UniqueJsonValue(value)) = sequence.next_element()? {
            values.push(value);
        }
        Ok(UniqueJsonValue(Value::Array(values)))
    }

    fn visit_map<A>(self, mut entries: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut object = Map::new();
        while let Some(key) = entries.next_key::<String>()? {
            if object.contains_key(&key) {
                return Err(A::Error::custom(format!(
                    "duplicate JSON object key {key:?}"
                )));
            }
            let UniqueJsonValue(value) = entries.next_value()?;
            object.insert(key, value);
        }
        Ok(UniqueJsonValue(Value::Object(object)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOCUMENTED_EXAMPLE: &str = include_str!("../examples/config.v1.json");

    #[test]
    fn documented_example_is_valid_and_canonical() {
        let config = CanonicalConfig::parse_json(DOCUMENTED_EXAMPLE)
            .expect("the documented v1 example should parse");

        assert_eq!(config.schema_version(), CANONICAL_SCHEMA_VERSION);
        assert_eq!(config.to_canonical_json().unwrap(), DOCUMENTED_EXAMPLE);

        let server = &config.servers()["project-files"];
        assert_eq!(server.command(), "example-mcp-server");
        assert_eq!(server.args(), ["--transport", "stdio"]);
        assert_eq!(server.env()["ACCESS_MODE"], "read-only");
    }

    #[test]
    fn empty_server_map_is_valid() {
        let document = "{\"schemaVersion\":1,\"servers\":{}}";
        let config = CanonicalConfig::parse_json(document).unwrap();

        assert!(config.servers().is_empty());
        assert_eq!(
            config.to_canonical_json().unwrap(),
            "{\n  \"schemaVersion\": 1,\n  \"servers\": {}\n}\n"
        );
    }

    #[test]
    fn omitted_optional_fields_normalize_to_explicit_empty_collections() {
        let document = r#"{"schemaVersion":1,"servers":{"minimal":{"command":"server"}}}"#;
        let config = CanonicalConfig::parse_json(document).unwrap();

        assert_eq!(
            config.to_canonical_json().unwrap(),
            concat!(
                "{\n",
                "  \"schemaVersion\": 1,\n",
                "  \"servers\": {\n",
                "    \"minimal\": {\n",
                "      \"command\": \"server\",\n",
                "      \"args\": [],\n",
                "      \"env\": {}\n",
                "    }\n",
                "  }\n",
                "}\n"
            )
        );
    }

    #[test]
    fn serialization_has_stable_field_and_key_order() {
        let document = r#"{
            "servers": {
                "zeta": {"env":{"ZED":"z","ALPHA":"a"},"args":[],"command":"z"},
                "alpha": {"command":"a","env":{},"args":["second","first"]}
            },
            "schemaVersion": 1
        }"#;
        let config = CanonicalConfig::parse_json(document).unwrap();
        let canonical = config.to_canonical_json().unwrap();

        assert!(canonical.find("schemaVersion").unwrap() < canonical.find("servers").unwrap());
        assert!(canonical.find("\"alpha\"").unwrap() < canonical.find("\"zeta\"").unwrap());
        assert!(canonical.find("\"command\"").unwrap() < canonical.find("\"args\"").unwrap());
        assert!(canonical.find("\"args\"").unwrap() < canonical.find("\"env\"").unwrap());
        assert!(canonical.find("\"ALPHA\"").unwrap() < canonical.find("\"ZED\"").unwrap());
        assert!(canonical.ends_with('\n'));
    }

    #[test]
    fn round_trips_preserve_literal_values_and_are_deterministic() {
        let document = r#"{
            "schemaVersion": 1,
            "servers": {
                "literal-values": {
                    "command": "runner with spaces",
                    "args": ["", "--token=${TOKEN}", "two words"],
                    "env": {"EMPTY":"", "MULTILINE":"first\nsecond", "REFERENCE":"${TOKEN}"}
                }
            }
        }"#;
        let first = CanonicalConfig::parse_json(document).unwrap();
        let rendered_once = first.to_canonical_json().unwrap();
        let second = CanonicalConfig::parse_json(&rendered_once).unwrap();
        let rendered_twice = second.to_canonical_json().unwrap();

        assert_eq!(first, second);
        assert_eq!(rendered_once, rendered_twice);

        let server = &second.servers()["literal-values"];
        assert_eq!(server.command(), "runner with spaces");
        assert_eq!(server.args(), ["", "--token=${TOKEN}", "two words"]);
        assert_eq!(server.env()["EMPTY"], "");
        assert_eq!(server.env()["MULTILINE"], "first\nsecond");
        assert_eq!(server.env()["REFERENCE"], "${TOKEN}");
    }

    #[test]
    fn unsupported_integer_versions_have_a_distinct_error() {
        for version in ["-1", "0", "2", "18446744073709551615"] {
            let document = format!(r#"{{"schemaVersion":{version},"servers":{{}}}}"#);
            let error = CanonicalConfig::parse_json(&document).unwrap_err();

            assert_eq!(
                error,
                ConfigError::UnsupportedSchemaVersion {
                    found: version.to_owned()
                }
            );
            assert!(error.to_string().contains("supported version is 1"));
        }
    }

    #[test]
    fn missing_or_non_integer_versions_are_invalid_documents() {
        let cases = [
            (r#"{"servers":{}}"#, DocumentError::MissingSchemaVersion),
            (
                r#"{"schemaVersion":"1","servers":{}}"#,
                DocumentError::SchemaVersionMustBeInteger,
            ),
            (
                r#"{"schemaVersion":1.0,"servers":{}}"#,
                DocumentError::SchemaVersionMustBeInteger,
            ),
            (
                r#"{"schemaVersion":null,"servers":{}}"#,
                DocumentError::SchemaVersionMustBeInteger,
            ),
        ];

        for (document, expected) in cases {
            assert_eq!(
                CanonicalConfig::parse_json(document).unwrap_err(),
                ConfigError::InvalidDocument(expected)
            );
        }
    }

    #[test]
    fn unknown_fields_are_rejected() {
        let root = r#"{"schemaVersion":1,"servers":{},"profile":"dev"}"#;
        assert!(matches!(
            CanonicalConfig::parse_json(root),
            Err(ConfigError::InvalidDocument(
                DocumentError::UnknownRootField { field }
            )) if field == "profile"
        ));

        let server = r#"{
            "schemaVersion":1,
            "servers":{"one":{"command":"server","transport":"stdio"}}
        }"#;
        assert!(matches!(
            CanonicalConfig::parse_json(server),
            Err(ConfigError::InvalidDocument(
                DocumentError::UnknownServerField { field, .. }
            )) if field == "transport"
        ));
    }

    #[test]
    fn duplicate_keys_are_rejected_at_every_object_level() {
        let cases = [
            r#"{"schemaVersion":1,"schemaVersion":1,"servers":{}}"#,
            r#"{"schemaVersion":1,"servers":{"one":{"command":"a"},"one":{"command":"b"}}}"#,
            r#"{"schemaVersion":1,"servers":{"one":{"command":"a","command":"b"}}}"#,
            r#"{"schemaVersion":1,"servers":{"one":{"command":"a","env":{"KEY":"a","KEY":"b"}}}}"#,
        ];

        for document in cases {
            let error = CanonicalConfig::parse_json(document).unwrap_err();
            assert!(matches!(error, ConfigError::InvalidJson { .. }));
            assert!(error.to_string().contains("duplicate JSON object key"));
        }
    }

    #[test]
    fn malformed_json_is_rejected_without_echoing_values() {
        let cases = [
            "{",
            r#"{"schemaVersion":1,"servers":{},}"#,
            r#"{"schemaVersion":1,"servers":{"one":{"command":"secret-value\q"}}}"#,
        ];

        for document in cases {
            let error = CanonicalConfig::parse_json(document).unwrap_err();
            assert!(matches!(error, ConfigError::InvalidJson { .. }));
            assert!(!error.to_string().contains("secret-value"));
            assert!(!format!("{error:?}").contains("secret-value"));
        }
    }

    #[test]
    fn required_fields_and_json_types_are_enforced() {
        let cases = [
            ("[]", DocumentError::RootMustBeObject),
            (r#"{"schemaVersion":1}"#, DocumentError::MissingServers),
            (
                r#"{"schemaVersion":1,"servers":[]}"#,
                DocumentError::ServersMustBeObject,
            ),
            (
                r#"{"schemaVersion":1,"servers":{"one":[]}}"#,
                DocumentError::ServerMustBeObject {
                    server: "one".to_owned(),
                },
            ),
            (
                r#"{"schemaVersion":1,"servers":{"one":{}}}"#,
                DocumentError::MissingCommand {
                    server: "one".to_owned(),
                },
            ),
            (
                r#"{"schemaVersion":1,"servers":{"one":{"command":1}}}"#,
                DocumentError::CommandMustBeString {
                    server: "one".to_owned(),
                },
            ),
            (
                r#"{"schemaVersion":1,"servers":{"one":{"command":"a","args":"one"}}}"#,
                DocumentError::ArgumentsMustBeArray {
                    server: "one".to_owned(),
                },
            ),
            (
                r#"{"schemaVersion":1,"servers":{"one":{"command":"a","args":[1]}}}"#,
                DocumentError::ArgumentMustBeString {
                    server: "one".to_owned(),
                    index: 0,
                },
            ),
            (
                r#"{"schemaVersion":1,"servers":{"one":{"command":"a","env":[]}}}"#,
                DocumentError::EnvironmentMustBeObject {
                    server: "one".to_owned(),
                },
            ),
            (
                r#"{"schemaVersion":1,"servers":{"one":{"command":"a","env":{"KEY":1}}}}"#,
                DocumentError::EnvironmentValueMustBeString {
                    server: "one".to_owned(),
                    position: 0,
                },
            ),
        ];

        for (document, expected) in cases {
            assert_eq!(
                CanonicalConfig::parse_json(document).unwrap_err(),
                ConfigError::InvalidDocument(expected)
            );
        }
    }

    #[test]
    fn server_names_reject_empty_padded_and_control_text() {
        for name in ["", " padded", "padded ", "line\\nfeed"] {
            let document = format!(
                "{{\"schemaVersion\":1,\"servers\":{{\"{name}\":{{\"command\":\"server\"}}}}}}"
            );
            let error = CanonicalConfig::parse_json(&document).unwrap_err();
            assert!(matches!(
                error,
                ConfigError::InvalidModel(ValidationError::InvalidServerName { .. })
            ));
        }
    }

    #[test]
    fn commands_reject_empty_padded_and_control_text() {
        for command in ["", " padded", "padded ", "line\\nfeed"] {
            let document = format!(
                "{{\"schemaVersion\":1,\"servers\":{{\"one\":{{\"command\":\"{command}\"}}}}}}"
            );
            let error = CanonicalConfig::parse_json(&document).unwrap_err();
            assert!(matches!(
                error,
                ConfigError::InvalidModel(ValidationError::InvalidCommand { .. })
            ));
        }
    }

    #[test]
    fn process_strings_reject_nul_without_exposing_values() {
        let cases = [
            r#"{"schemaVersion":1,"servers":{"one":{"command":"server","args":["secret-value\u0000"]}}}"#,
            r#"{"schemaVersion":1,"servers":{"one":{"command":"server","env":{"BAD\u0000KEY":"secret-value"}}}}"#,
            r#"{"schemaVersion":1,"servers":{"one":{"command":"server","env":{"TOKEN":"secret-value\u0000"}}}}"#,
        ];

        for document in cases {
            let error = CanonicalConfig::parse_json(document).unwrap_err();
            assert!(matches!(error, ConfigError::InvalidModel(_)));
            assert!(!error.to_string().contains("secret-value"));
            assert!(!format!("{error:?}").contains("secret-value"));
        }
    }

    #[test]
    fn invalid_environment_types_do_not_leak_their_values() {
        let document = r#"{
            "schemaVersion":1,
            "servers":{"one":{"command":"server","env":{"TOKEN":8675309}}}
        }"#;
        let error = CanonicalConfig::parse_json(document).unwrap_err();

        assert!(matches!(
            error,
            ConfigError::InvalidDocument(DocumentError::EnvironmentValueMustBeString { .. })
        ));
        assert!(!error.to_string().contains("8675309"));
        assert!(!format!("{error:?}").contains("8675309"));
    }

    #[test]
    fn debug_output_is_structurally_redacted() {
        let config = CanonicalConfig::parse_json(
            r#"{
                "schemaVersion":1,
                "servers":{
                    "one":{
                        "command":"secret-command-value",
                        "args":["secret-argument-value"],
                        "env":{"TOKEN":"secret-environment-value"}
                    }
                }
            }"#,
        )
        .unwrap();
        let debug = format!("{config:?}");

        assert!(debug.contains("CanonicalConfig"));
        assert!(debug.contains("TOKEN"));
        for secret in [
            "secret-command-value",
            "secret-argument-value",
            "secret-environment-value",
        ] {
            assert!(!debug.contains(secret));
        }
    }

    #[test]
    fn constructor_enforces_the_same_semantic_contract() {
        let servers = BTreeMap::from([(
            "one".to_owned(),
            CanonicalServer::new("server ", Vec::new(), BTreeMap::new()),
        )]);

        assert!(matches!(
            CanonicalConfig::new(servers),
            Err(ConfigError::InvalidModel(
                ValidationError::InvalidCommand { .. }
            ))
        ));
    }
}
