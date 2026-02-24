import { open } from "@tauri-apps/plugin-dialog";
import { useEffect, useState } from "react";
import DataPanel, { type SourceState } from "@/components/DataPanel/index";
import Preview from "@/components/Preview/index";
import SmtpSettingsDialog from "@/components/SmtpSettings/SmtpSettingsDialog";
import StatusBar from "@/components/StatusBar/index";
import TemplateEditor from "@/components/TemplateEditor/index";
import { Button } from "@/components/ui/button";
import { TooltipProvider } from "@/components/ui/tooltip";
import {
	getSmtpProfiles,
	parseTemplate,
	previewCsv,
	type SmtpProfile,
	saveSmtpProfiles,
	type TemplateInfo,
} from "@/lib/ipc";

function isCsvPath(path: string): boolean {
	return path.toLowerCase().endsWith(".csv");
}

export default function App() {
	// ── Template state ──────────────────────────────────────────────────────────
	const [templatePath, setTemplatePath] = useState<string | null>(null);
	const [templateInfo, setTemplateInfo] = useState<TemplateInfo | null>(null);
	const [templateError, setTemplateError] = useState<string | null>(null);

	// ── Source loading state ────────────────────────────────────────────────────
	const [sourcesState, setSourcesState] = useState<Record<string, SourceState>>(
		{},
	);

	// ── SMTP state ──────────────────────────────────────────────────────────────
	const [smtpProfiles, setSmtpProfiles] = useState<SmtpProfile[]>([]);
	const [selectedProfileName, setSelectedProfileName] = useState<string | null>(
		null,
	);
	const [smtpDialogOpen, setSmtpDialogOpen] = useState(false);

	// ── Derived ─────────────────────────────────────────────────────────────────
	const allSourcesLoaded =
		templateInfo?.sources.every(
			(slot) =>
				sourcesState[slot.namespace] != null &&
				!sourcesState[slot.namespace].error,
		) === true;

	// Load SMTP profiles on mount
	useEffect(() => {
		getSmtpProfiles()
			.then(setSmtpProfiles)
			.catch(() => {});
	}, []);

	// ── Handlers ────────────────────────────────────────────────────────────────

	const handleOpenTemplate = async () => {
		const selected = await open({
			multiple: false,
			filters: [{ name: "Mailnir Template", extensions: ["yml", "yaml"] }],
		});
		if (typeof selected !== "string") return;

		setTemplateError(null);
		try {
			const info = await parseTemplate(selected);
			setTemplatePath(selected);
			setTemplateInfo(info);
			setSourcesState({});
		} catch (err) {
			setTemplateError(String(err));
		}
	};

	const handleFileSelect = async (namespace: string, path: string) => {
		if (isCsvPath(path)) {
			// Load CSV with auto-detection
			try {
				const preview = await previewCsv(path, null, null);
				setSourcesState((prev) => ({
					...prev,
					[namespace]: {
						path,
						csvPreview: preview,
						separatorOverride: null,
						encodingOverride: null,
						error: null,
					},
				}));
			} catch (err) {
				setSourcesState((prev) => ({
					...prev,
					[namespace]: {
						path,
						csvPreview: null,
						separatorOverride: null,
						encodingOverride: null,
						error: String(err),
					},
				}));
			}
		} else {
			// Non-CSV: just record path as loaded (data will be read at join/send time)
			setSourcesState((prev) => ({
				...prev,
				[namespace]: {
					path,
					csvPreview: null,
					separatorOverride: null,
					encodingOverride: null,
					error: null,
				},
			}));
		}
	};

	const handleSeparatorChange = async (namespace: string, sep: string) => {
		const state = sourcesState[namespace];
		if (!state) return;

		const separatorOverride = sep === "auto" ? null : sep;
		const encodingOverride = state.encodingOverride;

		try {
			const preview = await previewCsv(
				state.path,
				separatorOverride,
				encodingOverride,
			);
			setSourcesState((prev) => ({
				...prev,
				[namespace]: {
					...prev[namespace],
					csvPreview: preview,
					separatorOverride,
					error: null,
				},
			}));
		} catch (err) {
			setSourcesState((prev) => ({
				...prev,
				[namespace]: {
					...prev[namespace],
					separatorOverride,
					error: String(err),
				},
			}));
		}
	};

	const handleEncodingChange = async (namespace: string, enc: string) => {
		const state = sourcesState[namespace];
		if (!state) return;

		const encodingOverride = enc === "auto" ? null : enc;
		const separatorOverride = state.separatorOverride;

		try {
			const preview = await previewCsv(
				state.path,
				separatorOverride,
				encodingOverride,
			);
			setSourcesState((prev) => ({
				...prev,
				[namespace]: {
					...prev[namespace],
					csvPreview: preview,
					encodingOverride,
					error: null,
				},
			}));
		} catch (err) {
			setSourcesState((prev) => ({
				...prev,
				[namespace]: {
					...prev[namespace],
					encodingOverride,
					error: String(err),
				},
			}));
		}
	};

	const handleSaveProfiles = async (profiles: SmtpProfile[]) => {
		await saveSmtpProfiles(profiles);
		setSmtpProfiles(profiles);
	};

	// ── Render ──────────────────────────────────────────────────────────────────

	return (
		<TooltipProvider>
			<div className="flex h-screen flex-col overflow-hidden bg-background">
				{/* Toolbar */}
				<div className="flex shrink-0 items-center gap-3 border-b px-4 py-2">
					<Button size="sm" variant="outline" onClick={handleOpenTemplate}>
						Open Template
					</Button>
					{templatePath && (
						<span className="max-w-md truncate text-xs text-muted-foreground">
							{templatePath}
						</span>
					)}
					{templateError && (
						<span className="text-xs text-destructive">{templateError}</span>
					)}
				</div>

				{/* Main panels */}
				<div className="flex min-h-0 flex-1">
					{/* Data panel */}
					<div className="w-72 shrink-0 border-r">
						<DataPanel
							templateInfo={templateInfo}
							sourcesState={sourcesState}
							onFileSelect={handleFileSelect}
							onSeparatorChange={handleSeparatorChange}
							onEncodingChange={handleEncodingChange}
						/>
					</div>

					{/* Template editor (placeholder) */}
					<div className="min-w-0 flex-1 border-r">
						<TemplateEditor templatePath={templatePath} />
					</div>

					{/* Preview (placeholder) */}
					<div className="w-96 shrink-0">
						<Preview />
					</div>
				</div>

				{/* Status bar */}
				<StatusBar
					profiles={smtpProfiles}
					selectedProfileName={selectedProfileName}
					onProfileChange={setSelectedProfileName}
					onSmtpSettings={() => setSmtpDialogOpen(true)}
					allSourcesLoaded={allSourcesLoaded}
					onPreview={() => {
						/* Phase 8 */
					}}
					onSend={() => {
						/* Phase 9 */
					}}
				/>

				<SmtpSettingsDialog
					open={smtpDialogOpen}
					onOpenChange={setSmtpDialogOpen}
					profiles={smtpProfiles}
					onSave={handleSaveProfiles}
				/>
			</div>
		</TooltipProvider>
	);
}
