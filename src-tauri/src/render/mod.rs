use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use crate::template::{BodyFormat, Template};
use crate::MailnirError;

/// The fully rendered output for one primary source row.
#[derive(Debug, Clone)]
pub struct RenderedEmail {
    pub to: String,
    pub cc: Option<String>,
    pub bcc: Option<String>,
    pub subject: String,
    /// `None` when `body_format` is `Text`.
    pub html_body: Option<String>,
    /// Always present.
    pub text_body: String,
    /// Resolved attachment file paths.
    pub attachments: Vec<PathBuf>,
}

/// Render one merged context against the template, producing a [`RenderedEmail`].
///
/// `context` is one entry from `build_contexts()` output.
/// `template_dir` is used to resolve relative `stylesheet` paths.
pub fn render_context(
    template: &Template,
    context: &Map<String, Value>,
    template_dir: &Path,
) -> crate::Result<RenderedEmail> {
    let hbs = make_handlebars();

    let to = render_field(&hbs, "to", &template.to, context)?;
    let subject = render_field(&hbs, "subject", &template.subject, context)?;
    let cc = template
        .cc
        .as_deref()
        .map(|s| render_field(&hbs, "cc", s, context))
        .transpose()?;
    let bcc = template
        .bcc
        .as_deref()
        .map(|s| render_field(&hbs, "bcc", s, context))
        .transpose()?;

    let rendered_body = render_field(&hbs, "body", &template.body, context)?;
    let css = resolve_css(template, template_dir)?;

    let (html_body, text_body) = match effective_body_format(template) {
        BodyFormat::Markdown => {
            let html = markdown_to_html(&rendered_body);
            let html = apply_css(&html, css.as_deref())?;
            let text = strip_html(&html);
            (Some(html), text)
        }
        BodyFormat::Html => {
            let html = apply_css(&rendered_body, css.as_deref())?;
            let text = strip_html(&html);
            (Some(html), text)
        }
        BodyFormat::Text => (None, rendered_body),
    };

    let attachments = template
        .attachments
        .as_deref()
        .map(|tmpl| {
            render_field(&hbs, "attachments", tmpl, context)
                .map(|s| split_attachments(&s, template_dir))
        })
        .transpose()?
        .unwrap_or_default();

    Ok(RenderedEmail {
        to,
        cc,
        bcc,
        subject,
        html_body,
        text_body,
        attachments,
    })
}

fn make_handlebars() -> handlebars::Handlebars<'static> {
    let mut hbs = handlebars::Handlebars::new();
    hbs.set_strict_mode(true);
    hbs.register_escape_fn(handlebars::no_escape);
    hbs
}

fn render_field(
    hbs: &handlebars::Handlebars<'_>,
    field_name: &str,
    template_str: &str,
    context: &Map<String, Value>,
) -> crate::Result<String> {
    hbs.render_template(template_str, context)
        .map_err(|e| MailnirError::HandlebarsRender {
            field: field_name.to_string(),
            reason: e.to_string(),
        })
}

fn effective_body_format(template: &Template) -> &BodyFormat {
    template
        .body_format
        .as_ref()
        .unwrap_or(&BodyFormat::Markdown)
}

fn markdown_to_html(markdown: &str) -> String {
    let mut options = comrak::Options::default();
    options.extension.strikethrough = true;
    options.extension.table = true;
    options.extension.autolink = true;
    options.render.r#unsafe = true;
    comrak::markdown_to_html(markdown, &options)
}

fn resolve_css(template: &Template, template_dir: &Path) -> crate::Result<Option<String>> {
    if let Some(inline_css) = &template.style {
        return Ok(Some(inline_css.clone()));
    }
    if let Some(stylesheet_path) = &template.stylesheet {
        let full_path = template_dir.join(stylesheet_path);
        let css = std::fs::read_to_string(&full_path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                MailnirError::StylesheetNotFound {
                    path: full_path.clone(),
                }
            } else {
                MailnirError::Io {
                    path: full_path.clone(),
                    source: e,
                }
            }
        })?;
        return Ok(Some(css));
    }
    Ok(None)
}

