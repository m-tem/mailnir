use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::join::build_contexts_lenient;
use crate::render::{render_context, RenderedEmail};
use crate::template::Template;
use crate::MailnirError;

/// One problem found for a specific primary source entry.
#[derive(Debug, Clone)]
pub enum ValidationIssue {
    /// A Handlebars template variable could not be resolved (strict mode).
    UnresolvedVariable { field: String, reason: String },
    /// A secondary join found no match or an ambiguous match for this entry.
    JoinFailure {
        namespace: String,
        detail: JoinFailureDetail,
    },
    /// The rendered `to`, `cc`, or `bcc` field is not a valid RFC 5322 address.
    InvalidEmail { field: String, value: String },
    /// An attachment path does not exist on the filesystem.
    AttachmentNotFound { path: PathBuf },
    /// The `to`, `subject`, or `body` field is empty after rendering.
    RequiredFieldEmpty { field: String },
    /// The stylesheet file referenced in the template does not exist.
    StylesheetNotFound { path: PathBuf },
    /// CSS inlining failed (malformed stylesheet or HTML).
    CssInlineError { reason: String },
}

#[derive(Debug, Clone)]
pub enum JoinFailureDetail {
    MissingMatch,
    AmbiguousMatch { match_count: usize },
}

/// Validation result for one primary source entry.
#[derive(Debug, Clone)]
pub struct EntryResult {
    /// Zero-based index into the primary source array.
    pub entry_index: usize,
    /// All issues found for this entry. Empty means valid.
    pub issues: Vec<ValidationIssue>,
}

impl EntryResult {
    pub fn is_valid(&self) -> bool {
        self.issues.is_empty()
    }
}

/// Aggregate validation result for an entire template run.
#[derive(Debug, Clone)]
pub struct ValidationReport {
    /// One entry per primary source row, in source order.
    pub entries: Vec<EntryResult>,
}

impl ValidationReport {
    pub fn is_valid(&self) -> bool {
        self.entries.iter().all(EntryResult::is_valid)
    }

    /// Returns only entries that have at least one issue.
    pub fn invalid_entries(&self) -> impl Iterator<Item = &EntryResult> {
        self.entries.iter().filter(|e| !e.is_valid())
    }
}

/// Run the full validation pipeline over all primary source entries.
///
/// Returns `Err` only on structural failures (e.g. no primary source declared,
/// malformed source shape). Per-entry problems are collected into the report.
pub fn validate_all(
    template: &Template,
    sources: &HashMap<String, Value>,
    template_dir: &Path,
) -> crate::Result<ValidationReport> {
    let per_entry_contexts = build_contexts_lenient(template, sources)?;

    let mut entries = Vec::with_capacity(per_entry_contexts.len());

    for (entry_index, ctx_result) in per_entry_contexts.into_iter().enumerate() {
        let mut issues: Vec<ValidationIssue> = Vec::new();

        match ctx_result {
            Err(join_err) => {
                issues.push(issue_from_join_error(join_err));
            }
            Ok(context) => match render_context(template, &context, template_dir) {
                Err(render_err) => {
                    issues.push(issue_from_render_error(render_err));
                }
                Ok(rendered) => {
                    post_render_checks(&rendered, &mut issues);
                }
            },
        }

        entries.push(EntryResult {
            entry_index,
            issues,
        });
    }

    Ok(ValidationReport { entries })
}

fn issue_from_join_error(err: MailnirError) -> ValidationIssue {
    match err {
        MailnirError::JoinMissingMatch { namespace, .. } => ValidationIssue::JoinFailure {
            namespace,
            detail: JoinFailureDetail::MissingMatch,
        },
        MailnirError::JoinAmbiguousMatch {
            namespace,
            match_count,
            ..
        } => ValidationIssue::JoinFailure {
            namespace,
            detail: JoinFailureDetail::AmbiguousMatch { match_count },
        },
        other => ValidationIssue::UnresolvedVariable {
            field: "<internal>".into(),
            reason: other.to_string(),
        },
    }
}

fn issue_from_render_error(err: MailnirError) -> ValidationIssue {
    match err {
        MailnirError::HandlebarsRender { field, reason } => {
            ValidationIssue::UnresolvedVariable { field, reason }
        }
        MailnirError::StylesheetNotFound { path } => ValidationIssue::StylesheetNotFound { path },
        MailnirError::CssInline { reason } => ValidationIssue::CssInlineError { reason },
        other => ValidationIssue::UnresolvedVariable {
            field: "<internal>".into(),
            reason: other.to_string(),
        },
    }
}

