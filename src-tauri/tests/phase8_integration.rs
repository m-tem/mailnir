//! Phase 8 integration: preview pipeline tests.
//!
//! Tests the full pipeline (parse → join → validate/render) through the library
//! APIs that the preview IPC commands rely on.

use std::collections::HashMap;
use std::path::Path;

use serde_json::json;

use mailnir_lib::join::build_contexts_lenient;
use mailnir_lib::render::render_context;
use mailnir_lib::template::parse_template_str;
use mailnir_lib::validate::validate_all;

fn make_sources(pairs: &[(&str, serde_json::Value)]) -> HashMap<String, serde_json::Value> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.clone()))
        .collect()
}

/// Exit criterion: 5-entry primary source → navigator shows "1 of 5".
#[test]
fn test_5_entry_validation_count() {
    let entries: Vec<serde_json::Value> = (0..5)
        .map(|i| {
            json!({
                "email": format!("user{i}@example.com"),
                "name": format!("User {i}")
            })
        })
        .collect();

    let t = parse_template_str(
        "sources:\n  p: {primary: true}\nto: '{{p.email}}'\nsubject: 'Hi {{p.name}}'\nbody: '# Welcome'\nbody_format: markdown",
    )
    .expect("template must parse");

    let sources = make_sources(&[("p", serde_json::Value::Array(entries))]);
    let report = validate_all(&t, &sources, Path::new(".")).expect("structural error");

    assert_eq!(report.entries.len(), 5, "expected 5 entries in report");
    assert!(report.is_valid(), "all entries should be valid");
}

/// Exit criterion: HTML preview renders markdown body as formatted HTML.
#[test]
fn test_markdown_renders_html() {
    let t = parse_template_str(
        "sources:\n  p: {primary: true}\nto: '{{p.email}}'\nsubject: 'Hi'\nbody: |\n  # Hello {{p.name}}\n\n  **Welcome** to the course.\n\n  - item 1\n  - item 2",
    )
    .expect("template must parse");

    let sources = make_sources(&[(
        "p",
        json!([{"email": "alice@example.com", "name": "Alice"}]),
    )]);

    let contexts =
        build_contexts_lenient(&t, &sources).expect("build_contexts_lenient must succeed");
    assert_eq!(contexts.len(), 1);

    let ctx = contexts
        .into_iter()
        .next()
        .unwrap()
        .expect("context must be Ok");
    let rendered = render_context(&t, &ctx, Path::new(".")).expect("render must succeed");

    let html = rendered
        .html_body
        .expect("markdown should produce html_body");
    assert!(html.contains("<h1>"), "expected <h1> in HTML: {html}");
    assert!(html.contains("Alice"), "expected name in HTML: {html}");
    assert!(
        html.contains("<strong>"),
        "expected <strong> in HTML: {html}"
    );
    assert!(html.contains("<li>"), "expected <li> in HTML: {html}");
}

/// Exit criterion: entry with validation error flagged with warning.
#[test]
fn test_join_failure_flagged_render_valid_works() {
    let t = parse_template_str(
        "sources:\n  p: {primary: true}\n  s:\n    join:\n      pid: p.id\nto: '{{p.email}}'\nsubject: 'Info'\nbody: 'Hello'\nbody_format: text",
    )
    .expect("template must parse");

    let sources = make_sources(&[
        (
            "p",
            json!([
                {"id": 1, "email": "a@example.com"},
                {"id": 2, "email": "b@example.com"},
                {"id": 99, "email": "c@example.com"},
            ]),
        ),
        ("s", json!([{"pid": 1, "val": "x"}, {"pid": 2, "val": "y"}])),
    ]);

    let report = validate_all(&t, &sources, Path::new(".")).expect("structural error");

    assert_eq!(report.entries.len(), 3);
    assert!(report.entries[0].is_valid(), "entry 0 should be valid");
    assert!(report.entries[1].is_valid(), "entry 1 should be valid");
    assert!(
        !report.entries[2].is_valid(),
        "entry 2 should be invalid (no join match for id=99)"
    );

    // Valid entries can still be rendered
    let contexts = build_contexts_lenient(&t, &sources).expect("structural ok");
    let ctx0 = contexts
        .into_iter()
        .next()
        .unwrap()
        .expect("entry 0 context ok");
    let rendered = render_context(&t, &ctx0, Path::new(".")).expect("render entry 0 ok");
    assert_eq!(rendered.to, "a@example.com");
    assert_eq!(rendered.text_body, "Hello");
}

/// Exit criterion: attachments listed in metadata match resolved paths.
#[test]
fn test_attachments_in_rendered_email() {
    let t = parse_template_str(
        "sources:\n  p: {primary: true}\nto: 'a@b.com'\nsubject: 's'\nbody: 'b'\nbody_format: text\nattachments: '{{p.file}}'",
    )
    .expect("template must parse");

    let sources = make_sources(&[(
        "p",
        json!([{"file": "report.pdf"}, {"file": "grades.xlsx"}]),
    )]);

    let contexts = build_contexts_lenient(&t, &sources).expect("ok");
    let ctx0 = contexts.into_iter().next().unwrap().expect("ok");
    let rendered = render_context(&t, &ctx0, Path::new("/data")).expect("ok");

    assert_eq!(rendered.attachments.len(), 1);
    assert_eq!(
        rendered.attachments[0].display().to_string(),
        "/data/report.pdf"
    );
}

/// Regression: css_inline::inline_fragment used to drop sibling elements.
/// Verify the full template renders all content through the complete pipeline.
#[test]
fn test_full_template_renders_all_content() {
    let template_path = std::path::Path::new("fixtures/templates/full.mailnir.yml");
    let template =
        mailnir_lib::template::parse_template(template_path).expect("template must parse");

    let classes_data =
        mailnir_lib::data::load_file(std::path::Path::new("fixtures/data/classes.json"))
            .expect("classes.json");
    let inst_data =
        mailnir_lib::data::load_file(std::path::Path::new("fixtures/data/instructors_clean.json"))
            .expect("instructors_clean.json");

    let sources = make_sources(&[("classes", classes_data), ("inst", inst_data)]);

    let contexts = build_contexts_lenient(&template, &sources).expect("build_contexts_lenient");
    let ctx = contexts
        .into_iter()
        .next()
        .unwrap()
        .expect("entry 0 context");
    let rendered =
        render_context(&template, &ctx, template_path.parent().unwrap()).expect("render");

    let html = rendered
        .html_body
        .expect("markdown template should produce html_body");

    assert!(html.contains("Hello"), "missing greeting in: {html}");
    assert!(html.contains("Alice"), "missing name in: {html}");
    assert!(
        html.contains("credentials are ready"),
        "missing second paragraph in: {html}"
    );
    assert!(
        html.contains("alice01"),
        "missing student username in: {html}"
    );
    assert!(html.contains("<li>"), "missing list items in: {html}");
}
