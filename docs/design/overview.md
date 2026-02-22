# Mailnir — Design Overview

Batch email tool with Handlebars templates, multi-source data binding, markdown body rendering, and SMTP sending. Desktop app for Windows and Linux.

**Example workflow**: Open a `.mailnir.yml` template declaring `classes` (primary), `inst`, `contacts` as sources. Mailnir shows three file slots. Select a JSON, CSV, and TOML file for each. Preview each resolved email. Send via SMTP.

## Tech Stack

- **Framework**: Tauri 2 — Rust backend + web frontend. Native packaging, small binary, OS-level APIs (keychain).
- **Frontend**: TypeScript + React. Modern UI via shadcn/ui.
- **Templating**: Handlebars (via `handlebars-rs` in backend).
- **Markdown**: `comrak` (GFM-compatible) — renders template output to HTML email body.
- **CSS inlining**: `css-inline` — converts stylesheet rules to inline `style` attributes for email client compatibility.
- **Email**: `lettre` — SMTP client with TLS, retry on 421/452.
- **Credential storage**: OS keychain via `keyring` crate (Windows Credential Manager / Linux Secret Service).

## Key Design Decisions

- **Handlebars** over Jinja — declarative, logic-light templates.
- **YAML template files** — robust parsing, anchors, no frontmatter ambiguity.
- **Joins as YAML maps** — `{ class_id: classes.id }`, no expression parser.
- **Markdown for email body** — sidesteps rich text editor entirely.
- **CSS inlining** via `css-inline` — email clients strip `<style>` tags.
- **OS keychain** for SMTP credentials — not plaintext config files.
- **File sources only for MVP** — URL and form sources are future.

## Non-Scope

- **Contact management / address book**: Data comes from files, not a built-in DB.
- **IMAP / receiving email**: Send-only tool.
- **Rich text editor / WYSIWYG**: Markdown covers this.
- **Scheduling / cron**: Manual send only. (future)
- **URL and form data sources**: Architecture supports them, but MVP is file-only. (future)
- **OAuth2 SMTP auth**: Username/password + app passwords only for MVP. (future)
- **macOS support**: Windows + Linux only for now. (Tauri supports macOS, low effort to add later.)

## Open Questions

- **Large batch UX**: For 500+ recipients, configurable send parallelism (in SMTP profile settings). Need to determine sensible defaults and upper bounds.
- **Inline images**: Markdown `![](image.png)` → embedded CID attachment in HTML email? (future, not MVP)
