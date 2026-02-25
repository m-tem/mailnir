use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use lettre::{
    message::{Attachment, Mailbox, MultiPart, SinglePart},
    transport::smtp::{authentication::Credentials, Error as SmtpError},
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use serde::{Deserialize, Serialize};
use tokio::sync::Semaphore;

use crate::{render::RenderedEmail, MailnirError, Result};

fn default_parallelism() -> usize {
    1
}

/// Encryption mode for an SMTP connection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Encryption {
    None,
    StartTls,
    Tls,
}

/// Named SMTP send profile — connection settings and send behaviour.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SmtpProfile {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub encryption: Encryption,
    /// RFC 5322 from address used for all sent messages.
    pub from: String,
    /// Maximum number of concurrent SMTP connections (default: 1).
    #[serde(default = "default_parallelism")]
    pub parallelism: usize,
}

/// SMTP account credentials retrieved from the OS keychain.
#[derive(Debug, Clone)]
pub struct SmtpCredentials {
    pub username: String,
    pub password: String,
}

/// Send outcome for a single email entry.
#[derive(Debug, Clone, Serialize)]
pub struct SendResult {
    pub entry_index: usize,
    pub recipient: String,
    pub success: bool,
    pub error: Option<String>,
}

/// Aggregate send report for all entries.
#[derive(Debug, Serialize)]
pub struct SendReport {
    pub results: Vec<SendResult>,
}

/// Emitted per-entry as a Tauri event during batch send.
#[derive(Debug, Clone, Serialize)]
pub struct SendProgress {
    pub completed: usize,
    pub total: usize,
    pub entry_index: usize,
    pub recipient: String,
    pub success: bool,
    pub error: Option<String>,
}

impl SendReport {
    pub fn success_count(&self) -> usize {
        self.results.iter().filter(|r| r.success).count()
    }

    pub fn failure_count(&self) -> usize {
        self.results.iter().filter(|r| !r.success).count()
    }

    pub fn failures(&self) -> impl Iterator<Item = &SendResult> {
        self.results.iter().filter(|r| !r.success)
    }
}

