use std::path::Path;

use serde::Serialize;
use tauri::Manager;

// ── IPC response types ────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct SourceSlot {
    pub namespace: String,
    pub is_primary: bool,
    pub has_join: bool,
    pub join_keys: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct TemplateInfo {
    pub path: String,
    pub sources: Vec<SourceSlot>,
}

#[derive(Debug, Serialize)]
pub struct CsvPreviewResult {
    pub detected_separator: String,
    pub headers: Vec<String>,
    pub preview_rows: Vec<Vec<String>>,
    pub total_rows: usize,
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
            }
        })
        .collect();

    sources.sort_by(|a, b| {
        b.is_primary
            .cmp(&a.is_primary)
            .then(a.namespace.cmp(&b.namespace))
    });

    Ok(TemplateInfo { path, sources })
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

    let sep_byte: u8 = match separator.as_deref() {
        Some("\\t") | Some("\t") => b'\t',
        Some(s) if !s.is_empty() => s.as_bytes()[0],
        _ => {
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

// ── Helpers ───────────────────────────────────────────────────────────────────

fn smtp_profiles_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let config_dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    Ok(config_dir.join("smtp_profiles.json"))
}
