# Data System

## Template-Declared Sources

Templates declare their data requirements as the `sources` key in the YAML template file. Namespaces, primary source, and joins are part of the template — not the UI session.

On open, Mailnir parses `sources` and knows exactly what data is needed. The user only has to select a file for each declared namespace.

Supported data formats (auto-detected by extension): JSON, YAML, TOML, CSV.

**CSV handling**: First row = headers. On file load, Mailnir auto-detects separator and encoding. The UI shows a CSV config panel for manual override:
- **Separator**: `,` `;` `|` `\t` or custom character
- **Encoding**: UTF-8 (default), Latin-1, Windows-1252, etc.

A data preview table (first 5 rows) confirms correct parsing before committing.

## Source Types

By resolution strategy:
- **Primary source**: Iterated directly — one email per entry.
- **Secondary sources**: Joined via `join` map (e.g. `{ class_id: classes.id }`). Resolves to single record (1:1) or list (1:N, accessed via `{{#each}}`).
- **Global sources**: No join — entire dataset available (e.g. `sources: cfg: {}`).

By data origin:
- **File**: Load from disk — user picks file in UI. (MVP)
- **URL**: Fetch from HTTP endpoint, e.g. `sources: api: { url: "https://..." }`. (future)
- **Form**: No file — Mailnir infers fields from template variables and displays an input form. Ideal for one-off emails without a data file. (future)

All origins produce the same namespace → data shape. They compose freely within one template.

## Autocomplete

Namespace names are known from `sources` before any files are loaded → editor can scope autocomplete by namespace immediately. Once a file is loaded for a namespace, field-level autocomplete activates using parsed schema (field names + types).
