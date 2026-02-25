import { invoke } from "@tauri-apps/api/core";

// ── Types (mirror Rust command.rs structs) ────────────────────────────────────

export interface SourceSlot {
	namespace: string;
	is_primary: boolean;
	has_join: boolean;
	join_keys: string[];
}

export interface TemplateFields {
	to: string;
	cc: string | null;
	bcc: string | null;
	subject: string;
	body: string;
	attachments: string | null;
	/** "markdown" | "html" | "text" | null (null = absent from YAML = default markdown) */
	body_format: "markdown" | "html" | "text" | null;
	stylesheet: string | null;
	style: string | null;
}

export interface TemplateInfo {
	path: string;
	sources: SourceSlot[];
	fields: TemplateFields;
}

export interface CsvPreviewResult {
	/** Detected or overridden separator: "," ";" "|" "\t" */
	detected_separator: string;
	headers: string[];
	/** Up to 5 data rows, values in header order */
	preview_rows: string[][];
	total_rows: number;
}

export interface SmtpProfile {
	name: string;
	host: string;
	port: number;
	encryption: "none" | "start_tls" | "tls";
	from: string;
	parallelism: number;
}

export interface SourceFileSpec {
	namespace: string;
	path: string;
	separator: string | null;
	encoding: string | null;
}

export interface PreviewEntryStatus {
	entry_index: number;
	is_valid: boolean;
	issues: string[];
}

export interface PreviewValidation {
	entry_count: number;
	entries: PreviewEntryStatus[];
}

export interface PreviewRenderedEmail {
	to: string;
	cc: string | null;
	bcc: string | null;
	subject: string;
	html_body: string | null;
	text_body: string;
	attachments: string[];
}

export interface SendResultEntry {
	entry_index: number;
	recipient: string;
	success: boolean;
	error: string | null;
}

export interface SendBatchReport {
	total: number;
	success_count: number;
	failure_count: number;
	results: SendResultEntry[];
}

export interface SendProgressEvent {
	completed: number;
	total: number;
	entry_index: number;
	recipient: string;
	success: boolean;
	error: string | null;
}

// ── Command wrappers ──────────────────────────────────────────────────────────

export const parseTemplate = (path: string): Promise<TemplateInfo> =>
	invoke("parse_template_cmd", { path });

export const previewCsv = (
	path: string,
	separator?: string | null,
	encoding?: string | null,
): Promise<CsvPreviewResult> =>
	invoke("preview_csv", {
		path,
		separator: separator ?? null,
		encoding: encoding ?? null,
	});

export const getSmtpProfiles = (): Promise<SmtpProfile[]> =>
	invoke("get_smtp_profiles");

export const saveSmtpProfiles = (profiles: SmtpProfile[]): Promise<void> =>
	invoke("save_smtp_profiles", { profiles });

export const storeSmtpCredential = (
	profileName: string,
	username: string,
	password: string,
): Promise<void> =>
	invoke("store_smtp_credential", { profileName, username, password });

export const deleteSmtpCredential = (profileName: string): Promise<void> =>
	invoke("delete_smtp_credential", { profileName });

export const testSmtpConnection = (
	profile: SmtpProfile,
	username: string,
	password: string,
): Promise<void> =>
	invoke("test_smtp_connection", { profile, username, password });

export const getDataFields = (path: string): Promise<string[]> =>
	invoke("get_data_fields", { path });

export const saveTemplate = (
	path: string,
	patch: TemplateFields,
): Promise<void> => invoke("save_template", { path, patch });

export const previewValidate = (
	templatePath: string,
	fields: TemplateFields,
	sourceFiles: SourceFileSpec[],
): Promise<PreviewValidation> =>
	invoke("preview_validate", { templatePath, fields, sourceFiles });

export const previewRenderEntry = (
	templatePath: string,
	fields: TemplateFields,
	sourceFiles: SourceFileSpec[],
	entryIndex: number,
): Promise<PreviewRenderedEmail> =>
	invoke("preview_render_entry", {
		templatePath,
		fields,
		sourceFiles,
		entryIndex,
	});

export const sendBatch = (
	templatePath: string,
	fields: TemplateFields,
	sourceFiles: SourceFileSpec[],
	profileName: string,
	entryIndices?: number[] | null,
): Promise<SendBatchReport> =>
	invoke("send_batch", {
		templatePath,
		fields,
		sourceFiles,
		profileName,
		entryIndices: entryIndices ?? null,
	});

export const cancelSend = (): Promise<void> => invoke("cancel_send");
