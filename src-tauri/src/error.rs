#[derive(Debug, thiserror::Error)]
pub enum MailnirError {
    #[error("I/O error reading {path}: {source}")]
    Io {
        path: std::path::PathBuf,
        source: std::io::Error,
    },

    #[error("YAML parse error in {path}: {source}")]
    TemplateParseYaml {
        path: std::path::PathBuf,
        source: serde_yaml::Error,
    },

    #[error("no source has primary: true")]
    NoPrimarySource,

    #[error("multiple sources declare primary: true: {namespaces:?}")]
    MultiplePrimarySource { namespaces: Vec<String> },

    #[error("join in '{namespace}' key '{join_key}' has invalid ref '{ref_value}' (must be namespace.field)")]
    InvalidJoinRef {
        namespace: String,
        join_key: String,
        ref_value: String,
    },

    #[error("join in '{namespace}' references unknown namespace '{ref_namespace}'")]
    UnknownJoinNamespace {
        namespace: String,
        join_key: String,
        ref_namespace: String,
    },

    #[error("source '{namespace}' joins on itself")]
    SelfJoin { namespace: String },

    #[error("unsupported file format: '{extension}'")]
    UnsupportedFormat { extension: String },

    #[error("JSON parse error in {path}: {source}")]
    JsonParse {
        path: std::path::PathBuf,
        source: serde_json::Error,
    },

    #[error("YAML parse error in {path}: {source}")]
    YamlParse {
        path: std::path::PathBuf,
        source: serde_yaml::Error,
    },

    #[error("TOML parse error in {path}: {source}")]
    TomlParse {
        path: std::path::PathBuf,
        source: toml::de::Error,
    },

    #[error("CSV parse error in {path}: {source}")]
    CsvParse {
        path: std::path::PathBuf,
        source: csv::Error,
    },

    #[error("CSV file has no headers: {path}")]
    CsvNoHeaders { path: std::path::PathBuf },

    #[error("invalid data shape in {path}: {message}")]
    InvalidDataShape {
        path: std::path::PathBuf,
        message: String,
    },

    #[error("join '{namespace}' found no match for primary entry {entry_index}")]
    JoinMissingMatch {
        namespace: String,
        entry_index: usize,
    },

    #[error(
        "join '{namespace}' is ambiguous for primary entry {entry_index}: {match_count} matches"
    )]
    JoinAmbiguousMatch {
        namespace: String,
        entry_index: usize,
        match_count: usize,
    },
}
