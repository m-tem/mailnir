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
  Cargo.toml            # tauri-backend feature gates tauri/tauri-plugin-dialog as optional
  build.rs              # tauri_build::build() gated by CARGO_FEATURE_TAURI_BACKEND
  tauri.conf.json       # App identifier, window size, devUrl/frontendDist
  capabilities/
    main-capability.json  # core:*:default, dialog:allow-open, dialog:allow-save
  src/
    lib.rs              # Library root — module declarations, Result alias (no Tauri)
    main.rs             # Tauri entry point — mod commands, builder, invoke_handler
    commands.rs         # IPC commands: parse_template_cmd, preview_csv, smtp CRUD, preview_validate, preview_render_entry, send_batch, cancel_send, get_form_fields
    error.rs            # MailnirError enum (thiserror)
    template/
      mod.rs
      types.rs          # Template, SourceConfig (incl. form flag), BodyFormat
      parse.rs          # parse_template(path), parse_template_str(str)
      infer.rs          # infer_form_fields(template, namespace) — field inference for form sources
      validate.rs       # validate_sources(&Template)
    data/
      mod.rs            # Shared helpers: normalize_shape, value_type_name
      format.rs         # DataFormat enum + detect_format(path)
      loader.rs         # load_file(path), load_file_csv(path, opts)
      json.rs           # load_json(path)
      yaml.rs           # load_yaml(path)
      toml.rs           # load_toml(path)
      csv.rs            # CsvOptions, detect_separator, decode_bytes, load_csv
    join/
      mod.rs            # build_contexts, build_contexts_lenient — primary iteration, 1:1/1:N, global (Phase 2)
    render/
      mod.rs            # render_context, RenderedEmail — Handlebars, markdown→HTML, CSS inlining (Phase 3)
    validate/
      mod.rs            # validate_all, ValidationReport, ValidationIssue — per-entry validation (Phase 4)
    smtp/
      mod.rs            # SmtpProfile, Encryption, SmtpCredentials, SendReport, send_all, send_all_with_progress (Phase 5/9)
  tests/
    phase1_integration.rs
    phase4_integration.rs
    phase5_integration.rs
    phase8_integration.rs
    phase9_integration.rs
    form_integration.rs   # Form data source pipeline tests
  fixtures/
    templates/          # minimal, full, anchors, composite_join, html_body, text_body, form_source
    data/               # simple.json/yaml/toml, comma/semicolon/pipe/tab.csv, latin1/windows1252.csv
package.json            # React 19, Vite, @tauri-apps/api v2, tailwindcss v4, shadcn/ui
vite.config.ts          # @tailwindcss/vite plugin, port 1420, @/ alias
tsconfig.json           # TypeScript project references
tsconfig.app.json       # App TypeScript config with @/* path alias
tsconfig.node.json      # Node/Vite TypeScript config
index.html              # Vite entry HTML
biome.json              # Biome linter/formatter config (run: npx biome check --write src/)
src/                    # React frontend
  index.css             # Tailwind v4 import + shadcn CSS variable theme
  main.tsx              # React entry point
  App.tsx               # Root component — all app state + 4-panel layout
  lib/
    ipc.ts              # Typed invoke() wrappers for all Tauri IPC commands
  components/
    DataPanel/
      index.tsx         # Namespace slot list, "Open a template" placeholder
      SourceSlotRow.tsx # Per-namespace row: file picker or form inputs, status icon, badges
      CsvConfigPanel.tsx  # Separator/encoding selectors + 5-row preview table
      FormFieldsPanel.tsx # Labeled inputs for form source fields
    SendDialog/
      SendDialog.tsx    # Confirm → progress → report dialog for batch send
    SmtpSettings/
      SmtpSettingsDialog.tsx  # Profile list/CRUD dialog with inline test
      SmtpProfileForm.tsx     # Add/edit SMTP profile form
    StatusBar/
      index.tsx         # Profile selector, SMTP Settings, Preview toggle, Send buttons
    TemplateEditor/
      index.tsx         # Per-field editors with save, body format selector
      FieldEditor.tsx   # Single-line CodeMirror editor with Handlebars autocomplete
      BodyEditor.tsx    # Multi-line CodeMirror editor for body field
      BodyFormatSelect.tsx  # Body format dropdown (markdown/html/text)
      handlebarsExtension.ts  # CM6 syntax highlighting + namespace/field autocomplete
    Preview/
      index.tsx         # Preview panel: navigator, metadata, HTML/text tabs, validation errors
      InstanceNavigator.tsx  # Prev/next through entries with validation warning icons
      MetadataPanel.tsx # Resolved To/CC/BCC/Subject/Attachments display
      HtmlPreview.tsx   # Sandboxed iframe srcdoc for rendered HTML
      TextPreview.tsx   # Plain-text view in scrollable monospace pre
      ValidationErrors.tsx  # Amber warning box with per-entry issue list
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
npx biome check --write src/
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

**Current phase: 9 — complete.** (Phases 1–9 complete)

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
- Form sources: field inference, form flag parsing, full pipeline with form data

### Integration Tests (Rust)

In `src-tauri/tests/`. Full pipeline tests: template → data → join → render → validate.

SMTP integration tests use a local test server (`smtp4dev` or `mailhog`). Assert: email count, headers, body content, attachments, retry behavior, parallelism.

### Frontend Tests

Component-level tests for UI state logic (e.g. "all sources loaded → preview enabled").
