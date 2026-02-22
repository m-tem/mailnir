# Phase 6 — UI Shell

## Tasks

1. Tauri + React + shadcn/ui scaffolding
2. Template open/save via OS file dialog
3. Data panel: namespace slots from parsed `sources`, file picker per slot, status indicators
4. CSV config panel: separator/encoding override + 5-row data preview table
5. Unlock preview/send only when all sources loaded
6. SMTP profile selector + settings dialog

## Exit Criteria

- Integration: Open fixture template → data panel shows correct namespace count with ⚠ status
- Integration: Load file for each namespace → all show ✓, preview button enabled
- Integration: Load `;`-separated CSV → auto-detects separator, preview table shows correct columns
- Integration: Override encoding to Latin-1 → preview table shows correctly decoded text
- Integration: No primary file loaded → send button disabled
