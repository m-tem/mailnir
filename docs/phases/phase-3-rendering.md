# Phase 3 — Rendering Pipeline

## Tasks

1. Handlebars rendering: apply merged context to each template field (to, cc, bcc, subject, body, attachments)
2. Markdown → HTML: pipe rendered body through `comrak` (GFM tables, strikethrough, autolinks)
3. CSS inlining: if `stylesheet` or `style` present, apply `css-inline` to HTML output
4. Plain-text fallback: strip markdown from rendered body
5. Raw HTML + plain-text-only modes via `body_format` key
6. Attachment path resolution: split rendered attachments by newline, trim blanks

## Exit Criteria

- Unit: Render template with nested `{{#each}}`, assert output matches expected string
- Unit: Markdown body → HTML contains expected tags (`<h1>`, `<strong>`, `<ul>`)
- Unit: CSS inlining — `h1 { color: red }` + `<h1>Hi</h1>` → `<h1 style="color: red;">Hi</h1>`
- Unit: Stylesheet from file path loaded and applied correctly
- Unit: Plain-text fallback strips HTML/markdown, preserves content
- Unit: `body_format: html` skips markdown step, still applies CSS inlining
- Unit: `body_format: text` produces no HTML part, ignores stylesheet
- Unit: Attachments field with 3 `{{#each}}` entries → 3 file paths
- Unit: Unresolved `{{variable}}` → error (strict mode), not empty string
