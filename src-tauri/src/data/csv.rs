use std::path::Path;

use serde_json::{Map, Value};

#[derive(Debug, Clone, Default)]
pub struct CsvOptions {
    pub separator: Option<u8>,
    pub encoding: Option<String>,
}

pub fn detect_separator(first_line: &str) -> u8 {
    let candidates: &[(u8, char)] = &[(b',', ','), (b';', ';'), (b'|', '|'), (b'\t', '\t')];
    candidates
        .iter()
        .max_by_key(|(_, ch)| first_line.chars().filter(|c| c == ch).count())
        .map(|(byte, _)| *byte)
        .unwrap_or(b',')
}

pub fn decode_bytes(bytes: &[u8], hint: Option<&str>) -> crate::Result<String> {
    if let Some(label) = hint {
        let encoding =
            encoding_rs::Encoding::for_label(label.as_bytes()).unwrap_or(encoding_rs::WINDOWS_1252);
        let (decoded, _, _) = encoding.decode(bytes);
        return Ok(decoded.into_owned());
    }

    match String::from_utf8(bytes.to_vec()) {
        Ok(s) => Ok(s),
        Err(_) => {
            let (decoded, _, _) = encoding_rs::WINDOWS_1252.decode(bytes);
            Ok(decoded.into_owned())
        }
    }
}

pub fn load_csv(path: &Path, opts: &CsvOptions) -> crate::Result<Value> {
    let bytes = std::fs::read(path).map_err(|source| crate::MailnirError::Io {
        path: path.to_path_buf(),
        source,
    })?;

    let content = decode_bytes(&bytes, opts.encoding.as_deref())?;

    let delimiter = if let Some(sep) = opts.separator {
        sep
    } else {
        let first_line = content.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
        detect_separator(first_line)
    };

    let mut reader = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .has_headers(true)
        .from_reader(content.as_bytes());

    let headers: Vec<String> = {
        let h = reader
            .headers()
            .map_err(|source| crate::MailnirError::CsvParse {
                path: path.to_path_buf(),
                source,
            })?;
        if h.is_empty() {
            return Err(crate::MailnirError::CsvNoHeaders {
                path: path.to_path_buf(),
            });
        }
        h.iter().map(String::from).collect()
    };

    let mut rows: Vec<Value> = Vec::new();
    for result in reader.records() {
        let record = result.map_err(|source| crate::MailnirError::CsvParse {
            path: path.to_path_buf(),
            source,
        })?;
        let mut map = Map::new();
        for (key, val) in headers.iter().zip(record.iter()) {
            map.insert(key.clone(), Value::String(val.to_string()));
        }
        rows.push(Value::Object(map));
    }

    Ok(Value::Array(rows))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixtures_dir() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures")
            .join("data")
    }

    fn default_opts() -> CsvOptions {
        CsvOptions::default()
    }

    #[test]
    fn test_load_csv_comma() {
        let v = load_csv(&fixtures_dir().join("comma.csv"), &default_opts()).unwrap();
        let arr = v.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert!(arr[0].get("name").is_some());
    }

    #[test]
    fn test_load_csv_semicolon_autodetect() {
        let v = load_csv(&fixtures_dir().join("semicolon.csv"), &default_opts()).unwrap();
        let arr = v.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        // Should have individual columns, not a single merged column
        let first = &arr[0];
        assert!(first.get("name").is_some());
        assert!(first.get("age").is_some());
    }

    #[test]
    fn test_load_csv_pipe_autodetect() {
        let v = load_csv(&fixtures_dir().join("pipe.csv"), &default_opts()).unwrap();
        let arr = v.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert!(arr[0].get("name").is_some());
    }

    #[test]
    fn test_load_csv_tab_autodetect() {
        let v = load_csv(&fixtures_dir().join("tab.csv"), &default_opts()).unwrap();
        let arr = v.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert!(arr[0].get("name").is_some());
    }

    #[test]
    fn test_load_csv_explicit_separator_override() {
        // Force comma separator on semicolon file → single column with semicolons in value
        let opts = CsvOptions {
            separator: Some(b','),
            encoding: None,
        };
        let v = load_csv(&fixtures_dir().join("semicolon.csv"), &opts).unwrap();
        let arr = v.as_array().unwrap();
        // With wrong separator, entire line becomes one field — column count still 1 per header
        assert!(!arr.is_empty());
    }

    #[test]
    fn test_load_csv_latin1_autodetect() {
        let v = load_csv(&fixtures_dir().join("latin1.csv"), &default_opts()).unwrap();
        let arr = v.as_array().unwrap();
        assert!(!arr.is_empty());
        let first = &arr[0];
        let name = first.get("name").and_then(|v| v.as_str()).unwrap_or("");
        // Latin-1 chars like é and ü should be present
        assert!(
            name.contains('é') || name.contains('ü'),
            "Expected Latin-1 chars in name, got: {name}"
        );
    }

    #[test]
    fn test_load_csv_windows1252_explicit_hint() {
        let opts = CsvOptions {
            separator: None,
            encoding: Some("windows-1252".to_string()),
        };
        let v = load_csv(&fixtures_dir().join("windows1252.csv"), &opts).unwrap();
        let arr = v.as_array().unwrap();
        assert!(!arr.is_empty());
        // € sign (0x80 in Windows-1252) should appear
        let first = &arr[0];
        let currency = first.get("currency").and_then(|v| v.as_str()).unwrap_or("");
        assert!(
            currency.contains('€'),
            "Expected € in currency, got: {currency}"
        );
    }

    #[test]
    fn test_detect_separator_comma() {
        assert_eq!(detect_separator("a,b,c,d"), b',');
    }

    #[test]
    fn test_detect_separator_semicolon() {
        assert_eq!(detect_separator("a;b;c;d"), b';');
    }

    #[test]
    fn test_detect_separator_pipe() {
        assert_eq!(detect_separator("a|b|c|d"), b'|');
    }

    #[test]
    fn test_detect_separator_tab() {
        assert_eq!(detect_separator("a\tb\tc\td"), b'\t');
    }
}
