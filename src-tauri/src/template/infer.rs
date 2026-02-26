use std::collections::BTreeSet;

use super::types::Template;

/// Extract field names referenced in template strings for a given namespace.
///
/// Scans to, cc, bcc, subject, body, and attachments for patterns like
/// `namespace.field` inside Handlebars expressions. Returns a sorted,
/// deduplicated list of field names.
pub fn infer_form_fields(template: &Template, namespace: &str) -> Vec<String> {
    let strings: Vec<&str> = [
        Some(template.to.as_str()),
        template.cc.as_deref(),
        template.bcc.as_deref(),
        Some(template.subject.as_str()),
        Some(template.body.as_str()),
        template.attachments.as_deref(),
    ]
    .into_iter()
    .flatten()
    .collect();

    let needle = format!("{namespace}.");
    let mut fields = BTreeSet::new();

    for s in &strings {
        let mut start = 0;
        while let Some(pos) = s[start..].find(&needle) {
            let abs = start + pos;
            // Ensure the match is at a word boundary (not preceded by a word char).
            if abs > 0 {
                let prev = s.as_bytes()[abs - 1];
                if prev.is_ascii_alphanumeric() || prev == b'_' {
                    start = abs + needle.len();
                    continue;
                }
            }
            // Extract the field name: word characters after the dot.
            let field_start = abs + needle.len();
            let field_end = s[field_start..]
                .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                .map(|i| field_start + i)
                .unwrap_or(s.len());
            let field = &s[field_start..field_end];
            if !field.is_empty() {
                fields.insert(field.to_string());
            }
            start = field_end;
        }
    }

    fields.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::parse_template_str;

    #[test]
    fn test_simple_variable_references() {
        let t = parse_template_str(
            "sources:\n  rcpt: {primary: true, form: true}\n\
             to: '{{rcpt.email}}'\nsubject: 'Hello {{rcpt.name}}'\nbody: hi",
        )
        .unwrap();
        assert_eq!(infer_form_fields(&t, "rcpt"), vec!["email", "name"]);
    }

    #[test]
    fn test_block_helper_each() {
        let t = parse_template_str(
            "sources:\n  rcpt: {primary: true, form: true}\n\
             to: x\nsubject: s\nbody: '{{#each rcpt.items}}{{this}}{{/each}}'",
        )
        .unwrap();
        assert_eq!(infer_form_fields(&t, "rcpt"), vec!["items"]);
    }

    #[test]
    fn test_deduplication() {
        let t = parse_template_str(
            "sources:\n  rcpt: {primary: true, form: true}\n\
             to: '{{rcpt.email}}'\nsubject: '{{rcpt.email}}'\nbody: '{{rcpt.email}}'",
        )
        .unwrap();
        assert_eq!(infer_form_fields(&t, "rcpt"), vec!["email"]);
    }

    #[test]
    fn test_no_false_match_on_partial_namespace() {
        let t = parse_template_str(
            "sources:\n  recipient: {primary: true, form: true}\n\
             to: '{{recipient.email}}'\nsubject: s\nbody: b",
        )
        .unwrap();
        // "r" should not match "recipient.email"
        assert!(infer_form_fields(&t, "r").is_empty());
        // "recipien" should not match either
        assert!(infer_form_fields(&t, "recipien").is_empty());
    }

    #[test]
    fn test_unknown_namespace_returns_empty() {
        let t = parse_template_str(
            "sources:\n  rcpt: {primary: true, form: true}\n\
             to: '{{rcpt.email}}'\nsubject: s\nbody: b",
        )
        .unwrap();
        assert!(infer_form_fields(&t, "other").is_empty());
    }

    #[test]
    fn test_multiple_fields_sorted() {
        let t = parse_template_str(
            "sources:\n  rcpt: {primary: true, form: true}\n\
             to: '{{rcpt.email}}'\nsubject: '{{rcpt.first_name}} {{rcpt.last_name}}'\n\
             body: 'Dear {{rcpt.first_name}}, your ID is {{rcpt.id}}'",
        )
        .unwrap();
        assert_eq!(
            infer_form_fields(&t, "rcpt"),
            vec!["email", "first_name", "id", "last_name"]
        );
    }

    #[test]
    fn test_fields_in_optional_template_fields() {
        let t = parse_template_str(
            "sources:\n  rcpt: {primary: true, form: true}\n\
             to: '{{rcpt.email}}'\ncc: '{{rcpt.cc_email}}'\n\
             subject: s\nbody: b\nattachments: '{{rcpt.name}}/doc.pdf'",
        )
        .unwrap();
        assert_eq!(
            infer_form_fields(&t, "rcpt"),
            vec!["cc_email", "email", "name"]
        );
    }
}