fn post_render_checks(rendered: &RenderedEmail, issues: &mut Vec<ValidationIssue>) {
    // Required fields first — skip email check if field is empty.
    check_required("to", &rendered.to, issues);
    check_required("subject", &rendered.subject, issues);
    check_required("body", &rendered.text_body, issues);

    // Email format (only when field is non-empty to avoid double-reporting).
    if !rendered.to.trim().is_empty() {
        check_email("to", &rendered.to, issues);
    }
    if let Some(cc) = &rendered.cc {
        if !cc.trim().is_empty() {
            check_email("cc", cc, issues);
        }
    }
    if let Some(bcc) = &rendered.bcc {
        if !bcc.trim().is_empty() {
            check_email("bcc", bcc, issues);
        }
    }

    // Attachment existence.
    for path in &rendered.attachments {
        if !path.exists() {
            issues.push(ValidationIssue::AttachmentNotFound { path: path.clone() });
        }
    }
}

fn check_required(field: &str, value: &str, issues: &mut Vec<ValidationIssue>) {
    if value.trim().is_empty() {
        issues.push(ValidationIssue::RequiredFieldEmpty {
            field: field.to_string(),
        });
    }
}

fn check_email(field: &str, value: &str, issues: &mut Vec<ValidationIssue>) {
    if value.parse::<lettre::message::Mailboxes>().is_err() {
        issues.push(ValidationIssue::InvalidEmail {
            field: field.to_string(),
            value: value.to_string(),
        });
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::io::Write as _;

    use serde_json::json;

    use super::*;
    use crate::template::parse_template_str;

    fn make_sources(pairs: &[(&str, serde_json::Value)]) -> HashMap<String, serde_json::Value> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect()
    }

    fn simple_template(to: &str, subject: &str, body: &str) -> Template {
        parse_template_str(&format!(
            "sources:\n  p: {{primary: true}}\nto: '{}'\nsubject: '{}'\nbody: '{}'\nbody_format: text",
            to, subject, body
        ))
        .expect("fixture must parse")
    }

    // --- Exit criterion: unresolved variable names field and entry ---

    #[test]
    fn test_unresolved_variable_names_field_and_entry() {
        // "classses" is a deliberate typo
        let t = parse_template_str(
            "sources:\n  p: {primary: true}\nto: 'a@b.com'\nsubject: '{{classses.name}}'\nbody: hi\nbody_format: text",
        )
        .unwrap();
        let sources = make_sources(&[("p", json!([{"name": "Alice"}, {"name": "Bob"}]))]);

        let report = validate_all(&t, &sources, Path::new(".")).unwrap();
        assert!(!report.is_valid());

        let e0 = &report.entries[0];
        assert_eq!(e0.entry_index, 0);
        assert!(!e0.is_valid());
        assert!(
            e0.issues.iter().any(|i| matches!(
                i,
                ValidationIssue::UnresolvedVariable { field, .. } if field == "subject"
            )),
            "expected UnresolvedVariable on subject, got: {:?}",
            e0.issues
        );
    }

    // --- Exit criterion: valid email passes, invalid email fails ---

    #[test]
    fn test_email_validation_passes_for_valid() {
        let t = simple_template("alice@example.com", "hi", "body");
        let sources = make_sources(&[("p", json!([{"dummy": 1}]))]);

        let report = validate_all(&t, &sources, Path::new(".")).unwrap();
        assert!(report.is_valid(), "valid email should produce no issues");
    }

    #[test]
    fn test_email_validation_fails_for_invalid() {
        let t = parse_template_str(
            "sources:\n  p: {primary: true}\nto: '{{p.email}}'\nsubject: s\nbody: b\nbody_format: text",
        )
        .unwrap();
        let sources = make_sources(&[(
            "p",
            json!([
                {"email": "alice@example.com"},
                {"email": "not-an-email"},
            ]),
        )]);

        let report = validate_all(&t, &sources, Path::new(".")).unwrap();
        assert!(report.entries[0].is_valid(), "entry 0 should be valid");
        assert!(!report.entries[1].is_valid(), "entry 1 should be invalid");
        assert!(
            report.entries[1].issues.iter().any(|i| matches!(
                i,
                ValidationIssue::InvalidEmail { field, value }
                if field == "to" && value == "not-an-email"
            )),
            "expected InvalidEmail on to, got: {:?}",
            report.entries[1].issues
        );
    }

    // --- Exit criterion: attachment to nonexistent file ---

    #[test]
    fn test_attachment_not_found() {
        let t = parse_template_str(
            "sources:\n  p: {primary: true}\nto: 'a@b.com'\nsubject: s\nbody: b\nbody_format: text\nattachments: 'nonexistent_file_xyz_12345.pdf'",
        )
        .unwrap();
        let sources = make_sources(&[("p", json!([{"dummy": 1}]))]);

        let report = validate_all(&t, &sources, Path::new(".")).unwrap();
        assert!(!report.is_valid());

        let e0 = &report.entries[0];
        assert_eq!(e0.entry_index, 0);
        assert!(
            e0.issues.iter().any(|i| matches!(
                i,
                ValidationIssue::AttachmentNotFound { path }
                if path.to_string_lossy().contains("nonexistent_file_xyz_12345.pdf")
            )),
            "expected AttachmentNotFound, got: {:?}",
            e0.issues
        );
    }

    // --- Exit criterion: empty `to` → required field error ---

    #[test]
    fn test_empty_to_required_field() {
        let t = parse_template_str(
            "sources:\n  p: {primary: true}\nto: '{{p.email}}'\nsubject: s\nbody: b\nbody_format: text",
        )
        .unwrap();
        let sources = make_sources(&[("p", json!([{"email": ""}]))]);

        let report = validate_all(&t, &sources, Path::new(".")).unwrap();
        assert!(!report.is_valid());
        assert!(
            report.entries[0].issues.iter().any(|i| matches!(
                i,
                ValidationIssue::RequiredFieldEmpty { field } if field == "to"
            )),
            "expected RequiredFieldEmpty on to, got: {:?}",
            report.entries[0].issues
        );
        // Should NOT also emit InvalidEmail for the same empty field
        assert!(
            !report.entries[0].issues.iter().any(|i| matches!(
                i,
                ValidationIssue::InvalidEmail { field, .. } if field == "to"
            )),
            "empty to should not also produce InvalidEmail"
        );
    }

    // --- Exit criterion: stylesheet pointing to nonexistent file ---

    #[test]
    fn test_stylesheet_not_found() {
        let t = parse_template_str(
            "sources:\n  p: {primary: true}\nto: 'a@b.com'\nsubject: s\nbody: '# hello'\nstylesheet: 'nonexistent_style_xyz.css'",
        )
        .unwrap();
        let sources = make_sources(&[("p", json!([{"dummy": 1}]))]);

        let report = validate_all(&t, &sources, Path::new(".")).unwrap();
        assert!(!report.is_valid());
        assert!(
            report.entries[0]
                .issues
                .iter()
                .any(|i| matches!(i, ValidationIssue::StylesheetNotFound { .. })),
            "expected StylesheetNotFound, got: {:?}",
            report.entries[0].issues
        );
    }

    // --- Join failure captured as per-entry issue ---

    #[test]
    fn test_join_failure_captured_per_entry() {
        let t = parse_template_str(
            "sources:\n  p: {primary: true}\n  s:\n    join:\n      pid: p.id\nto: 'a@b.com'\nsubject: s\nbody: b\nbody_format: text",
        )
        .unwrap();
        let sources = make_sources(&[
            ("p", json!([{"id": 1}, {"id": 99}])),
            ("s", json!([{"pid": 1, "val": "ok"}])),
        ]);

        let report = validate_all(&t, &sources, Path::new(".")).unwrap();
        // Entry 0 (id=1) matches s
        assert!(report.entries[0].is_valid(), "entry 0 should be valid");
        // Entry 1 (id=99) has no match in s
        assert!(!report.entries[1].is_valid(), "entry 1 should be invalid");
        assert!(
            report.entries[1].issues.iter().any(|i| matches!(
                i,
                ValidationIssue::JoinFailure {
                    namespace,
                    detail: JoinFailureDetail::MissingMatch
                }
                if namespace == "s"
            )),
            "expected JoinFailure(MissingMatch) on s, got: {:?}",
            report.entries[1].issues
        );
    }

    // --- All valid report ---

    #[test]
    fn test_all_valid_report() {
        let t = parse_template_str(
            "sources:\n  p: {primary: true}\nto: '{{p.email}}'\nsubject: 'Hello {{p.name}}'\nbody: hi\nbody_format: text",
        )
        .unwrap();
        let sources = make_sources(&[(
            "p",
            json!([
                {"email": "a@example.com", "name": "Alice"},
                {"email": "b@example.com", "name": "Bob"},
            ]),
        )]);

        let report = validate_all(&t, &sources, Path::new(".")).unwrap();
        assert!(report.is_valid());
        assert_eq!(report.invalid_entries().count(), 0);
        assert_eq!(report.entries.len(), 2);
    }

    // --- Display-name email format passes ---

    #[test]
    fn test_display_name_email_passes() {
        let t = simple_template("Alice Smith <alice@example.com>", "hi", "body");
        let sources = make_sources(&[("p", json!([{"dummy": 1}]))]);

        let report = validate_all(&t, &sources, Path::new(".")).unwrap();
        assert!(
            report.is_valid(),
            "display-name address should be valid, got: {:?}",
            report.entries[0].issues
        );
    }

    // --- Attachment existence: existing file passes ---

    #[test]
    fn test_existing_attachment_passes() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        writeln!(tmp, "content").unwrap();
        let filename = tmp
            .path()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let dir = tmp.path().parent().unwrap().to_path_buf();

        let t = parse_template_str(&format!(
            "sources:\n  p: {{primary: true}}\nto: 'a@b.com'\nsubject: s\nbody: b\nbody_format: text\nattachments: '{filename}'",
        ))
        .unwrap();
        let sources = make_sources(&[("p", json!([{"dummy": 1}]))]);

        let report = validate_all(&t, &sources, &dir).unwrap();
        assert!(
            report.is_valid(),
            "existing attachment should pass, got: {:?}",
            report.entries[0].issues
        );
    }
}
