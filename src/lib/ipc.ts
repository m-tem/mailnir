import { invoke } from "@tauri-apps/api/core";

// ── Types (mirror Rust command.rs structs) ────────────────────────────────────

export interface SourceSlot {
	namespace: string;
	is_primary: boolean;
	has_join: boolean;
	join_keys: string[];
}

export interface TemplateInfo {
	path: string;
	sources: SourceSlot[];
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
