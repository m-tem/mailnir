# Mailnir

Batch email tool: Handlebars templates + multi-source data + markdown rendering + SMTP sending. Tauri 2 desktop app (Windows/Linux).

> **This is a living document.** Keep it accurate as the project evolves — especially project structure, current phase, and any new conventions. If something here is wrong, fix it.

## Quick Reference

Template = YAML file (`.mailnir.yml`). Every field is a Handlebars template. Sources declared in `sources:` key with join maps. Rendering pipeline: Handlebars → Markdown → HTML → CSS inline.

```yaml
sources:
  classes: { primary: true }
  inst:    { join: { class_id: classes.id } }
to: "{{inst.email}}"
subject: "Credentials for {{classes.name}}"
body: |
  # Hello {{inst.first_name}},
  {{#each classes.students}}
  - {{this.username}} / {{this.password}}
  {{/each}}
```

## Design Docs

For architecture and rationale, read these as needed:

| Doc | Contents |
|---|---|
| [docs/design/overview.md](docs/design/overview.md) | Tech stack, key decisions, non-scope, open questions |
| [docs/design/data-system.md](docs/design/data-system.md) | Sources, namespaces, joins, CSV handling, autocomplete |
| [docs/design/template-system.md](docs/design/template-system.md) | YAML format, attachments, stylesheets, body rendering modes |
| [docs/design/validation-preview.md](docs/design/validation-preview.md) | Validation rules, preview UI |
| [docs/design/smtp.md](docs/design/smtp.md) | SMTP profiles, credentials, send flow |
| [docs/design/ui.md](docs/design/ui.md) | UI layout: data panel, editor, preview, status bar |

## Tech Stack

| Layer | Choice |
|---|---|
| App framework | Tauri 2 |
| Backend | Rust |
| Frontend | TypeScript, React, shadcn/ui |
| Templating | `handlebars-rs` |
| Markdown | `comrak` (GFM) |
| CSS inlining | `css-inline` |
| Email | `lettre` |
| Credentials | `keyring` (OS keychain) |
| CSV | `csv` crate + `encoding_rs` |

## Project Structure

> **Keep this up to date.** When adding modules, files, or directories, update this section.

```
docs/
  design/
    overview.md         # Tech stack, key decisions, non-scope
    data-system.md      # Sources, namespaces, joins, CSV
    template-system.md  # YAML format, attachments, stylesheets
    validation-preview.md
    smtp.md
    ui.md
  phases/               # Development phases with exit criteria
    phase-1-data-loading.md
    phase-2-join-engine.md
    phase-3-rendering.md
    phase-4-validation.md
    phase-5-smtp.md
    phase-6-ui-shell.md
    phase-7-template-editor.md
    phase-8-preview.md
    phase-9-polish.md
src-tauri/              # Rust backend
  Cargo.toml
  src/
    lib.rs              # Library root — module declarations, Result alias
    main.rs             # Stub fn main() {} (Tauri wired in Phase 6)
    error.rs            # MailnirError enum (thiserror)
    template/
      mod.rs
      types.rs          # Template, SourceConfig, BodyFormat
      parse.rs          # parse_template(path), parse_template_str(str)
      validate.rs       # validate_sources(&Template)
    data/
      mod.rs
      format.rs         # DataFormat enum + detect_format(path)
      loader.rs         # load_file(path), load_file_csv(path, opts)
      json.rs           # load_json(path)
      yaml.rs           # load_yaml(path)
      toml.rs           # load_toml(path)
      csv.rs            # CsvOptions, detect_separator, decode_bytes, load_csv
    join/
      mod.rs            # build_contexts(&Template, sources) → Vec<Context> (Phase 2)
    render/             # Handlebars rendering, markdown→HTML, CSS inlining (Phase 3)
    validate/           # Validation: variables, emails, attachments, required fields (Phase 4)
    smtp/               # Profiles, keyring credentials, send flow (Phase 5)
  tests/
    phase1_integration.rs
  fixtures/
    templates/          # minimal, full, anchors, composite_join, html_body, text_body
    data/               # simple.json/yaml/toml, comma/semicolon/pipe/tab.csv, latin1/windows1252.csv
src/                    # React frontend
  components/
    DataPanel/          # Namespace slots, file picker, CSV config, status
    TemplateEditor/     # Per-field editors, Handlebars autocomplete
    Preview/            # Instance navigator, HTML/plain-text preview
    SmtpSettings/       # Profile CRUD, connection test
    SendDialog/         # Confirmation, progress, report
```

