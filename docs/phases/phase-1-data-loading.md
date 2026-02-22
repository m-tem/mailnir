# Phase 1 — Data Loading & Template Parsing

## Tasks

1. Parse `.mailnir.yml` into typed struct: `sources`, `to`, `cc`, `bcc`, `subject`, `body`, `attachments`, `body_format`
2. Load data files: JSON, YAML, TOML → `serde_json::Value`; CSV → vec of maps (first row = headers)
3. Auto-detect format by extension, fall back to content sniffing
4. Validate `sources` block: exactly one `primary: true`, join keys reference valid `namespace.field` paths

## Exit Criteria

- Unit: Parse 5+ template fixtures covering all field combos (minimal, full, anchors, composite joins, no attachments)
- Unit: Load each data format, assert field access by namespace + key
- Unit: CSV with `;` separator parsed correctly
- Unit: CSV with `|` separator parsed correctly
- Unit: CSV with Latin-1 encoding decoded correctly
- Unit: CSV separator auto-detection: `,` vs `;` vs `|` vs `\t`
- Unit: Reject invalid templates: no primary, duplicate primaries, join referencing unknown namespace
- Integration: Round-trip test — parse template YAML, serialize back, parse again, assert equality
