use mailnir_lib::{
    data::{load_file, load_file_csv, CsvOptions},
    template::{parse_template, validate_sources},
};

fn fixtures_templates() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("templates")
}

fn fixtures_data() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("data")
}

#[test]
fn test_template_roundtrip_full() {
    let path = fixtures_templates().join("full.mailnir.yml");
    let original = parse_template(&path).unwrap();
    let serialized = serde_yaml::to_string(&original).unwrap();
    let reparsed = mailnir_lib::template::parse_template_str(&serialized).unwrap();
    assert_eq!(original, reparsed);
}

#[test]
fn test_parse_and_validate_composite_join() {
    let path = fixtures_templates().join("composite_join.mailnir.yml");
    let template = parse_template(&path).unwrap();
    validate_sources(&template).unwrap();
}

#[test]
fn test_load_all_formats() {
    let cases: &[(&str, &str, &str)] = &[
        ("simple.json", "name", "Alice"),
        ("simple.yaml", "name", "Alice"),
        ("simple.toml", "name", "Alice"),
        ("comma.csv", "name", "Alice"),
    ];

    for (filename, field, expected) in cases {
        let path = fixtures_data().join(filename);
        let value = load_file(&path).unwrap_or_else(|e| panic!("Failed to load {filename}: {e}"));
        let arr = value.as_array().expect("expected array");
        assert_eq!(arr.len(), 3, "expected 3 elements in {filename}");
        let first_field = arr[0]
            .get(*field)
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("field '{field}' missing in first element of {filename}"));
        assert_eq!(
            first_field, *expected,
            "wrong value for '{field}' in {filename}"
        );
    }
}

#[test]
fn test_csv_auto_detect_semicolon() {
    let path = fixtures_data().join("semicolon.csv");
    let value = load_file_csv(&path, &CsvOptions::default()).unwrap();
    let arr = value.as_array().unwrap();
    assert_eq!(arr.len(), 3);
    // Columns should be individually parsed, not merged into one
    let first = &arr[0];
    assert!(
        first.get("name").is_some(),
        "expected 'name' column to exist as separate field"
    );
    assert!(
        first.get("age").is_some(),
        "expected 'age' column to exist as separate field"
    );
    let name = first["name"].as_str().unwrap();
    assert!(
        !name.contains(';'),
        "name value should not contain semicolons (wrong separator used): {name}"
    );
}

#[test]
fn test_csv_latin1_fallback() {
    let path = fixtures_data().join("latin1.csv");
    let value = load_file_csv(&path, &CsvOptions::default()).unwrap();
    let arr = value.as_array().unwrap();
    assert!(!arr.is_empty());
    // First row has "Renée" — check that é is present
    let name = arr[0]["name"].as_str().unwrap();
    assert!(
        name.contains('é'),
        "Expected 'é' in decoded Latin-1 name, got: {name}"
    );
}