/// Serialize `profiles` to a pretty-printed JSON file at `path` (creates or overwrites).
pub fn save_profiles(profiles: &[SmtpProfile], path: &Path) -> Result<()> {
    let file = std::fs::File::create(path).map_err(|e| MailnirError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;
    serde_json::to_writer_pretty(file, profiles).map_err(|e| MailnirError::ProfileJson {
        path: path.to_path_buf(),
        source: e,
    })
}

/// Deserialize profiles from a JSON file at `path`.
pub fn load_profiles(path: &Path) -> Result<Vec<SmtpProfile>> {
    let file = std::fs::File::open(path).map_err(|e| MailnirError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;
    serde_json::from_reader(file).map_err(|e| MailnirError::ProfileJson {
        path: path.to_path_buf(),
        source: e,
    })
}

const KEYRING_SERVICE: &str = "mailnir";

/// Store SMTP credentials in the OS keychain for `profile_name`.
///
/// Both `username` and `password` are stored in a single keyring entry,
/// separated by a newline.
pub fn store_credential(profile_name: &str, username: &str, password: &str) -> Result<()> {
    let entry =
        keyring::Entry::new(KEYRING_SERVICE, profile_name).map_err(|e| MailnirError::Keyring {
            reason: e.to_string(),
        })?;
    let value = format!("{username}\n{password}");
    entry
        .set_password(&value)
        .map_err(|e| MailnirError::Keyring {
            reason: e.to_string(),
        })
}

/// Retrieve SMTP credentials from the OS keychain for `profile_name`.
pub fn retrieve_credential(profile_name: &str) -> Result<SmtpCredentials> {
    let entry =
        keyring::Entry::new(KEYRING_SERVICE, profile_name).map_err(|e| MailnirError::Keyring {
            reason: e.to_string(),
        })?;
    let value = entry.get_password().map_err(|e| MailnirError::Keyring {
        reason: e.to_string(),
    })?;
    let (username, password) = value
        .split_once('\n')
        .ok_or_else(|| MailnirError::Keyring {
            reason: format!("malformed credential entry for profile '{profile_name}'"),
        })?;
    Ok(SmtpCredentials {
        username: username.to_string(),
        password: password.to_string(),
    })
}

/// Remove SMTP credentials from the OS keychain for `profile_name`.
pub fn delete_credential(profile_name: &str) -> Result<()> {
    let entry =
        keyring::Entry::new(KEYRING_SERVICE, profile_name).map_err(|e| MailnirError::Keyring {
            reason: e.to_string(),
        })?;
    entry
        .delete_credential()
        .map_err(|e| MailnirError::Keyring {
            reason: e.to_string(),
        })
}

/// Open an SMTP connection and verify the server is reachable (no message sent).
pub async fn test_connection(profile: &SmtpProfile, credentials: &SmtpCredentials) -> Result<()> {
    let transport = build_transport(profile, credentials)?;
    transport
        .test_connection()
        .await
        .map_err(|e| MailnirError::SmtpConnect {
            reason: e.to_string(),
        })?;
    Ok(())
}

/// Send all rendered emails using the given profile and credentials.
///
/// Concurrency is capped to `profile.parallelism` via a [`Semaphore`].
/// Per-entry failures are captured in [`SendReport`] — this function never returns `Err`.
pub async fn send_all(
    emails: &[RenderedEmail],
    profile: &SmtpProfile,
    credentials: &SmtpCredentials,
) -> SendReport {
    send_all_with_progress(emails, profile, credentials, None, None).await
}

/// Extended version of [`send_all`] with cancellation and progress reporting.
///
/// When `cancel` is set to `true`, remaining unsent emails are marked as cancelled.
/// In-flight sends (already acquired a semaphore permit) will complete.
/// The `on_progress` callback is invoked after each email completes (sent or failed).
pub async fn send_all_with_progress(
    emails: &[RenderedEmail],
    profile: &SmtpProfile,
    credentials: &SmtpCredentials,
    cancel: Option<Arc<std::sync::atomic::AtomicBool>>,
    on_progress: Option<Arc<dyn Fn(SendProgress) + Send + Sync>>,
) -> SendReport {
    use std::sync::atomic::Ordering;

    let transport = match build_transport(profile, credentials) {
        Ok(t) => t,
        Err(e) => {
            let reason = e.to_string();
            let results = emails
                .iter()
                .enumerate()
                .map(|(i, email)| SendResult {
                    entry_index: i,
                    recipient: email.to.clone(),
                    success: false,
                    error: Some(reason.clone()),
                })
                .collect();
            return SendReport { results };
        }
    };

    // Pre-build all messages before spawning tasks to avoid cloning RenderedEmail.
    let from = &profile.from;
    let pre_built: Vec<(usize, String, std::result::Result<Message, MailnirError>)> = emails
        .iter()
        .enumerate()
        .map(|(i, email)| (i, email.to.clone(), build_message(email, from, i)))
        .collect();

    let total = pre_built.len();
    let semaphore = Arc::new(Semaphore::new(profile.parallelism.max(1)));
    let mut handles: Vec<tokio::task::JoinHandle<SendResult>> = Vec::with_capacity(total);
    let mut cancelled_results: Vec<SendResult> = Vec::new();

    for (entry_index, recipient, message_result) in pre_built {
        // Check cancellation before spawning new tasks.
        if cancel.as_ref().is_some_and(|c| c.load(Ordering::Relaxed)) {
            cancelled_results.push(SendResult {
                entry_index,
                recipient,
                success: false,
                error: Some("cancelled".to_string()),
            });
            continue;
        }

        let transport = transport.clone();
        let sem = semaphore.clone();
        let cancel_inner = cancel.clone();
        let handle = tokio::spawn(async move {
            let _permit = sem.acquire().await.expect("semaphore closed");
            // Check cancellation after acquiring the permit — tasks queued behind
            // the semaphore will see the flag once they wake up.
            if cancel_inner
                .as_ref()
                .is_some_and(|c| c.load(Ordering::Relaxed))
            {
                return SendResult {
                    entry_index,
                    recipient,
                    success: false,
                    error: Some("cancelled".to_string()),
                };
            }
            match message_result {
                Ok(message) => match send_with_retry(&transport, message).await {
                    Ok(()) => SendResult {
                        entry_index,
                        recipient,
                        success: true,
                        error: None,
                    },
                    Err(e) => SendResult {
                        entry_index,
                        recipient,
                        success: false,
                        error: Some(e.to_string()),
                    },
                },
                Err(e) => SendResult {
                    entry_index,
                    recipient,
                    success: false,
                    error: Some(e.to_string()),
                },
            }
        });
        handles.push(handle);
    }

    let mut results = Vec::with_capacity(total);
    for handle in handles {
        let result = match handle.await {
            Ok(result) => result,
            Err(e) => SendResult {
                entry_index: results.len(),
                recipient: String::new(),
                success: false,
                error: Some(format!("task panicked: {e}")),
            },
        };

        if let Some(ref progress_fn) = on_progress {
            progress_fn(SendProgress {
                completed: results.len() + 1,
                total,
                entry_index: result.entry_index,
                recipient: result.recipient.clone(),
                success: result.success,
                error: result.error.clone(),
            });
        }

        results.push(result);

        // Check cancellation between completions — stop spawning was handled above,
        // but in-flight tasks still complete. Mark any remaining awaited handles
        // as cancelled if flag was set during iteration.
        if cancel.as_ref().is_some_and(|c| c.load(Ordering::Relaxed)) {
            // Let already-spawned tasks complete naturally (they hold semaphore permits).
            // We continue collecting their results below.
        }
    }

    // Append cancelled entries and emit progress for them.
    for cr in cancelled_results {
        if let Some(ref progress_fn) = on_progress {
            progress_fn(SendProgress {
                completed: results.len() + 1,
                total,
                entry_index: cr.entry_index,
                recipient: cr.recipient.clone(),
                success: cr.success,
                error: cr.error.clone(),
            });
        }
        results.push(cr);
    }

    SendReport { results }
}

/// Build a lettre async SMTP transport from the given profile and credentials.
fn build_transport(
    profile: &SmtpProfile,
    credentials: &SmtpCredentials,
) -> Result<AsyncSmtpTransport<Tokio1Executor>> {
    let creds = Credentials::new(credentials.username.clone(), credentials.password.clone());
    let transport = match profile.encryption {
        Encryption::Tls => AsyncSmtpTransport::<Tokio1Executor>::relay(&profile.host)
            .map_err(|e| MailnirError::SmtpConnect {
                reason: e.to_string(),
            })?
            .port(profile.port)
            .credentials(creds)
            .build(),
        Encryption::StartTls => AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&profile.host)
            .map_err(|e| MailnirError::SmtpConnect {
                reason: e.to_string(),
            })?
            .port(profile.port)
            .credentials(creds)
            .build(),
        Encryption::None => AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&profile.host)
            .port(profile.port)
            .credentials(creds)
            .build(),
    };
    Ok(transport)
}