fn apply_css(html: &str, css: Option<&str>) -> crate::Result<String> {
    let Some(css_str) = css else {
        return Ok(html.to_string());
    };
    let inliner = css_inline::CSSInliner::options()
        .load_remote_stylesheets(false)
        .build();
    // Wrap in a <div> because inline_fragment only processes the first
    // top-level element when there are multiple siblings.
    let wrapped = format!("<div>{html}</div>");
    let inlined = inliner
        .inline_fragment(&wrapped, css_str)
        .map_err(|e| MailnirError::CssInline {
            reason: e.to_string(),
        })?;
    // Strip the wrapper <div>…</div> (the opening tag may have gained
    // inline styles if a CSS rule targets `div`).
    let inner = if inlined.starts_with("<div") {
        let start = inlined.find('>').map(|i| i + 1).unwrap_or(0);
        let end = inlined.rfind("</div>").unwrap_or(inlined.len());
        &inlined[start..end]
    } else {
        &inlined
    };
    Ok(inner.to_string())
}

fn strip_html(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    result
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", " ")
        .replace("&#39;", "'")
        .replace("&quot;", "\"")
}

fn split_attachments(rendered: &str, template_dir: &Path) -> Vec<PathBuf> {
    rendered
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| template_dir.join(s))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::io::Write as _;

    use serde_json::json;

    use super::*;
    use crate::template::{parse_template_str, BodyFormat, SourceConfig};

    fn make_template(yaml: &str) -> Template {
        parse_template_str(yaml).expect("fixture must parse")
    }

    fn make_context(pairs: &[(&str, Value)]) -> Map<String, Value> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect()
    }

    fn minimal_template() -> Template {
        let mut sources = HashMap::new();
        sources.insert(
            "p".to_string(),
            SourceConfig {
                primary: Some(true),
                join: None,
                many: None,
            },
        );
        Template {
            sources,
            to: "x@example.com".to_string(),
            cc: None,
            bcc: None,
            subject: "s".to_string(),
            body: String::new(),
            attachments: None,
            body_format: None,
            stylesheet: None,
            style: None,
        }
    }

    #[test]
    fn test_render_each_block() {
        let t = make_template(
            "sources:\n  p: {primary: true}\nto: 'a@b.com'\nsubject: s\nbody_format: text\nbody: |\n  {{#each items}}- {{this.name}}\n  {{/each}}",
        );
        let ctx = make_context(&[(
            "items",
            json!([{"name": "Alice"}, {"name": "Bob"}, {"name": "Carol"}]),
        )]);
        let email = render_context(&t, &ctx, Path::new(".")).unwrap();
        assert!(email.html_body.is_none());
        assert!(email.text_body.contains("Alice"));
        assert!(email.text_body.contains("Bob"));
        assert!(email.text_body.contains("Carol"));
    }

    #[test]
    fn test_markdown_produces_html_tags() {
        let t = make_template(
            "sources:\n  p: {primary: true}\nto: 'a@b.com'\nsubject: s\nbody: |\n  # Title\n\n  **bold** and _italic_\n\n  - item1\n  - item2",
        );
        let ctx = make_context(&[]);
        let email = render_context(&t, &ctx, Path::new(".")).unwrap();
        let html = email.html_body.unwrap();
        assert!(html.contains("<h1>"), "expected <h1> in: {html}");
        assert!(html.contains("<strong>"), "expected <strong> in: {html}");
        assert!(html.contains("<ul>"), "expected <ul> in: {html}");
        assert!(html.contains("<li>"), "expected <li> in: {html}");
    }

    #[test]
    fn test_css_inlining_from_style() {
        let t = Template {
            body: "<h1>Hi</h1>".to_string(),
            body_format: Some(BodyFormat::Html),
            style: Some("h1 { color: red; }".to_string()),
            ..minimal_template()
        };
        let ctx = make_context(&[]);
        let email = render_context(&t, &ctx, Path::new(".")).unwrap();
        let html = email.html_body.unwrap();
        assert!(
            html.contains("color: red") || html.contains("color:red"),
            "expected inlined color in: {html}"
        );
        assert!(html.contains("<h1"), "expected <h1 in: {html}");
    }

    #[test]
    fn test_css_inlining_from_stylesheet_file() {
        let mut css_file = tempfile::NamedTempFile::new().unwrap();
        write!(css_file, "h1 {{ color: blue; }}").unwrap();
        let css_dir = css_file.path().parent().unwrap();
        let css_filename = css_file.path().file_name().unwrap().to_str().unwrap();
        let t = Template {
            body: "<h1>Test</h1>".to_string(),
            body_format: Some(BodyFormat::Html),
            stylesheet: Some(css_filename.to_string()),
            ..minimal_template()
        };
        let ctx = make_context(&[]);
        let email = render_context(&t, &ctx, css_dir).unwrap();
        let html = email.html_body.unwrap();
        assert!(
            html.contains("color: blue") || html.contains("color:blue"),
            "expected inlined color in: {html}"
        );
    }

    #[test]
    fn test_plaintext_fallback() {
        let t = make_template(
            "sources:\n  p: {primary: true}\nto: 'a@b.com'\nsubject: s\nbody: |\n  # Hello\n\n  **world**",
        );
        let ctx = make_context(&[]);
        let email = render_context(&t, &ctx, Path::new(".")).unwrap();
        assert!(
            !email.text_body.contains("<h1>"),
            "text_body must not contain HTML tags"
        );
        assert!(
            !email.text_body.contains("<strong>"),
            "text_body must not contain HTML tags"
        );
        assert!(
            email.text_body.contains("Hello"),
            "text_body must preserve content"
        );
        assert!(
            email.text_body.contains("world"),
            "text_body must preserve content"
        );
    }

    #[test]
    fn test_html_format_skips_markdown() {
        let t = make_template(
            "sources:\n  p: {primary: true}\nto: 'a@b.com'\nsubject: s\nbody: '# Not a heading'\nbody_format: html",
        );
        let ctx = make_context(&[]);
        let email = render_context(&t, &ctx, Path::new(".")).unwrap();
        let html = email.html_body.unwrap();
        assert!(
            !html.contains("<h1>"),
            "markdown must not run in html mode; got: {html}"
        );
        assert!(
            html.contains("# Not a heading"),
            "raw body must appear unchanged"
        );
    }

    #[test]
    fn test_text_format_no_html() {
        let t = Template {
            body: "Hello {{name}}".to_string(),
            body_format: Some(BodyFormat::Text),
            style: Some("h1 { color: red; }".to_string()),
            ..minimal_template()
        };
        let ctx = make_context(&[("name", json!("World"))]);
        let email = render_context(&t, &ctx, Path::new(".")).unwrap();
        assert!(
            email.html_body.is_none(),
            "html_body must be None in text mode"
        );
        assert!(email.text_body.contains("Hello World"));
    }

    #[test]
    fn test_attachments_split() {
        let t = make_template(
            "sources:\n  p: {primary: true}\nto: 'a@b.com'\nsubject: s\nbody: ''\nbody_format: text\nattachments: |\n  {{#each files}}{{this.path}}\n  {{/each}}",
        );
        let ctx = make_context(&[(
            "files",
            json!([
                {"path": "report1.pdf"},
                {"path": "report2.pdf"},
                {"path": "report3.pdf"}
            ]),
        )]);
        let email = render_context(&t, &ctx, Path::new(".")).unwrap();
        assert_eq!(email.attachments.len(), 3);
        assert!(email.attachments[0]
            .to_string_lossy()
            .contains("report1.pdf"));
        assert!(email.attachments[1]
            .to_string_lossy()
            .contains("report2.pdf"));
        assert!(email.attachments[2]
            .to_string_lossy()
            .contains("report3.pdf"));
    }

    #[test]
    fn test_unresolved_variable_error() {
        let t = make_template(
            "sources:\n  p: {primary: true}\nto: 'a@b.com'\nsubject: 'Hello {{missing_var}}'\nbody: ''\nbody_format: text",
        );
        let ctx = make_context(&[]);
        let err = render_context(&t, &ctx, Path::new(".")).unwrap_err();
        assert!(
            matches!(err, MailnirError::HandlebarsRender { ref field, .. } if field == "subject"),
            "expected HandlebarsRender on subject, got: {err}"
        );
    }

    #[test]
    fn test_optional_fields_none() {
        let t = make_template(
            "sources:\n  p: {primary: true}\nto: 'a@b.com'\nsubject: s\nbody: ''\nbody_format: text",
        );
        let ctx = make_context(&[]);
        let email = render_context(&t, &ctx, Path::new(".")).unwrap();
        assert!(email.cc.is_none());
        assert!(email.bcc.is_none());
    }

    #[test]
    fn test_no_attachments_field() {
        let t = make_template(
            "sources:\n  p: {primary: true}\nto: 'a@b.com'\nsubject: s\nbody: ''\nbody_format: text",
        );
        let ctx = make_context(&[]);
        let email = render_context(&t, &ctx, Path::new(".")).unwrap();
        assert!(email.attachments.is_empty());
    }
}
