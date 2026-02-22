# Phase 5 — SMTP & Sending

## Tasks

1. Profile CRUD: host, port, encryption, from, parallelism — persisted to app config (Tauri's `app_data_dir`)
2. Credential storage: save/retrieve/delete via `keyring` crate, keyed by profile name
3. Connection test: verify SMTP settings without sending
4. Send flow: confirmation step → parallel send with configurable concurrency → progress tracking
5. Retry on transient errors (421, 452) via `lettre` transport config
6. Final report: success/failure per entry with error details

## Exit Criteria

- Unit: Profile serialization round-trip (create, save, load, assert equality)
- Unit: Credential store/retrieve/delete via keyring (mock or OS keychain in CI)
- Integration: Send to local SMTP test server (`mailhog` or `smtp4dev`), assert received email count, headers, body, attachments
- Integration: Simulate 421 response → assert retry occurs and succeeds on second attempt
- Integration: Parallelism = 3, 9 emails → assert max 3 concurrent connections (via test server connection log)
