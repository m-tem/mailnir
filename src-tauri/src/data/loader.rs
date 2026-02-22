use std::path::Path;

use serde_json::Value;

use crate::data::{
    csv::{load_csv, CsvOptions},
    format::{detect_format, DataFormat},
    json::load_json,
    toml::load_toml,
    yaml::load_yaml,
};

pub fn load_file(path: &Path) -> crate::Result<Value> {
    match detect_format(path)? {
        DataFormat::Json => load_json(path),
        DataFormat::Yaml => load_yaml(path),
        DataFormat::Toml => load_toml(path),
        DataFormat::Csv => load_csv(path, &CsvOptions::default()),
    }
}

pub fn load_file_csv(path: &Path, opts: &CsvOptions) -> crate::Result<Value> {
    load_csv(path, opts)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixtures_dir() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures")
            .join("data")
    }

    #[test]
    fn test_load_file_dispatches_json() {
        let v = load_file(&fixtures_dir().join("simple.json")).unwrap();
        assert!(v.is_array());
    }

    #[test]
    fn test_load_file_dispatches_yaml() {
        let v = load_file(&fixtures_dir().join("simple.yaml")).unwrap();
        assert!(v.is_array());
    }

    #[test]
    fn test_load_file_dispatches_toml() {
        let v = load_file(&fixtures_dir().join("simple.toml")).unwrap();
        assert!(v.is_array());
    }

    #[test]
    fn test_load_file_dispatches_csv() {
        let v = load_file(&fixtures_dir().join("comma.csv")).unwrap();
        assert!(v.is_array());
    }

    #[test]
    fn test_load_file_unknown_format() {
        let result = load_file(std::path::Path::new("/tmp/data.xlsx"));
        assert!(matches!(
            result,
            Err(crate::MailnirError::UnsupportedFormat { .. })
        ));
    }

    #[test]
    fn test_load_file_csv_with_opts() {
        let opts = CsvOptions {
            separator: Some(b';'),
            encoding: None,
        };
        let v = load_file_csv(&fixtures_dir().join("semicolon.csv"), &opts).unwrap();
        assert!(v.is_array());
        let arr = v.as_array().unwrap();
        assert_eq!(arr.len(), 3);
    }
}
