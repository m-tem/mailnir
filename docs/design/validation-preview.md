# Validation & Preview

## Validation

Before preview or send, every email instance is validated:

- **Unresolved variables**: Any `{{expr}}` that doesn't resolve to a value → error.
- **Missing joins**: Secondary source lookup returns no match → error.
- **Email format**: To/CC/BCC fields validated as RFC 5322 addresses.
- **Attachment paths**: Each resolved path checked for file existence.
- **Empty fields**: Required fields (To, Subject, Body) must be non-empty after rendering.
- **Stylesheet**: If `stylesheet` path specified, check file exists.

Validation runs on all N instances (one per primary entry). Results shown as a list: ✓ per instance, or specific errors with instance index + field.

## Preview

- **Instance navigator**: Step through each email (1 of N) with prev/next.
- **HTML preview**: Rendered email displayed in an embedded webview.
- **Plain text tab**: Shows the plain-text version side by side.
- **Metadata panel**: Shows resolved To/CC/BCC, Subject, Attachments for current instance.
- **Error highlights**: Instances with validation errors are flagged in the navigator.