/// Build a lettre [`Message`] from a [`RenderedEmail`] and a from-address.
///
/// Produces `multipart/alternative` when `html_body` is present, plain text otherwise.
/// Attachments are wrapped in an outer `multipart/mixed`.
fn build_message(email: &RenderedEmail, from: &str, entry_index: usize) -> Result<Message> {
    let from_mbox = from
        .parse::<Mailbox>()
        .map_err(|e| MailnirError::SmtpSend {
            entry_index,
            reason: format!("invalid from address '{from}': {e}"),
        })?;
    let to_mbox = email
        .to
        .parse::<Mailbox>()
        .map_err(|e| MailnirError::SmtpSend {
            entry_index,
            reason: format!("invalid to address '{}': {e}", email.to),
        })?;

    let mut builder = Message::builder()
        .from(from_mbox)
        .to(to_mbox)
        .subject(&email.subject);

    if let Some(cc) = &email.cc {
        let mbox = cc.parse::<Mailbox>().map_err(|e| MailnirError::SmtpSend {
            entry_index,
            reason: format!("invalid cc address '{cc}': {e}"),
        })?;
        builder = builder.cc(mbox);
    }
    if let Some(bcc) = &email.bcc {
        let mbox = bcc.parse::<Mailbox>().map_err(|e| MailnirError::SmtpSend {
            entry_index,
            reason: format!("invalid bcc address '{bcc}': {e}"),
        })?;
        builder = builder.bcc(mbox);
    }

    let message = if let Some(html) = &email.html_body {
        let alt = MultiPart::alternative()
            .singlepart(SinglePart::plain(email.text_body.clone()))
            .singlepart(SinglePart::html(html.clone()));
        if email.attachments.is_empty() {
            builder.multipart(alt)
        } else {
            let mut mixed = MultiPart::mixed().multipart(alt);
            for path in &email.attachments {
                let bytes = std::fs::read(path).map_err(|e| MailnirError::Io {
                    path: path.clone(),
                    source: e,
                })?;
                let name = attachment_name(path);
                let content_type = guess_content_type(path);
                mixed = mixed.singlepart(Attachment::new(name).body(bytes, content_type));
            }
            builder.multipart(mixed)
        }
    } else if email.attachments.is_empty() {
        builder.body(email.text_body.clone())
    } else {
        let mut mixed = MultiPart::mixed().singlepart(SinglePart::plain(email.text_body.clone()));
        for path in &email.attachments {
            let bytes = std::fs::read(path).map_err(|e| MailnirError::Io {
                path: path.clone(),
                source: e,
            })?;
            let name = attachment_name(path);
            let content_type = guess_content_type(path);
            mixed = mixed.singlepart(Attachment::new(name).body(bytes, content_type));
        }
        builder.multipart(mixed)
    };

    message.map_err(|e| MailnirError::SmtpSend {
        entry_index,
        reason: format!("failed to build message: {e}"),
    })
}

