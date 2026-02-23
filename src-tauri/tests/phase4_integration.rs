//! Phase 4 integration: full pipeline with 10-entry primary source, 2 deliberate failures.
//!
//! Entry 3: `to` renders to `"not-an-email"` → `InvalidEmail`
//! Entry 7: `to` renders to `""` → `RequiredFieldEmpty`
//! All others: valid

use std::collections::HashMap;
use std::path::Path;

use mailnir_lib::template::parse_template_str;
use mailnir_lib::validate::{validate_all, ValidationIssue};

fn make_sources(pairs: &[(&str, serde_json::Value)]) -> HashMap<String, serde_json::Value> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.clone()))
        .collect()
}

#[test]
fn test_10_entries_2_failures() {
    let primary_data: Vec<serde_json::Value> = (0..10)
        .map(|i| match i {
            3 => serde_json::json!({"id": i, "email": "not-an-email", "name": format!("Person {i}")}),
            7 => serde_json::json!({"id": i, "email": "",              "name": format!("Person {i}")}),
            _ => serde_json::json!({"id": i, "email": format!("person{i}@example.com"), "name": format!("Person {i}")}),
        })
        .collect();

    let t = parse_template_str(
        "sources:\n  p: {primary: true}\nto: '{{p.email}}'\nsubject: 'Hello {{p.name}}'\nbody: 'Hi {{p.name}}'\nbody_format: text",
    )
    .expect("template must parse");

    let sources = make_sources(&[("p", serde_json::Value::Array(primary_data))]);
    let report = validate_all(&t, &sources, Path::new(".")).expect("structural error");

    assert!(!report.is_valid(), "report must be invalid");

    let invalid: Vec<_> = report.invalid_entries().collect();
    assert_eq!(
        invalid.len(),
        2,
        "expected exactly 2 invalid entries, got {}: {:?}",
        invalid.len(),
        invalid.iter().map(|e| e.entry_index).collect::<Vec<_>>()
    );

    let invalid_indices: Vec<usize> = invalid.iter().map(|e| e.entry_index).collect();
    assert!(invalid_indices.contains(&3), "entry 3 must be invalid");
    assert!(invalid_indices.contains(&7), "entry 7 must be invalid");

    // Entry 3: invalid email
    let entry3 = invalid.iter().find(|e| e.entry_index == 3).unwrap();
    assert!(
        entry3.issues.iter().any(|i| matches!(
            i,
            ValidationIssue::InvalidEmail { field, value }
            if field == "to" && value == "not-an-email"
        )),
        "entry 3 must have InvalidEmail on to, got: {:?}",
        entry3.issues
    );

    // Entry 7: empty to
    let entry7 = invalid.iter().find(|e| e.entry_index == 7).unwrap();
    assert!(
        entry7.issues.iter().any(|i| matches!(
            i,
            ValidationIssue::RequiredFieldEmpty { field } if field == "to"
        )),
        "entry 7 must have RequiredFieldEmpty on to, got: {:?}",
        entry7.issues
    );

    // All other entries are valid
    for entry in &report.entries {
        if entry.entry_index != 3 && entry.entry_index != 7 {
            assert!(
                entry.is_valid(),
                "entry {} should be valid but has issues: {:?}",
                entry.entry_index,
                entry.issues
            );
        }
    }
}
