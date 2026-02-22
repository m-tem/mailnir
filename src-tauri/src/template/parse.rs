use std::path::Path;

use crate::template::types::Template;

pub fn parse_template(path: &Path) -> crate::Result<Template> {
    let content = std::fs::read_to_string(path).map_err(|source| crate::MailnirError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    serde_yaml::from_str(&content).map_err(|source| crate::MailnirError::TemplateParseYaml {
        path: path.to_path_buf(),
        source,
    })
}

pub fn parse_template_str(content: &str) -> crate::Result<Template> {
    serde_yaml::from_str(content).map_err(|source| crate::MailnirError::TemplateParseYaml {
        path: std::path::PathBuf::from("<string>"),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::types::BodyFormat;

    fn fixtures_dir() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures")
            .join("templates")
    }

    #[test]
    fn test_parse_minimal() {
        let t = parse_template(&fixtures_dir().join("minimal.mailnir.yml")).unwrap();
        assert_eq!(t.to, "{{primary.email}}");
        assert_eq!(t.subject, "Hello {{primary.name}}");
        assert!(t.cc.is_none());
        assert!(t.bcc.is_none());
        assert!(t.attachments.is_none());
        assert!(t.body_format.is_none());
        assert!(t.stylesheet.is_none());
        assert!(t.style.is_none());
        assert_eq!(t.sources.len(), 1);
        let src = t.sources.get("primary").unwrap();
        assert_eq!(src.primary, Some(true));
        assert!(src.join.is_none());
    }

    #[test]
    fn test_parse_full() {
        let t = parse_template(&fixtures_dir().join("full.mailnir.yml")).unwrap();
        assert_eq!(t.cc, Some("{{classes.coordinator}}".to_string()));
        assert_eq!(t.bcc, Some("admin@example.com".to_string()));
        assert!(t.attachments.is_some());
        assert!(t.style.is_some());
        assert_eq!(t.body_format, Some(BodyFormat::Markdown));
        assert_eq!(t.sources.len(), 2);
    }

    #[test]
    fn test_parse_body_format_html() {
        let t = parse_template(&fixtures_dir().join("html_body.mailnir.yml")).unwrap();
        assert_eq!(t.body_format, Some(BodyFormat::Html));
    }

    #[test]
    fn test_parse_body_format_text() {
        let t = parse_template(&fixtures_dir().join("text_body.mailnir.yml")).unwrap();
        assert_eq!(t.body_format, Some(BodyFormat::Text));
    }

    #[test]
    fn test_parse_body_format_default() {
        let t = parse_template_str("sources:\n  p: {primary: true}\nto: a\nsubject: b\nbody: c")
            .unwrap();
        assert!(t.body_format.is_none());
    }

    #[test]
    fn test_parse_yaml_anchors() {
        let t = parse_template(&fixtures_dir().join("anchors.mailnir.yml")).unwrap();
        // anchors fixture uses YAML anchors in body; template must parse without error
        assert!(!t.body.is_empty());
        assert_eq!(t.sources.len(), 1);
    }

    #[test]
    fn test_parse_composite_join() {
        let t = parse_template(&fixtures_dir().join("composite_join.mailnir.yml")).unwrap();
        let inst = t.sources.get("inst").unwrap();
        let join = inst.join.as_ref().unwrap();
        assert!(join.len() >= 2);
    }

    #[test]
    fn test_parse_invalid_yaml() {
        let result = parse_template_str("sources: [not: a: valid: yaml: structure");
        assert!(matches!(
            result,
            Err(crate::MailnirError::TemplateParseYaml { .. })
        ));
    }

    #[test]
    fn test_parse_missing_required_field() {
        // omit `to`
        let result = parse_template_str("sources:\n  p: {primary: true}\nsubject: b\nbody: c");
        assert!(result.is_err());
    }
}