fn attachment_name(path: &std::path::Path) -> String {
    path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("attachment")
        .to_string()
}

/// Guess the MIME content type from a file extension, falling back to `application/octet-stream`.
fn guess_content_type(path: &std::path::Path) -> lettre::message::header::ContentType {
    let fallback: lettre::message::header::ContentType =
        "application/octet-stream".parse().expect("valid MIME");
    mime_guess::from_path(path)
        .first()
        .and_then(|mime| mime.to_string().parse().ok())
        .unwrap_or(fallback)
}

/// Send `message`, retrying up to 3 times on transient SMTP errors (421, 452).
async fn send_with_retry(
    transport: &AsyncSmtpTransport<Tokio1Executor>,
    message: Message,
) -> std::result::Result<(), SmtpError> {
    const MAX_ATTEMPTS: u32 = 3;
    const RETRY_DELAY: Duration = Duration::from_millis(500);

    let mut last_err: Option<SmtpError> = None;
    for attempt in 0..MAX_ATTEMPTS {
        match transport.send(message.clone()).await {
            Ok(_) => return Ok(()),
            Err(e) if attempt + 1 < MAX_ATTEMPTS && is_transient_error(&e) => {
                last_err = Some(e);
                tokio::time::sleep(RETRY_DELAY).await;
            }
            Err(e) => return Err(e),
        }
    }
    Err(last_err.unwrap())
}

