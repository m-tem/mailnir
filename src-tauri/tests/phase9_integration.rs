//! Phase 9 integration tests — require a local mailhog instance.
//!
//! Run with:
//!   cargo test --test phase9_integration -- --include-ignored
//!
//! Start mailhog first (default: localhost:1025, no auth, no TLS):
//!   docker run -p 1025:1025 -p 8025:8025 mailhog/mailhog

use std::io::Write as _;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use mailnir_lib::{
    render::RenderedEmail,
    smtp::{send_all, send_all_with_progress, Encryption, SmtpCredentials, SmtpProfile},
};

fn mailhog_profile(parallelism: usize) -> SmtpProfile {
    SmtpProfile {
        name: "mailhog".to_string(),
        host: "localhost".to_string(),
        port: 1025,
        encryption: Encryption::None,
        from: "sender@example.com".to_string(),
        parallelism,
    }
}

fn no_credentials() -> SmtpCredentials {
    SmtpCredentials {
        username: String::new(),
        password: String::new(),
    }
}

/// Search mailhog messages for one matching the given subject.
async fn find_mailhog_message(subject: &str) -> serde_json::Value {
    let url = format!(
        "http://localhost:8025/api/v2/search?kind=containing&query={}",
        urlencoding::encode(subject)
    );
    let resp: serde_json::Value = reqwest::get(&url)
        .await
        .expect("mailhog API should be reachable")
        .json()
        .await
        .expect("mailhog response should be valid JSON");

    let count = resp["count"].as_u64().unwrap_or(0);
    assert!(
        count >= 1,
        "expected at least 1 message with subject containing '{subject}', got {count}"
    );
    resp["items"][0].clone()
}

// ── Exit criterion: attachments with correct MIME types ──────────────────────

#[tokio::test]
#[ignore = "requires mailhog on localhost:1025"]
async fn test_send_email_with_2_attachments_correct_mime() {
    let mut tmp_pdf = tempfile::Builder::new().suffix(".pdf").tempfile().unwrap();
    tmp_pdf.write_all(b"%PDF-1.4 fake content").unwrap();

    let mut tmp_png = tempfile::Builder::new().suffix(".png").tempfile().unwrap();
    tmp_png.write_all(b"\x89PNG fake content").unwrap();

    let subject = format!("MIME-test-{}", std::process::id());
    let email = RenderedEmail {
        to: "attach-test@example.com".to_string(),
        cc: None,
        bcc: None,
        subject: subject.clone(),
        html_body: Some("<p>See attachments</p>".to_string()),
        text_body: "See attachments".to_string(),
        attachments: vec![tmp_pdf.path().to_path_buf(), tmp_png.path().to_path_buf()],
    };

    let report = send_all(&[email], &mailhog_profile(1), &no_credentials()).await;
    assert_eq!(report.success_count(), 1, "email should send successfully");
    assert_eq!(report.failure_count(), 0);

    // Verify via mailhog API that the email was received with correct MIME parts.
    let item = find_mailhog_message(&subject).await;
    let raw = item["Raw"]["Data"]
        .as_str()
        .expect("raw message data should exist");
    assert!(
        raw.contains("application/pdf"),
        "expected application/pdf in raw message"
    );
    assert!(
        raw.contains("image/png"),
        "expected image/png in raw message"
    );
}

// ── Exit criterion: CC/BCC fields populated ──────────────────────────────────

#[tokio::test]
#[ignore = "requires mailhog on localhost:1025"]
async fn test_cc_bcc_fields_populated() {
    let subject = format!("CCBCC-test-{}", std::process::id());
    let email = RenderedEmail {
        to: "to-ccbcc@example.com".to_string(),
        cc: Some("cc-ccbcc@example.com".to_string()),
        bcc: Some("bcc-ccbcc@example.com".to_string()),
        subject: subject.clone(),
        html_body: Some("<p>CC/BCC test</p>".to_string()),
        text_body: "CC/BCC test".to_string(),
        attachments: vec![],
    };

    let report = send_all(&[email], &mailhog_profile(1), &no_credentials()).await;
    assert_eq!(report.success_count(), 1);
    assert_eq!(report.failure_count(), 0);

    let item = find_mailhog_message(&subject).await;

    // CC header should be present in the message headers.
    let headers = &item["Content"]["Headers"];
    let cc_header = headers["Cc"].as_array().expect("CC header should exist");
    assert!(
        cc_header
            .iter()
            .any(|v| v.as_str().unwrap_or("").contains("cc-ccbcc@example.com")),
        "CC header should contain cc-ccbcc@example.com, got: {cc_header:?}"
    );

    // BCC should NOT appear in headers (per SMTP spec), but the BCC recipient
    // should still receive the message. Mailhog tracks this in the envelope.
    let bcc_in_headers = headers.get("Bcc");
    assert!(
        bcc_in_headers
            .and_then(|v| v.as_array())
            .is_none_or(|a| a.is_empty()),
        "BCC should not appear in headers"
    );
}

