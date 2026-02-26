pub mod csv;
pub mod format;
pub mod json;
pub mod loader;
pub mod toml;
pub mod yaml;

pub use csv::CsvOptions;
pub use format::{detect_format, DataFormat};
pub use loader::{load_file, load_file_csv};

use serde_json::Value;
use std::path::Path;

/// Normalize a parsed data value into an array of objects.
///
/// - Arrays pass through unchanged.
/// - A single object is wrapped in a one-element array.
/// - Anything else is rejected.
pub(crate) fn normalize_shape(path: &Path, value: Value) -> crate::Result<Value> {
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

pub(crate) fn value_type_name(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}
