# Phase 4 — Validation Engine

## Tasks

1. Unresolved variables: Handlebars strict mode, catch and report with field name + entry index
2. Missing joins: surface join engine errors per entry
3. Email format: validate To/CC/BCC as RFC 5322 after rendering
4. Attachment existence: check each resolved path against filesystem
5. Required fields: To, Subject, Body must be non-empty after rendering
6. Stylesheet: if `stylesheet` path specified, check file exists
7. Aggregate results: per-entry pass/fail with specific errors

## Exit Criteria

- Unit: Template with typo `{{classses.name}}` → error naming the field and entry
- Unit: Valid email passes, `not-an-email` fails RFC 5322 check
- Unit: Attachment path to nonexistent file → error with path and entry index
- Unit: Empty `to` after rendering → required field error
- Unit: `stylesheet` pointing to nonexistent file → error
- Integration: Full pipeline — 10-entry primary source, 2 with deliberate errors → validation report lists exactly those 2