// ── Exit criterion: 50-entry batch, 2 failures, retry ────────────────────────

#[tokio::test]
#[ignore = "requires mailhog on localhost:1025"]
async fn test_50_entry_batch_2_failures_retry() {
    let mut emails: Vec<RenderedEmail> = (0..50)
        .map(|i| RenderedEmail {
            to: format!("batch{i}@example.com"),
            cc: None,
            bcc: None,
            subject: format!("Batch #{i}"),
            html_body: Some(format!("<p>Entry {i}</p>")),
            text_body: format!("Entry {i}"),
            attachments: vec![],
        })
        .collect();

    // Deliberately break 2 entries with invalid addresses that fail build_message.
    emails[10].to = "not@@valid".to_string();
    emails[30].to = "also@@broken".to_string();

    let report = send_all(&emails, &mailhog_profile(4), &no_credentials()).await;

    assert_eq!(
        report.success_count(),
        48,
        "48 of 50 should succeed; failures: {:?}",
        report.failures().collect::<Vec<_>>()
    );
    assert_eq!(report.failure_count(), 2, "2 should fail");

    let failures: Vec<_> = report.failures().collect();
    assert!(
        failures.iter().any(|f| f.entry_index == 10),
        "entry 10 should have failed"
    );
    assert!(
        failures.iter().any(|f| f.entry_index == 30),
        "entry 30 should have failed"
    );

    // Fix the failed entries and retry only those.
    emails[10].to = "batch10fixed@example.com".to_string();
    emails[30].to = "batch30fixed@example.com".to_string();

    let retry_emails: Vec<RenderedEmail> = failures
        .iter()
        .map(|f| emails[f.entry_index].clone())
        .collect();

    let retry_report = send_all(&retry_emails, &mailhog_profile(1), &no_credentials()).await;
    assert_eq!(
        retry_report.success_count(),
        2,
        "retry should succeed for both"
    );
    assert_eq!(retry_report.failure_count(), 0);
}

// ── Cancellation test ────────────────────────────────────────────────────────

#[tokio::test]
#[ignore = "requires mailhog on localhost:1025"]
async fn test_cancel_mid_batch() {
    let emails: Vec<RenderedEmail> = (0..20)
        .map(|i| RenderedEmail {
            to: format!("cancel{i}@example.com"),
            cc: None,
            bcc: None,
            subject: format!("Cancel test #{i}"),
            html_body: Some(format!("<p>Cancel {i}</p>")),
            text_body: format!("Cancel {i}"),
            attachments: vec![],
        })
        .collect();

    let cancel = Arc::new(AtomicBool::new(false));
    let cancel_clone = cancel.clone();

    // Cancel after 3 emails complete.
    let report = send_all_with_progress(
        &emails,
        &mailhog_profile(1),
        &no_credentials(),
        Some(cancel.clone()),
        Some(Arc::new(move |p| {
            if p.completed >= 3 {
                cancel_clone.store(true, Ordering::SeqCst);
            }
        })),
    )
    .await;

    // Some should succeed (at least 3), rest should be cancelled.
    assert!(
        report.success_count() >= 3,
        "at least 3 should succeed before cancellation, got {}",
        report.success_count()
    );
    assert!(
        report.success_count() < 20,
        "not all 20 should succeed if cancellation worked, got {}",
        report.success_count()
    );

    let cancelled_count = report
        .failures()
        .filter(|f| f.error.as_deref() == Some("cancelled"))
        .count();
    assert!(
        cancelled_count > 0,
        "at least some failures should be cancellations, failures: {:?}",
        report.failures().collect::<Vec<_>>()
    );
}
