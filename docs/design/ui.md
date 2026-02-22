# UI Structure

Four main areas:

1. **Data panel** (left): Lists all namespaces from template `sources`. Each shows: namespace name, join keys, file load status (⚠ required / ✓ loaded). User clicks to select a file for each. Validation and preview disabled until all required sources are loaded.
2. **Template editor** (center): Per-field editors for To, CC, BCC, Subject, Body, Attachments — mapped to YAML keys. Autocomplete for `{{namespace.field}}`. Syntax highlighting for Handlebars expressions. Editing writes back to the YAML file.
3. **Preview panel** (right): Rendered email preview with instance navigation.
4. **Status bar** (bottom): Validation summary, SMTP profile selector, Send button.
