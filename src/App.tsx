import { open } from "@tauri-apps/plugin-dialog";
import { useEffect, useMemo, useState } from "react";
import DataPanel, { type SourceState } from "@/components/DataPanel/index";
import Preview from "@/components/Preview/index";
import SmtpSettingsDialog from "@/components/SmtpSettings/SmtpSettingsDialog";
import StatusBar from "@/components/StatusBar/index";
import TemplateEditor from "@/components/TemplateEditor/index";
import { Button } from "@/components/ui/button";
import {
	ResizableHandle,
	ResizablePanel,
	ResizablePanelGroup,
} from "@/components/ui/resizable";
import { TooltipProvider } from "@/components/ui/tooltip";
import {
	getDataFields,
	getSmtpProfiles,
	parseTemplate,
	previewCsv,
	type SmtpProfile,
	saveSmtpProfiles,
	saveTemplate,
	type TemplateFields,
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
	const [templateFields, setTemplateFields] = useState<TemplateFields | null>(
		null,
	);
	/** Incremented on every successful template open to force editor remount. */
	const [templateLoadId, setTemplateLoadId] = useState(0);

	// ── Source loading state ────────────────────────────────────────────────────
	const [sourcesState, setSourcesState] = useState<Record<string, SourceState>>(
		{},
	);
	/** Field names per namespace, populated when a data file is loaded. */
	const [namespaceFields, setNamespaceFields] = useState<
		Record<string, string[]>
	>({});

	// ── Save state ──────────────────────────────────────────────────────────────
	const [saveStatus, setSaveStatus] = useState<
		"idle" | "saving" | "saved" | "error"
	>("idle");
	const [saveError, setSaveError] = useState<string | null>(null);

	// ── SMTP state ──────────────────────────────────────────────────────────────
	const [smtpProfiles, setSmtpProfiles] = useState<SmtpProfile[]>([]);
	const [selectedProfileName, setSelectedProfileName] = useState<string | null>(
		null,
	);
	const [smtpDialogOpen, setSmtpDialogOpen] = useState(false);

	// ── Derived ─────────────────────────────────────────────────────────────────
	const namespaces = useMemo(
		() => templateInfo?.sources.map((s) => s.namespace) ?? [],
		[templateInfo],
	);

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
			setTemplateFields(info.fields);
			setTemplateLoadId((n) => n + 1);
			setSourcesState({});
			setNamespaceFields({});
			setSaveStatus("idle");
			setSaveError(null);
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
				setNamespaceFields((prev) => ({
					...prev,
					[namespace]: preview.headers,
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
			// Non-CSV: record path and fetch field names for autocomplete
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
			try {
				const fields = await getDataFields(path);
				setNamespaceFields((prev) => ({ ...prev, [namespace]: fields }));
			} catch {
				// Non-critical: autocomplete just won't offer field names
				setNamespaceFields((prev) => ({ ...prev, [namespace]: [] }));
			}
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

	const handleFieldChange = (
		field: keyof TemplateFields,
		value: string | null,
	) => {
		setTemplateFields((prev) => (prev ? { ...prev, [field]: value } : prev));
		setSaveStatus("idle");
	};

	const handleSaveTemplate = async () => {
		if (!templatePath || !templateFields) return;
		setSaveStatus("saving");
		setSaveError(null);
		try {
			await saveTemplate(templatePath, templateFields);
			setSaveStatus("saved");
			setTimeout(() => setSaveStatus("idle"), 2000);
		} catch (err) {
			setSaveError(String(err));
			setSaveStatus("error");
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
				<ResizablePanelGroup direction="horizontal" className="min-h-0 flex-1">
					{/* Data panel */}
					<ResizablePanel defaultSize={22} minSize={10}>
						<DataPanel
							templateInfo={templateInfo}
							sourcesState={sourcesState}
							onFileSelect={handleFileSelect}
							onSeparatorChange={handleSeparatorChange}
							onEncodingChange={handleEncodingChange}
						/>
					</ResizablePanel>

					<ResizableHandle withHandle />

					{/* Template editor */}
					<ResizablePanel defaultSize={48} minSize={25}>
						<TemplateEditor
							templatePath={templatePath}
							templateFields={templateFields}
							loadId={templateLoadId}
							namespaces={namespaces}
							namespaceFields={namespaceFields}
							saveStatus={saveStatus}
							saveError={saveError}
							onFieldChange={handleFieldChange}
							onSave={handleSaveTemplate}
						/>
					</ResizablePanel>

					<ResizableHandle withHandle />

					{/* Preview (placeholder) */}
					<ResizablePanel defaultSize={30} minSize={10}>
						<Preview />
					</ResizablePanel>
				</ResizablePanelGroup>

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
