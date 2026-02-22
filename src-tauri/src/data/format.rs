use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub enum DataFormat {
    Json,
    Yaml,
    Toml,
    Csv,
}

pub fn detect_format(path: &Path) -> crate::Result<DataFormat> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "json" => Ok(DataFormat::Json),
        "yml" | "yaml" => Ok(DataFormat::Yaml),
        "toml" => Ok(DataFormat::Toml),
        "csv" => Ok(DataFormat::Csv),
        other => Err(crate::MailnirError::UnsupportedFormat {
            extension: other.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_detect_json() {
        assert_eq!(
            detect_format(Path::new("data.json")).unwrap(),
            DataFormat::Json
        );
    }

    #[test]
    fn test_detect_yml() {
        assert_eq!(
            detect_format(Path::new("data.yml")).unwrap(),
            DataFormat::Yaml
        );
    }

    #[test]
    fn test_detect_yaml() {
        assert_eq!(
            detect_format(Path::new("data.yaml")).unwrap(),
            DataFormat::Yaml
        );
    }

    #[test]
    fn test_detect_toml() {
        assert_eq!(
            detect_format(Path::new("data.toml")).unwrap(),
            DataFormat::Toml
        );
    }

    #[test]
    fn test_detect_csv() {
        assert_eq!(
            detect_format(Path::new("data.csv")).unwrap(),
            DataFormat::Csv
        );
    }

    #[test]
    fn test_detect_uppercase_extension() {
        assert_eq!(
            detect_format(Path::new("data.JSON")).unwrap(),
            DataFormat::Json
        );
        assert_eq!(
            detect_format(Path::new("data.CSV")).unwrap(),
            DataFormat::Csv
        );
    }

    #[test]
    fn test_detect_no_extension() {
        let result = detect_format(Path::new("datafile"));
        assert!(matches!(
            result,
            Err(crate::MailnirError::UnsupportedFormat { .. })
        ));
    }

    #[test]
    fn test_detect_unknown_extension() {
        let result = detect_format(Path::new("data.xlsx"));
        assert!(
            matches!(result, Err(crate::MailnirError::UnsupportedFormat { extension }) if extension == "xlsx")
        );
    }
}
