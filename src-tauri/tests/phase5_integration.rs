//! Phase 5 integration tests — require a local mailhog instance.
//!
//! Run with:
//!   cargo test --test phase5_integration -- --include-ignored
//!
//! Start mailhog first (default: localhost:1025, no auth, no TLS):
//!   docker run -p 1025:1025 -p 8025:8025 mailhog/mailhog

use mailnir_lib::{
    render::RenderedEmail,
    smtp::{send_all, Encryption, SmtpCredentials, SmtpProfile},
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

fn make_email(index: usize) -> RenderedEmail {
    RenderedEmail {
        to: format!("recipient{index}@example.com"),
        cc: None,
        bcc: None,
        subject: format!("Phase 5 integration test #{index}"),
        html_body: Some(format!("<p>Entry {index}</p>")),
        text_body: format!("Entry {index}"),
        attachments: vec![],
    }
}

#[tokio::test]
#[ignore = "requires mailhog on localhost:1025"]
async fn test_send_10_emails() {
    let emails: Vec<RenderedEmail> = (0..10).map(make_email).collect();
    let profile = mailhog_profile(1);
    let creds = no_credentials();

    let report = send_all(&emails, &profile, &creds).await;

    assert_eq!(report.success_count(), 10, "all 10 emails should succeed");
    assert_eq!(report.failure_count(), 0, "no failures expected");

    for result in &report.results {
        assert!(
            result.success,
            "entry {} failed: {:?}",
            result.entry_index, result.error
        );
    }
}

#[tokio::test]
#[ignore = "requires mailhog on localhost:1025"]
async fn test_parallel_send_9_emails_concurrency_3() {
    // Send 9 emails with parallelism=3.
    // Correctness: all 9 succeed.
    // Concurrency cap: verifiable via mailhog connection log (manual step).
    let emails: Vec<RenderedEmail> = (0..9).map(make_email).collect();
    let profile = mailhog_profile(3);
    let creds = no_credentials();

    let report = send_all(&emails, &profile, &creds).await;

    assert_eq!(report.success_count(), 9, "all 9 emails should succeed");
    assert_eq!(report.failure_count(), 0, "no failures expected");
}

#[tokio::test]
#[ignore = "requires mailhog on localhost:1025"]
async fn test_send_with_cc_and_subject() {
    let email = RenderedEmail {
        to: "to@example.com".to_string(),
        cc: Some("cc@example.com".to_string()),
        bcc: None,
        subject: "Headers test".to_string(),
        html_body: None,
        text_body: "Plain text body.".to_string(),
        attachments: vec![],
    };
    let profile = mailhog_profile(1);
    let creds = no_credentials();

    let report = send_all(&[email], &profile, &creds).await;
    assert_eq!(report.success_count(), 1);
    assert_eq!(report.failure_count(), 0);
}
