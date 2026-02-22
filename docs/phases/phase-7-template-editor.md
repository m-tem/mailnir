# Phase 7 — Template Editor

## Tasks

1. Per-field editors mapped to YAML keys (To, CC, BCC, Subject, Body, Attachments)
2. Syntax highlighting for `{{handlebars}}` expressions
3. Namespace-aware autocomplete: namespace names available immediately, field names after file load
4. Edits write back to YAML structure

## Exit Criteria

- Integration: Edit `subject` field in UI → save → re-open → field value persisted
- Integration: Type `{{cl` → autocomplete suggests `classes` namespace
- Integration: Load classes.json → type `{{classes.` → autocomplete suggests field names from file
