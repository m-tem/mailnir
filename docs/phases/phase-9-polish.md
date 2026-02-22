# Phase 9 — Polish & Edge Cases

## Tasks

1. Attachment support end-to-end (path resolution + MIME type detection + email attachment)
2. CC/BCC fields wired through full pipeline
3. Raw HTML and plain-text-only body modes in UI toggle
4. Large batch UX: progress bar, cancel button, send report
5. Error recovery: partial send failure → report which succeeded, allow retry of failures

## Exit Criteria

- Integration: Send email with 2 attachments to test SMTP server → server receives email with correct MIME parts
- Integration: CC/BCC fields populated → test server receives email with all recipients
- Integration: 50-entry batch, 2 failures mid-send → report shows 48 success, 2 failed with detail, retry sends only the 2
