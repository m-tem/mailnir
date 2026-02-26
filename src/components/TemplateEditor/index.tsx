import { CheckIcon, LoaderCircleIcon, SaveIcon } from "lucide-react";
import { useEffect } from "react";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import type { TemplateFields } from "@/lib/ipc";
import BodyEditor from "./BodyEditor";
import BodyFormatSelect from "./BodyFormatSelect";
import FieldEditor from "./FieldEditor";

interface Props {
	templatePath: string | null;
	templateFields: TemplateFields | null;
	loadId: number;
	namespaces: string[];
	namespaceFields: Record<string, string[]>;
	saveStatus: "idle" | "saving" | "saved" | "error";
	saveError: string | null;
	onFieldChange: (field: keyof TemplateFields, value: string | null) => void;
	onSave: () => void;
}

export default function TemplateEditor({
	templatePath,
	templateFields,
	loadId,
	namespaces,
	namespaceFields,
	saveStatus,
	saveError,
	onFieldChange,
	onSave,
}: Props) {
	// Ctrl+S / Cmd+S to save
	useEffect(() => {
		const handler = (e: KeyboardEvent) => {
			if ((e.ctrlKey || e.metaKey) && e.key === "s") {
				e.preventDefault();
				onSave();
			}
		};
		window.addEventListener("keydown", handler);
		return () => window.removeEventListener("keydown", handler);
	}, [onSave]);

	if (!templatePath || !templateFields) {
		return (
			<div className="flex h-full flex-col items-center justify-center gap-2 p-8 text-center">
				<p className="text-sm font-medium text-muted-foreground">
					Template Editor
				</p>
				<p className="text-xs text-muted-foreground">
					Open a template to edit it
				</p>
			</div>
		);
	}

	// Handlers for optional fields — convert empty string to null
	const set = (field: keyof TemplateFields) => (value: string) =>
		onFieldChange(field, value === "" ? null : value);

	const setRequired = (field: keyof TemplateFields) => (value: string) =>
		onFieldChange(field, value);

	return (
		<div className="flex h-full flex-col overflow-hidden">
			{/* Header */}
			<div className="flex shrink-0 items-center justify-between border-b px-4 py-2">
				<p className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
					Template Editor
				</p>
				<div className="flex items-center gap-2">
					{saveStatus === "saved" && (
						<span className="flex items-center gap-1 text-xs text-green-600">
							<CheckIcon className="h-3 w-3" />
							Saved
						</span>
					)}
					{saveStatus === "error" && saveError && (
						<span className="max-w-48 truncate text-xs text-destructive">
							{saveError}
						</span>
					)}
					<Button
						size="sm"
						variant="outline"
						onClick={onSave}
						disabled={saveStatus === "saving"}
						className="h-7 gap-1 text-xs"
					>
						{saveStatus === "saving" ? (
							<LoaderCircleIcon className="h-3 w-3 animate-spin" />
						) : (
							<SaveIcon className="h-3 w-3" />
						)}
						Save
					</Button>
				</div>
			</div>

			{/* Fields */}
			<ScrollArea className="flex-1 overflow-hidden">
				<div className="space-y-4 p-4">
					<FieldEditor
						label="To"
						defaultValue={templateFields.to}
						loadId={loadId}
						placeholder="{{namespace.email}}"
						namespaces={namespaces}
						namespaceFields={namespaceFields}
						onChange={setRequired("to")}
					/>

					<FieldEditor
						label="CC"
						defaultValue={templateFields.cc ?? ""}
						loadId={loadId}
						placeholder="{{namespace.cc_email}}"
						namespaces={namespaces}
						namespaceFields={namespaceFields}
						onChange={set("cc")}
						optional
					/>

					<FieldEditor
						label="BCC"
						defaultValue={templateFields.bcc ?? ""}
						loadId={loadId}
						placeholder="admin@example.com"
						namespaces={namespaces}
						namespaceFields={namespaceFields}
						onChange={set("bcc")}
						optional
					/>

					<FieldEditor
						label="Subject"
						defaultValue={templateFields.subject}
						loadId={loadId}
						placeholder="{{namespace.name}} — your credentials"
						namespaces={namespaces}
						namespaceFields={namespaceFields}
						onChange={setRequired("subject")}
					/>

					<Separator />

					<BodyFormatSelect
						value={templateFields.body_format}
						onChange={(v) => onFieldChange("body_format", v)}
					/>

					<BodyEditor
						defaultValue={templateFields.body}
						loadId={loadId}
						namespaces={namespaces}
						namespaceFields={namespaceFields}
						onChange={setRequired("body")}
					/>

					<Separator />

					<FieldEditor
						label="Attachments"
						defaultValue={templateFields.attachments ?? ""}
						loadId={loadId}
						placeholder="{{namespace.name}}/handout.pdf"
						namespaces={namespaces}
						namespaceFields={namespaceFields}
						onChange={set("attachments")}
						optional
					/>

					<FieldEditor
						label="Stylesheet"
						defaultValue={templateFields.stylesheet ?? ""}
						loadId={loadId}
						placeholder="style.css"
						namespaces={namespaces}
						namespaceFields={namespaceFields}
						onChange={set("stylesheet")}
						optional
					/>

					<FieldEditor
						label="Style (inline CSS)"
						defaultValue={templateFields.style ?? ""}
						loadId={loadId}
						placeholder="body { font-family: sans-serif; }"
						namespaces={namespaces}
						namespaceFields={namespaceFields}
						onChange={set("style")}
						optional
					/>
				</div>
			</ScrollArea>
		</div>
	);
}
