# SMTP & Sending

## Configuration

SMTP settings stored per-profile in app config:

- Host, port, encryption (TLS/STARTTLS/none)
- From address
- Send parallelism (default: 1, configurable)
- Credentials stored in OS keychain, referenced by profile name

Multiple profiles supported (e.g. `work`, `personal`).

## Send Flow

1. User clicks "Send" → full validation runs on all instances.
2. **Confirmation dialog**: Shows total count, lists all resolved recipients (To/CC/BCC per instance). Scrollable.
3. On confirm → emails sent with configured parallelism. Progress bar with per-instance status.
4. Retry on transient SMTP errors (421, 452) handled by `lettre`.
5. Final report: success count, failures with error detail.
