use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::Manager;

// ── IPC response types ────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct SourceSlot {
    pub namespace: String,
    pub is_primary: bool,
    pub has_join: bool,
    pub join_keys: Vec<String>,
    pub is_form: bool,
}

/// Editable template field values, returned on parse and sent back on save.
#[derive(Debug, Serialize)]
pub struct TemplateFields {
    pub to: String,
    pub cc: Option<String>,
    pub bcc: Option<String>,
    pub subject: String,
    pub body: String,
    pub attachments: Option<String>,
    /// "markdown" | "html" | "text" | null (null means absent from YAML = default markdown)
    pub body_format: Option<String>,
    pub stylesheet: Option<String>,
    pub style: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TemplateInfo {
    pub path: String,
    pub sources: Vec<SourceSlot>,
    pub fields: TemplateFields,
}

/// Source configuration sent from the frontend for new templates.
#[derive(Debug, Deserialize)]
pub struct SourceSpec {
    pub namespace: String,
    pub primary: Option<bool>,
    pub join: Option<HashMap<String, String>>,
    pub many: Option<bool>,
    pub form: Option<bool>,
}

/// Patch payload for save_template — mirrors TemplateFields.
#[derive(Debug, Deserialize)]
pub struct TemplatePatch {
    pub to: String,
    pub cc: Option<String>,
    pub bcc: Option<String>,
    pub subject: String,
    pub body: String,
    pub attachments: Option<String>,
    pub body_format: Option<String>,
    pub stylesheet: Option<String>,
    pub style: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CsvPreviewResult {
    pub detected_separator: String,
    pub headers: Vec<String>,
    pub preview_rows: Vec<Vec<String>>,
    pub total_rows: usize,
}

/// Source file specification sent from the frontend for preview commands.
#[derive(Debug, Deserialize)]
pub struct SourceFileSpec {
    pub namespace: String,
    pub path: String,
    pub separator: Option<String>,
    pub encoding: Option<String>,
    pub form_data: Option<HashMap<String, String>>,
}

/// Per-entry summary for the preview validation report.
#[derive(Debug, Serialize)]
pub struct PreviewEntryStatus {
    pub entry_index: usize,
    pub is_valid: bool,
    pub issues: Vec<String>,
}

/// Result of the preview_validate command.
#[derive(Debug, Serialize)]
pub struct PreviewValidation {
    pub entry_count: usize,
    pub entries: Vec<PreviewEntryStatus>,
}

/// One fully rendered email for preview.
#[derive(Debug, Serialize)]
pub struct PreviewRenderedEmail {
    pub to: String,
    pub cc: Option<String>,
    pub bcc: Option<String>,
    pub subject: String,
    pub html_body: Option<String>,
    pub text_body: String,
    pub attachments: Vec<String>,
}

/// IPC result for a single sent entry.
#[derive(Debug, Serialize)]
pub struct SendResultEntry {
    pub entry_index: usize,
    pub recipient: String,
    pub success: bool,
    pub error: Option<String>,
}

/// Final send report returned to the frontend.
#[derive(Debug, Serialize)]
pub struct SendBatchReport {
    pub total: usize,
    pub success_count: usize,
    pub failure_count: usize,
    pub results: Vec<SendResultEntry>,
}

/// Managed state that tracks an active batch send session.
pub struct SendState {
    pub cancel_flag: Arc<AtomicBool>,
    pub active: Arc<AtomicBool>,
}

impl Default for SendState {
    fn default() -> Self {
        Self {
            cancel_flag: Arc::new(AtomicBool::new(false)),
            active: Arc::new(AtomicBool::new(false)),
        }
    }
}

// ── Commands ──────────────────────────────────────────────────────────────────

/// Parse a template YAML file and return its source slot layout.
///
/// Sources are sorted primary-first, then alphabetically by namespace name.
#[tauri::command]
pub fn parse_template_cmd(path: String) -> Result<TemplateInfo, String> {
    let p = Path::new(&path);
    let template = mailnir_lib::template::parse_template(p).map_err(|e| e.to_string())?;
    mailnir_lib::template::validate_sources(&template).map_err(|e| e.to_string())?;

    let mut sources: Vec<SourceSlot> = template
        .sources
        .iter()
        .map(|(name, cfg)| {
            let join_keys = cfg
                .join
                .as_ref()
                .map(|j| {
                    let mut keys: Vec<String> = j.keys().cloned().collect();
                    keys.sort();
                    keys
                })
                .unwrap_or_default();
            SourceSlot {
                namespace: name.clone(),
                is_primary: cfg.primary == Some(true),
                has_join: cfg.join.is_some(),
                join_keys,
                is_form: cfg.form == Some(true),
            }
        })
        .collect();

    sources.sort_by(|a, b| {
        b.is_primary
            .cmp(&a.is_primary)
            .then(a.namespace.cmp(&b.namespace))
    });

    let fields = TemplateFields {
        to: template.to.clone(),
        cc: template.cc.clone(),
        bcc: template.bcc.clone(),
        subject: template.subject.clone(),
        body: template.body.clone(),
        attachments: template.attachments.clone(),
        body_format: template.body_format.as_ref().map(|f| match f {
            mailnir_lib::template::BodyFormat::Markdown => "markdown".to_string(),
            mailnir_lib::template::BodyFormat::Html => "html".to_string(),
            mailnir_lib::template::BodyFormat::Text => "text".to_string(),
        }),
        stylesheet: template.stylesheet.clone(),
        style: template.style.clone(),
    };

    Ok(TemplateInfo {
        path,
        sources,
        fields,
    })
}

/// Load a CSV file with optional separator/encoding overrides and return a preview.
///
/// When `separator` is `None`, the separator is auto-detected from the first line.
/// When `encoding` is `None`, UTF-8 is tried first, then Windows-1252.
/// Returns headers in CSV column order, up to 5 data rows, and total row count.
#[tauri::command]
pub fn preview_csv(
    path: String,
    separator: Option<String>,
    encoding: Option<String>,
) -> Result<CsvPreviewResult, String> {
    let p = Path::new(&path);
    let bytes = std::fs::read(p).map_err(|e| e.to_string())?;
    let content = mailnir_lib::data::csv::decode_bytes(&bytes, encoding.as_deref())
        .map_err(|e| e.to_string())?;

    let sep_byte: u8 = match parse_separator_override(separator.as_deref()) {
        Some(b) => b,
        None => {
            let first_line = content.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
            mailnir_lib::data::csv::detect_separator(first_line)
        }
    };

    let detected_separator = match sep_byte {
        b'\t' => "\\t".to_string(),
        other => (other as char).to_string(),
    };

    let mut reader = csv::ReaderBuilder::new()
        .delimiter(sep_byte)
        .has_headers(true)
        .from_reader(content.as_bytes());

    let headers: Vec<String> = reader
        .headers()
        .map_err(|e| e.to_string())?
        .iter()
        .map(String::from)
        .collect();

    let mut preview_rows: Vec<Vec<String>> = Vec::new();
    let mut total_rows = 0usize;

    for result in reader.records() {
        let record = result.map_err(|e| e.to_string())?;
        if total_rows < 5 {
            preview_rows.push(record.iter().map(String::from).collect());
        }
        total_rows += 1;
    }

    Ok(CsvPreviewResult {
        detected_separator,
        headers,
        preview_rows,
        total_rows,
    })
}

/// Load SMTP profiles from the app config directory.
///
/// Returns an empty list if the profiles file does not exist yet.
#[tauri::command]
pub fn get_smtp_profiles(
    app: tauri::AppHandle,
) -> Result<Vec<mailnir_lib::smtp::SmtpProfile>, String> {
    let path = smtp_profiles_path(&app)?;
    if !path.exists() {
        return Ok(vec![]);
    }
    mailnir_lib::smtp::load_profiles(&path).map_err(|e| e.to_string())
}

/// Persist SMTP profiles to the app config directory (overwrites).
#[tauri::command]
pub fn save_smtp_profiles(
    app: tauri::AppHandle,
    profiles: Vec<mailnir_lib::smtp::SmtpProfile>,
) -> Result<(), String> {
    let path = smtp_profiles_path(&app)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    mailnir_lib::smtp::save_profiles(&profiles, &path).map_err(|e| e.to_string())
}

/// Store SMTP credentials in the OS keychain for the given profile name.
#[tauri::command]
pub fn store_smtp_credential(
    profile_name: String,
    username: String,
    password: String,
) -> Result<(), String> {
    mailnir_lib::smtp::store_credential(&profile_name, &username, &password)
        .map_err(|e| e.to_string())
}

/// Remove SMTP credentials from the OS keychain for the given profile name.
#[tauri::command]
pub fn delete_smtp_credential(profile_name: String) -> Result<(), String> {
    mailnir_lib::smtp::delete_credential(&profile_name).map_err(|e| e.to_string())
}

/// Verify that an SMTP server is reachable using the supplied credentials.
///
/// Credentials are passed directly rather than retrieved from the keychain so
/// the user can test before saving.
#[tauri::command]
pub async fn test_smtp_connection(
    profile: mailnir_lib::smtp::SmtpProfile,
    username: String,
    password: String,
) -> Result<(), String> {
    let creds = mailnir_lib::smtp::SmtpCredentials { username, password };
    mailnir_lib::smtp::test_connection(&profile, &creds)
        .await
        .map_err(|e| e.to_string())
}

/// Extract field names (keys of the first object) from any supported data file.
///
/// Returns a sorted list of key names. Returns an empty list if the file is
/// empty, has no objects, or the format is not recognised — never errors on
/// unexpected structure.
#[tauri::command]
pub fn get_data_fields(path: String) -> Result<Vec<String>, String> {
    let value = mailnir_lib::data::load_file(Path::new(&path)).map_err(|e| e.to_string())?;
    let mut keys: Vec<String> = value
        .as_array()
        .and_then(|a| a.first())
        .and_then(|v| v.as_object())
        .map(|o| o.keys().cloned().collect())
        .unwrap_or_default();
    keys.sort();
    Ok(keys)
}

/// Infer form field names for a given namespace from template variable references.
///
/// Scans all template string fields for `{{namespace.field}}` patterns and
/// returns a sorted, deduplicated list of field names. Uses the current editor
/// state (TemplatePatch) so fields update as the user edits the template.
#[tauri::command]
pub fn get_form_fields(
    template_path: String,
    fields: TemplatePatch,
    namespace: String,
) -> Result<Vec<String>, String> {
    let path = Path::new(&template_path);
    let mut template = mailnir_lib::template::parse_template(path).map_err(|e| e.to_string())?;
    apply_patch(&mut template, &fields);
    Ok(mailnir_lib::template::infer_form_fields(
        &template, &namespace,
    ))
}

/// Overwrite the editable fields of a template YAML file, preserving `sources`
/// and any other keys not managed by the editor.
///
/// Uses a serde_yaml::Value read-modify-write so the sources block is
/// preserved verbatim. YAML anchors are expanded on write (acceptable trade-off
/// for Phase 7).
#[tauri::command]
pub fn save_template(path: String, patch: TemplatePatch) -> Result<(), String> {
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut doc: serde_yaml::Value = serde_yaml::from_str(&content).map_err(|e| e.to_string())?;
    let map = doc
        .as_mapping_mut()
        .ok_or_else(|| "template root is not a YAML mapping".to_string())?;

    // Helper: upsert a string key in the mapping.
    macro_rules! set_str {
        ($key:expr, $val:expr) => {
            map.insert(
                serde_yaml::Value::String($key.to_string()),
                serde_yaml::Value::String($val),
            );
        };
    }
    // Helper: set an optional key; remove the key when the value is None or empty.
    macro_rules! set_opt {
        ($key:expr, $val:expr) => {
            match $val {
                Some(v) if !v.is_empty() => {
                    map.insert(
                        serde_yaml::Value::String($key.to_string()),
                        serde_yaml::Value::String(v),
                    );
                }
                _ => {
                    map.remove(serde_yaml::Value::String($key.to_string()));
                }
            }
        };
    }

    set_str!("to", patch.to);
    set_str!("subject", patch.subject);
    set_str!("body", patch.body);
    set_opt!("cc", patch.cc);
    set_opt!("bcc", patch.bcc);
    set_opt!("attachments", patch.attachments);
    set_opt!("stylesheet", patch.stylesheet);
    set_opt!("style", patch.style);

    // body_format: write "markdown"/"html"/"text" if present; remove key otherwise.
    match patch.body_format.as_deref() {
        Some("markdown") | Some("html") | Some("text") => {
            map.insert(
                serde_yaml::Value::String("body_format".to_string()),
                serde_yaml::Value::String(patch.body_format.unwrap()),
            );
        }
        _ => {
            map.remove(serde_yaml::Value::String("body_format".to_string()));
        }
    }

    let yaml_out = serde_yaml::to_string(&doc).map_err(|e| e.to_string())?;
    std::fs::write(&path, yaml_out).map_err(|e| e.to_string())?;
    Ok(())
}

/// Create a new template YAML file from scratch.
///
/// Unlike `save_template` (which patches an existing file), this writes a
/// complete template including the `sources` block. Used for the "New Template"
/// flow where no file exists on disk yet.
#[tauri::command]
pub fn create_template(
    path: String,
    sources: Vec<SourceSpec>,
    patch: TemplatePatch,
) -> Result<(), String> {
    let mut doc = serde_yaml::Mapping::new();

    // Build sources mapping.
    let mut sources_map = serde_yaml::Mapping::new();
    for spec in &sources {
        let mut source_cfg = serde_yaml::Mapping::new();
        if spec.primary == Some(true) {
            source_cfg.insert(
                serde_yaml::Value::String("primary".into()),
                serde_yaml::Value::Bool(true),
            );
        }
        if let Some(join) = &spec.join {
            let mut join_map = serde_yaml::Mapping::new();
            for (k, v) in join {
                join_map.insert(
                    serde_yaml::Value::String(k.clone()),
                    serde_yaml::Value::String(v.clone()),
                );
            }
            source_cfg.insert(
                serde_yaml::Value::String("join".into()),
                serde_yaml::Value::Mapping(join_map),
            );
        }
        if spec.many == Some(true) {
            source_cfg.insert(
                serde_yaml::Value::String("many".into()),
                serde_yaml::Value::Bool(true),
            );
        }
        if spec.form == Some(true) {
            source_cfg.insert(
                serde_yaml::Value::String("form".into()),
                serde_yaml::Value::Bool(true),
            );
        }
        sources_map.insert(
            serde_yaml::Value::String(spec.namespace.clone()),
            serde_yaml::Value::Mapping(source_cfg),
        );
    }
    doc.insert(
        serde_yaml::Value::String("sources".into()),
        serde_yaml::Value::Mapping(sources_map),
    );

    // Required fields.
    doc.insert(
        serde_yaml::Value::String("to".into()),
        serde_yaml::Value::String(patch.to),
    );
    doc.insert(
        serde_yaml::Value::String("subject".into()),
        serde_yaml::Value::String(patch.subject),
    );
    doc.insert(
        serde_yaml::Value::String("body".into()),
        serde_yaml::Value::String(patch.body),
    );

    // Optional fields.
    macro_rules! set_opt {
        ($key:expr, $val:expr) => {
            if let Some(v) = $val {
                if !v.is_empty() {
                    doc.insert(
                        serde_yaml::Value::String($key.into()),
                        serde_yaml::Value::String(v),
                    );
                }
            }
        };
    }
    set_opt!("cc", patch.cc);
    set_opt!("bcc", patch.bcc);
    set_opt!("attachments", patch.attachments);
    set_opt!("stylesheet", patch.stylesheet);
    set_opt!("style", patch.style);

    if let Some(bf) = &patch.body_format {
        if matches!(bf.as_str(), "markdown" | "html" | "text") {
            doc.insert(
                serde_yaml::Value::String("body_format".into()),
                serde_yaml::Value::String(bf.clone()),
            );
        }
    }

    let yaml_out =
        serde_yaml::to_string(&serde_yaml::Value::Mapping(doc)).map_err(|e| e.to_string())?;
    std::fs::write(&path, yaml_out).map_err(|e| e.to_string())?;
    Ok(())
}

/// Validate all entries for a template with the given field overrides and sources.
///
/// Returns per-entry validation status without saving anything to disk.
#[tauri::command]
pub fn preview_validate(
    template_path: String,
    fields: TemplatePatch,
    source_files: Vec<SourceFileSpec>,
) -> Result<PreviewValidation, String> {
    let path = Path::new(&template_path);
    let mut template = mailnir_lib::template::parse_template(path).map_err(|e| e.to_string())?;
    apply_patch(&mut template, &fields);

    let template_dir = path.parent().unwrap_or(Path::new("."));
    let sources = load_sources(&source_files)?;

    let report = mailnir_lib::validate::validate_all(&template, &sources, template_dir)
        .map_err(|e| e.to_string())?;

    let entries: Vec<PreviewEntryStatus> = report
        .entries
        .iter()
        .map(|entry| PreviewEntryStatus {
            entry_index: entry.entry_index,
            is_valid: entry.is_valid(),
            issues: entry.issues.iter().map(format_issue).collect(),
        })
        .collect();

    Ok(PreviewValidation {
        entry_count: entries.len(),
        entries,
    })
}

/// Render a single email entry for preview with the given field overrides.
///
/// Returns the fully rendered email without saving anything to disk.
#[tauri::command]
pub fn preview_render_entry(
    template_path: String,
    fields: TemplatePatch,
    source_files: Vec<SourceFileSpec>,
    entry_index: usize,
) -> Result<PreviewRenderedEmail, String> {
    let path = Path::new(&template_path);
    let mut template = mailnir_lib::template::parse_template(path).map_err(|e| e.to_string())?;
    apply_patch(&mut template, &fields);

    let template_dir = path.parent().unwrap_or(Path::new("."));
    let sources = load_sources(&source_files)?;

    let contexts = mailnir_lib::join::build_contexts_lenient(&template, &sources)
        .map_err(|e| e.to_string())?;

    let context = contexts
        .into_iter()
        .nth(entry_index)
        .ok_or_else(|| format!("entry index {entry_index} out of range"))?
        .map_err(|e| e.to_string())?;

    let rendered = mailnir_lib::render::render_context(&template, &context, template_dir)
        .map_err(|e| e.to_string())?;

    Ok(PreviewRenderedEmail {
        to: rendered.to,
        cc: rendered.cc,
        bcc: rendered.bcc,
        subject: rendered.subject,
        html_body: rendered.html_body,
        text_body: rendered.text_body,
        attachments: rendered
            .attachments
            .iter()
            .map(|p| p.display().to_string())
            .collect(),
    })
}

/// Send a batch of emails using the full pipeline: parse → join → render → send.
///
/// Emits `send-progress` events as each email completes. Supports cancellation
/// via the managed [`SendState`] and retry of a subset via `entry_indices`.
#[tauri::command]
pub async fn send_batch(
    app: tauri::AppHandle,
    send_state: tauri::State<'_, SendState>,
    template_path: String,
    fields: TemplatePatch,
    source_files: Vec<SourceFileSpec>,
    profile_name: String,
    entry_indices: Option<Vec<usize>>,
) -> Result<SendBatchReport, String> {
    // Guard: prevent concurrent sends.
    if send_state.active.swap(true, Ordering::SeqCst) {
        return Err("A send operation is already in progress".to_string());
    }
    send_state.cancel_flag.store(false, Ordering::SeqCst);

    let result = send_batch_inner(
        &app,
        &send_state,
        &template_path,
        &fields,
        &source_files,
        &profile_name,
        entry_indices.as_deref(),
    )
    .await;

    send_state.active.store(false, Ordering::SeqCst);
    result
}

async fn send_batch_inner(
    app: &tauri::AppHandle,
    send_state: &SendState,
    template_path: &str,
    fields: &TemplatePatch,
    source_files: &[SourceFileSpec],
    profile_name: &str,
    entry_indices: Option<&[usize]>,
) -> Result<SendBatchReport, String> {
    use tauri::Emitter;

    // 1. Parse template + apply field overrides.
    let path = Path::new(template_path);
    let mut template = mailnir_lib::template::parse_template(path).map_err(|e| e.to_string())?;
    apply_patch(&mut template, fields);
    let template_dir = path.parent().unwrap_or(Path::new("."));

    // 2. Load sources.
    let sources = load_sources(source_files)?;

    // 3. Build contexts (lenient — join failures become per-entry errors).
    let all_contexts = mailnir_lib::join::build_contexts_lenient(&template, &sources)
        .map_err(|e| e.to_string())?;

    // 4. Determine which entries to send.
    let indices: Vec<usize> = match entry_indices {
        Some(subset) => subset.to_vec(),
        None => (0..all_contexts.len()).collect(),
    };

    // 5. Render emails for the selected entries.
    let mut emails: Vec<mailnir_lib::render::RenderedEmail> = Vec::with_capacity(indices.len());
    let mut index_map: Vec<usize> = Vec::with_capacity(indices.len());
    let mut pre_send_failures: Vec<SendResultEntry> = Vec::new();

    for &idx in &indices {
        let ctx_result = all_contexts
            .get(idx)
            .ok_or_else(|| format!("entry index {idx} out of range"))?;

        match ctx_result {
            Err(e) => {
                pre_send_failures.push(SendResultEntry {
                    entry_index: idx,
                    recipient: String::new(),
                    success: false,
                    error: Some(e.to_string()),
                });
            }
            Ok(context) => {
                match mailnir_lib::render::render_context(&template, context, template_dir) {
                    Err(e) => {
                        pre_send_failures.push(SendResultEntry {
                            entry_index: idx,
                            recipient: String::new(),
                            success: false,
                            error: Some(e.to_string()),
                        });
                    }
                    Ok(rendered) => {
                        index_map.push(idx);
                        emails.push(rendered);
                    }
                }
            }
        }
    }

    // 6. Load SMTP profile and credentials.
    let profiles_path = smtp_profiles_path(app)?;
    let profiles = mailnir_lib::smtp::load_profiles(&profiles_path).map_err(|e| e.to_string())?;
    let profile = profiles
        .iter()
        .find(|p| p.name == profile_name)
        .ok_or_else(|| format!("profile '{profile_name}' not found"))?
        .clone();
    let credentials =
        mailnir_lib::smtp::retrieve_credential(profile_name).map_err(|e| e.to_string())?;

    // 7. Send with progress events.
    let cancel = send_state.cancel_flag.clone();
    let app_handle = app.clone();
    let total = indices.len();

    let report = mailnir_lib::smtp::send_all_with_progress(
        &emails,
        &profile,
        &credentials,
        Some(cancel),
        Some(Arc::new(move |progress| {
            let _ = app_handle.emit("send-progress", &progress);
        })),
    )
    .await;

    // 8. Map send results back to original entry indices and merge with pre-send failures.
    let mut results: Vec<SendResultEntry> = pre_send_failures;
    for r in &report.results {
        let original_idx = index_map
            .get(r.entry_index)
            .copied()
            .unwrap_or(r.entry_index);
        results.push(SendResultEntry {
            entry_index: original_idx,
            recipient: r.recipient.clone(),
            success: r.success,
            error: r.error.clone(),
        });
    }

    let success_count = results.iter().filter(|r| r.success).count();
    let failure_count = results.iter().filter(|r| !r.success).count();

    Ok(SendBatchReport {
        total,
        success_count,
        failure_count,
        results,
    })
}

/// Cancel an in-progress batch send. In-flight emails will complete, but no new
/// emails will be sent.
#[tauri::command]
pub fn cancel_send(send_state: tauri::State<'_, SendState>) -> Result<(), String> {
    send_state.cancel_flag.store(true, Ordering::SeqCst);
    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn smtp_profiles_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let config_dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    Ok(config_dir.join("smtp_profiles.json"))
}

/// Convert a frontend separator string to a byte for CSV parsing.
fn parse_separator_override(sep: Option<&str>) -> Option<u8> {
    match sep {
        Some("\\t") | Some("\t") => Some(b'\t'),
        Some(s) if !s.is_empty() => Some(s.as_bytes()[0]),
        _ => None,
    }
}

/// Load all source data files (or form data) into a namespace→Value map.
fn load_sources(specs: &[SourceFileSpec]) -> Result<HashMap<String, Value>, String> {
    let mut sources = HashMap::new();
    for spec in specs {
        let value = if let Some(form_data) = &spec.form_data {
            let obj: serde_json::Map<String, Value> = form_data
                .iter()
                .map(|(k, v)| (k.clone(), Value::String(v.clone())))
                .collect();
            Value::Array(vec![Value::Object(obj)])
        } else {
            let path = Path::new(&spec.path);
            let is_csv = path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("csv"));
            if is_csv {
                let opts = mailnir_lib::data::CsvOptions {
                    separator: parse_separator_override(spec.separator.as_deref()),
                    encoding: spec.encoding.clone(),
                };
                mailnir_lib::data::load_file_csv(path, &opts)
            } else {
                mailnir_lib::data::load_file(path)
            }
            .map_err(|e| e.to_string())?
        };
        sources.insert(spec.namespace.clone(), value);
    }
    Ok(sources)
}

/// Overlay current editor field values onto a parsed template.
fn apply_patch(template: &mut mailnir_lib::template::Template, patch: &TemplatePatch) {
    template.to = patch.to.clone();
    template.cc = patch.cc.clone();
    template.bcc = patch.bcc.clone();
    template.subject = patch.subject.clone();
    template.body = patch.body.clone();
    template.attachments = patch.attachments.clone();
    template.stylesheet = patch.stylesheet.clone();
    template.style = patch.style.clone();
    template.body_format = match patch.body_format.as_deref() {
        Some("html") => Some(mailnir_lib::template::BodyFormat::Html),
        Some("text") => Some(mailnir_lib::template::BodyFormat::Text),
        Some("markdown") => Some(mailnir_lib::template::BodyFormat::Markdown),
        _ => None,
    };
}

/// Convert a ValidationIssue to a human-readable string.
fn format_issue(issue: &mailnir_lib::ValidationIssue) -> String {
    use mailnir_lib::validate::JoinFailureDetail;
    use mailnir_lib::ValidationIssue;
    match issue {
        ValidationIssue::UnresolvedVariable { field, reason } => {
            format!("Unresolved variable in {field}: {reason}")
        }
        ValidationIssue::JoinFailure { namespace, detail } => match detail {
            JoinFailureDetail::MissingMatch => {
                format!("Join '{namespace}': no match found")
            }
            JoinFailureDetail::AmbiguousMatch { match_count } => {
                format!("Join '{namespace}': {match_count} matches (expected 1)")
            }
        },
        ValidationIssue::InvalidEmail { field, value } => {
            format!("Invalid email in {field}: \"{value}\"")
        }
        ValidationIssue::AttachmentNotFound { path } => {
            format!("Attachment not found: {}", path.display())
        }
        ValidationIssue::RequiredFieldEmpty { field } => {
            format!("Required field empty: {field}")
        }
        ValidationIssue::StylesheetNotFound { path } => {
            format!("Stylesheet not found: {}", path.display())
        }
        ValidationIssue::CssInlineError { reason } => {
            format!("CSS inlining error: {reason}")
        }
    }
}