## Code Standards

### Rust

Functions are single-responsibility, single level of abstraction. No deep nesting.

Run regularly during development:
```sh
cargo clippy --all-features --all-targets -- -D warnings
cargo fmt --all
```

Every `pub fn` in core modules (`template/`, `data/`, `join/`, `render/`, `validate/`) must have a unit test. Use `#[cfg(test)]` modules colocated in each file.

Errors use a crate-level error enum with `thiserror`. No `.unwrap()` in non-test code.

### Frontend

Run regularly during development:
```sh
npx biome check --write
```

Components are single-responsibility. State flows down, events flow up. Tauri IPC commands are the only bridge to backend logic — no business logic in the frontend.

### Git Workflow

After finishing a development phase, divide all changes into focused commits with descriptive messages. Each commit should be a coherent unit of work.

Examples:
- `feat(data): add CSV loader with separator auto-detection`
- `feat(join): implement 1:1 join resolution`
- `test(join): add composite join and missing match cases`
- `feat(render): handlebars rendering with strict mode`
- `fix(validate): report entry index on unresolved variable`

Do NOT create a single large commit per phase. Do NOT mix refactors with features.

## Development Phases

Phases 1–5 are pure Rust backend. Phase 6+ adds UI. Each phase doc has tasks and exit criteria.

**Current phase: 3 — not started.** (Phases 1–2 complete)

| Phase | Doc | Summary |
|---|---|---|
| 1 | [phase-1-data-loading.md](docs/phases/phase-1-data-loading.md) | Template YAML parsing, data file loading, CSV handling |
| 2 | [phase-2-join-engine.md](docs/phases/phase-2-join-engine.md) | Primary iteration, 1:1/1:N joins, global sources |
| 3 | [phase-3-rendering.md](docs/phases/phase-3-rendering.md) | Handlebars, markdown→HTML, CSS inlining, attachments |
| 4 | [phase-4-validation.md](docs/phases/phase-4-validation.md) | Variable resolution, email format, file existence |
| 5 | [phase-5-smtp.md](docs/phases/phase-5-smtp.md) | Profiles, keyring, send flow, retries, parallelism |
| 6 | [phase-6-ui-shell.md](docs/phases/phase-6-ui-shell.md) | Tauri scaffolding, data panel, CSV config |
| 7 | [phase-7-template-editor.md](docs/phases/phase-7-template-editor.md) | Per-field editors, autocomplete, syntax highlighting |
| 8 | [phase-8-preview.md](docs/phases/phase-8-preview.md) | Instance navigation, HTML/text preview |
| 9 | [phase-9-polish.md](docs/phases/phase-9-polish.md) | Attachments e2e, batch UX, error recovery |

### Phase Completion Checklist

Before marking a phase complete:
1. All exit criteria tests from the phase doc pass
2. `cargo clippy --all-features --all-targets -- -D warnings` clean
3. `cargo fmt --all` applied
4. `npx biome check --write` applied (if frontend changes)
5. Changes divided into focused commits
6. Update "Current phase" above
7. Update project structure above if it changed

## Testing Strategy

### Unit Tests (Rust)

Colocated `#[cfg(test)]` modules. Test fixtures in `src-tauri/fixtures/`.

Focus areas:
- Template parsing: valid/invalid YAML, all field combos, anchors
- Data loading: each format, CSV separators (`,` `;` `|` `\t`), CSV encodings (UTF-8, Latin-1, Windows-1252)
- Join engine: 1:1, 1:N, composite, global, missing match, ambiguous match
- Rendering: Handlebars resolution, markdown→HTML, CSS inlining, body_format modes, attachment splitting
- Validation: unresolved vars, RFC 5322 emails, file existence, required fields

### Integration Tests (Rust)

In `src-tauri/tests/`. Full pipeline tests: template → data → join → render → validate.

SMTP integration tests use a local test server (`smtp4dev` or `mailhog`). Assert: email count, headers, body content, attachments, retry behavior, parallelism.

### Frontend Tests

Component-level tests for UI state logic (e.g. "all sources loaded → preview enabled").
