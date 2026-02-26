use std::path::Path;

use serde_json::Value;

pub fn load_toml(path: &Path) -> crate::Result<Value> {
    let content = std::fs::read_to_string(path).map_err(|source| crate::MailnirError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let value: toml::Value =
        toml::from_str(&content).map_err(|source| crate::MailnirError::TomlParse {
            path: path.to_path_buf(),
            source,
        })?;
    let json_value = toml_to_json(value);
    normalize_shape(path, json_value)
}

fn toml_to_json(value: toml::Value) -> Value {
    match value {
        toml::Value::String(s) => Value::String(s),
        toml::Value::Integer(i) => Value::Number(i.into()),
        toml::Value::Float(f) => serde_json::Number::from_f64(f)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        toml::Value::Boolean(b) => Value::Bool(b),
        toml::Value::Array(arr) => Value::Array(arr.into_iter().map(toml_to_json).collect()),
        toml::Value::Table(table) => Value::Object(
            table
                .into_iter()
                .map(|(k, v)| (k, toml_to_json(v)))
                .collect(),
        ),
        toml::Value::Datetime(dt) => Value::String(dt.to_string()),
    }
}

/// TOML-specific normalization: unwrap single-key tables whose value is an
/// array of objects (the shape produced by `[[entry]]` syntax), then fall
/// through to the shared normalize_shape.
fn normalize_shape(path: &Path, value: Value) -> crate::Result<Value> {
    if let Value::Object(ref map) = value {
        if map.len() == 1 {
            let (_, inner) = map.iter().next().unwrap();
            if let Value::Array(arr) = inner {
                if arr.iter().all(|v| v.is_object()) {
                    return Ok(inner.clone());
                }
            }
        }
    }
    super::normalize_shape(path, value)
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
    fn test_load_toml_array_of_tables() {
        let v = load_toml(&fixtures_dir().join("simple.toml")).unwrap();
        assert!(v.is_array());
        assert_eq!(v.as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_load_toml_single_object_wrapped() {
        use std::io::Write;
        let mut f = tempfile::NamedTempFile::with_suffix(".toml").unwrap();
        f.write_all(b"name = \"Alice\"\nage = 30\n").unwrap();
        let v = load_toml(f.path()).unwrap();
        assert!(v.is_array());
        assert_eq!(v.as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_load_toml_invalid_syntax() {
        use std::io::Write;
        let mut f = tempfile::NamedTempFile::with_suffix(".toml").unwrap();
        f.write_all(b"name = [unclosed").unwrap();
        assert!(matches!(
            load_toml(f.path()),
            Err(crate::MailnirError::TomlParse { .. })
        ));
    }

    #[test]
    fn test_load_toml_invalid_shape_null() {
        // TOML cannot represent null/bare scalars at root; test with a bare integer
        // Actually TOML always parses to a table at root, so test this differently:
        // A TOML root that has no [[array]] and is a plain key-value becomes an Object,
        // which gets wrapped. This test verifies there's no panic on edge cases.
        use std::io::Write;
        let mut f = tempfile::NamedTempFile::with_suffix(".toml").unwrap();
        f.write_all(b"x = 1\n").unwrap();
        let v = load_toml(f.path()).unwrap();
        // single-key object with numeric value → wrapped in array
        assert!(v.is_array());
    }
}
