//! Form data source integration tests.
//!
//! Tests the full pipeline with form-provided data instead of file-loaded data.

use std::collections::HashMap;
use std::path::Path;

use serde_json::{json, Value};

use mailnir_lib::join::build_contexts_lenient;
use mailnir_lib::render::render_context;
use mailnir_lib::template::{infer_form_fields, parse_template, parse_template_str};
use mailnir_lib::validate::validate_all;

fn make_sources(pairs: &[(&str, Value)]) -> HashMap<String, Value> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.clone()))
        .collect()
}

/// Form source produces a single-entry array that flows through the pipeline.
#[test]
fn test_form_source_full_pipeline() {
    let t = parse_template_str(
        "sources:\n  recipient: {primary: true, form: true}\n\
         to: '{{recipient.email}}'\n\
         subject: 'Hello {{recipient.name}}'\n\
         body: 'Hi {{recipient.name}}, welcome!'\n\
         body_format: text",
    )
    .expect("template must parse");

    // Simulate form data: a single-entry array matching what the IPC layer produces.
    let form_data = json!([{
        "email": "alice@example.com",
        "name": "Alice"
    }]);
    let sources = make_sources(&[("recipient", form_data)]);

    // Validate
    let report = validate_all(&t, &sources, Path::new(".")).expect("validation ok");
    assert_eq!(report.entries.len(), 1, "form source produces 1 entry");
    assert!(report.is_valid(), "entry should be valid");

    // Render
    let contexts = build_contexts_lenient(&t, &sources).expect("contexts ok");
    let ctx = contexts.into_iter().next().unwrap().expect("entry 0 ok");
    let rendered = render_context(&t, &ctx, Path::new(".")).expect("render ok");

    assert_eq!(rendered.to, "alice@example.com");
    assert_eq!(rendered.subject, "Hello Alice");
    assert_eq!(rendered.text_body, "Hi Alice, welcome!");
}

/// Field inference extracts the correct fields from the fixture template.
#[test]
fn test_form_field_inference_from_fixture() {
    let t = parse_template(Path::new("fixtures/templates/form_source.mailnir.yml"))
        .expect("fixture must parse");
    let fields = infer_form_fields(&t, "recipient");
    assert_eq!(fields, vec!["email", "name"]);
}

/// form: true is correctly parsed and preserved in SourceConfig.
#[test]
fn test_form_flag_parsing() {
    let t = parse_template_str(
        "sources:\n  rcpt: {primary: true, form: true}\n  data: {}\n\
         to: x\nsubject: s\nbody: b",
    )
    .expect("template must parse");

    assert_eq!(t.sources["rcpt"].form, Some(true));
    assert_eq!(t.sources["data"].form, None);
}

/// Form source works as a joined secondary (single-entry join target).
#[test]
fn test_form_as_joined_source() {
    let t = parse_template_str(
        "sources:\n  p: {primary: true}\n  cfg: {form: true, join: {key: p.key}}\n\
         to: '{{p.email}}'\nsubject: '{{cfg.greeting}}'\nbody: '{{cfg.greeting}} {{p.name}}'",
    )
    .expect("template must parse");

    let sources = make_sources(&[
        (
            "p",
            json!([{"email": "a@b.com", "name": "Alice", "key": "x"}]),
        ),
        ("cfg", json!([{"greeting": "Welcome", "key": "x"}])),
    ]);

    let contexts = build_contexts_lenient(&t, &sources).expect("ok");
    let ctx = contexts.into_iter().next().unwrap().expect("ok");
    let rendered = render_context(&t, &ctx, Path::new(".")).expect("ok");

    assert_eq!(rendered.subject, "Welcome");
}