/// Return `true` for SMTP 421/452 response codes (transient server-side failures).
fn is_transient_error(err: &SmtpError) -> bool {
    let s = err.to_string();
    s.starts_with("421") || s.starts_with("452")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::RenderedEmail;
    use tempfile::NamedTempFile;

    fn sample_profile(name: &str) -> SmtpProfile {
        SmtpProfile {
            name: name.to_string(),
            host: "smtp.example.com".to_string(),
            port: 587,
            encryption: Encryption::StartTls,
            from: "sender@example.com".to_string(),
            parallelism: 1,
        }
    }

    fn sample_email(to: &str) -> RenderedEmail {
        RenderedEmail {
            to: to.to_string(),
            cc: None,
            bcc: None,
            subject: "Test Subject".to_string(),
            html_body: Some("<p>Hello</p>".to_string()),
            text_body: "Hello".to_string(),
            attachments: vec![],
        }
    }

    #[test]
    fn test_profile_serialization_roundtrip() {
        let profile = sample_profile("work");
        let tmp = NamedTempFile::new().unwrap();
        save_profiles(std::slice::from_ref(&profile), tmp.path()).unwrap();
        let loaded = load_profiles(tmp.path()).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0], profile);
    }

    #[test]
    fn test_profile_multiple_roundtrip() {
        let profiles = vec![
            sample_profile("work"),
            SmtpProfile {
                name: "personal".to_string(),
                host: "mail.personal.com".to_string(),
                port: 465,
                encryption: Encryption::Tls,
                from: "me@personal.com".to_string(),
                parallelism: 3,
            },
            SmtpProfile {
                name: "relay".to_string(),
                host: "localhost".to_string(),
                port: 25,
                encryption: Encryption::None,
                from: "relay@local".to_string(),
                parallelism: 1,
            },
        ];
        let tmp = NamedTempFile::new().unwrap();
        save_profiles(&profiles, tmp.path()).unwrap();
        let loaded = load_profiles(tmp.path()).unwrap();
        assert_eq!(loaded, profiles);
    }

    #[test]
    fn test_profile_default_parallelism() {
        let json =
            r#"[{"name":"p","host":"h","port":587,"encryption":"start_tls","from":"f@h.com"}]"#;
        let profiles: Vec<SmtpProfile> = serde_json::from_str(json).unwrap();
        assert_eq!(profiles[0].parallelism, 1);
    }

    #[test]
    fn test_credential_retrieve_missing_returns_error() {
        // A non-existent entry must produce our Keyring error regardless of backend.
        let result = retrieve_credential("mailnir-unit-test-nonexistent-xyz");
        assert!(
            result.is_err(),
            "retrieving a non-existent credential should return Err"
        );
        assert!(matches!(result, Err(MailnirError::Keyring { .. })));
    }

    #[test]
    fn test_send_report_counts() {
        let report = SendReport {
            results: vec![
                SendResult {
                    entry_index: 0,
                    recipient: "a@b.com".to_string(),
                    success: true,
                    error: None,
                },
                SendResult {
                    entry_index: 1,
                    recipient: "c@d.com".to_string(),
                    success: false,
                    error: Some("timeout".to_string()),
                },
                SendResult {
                    entry_index: 2,
                    recipient: "e@f.com".to_string(),
                    success: true,
                    error: None,
                },
            ],
        };
        assert_eq!(report.success_count(), 2);
        assert_eq!(report.failure_count(), 1);
        let failures: Vec<_> = report.failures().collect();
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].entry_index, 1);
    }

    #[test]
    fn test_build_message_headers() {
        let email = sample_email("recipient@example.com");
        let msg = build_message(&email, "sender@example.com", 0).unwrap();
        let raw = String::from_utf8(msg.formatted()).unwrap();
        assert!(raw.contains("recipient@example.com"), "missing To address");
        assert!(raw.contains("Subject: Test Subject"), "missing Subject");
        assert!(raw.contains("sender@example.com"), "missing From address");
    }

    #[test]
    fn test_build_message_multipart_html() {
        let email = sample_email("r@example.com");
        let msg = build_message(&email, "s@example.com", 0).unwrap();
        let raw = String::from_utf8(msg.formatted()).unwrap();
        assert!(
            raw.contains("multipart/alternative"),
            "expected multipart/alternative"
        );
        assert!(raw.contains("<p>Hello</p>"), "missing html body");
        assert!(raw.contains("Hello"), "missing text body");
    }

    #[test]
    fn test_build_message_plain_text_only() {
        let mut email = sample_email("r@example.com");
        email.html_body = None;
        let msg = build_message(&email, "s@example.com", 0).unwrap();
        let raw = String::from_utf8(msg.formatted()).unwrap();
        assert!(
            !raw.contains("multipart/alternative"),
            "should not have html alternative"
        );
        assert!(raw.contains("Hello"), "missing body content");
    }

    #[test]
    fn test_build_message_attachment_mime_types() {
        let mut tmp_pdf = tempfile::Builder::new().suffix(".pdf").tempfile().unwrap();
        std::io::Write::write_all(&mut tmp_pdf, b"%PDF-fake").unwrap();
        let mut tmp_png = tempfile::Builder::new().suffix(".png").tempfile().unwrap();
        std::io::Write::write_all(&mut tmp_png, b"\x89PNG-fake").unwrap();

        let email = RenderedEmail {
            to: "r@example.com".to_string(),
            cc: None,
            bcc: None,
            subject: "MIME test".to_string(),
            html_body: Some("<p>Hi</p>".to_string()),
            text_body: "Hi".to_string(),
            attachments: vec![tmp_pdf.path().to_path_buf(), tmp_png.path().to_path_buf()],
        };
        let msg = build_message(&email, "s@example.com", 0).unwrap();
        let raw = String::from_utf8(msg.formatted()).unwrap();
        assert!(
            raw.contains("application/pdf"),
            "expected PDF MIME type in: {raw}"
        );
        assert!(
            raw.contains("image/png"),
            "expected PNG MIME type in: {raw}"
        );
    }

    #[test]
    fn test_build_message_unknown_extension_fallback() {
        let mut tmp = tempfile::Builder::new()
            .suffix(".xyz123unknown")
            .tempfile()
            .unwrap();
        std::io::Write::write_all(&mut tmp, b"data").unwrap();

        let email = RenderedEmail {
            to: "r@example.com".to_string(),
            cc: None,
            bcc: None,
            subject: "Fallback MIME".to_string(),
            html_body: None,
            text_body: "Hi".to_string(),
            attachments: vec![tmp.path().to_path_buf()],
        };
        let msg = build_message(&email, "s@example.com", 0).unwrap();
        let raw = String::from_utf8(msg.formatted()).unwrap();
        assert!(
            raw.contains("application/octet-stream"),
            "unknown ext should fallback to octet-stream"
        );
    }

    #[test]
    fn test_guess_content_type_common_types() {
        let pdf = format!(
            "{:?}",
            guess_content_type(std::path::Path::new("report.pdf"))
        );
        assert!(
            pdf.contains("application") && pdf.contains("pdf"),
            "expected application/pdf, got: {pdf}"
        );

        let png = format!(
            "{:?}",
            guess_content_type(std::path::Path::new("image.png"))
        );
        assert!(
            png.contains("image") && png.contains("png"),
            "expected image/png, got: {png}"
        );

        let jpg = format!(
            "{:?}",
            guess_content_type(std::path::Path::new("photo.jpg"))
        );
        assert!(
            jpg.contains("image") && jpg.contains("jpeg"),
            "expected image/jpeg, got: {jpg}"
        );

        let unknown = format!(
            "{:?}",
            guess_content_type(std::path::Path::new("file.xyzunkn"))
        );
        assert!(
            unknown.contains("octet-stream"),
            "expected application/octet-stream, got: {unknown}"
        );
    }
}
