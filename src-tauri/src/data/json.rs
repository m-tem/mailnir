use std::path::Path;

use serde_json::Value;

pub fn load_json(path: &Path) -> crate::Result<Value> {
    let content = std::fs::read_to_string(path).map_err(|source| crate::MailnirError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let value: Value =
        serde_json::from_str(&content).map_err(|source| crate::MailnirError::JsonParse {
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
    fn test_load_json_array() {
        let v = load_json(&fixtures_dir().join("simple.json")).unwrap();
        assert!(v.is_array());
        assert_eq!(v.as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_load_json_single_object_wrapped() {
        let v = load_json(&fixtures_dir().join("single_object.json")).unwrap();
        assert!(v.is_array());
        assert_eq!(v.as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_load_json_invalid_syntax() {
        use std::io::Write;
        let mut f = tempfile::NamedTempFile::new().unwrap();
        f.write_all(b"{not valid json}").unwrap();
        assert!(matches!(
            load_json(f.path()),
            Err(crate::MailnirError::JsonParse { .. })
        ));
    }

    #[test]
    fn test_load_json_invalid_shape_string() {
        use std::io::Write;
        let mut f = tempfile::NamedTempFile::new().unwrap();
        f.write_all(b"\"just a string\"").unwrap();
        assert!(matches!(
            load_json(f.path()),
            Err(crate::MailnirError::InvalidDataShape { .. })
        ));
    }

    #[test]
    fn test_load_json_invalid_shape_null() {
        use std::io::Write;
        let mut f = tempfile::NamedTempFile::new().unwrap();
        f.write_all(b"null").unwrap();
        assert!(matches!(
            load_json(f.path()),
            Err(crate::MailnirError::InvalidDataShape { .. })
        ));
    }
}
