import { open } from "@tauri-apps/plugin-dialog";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import DataPanel, { type SourceState } from "@/components/DataPanel/index";
import Preview from "@/components/Preview/index";
import SendDialog from "@/components/SendDialog/SendDialog";
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
	getFormFields,
	getSmtpProfiles,
	type PreviewRenderedEmail,
	type PreviewValidation,
	parseTemplate,
	previewCsv,
	previewRenderEntry,
	previewValidate,
	type SmtpProfile,
	type SourceFileSpec,
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
	const [sendDialogOpen, setSendDialogOpen] = useState(false);
	const [previewVisible, setPreviewVisible] = useState(true);

	// ── Preview state ──────────────────────────────────────────────────────────
	const [previewValidation, setPreviewValidation] =
		useState<PreviewValidation | null>(null);
	const [previewCurrentIndex, setPreviewCurrentIndex] = useState(0);
	const [previewRendered, setPreviewRendered] =
		useState<PreviewRenderedEmail | null>(null);
	const [previewLoading, setPreviewLoading] = useState(false);
	const [previewError, setPreviewError] = useState<string | null>(null);

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

	// ── Preview helpers ─────────────────────────────────────────────────────────

	const buildSourceFileSpecs = useCallback((): SourceFileSpec[] => {
		if (!templateInfo) return [];
		return templateInfo.sources.map((slot) => {
			const state = sourcesState[slot.namespace];
			return {
				namespace: slot.namespace,
				path: state?.path ?? "",
				separator: state?.separatorOverride ?? null,
				encoding: state?.encodingOverride ?? null,
				form_data: state?.formValues ?? null,
			};
		});
	}, [templateInfo, sourcesState]);

	// Track current index in a ref so the effect can read the latest value
	// without re-triggering on navigation.
	const previewIndexRef = useRef(previewCurrentIndex);
	previewIndexRef.current = previewCurrentIndex;

	// Auto-refresh preview when inputs change (debounced).
	useEffect(() => {
		if (!templatePath || !templateFields || !allSourcesLoaded) {
			setPreviewValidation(null);
			setPreviewRendered(null);
			return;
		}

		const controller = new AbortController();
		const timer = setTimeout(async () => {
			setPreviewLoading(true);
			setPreviewError(null);
			try {
				const specs = buildSourceFileSpecs();
				const validation = await previewValidate(
					templatePath,
					templateFields,
					specs,
				);
				if (controller.signal.aborted) return;
				setPreviewValidation(validation);

				if (validation.entry_count > 0) {
					const idx = Math.min(
						previewIndexRef.current,
						validation.entry_count - 1,
					);
					setPreviewCurrentIndex(idx);
					const rendered = await previewRenderEntry(
						templatePath,
						templateFields,
						specs,
						idx,
					);
					if (controller.signal.aborted) return;
					setPreviewRendered(rendered);
				} else {
					setPreviewRendered(null);
				}
			} catch (err) {
				if (!controller.signal.aborted) {
					setPreviewError(String(err));
				}
			} finally {
				if (!controller.signal.aborted) {
					setPreviewLoading(false);
				}
			}
		}, 300);

		return () => {
			clearTimeout(timer);
			controller.abort();
		};
	}, [templatePath, templateFields, allSourcesLoaded, buildSourceFileSpecs]);

	const handlePreviewNavigate = async (index: number) => {
		if (!templatePath || !templateFields || !previewValidation) return;
		if (index < 0 || index >= previewValidation.entry_count) return;
		setPreviewCurrentIndex(index);
		try {
			const specs = buildSourceFileSpecs();
			const rendered = await previewRenderEntry(
				templatePath,
				templateFields,
				specs,
				index,
			);
			setPreviewRendered(rendered);
		} catch (err) {
			setPreviewError(String(err));
			setPreviewRendered(null);
		}
	};

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

			// Initialize form sources immediately with inferred fields.
			const initialSources: Record<string, SourceState> = {};
			const initialFields: Record<string, string[]> = {};
			for (const slot of info.sources) {
				if (slot.is_form) {
					const fields = await getFormFields(
						selected,
						info.fields,
						slot.namespace,
					);
					const values: Record<string, string> = {};
					for (const f of fields) values[f] = "";
					initialSources[slot.namespace] = {
						path: "",
						csvPreview: null,
						separatorOverride: null,
						encodingOverride: null,
						error: null,
						formFields: fields,
						formValues: values,
					};
					initialFields[slot.namespace] = fields;
				}
			}
			setSourcesState(initialSources);
			setNamespaceFields(initialFields);

			setSaveStatus("idle");
			setSaveError(null);
			setPreviewValidation(null);
			setPreviewRendered(null);
			setPreviewCurrentIndex(0);
			setPreviewError(null);
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
						formFields: null,
						formValues: null,
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
						formFields: null,
						formValues: null,
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
					formFields: null,
					formValues: null,
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

	const handleFormValueChange = (
		namespace: string,
		field: string,
		value: string,
	) => {
		setSourcesState((prev) => {
			const state = prev[namespace];
			if (!state?.formValues) return prev;
			return {
				...prev,
				[namespace]: {
					...state,
					formValues: { ...state.formValues, [field]: value },
				},
			};
		});
	};

	// Re-infer form fields when template fields change.
	useEffect(() => {
		if (!templatePath || !templateFields || !templateInfo) return;
		const formSources = templateInfo.sources.filter((s) => s.is_form);
		if (formSources.length === 0) return;

		const controller = new AbortController();
		const timer = setTimeout(async () => {
			for (const slot of formSources) {
				try {
					const fields = await getFormFields(
						templatePath,
						templateFields,
						slot.namespace,
					);
					if (controller.signal.aborted) return;
					setSourcesState((prev) => {
						const state = prev[slot.namespace];
						if (!state) return prev;
						const oldValues = state.formValues ?? {};
						const newValues: Record<string, string> = {};
						for (const f of fields) newValues[f] = oldValues[f] ?? "";
						return {
							...prev,
							[slot.namespace]: {
								...state,
								formFields: fields,
								formValues: newValues,
							},
						};
					});
					setNamespaceFields((prev) => ({
						...prev,
						[slot.namespace]: fields,
					}));
				} catch {
					// Non-critical: field inference failure doesn't block anything
				}
			}
		}, 500);

		return () => {
			clearTimeout(timer);
			controller.abort();
		};
	}, [templatePath, templateFields, templateInfo]);

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
							onFormValueChange={handleFormValueChange}
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

					{previewVisible && (
						<>
							<ResizableHandle withHandle />

							{/* Preview */}
							<ResizablePanel defaultSize={30} minSize={10}>
								<Preview
									validation={previewValidation}
									currentIndex={previewCurrentIndex}
									rendered={previewRendered}
									loading={previewLoading}
									error={previewError}
									onNavigate={handlePreviewNavigate}
								/>
							</ResizablePanel>
						</>
					)}
				</ResizablePanelGroup>

				{/* Status bar */}
				<StatusBar
					profiles={smtpProfiles}
					selectedProfileName={selectedProfileName}
					onProfileChange={setSelectedProfileName}
					onSmtpSettings={() => setSmtpDialogOpen(true)}
					allSourcesLoaded={allSourcesLoaded}
					previewVisible={previewVisible}
					onTogglePreview={() => setPreviewVisible((v) => !v)}
					onSend={() => setSendDialogOpen(true)}
				/>

				<SmtpSettingsDialog
					open={smtpDialogOpen}
					onOpenChange={setSmtpDialogOpen}
					profiles={smtpProfiles}
					onSave={handleSaveProfiles}
				/>

				{templatePath && templateFields && selectedProfileName && (
					<SendDialog
						open={sendDialogOpen}
						onOpenChange={setSendDialogOpen}
						templatePath={templatePath}
						templateFields={templateFields}
						sourceFileSpecs={buildSourceFileSpecs()}
						profileName={selectedProfileName}
						entryCount={previewValidation?.entry_count ?? 0}
						validationEntries={previewValidation?.entries ?? []}
					/>
				)}
			</div>
		</TooltipProvider>
	);
}
