use std::path::Path;

use serde_json::Value;

pub fn load_yaml(path: &Path) -> crate::Result<Value> {
    let content = std::fs::read_to_string(path).map_err(|source| crate::MailnirError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let value: Value =
        serde_yaml::from_str(&content).map_err(|source| crate::MailnirError::YamlParse {
            path: path.to_path_buf(),
            source,
        })?;
    normalize_shape(path, value)
}

fn normalize_shape(path: &Path, value: Value) -> crate::Result<Value> {
    match value {
        Value::Array(_) => Ok(value),
        Value::Object(_) => Ok(Value::Array(vec![value])),
        other => Err(crate::MailnirError::InvalidDataShape {
            path: path.to_path_buf(),
            message: format!(
                "expected array or object at root, got {}",
                value_type_name(&other)
            ),
        }),
    }
}

fn value_type_name(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
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
    fn test_load_yaml_sequence() {
        let v = load_yaml(&fixtures_dir().join("simple.yaml")).unwrap();
        assert!(v.is_array());
        assert_eq!(v.as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_load_yaml_single_object_wrapped() {
        use std::io::Write;
        let mut f = tempfile::NamedTempFile::with_suffix(".yaml").unwrap();
        f.write_all(b"name: Alice\nage: 30\n").unwrap();
        let v = load_yaml(f.path()).unwrap();
        assert!(v.is_array());
        assert_eq!(v.as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_load_yaml_invalid_syntax() {
        use std::io::Write;
        let mut f = tempfile::NamedTempFile::with_suffix(".yaml").unwrap();
        f.write_all(b"key: [unclosed bracket").unwrap();
        assert!(matches!(
            load_yaml(f.path()),
            Err(crate::MailnirError::YamlParse { .. })
        ));
    }

    #[test]
    fn test_load_yaml_invalid_shape_string() {
        use std::io::Write;
        let mut f = tempfile::NamedTempFile::with_suffix(".yaml").unwrap();
        f.write_all(b"just a bare string\n").unwrap();
        assert!(matches!(
            load_yaml(f.path()),
            Err(crate::MailnirError::InvalidDataShape { .. })
        ));
    }

    #[test]
    fn test_load_yaml_invalid_shape_null() {
        use std::io::Write;
        let mut f = tempfile::NamedTempFile::with_suffix(".yaml").unwrap();
        f.write_all(b"null\n").unwrap();
        assert!(matches!(
            load_yaml(f.path()),
            Err(crate::MailnirError::InvalidDataShape { .. })
        ));
    }
}
