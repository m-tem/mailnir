# Phase 8 — Preview

## Tasks

1. Instance navigator: prev/next through N resolved emails
2. HTML preview: rendered email in Tauri webview
3. Plain-text tab alongside HTML
4. Metadata panel: resolved To/CC/BCC, Subject, Attachments for current instance
5. Validation error flags in navigator

## Exit Criteria

- Integration: 5-entry primary source → navigator shows "1 of 5", prev/next cycles correctly
- Integration: HTML preview renders markdown body as formatted HTML
- Integration: Entry with validation error → flagged with ⚠ in navigator, error detail visible
- Integration: Attachments listed in metadata panel match resolved paths
